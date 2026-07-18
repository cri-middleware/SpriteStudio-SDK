> **日本語** | [English](./95_special_parts.md)

# 特殊なパーツタイプへの対応

9 スライス・シェイプ・サウンドといった特殊なパーツ種別の実装ヒントです。

| パーツ種別 | 実装のヒント |
| :--- | :--- |
| **9 スライス** | SoA の `local_vertices` には既に 9 スライス計算済みの座標が入っています。通常のパーツと同様に描画するだけで対応可能です。 |
| **シェイプ** | 頂点は `ss_runtime_get_shape_vertices` 等の SoA バッファ、三角形分割は `ss_runtime_get_shape_indices` から取得して描画します。詳細は下記 [シェイプの三角形分割](#シェイプの三角形分割) を参照。 |
| **サウンド** | 更新ループ内で `passed_event` を監視し、`audios` イベントが含まれていれば再生します（詳細は [オーディオ（サウンド）再生の統合](./90_audio.ja.md) を参照）。 |

---

## シェイプの三角形分割

シェイプパーツ（矩形 / 三角形 / 星 / 矢印）は、Brain が **頂点** と **三角形インデックス** の両方を SoA バッファで供給します。Player はインデックスに従って三角形を組んで描画します。多角形の分割ロジックを Player 側に持つ必要はありません。

### 供給されるバッファ

いずれも `part_index` で並ぶ per-part バッファで、`ss_runtime_get_frame_data`（＝ 1 フレーム評価）の後に有効になります。

| データ | C-API | WASM | 型 | レイアウト |
| :--- | :--- | :--- | :--- | :--- |
| 頂点座標 | `ss_runtime_get_shape_vertices` | `get_shape_vertices_buf` / `_len` | `f32` | パーツ `p` は `[p*24 .. p*24 + count*2]`、`(x, y)` インターリーブ（stride 24 = 最大 12 頂点 × 2） |
| 頂点数 | `ss_runtime_get_shape_vertex_counts` | `get_shape_vertex_counts_buf` / `_len` | `i32` | `counts[p]` = パーツ `p` の有効頂点数（シェイプ以外は 0） |
| 三角形インデックス | `ss_runtime_get_shape_indices` | `get_shape_indices_buf` / `_len` | `i32` | 全パーツ連結。パーツ `p` の範囲は下記オフセットで切り出す。値は **パーツローカル**（`0..count`） |
| インデックスオフセット | `ss_runtime_get_shape_index_offsets` | `get_shape_index_offsets_buf` / `_len` | `u32` | CSR、長さ `num_parts + 1`。パーツ `p` は `[offsets[p], offsets[p+1])`。シェイプ以外は空範囲 |

> インデックス／オフセットは `mesh` の三角形インデックス（`ss_runtime_get_mesh_indices` / `ss_runtime_get_mesh_index_offsets`）と**同じ CSR 構造**です。既に mesh を実装している Player は同じ消費コードを流用できます。

### 描画方法

パーツ `p` がシェイプ（`shape_vertex_counts[p] >= 3`）のとき:

```text
count = shape_vertex_counts[p]
vbase = p * 24                          // 頂点バッファ内のこのパーツの先頭
ibeg  = shape_index_offsets[p]
iend  = shape_index_offsets[p + 1]

// 三角形は 3 個ずつ。インデックスはパーツローカル（0..count）。
for k in (ibeg .. iend) step 3:
    for t in 0 .. 3:
        local = shape_indices[k + t]              // 0..count
        x = shape_vertices[vbase + local * 2]
        y = shape_vertices[vbase + local * 2 + 1]
        emit_vertex(x, y)                         // UV / 頂点カラー等は既存経路と同じ
```

PartColor（頂点カラー）や頂点デフォームは、頂点と同じ `part_index` / 頂点順で他の SoA（`shape_vertex_box_coords` 等）から引けます。インデックスは**トポロジのみ**を表し、ポーズやサイズには依存しません（フレームをまたいで不変）。

---
[目次に戻る](../README.ja.md)
