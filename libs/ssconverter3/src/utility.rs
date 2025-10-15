use std::ffi::OsString;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn parse_args(args: &[OsString]) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = args
        .iter()
        .filter_map(|arg| {
            let path = Path::new(arg);
            path.exists().then(|| collect_sspj_files(path))
        })
        .flatten()
        .collect();
    paths.sort_unstable();
    paths.dedup();
    paths
}

fn collect_sspj_files(path: &Path) -> Vec<PathBuf> {
    if is_sspj_file(path) {
        return vec![path.canonicalize().unwrap().to_path_buf()];
    }

    if path.is_dir() {
        return WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| is_sspj_file(entry.path()))
            .map(|entry| entry.path().canonicalize().unwrap().to_path_buf())
            .collect();
    }

    Vec::new()
}

fn is_sspj_file(path: &Path) -> bool {
    path.is_file()
        && path.extension()
        .and_then(|ext| ext.to_str())
        .map_or(false, |ext| ext == "sspj")
}