use std::ffi::OsString;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn parse_args(args: &Vec<OsString>) -> Vec<PathBuf> {
    args.iter().filter_map(|arg| {
        let p = Path::new(arg);
        if !p.exists() {
            return None;
        }
        if p.extension().unwrap_or("".as_ref()).to_string_lossy() == "sspj" {
            return Some(vec![p.to_path_buf()]);
        }
        if p.is_dir() {
            let a = WalkDir::new(arg).into_iter().filter_map(|e| e.ok()).filter_map(|entry| {
                if entry.path().is_dir() || entry.path().extension().unwrap().to_string_lossy() != "sspj" {
                    return None;
                }
                return Some(entry.path().to_path_buf());
            }).collect::<Vec<PathBuf>>();
            return Some(a);
        }
        None
    }).flatten().collect::<Vec<PathBuf>>()
}