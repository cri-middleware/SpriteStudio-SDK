use std::collections::HashMap;

fn convert_animedata_labels(labels: &[sspj::ssae::SsLabel]) -> Vec<ssab::ssab::ss::format::LabelT> {
    let mut v = labels
        .iter()
        .map(|label| {
            let mut l = ssab::ssab::ss::format::LabelT::default();
            l.name_hash = crate::utils::fnv1a_hash_str(&label.name);
            l.name = if label.name.is_empty() { None } else { Some(label.name.to_string()) };
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
                            let mut r = ssab::ssab::ss::format::RectT::default();
                            r.x1 = v.x1 as f32;
                            r.x2 = v.x2 as f32;
                            r.y1 = v.y1 as f32;
                            r.y2 = v.y2 as f32;
                            Box::new(r)
                        });
                        userdata.point = key.value.point.as_ref().map(|v| {
                            let mut p = ssab::ssab::ss::format::Vec2T::default();
                            p.v1 = v.x;
                            p.v2 = v.y;
                            Box::new(p)
                        });
                        userdata.str = key.value.user_string.as_ref().map(|v| v.to_string());
                        u_args.value = Some(Box::new(userdata));

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
                            s.command_id = Some(command.command_id.to_string());
                            s.params = if command.params.is_empty() {
                                None
                            } else {
                                let v = command
                                    .params
                                    .iter()
                                    .map(|v| {
                                        let mut command_param_t = ssab::ssab::ss::format::SignalCommandParamT::default();
                                        command_param_t.param_id = Some(v.param_id.to_string());
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
                                                s.value = Some(v.to_string());
                                                ssab::ssab::ss::format::SignalCommandParamValueT::Text(Box::new(s))
                                            }
                                            sspj::ssattribute::SsAttributeSignalCommandParamValue::None => {
                                                let n = ssab::ssab::ss::format::NoneValueEntryT::default();
                                                ssab::ssab::ss::format::SignalCommandParamValueT::None(Box::new(n))
                                            }
                                        };

                                        command_param_t
                                    })
                                    .collect::<Vec<_>>();
                                Some(v)
                            };
                            s_args.value = Some(Box::new(s));

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
                    // MEMO: "Instance" is complemented by backfilling.
                    v.keys.iter().for_each(|key| {
                        let mut instance_t = ssab::ssab::ss::format::PartAttributeInstanceT::default();
                        instance_t.pingpong = key.value.pingpong;
                        instance_t.independent = key.value.independent;
                        instance_t.reverse = key.value.reverse;
                        // SS6's `SsInstanceAttr` carried both `infinity: bool`
                        // and `loop_num: i32`; SS7's `PartAttributeInstance`
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

                        // Complement for frame 0.
                        if 0 < key.time {
                            let events_per_frame_0 = map_events_per_frames.entry(0).or_insert_with(|| {
                                let mut e_frame_0 = ssab::ssab::ss::format::EventsPerFrameT::default();
                                e_frame_0.frame_index = 0;
                                e_frame_0
                            });

                            if events_per_frame_0.instances.get_or_insert_with(Vec::new).iter().position(|item| item.part_index == part_index).is_none() {
                                let instance_t_frame_0 = instance_t.clone();

                                let mut i_args_frame_0 = ssab::ssab::ss::format::EventInstanceT::default();
                                i_args_frame_0.part_index = part_index;
                                i_args_frame_0.value = Some(Box::new(instance_t_frame_0));

                                events_per_frame_0.instances.get_or_insert_with(Vec::new).push(i_args_frame_0);
                            }
                        }

                        // for target frame.
                        let mut i_args = ssab::ssab::ss::format::EventInstanceT::default();
                        i_args.part_index = part_index;
                        i_args.value = Some(Box::new(instance_t));

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
                    // MEMO: "Effect" is complemented by backfilling.
                    v.keys.iter().for_each(|key| {
                        let mut effect_t = ssab::ssab::ss::format::PartAttributeEffectT::default();
                        effect_t.start_time = key.value.start_time;
                        effect_t.speed = key.value.speed;
                        effect_t.independent = key.value.independent;

                        // Complement for frame 0.
                        if 0 < key.time {
                            let events_per_frame_0 = map_events_per_frames.entry(0).or_insert_with(|| {
                                let mut e_frame_0 = ssab::ssab::ss::format::EventsPerFrameT::default();
                                e_frame_0.frame_index = 0;
                                e_frame_0
                            });

                            if events_per_frame_0.effects.get_or_insert_with(Vec::new).iter().position(|item| item.part_index == part_index).is_none() {
                                let effect_t_frame_0 = ssab::ssab::ss::format::PartAttributeEffectT::default();

                                let mut e_args_frame_0 = ssab::ssab::ss::format::EventEffectT::default();
                                e_args_frame_0.part_index = part_index;
                                e_args_frame_0.value = Some(Box::new(effect_t_frame_0));

                                events_per_frame_0.effects.get_or_insert_with(Vec::new).push(e_args_frame_0);
                            }
                        }

                        // for target frame.
                        let mut e_args = ssab::ssab::ss::format::EventEffectT::default();
                        e_args.part_index = part_index;
                        e_args.value = Some(Box::new(effect_t));

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
                        audio_t.sound_name = Some(key.value.sound_name.to_string());
                        audio_t.loop_num = key.value.loop_num;
                        a_args.value = Some(Box::new(audio_t));

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
    cellmap_names: &[String],
) -> ssab::ssab::ss::format::AnimationDataT {
    let mut anime_data_t = ssab::ssab::ss::format::AnimationDataT::default();

    anime_data_t.name = anime_data.name.to_string();
    anime_data_t.name_hash = crate::utils::fnv1a_hash_str(&anime_data.name);
    anime_data_t.parts_initial_attributes = Some(convert_parts_initial_attributes(setup_animation, parts_map, cellmap_names));
    anime_data_t.parts_animation_data = Some(convert_parts_animation_data(anime_data, setup_animation, parts_map, cellmap_names));
    anime_data_t.initial_events = convert_parts_initial_events(setup_animation, parts_map, sound_list_id_to_name_hash);
    anime_data_t.events = convert_events_per_frames(anime_data, setup_animation, parts_map, sound_list_id_to_name_hash);
    anime_data_t.labels = if anime_data.labels.is_empty() { None } else { Some(convert_animedata_labels(&anime_data.labels)) };
    anime_data_t.start_frame = anime_data.settings.start_frame as i16;
    anime_data_t.end_frame = anime_data.settings.end_frame as i16;
    anime_data_t.total_frame = anime_data.settings.frame_count as i16;
    anime_data_t.fps = anime_data.settings.fps as i16;
    let mut canvas = ssab::ssab::ss::format::RectT::default();
    canvas.x1 = anime_data.settings.pivot.x;
    canvas.y1 = anime_data.settings.pivot.y;
    canvas.x2 = anime_data.settings.canvas_size.x;
    canvas.y2 = anime_data.settings.canvas_size.y;
    anime_data_t.canvas = Some(Box::new(canvas));

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

fn merge_setup_attributes_to_part_attribute_t(p_args: &mut ssab::ssab::ss::format::InitialPartAttributesT, setup_attributes: &[sspj::ssattribute::SsAttributeKind], cellmap_names: &[String]) {
    // MEMO: This function. Since only “setup” is targeted, “key.time == 0” might not be necessary.
    //       It's required because the caller is attempting to composite with frame 0,
    //         but that should probably be done during key-frame placement ?    Yuzu.
    setup_attributes.iter().for_each(|attribute| {
        //        p_args.update_flag = 0;
        match attribute {
            sspj::ssattribute::SsAttributeKind::Cell(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    let mut value = ssab::ssab::ss::format::PartAttributeCellT::default();
                    value.cellmap_name_hash = cellmap_id_to_name_hash(cellmap_names, _key.value.map_id);
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
                    // TODO: When Z-sort, priority must be based on  spatially transformed Z-values. Yuzu.
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
                    let mut value = ssab::ssab::ss::format::PartAttributePartColorT::default();
                    value.blend_type = convert_alpha_blend_type(&_key.value.blend_type);

                    let mut color = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
                    let mut rgba = ssab::ssab::ss::format::U8RectT::default();

                    //                    color = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
                    //                    rgba = ssab::ssab::ss::format::U8RectT::default();
                    rgba.a = _key.value.lt.rgba.a;
                    rgba.r = _key.value.lt.rgba.r;
                    rgba.g = _key.value.lt.rgba.g;
                    rgba.b = _key.value.lt.rgba.b;
                    color.rgba = Some(Box::new(rgba));
                    color.rate = _key.value.lt.rate;
                    value.lt = Some(Box::new(color));

                    color = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
                    rgba = ssab::ssab::ss::format::U8RectT::default();
                    rgba.a = _key.value.rt.rgba.a;
                    rgba.r = _key.value.rt.rgba.r;
                    rgba.g = _key.value.rt.rgba.g;
                    rgba.b = _key.value.rt.rgba.b;
                    color.rgba = Some(Box::new(rgba));
                    color.rate = _key.value.rt.rate;
                    value.rt = Some(Box::new(color));

                    color = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
                    rgba = ssab::ssab::ss::format::U8RectT::default();
                    rgba.a = _key.value.lb.rgba.a;
                    rgba.r = _key.value.lb.rgba.r;
                    rgba.g = _key.value.lb.rgba.g;
                    rgba.b = _key.value.lb.rgba.b;
                    color.rgba = Some(Box::new(rgba));
                    color.rate = _key.value.lb.rate;
                    value.lb = Some(Box::new(color));

                    color = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
                    rgba = ssab::ssab::ss::format::U8RectT::default();
                    rgba.a = _key.value.rb.rgba.a;
                    rgba.r = _key.value.rb.rgba.r;
                    rgba.g = _key.value.rb.rgba.g;
                    rgba.b = _key.value.rb.rgba.b;
                    color.rgba = Some(Box::new(rgba));
                    color.rate = _key.value.rb.rate;
                    value.rb = Some(Box::new(color));

                    p_args.part_color = Some(Box::new(value));
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePartColor;
                }
            }
            sspj::ssattribute::SsAttributeKind::Color => { /* MEMO: Not Supported (Feature prior to Ver.6.0) */ }
            sspj::ssattribute::SsAttributeKind::Shader(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    let mut value = ssab::ssab::ss::format::PartAttributeShaderT::default();
                    value.id = Some(_key.value.id.clone());
                    value.id_hash = crate::utils::fnv1a_hash_str(&_key.value.id);

                    value.param0 = _key.value.param0;
                    value.param1 = _key.value.param1;
                    value.param2 = _key.value.param2;
                    value.param3 = _key.value.param3;
                    value.param4 = _key.value.param4;
                    value.param5 = _key.value.param5;
                    value.param6 = _key.value.param6;
                    value.param7 = _key.value.param7;

                    value.map0_cellmap_name_hash = cellmap_id_to_name_hash(cellmap_names, _key.value.map0.map_id);
                    value.map0_name_hash = crate::utils::fnv1a_hash_str(&_key.value.map0.name);
                    value.map1_cellmap_name_hash = cellmap_id_to_name_hash(cellmap_names, _key.value.map1.map_id);
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

                        //                       coordinate = ssab::ssab::ss::format::Vec2T::default();
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

                    let mut coordinate = Vec::new();
                    let mut index = Vec::<i16>::new();
                    _key.value.vchg.iter().for_each(|_item| {
                        let mut vec2 = ssab::ssab::ss::format::Vec2T::default();
                        vec2.v1 = _item.x;
                        vec2.v2 = _item.y;
                        coordinate.push(vec2);
                        index.push(_item.x_index as i16);
                    });

                    value.coordinate = Some(coordinate);
                    value.index = Some(index);
                    p_args.deform = Some(Box::new(value));
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeDeform;
                }
            }
            sspj::ssattribute::SsAttributeKind::Texchange(v) => {
                if let Some(_key) = v.keys.iter().find(|key| key.time == 0) {
                    p_args.texture = crate::utils::fnv1a_hash_str(&_key.value.texture_name);
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
                        let mut r = ssab::ssab::ss::format::RectT::default();
                        r.x1 = v.x1 as f32;
                        r.x2 = v.x2 as f32;
                        r.y1 = v.y1 as f32;
                        r.y2 = v.y2 as f32;
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
                        value.command_id = Some(command.command_id.to_string());
                        value.params = if command.params.is_empty() {
                            None
                        } else {
                            let params = command
                                .params
                                .iter()
                                .map(|p| {
                                    let mut command_param_t = ssab::ssab::ss::format::SignalCommandParamT::default();
                                    command_param_t.param_id = Some(p.param_id.to_string());
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
                                            s.value = Some(v.to_string());
                                            ssab::ssab::ss::format::SignalCommandParamValueT::Text(Box::new(s))
                                        }
                                        sspj::ssattribute::SsAttributeSignalCommandParamValue::None => {
                                            let n = ssab::ssab::ss::format::NoneValueEntryT::default();
                                            ssab::ssab::ss::format::SignalCommandParamValueT::None(Box::new(n))
                                        }
                                    };

                                    command_param_t
                                })
                                .collect::<Vec<_>>();

                            Some(params)
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
                    value.sound_name = Some(_key.value.sound_name.to_string());
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

fn convert_parts_initial_attributes(anime_data: &sspj::ssae::SsAnimation, parts_map: &HashMap<String, &sspj::ssae::SsPart>, cellmap_names: &[String]) -> Vec<ssab::ssab::ss::format::InitialPartAttributesT> {
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
        merge_setup_attributes_to_part_attribute_t(&mut parts_initial_attributes_t[index_part], &part_anime.attributes, cellmap_names);

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

    parts_initial_attributes_t
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

fn convert_part_animation_data(p_arg: &mut ssab::ssab::ss::format::PartAnimationDataT, part_anime: &sspj::ssae::SsPartAnime, cellmap_names: &[String], is_setup: bool) {
    part_anime.attributes.iter().for_each(|attribute| match attribute {
        sspj::ssattribute::SsAttributeKind::Cell(v) => {
            process_simple_attribute!(&v.keys, p_arg.cell_array, ssab::ssab::ss::format::PartAnimationDataCellValueT, |k: &sspj::ssattribute::SsAttributeCellKey| {
                let mut c_args = ssab::ssab::ss::format::PartAttributeCellT::default();
                c_args.cellmap_name_hash = cellmap_id_to_name_hash(cellmap_names, k.value.map_id);
                c_args.name_hash = crate::utils::fnv1a_hash_str(&k.value.name);
                Some(Box::new(c_args))
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
            process_interpolatable_attribute!(&v.keys, p_arg.part_color_array, ssab::ssab::ss::format::PartAnimationDataPartColorT, |k: &sspj::ssattribute::SsAttributePartsColorKey| {
                let mut value = ssab::ssab::ss::format::PartAttributePartColorT::default();
                value.blend_type = convert_alpha_blend_type(&k.value.blend_type);
                let mut key_value = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
                let mut rgba = ssab::ssab::ss::format::U8RectT::default();
                rgba.a = k.value.lt.rgba.a;
                rgba.r = k.value.lt.rgba.r;
                rgba.g = k.value.lt.rgba.g;
                rgba.b = k.value.lt.rgba.b;
                key_value.rgba = Some(Box::new(rgba));
                key_value.rate = k.value.lt.rate;
                value.lt = Some(Box::new(key_value));
                let mut key_value = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
                let mut rgba = ssab::ssab::ss::format::U8RectT::default();
                rgba.a = k.value.rt.rgba.a;
                rgba.r = k.value.rt.rgba.r;
                rgba.g = k.value.rt.rgba.g;
                rgba.b = k.value.rt.rgba.b;
                key_value.rgba = Some(Box::new(rgba));
                key_value.rate = k.value.rt.rate;
                value.rt = Some(Box::new(key_value));
                let mut key_value = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
                let mut rgba = ssab::ssab::ss::format::U8RectT::default();
                rgba.a = k.value.lb.rgba.a;
                rgba.r = k.value.lb.rgba.r;
                rgba.g = k.value.lb.rgba.g;
                rgba.b = k.value.lb.rgba.b;
                key_value.rgba = Some(Box::new(rgba));
                key_value.rate = k.value.lb.rate;
                value.lb = Some(Box::new(key_value));
                let mut key_value = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
                let mut rgba = ssab::ssab::ss::format::U8RectT::default();
                rgba.a = k.value.rb.rgba.a;
                rgba.r = k.value.rb.rgba.r;
                rgba.g = k.value.rb.rgba.g;
                rgba.b = k.value.rb.rgba.b;
                key_value.rgba = Some(Box::new(rgba));
                key_value.rate = k.value.rb.rate;
                value.rb = Some(Box::new(key_value));
                Some(Box::new(value))
            });
        }
        sspj::ssattribute::SsAttributeKind::Shader(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.shader_array, ssab::ssab::ss::format::PartAnimationDataShaderT, |k: &sspj::ssattribute::SsAttributeShaderKey| {
                let mut value = ssab::ssab::ss::format::PartAttributeShaderT::default();
                value.id = Some(k.value.id.clone());
                value.id_hash = crate::utils::fnv1a_hash_str(&k.value.id);
                value.param0 = k.value.param0;
                value.param1 = k.value.param1;
                value.param2 = k.value.param2;
                value.param3 = k.value.param3;
                value.param4 = k.value.param4;
                value.param5 = k.value.param5;
                value.param6 = k.value.param6;
                value.param7 = k.value.param7;
                value.map0_cellmap_name_hash = cellmap_id_to_name_hash(cellmap_names, k.value.map0.map_id);
                value.map0_name_hash = crate::utils::fnv1a_hash_str(&k.value.map0.name);
                value.map1_cellmap_name_hash = cellmap_id_to_name_hash(cellmap_names, k.value.map1.map_id);
                value.map1_name_hash = crate::utils::fnv1a_hash_str(&k.value.map1.name);
                Some(Box::new(value))
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
                    k.value.skew.as_ref().map(|v| {
                        let mut val = ssab::ssab::ss::format::Vec2T::default();
                        val.v1 = v.x;
                        val.v2 = v.y;
                        Box::new(val)
                    })
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
                    Some(Box::new(value))
                });
            }
        }
        sspj::ssattribute::SsAttributeKind::Deform(v) => {
            process_interpolatable_attribute!(&v.keys, p_arg.deform_array, ssab::ssab::ss::format::PartAnimationDataDeformT, |k: &sspj::ssattribute::SsAttributeDeformKey| {
                let mut value = ssab::ssab::ss::format::PartAttributeDeformT::default();
                value.coordinate = Some(
                    k.value
                        .vchg
                        .iter()
                        .map(|p| {
                            let mut v = ssab::ssab::ss::format::Vec2T::default();
                            v.v1 = p.x;
                            v.v2 = p.y;
                            v
                        })
                        .collect(),
                );
                value.index = Some(k.value.vchg.iter().map(|p| p.x_index as i16).collect());
                Some(Box::new(value))
            });
        }
        sspj::ssattribute::SsAttributeKind::Texchange(v) => {
            process_simple_attribute!(&v.keys, p_arg.texture_array, ssab::ssab::ss::format::PartAnimationDataUIntValueT, |k: &sspj::ssattribute::SsAttributeTexchangeKey| { crate::utils::fnv1a_hash_str(&k.value.texture_name) });
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

fn convert_parts_animation_data(anime_data: &sspj::ssae::SsAnimation, setup_animation: &sspj::ssae::SsAnimation, parts_map: &HashMap<String, &sspj::ssae::SsPart>, cellmap_names: &[String]) -> Vec<ssab::ssab::ss::format::PartAnimationDataT> {
    let mut parts_animation_data_t = Vec::<ssab::ssab::ss::format::PartAnimationDataT>::new();
    parts_animation_data_t.resize(parts_map.len(), ssab::ssab::ss::format::PartAnimationDataT::default());

    for part_anime in &setup_animation.part_animes {
        let Some(p) = parts_map.get(&part_anime.part_name) else {
            // not found part (an illegal case)
            continue;
        };

        let index_part = p.array_index as usize;
        convert_part_animation_data(&mut parts_animation_data_t[index_part], part_anime, cellmap_names, true);
    }

    for part_anime in &anime_data.part_animes {
        let Some(p) = parts_map.get(&part_anime.part_name) else {
            // not found part (an illegal case)
            continue;
        };

        let index_part = p.array_index as usize;
        convert_part_animation_data(&mut parts_animation_data_t[index_part], part_anime, cellmap_names, false);
        modify_part_animation_data(&mut parts_animation_data_t[index_part], &p.part_type, part_anime);
    }

    for part in parts_map.values() {
        let index_part = part.array_index as usize;
        apply_part_type_defaults(&mut parts_animation_data_t[index_part], part);
    }

    parts_animation_data_t
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
            part_data_t.name = Some(part.name.to_string());
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
                    let vertex = mesh_bind
                        .mesh_vertices_bind_array
                        .iter()
                        .map(|bind_info| {
                            let influence = if bind_info.bind_bone_num > 0 {
                                Some(
                                    (0..bind_info.bind_bone_num as usize)
                                        .map(|i| {
                                            let mut inf = ssab::ssab::ss::format::PartMeshBindingInfluenceT::default();
                                            inf.bone = bone_index_to_part_index.get(&bind_info.bone_index[i]).copied().unwrap_or(0);
                                            inf.weight = bind_info.weight[i] as f32 / 100.0;
                                            let mut coord = ssab::ssab::ss::format::Vec2T::default();
                                            coord.v1 = bind_info.offset[i].x;
                                            coord.v2 = bind_info.offset[i].y;
                                            inf.coordinate = Some(Box::new(coord));
                                            inf
                                        })
                                        .collect::<Vec<_>>(),
                                )
                            } else {
                                None
                            };
                            let mut vtx = ssab::ssab::ss::format::PartMeshBindingVertexT::default();
                            vtx.influence = influence;
                            vtx
                        })
                        .collect::<Vec<_>>();
                    let mut binding = ssab::ssab::ss::format::PartMeshBindingT::default();
                    binding.vertex = Some(vertex);
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
                    part_type.shape_mask = part.shape_mask;

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
                    part_type.text_mask = part.text_mask;
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
                    part_type.nines_mask = part.nines_mask;

                    ssab::ssab::ss::format::PartTypeT::PartTypeNines(Box::new(part_type))
                }
                sspj::sstypes::SsPartType::Instance => {
                    let mut part_type = ssab::ssab::ss::format::PartTypeInstanceT::default();
                    part_type.ref_anime_hash = crate::utils::fnv1a_hash_str(&part.ref_anime);
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

pub fn convert_animation_data_list(anime_pack: &sspj::ssae::SsAnimePack, parts_map: &HashMap<String, &sspj::ssae::SsPart>, sound_list_id_to_name_hash: &HashMap<i32, u32>, cellmap_names: &[String]) -> Vec<ssab::ssab::ss::format::AnimationDataT> {
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
            Some(convert_animedatat(anime_data, setup_animation, parts_map, sound_list_id_to_name_hash, cellmap_names))
        })
        .collect::<Vec<ssab::ssab::ss::format::AnimationDataT>>()
}

fn cellmap_id_to_name_hash(cellmap_names: &[String], map_id: i32) -> u32 {
    // Negative map_id is the documented "no map" convention in sspj
    // (e.g. unset shader map slot).
    if map_id < 0 {
        return 0;
    }
    // map_id indexes into the same ssae cellmap_names list that produced it,
    // so out-of-range here means the sspj is malformed.
    let file_name = &cellmap_names[map_id as usize];
    crate::utils::fnv1a_hash_str(crate::utils::cellmap_basename(file_name))
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
}
