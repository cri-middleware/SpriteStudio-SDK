> [日本語](./README.ja.md) | **English**

# SpriteStudio Player Porting Guide

This guide explains how to implement a SpriteStudio Player for any game engine or platform using `libssruntime` (the Brain). The Brain computes the animation state, and the Player loads the ssab, draws, notifies events and plays sounds ([Overview](./10_introduction/10_overview.md)).

> Every page of this guide is available in English, and each page has a Japanese source: the language selector in the site's header switches to it, and on GitHub each page links to it at the top. The Japanese version is the source of truth. If the two disagree, follow the Japanese page and report the discrepancy.

- **Read first**: [Limitations](./10_introduction/20_limitations.md) and [Player API Conventions](./20_design/30_api_conventions.md). Both shape the design of your Player.
- **How the pages are written**: Each page begins by stating what the Player does on that page. "Do …" and "Do not …" mark what you must follow; "We recommend …" marks what you may choose. Terms are used in the sense given in the [Glossary](./10_introduction/30_glossary.md).
- **Code examples**: They are written in C++ that calls the C-API. Where an example uses a function that does not exist (such as the engine's drawing routine), the example says so next to it.

---

## 1. Implementation steps

The SDK's `tests/overall/overall.sspj` contains an animation for each feature. Implement in the order below, and at each stage compare **whether it looks the same as in the SpriteStudio editor**. When it does, that stage is done.

| Stage | Animation to check | What to check | Pages to read |
| :--- | :--- | :--- | :--- |
| 0. Preparation | — | The build passes, and you can read animation names from the ssab | [Preliminary Checks](./20_design/10_preparation.md), [Design Approach](./20_design/20_architecture.md), [Loading and Initialization §5](./30_core/10_initialization.md#5-listing-animation-names) |
| 1. Basic drawing | `Basic.ssae` | Children follow the parent's movement, rotation and scaling. Lowering the parent's opacity also fades the children | [Loading and Initialization](./30_core/10_initialization.md), [Playback Control](./30_core/20_playback_control.md), [Render Loop](./30_core/40_render_loop.md) |
| 2. Blending and color | `BlendMode.ssae`, `PartsColor.ssae` | Additive and multiply blending composite correctly over the background. Tinting of the whole part and per vertex matches | [Parts Color](./40_parts/10_parts_color.md) |
| 3. Structure and deformation | `Mesh.ssae`, `Instance.ssae`, `Mask.ssae` | Mesh deformation and bone following are smooth. Child animations play. Only the area inside the mask is shown | [Mesh](./40_parts/20_mesh.md), [Instance Parts](./40_parts/50_instance.md), [Mask](./40_parts/40_mask.md) |
| 4. Integration with the game | `UserData.ssae`, `Signal.ssae` | Events arrive on the expected frames | [Events](./30_core/30_events.md) |
| 5. Effects | `Effect.ssae` | Particles scatter and fade out over time | [Effect Parts](./40_parts/60_effect.md) |
| 6. As needed | `NineSlice.ssae`, `Shape.ssae`, `Text.ssae`, `Sound.ssae`, `TextureChange.ssae`, `InstancePropagation.ssae` | The 9-slice corners are preserved when scaled up. Shapes and text appear. Sounds play on the expected frames. With texture change, two parts are drawn with different textures. A mask inside a child animation takes effect in the parent | [9-Slice, Shape, and Text](./40_parts/30_special_parts.md), [Audio](./50_features/50_audio.md), [External Resources and Textures](./50_features/30_external_resources.md), [Mask §2-7](./40_parts/40_mask.md#2-7-carry-over-of-clipping-written-inside-an-instance) |

---

## 2. Contents

### 10. Introduction

| Page | Contents |
| :--- | :--- |
| [Overview](./10_introduction/10_overview.md) | The division of work between Brain and Player, the audience, how to read the code examples |
| [Limitations](./10_introduction/20_limitations.md) | Technical constraints that affect Player design. **Read before implementing** |
| [Glossary](./10_introduction/30_glossary.md) | The terms used in this guide |

### 20. Design

| Page | Contents |
| :--- | :--- |
| [Preliminary Checks and Project Setup](./20_design/10_preparation.md) | Checking whether the Brain can be used on your target engine, and preparing an empty project |
| [Player Design Approach](./20_design/20_architecture.md) | How to integrate the library, ssab memory, class structure, drawing approach, coordinate systems |
| [Player API Conventions](./20_design/30_api_conventions.md) | The names, argument order, events, defaults and failure reporting that every Player shares. **Read before writing your public API** |
| [Integrating into no_std Environments](./20_design/40_no_std_integration.md) | Building it yourself, and using the engine's allocator |

### 30. Basic playback

| Page | Contents |
| :--- | :--- |
| [Loading and Initialization](./30_core/10_initialization.md) | Resources and Contexts, ssab validation, loading textures, logging and errors |
| [Playback Control](./30_core/20_playback_control.md) | The flow of one frame, playback state, speed, sub-frames, detecting loops and the end |
| [Events](./30_core/30_events.md) | How to read user data, signal and audio events |
| [Render Loop](./30_core/40_render_loop.md) | SoA buffers, drawing with DrawBatch, RenderMode, performance |

### 40. Parts

| Page | Contents |
| :--- | :--- |
| [Parts Color](./40_parts/10_parts_color.md) | Compositing vertex colors with the texture, shader examples |
| [Mesh](./40_parts/20_mesh.md) | How to read the deformed vertices |
| [9-Slice, Shape, and Text](./40_parts/30_special_parts.md) | How to read their dedicated buffers, and the text layout specification |
| [Mask](./40_parts/40_mask.md) | The rules that decide masking and clipping, and implementations with the stencil and a coverage bitmap |
| [Instance Parts](./40_parts/50_instance.md) | Preparing, advancing and drawing child animations; independent playback |
| [Effect Parts](./40_parts/60_effect.md) | Advancing and drawing particles |

### 50. Features

| Page | Contents |
| :--- | :--- |
| [Animation Mixing](./50_features/10_animation_mixing.md) | Stacking animations on tracks to overlay, switch and blend them |
| [Part Overrides](./50_features/20_part_overrides.md) | Overriding a part's color, cell and visibility at runtime |
| [External Resources and Textures](./50_features/30_external_resources.md) | Replacing textures, and referencing another ssab |
| [Sequence](./50_features/40_sequence.md) | Playing several animations in order |
| [Audio](./50_features/50_audio.md) | When to play a sound, and how long a sound lives |
| [Editor Integration](./50_features/60_editor_integration.md) | Converting sspj in the engine's editor |
| [Placement: Fit and Align](./50_features/70_placement.md) | Fitting the animation into a box the host laid out |
| [UI Integration: Hit Testing, UI States and Reduced Motion](./50_features/80_ui_integration.md) | Making the animation a UI component, such as a button |

### 60. Troubleshooting

| Page | Contents |
| :--- | :--- |
| [Troubleshooting](./60_troubleshooting/10_troubleshooting.md) | A table that leads from symptoms to causes, and link errors |

---
[SpriteStudio-SDK](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/README.md) / [libssruntime](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/libs/ssruntime/README.md)
