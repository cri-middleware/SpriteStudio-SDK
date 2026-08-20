//! Binary faces of [`ssconverter`]: the C ABI that the Players link, and the
//! wasm-bindgen surface `wasm-pack` builds. Both live in `ssconverter` itself;
//! this crate exists only to emit them as `staticlib` / `cdylib` so the
//! published `ssconverter` can stay a plain `rlib`.

#[cfg(feature = "ffi")]
pub mod ffi {
    pub use ssconverter::ffi::*;
}

#[cfg(feature = "web")]
pub mod web {
    pub use ssconverter::web::*;
}
