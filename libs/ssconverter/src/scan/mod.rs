use crate::utility::collect_sspj_files_cancellable;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub struct SspjScan {
    should_stop: Arc<AtomicBool>,
    visited: Arc<AtomicUsize>,
    found: Arc<AtomicUsize>,
    budget_exceeded: Arc<AtomicBool>,
    results: Arc<Mutex<Vec<PathBuf>>>,
    handle: Option<JoinHandle<()>>,
}

impl SspjScan {
    pub fn start(dir: PathBuf, max_depth: usize, max_entries: usize, max_millis: u64) -> Self {
        let should_stop = Arc::new(AtomicBool::new(false));
        let visited = Arc::new(AtomicUsize::new(0));
        let found = Arc::new(AtomicUsize::new(0));
        let budget_exceeded = Arc::new(AtomicBool::new(false));
        let results = Arc::new(Mutex::new(Vec::new()));

        let handle = {
            let should_stop = should_stop.clone();
            let visited = visited.clone();
            let found = found.clone();
            let budget_exceeded = budget_exceeded.clone();
            let results = results.clone();
            std::thread::spawn(move || {
                let collected = collect_sspj_files_cancellable(&dir, &should_stop, max_depth, max_entries, max_millis, &visited, &found, &budget_exceeded);
                if let Ok(mut guard) = results.lock() {
                    *guard = collected;
                }
            })
        };

        Self { should_stop, visited, found, budget_exceeded, results, handle: Some(handle) }
    }

    pub fn abort(&self) {
        self.should_stop.store(true, Ordering::Relaxed);
    }

    pub fn visited(&self) -> usize {
        self.visited.load(Ordering::Relaxed)
    }

    pub fn found(&self) -> usize {
        self.found.load(Ordering::Relaxed)
    }

    pub fn budget_exceeded(&self) -> bool {
        self.budget_exceeded.load(Ordering::Relaxed)
    }

    pub fn is_finished(&self) -> bool {
        self.handle.as_ref().map(|h| h.is_finished()).unwrap_or(true)
    }

    pub fn results(&self) -> Vec<PathBuf> {
        self.results.lock().map(|g| g.clone()).unwrap_or_default()
    }
}
