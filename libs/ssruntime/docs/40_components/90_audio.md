> [日本語](./90_audio.ja.md) | **English**

# Integrating Audio (Sound) Playback

The `audios` events emitted by audio parts do not play sound inside the runtime; each platform's Player must play them using the engine's own audio API.
Full audio support requires more than a mere playback trigger - it requires managing the playback state (lifecycle).

- **Why preloading matters**:
  - Downloading and decoding an audio file takes time. Starting the load when the event fires delays the sound, so we strongly recommend extracting the audio defined in the SSAB at animation setup time and decoding it into memory in the audio engine ahead of time (preloading).
- **Lazy initialization**:
  - To avoid needlessly starting an audio engine (such as the Web Audio API's `AudioContext`) and consuming resources in projects that use no sound, it is preferable to create the audio engine only the first time you detect that the SSAB contains audio data (lazy initialization).
- **Basic playback**:
  - Audio is played in response to the `audios` events fired on each frame's update (retrieved with `ss_runtime_get_passed_event_count` / `index`).
  - **Important**: event processing also fires during reverse playback, but reverse audio playback is often unsupported by audio APIs, so you must filter so that **sound is played only during forward playback (playback speed > 0 and moving forward)**. (Audio playback during reverse playback is a limitation.)
  - Unlike the SpriteStudio editor itself, a Player implementation **does not synchronize the audio playback position (offset) when seeking (jumping to an arbitrary frame)**. By design, sound is played **only at the moment the corresponding frame on the timeline is passed (when the event fires)**.
  - Although `loopNum` is named as a "loop count", it effectively means "number of times to play". A value of 1 or more is specified, and playback repeats that many times. SpriteStudio audio has no concept of an infinite loop.
- **A sound outlives the playback that triggered it**:
  - **The transport does not own the sound's lifetime.** `pause`, `resume`, `stop` and `setup_animation` end (or hold) the *animation*; a sound already sounding is left alone. This is not an omission — it is the only definition a Player can honour. Pausing the voices reaches only the sounds already playing, so a sound whose key would have been crossed during the hold never starts and no `resume` brings it back; the hold becomes a hole. And because `loopNum` is a play count with no infinite value, every sound ends on its own: a Player that sweeps sounds on a stop or an animation switch is not preventing a runaway, it is cutting one short. A sound that outlasts its animation is the animation data's business.
  - **Teardown is the exception**, and the only one: when the Player itself goes away (destroyed, disabled, removed from the scene) it must stop every sound it started, because nothing is left to own those voices. Give the host an explicit "stop the sound now" call as well — the point is that the Player does not make that call on its behalf.
  - Every official Player follows this: Godot stops on `NOTIFICATION_EXIT_TREE` and destruction, Unity on `OnDisable` / `OnDestroy`, Web in `destroy()`, Ren'Py on `hide` / `scene`. Nothing else stops a sound.
- **Event firing on the starting frame (implementation note)**:
  - Immediately after `play()` (and after starting playback at a specified frame), the first `update` reports the starting frame (for example frame 0) as a "passed frame". This is the behavior of the runtime's internal re-evaluation flag (`needs_evaluation`), which causes the starting frame to be included in `ss_runtime_get_passed_event_count` / `index`. Sounds placed on frame 0 (audio keys with a `LoopNum`, for example) fire on the first playback thanks to this mechanism.
  - For this reason, **you must not gate event retrieval / processing behind rendering-side optimization conditions such as "has the drawn frame changed since last time?"**. If you early-return on a tick where the integer frame does not advance at the starting frame (for example when the first elapsed time is short), you drop the starting-frame events the runtime already reported. Because that loss depends on the elapsed time of the first tick, it becomes a hard-to-reproduce bug where the sound plays in some environments but not others (for example, it plays when running the project but not in the editor preview). Always process events when the runtime reports them, regardless of whether you draw.
- **Relationship with seeking (frame jumps) - a limitation**:
  - The re-evaluation flag above is also raised by frame seeks (`ss_runtime_set_frame_no` and friends), so **seeking to an arbitrary frame during playback fires that frame's events (including audio) on the next `update`**. If you want to strictly honor "no audio position synchronization or playback on seek", the Player needs an implementation that suppresses audio firing for frames whose re-evaluation flag was raised by something other than starting playback (`play`).
  - Note that scrubbing while stopped (moving frames while not playing) makes no sound, because `update` is not running. Only "seeking during playback" needs consideration.

---
[Back to Table of Contents](../README.md)
