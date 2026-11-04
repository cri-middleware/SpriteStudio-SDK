//! `--version` is what a bug report quotes, so it names the schema versions of the
//! `.ssab` and `.ssqb` this build writes along with the release.

use std::process::Command;

#[test]
fn version_names_both_schema_versions() {
    let output = Command::new(env!("CARGO_BIN_EXE_ssconverter-cli")).arg("--version").output().expect("run ssconverter-cli");
    assert!(output.status.success(), "--version failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [format!("ssab schema v{}", ssconverter::ssab_schema_version()), format!("ssqb schema v{}", ssconverter::ssqb_schema_version())] {
        assert!(stdout.contains(&expected), "--version does not name {expected}:\n{stdout}");
    }
}
