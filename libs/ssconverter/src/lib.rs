//! Converts SpriteStudio projects into the runtime binary format.
//!
//! `ssconverter` is the offline half of the SDK: it takes a `.sspj` project — parsed
//! into typed structs by [`sspj`] — and emits the compact FlatBuffers binaries
//! (`.ssab` for animations, `.ssqb` for sequences) that
//! [`ssruntime`](https://github.com/cri-middleware/SpriteStudio-SDK/tree/HEAD/libs/ssruntime)
//! plays back. Everything expensive happens here so that playback stays allocation-free.
//!
//! ```text
//!   .sspj / .ssae / .ssce / .ssee / .ssqe  ──sspj──>  typed structs  ──ssconverter──>  .ssab / .ssqb
//! ```
//!
//! # Two build profiles
//!
//! Both profiles share one in-memory conversion core, so the bytes they produce are
//! identical; they differ only in how assets get in and out.
//!
//! - **Native** (`native`, the default; also implied by `ffi`) — filesystem I/O, directory
//!   scanning, and rayon-parallel conversion. This is what `ssconverter-cli` and the
//!   Tauri GUI use, and what Unity / Godot / Ren'Py embed to convert in place.
//! - **Web** (`web`) — `wasm-bindgen` bindings for `wasm32-unknown-unknown`. Sequential,
//!   with assets passed in and out as byte buffers. Built by `scripts/release-web.sh`.
//!
//! # Quick start (native)
//!
//! ```no_run
//! use ssconverter::{ConverterConfig, convert_sspj_to_ssab_files};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), ssconverter::ConverterError> {
//! let config = ConverterConfig::default(); // `output: None` writes next to the project
//! convert_sspj_to_ssab_files(Path::new("assets/character.sspj"), &config, &|msg: &str| {
//!     println!("{msg}");
//! })?;
//! # Ok(())
//! # }
//! ```
//!
//! Use [`convert_sspj_to_ssab_files_cancellable`] instead when a GUI needs to abort a
//! long conversion; it takes an `&AtomicBool` that is polled between work items.
//!
//! # Feature flags
//!
//! | Feature | Purpose |
//! |---|---|
//! | `ffi` | C-API exports (the `ffi` module). Implies `native`. For `ssconverter-ffi`, not for Rust dependents — see below. |
//! | `native` *(default)* | Filesystem conversion paths. Implies `parallel` and `scan`. |
//! | `parallel` | rayon-backed parallel conversion; also switches [`sspj`] to parallel parsing. |
//! | `scan` | Directory traversal (the `scan` module) for batch conversion. |
//! | `web` | Browser WASM bindings (the `web` module). Sequential — rayon cannot spawn threads on `wasm32-unknown-unknown`. Same caveat as `ffi`. |
//!
//! `ffi` and `web` exist for `ssconverter-ffi`, the sibling crate that emits this one as
//! a `staticlib` / `cdylib`. Enabling either from a Rust dependency compiles
//! `#[no_mangle] extern "C"` symbols into your binary, and those collide if the same
//! binary also links a prebuilt `libssconverter`.
//!
//! `parallel` spans two crates: the web entry path is itself parallel inside [`sspj`],
//! so the feature gates rayon in both, with a sequential fallback in each crate's
//! `parallel` module.
//!
//! # Versioning
//!
//! [`version_string`] reports this converter's version, while [`ssab_schema_version`]
//! and [`ssqb_schema_version`] report the *format* versions of the binaries it emits.
//! A Player checks the latter to decide whether it can read a given asset.
//!
//! # Further reading
//!
//! - [Converter usage guide](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/libs/ssconverter/docs/USAGE.md)
//! - [SSAB / SSQB schema](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/libs/ssab/SCHEMA.md)

mod cache;
mod converter;
mod model_t;
mod parallel;
#[cfg(feature = "scan")]
pub mod utility;
mod utils;
pub use converter::converter::*;
pub use converter::converter_error::*;
// Version string (CARGO_PKG_VERSION + short git sha). Exposed as public API so
// every embedder — CLI, Tauri (native), C-API (ffi), and the web bindings —
// can report the converter version through a single source of truth.
// ssab/ssqb_schema_version report the emitted binary format versions instead.
pub use utils::{ssab_schema_version, ssqb_schema_version, version_string};
#[cfg(feature = "scan")]
pub mod scan;

mod asset_provider;

#[cfg(feature = "ffi")]
pub mod ffi;

#[cfg(feature = "web")]
pub mod web;
