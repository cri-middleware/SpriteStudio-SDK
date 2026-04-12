use crate::cache::{ssab_cache, ssqb_cache};
use crate::converter::{ssab_converter, ssqb_converter};
use rayon::prelude::*;
use std::collections::HashMap;
use std::ffi::OsString;

#[derive(Debug, Clone)]
pub struct ConvertConfig {
    pub embedded: bool,
    pub debug: bool,
    pub split: bool,
    pub optimize: bool,
}
impl Default for ConvertConfig {
    fn default() -> Self {
        Self { embedded: false, debug: false, split: false, optimize: true }
    }
}

pub fn convert_sspj_to_ssab_data<F>(
    asset_provider: &dyn crate::asset_provider::AssetProvider,
    config: &ConvertConfig,
    _on_log: &F,
) -> Result<(sspj::sspj::SsProject, HashMap<String, Vec<u8>>), crate::converter::converter_error::ConverterError>
where
    F: Fn(&str) + Sync + Send,
{
    let sspj = asset_provider.load_sspj()?;

    // pack sspj animation data and sequence data to ssab ssab and ssqb structures
    let ssab_model_list = sspj
        .anime_list
        .par_iter()
        .map(|anime_pack| ssab_converter::convert_ss_anime_binary_t(anime_pack, &sspj, asset_provider, config.embedded))
        .collect::<Result<Vec<ssab::ssab::ss::format::SsAnimeBinaryT>, _>>()?;

    let ssqb_model_list = sspj.sequence_list.par_iter().map(ssqb_converter::convert_ss_sequence_binary_t).collect::<Result<Vec<ssab::ssqb::ss::format::SsSequenceBinaryT>, _>>()?;

    // convert ssab structures to binary data
    let mut ssab_bin_map = ssab_model_list
        .par_iter()
        .zip(sspj.animepack_names.par_iter())
        .map(|(anime_binary_t, f)| {
            let mut fb = flatbuffers::FlatBufferBuilder::new();
            let offset = if config.optimize {
                let mut ssab_cache = ssab_cache::SsabCache::new();
                ssab_cache.create_shared_anime_binary(&mut fb, anime_binary_t)
            } else {
                anime_binary_t.pack(&mut fb)
            };
            ssab::ssab::ss::format::finish_ss_anime_binary_buffer(&mut fb, offset);
            let buf = fb.finished_data();
            let file_stem = std::path::PathBuf::from(f)
                .file_stem()
                .ok_or_else(|| crate::converter::converter_error::ConverterError::ConversionFailed("No file stem".into()))?
                .to_str()
                .ok_or_else(|| crate::converter::converter_error::ConverterError::ConversionFailed("Invalid UTF-8 in file stem".into()))?
                .to_string();
            let output_file = [file_stem, ssab::ssab::ss::format::SS_ANIME_BINARY_EXTENSION.to_string()].join(".");
            Ok::<_, crate::converter::converter_error::ConverterError>((output_file, buf.to_vec()))
        })
        .collect::<Result<HashMap<String, Vec<u8>>, _>>()?;

    // convert ssqb structures to binary data
    let ssqb_bin_map = ssqb_model_list
        .par_iter()
        .zip(sspj.sequencepack_names.par_iter())
        .map(|(sequence_binary_t, f)| {
            let mut fb = flatbuffers::FlatBufferBuilder::new();
            let offset = if config.optimize {
                let mut ssqb_cache = ssqb_cache::SsqbCache::new();
                ssqb_cache.create_shared_sequence_binary(&mut fb, sequence_binary_t)
            } else {
                sequence_binary_t.pack(&mut fb)
            };
            ssab::ssqb::ss::format::finish_ss_sequence_binary_buffer(&mut fb, offset);
            let buf = fb.finished_data();

            let file_stem = std::path::PathBuf::from(f)
                .file_stem()
                .ok_or_else(|| crate::converter::converter_error::ConverterError::ConversionFailed("No file stem".into()))?
                .to_str()
                .ok_or_else(|| crate::converter::converter_error::ConverterError::ConversionFailed("Invalid UTF-8 in file stem".into()))?
                .to_string();
            let output_file = [file_stem, ssab::ssqb::ss::format::SS_SEQUENCE_BINARY_EXTENSION.to_string()].join(".");
            Ok::<_, crate::converter::converter_error::ConverterError>((output_file, buf.to_vec()))
        })
        .collect::<Result<HashMap<String, Vec<u8>>, _>>()?;

    ssab_bin_map.extend(ssqb_bin_map);
    Ok((sspj, ssab_bin_map))
}

#[derive(Debug, Clone, Default)]
pub struct ConverterConfig {
    pub convert_config: ConvertConfig,
    pub output: Option<OsString>,
}

pub fn convert_sspj_to_ssab_files<F>(path: &std::path::Path, config: &ConverterConfig, on_log: &F) -> Result<(), crate::converter::converter_error::ConverterError>
where
    F: Fn(&str) + Sync + Send,
{
    let asset_provider = crate::asset_provider::create_asset_provider(path)?;
    let (sspj, convert_result) = convert_sspj_to_ssab_data(asset_provider.as_ref(), &config.convert_config, on_log)?;

    let outdir_path = match &config.output {
        Some(out_dir) => std::path::PathBuf::from(&out_dir),
        None => crate::utils::get_ssab_output_directory(path, config.convert_config.embedded)?,
    };

    if asset_provider.is_zip() {
        use std::io::Write;

        let out_zip_path = outdir_path.with_extension("zip");
        let file = std::fs::File::create(&out_zip_path).map_err(|e| crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(e.kind(), format!("create zip file: {:?}", e))))?;
        let mut zip_writer = zip::ZipWriter::new(file);

        let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for (name, data) in convert_result.iter() {
            zip_writer.start_file(name, options).map_err(|e| crate::converter::converter_error::ConverterError::IoError(std::io::Error::other(format!("zip start_file: {:?}", e))))?;
            zip_writer.write_all(data).map_err(|e| crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(e.kind(), format!("zip write: {:?}", e))))?;
        }

        if !config.convert_config.embedded {
            let assets = crate::utils::get_assets_of_ssab_files(&convert_result, &sspj);
            for (dst_rel_path, src_rel_path) in assets {
                let data = asset_provider
                    .read_asset(&src_rel_path)
                    .map_err(|e| crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(e.kind(), format!("read_asset: {}: {:?}", src_rel_path, e))))?;
                zip_writer
                    .start_file(dst_rel_path.clone(), options)
                    .map_err(|e| crate::converter::converter_error::ConverterError::IoError(std::io::Error::other(format!("zip start_file: {:?}", e))))?;
                zip_writer.write_all(&data).map_err(|e| crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(e.kind(), format!("zip write: {:?}", e))))?;
            }
        }

        zip_writer.finish().map_err(|e| crate::converter::converter_error::ConverterError::IoError(std::io::Error::other(format!("zip finish: {:?}", e))))?;
    } else {
        if !outdir_path.exists() {
            let r = std::fs::create_dir_all(outdir_path.as_path());
            match r {
                Ok(_) => (),
                Err(e) => {
                    return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(
                        e.kind(),
                        format!("create_dir_all: {}: {:?}", outdir_path.as_os_str().to_string_lossy(), e),
                    )));
                }
            }
        }

        convert_result.par_iter().try_for_each(|(name, data)| {
            let output_path = outdir_path.join(name);
            match std::fs::write(output_path.as_path(), data) {
                Ok(_) => Ok(()),
                Err(e) => Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(e.kind(), format!("write: {}: {:?}", output_path.as_os_str().to_string_lossy(), e)))),
            }
        })?;

        if !config.convert_config.embedded {
            let assets = crate::utils::get_assets_of_ssab_files(&convert_result, &sspj);
            assets
                .iter()
                .map(|(dst_rel_path, src_rel_path)| {
                    let dst = outdir_path.join(dst_rel_path);
                    let dst_parent = dst.parent().unwrap();
                    if !dst_parent.exists() {
                        let r = std::fs::create_dir_all(dst_parent);
                        match r {
                            Ok(_) => (),
                            Err(e) => {
                                return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(
                                    e.kind(),
                                    format!("create_dir_all: {}: {:?}", dst_parent.as_os_str().to_string_lossy(), e),
                                )));
                            }
                        };
                    }
                    //let data = asset_provider.read_asset(src_rel_path).map_err(|e| crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(e.kind(), format!("read_asset: {}: {:?}", src_rel_path, e))))?;
                    //let r = std::fs::write(&dst, data);
                    let r = asset_provider.copy_asset(src_rel_path, dst.as_os_str().to_str().unwrap());
                    match r {
                        Ok(_) => (),
                        Err(e) => {
                            return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(e.kind(), format!("write: dest:{} {:?}", dst.as_os_str().to_string_lossy(), e))));
                        }
                    }
                    Ok(())
                })
                .collect::<Result<Vec<_>, _>>()?;
        }
    }

    Ok(())
}
