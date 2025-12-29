use crate::runtime;
use flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Vector};

fn get_parts<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, p: &flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::PartData>>) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<crate::runtime::initialize::ss::runtime::PartData<'a>>>> {
    let parts = p.iter().map(|part| {
        let mut p_args = runtime::initialize::ss::runtime::PartDataArgs::default();
        p_args.name = part.name().map(|s| fb.create_string(s));
        p_args.parent_index = part.parent_index();
        p_args.bounds_type = match part.bounds_type() {
            ssab::ssab::BoundsType::None => runtime::initialize::ss::runtime::BoundsType::None,
            ssab::ssab::BoundsType::Quad => runtime::initialize::ss::runtime::BoundsType::Quad,
            ssab::ssab::BoundsType::Aabb => runtime::initialize::ss::runtime::BoundsType::Aabb,
            ssab::ssab::BoundsType::Circle => runtime::initialize::ss::runtime::BoundsType::Circle,
            ssab::ssab::BoundsType::CircleSmin => runtime::initialize::ss::runtime::BoundsType::CircleSmin,
            ssab::ssab::BoundsType::CircleSmax => runtime::initialize::ss::runtime::BoundsType::CircleSmax,
            _ => runtime::initialize::ss::runtime::BoundsType::None,
        };
        p_args.blend_type = match part.blend_type() {
            ssab::ssab::BlendType::Mix => runtime::initialize::ss::runtime::BlendType::Mix,
            ssab::ssab::BlendType::Mul => runtime::initialize::ss::runtime::BlendType::Mul,
            ssab::ssab::BlendType::Add => runtime::initialize::ss::runtime::BlendType::Add,
            ssab::ssab::BlendType::Sub => runtime::initialize::ss::runtime::BlendType::Sub,
            ssab::ssab::BlendType::Mulalpha => runtime::initialize::ss::runtime::BlendType::Mulalpha,
            ssab::ssab::BlendType::Screen => runtime::initialize::ss::runtime::BlendType::Screen,
            ssab::ssab::BlendType::Exclusion => runtime::initialize::ss::runtime::BlendType::Exclusion,
            ssab::ssab::BlendType::Invert => runtime::initialize::ss::runtime::BlendType::Invert,
            _ => runtime::initialize::ss::runtime::BlendType::Mix,
        };
        p_args.part_type_type = match part.part_type_type() {
            ssab::ssab::PartType::NONE => runtime::initialize::ss::runtime::PartType::NONE,
            ssab::ssab::PartType::PartTypeNull => runtime::initialize::ss::runtime::PartType::PartTypeNull,
            ssab::ssab::PartType::PartTypeNormal => runtime::initialize::ss::runtime::PartType::PartTypeNormal,
            ssab::ssab::PartType::PartTypeShape => runtime::initialize::ss::runtime::PartType::PartTypeShape,
            ssab::ssab::PartType::PartTypeText => runtime::initialize::ss::runtime::PartType::PartTypeText,
            ssab::ssab::PartType::PartTypeNines => runtime::initialize::ss::runtime::PartType::PartTypeNines,
            ssab::ssab::PartType::PartTypeInstance => runtime::initialize::ss::runtime::PartType::PartTypeInstance,
            ssab::ssab::PartType::PartTypeArmature => runtime::initialize::ss::runtime::PartType::PartTypeArmature,
            ssab::ssab::PartType::PartTypeEffect => runtime::initialize::ss::runtime::PartType::PartTypeEffect,
            ssab::ssab::PartType::PartTypeMesh => runtime::initialize::ss::runtime::PartType::PartTypeMesh,
            ssab::ssab::PartType::PartTypeMovenode => runtime::initialize::ss::runtime::PartType::PartTypeMovenode,
            ssab::ssab::PartType::PartTypeConstraint => runtime::initialize::ss::runtime::PartType::PartTypeConstraint,
            ssab::ssab::PartType::PartTypeMask => runtime::initialize::ss::runtime::PartType::PartTypeMask,
            ssab::ssab::PartType::PartTypeJoint => runtime::initialize::ss::runtime::PartType::PartTypeJoint,
            ssab::ssab::PartType::PartTypeBonepoint => runtime::initialize::ss::runtime::PartType::PartTypeBonepoint,
            ssab::ssab::PartType::PartTypeTransformConstraint => runtime::initialize::ss::runtime::PartType::PartTypeTransformConstraint,
            ssab::ssab::PartType::PartTypeCamera => runtime::initialize::ss::runtime::PartType::PartTypeCamera,
            ssab::ssab::PartType::PartTypeAudio => runtime::initialize::ss::runtime::PartType::PartTypeAudio,
            _ => runtime::initialize::ss::runtime::PartType::NONE,
        };
        p_args.part_type = match p_args.part_type_type {
            runtime::initialize::ss::runtime::PartType::PartTypeShape => {
                let mut args = runtime::initialize::ss::runtime::PartTypeShapeArgs::default();
                let part_type_shape = part.part_type_as_part_type_shape().unwrap();
                args.shape_type = match part_type_shape.shape_type() {
                    ssab::ssab::ShapeType::Unknown => runtime::initialize::ss::runtime::ShapeType::Unknown,
                    ssab::ssab::ShapeType::Triangle => runtime::initialize::ss::runtime::ShapeType::Triangle,
                    ssab::ssab::ShapeType::Rectangle => runtime::initialize::ss::runtime::ShapeType::Rectangle,
                    ssab::ssab::ShapeType::Arrow => runtime::initialize::ss::runtime::ShapeType::Arrow,
                    ssab::ssab::ShapeType::Star => runtime::initialize::ss::runtime::ShapeType::Star,
                    _ => runtime::initialize::ss::runtime::ShapeType::Unknown,
                };
                args.shape_mask = part_type_shape.shape_mask();
                let offset = runtime::initialize::ss::runtime::PartTypeShape::create(fb, &args);
                Some(offset.as_union_value())
            }
            runtime::initialize::ss::runtime::PartType::PartTypeText => {
                let mut args = runtime::initialize::ss::runtime::PartTypeTextArgs::default();
                let part_type_text = part.part_type_as_part_type_text().unwrap();
                args.text = part_type_text.text().map(|s| fb.create_string(s));
                args.text_bitmap = part_type_text.text_bitmap();
                args.text_family = part_type_text.text_family().map(|s| fb.create_string(s));
                args.text_char_map = part_type_text.text_char_map().map(|s| fb.create_string(s));
                args.text_size = part_type_text.text_size();
                args.text_space = part_type_text.text_space();
                args.text_smooth = part_type_text.text_smooth();
                args.text_mask = part_type_text.text_mask();
                args.text_width = part_type_text.text_width();
                args.text_height = part_type_text.text_height();
                args.e_anchor = match part_type_text.e_anchor() {
                    ssab::ssab::TextAnchor::LT => runtime::initialize::ss::runtime::TextAnchor::LT,
                    ssab::ssab::TextAnchor::LC => runtime::initialize::ss::runtime::TextAnchor::LC,
                    ssab::ssab::TextAnchor::LB => runtime::initialize::ss::runtime::TextAnchor::LB,
                    ssab::ssab::TextAnchor::CT => runtime::initialize::ss::runtime::TextAnchor::CT,
                    ssab::ssab::TextAnchor::CC => runtime::initialize::ss::runtime::TextAnchor::CC,
                    ssab::ssab::TextAnchor::CB => runtime::initialize::ss::runtime::TextAnchor::CB,
                    ssab::ssab::TextAnchor::RT => runtime::initialize::ss::runtime::TextAnchor::RT,
                    ssab::ssab::TextAnchor::RC => runtime::initialize::ss::runtime::TextAnchor::RC,
                    ssab::ssab::TextAnchor::RB => runtime::initialize::ss::runtime::TextAnchor::RB,
                    ssab::ssab::TextAnchor::NO => runtime::initialize::ss::runtime::TextAnchor::NO,
                    _ => runtime::initialize::ss::runtime::TextAnchor::LT,
                };
                let offset = runtime::initialize::ss::runtime::PartTypeText::create(fb, &args);
                Some(offset.as_union_value())
            }
            runtime::initialize::ss::runtime::PartType::PartTypeNines => {
                let mut args = runtime::initialize::ss::runtime::PartTypeNinesArgs::default();
                let part_type_nines = part.part_type_as_part_type_nines().unwrap();
                args.nines_margin_l = part_type_nines.nines_margin_l();
                args.nines_margin_r = part_type_nines.nines_margin_r();
                args.nines_margin_t = part_type_nines.nines_margin_t();
                args.nines_margin_b = part_type_nines.nines_margin_b();
                args.nines_fill_mode = part_type_nines.nines_fill_mode();
                args.nines_mask = part_type_nines.nines_mask();
                let offset = runtime::initialize::ss::runtime::PartTypeNines::create(fb, &args);
                Some(offset.as_union_value())
            }
            runtime::initialize::ss::runtime::PartType::PartTypeInstance => {
                let mut args = runtime::initialize::ss::runtime::PartTypeInstanceArgs::default();
                let part_type_instance = part.part_type_as_part_type_instance().unwrap();
                args.ref_anime = part_type_instance.ref_anime().map(|s| fb.create_string(s));
                let offset = runtime::initialize::ss::runtime::PartTypeInstance::create(fb, &args);
                Some(offset.as_union_value())
            }
            runtime::initialize::ss::runtime::PartType::PartTypeEffect => {
                let mut args = runtime::initialize::ss::runtime::PartTypeEffectArgs::default();
                let part_type_effect = part.part_type_as_part_type_effect().unwrap();
                args.ref_effect_name = part_type_effect.ref_effect_name().map(|s| fb.create_string(s));
                let offset = runtime::initialize::ss::runtime::PartTypeEffect::create(fb, &args);
                Some(offset.as_union_value())
            }
            _ => {
                let args = runtime::initialize::ss::runtime::PartTypeNoneArgs::default();
                let offset = runtime::initialize::ss::runtime::PartTypeNone::create(fb, &args);
                Some(offset.as_union_value())
            }
        };

        runtime::initialize::ss::runtime::PartData::create(fb, &p_args)
    }).collect::<Vec<_>>();

    fb.create_vector(&parts)
}

fn get_cells<'a>(fb: &mut FlatBufferBuilder<'a>, c: &Vector<'a, ForwardsUOffset<ssab::ssab::Cell>>) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<crate::runtime::initialize::ss::runtime::Cell<'a>>>> {
    let cells = c.iter().map(|cell| {
        let mut c_args = runtime::initialize::ss::runtime::CellArgs::default();
        c_args.name = Some(fb.create_string(cell.name()));
        let rectangle = cell.rectangle().map(|r| runtime::initialize::ss::runtime::Rect::new(r.x1(), r.y1(), r.x2(), r.y2()));
        c_args.rectangle = rectangle.as_ref();
        let pivot = cell.pivot().map(|p| runtime::initialize::ss::runtime::Vec2::new(p.v1(), p.v2()));
        c_args.pivot = pivot.as_ref();
        c_args.rotated = cell.rotated();
        c_args.table_coordinate = cell.table_coordinate().map(|t| {
            let v = t.iter().map(|s| runtime::initialize::ss::runtime::Vec2::new(s.v1(), s.v2())).collect::<Vec<_>>();
            fb.create_vector(&v)
        });
        c_args.table_index_vertex = cell.table_index_vertex().map(|t| {
            let v = t.iter().map(|s| s).collect::<Vec<_>>();
            fb.create_vector(&v)
        });
        runtime::initialize::ss::runtime::Cell::create(fb, &c_args)
    }).collect::<Vec<_>>();

    fb.create_vector(&cells)
}


fn get_cellmaps<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, c: &flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::CellMap>>) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<crate::runtime::initialize::ss::runtime::CellMap<'a>>>> {
    let cellmaps = c.iter().map(|cellmap| {
        let mut c_args = runtime::initialize::ss::runtime::CellMapArgs::default();
        c_args.name = Some(fb.create_string(cellmap.name()));
        c_args.image_path = cellmap.image_path().map(|s| fb.create_string(s));
        let size_original = cellmap.size_original().map(|s| runtime::initialize::ss::runtime::Vec2::new(s.v1(), s.v2()));
        c_args.size_original = size_original.as_ref();
        c_args.wrap_mode = match cellmap.wrap_mode() {
            ssab::ssab::TexWrapMode::Clamp => runtime::initialize::ss::runtime::TexWrapMode::Clamp,
            ssab::ssab::TexWrapMode::Repeat => runtime::initialize::ss::runtime::TexWrapMode::Repeat,
            ssab::ssab::TexWrapMode::Mirror => runtime::initialize::ss::runtime::TexWrapMode::Mirror,
            _ => runtime::initialize::ss::runtime::TexWrapMode::Clamp,
        };
        c_args.filter_mode = match cellmap.filter_mode() {
            ssab::ssab::TexFilterMode::Nearest => runtime::initialize::ss::runtime::TexFilterMode::Nearest,
            ssab::ssab::TexFilterMode::Linear => runtime::initialize::ss::runtime::TexFilterMode::Linear,
            _ => runtime::initialize::ss::runtime::TexFilterMode::Nearest,
        };
        c_args.cells = cellmap.cells().map(|c| get_cells(fb, &c) );

        runtime::initialize::ss::runtime::CellMap::create(fb, &c_args)
    }).collect::<Vec<_>>();

    fb.create_vector(&cellmaps)
}

fn get_effect_node_behavior<'a>(fb: &mut FlatBufferBuilder<'a>, b: &Vector<'a, ForwardsUOffset<ssab::ssab::EffectNodeBehaviorWrapper>>) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<crate::runtime::initialize::ss::runtime::EffectNodeBehaviorWrapper<'a>>>> {
    let behaviors = b.iter().map(|behavior| {
        let mut b_args = runtime::initialize::ss::runtime::EffectNodeBehaviorWrapperArgs::default();
        b_args.value_type = match behavior.value_type() {
            ssab::ssab::EffectNodeBehavior::EffectParticleElementBasic => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementBasic,
            ssab::ssab::EffectNodeBehavior::EffectParticleElementRndSeedChange => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementRndSeedChange,
            ssab::ssab::EffectNodeBehavior::EffectParticleElementDelay => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementDelay,
            ssab::ssab::EffectNodeBehavior::EffectParticleElementGravity => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementGravity,
            ssab::ssab::EffectNodeBehavior::EffectParticleElementPosition => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementPosition,
            ssab::ssab::EffectNodeBehavior::EffectParticleElementRotation => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementRotation,
            ssab::ssab::EffectNodeBehavior::EffectParticleElementRotationTrans => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementRotationTrans,
            ssab::ssab::EffectNodeBehavior::EffectParticleElementTransSpeed => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementTransSpeed,
            ssab::ssab::EffectNodeBehavior::EffectParticleElementTangentialAcceleration => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementTangentialAcceleration,
            ssab::ssab::EffectNodeBehavior::EffectParticleElementInitColor => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementInitColor,
            ssab::ssab::EffectNodeBehavior::EffectParticleElementTransColor => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementTransColor,
            ssab::ssab::EffectNodeBehavior::EffectParticleElementAlphaFade => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementAlphaFade,
            ssab::ssab::EffectNodeBehavior::EffectParticleElementSize => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementSize,
            ssab::ssab::EffectNodeBehavior::EffectParticleElementTransSize => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementTransSize,
            ssab::ssab::EffectNodeBehavior::EffectParticlePointGravity => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticlePointGravity,
            ssab::ssab::EffectNodeBehavior::EffectParticleTurnToDirectionEnabled => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleTurnToDirectionEnabled,
            ssab::ssab::EffectNodeBehavior::EffectParticleInfiniteEmitEnabled => runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleInfiniteEmitEnabled,
            _ => runtime::initialize::ss::runtime::EffectNodeBehavior::NONE,
        };
        b_args.value = match b_args.value_type {
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementBasic => {
                let mut e_args = runtime::initialize::ss::runtime::EffectParticleElementBasicArgs::default();
                let value = behavior.value_as_effect_particle_element_basic().unwrap();
                e_args.priority = value.priority();
                e_args.maximum_particle = value.maximum_particle();
                e_args.attime_create = value.attime_create();
                e_args.interval = value.interval();
                e_args.lifetime = value.lifetime();
                e_args.speed_min_value = value.speed_min_value();
                e_args.speed_max_value = value.speed_max_value();
                e_args.lifespan_min_value = value.lifespan_min_value();
                e_args.lifespan_max_value = value.lifespan_max_value();
                e_args.angle = value.angle();
                e_args.angle_variance = value.angle_variance();
                let e = runtime::initialize::ss::runtime::EffectParticleElementBasic::create(fb, &e_args);
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementRndSeedChange => {
                let mut e_args = runtime::initialize::ss::runtime::IntValueEntryArgs::default();
                let value = behavior.value_as_effect_particle_element_rnd_seed_change().unwrap();
                e_args.value = value.value();
                let e = runtime::initialize::ss::runtime::IntValueEntry::create(fb, &e_args);
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementDelay => {
                let mut e_args = runtime::initialize::ss::runtime::IntValueEntryArgs::default();
                let value = behavior.value_as_effect_particle_element_delay().unwrap();
                e_args.value = value.value();
                let e = runtime::initialize::ss::runtime::IntValueEntry::create(fb, &e_args);
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementGravity => {
                let mut e_args = runtime::initialize::ss::runtime::Vec2ValueEntryArgs::default();
                let value = behavior.value_as_effect_particle_element_gravity().unwrap();
                let v = value.value().map(|v| runtime::initialize::ss::runtime::Vec2::new(v.v1(), v.v2()));
                e_args.value = v.as_ref();
                let e = runtime::initialize::ss::runtime::Vec2ValueEntry::create(fb, &e_args);
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementPosition => {
                let mut e_args = runtime::initialize::ss::runtime::RectValueEntryArgs::default();
                let value = behavior.value_as_effect_particle_element_position().unwrap();
                let v = value.value().map(|r| runtime::initialize::ss::runtime::Rect::new(r.x1(), r.y1(), r.x2(), r.y2()));
                e_args.value = v.as_ref();
                let e = runtime::initialize::ss::runtime::RectValueEntry::create(fb, &e_args);
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementRotation => {
                let mut e_args = runtime::initialize::ss::runtime::EffectParticleElementRotationArgs::default();
                let value = behavior.value_as_effect_particle_element_rotation().unwrap();
                let rotation_value = value.rotation_value().map(|v| runtime::initialize::ss::runtime::Vec2::new(v.v1(), v.v2()));
                e_args.rotation_value = rotation_value.as_ref();
                let rotation_add_value = value.rotation_add_value().map(|v| runtime::initialize::ss::runtime::Vec2::new(v.v1(), v.v2()));
                e_args.rotation_add_value = rotation_add_value.as_ref();
                let e = runtime::initialize::ss::runtime::EffectParticleElementRotation::create(fb, &e_args);
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementRotationTrans => {
                let mut e_args = runtime::initialize::ss::runtime::EffectParticleElementRotationTransArgs::default();
                let value = behavior.value_as_effect_particle_element_rotation_trans().unwrap();
                e_args.rotation_factor = value.rotation_factor();
                e_args.end_life_time_per = value.end_life_time_per();
                let e = runtime::initialize::ss::runtime::EffectParticleElementRotationTrans::create(fb, &e_args);
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementTransSpeed => {
                let mut e_args = runtime::initialize::ss::runtime::Vec2ValueEntryArgs::default();
                let value = behavior.value_as_effect_particle_element_trans_speed().unwrap();
                let v = value.value().map(|v| runtime::initialize::ss::runtime::Vec2::new(v.v1(), v.v2()));
                e_args.value = v.as_ref();
                let e = runtime::initialize::ss::runtime::Vec2ValueEntry::create(fb, &e_args);
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementTangentialAcceleration => {
                let mut e_args = runtime::initialize::ss::runtime::Vec2ValueEntryArgs::default();
                let value = behavior.value_as_effect_particle_element_tangential_acceleration().unwrap();
                let v = value.value().map(|v| runtime::initialize::ss::runtime::Vec2::new(v.v1(), v.v2()));
                e_args.value = v.as_ref();
                let e = runtime::initialize::ss::runtime::Vec2ValueEntry::create(fb, &e_args);
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementInitColor => {
                let mut e_args = runtime::initialize::ss::runtime::EffectParticleElementInitColorArgs::default();
                let value = behavior.value_as_effect_particle_element_init_color().unwrap();
                e_args.color_min_value = value.color_min_value();
                e_args.color_max_value = value.color_max_value();
                let e = runtime::initialize::ss::runtime::EffectParticleElementInitColor::create(fb, &e_args);
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementTransColor => {
                let mut e_args = runtime::initialize::ss::runtime::EffectParticleElementTransColorArgs::default();
                let value = behavior.value_as_effect_particle_element_trans_color().unwrap();
                e_args.color_min_value = value.color_min_value();
                e_args.color_max_value = value.color_max_value();
                let e = runtime::initialize::ss::runtime::EffectParticleElementTransColor::create(fb, &e_args);
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementAlphaFade => {
                let mut e_args = runtime::initialize::ss::runtime::Vec2ValueEntryArgs::default();
                let value = behavior.value_as_effect_particle_element_alpha_fade().unwrap();
                let v = value.value().map(|v| runtime::initialize::ss::runtime::Vec2::new(v.v1(), v.v2()));
                e_args.value = v.as_ref();
                let e = runtime::initialize::ss::runtime::Vec2ValueEntry::create(fb, &e_args);
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementSize => {
                let mut e_args = runtime::initialize::ss::runtime::RectAndVec2ValueTableArgs::default();
                let value = behavior.value_as_effect_particle_element_size().unwrap();
                let size = value.size().map(|r| runtime::initialize::ss::runtime::Rect::new(r.x1(), r.y1(), r.x2(), r.y2()));
                e_args.size = size.as_ref();
                let scale_factor_value = value.scale_factor_value().map(|v| runtime::initialize::ss::runtime::Vec2::new(v.v1(), v.v2()));
                e_args.scale_factor_value = scale_factor_value.as_ref();
                let e = runtime::initialize::ss::runtime::RectAndVec2ValueTable::create(fb, &e_args);
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleElementTransSize => {
                let mut e_args = runtime::initialize::ss::runtime::RectAndVec2ValueTableArgs::default();
                let value = behavior.value_as_effect_particle_element_trans_size().unwrap();
                let size = value.size().map(|r| runtime::initialize::ss::runtime::Rect::new(r.x1(), r.y1(), r.x2(), r.y2()));
                e_args.size = size.as_ref();
                let scale_factor_value = value.scale_factor_value().map(|v| runtime::initialize::ss::runtime::Vec2::new(v.v1(), v.v2()));
                e_args.scale_factor_value = scale_factor_value.as_ref();
                let e = runtime::initialize::ss::runtime::RectAndVec2ValueTable::create(fb, &e_args);
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticlePointGravity => {
                let mut e_args = runtime::initialize::ss::runtime::EffectParticlePointGravityArgs::default();
                let value = behavior.value_as_effect_particle_point_gravity().unwrap();
                let position = value.position().map(|p| runtime::initialize::ss::runtime::Vec2::new(p.v1(), p.v2()));
                e_args.position = position.as_ref();
                e_args.power = value.power();
                let e = runtime::initialize::ss::runtime::EffectParticlePointGravity::create(fb, &e_args);
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleTurnToDirectionEnabled => {
                let mut e_args = runtime::initialize::ss::runtime::FloatValueEntryArgs::default();
                let value = behavior.value_as_effect_particle_turn_to_direction_enabled().unwrap();
                e_args.value = value.value();
                let e = runtime::initialize::ss::runtime::FloatValueEntry::create(fb, &e_args).as_union_value();
                Some(e.as_union_value())
            },
            runtime::initialize::ss::runtime::EffectNodeBehavior::EffectParticleInfiniteEmitEnabled => {
                let mut e_args = runtime::initialize::ss::runtime::IntValueEntryArgs::default();
                let value = behavior.value_as_effect_particle_infinite_emit_enabled().unwrap();
                e_args.value = value.value();
                let e = runtime::initialize::ss::runtime::IntValueEntry::create(fb, & e_args);
                Some(e.as_union_value())
            },
            _ => { None },
        };

        runtime::initialize::ss::runtime::EffectNodeBehaviorWrapper::create(fb, &b_args)
    }).collect::<Vec<_>>();
    fb.create_vector(&behaviors)
}

fn get_effect_node<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, c: &flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<ssab::ssab::EffectNode>>) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<crate::runtime::initialize::ss::runtime::EffectNode<'a>>>> {
    let effect_nodes = c.iter().map(|effect_node| {
        let mut e_args = runtime::initialize::ss::runtime::EffectNodeArgs::default();
        e_args.index = effect_node.index();
        e_args.parent_index = effect_node.parent_index();
        e_args.effect_node_type_type = match effect_node.effect_node_type() {
            ssab::ssab::EffectNodeType::Root => runtime::initialize::ss::runtime::EffectNodeType::Root,
            ssab::ssab::EffectNodeType::Emitter => runtime::initialize::ss::runtime::EffectNodeType::Emitter,
            ssab::ssab::EffectNodeType::Particle => runtime::initialize::ss::runtime::EffectNodeType::Particle,
            _ => runtime::initialize::ss::runtime::EffectNodeType::Root,
        };
        e_args.cell_index = effect_node.cell_index();
        e_args.blend_type = match effect_node.blend_type() {
            ssab::ssab::RenderBlendType::Mix => runtime::initialize::ss::runtime::RenderBlendType::Mix,
            ssab::ssab::RenderBlendType::Add => runtime::initialize::ss::runtime::RenderBlendType::Add,
            _ => runtime::initialize::ss::runtime::RenderBlendType::Mix,
        };
        e_args.behavior = effect_node.behavior().map(|b| get_effect_node_behavior(fb, &b));

        runtime::initialize::ss::runtime::EffectNode::create(fb, &e_args)
    }).collect::<Vec<_>>();
    fb.create_vector(&effect_nodes)
}

fn get_effects<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, e: &Vector<'a, ForwardsUOffset<ssab::ssab::Effect>>) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<crate::runtime::initialize::ss::runtime::Effect<'a>>>> {
    let effects = e.iter().map(|effect| {
        let mut e_args = runtime::initialize::ss::runtime::EffectArgs::default();
        e_args.name = Some(fb.create_string(effect.name()));
        e_args.fps = effect.fps();
        e_args.is_lock_rand_seed = effect.is_lock_rand_seed();
        e_args.lock_rand_seed = effect.lock_rand_seed();
        let layout_scale = effect.layout_scale().map(|s| runtime::initialize::ss::runtime::Vec2::new(s.v1(), s.v2()));
        e_args.layout_scale = layout_scale.as_ref();
        e_args.effect_node = effect.effect_node().map(|n| get_effect_node(fb, &n));

        runtime::initialize::ss::runtime::Effect::create(fb, &e_args)
    }).collect::<Vec<_>>();
    fb.create_vector(&effects)
}

fn get_sound_lists<'a>(fb: &mut FlatBufferBuilder<'a>, s: &Vector<ForwardsUOffset<ssab::ssab::SoundList>>) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<crate::runtime::initialize::ss::runtime::SoundList<'a>>>> {
    let sound_lists = s.iter().map(|sound_list| {
        let mut s_args = runtime::initialize::ss::runtime::SoundListArgs::default();
        s_args.name = Some(fb.create_string(sound_list.name()));
        s_args.table_data = sound_list.table_data().map(|table_data| {
            let v = table_data.iter().map(|sound_file| {
                let mut sf_args = runtime::initialize::ss::runtime::SoundFileArgs::default();
                sf_args.name = Some(fb.create_string(sound_file.name()));
                sf_args.time_total = sound_file.time_total();

                runtime::initialize::ss::runtime::SoundFile::create(fb, &sf_args)
            }).collect::<Vec<_>>();
            fb.create_vector(&v)
        });

        runtime::initialize::ss::runtime::SoundList::create(fb, &s_args)
    }).collect::<Vec<_>>();
    fb.create_vector(&sound_lists)
}

fn get_font_bitmaps<'a>(fb: &mut FlatBufferBuilder<'a>, f: &Vector<ForwardsUOffset<ssab::ssab::FontBitmap>>) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<crate::runtime::initialize::ss::runtime::FontBitmap<'a>>>> {
    let font_bitmaps = f.iter().map(|font_bitmap| {
        let mut f_args = runtime::initialize::ss::runtime::FontBitmapArgs::default();
        f_args.name = Some(fb.create_string(font_bitmap.name()));
        f_args.name_face = font_bitmap.name_face().map(|name_face| fb.create_string(name_face));
        f_args.size = font_bitmap.size();
        f_args.padding_l = font_bitmap.padding_l();
        f_args.padding_r = font_bitmap.padding_r();
        f_args.padding_u = font_bitmap.padding_u();
        f_args.padding_d = font_bitmap.padding_d();
        f_args.base = font_bitmap.base();
        f_args.height_line = font_bitmap.height_line();
        let scale = font_bitmap.scale().map(|scale| runtime::initialize::ss::runtime::Vec2::new(scale.v1(), scale.v2()));
        f_args.scale = scale.as_ref();
        let spacing = font_bitmap.spacing().map(|spacing| runtime::initialize::ss::runtime::Vec2::new(spacing.v1(), spacing.v2()));
        f_args.spacing = spacing.as_ref();
        let size_texture_original = font_bitmap.size_texture_original().map(|size_texture_original| runtime::initialize::ss::runtime::Vec2::new(size_texture_original.v1(), size_texture_original.v2()));
        f_args.size_texture_original = size_texture_original.as_ref();
        f_args.name_texture = font_bitmap.name_texture().map(|name_textures| {
            let v = name_textures.iter().map(|name_texture| fb.create_string(name_texture)).collect::<Vec<_>>();
            fb.create_vector(&v)
        });
        f_args.table_data = font_bitmap.table_data().map(|table_data| {
            let v = table_data.iter().map(|font_glyph_bitmap| {
                let mut fgb_args = runtime::initialize::ss::runtime::FontGlyphBitmapArgs::default();
                fgb_args.id = font_glyph_bitmap.id();
                let atlas = font_glyph_bitmap.atlas().map(|atlas| runtime::initialize::ss::runtime::Rect::new(atlas.x1(), atlas.y1(), atlas.x2(), atlas.y2()));
                fgb_args.atlas = atlas.as_ref();
                let offset = font_glyph_bitmap.offset().map(|offset| runtime::initialize::ss::runtime::Vec2::new(offset.v1(), offset.v2()));
                fgb_args.offset = offset.as_ref();
                fgb_args.width_advance = font_glyph_bitmap.width_advance();
                fgb_args.text = font_glyph_bitmap.text().map(|text| fb.create_string(text));
                fgb_args.page = font_glyph_bitmap.page();
                fgb_args.channel = font_glyph_bitmap.channel();

                runtime::initialize::ss::runtime::FontGlyphBitmap::create(fb, &fgb_args)
            }).collect::<Vec<_>>();
            fb.create_vector(&v)
        });

        runtime::initialize::ss::runtime::FontBitmap::create(fb, &f_args)
    }).collect::<Vec<_>>();
    fb.create_vector(&font_bitmaps)
}

pub fn get_initialize_data<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, ssab: &ssab::ssab::SsAnimeBinary<'a>) -> Option<Vec<u8>> {
    fb.reset();

    let mut args = runtime::initialize::ss::runtime::InitDataArgs::default();
    args.parts = ssab.parts().map(|p| get_parts(fb, &p));
    args.cellmaps = ssab.cellmaps().map(|c| get_cellmaps(fb, &c));
    args.effects = ssab.effects().map(|e| get_effects(fb, &e));
    args.sound_lists = ssab.sound_lists().map(|s| get_sound_lists(fb, &s));
    args.font_bitmaps = ssab.font_bitmaps().map(|f| get_font_bitmaps(fb, &f));
    args.external_instances = ssab.external_instances().map(|external_instances| {
        let v = external_instances.iter().map(|external_instance| {
            fb.create_string(external_instance)
        }).collect::<Vec<_>>();
        fb.create_vector(&v)
    });

    let offset = runtime::initialize::ss::runtime::InitData::create(fb, &args);
    fb.finish(offset, None);
    let buf = fb.finished_data();

    Some(buf.to_vec())
}
