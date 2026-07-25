> [日本語](./20_effect.ja.md) | **English**

# Effect Parts

You can obtain the simulation result of a particle effect instantly, based on absolute time (frame). This uses the **Slot API** provided by `libssruntime`.

## Using the Effect Slot
Calling `ss_effect_slot_step` returns the drawing data (vertices, UVs, colors, textures) of every particle on that frame as an **`EffectDrawPlan`**.

```cpp
// Inside the per-frame update loop
const ss_effect_event_info ev = ss_runtime_get_active_effect_event(runtime_ctx, p_idx);

// Run the simulation.
const ss_effect_step_result step = ss_effect_slot_step(
    slot.effect_slot, ev, f.frame_no, f.delta_seconds, fps, f.parent_looped,
    /*y_flip*/ true, /*vert_stride*/ 2);

if (step.visible) {
    auto plan = ss::runtime::GetEffectDrawPlan(step.draw_plan_buf);
    // → draw according to "Structure of EffectDrawPlan" below
}
```

> [!IMPORTANT]
> The `p_idx` above **must not be taken from `draw_order`.** The Brain excludes `Hide`-ed effects from
> `draw_order` / `DrawBatch` (see the notes in [Implementing Mask](./40_mask.md)), so an implementation that
> "walks `draw_order` and `step`s only the effects found in it" leaves **hidden independently-played effects
> un-`step`-ed and frozen in mid-air**. Instead, **enumerate every effect part of the pack from its metadata,
> give each part a persistent slot, and `step` them every frame**, using `step.visible` only to decide whether
> to draw (see [Independent Playback](#independent-playback) /
> [Instance Parts §Independent Playback](./10_instance.md#supporting-independent-playback)).

## Reference: the `y_flip` / `vert_stride` arguments of `ss_effect_slot_step`

- **`y_flip`**: Y-down engines (Godot 2D, pixi.js, and so on) pass `true`. SS flips the vertices, which are authored in a Y-up coordinate system, here at emit time. Y-up engines (Unity and others) pass `false`.
- **`vert_stride`**: the number of floats per vertex. 2D-native engines use `2` (XY); engines that require three-component vertices (Unity Mesh, Unreal `FProcMeshVertex`, and so on) use `3` (XYZ, z=0).

## Reference: structure of `EffectDrawPlan`

The `EffectDrawPlan` returned by `ss_effect_slot_step` contains a drawing command per emitter, plus the flat vertex buffers those commands point into.

| Field | Contents |
|---|---|
| `commands[]` | The drawing command per emitter (described below) |
| `vert_stride` | Matches the `vert_stride` argument you passed (2 = XY, 3 = XYZ) |
| `verts[]`    | Flat array of vertex coordinates (`total_quads * 4 * vert_stride` elements) |
| `uvs[]`      | Flat array of UV coordinates (`total_quads * 4 * 2` elements) |
| `colors[]`   | Flat array of vertex colors (RGBA normalized 0..1, `total_quads * 4 * 4` elements) |
| `indices[]`  | Triangle indices (**quad-local 0-based**, `total_quads * 6` elements) |

Each `EffectDrawCommand` carries the information needed to slice the flat buffers above:

| Field | Contents |
|---|---|
| `cellmap_hash`     | The key into the Player's cellmap-texture dictionary |
| `quad_count`       | The number of quads this command covers |
| `particle_offset`  | The index of the first quad (in quads) |
| `blend`            | The raw `EmitterState.blend_type` (`0=Mix`, `1=Add`) |

Compute the slice boundaries by taking `particle_offset` as a quad count and multiplying by each buffer's stride:

```cpp
const auto* plan = ss::runtime::GetEffectDrawPlan(step.draw_plan_buf);
const uint8_t stride = plan->vert_stride();

for (const auto* cmd : *plan->commands()) {
    const uint32_t v_off = cmd->particle_offset() * 4 * stride;
    const uint32_t u_off = cmd->particle_offset() * 4 * 2;
    const uint32_t c_off = cmd->particle_offset() * 4 * 4;
    const uint32_t i_off = cmd->particle_offset() * 6;

    ApplyMaterial(cmd->cellmap_hash(), cmd->blend());
    DrawTriangles(
        plan->verts()->data()   + v_off, cmd->quad_count() * 4 * stride,
        plan->uvs()->data()     + u_off, cmd->quad_count() * 4 * 2,
        plan->colors()->data()  + c_off, cmd->quad_count() * 4 * 4,
        plan->indices()->data() + i_off, cmd->quad_count() * 6
    );
}
```

`indices[]` is **quad-local 0-based** (pointing at 0..3 within each quad), so when drawing a single command you can pass the slice as-is and no base addition is required.

## 💡 Optimization: dynamic batching of Effect commands

Effects emit a separate `EffectDrawCommand` per particle emitter. Implementing this naively as one command = one draw call can inflate draw calls and allocations and hurt performance.

**When consecutive commands share the same `cellmap_hash` (texture) and `blend` (blend mode), merging them into a single vertex buffer and submitting them in one draw call (dynamic batching)** is strongly recommended.

One caveat when merging: each command's `indices[]` are relative indices pointing at `0..3` within each quad, so when copying them into the large merged index buffer you must **add "the total number of vertices merged so far" (`dest_quad_count * 4`) to each index value as you write it**.

## Independent Playback

Effects support "independent playback", advancing on their own time even while the parent is stopped. For the mechanism, see [Instance Parts §Independent Playback](./10_instance.md#supporting-independent-playback).

---
[Back to Table of Contents](../README.md)
