use std::path::Path;
use crate::sspj::ssloader::SsLoader;
use crate::sspj::sspj::SsProject;

pub mod sspj;

pub fn sspj(path: &Path) -> Result<SsProject, quick_xml::DeError> {
    sspj::sspj::SsProject::load_from_file(path)
}
