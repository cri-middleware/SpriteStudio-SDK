[**English**](./SCHEMA.md) | [**日本語**](./SCHEMA.ja.md)

# SS-Runtime FrameData Schema Documentation

This document describes the `FrameData` schema (`framedata.fbs`) used in the SS-Runtime. `FrameData` acts as the primary **data exchange interface** between the SS-Runtime core and the Player (rendering engine) via FFI.

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
 │    └─ parts_animation_data: [PartAnimationData] (Keyframe arrays: POSX, ROTZ, etc.)
 │
 ＋  Current Frame Number (e.g., Frame 15)
 │
 ▼  (SS-Runtime: Interpolation & Coordinate Conversion)
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

The following table shows exactly which arrays from SSAB's `PartAnimationData` are evaluated to populate the fields of `PartState` in `FrameData`.

| PartState Field | Source: SSAB `PartAnimationData` Array | Notes |
|:---|:---|:---|
| `position_x/y/z` | `position_x/y/z_array` | Interpolated position, converted to target coordinate system. |
| `rotation_x/y/z` | `rotation_x/y/z_array` | Interpolated rotation (degrees). |
| `scale_x/y` | `scale_x/y_array` | Interpolated scale multiplier. |
| `local_scale_x/y` | `local_scale_x/y_array` | Interpolated local scale multiplier. |
| `alpha` | `alpha_array` | Interpolated opacity (0.0 to 1.0). |
| `local_alpha` | `local_alpha_array` | Interpolated local opacity. |
| `priority` | `priority_array` | Current Z-order/priority. |
| `flip_h/v` | `flip_h/v_array` | Current boolean flip state. |
| `hide` | `hide_array` | Current boolean visibility state. |
| `pivot_x/y` | `pivot_x/y_array` | Interpolated center of rotation/scale. |
| `anchor_x/y` | `anchor_x/y_array` | Interpolated alignment anchor. |
| `size_x/y` | `size_x/y_array` | Interpolated physical dimensions. |
| `img_flip_h/v` | `img_flip_h/v_array` | Current visual-only flip state. |
| `uv_translation_x/y`| `uv_translation_x/y_array`| Interpolated texture coordinate offset. |
| `uv_rotation_z` | `uv_rotation_z_array` | Interpolated UV rotation. |
| `uv_scale_x/y` | `uv_scale_x/y_array` | Interpolated UV scale. |
| `bounding_radius` | `bounding_radius_array` | Interpolated collision radius. |
| `mask` | `mask_array` | Interpolated mask influence. |
| `skew_x/y` | `skew_array` | Derived from SSAB's Vec2 skew interpolation. |
| `texture` | `texture_array` | Texture override/change ID. |

---

## Complex Attribute Mapping (Pooled)

Complex attributes are stored in pools within `FrameData` to avoid duplicating large data structures. They are referenced by `short` indices in `PartState`.

| PartState Index Field | Source: SSAB `PartAnimationData` Array | Table in FrameData | Description |
|:---|:---|:---|:---|
| `cell` | `cell_array` | `cells` | Reference to cell map and cell name hash. Normal drawing texture is derived from this attribute. |
| `part_color` | `part_color_array` | `parts_color` | Per-vertex color and blend type. **The RGBA `a` channel is pre-multiplied with the part's hierarchical `alpha`**, so the Player can write the color directly without a second alpha multiply. |
| `shader` | `shader_array` | `shaders` | Interpolated shader parameters and map IDs. |
| `vertex` | `vertex_array` | `vertices` | **Pre-computed local vertex coordinates** (LT/RT/LB/RB plus a `center` vertex) for the part's quad, with `pivot`, `size`, `img_flip_h/v`, and the interpolated raw deform offsets all applied. The Player consumes these directly through the world matrix; the raw deform offsets are no longer exposed. The `center` vertex is the average of the four corners (Varignon's theorem) and lets Players render the quad as a triangle fan from the center for smoother per-vertex deform. The `AttributeVertex` flag in `update_flag` still indicates whether the part actually has deform animation in the source SSAB. |
| `deform` | `deform_array` | `deforms` | Interpolated mesh deformation coordinates. |

---

## Update Flags (`UpdateAttributeFlags`)

The `update_flag` in `PartState` provides a bitmask indicating which attributes were explicitly active or changed in the SSAB source data (either via setup/initial values or keyframes) for the current frame. This allows the Player to skip processing default attributes.

| Flag Name | Source: SSAB `PartAnimationData` Array |
|:---|:---|
| `AttributeCell` | `cell_array` |
| `AttributePosX/Y/Z` | `position_x/y/z_array` |
| `AttributeRotX/Y/Z` | `rotation_x/y/z_array` |
| `AttributeSclX/Y` | `scale_x/y_array` |
| `AttributeLosclX/Y` | `local_scale_x/y_array` |
| `AttributeAlpha` | `alpha_array` |
| `AttributeLoAlpha` | `local_alpha_array` |
| `AttributePrio` | `priority_array` |
| `AttributeFlipH/V` | `flip_h/v_array` |
| `AttributeHide` | `hide_array` |
| `AttributePartColor` | `part_color_array` |
| `AttributeShader` | `shader_array` |
| `AttributeVertex` | `vertex_array` |
| `AttributePivotX/Y` | `pivot_x/y_array` |
| `AttributeAnchorX/Y` | `anchor_x/y_array` |
| `AttributeSizeX/Y` | `size_x/y_array` |
| `AttributeImgFlipH/V` | `img_flip_h/v_array` |
| `AttributeUvtX/Y` | `uv_translation_x/y_array` |
| `AttributeUvrZ` | `uv_rotation_z_array` |
| `AttributeUvsX/Y` | `uv_scale_x/y_array` |
| `AttributeBoundr` | `bounding_radius_array` |
| `AttributeMask` | `mask_array` |
| `AttributeSkew` | `skew_array` |
| `AttributeDeform` | `deform_array` |
| `AttributeTexture` | `texture_array` |
