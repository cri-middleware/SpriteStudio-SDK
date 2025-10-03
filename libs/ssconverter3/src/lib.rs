use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use sspj::ssloader::SsLoader;
use sspj::sspj::SsProject;
use sspj::ssloader::SspjError;
use ssfb::*;

pub struct SspjLoader {
    path_buf: PathBuf,
}
impl SspjLoader {
    pub fn new(path: &Path) -> Self {
        SspjLoader {
            path_buf: path.to_path_buf()
        }
    }
    pub fn load_ss_project(&self) -> Result<SsProject, SspjError> {
        SsProject::load_from_file(self.path_buf.as_path())
    }
}

pub struct ConverterConfig {
    pub debug: bool,
    pub split: bool,
}
impl Default for ConverterConfig {
    fn default() -> Self {
        Self {
            debug: false,
            split: false,
        }
    }
}

pub fn convert_ss_project(path: &Path, config: ConverterConfig) -> Result<HashMap<String, Vec<u8>>, SspjError> {
    let sspj_loader = crate::SspjLoader::new(path);
    let sspj = match sspj_loader.load_ss_project() {
        Ok(sspj) => sspj,
        Err(error) => {
            println!("{:?} error: {:?}", path.as_os_str().to_string_lossy(), error);
            return Err(error);
        }
    };
    println!("{:?}", sspj);

    // pack sspj animation data to ssab flutbuffers structures
    sspj.anime_list.iter().map(|_anime| {

    });

    // pack sspj sequence data to ssqb flutbuffers structures
    sspj.sequence_list.iter().map(|_sequence| {

    });

    // convert ssab flutbuffers structures to binary data

    // convert ssqb flutbuffers structures to binary data

    Ok(HashMap::new())
}

pub fn convert_ss_project_to_ssab(path: &Path, config: ConverterConfig) -> Result<(), SspjError> {
    let ssab = convert_ss_project(path, config)?;

    let Some(parent) = path.parent() else {
        return Err(SspjError::FileError(std::io::Error::new(ErrorKind::NotFound, "")));
    };
    ssab.iter().for_each(|(key, value)| {
        let output_path = parent.join(key).with_extension("ssab");
        std::fs::write(output_path.as_path(), value).expect("TODO: panic message"); // TODO: FIXME
    });

    Ok(())
}

fn aa(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    let Ok(sspj) = sspj::sspj::SsProject::load_from_file(path) else {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "sspj load error"));
    };

    let mut anime_binary = ssfb::ssab::ss::data::SsAnimeBinaryT::default();
    anime_binary.version;

    Ok(Vec::new())
}
