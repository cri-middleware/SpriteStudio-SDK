use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;
use walkdir::WalkDir;

pub fn parse_args(args: &[PathBuf]) -> Vec<PathBuf> {
    args.iter().filter(|&arg| arg.exists()).flat_map(|arg| collect_sspj_files(arg)).collect::<HashSet<_>>().into_iter().collect::<Vec<PathBuf>>()
}

#[allow(clippy::too_many_arguments)]
pub fn collect_sspj_files_cancellable(root: &Path, should_stop: &AtomicBool, max_depth: usize, max_entries: usize, max_millis: u64, visited: &AtomicUsize, found: &AtomicUsize, budget_exceeded: &AtomicBool) -> Vec<PathBuf> {
    let mut results: Vec<PathBuf> = Vec::new();

    if is_sspj_file(root) {
        found.fetch_add(1, Ordering::Relaxed);
        results.push(root.to_path_buf());
        return results;
    }
    if !root.is_dir() {
        return results;
    }

    let start = Instant::now();
    let mut walker = WalkDir::new(root).max_depth(max_depth).same_file_system(true).into_iter().filter_entry(|entry| {
        let name = entry.file_name();
        !name.to_string_lossy().starts_with(".git")
    });

    loop {
        if should_stop.load(Ordering::Relaxed) {
            break;
        }
        if visited.load(Ordering::Relaxed) >= max_entries || (max_millis > 0 && start.elapsed().as_millis() as u64 >= max_millis) {
            budget_exceeded.store(true, Ordering::Relaxed);
            break;
        }

        let Some(next) = walker.next() else {
            break;
        };
        visited.fetch_add(1, Ordering::Relaxed);

        let Ok(entry) = next else {
            continue;
        };
        if is_sspj_file(entry.path()) {
            found.fetch_add(1, Ordering::Relaxed);
            results.push(entry.path().to_path_buf());
        }
    }

    results
}

fn collect_sspj_files(path: &Path) -> Vec<PathBuf> {
    if is_sspj_file(path) {
        return vec![path.canonicalize().unwrap_or_else(|_| path.to_path_buf())];
    }

    if path.is_dir() {
        return WalkDir::new(path)
            .into_iter()
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
