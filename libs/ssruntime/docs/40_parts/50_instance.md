> [日本語](./50_instance.ja.md) | **English**

# Instance Parts

An instance part plays another animation (a child animation) as a component. The Player keeps a child Context and a slot for each instance part, and advances the child every frame with `ss_instance_slot_step`. The slot decides when to start the child, which frame to show, and how to handle loops and independent playback. The Player prepares the child, draws the child, and does the work of the host that owns the child (notifications and so on).

## 1. Preparing the child

After loading an animation, find every instance part (`PartTypeInstance`) among the ssab's parts, and prepare the following for each.

1. **Find the referenced ssab.** `PartTypeInstance`'s `ref_anime_pack` (`ref_anime_pack_hash`) is the referenced ssab, and `ref_anime_name` (`ref_anime_name_hash`) is the animation in it. The reference is either the part's own ssab or another ssab ([External Resources and Textures §4](../50_features/30_external_resources.md#4-referencing-external-ssabs)).
2. **Create the child Context.** Bind the referenced ssab's resource, and set up the child animation with `ss_runtime_setup_animation_by_hash(child_ctx, ref_anime_name_hash)`. Do not play it yet: it starts when the slot meets its first key ([§2](#2-advancing-the-child-with-the-slot)).
3. **Create the slot** with `ss_instance_slot_create()`. The slot holds no resource; it works on the child Context it receives every frame.

The child is also a Player, so if the child animation has instance parts or effect parts, prepare them the same way inside the child.

## 2. Advancing the child with the slot

```cpp
// Every frame, after the parent's update, for every instance part
for (InstanceChild& c : instance_children) {
    const ss_event_instance_info info = ss_runtime_get_active_event_instance(ctx, c.part_index);

    const ss_instance_step_result r = ss_instance_slot_step(
        c.slot, info, c.child_ctx,
        parent_frame_no,        // the parent's playhead
        delta_seconds,          // elapsed time (seconds)
        parent_looped);         // the parent's ss_runtime_just_looped

    if (r.transitioned) {
        // The slot configured the child for the new key (section, direction, loop count);
        // starting it is the host's. play() snaps it to the start of that section.
        ss_runtime_play(c.child_ctx);
        c.child->on_started();   // the host work that comes with the child starting (notifications and so on)
    }
    c.visible = r.visible;
    c.frame_no = r.child_frame_no;   // the transition tick included
    c.looped = r.child_looped;
}
```

`InstanceChild` is the struct the Player keeps for each instance part (the child Context, the slot and the child Player).

**The elapsed time of `ss_instance_slot_step` is in seconds.** Note that this differs from `ss_runtime_update`, which takes milliseconds.

| Return value | Meaning |
| :--- | :--- |
| `visible` | Whether to draw the child in this frame |
| `transitioned` | The slot met a new key and configured the child for it. **The slot does not start the child: call `ss_runtime_play` on it**, or an independently played child never advances |
| `child_frame_no` | The frame number to draw the child at. **It includes a fraction. Use it without rounding** ([Playback Control §6-2](../30_core/20_playback_control.md#6-2-do-not-discard-the-fraction)). **Use it on a `transitioned` tick too.** The slot has already put the child at its section's first frame for the direction, which is where `ss_runtime_play` snaps it, and a synced child's value follows the parent. Reading the child's frame after the `play` instead would put a synced child back at its section start for that tick |
| `child_looped` | An independently played child completed one cycle (§4) |
| `independent` | Whether the child plays independently |

**Do not advance the child's clock from `draw_order`.** Hidden instance parts are not in `draw_order`, so if you `step` while walking `draw_order`, the clock of an independently played child stops while it is hidden, and when it is shown again it lags by the time it was stopped. **Enumerate every instance part from the ssab, keep a slot per part, and `step` every one of them every frame.** Use `draw_order` and `visible` only to decide whether to draw and in what order.

## 3. Independent playback

Instance parts and effect parts have an "independent playback" setting, which advances on the part's own elapsed time regardless of the parent's playback speed or pause. `ss_instance_slot_is_independent` / `ss_effect_slot_is_independent` (or `independent` in the step's return value) tells you whether a part plays independently.

- **Keep calling `step` even while the parent is paused, and even when a seek leaves the parent's frame unchanged.** Otherwise the result looks unnatural, for example particles freezing in mid-air.
- Only **explicitly placed playback keys** (`EventInstance` / `EventEffect`) can start or restart independent playback. When the current frame has no explicit key (for example, when none was placed at the start of the animation), the Brain uses an implicit key that starts from frame 0, but an independently played child does not restart on this implicit key, even when the parent loops.
- **Passing the same key again does not restart playback.** The slot remembers the key that last started playback; it ignores that key when a parent loop brings it back, and restarts only for a different key. So a setup of independent playback, a finite length and a single explicit key plays **only once**, however many times the parent loops (the same holds after it disappears at the end of its life). To make it appear periodically, either turn off independent playback so that it follows the parent's frame, or make the effect itself loop infinitely.

All of these decisions are made inside the slot. The Player only needs to act on `step`'s return values.

An independently played child does not follow the parent's reverse playback or seeking ([Limitations §3](../10_introduction/20_limitations.md#3-independent-playback-parts-do-not-follow-reverse-playback-or-seeking)).

## 4. What to pass to the child

When the child advances its own instance parts and effect parts (when it advances the grandchildren), pass it the following from the parent.

| Value | How to pass it |
| :--- | :--- |
| The parent's playhead | Pass `r.child_frame_no` as the parent's playhead from the child's point of view |
| Whether the parent looped | Pass `r.child_looped`. An independently played child runs on its own elapsed time, not the parent's, so for the grandchild "the parent looped" is this value, not the parent animation's `ss_runtime_just_looped`. If you forget to pass it, the grandchild's playback key is not restarted when the child loops, and it does not play from the second cycle on |
| Elapsed time | The parent's elapsed time multiplied by the absolute value of the instance key's speed (`speed` in `ss_event_instance_info`). The child runs at the key's speed, so the grandchildren run at that speed too. Passing the parent's value unchanged makes the grandchildren alone run at a different rate under a key whose speed is not 1. The child's own `ss_instance_slot_step` still takes the parent's elapsed time unchanged (the slot applies the speed itself) |
| Sub-frame setting | Pass the child the same setting as the parent. Pass it both when the parent's setting changes and when the child is created. Otherwise the parent moves smoothly while only the child moves in whole frames |
| Mask settings | Pass the composed settings ([Masks §2-6](./40_mask.md#2-6-composing-mask-settings-through-an-instance)) |

## 5. Drawing the child

Draw the child at the DrawBatch (`kind == Instance`) that comes at the instance part's position in `draw_order`.

- Build the child's FrameData by calling `ss_runtime_get_frame_data(child_ctx, frame)` on the child Context. `frame` is `r.child_frame_no` (truncated if you do not use sub-frames).
- The child's world matrices are values within the child animation. **Multiply by the instance part's world matrix as the root transform of the whole child.**
- **Multiply the whole child by the instance part's opacity (`PartState.alpha`).** The child is a separate Context, so the Brain does not pass the parent's opacity to the child. **Do not apply the instance part's own part color to the child.** Opacity is the only thing passed down.
- Do not draw the child when `visible` is false, or when the instance part is hidden and has no DrawBatch.
- Part overrides do not reach inside the child ([Part Overrides §3-3](../50_features/20_part_overrides.md#3-3-visibility-override)).

## 6. Restarting playback

When you replay the parent from the start with `play`, **reset every slot to its initial state with `ss_instance_slot_reset` / `ss_effect_slot_reset`.** Reset the slots inside children recursively too.

A slot remembers the key that last started playback (§3). If you do not reset it, the first key after the restart is treated as "the same key as last time", and independently played children and effects do not start.

## 7. The child ending and looping

For an **independently played child**, the slot advances the child Context by time, so you can use the ordinary playback API on the child Context as is.

```cpp
if (ss_runtime_is_finished(c.child_ctx)) { /* the child has finished its configured number of loops */ }
```

`ss_runtime_is_finished` is a state, so detect its rising edge and notify as in [Playback Control §7](../30_core/20_playback_control.md#7-detecting-loops-and-the-end). Reset the previous value used for edge detection (the equivalent of `prev_finished` in Playback Control §7-1) on `r.transitioned`.

**A child that follows the parent's frame (a child without independent playback)** is only seeked from the parent's frame every frame, and holds no playback state (`is_playing` or the remaining loops). So `ss_runtime_is_finished` is **always false** for it. Once such a child finishes its configured number of loops, it **stays displayed at its last frame.** If you want to run something when the child ends, express it with a key in the parent animation.

---
[Back to contents](../README.md)
