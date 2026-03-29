#[derive(Debug, Clone, PartialEq)]
pub enum SSRuntimeError {
    /// データの読み込みに失敗（空データなど）
    EmptyData,
    /// SSAB のバージョンが不一致
    InvalidVersion { expected: u32, got: u32 },
    /// その他のエラー（将来的な拡張用）
    Unknown,
}

impl core::fmt::Display for SSRuntimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SSRuntimeError::EmptyData => write!(f, "SSAB data is empty"),
            SSRuntimeError::InvalidVersion { expected, got } => {
                write!(f, "Invalid SSAB version. expected: {}, got: {}", expected, got)
            }
            SSRuntimeError::Unknown => write!(f, "Unknown error occurred"),
        }
    }
}
