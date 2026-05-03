# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [7.0.0-alpha01] - 2026-04-23
### Added
- **Initial Public Release** of the SpriteStudio7-SDK.
- **Unified Conversion Pipeline**: A complete workflow for converting SpriteStudio 7 XML projects (`.sspj`) into optimized FlatBuffers-based binary data (`.ssab`) via `ssconverter-cli`.
- **Platform-Agnostic Runtime**: A modular, `no_std` compatible runtime library (`ssruntime`) providing core animation playback logic with FFI and WASM support.
- **SS7.1 Phase 1 Parts**: Skew (shear), Nine-Slice, and Shape parts.
- **Fractional Frame Playback**: f32 frame numbers with no-bake playback.
- **Pure + Web Wasm Dual Mode**: deterministic `no_std` Wasm build alongside Web bindings via `wasm-bindgen`.
- **High-Performance Math Core**: SIMD-optimized transformation and interpolation logic (SSE/WASM SIMD128) for efficient frame calculations.
- **Flexible Integration**: Native support for multiple coordinate systems (Y-Up/Y-Down) and a stateless data model designed for easy integration with modern rendering engines.
- **Robust Event System**: A comprehensive event collection and management API, including support for custom user data and missed-event prevention.
- **Multi-Platform Support**: Validated CI/CD and release pipelines for Linux, Windows, macOS, Android, iOS, and Web (WASM).
- **C Headers in Release Artifacts**: `ssruntime.h` and `ssconverter.h` are bundled with releases.
- **Developer Documentation**: Bilingual documentation (English/Japanese) and detailed guidelines for implementing custom players.

### Known Limitations
- **Particle Effect (SSEE)**: Core simulation logic and emitter tree construction are currently unimplemented.
- **Font Bitmap**: Font-related properties and character map texture resolution are stubs.
- **Z-sort Priority**: Spatial Z-value transformation is not currently factored into part priority ordering.
- **Mesh Bind Info**: Conversion of vertex bind information for mesh deformation is not yet supported.
- **Code Signing**: Release binaries for macOS and iOS are not code-signed or notarized.
