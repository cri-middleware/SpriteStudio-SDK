use std::path::{Path, PathBuf};
use sspj::ssloader::SsLoader;
use sspj::sspj::SsProject;
use sspj::ssloader::SspjError;

pub struct SspjLoader {
    path_buf: PathBuf,
}
impl SspjLoader {
    pub fn new(path: &Path) -> Self {
        SspjLoader {
            path_buf: path.to_path_buf()
        }
    }
    pub fn load_ss_project(&self) -> Result<SsProject, SspjError> {
        SsProject::load_from_file(self.path_buf.as_path())
    }
}
