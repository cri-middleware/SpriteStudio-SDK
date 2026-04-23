# 実装フェーズ 5：拡張機能の実装 (Advanced Features)

アニメーションの表現をさらに高めるための拡張機能（インスタンス、エフェクト、メッシュ、パーツカラー等）の実装ガイドです。
これらの機能はイベント駆動ではなく、**毎フレームの描画ループ内において、親のフレーム時間に同期して状態を計算・適用**するアーキテクチャが基本となります。

## 1. インスタンス再生 (Instance)
SpriteStudio のインスタンス機能は、別のアニメーションを部品として参照し再生する機能です。
親の再生時間（フレーム）に完全に同期させるため、**毎フレーム親の現在のフレームから子のローカルフレームを算出し、子Playerの時間を強制的に合わせる**手法をとります。これにより、逆再生やフレームスキップ時でも常に正しい状態が描画されます。

### 同期計算と描画ループ
子アニメーションの複雑な同期計算（オフセット、速度、ループの剰余計算など）を行うため、SDKには `ss_util_calculate_instance_frame` というユーティリティ関数が用意されています。

```cpp
// 毎フレームの描画ループ内
if (part->type() == PartTypeInstance) {
    // 1. Attribute からインスタンス設定を取得
    // (通常は ssab の instance_params や UserData などから取得しますが、
    //  実装上は PartState または AnimationData に格納された設定を参照します)
    float instance_start_frame = ...; // 再生開始フレーム
    float instance_offset = ...;      // オフセット
    float instance_speed = ...;       // 再生速度
    bool independent_flag = ...;      // 独立動作フラグ

    // 2. SDKのユーティリティを使って、現在の子の正確な再生フレームを算出
    float local_frame = ss_util_calculate_instance_frame(
        current_parent_frame, // 親の現在のフレーム (float)
        instance_start_frame,
        instance_offset,
        instance_speed,
        independent_flag,
        accumulated_time      // 独立動作時の累積時間（非独立なら0.0f）
    );
    
    // 3. 子Playerの時間を強制的に合わせ、描画を実行
    // ※子Playerは事前に外部ssabからロード・生成しておく必要があります
    child_player->set_frame_no(local_frame);
    child_player->render(parent_world_matrix); // 親の行列をベースに描画
}
```

## 2. エフェクト (Effect)
パーティクルエフェクトのシミュレーションもインスタンスと同様に、イベント駆動ではなく**絶対時間（フレーム）を指定して状態を一意に決定する**アプローチをとります。
SpriteStudio 6/7 のエフェクトシミュレータはステートレス・決定論的なアルゴリズムであるため、指定したフレームの状態を瞬時に計算でき、逆再生にも対応可能です。

### エフェクトの更新と描画
Player 側で `ss_effect_create` を用いて Context を作成しておき、毎フレーム `ss_effect_update` に相対フレームを渡してパーティクル群を取得します。

```cpp
// 毎フレームの描画ループ内
if (part->type() == PartTypeEffect) {
    // 1. エフェクトの発生開始フレームからの相対フレームを計算
    float relative_frame = current_parent_frame - effect_start_frame;

    if (relative_frame >= 0.0f) {
        // 2. 絶対的なフレーム時間を指定してシミュレータを更新
        // ※delta_time ではなくフレーム値を渡すことで、逆再生やシーク時も一意の結果が得られます
        ss_effect_update(effect_ctx, relative_frame);

        // 3. パーティクルの状態（FlatBuffersチャンク）を取得
        const uint8_t* state_buf = nullptr;
        size_t state_len = 0;
        ss_effect_get_state(effect_ctx, &state_buf, &state_len);

        if (state_buf) {
            auto effect_state = ss::runtime::GetEffectState(state_buf);
            
            // 4. パーティクルの描画（エミッターの描画コマンドごとにループ）
            if (effect_state->emitters()) {
                for (auto emitter : *effect_state->emitters()) {
                    int cell_index = emitter->cell_index();
                    int blend_type = emitter->blend_type(); // 加算・通常などの合成モード
                    
                    if (emitter->particles()) {
                        for (auto particle : *emitter->particles()) {
                            // パーティクル個別の座標・スケール・回転・カラーを取得して描画
                            DrawParticle(
                                parent_world_matrix, 
                                particle->x(), particle->y(), 
                                particle->scale_x(), particle->scale_y(), 
                                particle->rotation(), 
                                particle->color_a(), particle->color_r(), particle->color_g(), particle->color_b(),
                                cell_index, blend_type
                            );
                        }
                    }
                }
            }
        }
    }
}
```

## 3. メッシュとデフォーム (Mesh / Deform)
`PartTypeMesh` パーツと、その頂点を変形させる `AttributeDeform` の実装です。複雑なデータは FlatBuffers のテーブル化（プーリング）により最適化されているため、インデックスを介して取得します。

- **頂点配列 (AttributeVertex)**: メッシュの基準となるローカルの頂点座標とUV座標です。`part_state->vertex()` のインデックスを使って、`FrameData` 直下の `vertices` テーブルから取得します。
- **デフォーム (AttributeDeform)**: アニメーションによって加算される変形オフセット値です。`part_state->deform()` のインデックスを使って、`FrameData` 直下の `deforms` テーブルから取得します。

```cpp
// メッシュパーツの頂点計算例
if (part->type() == PartTypeMesh) {
    int vertex_idx = part_state->vertex();
    int deform_idx = part_state->deform();

    if (vertex_idx >= 0) {
        auto vertex_data = frame_data->vertices()->Get(vertex_idx);
        auto base_positions = vertex_data->positions(); // 基準頂点 (X, Y) の配列
        auto uvs = vertex_data->uvs();                  // UV座標の配列

        // デフォームデータが存在すれば取得
        const flatbuffers::Vector<float>* deform_offsets = nullptr;
        if (deform_idx >= 0) {
            auto deform_data = frame_data->deforms()->Get(deform_idx);
            deform_offsets = deform_data->offsets();    // 変形オフセット (X, Y) の配列
        }

        // 各頂点の最終ローカル座標を計算し、ワールド行列を掛けて描画
        for (size_t i = 0; i < base_positions->size() / 2; i++) {
            float lx = base_positions->Get(i * 2);
            float ly = base_positions->Get(i * 2 + 1);

            if (deform_offsets && i * 2 + 1 < deform_offsets->size()) {
                lx += deform_offsets->Get(i * 2);
                ly += deform_offsets->Get(i * 2 + 1);
            }

            // lx, ly を parent_world_matrix で変換して最終頂点を生成
        }
    }
}
```

## 4. パーツカラー (PartsColor)
パーツ全体や頂点ごとにカラー合成を行うアトリビュートです。単色または4頂点のカラー情報（RGBA）が格納されています。
これも `part_state->part_color()` のインデックスを用いて、`FrameData` 直下の `parts_color` テーブルから取得します。

- **合成タイプ (BlendType)**: Mix(通常), Add(加算), Sub(減算), Mul(乗算)。シェーダーで色を合成する際の計算式を決定します。
- **カラー配列**: 1つの色（全体単色）または4つの色（各頂点）の RGBA (0〜255) 配列が格納されています。

```cpp
// パーツカラーの取得例
int color_idx = part_state->part_color();
if (color_idx >= 0) {
    auto color_data = frame_data->parts_color()->Get(color_idx);
    int blend_type = color_data->blend_type();
    auto colors = color_data->colors(); // [R, G, B, A, R, G, B, A, ...] 

    if (colors->size() == 4) {
        // 単色指定の場合: 全頂点に同じ色を適用
        Color c = Color(colors->Get(0), colors->Get(1), colors->Get(2), colors->Get(3));
    } else if (colors->size() == 16) {
        // 4頂点カラー指定の場合: 頂点ごとに色を適用（グラデーション）
        Color tl = Color(colors->Get(0),  colors->Get(1),  colors->Get(2),  colors->Get(3));   // 左上
        Color tr = Color(colors->Get(4),  colors->Get(5),  colors->Get(6),  colors->Get(7));   // 右上
        Color bl = Color(colors->Get(8),  colors->Get(9),  colors->Get(10), colors->Get(11)); // 左下
        Color br = Color(colors->Get(12), colors->Get(13), colors->Get(14), colors->Get(15));// 右下
    }
}
```

---
[目次に戻る](./README.ja.md)
