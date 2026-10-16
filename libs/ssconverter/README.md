[**日本語**](./README.ja.md) | [**English**](./README.md)

# libssconverter

A library for converting SpriteStudio project files (`.sspj`) into runtime-optimized binary formats (`.ssab`, `.ssqb`).

It utilizes `sspj` to parse the project and `ssab` to generate FlatBuffers binaries.

# Features

- **Project Conversion**: Converts `.sspj` (project) to `.ssab` (animation) and `.ssqe` (sequence) to `.ssqb`.
- **Optimization**: Deduplicates data structures (like coordinates, colors, strings) during FlatBuffers serialization. This can significantly reduce the binary size (often by 50% or more for complex animations) and improves cache efficiency during runtime playback.
- **Embedded Mode**: Optionally embeds referenced assets (images, etc.) directly into the binary.
- **C-API / FFI**: Provides C-compatible interfaces for integration with other languages (C++, etc.).

# Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
ssconverter = "7.0.0-beta.1"
```

# FFI (C/C++)

This library exposes a C API for creating converters and monitoring progress.
For detailed integration instructions, see the [libssconverter Integration Guide](./docs/USAGE.md).

`ssconverter.h` is a C header that C++ can include as well. A build of `ssconverter-ffi` writes it, with the C# bindings `SSConverterMethods.g.cs`, to `libs/ssconverter-ffi/target/`. Committed copies of both are in `libs/ssconverter-ffi/bindings/`, to read without building, and a release's library zips carry them in `include/` and `bindings/`.


## Example C++ API Usage

```cpp
#include "ssconverter.h"

void convert() {
    Context* ctx = ss_converter_create();
    
    // Start conversion in a background thread
    // The 4th argument (embedded) packs textures into the .ssab; pass false for the conventional layout.
    // Pass nullptr for the logging callback if not needed
    ss_converter_convert(ctx, "path/to/project.sspj", "output/dir", false, nullptr);
    
    // Wait for finish
    while (ss_converter_is_running(ctx)) {
        // sleep or update progress
    }
    
    // Check result
    CConverterError result = ss_converter_get_result(ctx);
    if (result != CConverterError::Success) {
        const char* err_msg;
        size_t len;
        ss_converter_get_error(ctx, &err_msg, &len);
        printf("Error: %.*s\n", (int)len, err_msg);
    }
    
    ss_converter_destroy(ctx);
}
```


# Build Features

- **`ffi`**: Compiles the C-API, the `extern "C"` functions of the `ffi` module. Implies `native`. The static and dynamic libraries are built from `ssconverter-ffi`, the sibling crate that turns this feature on.
- **`native`** (default): Native build (CLI / Tauri backend) — parallel conversion, filesystem I/O, and directory scanning. Implies `parallel` and `scan`.
- **`parallel`**: Parallel conversion via `rayon`. Propagates to `sspj`'s parallel parsing.
- **`scan`**: Directory traversal (`walkdir`) for batch conversion.
- **`web`**: Browser WASM bindings (`wasm32-unknown-unknown`). Sequential, since `rayon` cannot spawn threads there. Build with `--no-default-features --features web`.

# Architecture

- **`converter`**: Core logic for mapping `sspj` data structures to `ssab` FlatBuffers schemas.
- **`cache`**: Implements caching and deduplication logic for FlatBuffers objects to reduce file size.
- **`ffi`**: C-API wrapper functions.

