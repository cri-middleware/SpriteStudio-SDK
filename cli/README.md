[**日本語**](./README.ja.md) | [**English**](./README.md)

# ssconverter-cli

The build-time tool of the [SpriteStudio-SDK](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/README.md) workflow: parses SpriteStudio project files (`.sspj` and the referenced `.ssae` / `.ssee` / `.ssqe` / `.ssce`) and emits the runtime binaries the SDK plays back — `.ssab` (animation) and `.ssqb` (sequence pack).

This CLI wraps [libssconverter](https://github.com/cri-middleware/SpriteStudio-SDK/tree/HEAD/libs/ssconverter) — the same converter the Players' built-in importers run. The runtime side lives in [libssruntime](https://github.com/cri-middleware/SpriteStudio-SDK/tree/HEAD/libs/ssruntime).

# Installation

## Prebuilt binaries (recommended)

Download a release for **Windows**, **macOS**, or **Linux** from the [GitHub Releases](https://github.com/cri-middleware/SpriteStudio-SDK/releases) page. No build environment required.

Each release carries one zip per platform under a fixed name, so the latest is always at the same URL:

| Platform | Archive |
|---|---|
| macOS (Apple silicon and Intel) | `ssconverter-cli-macos.zip` |
| Windows x64 | `ssconverter-cli-windows.zip` |
| Windows ARM64 | `ssconverter-cli-windows-arm64.zip` |
| Linux x64 | `ssconverter-cli-linux.zip` |
| Linux ARM64 | `ssconverter-cli-linux-aarch64.zip` |

```bash
$ curl -fsSLO https://github.com/cri-middleware/SpriteStudio-SDK/releases/latest/download/ssconverter-cli-macos.zip
$ unzip ssconverter-cli-macos.zip -d ssconverter-cli
$ ./ssconverter-cli/ssconverter-cli --version
```

To pin a version, replace `latest/download` with `download/<tag>`. `SHA256SUMS` in the same release verifies the zip.

## Build from source

Requires a Rust toolchain ([rustup](https://rustup.rs/)), version **1.99 or newer**. From the repository root:

```bash
$ cargo install --path cli
```

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

## Exit status

In either mode, the run exits with a non-zero status when a path given does not exist or a project fails to convert. Every such path and project is named on stderr, and the projects that can be converted still are.

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
| Default | `<name>/` | `.ssab` (one per animation pack, `.ssae`), `.ssqb` (per sequence pack, if `.ssqe` is referenced), and copies of referenced textures |
| `--embedded` | `<name>_emb/` | Same as above, but referenced assets are inlined into each `.ssab` (no separate texture copies) |

The run prints a line for each binary it wrote, `Wrote <path> (animations: …)` or, for an `.ssqb`, `(sequences: …)`: the names a player can be asked to play.

# Next Steps

Once `.ssab` files are generated, hand them to your Player; its documentation says where they go. To bring them into an engine yourself, load them through [libssruntime](https://github.com/cri-middleware/SpriteStudio-SDK/tree/HEAD/libs/ssruntime), with the [Player Porting Guide](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/libs/ssruntime/docs/README.md) as the integration guidance.

# For Developers
## Build

You need Rust installed.

```bash
$ cargo build --release
# The binary will be at ./target/release/ssconverter-cli
```

## Test

The tests in `tests/` run the built CLI and check its exit status. From the repository root:

```bash
$ cargo test -p ssconverter-cli
```
