#[cfg(test)]
mod tests {
    use rayon::prelude::*;
    use ssconverter::{ConverterConfig, convert_sspj_to_ssab_files};
    use std::path::{Path, PathBuf};

    static TARGET_SSPJ_FILES: [&str; 5] = ["tests/ParticleEffect/ParticleEffect.sspj", "tests/allAttributeV7/allAttributeV7.sspj", "tests/allPartsV7/allPartsV7.sspj", "tests/overall/overall.sspj", "tests/Doll/Doll.sspj"];

    fn get_test_base_path() -> PathBuf {
        std::env::current_dir().expect("Failed to get the current directory").ancestors().nth(2).expect("Failed to get the base directory").to_path_buf()
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

            let r = convert_sspj_to_ssab_files(&sspj_path, &config, &|_msg: &str| {});
            match r {
                Ok(_) => {}
                Err(e) => panic!("{:?}", e),
            }
        });
    }

    /// Every emitted pack carries its schema version *in the bytes*.
    ///
    /// Neither `version` field declares a FlatBuffers default, so an absent one
    /// reads back as `0` and this comparison fails. That is the whole guard: with
    /// a schema default of `1`, writing `1` omitted the field, a reader
    /// substituted its own version for the absent one, and the runtime's
    /// `InvalidVersion` check could not fail for any input.
    #[test]
    fn an_emitted_pack_carries_its_schema_version() {
        let out_dir = get_test_base_path().join("target").join("ssab_version_test_output");
        if out_dir.exists() {
            std::fs::remove_dir_all(&out_dir).unwrap();
        }

        let mut checked = 0;
        for &f in TARGET_SSPJ_FILES.iter() {
            let p = Path::new(f);
            let pack_dir = out_dir.join(p.file_stem().unwrap());
            std::fs::create_dir_all(&pack_dir).unwrap();

            let mut config = ConverterConfig::default();
            config.output = Some(pack_dir.as_os_str().to_os_string());
            convert_sspj_to_ssab_files(&get_test_base_path().join(p), &config, &|_msg: &str| {}).expect("conversion failed");

            for entry in std::fs::read_dir(&pack_dir).unwrap() {
                let path = entry.unwrap().path();
                let buf = match path.extension().and_then(|e| e.to_str()) {
                    Some("ssab") | Some("ssqb") => std::fs::read(&path).unwrap(),
                    _ => continue,
                };
                let got = match path.extension().unwrap().to_str().unwrap() {
                    "ssab" => ssab::ssab::ss::format::root_as_ss_anime_binary(&buf).expect("not a readable .ssab").version(),
                    _ => ssab::ssqb::ss::format::root_as_ss_sequence_binary(&buf).expect("not a readable .ssqb").version(),
                };
                let want = match path.extension().unwrap().to_str().unwrap() {
                    "ssab" => ssab::SSAB_SCHEMA_VERSION,
                    _ => ssab::SSQB_SCHEMA_VERSION,
                };
                assert_eq!(got, want, "{} carries version {got}, expected {want}", path.display());
                checked += 1;
            }
        }
        assert!(checked > 0, "no packs were emitted, so nothing was checked");
    }
}
