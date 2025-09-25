use std::path::Path;
use crate::sspj::ssloader::SsLoader;
use crate::sspj::sspj::SsProject;
use ssfb3::*;

pub mod sspj;

pub fn sspj(path: &Path) -> Result<SsProject, quick_xml::DeError> {
    sspj::sspj::SsProject::load_from_file(path)
}

fn aa(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    let Ok(sspj) = sspj::sspj::SsProject::load_from_file(path) else {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "sspj load error"));
    };

    let mut project_data = ssfb3::ssfb2::ss::ssfb_2::ProjectDataT::default();
    project_data.name;

    Ok(Vec::new())
}