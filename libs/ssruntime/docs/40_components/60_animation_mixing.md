> [日本語](./60_animation_mixing.ja.md) | **English**

# Animation Mixing

Play several animations at once — a walk with a limp laid over it, a head turn driven by
tracking, a transition from idle to attack — by stacking them on **tracks**. Mixing is runtime
state and FFI only; it needs no change to your `.ssab`.

## 1. The model

An instance owns a stack of tracks. They resolve in ascending order onto the **setup pose**,
the rest pose your `.ssae` defines:

```
resolved = setup pose
track 0  →  fold on   (your main animation)
track 1  →  fold on   (an overlay)
track 2  →  fold on   (another overlay)
```

Two knobs, and it matters which one you reach for:

| | What it does | Where it lives |
|---|---|---|
| **`alpha`** | How much of a track survives onto the pose below it. **This is the strength knob.** | per track |
| **`weight`** | The proportion between several animations sharing **one** track. | per source |

A track's `alpha` always means what you expect. A source's `weight` does not, on its own: if
one animation is the only one animating an attribute, it owns that attribute outright no
matter how small its weight, because there is nothing to be proportional against. Reach for
`weight` only when you are genuinely blending animations against each other on one track
(walk + run), and for `alpha` in every other case.

### Track 0 is special

Track 0 carries the **structure**: every discrete attribute (cell, part colour, shader, flip,
hide, priority, texture) and the events that fire come from it. `setup_animation` is exactly
"put this animation on track 0". Track 0 cannot be cleared — fade its `alpha` instead.

### A track only touches what it animates

An attribute a track's animation carries no keys for is passed through untouched. An overlay
that only animates the head therefore leaves the legs alone **without needing a mask** — which
also means an overlay is only as narrow as the animation you authored. If you want an overlay
confined to a body region, key only that region in it.

### The same SSAB

Only animations inside one `AnimationResource` (one `.ssab`) can be stacked together.

---

## 2. Playing one animation

Nothing changes for single-animation playback:

```cpp
ss_runtime_setup_animation(context, "walk");   // == track 0
ss_runtime_update(context, delta_ms);
ss_runtime_get_frame_data(context, frame_no, &out_data, &out_len);
```

## 3. Layering

```cpp
ss_runtime_setup_animation(context, "idle");        // track 0: the performance

ss_runtime_set_track_animation(context, 1, "look_around");
ss_runtime_set_track_alpha(context, 1, 0.4f);       // 40% of the overlay
```

Tracks are created on demand: setting track 3 also creates 1 and 2, silent. `alpha` is not
clamped — above 1 it extrapolates, which an overlay occasionally wants.

### Additive tracks

`SS_RUNTIME_TRACK_BLEND_ADDITIVE` adds a track's **offset from the setup pose** instead of
lerping toward its pose, so the animation underneath keeps showing through:

```cpp
ss_runtime_set_track_animation(context, 1, "lean_left");
ss_runtime_set_track_blend(context, 1, SS_RUNTIME_TRACK_BLEND_ADDITIVE);
ss_runtime_set_track_alpha(context, 1, 0.6f);       // 60% of the lean, on top of the walk
```

This is what a continuous-control axis wants. Author the axis so its **neutral frame matches
the setup pose**, then either play it or scrub it:

```cpp
// drive the axis from a tracked value instead of playing it
ss_runtime_set_track_frame(context, 1, 30.0f + yaw * 30.0f);
```

## 4. Transitions

`ss_runtime_crossfade` fades a track from whatever it is playing to a new animation. There are
no handles to pair up and nothing to clean up afterwards:

```cpp
ss_runtime_crossfade(context, 0, "run", 300.0f);   // 300ms, same unit as update()'s delta

ss_runtime_update(context, delta_ms);
float progress = ss_runtime_track_fade_progress(context, 0);  // 0..1, NaN when not fading
```

On completion the incoming animation becomes the track's and the outgoing one is released.

| Situation | Behaviour |
|---|---|
| `duration_ms <= 0` or `NaN`, or an unknown animation name | returns `false`; details in `get_last_error` |
| `crossfade` on a track that is playing nothing | starts playing immediately, no fade |
| `crossfade` while one is already running on that track | the running fade **completes first**, then the new one starts from there |
| you want to stop a fade | `ss_runtime_cancel_track_fade` — alpha and weights stay where they are |

### Fading out to the rest pose

The floor of the stack is the setup pose, so fading a track out *is* fading to rest:

```cpp
ss_runtime_fade_track(context, 0, 0.0f, 200.0f);   // settle to the setup pose over 200ms
```

## 5. Blending animations against each other

When you want proportions between animations — the locomotion case — put them on the same
track and set weights:

```cpp
ss_runtime_setup_animation(context, "walk");
uint32_t walk = ss_runtime_track_main_source(context, 0);
uint32_t run  = ss_runtime_track_add_source(context, 0, "run", 0.0f);

// 70% walk, 30% run
ss_runtime_set_source_weight(context, walk, 0.7f);
ss_runtime_set_source_weight(context, run, 0.3f);
ss_runtime_set_source_frame(context, run, 0.0f);
```

`ss_runtime_track_main_source` hands you the animation `set_track_animation` (or
`setup_animation`) put on the track, which starts at weight 1.

Weights are relative, not absolute: `(0.7, 0.3)` and `(7, 3)` mix identically. Each source
owns its own playback clock, so `ss_runtime_update(delta)` advances them all; sources of
different lengths are **not** phase-synchronised — seek them yourself if you need alignment.

## 6. Events

By default only a track's fully-weighted animation fires events. A source opts in by lowering
its threshold below its current weight:

```cpp
ss_runtime_set_source_event_threshold(context, run, 0.5f);  // fires once run reaches 0.5
```

Events are reported with the handle of the source that fired them, so several sources can be
told apart:

```cpp
int n = ss_runtime_get_passed_event_count(context);
for (int i = 0; i < n; ++i) {
    int32_t  event_idx = ss_runtime_get_passed_event_index(context, i);
    uint32_t src       = ss_runtime_get_passed_event_source_handle(context, i);
}
```

---

## 7. C-API reference

### 7.1 Tracks

| Function | Returns | On error |
|---|---|---|
| `ss_runtime_set_track_animation(ctx, track, name)` | `bool` | `false` |
| `ss_runtime_set_track_animation_by_hash(ctx, track, hash)` | `bool` | `false` |
| `ss_runtime_set_track_alpha(ctx, track, alpha)` | `bool` | `false` |
| `ss_runtime_track_alpha(ctx, track)` | `float` | `NaN` |
| `ss_runtime_set_track_blend(ctx, track, blend)` | `bool` | `false` |
| `ss_runtime_track_blend(ctx, track)` | `uint32_t` (`0` mix / `1` additive) | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_track_count(ctx)` | `uint32_t` | `0` |
| `ss_runtime_clear_track(ctx, track)` | `bool` | `false` (track 0 is rejected) |
| `ss_runtime_set_track_frame(ctx, track, frame)` | `bool` | `false` |
| `ss_runtime_track_frame(ctx, track)` | `float` | `NaN` |
| `ss_runtime_set_track_speed(ctx, track, speed)` | `bool` | `false` |

### 7.2 Transitions

| Function | Returns | On error |
|---|---|---|
| `ss_runtime_crossfade(ctx, track, name, duration_ms)` | `bool` | `false` |
| `ss_runtime_crossfade_by_hash(ctx, track, hash, duration_ms)` | `bool` | `false` |
| `ss_runtime_fade_track(ctx, track, target_alpha, duration_ms)` | `bool` | `false` |
| `ss_runtime_track_fade_progress(ctx, track)` | `float` (0-1) | `NaN` (not fading) |
| `ss_runtime_cancel_track_fade(ctx, track)` | `bool` | `false` |

### 7.3 Several sources on one track

| Function | Returns | On error |
|---|---|---|
| `ss_runtime_track_main_source(ctx, track)` | `uint32_t` handle | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_track_add_source(ctx, track, name, weight)` | `uint32_t` handle | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_track_add_source_by_hash(ctx, track, hash, weight)` | `uint32_t` handle | The same |
| `ss_runtime_set_source_weight(ctx, handle, weight)` | `bool` | `false` |
| `ss_runtime_source_weight(ctx, handle)` | `float` | `NaN` |
| `ss_runtime_set_source_frame(ctx, handle, frame)` | `bool` | `false` |
| `ss_runtime_source_frame(ctx, handle)` | `float` | `NaN` |
| `ss_runtime_set_source_event_threshold(ctx, handle, threshold)` | `bool` | `false` |
| `ss_runtime_source_event_threshold(ctx, handle)` | `float` | `NaN` |

Error codes come from `ss_runtime_get_last_error(ctx)`. The `NaN` sentinel exists so a valid
`0.0` stays distinguishable from "no such track/source" — test with `isnan` / `isnanf` in C,
`Number.isNaN` in JS.

## 8. WASM (JS/TS)

The Web bindings mirror the C-API one to one. Errors surface as exceptions and `Option<f32>`
returns `undefined`:

```typescript
ctx.setup_animation("idle");

ctx.set_track_animation(1, "look_around");
ctx.set_track_blend(1, 1);          // 1 = additive
ctx.set_track_alpha(1, 0.4);

ctx.crossfade(0, "run", 300.0);
const progress = ctx.track_fade_progress(0);   // number | undefined

const run = ctx.track_add_source(0, "run", 0.3);
ctx.set_source_weight(run, 0.5);
```

---

## 9. Limitations

- **Discrete attributes come from track 0.** Cell, part colour, shader, flip, hide, priority
  and texture are not blended — an overlay track cannot swap a cell. Use the
  [Override Layer](./65_override_layer.md) for that.
- **`parts_color` follows track 0.** For parts with a `parts_color`, that colour and its alpha
  dominate; the mixed `alpha` reaches the vertex colour only for parts without one. This does
  not affect blending the same character's own animations, the common case.
- **No phase synchronisation.** Sources of different lengths run on independent clocks. That is
  what you want for unrelated cycles (breathing and blinking); if you need a 30-frame walk and
  a 60-frame run to stay in step, seek them yourself.
- **No per-part masks yet.** A track's reach is decided by what its animation keys (§1).
- **One `.ssab`.** Animations from different `.ssab` files cannot be stacked.

## 10. Performance

- **Single-animation playback costs nothing.** With one audible track at full alpha carrying
  one animation, the mix collapses to a pointer swap.
- **Mixing scales with parts, not sources.** Each active source is evaluated once; the fold
  itself is proportional to the part count.
- **Silent sources are skipped.** A source at weight 0 — including one left over after a
  crossfade — costs nothing.

---

[Back to Table of Contents](../README.md)
