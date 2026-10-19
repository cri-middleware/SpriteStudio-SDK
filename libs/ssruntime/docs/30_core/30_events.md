> [日本語](./30_events.ja.md) | **English**

# Events

This page covers how to notify game code of the keys on the timeline (user data, signals, audio). The names and the set of events a Player exposes are set in [Player API Conventions §4](../20_design/30_api_conventions.md#4-events).

## 1. How it works

`ss_runtime_update` records the frames passed during that call. Right after `update`, the Player receives the events of the passed frames as a list.

- What the Brain returns is the **position** of each event in the ssab. The Player reads the event's contents (the user data value, the signal parameters, the sound name) from the ssab.
- The position is an index into `AnimationData.events()` of the animation being played. There is no event table at the top level of the ssab.
- If one `update` passes several frames, the events of all those frames are returned.

## 2. Reading events

```cpp
ss_runtime_update(ctx, delta_ms);

// The event table of the animation track 0 plays. With animation mixing, look it up per event (§4)
const auto* events = current_animation->events();
if (events != nullptr) {
    int count = ss_runtime_get_passed_event_count(ctx);
    for (int i = 0; i < count; i++) {
        int event_idx = ss_runtime_get_passed_event_index(ctx, i);
        if (event_idx < 0 || (uint32_t)event_idx >= events->size()) continue;
        auto per_frame = events->Get(event_idx);

        // The frame the key was placed on. Not necessarily the frame in which the pass was noticed
        int frame_no = per_frame->frame_index();

        // User data
        if (auto users = per_frame->users()) {
            for (auto user : *users) {
                emit_user_data(user->part_index(), user->value());
            }
        }

        // Signal
        if (auto signals = per_frame->signals()) {
            for (auto sig : *signals) {
                auto val = sig->value();
                if (!val->active()) continue;
                const char* command = val->command_id() ? val->command_id()->c_str() : "";

                std::map<std::string, MyVariant> params;
                if (auto list = val->params()) {
                    for (auto param : *list) {
                        if (!param->param_id()) continue;
                        std::string id = param->param_id()->str();
                        switch (param->value_type()) {
                            case ss::format::SignalCommandParamValue_Bool:
                                params[id] = param->value_as_Bool()->value(); break;
                            case ss::format::SignalCommandParamValue_Index:
                                params[id] = param->value_as_Index()->value(); break;
                            case ss::format::SignalCommandParamValue_Integer:
                                params[id] = param->value_as_Integer()->value(); break;
                            case ss::format::SignalCommandParamValue_Floating:
                                params[id] = param->value_as_Floating()->value(); break;
                            case ss::format::SignalCommandParamValue_Text:
                                if (auto t = param->value_as_Text()->value()) params[id] = t->str();
                                break;
                            default: break;
                        }
                    }
                }
                emit_signal(sig->part_index(), command, params);
            }
        }

        // Audio
        if (auto audios = per_frame->audios()) {
            for (auto audio : *audios) {
                auto val = audio->value();
                const char* sound_name = val->sound_name() ? val->sound_name()->c_str() : "";
                emit_audio(audio->part_index(), sound_name, val->loop_num());
            }
        }
    }
}
```

`emit_*` are placeholder functions standing for the Player's notification to the host; they do not exist. How to play a sound on an audio event is in [Audio](../50_features/50_audio.md).

## 3. Which frames are reported

| Operation | Frames reported by the next `update` |
| :--- | :--- |
| Normal playback | The frames passed since the previous `update` |
| `ss_runtime_play` / `ss_runtime_play_with_start_frame` | The start frame (and the frames passed after it) |
| `ss_runtime_set_frame_no` (seek) | The destination frame, whether playing or stopped. **The frames between the origin and the destination are not reported** |
| `ss_runtime_set_animation_section` | The frame at the start of the section (the end when backward) |

The events of passed frames are reported in backward playback too.

### 3-1. Events on the start frame

The first `update` after `play` reports the start frame (frame 0, for example) as a passed frame. This is how a key placed on frame 0 fires from the first playback.

**Do not gate event handling on a condition meant for drawing.** If you handle events inside an optimization such as "skip drawing if the integer frame number has not changed since last time", then when the first `update` does not advance the integer frame (when the first elapsed time is short, for example), the events of the start frame are missed. Whether they are missed depends on the first elapsed time, so the result is a defect that is hard to reproduce, such as "it sounds when the game runs but not in the editor preview". Handle the events the Brain reports, whether or not you draw.

### 3-2. Events skipped by a seek

A seek does not report the events between the origin and the destination. To notify those events as well, have the Player call `update` in small steps instead of seeking, or read the events of the frames in between from the ssab and notify them.

A seek, whether playing or stopped, makes the next `update` report the events of the destination frame (audio included). If you do not want a seek to play sounds, see [Audio §3](../50_features/50_audio.md#3-no-sound-on-seek).

### 3-3. Ticks the Player does not run

**Report nothing for a tick the Player does not run.** A tick that skipped evaluation because nothing moves, or that skipped a character off screen, calls no `ss_runtime_update`, so it has nothing to report. On such a tick, empty the previous tick's events and its loop, finish and start notifications. Left in place, the same event keeps playing a sound for as long as ticks are skipped, and the host keeps reading a loop or an end that was crossed long ago. When a tick may be skipped is in [Animation Mixing §1-6](../50_features/10_animation_mixing.md#1-6-skipping-ticks-when-nothing-moves).

A Player that reports the start notification from a tick (for example as a pulse) must not lose a start from `ss_runtime_play` / `ss_runtime_play_with_start_frame` called while ticks are skipped or while the character is off screen: report it on the first tick that runs after the call.

## 4. When several animations are layered

With animation mixing, by default only the events of the animation on track 0 are reported. To have the events of other sources reported too, lower that source's threshold; for the outgoing side of a crossfade, lower the track's outgoing threshold ([Animation Mixing §6](../50_features/10_animation_mixing.md#6-events)).

The value of `ss_runtime_get_passed_event_index` is an index into the event table of **the animation of the source that fired**. Get the handle of the source that fired with `ss_runtime_get_passed_event_source_handle(ctx, i)`, ask which animation it plays with `ss_runtime_get_source_animation_index(ctx, handle)` (an index into the ssab's `animations()`), and look the index up in that animation's `events()`.

**This applies to track 0 as well.** A crossfade on track 0 hands the track to the incoming animation when it begins, so from then on an event table kept from `ss_runtime_setup_animation` is the wrong one: events come back as other events, or are dropped as out of range. Only a Player that neither crossfades nor mixes can keep one table.

---
[Back to contents](../README.md)
