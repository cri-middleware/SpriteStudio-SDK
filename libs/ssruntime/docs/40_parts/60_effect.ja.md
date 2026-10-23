> **日本語** | [English](./60_effect.md)

# エフェクトパーツ

エフェクトパーツは、パーティクルのエフェクトを再生するパーツです。Player はエフェクトパーツごとにスロットを持ち、毎フレーム `ss_effect_slot_step` を呼びます。パーティクルの計算は Brain が行い、そのフレームのすべてのパーティクルの頂点・UV・色を `EffectDrawPlan` で返します。Player はそれを描くだけです。

進め方の多く（すべてのパーツを毎フレーム進める、独立再生、再生し直すときの初期化）は、インスタンスパーツと同じです（[インスタンスパーツ](./50_instance.ja.md)）。

## 1. スロットを作って進める

アニメーションを読み込んだら、ssab のパーツからエフェクトパーツ（`PartTypeEffect`）をすべて探し、それぞれにスロットを作ります。

```cpp
// 用意: エフェクトパーツごとに
void* slot = ss_effect_slot_create(resource, effect_part->ref_effect_name_hash(), seed);
if (slot == nullptr) { /* 参照先のエフェクトがこの ssab に無い */ }
```

`seed` はパーティクルの乱数の種です。パーツごとに違う値（パーツのインデックスとエフェクト名のハッシュの XOR など）にしてください。スロットはリソースへの参照を持つので、スロットより先に `ss_resource_destroy` を呼んでもかまいません。

```cpp
// 毎フレーム、親の update の後に、すべてのエフェクトパーツについて
const ss_effect_event_info ev = ss_runtime_get_active_effect_event(ctx, part_index);

const ss_effect_step_result step = ss_effect_slot_step(
    slot, ev,
    parent_frame_no,       // 親の再生ヘッド
    delta_seconds,         // 経過時間（秒）
    fps,                   // アニメーションのフレームレート（AnimationData.fps）
    parent_looped,         // 親の ss_runtime_just_looped
    /*y_flip*/ true,       // §2
    /*vert_stride*/ 2,     // §2
    /*draw*/ part_drawn);  // §2: このフレームの draw_order にパーツがあるか

if (step.visible) {
    auto plan = ss::runtime::GetEffectDrawPlan(step.draw_plan_buf);
    // §3 のとおりに描く
}
```

- **経過時間は秒です**（`ss_runtime_update` はミリ秒）。
- **エフェクトの時計を `draw_order` で進めないでください。** 非表示のエフェクトパーツは `draw_order` に入らないので、`draw_order` を走査しながら `step` すると、非表示の間は独立再生のエフェクトが空中で止まります。**すべてのエフェクトパーツを ssab から列挙し、毎フレーム漏れなく `step` してください。** `step.visible` は、描くかどうかを決めるためだけに使います。非表示のパーツは `draw = false` で `step` します（§2）。時計は進み、誰も描かないパーティクルの計算は省かれます。
- 返された `EffectDrawPlan` のバッファは、同じスロットで次に `step` を呼ぶまで有効です。
- 親を `play` で先頭から再生し直すときと、エフェクトパーツを持つインスタンスの子をスロットが始めたときは、`ss_effect_slot_reset` でスロットを初期状態に戻してください（[インスタンスパーツ §6](./50_instance.ja.md#6-再生し直すとき)）。
- 独立再生の扱いはインスタンスパーツと同じです（[インスタンスパーツ §3](./50_instance.ja.md#3-独立再生)）。

## 2. `ss_effect_slot_step` の引数

| 引数 | 値 |
| :--- | :--- |
| `y_flip` | Y-Down のエンジン（Godot の 2D、PixiJS など）は `true`、Y-Up のエンジン（Unity など）は `false`。SpriteStudio は Y-Up で作られているので、`true` ではパーティクルの Y 座標と回転の向きを反転して返す |
| `vert_stride` | 1 頂点あたりの float の数。2D のエンジンは `2`（XY）、3 成分の頂点を求めるエンジン（Unity の Mesh、Unreal の FProcMeshVertex など）は `3`（XYZ、z = 0） |
| `draw` | このフレームの `draw_order` にパーツがあるか、つまり描くかどうか。`false` ではエフェクトの時計と遷移だけを進め、パーティクルは作らず、見えないとして返す。次に `true` で `step` したときは、毎回描いていた場合と同じ plan が返る。判定にはこのフレームの `ss_runtime_get_frame_data` が先に要るので、それより前に `step` する Player は `true` を渡す |

## 3. `EffectDrawPlan` を描く

`EffectDrawPlan` は、エミッターごとの描画コマンドと、それらが指す連結された頂点のバッファを持ちます。

| フィールド | 内容 |
|---|---|
| `commands[]` | エミッターごとの描画コマンド |
| `vert_stride` | 呼び出しの `vert_stride` と同じ値（2 = XY、3 = XYZ） |
| `verts[]` | 頂点の座標（`総矩形数 * 4 * vert_stride` 個）。エフェクトパーツのローカル座標 |
| `uvs[]` | UV（`総矩形数 * 4 * 2` 個）。**正規化済み**（0〜1） |
| `colors[]` | 頂点の色（RGBA、0〜1、`総矩形数 * 4 * 4` 個） |
| `indices[]` | 三角形のインデックス（`総矩形数 * 6` 個）。**コマンドの中での 0 始まり** |

| `EffectDrawCommand` | 内容 |
|---|---|
| `cellmap_hash` | テクスチャのキー（セルマップ名のハッシュ） |
| `quad_count` | このコマンドのパーティクル（矩形）の数 |
| `particle_offset` | このコマンドの最初の矩形の位置（矩形単位） |
| `blend` | 描画先との合成（`0` = Mix、`1` = Add） |

コマンドごとに、各バッファを `particle_offset` から切り出します。

```cpp
const auto* plan = ss::runtime::GetEffectDrawPlan(step.draw_plan_buf);
const uint8_t stride = plan->vert_stride();

for (const auto* cmd : *plan->commands()) {
    const uint32_t q  = cmd->particle_offset();
    const uint32_t qc = cmd->quad_count();

    apply_material(cmd->cellmap_hash(), cmd->blend());
    draw_triangles(
        plan->verts()->data()   + q * 4 * stride, qc * 4 * stride,
        plan->uvs()->data()     + q * 4 * 2,      qc * 4 * 2,
        plan->colors()->data()  + q * 4 * 4,      qc * 4 * 4,
        plan->indices()->data() + q * 6,          qc * 6);
}
```

`apply_material` / `draw_triangles` は、エンジンの描画処理を表す実在しない関数です。

- インデックスはコマンドの中での 0 始まり（`0` から `quad_count * 4` の手前まで）なので、1 コマンドを 1 回のドローコールで描くなら、切り出したものをそのまま渡せます。
- 頂点はエフェクトパーツのローカル座標です。**エフェクトパーツのワールド行列を掛けてください。**
- **エフェクトパーツの不透明度（`PartState.alpha`）を、パーティクル全体に掛けてください。** `EffectDrawPlan` の色はパーティクルの色だけで、エフェクトパーツ自身の不透明度（パーツのフェードアウトなど）は入っていません。
- `draw_order` の中でエフェクトパーツの位置に来た DrawBatch（`kind == Effect`）で描きます。

## 4. コマンドをまとめて描く

エフェクトは、エミッターごとに別のコマンドになります。1 コマンドを 1 回のドローコールにすると、ドローコールとメモリ確保が増えて遅くなることがあります。

**続くコマンドどうしで `cellmap_hash` と `blend` が同じなら、1 つの頂点バッファにまとめて 1 回で描くことを推奨します。** まとめるときは、インデックスがコマンドの中での値なので、**それまでにまとめた頂点の数を各インデックスに足して**書き込んでください。

エフェクトパーツは、1 つずつ別の DrawBatch になります。**`Effect` の DrawBatch が続くときは、パーツをまたいでも、続くコマンドどうしで `cellmap_hash` と `blend` が同じならまとめられます**（まとめるときの決まりは [描画ループ §4-3](../30_core/40_render_loop.ja.md#4-3-続く-drawbatch-をまとめて描く)）。パーツごとに違うワールド行列と不透明度（§3）は、頂点に掛けてからつなげてください。不透明度が同じパーツの間だけつなげて、不透明度はドローコールの単位で掛ける方法もあります。

> [!NOTE]
> **実測（Web の Player、WebGL）:** SDK のテストの overall の Effect では、1 フレームのドローコールが平均 8.8 回から 1 回になりました。

---
[目次に戻る](../README.ja.md)
