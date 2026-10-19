> [日本語](./20_part_overrides.ja.md) | **English**

# Part Overrides

Part overrides overwrite **a specific part's color, cell and visibility** at runtime, taking priority over the animation's keyframes. Use them for operations such as "make this part this color now", "swap in this equipment cell" and "hide only this body part". The Brain holds the override state, so once the Player sets an override, it does not need to set it again every frame. The argument order and default values in a Player's public API are in [Player API Conventions §1](../20_design/30_api_conventions.md#1-argument-order) and [§6](../20_design/30_api_conventions.md#6-defaults).

## 1. Concepts

### 1-1. Why overrides take priority over keyframes

Overrides are applied at **one fixed point** in the computation of a frame.

```
compute keyframes per source
        ↓
   mix with weights (animation mixing)
        ↓
 ★ overwrite with overrides (here) ★
        ↓
   build FrameData
```

The resolved pose is overwritten **after** mixing and **before** output. So overrides **overwrite both the keyframe values and the result of mixing.** An override is a replacement; it is a separate stage from mixing (weighted blending) ([Animation Mixing](./10_animation_mixing.md)).

### 1-2. What can be overwritten

| Feature | Parts it affects | Priority mode |
|---|---|---|
| **Parts color** | Normal parts only | Yes |
| **Cell** | Normal parts and mask parts | Yes |
| **Visibility (force hide)** | All parts (can cascade to descendants) | **No** |

**Setting an override on a part it does not affect still succeeds, and `set` returns `true`.** For example, a color override set on a null part is simply ignored when it is applied. Do not judge from the call's result whether an override took effect.

The following are not override targets.

| What you want to do | Where to implement it |
|---|---|
| Swapping a texture or cellmap | **Player**. Swap the texture bound under the key `DrawBatch.texture_hash` ([External Resources and Textures §3](./30_external_resources.md#3-replacing-textures-at-runtime)). An authored texture change (`TCHG`) is already computed by the Brain and reflected in `texture_hash` |
| Flip | **Player**. Flip it in the drawing transform |
| Color, opacity or draw order of the whole object | **Player**. These are not per-part concerns |
| Swapping what an instance part references | Not possible |

### 1-3. Priority modes

Parts color and cell conflict when the animation has keys on the same attribute. You therefore choose, from three levels, **which wins: the keyframe or the override**. Visibility does not conflict, so it has no priority mode (the override always wins).

| Value | Name | Meaning | When another animation is set up |
|---|---|---|---|
| `0` | `OverwriteOnNextKeyframe` | **Yields to keyframes.** Cleared when a keyframe for that attribute arrives | Cleared |
| `1` | `HoldUntilNextAnimation` | **The override wins for the rest of the current animation** (the usual choice) | Cleared |
| `2` | `Permanent` | The override keeps winning and **survives switching animations** | Kept (re-checked against the part count) |

Think of `0` as a temporary overwrite that "the next keyframe takes back", `1` as "a setting for this animation", and `2` as "a setting for as long as this ssab plays".

A keyframe "arrives" when playback passes through a frame on which the animation track 0 plays has a key for that attribute (Part Color or Cell) on that part. The frames `ss_runtime_update` passed through are checked when the next frame is built. Only frames the playhead actually visits count, so a play section, loops, reverse and ping-pong need no special handling, and a seek counts the frame it lands on. The frame the override was set on is not counted by the first frame built after setting it, nor by any other frame built before the next `ss_runtime_update`, so an override set on a keyframe wins that keyframe's segment however many times a frame is built. After `play` or a seek, the next `ss_runtime_update` passes the frame it starts on, so an override set there wins that frame's segment only if no frame is built between setting it and that update. A Player that builds one to answer a part read loses it ([Player API Conventions §9](../20_design/30_api_conventions.md#9-reading-a-part)). `PartState.update_flag` plays no part in it.

## 2. Usage

```cpp
// 1. Look up the index from the part name (-1 if not found)
int32_t part = ss_runtime_find_part_index(ctx, "body");
if (part < 0) { /* wrong part name, or no animation set up */ }

// 2. Color: multiply body by red (for the current animation)
ss_runtime_set_part_color_override(ctx, (uint32_t)part,
                                   /*blend_op*/ 1,          // 1 = Mul
                                   /*rgba*/ 0xFF0000FF,     // 0xRRGGBBAA
                                   /*priority*/ 1);         // 1 = HoldUntilNextAnimation

// 3. Cell: draw a different cell (names are passed as hashes)
uint32_t cellmap = ss_runtime_hash_string("Ringo");        // no .ssce
uint32_t cell    = ss_runtime_hash_string("effect3");
if (!ss_runtime_set_part_cell_override(ctx, (uint32_t)part, cellmap, cell, 1)) {
    // the cell was not found; the reason is in ss_runtime_get_last_error
}

// 4. Visibility: force hide, cascading to descendants
ss_runtime_set_part_visibility_override(ctx, (uint32_t)part,
                                        /*force*/ 1,        // 1 = force hide
                                        /*cascade*/ 1);     // cascade to descendants

// 5. Clear
ss_runtime_clear_part_color_override(ctx, (uint32_t)part);
ss_runtime_clear_all_part_overrides(ctx);                   // every part, every kind
```

The C-API specifies parts by **index** (`part_index`). In a public API that works by part name, cache in the Player the index looked up with `ss_runtime_find_part_index`. The index **does not change within the same ssab.**

The only C-API that operates on several parts at once is `ss_runtime_clear_all_part_overrides`. There is no special value meaning "all parts".

## 3. Each override

### 3-1. Color override

Affects **normal parts only**.

- **Blend type (`blend_op`)**: `0` = Mix / `1` = Mul / `2` = Add / `3` = Sub. These are **the same four types** as keyframe parts color ([Parts Color §2](../40_parts/10_parts_color.md#2-blend-types)). For `4` or above, `set` returns `false` (keyframe parts color treats such a value as Mix, but an override **rejects** it).
- **Single color and four corners**: `ss_runtime_set_part_color_override` uses one color for all four corners. For a gradient, use `ss_runtime_set_part_color_override_corners`. **The corner order is `lt, rt, lb, rb`** (top-left, top-right, bottom-left, bottom-right).
- **`rate` is always `1.0`.**
- **Do not apply alpha twice.** The alpha of the color you specify is multiplied by the part's opacity, the same as keyframe parts color ([Parts Color §3](../40_parts/10_parts_color.md#3-reading-the-parts-color)).
- **Colors are treated as 8-bit sRGB values.** The Brain does all color arithmetic in 8-bit sRGB integers and does no linear, gamma or PMA conversion. If your Player needs conversion to linear, do it in one place, in the shader input and output functions (`ss_input_texture()` / `ss_output_color()`), the same as for keyframe parts color. Building separate color handling just for overrides converts twice.

When you set a color override on **a part with no parts color in its keyframes**, that part is output as a part with parts color (`AttributePartColor` in `update_flag` is set, and FrameData carries parts color for it). If your Player draws differently depending on whether a part has parts color, expect this switch (if it does not draw differently, as in [Parts Color §5](../40_parts/10_parts_color.md#5-parts-without-a-parts-color), nothing is needed).

### 3-2. Cell override

Affects **normal parts and mask parts**.

Specify a cell as `(cellmap_name_hash, cell_name_hash)`. Make the hashes with `ss_runtime_hash_string()`. The cellmap name **does not include `.ssce`.**

- **The cell is looked up once, at `set`.** An unknown cellmap name or cell name gives **`false` at `set`** (there is no lookup every frame).
- **Passing `(0, 0)` clears the override**, and the part returns to the animation's cell.

Swapping the cell changes the following four things. The Player mainly sees (2) and (3).

| # | What changes | Effect |
|---|---|---|
| (1) | Shape (origin, size, source rectangle) | Reflected in the vertices |
| (2) | **`texture_hash`** | **Changes how DrawBatches are split** |
| (3) | UV | `ss_runtime_get_local_uvs` and similar return the UVs of the swapped-in cell |
| (4) | Cell reference | FrameData's `cell` points at the swapped-in cell |

If the swapped-in cell is on a different texture, the part goes into a different DrawBatch. The Brain's draw plan already reflects this, so the Player draws correctly **just by reading `draw_batches` as usual**. Do not optimize on the assumption that "the number of DrawBatches is constant".

For a part with no size (`SizeX` / `SizeY`) keys, the swapped-in cell's size becomes the part's size (the same as when a keyframe changes the cell).

### 3-3. Visibility override

- `force = 0` clears it and `force = 1` force-hides. `force = 2` (force show) is reserved and always returns `false`.
- **There is no priority mode.** It always beats keyframes, and it is **always cleared** when another animation is set up (the same behavior as `HoldUntilNextAnimation`).
- With `cascade != 0`, every descendant part is hidden too.

**Cascading to descendants happens only for hiding by override.** SpriteStudio's keyframe hide applies **to that part only** and does not reach its descendants. This difference is intended. If a Player builds its own cascade of "do not draw the children when the parent is hidden", it changes the meaning of keyframe hide. **`PartState.hide` says only whether that part itself is drawn.**

**Overrides do not reach an instance part's child animation.** The child animation is a separate Context, so the parent Context's overrides do not affect the parts inside the child. When you hide the instance part itself, not drawing its child animation is the Player's job. A hidden instance part emits no DrawBatch, so a Player that draws the child at the DrawBatch's position ([Instance Parts §5](../40_parts/50_instance.md#5-drawing-the-child)) draws nothing without further work.

## 4. Lifetime and `part_index`

The override tables are arrays indexed by `part_index`, held by the Context. `part_index` is stable **only within the same ssab**. When you switch to another ssab, look the indices up again.

| Trigger | Behavior |
|---|---|
| `ss_runtime_setup_animation` (another animation in the same ssab) | **All visibility overrides are cleared.** For color and cell, **only `Permanent` remains**; `OverwriteOnNextKeyframe` / `HoldUntilNextAnimation` are cleared. The remaining `Permanent` overrides are re-checked against the new part count (those out of range are discarded) |
| `ss_runtime_bind_resource` (binding another ssab) | **All cleared** (the parts are no longer the same) |
| `ss_runtime_reset` | All cleared |

When there are no overrides at all, the overwrite step is skipped entirely. A Player that does not use this feature pays no performance cost.

## 5. Common mistakes

### 5-1. Caching by frame number blocks overrides

Overrides **change the output without changing the frame number.** In a Player with the optimization "skip building when the frame number is the same as last time" (which pays off while paused, on still frames, and in low frame rate playback), `set` and `clear` do not update the vertices, and **the API looks as if it does not work.** The same happens in a Player that skips ticks when nothing moves ([Animation Mixing §1-6](./10_animation_mixing.md#1-6-skipping-ticks-when-nothing-moves)).

A Player that skips building frames must request a rebuild every time a call succeeds that changes what the next frame shows without moving the clock: an override `set` / `clear`, and also track, source, fade and crossfade calls, seeks, section changes and animation changes. Make the request one-shot, withdrawn by the next frame built. Left as a sticky flag, it keeps that character from ever skipping a tick again.

Player-side settings an engine typically writes every frame (how the character is placed, its visibility, opacity, color, the sub-frame switch and the like) must compare the new value with the old one and request a rebuild only when it changes. A request per write keeps that character from skipping ticks.

### 5-2. In `Direct` mode, FrameData has no `cells`

In `Direct` mode (the default), FrameData does not carry `cells` / `vertices` / `deforms` (they are passed in the SoA buffers). It does carry `parts_color`.

So **`FrameData.cells` does not tell you whether a cell override took effect** (it is easy to misread "not present = not working"). To check in `Direct` mode, look at the following.

- The UVs `ss_runtime_get_local_uvs()` returns (they become the swapped-in cell's UVs)
- `DrawBatch.texture_hash` (it changes if the swapped-in cell is on a different texture)

## 6. C-API reference

On failure, functions return `false` (or `-1`); get the reason with `ss_runtime_get_last_error(ctx)`. `priority_mode` is always `0` = `OverwriteOnNextKeyframe` / `1` = `HoldUntilNextAnimation` / `2` = `Permanent` (§1-3), and an out-of-range value gives `false`.

| Function | Arguments | Notes |
|---|---|---|
| `ss_runtime_find_part_index(ctx, part_name)` | — | `int32_t`. `-1` if not found or no animation is set up |
| `ss_runtime_hash_string(str)` | — | `uint32_t`. `0` for `NULL` or a string that is not UTF-8 |
| `ss_runtime_set_part_color_override(ctx, part, blend_op, rgba, priority_mode)` | `blend_op`: `0` = Mix `1` = Mul `2` = Add `3` = Sub<br>`rgba`: `0xRRGGBBAA` | Uses one color for all four corners. Normal parts only |
| `ss_runtime_set_part_color_override_corners(ctx, part, blend_op, rgba_ltrtlbrb, priority_mode)` | `rgba_ltrtlbrb`: `uint32_t` × 4 (**in `lt, rt, lb, rb` order**) | Specifies the four corners separately |
| `ss_runtime_clear_part_color_override(ctx, part)` | — | |
| `ss_runtime_set_part_cell_override(ctx, part, cellmap_name_hash, cell_name_hash, priority_mode)` | Hashes of the cellmap name and the cell name | `(0, 0)` clears. `false` if not found. Normal parts and mask parts only |
| `ss_runtime_clear_part_cell_override(ctx, part)` | — | |
| `ss_runtime_set_part_visibility_override(ctx, part, force, cascade)` | `force`: `0` = clear `1` = force hide `2` = reserved<br>`cascade != 0` cascades to descendants | No priority mode |
| `ss_runtime_clear_part_visibility_override(ctx, part)` | — | |
| `ss_runtime_clear_all_part_overrides(ctx)` | — | Clears every kind on every part |

---
[Back to contents](../README.md)
