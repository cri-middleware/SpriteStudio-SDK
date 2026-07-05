# メッシュとデフォーム (Mesh / Deform)

ボーンによるスキンメッシュ変形や、自由な頂点変形をサポートします。`libssruntime` が提供する **SoA バッファ** から計算済みの頂点を取得して描画します。

## メッシュ SoA バッファ
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
- **`SceneGraph` モードでは空**：`ss_runtime_set_render_mode` で `RenderMode::SceneGraph` を選んだ場合、これらの SoA バッファは length 0 で返ります（Mesh の頂点データは `FrameData` (FlatBuffers) の `vertices` フィールド経由で提供されるため）。詳細は [レンダリングループ「RenderMode についての注意点」](../30_core/30_render_loop.ja.md#rendermode-についての注意点) を参照。  
- **`offsets` 配列の長さは `num_parts + 1`**：末尾要素が「最後のパーツの終端」を兼ねる CSR (Compressed Sparse Row) 形式。`offsets[p + 1]` を安全に参照できるよう、`p_idx + 1` の範囲チェックを入れる必要があります。
- **非 Mesh パーツの扱い**：メッシュデータが存在しない（`offsets[p] == offsets[p+1]`）場合には、空レンジとして処理します。draw batch から Mesh パーツだけ拾う実装なら原理上起こりませんが、汎用的なガードとして入れておくのが安全です。
- **インデックスは part-local 0-based**：上のコメント参照。複数パーツを単一ドローコールで描画する経路を取らない限り、ベース加算は不要です。

---
[目次に戻る](../README.ja.md)
