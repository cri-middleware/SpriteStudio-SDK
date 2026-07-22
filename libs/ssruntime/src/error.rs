//! The runtime's error type and its stable integer codes.
//!
//! [`SSRuntimeError::as_code`] maps each variant to a small integer that is part of the
//! C ABI: across FFI a call reports failure by return value and the caller retrieves the
//! code with `ss_runtime_get_last_error`. **These numbers are a compatibility surface** —
//! existing variants keep their codes, and new ones take fresh numbers rather than
//! reusing a retired one.

/// Everything that can go wrong loading or playing back an animation.
#[derive(Debug, Clone, PartialEq)]
pub enum SSRuntimeError {
    /// Failed to load data (e.g. empty data).
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

impl core::fmt::Display for SSRuntimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SSRuntimeError::EmptyData => write!(f, "SSAB data is empty"),
            SSRuntimeError::InvalidVersion { expected, got } => {
                write!(f, "Invalid SSAB version. expected: {}, got: {}", expected, got)
            }
            SSRuntimeError::AnimationNotFound => write!(f, "Animation not found"),
            SSRuntimeError::NotLoaded => write!(f, "SSAB data is not loaded"),
            SSRuntimeError::InvalidParameter => write!(f, "Invalid parameter"),
            SSRuntimeError::MemoryError => write!(f, "Memory allocation failed"),
            SSRuntimeError::MalformedEffect => write!(f, "Effect data is malformed"),
            SSRuntimeError::Unknown => write!(f, "Unknown error occurred"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SSRuntimeError {}

#[cfg(not(feature = "std"))]
impl core::error::Error for SSRuntimeError {}
