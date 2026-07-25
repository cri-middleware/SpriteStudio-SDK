> [日本語](./30_mesh.ja.md) | **English**

# Mesh & Deform

Supports skin-mesh deformation driven by bones, as well as free vertex deformation. You obtain the already-computed vertices from the **SoA buffers** provided by `libssruntime` and draw them.

## Mesh SoA buffers
Mesh vertex computation (including skinning) is completed on the Brain side. The Player simply obtains the **already-computed world vertex coordinates** directly from the SoA buffers and draws them.
It is fine to obtain each SoA buffer once per context.

```cpp
// Hold the SoA buffer pointers and lengths together
const float    *v_x, *v_y, *uvs;
const uint32_t *v_offsets, *index_offsets;
const int32_t  *indices;
size_t v_x_len, v_y_len, v_offsets_len, uvs_len, indices_len, index_offsets_len;

// 1. Obtain each SoA buffer
ss_runtime_get_mesh_vertices_x(ctx, &v_x, &v_x_len);
ss_runtime_get_mesh_vertices_y(ctx, &v_y, &v_y_len);
ss_runtime_get_mesh_vertex_offsets(ctx, &v_offsets, &v_offsets_len);
ss_runtime_get_mesh_uvs(ctx, &uvs, &uvs_len);
ss_runtime_get_mesh_indices(ctx, &indices, &indices_len);
ss_runtime_get_mesh_index_offsets(ctx, &index_offsets, &index_offsets_len);

// 2. Obtain the vertex range (this also rejects the empty ranges of non-Mesh parts)
if ((uintptr_t)(p_idx + 1) >= v_offsets_len) return; // guard against p_idx out of range
const uint32_t v0 = v_offsets[p_idx];
const uint32_t v1 = v_offsets[p_idx + 1];
if (v1 <= v0) return;                                 // non-Mesh part = empty range
if ((uintptr_t)v1 > v_x_len) return;                  // data-consistency guard
const uint32_t vert_count = v1 - v0;

// 3. Obtain the index range
if ((uintptr_t)(p_idx + 1) >= index_offsets_len) return;
const uint32_t i0 = index_offsets[p_idx];
const uint32_t i1 = index_offsets[p_idx + 1];
if (i1 <= i0 || (i1 - i0) % 3 != 0) return;
if ((uintptr_t)i1 > indices_len) return;
const uint32_t index_count = i1 - i0;

// 4. UVs are interleaved [u0, v0, u1, v1, ...], so v0 * 2 is the start offset
if ((uintptr_t)v1 * 2 > uvs_len) return;

// 5. Draw call
//    Note: indices are part-local 0-based. With singleton drawing (1 part = 1 draw call) you can pass them as-is.
//          If your implementation gathers several parts into one buffer for a single draw call, add each part's
//          vertex_base to every index element while copying.
DrawMeshPart(
    v_x + v0,
    v_y + v0,
    uvs + (uintptr_t)v0 * 2,
    vert_count,
    indices + i0,
    index_count
);
```

Notes:
- **UVs are static**: UVs are resolved once from the Cell at frame 0 and do not change between frames (by specification, a Mesh part's Cell cannot change mid-animation). The Player may cache them.
- **Empty in `SceneGraph` mode**: if you select `RenderMode::SceneGraph` via `ss_runtime_set_render_mode`, these SoA buffers are returned with length 0 (because Mesh vertex data is then provided through the `vertices` field of `FrameData` (FlatBuffers)). See [Rendering Loop, "Notes on RenderMode"](../30_core/30_render_loop.md) for details.
- **The `offsets` arrays have length `num_parts + 1`**: a CSR (Compressed Sparse Row) layout where the final element doubles as "the end of the last part". You need a range check on `p_idx + 1` so that `offsets[p + 1]` can be referenced safely.
- **Handling non-Mesh parts**: when no mesh data exists (`offsets[p] == offsets[p+1]`), treat it as an empty range. This cannot happen in principle if your implementation picks only Mesh parts out of the draw batch, but it is safer to keep the guard as a general-purpose check.
- **Indices are part-local 0-based**: see the comment above. Unless you take the path of drawing several parts in a single draw call, no base addition is required.

---
[Back to Table of Contents](../README.md)
