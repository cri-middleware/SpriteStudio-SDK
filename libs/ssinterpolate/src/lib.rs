//! Interpolation math for SpriteStudio animation curves.
//!
//! The curve evaluation shared by the SDK, isolated into a crate with no dependency on
//! `ssab` or any other SpriteStudio type — just numbers in, numbers out. That makes it
//! reusable on its own, and keeps the maths identical between the converter, the
//! runtime, and any Player that reimplements part of the pipeline natively.
//!
//! Everything lives in [`interpolate`]: linear, Hermite and Bezier interpolation plus
//! the easing rate functions, all generic over the [`interpolate::SsFloat`] trait so the
//! same code serves `f32` and `f64`.
//!
//! ```
//! use ssinterpolate::interpolate::{linear, rate_ease_in};
//!
//! assert_eq!(linear(0.0f32, 100.0, 0.5), 50.0);
//! assert_eq!(linear(0.0f64, 100.0, 0.5), 50.0); // same code path, f64
//!
//! // Easing shapes the rate, which then drives the interpolation.
//! let eased = linear(0.0f32, 100.0, rate_ease_in(0.5, 2.0));
//! assert!(eased < 50.0);
//! ```
//!
//! This crate is `#![no_std]` unconditionally and uses `libm` for the float maths, so it
//! builds for embedded and WASM targets without configuration.
//!
//! # Feature flags
//!
//! | Feature | Purpose |
//! |---|---|
//! | `ffi` | `extern "C"` wrappers (the `ffi` module) for C/C++ interoperability. Off by default. |

#![no_std]
#[cfg(feature = "ffi")]
pub mod ffi;
pub mod interpolate;
