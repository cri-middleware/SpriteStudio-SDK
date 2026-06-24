# 実装フェーズ 4：コンポーネント実装 (Instance / Effect / Mesh)

SpriteStudio の高度な表現を支える「入れ子構造のアニメーション」「パーティクルエフェクト」「メッシュ変形」の実装ガイドです。これらは `libssruntime` が提供する **Slot API** と **SoA バッファ** を活用することで、複雑な同期ロジックを意識せずに実装できます。

---

## 1. インスタンスパーツ (Instance)

別のアニメーションを部品として参照し、親の再生時間に同期して再生します。

### Slot API による管理
プレイヤー はパーツごとに `ss_instance_slot_create()` で作成した slot を保持しておき、毎フレーム `ss_instance_slot_step()` を呼び出します。

```cpp
// 毎フレームの更新ループ内
const ss_event_instance_info info = ss_runtime_get_active_event_instance(runtime_ctx, p_idx);

// 状態更新と同期
const ss_instance_step_result r = ss_instance_slot_step(
    state.instance_slot, info, child->runtime_ctx,
    child->previous_frame_no, parent_frame_no, delta_seconds, parent_looped);

if (r.transitioned) {
    // インスタンスの切り替わり（イベントの発生）を検知。
    // 子 プレイヤー の play() を呼び出し、開始イベントなどを発火させる。
    child->play();
}

if (r.visible) {
    child->render(r.child_frame_no);
}
```

#### 💡 サブフレーム再生設定の同期・伝搬
親プレイヤーがサブフレーム補間（実数フレームによる滑らかなアニメーション再生）を有効にしている場合、その設定（有効/無効）を子プレイヤー（インスタンスパーツ）にも正しく伝搬させる必要があります。
これを怠ると、親のトランスフォームやパーツの動きは滑らかであるにもかかわらず、子プレイヤーのアニメーションだけが整数フレームでカクカクと更新され、同期が崩れる不自然な挙動が発生します。

- **同期のタイミング**:
  1. 親プレイヤーのサブフレーム有効/無効設定が変更された時、すべての子プレイヤーに対して再帰的に同等の設定（`set_sub_frame_enabled`等）を同期します。
  2. 新しくインスタンス（子プレイヤー）がセットアップされる際にも、親の現在のサブフレーム設定を初期パラメータとして引き継ぎます。

#### 外部 SSAB の解決 (External Instances)
インスタンスパーツが「別ファイルの `.ssab`（別のアニメーションパック）」を参照している場合があります。プレイヤーは初期化時に、SSAB の `external_instances()` セクションを確認し、必要な外部ファイルを再帰的にロードして、適切な Context にバインドする責任があります。

---

## 2. エフェクトパーツ (Effect)

パーティクルエフェクトのシミュレーション結果を、絶対時間（フレーム）に基づいて瞬時に取得できます。

### Effect Slot の活用
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

### 参考:`ss_effect_slot_step` の引数：`y_flip` / `vert_stride`

- **`y_flip`**：Y-down のエンジン（Godot 2D、pixi.js など）は `true` を渡します。SS は Y-up 座標系で作成された頂点を、エミット時にここで反転して返します。Y-up エンジン（Unity 等）は `false` を渡します。
- **`vert_stride`**：頂点 1 つあたりのフロート数。2D-native は `2` (XY)、3 要素頂点を要求するエンジン（Unity Mesh、Unreal FProcMeshVertex 等）は `3` (XYZ, z=0)。

### 参考:`EffectDrawPlan` の構造

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

### 💡 最適化：Effect コマンドの動的バッチング

エフェクトはパーティクルエミッターごとに `EffectDrawCommand` が分かれて出力されます。これを愚直に 1 コマンド = 1 ドローコールとして実装すると、ドローコールとアロケーションが膨れ上がりパフォーマンス低下を招く場合があります。

**連続するコマンド同士で `cellmap_hash` (テクスチャ) と `blend` (ブレンドモード) が同一である場合、それらをマージして単一の頂点バッファとして 1 つのドローコールで送信する（動的バッチング）** ことが強く推奨されます。

結合する際の注意点として、各コマンドの `indices[]` は各 Quad 内の `0..3` を指す相対インデックスであるため、結合先の巨大なインデックスバッファにコピーする際は、**「それまでにマージした頂点の総数 (`dest_quad_count * 4`)」を各インデックス値に加算して書き込む** 必要があります。

---

## 3. 独立再生 (Independent Playback) の対応

エフェクトやインスタンスには、親アニメーションの再生速度や停止状態に関わらず、独自の経過時間でシミュレーションを継続する「独立再生」設定があります。

- **検知**: `ss_effect_slot_is_independent()` や `ss_instance_slot_is_independent()` が `true` を返す場合、そのパーツは独立再生モードです。
- **更新の継続**: 親アニメーションが一時停止中や、シークによってフレームが動いていない場合でも、これらのパーツは `delta_seconds` を進めて `step` を呼び出し続ける必要があります。これを怠ると、パーティクルが空中で静止するなどの不自然な挙動になります。
- **暗黙的な再生キーはループ境界で無視される**: SpriteStudio は、現在のフレームに明示的な再生キー（ `EventInstance` / `EventEffect` ）が存在しない場合、暗黙の再生キー（**synthetic default**）を返します（典型的にはアニメーション先頭の frame-0 に明示キーを置かなかったケース）。このキーは実質 frame-0 リセットに相当します。独立再生モードの時は、親アニメーションがループしても **この暗黙キーで再生状態のリセットをせず**、自分の経過時間で進み続けます。**独立再生を開始・リセットできるのは明示的に置かれた `EventInstance` / `EventEffect` のキーのみ** です（暗黙キーは対象外）。
- **同一キーの再横断は再生をリセットしない（最後の再生キーで判定）**: 独立再生では、スロットごとに「**最後に再生を始めたキー**」を1つだけ覚えており、親ループで明示キーに再び達したとき、それが記憶と**同じキーなら無視**（再生状態を維持）、**異なるキーなら再生してリセット**します。この結果、**独立 + 有限長 + 単一の明示キー**の構成は親ループ全体を通して「**一発限り**」になります（同じキーに何周来ても再生し直さない。寿命で消えた後も同様）。周期的に出したい場合は「独立動作を外す（親フレームに同期）」か「エフェクト側を無限ループリソースにする」で棲み分けます。常在系（炎・煙など無限ループ）は持続、一発系は一発限りに自然に収まります。
- これらの判定は ssruntime の中で自動的に行われるので、プレイヤー側で `ev.is_synthetic_default` 等を読んで分岐する必要はなく、通常通り `ss_*_slot_step` の戻り値（`transitioned` 等）で動けば良いことになります（その戻り値は実質「再生をリセットすべき別キーに切り替わったか」を表します）。
- **実数フレーム（小数）の正確な書き戻し**:
  `ss_instance_slot_step` や `ss_effect_slot_step` が返す `step.child_frame_no` は、小数点以下の進行度を含んだ正確な実数（float）です。これを子プレイヤー（インスタンスパーツ等）のフレーム更新のために書き戻す際、**絶対に `floor` 等で整数に丸めたりクランプしたりしないでください。**
  丸めた値を状態として書き戻してしまうと、次フレームの `delta` 加算が常に丸められた整数フレームを起点に行われることになります。その結果、「1ティックあたりの進行時間が1フレーム未満」の環境において微小な進捗が毎フレーム切り捨てられ続け、アニメーションが完全にフリーズするバグの原因になります。表示フレームの丸め処理は描画の直前でのみ行い、状態としてのフレーム進行値は常に「生の実数」を保ってください。

---

## 4. メッシュとデフォーム (Mesh / Deform)

ボーンによるスキンメッシュ変形や、自由な頂点変形をサポートします。

### メッシュ SoA バッファ
メッシュの頂点計算（スキニング含む）は Brain 側で完了しています。プレイヤー は **計算済みのワールド頂点座標** を SoA バッファから直接取得して描画するだけです。  
各 SoA バッファは context 単位で一度取得しておけば問題ありません。

```cpp
// SoA バッファのポインタとレングスをまとめて保持
const float    *v_x, *v_y, *uvs;
const uint32_t *v_offsets, *index_offsets;
const int32_t  *indices;
size_t v_x_len, v_y_len, v_offsets_len, uvs_len, indices_len, index_offsets_len;

// 1. 各 SoA バッファを取得する
ss_runtime_get_mesh_vertices_x(ctx, &v_x, &v_x_len);
ss_runtime_get_mesh_vertices_y(ctx, &v_y, &v_y_len);
ss_runtime_get_mesh_vertex_offsets(ctx, &v_offsets, &v_offsets_len);
ss_runtime_get_mesh_uvs(ctx, &uvs, &uvs_len);
ss_runtime_get_mesh_indices(ctx, &indices, &indices_len);
ss_runtime_get_mesh_index_offsets(ctx, &index_offsets, &index_offsets_len);

// 2. 頂点レンジを取得（非 Mesh パーツの空レンジもここで弾く）
if ((uintptr_t)(p_idx + 1) >= v_offsets_len) return; // p_idx 範囲外ガード
const uint32_t v0 = v_offsets[p_idx];
const uint32_t v1 = v_offsets[p_idx + 1];
if (v1 <= v0) return;                                 // 非 Mesh パーツ＝空レンジ
if ((uintptr_t)v1 > v_x_len) return;                  // データ整合性ガード
const uint32_t vert_count = v1 - v0;

// 3. インデックスレンジを取得
if ((uintptr_t)(p_idx + 1) >= index_offsets_len) return;
const uint32_t i0 = index_offsets[p_idx];
const uint32_t i1 = index_offsets[p_idx + 1];
if (i1 <= i0 || (i1 - i0) % 3 != 0) return;
if ((uintptr_t)i1 > indices_len) return;
const uint32_t index_count = i1 - i0;

// 4. UV はインターリーブ [u0, v0, u1, v1, ...] なので v0 * 2 が開始オフセット
if ((uintptr_t)v1 * 2 > uvs_len) return;

// 5. 描画呼び出し
//    ※ indices は part-local 0-based。シングルトン描画（1 パーツ = 1 ドローコール）ならそのまま渡せる。
//       複数パーツを 1 バッファに集めて単一ドローコールにする実装の場合は、コピー時に各パーツの vertex_base を
//       indices 全要素に加算する。
DrawMeshPart(
    v_x + v0,
    v_y + v0,
    uvs + (uintptr_t)v0 * 2,
    vert_count,
    indices + i0,
    index_count
);
```

補足:  
- **UV は静的**：UV はフレーム 0 の Cell から一度だけ解決されフレーム間で変化しません（Mesh パーツは仕様上 Cell の途中変更ができないため）。プレイヤー はキャッシュ可能です。
- **`SceneGraph` モードでは空**：`ss_runtime_set_render_mode` で `RenderMode::SceneGraph` を選んだ場合、これらの SoA バッファは length 0 で返ります（Mesh の頂点データは `FrameData` (FlatBuffers) の `vertices` フィールド経由で提供されるため）。詳細は §07「RenderMode についての注意点」を参照。  
- **`offsets` 配列の長さは `num_parts + 1`**：末尾要素が「最後のパーツの終端」を兼ねる CSR (Compressed Sparse Row) 形式。`offsets[p + 1]` を安全に参照できるよう、`p_idx + 1` の範囲チェックを入れる必要があります。
- **非 Mesh パーツの扱い**：メッシュデータが存在しない（`offsets[p] == offsets[p+1]`）場合には、空レンジとして処理します。draw batch から Mesh パーツだけ拾う実装なら原理上起こりませんが、汎用的なガードとして入れておくのが安全です。
- **インデックスは part-local 0-based**：上のコメント参照。複数パーツを単一ドローコールで描画する経路を取らない限り、ベース加算は不要です。

## 5. この段階での動作確認 (Validation)

`overall.sspj` の以下のデータを使用して、複雑な構造が正しく再現されるかを確認します。

- **`Instance.ssae`**: 子アニメーションが親の動き・時間に同期してループ再生されているか。外部参照が解決されているか。
- **`Effect.ssae`**: パーティクルが正しく発生し、テクスチャや合成モードが反映されているか。独立再生が機能しているか。
- **`Mesh.ssae`**: 頂点変形やボーン追従によってメッシュが滑らかに歪んでいるか。

---
[目次に戻る](./README.ja.md)
