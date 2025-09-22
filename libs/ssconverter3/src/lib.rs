use std::path::Path;
use crate::sspj::ssce::SsCellMap;
use crate::sspj::ssloader::SsLoader;

 pub mod sspj;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

// test func
pub fn ssce(path: &Path) -> Result<SsCellMap, quick_xml::DeError> {
    sspj::ssce::SsCellMap::load_from_file(path)
}