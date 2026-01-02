
fn convert_animedata_labels(labels: &Vec<sspj::ssae::SsLabel>) -> Vec<ssab::ssab::LabelT> {
    labels.iter().map(|label| {
        let mut l = ssab::ssab::LabelT::default();
        l.name = label.name.to_string();
        l.time = label.time;
        l
    }).collect::<Vec<ssab::ssab::LabelT>>()
}

fn convert_animedatat(anime_data: &sspj::ssae::SsAnimation, _setup_animation: &sspj::ssae::SsAnimation, sspj: &sspj::sspj::SsProject) -> ssab::ssab::AnimationDataT {
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

pub fn convert_animation_data_list(anime_pack: &sspj::ssae::SsAnimePack, sspj: &sspj::sspj::SsProject) -> Vec<ssab::ssab::AnimationDataT> {
    let Some(setup_animation) = anime_pack.anime_list.iter().find(|anime_data| anime_data.is_setup) else {
        // not found setup animation
        return vec![];
    };

    anime_pack.anime_list.iter().filter_map(|anime_data| {
        if anime_data == setup_animation {
            return None;
        }
        Some(convert_animedatat(anime_data, setup_animation, sspj))
    }).collect::<Vec<ssab::ssab::AnimationDataT>>()
}

pub fn convert_part_list(anime_pack: &sspj::ssae::SsAnimePack) -> Vec<ssab::ssab::PartDataT> {
    anime_pack.model.part_list.iter().map(|part| {
        let mut part_data_t = ssab::ssab::PartDataT::default();
        part_data_t.name = Some(part.name.to_string());
        part_data_t.parent_index = part.parent_index as i16;
        part_data_t.bounds_type = match part.bounds_type {
            sspj::sstypes::SsBoundsType::None => ssab::ssab::BoundsType::None,
            sspj::sstypes::SsBoundsType::Quad => ssab::ssab::BoundsType::Quad,
            sspj::sstypes::SsBoundsType::Aabb => ssab::ssab::BoundsType::Aabb,
            sspj::sstypes::SsBoundsType::Circle => ssab::ssab::BoundsType::Circle,
            sspj::sstypes::SsBoundsType::CircleSmin => ssab::ssab::BoundsType::Circle,
            sspj::sstypes::SsBoundsType::CircleSmax => ssab::ssab::BoundsType::Circle,
        };
        part_data_t.blend_type = match part.alpha_blend_type {
            sspj::sstypes::SsBlendType::Mix => ssab::ssab::BlendType::Mix,
            sspj::sstypes::SsBlendType::Mul => ssab::ssab::BlendType::Mul,
            sspj::sstypes::SsBlendType::Add => ssab::ssab::BlendType::Add,
            sspj::sstypes::SsBlendType::Sub => ssab::ssab::BlendType::Sub,
            sspj::sstypes::SsBlendType::Mulalpha => ssab::ssab::BlendType::Mulalpha,
            sspj::sstypes::SsBlendType::Screen => ssab::ssab::BlendType::Screen,
            sspj::sstypes::SsBlendType::Exclusion => ssab::ssab::BlendType::Exclusion,
            sspj::sstypes::SsBlendType::Invert => ssab::ssab::BlendType::Invert,
        };
        part_data_t.part_type = match part.part_type {
            sspj::sstypes::SsPartType::Null => ssab::ssab::PartTypeT::PartTypeNull(Box::new(ssab::ssab::PartTypeNoneT::default())),
            sspj::sstypes::SsPartType::Normal => ssab::ssab::PartTypeT::PartTypeNormal(Box::new(ssab::ssab::PartTypeNoneT::default())),
            sspj::sstypes::SsPartType::Shape => {
                let mut part_type = ssab::ssab::PartTypeShapeT::default();
                part_type.shape_type = match part.shape_type {
                    sspj::sstypes::SsShapeType::Unknown => ssab::ssab::ShapeType::Unknown,
                    sspj::sstypes::SsShapeType::Triangle => ssab::ssab::ShapeType::Triangle,
                    sspj::sstypes::SsShapeType::Rectangle => ssab::ssab::ShapeType::Rectangle,
                    sspj::sstypes::SsShapeType::Arrow => ssab::ssab::ShapeType::Arrow,
                    sspj::sstypes::SsShapeType::Star => ssab::ssab::ShapeType::Star,
                };
                part_type.shape_mask = part.shape_mask;

                ssab::ssab::PartTypeT::PartTypeShape(Box::new(part_type))
            },
            sspj::sstypes::SsPartType::Text => {
                let mut part_type = ssab::ssab::PartTypeTextT::default();
                part_type.text = Some(part.text.to_string());
                part_type.text_bitmap = part.text_bitmap;
                part_type.text_family = if part.text_family.is_empty() { None } else { Some(part.text_family.to_string()) };
                part_type.text_char_map = if part.text_char_map.is_empty() { None } else { Some(part.text_char_map.to_string()) };
                part_type.text_size = part.text_size;
                part_type.text_space = part.text_space;
                part_type.text_smooth = part.text_smooth;
                part_type.text_mask = part.text_mask;
                part_type.text_width = part.text_width;
                part_type.text_height = part.text_height;
                part_type.e_anchor = match part.e_anchor {
                    0  => ssab::ssab::TextAnchor::LT,
                    1  => ssab::ssab::TextAnchor::LC,
                    2  => ssab::ssab::TextAnchor::LB,
                    16 => ssab::ssab::TextAnchor::CT,
                    17 => ssab::ssab::TextAnchor::CC,
                    18 => ssab::ssab::TextAnchor::CB,
                    32 => ssab::ssab::TextAnchor::RT,
                    33 => ssab::ssab::TextAnchor::RC,
                    34 => ssab::ssab::TextAnchor::RB,
                    255 =>ssab::ssab::TextAnchor::NO,
                    _ => ssab::ssab::TextAnchor::LT,
                };

                ssab::ssab::PartTypeT::PartTypeText(Box::new(part_type))
            },
            sspj::sstypes::SsPartType::Nines => {
                let mut part_type = ssab::ssab::PartTypeNinesT::default();
                part_type.nines_margin_l = part.nines_margin_l;
                part_type.nines_margin_r = part.nines_margin_r;
                part_type.nines_margin_t = part.nines_margin_t;
                part_type.nines_margin_b = part.nines_margin_b;
                part_type.nines_fill_mode = part.nines_fill_mode;
                part_type.nines_mask = part.nines_mask;

                ssab::ssab::PartTypeT::PartTypeNines(Box::new(part_type))
            },
            sspj::sstypes::SsPartType::Instance => {
                let mut part_type = ssab::ssab::PartTypeInstanceT::default();
                part_type.ref_anime = Some(part.ref_anime.to_string());
                ssab::ssab::PartTypeT::PartTypeInstance(Box::new(part_type))
            },
            sspj::sstypes::SsPartType::Armature => ssab::ssab::PartTypeT::PartTypeArmature(Box::new(ssab::ssab::PartTypeNoneT::default())),
            sspj::sstypes::SsPartType::Effect => {
                let mut part_type = ssab::ssab::PartTypeEffectT::default();
                part_type.ref_effect_name = Some(part.ref_effect_name.to_string());
                ssab::ssab::PartTypeT::PartTypeEffect(Box::new(part_type))
            },
            sspj::sstypes::SsPartType::Mesh => ssab::ssab::PartTypeT::PartTypeMesh(Box::new(ssab::ssab::PartTypeNoneT::default())),
            sspj::sstypes::SsPartType::Movenode => ssab::ssab::PartTypeT::PartTypeMovenode(Box::new(ssab::ssab::PartTypeNoneT::default())),
            sspj::sstypes::SsPartType::Constraint => ssab::ssab::PartTypeT::PartTypeConstraint(Box::new(ssab::ssab::PartTypeNoneT::default())),
            sspj::sstypes::SsPartType::Mask => ssab::ssab::PartTypeT::PartTypeMask(Box::new(ssab::ssab::PartTypeNoneT::default())),
            sspj::sstypes::SsPartType::Joint => ssab::ssab::PartTypeT::PartTypeJoint(Box::new(ssab::ssab::PartTypeNoneT::default())),
            sspj::sstypes::SsPartType::Bonepoint => ssab::ssab::PartTypeT::PartTypeBonepoint(Box::new(ssab::ssab::PartTypeNoneT::default())),
            sspj::sstypes::SsPartType::TransformConstraint => ssab::ssab::PartTypeT::PartTypeTransformConstraint(Box::new(ssab::ssab::PartTypeNoneT::default())),
            sspj::sstypes::SsPartType::Camera => ssab::ssab::PartTypeT::PartTypeCamera(Box::new(ssab::ssab::PartTypeNoneT::default())),
            sspj::sstypes::SsPartType::Audio => ssab::ssab::PartTypeT::PartTypeAudio(Box::new(ssab::ssab::PartTypeNoneT::default())),
        };

        part_data_t
    }).collect::<Vec<_>>()
}