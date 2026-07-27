> [日本語](./README.ja.md) | **English**

# SpriteStudio 7 Player Porting Guide

This guide explains how to implement a **SpriteStudio 7 Player** for your own game engine or platform using `libssruntime` (the Brain).

> Every page of this guide is available in English; each page links to its Japanese source at the top. The Japanese version remains the source of truth — if the two ever disagree, follow the Japanese page and please report the discrepancy.

## 📚 Table of Contents

> [!IMPORTANT]
> **Before** you start implementing, read [**Limitations**](./10_introduction/20_limitations.md). The support scope and spec constraints directly affect Player design decisions.

### 10. Introduction
- [**Introduction & Glossary**](./10_introduction/10_introduction.md) — core concepts and terminology
- [**Limitations**](./10_introduction/20_limitations.md) — support scope and spec constraints (**read first**)

### 20. Design
- [**Preparation & Prerequisites**](./20_design/10_preparation.md) — environment setup (a prerequisite to implementation)
- [**Architecture & Implementation Approach**](./20_design/20_architecture.md) — Brain (compute) vs. Body (render)
- [**Implementation Roadmap**](./20_design/30_implementation_roadmap.md) — staged implementation/verification using `overall.sspj`

### 30. Core (Basic Playback)
Steps to get a minimal animation on screen and control it.
- [**Loading & Initialization**](./30_core/10_initialization.md)
- [**Playback Control & Events**](./30_core/20_playback_control.md)
- [**Rendering Loop & Batching**](./30_core/30_render_loop.md)

### 40. Components (Advanced Expression)
Per part-type implementation details for richer expression.
- [**Instance Parts**](./40_components/10_instance.md)
- [**Effect Parts**](./40_components/20_effect.md)
- [**Mesh & Deform**](./40_components/30_mesh.md)
- [**Mask**](./40_components/40_mask.md)
- [**Parts Color**](./40_components/50_parts_color.md)
- [**Animation Blending**](./40_components/60_animation_blending.md)
- [**Part Overrides (Override Layer API)**](./40_components/65_override_layer.md) — per-part runtime overrides for color / cell / visibility (Phase 2)
- [**External Resources & Textures**](./40_components/70_external_resources.md) — texture resolution / authored texture change (TCHG) / external SSAB references
- [**Sequence**](./40_components/80_sequence.md)
- [**Audio**](./40_components/90_audio.md)
- [**Special Part Types**](./40_components/95_special_parts.md)

### 50. Reference
- [**no_std Integration**](./50_reference/10_no_std_integration.md)
- [**Troubleshooting**](./50_reference/20_troubleshooting.md)

### 60. Tooling
- [**Editor Integration & Conversion**](./60_tooling/10_editor_integration.md) — integrating `ssconverter` into the engine editor

---
> [!TIP]
> Run **`tests/overall/overall.sspj`** at each implementation stage. If it looks the same as the editor, that step is done!

[SpriteStudio-SDK Top](../../../README.md) / [libssruntime Top](../README.md)
