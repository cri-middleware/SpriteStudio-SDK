# SpriteStudio7-SDK

## Minimal operating rules

* Keep changes scoped to the requested task.
* Do not commit unless the user explicitly asks.
* Before editing, read files in full, especially if the read tool truncates them.
* Follow existing code style in touched files (naming, type usage, control flow, and error handling patterns).

## Project Overview

**SpriteStudio7-SDK** is a software development kit designed to work with [OPTPiX SpriteStudio 7](https://www.webtech.co.jp/eng/spritestudio/) projects. It provides a suite of tools and libraries for parsing project files (`.sspj`), converting them into an optimized binary format (`.ssab`), and playing back animations at runtime.

### Core Goals & Strategy

*   **Portability & Maintainability**: Centralize animation logic in a platform-agnostic Rust core (`libssruntime`) and use a common binary format (`ssab`) to simplify Player implementations across different game engines (Unity, Godot, Web, etc.).
*   **Performance & Efficiency**:
    *   **SSAB Format**: A FlatBuffers-based binary format that stores only change points (sparse data) to minimize file size and memory footprint.
    *   **Runtime Logic**: `libssruntime` handles complex interpolation (Hermite, Bezier) and state management, providing calculated local attributes to Players to avoid redundant logic in every engine.
*   **Workflow**:
    *   **SSConverter**: Converts XML-based `.sspj` projects to optimized `.ssab` files.
    *   **Player-Runtime Split**: `libssruntime` acts as the "Brain" (calculations), while the "Player" acts as the "Body" (rendering and engine integration).

## Strategic Architectural Advantages

SpriteStudio7-SDK is designed to provide professional-grade animation capabilities with a focus on the following architectural and functional strengths:

*   **Data-Oriented "Brain" Architecture**: Unlike monolithic runtimes that force a specific object-oriented structure, SS7-SDK provides a pure, stateless calculation engine. This allows seamless integration into modern **ECS (Entity Component System)** and Job Systems, providing the flexibility that professional engine developers crave.
*   **Zero-copy & SIMD Performance**: By utilizing **FlatBuffers** for zero-deserialization overhead and Rust's **SIMD** capabilities for batch interpolation, SS7-SDK delivers superior performance in high-density scenarios (e.g., thousands of characters) where traditional runtimes hit CPU and GC bottlenecks.
*   **Specialization in Cel-Animation & Effects**: While many 2D animation tools focus heavily on skeletal mesh deformation, SS7-SDK excels in high-fidelity **cel-animation** and integrated **particle simulation (SSEE)**. This "All-in-one" package is optimized for the "ケレン味" (stylized aesthetics) of Japanese animation.
*   **Cross-Platform Determinism**: A single Rust core ensures **100% bit-identical results** across all platforms (Mobile, Console, Web/Wasm). This eliminates platform-specific animation bugs and significantly reduces QA and maintenance costs for multi-platform projects.

### Roadmap (Ongoing & Future)

*   [x] Restructure and improve Player Implementation Documentation.
*   [x] Web (WASM) support for direct JS/TS usage via `wasm-pack`.
*   [ ] ECS Optimization: Stateless Evaluation API (Decomposing Context).
*   [x] Support for `no_std` environments (embedded/bare-metal).
*   [ ] Animation Mixing/Blending implementation in `ssruntime`.
*   [ ] Migration of existing Players to use `ssruntime`.
*   [ ] Support for SOA (Structure of Arrays) direct memory writing.
*   [ ] Batch processing with SIMD for high-performance entities update.
*   [ ] Batch processing support.
*   [ ] State machine implementation.

### Key Components

*   **`cli` (ssconverter-cli)**: A command-line tool that wraps `ssconverter` to convert SpriteStudio project files (`.sspj`) into binary animation files (`.ssab`). It supports batch processing and parallel conversion.
*   **`libs/sspj`**: A pure Rust library responsible for parsing the XML-based SpriteStudio project files (`.sspj`, `.ssae`, `.ssee`, etc.) using `quick-xml` and `serde`.
*   **`libs/ssconverter`**: The core conversion logic that transforms parsed `sspj` data into the `ssab` binary format. It utilizes `rayon` for parallel processing and exposes a C-API for integration.
*   **`libs/ssab`**: Defines the schema and structure for the `.ssab` binary format, utilizing **FlatBuffers** for efficient serialization and deserialization.
*   **`libs/ssinterpolate`**: A platform-agnostic math library for animation interpolations.
*   **`libs/ssruntime`**: A platform-agnostic runtime library for playing back `.ssab` animations. It handles playback control, interpolation, and state updates, exposing a C-API for integration into game engines (e.g., Unity, Unreal, custom engines). **Note:** It does not handle rendering directly; it provides the state required for a renderer.

## Architecture & Technologies

*   **Language**: Rust (Edition 2024)
*   **Build System**: Cargo (Workspace)
*   **Serialization**:
    *   **XML**: `quick-xml`, `serde` (for parsing project files)
    *   **Binary**: `flatbuffers` (for runtime data)
*   **CLI**: `clap`
*   **Concurrency**: `rayon` (for parallel conversion)
*   **FFI**: `cbindgen` is used to generate C/C++ headers for `ssconverter` and `ssruntime`.

## AI Agent Instructions & Constraints

When modifying code in this repository, you MUST adhere to the following rules:

*   **Automated Formatting & Linting:** AFTER editing any `.rs` files, you MUST run the following commands to ensure formatting and static analysis fixes are applied automatically:
    ```bash
    cargo clippy --workspace --all-targets --fix --allow-dirty --allow-staged -- -D warnings
    cargo fmt
    ```
    *(Note: `--allow-dirty` and `--allow-staged` are often needed for `cargo fix` when running in uncommitted workspaces).*
*   **no_std strictness:** `libssruntime` must remain `#![no_std]` compatible. NEVER introduce `std` dependencies or standard library calls (like `std::time`, `std::fs`, or `std::collections` that require `std`) into the runtime core. Use `alloc` or `core` instead.
*   **FFI Boundaries:** When modifying the C-API in `ssruntime` or `ssconverter`, you must ensure that memory safety is preserved across the FFI boundary. Avoid exposing Rust-specific structures directly.
*   **Serialization/FlatBuffers:** If you modify anything related to `.ssab` parsing, be aware that it relies on FlatBuffers zero-copy deserialization. Do not introduce allocations during the playback loop unless explicitly necessary.

## SpriteStudio Terminology (Glossary)

*   **`.sspj`**: The main SpriteStudio project file (XML format).
*   **`.ssae`**: Animation data file (XML format). Contains keyframes and part structures.
*   **`.ssce`**: Cell map data file (XML format). Defines how textures are sliced.
*   **`.ssee`**: Effect data file (XML format). Defines particle systems.
*   **`.ssab`**: The optimized binary format (FlatBuffers) compiled by `ssconverter`, used by `libssruntime`.
*   **Part**: A node in the animation hierarchy.
*   **Attribute**: Animatable properties of a Part (e.g., Position, Rotation, Alpha, UV translation).

## Building and Running

### Prerequisites

*   Rust toolchain (stable)
*   `rustup` (for cross-compilation targets if needed)

### Build Commands

*   **Build Release**:
    ```bash
    cargo build --release
    ```

*   **Build CLI Only**:
    ```bash
    cargo build -p ssconverter-cli --release
    ```

### Verification Commands (For Agents)

Run these commands to verify your changes:
*   **Linting:** `cargo clippy --workspace --all-targets -- -D warnings`
*   **Formatting:** `cargo fmt --all -- --check`
*   **Testing (All):** `cargo test --workspace`
*   **Testing (no_std explicitly):** `cargo build -p ssruntime --no-default-features --features ffi,libc_alloc,panic-handler`

### Updating `.fbs` Files (Code Generation)

If you modify or update the `.fbs` files in `libs/ssab/fbs/` or `libs/ssruntime/fbs/`, you MUST regenerate the Rust code.
Regeneration requires the `flatc` command (Google FlatBuffers Compiler) to be installed.

Run the corresponding script for your platform from the respective library directory:

**Windows (PowerShell):**
```powershell
./scripts/generate.ps1
```

**macOS / Linux (Bash):**
```bash
./scripts/generate.sh
```

### Release Scripts

Platform-specific release scripts are located in the `scripts/` directory. These scripts typically handle cross-compilation and bundling of universal binaries (e.g., using `lipo` on macOS).

*   `scripts/release-android.sh`
*   `scripts/release-ios-sim.sh`
*   `scripts/release-ios.sh`
*   `scripts/release-linux.sh`
*   `scripts/release-macos.sh`
*   `scripts/release-wasm.sh`
*   `scripts/release-web.sh`
*   `scripts/release-windows-arm64.ps1`
*   `scripts/release-windows.ps1`

## Development Conventions

*   **Workspace**: All crates are part of a single Cargo workspace. Dependencies between local crates are specified with `{ path = "..." }` or `{ workspace = true }`.
*   **Versioning**: The project uses centralized workspace versioning. The version is defined in the root `Cargo.toml` (`[workspace.package]`) and inherited by all member crates.
*   **Testing**: Unit tests are co-located with source code or in the `tests/` directory. The CLI has integration tests that run conversions on sample projects in `tests/`.
*   **FFI**: Libraries intended for external use (`ssconverter`, `ssruntime`) expose C-compatible APIs and use `cbindgen` for header generation.
*   **Formatting**: Standard `rustfmt` guidelines are enforced. Auto-formatting and linting (`clippy`) are checked in CI.

## Player Implementation Guidelines

The playback environment of SpriteStudio7-SDK is clearly separated into two components: **libssruntime** (Brain), which is responsible for the calculation logic, and the **Player** (Body), which handles the actual rendering and platform-dependent processing.

For detailed implementation steps and technical specifications, please refer to the following index file:

- [**Player Implementation Guide (Japanese)**](./libs/ssruntime/docs/README.ja.md)

### Core Design Philosophy
*   **libssruntime (Brain)**: The animation calculator. Responsible for keyframe interpolation, time management, and calculation of local attributes.
*   **Player (Body)**: The renderer. Responsible for parsing `ssab` metadata, managing textures, matrix transformations to world coordinates, and final rendering.
*   **Boundary**: `libssruntime` does not return metadata required for rendering (e.g., texture paths or part names). These must be obtained directly by the Player by referencing the `ssab` (FlatBuffers) data.
