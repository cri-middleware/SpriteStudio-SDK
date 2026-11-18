//! Binary faces of [`ssruntime`]: the C ABI that the Players link, and the
//! wasm-bindgen surface `wasm-pack` builds. Both live in `ssruntime` itself;
//! this crate exists only to emit them as `staticlib` / `cdylib` so the
//! published `ssruntime` can stay a plain `rlib`.
//!
//! The two faces re-export into modules of their own rather than into the crate
//! root: they share names, and no build ever wants both, but `--all-features`
//! turns them on together.
#![no_std]

// Links the library even when no face is enabled: the `#[global_allocator]` and
// `#[panic_handler]` that the staticlib/cdylib need live in `ssruntime`, and an
// unreferenced dependency is not linked at all.
extern crate ssruntime;

#[cfg(feature = "ffi")]
pub mod ffi {
    pub use ssruntime::ffi::*;
}

#[cfg(feature = "web")]
pub mod web {
    pub use ssruntime::web::*;
}
