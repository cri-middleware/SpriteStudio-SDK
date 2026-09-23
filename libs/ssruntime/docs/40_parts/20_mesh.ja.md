> **日本語** | [English](./20_mesh.md)

# メッシュ

メッシュパーツは、ボーンによるスキニングや頂点の変形で形が変わるパーツです。変形の計算は Brain が済ませ、**ワールド座標の頂点**を SoA バッファで渡します。Player はそれを描くだけです。ワールド行列を掛ける必要はありません。

## 1. メッシュの SoA バッファ

| 関数 | 内容 |
| :--- | :--- |
| `ss_runtime_get_mesh_vertices_x` / `_y` | 頂点のワールド座標。全パーツ分を連結したもの |
| `ss_runtime_get_mesh_vertex_offsets` | 長さ `パーツ数 + 1` の累積和。パーツ `p` の頂点は `[offsets[p], offsets[p + 1])` |
| `ss_runtime_get_mesh_uvs` | 頂点の UV。`[u, v, ...]` の順で、頂点のバッファと同じ区切りで切り出す。セルマップのテクスチャのピクセル座標 |
| `ss_runtime_get_mesh_indices` | 三角形のインデックス。**パーツの中での 0 始まり**の値 |
| `ss_runtime_get_mesh_index_offsets` | 長さ `パーツ数 + 1` の累積和。パーツ `p` のインデックスは `[offsets[p], offsets[p + 1])` |

- メッシュでないパーツは、空の範囲（`offsets[p] == offsets[p + 1]`）になります。
- **UV とインデックスはフレームによって変わりません。** UV はメッシュパーツのフレーム 0 のセルから求めます（メッシュパーツは途中でセルを変えられません）。Player はキャッシュしてかまいません。
- `SceneGraph` モードでは、これらのバッファは空です。メッシュの頂点は FrameData の `vertices` で渡されます（[描画ループ §3](../30_core/40_render_loop.ja.md#3-rendermode)）。

## 2. パーツの頂点を切り出す

```cpp
const float    *v_x, *v_y, *uvs;
const uint32_t *v_offsets, *i_offsets;
const int32_t  *indices;
size_t v_x_len, v_y_len, v_offsets_len, uvs_len, indices_len, i_offsets_len;

ss_runtime_get_mesh_vertices_x(ctx, &v_x, &v_x_len);
ss_runtime_get_mesh_vertices_y(ctx, &v_y, &v_y_len);
ss_runtime_get_mesh_vertex_offsets(ctx, &v_offsets, &v_offsets_len);
ss_runtime_get_mesh_uvs(ctx, &uvs, &uvs_len);
ss_runtime_get_mesh_indices(ctx, &indices, &indices_len);
ss_runtime_get_mesh_index_offsets(ctx, &i_offsets, &i_offsets_len);

// パーツ p の頂点の範囲
if ((size_t)p + 1 >= v_offsets_len) return;
const uint32_t v0 = v_offsets[p], v1 = v_offsets[p + 1];
if (v1 <= v0 || v1 > v_x_len || (size_t)v1 * 2 > uvs_len) return;   // メッシュでない、または範囲外

// パーツ p のインデックスの範囲
if ((size_t)p + 1 >= i_offsets_len) return;
const uint32_t i0 = i_offsets[p], i1 = i_offsets[p + 1];
if (i1 <= i0 || (i1 - i0) % 3 != 0 || i1 > indices_len) return;

draw_mesh(v_x + v0, v_y + v0, uvs + (size_t)v0 * 2, v1 - v0,
          indices + i0, i1 - i0);
```

`draw_mesh` は、エンジンの描画処理を表す実在しない関数です。

インデックスはパーツの中での 0 始まりなので、1 パーツを 1 回のドローコールで描くなら、そのまま渡せます。**複数のパーツを 1 つのバッファにまとめて描く場合は、それまでに詰めた頂点の数を、各インデックスに足してください。**

メッシュのパーツカラーは、全頂点に同じ値を使います（[パーツカラー §4-2](./10_parts_color.ja.md#4-2-メッシュ)）。

---
[目次に戻る](../README.ja.md)
