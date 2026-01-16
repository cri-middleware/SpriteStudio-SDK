use std::collections::HashMap;

fn convert_animedata_labels(labels: &[sspj::ssae::SsLabel]) -> Vec<ssab::ssab::LabelT> {
    let mut v = labels.iter().map(|label| {
        let mut l = ssab::ssab::LabelT::default();
        l.name_hash = crate::utils::fnv1a_hash_str(&label.name);
        l.name = if label.name.is_empty() { None } else { Some(label.name.to_string()) };
        l.time = label.time;
        l
    }).collect::<Vec<ssab::ssab::LabelT>>();
    v.sort_by_key(|l| l.name_hash);
    v
}

fn convert_events_per_frames(anime_data: &sspj::ssae::SsAnimation, parts_map: &HashMap<String, &sspj::ssae::SsPart>) -> Option<Vec<ssab::ssab::EventsPerFrameT>> {
    let mut events_per_frames = HashMap::<i32, ssab::ssab::EventsPerFrameT>::new();
    for part_anime in &anime_data.part_animes {
        let Some(p) = parts_map.get(&part_anime.part_name) else {
            // not found part (an illegal case)
            continue;
        };
        let part_index = p.array_index as u16;

        part_anime.attributes.iter().for_each(|attribute| {
            match attribute {
                sspj::ssattribute::SsAttributeKind::User(v) => {
                    v.keys.iter().for_each(|key|{
                        let mut u_args = ssab::ssab::EventUserDataT::default();
                        u_args.part_index = part_index;
                        let mut userdata = ssab::ssab::PartAttributeUserDataT::default();
                        userdata.integer = key.value.integer.as_ref().map(|v| {
                            let mut i = ssab::ssab::IntValueEntryT::default(); i.value = *v;
                            Box::new(i)
                        });
                        userdata.rect = key.value.rect.as_ref().map(|v| {
                            let mut r = ssab::ssab::RectT::default();
                            r.x1 = v.x1 as f32;
                            r.x2 = v.x2 as f32;
                            r.y1 = v.y1 as f32;
                            r.y2 = v.y2 as f32;
                            Box::new(r)
                        });
                        userdata.point = key.value.point.as_ref().map(|v| {
                            let mut p = ssab::ssab::Vec2T::default();
                            p.v1 = v.x;
                            p.v2 = v.y;
                            Box::new(p)
                        });
                        userdata.str = key.value.user_string.as_ref().map(|v| v.to_string());
                        u_args.value = Some(Box::new(userdata));

                        let events_per_frame = events_per_frames.entry(key.time).or_insert_with(|| {
                            let mut e = ssab::ssab::EventsPerFrameT::default();
                            e.frame_index = key.time as u16;
                            e
                        });
                        events_per_frame.users.get_or_insert_with(Vec::new).push(u_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::Signal(v) => {
                    v.keys.iter().for_each(|key| {
                        key.value.commands.iter().for_each(|command| {
                            let mut s_args = ssab::ssab::EventSignalT::default();
                            s_args.part_index = part_index;
                            let mut s = ssab::ssab::PartAttributeSignalT::default();
                            s.active = command.active;
                            s.command_id = Some(command.command_id.to_string());
                            s.params = if command.params.is_empty() { None } else {
                                let v = command.params.iter().map(|v| {
                                    let mut command_param_t = ssab::ssab::SignalCommandParamT::default();
                                    command_param_t.param_id = Some(v.param_id.to_string());
                                    command_param_t.type_ = match &v.param_type {
                                        sspj::ssattribute::SsAttributeSignalCommandParamType::Bool => ssab::ssab::SignalCommandParamType::Bool,
                                        sspj::ssattribute::SsAttributeSignalCommandParamType::Index => ssab::ssab::SignalCommandParamType::Index,
                                        sspj::ssattribute::SsAttributeSignalCommandParamType::Integer => ssab::ssab::SignalCommandParamType::Integer,
                                        sspj::ssattribute::SsAttributeSignalCommandParamType::Floating => ssab::ssab::SignalCommandParamType::Floating,
                                        sspj::ssattribute::SsAttributeSignalCommandParamType::Text => ssab::ssab::SignalCommandParamType::Text,
                                        sspj::ssattribute::SsAttributeSignalCommandParamType::None => ssab::ssab::SignalCommandParamType::None,
                                    };
                                    command_param_t.value = match &v.value {
                                        sspj::ssattribute::SsAttributeSignalCommandParamValue::Bool(v) => {
                                            let mut b = ssab::ssab::BoolValueEntryT::default();
                                            b.value = *v;
                                            ssab::ssab::SignalCommandParamValueT::Bool(Box::new(b))
                                        },
                                        sspj::ssattribute::SsAttributeSignalCommandParamValue::Index(v) => {
                                            let mut i = ssab::ssab::IntValueEntryT::default();
                                            i.value = *v;
                                            ssab::ssab::SignalCommandParamValueT::Index(Box::new(i))
                                        },
                                        sspj::ssattribute::SsAttributeSignalCommandParamValue::Integer(v) => {
                                            let mut i = ssab::ssab::IntValueEntryT::default();
                                            i.value = *v;
                                            ssab::ssab::SignalCommandParamValueT::Integer(Box::new(i))
                                        },
                                        sspj::ssattribute::SsAttributeSignalCommandParamValue::Floating(v) =>{
                                            let mut f = ssab::ssab::FloatValueEntryT::default();
                                            f.value = *v;
                                            ssab::ssab::SignalCommandParamValueT::Floating(Box::new(f))
                                        },
                                        sspj::ssattribute::SsAttributeSignalCommandParamValue::Text(v) => {
                                            let mut s = ssab::ssab::StringValueEntryT::default();
                                            s.value = Some(v.to_string());
                                            ssab::ssab::SignalCommandParamValueT::Text(Box::new(s))
                                        },
                                        sspj::ssattribute::SsAttributeSignalCommandParamValue::None => {
                                            let n = ssab::ssab::NoneValueEntryT::default();
                                            ssab::ssab::SignalCommandParamValueT::None(Box::new(n))
                                        },
                                    };

                                    command_param_t
                                }).collect::<Vec<_>>();
                                Some(v)
                            };
                            s_args.value = Some(Box::new(s));

                            let events_per_frame = events_per_frames.entry(key.time).or_insert_with(|| {
                                let mut e = ssab::ssab::EventsPerFrameT::default();
                                e.frame_index = key.time as u16;
                                e
                            });
                            events_per_frame.signals.get_or_insert_with(Vec::new).push(s_args);
                        });
                    });
                },
                sspj::ssattribute::SsAttributeKind::Instance(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut i_args = ssab::ssab::EventInstanceT::default();
                        i_args.part_index = part_index;
                        let mut instance_t = ssab::ssab::PartAttributeInstanceT::default();
                        instance_t.pingpong = key.value.pingpong;
                        instance_t.independent = key.value.independent;
                        instance_t.loop_num = key.value.loop_num;
                        instance_t.start_label_hash = crate::utils::fnv1a_hash_str(&key.value.start_label);
                        instance_t.start_offset = key.value.start_offset;
                        instance_t.end_label_hash = crate::utils::fnv1a_hash_str(&key.value.end_label);
                        instance_t.end_offset = key.value.end_offset;
                        instance_t.speed = key.value.speed;
                        i_args.value = Some(Box::new(instance_t));

                        let events_per_frame = events_per_frames.entry(key.time).or_insert_with(|| {
                            let mut e = ssab::ssab::EventsPerFrameT::default();
                            e.frame_index = key.time as u16;
                            e
                        });
                        events_per_frame.instances.get_or_insert_with(Vec::new).push(i_args);

                    });
                },
                sspj::ssattribute::SsAttributeKind::Effect(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut e_args = ssab::ssab::EventEffectT::default();
                        e_args.part_index = part_index;
                        let mut effect_t = ssab::ssab::PartAttributeEffectT::default();
                        effect_t.cur_keyframe; // TODO: implement
                        effect_t.start_time = key.value.start_time;
                        effect_t.speed = key.value.speed;
                        effect_t.loopflag; // TODO: implement
                        e_args.value = Some(Box::new(effect_t));

                        let events_per_frame = events_per_frames.entry(key.time).or_insert_with(|| {
                            let mut e = ssab::ssab::EventsPerFrameT::default();
                            e.frame_index = key.time as u16;
                            e
                        });
                        events_per_frame.effects.get_or_insert_with(Vec::new).push(e_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::Audio(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut a_args = ssab::ssab::EventAudioT::default();
                        a_args.part_index = part_index;
                        let mut audio_t = ssab::ssab::PartAttributeAudioT::default();
                        audio_t.sound_list_id = key.value.sound_list_id;
                        audio_t.sound_name = Some(key.value.sound_name.to_string());
                        audio_t.loop_num = key.value.loop_num;
                        a_args.value = Some(Box::new(audio_t));

                        let events_per_frame = events_per_frames.entry(key.time).or_insert_with(|| {
                            let mut e = ssab::ssab::EventsPerFrameT::default();
                            e.frame_index = key.time as u16;
                            e
                        });
                        events_per_frame.audios.get_or_insert_with(Vec::new).push(a_args);
                    });
                },
                _ => {
                    // skip
                },
            }
        });
    }

    if events_per_frames.is_empty() {
        None
    } else {
        let mut vec_map: Vec<(i32, ssab::ssab::EventsPerFrameT)> = events_per_frames.into_iter().collect();
        vec_map.sort_by(|a, b| a.0.cmp(&b.0));
        Some(vec_map.into_iter().map(|(_, v)| v).collect::<Vec<ssab::ssab::EventsPerFrameT>>())
    }
}

fn convert_animedatat(anime_data: &sspj::ssae::SsAnimation, setup_animation: &sspj::ssae::SsAnimation, parts_map: &HashMap<String, &sspj::ssae::SsPart>) -> ssab::ssab::AnimationDataT {
    let mut anime_data_t = ssab::ssab::AnimationDataT::default();

    anime_data_t.name = anime_data.name.to_string();
    anime_data_t.parts_initial_attributes = Some(convert_parts_initial_attributes(anime_data, setup_animation));
    anime_data_t.parts_animation_data = Some(convert_parts_animation_data(anime_data, setup_animation));
    anime_data_t.parts_mesh_data = None;
    anime_data_t.events = convert_events_per_frames(&anime_data, parts_map);
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

fn convert_interpolation_type(interpolation: &sspj::sstypes::SsInterpolationType) -> ssab::ssab::InterpolationType {
    match interpolation {
        sspj::sstypes::SsInterpolationType::None => ssab::ssab::InterpolationType::None,
        sspj::sstypes::SsInterpolationType::Linear => ssab::ssab::InterpolationType::Linear,
        sspj::sstypes::SsInterpolationType::Hermite => ssab::ssab::InterpolationType::Hermite,
        sspj::sstypes::SsInterpolationType::Bezier => ssab::ssab::InterpolationType::Bezier,
        sspj::sstypes::SsInterpolationType::Acceleration => ssab::ssab::InterpolationType::Acceleration,
        sspj::sstypes::SsInterpolationType::Deceleration => ssab::ssab::InterpolationType::Deceleration,
        sspj::sstypes::SsInterpolationType::EaseIn => ssab::ssab::InterpolationType::EaseIn,
        sspj::sstypes::SsInterpolationType::EaseOut => ssab::ssab::InterpolationType::EaseOut,
        sspj::sstypes::SsInterpolationType::EaseInOut => ssab::ssab::InterpolationType::EaseInOut,
        sspj::sstypes::SsInterpolationType::EaseExponentialIn => ssab::ssab::InterpolationType::EaseExponentialIn,
        sspj::sstypes::SsInterpolationType::EaseExponentialOut => ssab::ssab::InterpolationType::EaseExponentialOut,
        sspj::sstypes::SsInterpolationType::EaseExponentialInOut => ssab::ssab::InterpolationType::EaseExponentialInOut,
        sspj::sstypes::SsInterpolationType::EaseSineIn => ssab::ssab::InterpolationType::EaseSineIn,
        sspj::sstypes::SsInterpolationType::EaseSineOut => ssab::ssab::InterpolationType::EaseSineOut,
        sspj::sstypes::SsInterpolationType::EaseSineInOut => ssab::ssab::InterpolationType::EaseSineInOut,
        sspj::sstypes::SsInterpolationType::EaseElasticIn => ssab::ssab::InterpolationType::EaseElasticIn,
        sspj::sstypes::SsInterpolationType::EaseElasticOut => ssab::ssab::InterpolationType::EaseElasticOut,
        sspj::sstypes::SsInterpolationType::EaseElasticInOut => ssab::ssab::InterpolationType::EaseElasticInOut,
        sspj::sstypes::SsInterpolationType::EaseBounceIn => ssab::ssab::InterpolationType::EaseBounceIn,
        sspj::sstypes::SsInterpolationType::EaseBounceOut => ssab::ssab::InterpolationType::EaseBounceOut,
        sspj::sstypes::SsInterpolationType::EaseBounceInOut => ssab::ssab::InterpolationType::EaseBounceInOut,
        sspj::sstypes::SsInterpolationType::EaseBackIn => ssab::ssab::InterpolationType::EaseBackIn,
        sspj::sstypes::SsInterpolationType::EaseBackOut => ssab::ssab::InterpolationType::EaseBackOut,
        sspj::sstypes::SsInterpolationType::EaseBackInOut => ssab::ssab::InterpolationType::EaseBackInOut,
        _ => ssab::ssab::InterpolationType::None,
    }
}

fn merge_setup_attributes_to_part_attribute_t(p_args: &mut ssab::ssab::PartAttributesT, setup_attributes: &[sspj::ssattribute::SsAttributeKind]) {
    setup_attributes.iter().for_each(|attribute| {
        match attribute {
            sspj::ssattribute::SsAttributeKind::Cell(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::PosX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::PosY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::PosZ(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::RotX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::RotY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::RotZ(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::SclX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::SclY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::LosclX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::LosclY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::Alpha(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::LoAlpha(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::Prio(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::FlipH(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::FlipV(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::Hide(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::PartsColor(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::Color => {
                // TODO: impl
            },
            sspj::ssattribute::SsAttributeKind::Shader(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::Vertex(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::PivotX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::PivotY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::AnchorX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::AnchorY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::SizeX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::SizeY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::ImgFlipH(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::ImgFlipV(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::UvtX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::UvtY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::UvrZ(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::UvsX(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::UvsY(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::Boundr(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::Mask(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::User(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::Signal(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::Instance(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::Effect(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::Deform(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::Audio(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
            sspj::ssattribute::SsAttributeKind::Texchange(v) => {
                v.keys.iter().find(|key| key.time == 0).map(|_key| {
                    // TODO: impl
                });
            },
        }
    });
}

fn convert_parts_initial_attributes(anime_data: &sspj::ssae::SsAnimation, setup_animation: &sspj::ssae::SsAnimation) -> Vec<ssab::ssab::PartAttributesT>{
    anime_data.part_animes.iter().zip(setup_animation.part_animes.iter()).map(|(part_anime, setup_part_anime)| {
        let mut part_attribute_t = ssab::ssab::PartAttributesT::default();
        // TODO: impl
        merge_setup_attributes_to_part_attribute_t(&mut part_attribute_t, &setup_part_anime.attributes);
        merge_setup_attributes_to_part_attribute_t(&mut part_attribute_t, &part_anime.attributes);

        part_attribute_t
    }).collect::<Vec<_>>()
}

fn convert_parts_animation_data(anime_data: &sspj::ssae::SsAnimation, _setup_animation: &sspj::ssae::SsAnimation) -> Vec<ssab::ssab::PartAnimationDataT> {
    anime_data.part_animes.iter().map(|part_anime| {
        let mut part_animation_data_t = ssab::ssab::PartAnimationDataT::default();
        part_anime.attributes.iter().for_each(|attribute| {
            match attribute {
                sspj::ssattribute::SsAttributeKind::Cell(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut c_args = ssab::ssab::PartAnimationDataCellValueT::default();
                        c_args.index = key.time as u16;
                        let mut value = ssab::ssab::PartAttributeCellT::default();
                        value.map_id = key.value.map_id;
                        value.name_hash = crate::utils::fnv1a_hash_str(&key.value.name);
                        c_args.value = Some(Box::new(value));
                        part_animation_data_t.cell_array.get_or_insert_with(Vec::new).push(c_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::PosX(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut x_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        x_args.index = key.time as u16;
                        x_args.interpolation = convert_interpolation_type(&key.ip_type);
                        x_args.value = key.value;
                        part_animation_data_t.pos_x_array.get_or_insert_with(Vec::new).push(x_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::PosY(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut y_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        y_args.index = key.time as u16;
                        y_args.interpolation = convert_interpolation_type(&key.ip_type);
                        y_args.value = key.value;
                        part_animation_data_t.pos_y_array.get_or_insert_with(Vec::new).push(y_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::PosZ(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut z_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        z_args.index = key.time as u16;
                        z_args.interpolation = convert_interpolation_type(&key.ip_type);
                        z_args.value = key.value;
                        part_animation_data_t.pos_z_array.get_or_insert_with(Vec::new).push(z_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::RotX(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut x_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        x_args.index = key.time as u16;
                        x_args.interpolation = convert_interpolation_type(&key.ip_type);
                        x_args.value = key.value;
                        part_animation_data_t.rot_x_array.get_or_insert_with(Vec::new).push(x_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::RotY(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut y_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        y_args.index = key.time as u16;
                        y_args.interpolation = convert_interpolation_type(&key.ip_type);
                        y_args.value = key.value;
                        part_animation_data_t.rot_y_array.get_or_insert_with(Vec::new).push(y_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::RotZ(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut z_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        z_args.index = key.time as u16;
                        z_args.interpolation = convert_interpolation_type(&key.ip_type);
                        z_args.value = key.value;
                        part_animation_data_t.rot_y_array.get_or_insert_with(Vec::new).push(z_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::SclX(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut x_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        x_args.index = key.time as u16;
                        x_args.interpolation = convert_interpolation_type(&key.ip_type);
                        x_args.value = key.value;
                        part_animation_data_t.scl_x_array.get_or_insert_with(Vec::new).push(x_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::SclY(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut y_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        y_args.index = key.time as u16;
                        y_args.interpolation = convert_interpolation_type(&key.ip_type);
                        y_args.value = key.value;
                        part_animation_data_t.scl_y_array.get_or_insert_with(Vec::new).push(y_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::LosclX(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut x_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        x_args.index = key.time as u16;
                        x_args.interpolation = convert_interpolation_type(&key.ip_type);
                        x_args.value = key.value;
                        part_animation_data_t.loscl_x_array.get_or_insert_with(Vec::new).push(x_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::LosclY(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut y_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        y_args.index = key.time as u16;
                        y_args.interpolation = convert_interpolation_type(&key.ip_type);
                        y_args.value = key.value;
                        part_animation_data_t.loscl_y_array.get_or_insert_with(Vec::new).push(y_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::Alpha(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut a_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        a_args.index = key.time as u16;
                        a_args.interpolation = convert_interpolation_type(&key.ip_type);
                        a_args.value = key.value;
                        part_animation_data_t.alpha_array.get_or_insert_with(Vec::new).push(a_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::LoAlpha(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut a_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        a_args.index = key.time as u16;
                        a_args.interpolation = convert_interpolation_type(&key.ip_type);
                        a_args.value = key.value;
                        part_animation_data_t.lo_alpha_array.get_or_insert_with(Vec::new).push(a_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::Prio(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut p_args = ssab::ssab::PartAnimationDataIntValueT::default();
                        p_args.index = key.time as u16;
                        p_args.interpolation = convert_interpolation_type(&key.ip_type);
                        p_args.value = key.value;
                        part_animation_data_t.prio_array.get_or_insert_with(Vec::new).push(p_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::FlipH(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut h_args = ssab::ssab::PartAnimationDataBoolValueT::default();
                        h_args.index = key.time as u16;
                        h_args.value = key.value;
                        part_animation_data_t.flip_h_array.get_or_insert_with(Vec::new).push(h_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::FlipV(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut v_args = ssab::ssab::PartAnimationDataBoolValueT::default();
                        v_args.index = key.time as u16;
                        v_args.value = key.value;
                        part_animation_data_t.flip_v_array.get_or_insert_with(Vec::new).push(v_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::Hide(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut v_args = ssab::ssab::PartAnimationDataBoolValueT::default();
                        v_args.index = key.time as u16;
                        v_args.value = key.value;
                        part_animation_data_t.flip_v_array.get_or_insert_with(Vec::new).push(v_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::PartsColor(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut c_args = ssab::ssab::PartAnimationDataPartColorT::default();
                        c_args.index = key.time as u16;
                        let mut value = ssab::ssab::PartAttributePartColorT::default();
                        value.blend_type = convert_alpha_blend_type(&key.value.blend_type);
                        let mut key_value;
                        let mut rgba;
                        key_value = ssab::ssab::SsAttributePartColorKeyValueColorT::default();
                        rgba = ssab::ssab::U8RectT::default();
                        rgba.a = key.value.lt.rgba.a;
                        rgba.r = key.value.lt.rgba.r;
                        rgba.g = key.value.lt.rgba.g;
                        rgba.b = key.value.lt.rgba.b;
                        key_value.rgba = Some(Box::new(rgba));
                        key_value.rate = key.value.lt.rate;
                        value.lt = Some(Box::new(key_value));

                        key_value = ssab::ssab::SsAttributePartColorKeyValueColorT::default();
                        rgba = ssab::ssab::U8RectT::default();
                        rgba.a = key.value.rt.rgba.a;
                        rgba.r = key.value.rt.rgba.r;
                        rgba.g = key.value.rt.rgba.g;
                        rgba.b = key.value.rt.rgba.b;
                        key_value.rgba = Some(Box::new(rgba));
                        key_value.rate = key.value.rt.rate;
                        value.rt = Some(Box::new(key_value));

                        key_value = ssab::ssab::SsAttributePartColorKeyValueColorT::default();
                        rgba = ssab::ssab::U8RectT::default();
                        rgba.a = key.value.lb.rgba.a;
                        rgba.r = key.value.lb.rgba.r;
                        rgba.g = key.value.lb.rgba.g;
                        rgba.b = key.value.lb.rgba.b;
                        key_value.rgba = Some(Box::new(rgba));
                        key_value.rate = key.value.lb.rate;
                        value.lb = Some(Box::new(key_value));

                        key_value = ssab::ssab::SsAttributePartColorKeyValueColorT::default();
                        rgba = ssab::ssab::U8RectT::default();
                        rgba.a = key.value.rb.rgba.a;
                        rgba.r = key.value.rb.rgba.r;
                        rgba.g = key.value.rb.rgba.g;
                        rgba.b = key.value.rb.rgba.b;
                        key_value.rgba = Some(Box::new(rgba));
                        key_value.rate = key.value.rb.rate;
                        value.rb = Some(Box::new(key_value));

                        c_args.value = Some(Box::new(value));

                        part_animation_data_t.parts_color_array.get_or_insert_with(Vec::new).push(c_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::Color => {
                    // TODO: impl
                },
                sspj::ssattribute::SsAttributeKind::Shader(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut s_args = ssab::ssab::PartAnimationDataShaderT::default();
                        s_args.index = key.time as u16;
                        s_args.interpolation = convert_interpolation_type(&key.ip_type);
                        let mut value = ssab::ssab::PartAttributeShaderT::default();
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
                        part_animation_data_t.shader_array.get_or_insert_with(Vec::new).push(s_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::Vertex(v) => {
                    v.keys.iter().for_each(|key| {
                        if key.value.use_skew.is_some() && key.value.use_skew.unwrap() == true {
                            // Skew
                            let mut s_args = ssab::ssab::PartAnimationDataVec2T::default();
                            s_args.index = key.time as u16;
                            s_args.interpolation = convert_interpolation_type(&key.ip_type);
                            s_args.value = key.value.skew.as_ref().map(|v| {
                                let mut value = ssab::ssab::Vec2T::default();
                                value.v1 = v.x;
                                value.v2 = v.y;
                                Box::new(value)
                            });

                            part_animation_data_t.skew_array.get_or_insert_with(Vec::new).push(s_args);
                        } else {
                            // Vertex
                            let mut s_args = ssab::ssab::PartAnimationDataVertexT::default();
                            s_args.index = key.time as u16;
                            s_args.interpolation = convert_interpolation_type(&key.ip_type);
                            let mut value = ssab::ssab::PartAttributeVertexT::default();
                            let mut v: ssab::ssab::Vec2T;
                            v = ssab::ssab::Vec2T::default();
                            key.value.lt.as_ref().map(|lt| {
                                v.v1 = lt.x;
                                v.v2 = lt.y;
                            });
                            value.lt = Some(Box::new(v));
                            v = ssab::ssab::Vec2T::default();
                            key.value.rt.as_ref().map(|rt| {
                                v.v1 = rt.x;
                                v.v2 = rt.y;
                            });
                            value.rt = Some(Box::new(v));

                            v = ssab::ssab::Vec2T::default();
                            key.value.lb.as_ref().map(|lb| {
                                v.v1 = lb.x;
                                v.v2 = lb.y;
                            });
                            value.lb = Some(Box::new(v));

                            v = ssab::ssab::Vec2T::default();
                            key.value.rb.as_ref().map(|rb| {
                                v.v1 = rb.x;
                                v.v2 = rb.y;
                            });
                            value.rb = Some(Box::new(v));

                            s_args.value = Some(Box::new(value));
                            part_animation_data_t.vertex_array.get_or_insert_with(Vec::new).push(s_args);
                        }
                    });
                },
                sspj::ssattribute::SsAttributeKind::PivotX(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut x_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        x_args.index = key.time as u16;
                        x_args.interpolation = convert_interpolation_type(&key.ip_type);
                        x_args.value = key.value;
                        part_animation_data_t.pivot_x_array.get_or_insert_with(Vec::new).push(x_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::PivotY(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut y_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        y_args.index = key.time as u16;
                        y_args.interpolation = convert_interpolation_type(&key.ip_type);
                        y_args.value = key.value;
                        part_animation_data_t.pivot_y_array.get_or_insert_with(Vec::new).push(y_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::AnchorX(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut x_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        x_args.index = key.time as u16;
                        x_args.interpolation = convert_interpolation_type(&key.ip_type);
                        x_args.value = key.value;
                        part_animation_data_t.anchor_x_array.get_or_insert_with(Vec::new).push(x_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::AnchorY(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut y_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        y_args.index = key.time as u16;
                        y_args.interpolation = convert_interpolation_type(&key.ip_type);
                        y_args.value = key.value;
                        part_animation_data_t.anchor_y_array.get_or_insert_with(Vec::new).push(y_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::SizeX(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut x_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        x_args.index = key.time as u16;
                        x_args.interpolation = convert_interpolation_type(&key.ip_type);
                        x_args.value = key.value;
                        part_animation_data_t.size_x_array.get_or_insert_with(Vec::new).push(x_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::SizeY(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut y_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        y_args.index = key.time as u16;
                        y_args.interpolation = convert_interpolation_type(&key.ip_type);
                        y_args.value = key.value;
                        part_animation_data_t.size_y_array.get_or_insert_with(Vec::new).push(y_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::ImgFlipH(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut h_args = ssab::ssab::PartAnimationDataBoolValueT::default();
                        h_args.index = key.time as u16;
                        h_args.value = key.value;
                        part_animation_data_t.img_flip_h_array.get_or_insert_with(Vec::new).push(h_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::ImgFlipV(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut v_args = ssab::ssab::PartAnimationDataBoolValueT::default();
                        v_args.index = key.time as u16;
                        v_args.value = key.value;
                        part_animation_data_t.img_flip_v_array.get_or_insert_with(Vec::new).push(v_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::UvtX(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut x_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        x_args.index = key.time as u16;
                        x_args.interpolation = convert_interpolation_type(&key.ip_type);
                        x_args.value = key.value;
                        part_animation_data_t.uvt_x_array.get_or_insert_with(Vec::new).push(x_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::UvtY(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut y_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        y_args.index = key.time as u16;
                        y_args.interpolation = convert_interpolation_type(&key.ip_type);
                        y_args.value = key.value;
                        part_animation_data_t.uvt_y_array.get_or_insert_with(Vec::new).push(y_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::UvrZ(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut z_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        z_args.index = key.time as u16;
                        z_args.interpolation = convert_interpolation_type(&key.ip_type);
                        z_args.value = key.value;
                        part_animation_data_t.uvr_z_array.get_or_insert_with(Vec::new).push(z_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::UvsX(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut z_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        z_args.index = key.time as u16;
                        z_args.interpolation = convert_interpolation_type(&key.ip_type);
                        z_args.value = key.value;
                        part_animation_data_t.uvs_x_array.get_or_insert_with(Vec::new).push(z_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::UvsY(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut z_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        z_args.index = key.time as u16;
                        z_args.interpolation = convert_interpolation_type(&key.ip_type);
                        z_args.value = key.value;
                        part_animation_data_t.uvs_y_array.get_or_insert_with(Vec::new).push(z_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::Boundr(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut z_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        z_args.index = key.time as u16;
                        z_args.interpolation = convert_interpolation_type(&key.ip_type);
                        z_args.value = key.value;
                        part_animation_data_t.boundr_array.get_or_insert_with(Vec::new).push(z_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::Mask(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut m_args = ssab::ssab::PartAnimationDataFloatValueT::default();
                        m_args.index = key.time as u16;
                        m_args.interpolation = convert_interpolation_type(&key.ip_type);
                        m_args.value = key.value;
                        part_animation_data_t.mask_array.get_or_insert_with(Vec::new).push(m_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::Deform(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut m_args = ssab::ssab::PartAnimationDataDeformT::default();
                        m_args.index = key.time as u16;
                        m_args.interpolation = convert_interpolation_type(&key.ip_type);
                        if key.value.vchg.is_empty() {
                            m_args.value = None;
                        } else {
                            let mut coordinate = Vec::new();
                            let mut index = Vec::<i16>::new();
                            key.value.vchg.iter().for_each(|item| {
                                let mut vec2 = ssab::ssab::Vec2T::default();
                                vec2.v1 = item.x;
                                vec2.v2 = item.y;
                                coordinate.push(vec2);
                                index.push(item.x_index as i16);
                            });
                            let mut value = ssab::ssab::PartAttributeDeformT::default();
                            value.coordinate = Some(coordinate);
                            value.index = Some(index);
                            m_args.value = Some(Box::new(value));
                        }
                        part_animation_data_t.deform_array.get_or_insert_with(Vec::new).push(m_args);
                    });
                },
                sspj::ssattribute::SsAttributeKind::Texchange(v) => {
                    v.keys.iter().for_each(|key| {
                        let mut m_args = ssab::ssab::PartAnimationDataUIntValueT::default();
                        m_args.index = key.time as u16;
                        m_args.interpolation = ssab::ssab::InterpolationType::None;
                        m_args.value = crate::utils::fnv1a_hash_str(&key.value.texture_name);
                        part_animation_data_t.texture_array.get_or_insert_with(Vec::new).push(m_args);
                    });
                },
                _ => {},
            }
        });
        part_animation_data_t
    }).collect::<Vec<_>>()
}

fn convert_alpha_blend_type(blend_type: &sspj::sstypes::SsBlendType) -> ssab::ssab::BlendType {
    match blend_type {
        sspj::sstypes::SsBlendType::Mix => ssab::ssab::BlendType::Mix,
        sspj::sstypes::SsBlendType::Mul => ssab::ssab::BlendType::Mul,
        sspj::sstypes::SsBlendType::Add => ssab::ssab::BlendType::Add,
        sspj::sstypes::SsBlendType::Sub => ssab::ssab::BlendType::Sub,
        sspj::sstypes::SsBlendType::Mulalpha => ssab::ssab::BlendType::Mulalpha,
        sspj::sstypes::SsBlendType::Screen => ssab::ssab::BlendType::Screen,
        sspj::sstypes::SsBlendType::Exclusion => ssab::ssab::BlendType::Exclusion,
        sspj::sstypes::SsBlendType::Invert => ssab::ssab::BlendType::Invert,
        sspj::sstypes::SsBlendType::Mul2 => ssab::ssab::BlendType::Mul2,
        sspj::sstypes::SsBlendType::Div2 => ssab::ssab::BlendType::Div2,
        sspj::sstypes::SsBlendType::Screen2 => ssab::ssab::BlendType::Screen2,
        sspj::sstypes::SsBlendType::Overlay2 => ssab::ssab::BlendType::Overlay2,
    }
}

pub fn convert_part_list(anime_pack: &sspj::ssae::SsAnimePack) -> Vec<ssab::ssab::PartDataT> {
    anime_pack.model.part_list.iter().map(|part| {
        let mut part_data_t = ssab::ssab::PartDataT::default();
        part_data_t.name = Some(part.name.to_string());
        part_data_t.parent_index = part.parent_index as i16;
        part_data_t.bounds_type = match &part.bounds_type {
            sspj::sstypes::SsBoundsType::None => ssab::ssab::BoundsType::None,
            sspj::sstypes::SsBoundsType::Quad => ssab::ssab::BoundsType::Quad,
            sspj::sstypes::SsBoundsType::Aabb => ssab::ssab::BoundsType::Aabb,
            sspj::sstypes::SsBoundsType::Circle => ssab::ssab::BoundsType::Circle,
            sspj::sstypes::SsBoundsType::CircleSmin => ssab::ssab::BoundsType::Circle,
            sspj::sstypes::SsBoundsType::CircleSmax => ssab::ssab::BoundsType::Circle,
        };
        part_data_t.blend_type = convert_alpha_blend_type(&part.alpha_blend_type);
        part_data_t.part_type = match &part.part_type {
            sspj::sstypes::SsPartType::Null => ssab::ssab::PartTypeT::PartTypeNull(Box::new(ssab::ssab::PartTypeNoneT::default())),
            sspj::sstypes::SsPartType::Normal => ssab::ssab::PartTypeT::PartTypeNormal(Box::new(ssab::ssab::PartTypeNoneT::default())),
            sspj::sstypes::SsPartType::Shape => {
                let mut part_type = ssab::ssab::PartTypeShapeT::default();
                part_type.shape_type = match &part.shape_type {
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
                part_type.ref_anime_hash = crate::utils::fnv1a_hash_str(&part.ref_anime);
                ssab::ssab::PartTypeT::PartTypeInstance(Box::new(part_type))
            },
            sspj::sstypes::SsPartType::Armature => ssab::ssab::PartTypeT::PartTypeArmature(Box::new(ssab::ssab::PartTypeNoneT::default())),
            sspj::sstypes::SsPartType::Effect => {
                let mut part_type = ssab::ssab::PartTypeEffectT::default();
                part_type.ref_effect_name_hash = crate::utils::fnv1a_hash_str(&part.ref_effect_name);
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

pub fn convert_animation_data_list(anime_pack: &sspj::ssae::SsAnimePack, parts_map: &HashMap<String, &sspj::ssae::SsPart>) -> Vec<ssab::ssab::AnimationDataT> {
    let Some(setup_animation) = anime_pack.anime_list.iter().find(|anime_data| anime_data.is_setup) else {
        // not found setup animation
        return vec![];
    };

    anime_pack.anime_list.iter().filter_map(|anime_data| {
        if anime_data == setup_animation {
            return None;
        }
        Some(convert_animedatat(anime_data, setup_animation, parts_map))
    }).collect::<Vec<ssab::ssab::AnimationDataT>>()
}
