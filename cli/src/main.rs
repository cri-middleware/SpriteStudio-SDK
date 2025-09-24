use std::env;
use std::ffi::OsString;
use std::path::Path;
use ssconverter3;

fn main() {
    let args: Vec<OsString> = env::args_os().collect();
    if args.len() <= 1 {
        println!("usage: ssconverter3 <sspj path>");
        return;
    }
    let path = Path::new(args[1].as_os_str());
    let Ok(sspj) = ssconverter3::sspj(&path) else {
        println!("not found: {}", path.as_os_str().to_string_lossy());
        return;
    };
    println!("{:?}", sspj);
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use super::*;
    use rayon::prelude::*;

    fn get_test_base_path() -> PathBuf {
        std::env::current_dir()
            .expect("Failed to get current directory")
            .parent()
            .expect("Failed to get base directory")
            .to_path_buf()
    }

    #[test]
    fn test_parse_sspj() {
        [
            "tests/ParticleEffect/ParticleEffect.sspj",
            "tests/allAttributeV7/allAttributeV7.sspj",
            "tests/allPartsV7/allPartsV7.sspj",
            "tests/overall/overall.sspj",
        ].par_iter().for_each(|&p| {
            let path = Path::new(p);
            let Ok(sspj) = ssconverter3::sspj(&get_test_base_path().join(p).as_path()) else {
                panic!("not found: {}", path.as_os_str().to_string_lossy());
            };
            // println!("{:?}", sspj);
        });

    }
}
