//! Parser for SpriteStudio project files — the SDK's XML front end.
//!
//! A SpriteStudio project is a tree of XML documents: a `.sspj` that references anime
//! packs, cell maps, effects, sequences and sound lists. This crate deserializes all of
//! them into strongly-typed Rust structs with `quick-xml` and `serde`, and does nothing
//! else — [`ssconverter`](https://github.com/cri-middleware/SpriteStudio-SDK/tree/HEAD/libs/ssconverter)
//! is what turns the result into a runtime binary.
//!
//! | Extension | Module | Contents |
//! |---|---|---|
//! | `.sspj` | [`sspj`] | project settings and file references (the root) |
//! | `.ssae` | [`ssae`] | animations, part trees, keyframes |
//! | `.ssce` | [`ssce`] | cell maps (sprite sheets) |
//! | `.ssee` | [`ssee`] | particle effects |
//! | `.ssqe` | [`ssqe`] | sequence packs |
//! | `.ssse` | [`ssse`] | sound effect lists |
//! | `.fnt` | [`sscharmap`] | bitmap font character maps, in BMFont's text format |
//!
//! Keyframe attribute payloads live in [`ssattribute`], and the SpriteStudio enums
//! (interpolation kinds, blend modes, part types) in [`sstypes`].
//!
//! # Loading a project
//!
//! [`ssloader::SsLoader`] is the entry trait. Loading the root pulls in every
//! referenced file for you — in parallel, unless the `parallel` feature is off.
//!
//! ```no_run
//! use sspj::sspj::SsProject;
//! use sspj::ssloader::SsLoader;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let project = SsProject::load_from_file(Path::new("assets/character.sspj"))?;
//!
//! for pack in &project.anime_list {
//!     for anime in &pack.anime_list {
//!         println!("{}/{}: {} frames", pack.name, anime.name, anime.settings.frame_count);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Feature flags
//!
//! | Feature | Purpose |
//! |---|---|
//! | `parallel` *(default)* | Load referenced files concurrently via rayon. Turn off for `wasm32-unknown-unknown`, where rayon cannot spawn threads; the sequential fallback lives in this crate's `parallel` module. |

mod parallel;

pub mod asset_path;
pub mod ssae;
pub mod ssattribute;
pub mod ssce;
pub mod sscharmap;
pub mod ssee;
pub mod ssloader;
pub mod sspj;
pub mod sspj_error;
pub mod ssqe;
pub mod ssse;
pub mod sstypes;

#[cfg(test)]
mod tests_robustness;
