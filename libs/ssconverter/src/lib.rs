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
