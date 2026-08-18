> [日本語](./65_override_layer.ja.md) | **English**

# Part Overrides (Override Layer API)

An API that overwrites **a specific part's color / cell / visibility** at runtime, taking priority over the animation's keyframes. It handles operations such as "make this part this color right now", "swap in this equipment cell", and "hide just this body part".

---

## 1. Conceptual model

### 1-1. Why it takes priority over keyframes

Overrides are applied at **one specific point (the seam)** in the single-frame evaluation pipeline.

```
per-source keyframe expansion
        ↓
   weighted mix (blending)
        ↓
 ★ override overwrite (here) ★
        ↓
   compose / FrameData output
```
> For details of the weighted mix, see [Animation Mixing](./60_animation_mixing.md).

It overwrites the resolved pose **after** the mix and **before** the output. Consequently an override ends up **overwriting the keyframe values and the blend result**.

An override is an overwrite (replacement) feature; do not implement it mixed together with blending (weighted composition). It is the same pipeline, but a different stage - keep that in mind.

> **The Player does not re-apply the overwrite every frame**: the override state is held on the Brain side. Once you `set` it, it keeps being applied automatically on subsequent frames. Do not implement it as "re-set it on every animation update". For the priority modes see [§1-3](#1-3-override-priority-modes); for the lifecycle see [§4](#4-lifecycle-and-part_index).

### 1-2. What can be overridden

The attributes that can be overridden are as follows.

| Feature | Applicable parts | Priority mode |
|---|---|---|
| **Parts color** | Normal only | Yes |
| **Cell** | Normal + Mask | Yes |
| **Visibility (force hide)** | All parts (can cascade to descendants) | **No** |

The things that are **not** override targets, and why, are below. Do not confuse them and try to "implement it with an override".

| What you want to do | Where it belongs (the responsible side) |
|---|---|
| Swapping a texture / cellmap (atlas) | **The Player**. Swap the texture bound by `DrawBatch.texture_hash` → [Managing External Resources and Textures](./70_external_resources.md). Note that an **authored** texture change (`TCHG`) is already evaluated by the runtime and reflected in `texture_hash` |
| Flipping | **The Player**. Express it in post-processing of the FrameData (scale inversion / UV swap) |
| Whole-object color / opacity / draw order | **The Player**. Not a per-part concern |
| Swapping an instance part | Planned for a different stage in future (the **Phase 3** Instance Lifecycle API) |

> **The part-type guard takes effect "at application time"**: for example, setting a color override on a Null part **succeeds and returns `true`** for the `set` call itself, but nothing happens because it is ignored at the seam. Do not use the call's success to decide "whether it took effect".

### 1-3. Override priority modes

Parts color and cell conflict when the animation uses the same attribute. You therefore specify **which of the keyframe and the override wins** in three levels. Visibility (force hide) does not conflict, so it has no priority mode (it always wins).

| Value | Name | Meaning | When a new animation is `setup` |
|---|---|---|---|
| `0` | `OverwriteOnNextKeyframe` | **Yields to keyframes**. If the animation updates that attribute, the override is cleared. | Cleared |
| `1` | `HoldUntilNextAnimation` | **The override keeps winning for the duration of the current animation** (the standard choice) | Cleared |
| `2` | `Permanent` | The override keeps winning and **survives animation changes** | Kept (re-validated against the part count) |

Think of `0` as a temporary overwrite that "gets taken back at the next keyframe", `1` as "a setting for this animation", and `2` as "a permanent setting for as long as this `ssab` (animation pack) is played".

The decision is made by looking at `PartState`'s `UpdateAttributeFlags` (if the `AttributePartColor` / `AttributeCell` bit is set, "a new attribute was updated on that frame").

---

## 2. Basic usage

```cpp
// 1. Resolve the part name into a part_index (-1 if not found)
int32_t part = ss_runtime_find_part_index(context, "body");
if (part < 0) { /* wrong part name / setup_animation not called */ }

// 2. Color: multiply body with red (held for the duration of the current animation)
ss_runtime_set_part_color_override(context, (uint32_t)part,
                                   /*blend_op*/ 1,          // 1 = Mul
                                   /*rgba*/ 0xFF0000FF,     // 0xRRGGBBAA
                                   /*priority*/ 1);         // 1 = HoldUntilNextAnimation

// 3. Cell: draw a different cell (names are passed as hashes)
uint32_t cellmap = ss_runtime_hash_string("Ringo");    // do not include the .ssce extension
uint32_t cell    = ss_runtime_hash_string("effect3");
if (!ss_runtime_set_part_cell_override(context, (uint32_t)part, cellmap, cell, 1)) {
    // the cell name could not be resolved → false immediately at set time (details in get_last_error)
}

// 4. Visibility: force hide, cascading to descendants
ss_runtime_set_part_visibility_override(context, (uint32_t)part,
                                        /*force*/ 1,        // 1 = force hide
                                        /*cascade*/ 1);     // cascade to descendants

// 5. Clearing
ss_runtime_clear_part_color_override(context, (uint32_t)part);
ss_runtime_clear_all_part_overrides(context);   // clear every attribute on every part at once
```

Parts are specified **only by `part_index` (`uint32`)**. If you want to operate by name, resolve it once with `ss_runtime_find_part_index` and cache the index on the Player side. A `part_index` is **stable within the same SSAB**.

> **There is no sentinel value meaning "all parts"**: the only cross-part bulk operation in the Brain's override feature is `ss_runtime_clear_all_part_overrides`; any other bulk operation is the Player's responsibility.

---

## 3. Details of each override

### 3-1. Parts-color override

Applies **only to Normal parts**.

- **Compositing mode (`blend_op`)**: `0=Mix` / `1=Mul` / `2=Add` / `3=Sub` - **the same four kinds** as the keyframe PartColor. For the compositing formulas themselves see [Parts Color §1-4](./50_parts_color.md#1-4-reference-applying-parts-color-in-a-shader-table-driven-reference-implementation). A value of `4` or greater returns `false` at `set` time (unlike the parts-color side's "defensively clamp to Mix", this **rejects** it).
- **Single-color and four-corner versions**: `ss_runtime_set_part_color_override` feeds one RGBA to all four corners. To give a gradient, use `ss_runtime_set_part_color_override_corners`.
- **`rate` is always `1.0`**: an override's compositing strength is fixed at maximum.

> **The corner order is `lt, rt, lb, rb`** (top-left, top-right, bottom-left, bottom-right). Getting this wrong leaves the gradient pointing the wrong way.

> **Do not apply alpha twice**: the `a` of the RGBA you specify is **automatically pre-multiplied with that part's composited α at the seam**. This is exactly the same rule as the keyframe PartColor ([Parts Color §1-3](./50_parts_color.md#1-3-reference-the-data-contained-in-a-parts-color)). Do not multiply `state->alpha()` again on the Player side.
>
> **Colors are interpreted in 8-bit sRGB space**: the Brain performs all color arithmetic in 8-bit sRGB integer space and performs no linear / gamma / PMA conversion at all. Override colors are treated in **exactly the same space as keyframe PartColor**. Players that need linear conversion should concentrate it in the shader hooks (`ss_input_texture()` / `ss_output_color()`) as before. **Do not build a separate color pipeline just for overrides** (that would convert twice).

Setting a color override on **a part that had no PartColor in its keyframes** raises the `AttributePartColor` flag at the seam. As a result, that part starts being emitted in the **five-vertex (four corners plus center) form** (the center vertex of [Parts Color §1-2-1](./50_parts_color.md#1-2-1-example-for-normal-parts)). A Player that "branches its drawing path on whether a PartColor exists" must expect this switch to happen (if you unified the path as recommended in [Parts Color §1-2-4](./50_parts_color.md#1-2-4-defaults-when-there-is-no-parts-color-color_idx--0), there is nothing to do).

### 3-2. Cell override

Applies to **Normal parts and Mask parts**.

A cell is specified as `(cellmap_name_hash, cell_name_hash)`. The cellmap name **does not include the `.ssce` extension**. Generate the hashes of the cellmap name and cell name with `ss_runtime_hash_string()`.

- **Resolution happens exactly once at `set` time** and the result (`ResolvedCellMeta`) is cached. Therefore **an unknown cellmap name / cell name returns `false` immediately at `set` time** (no per-frame hash lookup occurs).
- **Passing `(0, 0)` clears it**, returning to the animation's cell.

Swapping a cell makes the seam rewrite **four families** of data. What is mainly visible from the Player is (2) and (3).

| # | Family | Effect |
|---|---|---|
| (1) | Geometry (pivot / size / rect) | Reflected in vertex computation |
| (2) | **`texture_hash`** | **The `DrawBatch` split points change** |
| (3) | UV | `ss_runtime_get_local_uvs` and friends return the UVs of the swapped-in cell |
| (4) | Cell reference | The FrameData `cell` index points at the swapped-in cell |

> **If the replacement cell is on a different texture, the batch split changes**: when `texture_hash` changes, that part is split out into a different `DrawBatch`. This is already accounted for in the Brain's drawing planning, so the Player can draw correctly by **simply reading `draw_batches` as usual**. Note however that if the Player has optimizations that assume "the number of batches is fixed", those break.

**Parts that have no SizeX / SizeY keys** re-derive their default size from the swapped-in cell's size (the same behavior as the keyframe side's "fall back to the cell size").

### 3-3. Visibility override (force hide)

- `force = 0` clears it, `force = 1` forces hiding. **`force = 2` (force show) is reserved but currently always returns `false`.**
- **There is no priority mode.** It always beats keyframes, and it is **always cleared** when a new animation is `setup` (behaviorally equivalent to the `HoldUntilNextAnimation` priority mode).
- With `cascade != 0`, the forced hiding cascades to every descendant part.

> **The cascade propagates only "hiding that came from an override"**: SS7's keyframe `hide` is **per part** and does **not** propagate to descendants. This asymmetry is deliberate. If the Player implements its own propagation of "if the parent is hidden, do not draw the children", the meaning of the keyframe `hide` changes. **`PartState.hide` expresses only whether that part itself is drawn.**

> **It does not reach inside instance parts**: a child animation runs as a separate context, so the parent context's overrides do not affect the parts inside the child. When you force-hide an instance part itself, not drawing the child animation beneath it is **the Player's responsibility**.

---

## 4. Lifecycle and part_index

The override tables are dense arrays indexed by `part_index`, held by the context (the instance). A `part_index` is **stable only within the same SSAB**. If you swap in a different `.ssab`, re-obtain the indices.

| Trigger | Behavior |
|---|---|
| `ss_runtime_setup_animation` (a different animation within the same SSAB) | **All visibility overrides are cleared**. For color / cell, **only `Permanent` remains**; `OverwriteOnNextKeyframe` / `HoldUntilNextAnimation` are cleared. The surviving `Permanent` entries are re-validated against the new part count (out-of-range ones are discarded) |
| `ss_runtime_bind_resource` (binding a different `.ssab`) | **All cleared** (part identity is lost) |
| `reset` | All cleared |

> **When there is not a single override, the seam's processing is skipped entirely**: the whole seam is gated on "is there at least one active override?", so there is effectively no performance overhead for Players that do not use this API.

---

## 5. Traps Player implementations easily fall into

Here are the ones actually hit during porting work.

### 5-1. A frame-number-based cache gets in the way

Overrides **change the output without changing `frame_no`**. In a Player that has the optimization "skip rebuilding if the frame number is the same as last time" (effective while paused, on static frames, and in low-frame-rate playback), the geometry is not updated after `set` / `clear`, and **the API appears not to work**.

**Countermeasure**: when an override `set` / `clear` succeeds, explicitly invalidate the frame cache. This is a trap the Unity Player actually hit.

### 5-2. `RenderMode::Direct` does not put `cells` into the FrameData

In `RenderMode::Direct` (the default), `compose_frame_data` **does not put the `cells` / `vertices` / `deforms` vectors into the FrameData** (those are passed through the SoA buffers instead). **`parts_color` is included even in Direct.**

Therefore **you cannot verify the effect of a cell override by looking at `FrameData.cells`** (it is easy to misdiagnose "not present = not working"). To confirm the result of a cell swap in Direct mode, look at:

- The UVs returned by `ss_runtime_get_local_uvs()` (they become the swapped-in cell's UVs)
- `DrawBatch.texture_hash` (it changes if the replacement is on a different texture)

---

## 6. C-API reference

All functions return `false` (or `-1`) on failure; details can be obtained with `ss_runtime_get_last_error(ctx)`.

### 6-1. Resolving parts

| Function | Return value | On error |
|---|---|---|
| `ss_runtime_find_part_index(ctx, part_name)` | `int32_t` (`part_index`) | `-1` (name mismatch / `setup_animation` not called) |
| `ss_runtime_hash_string(str)` | `uint32_t` (the hash) | `0` |

### 6-2. Color

| Function | Arguments | Notes |
|---|---|---|
| `ss_runtime_set_part_color_override(ctx, part, blend_op, rgba, priority_mode)` | `blend_op`: `0=Mix 1=Mul 2=Add 3=Sub`<br>`rgba`: `0xRRGGBBAA` | Feeds one color to all four corners. Normal parts only |
| `ss_runtime_set_part_color_override_corners(ctx, part, blend_op, rgba_ltrtlbrb, priority_mode)` | `rgba_ltrtlbrb`: `uint32` × 4 (**in `lt, rt, lb, rb` order**) | Specifies the four corners individually |
| `ss_runtime_clear_part_color_override(ctx, part)` | — | |

### 6-3. Cell

| Function | Arguments | Notes |
|---|---|---|
| `ss_runtime_set_part_cell_override(ctx, part, cellmap_name_hash, cell_name_hash, priority_mode)` | cellmap_name_hash: the hash of the cellmap name<br>cell_name_hash: the hash of the cell name | `(0, 0)` clears it. `false` if resolution fails. Normal + Mask only |
| `ss_runtime_clear_part_cell_override(ctx, part)` | — | |

### 6-4. Visibility

| Function | Arguments | Notes |
|---|---|---|
| `ss_runtime_set_part_visibility_override(ctx, part, force, cascade)` | `force`: `0=clear 1=force hide 2=force show (unimplemented)`<br>`cascade != 0` cascades to descendants | No priority mode |
| `ss_runtime_clear_part_visibility_override(ctx, part)` | — | |

### 6-5. Bulk

| Function | Notes |
|---|---|
| `ss_runtime_clear_all_part_overrides(ctx)` | Clears every attribute on every part |

`priority_mode` is uniformly `0=OverwriteOnNextKeyframe` / `1=HoldUntilNextAnimation` / `2=Permanent` throughout ([§1-3](#1-3-override-priority-modes)). Out-of-range values return `false`.

---

[Back to Table of Contents](../README.md)
