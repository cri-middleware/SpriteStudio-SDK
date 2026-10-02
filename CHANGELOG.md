# Changelog

All notable changes to this project are documented in this file.
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Entries follow [Conventional Commits](https://www.conventionalcommits.org/); newest first.

## v7.0.0-beta.1 - Unreleased

### Adds APIs / Features
- **Initial Public Release**: SpriteStudio-SDK (`ssconverter-cli` + `libssruntime`), supporting SpriteStudio.
- **Core Runtime**: Engine-agnostic, `#![no_std]` brain with zero-math integration and SIMD (f32x4) state computation.
- **Features**: Supports Mesh Deformation, Particles (SSEE), Instances, and SS7.1 parts (Skew, 9-Slice, Shape).
- **Integrations**: C-API and Web (WASM) ready.
- **Animation Mixing**: a track stack. Tracks fold onto the setup pose in ascending order, each with its own `alpha` and an interpolative or additive operator; several animations can share a track and mix by weight. Covers crossfades (`ss_runtime_crossfade`), overlays, additive offset axes, and fading a track out to the rest pose.
- **Custom Allocator Injection**: `custom_alloc` feature lets game engines / consoles supply their own allocator and abort handler via `ss_custom_alloc` / `ss_custom_free` / `ss_custom_abort` C symbols — no wrapper crate required.
- **Documentation**: Overhauled bilingual (EN/JA) README and contribution guidelines.

### Updates
- **Minimum Supported Rust Version**: building from source requires Rust 1.99 or newer.
