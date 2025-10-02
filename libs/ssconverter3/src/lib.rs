use std::path::{Path, PathBuf};
use crate::sspj::ssloader::SsLoader;
use crate::sspj::sspj::SsProject;
use crate::sspj::ssloader::SspjError;

pub mod sspj;

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
