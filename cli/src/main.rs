use ssconverter3;
use std::env;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::thread::JoinHandle;
use clap::Parser;


#[derive(Parser, Debug)]
#[command(
    name = "ssconverter3-cli",
    override_usage = "
single mode: convert a single sspj project or a single sspj project zip file usage:
  $ ssconverter3-cli [OPTIONS] <sspj_file>
  $ ssconverter3-cli [OPTIONS] <sspj_zip_file>

batch mode: convert multiple sspj projects usage:
  $ ssconverter3-cli [OPTIONS] <sspj_file> <sspj_file> ...
  $ ssconverter3-cli [OPTIONS] <directory> <directory> ...
"
)]
struct Cli {
    #[arg(required = true, value_names = &["sspj_files", "sspj_zip_files", "directories"], num_args = 1..)]
    files: Vec<PathBuf>,

    #[arg(short, long, help = "embed assets into a ssab file")]
    embedded: bool,

    #[arg(short, long, value_name = "directory", help = "output directory path (available for single mode only)")]
    out_dir: Option<OsString>,

}

fn main() -> Result<(), ssconverter3::ConverterError> {
    let args = Cli::parse();

    let sspj_paths = ssconverter3::utility::parse_args(&args.files);
    let mut config = ssconverter3::ConverterConfig::default();
    if sspj_paths.len() == 1 {
        // single mode (sspj file or sspj zip file)
        config.convert_config = ssconverter3::ConvertConfig {
            embedded: args.embedded,
            ..Default::default()
        };
        config.out_dir = args.out_dir;
    } else {
        // batch mode
        config.convert_config = ssconverter3::ConvertConfig {
            embedded: args.embedded,
            ..Default::default()
        };
        config.out_dir = None; // batch mode does not support out_dir
    }

    let mut handlers: Vec<JoinHandle<Result<(), ssconverter3::ConverterError>>> = vec![];
    sspj_paths.iter().for_each(|path_buf| {
        println!("convert {:?}", path_buf.display());
        handlers.push({
            let p = path_buf.clone();
            let c = config.clone();
            std::thread::spawn(move || {
                ssconverter3::convert_sspj_to_ssab_files(p.as_path(), &c)
            })
        })
    });
    for h in handlers {
        match h.join() {
            Ok(result) => {
                if let Err(e) = result {
                    return Err(e);
                }
            }
            Err(_) => {
                eprintln!("Thread panic occurred");
                panic!("A thread has panicked during execution");
            }
        }
    }

    Ok(())
}

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
            .parent()
            .expect("Failed to get the base directory")
            .to_path_buf()
    }

    #[test]
    fn test_convert_ssab() {
        TARGET_SSPJ_FILES.par_iter().for_each(|&f| {
            let p = Path::new(f);
            let config = ssconverter3::ConverterConfig::default();
            let r = ssconverter3::convert_sspj_to_ssab_files(
                &get_test_base_path().join(p).as_path(),
                &config,
            );
            match r {
                Ok(_) => {
                    assert!(true)
                }
                Err(e) => assert!(false, "{:?}", e),
            }
        });
    }
}
