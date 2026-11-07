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
    /// reads back as `0` and this comparison fails.
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

    /// The log says what each written binary holds, by full path: the animation names
    /// of an `.ssab`, the sequence names of an `.ssqb`. A script or a coding assistant
    /// reads the names to play from there instead of from the project's XML.
    #[test]
    fn the_log_names_what_each_written_binary_holds() {
        let out_dir = get_test_base_path().join("target").join("ssab_contents_test_output");
        if out_dir.exists() {
            std::fs::remove_dir_all(&out_dir).unwrap();
        }
        std::fs::create_dir_all(&out_dir).unwrap();

        let mut config = ConverterConfig::default();
        config.output = Some(out_dir.as_os_str().to_os_string());
        let log = std::sync::Mutex::new(Vec::new());
        convert_sspj_to_ssab_files(&get_test_base_path().join("tests/Ringo/Ringo.sspj"), &config, &|msg: &str| log.lock().unwrap().push(msg.to_string())).expect("conversion failed");
        let log = log.into_inner().unwrap();

        let mut checked = 0;
        for entry in std::fs::read_dir(&out_dir).unwrap() {
            let path = entry.unwrap().path();
            let buf = match path.extension().and_then(|e| e.to_str()) {
                Some("ssab") | Some("ssqb") => std::fs::read(&path).unwrap(),
                _ => continue,
            };
            let (kind, names) = match path.extension().unwrap().to_str().unwrap() {
                "ssab" => ("animations", ssab::ssab::ss::format::root_as_ss_anime_binary(&buf).expect("not a readable .ssab").animations().iter().map(|a| a.name().to_string()).collect::<Vec<_>>()),
                _ => ("sequences", ssab::ssqb::ss::format::root_as_ss_sequence_binary(&buf).expect("not a readable .ssqb").sequence_list().iter().map(|s| s.name().to_string()).collect::<Vec<_>>()),
            };
            assert!(!names.is_empty(), "{} holds no {kind}, so the fixture no longer exercises the list", path.display());
            let want = format!("Wrote {} ({kind}: {})", path.display(), names.join(", "));
            assert!(log.contains(&want), "the log has no line {want:?}:\n{log:#?}");
            checked += 1;
        }
        assert!(checked >= 2, "expected an .ssab and an .ssqb, checked {checked}");
    }

    /// The output can be the project's own folder, where the assets already are. They
    /// stay as they were, and a read-only one does not stop the conversion.
    #[test]
    fn an_output_in_the_project_leaves_its_assets_alone() {
        let project = get_test_base_path().join("target").join("ssab_in_place_test_output");
        if project.exists() {
            std::fs::remove_dir_all(&project).unwrap();
        }
        std::fs::create_dir_all(&project).unwrap();
        for entry in std::fs::read_dir(get_test_base_path().join("tests/Ringo")).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                std::fs::copy(&path, project.join(path.file_name().unwrap())).unwrap();
            }
        }

        let image = project.join("Ringo.png");
        let authored = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000_000);
        std::fs::File::options().write(true).open(&image).unwrap().set_modified(authored).unwrap();
        let before = std::fs::read(&image).unwrap();
        let writable = image.metadata().unwrap().permissions();
        let mut read_only = writable.clone();
        read_only.set_readonly(true);
        std::fs::set_permissions(&image, read_only).unwrap();

        let mut config = ConverterConfig::default();
        config.output = Some(project.as_os_str().to_os_string());
        let converted = convert_sspj_to_ssab_files(&project.join("Ringo.sspj"), &config, &|_msg: &str| {});

        let modified = image.metadata().unwrap().modified().unwrap();
        let after = std::fs::read(&image).unwrap();
        std::fs::set_permissions(&image, writable).unwrap();

        converted.expect("conversion failed");
        assert!(project.join("Ringo.ssab").exists());
        assert_eq!(modified, authored);
        assert_eq!(after, before);
    }

    /// `canvas_size` is the authored canvas, not the canvas minus the pivot. A pivot
    /// of `0 -0.5` is the ordinary way to author a character standing on the ground,
    /// which is why `Doll` and `ParticleEffect` both carry one.
    #[test]
    fn canvas_size_is_independent_of_the_pivot() {
        let out_dir = get_test_base_path().join("target").join("ssab_canvas_test_output");
        if out_dir.exists() {
            std::fs::remove_dir_all(&out_dir).unwrap();
        }

        let mut checked = 0;
        let mut saw_offset_pivot = false;
        for &f in TARGET_SSPJ_FILES.iter() {
            let p = Path::new(f);
            let pack_dir = out_dir.join(p.file_stem().unwrap());
            std::fs::create_dir_all(&pack_dir).unwrap();

            let mut config = ConverterConfig::default();
            config.output = Some(pack_dir.as_os_str().to_os_string());
            convert_sspj_to_ssab_files(&get_test_base_path().join(p), &config, &|_msg: &str| {}).expect("conversion failed");

            for entry in std::fs::read_dir(&pack_dir).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().and_then(|e| e.to_str()) != Some("ssab") {
                    continue;
                }
                let buf = std::fs::read(&path).unwrap();
                let pack = ssab::ssab::ss::format::root_as_ss_anime_binary(&buf).expect("not a readable .ssab");
                for anim in pack.animations() {
                    let canvas = anim.canvas_size();
                    let pivot = anim.pivot();
                    assert_eq!(canvas.v1(), canvas.v1().trunc(), "{} / {} has a fractional canvas width {}", path.display(), anim.name(), canvas.v1());
                    assert_eq!(canvas.v2(), canvas.v2().trunc(), "{} / {} has a fractional canvas height {}", path.display(), anim.name(), canvas.v2());
                    assert!(canvas.v1() > 0.0 && canvas.v2() > 0.0, "{} / {} has an empty canvas", path.display(), anim.name());
                    saw_offset_pivot |= pivot.v1() != 0.0 || pivot.v2() != 0.0;
                    checked += 1;
                }
            }
        }
        assert!(checked > 0, "no animations were emitted, so nothing was checked");
        assert!(saw_offset_pivot, "no animation carried a non-zero pivot, so the fixture no longer exercises the case");
    }
}
