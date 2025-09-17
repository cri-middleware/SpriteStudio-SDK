use std::path::Path;

pub trait SsLoader {
    type Output;
    fn load_from_file(path: &Path) -> Result<Self::Output, quick_xml::DeError> {
        let Ok(xml_data) = std::fs::read_to_string(path) else {
            return Err(quick_xml::DeError::Custom(format!("Failed to read file: {}", path.display())))
        };

        Self::load(&xml_data)
    }
    fn load(s: &str) -> Result<Self::Output, quick_xml::DeError>;
}