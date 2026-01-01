use ssab::ssab::{AnimationDataT, CellMapT, CellT, EffectNodeBehaviorWrapperT, EffectNodeT, EffectNodeType, EffectT, EventsPerFrameT, FontBitmapT, LabelT, PartAnimationDataT, PartAttributesT, PartMeshDataT, RectT, RenderBlendType, Vec2T};
use sspj::ssae::{SsAnimation, SsAnimePack, SsLabel, SsPart};
use sspj::ssce::{SsCell, SsCellMap};
use sspj::sscharmap::SsCharMap;
use sspj::ssee::{ParticleElementAlphaFade, ParticleElementBasic, ParticleElementDelay, ParticleElementGravity, ParticleElementInitColor, ParticleElementPosition, ParticleElementRndSeedChange, ParticleElementRotation, ParticleElementRotationTrans, ParticleElementSize, ParticleElementTangentialAcceleration, ParticleElementTransColor, ParticleElementTransSize, ParticleElementTransSpeed, ParticleInfiniteEmitEnabled, ParticlePointGravity, ParticleTurnToDirectionEnabled, SsEffectFile, SsEffectFunctionType, SsEffectNode};
use sspj::sspj::SsProject;
use sspj::ssse::{SsAudioPack, SsSoundFileList};
use sspj::sstypes::{SsPartType, SsPoint2, SsPoint3, SsU8Color};
use std::collections::{HashMap, HashSet};
use crate::model_t;

pub fn convert_vec2t(point2: &SsPoint2) -> ssab::ssab::Vec2T {
    let mut vec = ssab::ssab::Vec2T::default();
    vec.v1 = point2.x;
    vec.v2 = point2.y;
    vec
}
pub fn convert_vec2t_list(point2_list: &[SsPoint2]) -> Vec<ssab::ssab::Vec2T> {
    point2_list.iter().map(|point2| convert_vec2t(point2)).collect::<Vec<ssab::ssab::Vec2T>>()
}

pub fn convert_flat_list_from_vec3t_list(point3_list: &[SsPoint3]) -> Vec<u16> {
    point3_list.iter().flat_map(|point3| [point3.x as u16, point3.y as u16, point3.z as u16]).collect::<Vec<u16>>()
}

pub fn convert_rectt(x1: f32, y1: f32, x2: f32, y2: f32) -> ssab::ssab::RectT {
    let mut rect = ssab::ssab::RectT::default();
    rect.x1 = x1;
    rect.y1 = y1;
    rect.x2 = x2;
    rect.y2 = y2;

    rect
}

fn convert_color(c: &sspj::sstypes::SsU8Color) -> u32 {
    ((c.a as u32) << 24) | ((c.r as u32) << 16) | ((c.g as u32)<< 8) | (c.b as u32)
}

fn convert_animedata_labels(labels: &Vec<SsLabel>) -> Vec<LabelT> {
    labels.iter().map(|label| {
        let mut l = LabelT::default();
        l.name = label.name.to_string();
        l.time = label.time;
        l
    }).collect::<Vec<LabelT>>()
}

fn convert_animedatat(anime_data: &SsAnimation, _setup_animation: &SsAnimation) -> AnimationDataT {
    let mut anime_data_t = AnimationDataT::default();

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

fn convert_animation_data_list(_sspj: &SsProject, anime_pack: &SsAnimePack, ) -> Vec<AnimationDataT> {
    let Some(setup_animation) = anime_pack.anime_list.iter().find(|anime_data| anime_data.is_setup) else {
        // not found setup animation
        return vec![];
    };

    anime_pack.anime_list.iter().filter_map(|anime_data| {
        if anime_data == setup_animation {
            return None;
        }
        Some(convert_animedatat(anime_data, setup_animation))
    }).collect::<Vec<AnimationDataT>>()
}

pub fn convert_cellt(sscell: &SsCell) -> CellT {
    let mut cell_t = CellT::default();
    cell_t.name = sscell.name.to_string();
    cell_t.rectangle = Some(Box::new(convert_rectt(sscell.pos.x, sscell.pos.y, sscell.size.x, sscell.size.y, )));
    cell_t.pivot = Some(Box::new(convert_vec2t(&sscell.pivot)));
    cell_t.rotated = sscell.rotated;
    cell_t.table_coordinate = if sscell.mesh_point_list.is_empty() { None } else { Some(convert_vec2t_list(&sscell.mesh_point_list)) };
    cell_t.table_index_vertex = if sscell.mesh_tri_list.is_empty() { None } else { Some(convert_flat_list_from_vec3t_list(&sscell.mesh_tri_list)) };

    cell_t
}
pub fn convert_cellt_list(cells: &[SsCell]) -> Vec<CellT> {
    cells.iter().map(|cell| convert_cellt(cell)).collect::<Vec<CellT>>()
}

pub fn convert_cellmapt(cellmap: &SsCellMap) -> CellMapT {
    let mut cell_map_t = CellMapT::default();
    cell_map_t.name = cellmap.name.to_string();
    cell_map_t.image_path = if cellmap.image_path.is_empty() { None } else { Some(cellmap.image_path.to_string()) };
    cell_map_t.size_original = Some(Box::new(convert_vec2t(&cellmap.pixel_size)));
    cell_map_t.wrap_mode = match cellmap.wrap_mode {
        sspj::sstypes::SsTexWrapMode::Clamp => ssab::ssab::TexWrapMode::Clamp,
        sspj::sstypes::SsTexWrapMode::Repeat => ssab::ssab::TexWrapMode::Repeat,
        sspj::sstypes::SsTexWrapMode::Mirror => ssab::ssab::TexWrapMode::Mirror,
    };
    cell_map_t.filter_mode = match cellmap.filter_mode {
        sspj::sstypes::SsTexFilterMode::Nearest => ssab::ssab::TexFilterMode::Nearest,
        sspj::sstypes::SsTexFilterMode::Linear => ssab::ssab::TexFilterMode::Linear,
    };
    cell_map_t.cells = if cellmap.cells.is_empty() { None } else { Some(convert_cellt_list(&cellmap.cells)) };

    cell_map_t
}
pub fn convert_cellmapt_list(_sspj: &SsProject, cellmap_names: &[SsCellMap]) -> Vec<CellMapT> {
    cellmap_names.iter().map(|cellmap| convert_cellmapt(cellmap)).collect::<Vec<CellMapT>>()
}

fn convert_effect_node_list(nodes: &Vec<SsEffectNode>) -> Vec<EffectNodeT> {
    nodes.iter().map(|node| {
        let mut effect_node_t = EffectNodeT::default();
        effect_node_t.index = node.array_index as i16;
        effect_node_t.parent_index = node.parent_index as i16;
        effect_node_t.effect_node_type = match &node.node_type {
            sspj::ssee::SsEffectNodeType::Root => ssab::ssab::EffectNodeType::Root,
            sspj::ssee::SsEffectNodeType::Emitter => ssab::ssab::EffectNodeType::Emitter,
            sspj::ssee::SsEffectNodeType::Particle => ssab::ssab::EffectNodeType::Particle,
        };
        let behavior = &node.behavior;
        // effect_node_t.cell_index = ; // TODO: impl
        effect_node_t.blend_type = match &behavior.blend_type {
            sspj::sstypes::SsRenderBlendType::Mix => ssab::ssab::RenderBlendType::Mix,
            sspj::sstypes::SsRenderBlendType::Add => ssab::ssab::RenderBlendType::Add,
        };
        let behavior_list = behavior.list.iter().filter_map(|function_type| {
                match function_type {
                    SsEffectFunctionType::Base => None,
                    SsEffectFunctionType::Basic(p) => {
                        let mut behavior_t = ssab::ssab::EffectParticleElementBasicT::default();
                        behavior_t.priority = p.priority;
                        behavior_t.maximum_particle = p.maximum_particle;
                        behavior_t.attime_create = p.attime_create;
                        behavior_t.interval = p.interval;
                        behavior_t.lifetime = p.lifetime;
                        behavior_t.speed_min_value = p.speed.value;
                        behavior_t.speed_max_value = p.speed.subvalue;
                        behavior_t.lifespan_min_value = p.lifespan.value;
                        behavior_t.lifespan_max_value = p.lifespan.subvalue;
                        behavior_t.angle = p.angle;
                        behavior_t.angle_variance = p.angle_variance;

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleElementBasic(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::RndSeedChange(p) => {
                        let mut behavior_t = ssab::ssab::IntValueEntryT::default();
                        behavior_t.value = p.seed;

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleElementRndSeedChange(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::Delay(p) => {
                        let mut behavior_t = ssab::ssab::IntValueEntryT::default();
                        behavior_t.value = p.delay_time;

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleElementDelay(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::Gravity(p) => {
                        let mut behavior_t = ssab::ssab::Vec2ValueEntryT::default();
                        behavior_t.value = Some(Box::new(convert_vec2t(&p.gravity)));

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleElementGravity(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::Position(p) => {
                        let mut behavior_t = ssab::ssab::RectValueEntryT::default();
                        behavior_t.value = Some(Box::new(convert_rectt(p.offset_x.value, p.offset_y.value, p.offset_x.subvalue, p.offset_y.subvalue)));

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleElementPosition(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::Rotation(p) => {
                        let mut behavior_t = ssab::ssab::EffectParticleElementRotationT::default();
                        let mut rotation_value = ssab::ssab::Vec2T::default();
                        rotation_value.v1 = p.rotation.value;
                        rotation_value.v2 = p.rotation.subvalue;
                        behavior_t.rotation_value = Some(Box::new(rotation_value));
                        let mut rotation_add_value = ssab::ssab::Vec2T::default();
                        rotation_add_value.v1 = p.rotation_add.value;
                        rotation_add_value.v2 = p.rotation_add.subvalue;
                        behavior_t.rotation_add_value = Some(Box::new(rotation_add_value));

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleElementRotation(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::TransRotation(p) => {
                        let mut behavior_t = ssab::ssab::EffectParticleElementRotationTransT::default();
                        behavior_t.rotation_factor = p.rotation_factor;
                        behavior_t.end_life_time_per = p.end_life_time_per;

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleElementRotationTrans(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::TransSpeed(p) => {
                        let mut behavior_t = ssab::ssab::Vec2ValueEntryT::default();
                        let mut speed = ssab::ssab::Vec2T::default();
                        speed.v1 = p.speed.value;
                        speed.v2 = p.speed.subvalue;
                        behavior_t.value = Some(Box::new(speed));

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleElementTransSpeed(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::TangentialAcceleration(p) => {
                        let mut behavior_t = ssab::ssab::Vec2ValueEntryT::default();
                        let mut acceleration = ssab::ssab::Vec2T::default();
                        acceleration.v1 = p.acceleration.value;
                        acceleration.v2 = p.acceleration.subvalue;
                        behavior_t.value = Some(Box::new(acceleration));

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleElementTangentialAcceleration(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::InitColor(p) => {
                        let mut behavior_t = ssab::ssab::EffectParticleElementInitColorT::default();
                        behavior_t.color_min_value = convert_color(&p.color.value);
                        behavior_t.color_max_value = convert_color(&p.color.subvalue);

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleElementInitColor(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::TransColor(p) => {
                        let mut behavior_t = ssab::ssab::EffectParticleElementTransColorT::default();
                        behavior_t.color_min_value = convert_color(&p.color.value);
                        behavior_t.color_max_value = convert_color(&p.color.subvalue);

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleElementTransColor(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::AlphaFade(p) => {
                        let mut behavior_t = ssab::ssab::Vec2ValueEntryT::default();
                        let mut alpha = ssab::ssab::Vec2T::default();
                        alpha.v1 = p.disprange.value;
                        alpha.v2 = p.disprange.subvalue;
                        behavior_t.value = Some(Box::new(alpha));

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleElementAlphaFade(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::Size(p) => {
                        let mut behavior_t = ssab::ssab::RectAndVec2ValueTableT::default();
                        let mut size = ssab::ssab::RectT::default();
                        size.x1 = p.size_x.value;
                        size.x2 = p.size_x.subvalue;
                        size.y1 = p.size_y.value;
                        size.y2 = p.size_y.subvalue;
                        behavior_t.size = Some(Box::new(size));
                        let mut scale_factor_value = ssab::ssab::Vec2T::default();
                        scale_factor_value.v1 = p.scale_factor.value;
                        scale_factor_value.v2 = p.scale_factor.subvalue;
                        behavior_t.scale_factor_value = Some(Box::new(scale_factor_value));

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleElementSize(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::TransSize(p) => {
                        let mut behavior_t = ssab::ssab::RectAndVec2ValueTableT::default();
                        let mut size = ssab::ssab::RectT::default();
                        size.x1 = p.size_x.value;
                        size.x2 = p.size_x.subvalue;
                        size.y1 = p.size_y.value;
                        size.y2 = p.size_y.subvalue;
                        behavior_t.size = Some(Box::new(size));
                        let mut scale_factor_value = ssab::ssab::Vec2T::default();
                        scale_factor_value.v1 = p.scale_factor.value;
                        scale_factor_value.v2 = p.scale_factor.subvalue;
                        behavior_t.scale_factor_value = Some(Box::new(scale_factor_value));

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleElementTransSize(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::PointGravity(p) => {
                        let mut behavior_t = ssab::ssab::EffectParticlePointGravityT::default();
                        let mut position = ssab::ssab::Vec2T::default();
                        position.v1 = p.position.x;
                        position.v2 = p.position.y;
                        behavior_t.position = Some(Box::new(position));
                        behavior_t.power = p.power;

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticlePointGravity(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::TurnToDirectionEnabled(p) => {
                        let mut behavior_t = ssab::ssab::FloatValueEntryT::default();
                        behavior_t.value = p.rotation;

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleTurnToDirectionEnabled(Box::new(behavior_t));
                        Some(wrap)
                    },
                    SsEffectFunctionType::InfiniteEmitEnabled(p) => {
                        let mut behavior_t = ssab::ssab::IntValueEntryT::default();
                        behavior_t.value = 1;

                        let mut wrap = ssab::ssab::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::EffectNodeBehaviorT::EffectParticleInfiniteEmitEnabled(Box::new(behavior_t));
                        Some(wrap)
                    },
                    _ => { None },
                }
        }).collect::<Vec<_>>();
        effect_node_t.behavior = if behavior_list.is_empty() { None } else { Some(behavior_list) };

        effect_node_t
    }).collect::<Vec<EffectNodeT>>()
}

fn convert_effect_list(_sspj: &SsProject, effect_file_vec: &[SsEffectFile], ) -> Vec<EffectT> {
    effect_file_vec.iter().map(|effect_file| {
        let mut effect_t = EffectT::default();
        effect_t.name = effect_file.name.to_string();
        effect_t.fps = effect_file.effect_data.fps as i16;
        effect_t.is_lock_rand_seed = effect_file.effect_data.is_lock_rand_seed;
        effect_t.lock_rand_seed = effect_file.effect_data.lock_rand_seed as i16;
        let mut layout_scale = ssab::ssab::Vec2T::default();
        layout_scale.v1 = effect_file.effect_data.layout_scale_x as f32;
        layout_scale.v2 = effect_file.effect_data.layout_scale_y as f32;
        effect_t.layout_scale = Some(Box::new(layout_scale));
        effect_t.effect_node = if effect_file.effect_data.node_list.is_empty() { None } else { Some(convert_effect_node_list(&effect_file.effect_data.node_list)) };

        effect_t
    }).collect::<Vec<_>>()
}

pub fn convert_sound_file_list(_sspj: &SsProject, sssound_list: &SsSoundFileList, ) -> Vec<ssab::ssab::SoundFileT> {
    sssound_list.sounds.iter().map(|sssound| {
        let mut sound_file_t = ssab::ssab::SoundFileT::default();

        sound_file_t.name = sssound.alias.to_string();
        sound_file_t.time_total = sssound.msec;

        sound_file_t
    }).collect::<Vec<ssab::ssab::SoundFileT>>()
}

pub fn convert_sound_list(sspj: &SsProject, sound_list: &[SsAudioPack], ) -> Option<Vec<ssab::ssab::SoundListT>> {
    if sound_list.is_empty() {
        return None;
    }
    Some(
        sound_list.iter().map(|sssound_pack| {
            let mut sound_list_t = ssab::ssab::SoundListT::default();

            sound_list_t.name = sssound_pack.name.to_string();
            sound_list_t.table_data = Some(convert_sound_file_list(sspj, &sssound_pack.table));

            sound_list_t
        }).collect::<Vec<ssab::ssab::SoundListT>>(),
    )
}

pub fn convert_font_bitmaps(_sspj: &SsProject, charmap_dic: &HashMap<String, SsCharMap>, ) -> Vec<FontBitmapT> {
    charmap_dic.iter().map(|(key, value)| {
        let mut bitmap = FontBitmapT::default();

        bitmap.name = key.to_string();
        // TODO: impl
        /*
        bitmap.name_face ;
        bitmap.size;
        bitmap.padding_l;
        bitmap.padding_r;
        bitmap.padding_u;
        bitmap.padding_d;
        bitmap.base;
        bitmap.height_line;
        bitmap.scale;
        bitmap.spacing;
        bitmap.size_texture_original;
        bitmap.name_texture;
        bitmap.table_data;
         */

        bitmap
    }).collect::<Vec<_>>()
}

pub fn convert_ss_anime_binary_t(anime_pack: &SsAnimePack, sspj: &SsProject, ) -> ssab::ssab::SsAnimeBinaryT {
    let mut anime_binary = ssab::ssab::SsAnimeBinaryT::default();
    anime_binary.name = if anime_pack.name.is_empty() { None } else { Some(anime_pack.name.to_string()) };
    anime_binary.animations = if anime_pack.anime_list.is_empty() { None } else { Some(model_t::ssabt::convert_animation_data_list(&sspj, &anime_pack)) };

    let using_effect_file_names = anime_pack.model.part_list.iter().filter_map(|part| {
        if part.part_type == SsPartType::Effect {
            Some(part.ref_effect_name.to_string())
        } else {
            None
        }
    }).collect::<HashSet<String>>();
    let effect_file_list = using_effect_file_names.iter().filter_map(|using_effect_file_name| {
        sspj.effect_file_list.iter().find(|effect_file| effect_file.name == *using_effect_file_name).cloned()
    }).collect::<Vec<SsEffectFile>>();
    anime_binary.effects = if effect_file_list.is_empty() { None } else { Some(model_t::ssabt::convert_effect_list(&sspj, &effect_file_list)) };

    let effect_cellmap_names = effect_file_list.iter().flat_map(|f| &f.effect_data.node_list)
        .map(|n| &n.behavior.cell_map_name)
        .filter(|name| !name.is_empty()).cloned().collect::<Vec<String>>();

    anime_binary.sound_lists = if anime_pack.model.part_list.iter().filter(|part| part.part_type == SsPartType::Audio).collect::<Vec<&SsPart>>().is_empty() { // don't use sound
        None
    } else {
        convert_sound_list(&sspj, &sspj.sound_list)
    };

    /*
    anime_pack.model.part_list.iter().filter(|part| part.part_type == SsPartType::Text)
        .filter_map(|part| {
            // TODO: ?
            Some(part.array_index)
        });
     */
    anime_binary.font_bitmaps = if sspj.charmap_dic.is_empty() { None } else { Some(model_t::ssabt::convert_font_bitmaps(&sspj, &sspj.charmap_dic)) };

    let cellmap_names = anime_pack.cellmap_names.iter().map(|cellmap_file| cellmap_file.to_string()).chain(effect_cellmap_names).collect::<HashSet<String>>();
    let cellmaps = cellmap_names.iter().filter_map(|cellmap_name| {
        sspj.cellmap_list.iter().find(|cellmap| cellmap.load_file_path == *cellmap_name).cloned()
    }).collect::<Vec<SsCellMap>>();
    anime_binary.cellmaps = if cellmaps.is_empty() { None } else { Some(model_t::ssabt::convert_cellmapt_list(&sspj, &cellmaps)) };

    let external_instances = anime_pack.model.part_list.iter().filter(|part| part.part_type == SsPartType::Instance)
        .filter(|part| part.ref_anime_pack != anime_pack.name)
        .map(|part| format!("{}/{}", anime_pack.name, part.ref_anime_pack)).collect::<Vec<String>>();
    anime_binary.external_instances = if external_instances.is_empty() { None } else { Some(external_instances) };

    anime_binary
}
