#[derive(Debug)]
pub enum SspjError {
    FileError(std::io::Error),
    XmlParseError(quick_xml::DeError),
    FntParseError(String),
    /// Raised while loading a file the `.sspj` lists; `path` is relative to the `.sspj`'s directory.
    InFile {
        path: String,
        source: Box<SspjError>,
    },
}
impl SspjError {
    pub(crate) fn in_file(self, path: String) -> Self {
        SspjError::InFile { path, source: Box::new(self) }
    }
}
impl std::fmt::Display for SspjError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SspjError::FileError(e) => write!(f, "IO Error: {}", e),
            SspjError::XmlParseError(e) => write!(f, "XML Parse Error: {}", e),
            SspjError::FntParseError(s) => write!(f, "Conversion Failed: {}", s),
            SspjError::InFile { path, source } => write!(f, "{}: {}", path, source),
        }
    }
}
impl std::error::Error for SspjError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SspjError::FileError(e) => Some(e),
            SspjError::XmlParseError(e) => Some(e),
            SspjError::FntParseError(_) => None,
            SspjError::InFile { source, .. } => Some(source.as_ref()),
        }
    }
}
impl From<std::num::ParseIntError> for SspjError {
    fn from(e: std::num::ParseIntError) -> Self {
        SspjError::FntParseError(e.to_string())
    }
}
impl From<std::num::ParseFloatError> for SspjError {
    fn from(e: std::num::ParseFloatError) -> Self {
        SspjError::FntParseError(e.to_string())
    }
}
