use std::path::Path;
use crate::sspj::ssloader::SsLoader;
use crate::sspj::sspj::SsProject;
use crate::sspj::ssloader::SspjError;

pub mod sspj;

pub fn sspj(path: &Path) -> Result<SsProject, SspjError> {
    sspj::sspj::SsProject::load_from_file(path)
}
