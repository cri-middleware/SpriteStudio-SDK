use crate::cellmap::SsCellMapList;
use crate::part_state::SsPartState;
use sspj::ssae::{SsAnimation, SsModel, SsPart, SsPartAnime};
use sspj::sspj::SsProject;
use sspj::ssqe::SsSequence;
use std::collections::HashMap;
use std::rc::Weak;
use std::sync::Arc;

#[derive(Default)]
pub struct SsAnimeDecoder<'a> {
    project: Option<&'a SsProject>,
    // FontTextureManager* fontTexManager;
    part_anime: Vec<HashMap<SsPart, SsPartAnime>>,
    setup_part_anime: Vec<HashMap<SsPart, SsPartAnime>>,

    part_anime_dic: HashMap<&'a str, &'a SsPartAnime>,
    setup_part_anime_dic: HashMap<&'a str, &'a SsPartAnime>,

    // cur_cell_map_manager: SsCellMapList,
    part_state: Vec<SsPartState>,

    /*
    std::list<SsPartState*>			sortList;			///ソート状態
    std::list<SsPartState*>			partStatesMask_;	///マスクテンポラリ
    std::vector<SsPartState*>		maskIndexList;
     */
    seed_offset: i32,
    now_plat_time: f32,
    now_plat_time_old: f32,
    frame_delta: f32,
    cur_anime_start_frame: i32,
    cur_anime_end_frame: i32,
    cur_anime_total_frame: i32,
    cur_anime_fps: i32,
    cur_animation: Option<&'a mut SsAnimation>,
    instance_parts_hide: bool,
    mask_func_flag: bool,
    mask_parent_setting: bool,
    cur_sequence: Option<&'a SsSequence>,

    state_num: isize,

    // std::unique_ptr<SsMeshAnimator>	meshAnimator;
    my_model: Option<&'a SsModel>,
}
impl<'a> SsAnimeDecoder<'a> {
    pub fn set_animation(
        &mut self,
        model: &'a SsModel,
        anime: &'a SsAnimation,
        cellmap: &SsCellMapList,
        sspj: &'a mut SsProject,
    ) {
        self.part_anime_dic.clear();
        self.setup_part_anime_dic.clear();

        self.my_model = Some(model);

        anime.part_animes.iter().for_each(|anime| {
            self.part_anime_dic.insert(&anime.part_name, &anime);
        });
        // setup data
        if let Some(setup) =
            sspj.find_animation(&model.ref_setup_anime_pack, &model.ref_setup_animation)
        {
            setup.part_animes.iter().for_each(|anime| {
                self.setup_part_anime_dic.insert(&anime.part_name, &anime);
            })
        }
    }
    // void	setSequence( SsSequence* sequence , SsProject* sspj=0 );

    pub fn set_play_frame(&mut self, time: f32) {
        self.now_plat_time = time;
    }
    pub fn get_anime_start_frame(&mut self) -> i32 {
        self.cur_anime_start_frame
    }
    pub fn get_anime_end_frame(&mut self) -> i32 {
        self.cur_anime_end_frame
    }
    pub fn get_anime_total_frame(&mut self) -> i32 {
        self.cur_anime_total_frame
    }
    pub fn get_anime_fps(&mut self) -> i32 {
        self.cur_anime_fps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_anime_decoder() {
        let decoder = SsAnimeDecoder::default();
    }
}
