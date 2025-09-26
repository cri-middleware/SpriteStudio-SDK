use std::path::Path;

#[derive(Debug)]
pub enum SspjError {
    FileError(std::io::Error),
    ParseError(quick_xml::DeError),
}


pub trait SsLoader {
    type Output;
    fn load_from_file(path: &Path) -> Result<Self::Output, SspjError> {
        let r = std::fs::read_to_string(path);
        let xml_data = match r {
            Ok(data) => data,
            Err(e) => return Err(SspjError::FileError(e)),
        };

        Self::load(&xml_data)
    }
    fn load(s: &str) -> Result<Self::Output, SspjError>;
}