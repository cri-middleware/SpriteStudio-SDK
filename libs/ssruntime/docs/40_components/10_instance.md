> [日本語](./10_instance.ja.md) | **English**

# Instance Parts

An instance part references another animation as a component and plays it in sync with the parent's playback time. Using the **Slot API** provided by `libssruntime`, you can implement this without having to think about the complex synchronization logic.

## Management through the Slot API
The Player keeps a slot created with `ss_instance_slot_create()` per part, and calls `ss_instance_slot_step()` every frame.

```cpp
// Inside the per-frame update loop
const ss_event_instance_info info = ss_runtime_get_active_event_instance(runtime_ctx, p_idx);

// State update and synchronization
const ss_instance_step_result r = ss_instance_slot_step(
    state.instance_slot, info, child->runtime_ctx,
    parent_frame_no, delta_seconds, parent_looped);

if (r.transitioned) {
    // An instance switch (event occurrence) was detected.
    // Call the child Player's play() so that the start events and so on fire.
    child->play();
}

if (r.visible) {
    child->render(r.child_frame_no);
}
```

### Detecting the child finishing or looping

For an **independently-played child**, the child Context is advanced by time inside `step`, so **the ordinary playback-control APIs can be used on the child Context as-is**.

```cpp
// Has the independently-played child completed the specified number of loops?
if (ss_runtime_is_finished(child->runtime_ctx)) { /* ... */ }
```

`ss_runtime_is_finished` is a state, so take the edge and notify exactly as described in [Playback Control & Events](../30_core/20_playback_control.md). You can clear the latch at the point where you call `child->play()` on `r.transitioned`.

A **child synchronized to the parent frame (non-independent)**, on the other hand, is merely seeked deterministically from the parent's frame every frame and holds no playback state (`is_playing`, remaining loop count). Consequently `ss_runtime_is_finished(child->runtime_ctx)` is **always false** for a synchronized child. Once a synchronized child has finished the specified number of loops it **stays displayed at the final frame** (the same behavior as SpriteStudio 6). If you want to run processing triggered by the end itself, express it with a key on the parent animation.

`r.child_looped` is a pulse indicating that **an independently-played child completed one cycle** (always false for a synchronized child). If the child itself has further instance / effect parts, **pass this value to the grandchild's `step` as `parent_looped`**. Because an independently-played child runs on its own elapsed time rather than the parent's, "the parent looped" from the grandchild's point of view is this value, not the parent animation's `ss_runtime_is_looped`. If you forget to pass it, the grandchild's playback key is not re-armed when the child loops, and it will not play again from the second cycle onwards.

### 💡 Synchronizing and propagating the sub-frame playback setting
When the parent Player has sub-frame interpolation enabled (smooth animation playback using real-numbered frames), that setting (enabled / disabled) must also be propagated correctly to child Players (instance parts).
Neglecting this produces unnatural behavior where the parent's transforms and part movements are smooth while only the child Player's animation updates in choppy integer frames, breaking the synchronization.

- **When to synchronize**:
  1. When the parent Player's sub-frame enabled / disabled setting changes, recursively synchronize the equivalent setting (`set_sub_frame_enabled` and so on) to every child Player.
  2. When a new instance (child Player) is set up, also carry over the parent's current sub-frame setting as an initial parameter.

### Resolving external SSABs (External Instances)
An instance part may reference "a `.ssab` in another file (another animation pack)". At initialization the Player is responsible for checking the SSAB's `external_instances()` section, recursively loading the required external files, and binding them to the appropriate Context. For the loading and management procedure, see [Managing External Resources and Textures](./70_external_resources.md).

## Supporting Independent Playback

> This mechanism is shared with effect parts (see also [Effect Parts](./20_effect.md)).

Effects and instances have an "independent playback" setting, where the simulation continues on its own elapsed time regardless of the parent animation's playback speed or paused state.

- **Detection**: when `ss_effect_slot_is_independent()` or `ss_instance_slot_is_independent()` returns `true`, that part is in independent playback mode.
- **Keep updating**: even while the parent animation is paused, or when the frame is not moving because of a seek, these parts must keep advancing `delta_seconds` and calling `step`. Neglecting this produces unnatural behavior such as particles freezing in mid-air.
- **Do not drive `step` from `draw_order` (important)**: the Brain does not include `Hide`-ed parts in `draw_order` / `DrawBatch` (see the notes in [Implementing Mask](./40_mask.md)). Therefore, if you implement "walk `draw_order` and `step` only the Instance / Effect parts found in it", **an independently-played child that becomes hidden never gets `step`-ed and its clock freezes**, so when it is shown again it resumes late by exactly the time it was stopped (which violates SS7's independent-playback semantics). Decouple `step` (advancing the clock) from the draw order: **enumerate every Instance / Effect part from the pack's metadata, give each part a persistent slot, and `step` them all every frame without omission**. Use `draw_order` and the visibility decision (`step`'s return value `visible`) only to decide **whether to draw and in what order**.
- **Implicit playback keys are ignored at loop boundaries**: when no explicit playback key (`EventInstance` / `EventEffect`) exists on the current frame, SpriteStudio returns an implicit playback key (a **synthetic default**) - typically the case where no explicit key was placed on frame 0 at the head of the animation. That key is effectively equivalent to a frame-0 reset. In independent playback mode, **the playback state is not reset by this implicit key** even when the parent animation loops; the part keeps advancing on its own elapsed time. **Only explicitly placed `EventInstance` / `EventEffect` keys can start or reset independent playback** (implicit keys are excluded).
- **Re-crossing the same key does not reset playback (judged by the last playback key)**: in independent playback, each slot remembers exactly one "**key that last started playback**". When the parent loops and reaches an explicit key again, it is **ignored if it is the same key as remembered** (playback state is preserved) and **played, resetting the state, if it is a different key**. As a result, the combination of **independent + finite length + a single explicit key** becomes a "**one-shot**" across the whole parent loop (it is not replayed no matter how many cycles arrive at the same key, and likewise after it has disappeared at end of life). If you want it to appear periodically, choose either "turn off independent operation (synchronize to the parent frame)" or "make the effect resource an infinite loop". Persistent effects (flames, smoke, and other infinite loops) continue, while one-shots naturally settle as one-shots.
- These decisions are made automatically inside ssruntime, so the Player does not need to read `ev.is_synthetic_default` and branch on it - just act on the return values of `ss_*_slot_step` (`transitioned` and so on) as usual. (That return value effectively means "did we switch to a different key that should reset playback?")
- **Writing back the real-numbered (fractional) frame exactly**:
  The `step.child_frame_no` returned by `ss_instance_slot_step` and `ss_effect_slot_step` is an exact real number (float) that includes the sub-frame progress. When you write it back to update the frame of the child Player (an instance part and so on), **never round it to an integer with `floor` or clamp it.**
  If you write a rounded value back as state, the next frame's `delta` addition always starts from the rounded integer frame. In environments where "the time advanced per tick is less than one frame", the tiny progress is then truncated every frame, which causes a bug where the animation freezes completely. Round the displayed frame only immediately before drawing, and always keep the frame-progress value held as state as a "raw real number".

---
[Back to Table of Contents](../README.md)
