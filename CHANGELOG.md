# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [7.0.0] - 2026-04-12
### Added
- **Initial Public Release** of the SpriteStudio7-SDK.
- `ssconverter-cli`: Command-line tool for converting `.sspj` projects to optimized `.ssab` binary files. Supports single and batch processing modes.
- `libsspj`: Library for parsing SpriteStudio 7 XML project files (`.sspj`, `.ssae`, `.ssee`, `.ssse`, `.ssce`, `.charmap`).
- `libssab`: Library for reading and writing SpriteStudio 7 animation binary data (`.ssab`, `.ssqb`) based on FlatBuffers.
- `libssconverter`: Core conversion library from `.sspj` to `.ssab` format with C-API support.
- `libssruntime`: Platform-agnostic runtime library for animation playback with FFI (C/C++/C#) and WASM support.
- `libssinterpolate`: Math library for animation interpolations (Hermite, Bezier, etc.).
- `no_std` support for `ssruntime` with `libc_alloc` feature for embedded and console environments.
- Multi-platform CI/CD pipeline: Linux, Windows, macOS, Android, iOS, Web (WASM).
- Multi-platform release pipeline with static and dynamic library outputs.
- Bilingual documentation (English / Japanese) for all components.
- Player implementation guidelines in `libs/ssruntime/docs/`.
