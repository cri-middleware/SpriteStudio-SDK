[**日本語**](./README.ja.md) | [**English**](./README.md)

# libssab

A Rust library for reading and writing SpriteStudio animation binary data (`.ssab`, `.ssqb`).
It uses Google FlatBuffers for serialization and deserialization.

# Supported Formats

This library supports the following two format definitions (`fbs`). For detailed design specifications, see the [**Schema Documentation**](./SCHEMA.md).

- **SSAB ([ssab.fbs](./fbs/ssab.fbs))**: Animation data (`SsAnimeBinary`)
- **SSQB ([ssqb.fbs](./fbs/ssqb.fbs))**: Sequence data (`SsSequenceBinary`)

# Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
ssab = "7.0.0-beta.1"
```

# Features

You need to switch `features` depending on your use case.

## `default` (Runtime)
The default state. Optimized for reading data (deserialization).
Suitable for game runtimes or players that only need to playback data.

- **Characteristics**: Read-only access, Zero-copy reading.

```toml
[dependencies]
ssab = "7.0.0-beta.1"
```

## `no_std` Support
`libssab` is compatible with `no_std` environments. To use it, disable the default features:

```toml
[dependencies]
ssab = { version = "7.0.0-beta.1", default-features = false }
```

## `converter`
Mode for tools that generate or edit data.
Enables FlatBuffers Object API (`--gen-object-api`), allowing manipulation of data as Rust structs and writing back to binary.

- **Characteristics**: Read/Write access, Object API (T-structs) available.
```toml
[dependencies]
ssab = { version = "7.0.0-beta.1", features = ["converter"] }
```

# Usage

## Reading Data (Runtime)

```rust
use std::fs;
use ssab::ssab::ss::format::root_as_ss_anime_binary;

fn main() {
    let bytes = fs::read("test.ssab").unwrap();

    // Get the root object from the binary. `root_as_*` verifies the buffer;
    // the `_unchecked` variants skip verification.
    let binary = root_as_ss_anime_binary(&bytes).unwrap();

    println!("Name: {}", binary.name());
    for anime in binary.animations() {
        println!("  {} ({} frames @ {} fps)", anime.name(), anime.total_frame(), anime.fps());
    }
}
```

## Creating/Writing Data (Converter)

You must enable the `converter` feature.

```rust
#[cfg(feature = "converter")]
fn create_ssab() {
    use flatbuffers::FlatBufferBuilder;
    use ssab::ssab::ss::format::{AnimationDataT, SsAnimeBinaryT, finish_ss_anime_binary_buffer};

    // Build data using Object API (T-structs). They are `#[non_exhaustive]`,
    // so start from `default()` and assign fields.
    // Per-animation properties such as fps live on `AnimationDataT`.
    let mut anime = AnimationDataT::default();
    anime.name = "MyAnimation".to_string();
    anime.fps = 60;
    anime.total_frame = 100;

    // `SsAnimeBinaryT` is the root: it holds the parts and the animation list.
    let mut binary = SsAnimeBinaryT::default();
    binary.name = "MyProject".to_string();
    binary.animations = vec![anime];

    // Serialize with FlatBufferBuilder. The helper writes the "SSAB" file
    // identifier, which `root_as_ss_anime_binary` checks for.
    let mut builder = FlatBufferBuilder::new();
    let offset = binary.pack(&mut builder);
    finish_ss_anime_binary_buffer(&mut builder, offset);

    let buf = builder.finished_data();
    // Write buf to a file, etc...
}
```

# Development & Code Generation (Updating fbs files)

If you modify or update the `.fbs` files in the `fbs` folder, you need to regenerate the Rust code.
Regeneration requires the `flatc` command (Google FlatBuffers Compiler) to be installed.

## Update Procedure

1. Install `flatc` (Google FlatBuffers Compiler).
2. Update or modify the `.fbs` files in the `fbs/` folder.
3. Run the following script to regenerate the Rust code.

### Windows (PowerShell)

```powershell
./scripts/generate.ps1
```

### macOS / Linux (Bash)

```bash
./scripts/generate.sh
```

These scripts perform the following steps:

1. Compile definition files in the `fbs` folder.
2. Generate source code for `runtime` and `converter` into `src/runtime` and `src/converter` respectively.
3. Apply Object API generation options and other settings to the `converter` code.

