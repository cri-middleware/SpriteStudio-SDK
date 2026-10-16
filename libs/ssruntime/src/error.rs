//! The runtime's error type and its stable integer codes.
//!
//! [`SSRuntimeError::as_code`] maps each variant to a small integer that is part of the
//! C ABI: across FFI a call reports failure by return value and the caller retrieves the
//! code with `ss_runtime_get_last_error`. **These numbers are a compatibility surface** —
//! existing variants keep their codes, and new ones take fresh numbers rather than
//! reusing a retired one.
//!
//! [`SSRuntimeError::as_str`] is the other half, reached across FFI as
//! `ss_runtime_error_message(code)`. A code tells a Player's *code* what happened; the
//! text is what it shows its user, so it is written to be read by someone holding a
//! `.sspj` rather than a debugger. Keep a new variant's wording in that register.
//!
//! **The text carries no payload.** `InvalidVersion` knows two version numbers, but a
//! Player that wants them should read `ss_runtime_ssab_schema_version` and the pack's own
//! `version` field — two integers it can act on, rather than a sentence it would have to
//! parse. [`core::fmt::Display`] does append them, for Rust callers and the wasm face.

/// Everything that can go wrong loading or playing back an animation.
#[derive(Debug, Clone, PartialEq)]
pub enum SSRuntimeError {
    /// A resource whose pack did not parse was bound to a context or an effect slot.
    /// Neither the C nor the web face hands out such a resource, so nothing reports
    /// this today. Empty or truncated bytes are not caught here: loading does not
    /// validate the `.ssab`.
    EmptyData,
    /// SSAB version mismatch.
    InvalidVersion { expected: u32, got: u32 },
    /// Animation not found.
    AnimationNotFound,
    /// Data is not loaded.
    NotLoaded,
    /// Invalid parameter (e.g. frame number out of range).
    InvalidParameter,
    /// Memory allocation failed.
    MemoryError,
    /// Effect data is malformed (e.g. missing required fields).
    MalformedEffect,
    /// Other/unknown error (reserved for future extension).
    Unknown,
}

impl SSRuntimeError {
    /// The stable integer code reported across the C ABI. See the module docs.
    pub fn as_code(&self) -> i32 {
        match self {
            SSRuntimeError::EmptyData => 1,
            SSRuntimeError::InvalidVersion { .. } => 2,
            SSRuntimeError::AnimationNotFound => 3,
            SSRuntimeError::NotLoaded => 4,
            SSRuntimeError::InvalidParameter => 5,
            SSRuntimeError::MemoryError => 6,
            SSRuntimeError::MalformedEffect => 7,
            SSRuntimeError::Unknown => 99,
        }
    }
}

impl SSRuntimeError {
    /// The wording, without the payload any variant carries.
    ///
    /// Static, so it crosses the C ABI as a plain pointer with no context to
    /// borrow from and no lifetime to respect — which is what
    /// `ss_runtime_error_message` hands out. [`core::fmt::Display`] builds on
    /// this rather than repeating it, so there is one wording per variant.
    pub fn as_str(&self) -> &'static str {
        match self {
            SSRuntimeError::EmptyData => "SSAB data is empty",
            SSRuntimeError::InvalidVersion { .. } => "Invalid SSAB version",
            SSRuntimeError::AnimationNotFound => "Animation not found",
            SSRuntimeError::NotLoaded => "SSAB data is not loaded",
            SSRuntimeError::InvalidParameter => "Invalid parameter",
            SSRuntimeError::MemoryError => "Memory allocation failed",
            SSRuntimeError::MalformedEffect => "Effect data is malformed",
            SSRuntimeError::Unknown => "Unknown error occurred",
        }
    }
}

impl core::fmt::Display for SSRuntimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SSRuntimeError::InvalidVersion { expected, got } => {
                write!(f, "{}. expected: {}, got: {}", self.as_str(), expected, got)
            }
            _ => f.write_str(self.as_str()),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SSRuntimeError {}

#[cfg(not(feature = "std"))]
impl core::error::Error for SSRuntimeError {}
