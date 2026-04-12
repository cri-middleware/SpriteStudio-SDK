use ssconverter::{ConverterConfig, convert_sspj_to_ssab_files};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

fn get_test_base_path() -> PathBuf {
    std::env::current_dir().expect("Failed to get the current directory").ancestors().nth(2).expect("Failed to get the base directory").to_path_buf()
}

fn create_zip_fixture(src_dir: &Path, zip_path: &Path) {
    let file = std::fs::File::create(zip_path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for entry in walkdir::WalkDir::new(src_dir) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let relative_path = path.strip_prefix(src_dir.parent().unwrap()).unwrap();
            let mut file = std::fs::File::open(path).unwrap();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            zip.start_file(relative_path.to_str().unwrap(), options).unwrap();
            zip.write_all(&buffer).unwrap();
        }
    }
    zip.finish().unwrap();
}

#[test]
fn test_convert_ssab_zip() {
    let src_dir = get_test_base_path().join("tests/allAttributeV7");
    let zip_path = get_test_base_path().join("target").join("allAttributeV7.zip");

    // 1. Create a zip file
    create_zip_fixture(&src_dir, &zip_path);

    // 2. Convert from zip file
    let out_dir = get_test_base_path().join("target").join("ssab_test_zip_output");
    if out_dir.exists() {
        let _ = std::fs::remove_dir_all(&out_dir);
    }
    std::fs::create_dir_all(&out_dir).unwrap();

    let mut config = ConverterConfig::default();
    config.output = Some(out_dir.as_os_str().to_os_string());

    let r = convert_sspj_to_ssab_files(&zip_path, &config, &|_msg: &str| {});
    match r {
        Ok(_) => {}
        Err(e) => panic!("{:?}", e),
    }

    // 3. Verify output zip file exists and contains specific files
    let out_zip_path = out_dir.with_extension("zip");
    assert!(out_zip_path.exists(), "Output zip file does not exist");

    let file = std::fs::File::open(&out_zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut files_in_zip = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        files_in_zip.push(file.name().to_string());
    }

    // Verify it contains the ssab and copied assets
    assert!(files_in_zip.iter().any(|f| f.ends_with(".ssab")), "SSAB files are missing");
    assert!(files_in_zip.iter().any(|f| f.ends_with(".png")), "PNG assets are missing");
}
