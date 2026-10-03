//! The `.ssab` / `.ssqb` binary format — SpriteStudio's on-disk runtime data.
//!
//! This crate is the schema, not a library of behaviour: almost everything under
//! [`ssab`] and [`ssqb`] is FlatBuffers code generated from `fbs/ssab.fbs` and
//! `fbs/ssqb.fbs`. It sits between the two halves of the SDK — `ssconverter` writes
//! these binaries, `ssruntime` reads them zero-copy.
//!
//! | Module | Root table | Holds |
//! |---|---|---|
//! | [`ssab`] | `SsAnimeBinary` | animations, parts, keyframes, cell maps, effects |
//! | [`ssqb`] | `SsSequenceBinary` | sequence packs (`.ssqe`) |
//!
//! # Reading (default)
//!
//! The default feature set is read-only and zero-copy, which is what a Player wants:
//! accessors borrow directly out of the mapped buffer, with no parse step and no
//! allocation.
//!
//! ```no_run
//! use ssab::ssab::ss::format::root_as_ss_anime_binary;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let bytes = std::fs::read("character.ssab")?;
//! let binary = root_as_ss_anime_binary(&bytes)?;
//! println!("{}", binary.name());
//! for anime in binary.animations() {
//!     println!("  {} @ {} fps", anime.name(), anime.fps());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Writing (`converter`)
//!
//! Enabling `converter` regenerates the modules with the FlatBuffers *object API*
//! (`--gen-object-api`), adding owned `…T` structs that can be built up in memory and
//! packed back into a buffer. Only tools that emit data need this; it implies `std`.
//!
//! # Feature flags
//!
//! | Feature | Purpose |
//! |---|---|
//! | `std` *(default)* | `std` support in `flatbuffers`. Disable for `no_std` Players. |
//! | `converter` | Read/write object API (`SsAnimeBinaryT` and friends). Implies `std`. |
//!
//! # Validation
//!
//! `get_root_as_*` verifies the buffer; the `_unchecked` variants do not. `ssruntime`
//! deliberately uses the unchecked path in the playback hot loop, which is why
//! validating untrusted assets is the Player's responsibility.
//!
//! # Regenerating
//!
//! The Rust modules are generated output — edit `fbs/*.fbs` and re-run
//! `scripts/generate.sh` (or `.ps1`), which needs `flatc` on `PATH`. Never hand-edit
//! `src/runtime/` or `src/converter/`.
//!
//! See the [schema documentation](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/libs/ssab/SCHEMA.md)
//! for the field-by-field design spec.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
extern crate flatbuffers;

// The `ssab` / `ssqb` modules are `flatc` output, so `missing_docs` is waived for
// them. The attribute sits on the declaration, not inside the generated file, so
// that `scripts/generate.{sh,ps1}` cannot wipe it.

#[allow(missing_docs)]
#[rustfmt::skip]
#[cfg(feature = "converter")]
#[path = "converter/ssab.rs"]
pub mod ssab;

#[allow(missing_docs)]
#[rustfmt::skip]
#[cfg(feature = "converter")]
#[path = "converter/ssqb.rs"]
pub mod ssqb;

#[allow(missing_docs)]
#[rustfmt::skip]
#[cfg(not(feature = "converter"))]
#[path = "runtime/ssab.rs"]
pub mod ssab;

#[allow(missing_docs)]
#[rustfmt::skip]
#[cfg(not(feature = "converter"))]
#[path = "runtime/ssqb.rs"]
pub mod ssqb;

/// The `.ssab` schema version the converter stamps into every pack it emits and
/// the runtime refuses to read anything else as.
///
/// Both `version` fields declare no FlatBuffers default, so this value is never
/// equal to one and is therefore always present in the emitted bytes. A pack
/// that reads back as `0` predates the field being written at all, which is a
/// state the check has to be able to see — with a schema default of `1` the
/// reader substituted its own version for the absent field and every pack ever
/// produced passed, whatever it actually was.
///
/// Raise it in the same commit that changes the wire format, and
/// `an_emitted_pack_carries_its_schema_version` proves the bytes still carry it.
pub const SSAB_SCHEMA_VERSION: u32 = 1;

/// The `.ssqb` counterpart of [`SSAB_SCHEMA_VERSION`].
pub const SSQB_SCHEMA_VERSION: u32 = 1;

/// `.ssab` schema version. See [`SSAB_SCHEMA_VERSION`].
pub fn ssab_schema_version() -> u32 {
    SSAB_SCHEMA_VERSION
}

/// `.ssqb` schema version. See [`SSQB_SCHEMA_VERSION`].
pub fn ssqb_schema_version() -> u32 {
    SSQB_SCHEMA_VERSION
}
