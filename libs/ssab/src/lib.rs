#![cfg_attr(not(feature = "std"), no_std)]
extern crate flatbuffers;

#[cfg(feature = "converter")]
#[path = "converter/ssab.rs"]
pub mod ssab;

#[cfg(feature = "converter")]
#[path = "converter/ssqb.rs"]
pub mod ssqb;

#[cfg(not(feature = "converter"))]
#[path = "runtime/ssab.rs"]
pub mod ssab;

#[cfg(not(feature = "converter"))]
#[path = "runtime/ssqb.rs"]
pub mod ssqb;
