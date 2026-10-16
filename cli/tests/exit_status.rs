//! What a script driving the CLI can rely on: the run fails when any path or
//! project does, and stderr names every one of them.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(tag: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(tag);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create the scratch directory");
    dir
}

/// Whether the run succeeded, and its stderr.
fn run<S: AsRef<OsStr>>(args: &[S]) -> (bool, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_ssconverter-cli")).args(args).output().expect("run ssconverter-cli");
    (output.status.success(), String::from_utf8_lossy(&output.stderr).into_owned())
}

/// A path that does not exist used to be skipped, and the run exited 0.
#[test]
fn a_path_that_does_not_exist_fails_the_run() {
    let dir = scratch("missing");
    let ringo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/Ringo/Ringo.sspj");
    let missing = dir.join("missing.sspj");
    let out = dir.join("out");
    let (ok, stderr) = run(&[ringo.as_os_str(), missing.as_os_str(), OsStr::new("-o"), out.as_os_str()]);
    assert!(!ok, "the run succeeded");
    assert!(stderr.contains(&*missing.to_string_lossy()), "stderr does not name the missing path:\n{stderr}");
    assert!(out.join("Ringo.ssab").exists(), "the project that does exist was not converted");
}

/// The first failure used to end the run, so the others went unreported.
#[test]
fn every_project_that_fails_is_reported() {
    let dir = scratch("broken");
    for name in ["a.sspj", "b.sspj"] {
        std::fs::write(dir.join(name), "not xml").expect("write a broken project");
    }
    let (ok, stderr) = run(&[&dir]);
    assert!(!ok, "the run succeeded");
    for name in ["a.sspj", "b.sspj"] {
        assert!(stderr.contains(name), "stderr does not name {name}:\n{stderr}");
    }
}
