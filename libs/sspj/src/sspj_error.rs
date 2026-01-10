#[derive(Debug)]
pub enum SspjError {
    FileError(std::io::Error),
    XmlParseError(quick_xml::DeError),
    FntParseError(String),
}
impl std::fmt::Display for SspjError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SspjError::FileError(e) => write!(f, "IO Error: {}", e),
            SspjError::XmlParseError(e) => write!(f, "SSPJ Parse Error: {}", e),
            SspjError::FntParseError(s) => write!(f, "Conversion Failed: {}", s),
        }
    }
}
impl std::error::Error for SspjError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SspjError::FileError(e) => Some(e),
            SspjError::XmlParseError(e) => Some(e), // 元のエラーを参照可能にする
            SspjError::FntParseError(_) => None,
        }
    }
}
