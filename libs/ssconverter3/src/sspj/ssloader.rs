use std::path::Path;

#[derive(Debug)]
pub enum SspjError {
    FileError(std::io::Error),
    XmlParseError(quick_xml::DeError),
    FntParseError(String),
}


pub trait SsLoader {
    type Output;
    fn load_from_file(path: &Path) -> Result<Self::Output, SspjError> {
        let xml_data = std::fs::read_to_string(path).map_err(SspjError::FileError)?;
        Self::load(&xml_data)
    }
    fn load(s: &str) -> Result<Self::Output, SspjError>;
}