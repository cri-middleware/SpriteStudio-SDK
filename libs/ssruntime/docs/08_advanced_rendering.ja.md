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

## 3. 頂点配列 (AttributeVertex)
通常パーツ（スプライトパーツ）の描画に使う **算出済みローカル頂点座標** です。
Brain 側で `pivot` / `size` / `img_flip_h/v` および補間済みの raw deform オフセットがすべて適用された状態で配信されるため、Player はワールド行列を掛けるだけで描画頂点を得られます。

頂点は最大 **5 点構成** (`LT` / `RT` / `LB` / `RB` / `center`) で格納されます。ただし `center` は **`update_flag` に `AttributeVertex` か `AttributePartColor` が立っているときのみ** 有効値が入ります (頂点変形・パーツカラーのグラデーションを center 起点の三角形ファンで滑らかに描画するため)。それ以外の場合は `center = (0, 0)` で、Player は 4 隅から 2 三角形を組むだけで描画して構いません。

`part_state->vertex()` のインデックスを使って、`FrameData` 直下の `vertices` テーブル（`PartAttributeVertex`）から取得します。

```cpp
// 通常パーツの頂点取得例
int vertex_idx = part_state->vertex();
if (vertex_idx >= 0) {
    auto v = frame_data->vertices()->Get(vertex_idx);

    // 4 隅は常に算出済み (pivot / size / img_flip / deform 適用済み)
    float lx[5] = { v->lt()->x(), v->rt()->x(), v->lb()->x(), v->rb()->x(), 0.0f };
    float ly[5] = { v->lt()->y(), v->rt()->y(), v->lb()->y(), v->rb()->y(), 0.0f };

    uint64_t flags = part_state->update_flag();
    bool needs_center =
        (flags & UpdateAttributeFlags_AttributeVertex) ||
        (flags & UpdateAttributeFlags_AttributePartColor);

    if (needs_center) {
        // 5 頂点 + 4 三角形のファン (center を起点に LT-RT-RB-LB を回す)
        lx[4] = v->center()->x();
        ly[4] = v->center()->y();
        // indices: { 0,1,4, 1,3,4, 3,2,4, 2,0,4 }
    } else {
        // 4 頂点 + 2 三角形 (center 不使用)
        // indices: { 0,1,2, 1,3,2 }
    }

    // それぞれを world_matrix で変換するだけで最終描画頂点になる
}
```

### update_flag との関係
`update_flag & AttributeVertex` は **ソース SSAB に頂点変形アニメ（raw deform）が含まれていたか** を示すヒントとして引き続き機能します。フラグが立っていなくても 4 隅の `vertex()` 値は有効で、Player は「center を使う / 使わない」の判定にこのフラグと `AttributePartColor` を使います。

## 3.5 メッシュとデフォーム (Mesh / Deform) — 未実装
`PartTypeMesh` パーツのサポートは現在 **未実装** です（[ロードマップ](./04_roadmap.ja.md)参照）。
`PartAttributeDeform`（`FrameData::deforms`）はメッシュパーツの可変数頂点向けに予約された構造で、現時点ではプレイヤー実装の対象外です。実装は今後のフェーズで行います。

## 4. パーツカラー (PartsColor)
頂点ごとにカラー合成を行うアトリビュートです。`PartAttributePartColor` は 4 隅 (`lt` / `rt` / `lb` / `rb`) ぶんの色情報を保持します。
`part_state->part_color()` のインデックスを使って、`FrameData` 直下の `parts_color` テーブルから取得します。

- **合成タイプ (`blend_type`)**: Mix(通常), Mul(乗算), Add(加算), Sub(減算) ほか。シェーダーで色を合成する際の計算式を決定します。
- **頂点カラー (`lt` / `rt` / `lb` / `rb`)**: それぞれ `SsAttributePartColorKeyValueColor`（`rgba: U8Rect{a, r, g, b}` と補間レート `rate`）。`a` チャンネルには **パーツの階層的 `alpha` が事前合成済み** なので、Player 側で `state.alpha()` を再度乗算する必要はありません。
- 単色指定（4 頂点で同じ色）の場合も同様に 4 隅すべてが同一値で配信されます。

```cpp
// パーツカラーの取得例
int color_idx = part_state->part_color();
if (color_idx >= 0) {
    auto pc = frame_data->parts_color()->Get(color_idx);
    auto blend_type = pc->blend_type();

    auto to_color = [](const ss::runtime::SsAttributePartColorKeyValueColor* c) {
        const auto& rgba = c->rgba();
        // a は階層 alpha 事前合成済み。再乗算しない。
        return Color(rgba.r() / 255.0f, rgba.g() / 255.0f, rgba.b() / 255.0f, rgba.a() / 255.0f);
    };

    Color clt = to_color(pc->lt());  // 左上
    Color crt = to_color(pc->rt());  // 右上
    Color clb = to_color(pc->lb());  // 左下
    Color crb = to_color(pc->rb());  // 右下

    // 5 頂点描画する場合、center 頂点には 4 隅の平均色を割り当てるのが自然
    Color ccenter = (clt + crt + clb + crb) * 0.25f;
}
```

---
[目次に戻る](./README.ja.md)
