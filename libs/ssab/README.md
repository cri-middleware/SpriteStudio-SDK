[**日本語**](./README.ja.md) | [**English**](./README.md)

# libssab

A Rust library for reading and writing SpriteStudio 7 animation binary data (`.ssab`, `.ssqb`).
It uses Google FlatBuffers for serialization and deserialization.

# Supported Formats

This library supports the following two format definitions (`fbs`):

- **SSAB ([ssab.fbs](./fbs/ssab.fbs))**: Animation data (`SsAnimeBinary`)
- **SSQB ([ssqb.fbs](./fbs/ssqb.fbs))**: Sequence data (`SsSequenceBinary`)

# Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
ssab = { path = "libs/ssab" }
```

# Features

You need to switch `features` depending on your use case.

## `default` (Runtime)
The default state. Optimized for reading data (deserialization).
Suitable for game runtimes or players that only need to playback data.

- **Characteristics**: Read-only access, Zero-copy reading.

```toml
[dependencies]
ssab = { path = "libs/ssab" }
```

## `no_std` Support
`libssab` is compatible with `no_std` environments. To use it, disable the default features:

```toml
[dependencies]
ssab = { path = "libs/ssab", default-features = false }
```

## `converter`
Mode for tools that generate or edit data.
Enables FlatBuffers Object API (`--gen-object-api`), allowing manipulation of data as Rust structs and writing back to binary.

- **Characteristics**: Read/Write access, Object API (T-structs) available.
```toml
[dependencies]
ssab = { path = "libs/ssab", features = ["converter"] }
```

# Usage

## Reading Data (Runtime)

```rust
use std::fs;
use ssab::ssab::ss::format::get_root_as_ss_anime_binary;

fn main() {
let bytes = fs::read("test.ssab").unwrap();

    // Get the root object from the binary
    let anime = get_root_as_ss_anime_binary(&bytes).unwrap();

    println!("Anime Name: {}", anime.name().unwrap_or("unknown"));
    println!("Total Frames: {}", anime.total_frame());
}
```

## Creating/Writing Data (Converter)

You must enable the `converter` feature.

```rust
#[cfg(feature = "converter")]
fn create_ssab() {
use flatbuffers::FlatBufferBuilder;
use ssab::ssab::ss::format::{SsAnimeBinary, SsAnimeBinaryT};

    // Build data using Object API (T-struct)
    let mut anime_obj = SsAnimeBinaryT::default();
    anime_obj.name = Some("MyAnimation".to_string());
    anime_obj.fps = 60;
    anime_obj.total_frame = 100;

    // Serialize with FlatBufferBuilder
    let mut builder = FlatBufferBuilder::new();
    let offset = anime_obj.pack(&mut builder);
    builder.finish(offset, Some(SsAnimeBinary::IDENTIFIER));

    let buf = builder.finished_data();
    // Write buf to a file, etc...
}
```

# Development & Code Generation

If you modify the `fbs` files, you need to regenerate the Rust code.
Regeneration requires the `flatc` command (Google FlatBuffers Compiler) to be installed.

## Windows (PowerShell)

```powershell
./scripts/generate.ps1
```

## macOS / Linux (Bash)

```bash
./scripts/generate.sh
```

These scripts perform the following steps:

1. Compile definition files in the `fbs` folder.
2. Generate source code for `runtime` and `converter` into `src/runtime` and `src/converter` respectively.
3. Apply Object API generation options and other settings to the `converter` code.

