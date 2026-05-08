use crate::model_t;

fn convert_effect_node_list(nodes: &[sspj::ssee::SsEffectNode]) -> Vec<ssab::ssab::ss::format::EffectNodeT> {
    nodes
        .iter()
        .map(|node| {
            let mut effect_node_t = ssab::ssab::ss::format::EffectNodeT::default();
            effect_node_t.index = node.array_index as i16;
            effect_node_t.parent_index = node.parent_index as i16;
            effect_node_t.effect_node_type = match &node.node_type {
                sspj::ssee::SsEffectNodeType::Root => ssab::ssab::ss::format::EffectNodeType::Root,
                sspj::ssee::SsEffectNodeType::Emitter => ssab::ssab::ss::format::EffectNodeType::Emitter,
                sspj::ssee::SsEffectNodeType::Particle => ssab::ssab::ss::format::EffectNodeType::Particle,
            };
            let behavior = &node.behavior;
            effect_node_t.cell_map_name_hash = crate::utils::fnv1a_hash_str(crate::utils::cellmap_basename(&behavior.cell_map_name));
            effect_node_t.cell_name_hash = crate::utils::fnv1a_hash_str(&behavior.cell_name);
            effect_node_t.blend_type = match &behavior.blend_type {
                sspj::sstypes::SsRenderBlendType::Mix => ssab::ssab::ss::format::RenderBlendType::Mix,
                sspj::sstypes::SsRenderBlendType::Add => ssab::ssab::ss::format::RenderBlendType::Add,
            };
            let behavior_list = behavior
                .list
                .iter()
                .filter_map(|function_type| match function_type {
                    sspj::ssee::SsEffectFunctionType::Base => None,
                    sspj::ssee::SsEffectFunctionType::Basic(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::EffectParticleElementBasicT::default();
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

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementBasic(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::RndSeedChange(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::IntValueEntryT::default();
                        behavior_t.value = p.seed;

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementRndSeedChange(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::Delay(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::IntValueEntryT::default();
                        behavior_t.value = p.delay_time;

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementDelay(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::Gravity(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::Vec2ValueEntryT::default();
                        behavior_t.value = Some(Box::new(model_t::common::convert_vec2t(&p.gravity)));

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementGravity(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::Position(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::RectValueEntryT::default();
                        behavior_t.value = Some(Box::new(model_t::common::convert_rectt(p.offset_x.value, p.offset_y.value, p.offset_x.subvalue, p.offset_y.subvalue)));

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementPosition(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::Rotation(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::EffectParticleElementRotationT::default();
                        let mut rotation_value = ssab::ssab::ss::format::Vec2T::default();
                        rotation_value.v1 = p.rotation.value;
                        rotation_value.v2 = p.rotation.subvalue;
                        behavior_t.rotation_value = Some(Box::new(rotation_value));
                        let mut rotation_add_value = ssab::ssab::ss::format::Vec2T::default();
                        rotation_add_value.v1 = p.rotation_add.value;
                        rotation_add_value.v2 = p.rotation_add.subvalue;
                        behavior_t.rotation_add_value = Some(Box::new(rotation_add_value));

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementRotation(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::TransRotation(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::EffectParticleElementRotationTransT::default();
                        behavior_t.rotation_factor = p.rotation_factor;
                        behavior_t.end_life_time_per = p.end_life_time_per;

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementRotationTrans(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::TransSpeed(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::Vec2ValueEntryT::default();
                        let mut speed = ssab::ssab::ss::format::Vec2T::default();
                        speed.v1 = p.speed.value;
                        speed.v2 = p.speed.subvalue;
                        behavior_t.value = Some(Box::new(speed));

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementTransSpeed(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::TangentialAcceleration(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::Vec2ValueEntryT::default();
                        let mut acceleration = ssab::ssab::ss::format::Vec2T::default();
                        acceleration.v1 = p.acceleration.value;
                        acceleration.v2 = p.acceleration.subvalue;
                        behavior_t.value = Some(Box::new(acceleration));

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementTangentialAcceleration(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::InitColor(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::EffectParticleElementInitColorT::default();
                        behavior_t.color_min_value = model_t::common::convert_color(&p.color.value);
                        behavior_t.color_max_value = model_t::common::convert_color(&p.color.subvalue);

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementInitColor(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::TransColor(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::EffectParticleElementTransColorT::default();
                        behavior_t.color_min_value = model_t::common::convert_color(&p.color.value);
                        behavior_t.color_max_value = model_t::common::convert_color(&p.color.subvalue);

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementTransColor(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::AlphaFade(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::Vec2ValueEntryT::default();
                        let mut alpha = ssab::ssab::ss::format::Vec2T::default();
                        alpha.v1 = p.disprange.value;
                        alpha.v2 = p.disprange.subvalue;
                        behavior_t.value = Some(Box::new(alpha));

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementAlphaFade(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::Size(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::RectAndVec2ValueTableT::default();
                        let mut size = ssab::ssab::ss::format::RectT::default();
                        size.x1 = p.size_x.value;
                        size.x2 = p.size_x.subvalue;
                        size.y1 = p.size_y.value;
                        size.y2 = p.size_y.subvalue;
                        behavior_t.size = Some(Box::new(size));
                        let mut scale_factor_value = ssab::ssab::ss::format::Vec2T::default();
                        scale_factor_value.v1 = p.scale_factor.value;
                        scale_factor_value.v2 = p.scale_factor.subvalue;
                        behavior_t.scale_factor_value = Some(Box::new(scale_factor_value));

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementSize(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::TransSize(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::RectAndVec2ValueTableT::default();
                        let mut size = ssab::ssab::ss::format::RectT::default();
                        size.x1 = p.size_x.value;
                        size.x2 = p.size_x.subvalue;
                        size.y1 = p.size_y.value;
                        size.y2 = p.size_y.subvalue;
                        behavior_t.size = Some(Box::new(size));
                        let mut scale_factor_value = ssab::ssab::ss::format::Vec2T::default();
                        scale_factor_value.v1 = p.scale_factor.value;
                        scale_factor_value.v2 = p.scale_factor.subvalue;
                        behavior_t.scale_factor_value = Some(Box::new(scale_factor_value));

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleElementTransSize(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::PointGravity(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::EffectParticlePointGravityT::default();
                        let mut position = ssab::ssab::ss::format::Vec2T::default();
                        position.v1 = p.position.x;
                        position.v2 = p.position.y;
                        behavior_t.position = Some(Box::new(position));
                        behavior_t.power = p.power;

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticlePointGravity(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::TurnToDirectionEnabled(p) => {
                        let mut behavior_t = ssab::ssab::ss::format::FloatValueEntryT::default();
                        behavior_t.value = p.rotation;

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleTurnToDirectionEnabled(Box::new(behavior_t));
                        Some(wrap)
                    }
                    sspj::ssee::SsEffectFunctionType::InfiniteEmitEnabled(_p) => {
                        let mut behavior_t = ssab::ssab::ss::format::IntValueEntryT::default();
                        behavior_t.value = 1;

                        let mut wrap = ssab::ssab::ss::format::EffectNodeBehaviorWrapperT::default();
                        wrap.value = ssab::ssab::ss::format::EffectNodeBehaviorT::EffectParticleInfiniteEmitEnabled(Box::new(behavior_t));
                        Some(wrap)
                    }
                })
                .collect::<Vec<_>>();
            effect_node_t.behavior = if behavior_list.is_empty() { None } else { Some(behavior_list) };

            effect_node_t
        })
        .collect::<Vec<ssab::ssab::ss::format::EffectNodeT>>()
}

pub fn convert_effect_list(_sspj: &sspj::sspj::SsProject, effect_file_vec: &[&sspj::ssee::SsEffectFile]) -> Vec<ssab::ssab::ss::format::EffectT> {
    let mut v = effect_file_vec
        .iter()
        .map(|effect_file| {
            let mut effect_t = ssab::ssab::ss::format::EffectT::default();
            effect_t.name_hash = crate::utils::fnv1a_hash_str(&effect_file.name);
            effect_t.name = if effect_file.name.is_empty() { None } else { Some(effect_file.name.to_string()) };
            effect_t.fps = effect_file.effect_data.fps as i16;
            effect_t.is_lock_rand_seed = effect_file.effect_data.is_lock_rand_seed;
            effect_t.lock_rand_seed = effect_file.effect_data.lock_rand_seed as i16;
            let mut layout_scale = ssab::ssab::ss::format::Vec2T::default();
            layout_scale.v1 = effect_file.effect_data.layout_scale_x as f32;
            layout_scale.v2 = effect_file.effect_data.layout_scale_y as f32;
            effect_t.layout_scale = Some(Box::new(layout_scale));
            effect_t.effect_node = if effect_file.effect_data.node_list.is_empty() { None } else { Some(convert_effect_node_list(&effect_file.effect_data.node_list)) };

            effect_t
        })
        .collect::<Vec<_>>();
    v.sort_by_key(|e| e.name_hash);
    v
}
