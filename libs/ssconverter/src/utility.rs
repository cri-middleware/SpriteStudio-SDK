use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn parse_args(args: &[PathBuf]) -> Vec<PathBuf> {
    args.iter()
        .filter(|&arg| arg.exists()).flat_map(|arg| collect_sspj_files(arg))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<PathBuf>>()
}

fn collect_sspj_files(path: &Path) -> Vec<PathBuf> {
    if is_sspj_file(path) || is_zip_file(path) {
        return vec![path.canonicalize().unwrap_or_else(|_| path.to_path_buf())];
    }

    if path.is_dir() {
        return WalkDir::new(path).into_iter()
            .filter_entry(|entry| {
                let name = entry.file_name();
                !name.to_string_lossy().starts_with(".git")
            })
            .filter_map(Result::ok)
            .filter(|entry| is_sspj_file(entry.path()))
            .map(|entry| entry.path().canonicalize().unwrap_or_else(|_| entry.path().to_path_buf()))
            .collect();
    }

    vec![]
}

fn is_sspj_file(path: &Path) -> bool {
    path.is_file() && (path.extension().and_then(|ext| ext.to_str()) == Some("sspj"))
}

fn is_zip_file(path: &Path) -> bool {
    path.is_file() && (path.extension().and_then(|ext| ext.to_str()) == Some("zip"))
}
