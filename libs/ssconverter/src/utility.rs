use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;
use walkdir::WalkDir;

pub fn parse_args(args: &[PathBuf]) -> Vec<PathBuf> {
    args.iter().filter(|&arg| arg.exists()).flat_map(|arg| collect_sspj_files(arg)).collect::<HashSet<_>>().into_iter().collect::<Vec<PathBuf>>()
}

/// Walk `root` for `.sspj` files within three budgets — `max_depth` levels below
/// it, `max_entries` entries looked at, and `max_millis` of wall clock (`0` for
/// none) — and raise `budget_exceeded` when any of them cuts the walk short, so a
/// caller can tell a partial listing from a whole one.
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
    let mut walker = WalkDir::new(root).max_depth(max_depth).same_file_system(true).into_iter().filter_entry(|entry| is_walked(entry.file_name()));

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
        if entry.depth() == max_depth && entry.file_type().is_dir() && has_walked_entries(entry.path()) {
            // `WalkDir::max_depth` stops descending without a word, so a project
            // below the limit would go missing from a listing that reports itself
            // whole. Look one level in rather than walk it: walking it opens and
            // counts every subdirectory there.
            budget_exceeded.store(true, Ordering::Relaxed);
        }
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
            .filter_entry(|entry| is_walked(entry.file_name()))
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

fn is_walked(name: &OsStr) -> bool {
    !name.to_string_lossy().starts_with(".git")
}

/// Whether a walk let one level deeper would find anything inside `dir`.
fn has_walked_entries(dir: &Path) -> bool {
    std::fs::read_dir(dir).is_ok_and(|mut entries| entries.any(|entry| entry.is_ok_and(|entry| is_walked(&entry.file_name()))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("ssconverter-scan-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("Failed to create the scratch directory");
        dir
    }

    /// `<root>/a/b/project.sspj`: one project, three levels down.
    fn nested_project(tag: &str) -> PathBuf {
        let root = scratch_dir(tag);
        let dir = root.join("a").join("b");
        std::fs::create_dir_all(&dir).expect("Failed to create the nested directories");
        std::fs::write(dir.join("project.sspj"), b"").expect("Failed to write the project");
        root
    }

    /// A scan held only by `max_depth`: how many projects it found, and whether
    /// it reported a budget cutting it short.
    fn scan(root: &Path, max_depth: usize) -> (usize, bool) {
        let budget_exceeded = AtomicBool::new(false);
        let found = collect_sspj_files_cancellable(root, &AtomicBool::new(false), max_depth, usize::MAX, 0, &AtomicUsize::new(0), &AtomicUsize::new(0), &budget_exceeded);
        (found.len(), budget_exceeded.load(Ordering::Relaxed))
    }

    // A project below the depth budget is dropped, so `budget_exceeded` has to say
    // so, or an editor that offers to rescan on a cut never offers.
    #[test]
    fn a_walk_cut_by_the_depth_budget_says_so() {
        let root = nested_project("depth-cut");
        assert_eq!(scan(&root, 2), (0, true));
        let _ = std::fs::remove_dir_all(&root);
    }

    // A link to a directory below the root is not followed, and that is not a cut.
    // The root itself is followed when it is one.
    #[cfg(unix)]
    #[test]
    fn a_symlink_to_a_directory_is_followed_only_as_the_root() {
        let root = scratch_dir("symlink");
        let tree = root.join("tree");
        let elsewhere = nested_project("symlink-target");
        std::fs::create_dir_all(&tree).expect("Failed to create the tree");
        std::os::unix::fs::symlink(&elsewhere, tree.join("link")).expect("Failed to create the symlink");
        std::os::unix::fs::symlink(&elsewhere, root.join("root_link")).expect("Failed to create the symlink");
        assert_eq!(scan(&tree, 8), (0, false));
        assert_eq!(scan(&root.join("root_link"), 8), (1, false));
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&elsewhere);
    }

    // Telling a cut must not cost the walk another level: a wide directory at the
    // limit is looked into, not opened subdirectory by subdirectory and counted
    // against `max_entries`.
    #[test]
    fn a_walk_cut_by_the_depth_budget_reads_no_further() {
        let root = scratch_dir("depth-wide");
        for i in 0..8 {
            std::fs::create_dir_all(root.join("a").join(format!("d{i}"))).expect("Failed to create the wide directory");
        }
        let visited = AtomicUsize::new(0);
        let budget_exceeded = AtomicBool::new(false);
        collect_sspj_files_cancellable(&root, &AtomicBool::new(false), 1, usize::MAX, 0, &visited, &AtomicUsize::new(0), &budget_exceeded);
        // `<root>` and `a`.
        assert_eq!((visited.load(Ordering::Relaxed), budget_exceeded.load(Ordering::Relaxed)), (2, true));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_walk_that_reaches_the_bottom_reports_no_cut() {
        let root = nested_project("depth-whole");
        assert_eq!(scan(&root, 3), (1, false));
        assert_eq!(scan(&root, usize::MAX), (1, false));

        // An empty directory at the limit hides nothing, so it is no cut either.
        std::fs::create_dir_all(root.join("a").join("b").join("empty")).expect("Failed to create the empty directory");
        assert_eq!(scan(&root, 3), (1, false));
        let _ = std::fs::remove_dir_all(&root);
    }
}
