use std::collections::HashSet;
use sspj::ssae::{SsAnimePack, SsPart};
use sspj::ssce::SsCellMap;
use sspj::ssee::SsEffectFile;
use sspj::sspj::SsProject;
use sspj::sstypes::SsPartType;
use crate::model_t::ssabt;
use crate::model_t::ssabt::convert_sound_list;

pub fn convert_ss_anime_binary_t(anime_pack: &SsAnimePack, sspj: &SsProject, ) -> ssab::ssab::SsAnimeBinaryT {
    ssabt::convert_ss_anime_binary_t(anime_pack, sspj)
}