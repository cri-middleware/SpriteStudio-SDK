#![cfg_attr(not(feature = "std"), no_std)]
extern crate flatbuffers;

#[rustfmt::skip]
#[cfg(feature = "converter")]
#[path = "converter/ssab.rs"]
pub mod ssab;

#[rustfmt::skip]
#[cfg(feature = "converter")]
#[path = "converter/ssqb.rs"]
pub mod ssqb;

#[rustfmt::skip]
#[cfg(not(feature = "converter"))]
#[path = "runtime/ssab.rs"]
pub mod ssab;

#[rustfmt::skip]
#[cfg(not(feature = "converter"))]
#[path = "runtime/ssqb.rs"]
pub mod ssqb;
