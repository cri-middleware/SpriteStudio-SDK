//! `ssconverter-cli` — command-line front end for
//! [`ssconverter`](https://github.com/cri-middleware/SpriteStudio-SDK/tree/HEAD/libs/ssconverter).
//!
//! Converts SpriteStudio projects (`.sspj`) into the runtime binaries (`.ssab`,
//! `.ssqb`) in either single or batch mode, the latter walking directories for projects
//! and converting them in parallel. It is a thin shell around the library: argument
//! parsing, progress logging, and exit codes only — the conversion itself, and therefore
//! the bytes produced, are identical to what the GUI and the in-engine converters emit.
//!
//! Run `ssconverter-cli --help` for the full option list, or see the
//! [usage guide](https://github.com/cri-middleware/SpriteStudio-SDK/blob/HEAD/cli/README.md).

use clap::{CommandFactory, Parser};
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::LazyLock;
use std::thread::JoinHandle;

static VERSION: LazyLock<String> = LazyLock::new(|| format!("{}+{} (ssab schema v{}, ssqb schema v{})", env!("CARGO_PKG_VERSION"), env!("VERGEN_GIT_SHA"), ssconverter::ssab_schema_version(), ssconverter::ssqb_schema_version()));

#[derive(Parser, Debug)]
#[command(
    name = "ssconverter-cli",
    version = VERSION.as_str(),
    about = "Converts SpriteStudio projects (.sspj) to optimized binary files (.ssab)",
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

fn main() -> ExitCode {
    let args = Cli::parse();
    let mut cmd = Cli::command();

    // The walk below skips a path that does not exist without a word.
    let missing: Vec<&PathBuf> = args.files.iter().filter(|path| !path.exists()).collect();
    for path in &missing {
        eprintln!("Error: {} does not exist.", path.display());
    }

    let sspj_paths = ssconverter::utility::parse_args(&args.files);
    if sspj_paths.is_empty() {
        if missing.len() < args.files.len() {
            eprintln!("Error: No SpriteStudio project files found at the specified paths.");
            eprintln!("{}", cmd.render_usage());
        }
        return ExitCode::FAILURE;
    }

    println!("{} {}", cmd.get_name(), *VERSION);

    let mut config = ssconverter::ConverterConfig::default();
    if sspj_paths.len() == 1 {
        config.convert_config = ssconverter::ConvertConfig { embedded: args.embedded, ..Default::default() };
        config.output = args.out_dir;
    } else {
        config.convert_config = ssconverter::ConvertConfig { embedded: args.embedded, ..Default::default() };
        config.output = None;
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
    // Every conversion is joined, so a failure neither cuts the others short nor
    // hides theirs.
    let mut failed = 0;
    for (path, handler) in sspj_paths.iter().zip(handlers) {
        let error = match handler.join() {
            Ok(Ok(())) => continue,
            Ok(Err(e)) => e.to_string(),
            Err(_) => "the conversion panicked".to_string(),
        };
        eprintln!("Error: {}: {}", path.display(), error);
        failed += 1;
    }

    if failed > 0 {
        eprintln!("{} of {} conversions failed.", failed, sspj_paths.len());
    }
    if failed > 0 || !missing.is_empty() {
        return ExitCode::FAILURE;
    }
    println!("All conversions completed successfully.");
    ExitCode::SUCCESS
}
