[**日本語**](./README.ja.md) | [**English**](./README.md)

# libsspj

A Rust library for parsing OPTPiX SpriteStudio 7 project files (`.sspj`) and associated XML assets.
It deserializes the XML structure into strongly-typed Rust structs using `quick-xml` and `serde`.

# Supported Formats

This library handles the parsing of the following file types comprising a SpriteStudio project:

- **SSPJ (`.sspj`)**: Project settings and file references.
- **SSAE (`.ssae`)**: Animation data.
- **SSCE (`.ssce`)**: Cell map definitions (sprite sheets).
- **SSEE (`.ssee`)**: Particle effect data.
- **SSQE (`.ssqe`)**: Sequence data.
- **SSSE (`.ssse`)**: Sound effect lists.
- **BitmapFont (`.fnt` / `.json`)**: Character maps (parsed via `sscharmap`).

# Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
sspj = { path = "libs/sspj" }
```

# Usage

The main entry point is `SsProject`. You can load a project file using the `SsLoader` trait.
When loading an `.sspj` file, referenced assets (anime packs, cell maps, effects, etc.) are automatically loaded in parallel using `rayon`.

```rust
use std::path::Path;
use sspj::sspj::SsProject;
use sspj::ssloader::SsLoader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
let path = Path::new("path/to/project.sspj");

    // Load the project and all referenced files
    let project = SsProject::load_from_file(&path)?;

    println!("Project Name: {}", project.name);
    println!("Version: {}", project.version);

    // Access loaded resources
    for anime_pack in &project.anime_list {
        println!("Loaded AnimePack: {}", anime_pack.name);
        for anime in &anime_pack.anime_list {
             println!(" - Animation: {} (Frames: {})", anime.name, anime.settings.frame_count);
        }
    }

    for cell_map in &project.cellmap_list {
        println!("Loaded CellMap: {}", cell_map.name);
    }

    Ok(())
}
```

# Features

- **Full Type Support**: Maps SpriteStudio internal types (Interpolation types, Blend modes, Attributes) to Rust enums and structs.
- **Parallel Loading**: Uses `rayon` to load referenced external files (like `.ssae`, `.ssce`) concurrently for faster startup times.
- **Serde Integration**: All data structures derive `Deserialize`, allowing for easy integration with other serde-compatible tools.

# Architecture

- `sspj::sspj`: Defines the root `SsProject` struct.
- `sspj::ssae`: Definitions for animation data, parts, and keyframes.
- `sspj::ssattribute`: specific attribute definitions (Position, Rotation, Scale, User Data, etc.).
- `sspj::ssce`: Cell map and cell definitions.
- `sspj::ssee`: Effect node and particle behavior definitions.

