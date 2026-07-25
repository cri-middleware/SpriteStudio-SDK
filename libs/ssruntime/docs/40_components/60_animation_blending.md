> [日本語](./60_animation_blending.ja.md) | **English**

# Animation Blending / Mixing

An API that composites several animations with weights, achieving smooth transitions (state-machine crossfades) and locomotion blending (mixing walk + run). It was added in Phase 3 and consists purely of FFI and runtime state, with no `.fbs` change.

## 1. Conceptual model

### Primary and Secondary
- **Primary**: the animation's "structural source". Attributes decided **discretely** - cell / blend_type / shader / visibility and so on - and the origin of event firing are all taken from the primary.
- **Secondary**: participates in the weighted composition of **continuous values** such as TRSA / size / pivot / UV / α.

### How continuous and discrete values are handled

| Kind | Handling | Examples |
|---|---|---|
| Continuous | The weighted linear sum of active sources (auto-normalized) | position_xyz, rotation_xyz, scale, local_scale, α, local_α, pivot, anchor, size, uv_*, bounding_radius, mask, skew, `raw_vertex_deform` |
| Discrete | **Copied from the primary** | cell index, part_color, shader, deform table, flip_h/v, hide, img_flip, priority, texture, mesh_deform |
| `update_flag` | OR-composed across active sources | — |

Although `priority` is an f32, it is used as the sort key for draw z-order, so it is treated as discrete (taken from the primary) to avoid z-order flicker caused by an oscillating composited value.

### Conditions for an active source
Only slots with `animation_idx >= 0 && weight > 0.0` participate in continuous-value composition. A weight ≤ 0 is inactive (no contribution).

**Primary exception**: even when the primary has `is_active() == false` (weight = 0), pose extraction (structural expansion) is always performed for it as the structural source. A primary with weight = 0 means "I decide the cell and the discrete attributes, but I leave the continuous values to the secondaries".

### Automatic weight normalization
Weights are interpreted as relative ratios, not absolute values. `(0.7, 0.3)` and `(7.0, 3.0)` produce the same composition (they are normalized to sum to 1 internally).

### The same-SSAB constraint
Only animations within the same `AnimationResource` (= the same SSAB) can be blended. Composition across several SSABs is not supported on the Player side.

---

## 2. Basic usage

```cpp
// 1. Set up the primary as usual (at this point blend_source_count = 1)
ss_runtime_setup_animation(context, "walk");

// 2. Add a secondary. A stable handle is returned
uint32_t run = ss_runtime_add_blend_source(context, "run", 0.5f);
if (run == SS_RUNTIME_INVALID_HANDLE) {
    // setup_animation not called / name mismatch and so on. Details in get_last_error.
}

// 3. Blend walk and run 50/50
ss_runtime_set_source_weight(context, /*primary*/ 0, 0.5f);
ss_runtime_set_source_weight(context, run, 0.5f);

// 4. update / get_frame_data as usual
ss_runtime_update(context, delta_ms);
uint8_t* out_data; size_t out_len;
ss_runtime_get_frame_data(context, frame_no, &out_data, &out_len);
```

---

## 3. Per-source playback control

Each source has its own independent PlaybackController and holds its own frame / speed.

```cpp
// Explicitly seek the secondary's frame
ss_runtime_set_source_frame(context, run, 0.0f);

// Change the secondary's playback speed (play at half the speed of walk)
ss_runtime_set_source_speed(context, run, 0.5f);

// Get the current frame / weight. NaN when the handle is invalid or inactive
float w = ss_runtime_source_weight(context, run);
if (!isnanf(w)) { /* a valid value */ }
```

### Behavior of `ss_runtime_update(delta)`
It advances the controllers of **every active source** by `delta`. The fading-in source during a crossfade also advances in time naturally, so for state-machine use a single call to `Context::update` advances every source in sync.

### Behavior of `ss_runtime_get_frame_data(frame_no)`
- **Primary**: evaluated at the `frame_no` argument (preserving the single-animation behavior).
- **Secondary**: evaluated at each source's controller's current frame (`get_frame_no`).
- The `frame_no` argument accepts the half-open interval `[start_frame, end_frame + 1)`; the final interval `(end_frame, end_frame + 1)` returns the final keyframe held. For details see [Sub-frame playback - the accepted frame range and how the end is handled](../30_core/20_playback_control.md#the-accepted-frame-range-and-how-the-end-is-handled).

---

## 4. State-machine crossfade patterns

### 4.1 The auto-crossfade helper (recommended)

`ss_runtime_crossfade(ctx, from, to, duration_ms)` lets the runtime handle a linear crossfade. A timer advances on every `update(delta)` call, interpolating `(from.weight, to.weight)` from `(1, 0)` to `(0, 1)`. On completion, `to` is promoted to primary and `from` is automatically deactivated (`animation_idx = -1`).

```cpp
ss_runtime_setup_animation(context, "walk");
uint32_t walk = ss_runtime_primary_handle(context);
uint32_t run = ss_runtime_add_blend_source(context, "run", 0.0f);
ss_runtime_crossfade(context, walk, run, 300.0f);  // 300ms = the same unit as update()'s delta

// Leave the fade's progression to update() and just query the progress
ss_runtime_update(context, delta_ms);
float progress = ss_runtime_crossfade_progress(context);  // 0.0-1.0, NaN when not running
```

The state after completion:
- `primary_handle()` == `run`
- The `walk` slot is inactive (`source_weight(walk)` returns NaN)
- `crossfade_progress()` is NaN

At the moment of `set_primary`, the structural source (cell and so on) swaps to `to`. Because the fading-in secondary's weight reaches 1.0 at the end of the fade, visual discontinuity is minimized.

### 4.2 Errors and in-flight handling

| Situation | Behavior |
|---|---|
| `from == to` / `duration_ms <= 0` / `NaN` / an invalid handle | `crossfade` returns `false`, `last_error = InvalidParameter` |
| `from` is empty / inactive | The same (`from` must be a real animation) |
| `to` is an empty source | Accepted. On completion the primary is not swapped and it stops with `to`'s weight = 1.0 (a fade-out to the setup pose) |
| Calling `crossfade` again during a `crossfade` | The old fade is discarded and the new one starts immediately. The weights of `from` / `to` start again from `(1, 0)` |
| `remove_blend_source(from)` or `(to)` during a `crossfade` | The fade is cancelled automatically |
| You want to stop the fade manually | `ss_runtime_cancel_crossfade(ctx)`. The weights keep the values they had at call time (normalize on the caller side) |

### 4.3 The legacy manual control (compatible)

The pattern of driving the weights yourself every frame without `crossfade` remains supported. Use it when implementing compound transitions such as a state machine (curves, easing, hold-previous):

```cpp
float t = elapsed / fade_duration;
ss_runtime_set_source_weight(context, walk, 1.0f - t);
ss_runtime_set_source_weight(context, run, t);
// Completion timing and slot cleanup are the caller's responsibility
ss_runtime_set_primary(context, run);
ss_runtime_remove_blend_source(context, walk);
```

---

## 4.4 Empty source (fading out to the setup pose)

You can add an "identity pose source" that has no real animation with `ss_runtime_add_empty_blend_source(ctx, weight)`. Use it when you want to fade the primary out toward the setup pose (returning naturally to a rest pose at the end of an attack motion, for example).

**An empty source's contributions** (when mixing):

| Attribute | Value |
|---|---|
| `scale_*`, `local_scale_*`, `alpha`, `local_alpha`, `uv_scale_*` | `1.0` |
| `position_*`, `rotation_*`, `pivot_*`, `anchor_*`, `uv_translation_*`, `uv_rotation_*`, `skew_*`, `mask`, `bounding_radius`, `size_*` | `0.0` |
| `update_flag` | empty (the primary's flag dominates) |
| `raw_vertex_deform` | `present=false` (no contribution) |

```cpp
ss_runtime_setup_animation(context, "attack");
uint32_t attack = ss_runtime_primary_handle(context);
uint32_t empty = ss_runtime_add_empty_blend_source(context, 0.0f);
ss_runtime_crossfade(context, attack, empty, 200.0f);  // fade the attack pose to the setup pose
```

On completion:
- The primary is still `attack` (an empty source is not promoted to primary)
- attack's weight = 0, empty's weight = 1
- The resolved pose is the identity pose

The caller can then clean the empty source up with `ss_runtime_remove_blend_source(empty)`, or fade another animation in with `crossfade`.

**Limitations**:
- `set_source_frame` / `set_source_speed` / `set_source_event_threshold` / `set_primary` on an empty source return `InvalidParameter` (it has no controller / animation).
- `set_source_weight` / `source_weight` / `remove_blend_source` / `clear_blend_sources` work the same as for any other source.
- `add_empty_blend_source` returns `AnimationNotFound` unless a real primary has been set up. The primary must always be a real animation (it is the authoritative source of discrete attributes / events).
- Empty means "interpolate toward the identity pose", not "interpolate toward invisible". If you want a visual fade-out (alpha → 0), achieve it with primary-side keyframes or with alpha in the Player layer.

---

## 4.5 Per-source event firing (event threshold)

By default only the primary fires events (every source is initialized with `event_threshold = 1.0`). To let secondaries fire events too, lower the per-source threshold with `ss_runtime_set_source_event_threshold(ctx, handle, threshold)`.

**Firing rule**: fires when `source.weight >= source.event_threshold` (the primary ignores the threshold and always fires).

```cpp
uint32_t walk = ss_runtime_primary_handle(context);
uint32_t run = ss_runtime_add_blend_source(context, "run", 0.0f);
ss_runtime_set_source_event_threshold(context, run, 0.5f);
// events start firing from the moment run's weight reaches 0.5
```

To distinguish cases where several sources fire on the same frame, obtain the source handle from a parallel array:

```cpp
int n = ss_runtime_get_passed_event_count(context);
for (int i = 0; i < n; ++i) {
    int32_t event_idx = ss_runtime_get_passed_event_index(context, i);
    uint32_t src     = ss_runtime_get_passed_event_source_handle(context, i);
    // event_idx is an index into the animation events table of the source that src points at
}
```

When `src == primary_handle()` this is the legacy behavior (the primary fired). When `src` is a secondary handle, resolve the event details using the animation metadata of the source obtained from `src` (keep a source → animation mapping on the Player side).

Use cases:
- You want the new animation's footstep events to fire in the latter half of a crossfade → `set_source_event_threshold(new_source, 0.5)`
- A damage overlay should keep its events suppressed (leave it at the default 1.0)

---

## 5. Handle and slot lifecycle

- `add_blend_source` appends to the slot at the end and returns a stable `u32` handle.
- `remove_blend_source` **marks the slot inactive** (`animation_idx = -1`). The slot itself remains in the array, so existing handles are unchanged and no other handles shift.
- `clear_blend_sources` makes every slot except the primary inactive. The primary is untouched.
- The primary slot cannot be `remove_blend_source`-ed (it returns `InvalidParameter`). To change the primary, first promote another slot with `set_primary`.

---

## 6. C-API reference

### 6.1 Source management

| Function | Return value | On error |
|---|---|---|
| `ss_runtime_add_blend_source(ctx, name, weight)` | `uint32_t` handle | `SS_RUNTIME_INVALID_HANDLE` |
| `ss_runtime_add_blend_source_by_hash(ctx, hash, weight)` | `uint32_t` handle | The same |
| `ss_runtime_add_empty_blend_source(ctx, weight)` | `uint32_t` handle (an empty source) | The same |
| `ss_runtime_remove_blend_source(ctx, handle)` | `bool` | `false` |
| `ss_runtime_clear_blend_sources(ctx)` | void | — |
| `ss_runtime_blend_source_count(ctx)` | `uint32_t` (the active count, **including the primary and empty sources**) | `0` |
| `ss_runtime_set_primary(ctx, handle)` | `bool` | `false` (empty sources are rejected) |
| `ss_runtime_primary_handle(ctx)` | `uint32_t` | `SS_RUNTIME_INVALID_HANDLE` |

### 6.2 Per-source playback control / weight

| Function | Return value | On error |
|---|---|---|
| `ss_runtime_set_source_weight(ctx, handle, weight)` | `bool` | `false` |
| `ss_runtime_source_weight(ctx, handle)` | `float` | `NaN` (invalid / inactive) |
| `ss_runtime_set_source_frame(ctx, handle, frame)` | `bool` | `false` (empty sources rejected) |
| `ss_runtime_source_frame(ctx, handle)` | `float` | `NaN` |
| `ss_runtime_set_source_speed(ctx, handle, speed)` | `bool` | `false` (empty sources rejected) |

### 6.3 Auto-crossfade

| Function | Return value | On error |
|---|---|---|
| `ss_runtime_crossfade(ctx, from, to, duration_ms)` | `bool` | `false` |
| `ss_runtime_crossfade_progress(ctx)` | `float` (0.0-1.0) | `NaN` (not running) |
| `ss_runtime_cancel_crossfade(ctx)` | void | — |

### 6.4 Per-source events

| Function | Return value | On error |
|---|---|---|
| `ss_runtime_set_source_event_threshold(ctx, handle, threshold)` | `bool` | `false` (empty sources rejected) |
| `ss_runtime_source_event_threshold(ctx, handle)` | `float` | `NaN` |
| `ss_runtime_get_passed_event_count(ctx)` | `int32_t` | — |
| `ss_runtime_get_passed_event_index(ctx, i)` | `int32_t` (the event index within the animation) | `-1` |
| `ss_runtime_get_passed_event_source_handle(ctx, i)` | `uint32_t` (the handle of the source that fired) | `SS_RUNTIME_INVALID_HANDLE` |

Detailed error codes are obtained with `ss_runtime_get_last_error(ctx)`. The `NaN` sentinel is used so that callers can distinguish "`weight = 0.0` (a valid value)" from "an invalid handle". Test for it with `isnan` / `isnanf` in C and with `Number.isNaN` in JS.

---

## 7. WASM (JS/TS) bindings

The Web bindings mirror the C-API 1:1. Errors propagate as `Result<(), JsValue>`, and `Option<f32>` returns `undefined` on the JS side.

```typescript
const ctx = /* SSRuntimeContext */;
ctx.setup_animation("walk");
const run = ctx.add_blend_source("run", 0.5);
ctx.set_source_weight(run, 0.3);

const w = ctx.source_weight(run);  // number | undefined
if (w !== undefined) {
    // ...
}

// Auto-crossfade
ctx.crossfade(walk, run, 300.0);
const progress = ctx.crossfade_progress();  // number | undefined

// Empty source for setup-pose fade-out
const empty = ctx.add_empty_blend_source(0.0);
ctx.crossfade(walk, empty, 200.0);

// Per-source event threshold + parallel handle
ctx.set_source_event_threshold(run, 0.5);
const src_handle = ctx.get_passed_event_source_handle(0);  // u32 (0xFFFFFFFF if out-of-range)
```

---

## 8. Known limitations

### Alpha premultiplication of `parts_color`
When blending, each source's `parts_color` is pre-multiplied with that source's own hierarchical α. Because the vertex color after the mix uses the primary's `parts_color` as-is, it ends up **reflecting the primary's α** and is on a different track from the mixed `state.alpha`.

**Practical behavior**:
- Parts without `parts_color`: the mixed `state.alpha` becomes the visual α directly, blending smoothly.
- Parts with `parts_color`: the primary's color and α dominate (the secondaries' `parts_color` is not used).

This is not a problem for locomotion blending of the same character (the common use case). If you have a requirement to blend the colors themselves, that is currently out of scope.

### Cell consistency
It is assumed that "the same cell is assigned to the same part" across every blended animation. Because `cell_meta_per_part` (size, pivot, texture_hash, and so on) is taken from the primary, assigning different cells can produce a pose inconsistent with the secondaries' TRSA mix.

Different animations within the same SSAE normally share the same cell assignment, so this is not a problem in typical use cases.

### FrameData grows when blending
When blending several sources, only the primary's deform (mesh deformation) is used for drawing, but the deform data of the secondary sources is also included in the FrameData. The FrameData's binary size grows accordingly, but drawing accuracy is unaffected.

---

## 9. Performance characteristics

### Cost when blending is unused
In the case where **only the primary is an active source** (single-animation playback without blending), the mix completes in O(1). There is no performance overhead for Players that do not use the blending API.

### Blending several sources
When blending N sources, pose extraction runs N times (once per source) and composition runs once. The composition cost does not depend on the source count N; it is proportional to the number of parts.

### Skipping inactive slots
Secondary sources with weight = 0 or that have already been `remove_blend_source`-ed are skipped. Even with a state-machine style of "leave the slot around after the fade-out completes", no extraction cost is incurred for that slot.

---

[Back to Table of Contents](../README.md)
