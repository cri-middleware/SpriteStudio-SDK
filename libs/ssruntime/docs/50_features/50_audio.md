> [日本語](./50_audio.ja.md) | **English**

# Audio

Audio part keys do not play sound inside the Brain; they are reported as `audios` events ([Events](../30_core/30_events.md)). The Player receives those events and plays the sound with the engine's audio features.

## 1. Preparation

- **Load ahead of time.** Loading and decoding an audio file takes time. Loading when the event arrives delays the sound, so load and decode the audio in the ssab ahead of time, for example when you set up the animation.
- **Do not start audio until it is needed.** So that a project with no sound does not start the audio system (such as the Web Audio API's `AudioContext`) and spend resources on it, start it only when you find that the ssab contains audio.

## 2. Conditions for playing

- Play the sound named by the `audios` event's `sound_name`.
- **Do not play sound while playback runs in reverse.** Many audio APIs do not support playing audio backwards. Use `ss_runtime_is_playing_forward()` to decide. It returns the current direction of travel, reflecting the return leg of a ping-pong and the reverse setting. Speed never takes a negative value ([Playback Control §5-1](../30_core/20_playback_control.md#5-1-speed)), so there is no need to look at the sign of the speed.
- **What you suppress is the sound, not the notification.** Notify the host of `audios` events whatever the direction of travel. The audio handling decides whether to play. Events are also used for things other than playing sound (subtitles, vibration, logging). Suppressing the event itself hides that key from consumers other than sound as well. SSPlayerForGodot, SSPlayerForUnity and SSPlayerForRenPy all always emit the notification and suppress only their built-in playback.
- `loop_num` is named as a loop count, but it is **the number of times to play**. It is 1 or more, and the sound plays that many times. SpriteStudio audio has no infinite loop.

## 3. No sound on seek

The Player plays a sound **only when playback passes its key** on the timeline. Unlike the SpriteStudio editor, it does not align the sound's playback position on a seek.

However, a seek, whether playing or stopped, makes the next `update` the Player runs report the events (audio included) of the destination frame ([Events §3](../30_core/30_events.md#3-which-frames-are-reported)). If you do not want seeks (scrubs) to play sound, have the Player not play the audio events reported by the first `update` after the seek. The audio on the start frame right after `play` is played.

## 4. The lifetime of a sound

**Playback control does not own the lifetime of a sound that is playing.** What `pause` / `resume` / `stop` / setup end (or hold) is the *animation*; do not touch sounds that are already playing.

- Even if pausing also pauses the sound, it can only reach sounds already playing. A sound whose key would have been passed during the pause never plays, and `resume` does not bring it back. What was meant as a hold becomes a gap in the sound.
- `loop_num` is a play count with no infinite value. Every sound ends on its own eventually. A Player that silences sounds on stop or on an animation switch is not preventing a runaway sound; it is cutting short a sound that should play to the end. A sound that plays longer than its animation is a problem in the animation data.

**The only exception is when the Player itself goes away.** On destruction, disabling, or removal from the scene, stop every sound the Player started, because nothing is left to own those sounds. Also expose a "stop the sound now" API to the host. The point is that the Player does not decide on its own to stop sounds.

Every official Player follows this policy. Godot stops sounds on `NOTIFICATION_EXIT_TREE` and on destruction, Unity on `OnDisable` / `OnDestroy`, Web in `destroy()`, and Ren'Py on `hide` / `scene`; none of them stops sounds otherwise.

---
[Back to contents](../README.md)
