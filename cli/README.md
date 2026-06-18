[**日本語**](./README.ja.md) | [**English**](./README.md)

# ssconverter-cli

The build-time tool of the [SpriteStudio-SDK](https://github.com/cri-middleware/SpriteStudio-SDK/blob/main/README.md) workflow: parses SpriteStudio 7 project files (`.sspj` and the referenced `.ssae` / `.ssee` / `.ssqe` / `.ssce`) and emits the runtime binaries the SDK plays back — `.ssab` (animation) and `.ssqb` (sequence pack).

This CLI wraps [libssconverter](https://github.com/cri-middleware/SpriteStudio-SDK/tree/main/libs/ssconverter) and exposes the same conversion pipeline that the SDK's release artifacts are built with. The runtime side lives in [libssruntime](https://github.com/cri-middleware/SpriteStudio-SDK/tree/main/libs/ssruntime).

# Usage

`ssconverter-cli` operates in two main modes: **Single Mode** and **Batch Mode**.

## Single Mode

Converts a single `.sspj` project file.
In this mode, you can specify an output directory.

```bash
# Convert a single project
$ ssconverter-cli project.sspj

# Convert and specify output directory
$ ssconverter-cli project.sspj -o ./output_folder
```

### Options (Single Mode)
- `-e`, `--embedded`: Embed referenced assets (images, etc.) directly into the generated `.ssab` file.
- `-o`, `--out-dir <directory>`: Specify the output directory for generated files.
- `-h`, `--help`: Print help information.

## Batch Mode

Converts multiple `.sspj` projects at once.
You can pass multiple file paths or directories containing `.sspj` files.
In this mode, the `--out-dir` option is ignored; each output is written next to its source `.sspj` in `<name>/`.

```bash
# Convert multiple specific files
$ ssconverter-cli project1.sspj project2.sspj

# Convert all sspj files found in directories (recursive)
$ ssconverter-cli ./assets/projects ./extra_assets
```

### Options (Batch Mode)
- `-e`, `--embedded`: Embed referenced assets directly into the generated `.ssab` files.
- `-h`, `--help`: Print help information.

_Note: The `--out-dir` option is not available in batch mode._

# Installation

## Prebuilt binaries (recommended)

Download a release for **Windows**, **macOS**, or **Linux** from the [GitHub Releases](https://github.com/cri-middleware/SpriteStudio-SDK/releases) page. No build environment required.

## Build from source

Requires a Rust toolchain ([rustup](https://rustup.rs/)). From the repository root:

```bash
$ cargo install --path cli
```

# Examples

**1. Basic Conversion**

Generates `project/` folder next to `project.sspj`.

```bash
$ ssconverter-cli my_game/project.sspj
```

**2. Embedded Assets**

Creates a single binary containing all textures, useful for simple distribution.

```bash
$ ssconverter-cli my_game/project.sspj --embedded
```

# Output

For each `.sspj` converted, a directory is produced next to the source file (unless `--out-dir` is set in single mode):

| Mode | Output directory | Contents |
|---|---|---|
| Default | `<name>/` | `.ssab` (one per animation), `.ssqb` (per sequence pack, if `.ssqe` is referenced), and copies of referenced textures |
| `--embedded` | `<name>_emb/` | Same as above, but referenced assets are inlined into each `.ssab` (no separate texture copies) |

# Next Steps

Once `.ssab` files are generated, load them into your engine via [libssruntime](https://github.com/cri-middleware/SpriteStudio-SDK/tree/main/libs/ssruntime). For integration guidance, see the [Player Implementation Guide](https://github.com/cri-middleware/SpriteStudio-SDK/blob/main/libs/ssruntime/docs/README.ja.md) (Japanese).

# For Developers
## Build

You need Rust installed.

```bash
$ cargo build --release
# The binary will be at ./target/release/ssconverter-cli
```

## Test

Run integration tests defined in `src/main.rs`.

```bash
$ cargo test
```
