> [日本語](./20_playback_control.ja.md) | **English**

# Playback Control & Events

Here you implement management of the animation playback state (play, stop, loop) and the mechanism that notifies your game logic of timeline events (user data, signals).

## 1. Setting up and playing an animation

Specify the animation you want to play on the `libssruntime` context by name and start playback.

```cpp
// 1. Set up the animation
ss_runtime_setup_animation(context, "attack_pose");

// 2. Playback settings (loop count: -1 = infinite, 1 = once)
ss_runtime_set_loop(context, -1);

// 3. Start playback
ss_runtime_play(context);
```

### Values for `ss_runtime_set_loop`

| Value | Meaning |
| :--- | :--- |
| `n` (> 0) | Play `n` cycles and stop. `1` = once |
| `-1` | Infinite loop |
| `0` | Infinite loop (an alias of `-1`; anything ≤ 0 is normalized to `-1`) |

> [!NOTE]
> `setup_animation` resets the loop count to its default (infinite). Re-apply it after `setup_animation` and before `play`. `play` re-arms the counter with the loop count in effect at that moment, so replaying a finished animation with `play` applies the same loop count again.

### Moving the playhead
- **Seek**: jump to an arbitrary frame with `ss_runtime_set_frame_no(context, frame)`.
- **Pause**: `ss_runtime_pause(context)` pauses / resumes playback (a toggle).

## 2. Per-frame updates and detecting the end

Every frame the Player passes the game engine's delta time to the library to update the state.

### The implementation pattern

**Implement `looped` / `finished` notifications in the following form.** The two flags have different lifetimes, so they must be handled differently.

```cpp
class MyPlayer {
    bool prev_finished = false;   // for rising-edge detection of finished

    void update(float delta_seconds) {
        // 1. Run the update (pass the delta time in milliseconds)
        ss_runtime_update(context, delta_seconds * 1000.0f);

        // 2. looped is a "pulse". Read it as-is immediately after update
        if (ss_runtime_is_looped(context)) {
            emit_looped();
        }

        // 3. finished is a "state". Notify exactly once on the rising edge
        bool finished = ss_runtime_is_finished(context);
        if (finished && !prev_finished) {
            emit_finished();
        }
        prev_finished = finished;
    }

    void play() {
        prev_finished = false;   // reset the latch in preparation for replaying
        ss_runtime_play(context);
    }
};
```

### The difference between the two flags

| API | Meaning | Lifetime | How to handle it |
| :--- | :--- | :--- | :--- |
| `ss_runtime_is_looped` | The playhead **returned to the start** (completed one cycle) | A **pulse**. Cleared at the top of every `update` | Notify as-is |
| `ss_runtime_is_finished` | **All** the specified loops have completed | A **state**. Stays `true` until the next `play` / `setup_animation` | Notify **on the edge** |

- `is_looped` collapses into a single occurrence even when several cycles complete within one `update` (a large `delta` with frame skipping enabled).
- `is_finished` is not raised by `stop()`. It is raised only when the specified loop count completes.
- `is_finished` is never raised for an infinite loop (the default). To detect the end of an infinite loop, count `looped` or decide it on the Player side by calling `stop()`.

### Common mistakes

These are real defects that occurred in several Players. None of them happen if you use `is_finished` in the pattern above.

| Wrong implementation | Symptom |
| :--- | :--- |
| Notifying `is_finished` directly without taking the edge | **Fires every frame** after completion |
| Not resetting `prev_finished` in `play` | The next completion can be **missed** when replaying after a completion |
| Substituting `!is_playing` for `finished` | **False firing** - indistinguishable from `stop()` or from before playback starts |
| Deciding completion from "did we reach the final frame?" | **Fires every cycle** with a finite loop; **never fires** with an infinite loop; and if you stop the playback loop there, **playback is cut off during the first cycle** |

> [!NOTE]
> The internal state "the final frame is being displayed" (`end_frame_reached`) used to be exposed in the C-API / WASM, but **every Player that used it for completion detection hit a bug**, so it was removed from the Player-facing API. Use only `is_finished` for completion detection.

> [!NOTE]
> If your `update` early-returns while `is_playing` is false, that guard also serves as edge detection, so `prev_finished` is unnecessary in theory. But the dependency becomes implicit and moving the guard later breaks it. **We recommend holding the latch explicitly, exactly as in the pattern above.**

### Example event sequence

Playing a 0-4 frame animation with `set_loop(3)`:

| Cycle | Notification |
| :--- | :--- |
| End of cycle 1 → back to the start | `looped` |
| End of cycle 2 → back to the start | `looped` |
| End of cycle 3 | `finished` (it does not return to the start; it stops on the final frame and `is_playing` becomes false) |

With an infinite loop (`-1` / `0`), only `looped` fires repeatedly and `finished` never fires.

## 3. Retrieving events (polling)

`libssruntime` collects the events passed during an update (user data, signals, sounds). The Player retrieves them in bulk (polls) immediately after `update`.

```cpp
// Get the number of events passed
int count = ss_runtime_get_passed_event_count(context);

for (int i = 0; i < count; i++) {
    // Get the index of the event data
    int event_idx = ss_runtime_get_passed_event_index(context, i);

    // Reference the concrete data from the SSAB (FlatBuffers)
    auto events = ssab_binary->events()->Get(event_idx);

    // Handling UserData
    if (events->users()) {
        for (auto user : *events->users()) {
            emit_signal("user_data", user->part_index(), user->value());
        }
    }
    // Handling Signal (SpriteStudio v7 and later)
    if (events->signals()) {
        for (auto sig : *events->signals()) {
            if (!sig || !sig->value() || !sig->value()->active()) continue;

            auto val = sig->value();
            const char* command = val->command_id() ? val->command_id()->c_str() : "";

            // Expanding the parameters (example: storing them in a generic C++ map)
            std::map<std::string, MyVariant> params_map;
            if (auto params = val->params()) {
                for (uint32_t k = 0; k < params->size(); k++) {
                    auto param = params->Get(k);
                    if (!param || !param->param_id()) continue;
                    std::string p_id = param->param_id()->c_str();

                    // Take the value out of the FlatBuffers union type and store it
                    switch (param->value_type()) {
                        case ss::format::SignalCommandParamValue_Bool:
                            if (auto b = param->value_as_Bool()) params_map[p_id] = b->value();
                            break;
                        case ss::format::SignalCommandParamValue_Index:
                            if (auto i = param->value_as_Index()) params_map[p_id] = i->value();
                            break;
                        case ss::format::SignalCommandParamValue_Integer:
                            if (auto i = param->value_as_Integer()) params_map[p_id] = i->value();
                            break;
                        case ss::format::SignalCommandParamValue_Floating:
                            if (auto f = param->value_as_Floating()) params_map[p_id] = f->value();
                            break;
                        case ss::format::SignalCommandParamValue_Text:
                            if (auto t = param->value_as_Text()) {
                                if (t->value()) params_map[p_id] = t->value()->c_str();
                            }
                            break;
                        default:
                            break;
                    }
                }
            }
            emit_signal("signal", command, params_map);
        }
    }

    // Handling Audio
    if (events->audios()) {
        for (auto audio : *events->audios()) {
            if (!audio || !audio->value()) continue;

            auto val = audio->value();
            const char* sound_name = val->sound_name() ? val->sound_name()->c_str() : "";
            int loop_num = val->loop_num();

            emit_signal("audio", audio->part_index(), sound_name, loop_num);
        }
    }
}
```

## 4. Recommended Player API (design guidelines)

To make animations easy for your users to work with, we recommend providing the following API interface.

| Category | Recommended API | Contents |
| :--- | :--- | :--- |
| **Basic operations** | `set_animation(name)`, `play(loop)`, `stop()`, `pause()` | Setting, playing, stopping, and pausing an animation |
| **Parameters** | `set_speed(f32)`, `set_frame(f32)` | Speed and the current frame position |
| **Callbacks** | `finished`, `looped`, `user_data`, `signal` | Notification when each event occurs (see §2 of this chapter) |

### Separating `set_animation` from `play` (recommended)

We recommend exposing **setup** (`set_animation`) and **starting playback** (`play`) as separate methods (adopted by the Godot and Unity ports). Internally, map the C-API's `ss_runtime_setup_animation` → `ss_runtime_play` (see §1 of this chapter) onto those two public APIs directly.

> **Why separate them (memory allocation)**: `set_animation` (`ss_runtime_setup_animation`) involves heap allocation inside ssruntime when setting the animation up. Doing that inside `play` every time means an allocation on every playback. The aim is to **finish the allocating setup before playback and keep `play()` a light, allocation-free operation**.

| API | Contents |
| :--- | :--- |
| `set_animation(name)` | Loads and sets the animation data and waits on the first frame (**does not start playback**). The internal allocation happens here |
| `play()` / `play(startFrame)` | Starts playing the already set-up animation from where it is (or from a specified frame) |

Benefits:

- Heavy preparation (`setup_animation`, and loading the assets that animation needs) can be finished before `play()`, so playback can start immediately.
- You can display the first frame posed without playing (an idle pose).
- You can swap the animation even while not playing (it does not start until the next `play()`).

With this design, also providing a `started` callback as the notification for playback beginning makes it easier for users to distinguish `set_animation` (swapping) from `play` (starting).

> [!NOTE]
> Because `setup_animation` resets the loop count, speed, playback direction, and so on to their defaults, re-apply them inside `setAnimation` (or before `play`).

### Sub-frame playback
`libssruntime` manages the frame number as an `f32`, which allows "interpolation" between integer frames. To achieve smooth motion on high-refresh-rate displays, we recommend that the Player support `f32` frame management too.

#### The accepted frame range and how the end is handled
The `frame_no` accepted by `ss_runtime_get_frame_data(frame_no)` is the half-open interval **`[start_frame, end_frame + 1)`**. This corresponds to the idea that "frame N occupies the time interval `[N, N+1)` on a continuous playback timeline".

- During sub-frame playback, the playhead momentarily takes a fractional value in `(end_frame, end_frame + 1)` at the end and at loop boundaries. That interval is **valid** and returns a pose holding the final keyframe's values (identical to exactly `end_frame`). No extrapolation beyond the end is performed.
- Out-of-range values (`frame_no < start_frame` / `frame_no >= end_frame + 1` / `NaN`) return `InvalidParameter`. Note that in that case **no FrameData is obtained and that frame's drawing disappears** (every part vanishes momentarily).

> [!NOTE]
> During sub-frame playback there is **no need** to clamp `frame_no` to `end_frame` in order to avoid the disappearance at the end. The runtime handles the final interval `(end_frame, end_frame + 1)` correctly.
>
> Note however that this range is that of **the whole animation data (the clip)**. When you have limited the playback range with `set_animation_section`, the runtime does not clamp to the section boundaries and evaluates the real keyframes within the clip even beyond the section. When combining section playback with sub-frame playback, **clamp `frame_no` to the section boundary on the Player side** for display purposes.

### Blending several animations (state machine / locomotion blend)
To implement crossfades or to stack animations, see [Animation Mixing](../40_components/60_animation_mixing.md). It covers tracks and their `alpha`, additive overlays, and blending several animations on one track by weight.

---
[Back to Table of Contents](../README.md)
