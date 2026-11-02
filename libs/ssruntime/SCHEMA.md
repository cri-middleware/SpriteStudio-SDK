[**English**](./SCHEMA.md) | [**日本語**](./SCHEMA.ja.md)

# libssruntime FrameData Schema Documentation

This document describes the `FrameData` schema (`framedata.fbs`) used in `libssruntime`. `FrameData` acts as the primary **data exchange interface** between the `libssruntime` core and the Player (rendering engine) via FFI.

## Overview: When and What Data is Generated

To understand `FrameData`, it is helpful to look at the data pipeline from the original SpriteStudio data to the runtime output. 

While `.ssab` is generated **offline** (during the conversion phase) and contains the entire timeline of keyframes, `FrameData` is generated **dynamically at runtime** and contains only the snapshot of a single frame.

Specifically, `FrameData` takes the `AnimationData` from an `.ssab` file, evaluates the keyframes at a specific frame number, and outputs the `PartState` for all parts.

```text
[Offline Conversion]
.ssae (Animation Pack) ──────► .ssab (SsAnimeBinary)

[Runtime Playback]
.ssab (SsAnimeBinary)
 │
 ├─ AnimationData
 │    └─ parts_animation_data: [PartAnimationData] (a run of keyframes per attribute: POSX, ROTZ, etc.)
 │
 ＋  Current Frame Number (e.g., Frame 15)
 │
 ▼  (libssruntime: Interpolation & Coordinate Conversion)
 │
FrameData (Memory Buffer via FFI)
 └─ parts: [PartState] (Evaluated result for Frame 15: position_x, rotation_z, etc.)
```

### Key Characteristics

| Item | SSAB (`SsAnimeBinary`) | FrameData (`FrameData`) |
|------|------------------------|-------------------------|
| **Timing** | Generated offline (Asset) | Generated at runtime (per frame) |
| **Source Data**| `.ssae`, `.ssce`, `.ssee`, etc. | `.ssab`'s `AnimationData` |
| **Content** | Arrays of keyframes (Sparse) | Evaluated values for one frame (Dense) |
| **Unit** | Per Animation Pack | Per Frame |

---

## Attribute Mapping: SSAB to PartState

The following table shows exactly which fields of SSAB's `PartAnimationData` are evaluated to populate the fields of `PartState` in `FrameData`.

| PartState Field | Source: SSAB `PartAnimationData` Field | Notes |
|:---|:---|:---|
| `position_x/y/z` | `position_x/y/z_keys` | Interpolated position, converted to target coordinate system. |
| `rotation_x/y/z` | `rotation_x/y/z_keys` | Interpolated rotation (degrees). |
| `scale_x/y` | `scale_x/y_keys` | Interpolated scale multiplier. |
| `local_scale_x/y` | `local_scale_x/y_keys` | Interpolated local scale multiplier. |
| `alpha` | `alpha_keys` | Interpolated opacity (0.0 to 1.0). |
| `local_alpha` | `local_alpha_keys` | Interpolated local opacity. |
| `priority` | `priority_keys` | Current Z-order/priority. |
| `flip_h/v` | `flip_h/v_keys` | Current boolean flip state. |
| `hide` | `hide_keys` | Current boolean visibility state. |
| `pivot_x/y` | `pivot_x/y_keys` | Interpolated center of rotation/scale. |
| `anchor_x/y` | `anchor_x/y_keys` | Interpolated alignment anchor. |
| `size_x/y` | `size_x/y_keys` | Interpolated physical dimensions. |
| `img_flip_h/v` | `img_flip_h/v_keys` | Current visual-only flip state. |
| `uv_translation_x/y`| `uv_translation_x/y_keys`| Interpolated texture coordinate offset. |
| `uv_rotation_z` | `uv_rotation_z_keys` | Interpolated UV rotation. |
| `uv_scale_x/y` | `uv_scale_x/y_keys` | Interpolated UV scale. |
| `bounding_radius` | `bounding_radius_keys` | Interpolated collision radius. |
| `mask` | `mask_keys` | Interpolated mask influence. |
| `skew_x/y` | `skew_keys` | Derived from SSAB's Vec2 skew interpolation. |
| `texture` | `texture_keys` | Texture override (`TCHG`): fnv1a hash of the texture name, resolvable via `external_textures`. `0` = no override (the part keeps its cell's own texture). Step-evaluated — never interpolated. Already folded into `DrawBatch.texture_hash`, so a Player that batches by `texture_hash` needs no per-part handling. |

---

## `PartState` Identity

`PartState.part_index` is the part's index in the source SSAB `parts` array (and in `AnimationData.parts_animation_data`). The runtime currently emits `FrameData.parts` in the same order, so `parts[i].part_index() == i`, but treat `part_index` as the authoritative identity — `EventsPerFrame.users[i].part_index` and other event payloads address the SSAB part-index space, and the field is what bridges them to `FrameData`.

---

## Complex Attribute Mapping (Pooled)

Complex attributes are stored in pools within `FrameData` to avoid duplicating large data structures. They are referenced by `short` indices in `PartState`. An index of `-1` means "no entry for this attribute on this frame."

> **Note (`RenderMode`):** In `Direct` mode (the default), `cells`, `vertices`, and `deforms` are omitted (`None`) from `FrameData` because they are served through the SoA buffers (`ss_runtime_get_local_vertices`, `ss_runtime_get_local_uvs`, `ss_runtime_get_mesh_vertices_x/y`, etc.). They are populated in `FrameData` only in `SceneGraph` mode (`parts_color` and `shaders` are populated in both modes). See [Render Loop §3](./docs/30_core/40_render_loop.md#3-rendermode).

| PartState Index Field | Source: SSAB `PartAnimationData` Field | Pool in FrameData | Description |
|:---|:---|:---|:---|
| `cell` | `cell_keys` | `cells` (`[PartAttributeCell]` struct vector; `SceneGraph` mode only) | Reference to cell map and cell name hash. Normal drawing texture is derived from this attribute. |
| `part_color` | `part_color_keys` | `parts_color` (`[PartAttributePartColor]` struct vector) | Per-vertex color and blend type. **The RGBA `a` channel is pre-multiplied with the part's hierarchical `alpha`**, so the Player can write the color directly without a second alpha multiply. |
| `shader` | `shader_keys` | `shaders` (`[PartAttributeShader]` struct vector) | Interpolated shader parameters and map IDs. |
| `vertex` | `vertex_keys` | `vertices` (`[PartAttributeVertex]` struct vector; `SceneGraph` mode only) | **Pre-computed local vertex coordinates** for the part's quad. The four corners (`lt`/`rt`/`lb`/`rb`) are always populated with `pivot`, `size`, `img_flip_h/v`, and the interpolated raw deform offsets all applied; the Player consumes them directly through the world matrix. The fifth `center` vertex is populated **only when** `update_flag` has `AttributeVertex` or `AttributePartColor` set — those cases want a triangle fan from the center for smoother per-vertex deform / colour gradients. Otherwise `center` is `(0, 0)` and the Player should render the quad as two triangles from the four corners only. The raw deform offsets are not exposed; the `AttributeVertex` flag still indicates whether the part actually has deform animation in the source SSAB. |
| `deform` | `deform_keys` | `deforms` (`[PartAttributeDeform]` table vector; `SceneGraph` mode only) | Interpolated mesh deformation coordinates. |

---

## Draw Plan: Order, Batches, Mask Slice

`FrameData` carries not just per-part state but also a **cross-engine draw plan**: a stable, pre-sorted ordering of parts plus a contiguous batch list that the Player walks once per frame. This is the "DrawBatch contract"; [Render Loop](./docs/30_core/40_render_loop.md) is the reference for consuming it.

| FrameData Field | Type | Description |
|:---|:---|:---|
| `draw_order` | `[ushort]` (required) | Parts ordered for rendering. Each value is an **index into `FrameData.parts`** (not `part_index`). Parts that are hidden, masked-out, or otherwise non-drawable are not included. |
| `draw_batches` | `[DrawBatch]` (required) | Pre-merged draw commands. Each batch covers a contiguous slice of `draw_order` whose entries share the same renderable kind, blend mode, and texture. The Player issues one draw call per batch. |
| `mask_indices` | `[ushort]` (required) | Indices into `FrameData.parts` for the frame's **pure-mask** parts (`PartTypeMask` and `draw_as_mask`), in the same order as `draw_order`. **Not the writer list**: it excludes clipping (`mask_write`) and does not drop masks with `Hide == true`, so it does not agree with `draw_order`. Settle the writers by walking `draw_order` — see [Mask §3](./docs/40_parts/40_mask.md#3-what-to-read-from-framedata). |

### `DrawBatch`

Inline `struct` (fixed-size, no vtable) keyed by position in `draw_batches`:

| Field | Type | Description |
|:---|:---|:---|
| `start_rank` | ushort | Starting index into `draw_order` for this batch. |
| `count` | ushort | Number of consecutive `draw_order` entries belonging to this batch. |
| `texture_hash` | uint32 | fnv1a hash of the texture the batch binds — normally `cellmap_name_hash` (the cellmap name without `.ssce`), but the texture *name* hash of an authored `TextureChange` (`TCHG`) when one overrides the parts in this slice. A TCHG override participates in the batch key, so parts sharing a cellmap but bound to different textures are split into separate batches. Players must resolve both hash spaces: cellmap names and `external_textures` entries (every TCHG texture name is guaranteed present there). `0` if the kind does not need a texture (e.g. `Shape`). |
| `vertex_count` | ushort | Total vertex count contributed by all parts in this batch (sum across the slice). |
| `index_count` | ushort | Total index count contributed by all parts in this batch. |
| `blend_type` | BlendType | Shared framebuffer blend mode for the slice, from the ssab's `PartData.blend_type`. Not the parts color `blend_type`, which blends with the texture. |
| `kind` | DrawBatchKind | Renderable category (see below). Determines which geometry / data the Player should fetch for parts in the slice. |
| `flags` | ubyte | Per-batch flag bits. Defined: bit `0x01` = `MASK_INFLUENCE` (parts in this slice are masked by the active mask slice). Remaining bits are reserved — unrecognised bits should be ignored. See [Render Loop](./docs/30_core/40_render_loop.md) for the full contract. |

### `DrawBatchKind`

| Variant | Geometry source per part |
|:---|:---|
| `Normal` | Standard sprite quad — the `local_vertices` / `local_uvs` SoA buffers in `Direct` mode, the `vertices` + `cells` pools in `SceneGraph` mode. Texture from `texture_hash` (`cellmap_name_hash`, or the TCHG texture name when overridden). |
| `Shape` | Procedural shape from `PartTypeShape` (no texture). |
| `Instance` | Sub-anime instance slot — see [Player Porting Guide](./docs/README.md) for slot pulling. |
| `Effect` | Particle quads from the matching `EffectSlot` (see `effect_draw_plan.fbs`). |
| `Mesh` | Indexed mesh with bone-skinned vertices using `PartMeshBinding`. |
| `Text` | A text part's slot in the draw order — one part per batch, with no geometry (`vertex_count` / `index_count` are `0`) and no `texture_hash`. Glyph layout, rasterisation and font resolution are the Player's (see [Limitations](./docs/10_introduction/20_limitations.md)); text parts authored as masks are omitted from the plan. |
| `Nines` | 9-patch grid built from `PartTypeNines` — vertices, UVs and indices come from the Nines SoA buffers, sliced per part (see [9-Slice, Shape and Text](./docs/40_parts/30_special_parts.md)). |
| `Mask` | Stencil-write batch — does not produce colour output; entries are also listed in `mask_indices`. |

> **Walking the plan:** for each `batch` in `draw_batches`, iterate `draw_order[batch.start_rank .. batch.start_rank + batch.count]`, fetch each `PartState`, and emit geometry of `batch.kind`. Switch GPU state (texture, blend) once per batch rather than per part.

---

## Companion schema: `EffectDrawPlan`

The effect renderer is delivered through a separate root buffer (`effect_draw_plan.fbs`, root `EffectDrawPlan`), returned by `EffectSlot::step` / `ss_effect_slot_step`. `FrameData` references it only indirectly via `DrawBatchKind::Effect` batches; the actual quad geometry lives in `EffectDrawPlan` and is meant to be consumed alongside `FrameData` for the same frame.

| Field | Type | Description |
|:---|:---|:---|
| `commands` | `[EffectDrawCommand]` (required) | Per-emitter draw commands. Each carries `cellmap_hash`, `quad_count`, `particle_offset`, and `blend` (`0=Mix`, `1=Add` — raw `EmitterState.blend_type`). |
| `vert_stride` | ubyte | `2` for 2D-native engines (Godot, pixi.js), `3` for engines requiring XYZ vertices (Unity Mesh, Unreal FProcMeshVertex). Selected per `step` call. |
| `verts` | `[float]` (required) | Length `total_quads * 4 * vert_stride`. Y-flip is pre-applied per the caller's request. |
| `uvs` | `[float]` (required) | Length `total_quads * 4 * 2`. |
| `colors` | `[float]` (required) | Length `total_quads * 4 * 4` (RGBA, normalized 0..1). |
| `indices` | `[int]` (required) | Length `total_quads * 6`. Values are **quad-local** (`0..quad_count*4`); the engine slices and passes them alongside the slot-relative verts without rebasing. |

Slicing for command `c` (with `vs = vert_stride`):

```
verts  : c.particle_offset * 4 * vs .. (c.particle_offset + c.quad_count) * 4 * vs
uvs    : c.particle_offset * 4 * 2  .. (c.particle_offset + c.quad_count) * 4 * 2
colors : c.particle_offset * 4 * 4  .. (c.particle_offset + c.quad_count) * 4 * 4
indices: c.particle_offset * 6      .. (c.particle_offset + c.quad_count) * 6
```

---

## Update Flags (`UpdateAttributeFlags`)

The `update_flag` in `PartState` provides a bitmask indicating which attributes were explicitly active or changed in the SSAB source data (either via setup/initial values or keyframes) for the current frame. This allows the Player to skip processing default attributes.

| Flag Name | Source: SSAB `PartAnimationData` Field |
|:---|:---|
| `AttributeCell` | `cell_keys` |
| `AttributePosX/Y/Z` | `position_x/y/z_keys` |
| `AttributeRotX/Y/Z` | `rotation_x/y/z_keys` |
| `AttributeSclX/Y` | `scale_x/y_keys` |
| `AttributeLosclX/Y` | `local_scale_x/y_keys` |
| `AttributeAlpha` | `alpha_keys` |
| `AttributeLoAlpha` | `local_alpha_keys` |
| `AttributePrio` | `priority_keys` |
| `AttributeFlipH/V` | `flip_h/v_keys` |
| `AttributeHide` | `hide_keys` |
| `AttributePartColor` | `part_color_keys` |
| `AttributeShader` | `shader_keys` |
| `AttributeVertex` | `vertex_keys` |
| `AttributePivotX/Y` | `pivot_x/y_keys` |
| `AttributeAnchorX/Y` | `anchor_x/y_keys` |
| `AttributeSizeX/Y` | `size_x/y_keys` |
| `AttributeImgFlipH/V` | `img_flip_h/v_keys` |
| `AttributeUvtX/Y` | `uv_translation_x/y_keys` |
| `AttributeUvrZ` | `uv_rotation_z_keys` |
| `AttributeUvsX/Y` | `uv_scale_x/y_keys` |
| `AttributeBoundr` | `bounding_radius_keys` |
| `AttributeMask` | `mask_keys` |
| `AttributeSkew` | `skew_keys` |
| `AttributeDeform` | `deform_keys` |
| `AttributeTexture` | `texture_keys` |
