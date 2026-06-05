#[derive(Debug)]
pub enum ConverterError {
    IoError(std::io::Error),
    SspjError(sspj::sspj_error::SspjError),
    ConversionFailed(String),
    Aborted,
}
impl From<sspj::sspj_error::SspjError> for ConverterError {
    fn from(err: sspj::sspj_error::SspjError) -> Self {
        ConverterError::SspjError(err)
    }
}
impl std::fmt::Display for ConverterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConverterError::IoError(e) => write!(f, "IO Error: {}", e),
            ConverterError::SspjError(e) => write!(f, "SSPJ Parse Error: {}", e),
            ConverterError::ConversionFailed(s) => write!(f, "Conversion Failed: {}", s),
            ConverterError::Aborted => write!(f, "Aborted"),
        }
    }
}
impl std::error::Error for ConverterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConverterError::IoError(e) => Some(e),
            ConverterError::SspjError(e) => Some(e),
            ConverterError::ConversionFailed(_e) => None,
            ConverterError::Aborted => None,
        }
    }
}
