[**日本語**](./README.ja.md) | [**English**](./README.md)

# libssconverter

A library for converting SpriteStudio 7 project files (`.sspj`) into runtime-optimized binary formats (`.ssab`, `.ssqb`).

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
ssconverter = { path = "libs/ssconverter" }
```

# Usage (Rust)

```rust
use std::path::Path;
use ssconverter::{convert_sspj_to_ssab_files, ConverterConfig, ConvertConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sspj_path = Path::new("path/to/project.sspj");
    
    let config = ConverterConfig {
        convert_config: ConvertConfig {
            optimize: true,   // Enable deduplication
            embedded: false,  // Do not embed textures
            ..Default::default()
        },
        output: None, // Use default output directory (e.g. project/)
    };

    // Convert and save files (with a logging callback)
    convert_sspj_to_ssab_files(sspj_path, &config, &|msg| println!("{}", msg))?;
    
    Ok(())
}
```

# FFI (C/C++)

This library exposes a C API for creating converters and monitoring progress.
For detailed integration instructions, see the [libssconverter Integration Guide](./docs/USAGE.md).

Headers are generated in `target/ssconverter.h` by `cbindgen`.


## Example C++ API Usage

```cpp
#include "ssconverter.h"

void convert() {
    ss_converter_context* ctx = ss_converter_create();
    
    // Start conversion in a background thread
    // Pass nullptr for the logging callback if not needed
    ss_converter_convert(ctx, "path/to/project.sspj", "output/dir", nullptr);
    
    // Wait for finish
    while (ss_converter_is_running(ctx)) {
        // sleep or update progress
    }
    
    // Check result
    CConverterError result = ss_converter_get_result(ctx);
    if (result != CConverterError::Success) {
        char* err_msg;
        size_t len;
        ss_converter_get_error(ctx, &err_msg, &len);
        printf("Error: %.*s\n", (int)len, err_msg);
    }
    
    ss_converter_destroy(ctx);
}
```


# Build Features

- **`ffi`** (default): Enables C-API and static/dynamic library generation.
- **`web`**: Enables WASM support (experimental).

# Architecture

- **`converter`**: Core logic for mapping `sspj` data structures to `ssab` FlatBuffers schemas.
- **`cache`**: Implements caching and deduplication logic for FlatBuffers objects to reduce file size.
- **`ffi`**: C-API wrapper functions.

