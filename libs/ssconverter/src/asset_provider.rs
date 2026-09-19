use sspj::ssloader::SsLoader;

#[cfg(feature = "web")]
use std::collections::HashMap;

pub trait AssetProvider: Sync + Send {
    fn validate_asset(&self, path: &str) -> Result<(), crate::ConverterError>;
    fn read_asset(&self, path: &str) -> Result<Vec<u8>, std::io::Error>;
    fn load_sspj(&self) -> Result<sspj::sspj::SsProject, sspj::sspj_error::SspjError>;
}

/// Filesystem-backed provider used by the CLI / native (Tauri) builds.
#[cfg(feature = "native")]
pub struct FileSystemAssetProvider {
    path_buf: std::path::PathBuf,
    base_path: std::path::PathBuf,
}

#[cfg(feature = "native")]
impl FileSystemAssetProvider {
    pub fn new(path: &std::path::Path) -> Result<Self, crate::converter::converter_error::ConverterError> {
        Ok(Self { path_buf: path.to_path_buf(), base_path: path.parent().unwrap_or(std::path::Path::new("")).to_path_buf() })
    }
}

#[cfg(feature = "native")]
impl AssetProvider for FileSystemAssetProvider {
    fn validate_asset(&self, path: &str) -> Result<(), crate::ConverterError> {
        crate::utils::validate_file_path(path, &self.base_path)
    }
    fn read_asset(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        let path = self.base_path.join(path);
        std::fs::read(path)
    }
    fn load_sspj(&self) -> Result<sspj::sspj::SsProject, sspj::sspj_error::SspjError> {
        sspj::sspj::SsProject::load_from_file(self.path_buf.as_path())
    }
}

#[cfg(feature = "native")]
pub fn create_asset_provider(path: &std::path::Path) -> Result<Box<dyn AssetProvider>, crate::converter::converter_error::ConverterError> {
    if path.as_os_str().is_empty() {
        return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Path is empty")));
    }
    if !path.exists() {
        return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Path does not exist: {:?}", path))));
    }
    Ok(Box::new(FileSystemAssetProvider::new(path)?))
}

/// In-memory provider for the web (WASM) build. JS reads the project files
/// (via the File System Access API / `<input webkitdirectory>` / drag-drop)
/// and hands them across the boundary as `name -> bytes`; there is no
/// filesystem on this target. Asset paths are relative to the `.sspj`
/// directory, matching what [`sspj::sspj::SsProject::load_from_maps`] expects.
#[cfg(feature = "web")]
pub struct MemoryAssetProvider {
    sspj_name: String,
    sspj_map: HashMap<String, String>,
    binary_map: HashMap<String, Vec<u8>>,
}

#[cfg(feature = "web")]
impl MemoryAssetProvider {
    /// Build a provider from a flat `(path, bytes)` list supplied by the host
    /// (e.g. a directory the user picked in the browser). Locates the `.sspj`,
    /// re-bases every path on the `.sspj` directory, and classifies entries
    /// into text (XML / font) vs binary (images, ...) maps — mirroring how the
    /// project layout is interpreted on the filesystem.
    pub fn from_files(files: Vec<(String, Vec<u8>)>) -> Result<Self, crate::ConverterError> {
        use std::path::{Path, PathBuf};

        let mut sspj_name = String::new();
        let mut sspj_dir = String::new();
        for (path, _) in &files {
            if path.ends_with(".sspj") {
                let pb = PathBuf::from(path);
                sspj_name = pb.file_name().and_then(|n| n.to_str()).ok_or_else(|| crate::ConverterError::ConversionFailed("Invalid .sspj file name".to_string()))?.to_string();
                sspj_dir = pb.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
            }
        }
        if sspj_name.is_empty() {
            return Err(crate::ConverterError::ConversionFailed("SsProject (.sspj) file not found in the provided files".to_string()));
        }

        let mut sspj_map = HashMap::new();
        let mut binary_map = HashMap::new();
        for (path, data) in files {
            let rel = crate::utils::normalize_asset_path(&Path::new(&path).strip_prefix(&sspj_dir).map_err(|_| crate::ConverterError::ConversionFailed(format!("Failed to strip prefix '{}' from '{}'", sspj_dir, path)))?.to_string_lossy());
            if path.ends_with(".sspj") || path.ends_with(".ssae") || path.ends_with(".ssce") || path.ends_with(".ssee") || path.ends_with(".ssqe") || path.ends_with(".ssse") || path.ends_with(".fnt") {
                let text = String::from_utf8(data).map_err(|e| crate::ConverterError::ConversionFailed(format!("Non-UTF8 text asset '{}': {:?}", path, e)))?;
                sspj_map.insert(rel, text);
            } else {
                binary_map.insert(rel, data);
            }
        }

        Ok(Self { sspj_name, sspj_map, binary_map })
    }
}

#[cfg(feature = "web")]
impl AssetProvider for MemoryAssetProvider {
    fn validate_asset(&self, path: &str) -> Result<(), crate::ConverterError> {
        let key = crate::utils::normalize_asset_path(path);
        if self.sspj_map.contains_key(&key) || self.binary_map.contains_key(&key) { Ok(()) } else { Err(crate::ConverterError::ConversionFailed(format!("Asset not found: {}", path))) }
    }
    fn read_asset(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        let key = crate::utils::normalize_asset_path(path);
        if let Some(s) = self.sspj_map.get(&key) {
            return Ok(s.as_bytes().to_vec());
        }
        if let Some(b) = self.binary_map.get(&key) {
            return Ok(b.clone());
        }
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Asset not found: {}", path)))
    }
    fn load_sspj(&self) -> Result<sspj::sspj::SsProject, sspj::sspj_error::SspjError> {
        sspj::sspj::SsProject::load_from_maps(&self.sspj_name, &self.sspj_map)
    }
}

#[cfg(all(test, feature = "web"))]
mod web_test {
    use super::*;

    fn provider() -> MemoryAssetProvider {
        MemoryAssetProvider::from_files(vec![("Doll/Doll.sspj".to_string(), b"<SpriteStudioProject/>".to_vec()), ("Doll/Doll.png".to_string(), vec![0u8; 4]), ("Doll/images/chara.png".to_string(), vec![0u8; 4])]).expect("Failed to build the provider")
    }

    #[test]
    fn a_path_the_filesystem_would_resolve_is_found() {
        let provider = provider();

        assert!(provider.validate_asset("Doll.png").is_ok());
        assert!(provider.validate_asset("./Doll.png").is_ok());
        assert!(provider.validate_asset("images//chara.png").is_ok());
        assert!(provider.validate_asset("images/./chara.png").is_ok());
        assert!(provider.validate_asset("images/../Doll.png").is_ok());

        assert!(provider.read_asset("./Doll.png").is_ok());
        assert!(provider.read_asset("./Doll.sspj").is_ok());
    }

    #[test]
    fn a_path_that_leaves_the_project_is_not_found() {
        let provider = provider();

        assert!(provider.validate_asset("../Doll.png").is_err());
        assert!(provider.validate_asset("/Doll.png").is_err());
        assert!(provider.validate_asset("missing.png").is_err());
    }
}
