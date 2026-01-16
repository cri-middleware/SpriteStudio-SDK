use rayon::prelude::*;
use crate::cache::{ssab_cache, ssqb_cache};
use std::collections::HashMap;
use std::ffi::OsString;
use std::io::ErrorKind;
use crate::converter::{ssab_converter, ssqb_converter};
use sspj::ssloader::SsLoader;

#[derive(Debug, Clone)]
pub struct ConvertConfig {
    pub embedded: bool,
    pub debug: bool,
    pub split: bool,
    pub shared: bool,
}
impl Default for ConvertConfig {
    fn default() -> Self {
        Self {
            embedded: false,
            debug: false,
            split: false,
            shared: true,
        }
    }
}

pub struct SspjLoader {
    path_buf: std::path::PathBuf,
}
impl SspjLoader {
    pub fn new(path: &std::path::Path) -> Self {
        SspjLoader {
            path_buf: path.to_path_buf(),
        }
    }
    pub fn load_ss_project(&self) -> Result<sspj::sspj::SsProject, sspj::sspj_error::SspjError> {
        sspj::sspj::SsProject::load_from_file(self.path_buf.as_path())
    }
}

pub fn convert_sspj_to_ssab_data(path: &std::path::Path, config: &ConvertConfig, ) -> Result<(sspj::sspj::SsProject, HashMap<String, Vec<u8>>), crate::converter::converter_error::ConverterError> {
    let sspj = SspjLoader::new(path).load_ss_project()?;
    let Some(sspj_parent) = path.parent() else {
        return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(ErrorKind::NotFound, format!("not found: path parent: {}", path.as_os_str().to_string_lossy()))))
    };

    // pack sspj animation data and sequence data to ssab ssab and ssqb structures
    let ssab_model_list = sspj.anime_list.par_iter().map(|anime_pack| ssab_converter::convert_ss_anime_binary_t(anime_pack, &sspj, &sspj_parent, config.embedded)).collect::<Result<Vec<ssab::ssab::SsAnimeBinaryT>, _>>()?;

    let ssqb_model_list = sspj.sequence_list.par_iter().map(|sequence| ssqb_converter::convert_ss_sequence_binary_t(sequence)).collect::<Result<Vec<ssab::ssqb::SsSequenceBinaryT>, _>>()?;

    // convert ssab structures to binary data
    let mut ssab_bin_map = ssab_model_list.par_iter().zip(sspj.animepack_names.par_iter()).map(|(anime_binary_t, f)| {
        let mut fb = flatbuffers::FlatBufferBuilder::new();
        let offset = if config.shared {
            let mut ssab_cache = ssab_cache::SsabCache::new();
            ssab_cache.create_shared_anime_binary(&mut fb, anime_binary_t)
        } else {
            anime_binary_t.pack(&mut fb)
        };
        ssab::ssab::finish_ss_anime_binary_buffer(&mut fb, offset);
        let buf = fb.finished_data();
        let output_file = [std::path::PathBuf::from(f).file_stem().unwrap().to_str().unwrap(), ssab::ssab::SS_ANIME_BINARY_EXTENSION].join(".");
        (output_file, buf.to_vec())
    }).collect::<HashMap<String, Vec<u8>>>();

    // convert ssqb structures to binary data
    let ssqb_bin_map = ssqb_model_list.par_iter().zip(sspj.sequencepack_names.par_iter()).map(|(sequence_binary_t, f)| {
        let mut fb = flatbuffers::FlatBufferBuilder::new();
        let offset = if config.shared {
            let mut ssqb_cache = ssqb_cache::SsqbCache::new();
            ssqb_cache.create_shared_sequence_binary(&mut fb, sequence_binary_t)
        } else {
            sequence_binary_t.pack(&mut fb)
        };
        ssab::ssqb::finish_ss_sequence_binary_buffer(&mut fb, offset);
        let buf = fb.finished_data();

        let output_file = [std::path::PathBuf::from(f).file_stem().unwrap().to_str().unwrap(), ssab::ssqb::SS_SEQUENCE_BINARY_EXTENSION].join(".");
        (output_file, buf.to_vec())
    }).collect::<HashMap<String, Vec<u8>>>();


    ssab_bin_map.extend(ssqb_bin_map);
    Ok((sspj, ssab_bin_map))
}

#[derive(Debug, Clone)]
pub struct ConverterConfig {
    pub convert_config: ConvertConfig,
    pub out_dir: Option<OsString>,
}
impl Default for ConverterConfig {
    fn default() -> Self {
        Self {
            convert_config: ConvertConfig::default(),
            out_dir: None,
        }
    }
}

pub fn convert_sspj_to_ssab_files(path: &std::path::Path, config: &ConverterConfig) -> Result<(), crate::converter::converter_error::ConverterError> {
    let (sspj, convert_result) = convert_sspj_to_ssab_data(path, &config.convert_config)?;

    let outdir_path = match &config.out_dir {
        Some(out_dir) => std::path::PathBuf::from(&out_dir),
        None => crate::utils::get_ssab_outout_directory(path, config.convert_config.embedded)?
    };

    if !outdir_path.exists() {
        let Ok(_) = std::fs::create_dir_all(outdir_path.as_path()) else {
            return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(ErrorKind::PermissionDenied, format!("can't create a directory: {}", outdir_path.as_path().to_str().unwrap()))))
        };
    }

    convert_result.par_iter().try_for_each(|(name, data)| {
        let output_path = outdir_path.join(name);
        match std::fs::write(output_path.as_path(), data) {
            Ok(_) => Ok(()),
            Err(_) => Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(ErrorKind::Other, format!("can't write a ssab file: {}", output_path.as_path().to_str().unwrap()))))
        }
    })?;

    if !config.convert_config.embedded {
        let src_path_buf = std::path::PathBuf::from(path);
        let src_prefix = src_path_buf.parent().unwrap();
        let assets = crate::delivery::get_assets_of_ssab_using(&convert_result, &sspj);
        assets.iter().map(|asset| {
            let src = src_prefix.join(asset);
            let dst = outdir_path.join(asset);
            let Ok(_) = std::fs::create_dir_all(dst.parent().unwrap()) else {
                return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(ErrorKind::PermissionDenied, format!("can't create a directory: {}", outdir_path.as_path().to_str().unwrap()))))
            };
            let r = std::fs::copy(&src, &dst);
            if r.is_err() {
                return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(ErrorKind::Other, format!("can't copy a file: {}", dst.as_path().to_str().unwrap()))));
            }
            Ok(())
        }).collect::<Result<Vec<_>, _>>()?;
    }

    Ok(())
}
