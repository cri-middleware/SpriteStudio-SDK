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

Track 0 carries the **structure**. The discrete attributes (cell, parts color, shader, flip, hide, draw priority, texture) and the default event firing all come from track 0. `ss_runtime_setup_animation` is the operation "put this animation on track 0". Track 0 cannot be emptied; to remove it, fade its `alpha`.

### 1-2. A track affects only the attributes it keys

For an attribute that a track's animation has no keys for, the result from below passes through unchanged. Layering an animation keyed only on the head does not affect the legs, **even without a mask**. Put the other way, **a layered animation reaches exactly the range that was authored**. To affect only part of the body, key only that part.

### 1-3. Within one ssab only

Only animations inside one resource (one ssab) can be stacked.

## 2. Playing a single animation

If you do not use mixing, playback works as before.

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

Tracks are created as needed (setting track 3 also creates tracks 1 and 2, empty). `alpha` is not clamped to 0–1. Above 1 it extrapolates, which layering sometimes wants.

### 3-1. Additive tracks

A `SS_RUNTIME_TRACK_BLEND_ADDITIVE` track adds its **difference from the setup pose** instead of moving toward the track's pose. The animation below stays visible.

```cpp
ss_runtime_set_track_animation(ctx, 1, "lean_left");
ss_runtime_set_track_blend(ctx, 1, SS_RUNTIME_TRACK_BLEND_ADDITIVE);
ss_runtime_set_track_alpha(ctx, 1, 0.6f);           // the lean at 60%, on top of the walk
```

Use this for an axis driven by a continuous value. Author the axis animation so that **its middle frame matches the setup pose**, then play it or move its frame directly.

```cpp
// move the frame from a tracking value instead of playing it
ss_runtime_set_track_frame(ctx, 1, 30.0f + yaw * 30.0f);
```

## 4. Switching

`ss_runtime_crossfade` fades a track from the animation it is playing now to a new animation. There are no handles to manage in pairs and nothing to clean up.

```cpp
ss_runtime_crossfade(ctx, 0, "run", 300.0f);   // 300ms, the same unit as update's elapsed time

ss_runtime_update(ctx, delta_ms);
float progress = ss_runtime_get_track_fade_progress(ctx, 0);   // 0 to 1; NaN when not fading
```

When the fade ends, the incoming animation becomes the track's animation, and the outgoing source is released.

| Situation | Behavior |
|---|---|
| `duration_ms <= 0` / `NaN` / an animation name that does not exist | Returns `false`. The reason is in `ss_runtime_get_last_error` |
| `crossfade` on a track that is playing nothing | Starts playing at once, without a fade |
| `crossfade` again on a track that is fading | **Finishes the running fade first**, then starts the new fade from there |
| You want to stop a fade | `ss_runtime_cancel_track_fade`. `alpha` and `weight` stop at their current values |

### 4-1. Returning to the setup pose

The bottom of the stack is the setup pose, so fading a track out returns it straight to the setup pose.

```cpp
ss_runtime_fade_track(ctx, 0, 0.0f, 200.0f);   // to the setup pose over 200ms
```

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

`ss_runtime_get_track_main_source` returns the handle of the animation that `ss_runtime_set_track_animation` (or `ss_runtime_setup_animation`) put on the track. `weight` starts at 1.

`weight` is a ratio, not an absolute value (`(0.7, 0.3)` and `(7, 3)` give the same result). Each source has its own playback clock, and `ss_runtime_update` advances them all. **Sources of different lengths do not stay in phase.** To align them, seek them in the Player.

## 6. Events

By default, only the animation on track 0 fires events. To make another source fire too, lower its threshold to its `weight` or below. Events are reported from sources where `weight >= threshold`. The default threshold is `+∞`, which no weight reaches, so the lone source of another track does not fire at weight 1 either.

```cpp
ss_runtime_set_source_event_threshold(ctx, run, 0.5f);   // fires when run's weight is 0.5 or more
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

Get the reason for a failure with `ss_runtime_get_last_error(ctx)`. Functions that return `float` use `NaN` to signal failure so that a valid `0.0` can be told apart from "that track or source does not exist". Test with `isnan` in C and `Number.isNaN` in JavaScript.

### 7-1. Tracks

| Function | Returns | On failure |
|---|---|---|
| `ss_runtime_set_track_animation(ctx, track, name)` | `bool` | `false` |
| `ss_runtime_set_track_animation_by_hash(ctx, track, hash)` | `bool` | `false` |
| `ss_runtime_set_track_alpha(ctx, track, alpha)` | `bool` | `false` |
| `ss_runtime_get_track_alpha(ctx, track)` | `float` | `NaN` |
| `ss_runtime_set_track_blend(ctx, track, blend)` | `bool` | `false` |
| `ss_runtime_get_track_blend(ctx, track)` | `uint32_t` (`0` = Mix / `1` = Additive) | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_get_track_count(ctx)` | `uint32_t` | `0` |
| `ss_runtime_clear_track(ctx, track)` | `bool` | `false` (track 0 cannot be emptied) |
| `ss_runtime_set_track_frame(ctx, track, frame)` | `bool` | `false` |
| `ss_runtime_get_track_frame(ctx, track)` | `float` | `NaN` |
| `ss_runtime_set_track_speed(ctx, track, speed)` | `bool` | `false` |

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
| `ss_runtime_set_source_event_threshold(ctx, handle, threshold)` | `bool` | `false` |
| `ss_runtime_get_source_event_threshold(ctx, handle)` | `float` | `NaN` |

## 8. WebAssembly

The WebAssembly bindings map one to one onto the C-API. Failures are thrown as exceptions, and a `float` with no value becomes `undefined`.

```typescript
ctx.setup_animation("idle");

ctx.set_track_animation(1, "look_around");
ctx.set_track_blend(1, 1);          // 1 = Additive
ctx.set_track_alpha(1, 0.4);

ctx.crossfade(0, "run", 300.0);
const progress = ctx.get_track_fade_progress(0);   // number | undefined

const run = ctx.add_track_source(0, "run", 0.3);
ctx.set_source_weight(run, 0.5);
```

## 9. Constraints

- **Discrete attributes come from track 0.** Cell, parts color, shader, flip, hide, draw priority and texture cannot be mixed, so a layered track cannot swap a cell. To swap a cell, use [Part Overrides](./20_part_overrides.md).
- **Assign the same cell to the same part in every animation you stack.** The cell's size and origin come from track 0, so a different cell shifts the size and origin and breaks the pose. Animations within the same ssae normally do not have this problem.
- **Parts color follows track 0.** For a part with parts color, that color and its alpha take precedence. The mixed opacity reaches the vertex color only for parts without parts color. This is not a problem in the usual case of mixing animations of the same character.
- **Phases are not aligned.** Sources of different lengths advance on their own clocks. For unrelated cycles (breathing and blinking) that is the correct behavior. To keep a 30-frame walk and a 60-frame run in step, seek them in the Player.
- **There are no per-part masks.** The range a track affects is decided by where its animation has keys (§1-2).
- **One ssab only.** Animations from another ssab cannot be stacked.

## 10. Performance

- **Playing a single animation costs nothing extra.** With one track at `alpha` 1 carrying one animation, the mix is only a pointer swap.
- **The cost scales with the number of parts, not the number of sources.** Each active source is computed once, and the layering is proportional to the number of parts.
- **Sources with a weight of 0 are not computed** (including one left over after a crossfade).
- While several sources are mixed, FrameData also carries the vertex deformation data of sources not used for drawing, so FrameData grows. The result is not affected.

---
[Back to contents](../README.md)
