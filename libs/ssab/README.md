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
The default state enables the `std` feature (`flatbuffers/std`). Optimized for reading data (deserialization) and suitable for runtimes or tools running with the standard library.

- **Characteristics**: Read-only access, Zero-copy reading.

```toml
[dependencies]
ssab = "7.0.0-beta.1"
```

## `no_std` Support
`libssab` is compatible with `no_std` environments (`core` + `alloc`). Disable default features to drop the `std` dependency:

```toml
[dependencies]
ssab = { version = "7.0.0-beta.1", default-features = false }
```

## `converter`
Mode for tools that generate or edit data (implies `std`).
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

You must enable the `converter` feature. The example also uses the `flatbuffers` crate directly, which `ssab` does not re-export: add it to `[dependencies]` at the version in the SDK's root `Cargo.toml`.

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
    // The runtime refuses a pack whose version is not this one.
    binary.version = ssab::SSAB_SCHEMA_VERSION;
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

When you edit `.fbs` files under `fbs/`, regenerate the committed Rust bindings in `src/runtime/` and `src/converter/` with `./scripts/generate.sh` (or `./scripts/generate.ps1` on Windows). The `flatc` release to use is in [CONTRIBUTING.md](../../CONTRIBUTING.md#prerequisites).

