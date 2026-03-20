use std::collections::HashMap;
// MEMO: Omit entire namespace, because public functions  (in ssinterpolate) are defined extern "C".
use ssinterpolate::interpolate::*;

fn convert_animedata_labels(labels: &[sspj::ssae::SsLabel]) -> Vec<ssab::ssab::ss::format::LabelT> {
    let mut v = labels.iter().map(|label| {
        let mut l = ssab::ssab::ss::format::LabelT::default();
        l.name_hash = crate::utils::fnv1a_hash_str(&label.name);
        l.name = if label.name.is_empty() { None } else { Some(label.name.to_string()) };
        l.time = label.time;
        l
    }).collect::<Vec<ssab::ssab::ss::format::LabelT>>();
    v.sort_by_key(|l| l.name_hash);
    v
}

fn convert_events_per_frames_part(map_events_per_frames: &mut HashMap<i32, ssab::ssab::ss::format::EventsPerFrameT>, anime_data: &sspj::ssae::SsAnimation, is_timeline: bool, parts_map: &HashMap<String, &sspj::ssae::SsPart>) {
    for part_anime in &anime_data.part_animes {
        let Some(p) = parts_map.get(& part_anime.part_name) else {
            // not found part (an illegal case)
            continue;
        };

        let part_index = p.array_index as u16;

        part_anime.attributes.iter().for_each(|attribute| {
            match attribute {
                sspj::ssattribute::SsAttributeKind::User(v) => {
                    if is_timeline {
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
                },
                sspj::ssattribute::SsAttributeKind::Signal(v) => {
                    if is_timeline {
                        // MEMO: "Signal" set in setup-animation is not adopted in the timeline.
                        // MEMO: "Signal" does not backfill-complement.
                        v.keys.iter().for_each(|key| {
                            key.value.commands.iter().for_each(|command| {
                                let mut s_args = ssab::ssab::ss::format::EventSignalT::default();
                                s_args.part_index = part_index;
                                let mut s = ssab::ssab::ss::format::PartAttributeSignalT::default();
                                s.active = command.active;
                                s.command_id = Some(command.command_id.to_string());
                                s.params = if command.params.is_empty() { None } else {
                                    let v = command.params.iter().map(|v| {
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
                                            },
                                            sspj::ssattribute::SsAttributeSignalCommandParamValue::Index(v) => {
                                                let mut i = ssab::ssab::ss::format::IntValueEntryT::default();
                                                i.value = *v;
                                                ssab::ssab::ss::format::SignalCommandParamValueT::Index(Box::new(i))
                                            },
                                            sspj::ssattribute::SsAttributeSignalCommandParamValue::Integer(v) => {
                                                let mut i = ssab::ssab::ss::format::IntValueEntryT::default();
                                                i.value = *v;
                                                ssab::ssab::ss::format::SignalCommandParamValueT::Integer(Box::new(i))
                                            },
                                            sspj::ssattribute::SsAttributeSignalCommandParamValue::Floating(v) => {
                                                let mut f = ssab::ssab::ss::format::FloatValueEntryT::default();
                                                f.value = *v;
                                                ssab::ssab::ss::format::SignalCommandParamValueT::Floating(Box::new(f))
                                            },
                                            sspj::ssattribute::SsAttributeSignalCommandParamValue::Text(v) => {
                                                let mut s = ssab::ssab::ss::format::StringValueEntryT::default();
                                                s.value = Some(v.to_string());
                                                ssab::ssab::ss::format::SignalCommandParamValueT::Text(Box::new(s))
                                            },
                                            sspj::ssattribute::SsAttributeSignalCommandParamValue::None => {
                                                let n = ssab::ssab::ss::format::NoneValueEntryT::default();
                                                ssab::ssab::ss::format::SignalCommandParamValueT::None(Box::new(n))
                                            },
                                        };

                                        command_param_t
                                    }).collect::<Vec<_>>();
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
                },
                sspj::ssattribute::SsAttributeKind::Instance(v) => {
                    // MEMO: "Instance" is complemented by backfilling.
                    v.keys.iter().for_each(|key| {
                        let mut instance_t = ssab::ssab::ss::format::PartAttributeInstanceT::default();
                        instance_t.pingpong = key.value.pingpong;
                        instance_t.independent = key.value.independent;
                        instance_t.loop_num = key.value.loop_num;
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

                            if let None = events_per_frame_0.instances.get_or_insert_with(Vec::new).iter().position(|item| item.part_index == part_index) {
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
                },
                sspj::ssattribute::SsAttributeKind::Effect(v) => {
                    // MEMO: "Effect" is complemented by backfilling.
                    v.keys.iter().for_each(|key| {
                        let mut effect_t = ssab::ssab::ss::format::PartAttributeEffectT::default();
                        effect_t.cur_keyframe; // TODO: implement
                        effect_t.start_time = key.value.start_time;
                        effect_t.speed = key.value.speed;
                        effect_t.loopflag; // TODO: implement

                        // Complement for frame 0.
                        if 0 < key.time {
                            let events_per_frame_0 = map_events_per_frames.entry(0).or_insert_with(|| {
                                let mut e_frame_0 = ssab::ssab::ss::format::EventsPerFrameT::default();
                                e_frame_0.frame_index = 0;
                                e_frame_0
                            });

                            if let None = events_per_frame_0.effects.get_or_insert_with(Vec::new).iter().position(|item| item.part_index == part_index) {
                                let effect_t_frame_0 = effect_t.clone();

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
                },
                sspj::ssattribute::SsAttributeKind::Audio(v) => {
                    // MEMO: "Audio" does not backfill-complement.
                    v.keys.iter().for_each(|key| {
                        let mut a_args = ssab::ssab::ss::format::EventAudioT::default();
                        a_args.part_index = part_index;
                        let mut audio_t = ssab::ssab::ss::format::PartAttributeAudioT::default();
                        audio_t.sound_list_id = key.value.sound_list_id;
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
                },
                _ => {
                    // skip
                },
            }
        });
    }
}

fn convert_events_per_frames(anime_data: &sspj::ssae::SsAnimation, setup_animation: &sspj::ssae::SsAnimation, parts_map: &HashMap<String, &sspj::ssae::SsPart>) -> Option<Vec<ssab::ssab::ss::format::EventsPerFrameT>> {
    let mut map_events_per_frames = HashMap::<i32, ssab::ssab::ss::format::EventsPerFrameT>::new();

    convert_events_per_frames_part(&mut map_events_per_frames, setup_animation, false, parts_map);
    convert_events_per_frames_part(&mut map_events_per_frames, anime_data, true, parts_map);

    if map_events_per_frames.is_empty() {
        None
    } else {
        let mut vec_map: Vec<(i32, ssab::ssab::ss::format::EventsPerFrameT)> = map_events_per_frames.into_iter().collect();
        vec_map.sort_by(|a, b| a.0.cmp(&b.0));
        Some(vec_map.into_iter().map(|(_, v)| v).collect::<Vec<ssab::ssab::ss::format::EventsPerFrameT>>())
    }
}
fn convert_animedatat(anime_data: &sspj::ssae::SsAnimation, setup_animation: &sspj::ssae::SsAnimation, parts_map: &HashMap<String, &sspj::ssae::SsPart>) -> ssab::ssab::ss::format::AnimationDataT {
    let mut anime_data_t = ssab::ssab::ss::format::AnimationDataT::default();

    anime_data_t.name = anime_data.name.to_string();
    anime_data_t.parts_initial_attributes = Some(convert_parts_initial_attributes(setup_animation));
    anime_data_t.parts_animation_data = Some(convert_parts_animation_data(anime_data, setup_animation, parts_map));
    anime_data_t.parts_mesh_data = None;
    anime_data_t.initial_events = convert_parts_initial_events(&setup_animation, parts_map);
    anime_data_t.events = convert_events_per_frames(&anime_data, &setup_animation, parts_map);
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

fn merge_setup_attributes_to_part_attribute_t(p_args: &mut ssab::ssab::ss::format::InitialPartAttributesT, setup_attributes: &[sspj::ssattribute::SsAttributeKind]) {
    // MEMO: This function. Since only “setup” is targeted, “key.time == 0” might not be necessary.
    //       It's required because the caller is attempting to composite with frame 0,
    //         but that should probably be done during key-frame placement ?    Yuzu.
    setup_attributes.iter().for_each(|attribute| {
//        p_args.update_flag = 0;
        match attribute {
            sspj::ssattribute::SsAttributeKind::Cell(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    let mut value = ssab::ssab::ss::format::PartAttributeCellT::default();
                    value.map_id = _key.value.map_id;
                    value.name_hash = crate::utils::fnv1a_hash_str(&_key.value.name);
                    p_args.cell = Some(Box::new(value));
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeCell;
                });
            },
            sspj::ssattribute::SsAttributeKind::PosX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.position_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePosX;
                });
            },
            sspj::ssattribute::SsAttributeKind::PosY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.position_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePosY;
                });
            },
            sspj::ssattribute::SsAttributeKind::PosZ(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.position_z = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePosZ;
                });
            },
            sspj::ssattribute::SsAttributeKind::RotX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.rotation_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeRotX;
                });
            },
            sspj::ssattribute::SsAttributeKind::RotY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.rotation_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeRotY;
                });
            },
            sspj::ssattribute::SsAttributeKind::RotZ(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.rotation_z = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeRotZ;
                });
            },
            sspj::ssattribute::SsAttributeKind::SclX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.scale_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeSclX;
                });
            },
            sspj::ssattribute::SsAttributeKind::SclY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.scale_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeSclY;
                });
            },
            sspj::ssattribute::SsAttributeKind::LosclX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.local_scale_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeLosclX;
                });
            },
            sspj::ssattribute::SsAttributeKind::LosclY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.local_scale_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeLosclY;
                });
            },
            sspj::ssattribute::SsAttributeKind::Alpha(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.alpha = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeAlpha;
                });
            },
            sspj::ssattribute::SsAttributeKind::LoAlpha(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.local_alpha = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeLoAlpha;
                });
            },
            sspj::ssattribute::SsAttributeKind::Prio(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: When Z-sort, priority must be based on  spatially transformed Z-values. Yuzu.
                    p_args.priority = _key.value as f32;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePrio;
                });
            },
            sspj::ssattribute::SsAttributeKind::FlipH(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.flip_h = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeFlipH;
                });
            },
            sspj::ssattribute::SsAttributeKind::FlipV(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.flip_v = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeFlipV;
                });
            },
            sspj::ssattribute::SsAttributeKind::Hide(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.hide = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeHide;
                });
            },
            sspj::ssattribute::SsAttributeKind::PartsColor(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    let mut value =  ssab::ssab::ss::format::PartAttributePartColorT::default();
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
                });
            },
            sspj::ssattribute::SsAttributeKind::Color => {
                /* MEMO: Not Supported (Feature prior to Ver.6.0) */
            },
            sspj::ssattribute::SsAttributeKind::Shader(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
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

                    value.map0_id = _key.value.map0.map_id;
                    value.map0_name_hash = crate::utils::fnv1a_hash_str(&_key.value.map0.name);
                    value.map1_id = _key.value.map1.map_id;
                    value.map1_name_hash = crate::utils::fnv1a_hash_str(&_key.value.map1.name);

                    p_args.shader = Some(Box::new(value));
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeShader;
                });
            },
            sspj::ssattribute::SsAttributeKind::Vertex(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    if _key.value.use_skew.is_some() && _key.value.use_skew.unwrap() == true {
                        // Skew
                        match &_key.value.skew {
                            None => {
                                p_args.skew_x = 0.0 as f32;
                                p_args.skew_y = 0.0 as f32;
                            }
                            Some(_skew) => {
                                p_args.skew_x = _skew.x;
                                p_args.skew_y = _skew.x;
                            }
                        }
                        p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeSkew;

                        p_args.vertex = None;   // Mutually exclusive with "Vertex"
                    } else {
                        // Vertex
                        let mut value = ssab::ssab::ss::format::PartAttributeVertexT::default();
                        let mut coordinate = ssab::ssab::ss::format::Vec2T::default();

//                       coordinate = ssab::ssab::ss::format::Vec2T::default();
                        _key.value.lt.as_ref().map(|_lt| {
                            coordinate.v1 = _lt.x;
                            coordinate.v2 = _lt.y;
                        });
                        value.lt = Some(Box::new(coordinate));

                        coordinate = ssab::ssab::ss::format::Vec2T::default();
                        _key.value.rt.as_ref().map(|_rt| {
                            coordinate.v1 = _rt.x;
                            coordinate.v2 = _rt.y;
                        });
                        value.rt = Some(Box::new(coordinate));

                        coordinate = ssab::ssab::ss::format::Vec2T::default();
                        _key.value.lb.as_ref().map(|_lb| {
                            coordinate.v1 = _lb.x;
                            coordinate.v2 = _lb.y;
                        });
                        value.lb = Some(Box::new(coordinate));

                        coordinate = ssab::ssab::ss::format::Vec2T::default();
                        _key.value.rb.as_ref().map(|_rb| {
                            coordinate.v1 = _rb.x;
                            coordinate.v2 = _rb.y;
                        });
                        value.rb = Some(Box::new(coordinate));

                        p_args.vertex = Some(Box::new(value));
                        p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeVertex;

                        p_args.skew_x = 0.0 as f32; // mutually exclusive with "Skew"
                        p_args.skew_y = 0.0 as f32;
                    }
                });
            },
            sspj::ssattribute::SsAttributeKind::PivotX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.pivot_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePivotX;
                });
            },
            sspj::ssattribute::SsAttributeKind::PivotY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.pivot_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributePivotY;
                });
            },
            sspj::ssattribute::SsAttributeKind::AnchorX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.anchor_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeAnchorX;
                });
            },
            sspj::ssattribute::SsAttributeKind::AnchorY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.anchor_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeAnchorY;
                });
            },
            sspj::ssattribute::SsAttributeKind::SizeX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.size_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeSizeX;
               });
            },
            sspj::ssattribute::SsAttributeKind::SizeY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.size_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeSizeY;
                });
            },
            sspj::ssattribute::SsAttributeKind::ImgFlipH(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.img_flip_h = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeImgFlipH;
                });
            },
            sspj::ssattribute::SsAttributeKind::ImgFlipV(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.img_flip_v = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeImgFlipV;
                });
            },
            sspj::ssattribute::SsAttributeKind::UvtX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.uv_translation_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeUvtX;
                });
            },
            sspj::ssattribute::SsAttributeKind::UvtY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.uv_translation_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeUvtY;
                });
            },
            sspj::ssattribute::SsAttributeKind::UvrZ(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    /* MEMO: (Preliminary) "rotation" is set to "move.z". */
                    p_args.uv_rotation_z = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeUvrZ;
                });
            },
            sspj::ssattribute::SsAttributeKind::UvsX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.uv_scale_x = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeUvsX;
                });
            },
            sspj::ssattribute::SsAttributeKind::UvsY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.uv_scale_y = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeUvsY;
                });
            },
            sspj::ssattribute::SsAttributeKind::Boundr(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.bounding_radius = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeBoundr;
                });
            },
            sspj::ssattribute::SsAttributeKind::Mask(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    p_args.mask = _key.value;
                    p_args.update_flag |= ssab::ssab::ss::format::PartAttributesUpdateFlags::AttributeMask;
                });
            },
            sspj::ssattribute::SsAttributeKind::Deform(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
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
                });
            },
            _ => {
                // Empty.
            },
        }
    });
}

fn merge_setup_attributes_to_part_event_t(p_args: &mut Vec<ssab::ssab::ss::format::InitialEventsT>, part_index: u16, setup_attributes: &[sspj::ssattribute::SsAttributeKind]) {
    let mut initial_event_t = ssab::ssab::ss::format::InitialEventsT::default();
    let mut flag_exist_event = false;

    setup_attributes.iter().for_each(|attribute| {
        match attribute {
            sspj::ssattribute::SsAttributeKind::User(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
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
                });
            },
            sspj::ssattribute::SsAttributeKind::Signal(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    _key.value.commands.iter().for_each(|command| {
                        let mut value = ssab::ssab::ss::format::PartAttributeSignalT::default();
                        value.active = command.active;
                        value.command_id = Some(command.command_id.to_string());
                        value.params = if command.params.is_empty() { None } else {
                            let params = command.params.iter().map(|p| {
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
                                    },
                                    sspj::ssattribute::SsAttributeSignalCommandParamValue::Index(v) => {
                                        let mut i = ssab::ssab::ss::format::IntValueEntryT::default();
                                        i.value = *v;
                                        ssab::ssab::ss::format::SignalCommandParamValueT::Index(Box::new(i))
                                    },
                                    sspj::ssattribute::SsAttributeSignalCommandParamValue::Integer(v) => {
                                        let mut i = ssab::ssab::ss::format::IntValueEntryT::default();
                                        i.value = *v;
                                        ssab::ssab::ss::format::SignalCommandParamValueT::Integer(Box::new(i))
                                    },
                                    sspj::ssattribute::SsAttributeSignalCommandParamValue::Floating(v) =>{
                                        let mut f = ssab::ssab::ss::format::FloatValueEntryT::default();
                                        f.value = *v;
                                        ssab::ssab::ss::format::SignalCommandParamValueT::Floating(Box::new(f))
                                    },
                                    sspj::ssattribute::SsAttributeSignalCommandParamValue::Text(v) => {
                                        let mut s = ssab::ssab::ss::format::StringValueEntryT::default();
                                        s.value = Some(v.to_string());
                                        ssab::ssab::ss::format::SignalCommandParamValueT::Text(Box::new(s))
                                    },
                                    sspj::ssattribute::SsAttributeSignalCommandParamValue::None => {
                                        let n = ssab::ssab::ss::format::NoneValueEntryT::default();
                                        ssab::ssab::ss::format::SignalCommandParamValueT::None(Box::new(n))
                                    },
                                };

                                command_param_t
                            }).collect::<Vec<_>>();

                            Some(params)
                        };

                        initial_event_t.signal = Some(Box::new(value));
                        flag_exist_event = true;
                    });
                });
            },
            sspj::ssattribute::SsAttributeKind::Instance(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    let mut value = ssab::ssab::ss::format::PartAttributeInstanceT::default();
                    value.pingpong = _key.value.pingpong;
                    value.independent = _key.value.independent;
                    value.loop_num = _key.value.loop_num;
                    value.start_label_hash = crate::utils::fnv1a_hash_str(&_key.value.start_label);
                    value.start_offset = _key.value.start_offset;
                    value.end_label_hash = crate::utils::fnv1a_hash_str(&_key.value.end_label);
                    value.end_offset = _key.value.end_offset;
                    value.speed = _key.value.speed;

                    initial_event_t.instance = Some(Box::new(value));
                    flag_exist_event = true;
                });
            },
            sspj::ssattribute::SsAttributeKind::Effect(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    let mut value = ssab::ssab::ss::format::PartAttributeEffectT::default();
                    value.cur_keyframe;
                    value.start_time = _key.value.start_time;
                    value.speed = _key.value.speed;
                    value.loopflag;

                    initial_event_t.effect = Some(Box::new(value));
                    flag_exist_event = true;
                });
            },
            sspj::ssattribute::SsAttributeKind::Audio(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    let mut value = ssab::ssab::ss::format::PartAttributeAudioT::default();
                    value.sound_list_id = _key.value.sound_list_id;
                    value.sound_name = Some(_key.value.sound_name.to_string());
                    value.loop_num = _key.value.loop_num;

                    initial_event_t.audio = Some(Box::new(value));
                    flag_exist_event = true;
                });
            },
            _ => {
                // Empty.
            },
        }
    });

    if flag_exist_event {
        p_args[part_index as usize] = initial_event_t;
    }
}

fn convert_parts_initial_attributes(anime_data: &sspj::ssae::SsAnimation) -> Vec<ssab::ssab::ss::format::InitialPartAttributesT>{
    anime_data.part_animes.iter().map(|part_anime| {
        let mut part_attribute_t = ssab::ssab::ss::format::InitialPartAttributesT::default();
        // TODO: impl
        merge_setup_attributes_to_part_attribute_t(&mut part_attribute_t, &part_anime.attributes);

        part_attribute_t
    }).collect::<Vec<_>>()
}

fn convert_parts_initial_events(anime_data: &sspj::ssae::SsAnimation, parts_map: &HashMap<String, &sspj::ssae::SsPart>) -> Option<Vec<ssab::ssab::ss::format::InitialEventsT>> {
    let mut events_t = Vec::<ssab::ssab::ss::format::InitialEventsT>::new();
    events_t.resize(parts_map.len(), ssab::ssab::ss::format::InitialEventsT::default());

    for part_anime in &anime_data.part_animes {
        let Some(p) = parts_map.get(&part_anime.part_name) else {
            // not found part (an illegal case)
            continue;
        };

        merge_setup_attributes_to_part_event_t(&mut events_t, p.array_index as u16, &part_anime.attributes);
   }

    Some(events_t)
}

fn convert_part_animation_data_required_precalculation(interpolation:&sspj::sstypes::SsInterpolationType) -> bool {
    // Determines whether to calculate in runtime or pre-interpolation.
    // Return also false when range is need not interpolate.
    match interpolation {
        sspj::sstypes::SsInterpolationType::None => false,
        sspj::sstypes::SsInterpolationType::Linear => false,
        sspj::sstypes::SsInterpolationType::Hermite => true,
        sspj::sstypes::SsInterpolationType::Bezier => true,
        sspj::sstypes::SsInterpolationType::Acceleration => false,
        sspj::sstypes::SsInterpolationType::Deceleration => false,
        sspj::sstypes::SsInterpolationType::EaseIn => true,
        sspj::sstypes::SsInterpolationType::EaseOut => true,
        sspj::sstypes::SsInterpolationType::EaseInOut => true,
        sspj::sstypes::SsInterpolationType::EaseExponentialIn => false,
        sspj::sstypes::SsInterpolationType::EaseExponentialOut => false,
        sspj::sstypes::SsInterpolationType::EaseExponentialInOut => false,
        sspj::sstypes::SsInterpolationType::EaseSineIn => false,
        sspj::sstypes::SsInterpolationType::EaseSineOut => false,
        sspj::sstypes::SsInterpolationType::EaseSineInOut => false,
        sspj::sstypes::SsInterpolationType::EaseElasticIn => true,
        sspj::sstypes::SsInterpolationType::EaseElasticOut => true,
        sspj::sstypes::SsInterpolationType::EaseElasticInOut => true,
        sspj::sstypes::SsInterpolationType::EaseBounceIn => false,
        sspj::sstypes::SsInterpolationType::EaseBounceOut => false,
        sspj::sstypes::SsInterpolationType::EaseBounceInOut => false,
        sspj::sstypes::SsInterpolationType::EaseBackIn => false,
        sspj::sstypes::SsInterpolationType::EaseBackOut => false,
        sspj::sstypes::SsInterpolationType::EaseBackInOut => false,
        _ => false,
    }
}
fn convert_part_animation_data_expand_get_rate(interpolation:&sspj::sstypes::SsInterpolationType, frame_now:i32, frame_start:i32, frame_end:i32, curve:&sspj::sstypes::SsCurve, easing_rate:f32) -> f32 {
    let rate = ssinterpolate::interpolate::rate_linear(frame_now as f32, frame_start as f32, frame_end as f32);
    match interpolation {
        sspj::sstypes::SsInterpolationType::None => {
            0.0 as f32
        },
        sspj::sstypes::SsInterpolationType::Linear => {
            rate
        },
        sspj::sstypes::SsInterpolationType::Hermite => {
            ssinterpolate::interpolate::hermite(0.0_f32, 1.0_f32, rate, curve.start_value, curve.end_value)
        },
        sspj::sstypes::SsInterpolationType::Bezier => {
            ssinterpolate::interpolate::bezier(frame_start as f32, 0.0_f32, frame_end as f32, 1.0_f32, rate, curve.start_time, curve.start_value, curve.end_time, curve.end_value)
        },
        sspj::sstypes::SsInterpolationType::Acceleration => {
            ssinterpolate::interpolate::rate_accelerate(rate)
        },
        sspj::sstypes::SsInterpolationType::Deceleration => {
            ssinterpolate::interpolate::rate_decelerate(rate)
        },
        sspj::sstypes::SsInterpolationType::EaseIn => {
            ssinterpolate::interpolate::rate_ease_in(rate, easing_rate)
        },
        sspj::sstypes::SsInterpolationType::EaseOut => {
            ssinterpolate::interpolate::rate_ease_out(rate, easing_rate)
        },
        sspj::sstypes::SsInterpolationType::EaseInOut => {
            ssinterpolate::interpolate::rate_ease_inout(rate, easing_rate)
        },
        sspj::sstypes::SsInterpolationType::EaseExponentialIn => {
            ssinterpolate::interpolate::rate_ease_exponential_in(rate)
        },
        sspj::sstypes::SsInterpolationType::EaseExponentialOut => {
            ssinterpolate::interpolate::rate_ease_exponential_out(rate)
        },
        sspj::sstypes::SsInterpolationType::EaseExponentialInOut => {
            ssinterpolate::interpolate::rate_ease_exponential_inout(rate)
        },
        sspj::sstypes::SsInterpolationType::EaseSineIn => {
            ssinterpolate::interpolate::rate_ease_sine_in(rate)
        },
        sspj::sstypes::SsInterpolationType::EaseSineOut => {
            ssinterpolate::interpolate::rate_ease_sine_out(rate)
        },
        sspj::sstypes::SsInterpolationType::EaseSineInOut => {
            ssinterpolate::interpolate::rate_ease_sine_inout(rate)
        },
        sspj::sstypes::SsInterpolationType::EaseElasticIn => {
            ssinterpolate::interpolate::rate_ease_elastic_in(rate, easing_rate)
        },
        sspj::sstypes::SsInterpolationType::EaseElasticOut => {
            ssinterpolate::interpolate::rate_ease_elastic_out(rate, easing_rate)
        },
        sspj::sstypes::SsInterpolationType::EaseElasticInOut => {
            ssinterpolate::interpolate::rate_ease_elastic_inout(rate, easing_rate)
        },
        sspj::sstypes::SsInterpolationType::EaseBounceIn => {
            ssinterpolate::interpolate::rate_ease_bounce_in(rate)
        },
        sspj::sstypes::SsInterpolationType::EaseBounceOut => {
            ssinterpolate::interpolate::rate_ease_bounce_out(rate)
        },
        sspj::sstypes::SsInterpolationType::EaseBounceInOut => {
            ssinterpolate::interpolate::rate_ease_bounce_inout(rate)
        },
        sspj::sstypes::SsInterpolationType::EaseBackIn => {
            ssinterpolate::interpolate::rate_ease_back_in(rate)
        },
        sspj::sstypes::SsInterpolationType::EaseBackOut => {
            ssinterpolate::interpolate::rate_ease_back_out(rate)
        },
        sspj::sstypes::SsInterpolationType::EaseBackInOut => {
            ssinterpolate::interpolate::rate_ease_back_inout(rate)
        },
        _ => {
            0.0 as f32
        },
    }
}
fn convert_part_animation_data_expand_key_float(value_start:&sspj::ssattribute::SsAttributeFloatKey, value_end:&sspj::ssattribute::SsAttributeFloatKey, time:i32) -> f32 {
    let rate = ssinterpolate::interpolate::rate_linear(time as f32, value_start.time as f32, value_end.time as f32);
    match value_start.ip_type {
        sspj::sstypes::SsInterpolationType::None => {
            value_start.value
        },
        sspj::sstypes::SsInterpolationType::Linear => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, rate)
        },
        sspj::sstypes::SsInterpolationType::Hermite => {
            ssinterpolate::interpolate::hermite(
                value_start.value, value_end.value, rate,
                value_start.curve.start_value,
                value_start.curve.end_value
            )
        },
        sspj::sstypes::SsInterpolationType::Bezier => {
            ssinterpolate::interpolate::bezier(
                value_start.time as f32, value_start.value,
                value_end.time as f32, value_end.value,
                rate,
                value_start.curve.start_time, value_start.curve.start_value,
                value_start.curve.end_time, value_start.curve.end_value
            )
        },
        sspj::sstypes::SsInterpolationType::Acceleration => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_accelerate(rate))
        },
        sspj::sstypes::SsInterpolationType::Deceleration => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_decelerate(rate))
        },
        sspj::sstypes::SsInterpolationType::EaseIn => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_in(rate, value_start.easing_rate))
        },
        sspj::sstypes::SsInterpolationType::EaseOut => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_out(rate, value_start.easing_rate))
        },
        sspj::sstypes::SsInterpolationType::EaseInOut => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_inout(rate, value_start.easing_rate))
        },
        sspj::sstypes::SsInterpolationType::EaseExponentialIn => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_exponential_in(rate))
        },
        sspj::sstypes::SsInterpolationType::EaseExponentialOut => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_exponential_out(rate))
        },
        sspj::sstypes::SsInterpolationType::EaseExponentialInOut => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_exponential_inout(rate))
        },
        sspj::sstypes::SsInterpolationType::EaseSineIn => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_sine_in(rate))
        },
        sspj::sstypes::SsInterpolationType::EaseSineOut => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_sine_out(rate))
        },
        sspj::sstypes::SsInterpolationType::EaseSineInOut => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_sine_inout(rate))
        },
        sspj::sstypes::SsInterpolationType::EaseElasticIn => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_elastic_in(rate, value_start.easing_rate))
        },
        sspj::sstypes::SsInterpolationType::EaseElasticOut => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_elastic_out(rate, value_start.easing_rate))
        },
        sspj::sstypes::SsInterpolationType::EaseElasticInOut => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_elastic_inout(rate, value_start.easing_rate))
        },
        sspj::sstypes::SsInterpolationType::EaseBounceIn => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_bounce_in(rate))
        },
        sspj::sstypes::SsInterpolationType::EaseBounceOut => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_bounce_out(rate))
        },
        sspj::sstypes::SsInterpolationType::EaseBounceInOut => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_bounce_inout(rate))
        },
        sspj::sstypes::SsInterpolationType::EaseBackIn => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_back_in(rate))
        },
        sspj::sstypes::SsInterpolationType::EaseBackOut => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_back_out(rate))
        },
        sspj::sstypes::SsInterpolationType::EaseBackInOut => {
            ssinterpolate::interpolate::linear(value_start.value, value_end.value, ssinterpolate::interpolate::rate_ease_back_inout(rate))
        },
        _ => {
            value_start.value
        },
    }
}
fn convert_part_animation_data_expand_key_int(value_start:&sspj::ssattribute::SsAttributeIntKey, value_end:&sspj::ssattribute::SsAttributeIntKey, time:i32) -> i32 {
    let rate = ssinterpolate::interpolate::rate_linear(time as f32, value_start.time as f32, value_end.time as f32);
    match value_start.ip_type {
        sspj::sstypes::SsInterpolationType::None => {
            value_start.value
        },
        sspj::sstypes::SsInterpolationType::Linear => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, rate) as i32
        },
        sspj::sstypes::SsInterpolationType::Hermite => {
            ssinterpolate::interpolate::hermite(
                value_start.value as f32, value_end.value as f32, rate,
                value_start.curve.start_value,
                value_start.curve.end_value
            ) as i32
        },
        sspj::sstypes::SsInterpolationType::Bezier => {
            ssinterpolate::interpolate::bezier(
                value_start.time as f32, value_start.value as f32,
                value_end.time as f32, value_end.value as f32,
                rate,
                value_start.curve.start_time, value_start.curve.start_value,
                value_start.curve.end_time, value_start.curve.end_value
            ) as i32
        },
        sspj::sstypes::SsInterpolationType::Acceleration => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_accelerate(rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::Deceleration => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_decelerate(rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseIn => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_in(rate, value_start.easing_rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseOut => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_out(rate, value_start.easing_rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseInOut => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_inout(rate, value_start.easing_rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseExponentialIn => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_exponential_in(rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseExponentialOut => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_exponential_out(rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseExponentialInOut => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_exponential_inout(rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseSineIn => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_sine_in(rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseSineOut => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_sine_out(rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseSineInOut => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_sine_inout(rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseElasticIn => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_elastic_in(rate, value_start.easing_rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseElasticOut => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_elastic_out(rate, value_start.easing_rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseElasticInOut => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_elastic_inout(rate, value_start.easing_rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseBounceIn => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_bounce_in(rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseBounceOut => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_bounce_out(rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseBounceInOut => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_bounce_inout(rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseBackIn => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_back_in(rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseBackOut => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_back_out(rate)) as i32
        },
        sspj::sstypes::SsInterpolationType::EaseBackInOut => {
            ssinterpolate::interpolate::linear(value_start.value as f32, value_end.value as f32, ssinterpolate::interpolate::rate_ease_back_inout(rate)) as i32
        },
        _ => {
            value_start.value
        },
    }
}
fn convert_part_animation_data_expand_key_partcolor(value_start:&sspj::ssattribute::SsAttributePartsColorKey, value_end:&sspj::ssattribute::SsAttributePartsColorKey, time:i32) -> Option<Box<ssab::ssab::ss::format::PartAttributePartColorT>> {
    let rate_linear = convert_part_animation_data_expand_get_rate(&value_start.ip_type, time, value_start.time, value_end.time, &value_start.curve, value_start.easing_rate);

    let lt_start: &sspj::ssattribute::SsAttributePartsColorKeyValueColor;
    let rt_start: &sspj::ssattribute::SsAttributePartsColorKeyValueColor;
    let lb_start: &sspj::ssattribute::SsAttributePartsColorKeyValueColor;
    let rb_start: &sspj::ssattribute::SsAttributePartsColorKeyValueColor;
    match value_start.value.target {
        sspj::sstypes::SsColorBlendTarget::Whole => {
            lt_start = &value_start.value.color;
            rt_start = &value_start.value.color;
            lb_start = &value_start.value.color;
            rb_start = &value_start.value.color;
        },
        sspj::sstypes::SsColorBlendTarget::Vertex => {
            lt_start = &value_start.value.lt;
            rt_start = &value_start.value.rt;
            lb_start = &value_start.value.lb;
            rb_start = &value_start.value.rb;
        },
//        _ => {
//            lt_start = &value_start.value.color;
//            rt_start = &value_start.value.color;
//            lb_start = &value_start.value.color;
//            rb_start = &value_start.value.color;
//        }
    }

    let lt_end: &sspj::ssattribute::SsAttributePartsColorKeyValueColor;
    let rt_end: &sspj::ssattribute::SsAttributePartsColorKeyValueColor;
    let rb_end: &sspj::ssattribute::SsAttributePartsColorKeyValueColor;
    let lb_end: &sspj::ssattribute::SsAttributePartsColorKeyValueColor;
    match value_start.value.target {
        sspj::sstypes::SsColorBlendTarget::Whole => {
            lt_end = &value_end.value.color;
            rt_end = &value_end.value.color;
            lb_end = &value_end.value.color;
            rb_end = &value_end.value.color;
        },
        sspj::sstypes::SsColorBlendTarget::Vertex => {
            lt_end = &value_end.value.lt;
            rt_end = &value_end.value.rt;
            lb_end = &value_end.value.lb;
            rb_end = &value_end.value.rb;
        },
//        _ => {
//            lt_end = &value_end.value.color;
//            rt_end = &value_end.value.color;
//            lb_end = &value_end.value.color;
//            rb_end = &value_end.value.color;
//        }
    }

    let mut result = ssab::ssab::ss::format::PartAttributePartColorT::default();
    result.blend_type = convert_alpha_blend_type(&value_start.value.blend_type);

    let mut result_color = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
    let mut color = ssab::ssab::ss::format::U8RectT::default();
    color.a = ssinterpolate::interpolate::linear(lt_start.rgba.a as f32, lt_end.rgba.a as f32, rate_linear) as u8;
    color.r = ssinterpolate::interpolate::linear(lt_start.rgba.r as f32, lt_end.rgba.r as f32, rate_linear) as u8;
    color.g = ssinterpolate::interpolate::linear(lt_start.rgba.g as f32, lt_end.rgba.g as f32, rate_linear) as u8;
    color.b = ssinterpolate::interpolate::linear(lt_start.rgba.b as f32, lt_end.rgba.b as f32, rate_linear) as u8;
    result_color.rgba = Some(Box::new(color));
    result_color.rate = ssinterpolate::interpolate::linear(lt_start.rate, lt_end.rate, rate_linear);
    result.lt = Some(Box::new(result_color));

    result_color = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
    color = ssab::ssab::ss::format::U8RectT::default();
    color.a = ssinterpolate::interpolate::linear(rt_start.rgba.a as f32, rt_end.rgba.a as f32, rate_linear) as u8;
    color.r = ssinterpolate::interpolate::linear(rt_start.rgba.r as f32, rt_end.rgba.r as f32, rate_linear) as u8;
    color.g = ssinterpolate::interpolate::linear(rt_start.rgba.g as f32, rt_end.rgba.g as f32, rate_linear) as u8;
    color.b = ssinterpolate::interpolate::linear(rt_start.rgba.b as f32, rt_end.rgba.b as f32, rate_linear) as u8;
    result_color.rgba = Some(Box::new(color));
    result_color.rate = ssinterpolate::interpolate::linear(rt_start.rate, rt_end.rate, rate_linear);
    result.rt = Some(Box::new(result_color));

    result_color = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
    color = ssab::ssab::ss::format::U8RectT::default();
    color.a = ssinterpolate::interpolate::linear(lb_start.rgba.a as f32, lb_end.rgba.a as f32, rate_linear) as u8;
    color.r = ssinterpolate::interpolate::linear(lb_start.rgba.r as f32, lb_end.rgba.r as f32, rate_linear) as u8;
    color.g = ssinterpolate::interpolate::linear(lb_start.rgba.g as f32, lb_end.rgba.g as f32, rate_linear) as u8;
    color.b = ssinterpolate::interpolate::linear(lb_start.rgba.b as f32, lb_end.rgba.b as f32, rate_linear) as u8;
    result_color.rgba = Some(Box::new(color));
    result_color.rate = ssinterpolate::interpolate::linear(lb_start.rate, lb_end.rate, rate_linear);
    result.lb = Some(Box::new(result_color));

    result_color = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
    color = ssab::ssab::ss::format::U8RectT::default();
    color.a = ssinterpolate::interpolate::linear(rb_start.rgba.a as f32, rb_end.rgba.a as f32, rate_linear) as u8;
    color.r = ssinterpolate::interpolate::linear(rb_start.rgba.r as f32, rb_end.rgba.r as f32, rate_linear) as u8;
    color.g = ssinterpolate::interpolate::linear(rb_start.rgba.g as f32, rb_end.rgba.g as f32, rate_linear) as u8;
    color.b = ssinterpolate::interpolate::linear(rb_start.rgba.b as f32, rb_end.rgba.b as f32, rate_linear) as u8;
    result_color.rgba = Some(Box::new(color));
    result_color.rate = ssinterpolate::interpolate::linear(rb_start.rate, rb_end.rate, rate_linear);
    result.rb = Some(Box::new(result_color));

    Some(Box::new(result))
}
fn convert_part_animation_data_expand_key_shader(value_start:&sspj::ssattribute::SsAttributeShaderKey, value_end:&sspj::ssattribute::SsAttributeShaderKey, time:i32) -> Option<Box<ssab::ssab::ss::format::PartAttributeShaderT>> {
    let rate_linear = convert_part_animation_data_expand_get_rate(&value_start.ip_type, time, value_start.time, value_end.time, &value_start.curve, value_start.easing_rate);

    let mut result = ssab::ssab::ss::format::PartAttributeShaderT::default();
    result.id = Some(value_start.value.id.clone());
    result.id_hash = crate::utils::fnv1a_hash_str(&value_start.value.id);

    result.param0 = ssinterpolate::interpolate::linear(value_start.value.param0, value_end.value.param0, rate_linear);
    result.param1 = ssinterpolate::interpolate::linear(value_start.value.param1, value_end.value.param1, rate_linear);
    result.param2 = ssinterpolate::interpolate::linear(value_start.value.param2, value_end.value.param2, rate_linear);
    result.param3 = ssinterpolate::interpolate::linear(value_start.value.param3, value_end.value.param3, rate_linear);
    result.param4 = ssinterpolate::interpolate::linear(value_start.value.param4, value_end.value.param4, rate_linear);
    result.param5 = ssinterpolate::interpolate::linear(value_start.value.param5, value_end.value.param5, rate_linear);
    result.param6 = ssinterpolate::interpolate::linear(value_start.value.param6, value_end.value.param6, rate_linear);
    result.param7 = ssinterpolate::interpolate::linear(value_start.value.param7, value_end.value.param7, rate_linear);

    result.map0_id = value_start.value.map0.map_id;
    result.map0_name_hash = crate::utils::fnv1a_hash_str(&value_start.value.map0.name);
    result.map1_id = value_start.value.map1.map_id;
    result.map1_name_hash = crate::utils::fnv1a_hash_str(&value_start.value.map1.name);

    Some(Box::new(result))
}
fn convert_part_animation_data_expand_key_vertex(value_start:&sspj::ssattribute::SsAttributeVertexKey, value_end:&sspj::ssattribute::SsAttributeVertexKey, time:i32) -> Option<Box<ssab::ssab::ss::format::PartAttributeVertexT>> {
    let rate_linear = convert_part_animation_data_expand_get_rate(&value_start.ip_type, time, value_start.time, value_end.time, &value_start.curve, value_start.easing_rate);

    let mut result = ssab::ssab::ss::format::PartAttributeVertexT::default();
    let mut coordinate: ssab::ssab::ss::format::Vec2T;
    let mut coordinate_start: ssab::ssab::ss::format::Vec2T;
    let mut coordinate_end: ssab::ssab::ss::format::Vec2T;

    coordinate = ssab::ssab::ss::format::Vec2T::default();
    coordinate_start = ssab::ssab::ss::format::Vec2T::default();
    value_start.value.lt.as_ref().map(|c| {
        coordinate_start.v1 = c.x;
        coordinate_start.v2 = c.y;
    });
    coordinate_end = ssab::ssab::ss::format::Vec2T::default();
    value_end.value.lt.as_ref().map(|c| {
        coordinate_end.v1 = c.x;
        coordinate_end.v2 = c.y;
    });
    coordinate.v1 = ssinterpolate::interpolate::linear(coordinate_start.v1, coordinate_end.v1, rate_linear);
    coordinate.v2 = ssinterpolate::interpolate::linear(coordinate_start.v1, coordinate_end.v1, rate_linear);
    result.lt = Some(Box::new(coordinate));

    coordinate = ssab::ssab::ss::format::Vec2T::default();
    coordinate_start = ssab::ssab::ss::format::Vec2T::default();
    value_start.value.rt.as_ref().map(|c| {
        coordinate_start.v1 = c.x;
        coordinate_start.v2 = c.y;
    });
    coordinate_end = ssab::ssab::ss::format::Vec2T::default();
    value_end.value.rt.as_ref().map(|c| {
        coordinate_end.v1 = c.x;
        coordinate_end.v2 = c.y;
    });
    coordinate.v1 = ssinterpolate::interpolate::linear(coordinate_start.v1, coordinate_end.v1, rate_linear);
    coordinate.v2 = ssinterpolate::interpolate::linear(coordinate_start.v1, coordinate_end.v1, rate_linear);
    result.rt = Some(Box::new(coordinate));

    coordinate = ssab::ssab::ss::format::Vec2T::default();
    coordinate_start = ssab::ssab::ss::format::Vec2T::default();
    value_start.value.lb.as_ref().map(|c| {
        coordinate_start.v1 = c.x;
        coordinate_start.v2 = c.y;
    });
    coordinate_end = ssab::ssab::ss::format::Vec2T::default();
    value_end.value.lb.as_ref().map(|c| {
        coordinate_end.v1 = c.x;
        coordinate_end.v2 = c.y;
    });
    coordinate.v1 = ssinterpolate::interpolate::linear(coordinate_start.v1, coordinate_end.v1, rate_linear);
    coordinate.v2 = ssinterpolate::interpolate::linear(coordinate_start.v1, coordinate_end.v1, rate_linear);
    result.lb = Some(Box::new(coordinate));

    coordinate = ssab::ssab::ss::format::Vec2T::default();
    coordinate_start = ssab::ssab::ss::format::Vec2T::default();
    value_start.value.rb.as_ref().map(|c| {
        coordinate_start.v1 = c.x;
        coordinate_start.v2 = c.y;
    });
    coordinate_end = ssab::ssab::ss::format::Vec2T::default();
    value_end.value.rb.as_ref().map(|c| {
        coordinate_end.v1 = c.x;
        coordinate_end.v2 = c.y;
    });
    coordinate.v1 = ssinterpolate::interpolate::linear(coordinate_start.v1, coordinate_end.v1, rate_linear);
    coordinate.v2 = ssinterpolate::interpolate::linear(coordinate_start.v1, coordinate_end.v1, rate_linear);
    result.rb = Some(Box::new(coordinate));

    Some(Box::new(result))
}
fn convert_part_animation_data_expand_key_skew(value_start:&sspj::ssattribute::SsAttributeVertexKey, value_end:&sspj::ssattribute::SsAttributeVertexKey, time:i32) -> Option<Box<ssab::ssab::ss::format::Vec2T>> {
    let rate_linear = convert_part_animation_data_expand_get_rate(&value_start.ip_type, time, value_start.time, value_end.time, &value_start.curve, value_start.easing_rate);

    let mut result: ssab::ssab::ss::format::Vec2T;
    let mut coordinate_start: ssab::ssab::ss::format::Vec2T;
    let mut coordinate_end: ssab::ssab::ss::format::Vec2T;

    result = ssab::ssab::ss::format::Vec2T::default();
    coordinate_start = ssab::ssab::ss::format::Vec2T::default();
    value_start.value.skew.as_ref().map(|c| {
        coordinate_start.v1 = c.x;
        coordinate_start.v2 = c.y;
    });
    coordinate_end = ssab::ssab::ss::format::Vec2T::default();
    value_end.value.skew.as_ref().map(|c| {
        coordinate_end.v1 = c.x;
        coordinate_end.v2 = c.y;
    });
    result.v1 = ssinterpolate::interpolate::linear(coordinate_start.v1, coordinate_end.v1, rate_linear);
    result.v2 = ssinterpolate::interpolate::linear(coordinate_start.v1, coordinate_end.v1, rate_linear);

    Some(Box::new(result))
}
fn convert_part_animation_data_expand_key_deform(value_start:&sspj::ssattribute::SsAttributeDeformKey, value_end:&sspj::ssattribute::SsAttributeDeformKey, time:i32) -> Option<Vec<ssab::ssab::ss::format::Vec2T>> {
    let rate_linear = convert_part_animation_data_expand_get_rate(&value_start.ip_type, time, value_start.time, value_end.time, &value_start.curve, value_start.easing_rate);

    let mut result = Vec::new();
    let mut coordinate: ssab::ssab::ss::format::Vec2T;
    let mut coordinate_start: ssab::ssab::ss::format::Vec2T;
    let mut coordinate_end: ssab::ssab::ss::format::Vec2T;

    // MEMO: Usually, "value_start.value.size" and "value_end.value.size" are equivalent.
    let count_vertex_start = value_start.value.vsize as usize;
    let count_vertex_end = value_end.value.vsize as usize;
    let count_vertex = std::cmp::max(count_vertex_start, count_vertex_end);
    for i in 0..count_vertex {
        coordinate_start = ssab::ssab::ss::format::Vec2T::default();
        if i < count_vertex_start {
            coordinate_start.v1 = value_start.value.vchg[i].x;
            coordinate_start.v2 = value_start.value.vchg[i].y;
        }
        coordinate_end = ssab::ssab::ss::format::Vec2T::default();
        if i < count_vertex_end {
            coordinate_end.v1 = value_start.value.vchg[i].x;
            coordinate_end.v2 = value_start.value.vchg[i].y;
        }

        coordinate = ssab::ssab::ss::format::Vec2T::default();
        coordinate.v1 = ssinterpolate::interpolate::linear(coordinate_start.v1, coordinate_end.v1, rate_linear);
        coordinate.v2 = ssinterpolate::interpolate::linear(coordinate_start.v2, coordinate_end.v2, rate_linear);

        result.push(coordinate);
    }

    Some(result)
}
fn convert_part_animation_data(p_arg: &mut ssab::ssab::ss::format::PartAnimationDataT, anime_data: &sspj::ssae::SsPartAnime) {
    anime_data.attributes.iter().for_each(|attribute| {
        match attribute {
            sspj::ssattribute::SsAttributeKind::Cell(v) => {
                // MEMO: "Cell" has no interpolation.
                v.keys.iter().for_each(|key| {
                    let mut c_args = ssab::ssab::ss::format::PartAnimationDataCellValueT::default();
                    c_args.index = key.time as u16;
                    let mut value = ssab::ssab::ss::format::PartAttributeCellT::default();
                    value.map_id = key.value.map_id;
                    value.name_hash = crate::utils::fnv1a_hash_str(&key.value.name);
                    c_args.value = Some(Box::new(value));

                    let cell_array = &mut p_arg.cell_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = c_args.clone();
                            args_frame_0.index = 0;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    if let Some(pos) = cell_array.iter().position(|item| item.index == c_args.index) {
                        cell_array.remove(pos);
                        cell_array.insert(pos, c_args);
                    } else {
                        cell_array.push(c_args);
                    }
                });
            },
            sspj::ssattribute::SsAttributeKind::PosX(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut x_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    x_args.index = key.time as u16;
                    x_args.interpolation = convert_interpolation_type(&key.ip_type);
                    x_args.value = key.value;

                    let position_x_array = &mut p_arg.position_x_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = x_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = position_x_array.iter().position(|item| item.index == x_args.index) {
                        position_x_array.remove(pos);
                        position_x_array.insert(pos, x_args);
                        index_key_start = pos;
                    } else {
                        position_x_array.push(x_args);
                        index_key_start = position_x_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        position_x_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            position_x_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::PosY(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut y_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    y_args.index = key.time as u16;
                    y_args.interpolation = convert_interpolation_type(&key.ip_type);
                    y_args.value = key.value;

                    let position_y_array = &mut p_arg.position_y_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = y_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = position_y_array.iter().position(|item| item.index == y_args.index) {
                        position_y_array.remove(pos);
                        position_y_array.insert(pos, y_args);
                        index_key_start = pos;
                    } else {
                        position_y_array.push(y_args);
                        index_key_start = position_y_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        position_y_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            position_y_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::PosZ(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut z_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    z_args.index = key.time as u16;
                    z_args.interpolation = convert_interpolation_type(&key.ip_type);
                    z_args.value = key.value;

                    let position_z_array = &mut p_arg.position_z_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = z_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = position_z_array.iter().position(|item| item.index == z_args.index) {
                        position_z_array.remove(pos);
                        position_z_array.insert(pos, z_args);
                        index_key_start = pos;
                    } else {
                        position_z_array.push(z_args);
                        index_key_start = position_z_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        position_z_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            position_z_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::RotX(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut x_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    x_args.index = key.time as u16;
                    x_args.interpolation = convert_interpolation_type(&key.ip_type);
                    x_args.value = key.value;

                    let rotation_x_array = &mut p_arg.rotation_x_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = x_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = rotation_x_array.iter().position(|item| item.index == x_args.index) {
                        rotation_x_array.remove(pos);
                        rotation_x_array.insert(pos, x_args);
                        index_key_start = pos;
                    } else {
                        rotation_x_array.push(x_args);
                        index_key_start = rotation_x_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        rotation_x_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            rotation_x_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::RotY(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut y_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    y_args.index = key.time as u16;
                    y_args.interpolation = convert_interpolation_type(&key.ip_type);
                    y_args.value = key.value;

                    let rotation_y_array = &mut p_arg.rotation_y_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = y_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = rotation_y_array.iter().position(|item| item.index == y_args.index) {
                        rotation_y_array.remove(pos);
                        rotation_y_array.insert(pos, y_args);
                        index_key_start = pos;
                    } else {
                        rotation_y_array.push(y_args);
                        index_key_start = rotation_y_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        rotation_y_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            rotation_y_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::RotZ(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut z_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    z_args.index = key.time as u16;
                    z_args.interpolation = convert_interpolation_type(&key.ip_type);
                    z_args.value = key.value;

                    let rotation_z_array = &mut p_arg.rotation_z_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = z_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = rotation_z_array.iter().position(|item| item.index == z_args.index) {
                        rotation_z_array.remove(pos);
                        rotation_z_array.insert(pos, z_args);
                        index_key_start = pos;
                    } else {
                        rotation_z_array.push(z_args);
                        index_key_start = rotation_z_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        rotation_z_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            rotation_z_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::SclX(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut x_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    x_args.index = key.time as u16;
                    x_args.interpolation = convert_interpolation_type(&key.ip_type);
                    x_args.value = key.value;

                    let scale_x_array = &mut p_arg.scale_x_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = x_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = scale_x_array.iter().position(|item| item.index == x_args.index) {
                        scale_x_array.remove(pos);
                        scale_x_array.insert(pos, x_args);
                        index_key_start = pos;
                    } else {
                        scale_x_array.push(x_args);
                        index_key_start = scale_x_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        scale_x_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            scale_x_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::SclY(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut y_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    y_args.index = key.time as u16;
                    y_args.interpolation = convert_interpolation_type(&key.ip_type);
                    y_args.value = key.value;

                    let scale_y_array = &mut p_arg.scale_y_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = y_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = scale_y_array.iter().position(|item| item.index == y_args.index) {
                        scale_y_array.remove(pos);
                        scale_y_array.insert(pos, y_args);
                        index_key_start = pos;
                    } else {
                        scale_y_array.push(y_args);
                        index_key_start = scale_y_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        scale_y_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            scale_y_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::LosclX(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut x_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    x_args.index = key.time as u16;
                    x_args.interpolation = convert_interpolation_type(&key.ip_type);
                    x_args.value = key.value;

                    let local_scale_x_array = &mut p_arg.local_scale_x_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = x_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = local_scale_x_array.iter().position(|item| item.index == x_args.index) {
                        local_scale_x_array.remove(pos);
                        local_scale_x_array.insert(pos, x_args);
                        index_key_start = pos;
                    } else {
                        local_scale_x_array.push(x_args);
                        index_key_start = local_scale_x_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        local_scale_x_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            local_scale_x_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::LosclY(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut y_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    y_args.index = key.time as u16;
                    y_args.interpolation = convert_interpolation_type(&key.ip_type);
                    y_args.value = key.value;

                    let local_scale_y_array = &mut p_arg.local_scale_y_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = y_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = local_scale_y_array.iter().position(|item| item.index == y_args.index) {
                        local_scale_y_array.remove(pos);
                        local_scale_y_array.insert(pos, y_args);
                        index_key_start = pos;
                    } else {
                        local_scale_y_array.push(y_args);
                        index_key_start = local_scale_y_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        local_scale_y_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            local_scale_y_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::Alpha(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut a_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    a_args.index = key.time as u16;
                    a_args.interpolation = convert_interpolation_type(&key.ip_type);
                    a_args.value = key.value;

                    let alpha_array = &mut p_arg.alpha_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = a_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = alpha_array.iter().position(|item| item.index == a_args.index) {
                        alpha_array.remove(pos);
                        alpha_array.insert(pos, a_args);
                        index_key_start = pos;
                    } else {
                        alpha_array.push(a_args);
                        index_key_start = alpha_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        alpha_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            alpha_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::LoAlpha(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut a_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    a_args.index = key.time as u16;
                    a_args.interpolation = convert_interpolation_type(&key.ip_type);
                    a_args.value = key.value;

                    let local_alpha_array = &mut p_arg.local_alpha_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = a_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = local_alpha_array.iter().position(|item| item.index == a_args.index) {
                        local_alpha_array.remove(pos);
                        local_alpha_array.insert(pos, a_args);
                        index_key_start = pos;
                    } else {
                        local_alpha_array.push(a_args);
                        index_key_start = local_alpha_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        local_alpha_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            local_alpha_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::Prio(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut p_args = ssab::ssab::ss::format::PartAnimationDataIntValueT::default();
                    p_args.index = key.time as u16;
                    p_args.interpolation = convert_interpolation_type(&key.ip_type);
                    p_args.value = key.value;

                    let priority_array = &mut p_arg.priority_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = p_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = priority_array.iter().position(|item| item.index == p_args.index) {
                        priority_array.remove(pos);
                        priority_array.insert(pos, p_args);
                        index_key_start = pos;
                    } else {
                        priority_array.push(p_args);
                        index_key_start = priority_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        priority_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataIntValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_int(key, key_next, time_now);
                            priority_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::FlipH(v) => {
                // MEMO: "FlipH" has no interpolation.
                v.keys.iter().for_each(|key| {
                    let mut h_args = ssab::ssab::ss::format::PartAnimationDataBoolValueT::default();
                    h_args.index = key.time as u16;
                    h_args.value = key.value;

                    let flip_h_array = &mut p_arg.flip_h_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = h_args.clone();
                            args_frame_0.index = 0;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    if let Some(pos) = flip_h_array.iter().position(|item| item.index == h_args.index) {
                        flip_h_array.remove(pos);
                        flip_h_array.insert(pos, h_args);
                    } else {
                        flip_h_array.push(h_args);
                    }
               });
            },
            sspj::ssattribute::SsAttributeKind::FlipV(v) => {
                // MEMO: "FlipV" has no interpolation.
                v.keys.iter().for_each(|key| {
                    let mut v_args = ssab::ssab::ss::format::PartAnimationDataBoolValueT::default();
                    v_args.index = key.time as u16;
                    v_args.value = key.value;

                    let flip_v_array = &mut p_arg.flip_v_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = v_args.clone();
                            args_frame_0.index = 0;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    if let Some(pos) = flip_v_array.iter().position(|item| item.index == v_args.index) {
                        flip_v_array.remove(pos);
                        flip_v_array.insert(pos, v_args);
                    } else {
                        flip_v_array.push(v_args);
                    }
                });
            },
            sspj::ssattribute::SsAttributeKind::Hide(v) => {
                // MEMO: "Hide" has no interpolation.
                v.keys.iter().for_each(|key| {
                    let mut h_args = ssab::ssab::ss::format::PartAnimationDataBoolValueT::default();
                    h_args.index = key.time as u16;
                    h_args.value = key.value;

                    let hide_array = &mut p_arg.hide_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = h_args.clone();
                            args_frame_0.index = 0;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    if let Some(pos) = hide_array.iter().position(|item| item.index == h_args.index) {
                        hide_array.remove(pos);
                        hide_array.insert(pos, h_args);
                    } else {
                        hide_array.push(h_args);
                    }
                });
            },
            sspj::ssattribute::SsAttributeKind::PartsColor(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut c_args = ssab::ssab::ss::format::PartAnimationDataPartColorT::default();
                    c_args.index = key.time as u16;
                    let mut value = ssab::ssab::ss::format::PartAttributePartColorT::default();
                    value.blend_type = convert_alpha_blend_type(&key.value.blend_type);
                    let mut key_value;
                    let mut rgba;
                    key_value = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
                    rgba = ssab::ssab::ss::format::U8RectT::default();
                    rgba.a = key.value.lt.rgba.a;
                    rgba.r = key.value.lt.rgba.r;
                    rgba.g = key.value.lt.rgba.g;
                    rgba.b = key.value.lt.rgba.b;
                    key_value.rgba = Some(Box::new(rgba));
                    key_value.rate = key.value.lt.rate;
                    value.lt = Some(Box::new(key_value));

                    key_value = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
                    rgba = ssab::ssab::ss::format::U8RectT::default();
                    rgba.a = key.value.rt.rgba.a;
                    rgba.r = key.value.rt.rgba.r;
                    rgba.g = key.value.rt.rgba.g;
                    rgba.b = key.value.rt.rgba.b;
                    key_value.rgba = Some(Box::new(rgba));
                    key_value.rate = key.value.rt.rate;
                    value.rt = Some(Box::new(key_value));

                    key_value = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
                    rgba = ssab::ssab::ss::format::U8RectT::default();
                    rgba.a = key.value.lb.rgba.a;
                    rgba.r = key.value.lb.rgba.r;
                    rgba.g = key.value.lb.rgba.g;
                    rgba.b = key.value.lb.rgba.b;
                    key_value.rgba = Some(Box::new(rgba));
                    key_value.rate = key.value.lb.rate;
                    value.lb = Some(Box::new(key_value));

                    key_value = ssab::ssab::ss::format::SsAttributePartColorKeyValueColorT::default();
                    rgba = ssab::ssab::ss::format::U8RectT::default();
                    rgba.a = key.value.rb.rgba.a;
                    rgba.r = key.value.rb.rgba.r;
                    rgba.g = key.value.rb.rgba.g;
                    rgba.b = key.value.rb.rgba.b;
                    key_value.rgba = Some(Box::new(rgba));
                    key_value.rate = key.value.rb.rate;
                    value.rb = Some(Box::new(key_value));

                    c_args.value = Some(Box::new(value));

                    let part_color_array = &mut p_arg.part_color_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = c_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = part_color_array.iter().position(|item| item.index == c_args.index) {
                        part_color_array.remove(pos);
                        part_color_array.insert(pos, c_args);
                        index_key_start = pos;
                    } else {
                        part_color_array.push(c_args);
                        index_key_start = part_color_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        part_color_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataPartColorT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_partcolor(key, key_next, time_now);
                            part_color_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::Color => {
                // Unsupported (Obsolete) Attribute. (Transited to "Part-Color".)
            },
            sspj::ssattribute::SsAttributeKind::Shader(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut s_args = ssab::ssab::ss::format::PartAnimationDataShaderT::default();
                    s_args.index = key.time as u16;
                    s_args.interpolation = convert_interpolation_type(&key.ip_type);
                    let mut value = ssab::ssab::ss::format::PartAttributeShaderT::default();
                    value.id = Some(key.value.id.clone());
                    value.id_hash = crate::utils::fnv1a_hash_str(&key.value.id);
                    value.param0 = key.value.param0;
                    value.param1 = key.value.param1;
                    value.param2 = key.value.param2;
                    value.param3 = key.value.param3;
                    value.param4 = key.value.param4;
                    value.param5 = key.value.param5;
                    value.param6 = key.value.param6;
                    value.param7 = key.value.param7;
                    value.map0_id = key.value.map0.map_id;
                    value.map0_name_hash = crate::utils::fnv1a_hash_str(&key.value.map0.name);
                    value.map1_id = key.value.map1.map_id;
                    value.map1_name_hash = crate::utils::fnv1a_hash_str(&key.value.map1.name);
                    s_args.value = Some(Box::new(value));

                    let shader_array = &mut p_arg.shader_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = s_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = shader_array.iter().position(|item| item.index == s_args.index) {
                        shader_array.remove(pos);
                        shader_array.insert(pos, s_args);
                        index_key_start = pos;
                    } else {
                        shader_array.push(s_args);
                        index_key_start = shader_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        shader_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataShaderT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_shader(key, key_next, time_now);
                            shader_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::Vertex(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    if key.value.use_skew.is_some() && key.value.use_skew.unwrap() == true {
                        // Skew
                        let mut s_args = ssab::ssab::ss::format::PartAnimationDataVec2T::default();
                        s_args.index = key.time as u16;
                        s_args.interpolation = convert_interpolation_type(&key.ip_type);
                        s_args.value = key.value.skew.as_ref().map(|v| {
                            let mut value = ssab::ssab::ss::format::Vec2T::default();
                            value.v1 = v.x;
                            value.v2 = v.y;
                            Box::new(value)
                        });

                        let skew_array = &mut p_arg.skew_array.get_or_insert_with(|| {
                            let mut array_new = Vec::new();

                            // MEMO: Backfill if no key is in frame 0. (No interpolation)
                            if key.time > 0 {
                                let mut args_frame_0 = s_args.clone();
                                args_frame_0.index = 0;
                                args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                                array_new.push(args_frame_0);
                            }

                            array_new
                        });

                        let index_key_start: usize;
                        if let Some(pos) = skew_array.iter().position(|item| item.index == s_args.index) {
                            skew_array.remove(pos);
                            skew_array.insert(pos, s_args);
                            index_key_start = pos;
                        } else {
                            skew_array.push(s_args);
                            index_key_start = skew_array.len() - 1;
                        }
                        if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                            let key_next = &v.keys[i + 1];

                            skew_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                            for time_now in (key.time + 1)..key_next.time {
                                let mut args_now = ssab::ssab::ss::format::PartAnimationDataVec2T::default();
                                args_now.index = time_now as u16;
                                args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                                args_now.value = convert_part_animation_data_expand_key_skew(key, key_next, time_now);
                                skew_array.push(args_now);
                            }
                        }
                    } else {
                        // Vertex
                        let mut v_args = ssab::ssab::ss::format::PartAnimationDataVertexT::default();
                        v_args.index = key.time as u16;
                        v_args.interpolation = convert_interpolation_type(&key.ip_type);
                        let mut value = ssab::ssab::ss::format::PartAttributeVertexT::default();
                        let mut c: ssab::ssab::ss::format::Vec2T;
                        c = ssab::ssab::ss::format::Vec2T::default();
                        key.value.lt.as_ref().map(|lt| {
                            c.v1 = lt.x;
                            c.v2 = lt.y;
                        });
                        value.lt = Some(Box::new(c));
                        c = ssab::ssab::ss::format::Vec2T::default();
                        key.value.rt.as_ref().map(|rt| {
                            c.v1 = rt.x;
                            c.v2 = rt.y;
                        });
                        value.rt = Some(Box::new(c));

                        c = ssab::ssab::ss::format::Vec2T::default();
                        key.value.lb.as_ref().map(|lb| {
                            c.v1 = lb.x;
                            c.v2 = lb.y;
                        });
                        value.lb = Some(Box::new(c));

                        c = ssab::ssab::ss::format::Vec2T::default();
                        key.value.rb.as_ref().map(|rb| {
                            c.v1 = rb.x;
                            c.v2 = rb.y;
                        });
                        value.rb = Some(Box::new(c));

                        v_args.value = Some(Box::new(value));

                        let vertex_array = &mut p_arg.vertex_array.get_or_insert_with(|| {
                            let mut array_new = Vec::new();

                            // MEMO: Backfill if no key is in frame 0. (No interpolation)
                            if key.time > 0 {
                                let mut args_frame_0 = v_args.clone();
                                args_frame_0.index = 0;
                                args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                                array_new.push(args_frame_0);
                            }

                            array_new
                        });

                        let index_key_start: usize;
                        if let Some(pos) = vertex_array.iter().position(|item| item.index == v_args.index) {
                            vertex_array.remove(pos);
                            vertex_array.insert(pos, v_args);
                            index_key_start = pos;
                        } else {
                            vertex_array.push(v_args);
                            index_key_start = vertex_array.len() - 1;
                        }
                        if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                            let key_next = &v.keys[i + 1];

                            vertex_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                            for time_now in (key.time + 1)..key_next.time {
                                let mut args_now = ssab::ssab::ss::format::PartAnimationDataVertexT::default();
                                args_now.index = time_now as u16;
                                args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                                args_now.value = convert_part_animation_data_expand_key_vertex(key, key_next, time_now);
                                vertex_array.push(args_now);
                            }
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::PivotX(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut x_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    x_args.index = key.time as u16;
                    x_args.interpolation = convert_interpolation_type(&key.ip_type);
                    x_args.value = key.value;

                    let pivot_x_array = &mut p_arg.pivot_x_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = x_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = pivot_x_array.iter().position(|item| item.index == x_args.index) {
                        pivot_x_array.remove(pos);
                        pivot_x_array.insert(pos, x_args);
                        index_key_start = pos;
                    } else {
                        pivot_x_array.push(x_args);
                        index_key_start = pivot_x_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        pivot_x_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            pivot_x_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::PivotY(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut y_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    y_args.index = key.time as u16;
                    y_args.interpolation = convert_interpolation_type(&key.ip_type);
                    y_args.value = key.value;

                    let pivot_y_array = &mut p_arg.pivot_y_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = y_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = pivot_y_array.iter().position(|item| item.index == y_args.index) {
                        pivot_y_array.remove(pos);
                        pivot_y_array.insert(pos, y_args);
                        index_key_start = pos;
                    } else {
                        pivot_y_array.push(y_args);
                        index_key_start = pivot_y_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        pivot_y_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            pivot_y_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::AnchorX(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut x_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    x_args.index = key.time as u16;
                    x_args.interpolation = convert_interpolation_type(&key.ip_type);
                    x_args.value = key.value;

                    let anchor_x_array = &mut p_arg.anchor_x_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = x_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = anchor_x_array.iter().position(|item| item.index == x_args.index) {
                        anchor_x_array.remove(pos);
                        anchor_x_array.insert(pos, x_args);
                        index_key_start = pos;
                    } else {
                        anchor_x_array.push(x_args);
                        index_key_start = anchor_x_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        anchor_x_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            anchor_x_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::AnchorY(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut y_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    y_args.index = key.time as u16;
                    y_args.interpolation = convert_interpolation_type(&key.ip_type);
                    y_args.value = key.value;

                    let anchor_y_array = &mut p_arg.anchor_y_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = y_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = anchor_y_array.iter().position(|item| item.index == y_args.index) {
                        anchor_y_array.remove(pos);
                        anchor_y_array.insert(pos, y_args);
                        index_key_start = pos;
                    } else {
                        anchor_y_array.push(y_args);
                        index_key_start = anchor_y_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        anchor_y_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            anchor_y_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::SizeX(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut x_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    x_args.index = key.time as u16;
                    x_args.interpolation = convert_interpolation_type(&key.ip_type);
                    x_args.value = key.value;

                    let size_x_array = &mut p_arg.size_x_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = x_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = size_x_array.iter().position(|item| item.index == x_args.index) {
                        size_x_array.remove(pos);
                        size_x_array.insert(pos, x_args);
                        index_key_start = pos;
                    } else {
                        size_x_array.push(x_args);
                        index_key_start = size_x_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        size_x_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            size_x_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::SizeY(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut y_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    y_args.index = key.time as u16;
                    y_args.interpolation = convert_interpolation_type(&key.ip_type);
                    y_args.value = key.value;

                    let size_y_array = &mut p_arg.size_y_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = y_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = size_y_array.iter().position(|item| item.index == y_args.index) {
                        size_y_array.remove(pos);
                        size_y_array.insert(pos, y_args);
                        index_key_start = pos;
                    } else {
                        size_y_array.push(y_args);
                        index_key_start = size_y_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        size_y_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            size_y_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::ImgFlipH(v) => {
                // MEMO: "ImgFlipH" has no interpolation.
                v.keys.iter().for_each(|key| {
                    let mut h_args = ssab::ssab::ss::format::PartAnimationDataBoolValueT::default();
                    h_args.index = key.time as u16;
                    h_args.value = key.value;

                    let img_flip_h_array = &mut p_arg.img_flip_h_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = h_args.clone();
                            args_frame_0.index = 0;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    if let Some(pos) = img_flip_h_array.iter().position(|item| item.index == h_args.index) {
                        img_flip_h_array.remove(pos);
                        img_flip_h_array.insert(pos, h_args);
                    } else {
                        img_flip_h_array.push(h_args);
                    }
                });
            },
            sspj::ssattribute::SsAttributeKind::ImgFlipV(v) => {
                // MEMO: "ImgFlipV" has no interpolation.
                v.keys.iter().for_each(|key| {
                    let mut v_args = ssab::ssab::ss::format::PartAnimationDataBoolValueT::default();
                    v_args.index = key.time as u16;
                    v_args.value = key.value;

                    let img_flip_v_array = &mut p_arg.img_flip_v_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = v_args.clone();
                            args_frame_0.index = 0;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    if let Some(pos) = img_flip_v_array.iter().position(|item| item.index == v_args.index) {
                        img_flip_v_array.remove(pos);
                        img_flip_v_array.insert(pos, v_args);
                    } else {
                        img_flip_v_array.push(v_args);
                    }
                });
            },
            sspj::ssattribute::SsAttributeKind::UvtX(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut x_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    x_args.index = key.time as u16;
                    x_args.interpolation = convert_interpolation_type(&key.ip_type);
                    x_args.value = key.value;

                    let uv_translation_x_array = &mut p_arg.uv_translation_x_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = x_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = uv_translation_x_array.iter().position(|item| item.index == x_args.index) {
                        uv_translation_x_array.remove(pos);
                        uv_translation_x_array.insert(pos, x_args);
                        index_key_start = pos;
                    } else {
                        uv_translation_x_array.push(x_args);
                        index_key_start = uv_translation_x_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        uv_translation_x_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            uv_translation_x_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::UvtY(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut y_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    y_args.index = key.time as u16;
                    y_args.interpolation = convert_interpolation_type(&key.ip_type);
                    y_args.value = key.value;

                    let uv_translation_y_array = &mut p_arg.uv_translation_y_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = y_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = uv_translation_y_array.iter().position(|item| item.index == y_args.index) {
                        uv_translation_y_array.remove(pos);
                        uv_translation_y_array.insert(pos, y_args);
                        index_key_start = pos;
                    } else {
                        uv_translation_y_array.push(y_args);
                        index_key_start = uv_translation_y_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        uv_translation_y_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            uv_translation_y_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::UvrZ(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut z_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    z_args.index = key.time as u16;
                    z_args.interpolation = convert_interpolation_type(&key.ip_type);
                    z_args.value = key.value;

                    let uv_rotation_z_array = &mut p_arg.uv_rotation_z_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = z_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = uv_rotation_z_array.iter().position(|item| item.index == z_args.index) {
                        uv_rotation_z_array.remove(pos);
                        uv_rotation_z_array.insert(pos, z_args);
                        index_key_start = pos;
                    } else {
                        uv_rotation_z_array.push(z_args);
                        index_key_start = uv_rotation_z_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        uv_rotation_z_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            uv_rotation_z_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::UvsX(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut x_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    x_args.index = key.time as u16;
                    x_args.interpolation = convert_interpolation_type(&key.ip_type);
                    x_args.value = key.value;

                    let uv_scale_x_array = &mut p_arg.uv_scale_x_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = x_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = uv_scale_x_array.iter().position(|item| item.index == x_args.index) {
                        uv_scale_x_array.remove(pos);
                        uv_scale_x_array.insert(pos, x_args);
                        index_key_start = pos;
                    } else {
                        uv_scale_x_array.push(x_args);
                        index_key_start = uv_scale_x_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        uv_scale_x_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            uv_scale_x_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::UvsY(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut y_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    y_args.index = key.time as u16;
                    y_args.interpolation = convert_interpolation_type(&key.ip_type);
                    y_args.value = key.value;

                    let uv_scale_y_array = &mut p_arg.uv_scale_y_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = y_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = uv_scale_y_array.iter().position(|item| item.index == y_args.index) {
                        uv_scale_y_array.remove(pos);
                        uv_scale_y_array.insert(pos, y_args);
                        index_key_start = pos;
                    } else {
                        uv_scale_y_array.push(y_args);
                        index_key_start = uv_scale_y_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        uv_scale_y_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            uv_scale_y_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::Boundr(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut b_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    b_args.index = key.time as u16;
                    b_args.interpolation = convert_interpolation_type(&key.ip_type);
                    b_args.value = key.value;

                    let bounding_radius_array = &mut p_arg.bounding_radius_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = b_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = bounding_radius_array.iter().position(|item| item.index == b_args.index) {
                        bounding_radius_array.remove(pos);
                        bounding_radius_array.insert(pos, b_args);
                        index_key_start = pos;
                    } else {
                        bounding_radius_array.push(b_args);
                        index_key_start = bounding_radius_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        bounding_radius_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            bounding_radius_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::Mask(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut m_args = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                    m_args.index = key.time as u16;
                    m_args.interpolation = convert_interpolation_type(&key.ip_type);
                    m_args.value = key.value;

                    let mask_array = &mut p_arg.mask_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = m_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = mask_array.iter().position(|item| item.index == m_args.index) {
                        mask_array.remove(pos);
                        mask_array.insert(pos, m_args);
                        index_key_start = pos;
                    } else {
                        mask_array.push(m_args);
                        index_key_start = mask_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        mask_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataFloatValueT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = convert_part_animation_data_expand_key_float(key, key_next, time_now);
                            mask_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::Deform(v) => {
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut d_args = ssab::ssab::ss::format::PartAnimationDataDeformT::default();
                    d_args.index = key.time as u16;
                    d_args.interpolation = convert_interpolation_type(&key.ip_type);
                    if key.value.vchg.is_empty() {
                        d_args.value = None;
                    } else {
                        let mut coordinate = Vec::new();
                        let mut index = Vec::<i16>::new();
                        key.value.vchg.iter().for_each(|item| {
                            let mut vec2 = ssab::ssab::ss::format::Vec2T::default();
                            vec2.v1 = item.x;
                            vec2.v2 = item.y;
                            coordinate.push(vec2);
                            index.push(item.x_index as i16);
                        });
                        let mut value = ssab::ssab::ss::format::PartAttributeDeformT::default();
                        value.coordinate = Some(coordinate);
                        value.index = Some(index);
                        d_args.value = Some(Box::new(value));
                    }

                    let deform_array = &mut p_arg.deform_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = d_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    let index_key_start: usize;
                    if let Some(pos) = deform_array.iter().position(|item| item.index == d_args.index) {
                        deform_array.remove(pos);
                        deform_array.insert(pos, d_args);
                        index_key_start = pos;
                    } else {
                        deform_array.push(d_args);
                        index_key_start = deform_array.len() - 1;
                    }
                    if i < (count_keys - 1) && convert_part_animation_data_required_precalculation(&key.ip_type) {  // Expand between 2 keys
                        let key_next = &v.keys[i + 1];

                        deform_array[index_key_start].interpolation = ssab::ssab::ss::format::InterpolationType::None;  // overwrite beginning of range.

                        for time_now in (key.time + 1)..key_next.time {
                            let mut value = ssab::ssab::ss::format::PartAttributeDeformT::default();
                            value.coordinate = convert_part_animation_data_expand_key_deform(key, key_next, time_now);
                            // MEMO: Basically, vertex-indices table consists of the same values as other key-data (in the same part).
                            value.index = (deform_array[index_key_start].value.clone()).unwrap().index;

                            let mut args_now = ssab::ssab::ss::format::PartAnimationDataDeformT::default();
                            args_now.index = time_now as u16;
                            args_now.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            args_now.value = Some(Box::new(value));
                            deform_array.push(args_now);
                        }
                    }
                }
            },
            sspj::ssattribute::SsAttributeKind::Texchange(v) => {
                // MEMO: "Texchange" has no interpolation.
                let count_keys = v.keys.len();
                for i in 0..count_keys {
                    let key = &v.keys[i];

                    let mut t_args = ssab::ssab::ss::format::PartAnimationDataUIntValueT::default();
                    t_args.index = key.time as u16;
                    t_args.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                    t_args.value = crate::utils::fnv1a_hash_str(&key.value.texture_name);

                    let texture_array = &mut p_arg.texture_array.get_or_insert_with(|| {
                        let mut array_new = Vec::new();

                        // MEMO: Backfill if no key is in frame 0. (No interpolation)
                        if key.time > 0 {
                            let mut args_frame_0 = t_args.clone();
                            args_frame_0.index = 0;
                            args_frame_0.interpolation = ssab::ssab::ss::format::InterpolationType::None;
                            array_new.push(args_frame_0);
                        }

                        array_new
                    });

                    if let Some(pos) = texture_array.iter().position(|item| item.index == t_args.index) {
                        texture_array.remove(pos);
                        texture_array.insert(pos, t_args);
                    } else {
                        texture_array.push(t_args);
                    }
               }
            },
            _ => {},
        }
    });
}

fn modify_part_animation_data_value_changing_range(interpolation:&ssab::ssab::ss::format::InterpolationType) -> bool {
    // Check whether value change in interpolating even if range-begin and range-end values are the same.

    // MEMO: Inherently, while Hermite and Bezier curves change value in range even if beginning and ending values are the same.
    //       * However, currently, in the case of Hermite and Bezier curves, key-data are   baked (pre-interpolated).
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
fn modify_part_animation_data_float(p_attr: &mut Option<Vec<ssab::ssab::ss::format::PartAnimationDataFloatValueT>>) {
    if let Some(attribute) = p_attr {
        // Delete key-data that values do not change.
        let mut is_value_change_previous = true;

        let mut index_previous = 0;
        let mut index = 1;
        while index < attribute.len() {
            let key = &attribute[index];
            let key_previous = &attribute[index_previous];

            let mut is_value_change = true;
            let mut is_unnecessary_previous = false;
            if key.value == key_previous.value {    // Range in:[-1]/out:[0] values are not changing
                is_value_change = modify_part_animation_data_value_changing_range(&key_previous.interpolation);
                if !is_value_change_previous && !is_value_change  {    // Range [-2][-1][0] values are not changing.
                    // If interpolation has value not changing, the previous key:[-1] is unnecessary.
                    is_unnecessary_previous = true;
                }
            }

            if !is_value_change {
                // MEMO: Get previous-key-data[-1] separately for overwriting.
                if let Some(key_previous_mut) = attribute.get_mut(index_previous) {
                    key_previous_mut.interpolation = ssab::ssab::ss::format::InterpolationType::None;  // Disable interpolation
                }
            }
            if is_unnecessary_previous {
                attribute.remove(index_previous);    // Delete key-data:[-1].

                // "index" and "index_previous" must not increment (must not change).
            } else {
                index_previous = index;
                index += 1;
            }

            is_value_change_previous = is_value_change;
        }

        // MEMO: When the interpolation for the last range is either Bezier or Hermite, there are case that the last key-data is not optimized (rare cases).
        //       * Improving precise of function "modify_part_animation_data_value_changing_range" should probably resolve the issue.
        if is_value_change_previous {
            attribute.remove(index_previous);    // Delete key-data.
        }
        if let Some(key) = attribute.last_mut() {   // Remove trailing interpolation specifications (ineffective).
            key.interpolation = ssab::ssab::ss::format::InterpolationType::None;
        }

        // Remove empty
        if attribute.is_empty() {
            *p_attr = None;
        }
    }
}
fn modify_part_animation_data_int(p_attr: &mut Option<Vec<ssab::ssab::ss::format::PartAnimationDataIntValueT>>) {
    if let Some(attribute) = p_attr {
        // Delete key-data that values do not change.
        let mut is_value_change_previous = true;

        let mut index_previous = 0;
        let mut index = 1;
        while index < attribute.len() {
            let key = &attribute[index];
            let key_previous = &attribute[index_previous];

            let mut is_value_change = true;
            let mut is_unnecessary_previous = false;
            if key.value == key_previous.value {    // Range in:[-1]/out:[0] values are not changing
                is_value_change = modify_part_animation_data_value_changing_range(&key_previous.interpolation);
                if !is_value_change_previous && !is_value_change  {    // Range [-2][-1][0] values are not changing.
                    // If interpolation has value not changing, the previous key:[-1] is unnecessary.
                    is_unnecessary_previous = true;
                }
            }

            if !is_value_change {
                // MEMO: Get previous-key-data[-1] separately for overwriting.
                if let Some(key_previous_mut) = attribute.get_mut(index_previous) {
                    key_previous_mut.interpolation = ssab::ssab::ss::format::InterpolationType::None;  // Disable interpolation
                }
            }
            if is_unnecessary_previous {
                attribute.remove(index_previous);    // Delete key-data:[-1].

                // "index" and "index_previous" must not increment (must not change).
            } else {
                index_previous = index;
                index += 1;
            }

            is_value_change_previous = is_value_change;
        }

        // MEMO: When the interpolation for the last range is either Bezier or Hermite, there are case that the last key-data is not optimized (rare cases).
        //       * Improving precise of function "modify_part_animation_data_value_changing_range" should probably resolve the issue.
        if is_value_change_previous {
            attribute.remove(index_previous);    // Delete key-data.
        }
        if let Some(key) = attribute.last_mut() {   // Remove trailing interpolation specifications (ineffective).
            key.interpolation = ssab::ssab::ss::format::InterpolationType::None;
        }

        // Remove empty
        if attribute.is_empty() {
            *p_attr = None;
        }
    }
}
fn modify_part_animation_data_bool(p_attr: &mut Option<Vec<ssab::ssab::ss::format::PartAnimationDataBoolValueT>>) {
    // MEMO: Since "bool-value" has no interpolation, simply compares current to previous.
    if let Some(attribute) = p_attr {
        // Delete key-data that values do not change.
        let mut index_previous = 0;
        let mut index = 1;
        while index < attribute.len() {
            let key = &attribute[index];
            let key_previous = &attribute[index_previous];

            if key.value == key_previous.value {
                attribute.remove(index);    // Delete key-data:[0].

                // "index" and "index_previous" must not increment (must not change).
            } else {
                index_previous = index;
                index += 1;
            }
        }

        // Remove empty
        if attribute.is_empty() {
            *p_attr = None;
        }
    }
}
fn modify_part_animation_data_cell(p_attr: &mut Option<Vec<ssab::ssab::ss::format::PartAnimationDataCellValueT>>) {
    // MEMO: Since "Cell" has no interpolation, simply compares current to previous.
    if let Some(attribute) = p_attr {
        // Delete key-data that values do not change.
        let mut index_previous = 0;
        let mut index = 1;
        while index < attribute.len() {
            let key = &attribute[index];
            let key_previous = &attribute[index_previous];

            if key.value == key_previous.value {
                attribute.remove(index);    // Delete key-data:[0].

                // "index" and "index_previous" must not increment (must not change).
            } else {
                index_previous = index;
                index += 1;
            }
        }

        // Remove empty
        if attribute.is_empty() {
            *p_attr = None;
        }
    }
}
fn modify_part_animation_data_partcolor(p_attr: &mut Option<Vec<ssab::ssab::ss::format::PartAnimationDataPartColorT>>) {
    if let Some(attribute) = p_attr {
        // Delete key-data that values do not change.
        let mut is_value_change_previous = true;

        let mut index_previous = 0;
        let mut index = 1;
        while index < attribute.len() {
            let key = &attribute[index];
            let key_previous = &attribute[index_previous];

            let mut is_value_change = true;
            let mut is_unnecessary_previous = false;
            if key.value == key_previous.value {    // Range in:[-1]/out:[0] values are not changing
                is_value_change = modify_part_animation_data_value_changing_range(&key_previous.interpolation);
                if !is_value_change_previous && !is_value_change  {    // Range [-2][-1][0] values are not changing.
                    // If interpolation has value not changing, the previous key:[-1] is unnecessary.
                    is_unnecessary_previous = true;
                }
            }

            if !is_value_change {
                // MEMO: Get previous-key-data[-1] separately for overwriting.
                if let Some(key_previous_mut) = attribute.get_mut(index_previous) {
                    key_previous_mut.interpolation = ssab::ssab::ss::format::InterpolationType::None;  // Disable interpolation
                }
            }
            if is_unnecessary_previous {
                attribute.remove(index_previous);    // Delete key-data:[-1].

                // "index" and "index_previous" must not increment (must not change).
            } else {
                index_previous = index;
                index += 1;
            }

            is_value_change_previous = is_value_change;
        }

        // MEMO: When the interpolation for the last range is either Bezier or Hermite, there are case that the last key-data is not optimized (rare cases).
        //       * Improving precise of function "modify_part_animation_data_value_changing_range" should probably resolve the issue.
        if is_value_change_previous {
            attribute.remove(index_previous);    // Delete key-data.
        }
        if let Some(key) = attribute.last_mut() {   // Remove trailing interpolation specifications (ineffective).
            key.interpolation = ssab::ssab::ss::format::InterpolationType::None;
        }

        // Remove empty
        if attribute.is_empty() {
            *p_attr = None;
        }
    }
}
fn modify_part_animation_data_shader(p_attr: &mut Option<Vec<ssab::ssab::ss::format::PartAnimationDataShaderT>>) {
    if let Some(attribute) = p_attr {
        // Delete key-data that values do not change.
        let mut is_value_change_previous = true;

        let mut index_previous = 0;
        let mut index = 1;
        while index < attribute.len() {
            let key = &attribute[index];
            let key_previous = &attribute[index_previous];

            let mut is_value_change = true;
            let mut is_unnecessary_previous = false;
            if key.value == key_previous.value {    // Range in:[-1]/out:[0] values are not changing
                is_value_change = modify_part_animation_data_value_changing_range(&key_previous.interpolation);
                if !is_value_change_previous && !is_value_change  {    // Range [-2][-1][0] values are not changing.
                    // If interpolation has value not changing, the previous key:[-1] is unnecessary.
                    is_unnecessary_previous = true;
                }
            }

            if !is_value_change {
                // MEMO: Get previous-key-data[-1] separately for overwriting.
                if let Some(key_previous_mut) = attribute.get_mut(index_previous) {
                    key_previous_mut.interpolation = ssab::ssab::ss::format::InterpolationType::None;  // Disable interpolation
                }
            }
            if is_unnecessary_previous {
                attribute.remove(index_previous);    // Delete key-data:[-1].

                // "index" and "index_previous" must not increment (must not change).
            } else {
                index_previous = index;
                index += 1;
            }

            is_value_change_previous = is_value_change;
        }

        // MEMO: When the interpolation for the last range is either Bezier or Hermite, there are case that the last key-data is not optimized (rare cases).
        //       * Improving precise of function "modify_part_animation_data_value_changing_range" should probably resolve the issue.
        if is_value_change_previous {
            attribute.remove(index_previous);    // Delete key-data.
        }
        if let Some(key) = attribute.last_mut() {   // Remove trailing interpolation specifications (ineffective).
            key.interpolation = ssab::ssab::ss::format::InterpolationType::None;
        }

        // Remove empty
        if attribute.is_empty() {
            *p_attr = None;
        }
    }
}
fn modify_part_animation_data_vertex(p_attr: &mut Option<Vec<ssab::ssab::ss::format::PartAnimationDataVertexT>>) {
    if let Some(attribute) = p_attr {
        // Delete key-data that values do not change.
        let mut is_value_change_previous = true;

        let mut index_previous = 0;
        let mut index = 1;
        while index < attribute.len() {
            let key = &attribute[index];
            let key_previous = &attribute[index_previous];

            let mut is_value_change = true;
            let mut is_unnecessary_previous = false;
            if key.value == key_previous.value {    // Range in:[-1]/out:[0] values are not changing
                is_value_change = modify_part_animation_data_value_changing_range(&key_previous.interpolation);
                if !is_value_change_previous && !is_value_change  {    // Range [-2][-1][0] values are not changing.
                    // If interpolation has value not changing, the previous key:[-1] is unnecessary.
                    is_unnecessary_previous = true;
                }
            }

            if !is_value_change {
                // MEMO: Get previous-key-data[-1] separately for overwriting.
                if let Some(key_previous_mut) = attribute.get_mut(index_previous) {
                    key_previous_mut.interpolation = ssab::ssab::ss::format::InterpolationType::None;  // Disable interpolation
                }
            }
            if is_unnecessary_previous {
                attribute.remove(index_previous);    // Delete key-data:[-1].

                // "index" and "index_previous" must not increment (must not change).
            } else {
                index_previous = index;
                index += 1;
            }

            is_value_change_previous = is_value_change;
        }

        // MEMO: When the interpolation for the last range is either Bezier or Hermite, there are case that the last key-data is not optimized (rare cases).
        //       * Improving precise of function "modify_part_animation_data_value_changing_range" should probably resolve the issue.
        if is_value_change_previous {
            attribute.remove(index_previous);    // Delete key-data.
        }
        if let Some(key) = attribute.last_mut() {   // Remove trailing interpolation specifications (ineffective).
            key.interpolation = ssab::ssab::ss::format::InterpolationType::None;
        }

        // Remove empty
        if attribute.is_empty() {
            *p_attr = None;
        }
    }
}
fn modify_part_animation_data_skew(p_attr: &mut Option<Vec<ssab::ssab::ss::format::PartAnimationDataVec2T>>) {
    if let Some(attribute) = p_attr {
        // Delete key-data that values do not change.
        let mut is_value_change_previous = true;

        let mut index_previous = 0;
        let mut index = 1;
        while index < attribute.len() {
            let key = &attribute[index];
            let key_previous = &attribute[index_previous];

            let mut is_value_change = true;
            let mut is_unnecessary_previous = false;
            if key.value == key_previous.value {    // Range in:[-1]/out:[0] values are not changing
                is_value_change = modify_part_animation_data_value_changing_range(&key_previous.interpolation);
                if !is_value_change_previous && !is_value_change  {    // Range [-2][-1][0] values are not changing.
                    // If interpolation has value not changing, the previous key:[-1] is unnecessary.
                    is_unnecessary_previous = true;
                }
            }

            if !is_value_change {
                // MEMO: Get previous-key-data[-1] separately for overwriting.
                if let Some(key_previous_mut) = attribute.get_mut(index_previous) {
                    key_previous_mut.interpolation = ssab::ssab::ss::format::InterpolationType::None;  // Disable interpolation
                }
            }
            if is_unnecessary_previous {
                attribute.remove(index_previous);    // Delete key-data:[-1].

                // "index" and "index_previous" must not increment (must not change).
            } else {
                index_previous = index;
                index += 1;
            }

            is_value_change_previous = is_value_change;
        }

        // MEMO: When the interpolation for the last range is either Bezier or Hermite, there are case that the last key-data is not optimized (rare cases).
        //       * Improving precise of function "modify_part_animation_data_value_changing_range" should probably resolve the issue.
        if is_value_change_previous {
            attribute.remove(index_previous);    // Delete key-data.
        }
        if let Some(key) = attribute.last_mut() {   // Remove trailing interpolation specifications (ineffective).
            key.interpolation = ssab::ssab::ss::format::InterpolationType::None;
        }

        // Remove empty
        if attribute.is_empty() {
            *p_attr = None;
        }
    }
}
fn modify_part_animation_data_deform(p_attr: &mut Option<Vec<ssab::ssab::ss::format::PartAnimationDataDeformT>>) {
    if let Some(attribute) = p_attr {
        // Delete key-data that values do not change.
        let mut is_value_change_previous = true;

        let mut index_previous = 0;
        let mut index = 1;
        while index < attribute.len() {
            let key = &attribute[index];
            let key_previous = &attribute[index_previous];

            let mut is_value_change = true;
            let mut is_unnecessary_previous = false;
            if key.value == key_previous.value {    // Range in:[-1]/out:[0] values are not changing
                is_value_change = modify_part_animation_data_value_changing_range(&key_previous.interpolation);
                if !is_value_change_previous && !is_value_change  {    // Range [-2][-1][0] values are not changing.
                    // If interpolation has value not changing, the previous key:[-1] is unnecessary.
                    is_unnecessary_previous = true;
                }
            }

            if !is_value_change {
                // MEMO: Get previous-key-data[-1] separately for overwriting.
                if let Some(key_previous_mut) = attribute.get_mut(index_previous) {
                    key_previous_mut.interpolation = ssab::ssab::ss::format::InterpolationType::None;  // Disable interpolation
                }
            }
            if is_unnecessary_previous {
                attribute.remove(index_previous);    // Delete key-data:[-1].

                // "index" and "index_previous" must not increment (must not change).
            } else {
                index_previous = index;
                index += 1;
            }

            is_value_change_previous = is_value_change;
        }

        // MEMO: When the interpolation for the last range is either Bezier or Hermite, there are case that the last key-data is not optimized (rare cases).
        //       * Improving precise of function "modify_part_animation_data_value_changing_range" should probably resolve the issue.
        if is_value_change_previous {
            attribute.remove(index_previous);    // Delete key-data.
        }
        if let Some(key) = attribute.last_mut() {   // Remove trailing interpolation specifications (ineffective).
            key.interpolation = ssab::ssab::ss::format::InterpolationType::None;
        }

        // Remove empty
        if attribute.is_empty() {
            *p_attr = None;
        }
    }
}
fn modify_part_animation_data_texture(p_attr: &mut Option<Vec<ssab::ssab::ss::format::PartAnimationDataUIntValueT>>) {
    // MEMO: Since "Texture Change" has no interpolation, simply compares current to previous.
    if let Some(attribute) = p_attr {
        // Delete key-data that values do not change.
        let mut index_previous = 0;
        let mut index = 1;
        while index < attribute.len() {
            let key = &attribute[index];
            let key_previous = &attribute[index_previous];

            if key.value == key_previous.value {
                attribute.remove(index);    // Delete key-data:[0].

                // "index" and "index_previous" must not increment (must not change).
            } else {
                index_previous = index;
                index += 1;
            }
        }

        // Remove empty
        if attribute.is_empty() {
            *p_attr = None;
        }
    }
}

fn modify_part_animation_data(p_args: &mut ssab::ssab::ss::format::PartAnimationDataT, part_type: &sspj::sstypes::SsPartType, _anime_data: &sspj::ssae::SsPartAnime) {
    let support = part_type.support_profile();

    macro_rules! apply {
        ($field:ident, $supported:expr, $opt_fn:ident) => {
            if !$supported {
                p_args.$field = None;
            } else {
                $opt_fn(&mut p_args.$field);
            }
        };
    }

    apply!(cell_array, support.cell, modify_part_animation_data_cell);
    apply!(position_x_array, support.position_x, modify_part_animation_data_float);
    apply!(position_y_array, support.position_y, modify_part_animation_data_float);
    apply!(position_z_array, support.position_z, modify_part_animation_data_float);
    apply!(rotation_x_array, support.rotation_x, modify_part_animation_data_float);
    apply!(rotation_y_array, support.rotation_y, modify_part_animation_data_float);
    apply!(rotation_z_array, support.rotation_z, modify_part_animation_data_float);
    apply!(scale_x_array, support.scale_x, modify_part_animation_data_float);
    apply!(scale_y_array, support.scale_y, modify_part_animation_data_float);
    apply!(local_scale_x_array, support.local_scale_x, modify_part_animation_data_float);
    apply!(local_scale_y_array, support.local_scale_y, modify_part_animation_data_float);
    apply!(alpha_array, support.alpha, modify_part_animation_data_float);
    apply!(local_alpha_array, support.local_alpha, modify_part_animation_data_float);
    apply!(priority_array, support.priority, modify_part_animation_data_int);
    apply!(flip_h_array, support.flip_h, modify_part_animation_data_bool);
    apply!(flip_v_array, support.flip_v, modify_part_animation_data_bool);
    apply!(hide_array, support.hide, modify_part_animation_data_bool);
    apply!(part_color_array, support.part_color, modify_part_animation_data_partcolor);
    apply!(shader_array, support.shader, modify_part_animation_data_shader);
    apply!(vertex_array, support.vertex, modify_part_animation_data_vertex);
    apply!(pivot_x_array, support.pivot_x, modify_part_animation_data_float);
    apply!(pivot_y_array, support.pivot_y, modify_part_animation_data_float);
    apply!(anchor_x_array, support.anchor_x, modify_part_animation_data_float);
    apply!(anchor_y_array, support.anchor_y, modify_part_animation_data_float);
    apply!(size_x_array, support.size_x, modify_part_animation_data_float);
    apply!(size_y_array, support.size_y, modify_part_animation_data_float);
    apply!(img_flip_h_array, support.img_flip_h, modify_part_animation_data_bool);
    apply!(img_flip_v_array, support.img_flip_v, modify_part_animation_data_bool);
    apply!(uv_translation_x_array, support.uv_translation_x, modify_part_animation_data_float);
    apply!(uv_translation_y_array, support.uv_translation_y, modify_part_animation_data_float);
    apply!(uv_rotation_z_array, support.uv_rotation_z, modify_part_animation_data_float);
    apply!(uv_scale_x_array, support.uv_scale_x, modify_part_animation_data_float);
    apply!(uv_scale_y_array, support.uv_scale_y, modify_part_animation_data_float);
    apply!(bounding_radius_array, support.bounding_radius, modify_part_animation_data_float);
    apply!(mask_array, support.mask, modify_part_animation_data_float);
    apply!(deform_array, support.deform, modify_part_animation_data_deform);
    apply!(texture_array, support.texture, modify_part_animation_data_texture);

    // Special case: Skew is also controlled by the Vertex flag in the original matrix
    if !support.vertex {
        p_args.skew_array = None;
    } else {
        modify_part_animation_data_skew(&mut p_args.skew_array);
    }
}

fn convert_parts_animation_data(anime_data: &sspj::ssae::SsAnimation, setup_animation: &sspj::ssae::SsAnimation, parts_map: &HashMap<String, &sspj::ssae::SsPart>) -> Vec<ssab::ssab::ss::format::PartAnimationDataT> {
    let mut parts_animation_data_t = Vec::<ssab::ssab::ss::format::PartAnimationDataT>::new();
    parts_animation_data_t.resize(parts_map.len(), ssab::ssab::ss::format::PartAnimationDataT::default());

    for part_anime in &setup_animation.part_animes {
        let Some(p) = parts_map.get(&part_anime.part_name) else {
            // not found part (an illegal case)
            continue;
        };

        let index_part = p.array_index as usize;
        convert_part_animation_data(&mut parts_animation_data_t[index_part], &part_anime);
    }

    for part_anime in &anime_data.part_animes {
        let Some(p) = parts_map.get(&part_anime.part_name) else {
            // not found part (an illegal case)
            continue;
        };

        let index_part = p.array_index as usize;
        convert_part_animation_data(&mut parts_animation_data_t[index_part], &part_anime);
        modify_part_animation_data(&mut parts_animation_data_t[index_part], &p.part_type, &part_anime);
    }

    parts_animation_data_t
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
    anime_pack.model.part_list.iter().map(|part| {
        let mut part_data_t = ssab::ssab::ss::format::PartDataT::default();
        part_data_t.name = Some(part.name.to_string());
        part_data_t.parent_index = part.parent_index as i16;
        part_data_t.bounds_type = match &part.bounds_type {
            sspj::sstypes::SsBoundsType::None => ssab::ssab::ss::format::BoundsType::None,
            sspj::sstypes::SsBoundsType::Quad => ssab::ssab::ss::format::BoundsType::Quad,
            sspj::sstypes::SsBoundsType::Aabb => ssab::ssab::ss::format::BoundsType::Aabb,
            sspj::sstypes::SsBoundsType::Circle => ssab::ssab::ss::format::BoundsType::Circle,
            sspj::sstypes::SsBoundsType::CircleSmin => ssab::ssab::ss::format::BoundsType::Circle,
            sspj::sstypes::SsBoundsType::CircleSmax => ssab::ssab::ss::format::BoundsType::Circle,
        };
        part_data_t.blend_type = convert_alpha_blend_type(&part.alpha_blend_type);
        part_data_t.part_type = match &part.part_type {
            sspj::sstypes::SsPartType::Null => ssab::ssab::ss::format::PartTypeT::PartTypeNull(Box::new(ssab::ssab::ss::format::NoneValueEntryT::default())),
            sspj::sstypes::SsPartType::Normal => ssab::ssab::ss::format::PartTypeT::PartTypeNormal(Box::new(ssab::ssab::ss::format::NoneValueEntryT::default())),
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
            },
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
                    0  => ssab::ssab::ss::format::TextAnchor::LT,
                    1  => ssab::ssab::ss::format::TextAnchor::LC,
                    2  => ssab::ssab::ss::format::TextAnchor::LB,
                    16 => ssab::ssab::ss::format::TextAnchor::CT,
                    17 => ssab::ssab::ss::format::TextAnchor::CC,
                    18 => ssab::ssab::ss::format::TextAnchor::CB,
                    32 => ssab::ssab::ss::format::TextAnchor::RT,
                    33 => ssab::ssab::ss::format::TextAnchor::RC,
                    34 => ssab::ssab::ss::format::TextAnchor::RB,
                    255 =>ssab::ssab::ss::format::TextAnchor::NO,
                    _ => ssab::ssab::ss::format::TextAnchor::LT,
                };

                ssab::ssab::ss::format::PartTypeT::PartTypeText(Box::new(part_type))
            },
            sspj::sstypes::SsPartType::Nines => {
                let mut part_type = ssab::ssab::ss::format::PartTypeNinesT::default();
                part_type.nines_margin_l = part.nines_margin_l;
                part_type.nines_margin_r = part.nines_margin_r;
                part_type.nines_margin_t = part.nines_margin_t;
                part_type.nines_margin_b = part.nines_margin_b;
                part_type.nines_fill_mode = part.nines_fill_mode;
                part_type.nines_mask = part.nines_mask;

                ssab::ssab::ss::format::PartTypeT::PartTypeNines(Box::new(part_type))
            },
            sspj::sstypes::SsPartType::Instance => {
                let mut part_type = ssab::ssab::ss::format::PartTypeInstanceT::default();
                part_type.ref_anime_hash = crate::utils::fnv1a_hash_str(&part.ref_anime);
                ssab::ssab::ss::format::PartTypeT::PartTypeInstance(Box::new(part_type))
            },
            sspj::sstypes::SsPartType::Armature => ssab::ssab::ss::format::PartTypeT::PartTypeArmature(Box::new(ssab::ssab::ss::format::NoneValueEntryT::default())),
            sspj::sstypes::SsPartType::Effect => {
                let mut part_type = ssab::ssab::ss::format::PartTypeEffectT::default();
                part_type.ref_effect_name_hash = crate::utils::fnv1a_hash_str(&part.ref_effect_name);
                ssab::ssab::ss::format::PartTypeT::PartTypeEffect(Box::new(part_type))
            },
            sspj::sstypes::SsPartType::Mesh => ssab::ssab::ss::format::PartTypeT::PartTypeMesh(Box::new(ssab::ssab::ss::format::NoneValueEntryT::default())),
            sspj::sstypes::SsPartType::Movenode => ssab::ssab::ss::format::PartTypeT::PartTypeMovenode(Box::new(ssab::ssab::ss::format::NoneValueEntryT::default())),
            sspj::sstypes::SsPartType::Constraint => ssab::ssab::ss::format::PartTypeT::PartTypeConstraint(Box::new(ssab::ssab::ss::format::NoneValueEntryT::default())),
            sspj::sstypes::SsPartType::Mask => ssab::ssab::ss::format::PartTypeT::PartTypeMask(Box::new(ssab::ssab::ss::format::NoneValueEntryT::default())),
            sspj::sstypes::SsPartType::Joint => ssab::ssab::ss::format::PartTypeT::PartTypeJoint(Box::new(ssab::ssab::ss::format::NoneValueEntryT::default())),
            sspj::sstypes::SsPartType::Bonepoint => ssab::ssab::ss::format::PartTypeT::PartTypeBonepoint(Box::new(ssab::ssab::ss::format::NoneValueEntryT::default())),
            sspj::sstypes::SsPartType::TransformConstraint => ssab::ssab::ss::format::PartTypeT::PartTypeTransformConstraint(Box::new(ssab::ssab::ss::format::NoneValueEntryT::default())),
            sspj::sstypes::SsPartType::Camera => ssab::ssab::ss::format::PartTypeT::PartTypeCamera(Box::new(ssab::ssab::ss::format::NoneValueEntryT::default())),
            sspj::sstypes::SsPartType::Audio => ssab::ssab::ss::format::PartTypeT::PartTypeAudio(Box::new(ssab::ssab::ss::format::NoneValueEntryT::default())),
        };

        part_data_t
    }).collect::<Vec<_>>()
}

pub fn convert_animation_data_list(anime_pack: &sspj::ssae::SsAnimePack, parts_map: &HashMap<String, &sspj::ssae::SsPart>) -> Vec<ssab::ssab::ss::format::AnimationDataT> {
    let Some(setup_animation) = anime_pack.anime_list.iter().find(|anime_data| anime_data.is_setup) else {
        // not found setup animation
        return vec![];
    };

    anime_pack.anime_list.iter().filter_map(|anime_data| {
        if anime_data == setup_animation {
            return None;
        }
        Some(convert_animedatat(anime_data, setup_animation, parts_map))
    }).collect::<Vec<ssab::ssab::ss::format::AnimationDataT>>()
}
