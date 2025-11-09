use sspj::ssce::{SsCell, SsCellMap};
use sspj::sstypes::{SsPoint2, SsTexFilterMode, SsTexWrapMode};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Weak;

pub struct SsCellValue {
    pub cell: Option<Weak<RefCell<SsCellMap>>>,
    pub uvs: [SsPoint2; 5],
    pub wrap_mode: SsTexWrapMode,
    pub filter_mode: SsTexFilterMode,
}
impl SsCellValue {
    pub fn default() -> Self {
        Self {
            cell: None,
            uvs: [
                SsPoint2::default(),
                SsPoint2::default(),
                SsPoint2::default(),
                SsPoint2::default(),
                SsPoint2::default(),
            ],
            wrap_mode: SsTexWrapMode::Clamp,
            filter_mode: SsTexFilterMode::Nearest,
        }
    }
}

pub struct SsCelMapLinker {
    pub cell_map: Option<Weak<RefCell<SsCellMap>>>,

    cell_dic: HashMap<String, SsCell>,
}
impl SsCelMapLinker {
    pub fn find_cell(&self, name: &str) -> Option<&SsCell> {
        self.cell_dic.get(name)
    }
}

pub struct SsCellMapList {}
