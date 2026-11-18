> [日本語](./10_animation_mixing.ja.md) | **English**

# Animation Mixing

Animation mixing plays several animations at the same time. You stack animations on **tracks** to lay a limp over a walk, turn the head from a tracking value, or switch smoothly from idle to attack. Mixing is a feature of the Brain's state and the C-API only; the ssab does not need to change. The names it takes in a Player's public API are in [Player API Conventions §3-2](../20_design/30_api_conventions.md#3-2-animation-mixing).

## 1. Model

A Context holds a stack of tracks. The tracks are layered in ascending order of number onto the **setup pose** (the rest pose the ssae defines).

```
result = setup pose
track 0  →  layer on   (the main animation)
track 1  →  layer on   (an animation layered on top)
track 2  →  layer on   (one more)
```

There are two values to adjust, and **which one you use matters**.

| | Role | Unit |
|---|---|---|
| **`alpha`** | How much the track is layered onto the result below it. **This is the strength control** | per track |
| **`weight`** | The ratio between several animations on **the same track** | per source |

A track's `alpha` always works as you expect. A source's `weight`, on its own, does not. If only one animation keys an attribute, there is nothing to compare it with, so the attribute takes that animation's value however small its `weight` is. **Use `weight` only when you really mix animations with each other (walk and run), and use `alpha` in every other case.**

### 1-1. Track 0 is special

Track 0 carries the **structure**. The discrete attributes (cell, parts color, shader, flip, hide, draw priority, texture), the default event firing and the playback state (`is_playing`, `is_finished`, the frame) all come from track 0's main animation, the **primary**. `ss_runtime_setup_animation` is the operation "put this animation on track 0". Track 0 cannot be emptied; to remove it, fade its `alpha`.

Only a change of track 0's animation changes the primary: `ss_runtime_setup_animation`, `ss_runtime_set_track_animation` on track 0, and `ss_runtime_crossfade` on track 0, which hands the primary over **when the fade begins** (§4). `ss_runtime_get_current_animation_index` returns which animation it plays, as an index into the ssab's `animations()`.

### 1-2. A track affects only the attributes it keys

For an attribute that a track's animation has no keys for, the result from below passes through unchanged. Layering an animation keyed only on the head does not affect the legs, **even without a mask**. Put the other way, **a layered animation reaches exactly the range that was authored**. To affect only part of the body, key only that part.

### 1-3. Within one ssab only

Only animations inside one resource (one ssab) can be stacked.

### 1-4. One clock for the whole character

Every track runs on one clock, the character's. An animation laid on a track plays from the moment it is laid, and `ss_runtime_update` advances every track and every fade together. The playback operations act on that clock ([Playback Control §4](../30_core/20_playback_control.md#4-playback-state)):

| Call | Effect on the stack |
|---|---|
| `ss_runtime_play` | Rewinds **track 0** to the start of its section and starts the clock. The other tracks keep their playheads |
| `ss_runtime_pause` / `ss_runtime_resume` | Hold / release every track and every fade |
| `ss_runtime_stop` | Stops every track and every fade where they stand. Only `play` starts them again |

- **Track 0 finishing does not stop the clock.** When a finite loop count runs out on track 0, the other tracks and the fades carry on. `is_playing`, `is_pausing` and `is_finished` describe track 0, so all three report the finish as usual; a `pause` after it still holds the other tracks, and `resume` releases them.
- Fades advance with the clock: while it is held or stopped, a fade does not progress.
- **The clock has its own pair of flags.** `ss_runtime_is_clock_playing` is `true` from `play` until `stop` or the next setup, and `ss_runtime_is_clock_pausing` while `pause` holds it; unlike `is_playing` and `is_pausing`, both read on after track 0 finishes. A Player that stops ticking a character with nothing left to move (§1-6) reads these rather than keeping its own copy of the transport calls, which goes wrong the moment it misses one: a setup stops the clock too.
- **Whether anything beyond track 0 still moves.** `ss_runtime_is_stack_in_motion` is `true` while a fade runs on any track, or while a source other than the primary plays on a track whose `alpha` is above 0, with the track's speed and the source's weight both above 0. A cleared or finished track, one at `alpha` 0, or one at speed 0 does not count. A track at speed 0 moved by hand with `ss_runtime_set_track_frame` does not count either, while a track at a speed above 0 counts even when it is moved by hand. The outgoing side a cancelled crossfade left behind does count. It does not read the clock. How to use it is in §1-6.

### 1-5. A track's playback settings

The speed, the frame-rate override, frame skip and the playback direction belong to the **track**, not to the animation on it. A setting reaches every animation on the track now, both sides of a crossfade included, and every animation laid on it later, so a crossfade or `ss_runtime_set_track_animation` does not reset it.

| Setting | Any track | Track 0 only |
|---|---|---|
| Speed | `ss_runtime_set_track_speed` | `ss_runtime_set_speed_scale` |
| Frame-rate override | `ss_runtime_set_track_frame_rate` | `ss_runtime_set_frame_rate` |
| Frame skip | `ss_runtime_set_track_skip_frames` | `ss_runtime_set_skip_frames` |
| Direction and style | `ss_runtime_set_track_playback_direction` | `ss_runtime_set_playback_direction` |
| Loop count | `ss_runtime_set_track_loop_count` | `ss_runtime_set_loop_count` |

- The **loop count** and the **playback section** are the exception: they belong to the animation, because they count and number its frames. Laying an animation starts it over at the whole section, looping indefinitely, so state the count after laying it. A Player that keeps the host's loop count re-applies it after every change of track 0's animation, a crossfade included, as it does after `ss_runtime_setup_animation`.
- Track 0 keeps its settings across `ss_runtime_setup_animation`; `ss_runtime_reset` clears them. The other tracks lose theirs with the rest of the stack when `ss_runtime_setup_animation` collapses it.
- A setting may be given to a track with nothing on it yet; the track is created for it (which calls create a track is in §3). A loop count needs an animation on the track.
- A track above 0 reports its own end and loop: `ss_runtime_is_track_finished` and `ss_runtime_track_just_looped`, as `is_finished` and `just_looped` do for track 0. A finished track **holds its last frame** and keeps mixing it in; the host decides when to fade or clear it. Only track 0 finishing ends the character's playback.

### 1-6. Skipping ticks when nothing moves

A Player may skip a tick, calling neither `ss_runtime_update` nor `ss_runtime_get_frame_data` and drawing the frame it built before, only when nothing moves. A Player that skips ticks must not skip one in any of these cases:

- Track 0 is playing and the clock is not held (`ss_runtime_is_playing` is `true` and `ss_runtime_is_clock_pausing` is `false`)
- The clock runs unheld (`ss_runtime_is_clock_playing` is `true` and `ss_runtime_is_clock_pausing` is `false`) and the stack is in motion (`ss_runtime_is_stack_in_motion` is `true`)
- A rebuild has been requested ([Part Overrides §5-1](./20_part_overrides.md#5-1-caching-by-frame-number-blocks-overrides))

While the clock is held (paused), a tick may be skipped unless a rebuild has been requested. A character with an independent instance or effect child ([Instance Parts §3](../40_parts/50_instance.md#3-independent-playback)), however, must keep ticking while paused.

Decide whether to run the tick **before** `ss_runtime_update`. The update a fade ends on clears the fade, so reading `ss_runtime_is_stack_in_motion` after the update leaves the last frame of the fade unbuilt. Read these every time instead of keeping your own copy of the tracks and sources you laid: the runtime releases some on its own, when a crossfade completes.

What a skipped tick reports is in [Events §3](../30_core/30_events.md#3-which-frames-are-reported).

## 2. Playing a single animation

Without mixing, a Context plays one animation on track 0, as in [Playback Control](../30_core/20_playback_control.md).

```cpp
ss_runtime_setup_animation(ctx, "walk");   // = put it on track 0
ss_runtime_update(ctx, delta_ms);
ss_runtime_get_frame_data(ctx, frame_no, &fd, &fd_len);
```

## 3. Layering

```cpp
ss_runtime_setup_animation(ctx, "idle");            // track 0: the main performance

ss_runtime_set_track_animation(ctx, 1, "look_around");
ss_runtime_set_track_alpha(ctx, 1, 0.4f);           // layer it at 40%
```

Tracks are created as needed. The calls that lay an animation (`ss_runtime_set_track_animation`, `ss_runtime_crossfade`, `ss_runtime_add_track_source`) and the settings for speed, frame-rate override, frame skip, playback direction, sync and the outgoing event threshold create a track that does not exist yet (laying on track 3 also creates tracks 1 and 2, empty). `ss_runtime_set_track_alpha`, `ss_runtime_set_track_blend` and `ss_runtime_fade_track` do not create a track and return `false` for one that does not exist, so create the track before calling them. Until an animation has been set up with `ss_runtime_setup_animation`, nothing can be laid on any track, track 0 included, and no track above 0 can be created. `alpha` is not clamped to 0–1. Above 1 it extrapolates, which layering sometimes wants.

`ss_runtime_set_track_animation` replaces what the track carries. Every source on the track is released, including those added with `ss_runtime_add_track_source`, and any fade in flight on it is cancelled, an `alpha` fade included. A cancelled `alpha` stops at its current value.

### 3-1. Additive tracks

A `SS_RUNTIME_TRACK_BLEND_ADDITIVE` track adds its **difference from the setup pose** instead of moving toward the track's pose. The animation below stays visible.

```cpp
ss_runtime_set_track_animation(ctx, 1, "lean_left");
ss_runtime_set_track_blend(ctx, 1, SS_RUNTIME_TRACK_BLEND_ADDITIVE);
ss_runtime_set_track_alpha(ctx, 1, 0.6f);           // the lean at 60%, on top of the walk
```

Use this for an axis driven by a continuous value. Author the axis animation so that **its middle frame matches the setup pose**, then play it, or set its speed to 0 and move its frame directly. An animation laid on a track starts playing, so unless its speed is 0 the next `update` advances it from the frame you set.

```cpp
// move the frame from a tracking value instead of playing it (an axis of frames 0 to 60, yaw from -1 to 1)
ss_runtime_set_track_speed(ctx, 1, 0.0f);
ss_runtime_set_track_frame(ctx, 1, 30.0f + yaw * 30.0f);
```

## 4. Switching

`ss_runtime_crossfade` fades a track from the animation it is playing now to a new animation. There are no handles to manage in pairs and nothing to clean up.

```cpp
ss_runtime_crossfade(ctx, 0, "run", 300.0f);   // 300ms, the same unit as update's elapsed time

ss_runtime_update(ctx, delta_ms);
float progress = ss_runtime_get_track_fade_progress(ctx, 0);   // 0 to 1; NaN when not fading
```

The incoming animation becomes the track's main animation **from the call on**: `ss_runtime_get_track_main_source` returns it straight after the call. On track 0 it is the primary from then on (§1-1), so its cell and other discrete attributes show from the first frame of the fade, and the playback state and the frame are its own. An outgoing animation running out during the fade does not finish the character. The incoming animation plays with the track's settings (§1-5).

Everything already on the track fades out together as the **outgoing side**: the sources keep their proportions to one another, scaled to a share of `1 - t` of the track while the incoming animation has `t`. When the fade ends, every outgoing source is released.

| Situation | Behavior |
|---|---|
| `duration_ms <= 0` / `NaN` / an animation name that does not exist | Returns `false`. The reason is in `ss_runtime_get_last_error` |
| `crossfade` on a track that is playing nothing | Starts playing at once, without a fade |
| `crossfade` again on a track that is fading | The running fade is taken over **as it stands**: both its sides join the new outgoing side at their current weights, so the pose does not jump |
| `crossfade` on a track carrying several sources | All of them fade out together |
| `crossfade` on a track running an `alpha` fade | Replaces the `alpha` fade. `alpha` stops at its current value |
| You want to stop a fade | `ss_runtime_cancel_track_fade`. `alpha` and `weight` stop at their current values; what is left fades out with the next `crossfade` |

### 4-1. Returning to the setup pose

The bottom of the stack is the setup pose, so fading a track out returns it straight to the setup pose.

```cpp
ss_runtime_fade_track(ctx, 0, 0.0f, 200.0f);   // to the setup pose over 200ms
```

A track has a single fade, and an `alpha` fade and a crossfade share it. `ss_runtime_fade_track` on a track that is crossfading replaces the crossfade: as with `ss_runtime_cancel_track_fade`, the weights of both sides stop at their current values, and what is left fades out with the next `crossfade`. A `crossfade` during an `alpha` fade behaves as the table in §4 says.

## 5. Mixing animations with each other

To mix by ratio (combining walk and run, for example), put the animations on the same track and set `weight`.

```cpp
ss_runtime_setup_animation(ctx, "walk");
uint32_t walk = ss_runtime_get_track_main_source(ctx, 0);
uint32_t run  = ss_runtime_add_track_source(ctx, 0, "run", 0.0f);

// walk 70% / run 30%
ss_runtime_set_source_weight(ctx, walk, 0.7f);
ss_runtime_set_source_weight(ctx, run, 0.3f);
ss_runtime_set_source_frame(ctx, run, 0.0f);
```

`ss_runtime_get_track_main_source` returns the handle of the animation that `ss_runtime_set_track_animation`, `ss_runtime_crossfade` (or `ss_runtime_setup_animation`) put on the track. The first source `ss_runtime_add_track_source` adds to a track with nothing on it also becomes the track's main source. `weight` starts at 1.

`weight` is a ratio, not an absolute value (`(0.7, 0.3)` and `(7, 3)` give the same result). Each source has its own playhead, and `ss_runtime_update` advances them all. **Sources of different lengths do not stay in phase.** To start one in phase with track 0, use track sync (§5-1); to keep them aligned, seek them in the Player.

### 5-1. Starting in phase with track 0

An animation laid on a track starts from the head of its section. When a run is laid over a walk that is half-way through its stride, the feet disagree. Set the track's sync to `SS_RUNTIME_TRACK_SYNC_START` and what is laid on it next starts at the **phase** track 0's animation stands at: how far through its section it is, as a fraction from `0.0` at the start towards `1.0` at the end. Animations of different lengths line up by proportion.

```cpp
ss_runtime_set_track_sync(ctx, 0, SS_RUNTIME_TRACK_SYNC_START);
ss_runtime_crossfade(ctx, 0, "run", 300.0f);   // run starts at the walk's phase
```

| | `SS_RUNTIME_TRACK_SYNC_FREE` (default) | `SS_RUNTIME_TRACK_SYNC_START` |
|---|---|---|
| Where a laid animation starts | The head of its section | The same fraction of its section as track 0's animation |
| Suited to | One-off motions: a blink, a mouth shape, an attack | Cyclic motions laid over one another: walk and run |

- It applies to everything laid on the track afterwards: `ss_runtime_set_track_animation`, `ss_runtime_crossfade` (on track 0 too, where the incoming animation starts at the outgoing one's phase) and `ss_runtime_add_track_source`. **Set it before laying.** A track with nothing on it yet is created for it.
- The reference is track 0's main animation, which a crossfade on track 0 hands over when it begins. The incoming animation is laid before the handover, so it starts at the outgoing one's phase.
- The phase is read by position in the section, not by direction: a backward or ping-pong return leg starts the other animation where it stands.
- It aligns once, when the animation is laid. Both then run on the same clock, so animations of the same length and speed stay in phase; different lengths or speeds drift apart again.
- Looping is unchanged: an animation started part-way through wraps at its end as usual.
- Like a track's alpha and blend, it belongs to the track: `ss_runtime_setup_animation` collapses the stack and every track starts over at `FREE`.

## 6. Events

By default, only the primary (track 0's main animation) fires events. A source added to track 0 with `ss_runtime_add_track_source` does not fire either, like any other source. To make another source fire too, lower its threshold to its `weight` or below. Events are reported from sources where `weight >= threshold`. The default threshold is `+∞`, which no weight reaches, so the lone source of another track does not fire at weight 1 either. The threshold belongs to the source, so the new source that `ss_runtime_crossfade` or `ss_runtime_set_track_animation` lays on a track above 0 starts at the default `+∞` again. As with the loop count in §1-5, lower it again after every change of animation.

```cpp
ss_runtime_set_source_event_threshold(ctx, run, 0.5f);   // fires when run's weight is 0.5 or more
```

The outgoing side of a crossfade is the exception: its sources fire while the side's share of the track, `1 - t`, is at least the **track's** outgoing threshold, however many sources it holds, and their own thresholds are not consulted while they fade. The default is `+∞`, so the outgoing side falls silent the moment the fade begins, which is when the incoming animation starts firing on track 0. At `0.5` it fires through the first half of the fade. In the update that completes the fade the outgoing side has already been released, so it does not fire even at `0`. It belongs to the track and holds for every crossfade on it; track 0 keeps it across `ss_runtime_setup_animation`.

```cpp
ss_runtime_set_track_outgoing_event_threshold(ctx, 0, 0.5f);   // the outgoing side fires for the first half
```

Each event carries the handle of the source that fired it, so you can tell which source an event came from. How to read events is in [Events §4](../30_core/30_events.md#4-when-several-animations-are-layered).

```cpp
int n = ss_runtime_get_passed_event_count(ctx);
for (int i = 0; i < n; ++i) {
    int32_t  event_idx = ss_runtime_get_passed_event_index(ctx, i);
    uint32_t source    = ss_runtime_get_passed_event_source_handle(ctx, i);
}
```

## 7. C-API reference

Get the reason for a failure with `ss_runtime_get_last_error(ctx)`. Functions that return `float` use `NaN` to signal failure so that a valid `0.0` can be told apart from "that track or source does not exist". Test with `isnan` in C and `Number.isNaN` in JavaScript. The integer reads use a value the setting never reads back as.

### 7-1. Tracks

| Function | Returns | On failure |
|---|---|---|
| `ss_runtime_set_track_animation(ctx, track, name)` | `bool` | `false` |
| `ss_runtime_set_track_animation_by_hash(ctx, track, hash)` | `bool` | `false` |
| `ss_runtime_set_track_alpha(ctx, track, alpha)` | `bool` | `false` |
| `ss_runtime_get_track_alpha(ctx, track)` | `float` | `NaN` |
| `ss_runtime_set_track_blend(ctx, track, blend)` | `bool` | `false` |
| `ss_runtime_get_track_blend(ctx, track)` | `uint32_t` (`0` = Mix / `1` = Additive) | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_set_track_sync(ctx, track, sync)` | `bool` | `false` |
| `ss_runtime_get_track_sync(ctx, track)` | `uint32_t` (`0` = Free / `1` = Start) | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_get_track_count(ctx)` | `uint32_t` | `0` |
| `ss_runtime_clear_track(ctx, track)` | `bool` | `false` (track 0 cannot be emptied) |
| `ss_runtime_set_track_frame(ctx, track, frame)` | `bool` | `false` |
| `ss_runtime_get_track_frame(ctx, track)` | `float` | `NaN` |
| `ss_runtime_set_track_speed(ctx, track, speed)` | `bool` | `false` |
| `ss_runtime_get_track_speed(ctx, track)` | `float` | `NaN` |
| `ss_runtime_set_track_frame_rate(ctx, track, fps)` | `bool` | `false` |
| `ss_runtime_get_track_frame_rate(ctx, track)` | `int32_t` (`0` with no animation and no override) | `-1` |
| `ss_runtime_set_track_skip_frames(ctx, track, skip)` | `bool` | `false` |
| `ss_runtime_get_track_skip_frames(ctx, track)` | `bool` | `false` |
| `ss_runtime_set_track_playback_direction(ctx, track, direction, style)` | `bool` | `false` |
| `ss_runtime_get_track_playback_direction(ctx, track)` | `int32_t` (`0` = forward / `1` = backward) | `-1` |
| `ss_runtime_get_track_playback_style(ctx, track)` | `int32_t` (`0` = normal / `1` = ping-pong) | `-1` |
| `ss_runtime_set_track_loop_count(ctx, track, count)` | `bool` | `false` (no animation on the track) |
| `ss_runtime_get_track_loop_count(ctx, track)` | `int32_t` (`-1` = indefinitely) | `0` |
| `ss_runtime_is_track_finished(ctx, track)` | `bool` | `false` |
| `ss_runtime_track_just_looped(ctx, track)` | `bool` | `false` |
| `ss_runtime_set_track_outgoing_event_threshold(ctx, track, threshold)` | `bool` | `false` |
| `ss_runtime_get_track_outgoing_event_threshold(ctx, track)` | `float` | `NaN` |
| `ss_runtime_get_current_animation_index(ctx)` | `int32_t` (track 0's main animation) | `-1` (nothing set up) |

### 7-2. Switching

| Function | Returns | On failure |
|---|---|---|
| `ss_runtime_crossfade(ctx, track, name, duration_ms)` | `bool` | `false` |
| `ss_runtime_crossfade_by_hash(ctx, track, hash, duration_ms)` | `bool` | `false` |
| `ss_runtime_fade_track(ctx, track, target_alpha, duration_ms)` | `bool` | `false` |
| `ss_runtime_get_track_fade_progress(ctx, track)` | `float` (0 to 1) | `NaN` (not fading) |
| `ss_runtime_cancel_track_fade(ctx, track)` | `bool` | `false` |

### 7-3. Several sources on one track

| Function | Returns | On failure |
|---|---|---|
| `ss_runtime_get_track_main_source(ctx, track)` | `uint32_t` handle | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_add_track_source(ctx, track, name, weight)` | `uint32_t` handle | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_add_track_source_by_hash(ctx, track, hash, weight)` | `uint32_t` handle | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_set_source_weight(ctx, handle, weight)` | `bool` | `false` |
| `ss_runtime_get_source_weight(ctx, handle)` | `float` | `NaN` |
| `ss_runtime_set_source_frame(ctx, handle, frame)` | `bool` | `false` |
| `ss_runtime_get_source_frame(ctx, handle)` | `float` | `NaN` |
| `ss_runtime_get_source_animation_index(ctx, handle)` | `int32_t` | `-1` |
| `ss_runtime_set_source_event_threshold(ctx, handle, threshold)` | `bool` | `false` |
| `ss_runtime_get_source_event_threshold(ctx, handle)` | `float` | `NaN` |

## 8. WebAssembly

The WebAssembly bindings map one to one onto the C-API. Failures are thrown as exceptions. A read with no value is `undefined` whatever its type (`float`, integer, boolean or handle); the C-API's `NaN`, `-1` and `SS_RUNTIME_INVALID_HANDLE` are not used.

```typescript
ctx.setup_animation("idle");

ctx.set_track_animation(1, "look_around");
ctx.set_track_blend(1, 1);          // 1 = Additive
ctx.set_track_alpha(1, 0.4);

ctx.set_track_sync(0, 1);           // 1 = Start
ctx.crossfade(0, "run", 300.0);
const progress = ctx.get_track_fade_progress(0);   // number | undefined

const run = ctx.add_track_source(0, "run", 0.3);
ctx.set_source_weight(run, 0.5);

ctx.set_track_animation(2, "attack");
ctx.set_track_loop_count(2, 1);     // once, then hold the last frame
const done = ctx.is_track_finished(2);
```

## 9. Constraints

- **Discrete attributes come from track 0's main animation**, and switch to the incoming one when a crossfade on track 0 begins. Cell, parts color, shader, flip, hide, draw priority and texture cannot be mixed, so a layered track cannot swap a cell. To swap a cell, use [Part Overrides](./20_part_overrides.md).
- **Assign the same cell to the same part in every animation you stack.** The cell's size and origin come from track 0, so a different cell shifts the size and origin and breaks the pose. Animations within the same ssae normally do not have this problem.
- **Parts color follows track 0.** For a part with parts color, that color and its alpha take precedence. The mixed opacity reaches the vertex color only for parts without parts color. This is not a problem in the usual case of mixing animations of the same character.
- **Phases are not aligned.** Sources of different lengths advance on their own clocks. For unrelated cycles (breathing and blinking) that is the correct behavior. To keep a 30-frame walk and a 60-frame run in step, seek them in the Player.
- **There are no per-part masks.** The range a track affects is decided by where its animation has keys (§1-2).
- **One ssab only.** Animations from another ssab cannot be stacked.
- **Instance and Effect parts are not mixed.** Their slots belong to the parts of the ssab, not to one animation, so a crossfade on track 0 does not set them up again: the child animation or the effect keeps running across the switch. Their keys are discrete attributes and come from track 0's main animation, and a slot acts on the incoming animation's key only when it sees a new key, which it detects by the frame the key sits on. Where the outgoing and incoming animations key the part on the same frame, the slot sees no change, and the part carries on with the outgoing animation's setting.

## 10. Performance

- **Playing a single animation costs nothing extra.** With one track at `alpha` 1 carrying one animation, the mix is only a pointer swap.
- **The cost scales with the number of parts, not the number of sources.** Each active source is computed once, and the layering is proportional to the number of parts.
- **Sources with a weight of 0 are not computed** (including one left over after a cancelled fade).
- While several sources are mixed, FrameData also carries the vertex deformation data of sources not used for drawing, so FrameData grows. The result is not affected.

---
[Back to contents](../README.md)
