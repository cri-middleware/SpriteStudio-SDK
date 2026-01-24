# SpriteStudio7-SDK

An Software Development Kit for sspj projects that are created by  [OPTPiX SpriteStudio](https://www.webtech.co.jp/spritestudio/index.html) 7 or higher flexibly to implement an animation player on any game engines or platforms.

- Libraries
    - [libsspj](./libs/sspj): a library that parses `sspj` projects that contains of `sspj`, `ssae`, `ssee`, `ssse`, `charmap` files.
    - [libssab](./libs/ssab): a library that creates and parses `ssab` files that is an animation binary file.
    - [libssconverter](./libs/ssconverter): a library that parses `sspj` projects and creates `ssab` files.
    - [libssruntime](./libs/ssruntime): a library that helps implementing an animation player. e.g. controls playback, extracts keyframes and do interpolation  ssab.
- Tools
    - [ssconverter-cli](./cli): a command line tools that parses `sspj` projects and creates `ssab` files.

# Tools Usage 
## ssconverter-cli
ssconverter-cli has two modes, **single mode** and **batch mode**.

- single mode
    - The mode converts a sspj project or a sspj zip file.
- batch mode
    - The mode converts multiple sspj projects.

### single mode

```bash
$ ssconverter-cli [OPTIONS] <sspj_file>
$ ssconverter-cli [OPTIONS] <sspj_zip_file>
```

#### OPTIONS:
- `-e`, `--embedded`
    - embed assets into a ssab file
- `-o`, `--out-dir <directory>`
    - output directory path (available for single mode only)
- `-h`, `--help`
    - Print help

### batch mode

```bash
$ ssconverter-cli [OPTIONS] <sspj_file> <sspj_file> ...
$ ssconverter-cli [OPTIONS] <directory> <directory> ...
```

#### OPTIONS:
- `-e`, `--embedded`
    - embed assets into a ssab file
- `-h`, `--help`
    - Print help

# For Player implementers

Porting guide that is how to use libssruntime is coming soon.


# For SDK Developers
## Prepare
install [rustup](https://rustup.rs/)

## Build

```bash
cargo build --release
```

## Test
### Unit tests

```bash
cargo test
```

### Convert tests

```bash
./target/release/ssconverter-cli ./
```
