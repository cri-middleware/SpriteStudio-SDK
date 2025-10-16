use std::collections::HashMap;
use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use ssab::ssab::SsAnimeBinaryT;
use sspj::ssloader::{SsLoader, SspjError};
use sspj::sspj::SsProject;
use crate::cache::{ssab_cache, ssqb_cache};
use crate::model_t::{ssabt, ssqbt};

#[derive(Debug, Clone)]
pub struct ConvertConfig {
    pub debug: bool,
    pub split: bool,
}
impl Default for ConvertConfig {
    fn default() -> Self {
        Self {
            debug: false,
            split: false,
        }
    }
}

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

pub fn convert_sspj_to_ssab_data(path: &Path, _config: &ConvertConfig) -> Result<HashMap<String, Vec<u8>>, SspjError> {
    let sspj_loader = SspjLoader::new(path);
    let sspj = match sspj_loader.load_ss_project() {
        Ok(sspj) => sspj,
        Err(error) => {
            println!("{:?} error: {:?}", path.as_os_str().to_string_lossy(), error);
            return Err(error);
        }
    };

    // pack sspj animation data and sequence data to ssab ssab and ssqb structures
    let mut ssab_model_list = sspj.anime_list.iter().map(| anime_pack| {
        ssabt::convert_ss_anime_binary_t(anime_pack, &sspj)
    }).collect::<Vec<SsAnimeBinaryT>>();

    let ssqb_model_list = sspj.sequence_list.iter().map(|sequence| {
        ssqbt::convert_ss_sequence_binary_t(sequence)
    }).collect::<Vec<ssab::ssqb::SsSequenceBinaryT>>();


    ssab_model_list.iter_mut().for_each(|_ss_anime_binary_t| {
        // optimize ssab ssab structures (e.g. add dependency animations, remove unused ss*e data)

    });

    // convert ssab structures to binary data
    let mut ssab_bin_map = ssab_model_list.iter().zip(sspj.animepack_names.iter()).map(|(anime_binary_t, f)| {
        let mut fb = flatbuffers::FlatBufferBuilder::new();
        let mut ssab_cache = ssab_cache::SsabCache::new();
        let offset = ssab_cache.create_shared_anime_binary(&mut fb, anime_binary_t);
        ssab::ssab::finish_ss_anime_binary_buffer(&mut fb, offset);
        let buf = fb.finished_data();
        let output_file = format!("{}.ssab", PathBuf::from(f).file_stem().unwrap().to_str().unwrap());
        (output_file, buf.to_vec())
    }).collect::<HashMap<String, Vec<u8>>>();

    // convert ssqb structures to binary data
    let ssqb_bin_map = ssqb_model_list.iter().zip(sspj.sequencepack_names.iter()).map(|(sequence_binary_t, f)| {
        let mut fb = flatbuffers::FlatBufferBuilder::new();
        let mut ssqb_cache = ssqb_cache::SsqbCache::new();
        let offset = ssqb_cache.create_shared_sequence_binary(&mut fb, sequence_binary_t);
        ssab::ssqb::finish_ss_sequence_binary_buffer(&mut fb, offset);
        let buf = fb.finished_data();

        let output_file = format!("{}.ssqb", PathBuf::from(f).file_stem().unwrap().to_str().unwrap());
        (output_file, buf.to_vec())
    }).collect::<HashMap<String, Vec<u8>>>();

    ssab_bin_map.extend(ssqb_bin_map);
    Ok(ssab_bin_map)
}

#[derive(Debug, Clone)]
pub struct ConverterConfig {
    convert_config: ConvertConfig,
    out_dir: Option<OsString>,
}
impl Default for ConverterConfig {
    fn default() -> Self {
        Self {
            convert_config: ConvertConfig::default(),
            out_dir: None,
        }
    }
}

pub fn convert_sspj_to_ssab_files(path: &Path, config: &ConverterConfig) -> Result<(), SspjError> {
    let convert_result = convert_sspj_to_ssab_data(path, &config.convert_config)?;

    let outdir_path = match &config.out_dir {
        Some(out_dir) => {
            PathBuf::from(&out_dir)
        }
        None => {
            let Some(parent) = path.parent() else {
                return Err(SspjError::FileError(std::io::Error::new(ErrorKind::NotFound, "not found: path parent: {}")));
            };
            let Some(stem) = path.file_stem() else {
                return Err(SspjError::FileError(std::io::Error::new(ErrorKind::NotFound, "{}")));
            };
            let dir = parent.join(format!("{}_ssab", stem.to_string_lossy()));
            if !dir.exists() {
                std::fs::create_dir(dir.as_path()).expect("TODO: panic message"); // TODO: FIXME Error handling
            }

            dir
        }
    };

    convert_result.iter().for_each(|(name, data)| {
        let output_path = outdir_path.join(name);
        std::fs::write(output_path.as_path(), data).expect("TODO: panic message"); // TODO: FIXME Error handling
    });

    Ok(())
}
