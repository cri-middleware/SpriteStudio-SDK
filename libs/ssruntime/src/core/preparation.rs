use crate::runtime;

fn get_parts_initial_attributes<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, p: &flatbuffers::Vector<flatbuffers::ForwardsUOffset<ssab::ssab::PartAttributes>>) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<crate::runtime::preparation::ss::runtime::PartAttributes<'a>>>> {
    let parts_initial_attributes = p.iter().map(|part_attributes| {
        let mut p_args = runtime::preparation::ss::runtime::PartAttributesArgs::default();
        p_args.cell = part_attributes.cell().map(|c| {
            let mut c_args = runtime::preparation::ss::runtime::PartAttributeCellArgs::default();
            c_args.map_id = c.map_id();
            c_args.name = c.name().map(|s| fb.create_string(s));
            runtime::preparation::ss::runtime::PartAttributeCell::create(fb, &c_args)
        });
        part_attributes.position().map(|p| {
            p_args.position_x = p.v1();
            p_args.position_y = p.v2();
            p_args.position_z = p.v3();
        });
        part_attributes.rotation().map(|r| {
            p_args.rotation_x = r.v1();
            p_args.rotation_y = r.v2();
            p_args.rotation_z = r.v3();
        });
        part_attributes.scale().map(|s| {
            p_args.scale_x = s.v1();
            p_args.scale_y = s.v2();
        });
        part_attributes.local_scale().map(|s| {
            p_args.local_scale_x = s.v1();
            p_args.local_scale_y = s.v2();
        });
        p_args.opacity = part_attributes.opacity();
        p_args.localopacity = part_attributes.localopacity();
        p_args.priority = part_attributes.priority() as f32;
        p_args.flip_v = part_attributes.flip_v();
        p_args.flip_h = part_attributes.flip_h();
        p_args.hide = part_attributes.hide();
        p_args.parts_color;
        p_args.shader;
        p_args.vertex;
        part_attributes.pivot().map(|p| {
            p_args.pivot_x = p.v1();
            p_args.pivot_y = p.v2();
        });
        p_args.anchor_x;
        p_args.anchor_y;
        part_attributes.size().map(|s| {
            p_args.size_x = s.v1();
            p_args.size_y = s.v2();
        });
        p_args.img_flip_h;
        p_args.img_flip_v;
        part_attributes.uv_move().map(|uv_move| {
            p_args.uv_move_x = uv_move.v1();
            p_args.uv_move_y = uv_move.v2();
            p_args.uv_move_z = uv_move.v3();
        });
        part_attributes.uv_scale().map(|uv_scale| {
            p_args.uv_scale_x = uv_scale.v1();
            p_args.uv_scale_y = uv_scale.v2();
        });
        p_args.bounding_radius = part_attributes.bounding_radius();
        p_args.mask = part_attributes.mask();
        part_attributes.skew().map(|skew| {
            p_args.skew_x = skew.v1();
            p_args.skew_y = skew.v2();
        });
        p_args.deform = part_attributes.deform().map(|d| {
            let mut d_args = runtime::preparation::ss::runtime::PartAttributeDeformArgs::default();
            d_args.coordinate = d.coordinate().map(|coordinate| { let v = coordinate.iter().map(|c| runtime::preparation::ss::runtime::Vec2::new(c.v1(), c.v2())).collect::<Vec<_>>(); fb.create_vector(&v)});
            d_args.index = d.index().map(|index| { let v = index.iter().map(|i| i).collect::<Vec<_>>(); fb.create_vector(&v)});

            runtime::preparation::ss::runtime::PartAttributeDeform::create(fb, &d_args)
        });
        p_args.texture = part_attributes.texture();

        runtime::preparation::ss::runtime::PartAttributes::create(fb, &p_args)
    }).collect::<Vec<_>>();
    fb.create_vector(&parts_initial_attributes)
}

fn get_parts_mesh_data<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, p: &flatbuffers::Vector<flatbuffers::ForwardsUOffset<ssab::ssab::PartMeshData>>) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<runtime::preparation::ss::runtime::PartMeshData<'a>>>> {
    let parts_mesh_data = p.iter().map(|part_mesh_data| {
        let mut p_args = runtime::preparation::ss::runtime::PartMeshDataArgs::default();
        p_args.part_index = part_mesh_data.part_index();
        p_args.uv = part_mesh_data.uv().map(|uv| { let v = uv.iter().map(|u| u).collect::<Vec<_>>(); fb.create_vector(&v) });
        p_args.indices = part_mesh_data.indices().map(|indices| { let v = indices.iter().map(|i| i).collect::<Vec<_>>(); fb.create_vector(&v) });
        p_args.count_coordinate_mesh = part_mesh_data.count_coordinate_mesh();
        p_args.index_coordinate_deform = part_mesh_data.index_coordinate_deform().map(|i| { let v = i.iter().map(|i| i).collect::<Vec<_>>(); fb.create_vector(&v) });

        runtime::preparation::ss::runtime::PartMeshData::create(fb, &p_args)
    }).collect::<Vec<_>>();
    fb.create_vector(&parts_mesh_data)
}

fn get_users<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, u: &flatbuffers::Vector<flatbuffers::ForwardsUOffset<ssab::ssab::EventUserData>>) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<runtime::preparation::ss::runtime::EventUserData<'a>>>> {
    let users = u.iter().map(|user| {
        let mut u_args = runtime::preparation::ss::runtime::EventUserDataArgs::default();
        u_args.part_index = user.part_index();
        u_args.value = user.value().map(|udv| {
            let mut ud_args = runtime::preparation::ss::runtime::PartAttributeUserDataArgs::default();
            ud_args.integer = udv.integer().map(|i| {
                let mut i_args = runtime::preparation::ss::runtime::IntValueEntryArgs::default();
                i_args.value = i.value();
                runtime::preparation::ss::runtime::IntValueEntry::create(fb, &i_args)
            });
            let rect = udv.rect().map(|r| { runtime::preparation::ss::runtime::Rect::new(r.x1(), r.y1(), r.x2(), r.y2()) });
            ud_args.rect = rect.as_ref();
            let point = udv.point().map(|p| { runtime::preparation::ss::runtime::Vec2::new(p.v1(), p.v2()) });
            ud_args.point = point.as_ref();
            ud_args.str = udv.str().map(|s| fb.create_string(s));

            runtime::preparation::ss::runtime::PartAttributeUserData::create(fb, &ud_args)
        });

        runtime::preparation::ss::runtime::EventUserData::create(fb, &u_args)
    }).collect::<Vec<_>>();
    fb.create_vector(&users)
}

fn get_signals<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, s: &flatbuffers::Vector<flatbuffers::ForwardsUOffset<ssab::ssab::EventSignal>>) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<crate::runtime::preparation::ss::runtime::EventSignal<'a>>>> {
    let signals = s.iter().map(|signal| {
        let mut s_args = runtime::preparation::ss::runtime::EventSignalArgs::default();
        s_args.part_index = signal.part_index();
        s_args.value = signal.value().map(|v| {
            let mut v_args = runtime::preparation::ss::runtime::PartAttributeSignalArgs::default();
            v_args.active = v.active();
            v_args.command_id = v.command_id().map(|s| fb.create_string(s));
            v_args.params = v.params().map(|ps| {
                let pv = ps.iter().map(|p| {
                    let mut p_args = runtime::preparation::ss::runtime::SignalCommandParamArgs::default();
                    p_args.param_id = p.param_id().map(|s| fb.create_string(s));
                    p_args.type_ = match p.type_() {
                        ssab::ssab::SignalCommandParamType::Index => runtime::preparation::ss::runtime::SignalCommandParamType::Index,
                        ssab::ssab::SignalCommandParamType::Integer => runtime::preparation::ss::runtime::SignalCommandParamType::Integer,
                        ssab::ssab::SignalCommandParamType::Floating => runtime::preparation::ss::runtime::SignalCommandParamType::Floating,
                        ssab::ssab::SignalCommandParamType::None => runtime::preparation::ss::runtime::SignalCommandParamType::None,
                        _ => runtime::preparation::ss::runtime::SignalCommandParamType::None,
                    };
                    p_args.value_type = match p.value_type() {
                        ssab::ssab::SignalCommandParamValue::NONE => runtime::preparation::ss::runtime::SignalCommandParamValue::NONE,
                        ssab::ssab::SignalCommandParamValue::Bool => runtime::preparation::ss::runtime::SignalCommandParamValue::Bool,
                        ssab::ssab::SignalCommandParamValue::Index => runtime::preparation::ss::runtime::SignalCommandParamValue::Index,
                        ssab::ssab::SignalCommandParamValue::Integer => runtime::preparation::ss::runtime::SignalCommandParamValue::Integer,
                        ssab::ssab::SignalCommandParamValue::Floating => runtime::preparation::ss::runtime::SignalCommandParamValue::Floating,
                        ssab::ssab::SignalCommandParamValue::Text => runtime::preparation::ss::runtime::SignalCommandParamValue::Text,
                        ssab::ssab::SignalCommandParamValue::None => runtime::preparation::ss::runtime::SignalCommandParamValue::None,
                        _ => runtime::preparation::ss::runtime::SignalCommandParamValue::NONE,
                    };
                    p_args.value = match p.value_type() {
                        ssab::ssab::SignalCommandParamValue::NONE => None,
                        ssab::ssab::SignalCommandParamValue::Bool => {
                            let b = p.value_as_bool().unwrap();
                            let mut args = runtime::preparation::ss::runtime::BoolValueEntryArgs::default();
                            args.value = b.value();
                            let offset = runtime::preparation::ss::runtime::BoolValueEntry::create(fb, &args);
                            Some(offset.as_union_value())
                        },
                        ssab::ssab::SignalCommandParamValue::Index => {
                            let i = p.value_as_index().unwrap();
                            let mut args = runtime::preparation::ss::runtime::IntValueEntryArgs::default();
                            args.value = i.value();
                            let offset = runtime::preparation::ss::runtime::IntValueEntry::create(fb, &args);
                            Some(offset.as_union_value())
                        },
                        ssab::ssab::SignalCommandParamValue::Integer => {
                            let i = p.value_as_integer().unwrap();
                            let mut args = runtime::preparation::ss::runtime::IntValueEntryArgs::default();
                            args.value = i.value();
                            let offset = runtime::preparation::ss::runtime::IntValueEntry::create(fb, &args);
                            Some(offset.as_union_value())
                        },
                        ssab::ssab::SignalCommandParamValue::Floating => {
                            let f = p.value_as_floating().unwrap();
                            let mut args = runtime::preparation::ss::runtime::FloatValueEntryArgs::default();
                            args.value = f.value();
                            let offset = runtime::preparation::ss::runtime::FloatValueEntry::create(fb, &args);
                            Some(offset.as_union_value())
                        },
                        ssab::ssab::SignalCommandParamValue::Text => {
                            let s = p.value_as_text().unwrap();
                            let mut args = runtime::preparation::ss::runtime::StringValueEntryArgs::default();
                            args.value = s.value().map(|s| fb.create_string(s));
                            let offset = runtime::preparation::ss::runtime::StringValueEntry::create(fb, &args);
                            Some(offset.as_union_value())
                        },
                        ssab::ssab::SignalCommandParamValue::None => {
                            let mut args = runtime::preparation::ss::runtime::NoneValueEntryArgs::default();
                            let offset = runtime::preparation::ss::runtime::NoneValueEntry::create(fb, &args);
                            Some(offset.as_union_value())
                        },
                        _ => None,
                    };

                    runtime::preparation::ss::runtime::SignalCommandParam::create(fb, &p_args)
                }).collect::<Vec<_>>();

                fb.create_vector(&pv)
            });

            runtime::preparation::ss::runtime::PartAttributeSignal::create(fb, &v_args)
        });

        runtime::preparation::ss::runtime::EventSignal::create(fb, &s_args)
    }).collect::<Vec<_>>();
    fb.create_vector(&signals)
}

fn get_audios<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, a: &flatbuffers::Vector<flatbuffers::ForwardsUOffset<ssab::ssab::EventAudio>>) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<crate::runtime::preparation::ss::runtime::EventAudio<'a>>>> {
    let audios = a.iter().map(|audio| {
        let mut a_args = runtime::preparation::ss::runtime::EventAudioArgs::default();
        a_args.part_index = audio.part_index();
        a_args.value = audio.value().map(|audio| {
            let mut audio_args = runtime::preparation::ss::runtime::PartAttributeAudioArgs::default();
            audio_args.sound_list_id = audio.sound_list_id();
            audio_args.sound_name = audio.sound_name().map(|s| fb.create_string(s));
            audio_args.loop_num = audio.loop_num();

            runtime::preparation::ss::runtime::PartAttributeAudio::create(fb, &audio_args)
        });

        runtime::preparation::ss::runtime::EventAudio::create(fb, &a_args)
    }).collect::<Vec<_>>();
    fb.create_vector(&audios)
}

fn get_instances<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, i: &flatbuffers::Vector<flatbuffers::ForwardsUOffset<ssab::ssab::EventInstance>>) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<crate::runtime::preparation::ss::runtime::EventInstance<'a>>>> {
    let instances = i.iter().map(|instance| {
        let mut i_args = runtime::preparation::ss::runtime::EventInstanceArgs::default();
        i_args.part_index = instance.part_index();
        i_args.value = instance.value().map(|v| {
            let mut v_args = runtime::preparation::ss::runtime::PartAttributeInstanceArgs::default();
            v_args.pingpong = v.pingpong();
            v_args.independent = v.independent();
            v_args.loop_num = v.loop_num();
            v_args.start_label = v.start_label().map(|s| fb.create_string(s));
            v_args.start_offset = v.start_offset();
            v_args.end_label = v.end_label().map(|s| fb.create_string(s));
            v_args.end_offset = v.end_offset();
            v_args.speed = v.speed();

            runtime::preparation::ss::runtime::PartAttributeInstance::create(fb, &v_args)
        });
        runtime::preparation::ss::runtime::EventInstance::create(fb, &i_args)
    }).collect::<Vec<_>>();
    fb.create_vector(&instances)
}

fn get_effects<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, e: &flatbuffers::Vector<flatbuffers::ForwardsUOffset<ssab::ssab::EventEffect>>) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<crate::runtime::preparation::ss::runtime::EventEffect<'a>>>> {
    let effects = e.iter().map(|effect| {
        let mut e_args = runtime::preparation::ss::runtime::EventEffectArgs::default();
        e_args.part_index = effect.part_index();
        e_args.value = effect.value().map(|v| {
            let mut v_args = runtime::preparation::ss::runtime::PartAttributeEffectArgs::default();
            v_args.cur_keyframe = v.cur_keyframe();
            v_args.start_time = v.start_time();
            v_args.speed = v.speed();
            v_args.loopflag = v.loopflag();

            runtime::preparation::ss::runtime::PartAttributeEffect::create(fb, &v_args)
        });
        runtime::preparation::ss::runtime::EventEffect::create(fb, &e_args)
    }).collect::<Vec<_>>();
    fb.create_vector(&effects)
}

fn get_events<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, e: &flatbuffers::Vector<flatbuffers::ForwardsUOffset<ssab::ssab::EventsPerFrame>>) -> flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<crate::runtime::preparation::ss::runtime::EventsPerFrame<'a>>>> {
    let events = e.iter().map(|event| {
        let mut e_args = runtime::preparation::ss::runtime::EventsPerFrameArgs::default();
        e_args.frame_index = event.frame_index();
        e_args.users = event.users().map(|users| get_users(fb, &users));
        e_args.signals = event.signals().map(|signals| get_signals(fb, &signals));
        e_args.audios = event.audios().map(|audios| get_audios(fb, &audios));
        e_args.instances = event.instances().map(|instances| get_instances(fb, &instances));
        e_args.effects = event.effects().map(|effects| get_effects(fb, &effects));

        runtime::preparation::ss::runtime::EventsPerFrame::create(fb, &e_args)
    }).collect::<Vec<_>>();
    fb.create_vector(&events)
}

fn get_labels<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, l: &flatbuffers::Vector<flatbuffers::ForwardsUOffset<ssab::ssab::Label>>) -> ::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<crate::runtime::preparation::ss::runtime::Label<'a>>>> {
    let labels = l.iter().map(|label| {
        let mut l_args = runtime::preparation::ss::runtime::LabelArgs::default();
        l_args.name = Some(fb.create_string(label.name()));
        l_args.time = label.time();

        runtime::preparation::ss::runtime::Label::create(fb, &l_args)
    }).collect::<Vec<_>>();
    fb.create_vector(&labels)
}

pub fn get_animation_preparation_data<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, animation: &ssab::ssab::AnimationData) -> flatbuffers::WIPOffset<runtime::preparation::ss::runtime::AnimPrepData<'a>> {
    let mut args = runtime::preparation::ss::runtime::AnimPrepDataArgs::default();
    args.parts_initial_attributes = animation.parts_initial_attributes().map(|parts_initial_attributes| get_parts_initial_attributes(fb, &parts_initial_attributes));
    args.parts_mesh_data = animation.parts_mesh_data().map(|parts_mesh_data| get_parts_mesh_data(fb, &parts_mesh_data));
    args.events = animation.events().map(|events| get_events(fb, &events));
    args.labels = animation.labels().map(|labels| get_labels(fb, &labels) );
    args.start_frame = animation.start_frame();
    args.end_frame = animation.end_frame();
    args.total_frame = animation.total_frame();
    args.fps = animation.fps();
    let canvas = animation.canvas().map(|c| runtime::preparation::ss::runtime::Rect::new(c.x1(), c.y1(), c.x2(), c.y2()));
    args.canvas = canvas.as_ref();

    runtime::preparation::ss::runtime::AnimPrepData::create(fb, &args)
}
