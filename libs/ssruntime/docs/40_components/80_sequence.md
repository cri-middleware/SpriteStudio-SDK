> [日本語](./80_sequence.ja.md) | **English**

# Implementing Sequence

This is the feature that processes the "chained animation playlist" defined in a `.ssqb` file. `libssruntime` itself specializes in playing a single animation, so sequence management is handled by logic on the Player side.

- **Playback flow**:
    1. Detect the end of the current animation with `ss_runtime_is_finished` (see [Playback Control & Events](../30_core/20_playback_control.md)).
    2. Look at the next item in the sequence list.
    3. Set up the next animation with `ss_runtime_setup_animation` and continue playback.
- **Loop settings**: build a state machine on the Player side to control things like looping the whole sequence or looping only a particular item.

> **Future plan (Phase 3)**: the above is the Player-side implementation recommended at present. In the future we plan to bring sequence playback (a linear, auto-advancing state machine) inside `libssruntime`. See [ROADMAP.md](../../../../ROADMAP.md) for details.

---
[Back to Table of Contents](../README.md)
