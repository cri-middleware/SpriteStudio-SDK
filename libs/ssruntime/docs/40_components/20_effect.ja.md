# エフェクトパーツ (Effect)

パーティクルエフェクトのシミュレーション結果を、絶対時間（フレーム）に基づいて瞬時に取得できます。`libssruntime` が提供する **Slot API** を活用します。

## Effect Slot の活用
`ss_effect_slot_step` を呼び出すと、そのフレームの全パーティクルの描画データ（頂点、UV、色、テクスチャ）が **`EffectDrawPlan`** として返されます。

```cpp
// 毎フレームの更新ループ内
const ss_effect_event_info ev = ss_runtime_get_active_effect_event(runtime_ctx, p_idx);

// シミュレーション実行。
const ss_effect_step_result step = ss_effect_slot_step(
    slot.effect_slot, ev, f.frame_no, f.delta_seconds, fps, f.parent_looped,
    /*y_flip*/ true, /*vert_stride*/ 2);

if (step.visible) {
    auto plan = ss::runtime::GetEffectDrawPlan(step.draw_plan_buf);
    // → 後述の「EffectDrawPlan の構造」に沿って描画
}
```

## 参考:`ss_effect_slot_step` の引数：`y_flip` / `vert_stride`

- **`y_flip`**：Y-down のエンジン（Godot 2D、pixi.js など）は `true` を渡します。SS は Y-up 座標系で作成された頂点を、エミット時にここで反転して返します。Y-up エンジン（Unity 等）は `false` を渡します。
- **`vert_stride`**：頂点 1 つあたりのフロート数。2D-native は `2` (XY)、3 要素頂点を要求するエンジン（Unity Mesh、Unreal FProcMeshVertex 等）は `3` (XYZ, z=0)。

## 参考:`EffectDrawPlan` の構造

`ss_effect_slot_step` が返す `EffectDrawPlan` には、エミッターごとの描画コマンドと、それらが指すフラット頂点バッファ群が入っています。

| フィールド | 内容 |
|---|---|
| `commands[]` | エミッターごとの描画コマンド（後述） |
| `vert_stride` | 呼び出し時の `vert_stride` 引数と一致（2=XY、3=XYZ） |
| `verts[]`    | 頂点座標フラット配列（`total_quads * 4 * vert_stride` 要素） |
| `uvs[]`      | UV 座標フラット配列（`total_quads * 4 * 2` 要素） |
| `colors[]`   | 頂点色フラット配列（RGBA 0..1 正規化、`total_quads * 4 * 4` 要素） |
| `indices[]`  | 三角形インデックス（**quad-local 0-based**、`total_quads * 6` 要素） |

各 `EffectDrawCommand` は上記のフラットバッファをスライスする情報を持ちます：

| フィールド | 内容 |
|---|---|
| `cellmap_hash`     | プレイヤー 側のセルマップテクスチャ辞書のキー |
| `quad_count`       | このコマンドが扱う quad の数 |
| `particle_offset`  | 先頭 quad の index（quad 単位） |
| `blend`            | 生 `EmitterState.blend_type`（`0=Mix`, `1=Add`） |

スライス境界は `particle_offset` を quad 数として起点に、各バッファのストライドで掛け算します：

```cpp
const auto* plan = ss::runtime::GetEffectDrawPlan(step.draw_plan_buf);
const uint8_t stride = plan->vert_stride();

for (const auto* cmd : *plan->commands()) {
    const uint32_t v_off = cmd->particle_offset() * 4 * stride;
    const uint32_t u_off = cmd->particle_offset() * 4 * 2;
    const uint32_t c_off = cmd->particle_offset() * 4 * 4;
    const uint32_t i_off = cmd->particle_offset() * 6;

    ApplyMaterial(cmd->cellmap_hash(), cmd->blend());
    DrawTriangles(
        plan->verts()->data()   + v_off, cmd->quad_count() * 4 * stride,
        plan->uvs()->data()     + u_off, cmd->quad_count() * 4 * 2,
        plan->colors()->data()  + c_off, cmd->quad_count() * 4 * 4,
        plan->indices()->data() + i_off, cmd->quad_count() * 6
    );
}
```

`indices[]` は **quad-local 0-based**（各 quad 内で 0..3 を指す）なので、単一コマンドを描画する場合はスライスをそのまま渡せばよく、ベース加算は不要です。

## 💡 最適化：Effect コマンドの動的バッチング

エフェクトはパーティクルエミッターごとに `EffectDrawCommand` が分かれて出力されます。これを愚直に 1 コマンド = 1 ドローコールとして実装すると、ドローコールとアロケーションが膨れ上がりパフォーマンス低下を招く場合があります。

**連続するコマンド同士で `cellmap_hash` (テクスチャ) と `blend` (ブレンドモード) が同一である場合、それらをマージして単一の頂点バッファとして 1 つのドローコールで送信する（動的バッチング）** ことが強く推奨されます。

結合する際の注意点として、各コマンドの `indices[]` は各 Quad 内の `0..3` を指す相対インデックスであるため、結合先の巨大なインデックスバッファにコピーする際は、**「それまでにマージした頂点の総数 (`dest_quad_count * 4`)」を各インデックス値に加算して書き込む** 必要があります。

## 独立再生 (Independent Playback)

エフェクトは親の停止中も独自時間で進行する「独立再生」に対応します。仕組みは [インスタンスパーツ §独立再生](./10_instance.ja.md#独立再生-independent-playback-の対応) を参照してください。

---
[目次に戻る](../README.ja.md)
