use crate::utils;
use sspj::ssloader::SsLoader;
use std::collections::HashMap;
use std::io::{Cursor, ErrorKind};

pub trait AssetProvider: Sync + Send {
    fn validate_asset(&self, path: &str) -> Result<(), crate::ConverterError>;
    fn read_asset(&self, path: &str) -> Result<Vec<u8>, std::io::Error>;
    fn copy_asset(&self, src_path: &str, dst_path: &str) -> Result<(), std::io::Error>;
    fn load_sspj(&self) -> Result<sspj::sspj::SsProject, sspj::sspj_error::SspjError>;
    fn is_zip(&self) -> bool;
}

pub struct FileSystemAssetProvider {
    path_buf: std::path::PathBuf,
    base_path: std::path::PathBuf,
}

impl FileSystemAssetProvider {
    pub fn new(path: &std::path::Path) -> Result<Self, crate::converter::converter_error::ConverterError> {
        Ok(Self { path_buf: path.to_path_buf(), base_path: path.parent().unwrap_or(std::path::Path::new("")).to_path_buf() })
    }
}

impl AssetProvider for FileSystemAssetProvider {
    fn validate_asset(&self, path: &str) -> Result<(), crate::ConverterError> {
        utils::validate_file_path(path, &self.base_path)
    }
    fn read_asset(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        let path = self.base_path.join(path);
        std::fs::read(path)
    }
    fn copy_asset(&self, src_path: &str, dst_path: &str) -> Result<(), std::io::Error> {
        let src_path = self.base_path.join(src_path);
        let dst_path = std::path::PathBuf::from(dst_path);
        let r = std::fs::copy(src_path.as_path(), dst_path.as_path());
        match r {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::other(format!("copy_asset: {:?}", e))),
        }
    }
    fn load_sspj(&self) -> Result<sspj::sspj::SsProject, sspj::sspj_error::SspjError> {
        sspj::sspj::SsProject::load_from_file(self.path_buf.as_path())
    }
    fn is_zip(&self) -> bool {
        false
    }
}

pub struct ZipAssetProvider {
    sspj_name: String,
    sspj_map: HashMap<String, String>,
    binary_map: HashMap<String, Vec<u8>>,
}

impl ZipAssetProvider {
    pub fn new(_zip_name: &str, zip_content: &[u8]) -> Result<Self, crate::converter::converter_error::ConverterError> {
        let cursor = Cursor::new(zip_content);
        let mut archive = zip::ZipArchive::new(cursor).map_err(|e| crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(ErrorKind::InvalidData, format!("Zip error: {:?}", e))))?;

        // find sspj file
        let mut sspj_name = "".to_string();
        let mut sspj_dir = "".to_string();
        for i in 0..archive.len() {
            let file = archive.by_index(i).map_err(|e| crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(ErrorKind::InvalidData, format!("Zip entry error: {:?}", e))))?;
            if file.is_dir() {
                continue;
            }

            let path_key = file.name().to_string();
            if path_key.ends_with(".sspj") {
                let path_buf = std::path::PathBuf::from(file.name());
                sspj_name = path_buf
                    .file_name()
                    .ok_or_else(|| crate::converter::converter_error::ConverterError::ConversionFailed("Invalid SSPJ file name in zip".to_string()))?
                    .to_str()
                    .ok_or_else(|| crate::converter::converter_error::ConverterError::ConversionFailed("Non-UTF8 path in zip".to_string()))?
                    .to_string();
                sspj_dir = path_buf
                    .parent()
                    .ok_or_else(|| crate::converter::converter_error::ConverterError::ConversionFailed("Could not get parent of SSPJ in zip".to_string()))?
                    .to_str()
                    .ok_or_else(|| crate::converter::converter_error::ConverterError::ConversionFailed("Non-UTF8 parent path in zip".to_string()))?
                    .to_string();
            }
        }
        if sspj_name.is_empty() {
            return Err(crate::converter::converter_error::ConverterError::ConversionFailed("SsProject file not found in zip".to_string()));
        }

        let mut sspj_map = HashMap::new();
        let mut binary_map = HashMap::new();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(ErrorKind::InvalidData, format!("Zip entry error: {:?}", e))))?;
            if file.is_dir() {
                continue;
            }

            let path_buf = std::path::PathBuf::from(file.name());
            let path_key = file.name().to_string();
            let file_path = path_buf.strip_prefix(&sspj_dir).map_err(|_| crate::converter::converter_error::ConverterError::ConversionFailed(format!("Failed to strip prefix: {} from {}", sspj_dir, file.name())))?;

            if path_key.ends_with(".sspj") || path_key.ends_with(".ssae") || path_key.ends_with(".ssce") || path_key.ends_with(".ssee") || path_key.ends_with(".ssqe") || path_key.ends_with(".ssse") || path_key.ends_with(".fnt") {
                let mut s = String::new();
                use std::io::Read;
                file.read_to_string(&mut s).map_err(crate::converter::converter_error::ConverterError::IoError)?;

                sspj_map.insert(file_path.to_string_lossy().to_string(), s);
            } else {
                let mut buf = Vec::new();
                use std::io::Read;
                file.read_to_end(&mut buf).map_err(crate::converter::converter_error::ConverterError::IoError)?;
                binary_map.insert(file_path.to_string_lossy().to_string(), buf);
            }
        }

        Ok(Self { sspj_name, sspj_map, binary_map })
    }
}

impl AssetProvider for ZipAssetProvider {
    fn validate_asset(&self, path: &str) -> Result<(), crate::ConverterError> {
        // let asset_path = std::path::PathBuf::from(&self.sspj_dir).join(path).as_path().to_str().unwrap().to_string();
        if self.sspj_map.get(path).is_some() {
            return Ok(());
        }
        if self.binary_map.get(path).is_some() {
            return Ok(());
        };
        Err(crate::ConverterError::ConversionFailed(format!("Asset not found in zip: {}", path)))
    }

    fn read_asset(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        // let asset_path = std::path::PathBuf::from(&self.sspj_dir).join(path).as_path().to_str().unwrap().to_string();
        if let Some(r) = self.sspj_map.get(path) {
            return Ok(r.as_bytes().to_vec());
        }
        if let Some(r) = self.binary_map.get(path) {
            return Ok(r.clone());
        }
        Err(std::io::Error::new(ErrorKind::NotFound, format!("Asset not found in zip: {}", path)))
    }

    fn copy_asset(&self, src_path: &str, dst_path: &str) -> Result<(), std::io::Error> {
        let data = self.read_asset(src_path)?;
        let dst_path_buf = std::path::PathBuf::from(dst_path);
        
        if let Some(parent) = dst_path_buf.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(dst_path_buf.as_path(), data).map_err(|e| std::io::Error::other(format!("copy_asset: {:?}", e)))
    }

    fn load_sspj(&self) -> Result<sspj::sspj::SsProject, sspj::sspj_error::SspjError> {
        sspj::sspj::SsProject::load_from_maps(&self.sspj_name, &self.sspj_map)
    }
    fn is_zip(&self) -> bool {
        true
    }
}

pub fn create_asset_provider(path: &std::path::Path) -> Result<Box<dyn AssetProvider>, crate::converter::converter_error::ConverterError> {
    if path.as_os_str().is_empty() {
        return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Path is empty")));
    }
    if !path.exists() {
        return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Path does not exist: {:?}", path))));
    }
    if path.extension().is_some_and(|ext| ext == "zip") {
        let data = std::fs::read(path).map_err(crate::converter::converter_error::ConverterError::IoError)?;
        let file_name = path
            .file_name()
            .ok_or_else(|| crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid zip file name")))?
            .to_str()
            .ok_or_else(|| crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Non-UTF8 zip file name")))?;
        Ok(Box::new(ZipAssetProvider::new(file_name, &data)?))
    } else {
        Ok(Box::new(FileSystemAssetProvider::new(path)?))
    }
}
