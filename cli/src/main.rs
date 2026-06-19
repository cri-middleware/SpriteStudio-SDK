use clap::{CommandFactory, Parser};
use std::ffi::OsString;
use std::path::PathBuf;
use std::thread::JoinHandle;

const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "+", env!("VERGEN_GIT_SHA"));
#[derive(Parser, Debug)]
#[command(
    name = "ssconverter-cli",
    version = VERSION,
    about = "Converts SpriteStudio 7 projects (.sspj) to optimized binary files (.ssab)",
    override_usage = "
  single mode: convert a single sspj project
  usage:
    $ ssconverter-cli [OPTIONS] <sspj_file>

  batch mode: convert multiple sspj projects
  usage:
    $ ssconverter-cli [OPTIONS] <sspj_file> <sspj_file> ...
    $ ssconverter-cli [OPTIONS] <directory> <directory> ...
"
)]
struct Cli {
    #[arg(
        required = true,
        value_name = "PATHS", 
        help = "Paths to .sspj files or directories containing projects",
        num_args = 1..
    )]
    files: Vec<PathBuf>,

    #[arg(short, long, help = "embed assets into a ssab file")]
    embedded: bool,

    #[arg(short, long, value_name = "directory", help = "output directory path (available for single mode only)")]
    out_dir: Option<OsString>,
}

/*
fn version_string() -> String {
    let sha = env!("VERGEN_GIT_SHA");
    let short_sha = if sha.len() >= 7 { &sha[..7] } else { sha };
    format!("{}+{}", env!("CARGO_PKG_VERSION"), short_sha)
}
 */

fn main() -> Result<(), ssconverter::ConverterError> {
    let args = Cli::parse();
    let mut cmd = Cli::command();

    let sspj_paths = ssconverter::utility::parse_args(&args.files);
    if sspj_paths.is_empty() {
        eprintln!("Error: No SpriteStudio project files found at the specified paths.");
        eprintln!("{}", cmd.render_usage());
        std::process::exit(1);
    }

    println!("{} {} (ssab schema v{})", cmd.get_name(), VERSION, ssconverter::ssab_schema_version());

    let mut config = ssconverter::ConverterConfig::default();
    if sspj_paths.len() == 1 {
        config.convert_config = ssconverter::ConvertConfig { embedded: args.embedded, ..Default::default() };
        config.output = args.out_dir;
    } else {
        // batch mode
        config.convert_config = ssconverter::ConvertConfig { embedded: args.embedded, ..Default::default() };
        config.output = None; // batch mode does not support out_dir
    }

    let mut handlers: Vec<JoinHandle<Result<(), ssconverter::ConverterError>>> = vec![];
    sspj_paths.iter().for_each(|path_buf| {
        println!("Converting {:?}...", path_buf.display());
        handlers.push({
            let p = path_buf.clone();
            let c = config.clone();
            std::thread::spawn(move || ssconverter::convert_sspj_to_ssab_files(p.as_path(), &c, &|msg| println!("{}", msg)))
        })
    });
    for h in handlers {
        match h.join() {
            Ok(result) => result?,
            Err(_) => {
                eprintln!("Thread panic occurred");
                panic!("A thread has panicked during execution");
            }
        }
    }

    println!("All conversions completed successfully.");
    Ok(())
}

#[cfg(test)]
mod tests {

    use rayon::prelude::*;
    use std::path::{Path, PathBuf};

    static TARGET_SSPJ_FILES: [&str; 4] = ["tests/ParticleEffect/ParticleEffect.sspj", "tests/allAttributeV7/allAttributeV7.sspj", "tests/allPartsV7/allPartsV7.sspj", "tests/overall/overall.sspj"];

    fn get_test_base_path() -> PathBuf {
        std::env::current_dir().expect("Failed to get the current directory").parent().expect("Failed to get the base directory").to_path_buf()
    }

    #[test]
    fn test_convert_ssab() {
        TARGET_SSPJ_FILES.par_iter().for_each(|&f| {
            let p = Path::new(f);
            let config = ssconverter::ConverterConfig::default();
            let r = ssconverter::convert_sspj_to_ssab_files(get_test_base_path().join(p).as_path(), &config, &|msg: &str| println!("{}", msg));
            match r {
                Ok(_) => {}
                Err(e) => panic!("{:?}", e),
            }
        });
    }
}
