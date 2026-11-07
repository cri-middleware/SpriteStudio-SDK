use crate::cache::{ssab_cache, ssqb_cache};
use crate::converter::{ssab_converter, ssqb_converter};
use crate::parallel::*;
use std::collections::HashMap;
use std::ffi::OsString;
use std::sync::atomic::{AtomicBool, Ordering};

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

// What each emitted binary holds, by file name: the animation names of an `.ssab`, the
// sequence names of an `.ssqb`.
pub(crate) type Contents = HashMap<String, Vec<String>>;

// A converted project: the project itself, each binary's bytes by file name, and their
// `Contents`.
pub(crate) type Converted = (sspj::sspj::SsProject, HashMap<String, Vec<u8>>, Contents);

// The in-memory conversion core. Crate-private because `AssetProvider` is: an
// external caller could never name the parameter type. The entry points the crate
// actually exposes are `convert_sspj_to_ssab_files*` (native) and the `web` bindings.
//
// The uncancellable wrapper only has a caller under `web`; native paths thread a
// real `should_stop` through so the GUI can abort a long conversion.
#[cfg(feature = "web")]
pub(crate) fn convert_sspj_to_ssab_data<F>(asset_provider: &dyn crate::asset_provider::AssetProvider, config: &ConvertConfig, on_log: &F) -> Result<Converted, crate::converter::converter_error::ConverterError>
where
    F: Fn(&str) + Sync + Send,
{
    convert_sspj_to_ssab_data_cancellable(asset_provider, config, on_log, &AtomicBool::new(false))
}

pub(crate) fn convert_sspj_to_ssab_data_cancellable<F>(asset_provider: &dyn crate::asset_provider::AssetProvider, config: &ConvertConfig, on_log: &F, should_stop: &AtomicBool) -> Result<Converted, crate::converter::converter_error::ConverterError>
where
    F: Fn(&str) + Sync + Send,
{
    if should_stop.load(Ordering::Relaxed) {
        return Err(crate::converter::converter_error::ConverterError::Aborted);
    }

    on_log("Loading SSPJ project file...");
    let sspj = asset_provider.load_sspj()?;

    on_log(&format!("Starting conversion for {} animations and {} sequences...", sspj.anime_list.len(), sspj.sequence_list.len()));
    let ssab_model_list = par_iter!(sspj.anime_list)
        .map(|anime_pack| {
            if should_stop.load(Ordering::Relaxed) {
                return Err(crate::converter::converter_error::ConverterError::Aborted);
            }
            ssab_converter::convert_ss_anime_binary_t(anime_pack, &sspj, asset_provider, config.embedded)
        })
        .collect::<Result<Vec<ssab::ssab::ss::format::SsAnimeBinaryT>, _>>()?;

    let ssqb_model_list = par_iter!(sspj.sequence_list)
        .map(|sequence| {
            if should_stop.load(Ordering::Relaxed) {
                return Err(crate::converter::converter_error::ConverterError::Aborted);
            }
            ssqb_converter::convert_ss_sequence_binary_t(sequence)
        })
        .collect::<Result<Vec<ssab::ssqb::ss::format::SsSequenceBinaryT>, _>>()?;

    let ssab_bins = par_iter!(ssab_model_list)
        .zip(par_iter!(sspj.animepack_names))
        .map(|(anime_binary_t, f)| {
            if should_stop.load(Ordering::Relaxed) {
                return Err(crate::converter::converter_error::ConverterError::Aborted);
            }
            let mut fb = flatbuffers::FlatBufferBuilder::new();
            let offset = if config.optimize {
                let mut ssab_cache = ssab_cache::SsabCache::new();
                ssab_cache.create_shared_anime_binary(&mut fb, anime_binary_t)
            } else {
                anime_binary_t.pack(&mut fb)
            };
            ssab::ssab::ss::format::finish_ss_anime_binary_buffer(&mut fb, offset);
            let buf = fb.finished_data();
            // note: In SpriteStudio, ssae files must be placed in a single directory specified in sspj.
            // Therefore, name collisions between different directories do not occur.
            let file_stem = std::path::PathBuf::from(f)
                .file_stem()
                .ok_or_else(|| crate::converter::converter_error::ConverterError::ConversionFailed("No file stem".into()))?
                .to_str()
                .ok_or_else(|| crate::converter::converter_error::ConverterError::ConversionFailed("Invalid UTF-8 in file stem".into()))?
                .to_string();
            let output_file = [file_stem, ssab::ssab::ss::format::SS_ANIME_BINARY_EXTENSION.to_string()].join(".");
            let names = anime_binary_t.animations.iter().map(|animation| animation.name.clone()).collect::<Vec<_>>();
            Ok::<_, crate::converter::converter_error::ConverterError>((output_file, buf.to_vec(), names))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let ssqb_bins = par_iter!(ssqb_model_list)
        .zip(par_iter!(sspj.sequencepack_names))
        .map(|(sequence_binary_t, f)| {
            if should_stop.load(Ordering::Relaxed) {
                return Err(crate::converter::converter_error::ConverterError::Aborted);
            }
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
            let names = sequence_binary_t.sequence_list.iter().map(|sequence| sequence.name.clone()).collect::<Vec<_>>();
            Ok::<_, crate::converter::converter_error::ConverterError>((output_file, buf.to_vec(), names))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut bin_map = HashMap::new();
    let mut contents = Contents::new();
    for (output_file, data, names) in ssab_bins.into_iter().chain(ssqb_bins) {
        contents.insert(output_file.clone(), names);
        bin_map.insert(output_file, data);
    }
    Ok((sspj, bin_map, contents))
}

#[derive(Debug, Clone, Default)]
pub struct ConverterConfig {
    pub convert_config: ConvertConfig,
    pub output: Option<OsString>,
}

#[cfg(feature = "native")]
pub fn convert_sspj_to_ssab_files<F>(path: &std::path::Path, config: &ConverterConfig, on_log: &F) -> Result<(), crate::converter::converter_error::ConverterError>
where
    F: Fn(&str) + Sync + Send,
{
    convert_sspj_to_ssab_files_cancellable(path, config, on_log, &AtomicBool::new(false))
}

#[cfg(feature = "native")]
pub fn convert_sspj_to_ssab_files_cancellable<F>(path: &std::path::Path, config: &ConverterConfig, on_log: &F, should_stop: &AtomicBool) -> Result<(), crate::converter::converter_error::ConverterError>
where
    F: Fn(&str) + Sync + Send,
{
    let asset_provider = crate::asset_provider::create_asset_provider(path)?;
    let (sspj, convert_result, contents) = convert_sspj_to_ssab_data_cancellable(asset_provider.as_ref(), &config.convert_config, on_log, should_stop)?;

    let outdir_path = match &config.output {
        Some(out_dir) => std::path::PathBuf::from(&out_dir),
        None => crate::utils::get_ssab_output_directory(path, config.convert_config.embedded)?,
    };

    on_log(&format!("Writing output to: {}", outdir_path.display()));

    if !outdir_path.exists() {
        let r = std::fs::create_dir_all(outdir_path.as_path());
        match r {
            Ok(_) => (),
            Err(e) => {
                return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(e.kind(), format!("create_dir_all: {}: {:?}", outdir_path.as_os_str().to_string_lossy(), e))));
            }
        }
    }

    on_log(&format!("Generating {} binary files...", convert_result.len()));
    par_iter!(convert_result).try_for_each(|(name, data)| {
        if should_stop.load(Ordering::Relaxed) {
            return Err(crate::converter::converter_error::ConverterError::Aborted);
        }
        let output_path = outdir_path.join(name);
        match std::fs::write(output_path.as_path(), data) {
            Ok(_) => Ok(()),
            Err(e) => Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(e.kind(), format!("write: {}: {:?}", output_path.as_os_str().to_string_lossy(), e)))),
        }
    })?;

    // One line per binary, by full path so that a batch's interleaved logs still say
    // which project each came from, naming what a player can be asked to play.
    let mut written = contents.iter().collect::<Vec<_>>();
    written.sort();
    for (name, names) in written {
        let kind = if name.ends_with(ssab::ssqb::ss::format::SS_SEQUENCE_BINARY_EXTENSION) { "sequences" } else { "animations" };
        let held = if names.is_empty() { "none".to_string() } else { names.join(", ") };
        on_log(&format!("Wrote {} ({}: {})", outdir_path.join(name).display(), kind, held));
    }

    if !config.convert_config.embedded {
        let assets = crate::utils::get_assets_of_ssab_files(&convert_result, &sspj);
        on_log("Copying referenced assets...");
        assets
            .iter()
            .map(|(dst_rel_path, src_rel_path)| {
                if should_stop.load(Ordering::Relaxed) {
                    return Err(crate::converter::converter_error::ConverterError::Aborted);
                }
                on_log(&format!("Copying asset: {}", src_rel_path));
                let dst = outdir_path.join(dst_rel_path);
                let dst_parent = dst.parent().ok_or_else(|| crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid output path")))?;
                if !dst_parent.exists() {
                    let r = std::fs::create_dir_all(dst_parent);
                    match r {
                        Ok(_) => (),
                        Err(e) => {
                            return Err(crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(e.kind(), format!("create_dir_all: {}: {:?}", dst_parent.as_os_str().to_string_lossy(), e))));
                        }
                    };
                }
                let data = asset_provider.read_asset(src_rel_path).map_err(|e| crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(e.kind(), format!("read_asset: {}: {:?}", src_rel_path, e))))?;
                std::fs::write(&dst, data).map_err(|e| crate::converter::converter_error::ConverterError::IoError(std::io::Error::new(e.kind(), format!("write: dest:{} {:?}", dst.as_os_str().to_string_lossy(), e))))?;
                Ok(())
            })
            .collect::<Result<Vec<_>, _>>()?;
    }

    Ok(())
}
