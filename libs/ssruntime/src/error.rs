#[derive(Debug, Clone, PartialEq)]
pub enum SSRuntimeError {
    /// データの読み込みに失敗（空データなど）
    EmptyData,
    /// SSAB のバージョンが不一致
    InvalidVersion { expected: u32, got: u32 },
    /// アニメーションが見つからない
    AnimationNotFound,
    /// データがロードされていない
    NotLoaded,
    /// パラメータが不正（フレーム番号が範囲外など）
    InvalidParameter,
    /// その他のエラー（将来的な拡張用）
    Unknown,
}

impl SSRuntimeError {
    pub fn as_code(&self) -> i32 {
        match self {
            SSRuntimeError::EmptyData => 1,
            SSRuntimeError::InvalidVersion { .. } => 2,
            SSRuntimeError::AnimationNotFound => 3,
            SSRuntimeError::NotLoaded => 4,
            SSRuntimeError::InvalidParameter => 5,
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
            SSRuntimeError::Unknown => write!(f, "Unknown error occurred"),
        }
    }
}
