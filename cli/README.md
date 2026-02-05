[**日本語**](./README.ja.md) | [**English**](./README.md)

# ssconverter-cli

A command-line tool that parses SpriteStudio 7 project files (`.sspj`) and converts them into optimized runtime binary files (`.ssab`, `.ssqb`).

This tool wraps [libssconverter](../libs/ssconverter) and provides easy access to its features via the command line.

# Usage

`ssconverter-cli` operates in two main modes: **Single Mode** and **Batch Mode**.

## Single Mode

Converts a single `.sspj` project file.
In this mode, you can specify an output directory and enable file watching.

```bash
# Convert a single project
$ ssconverter-cli project.sspj

# Convert and specify output directory
$ ssconverter-cli project.sspj -o ./output_folder
```

### Options (Single Mode)
- `-e`, `--embedded`: Embed referenced assets (images, etc.) directly into the generated `.ssab` file.
- `-o`, `--out-dir <directory>`: Specify the output directory for generated files.
- `-w`, `--watcher`: Watch the project directory for changes and automatically re-convert when files are modified.
- `-h`, `--help`: Print help information.

## Batch Mode

Converts multiple `.sspj` projects at once.
You can pass multiple file paths or directories containing `.sspj` files.
In this mode, the `--out-dir` option is ignored; output files are generated alongside the source files (or in a default directory).

```bash
# Convert multiple specific files
$ ssconverter-cli project1.sspj project2.sspj

# Convert all sspj files found in directories (recursive)
$ ssconverter-cli ./assets/projects ./extra_assets
```

### Options (Batch Mode)
- `-e`, `--embedded`: Embed referenced assets directly into the generated `.ssab` files.
- `-h`, `--help`: Print help information.

_Note: The `--out-dir` and `--watcher` options are not available in batch mode._

# Examples

**1. Basic Conversion**

Generates `project_ssab/` folder next to `project.sspj`.

```bash
$ ssconverter-cli my_game/project.sspj
```

**2. Watch Mode**

Keeps the process running and re-converts whenever you save changes in SpriteStudio.

```bash
$ ssconverter-cli my_game/project.sspj --watch
```


**3. Embedded Assets**

Creates a single binary containing all textures, useful for simple distribution.

```bash
$ ssconverter-cli my_game/project.sspj --embedded
```

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
