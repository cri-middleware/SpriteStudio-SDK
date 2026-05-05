# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [7.0.0-alpha01] - 2026-05-05
### Added
- **Initial Public Release** of the SpriteStudio7-SDK.
- **Unified Conversion Pipeline**: A complete workflow for converting SpriteStudio 7 XML projects (`.sspj`) into optimized FlatBuffers-based binary data (`.ssab`) via `ssconverter-cli`.
- **Platform-Agnostic Runtime**: A modular, `no_std` compatible runtime library (`ssruntime`) providing core animation playback logic with FFI and WASM support.
- **Advanced Part Support**:
  - **Instance Parts**: Full support including parent-driven timeline control (`set_frame_relative`) and `reverse` playback flags.
  - **SS7.1 Phase 1 Parts**: Skew (shear), Nine-Slice, and Shape parts (including accurately generated complex geometries).
- **Rendering & Transformation Core**:
  - **High-Performance Math**: SIMD-optimized transformation and interpolation logic (SSE/WASM SIMD128) for efficient frame calculations, including vectorized 9-slice and shape vertex computation.
  - **Render-Ready Attributes**: Pre-computed local vertices, shape vertices, UVs, and cell texture hashes exposed via FFI to simplify engine integration.
  - **Optimized Metadata Resolution**: FFI access to per-part cell metadata (texture index, UV rect, etc.) that is completely pre-resolved at setup time, eliminating per-frame lookups.
  - **Z-sort Priority**: Fully implemented spatial Z-value transformation and priority ordering via `ZOrderTable`.
  - **Fractional Frame Playback**: Support for `f32` frame numbers with no-bake playback and robust update logic across frames.
- **Ecosystem Integration**:
  - **Pure + Web Wasm Dual Mode**: Deterministic `no_std` Wasm build alongside Web bindings via `wasm-bindgen`.
  - **Native Coordinate Support**: Built-in support for multiple coordinate systems (Y-Up/Y-Down) and a stateless data model.
  - **Robust Event System**: A comprehensive event collection API with support for custom user data and missed-event prevention.
- **Multi-Platform Support**: Validated CI/CD and release pipelines for Linux, Windows, macOS, Android, iOS, and Web (WASM).
- **C Headers**: `ssruntime.h` and `ssconverter.h` bundled with release artifacts.
- **Developer Documentation**: Bilingual documentation (English/Japanese) and detailed guidelines for implementing custom players.

### Known Limitations
- **Particle Effect (SSEE)**: Rendering and full integration are pending validation on the player side (e.g., Godot).
- **Text & Font Bitmap**: BMFont conversion is implemented, but player-side rendering and text properties are pending.
- **Mesh Bind Info**: Conversion of vertex bind information for mesh deformation is not yet supported.
- **Code Signing**: Release binaries for macOS and iOS are not code-signed or notarized.
