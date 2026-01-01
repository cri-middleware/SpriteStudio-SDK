fn convert_animedata_labels(labels: &Vec<sspj::ssae::SsLabel>) -> Vec<ssab::ssab::LabelT> {
    labels.iter().map(|label| {
        let mut l = ssab::ssab::LabelT::default();
        l.name = label.name.to_string();
        l.time = label.time;
        l
    }).collect::<Vec<ssab::ssab::LabelT>>()
}

fn convert_animedatat(anime_data: &sspj::ssae::SsAnimation, _setup_animation: &sspj::ssae::SsAnimation) -> ssab::ssab::AnimationDataT {
    let mut anime_data_t = ssab::ssab::AnimationDataT::default();

    anime_data_t.name = anime_data.name.to_string();
    // anime_data_t.parts_initial_attributes // TODO: impl
    // anime_data_t.parts_animation_data: Option<alloc::vec::Vec<PartAnimationDataT>> // TODO: impl
    // anime_data_t.parts_mesh_data: Option<alloc::vec::Vec<PartMeshDataT>> // TODO: impl
    // anime_data_t.events: Option<alloc::vec::Vec<EventsPerFrameT>> // TODO: impl
    anime_data_t.labels = if anime_data.labels.is_empty() { None } else { Some(convert_animedata_labels(&anime_data.labels)) };
    anime_data_t.start_frame = anime_data.settings.start_frame as i16;
    anime_data_t.end_frame = anime_data.settings.end_frame as i16;
    anime_data_t.total_frame = anime_data.settings.frame_count as i16;
    anime_data_t.fps = anime_data.settings.fps as i16;
    let mut canvas = ssab::ssab::RectT::default();
    canvas.x1 = anime_data.settings.pivot.x;
    canvas.y1 = anime_data.settings.pivot.y;
    canvas.x2 = anime_data.settings.canvas_size.x;
    canvas.y2 = anime_data.settings.canvas_size.y;
    anime_data_t.canvas = Some(Box::new(canvas));

    anime_data_t
}

pub fn convert_animation_data_list(_sspj: &sspj::sspj::SsProject, anime_pack: &sspj::ssae::SsAnimePack, ) -> Vec<ssab::ssab::AnimationDataT> {
    let Some(setup_animation) = anime_pack.anime_list.iter().find(|anime_data| anime_data.is_setup) else {
        // not found setup animation
        return vec![];
    };

    anime_pack.anime_list.iter().filter_map(|anime_data| {
        if anime_data == setup_animation {
            return None;
        }
        Some(convert_animedatat(anime_data, setup_animation))
    }).collect::<Vec<ssab::ssab::AnimationDataT>>()
}
