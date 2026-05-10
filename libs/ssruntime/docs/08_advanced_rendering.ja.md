# 実装フェーズ 5：拡張機能の実装 (Advanced Features)

アニメーションの表現をさらに高めるための拡張機能（インスタンス、エフェクト、メッシュ、パーツカラー等）の実装ガイドです。
これらの機能はイベント駆動ではなく、**毎フレームの描画ループ内において、親のフレーム時間に同期して状態を計算・適用**するアーキテクチャが基本となります。

## 1. インスタンス再生 (Instance)
SpriteStudio のインスタンス機能は、別のアニメーションを部品として参照し再生する機能です。
親の再生時間（フレーム）に完全に同期させるため、**毎フレーム親の現在のフレームから子のローカルフレームを算出し、子Playerの時間を強制的に合わせる**手法をとります。これにより、逆再生やフレームスキップ時でも常に正しい状態が描画されます。

> **【制限事項】 `independent=true` は逆再生 / シークでは動作しません** ([制限事項一覧](./00_limitations.ja.md#independenttrue-パーツの逆再生--シーク))
>
> `PartAttributeInstance.independent` を `true` にすると、子インスタンスは親の timeline から切り離されてリアルタイム (delta_time) で進行する仕様です。これは設計上、親フレームから子フレームを deterministic に逆算できないため、以下のような操作で破綻します:
>
> - 親の逆再生 (子のリアルタイム時計は単調増加なので、過去方向に進まない)
> - 親のシーク (過去の任意フレームへ戻ったときの「子の累積時間」が一意に決まらない)
>
> 一方、**親のフレームスキップ自体は対応可能**です。親が経過 `delta_time` を子に渡し、子が `update(delta)` で累積を進めれば、スキップ量に比例した分だけ子も進みます (順方向のみ)。
>
> Player 移植にあたっては、SS7 のアニメ制作時に `independent=true` を使わない運用を推奨します。やむを得ず使う場合は、対応する子インスタンスは「順方向再生時のみ正しく描画される」ことを利用者に明示してください。

> **【NO_KEY (Instance アトリビュートにキーがない) ときの挙動】**
>
> ある Instance パーツに `EventInstance` が一つも紐づかず `InitialEvents.instance` も `None` の場合、Player は SS6 SDK の `SsInstanceAttr` デフォルトコンストラクタ (`Common/Loader/sstypes.h:1294`) と同じ値で再生を組み立てる必要があります:
>
> - `loop_num = 1`（**`-1` 無限ループにしない**）
> - `infinity = false`
> - `pingpong = false`, `reverse = false`
> - `start_label = "_start"` / `end_label = "_end"` (= 子アニメの先頭〜末尾)
> - `start_offset = 0`, `end_offset = 0`
> - `speed = 1.0`
> - `curKeyframe = 0`（= 親フレームをそのまま `diff` に使う）
>
> SS6 の `updateInstance`（`Common/Animator/ssplayer_animedecode.cpp:1670-1677`) は `nowloop >= loop_num` になった時点で `reftime = inst_scale - 1` にクランプするため、結果として **1 サイクル再生 → end_frame で停止** という挙動になります。`loops = -1` をフォールバックにすると SS では止まる位置で Godot/Unity 側だけループし続けてしまうので注意してください。
>
> なお SS6 では `SsGetKeyValue` が `nowTime` より未来のキーを返してきた場合（= 今より前にキーがない）、`state->instanceValue` をデフォルトコンストラクタに**戻して**から `updateInstance` を続行します（`ssplayer_animedecode.cpp:1162-1166`）。SS7 では converter が `time == 0` のキーを `InitialEvents.instance` に複製するため通常は再現される一方、ユーザーが意図的にキーを入れていない（NO_KEY）パーツでは `InitialEvents.instance = None` になるので、上記デフォルトを Player 側で用意してください。

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
Player 側で `ss_effect_slot_create` を用いて per-Effect-part の slot を作成しておき、毎フレーム `ss_effect_slot_step` に親の現在フレームと delta を渡します。slot は内部で event 解決 / transition 検出 / シミュレータ更新 / cellmap・cell・UV 解決 / particle quad 生成までを行い、結果は `EffectDrawPlan` FlatBuffer として返ります。Player 側はその plan に従って Texture lookup と canvas/material 設定をするだけで描画できます。

```cpp
// 初回（Effect part を見つけたとき）
slot.effect_slot = ss_effect_slot_create(resource, ref_effect_name_hash, seed);

// 毎フレームの描画ループ内
if (part->type() == PartTypeEffect) {
    // 1. 親 Context から active event 情報を取得
    ss_effect_event_info ev = ss_runtime_get_active_effect_event(parent_ctx, part_index);

    // 2. slot を 1 ステップ進める。返される EffectDrawPlan は
    //    "describes how to draw the slot this frame" で完結。
    //    y_flip = true は Y-down レンダリング（Godot/pixi.js）、
    //    vert_stride = 2 は (x,y) 連結（3 にすると (x,y,0) — Unity Mesh 等）。
    ss_effect_step_result r = ss_effect_slot_step(
        slot.effect_slot, ev,
        parent_frame, delta_seconds, fps,
        parent_looped,
        /*y_flip*/ true, /*vert_stride*/ 2);

    if (!r.visible) { /* hide and return */ }

    // 3. EffectDrawPlan を読んで描画コマンドを実行
    auto plan = ss::runtime::GetEffectDrawPlan(r.draw_plan_buf);
    const float*  V = plan->verts()->data();
    const float*  U = plan->uvs()->data();
    const float*  C = plan->colors()->data();
    const int32_t* I = plan->indices()->data();

    for (uint32_t e = 0; e < plan->commands()->size(); e++) {
        const auto* cmd = plan->commands()->Get(e);
        if (cmd->quad_count() == 0) continue;

        // cellmap_hash → engine の Texture へ。マップは SSAB ロード時に構築。
        Texture tex = textures.lookup(cmd->cellmap_hash());
        BlendMode blend = (cmd->blend() == 1) ? BlendMode::Add : BlendMode::Mix;

        // 連結バッファから自分の quad 範囲だけスライスして描画
        const uint32_t qc  = cmd->quad_count();
        const uint32_t off = cmd->particle_offset();
        DrawTriangles(
            tex, blend,
            V + off * 8,  qc * 4,    // verts (x,y per corner)
            U + off * 8,             // uvs
            C + off * 16,            // colors (rgba per corner)
            I + off * 6,  qc * 6);   // indices (quad-local 0..qc*4)
    }
}
```

slot 側でキャッシュされる項目:
- effect 全体の lifetime / is_infinite (作成時)
- cellmap_hash + cell_hash + blend ごとの geometry/UV (初回参照時)
- transition state (last_event_frame, accumulated_time 等)

Player 側に残るのは Texture 管理 と canvas/blend material のエンジン依存ロジックのみです。

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
`PartTypeMesh` パーツのサポートは現在 **未実装** です（[ロードマップ](./04_roadmap.ja.md) / [制限事項一覧](./00_limitations.ja.md#現時点で未実装の機能) 参照）。
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
