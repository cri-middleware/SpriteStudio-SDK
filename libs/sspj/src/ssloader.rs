use std::path::Path;

pub trait SsLoader {
    type Output;
    fn load_from_file(path: &Path) -> Result<Self::Output, crate::sspj_error::SspjError> {
        let xml_data = std::fs::read_to_string(path).map_err(crate::sspj_error::SspjError::FileError)?;
        Self::load(&xml_data)
    }
    fn load_from_maps(name: &str, maps: &std::collections::HashMap<String, String>) -> Result<Self::Output, crate::sspj_error::SspjError> {
        let Some(xml_data) = maps.get(name) else {
            return Err(crate::sspj_error::SspjError::FileError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Failed to get a xml data: {:?}", name))))
        };
        Self::load(&xml_data)
    }
    fn load(s: &str) -> Result<Self::Output, crate::sspj_error::SspjError>;
}
