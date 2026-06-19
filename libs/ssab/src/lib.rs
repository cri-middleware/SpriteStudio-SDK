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

/// `.ssab` schema version, read from codegen so it tracks `fbs/ssab.fbs`.
pub fn ssab_schema_version() -> u32 {
    ssab::ss::format::SsAnimeBinaryArgs::default().version
}

/// `.ssqb` schema version, read from codegen so it tracks `fbs/ssqb.fbs`.
pub fn ssqb_schema_version() -> u32 {
    ssqb::ss::format::SsSequenceBinaryArgs::default().version
}
