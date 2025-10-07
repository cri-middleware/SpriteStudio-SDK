mod ssqbt;
mod ssabt;

use std::collections::HashMap;
use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use sspj::ssloader::SsLoader;
use sspj::sspj::SsProject;
use sspj::ssloader::SspjError;
use ssab::ssab::{SsAnimeBinaryT};

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



pub struct ResultConvert {
    pub name: String,
    pub data: Vec<u8>,
    pub len: usize,
}

pub fn convert_sspj_to_ssab_data(path: &Path, config: &ConvertConfig) -> Result<Vec<ResultConvert>, SspjError> {
    let sspj_name = path.file_stem().unwrap().to_str().unwrap();
    let sspj_loader = crate::SspjLoader::new(path);
    let sspj = match sspj_loader.load_ss_project() {
        Ok(sspj) => sspj,
        Err(error) => {
            println!("{:?} error: {:?}", path.as_os_str().to_string_lossy(), error);
            return Err(error);
        }
    };

    // pack sspj animation data and sequence data to ssab ssab and ssqb structures
    let mut ssab_model_list = sspj.anime_list.iter().map(| anime_pack| {
        // TODO: implement
        ssabt::convert_ss_anime_binary_t(anime_pack, &sspj)
    }).collect::<Vec<SsAnimeBinaryT>>();

    let ssqb_model_list = sspj.sequence_list.iter().map(|sequence| {
        ssqbt::convert_ss_sequence_binary_t(sequence)
    }).collect::<Vec<ssab::ssqb::SsSequenceBinaryT>>();

    // optimize ssab ssab structures (e.g. set dependencies)
    ssab_model_list.iter_mut().for_each(|ss_anime_binary_t| {
        // TODO: implement
    });

    // convert ssab structures to binary data
    let mut ssab_bin_map = ssab_model_list.iter().zip(sspj.animepack_names.iter()).map(|(anime_binary, f)| {
        // TODO: implement

        let output_file = format!("{}_{}.ssab", sspj_name, PathBuf::from(f).file_stem().unwrap().to_str().unwrap());
        (output_file, Vec::new())
    }).collect::<HashMap<String, Vec<u8>>>();

    // convert ssqb structures to binary data
    let ssqb_bin_map = ssqb_model_list.iter().zip(sspj.sequencepack_names.iter()).map(|(sequence_binary_t, f)| {
        let mut fb = flatbuffers::FlatBufferBuilder::new();
        let offset = ssab::ssqb_cache::create_shared_sequence_binary(&mut fb, sequence_binary_t);
        fb.finish(offset, None);
        let buf = fb.finished_data();

        let output_file = format!("{}_{}.ssqb", sspj_name, PathBuf::from(f).file_stem().unwrap().to_str().unwrap());
        (output_file, buf.to_vec())
    }).collect::<HashMap<String, Vec<u8>>>();

    ssab_bin_map.extend(ssqb_bin_map);
    ssab_bin_map.iter().map(|(name, data)| {
        Ok(ResultConvert {
            name: name.clone(),
            data: data.clone(),
            len: data.len(),
        })
    }).collect::<Result<Vec<ResultConvert>, SspjError>>()
}

pub fn convert_sspj_to_ssab_files(path: &Path, config: ConverterConfig) -> Result<(), SspjError> {
    let convert_result = convert_sspj_to_ssab_data(path, &config.convert_config)?;

    let outdir_path = match config.out_dir {
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
            parent.join(format!("{}_ssab", stem.to_string_lossy()))
        }
    };

    if !outdir_path.as_path().exists() {
        std::fs::create_dir(outdir_path.as_path()).expect("TODO: panic message"); // TODO: FIXME Error handling
    }
    convert_result.iter().for_each(|result| {
        let output_path = outdir_path.join(&result.name);
        std::fs::write(output_path.as_path(), &result.data).expect("TODO: panic message"); // TODO: FIXME Error handling
    });

    Ok(())
}
