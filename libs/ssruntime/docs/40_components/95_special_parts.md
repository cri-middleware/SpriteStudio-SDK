> [日本語](./95_special_parts.ja.md) | **English**

# Supporting Special Part Types

Implementation hints for special part kinds such as 9-slice, shape, and sound.

| Part type | Implementation hint |
| :--- | :--- |
| **9-slice** | Draw the `DrawBatchKind::Nines` batches from the dedicated SoA buffers described below. `local_vertices` only carries the part's plain bounding quad for these parts. |
| **Shape** | Use the dedicated SoA buffers such as `ss_runtime_get_shape_vertices` to obtain the polygon's vertex data. |
| **Text** | Glyphs are the Player's: read `PartTypeText` (and `FontBitmap` for bitmap-font projects) from the ssab and draw with the host engine's text stack. The runtime only reserves the part's slot — a geometry-less `DrawBatchKind::Text` batch per part — so text composites by priority like anything else (see [Limitations](../10_introduction/20_limitations.md)). |
| **Sound** | Watch `passed_event` inside your update loop and play the sound if `audios` events are included (see [Audio](./90_audio.md) for details). |

## 9-slice

The runtime builds the whole grid — margins, fill mode, repeats — so a Player only uploads vertices. A batch whose `kind` is `Nines` points at parts whose geometry lives in these buffers:

| Buffer | Contents |
| :--- | :--- |
| `ss_runtime_get_nines_vertices` | Interleaved `[x, y, ...]` in part-local space, with pivot, image flip and the Y-Up/Y-Down setting already applied — multiply by the part's world matrix, exactly like `local_vertices`. |
| `ss_runtime_get_nines_uvs` | Interleaved `[u, v, ...]` in cellmap texture-pixel space. Divide by the texture size to normalize, exactly like `local_uvs`. |
| `ss_runtime_get_nines_box_coords` | Interleaved `[fx, fy, ...]`, `0` at the part's left / top edge and `1` at its right / bottom. Bilinearly interpolate the four `part_color` corners with them, the same way `shape_vertex_box_coords` works. |
| `ss_runtime_get_nines_indices` | Triangle indices, **local to each part's vertex range**, wound `LT, RT, LB / LB, RT, RB`. |
| `ss_runtime_get_nines_vertex_offsets` | CSR prefix sum, length `num_parts + 1`, slicing the three vertex buffers. Multiply by 2 to index the interleaved floats. |
| `ss_runtime_get_nines_index_offsets` | CSR prefix sum, length `num_parts + 1`, slicing the index buffer. |

Part `p` therefore occupies vertices `[vertex_offsets[p], vertex_offsets[p + 1])` and indices `[index_offsets[p], index_offsets[p + 1])`; every other part has an empty range. `DrawBatch.vertex_count` / `index_count` carry the totals for the batch's parts, so buffers can be sized up front — but they are 16-bit and saturate at 65535, so a heavily tiled part should be measured from the CSR offsets instead.

The vertex count depends on the part's authored fill mode: **stretch** (`nines_fill_mode == 0`) shares a 4x4 grid — 16 vertices and 54 indices — while **tile** (`== 1`) emits one quad per repeat, so the count follows the part's size. The margins that authoring stores in `PartTypeNines` are in source-cell pixels and are clamped internally when they would overlap, so a Player never has to validate them.

> **Not applied to 9-slice parts:** the UV animation attributes (`UVTX` / `UVTY` / `UVRZ` / `UVSX` / `UVSY`) and cells rotated inside their cellmap. Both follow the authoring tool, which builds the grid from the cell rectangle alone.

---
[Back to Table of Contents](../README.md)
