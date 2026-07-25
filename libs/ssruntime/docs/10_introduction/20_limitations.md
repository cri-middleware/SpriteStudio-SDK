> [日本語](./20_limitations.ja.md) | **English**

# Limitations

Read this **before** you start implementing a Player.
It collects the things that directly affect Player design decisions: `ssruntime` / `ssab` specification constraints, WebAssembly environment constraints, and format constraints.

> [!IMPORTANT]
> This document doubles as an index of limitations. For implementation-level detail (code examples, recommended patterns), follow the link at the end of each item.

## Table of Contents

- [Supported Scope of Drawing and Playback Features](#supported-scope-of-drawing-and-playback-features)
- [WebAssembly (WASM) Environment](#webassembly-wasm-environment)
- [Reverse Playback and Seeking of `independent=true` Parts](#reverse-playback-and-seeking-of-independenttrue-parts)
- [Seeking (`set_frame_no`) Does Not Fire Events on Skipped Frames](#seeking-set_frame_no-does-not-fire-events-on-skipped-frames)
- [File References and Resource Resolution](#file-references-and-resource-resolution)
- [No Validation of Input Data (Security Note)](#no-validation-of-input-data-security-note)
- [Lifetime of AnimationResource and Context](#lifetime-of-animationresource-and-context)
- [Limitations of ssab Embedded Mode](#limitations-of-ssab-embedded-mode)
- [Limitations When Blending Animations](#limitations-when-blending-animations)
- [No Reverse Playback of Audio](#no-reverse-playback-of-audio)

---

## Supported Scope of Drawing and Playback Features

`ssruntime` is the layer that computes **animation state**. It does **not** provide the actual implementation of the features below - it exposes information-retrieval APIs only, so the Player has to implement them in cooperation with the host engine.

| Feature | What `ssruntime` provides | What the Player must implement |
| --- | --- | --- |
| **Text drawing** | Only what the converter stored in the ssab: the part's authored text (`PartTypeText`) and, for a bitmap-font project, its glyph atlas (`FontBitmap` / `FontGlyphBitmap`). `ssruntime` neither lays glyphs out nor rasterizes them | All of the drawing — glyph layout, rasterization for outline fonts, and the draw call — read straight from the ssab and rendered with the host engine's text stack |
| **Sound playback** | Playback instruction events (file name, trigger timing) | Actual audio playback (integration with the engine's audio system) |

> **How a text part reaches the Player.** It holds its slot in the draw plan: a `DrawBatchKind::Text` batch is emitted for each one — one part per batch, since the runtime cannot tell which font a part draws with — so text composites by priority alongside every other part. The batch carries no geometry (`vertex_count` / `index_count` are `0`) and no `texture_hash`; the Player builds the glyphs and resolves the font from the ssab. **A text part authored as a mask (`draw_as_mask`) is left out of the plan entirely**, because the runtime has no glyphs to write coverage with.

> Related chapters: [Playback Control & Events](../30_core/20_playback_control.md) / [Audio](../40_components/90_audio.md)

## WebAssembly (WASM) Environment

The WASM build of `ssruntime` uses **WebAssembly SIMD** instructions.

- **It will not run on browsers without SIMD support.**
- Treat WebAssembly SIMD support as a browser requirement for your distribution.

## Reverse Playback and Seeking of `independent=true` Parts

Setting `independent=true` on an Instance or Effect part **does not work correctly under reverse playback or seeking**.

- By design, an `independent=true` child is detached from the parent timeline and advances on a real-time clock (`delta_time`).
- It breaks under the parent's **reverse playback** (the child's clock increases monotonically and cannot run backwards) and under the parent's **seeking** (the child's "accumulated time" at an arbitrary past frame is not uniquely determined).
- The parent's **forward frame skips** are supported.

**Recommended practice**: author your SS7 animations without `independent=true`. If you must use it, tell your Player's users explicitly that the corresponding child instances are only drawn correctly during forward playback.

> Related chapter: [Instance Parts](../40_components/10_instance.md)

## Seeking (`set_frame_no`) Does Not Fire Events on Skipped Frames

When you jump the playback position directly with `ss_runtime_set_frame_no`, the **events (UserData / Signal / Audio) on the frames between the current frame and the destination frame are not fired**. The next `update()` collects only the events of the single destination frame.

If you need to handle every intermediate event, the Player needs a wrapper implementation - for example calling `update(delta)` several times according to the distance moved, or scanning the events of the seeked range directly from the ssab and firing them yourself.

> Related chapter: [Playback Control & Events](../30_core/20_playback_control.md)

## File References and Resource Resolution

The composition rules for the file-reference strings stored in the ssab (textures, sounds, fonts, and so on) are a specification, not a limitation. As a prerequisite for Player implementation, see **[Loading & Initialization](../30_core/10_initialization.md) §3.0 "Specification of the file-reference strings stored in the ssab"**.

## No Validation of Input Data (Security Note)

To maximize performance and minimize binary size (especially for embedded / WebAssembly targets), `ssruntime` performs **no structural validation** of the input `.ssab` data.

- Loading maliciously crafted or corrupted `.ssab` data risks undefined behavior or a crash, for example through out-of-bounds memory access.
- **Countermeasure**: in environments that load data dynamically over a network or from user input, either verify the data's integrity on the Player side beforehand, or design the Player so that it loads only trusted, safe assets.

## Lifetime of AnimationResource and Context

To optimize memory efficiency, a single animation resource (`AnimationResource`) is designed to be **shared** by multiple playback contexts (`Context`).

- After binding a resource to a Context with `ss_runtime_bind_resource`, **do not destroy the original resource before the Context is destroyed or another resource is bound**.
- Violating this produces a dangling pointer (use-after-free) and causes crashes.

## Limitations of ssab Embedded Mode

When you use ssab in **embedded mode** (generating an ssab with the assets embedded in the ssab itself), the following limitations apply.

### 1. Key collisions on asset file paths

ssab embedded uses the **asset file paths** recorded in the `sspj` and `ss*e` files **as keys**.

- If the same file path appears more than once, a **key collision** occurs.
- When using embedded mode, structure your project so that asset file paths within it are **not duplicated**.

### 2. Only the assets actually used are embedded (external instance bodies are not included)

ssab embedded embeds **only the assets that the animation pack actually uses**. The body of another ssab file referenced as an external instance is not embedded, so **when you use external instances you must supply the referenced ssab files separately**.

## Limitations When Blending Animations

Constraints that apply when compositing several animations with `ss_runtime_add_blend_source` and friends (Phase 3 API).

- **Blending is possible only within the same SSAB**: compositing across multiple `AnimationResource`s is out of scope.
- **Cell consistency is a prerequisite**: every blended animation must "assign the same Cell to the same part". `cell_meta_per_part` is taken from the primary source, so differing Cells break the pose through size / pivot mismatches. Animations within the same SSAE are normally fine.
- **Double definition of `parts_color` alpha**: the visual alpha of a part that has `parts_color` comes from the primary source (not the mixed `state.alpha`). Parts without `parts_color` reflect the mixed `state.alpha`, so alpha blending itself works.
- **Event firing is controlled by a per-source threshold**: by default only the Primary fires (`event_threshold = 1.0`). To let a Secondary fire as well, lower the threshold with `ss_runtime_set_source_event_threshold`; only the events of sources satisfying `source.weight >= source.event_threshold` are collected. The Primary ignores the threshold and always fires. Identify the source an event came from with `ss_runtime_get_passed_event_source_handle` (a parallel array to the events).
- **FrameData grows when blending**: when several sources are blended, the deform data of secondary sources that are not drawn is also included in the FrameData, inflating its size. This does not affect correctness.

> Related chapter: [Animation Blending](../40_components/60_animation_blending.md)

## No Reverse Playback of Audio

We recommend restricting audio playback to the times when **the animation is advancing forward along the time axis**.

- Most audio engines (the Web Audio API and others) do not support playing an audio file in reverse as standard.
- In your Player implementation, compare the runtime's `is_playing_forward()` (the current direction, accounting for the outbound / return legs of PingPong and the Backward setting) with the sign of the user-set playback speed (`speedScale`), and process `audios` events only when the effective playback direction is forward.

> Related chapter: [Audio](../40_components/90_audio.md)

---
[Back to Table of Contents](../README.md)
