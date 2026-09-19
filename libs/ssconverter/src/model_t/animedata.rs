use std::collections::HashMap;

fn convert_animedata_labels(labels: &[sspj::ssae::SsLabel]) -> Vec<ssab::ssab::ss::format::LabelT> {
    let mut v = labels
        .iter()
        .map(|label| {
            let mut l = ssab::ssab::ss::format::LabelT::default();
            l.name_hash = crate::utils::fnv1a_hash_str(&label.name);
            l.name = label.name.to_string();
            l.time = label.time;
            l
        })
        .collect::<Vec<ssab::ssab::ss::format::LabelT>>();
    v.sort_by_key(|l| l.name_hash);
    v
}

fn convert_events_per_frames_part(
    map_events_per_frames: &mut HashMap<i32, ssab::ssab::ss::format::EventsPerFrameT>,
    anime_data: &sspj::ssae::SsAnimation,
    is_timeline: bool,
    parts_map: &HashMap<String, &sspj::ssae::SsPart>,
    sound_list_id_to_name_hash: &HashMap<i32, u32>,
) {
    for part_anime in &anime_data.part_animes {
        let Some(p) = parts_map.get(&part_anime.part_name) else {
            // not found part (an illegal case)
            continue;
        };

        let part_index = p.array_index as u16;

        part_anime.attributes.iter().for_each(|attribute| {
            match attribute {
                sspj::ssattribute::SsAttributeKind::User(v) if is_timeline => {
                    // MEMO: "UserData" set in setup-animation is not adopted in the timeline.
                    // MEMO: "UserData" does not backfill-complement.
                    v.keys.iter().for_each(|key| {
                        let mut u_args = ssab::ssab::ss::format::EventUserDataT::default();
                        u_args.part_index = part_index;
                        let mut userdata = ssab::ssab::ss::format::PartAttributeUserDataT::default();
                        userdata.integer = key.value.integer.as_ref().map(|v| {
                            let mut i = ssab::ssab::ss::format::IntValueEntryT::default();
                            i.value = *v;
                            Box::new(i)
                        });
                        userdata.rect = key.value.rect.as_ref().map(|v| {
                            let mut r = ssab::ssab::ss::format::BoundsT::default();
                            r.left = v.x1 as f32;
                            r.right = v.x2 as f32;
                            r.top = v.y1 as f32;
                            r.bottom = v.y2 as f32;
                            Box::new(r)
                        });
                        userdata.point = key.value.point.as_ref().map(|v| {
                            let mut p = ssab::ssab::ss::format::Vec2T::default();
                            p.v1 = v.x;
                            p.v2 = v.y;
                            Box::new(p)
                        });
                        userdata.str = key.value.user_string.as_ref().map(|v| v.to_string());
                        u_args.value = Box::new(userdata);

                        let events_per_frame = map_events_per_frames.entry(key.time).or_insert_with(|| {
                            let mut e = ssab::ssab::ss::format::EventsPerFrameT::default();
                            e.frame_index = key.time as u16;
                            e
                        });
                        events_per_frame.users.get_or_insert_with(Vec::new).push(u_args);
                    });
                }
                sspj::ssattribute::SsAttributeKind::Signal(v) if is_timeline => {
                    // MEMO: "Signal" set in setup-animation is not adopted in the timeline.
                    // MEMO: "Signal" does not backfill-complement.
                    v.keys.iter().for_each(|key| {
                        key.value.commands.iter().for_each(|command| {
                            let mut s_args = ssab::ssab::ss::format::EventSignalT::default();
                            s_args.part_index = part_index;
                            let mut s = ssab::ssab::ss::format::PartAttributeSignalT::default();
                            s.active = command.active;
                            s.command_id = command.command_id.to_string();
                            s.params = if command.params.is_empty() {
                                Vec::new()
                            } else {
                                command
                                    .params
                                    .iter()
                                    .map(|v| {
                                        let mut command_param_t = ssab::ssab::ss::format::SignalCommandParamT::default();
                                        command_param_t.param_id = v.param_id.to_string();
                                        command_param_t.type_ = match &v.param_type {
                                            sspj::ssattribute::SsAttributeSignalCommandParamType::Bool => ssab::ssab::ss::format::SignalCommandParamType::Bool,
                                            sspj::ssattribute::SsAttributeSignalCommandParamType::Index => ssab::ssab::ss::format::SignalCommandParamType::Index,
                                            sspj::ssattribute::SsAttributeSignalCommandParamType::Integer => ssab::ssab::ss::format::SignalCommandParamType::Integer,
                                            sspj::ssattribute::SsAttributeSignalCommandParamType::Floating => ssab::ssab::ss::format::SignalCommandParamType::Floating,
                                            sspj::ssattribute::SsAttributeSignalCommandParamType::Text => ssab::ssab::ss::format::SignalCommandParamType::Text,
                                            sspj::ssattribute::SsAttributeSignalCommandParamType::None => ssab::ssab::ss::format::SignalCommandParamType::None,
                                        };
                                        command_param_t.value = match &v.value {
                                            sspj::ssattribute::SsAttributeSignalCommandParamValue::Bool(v) => {
                                                let mut b = ssab::ssab::ss::format::BoolValueEntryT::default();
                                                b.value = *v;
                                                ssab::ssab::ss::format::SignalCommandParamValueT::Bool(Box::new(b))
                                            }
                                            sspj::ssattribute::SsAttributeSignalCommandParamValue::Index(v) => {
                                                let mut i = ssab::ssab::ss::format::IntValueEntryT::default();
                                                i.value = *v;
                                                ssab::ssab::ss::format::SignalCommandParamValueT::Index(Box::new(i))
                                            }
                                            sspj::ssattribute::SsAttributeSignalCommandParamValue::Integer(v) => {
                                                let mut i = ssab::ssab::ss::format::IntValueEntryT::default();
                                                i.value = *v;
                                                ssab::ssab::ss::format::SignalCommandParamValueT::Integer(Box::new(i))
                                            }
                                            sspj::ssattribute::SsAttributeSignalCommandParamValue::Floating(v) => {
                                                let mut f = ssab::ssab::ss::format::FloatValueEntryT::default();
                                                f.value = *v;
                                                ssab::ssab::ss::format::SignalCommandParamValueT::Floating(Box::new(f))
                                            }
                                            sspj::ssattribute::SsAttributeSignalCommandParamValue::Text(v) => {
                                                let mut s = ssab::ssab::ss::format::StringValueEntryT::default();
                                                s.value = v.to_string();
                                                ssab::ssab::ss::format::SignalCommandParamValueT::Text(Box::new(s))
                                            }
                                            sspj::ssattribute::SsAttributeSignalCommandParamValue::None => {
                                                let n = ssab::ssab::ss::format::NoneValueEntryT::default();
                                                ssab::ssab::ss::format::SignalCommandParamValueT::None(Box::new(n))
                                            }
                                        };

                                        command_param_t
                                    })
                                    .collect::<Vec<_>>()
                            };
                            s_args.value = Box::new(s);

                            let events_per_frame = map_events_per_frames.entry(key.time).or_insert_with(|| {
                                let mut e = ssab::ssab::ss::format::EventsPerFrameT::default();
                                e.frame_index = key.time as u16;
                                e
                            });
                            events_per_frame.signals.get_or_insert_with(Vec::new).push(s_args);
                        });
                    });
                }
                sspj::ssattribute::SsAttributeKind::Instance(v) => {
                    // MEMO: no frame-0 backfill. When a part's first Instance key
                    // is after frame 0, no frame-0 event is emitted; the runtime
                    // supplies the implicit frame-0 playback key (`_start..=_end`,
                    // speed 1.0, play-once, forward) via `synthetic_default`.
                    v.keys.iter().for_each(|key| {
                        let mut instance_t = ssab::ssab::ss::format::PartAttributeInstanceT::default();
                        instance_t.pingpong = key.value.pingpong;
                        instance_t.independent = key.value.independent;
                        instance_t.reverse = key.value.reverse;
                        // SS6's `SsInstanceAttr` carried both `infinity: bool`
                        // and `loop_num: i32`; SpriteStudio's `PartAttributeInstance`
                        // collapses them into a single `loop_num` where -1
                        // means infinite, matching `PlaybackController`'s
                        // convention. Encode the `infinity` flag here so the
                        // runtime never has to guess.
                        instance_t.loop_num = if key.value.infinity { -1 } else { key.value.loop_num };
                        instance_t.start_label_hash = crate::utils::fnv1a_hash_str(&key.value.start_label);
                        instance_t.start_offset = key.value.start_offset;
                        instance_t.end_label_hash = crate::utils::fnv1a_hash_str(&key.value.end_label);
                        instance_t.end_offset = key.value.end_offset;
                        instance_t.speed = key.value.speed;

                        // for target frame.
                        let mut i_args = ssab::ssab::ss::format::EventInstanceT::default();
                        i_args.part_index = part_index;
                        i_args.value = Box::new(instance_t);

                        let events_per_frame = map_events_per_frames.entry(key.time).or_insert_with(|| {
                            let mut e = ssab::ssab::ss::format::EventsPerFrameT::default();
                            e.frame_index = key.time as u16;
                            e
                        });

                        if let Some(pos) = events_per_frame.instances.get_or_insert_with(Vec::new).iter().position(|item| item.part_index == part_index) {
                            // MEMO: Last arrival priority. (between setup-animation and animation)
                            events_per_frame.instances.get_or_insert_default().remove(pos);
                            events_per_frame.instances.get_or_insert_default().insert(pos, i_args);
                        } else {
                            events_per_frame.instances.get_or_insert_with(Vec::new).push(i_args);
                        }
                    });
                }
                sspj::ssattribute::SsAttributeKind::Effect(v) => {
                    // MEMO: no frame-0 backfill. When an effect part's first Effect
                    // key is after frame 0, no frame-0 event is emitted; the runtime
                    // supplies the implicit frame-0 playback key (start_time 0,
                    // speed 1.0) via `synthetic_default`.
                    v.keys.iter().for_each(|key| {
                        let mut effect_t = ssab::ssab::ss::format::PartAttributeEffectT::default();
                        effect_t.start_time = key.value.start_time;
                        effect_t.speed = key.value.speed;
                        effect_t.independent = key.value.independent;

                        // for target frame.
                        let mut e_args = ssab::ssab::ss::format::EventEffectT::default();
                        e_args.part_index = part_index;
                        e_args.value = Box::new(effect_t);

                        let events_per_frame = map_events_per_frames.entry(key.time).or_insert_with(|| {
                            let mut e = ssab::ssab::ss::format::EventsPerFrameT::default();
                            e.frame_index = key.time as u16;
                            e
                        });

                        if let Some(pos) = events_per_frame.effects.get_or_insert_with(Vec::new).iter().position(|item| item.part_index == part_index) {
                            // MEMO: Last arrival priority. (between setup-animation and animation)
                            events_per_frame.effects.get_or_insert_default().remove(pos);
                            events_per_frame.effects.get_or_insert_default().insert(pos, e_args);
                        } else {
                            events_per_frame.effects.get_or_insert_with(Vec::new).push(e_args);
                        }
                    });
                }
                sspj::ssattribute::SsAttributeKind::Audio(v) => {
                    // MEMO: "Audio" does not backfill-complement.
                    v.keys.iter().for_each(|key| {
                        let mut a_args = ssab::ssab::ss::format::EventAudioT::default();
                        a_args.part_index = part_index;
                        let mut audio_t = ssab::ssab::ss::format::PartAttributeAudioT::default();
                        audio_t.sound_list_name_hash = sound_list_id_to_name_hash.get(&key.value.sound_list_id).copied().unwrap_or(0);
                        audio_t.sound_name_hash = crate::utils::fnv1a_hash_str(&key.value.sound_name);
                        audio_t.sound_name = key.value.sound_name.to_string();
                        audio_t.loop_num = key.value.loop_num;
                        a_args.value = Box::new(audio_t);

                        let events_per_frame = map_events_per_frames.entry(key.time).or_insert_with(|| {
                            let mut e = ssab::ssab::ss::format::EventsPerFrameT::default();
                            e.frame_index = key.time as u16;
                            e
                        });

                        if let Some(pos) = events_per_frame.audios.get_or_insert_with(Vec::new).iter().position(|item| item.part_index == part_index) {
                            // MEMO: Last arrival priority. (between setup-animation and animation)
                            events_per_frame.audios.get_or_insert_default().remove(pos);
                            events_per_frame.audios.get_or_insert_default().insert(pos, a_args);
                        } else {
                            events_per_frame.audios.get_or_insert_with(Vec::new).push(a_args);
                        }
                    });
                }
                _ => {
                    // skip
                }
            }
        });
    }
}

fn convert_events_per_frames(
    anime_data: &sspj::ssae::SsAnimation,
    setup_animation: &sspj::ssae::SsAnimation,
    parts_map: &HashMap<String, &sspj::ssae::SsPart>,
    sound_list_id_to_name_hash: &HashMap<i32, u32>,
) -> Option<Vec<ssab::ssab::ss::format::EventsPerFrameT>> {
    let mut map_events_per_frames = HashMap::<i32, ssab::ssab::ss::format::EventsPerFrameT>::new();

    convert_events_per_frames_part(&mut map_events_per_frames, setup_animation, false, parts_map, sound_list_id_to_name_hash);
    convert_events_per_frames_part(&mut map_events_per_frames, anime_data, true, parts_map, sound_list_id_to_name_hash);

    if map_events_per_frames.is_empty() {
        None
    } else {
        let mut vec_map: Vec<(i32, ssab::ssab::ss::format::EventsPerFrameT)> = map_events_per_frames.into_iter().collect();
        vec_map.sort_by_key(|a| a.0);
        Some(vec_map.into_iter().map(|(_, v)| v).collect::<Vec<ssab::ssab::ss::format::EventsPerFrameT>>())
    }
}
fn convert_animedatat(
    anime_data: &sspj::ssae::SsAnimation,
    setup_animation: &sspj::ssae::SsAnimation,
    parts_map: &HashMap<String, &sspj::ssae::SsPart>,
    sound_list_id_to_name_hash: &HashMap<i32, u32>,
    cellmaps: &CellmapResolver,
) -> ssab::ssab::ss::format::AnimationDataT {
    let mut anime_data_t = ssab::ssab::ss::format::AnimationDataT::default();

    anime_data_t.name = anime_data.name.to_string();
    anime_data_t.name_hash = crate::utils::fnv1a_hash_str(&anime_data.name);
    anime_data_t.parts_initial_attributes = convert_parts_initial_attributes(setup_animation, parts_map, cellmaps);
    anime_data_t.parts_animation_data = convert_parts_animation_data(anime_data, setup_animation, parts_map, cellmaps);
    anime_data_t.initial_events = convert_parts_initial_events(setup_animation, parts_map, sound_list_id_to_name_hash);
    anime_data_t.events = convert_events_per_frames(anime_data, setup_animation, parts_map, sound_list_id_to_name_hash);
    anime_data_t.labels = if anime_data.labels.is_empty() { None } else { Some(convert_animedata_labels(&anime_data.labels)) };
    anime_data_t.start_frame = anime_data.settings.start_frame as i16;
    anime_data_t.end_frame = anime_data.settings.end_frame as i16;
    anime_data_t.total_frame = anime_data.settings.frame_count as i16;
    anime_data_t.fps = anime_data.settings.fps as i16;
    let mut canvas_size = ssab::ssab::ss::format::Vec2T::default();
    canvas_size.v1 = anime_data.settings.canvas_size.x;
    canvas_size.v2 = anime_data.settings.canvas_size.y;
    anime_data_t.canvas_size = Box::new(canvas_size);
    let mut pivot = ssab::ssab::ss::format::Vec2T::default();
    pivot.v1 = anime_data.settings.pivot.x;
    pivot.v2 = anime_data.settings.pivot.y;
    anime_data_t.pivot = Box::new(pivot);

    anime_data_t
}

fn convert_interpolation_type(interpolation: &sspj::sstypes::SsInterpolationType) -> ssab::ssab::ss::format::InterpolationType {
    match interpolation {
        sspj::sstypes::SsInterpolationType::None => ssab::ssab::ss::format::InterpolationType::None,
        sspj::sstypes::SsInterpolationType::Linear => ssab::ssab::ss::format::InterpolationType::Linear,
        sspj::sstypes::SsInterpolationType::Hermite => ssab::ssab::ss::format::InterpolationType::Hermite,
        sspj::sstypes::SsInterpolationType::Bezier => ssab::ssab::ss::format::InterpolationType::Bezier,
        sspj::sstypes::SsInterpolationType::Acceleration => ssab::ssab::ss::format::InterpolationType::Acceleration,
        sspj::sstypes::SsInterpolationType::Deceleration => ssab::ssab::ss::format::InterpolationType::Deceleration,
        sspj::sstypes::SsInterpolationType::EaseIn => ssab::ssab::ss::format::InterpolationType::EaseIn,
        sspj::sstypes::SsInterpolationType::EaseOut => ssab::ssab::ss::format::InterpolationType::EaseOut,
        sspj::sstypes::SsInterpolationType::EaseInOut => ssab::ssab::ss::format::InterpolationType::EaseInOut,
        sspj::sstypes::SsInterpolationType::EaseExponentialIn => ssab::ssab::ss::format::InterpolationType::EaseExponentialIn,
        sspj::sstypes::SsInterpolationType::EaseExponentialOut => ssab::ssab::ss::format::InterpolationType::EaseExponentialOut,
        sspj::sstypes::SsInterpolationType::EaseExponentialInOut => ssab::ssab::ss::format::InterpolationType::EaseExponentialInOut,
        sspj::sstypes::SsInterpolationType::EaseSineIn => ssab::ssab::ss::format::InterpolationType::EaseSineIn,
        sspj::sstypes::SsInterpolationType::EaseSineOut => ssab::ssab::ss::format::InterpolationType::EaseSineOut,
        sspj::sstypes::SsInterpolationType::EaseSineInOut => ssab::ssab::ss::format::InterpolationType::EaseSineInOut,
        sspj::sstypes::SsInterpolationType::EaseElasticIn => ssab::ssab::ss::format::InterpolationType::EaseElasticIn,
        sspj::sstypes::SsInterpolationType::EaseElasticOut => ssab::ssab::ss::format::InterpolationType::EaseElasticOut,
        sspj::sstypes::SsInterpolationType::EaseElasticInOut => ssab::ssab::ss::format::InterpolationType::EaseElasticInOut,
        sspj::sstypes::SsInterpolationType::EaseBounceIn => ssab::ssab::ss::format::InterpolationType::EaseBounceIn,
        sspj::sstypes::SsInterpolationType::EaseBounceOut => ssab::ssab::ss::format::InterpolationType::EaseBounceOut,
        sspj::sstypes::SsInterpolationType::EaseBounceInOut => ssab::ssab::ss::format::InterpolationType::EaseBounceInOut,
        sspj::sstypes::SsInterpolationType::EaseBackIn => ssab::ssab::ss::format::InterpolationType::EaseBackIn,
        sspj::sstypes::SsInterpolationType::EaseBackOut => ssab::ssab::ss::format::InterpolationType::EaseBackOut,
        sspj::sstypes::SsInterpolationType::EaseBackInOut => ssab::ssab::ss::format::InterpolationType::EaseBackInOut,
        _ => ssab::ssab::ss::format::InterpolationType::None,
    }
}

fn merge_setup_attributes_to_part_attribute_t(p_args: &mut ssab::ssab::ss::format::InitialPartAttributesT, setup_attributes: &[sspj::ssattribute::SsAttributeKind], cellmaps: &CellmapResolver, part_type: &sspj::sstypes::SsPartType) {
    // Only frame-0 ("setup") keys are composited here: the caller builds the
    // initial part attributes from frame 0, so keys with `time == 0` are the
    // ones that matter.
    setup_attributes.iter().for_each(|attribute| {
        match attribute {
            sspj::ssattribute::SsAttributeKind::Cell(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    let mut value = ssab::ssab::ss::format::PartAttributeCellT::default();
                    value.cellmap_name_hash = cellmaps.name_hash(_key.value.map_id);
                    value.name_hash = crate::utils::fnv1a_hash_str(&_key.value.name);
                    p_args.cell = Some(Box::new(value));
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeCell;
                }
            }
            sspj::ssattribute::SsAttributeKind::PosX(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.position_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePosX;
                }
            }
            sspj::ssattribute::SsAttributeKind::PosY(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.position_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePosY;
                }
            }
            sspj::ssattribute::SsAttributeKind::PosZ(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.position_z = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePosZ;
                }
            }
            sspj::ssattribute::SsAttributeKind::RotX(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.rotation_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeRotX;
                }
            }
            sspj::ssattribute::SsAttributeKind::RotY(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.rotation_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeRotY;
                }
            }
            sspj::ssattribute::SsAttributeKind::RotZ(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.rotation_z = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeRotZ;
                }
            }
            sspj::ssattribute::SsAttributeKind::SclX(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.scale_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeSclX;
                }
            }
            sspj::ssattribute::SsAttributeKind::SclY(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.scale_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeSclY;
                }
            }
            sspj::ssattribute::SsAttributeKind::LosclX(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.local_scale_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeLosclX;
                }
            }
            sspj::ssattribute::SsAttributeKind::LosclY(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.local_scale_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeLosclY;
                }
            }
            sspj::ssattribute::SsAttributeKind::Alpha(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.alpha = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeAlpha;
                }
            }
            sspj::ssattribute::SsAttributeKind::LoAlpha(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.local_alpha = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeLoAlpha;
                }
            }
            sspj::ssattribute::SsAttributeKind::Prio(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    // TODO: when Z-sorting, priority should be based on spatially transformed Z-values.
                    p_args.priority = _key.value as f32;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePrio;
                }
            }
            sspj::ssattribute::SsAttributeKind::FlipH(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.flip_h = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeFlipH;
                }
            }
            sspj::ssattribute::SsAttributeKind::FlipV(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.flip_v = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeFlipV;
                }
            }
            sspj::ssattribute::SsAttributeKind::Hide(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.hide = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeHide;
                }
            }
            sspj::ssattribute::SsAttributeKind::PartsColor(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    let value = build_part_attribute_part_color_t(&_key.value, part_type);
                    p_args.part_color = Some(Box::new(value));
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePartColor;
                }
            }
            sspj::ssattribute::SsAttributeKind::Color => { /* MEMO: Not Supported (Feature prior to Ver.6.0) */ }
            sspj::ssattribute::SsAttributeKind::Shader(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    let mut value = ssab::ssab::ss::format::PartAttributeShaderT::default();
                    value.id = _key.value.id.clone();
                    value.id_hash = crate::utils::fnv1a_hash_str(&_key.value.id);

                    value.param0 = _key.value.param0;
                    value.param1 = _key.value.param1;
                    value.param2 = _key.value.param2;
                    value.param3 = _key.value.param3;
                    value.param4 = _key.value.param4;
                    value.param5 = _key.value.param5;
                    value.param6 = _key.value.param6;
                    value.param7 = _key.value.param7;

                    value.map0_cellmap_name_hash = cellmaps.name_hash(_key.value.map0.map_id);
                    value.map0_name_hash = crate::utils::fnv1a_hash_str(&_key.value.map0.name);
                    value.map1_cellmap_name_hash = cellmaps.name_hash(_key.value.map1.map_id);
                    value.map1_name_hash = crate::utils::fnv1a_hash_str(&_key.value.map1.name);

                    p_args.shader = Some(Box::new(value));
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeShader;
                }
            }
            sspj::ssattribute::SsAttributeKind::Vertex(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    if _key.value.use_skew.unwrap_or(false) {
                        // Skew
                        match &_key.value.skew {
                            None => {
                                p_args.skew_x = 0.0_f32;
                                p_args.skew_y = 0.0_f32;
                            }
                            Some(_skew) => {
                                p_args.skew_x = _skew.x;
                                p_args.skew_y = _skew.y;
                            }
                        }
                        p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeSkew;

                        p_args.vertex = None; // Mutually exclusive with "Vertex"
                    } else {
                        // Vertex
                        let mut value = ssab::ssab::ss::format::PartAttributeVertexT::default();
                        let mut coordinate = ssab::ssab::ss::format::Vec2T::default();

                        if let Some(_lt) = _key.value.lt.as_ref() {
                            coordinate.v1 = _lt.x;
                            coordinate.v2 = _lt.y;
                        }
                        value.lt = Some(Box::new(coordinate));

                        coordinate = ssab::ssab::ss::format::Vec2T::default();
                        if let Some(_rt) = _key.value.rt.as_ref() {
                            coordinate.v1 = _rt.x;
                            coordinate.v2 = _rt.y;
                        }
                        value.rt = Some(Box::new(coordinate));

                        coordinate = ssab::ssab::ss::format::Vec2T::default();
                        if let Some(_lb) = _key.value.lb.as_ref() {
                            coordinate.v1 = _lb.x;
                            coordinate.v2 = _lb.y;
                        }
                        value.lb = Some(Box::new(coordinate));

                        coordinate = ssab::ssab::ss::format::Vec2T::default();
                        if let Some(_rb) = _key.value.rb.as_ref() {
                            coordinate.v1 = _rb.x;
                            coordinate.v2 = _rb.y;
                        }
                        value.rb = Some(Box::new(coordinate));

                        p_args.vertex = Some(Box::new(value));
                        p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeVertex;

                        p_args.skew_x = 0.0_f32; // mutually exclusive with "Skew"
                        p_args.skew_y = 0.0_f32;
                    }
                }
            }
            sspj::ssattribute::SsAttributeKind::PivotX(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.pivot_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePivotX;
                }
            }
            sspj::ssattribute::SsAttributeKind::PivotY(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.pivot_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePivotY;
                }
            }
            sspj::ssattribute::SsAttributeKind::AnchorX(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.anchor_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeAnchorX;
                }
            }
            sspj::ssattribute::SsAttributeKind::AnchorY(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.anchor_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeAnchorY;
                }
            }
            sspj::ssattribute::SsAttributeKind::SizeX(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.size_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeSizeX;
                }
            }
            sspj::ssattribute::SsAttributeKind::SizeY(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.size_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeSizeY;
                }
            }
            sspj::ssattribute::SsAttributeKind::ImgFlipH(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.img_flip_h = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeImgFlipH;
                }
            }
            sspj::ssattribute::SsAttributeKind::ImgFlipV(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.img_flip_v = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeImgFlipV;
                }
            }
            sspj::ssattribute::SsAttributeKind::UvtX(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.uv_translation_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeUvtX;
                }
            }
            sspj::ssattribute::SsAttributeKind::UvtY(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.uv_translation_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeUvtY;
                }
            }
            sspj::ssattribute::SsAttributeKind::UvrZ(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    /* MEMO: (Preliminary) "rotation" is set to "move.z". */
                    p_args.uv_rotation_z = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeUvrZ;
                }
            }
            sspj::ssattribute::SsAttributeKind::UvsX(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.uv_scale_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeUvsX;
                }
            }
            sspj::ssattribute::SsAttributeKind::UvsY(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.uv_scale_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeUvsY;
                }
            }
            sspj::ssattribute::SsAttributeKind::Boundr(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.bounding_radius = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeBoundr;
                }
            }
            sspj::ssattribute::SsAttributeKind::Mask(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.mask = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeMask;
                }
            }
            sspj::ssattribute::SsAttributeKind::Deform(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    let mut value = ssab::ssab::ss::format::PartAttributeDeformT::default();
                    value.coord_x = Some(_key.value.vchg.iter().map(|p| p.x).collect());
                    value.coord_y = Some(_key.value.vchg.iter().map(|p| p.y).collect());
                    value.index = Some(_key.value.vchg.iter().map(|p| p.x_index as i16).collect());
                    p_args.deform = Some(Box::new(value));
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeDeform;
                }
            }
            sspj::ssattribute::SsAttributeKind::Texchange(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.texture = crate::utils::fnv1a_hash_texture_name(&_key.value.texture_name);
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeTexture;
                }
            }
            _ => {
                // Empty.
            }
        }
    });
}

fn merge_setup_attributes_to_part_event_t(p_args: &mut [ssab::ssab::ss::format::InitialEventsT], part_index: u16, setup_attributes: &[sspj::ssattribute::SsAttributeKind], sound_list_id_to_name_hash: &HashMap<i32, u32>) {
    let mut initial_event_t = ssab::ssab::ss::format::InitialEventsT::default();
    let mut flag_exist_event = false;

    setup_attributes.iter().for_each(|attribute| {
        match attribute {
            sspj::ssattribute::SsAttributeKind::User(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    let mut value = ssab::ssab::ss::format::PartAttributeUserDataT::default();
                    value.integer = _key.value.integer.as_ref().map(|v| {
                        let mut i = ssab::ssab::ss::format::IntValueEntryT::default();
                        i.value = *v;
                        Box::new(i)
                    });
                    value.rect = _key.value.rect.as_ref().map(|v| {
                        let mut r = ssab::ssab::ss::format::BoundsT::default();
                        r.left = v.x1 as f32;
                        r.right = v.x2 as f32;
                        r.top = v.y1 as f32;
                        r.bottom = v.y2 as f32;
                        Box::new(r)
                    });
                    value.point = _key.value.point.as_ref().map(|v| {
                        let mut p = ssab::ssab::ss::format::Vec2T::default();
                        p.v1 = v.x;
                        p.v2 = v.y;
                        Box::new(p)
                    });
                    value.str = _key.value.user_string.as_ref().map(|v| v.to_string());

                    initial_event_t.user = Some(Box::new(value));
                    flag_exist_event = true;
                }
            }
            sspj::ssattribute::SsAttributeKind::Signal(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    _key.value.commands.iter().for_each(|command| {
                        let mut value = ssab::ssab::ss::format::PartAttributeSignalT::default();
                        value.active = command.active;
                        value.command_id = command.command_id.to_string();
                        value.params = if command.params.is_empty() {
                            Vec::new()
                        } else {
                            command
                                .params
                                .iter()
                                .map(|p| {
                                    let mut command_param_t = ssab::ssab::ss::format::SignalCommandParamT::default();
                                    command_param_t.param_id = p.param_id.to_string();
                                    command_param_t.type_ = match &p.param_type {
                                        sspj::ssattribute::SsAttributeSignalCommandParamType::Bool => ssab::ssab::ss::format::SignalCommandParamType::Bool,
                                        sspj::ssattribute::SsAttributeSignalCommandParamType::Index => ssab::ssab::ss::format::SignalCommandParamType::Index,
                                        sspj::ssattribute::SsAttributeSignalCommandParamType::Integer => ssab::ssab::ss::format::SignalCommandParamType::Integer,
                                        sspj::ssattribute::SsAttributeSignalCommandParamType::Floating => ssab::ssab::ss::format::SignalCommandParamType::Floating,
                                        sspj::ssattribute::SsAttributeSignalCommandParamType::Text => ssab::ssab::ss::format::SignalCommandParamType::Text,
                                        sspj::ssattribute::SsAttributeSignalCommandParamType::None => ssab::ssab::ss::format::SignalCommandParamType::None,
                                    };

                                    command_param_t.value = match &p.value {
                                        sspj::ssattribute::SsAttributeSignalCommandParamValue::Bool(v) => {
                                            let mut b = ssab::ssab::ss::format::BoolValueEntryT::default();
                                            b.value = *v;
                                            ssab::ssab::ss::format::SignalCommandParamValueT::Bool(Box::new(b))
                                        }
                                        sspj::ssattribute::SsAttributeSignalCommandParamValue::Index(v) => {
                                            let mut i = ssab::ssab::ss::format::IntValueEntryT::default();
                                            i.value = *v;
                                            ssab::ssab::ss::format::SignalCommandParamValueT::Index(Box::new(i))
                                        }
                                        sspj::ssattribute::SsAttributeSignalCommandParamValue::Integer(v) => {
                                            let mut i = ssab::ssab::ss::format::IntValueEntryT::default();
                                            i.value = *v;
                                            ssab::ssab::ss::format::SignalCommandParamValueT::Integer(Box::new(i))
                                        }
                                        sspj::ssattribute::SsAttributeSignalCommandParamValue::Floating(v) => {
                                            let mut f = ssab::ssab::ss::format::FloatValueEntryT::default();
                                            f.value = *v;
                                            ssab::ssab::ss::format::SignalCommandParamValueT::Floating(Box::new(f))
                                        }
                                        sspj::ssattribute::SsAttributeSignalCommandParamValue::Text(v) => {
                                            let mut s = ssab::ssab::ss::format::StringValueEntryT::default();
                                            s.value = v.to_string();
                                            ssab::ssab::ss::format::SignalCommandParamValueT::Text(Box::new(s))
                                        }
                                        sspj::ssattribute::SsAttributeSignalCommandParamValue::None => {
                                            let n = ssab::ssab::ss::format::NoneValueEntryT::default();
                                            ssab::ssab::ss::format::SignalCommandParamValueT::None(Box::new(n))
                                        }
                                    };

                                    command_param_t
                                })
                                .collect::<Vec<_>>()
                        };

                        initial_event_t.signal = Some(Box::new(value));
                        flag_exist_event = true;
                    });
                }
            }
            sspj::ssattribute::SsAttributeKind::Instance(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    let mut value = ssab::ssab::ss::format::PartAttributeInstanceT::default();
                    value.pingpong = _key.value.pingpong;
                    value.independent = _key.value.independent;
                    value.reverse = _key.value.reverse;
                    // Collapse SS6's `infinity` flag into SS7's loop_num=-1.
                    // See SsAttributeKind::Instance handling above for details.
                    value.loop_num = if _key.value.infinity { -1 } else { _key.value.loop_num };
                    value.start_label_hash = crate::utils::fnv1a_hash_str(&_key.value.start_label);
                    value.start_offset = _key.value.start_offset;
                    value.end_label_hash = crate::utils::fnv1a_hash_str(&_key.value.end_label);
                    value.end_offset = _key.value.end_offset;
                    value.speed = _key.value.speed;

                    initial_event_t.instance = Some(Box::new(value));
                    flag_exist_event = true;
                }
            }
            sspj::ssattribute::SsAttributeKind::Effect(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    let mut value = ssab::ssab::ss::format::PartAttributeEffectT::default();
                    value.start_time = _key.value.start_time;
                    value.speed = _key.value.speed;
                    value.independent = _key.value.independent;

                    initial_event_t.effect = Some(Box::new(value));
                    flag_exist_event = true;
                }
            }
            sspj::ssattribute::SsAttributeKind::Audio(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    let mut value = ssab::ssab::ss::format::PartAttributeAudioT::default();
                    value.sound_list_name_hash = sound_list_id_to_name_hash.get(&_key.value.sound_list_id).copied().unwrap_or(0);
                    value.sound_name_hash = crate::utils::fnv1a_hash_str(&_key.value.sound_name);
                    value.sound_name = _key.value.sound_name.to_string();
                    value.loop_num = _key.value.loop_num;

                    initial_event_t.audio = Some(Box::new(value));
                    flag_exist_event = true;
                }
            }
            _ => {
                // Empty.
            }
        }
    });

    if flag_exist_event {
        p_args[part_index as usize] = initial_event_t;
    }
}

fn convert_parts_initial_attributes(anime_data: &sspj::ssae::SsAnimation, parts_map: &HashMap<String, &sspj::ssae::SsPart>, cellmaps: &CellmapResolver) -> Vec<ssab::ssab::ss::format::InitialPartAttributesT> {
    // Index-aligned with the part list (same convention as `convert_parts_animation_data`):
    // size to the part count and place each entry by `array_index`, so the runtime can
    // address `parts_initial_attributes[part_index]` directly. Parts absent from the
    // setup animation keep `InitialPartAttributesT::default()`.
    let mut parts_initial_attributes_t = Vec::<ssab::ssab::ss::format::InitialPartAttributesT>::new();
    parts_initial_attributes_t.resize(parts_map.len(), ssab::ssab::ss::format::InitialPartAttributesT::default());

    for part_anime in &anime_data.part_animes {
        let Some(p) = parts_map.get(&part_anime.part_name) else {
            // not found part (an illegal case)
            continue;
        };

        let index_part = p.array_index as usize;
        merge_setup_attributes_to_part_attribute_t(&mut parts_initial_attributes_t[index_part], &part_anime.attributes, cellmaps, &p.part_type);

        // A bone's rest pose lives in the ssae `bonePosition`/`boneRotation` tags
        // rather than in setup keyframes; reflect it into the setup-pose transform
        // so the bone's rest frame is represented like any other part.
        if matches!(p.part_type, sspj::sstypes::SsPartType::Armature) {
            let attr = &mut parts_initial_attributes_t[index_part];
            attr.position_x = p.bone_position.x;
            attr.position_y = p.bone_position.y;
            attr.rotation_z = p.bone_rotation;
            attr.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePosX;
            attr.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePosY;
            attr.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeRotZ;
        }
    }

    bake_hide_inheritance_initial(&mut parts_initial_attributes_t, parts_map);

    parts_initial_attributes_t
}

/// Bakes SpriteStudio's per-part HIDE inheritance into the setup-pose `hide` of each
/// part's [`InitialPartAttributes`] — the scalar counterpart of
/// `bake_hide_inheritance`, which does the same for the per-frame `hide_array`.
/// A part that inherits HIDE (`ineheritRates/HIDE`) adopts its parent's
/// already-resolved setup hidden state, so the setup keys written to the SSAB
/// reflect inheritance just like the animation keyframes do.
fn bake_hide_inheritance_initial(parts_initial_attributes_t: &mut [ssab::ssab::ss::format::InitialPartAttributesT], parts_map: &HashMap<String, &sspj::ssae::SsPart>) {
    let num_parts = parts_initial_attributes_t.len();

    // Index parts by `array_index` for parent lookups.
    let mut parts_by_index: Vec<Option<&sspj::ssae::SsPart>> = vec![None; num_parts];
    for part in parts_map.values() {
        let idx = part.array_index as usize;
        if idx < num_parts {
            parts_by_index[idx] = Some(part);
        }
    }

    // effective[i] = part i's setup hidden state after resolving inheritance.
    // Parts are visited in `array_index` order (parent before child), so the
    // inheritance is transitive down a chain of HIDE-inheriting parts.
    let mut effective: Vec<bool> = Vec::with_capacity(num_parts);

    for i in 0..num_parts {
        let own = parts_initial_attributes_t[i].hide;

        let Some(part) = parts_by_index[i] else {
            effective.push(own);
            continue;
        };

        let parent_idx = part.parent_index;
        let inherits = part.inherit_rates.hide && parent_idx >= 0 && (parent_idx as usize) < i;
        let eff = if inherits { own || effective[parent_idx as usize] } else { own };

        if eff != own {
            // Inheritance turned a visible part hidden — record it as an
            // explicitly-set attribute, matching a real setup HIDE key.
            let attr = &mut parts_initial_attributes_t[i];
            attr.hide = true;
            attr.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeHide;
        }
        effective.push(eff);
    }
}

fn convert_parts_initial_events(anime_data: &sspj::ssae::SsAnimation, parts_map: &HashMap<String, &sspj::ssae::SsPart>, sound_list_id_to_name_hash: &HashMap<i32, u32>) -> Option<Vec<ssab::ssab::ss::format::InitialEventsT>> {
    let mut events_t = Vec::<ssab::ssab::ss::format::InitialEventsT>::new();
    events_t.resize(parts_map.len(), ssab::ssab::ss::format::InitialEventsT::default());

    for part_anime in &anime_data.part_animes {
        let Some(p) = parts_map.get(&part_anime.part_name) else {
            // not found part (an illegal case)
            continue;
        };

        merge_setup_attributes_to_part_event_t(&mut events_t, p.array_index as u16, &part_anime.attributes, sound_list_id_to_name_hash);
    }

    Some(events_t)
}
macro_rules! process_interpolatable_attribute {
    ($keys:expr, $target_array:expr, $arg_type:ty, $extract_value:expr) => {{
        for key in $keys {
            let mut args = <$arg_type>::default();
            args.index = key.time as u16;
            args.interpolation = convert_interpolation_type(&key.ip_type);
            args.value = $extract_value(key);

            match &key.ip_type {
                sspj::sstypes::SsInterpolationType::Hermite | sspj::sstypes::SsInterpolationType::Bezier => {
                    let mut curve_t = ssab::ssab::ss::format::SsCurveT::default();
                    curve_t.start_time = key.curve.start_time;
                    curve_t.start_value = key.curve.start_value;
                    curve_t.end_time = key.curve.end_time;
                    curve_t.end_value = key.curve.end_value;
                    args.curve = Some(Box::new(curve_t));
                }
                sspj::sstypes::SsInterpolationType::EaseIn
                | sspj::sstypes::SsInterpolationType::EaseOut
                | sspj::sstypes::SsInterpolationType::EaseInOut
                | sspj::sstypes::SsInterpolationType::EaseElasticIn
                | sspj::sstypes::SsInterpolationType::EaseElasticOut
                | sspj::sstypes::SsInterpolationType::EaseElasticInOut => {
                    args.easing_rate = key.easing_rate;
                }
                _ => {}
            }

            let array = $target_array.get_or_insert_with(|| {
                let mut array_new = Vec::new();
                if key.time > 0 {
                    let mut args_frame_0 = args.clone();
                    args_frame_0.index = 0;
                    args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                    args_frame_0.curve = None;
                    args_frame_0.easing_rate = 0.0;
                    array_new.push(args_frame_0);
                }
                array_new
            });

            if let Some(pos) = array.iter().position(|item| item.index == args.index) {
                array.remove(pos);
                array.insert(pos, args);
            } else {
                array.push(args);
            }
        }
    }};
}

macro_rules! process_simple_attribute {
    // A synthesized frame-0 key clones the first frame-1+ key's value.
    ($keys:expr, $target_array:expr, $arg_type:ty, $extract_value:expr) => {
        process_simple_attribute!($keys, $target_array, $arg_type, $extract_value, frame0_override = |_: &mut $arg_type| {})
    };
    // `frame0_override` runs on a synthesized frame-0 key, letting an attribute
    // adjust the back-filled value (used by Hide; see its match arm).
    ($keys:expr, $target_array:expr, $arg_type:ty, $extract_value:expr, frame0_override = $frame0_override:expr) => {{
        let frame0_override = $frame0_override;
        for key in $keys {
            let mut args = <$arg_type>::default();
            args.index = key.time as u16;
            args.value = $extract_value(key);

            let array = $target_array.get_or_insert_with(|| {
                let mut array_new = Vec::new();
                if key.time > 0 {
                    let mut args_frame_0 = args.clone();
                    args_frame_0.index = 0;
                    frame0_override(&mut args_frame_0);
                    array_new.push(args_frame_0);
                }
                array_new
            });

            if let Some(pos) = array.iter().position(|item| item.index == args.index) {
                array.remove(pos);
                array.insert(pos, args);
            } else {
                array.push(args);
            }
        }
    }};
}

fn convert_part_animation_data(p_arg: &mut ssab::ssab::ss::format::PartAnimationDataT, part_anime: &sspj::ssae::SsPartAnime, cellmaps: &CellmapResolver, is_setup: bool, part_type: &sspj::sstypes::SsPartType) {
    part_anime.attributes.iter().for_each(|attribute| match attribute {
        sspj::ssattribute::SsAttributeKind::Cell(v) => {
            process_simple_attribute!(&v.keys, p_arg.cell_array, ssab::ssab::ss::format::PartAnimationDataCellValueT, |k: &sspj::ssattribute::SsAttributeCellKey| {
                let mut c_args = ssab::ssab::ss::format::PartAttributeCellT::default();
                c_args.cellmap_name_hash = cellmaps.name_hash(k.value.map_id);
                c_args.name_hash = crate::utils::fnv1a_hash_str(&k.value.name);
                Box::new(c_args)
            });
        }
        sspj::ssattribute::SsAttributeKind::PosX(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.position_x_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::PosY(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.position_y_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::PosZ(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.position_z_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::RotX(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.rotation_x_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::RotY(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.rotation_y_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::RotZ(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.rotation_z_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::SclX(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.scale_x_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::SclY(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.scale_y_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::LosclX(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.local_scale_x_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::LosclY(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.local_scale_y_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::Alpha(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.alpha_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::LoAlpha(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.local_alpha_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::Prio(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.priority_array, ssab::ssab::ss::format::PartAnimationDataIntValueT, |k: &sspj::ssattribute::SsAttributeIntKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::FlipH(v) => {
            process_simple_attribute!(&v.keys, p_arg.flip_h_array, ssab::ssab::ss::format::PartAnimationDataBoolValueT, |k: &sspj::ssattribute::SsAttributeBoolKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::FlipV(v) => {
            process_simple_attribute!(&v.keys, p_arg.flip_v_array, ssab::ssab::ss::format::PartAnimationDataBoolValueT, |k: &sspj::ssattribute::SsAttributeBoolKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::Hide(v) => {
            // Hide is special-cased: when no frame-0 key is supplied by the setup
            // animation, a part defaults to hidden (`true`) rather than back-filling
            // frame 0 from the first frame-1+ key (the behavior of every other
            // attribute). A frame-0 key synthesized while processing the setup
            // animation itself keeps the back-fill behavior.
            process_simple_attribute!(
                &v.keys,
                p_arg.hide_array,
                ssab::ssab::ss::format::PartAnimationDataBoolValueT,
                |k: &sspj::ssattribute::SsAttributeBoolKey| k.value,
                frame0_override = |frame0: &mut ssab::ssab::ss::format::PartAnimationDataBoolValueT| {
                    if !is_setup {
                        frame0.value = true;
                    }
                }
            );
        }
        sspj::ssattribute::SsAttributeKind::PivotX(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.pivot_x_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::PivotY(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.pivot_y_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::AnchorX(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.anchor_x_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::AnchorY(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.anchor_y_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::SizeX(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.size_x_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::SizeY(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.size_y_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::ImgFlipH(v) => {
            process_simple_attribute!(&v.keys, p_arg.img_flip_h_array, ssab::ssab::ss::format::PartAnimationDataBoolValueT, |k: &sspj::ssattribute::SsAttributeBoolKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::ImgFlipV(v) => {
            process_simple_attribute!(&v.keys, p_arg.img_flip_v_array, ssab::ssab::ss::format::PartAnimationDataBoolValueT, |k: &sspj::ssattribute::SsAttributeBoolKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::UvtX(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.uv_translation_x_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::UvtY(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.uv_translation_y_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::UvrZ(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.uv_rotation_z_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::UvsX(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.uv_scale_x_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::UvsY(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.uv_scale_y_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::Boundr(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.bounding_radius_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::Mask(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.mask_array, ssab::ssab::ss::format::PartAnimationDataFloatValueT, |k: &sspj::ssattribute::SsAttributeFloatKey| k.value);
        }
        sspj::ssattribute::SsAttributeKind::PartsColor(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.part_color_array, ssab::ssab::ss::format::PartAnimationDataPartColorT, |k: &sspj::ssattribute::SsAttributePartsColorKey| { Box::new(build_part_attribute_part_color_t(&k.value, part_type)) });
        }
        sspj::ssattribute::SsAttributeKind::Shader(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.shader_array, ssab::ssab::ss::format::PartAnimationDataShaderT, |k: &sspj::ssattribute::SsAttributeShaderKey| {
                let mut value = ssab::ssab::ss::format::PartAttributeShaderT::default();
                value.id = k.value.id.clone();
                value.id_hash = crate::utils::fnv1a_hash_str(&k.value.id);
                value.param0 = k.value.param0;
                value.param1 = k.value.param1;
                value.param2 = k.value.param2;
                value.param3 = k.value.param3;
                value.param4 = k.value.param4;
                value.param5 = k.value.param5;
                value.param6 = k.value.param6;
                value.param7 = k.value.param7;
                value.map0_cellmap_name_hash = cellmaps.name_hash(k.value.map0.map_id);
                value.map0_name_hash = crate::utils::fnv1a_hash_str(&k.value.map0.name);
                value.map1_cellmap_name_hash = cellmaps.name_hash(k.value.map1.map_id);
                value.map1_name_hash = crate::utils::fnv1a_hash_str(&k.value.map1.name);
                Box::new(value)
            });
        }
        sspj::ssattribute::SsAttributeKind::Vertex(v) => {
            let mut skew_keys = Vec::new();
            let mut vertex_keys = Vec::new();
            for key in &v.keys {
                if key.value.use_skew.unwrap_or(false) {
                    skew_keys.push(key);
                } else {
                    vertex_keys.push(key);
                }
            }
            if !skew_keys.is_empty() {
                process_interpolatable_attribute!(&skew_keys, p_arg.skew_array, ssab::ssab::ss::format::PartAnimationDataVec2T, |k: &sspj::ssattribute::SsAttributeVertexKey| {
                    let mut val = ssab::ssab::ss::format::Vec2T::default();
                    if let Some(v) = k.value.skew.as_ref() {
                        val.v1 = v.x;
                        val.v2 = v.y;
                    }
                    Box::new(val)
                });
            }
            if !vertex_keys.is_empty() {
                process_interpolatable_attribute!(&vertex_keys, p_arg.vertex_array, ssab::ssab::ss::format::PartAnimationDataVertexT, |k: &sspj::ssattribute::SsAttributeVertexKey| {
                    let mut value = ssab::ssab::ss::format::PartAttributeVertexT::default();
                    let mut c = ssab::ssab::ss::format::Vec2T::default();
                    if let Some(p) = k.value.lt.as_ref() {
                        c.v1 = p.x;
                        c.v2 = p.y;
                    }
                    value.lt = Some(Box::new(c));
                    let mut c = ssab::ssab::ss::format::Vec2T::default();
                    if let Some(p) = k.value.rt.as_ref() {
                        c.v1 = p.x;
                        c.v2 = p.y;
                    }
                    value.rt = Some(Box::new(c));
                    let mut c = ssab::ssab::ss::format::Vec2T::default();
                    if let Some(p) = k.value.lb.as_ref() {
                        c.v1 = p.x;
                        c.v2 = p.y;
                    }
                    value.lb = Some(Box::new(c));
                    let mut c = ssab::ssab::ss::format::Vec2T::default();
                    if let Some(p) = k.value.rb.as_ref() {
                        c.v1 = p.x;
                        c.v2 = p.y;
                    }
                    value.rb = Some(Box::new(c));
                    Box::new(value)
                });
            }
        }
        sspj::ssattribute::SsAttributeKind::Deform(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.deform_array, ssab::ssab::ss::format::PartAnimationDataDeformT, |k: &sspj::ssattribute::SsAttributeDeformKey| {
                let mut value = ssab::ssab::ss::format::PartAttributeDeformT::default();
                value.coord_x = Some(k.value.vchg.iter().map(|p| p.x).collect());
                value.coord_y = Some(k.value.vchg.iter().map(|p| p.y).collect());
                value.index = Some(k.value.vchg.iter().map(|p| p.x_index as i16).collect());
                Box::new(value)
            });
        }
        sspj::ssattribute::SsAttributeKind::Texchange(v) => {
            process_simple_attribute!(&v.keys, p_arg.texture_array, ssab::ssab::ss::format::PartAnimationDataUIntValueT, |k: &sspj::ssattribute::SsAttributeTexchangeKey| { crate::utils::fnv1a_hash_texture_name(&k.value.texture_name) });
        }
        _ => {}
    });
}

fn optimize_part_animation_data_value_changing_range(interpolation: &ssab::ssab::ss::format::InterpolationType) -> bool {
    match *interpolation {
        ssab::ssab::ss::format::InterpolationType::None => false,
        ssab::ssab::ss::format::InterpolationType::Linear => false,
        ssab::ssab::ss::format::InterpolationType::Hermite => true,
        ssab::ssab::ss::format::InterpolationType::Bezier => true,
        ssab::ssab::ss::format::InterpolationType::Acceleration => false,
        ssab::ssab::ss::format::InterpolationType::Deceleration => false,
        ssab::ssab::ss::format::InterpolationType::EaseIn => false,
        ssab::ssab::ss::format::InterpolationType::EaseOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseInOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseExponentialIn => false,
        ssab::ssab::ss::format::InterpolationType::EaseExponentialOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseExponentialInOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseSineIn => false,
        ssab::ssab::ss::format::InterpolationType::EaseSineOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseSineInOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseElasticIn => false,
        ssab::ssab::ss::format::InterpolationType::EaseElasticOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseElasticInOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseBounceIn => false,
        ssab::ssab::ss::format::InterpolationType::EaseBounceOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseBounceInOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseBackIn => false,
        ssab::ssab::ss::format::InterpolationType::EaseBackOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseBackInOut => false,
        _ => false,
    }
}

#[cfg(false)]
macro_rules! impl_optimize_interpolatable_attribute {
    ($fn_name:ident, $type_t:ty) => {
        fn $fn_name(attribute: &[$type_t]) -> Option<Vec<$type_t>> {
            if attribute.is_empty() {
                return None;
            }
            let mut optimized = Vec::with_capacity(attribute.len());

            for i in 0..attribute.len() {
                let current = &attribute[i];
                let is_first = i == 0;
                let is_last = i == attribute.len() - 1;

                if is_first || is_last {
                    let mut key = current.clone();
                    if is_last {
                        key.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                    }
                    optimized.push(key);
                    continue;
                }

                let prev = &attribute[i - 1];
                let next = &attribute[i + 1];

                // 冗長なキーの判定: 前後と同じ値で、補間も発生しない場合
                // MEMO: 値の変化がない区間でも、補間タイプによっては曲線を描く可能性があるため interpolation もチェックする
                let is_redundant = current.value == prev.value && current.value == next.value && !optimize_part_animation_data_value_changing_range(&prev.interpolation) && !optimize_part_animation_data_value_changing_range(&current.interpolation);

                if !is_redundant {
                    optimized.push(current.clone());
                }
            }
            if optimized.is_empty() { None } else { Some(optimized) }
        }
    };
}
macro_rules! impl_optimize_interpolatable_attribute {
    ($fn_name:ident, $type_t:ty) => {
        fn $fn_name(attribute: &[$type_t]) -> Option<Vec<$type_t>> {
            if attribute.is_empty() {
                return None;
            }
            let mut optimized = Vec::with_capacity(attribute.len());
            optimized.push(attribute[0].clone()); // First key always exist.

            for i in 1..attribute.len() {
                let current = &attribute[i];
                if let Some(prev) = optimized.last_mut() {
                    let is_last = i == attribute.len() - 1;
                    let mut is_lost_interpolation_previous = false;

                    let mut is_redundant = false;
                    if current.value == prev.value && !optimize_part_animation_data_value_changing_range(&prev.interpolation) {
                        is_lost_interpolation_previous = true;

                        if is_last || current.interpolation == ssab::ssab::ss::format::InterpolationType::None {
                            is_redundant = true;
                        } else {
                            if !is_last {
                                let next = &attribute[i + 1];
                                if current.value == next.value && !optimize_part_animation_data_value_changing_range(&current.interpolation) {
                                    is_redundant = true;
                                }
                            }
                        }
                    }

                    if is_lost_interpolation_previous {
                        prev.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                    }
                    if !is_redundant {
                        let mut key = current.clone();
                        if is_last {
                            key.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                        }
                        optimized.push(key);
                    }
                }
            }
            if optimized.is_empty() { None } else { Some(optimized) }
        }
    };
}

macro_rules! impl_optimize_simple_attribute {
    ($fn_name:ident, $type_t:ty) => {
        fn $fn_name(attribute: &[$type_t]) -> Option<Vec<$type_t>> {
            if attribute.is_empty() {
                return None;
            }
            let mut optimized = Vec::with_capacity(attribute.len());
            for i in 0..attribute.len() {
                let current = &attribute[i];
                if i == 0 || current.value != attribute[i - 1].value {
                    optimized.push(current.clone());
                }
            }
            if optimized.is_empty() { None } else { Some(optimized) }
        }
    };
}

impl_optimize_interpolatable_attribute!(optimize_part_animation_data_float, ssab::ssab::ss::format::PartAnimationDataFloatValueT);
impl_optimize_interpolatable_attribute!(optimize_part_animation_data_int, ssab::ssab::ss::format::PartAnimationDataIntValueT);
impl_optimize_simple_attribute!(optimize_part_animation_data_bool, ssab::ssab::ss::format::PartAnimationDataBoolValueT);
impl_optimize_simple_attribute!(optimize_part_animation_data_cell, ssab::ssab::ss::format::PartAnimationDataCellValueT);
impl_optimize_interpolatable_attribute!(optimize_part_animation_data_partcolor, ssab::ssab::ss::format::PartAnimationDataPartColorT);
impl_optimize_interpolatable_attribute!(optimize_part_animation_data_shader, ssab::ssab::ss::format::PartAnimationDataShaderT);
impl_optimize_interpolatable_attribute!(optimize_part_animation_data_vertex, ssab::ssab::ss::format::PartAnimationDataVertexT);
impl_optimize_interpolatable_attribute!(optimize_part_animation_data_skew, ssab::ssab::ss::format::PartAnimationDataVec2T);
impl_optimize_interpolatable_attribute!(optimize_part_animation_data_deform, ssab::ssab::ss::format::PartAnimationDataDeformT);
impl_optimize_simple_attribute!(optimize_part_animation_data_texture, ssab::ssab::ss::format::PartAnimationDataUIntValueT);

fn modify_part_animation_data(p_args: &mut ssab::ssab::ss::format::PartAnimationDataT, _part_type: &sspj::sstypes::SsPartType, _anime_data: &sspj::ssae::SsPartAnime) {
    // Key-data optimization per attribute
    p_args.cell_array = p_args.cell_array.as_ref().and_then(|v| optimize_part_animation_data_cell(v));
    p_args.position_x_array = p_args.position_x_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.position_y_array = p_args.position_y_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.position_z_array = p_args.position_z_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.rotation_x_array = p_args.rotation_x_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.rotation_y_array = p_args.rotation_y_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.rotation_z_array = p_args.rotation_z_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.scale_x_array = p_args.scale_x_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.scale_y_array = p_args.scale_y_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.local_scale_x_array = p_args.local_scale_x_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.local_scale_y_array = p_args.local_scale_y_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.alpha_array = p_args.alpha_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.local_alpha_array = p_args.local_alpha_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.priority_array = p_args.priority_array.as_ref().and_then(|v| optimize_part_animation_data_int(v));
    p_args.flip_h_array = p_args.flip_h_array.as_ref().and_then(|v| optimize_part_animation_data_bool(v));
    p_args.flip_v_array = p_args.flip_v_array.as_ref().and_then(|v| optimize_part_animation_data_bool(v));
    p_args.hide_array = p_args.hide_array.as_ref().and_then(|v| optimize_part_animation_data_bool(v));
    p_args.part_color_array = p_args.part_color_array.as_ref().and_then(|v| optimize_part_animation_data_partcolor(v));
    p_args.shader_array = p_args.shader_array.as_ref().and_then(|v| optimize_part_animation_data_shader(v));
    p_args.vertex_array = p_args.vertex_array.as_ref().and_then(|v| optimize_part_animation_data_vertex(v));
    p_args.pivot_x_array = p_args.pivot_x_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.pivot_y_array = p_args.pivot_y_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.anchor_x_array = p_args.anchor_x_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.anchor_y_array = p_args.anchor_y_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.size_x_array = p_args.size_x_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.size_y_array = p_args.size_y_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.img_flip_h_array = p_args.img_flip_h_array.as_ref().and_then(|v| optimize_part_animation_data_bool(v));
    p_args.img_flip_v_array = p_args.img_flip_v_array.as_ref().and_then(|v| optimize_part_animation_data_bool(v));
    p_args.uv_translation_x_array = p_args.uv_translation_x_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.uv_translation_y_array = p_args.uv_translation_y_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.uv_rotation_z_array = p_args.uv_rotation_z_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.uv_scale_x_array = p_args.uv_scale_x_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.uv_scale_y_array = p_args.uv_scale_y_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.bounding_radius_array = p_args.bounding_radius_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.mask_array = p_args.mask_array.as_ref().and_then(|v| optimize_part_animation_data_float(v));
    p_args.skew_array = p_args.skew_array.as_ref().and_then(|v| optimize_part_animation_data_skew(v));
    p_args.deform_array = p_args.deform_array.as_ref().and_then(|v| optimize_part_animation_data_deform(v));
    p_args.texture_array = p_args.texture_array.as_ref().and_then(|v| optimize_part_animation_data_texture(v));
}

fn convert_parts_animation_data(anime_data: &sspj::ssae::SsAnimation, setup_animation: &sspj::ssae::SsAnimation, parts_map: &HashMap<String, &sspj::ssae::SsPart>, cellmaps: &CellmapResolver) -> Vec<ssab::ssab::ss::format::PartAnimationDataT> {
    let mut parts_animation_data_t = Vec::<ssab::ssab::ss::format::PartAnimationDataT>::new();
    parts_animation_data_t.resize(parts_map.len(), ssab::ssab::ss::format::PartAnimationDataT::default());

    for part_anime in &setup_animation.part_animes {
        let Some(p) = parts_map.get(&part_anime.part_name) else {
            // not found part (an illegal case)
            continue;
        };

        let index_part = p.array_index as usize;
        convert_part_animation_data(&mut parts_animation_data_t[index_part], part_anime, cellmaps, true, &p.part_type);
    }

    for part_anime in &anime_data.part_animes {
        let Some(p) = parts_map.get(&part_anime.part_name) else {
            // not found part (an illegal case)
            continue;
        };

        let index_part = p.array_index as usize;
        convert_part_animation_data(&mut parts_animation_data_t[index_part], part_anime, cellmaps, false, &p.part_type);
        modify_part_animation_data(&mut parts_animation_data_t[index_part], &p.part_type, part_anime);
    }

    for part in parts_map.values() {
        let index_part = part.array_index as usize;
        apply_part_type_defaults(&mut parts_animation_data_t[index_part], part);
    }

    bake_hide_inheritance(&mut parts_animation_data_t, parts_map);

    parts_animation_data_t
}

/// Bakes SpriteStudio's per-part HIDE inheritance into each part's `hide_array`.
///
/// In SpriteStudio the `hide` attribute does NOT unconditionally cascade to children:
/// whether a part follows its parent's hidden state is controlled per-part by
/// the `ineheritRates/HIDE` flag (`SsPart.inherit_rates.hide`). The runtime
/// treats `hide` as a pure per-part attribute and never propagates it, so any
/// inheritance is resolved here at convert time — a part that inherits HIDE has
/// its parent's already-resolved hidden state OR-ed into its own `hide_array`.
///
/// Parts are visited in `array_index` order; SS stores a parent before its
/// children, so a parent's effective hide is always resolved first, which makes
/// the inheritance transitive (a grandparent reaches a grandchild only when
/// every link in the chain has the flag set).
fn bake_hide_inheritance(parts_animation_data_t: &mut [ssab::ssab::ss::format::PartAnimationDataT], parts_map: &HashMap<String, &sspj::ssae::SsPart>) {
    let num_parts = parts_animation_data_t.len();

    // Index parts by `array_index` for parent lookups.
    let mut parts_by_index: Vec<Option<&sspj::ssae::SsPart>> = vec![None; num_parts];
    for part in parts_map.values() {
        let idx = part.array_index as usize;
        if idx < num_parts {
            parts_by_index[idx] = Some(part);
        }
    }

    // effective[i] = minimal `(frame, hidden)` step keyframes for part i after
    // resolving inheritance. Always non-empty and starting at frame 0.
    let mut effective: Vec<Vec<(u16, bool)>> = Vec::with_capacity(num_parts);

    for i in 0..num_parts {
        let own = hide_steps(parts_animation_data_t[i].hide_array.as_deref());

        let Some(part) = parts_by_index[i] else {
            effective.push(own);
            continue;
        };

        let parent_idx = part.parent_index;
        let inherits = part.inherit_rates.hide && parent_idx >= 0 && (parent_idx as usize) < i;
        if !inherits {
            effective.push(own);
            continue;
        }

        let eff = or_hide_steps(&own, &effective[parent_idx as usize]);
        if eff != own {
            parts_animation_data_t[i].hide_array = Some(
                eff.iter()
                    .map(|&(index, value)| {
                        let mut k = ssab::ssab::ss::format::PartAnimationDataBoolValueT::default();
                        k.index = index;
                        k.value = value;
                        k
                    })
                    .collect(),
            );
        }
        effective.push(eff);
    }
}

/// Extracts a part's own hide attribute as minimal `(frame, hidden)` step
/// keyframes. A missing `hide_array` means "never hidden"; the result always
/// begins at frame 0 so it can be sampled at any frame >= 0.
fn hide_steps(hide_array: Option<&[ssab::ssab::ss::format::PartAnimationDataBoolValueT]>) -> Vec<(u16, bool)> {
    match hide_array {
        Some(keys) if !keys.is_empty() => keys.iter().map(|k| (k.index, k.value)).collect(),
        _ => vec![(0, false)],
    }
}

/// Value of a `(frame, hidden)` step function at `frame`. `steps` is sorted
/// ascending and begins at frame 0.
fn hide_value_at(steps: &[(u16, bool)], frame: u16) -> bool {
    let mut value = steps[0].1;
    for &(idx, val) in steps {
        if idx <= frame {
            value = val;
        } else {
            break;
        }
    }
    value
}

/// OR of two `(frame, hidden)` step functions, returned as minimal keyframes
/// (frame 0 plus one keyframe per value change — matching the layout produced
/// by `optimize_part_animation_data_bool`).
fn or_hide_steps(a: &[(u16, bool)], b: &[(u16, bool)]) -> Vec<(u16, bool)> {
    let mut frames: Vec<u16> = a.iter().map(|&(i, _)| i).chain(b.iter().map(|&(i, _)| i)).collect();
    frames.sort_unstable();
    frames.dedup();

    let mut result: Vec<(u16, bool)> = Vec::new();
    for frame in frames {
        let hidden = hide_value_at(a, frame) || hide_value_at(b, frame);
        if result.last().map(|&(_, v)| v) != Some(hidden) {
            result.push((frame, hidden));
        }
    }
    result
}

fn apply_part_type_defaults(p_args: &mut ssab::ssab::ss::format::PartAnimationDataT, part: &sspj::ssae::SsPart) {
    match &part.part_type {
        sspj::sstypes::SsPartType::Shape => {
            const SHAPE_DEFAULT_SIZE: f32 = 64.0;
            if p_args.size_x_array.as_ref().is_none_or(|v| v.is_empty()) {
                p_args.size_x_array = Some(vec![float_value_constant_key(SHAPE_DEFAULT_SIZE)]);
            }
            if p_args.size_y_array.as_ref().is_none_or(|v| v.is_empty()) {
                p_args.size_y_array = Some(vec![float_value_constant_key(SHAPE_DEFAULT_SIZE)]);
            }
        }
        sspj::sstypes::SsPartType::Armature => {
            // A bone's rest pose is authored in the ssae `bonePosition`/`boneRotation`
            // tags, not as setup keyframes. Reflect them as the bone's default
            // transform so the runtime poses the bone correctly on animations that
            // do not key it; an animation that does key the bone overrides these.
            if p_args.position_x_array.as_ref().is_none_or(|v| v.is_empty()) {
                p_args.position_x_array = Some(vec![float_value_constant_key(part.bone_position.x)]);
            }
            if p_args.position_y_array.as_ref().is_none_or(|v| v.is_empty()) {
                p_args.position_y_array = Some(vec![float_value_constant_key(part.bone_position.y)]);
            }
            if p_args.rotation_z_array.as_ref().is_none_or(|v| v.is_empty()) {
                p_args.rotation_z_array = Some(vec![float_value_constant_key(part.bone_rotation)]);
            }
        }
        _ => {}
    }
}

fn float_value_constant_key(value: f32) -> ssab::ssab::ss::format::PartAnimationDataFloatValueT {
    let mut k = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
    k.index = 0;
    k.interpolation = ssab::ssab::ss::format::InterpolationType::None;
    k.value = value;
    k
}

// PartColor mediation. The XML carries (rgba, rate) per corner plus a per-attribute
// (blend_type, target). Older SpriteStudio runtimes encoded the same compatibility
// quirk that lives here: under Mix+Whole, `rate` and `rgba.a` swap roles. The same
// pair carries two unrelated semantics (a "blend weight" and a "final-alpha source"),
// and which XML field carries which depends on (blend_type, target).
//
// The `.ssab` schema reuses the same two slots after mediation:
//   - rgba.a (u8)  ← rateAlpha: source for the final-alpha multiplier (later
//                     multiplied by the part's overall alpha in the player).
//   - rate   (f32) ← colorA:    the PartColor blend weight (the renderer's
//                     ColorOverlay.a equivalent).
// `r/g/b` pass through unchanged. The `target` distinction is consumed here when
// the per-attribute is expanded into four corner entries; downstream code never
// sees it again.
//
// Reference: SS6PlayerForUnity/Assets/SpriteStudio6/Editor/Import/SSAE.cs
//   `ParseAnimationAttributePartsColor` (the C# converter applies the same table).
//   The C++ SDK applies an equivalent split at draw time in
//   Common/Drawer/ssplayer_render_gl.cpp, using a fixed-function texture combiner
//   constant instead of vertex data; we follow the shader-friendly Unity encoding.
fn build_part_color_key_value_color_t(
    src: &sspj::ssattribute::SsAttributePartsColorKeyValueColor,
    blend_type: &sspj::sstypes::SsBlendType,
    target: &sspj::sstypes::SsColorBlendTarget,
    part_type: &sspj::sstypes::SsPartType,
) -> ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT {
    let mut color = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
    let mut rgba = ssab::ssab::ss::format::U8ColorT::default();
    rgba.r = src.rgba.r;
    rgba.g = src.rgba.g;
    rgba.b = src.rgba.b;

    let (rate_alpha_u8, color_a_f32) = match (blend_type, target) {
        // Mix + Whole: the XML `rate` carries the blend weight, `rgba.a` the final alpha.
        (sspj::sstypes::SsBlendType::Mix, sspj::sstypes::SsColorBlendTarget::Whole) => (src.rgba.a, src.rate),
        // Mix + Vertex: per-corner `rgba.a` carries the blend weight; XML `rate`
        // is documented to be ~1.0 in this mode and feeds the final alpha.
        (sspj::sstypes::SsBlendType::Mix, sspj::sstypes::SsColorBlendTarget::Vertex) => {
            let rate_u8 = (src.rate * 255.0).clamp(0.0, 255.0) as u8;
            (rate_u8, (src.rgba.a as f32) / 255.0)
        }
        // All other blend types (Add/Sub/Mul and the newer additions): the blend
        // weight is implicitly 1.0; XML `rate` is unused; `rgba.a` is the final alpha.
        _ => (src.rgba.a, 1.0),
    };

    // Shape-specific override on top of the general mediation: Shape parts
    // don't apply the PartColor compositing formula — the per-vertex color is
    // just pc.rgba, and the GPU framebuffer blend (part.alpha_blend_type) does
    // the rest. The per-corner alpha rule, however, is special:
    //   - PartColor blend_type = Mix : vertex.a is forced to 1.0
    //   - PartColor blend_type ≠ Mix : vertex.a = pc.rgba.a (opacity, multiplied
    //                                   with part_alpha downstream)
    // The player forces (rate=1, blend_idx=Mix) per vertex so the shader formula
    // collapses to lerp(white, pc.rgb, 1.0) = pc.rgb (and the original PartColor
    // blend_type stays in ssab for inspection). To make Mix shape's vertex.a be
    // 1.0, we override rgba.a here.
    let rate_alpha_u8 = if matches!(part_type, sspj::sstypes::SsPartType::Shape) && matches!(blend_type, sspj::sstypes::SsBlendType::Mix) { 255 } else { rate_alpha_u8 };

    rgba.a = rate_alpha_u8;
    color.rgba = Some(Box::new(rgba));
    color.rate = color_a_f32;
    color
}

// SpriteStudio stores PartColor with a `target` of either Whole (single overall color in `color`)
// or Vertex (four per-corner colors). The `.ssab` schema only carries four corners,
// so Whole is expanded into four identical corner entries here. See
// `build_part_color_key_value_color_t` for the rate/alpha mediation applied per corner.
fn build_part_attribute_part_color_t(src: &sspj::ssattribute::SsAttributePartsColorKeyValue, part_type: &sspj::sstypes::SsPartType) -> ssab::ssab::ss::format::PartAttributePartColorT {
    let mut value = ssab::ssab::ss::format::PartAttributePartColorT::default();
    value.blend_type = convert_alpha_blend_type(&src.blend_type);
    match src.target {
        sspj::sstypes::SsColorBlendTarget::Whole => {
            let c = build_part_color_key_value_color_t(&src.color, &src.blend_type, &src.target, part_type);
            value.lt = Some(Box::new(c.clone()));
            value.rt = Some(Box::new(c.clone()));
            value.lb = Some(Box::new(c.clone()));
            value.rb = Some(Box::new(c));
        }
        sspj::sstypes::SsColorBlendTarget::Vertex => {
            value.lt = Some(Box::new(build_part_color_key_value_color_t(&src.lt, &src.blend_type, &src.target, part_type)));
            value.rt = Some(Box::new(build_part_color_key_value_color_t(&src.rt, &src.blend_type, &src.target, part_type)));
            value.lb = Some(Box::new(build_part_color_key_value_color_t(&src.lb, &src.blend_type, &src.target, part_type)));
            value.rb = Some(Box::new(build_part_color_key_value_color_t(&src.rb, &src.blend_type, &src.target, part_type)));
        }
    }
    value
}

fn convert_alpha_blend_type(blend_type: &sspj::sstypes::SsBlendType) -> ssab::ssab::ss::format::BlendType {
    match blend_type {
        sspj::sstypes::SsBlendType::Mix => ssab::ssab::ss::format::BlendType::Mix,
        sspj::sstypes::SsBlendType::Mul => ssab::ssab::ss::format::BlendType::Mul,
        sspj::sstypes::SsBlendType::Add => ssab::ssab::ss::format::BlendType::Add,
        sspj::sstypes::SsBlendType::Sub => ssab::ssab::ss::format::BlendType::Sub,
        sspj::sstypes::SsBlendType::Mulalpha => ssab::ssab::ss::format::BlendType::Mulalpha,
        sspj::sstypes::SsBlendType::Screen => ssab::ssab::ss::format::BlendType::Screen,
        sspj::sstypes::SsBlendType::Exclusion => ssab::ssab::ss::format::BlendType::Exclusion,
        sspj::sstypes::SsBlendType::Invert => ssab::ssab::ss::format::BlendType::Invert,
        sspj::sstypes::SsBlendType::Mul2 => ssab::ssab::ss::format::BlendType::Mul2,
        sspj::sstypes::SsBlendType::Div2 => ssab::ssab::ss::format::BlendType::Div2,
        sspj::sstypes::SsBlendType::Screen2 => ssab::ssab::ss::format::BlendType::Screen2,
        sspj::sstypes::SsBlendType::Overlay2 => ssab::ssab::ss::format::BlendType::Overlay2,
    }
}

pub fn convert_part_list(anime_pack: &sspj::ssae::SsAnimePack) -> Vec<ssab::ssab::ss::format::PartDataT> {
    let bone_index_to_part_index: std::collections::HashMap<i32, u16> = anime_pack
        .model
        .bone_list
        .iter()
        .filter_map(|(part_name, &bone_idx)| {
            let part_idx = anime_pack.model.part_list.iter().position(|p| &p.name == part_name)?;
            Some((bone_idx, part_idx as u16))
        })
        .collect();

    // Running counter into the compact, mesh-part-ordered `mesh_list`.
    let mut mesh_part_index = 0usize;
    anime_pack
        .model
        .part_list
        .iter()
        .map(|part| {
            let mut part_data_t = ssab::ssab::ss::format::PartDataT::default();
            part_data_t.name = part.name.to_string();
            part_data_t.parent_index = part.parent_index as i16;
            part_data_t.bounds_type = match &part.bounds_type {
                sspj::sstypes::SsBoundsType::None => ssab::ssab::ss::format::BoundsType::None,
                sspj::sstypes::SsBoundsType::Quad => ssab::ssab::ss::format::BoundsType::Quad,
                sspj::sstypes::SsBoundsType::Aabb => ssab::ssab::ss::format::BoundsType::Aabb,
                sspj::sstypes::SsBoundsType::Circle => ssab::ssab::ss::format::BoundsType::Circle,
                sspj::sstypes::SsBoundsType::CircleSmin => ssab::ssab::ss::format::BoundsType::CircleSmin,
                sspj::sstypes::SsBoundsType::CircleSmax => ssab::ssab::ss::format::BoundsType::CircleSmax,
            };
            part_data_t.blend_type = convert_alpha_blend_type(&part.alpha_blend_type);
            part_data_t.mask_influence = part.mask_influence;
            part_data_t.mask_write = part.write_mask;
            part_data_t.visible_inside_mask = part.visible_inside_mask;
            // Fold the per-type pure-mask flag from the source .ssae into the unified draw_as_mask.
            part_data_t.draw_as_mask = match &part.part_type {
                sspj::sstypes::SsPartType::Shape => part.shape_mask,
                sspj::sstypes::SsPartType::Text => part.text_mask,
                sspj::sstypes::SsPartType::Nines => part.nines_mask,
                _ => false,
            };
            part_data_t.output_influence = part.output_influence;

            // `mesh_list` is a compact list ordered by Mesh part (one SsMeshBind
            // per Mesh part — it carries no part identity), so index it by a
            // running mesh-part counter, not the global part index.
            part_data_t.mesh_binding = matches!(&part.part_type, sspj::sstypes::SsPartType::Mesh)
                .then(|| {
                    let i = mesh_part_index;
                    mesh_part_index += 1;
                    anime_pack.model.mesh_list.get(i)
                })
                .flatten()
                .filter(|mesh_bind| !mesh_bind.mesh_vertices_bind_array.is_empty())
                .map(|mesh_bind| {
                    let num_vertices = mesh_bind.mesh_vertices_bind_array.len();
                    let total_influences: usize = mesh_bind.mesh_vertices_bind_array.iter().map(|b| b.bind_bone_num.max(0) as usize).sum();
                    let mut vertex_influence_offset: Vec<u32> = Vec::with_capacity(num_vertices + 1);
                    let mut influence_bone: Vec<u16> = Vec::with_capacity(total_influences);
                    let mut influence_weight: Vec<f32> = Vec::with_capacity(total_influences);
                    let mut influence_coord_x: Vec<f32> = Vec::with_capacity(total_influences);
                    let mut influence_coord_y: Vec<f32> = Vec::with_capacity(total_influences);
                    vertex_influence_offset.push(0);
                    for bind_info in &mesh_bind.mesh_vertices_bind_array {
                        for i in 0..bind_info.bind_bone_num.max(0) as usize {
                            influence_bone.push(bone_index_to_part_index.get(&bind_info.bone_index[i]).copied().unwrap_or(0));
                            influence_weight.push(bind_info.weight[i] as f32 / 100.0);
                            influence_coord_x.push(bind_info.offset[i].x);
                            influence_coord_y.push(bind_info.offset[i].y);
                        }
                        vertex_influence_offset.push(influence_bone.len() as u32);
                    }
                    let mut binding = ssab::ssab::ss::format::PartMeshBindingT::default();
                    binding.vertex_influence_offset = Some(vertex_influence_offset);
                    binding.influence_bone = Some(influence_bone);
                    binding.influence_weight = Some(influence_weight);
                    binding.influence_coord_x = Some(influence_coord_x);
                    binding.influence_coord_y = Some(influence_coord_y);
                    Box::new(binding)
                });

            part_data_t.part_type = match &part.part_type {
                sspj::sstypes::SsPartType::Null => ssab::ssab::ss::format::PartTypeT::PartTypeNull(Box::default()),
                sspj::sstypes::SsPartType::Normal => ssab::ssab::ss::format::PartTypeT::PartTypeNormal(Box::default()),
                sspj::sstypes::SsPartType::Shape => {
                    let mut part_type = ssab::ssab::ss::format::PartTypeShapeT::default();
                    part_type.shape_type = match &part.shape_type {
                        sspj::sstypes::SsShapeType::Unknown => ssab::ssab::ss::format::ShapeType::Unknown,
                        sspj::sstypes::SsShapeType::Triangle => ssab::ssab::ss::format::ShapeType::Triangle,
                        sspj::sstypes::SsShapeType::Rectangle => ssab::ssab::ss::format::ShapeType::Rectangle,
                        sspj::sstypes::SsShapeType::Arrow => ssab::ssab::ss::format::ShapeType::Arrow,
                        sspj::sstypes::SsShapeType::Star => ssab::ssab::ss::format::ShapeType::Star,
                    };
                    ssab::ssab::ss::format::PartTypeT::PartTypeShape(Box::new(part_type))
                }
                sspj::sstypes::SsPartType::Text => {
                    let mut part_type = ssab::ssab::ss::format::PartTypeTextT::default();
                    part_type.text = Some(part.text.to_string());
                    part_type.text_bitmap = part.text_bitmap;
                    part_type.text_family = if part.text_family.is_empty() { None } else { Some(part.text_family.to_string()) };
                    part_type.text_char_map = if part.text_char_map.is_empty() { None } else { Some(part.text_char_map.to_string()) };
                    part_type.text_size = part.text_size;
                    part_type.text_space = part.text_space;
                    part_type.text_smooth = part.text_smooth;
                    part_type.text_width = part.text_width;
                    part_type.text_height = part.text_height;
                    part_type.e_anchor = match part.e_anchor {
                        0 => ssab::ssab::ss::format::TextAnchor::LT,
                        1 => ssab::ssab::ss::format::TextAnchor::LC,
                        2 => ssab::ssab::ss::format::TextAnchor::LB,
                        16 => ssab::ssab::ss::format::TextAnchor::CT,
                        17 => ssab::ssab::ss::format::TextAnchor::CC,
                        18 => ssab::ssab::ss::format::TextAnchor::CB,
                        32 => ssab::ssab::ss::format::TextAnchor::RT,
                        33 => ssab::ssab::ss::format::TextAnchor::RC,
                        34 => ssab::ssab::ss::format::TextAnchor::RB,
                        255 => ssab::ssab::ss::format::TextAnchor::NO,
                        _ => ssab::ssab::ss::format::TextAnchor::LT,
                    };

                    ssab::ssab::ss::format::PartTypeT::PartTypeText(Box::new(part_type))
                }
                sspj::sstypes::SsPartType::Nines => {
                    let mut part_type = ssab::ssab::ss::format::PartTypeNinesT::default();
                    part_type.nines_margin_l = part.nines_margin_l;
                    part_type.nines_margin_r = part.nines_margin_r;
                    part_type.nines_margin_t = part.nines_margin_t;
                    part_type.nines_margin_b = part.nines_margin_b;
                    part_type.nines_fill_mode = part.nines_fill_mode;

                    ssab::ssab::ss::format::PartTypeT::PartTypeNines(Box::new(part_type))
                }
                sspj::sstypes::SsPartType::Instance => {
                    let mut part_type = ssab::ssab::ss::format::PartTypeInstanceT::default();
                    part_type.ref_anime_pack = part.ref_anime_pack.clone();
                    part_type.ref_anime_pack_hash = crate::utils::fnv1a_hash_str(&part.ref_anime_pack);
                    part_type.ref_anime_name = part.ref_anime.clone();
                    part_type.ref_anime_name_hash = crate::utils::fnv1a_hash_str(&part.ref_anime);
                    ssab::ssab::ss::format::PartTypeT::PartTypeInstance(Box::new(part_type))
                }
                sspj::sstypes::SsPartType::Armature => ssab::ssab::ss::format::PartTypeT::PartTypeArmature(Box::default()),
                sspj::sstypes::SsPartType::Effect => {
                    let mut part_type = ssab::ssab::ss::format::PartTypeEffectT::default();
                    part_type.ref_effect_name_hash = crate::utils::fnv1a_hash_str(&part.ref_effect_name);
                    ssab::ssab::ss::format::PartTypeT::PartTypeEffect(Box::new(part_type))
                }
                sspj::sstypes::SsPartType::Mesh => ssab::ssab::ss::format::PartTypeT::PartTypeMesh(Box::default()),
                sspj::sstypes::SsPartType::Movenode => ssab::ssab::ss::format::PartTypeT::PartTypeMovenode(Box::default()),
                sspj::sstypes::SsPartType::Constraint => ssab::ssab::ss::format::PartTypeT::PartTypeConstraint(Box::default()),
                sspj::sstypes::SsPartType::Mask => ssab::ssab::ss::format::PartTypeT::PartTypeMask(Box::default()),
                sspj::sstypes::SsPartType::Joint => ssab::ssab::ss::format::PartTypeT::PartTypeJoint(Box::default()),
                sspj::sstypes::SsPartType::Bonepoint => ssab::ssab::ss::format::PartTypeT::PartTypeBonepoint(Box::default()),
                sspj::sstypes::SsPartType::TransformConstraint => ssab::ssab::ss::format::PartTypeT::PartTypeTransformConstraint(Box::default()),
                sspj::sstypes::SsPartType::Camera => ssab::ssab::ss::format::PartTypeT::PartTypeCamera(Box::default()),
                sspj::sstypes::SsPartType::Audio => ssab::ssab::ss::format::PartTypeT::PartTypeAudio(Box::default()),
            };

            part_data_t
        })
        .collect::<Vec<_>>()
}

pub fn convert_animation_data_list(anime_pack: &sspj::ssae::SsAnimePack, parts_map: &HashMap<String, &sspj::ssae::SsPart>, sound_list_id_to_name_hash: &HashMap<i32, u32>, cellmaps: &CellmapResolver) -> Vec<ssab::ssab::ss::format::AnimationDataT> {
    let Some(setup_animation) = anime_pack.anime_list.iter().find(|anime_data| anime_data.is_setup) else {
        // not found setup animation
        return vec![];
    };

    anime_pack
        .anime_list
        .iter()
        .filter_map(|anime_data| {
            if anime_data == setup_animation {
                return None;
            }
            Some(convert_animedatat(anime_data, setup_animation, parts_map, sound_list_id_to_name_hash, cellmaps))
        })
        .collect::<Vec<ssab::ssab::ss::format::AnimationDataT>>()
}

/// Resolves a cellmap `map_id` to its name hash.
///
/// `map_id` indexes the same `cellmap_names` list that produced it, so a value
/// outside it means the sspj is malformed. Every lookup happens inside a
/// `for_each` closure several calls down, which has no way to return an error,
/// so the first offending id is recorded here and [`Self::check`] reads it once
/// the walk is over. Without that the index panicked, and `panic = "abort"`
/// turned a malformed pack into a dead host process.
pub struct CellmapResolver<'a> {
    names: &'a [String],
    out_of_range: core::cell::Cell<Option<i32>>,
}

impl<'a> CellmapResolver<'a> {
    pub fn new(names: &'a [String]) -> Self {
        Self { names, out_of_range: core::cell::Cell::new(None) }
    }

    /// Negative `map_id` is the documented "no map" convention in sspj (e.g. an
    /// unset shader map slot) and resolves to `0`, same as before.
    fn name_hash(&self, map_id: i32) -> u32 {
        if map_id < 0 {
            return 0;
        }
        match self.names.get(map_id as usize) {
            Some(file_name) => crate::utils::fnv1a_hash_str(crate::utils::cellmap_basename(file_name)),
            None => {
                if self.out_of_range.get().is_none() {
                    self.out_of_range.set(Some(map_id));
                }
                0
            }
        }
    }

    /// The error for whatever [`Self::name_hash`] could not resolve, or `Ok`.
    pub fn check(&self, pack_name: &str) -> Result<(), crate::ConverterError> {
        match self.out_of_range.get() {
            Some(map_id) => Err(crate::ConverterError::ConversionFailed(format!("{}: a cell references cellmap {}, outside 0..{}", pack_name, map_id, self.names.len()))),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod cellmap_resolver_tests {
    use super::*;

    fn names() -> Vec<String> {
        vec!["ground.ssce".to_string(), "sky.ssce".to_string()]
    }

    #[test]
    fn an_in_range_map_id_resolves_and_passes() {
        let names = names();
        let r = CellmapResolver::new(&names);
        assert_eq!(r.name_hash(0), crate::utils::fnv1a_hash_str("ground"));
        assert_eq!(r.name_hash(1), crate::utils::fnv1a_hash_str("sky"));
        r.check("pack").expect("both ids are in range");
    }

    #[test]
    fn a_negative_map_id_is_the_no_map_convention() {
        let names = names();
        let r = CellmapResolver::new(&names);
        assert_eq!(r.name_hash(-1), 0);
        r.check("pack").expect("negative is sspj's documented \"no map\"");
    }

    #[test]
    fn an_out_of_range_map_id_is_an_error() {
        let names = names();
        let r = CellmapResolver::new(&names);
        assert_eq!(r.name_hash(2), 0, "the walk has to keep going to reach check()");
        let e = r.check("Action01_Body").expect_err("2 is outside 0..2");
        let text = format!("{e}");
        assert!(text.contains("Action01_Body"), "{text}");
        assert!(text.contains("cellmap 2"), "{text}");
    }

    #[test]
    fn the_first_offending_map_id_is_the_one_reported() {
        let names = names();
        let r = CellmapResolver::new(&names);
        r.name_hash(7);
        r.name_hash(9);
        let text = format!("{}", r.check("pack").expect_err("both are out of range"));
        assert!(text.contains("cellmap 7"), "{text}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ssab::ssab::ss::format::{InterpolationType, PartAnimationDataFloatValueT};

    #[test]
    fn test_optimize_part_animation_data_float_deduplication() {
        // Test data: [index(time)=0, val=1.0], [index=1, val=1.0], [index=2, val=1.0], [index=3, val=2.0]
        let mut data = Vec::new();

        let mut item = PartAnimationDataFloatValueT::default();
        item.index = 0;
        item.value = 1.0;
        item.interpolation = InterpolationType::Linear;
        data.push(item);

        let mut item = PartAnimationDataFloatValueT::default();
        item.index = 1;
        item.value = 1.0;
        item.interpolation = InterpolationType::Linear;
        data.push(item);

        let mut item = PartAnimationDataFloatValueT::default();
        item.index = 2;
        item.value = 1.0;
        item.interpolation = InterpolationType::Linear;
        data.push(item);

        let mut item = PartAnimationDataFloatValueT::default();
        item.index = 3;
        item.value = 2.0;
        item.interpolation = InterpolationType::None;
        data.push(item);

        let optimized = optimize_part_animation_data_float(&data).expect("Should not be empty");

        // Expected behavior:
        // - Frame 0 is kept. Interpolation becomes None because 1.0 -> 1.0 doesn't need interpolation.
        // - Frame 1 is omitted (redundant).
        // - Frame 2 is kept as the last frame before value change.
        // - Frame 3 is kept (value changed).
        assert_eq!(optimized.len(), 3);
        assert_eq!(optimized[0].index, 0);
        assert_eq!(optimized[0].interpolation, InterpolationType::None);
        assert_eq!(optimized[1].index, 2);
        assert_eq!(optimized[2].index, 3);
    }

    #[test]
    fn test_optimize_part_animation_data_value_changing_range() {
        assert!(!optimize_part_animation_data_value_changing_range(&InterpolationType::Linear));
        assert!(optimize_part_animation_data_value_changing_range(&InterpolationType::Hermite));
        assert!(optimize_part_animation_data_value_changing_range(&InterpolationType::Bezier));
        assert!(!optimize_part_animation_data_value_changing_range(&InterpolationType::None));
    }

    #[test]
    fn test_hide_value_at() {
        let steps = [(0u16, false), (87, true), (213, false)];
        assert!(!hide_value_at(&steps, 0));
        assert!(!hide_value_at(&steps, 86));
        assert!(hide_value_at(&steps, 87));
        assert!(hide_value_at(&steps, 200));
        assert!(!hide_value_at(&steps, 213));
        assert!(!hide_value_at(&steps, 9999));
    }

    #[test]
    fn test_or_hide_steps_inherits_parent_window() {
        // A part that is never hidden on its own takes its parent's hidden window.
        let own = [(0u16, false)];
        let parent = [(0u16, false), (87, true), (213, false)];
        assert_eq!(or_hide_steps(&own, &parent), vec![(0, false), (87, true), (213, false)]);
    }

    #[test]
    fn test_or_hide_steps_union_of_windows() {
        // OR of two overlapping hidden windows, collapsed to minimal keyframes.
        let own = [(0u16, false), (10, true), (20, false)];
        let parent = [(0u16, false), (5, true), (15, false)];
        assert_eq!(or_hide_steps(&own, &parent), vec![(0, false), (5, true), (20, false)]);
    }

    #[test]
    fn test_or_hide_steps_collapses_when_always_hidden() {
        // When the union is hidden at every frame, only the frame-0 keyframe remains.
        let own = [(0u16, true), (130, false), (199, true)];
        let parent = [(0u16, false), (87, true), (213, false)];
        assert_eq!(or_hide_steps(&own, &parent), vec![(0, true)]);
    }

    #[test]
    fn test_or_hide_steps_unchanged_when_parent_never_hidden() {
        // A parent that is never hidden leaves the child's own keyframes intact.
        let own = [(0u16, false), (130, true), (200, false)];
        let parent = [(0u16, false)];
        assert_eq!(or_hide_steps(&own, &parent), own.to_vec());
    }
}
