> [日本語](./60_effect.ja.md) | **English**

# Effect Parts

An effect part plays a particle effect. The Player keeps a slot for each effect part and calls `ss_effect_slot_step` every frame. The Brain computes the particles and returns the vertices, UVs and colors of every particle in that frame as an `EffectDrawPlan`. The Player only draws it.

Much of how they advance (advancing every part every frame, independent playback, resetting when playback restarts) is the same as for instance parts ([Instance Parts](./50_instance.md)).

## 1. Creating and advancing the slot

After loading an animation, find every effect part (`PartTypeEffect`) among the ssab's parts, and create a slot for each.

```cpp
// Setup: for each effect part
void* slot = ss_effect_slot_create(resource, effect_part->ref_effect_name_hash(), seed);
if (slot == nullptr) { /* the referenced effect is not in this ssab */ }
```

`seed` is the seed for the particles' random numbers. Give each part a different value (for example, the XOR of the part index and the effect name hash). The slot holds a reference to the resource, so you may call `ss_resource_destroy` before destroying the slot.

```cpp
// Every frame, after the parent's update, for every effect part
const ss_effect_event_info ev = ss_runtime_get_active_effect_event(ctx, part_index);

const ss_effect_step_result step = ss_effect_slot_step(
    slot, ev,
    parent_frame_no,       // the parent's playhead
    delta_seconds,         // elapsed time (seconds)
    fps,                   // the animation's frame rate (AnimationData.fps)
    parent_looped,         // the parent's ss_runtime_just_looped
    /*y_flip*/ true,       // §2
    /*vert_stride*/ 2,     // §2
    /*draw*/ part_drawn);  // §2: is the part in this frame's draw_order?

if (step.visible) {
    auto plan = ss::runtime::GetEffectDrawPlan(step.draw_plan_buf);
    // draw as in §3
}
```

- **The elapsed time is in seconds** (`ss_runtime_update` takes milliseconds).
- **Do not advance the effect's clock from `draw_order`.** Hidden effect parts are not in `draw_order`, so if you `step` while walking `draw_order`, an independently played effect freezes in mid-air while hidden. **Enumerate every effect part from the ssab and `step` every one of them every frame.** Use `step.visible` only to decide whether to draw. Step a hidden part with `draw = false` (§2): it keeps the clock moving and skips the particles nobody draws.
- The returned `EffectDrawPlan` buffer stays valid until the next `step` call on the same slot.
- When you replay the parent from the start with `play`, or when a slot starts an instance child that has effect parts, reset the slot to its initial state with `ss_effect_slot_reset` ([Instance Parts §6](./50_instance.md#6-restarting-playback)).
- Independent playback is handled the same as for instance parts ([Instance Parts §3](./50_instance.md#3-independent-playback)).

## 2. Arguments of `ss_effect_slot_step`

| Argument | Value |
| :--- | :--- |
| `y_flip` | `true` for Y-down engines (Godot's 2D, PixiJS and so on), `false` for Y-up engines (Unity and so on). SpriteStudio is authored in Y-up, so with `true` the particles' Y coordinates and rotation direction are returned flipped |
| `vert_stride` | The number of floats per vertex. `2` (XY) for 2D engines; `3` (XYZ, z = 0) for engines that require three-component vertices (Unity's Mesh, Unreal's FProcMeshVertex and so on) |
| `draw` | Whether the part is in this frame's `draw_order`, that is whether you draw it. With `false` the step moves only the effect's clock and transitions, builds no particles and returns not visible; the next step with `true` returns the same plan it would have had every step drawn. Knowing it needs this frame's `ss_runtime_get_frame_data` first, so a Player that steps before that passes `true` |

## 3. Drawing the `EffectDrawPlan`

An `EffectDrawPlan` holds a draw command per emitter, and the concatenated vertex buffers those commands point into.

| Field | Contents |
|---|---|
| `commands[]` | The draw command per emitter |
| `vert_stride` | The same value as the `vert_stride` of the call (2 = XY, 3 = XYZ) |
| `verts[]` | Vertex coordinates (`total_quads * 4 * vert_stride` values). In the effect part's local coordinates |
| `uvs[]` | UVs (`total_quads * 4 * 2` values). **Normalized** (0 to 1) |
| `colors[]` | Vertex colors (RGBA, 0 to 1, `total_quads * 4 * 4` values) |
| `indices[]` | Triangle indices (`total_quads * 6` values). **0-based within the command** |

| `EffectDrawCommand` | Contents |
|---|---|
| `cellmap_hash` | The texture key (the hash of the cell map name) |
| `quad_count` | The number of particles (quads) in this command |
| `particle_offset` | The position of this command's first quad (in quads) |
| `blend` | Blending with the destination (`0` = Mix, `1` = Add) |

For each command, slice each buffer starting at `particle_offset`.

```cpp
const auto* plan = ss::runtime::GetEffectDrawPlan(step.draw_plan_buf);
const uint8_t stride = plan->vert_stride();

for (const auto* cmd : *plan->commands()) {
    const uint32_t q  = cmd->particle_offset();
    const uint32_t qc = cmd->quad_count();

    apply_material(cmd->cellmap_hash(), cmd->blend());
    draw_triangles(
        plan->verts()->data()   + q * 4 * stride, qc * 4 * stride,
        plan->uvs()->data()     + q * 4 * 2,      qc * 4 * 2,
        plan->colors()->data()  + q * 4 * 4,      qc * 4 * 4,
        plan->indices()->data() + q * 6,          qc * 6);
}
```

`apply_material` / `draw_triangles` stand for the engine's drawing; they are not real functions.

- The indices are 0-based within the command (from `0` up to, but not including, `quad_count * 4`), so if you draw one command with one draw call, you can pass the slices as they are.
- The vertices are in the effect part's local coordinates. **Multiply them by the effect part's world matrix.**
- **Multiply all the particles by the effect part's opacity (`PartState.alpha`).** The colors in `EffectDrawPlan` are only the particles' colors; they do not include the effect part's own opacity (such as a fade-out of the part).
- Draw at the DrawBatch (`kind == Effect`) that comes at the effect part's position in `draw_order`.

## 4. Batching commands

An effect produces a separate command for each emitter. Drawing each command with its own draw call can slow things down by increasing draw calls and memory allocations.

**When consecutive commands share the same `cellmap_hash` and `blend`, we recommend merging them into one vertex buffer and drawing them in one call.** When merging, the indices are values within each command, so **add the number of vertices merged so far to each index** as you write it.

---
[Back to contents](../README.md)
