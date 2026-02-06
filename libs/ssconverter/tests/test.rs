
#[cfg(test)]
mod tests {
    use rayon::prelude::*;
    use std::path::{Path, PathBuf};
    use ssconverter::{convert_sspj_to_ssab_files, ConverterConfig};

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
    fn test_convert_ssab() {
        TARGET_SSPJ_FILES.par_iter().for_each(|&f| {
            let p = Path::new(f);
            let sspj_path = get_test_base_path().join(p);
            
            // Create a temporary output directory in target
            let out_dir = get_test_base_path().join("target").join("ssab_test_output").join(p.file_stem().unwrap());
            if out_dir.exists() {
                let _ = std::fs::remove_dir_all(&out_dir);
            }
            std::fs::create_dir_all(&out_dir).unwrap();

            let mut config = ConverterConfig::default();
            config.output = Some(out_dir.as_os_str().to_os_string());
            
            let r = convert_sspj_to_ssab_files(&sspj_path, &config, &|msg: &str| {});
            match r {
                Ok(_) => {
                    assert!(true)
                }
                Err(e) => assert!(false, "{:?}", e),
            }
        });
    }
}
