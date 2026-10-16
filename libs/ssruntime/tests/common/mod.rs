//! The fixture packs the integration tests play.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

/// `tests/<project>/<project>.sspj`, converted once per run into this test
/// binary's own scratch directory, so neither the unit tests' fixture
/// directories nor another test binary running alongside are touched.
///
/// Miri cannot run the converter in any reasonable time, so under Miri the
/// packs come from `SSRUNTIME_FIXTURES`: a directory with one converted
/// directory per project, which CI's Miri job fills with ssconverter-cli.
pub fn converted(project: &str) -> PathBuf {
    if cfg!(miri) {
        let dir = std::env::var_os("SSRUNTIME_FIXTURES").expect("under Miri, SSRUNTIME_FIXTURES names a directory of projects converted natively");
        return PathBuf::from(dir).join(project);
    }
    static CONVERTED: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join(env!("CARGO_CRATE_NAME")).join(project);
    let mut converted = CONVERTED.lock().unwrap_or_else(PoisonError::into_inner);
    if !converted.iter().any(|done| done == project) {
        let _ = std::fs::remove_dir_all(&out);
        let sspj = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests").join(project).join(format!("{project}.sspj"));
        let config = ssconverter::ConverterConfig { output: Some(out.clone().into()), ..Default::default() };
        ssconverter::convert_sspj_to_ssab_files(&sspj, &config, &|_: &str| {}).unwrap_or_else(|e| panic!("convert {project}: {e}"));
        converted.push(project.to_string());
    }
    out
}
