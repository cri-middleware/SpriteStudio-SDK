use std::env;
use std::ffi::{OsStr, OsString};
use std::thread::JoinHandle;
use ssconverter3;

fn print_usage(prog: &OsStr) {
    println!("usage: {} <sspj_file OR directory> [sspj_file OR directory...]", prog.to_string_lossy());
}
fn main() {
    let args: Vec<OsString> = env::args_os().collect();
    if args.len() <= 1 {
        print_usage(args[0].as_os_str());
        return;
    }
    let sspj_paths = ssconverter3::utility::parse_args(&args);
    if sspj_paths.is_empty() {
        print_usage(&args[0]);
        return;
    }
    let config = ssconverter3::ConverterConfig::default();

    let mut handlers: Vec<JoinHandle<()>> = Vec::new();
    sspj_paths.into_iter().for_each(|p| {
        println!("convert {:?}", p.display());
        handlers.push({
            let c = config.clone();
            std::thread::spawn(move || {
                let r = ssconverter3::convert_sspj_to_ssab_files(p.as_path(), &c);
                match r {
                    Ok(_) => (),
                    Err(e) => println!("{:?}: {:?}", p.display(), e),
                }
            })
        })
    });
    handlers.into_iter().for_each(|h| {h.join().unwrap()});
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use super::*;
    use rayon::prelude::*;

    static TARGET_SSPJ_FILES: [&str; 4] = [
        "tests/ParticleEffect/ParticleEffect.sspj",
        "tests/allAttributeV7/allAttributeV7.sspj",
        "tests/allPartsV7/allPartsV7.sspj",
        "tests/overall/overall.sspj",
    ];

    fn get_test_base_path() -> PathBuf {
        std::env::current_dir()
            .expect("Failed to get the current directory")
            .parent()
            .expect("Failed to get the base directory")
            .to_path_buf()
    }

    #[test]
    fn test_convert_ssab() {
        TARGET_SSPJ_FILES.par_iter().for_each(|&f| {
            let p = Path::new(f);
            let config = ssconverter3::ConverterConfig::default();
            let r = ssconverter3::convert_sspj_to_ssab_files(&get_test_base_path().join(p).as_path(), &config);
            match r {
                Ok(_) => { assert!(true) },
                Err(e) => assert!(false, "{:?}", e),
            }
        });
    }

}
