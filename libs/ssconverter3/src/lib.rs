mod cache;
mod converter;
mod model_t;

pub mod utility;
pub use converter::converter::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod ffi;

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;
    use std::path::{Path, PathBuf};

    static TARGET_SSPJ_FILES: [&str; 4] = [
        "tests/ParticleEffect/ParticleEffect.sspj",
        "tests/allAttributeV7/allAttributeV7.sspj",
        "tests/allPartsV7/allPartsV7.sspj",
        "tests/overall/overall.sspj",
    ];

    fn get_test_base_path() -> PathBuf {
        std::env::current_dir()
            .expect("Failed to get the current directory")
            .ancestors()
            .nth(2)
            .expect("Failed to get the base directory")
            .to_path_buf()
    }

    #[test]
    fn test_parse_sspj() {
        TARGET_SSPJ_FILES.par_iter().for_each(|&f| {
            let p = Path::new(f);
            let sspj_loader = SspjLoader::new(&get_test_base_path().join(p).as_path());
            let Ok(_sspj) = sspj_loader.load_ss_project() else {
                panic!("not found: {}", p.as_os_str().to_string_lossy());
            };
            // println!("{:?}", _sspj);
        });
    }

    #[test]
    fn test_convert_ssab() {
        TARGET_SSPJ_FILES.par_iter().for_each(|&f| {
            let p = Path::new(f);
            let config = ConverterConfig::default();
            let r = convert_sspj_to_ssab_files(&get_test_base_path().join(p).as_path(), &config);
            match r {
                Ok(_) => {
                    assert!(true)
                }
                Err(e) => assert!(false, "{:?}", e),
            }
        });
    }
}
