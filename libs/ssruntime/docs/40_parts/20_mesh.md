> [日本語](./20_mesh.ja.md) | **English**

# Mesh

A mesh part changes shape through bone skinning and vertex deformation. The Brain finishes the deformation and passes **world-space vertices** in SoA buffers. The Player only draws them. You do not need to multiply them by a world matrix.

## 1. Mesh SoA buffers

| Function | Contents |
| :--- | :--- |
| `ss_runtime_get_mesh_vertices_x` / `_y` | World coordinates of the vertices, concatenated for all parts |
| `ss_runtime_get_mesh_vertex_offsets` | Prefix sum of length `num_parts + 1`. The vertices of part `p` are `[offsets[p], offsets[p + 1])` |
| `ss_runtime_get_mesh_uvs` | UVs of the vertices, in `[u, v, ...]` order. Slice them at the same boundaries as the vertex buffers. They are pixel coordinates in the cellmap texture |
| `ss_runtime_get_mesh_indices` | Triangle indices. The values are **0-based within the part** |
| `ss_runtime_get_mesh_index_offsets` | Prefix sum of length `num_parts + 1`. The indices of part `p` are `[offsets[p], offsets[p + 1])` |

- A part that is not a mesh has an empty range (`offsets[p] == offsets[p + 1]`).
- **UVs and indices do not change from frame to frame while an animation plays.** The UVs are computed from the mesh part's cell at frame 0 (a mesh part cannot change its cell partway through). The Player may cache them, but read them again whenever the animation changes (track 0 included). Within one ssab, mesh parts that show different cells give different vertex ranges, UVs and indices.
- In `SceneGraph` mode, these buffers are empty and the Brain does not compute mesh vertices (FrameData's `vertices` holds only the cell's rectangle, and `deforms` only the deform offsets). A Player that draws meshes uses `Direct` mode ([Render Loop §3](../30_core/40_render_loop.md#3-rendermode)).

## 2. Slicing out a part's vertices

```cpp
const float    *v_x, *v_y, *uvs;
const uint32_t *v_offsets, *i_offsets;
const int32_t  *indices;
uintptr_t v_x_len, v_y_len, v_offsets_len, uvs_len, indices_len, i_offsets_len;

ss_runtime_get_mesh_vertices_x(ctx, &v_x, &v_x_len);
ss_runtime_get_mesh_vertices_y(ctx, &v_y, &v_y_len);
ss_runtime_get_mesh_vertex_offsets(ctx, &v_offsets, &v_offsets_len);
ss_runtime_get_mesh_uvs(ctx, &uvs, &uvs_len);
ss_runtime_get_mesh_indices(ctx, &indices, &indices_len);
ss_runtime_get_mesh_index_offsets(ctx, &i_offsets, &i_offsets_len);

// Vertex range of part p
if ((size_t)p + 1 >= v_offsets_len) return;
const uint32_t v0 = v_offsets[p], v1 = v_offsets[p + 1];
if (v1 <= v0 || v1 > v_x_len || (size_t)v1 * 2 > uvs_len) return;   // not a mesh, or out of range

// Index range of part p
if ((size_t)p + 1 >= i_offsets_len) return;
const uint32_t i0 = i_offsets[p], i1 = i_offsets[p + 1];
if (i1 <= i0 || (i1 - i0) % 3 != 0 || i1 > indices_len) return;

draw_mesh(v_x + v0, v_y + v0, uvs + (size_t)v0 * 2, v1 - v0,
          indices + i0, i1 - i0);
```

`draw_mesh` is a fictitious function that stands for the engine's drawing.

The indices are 0-based within the part, so if you draw one part with one draw call, you can pass them as is. **If you pack several parts into one buffer and draw them together, add the number of vertices already packed to each index.**

A mesh uses the same parts color value on every vertex ([Parts Color §4-2](./10_parts_color.md#4-2-mesh)).

---
[Back to contents](../README.md)
