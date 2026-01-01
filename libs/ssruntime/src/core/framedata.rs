use crate::runtime;

fn get_part<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, index: u16, part: &ssab::ssab::PartAnimationData, frame_no: i32) -> flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartState<'a>> {
    let mut p_args = runtime::framedata::ss::runtime::PartStateArgs::default();

    p_args.part_index = index;
    p_args.update_flag = runtime::framedata::ss::runtime::UpdateAttributeFlags::empty();
    part.cell_array().map(|cell_array| {
        cell_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|cell| {
            cell.value().map(|value| {
                let mut c_args = runtime::framedata::ss::runtime::PartAttributeCellArgs::default();
                c_args.map_id = value.map_id();
                c_args.name = value.name().map(|name| fb.create_string(&name));

                p_args.cell = Some(runtime::framedata::ss::runtime::PartAttributeCell::create(fb, &c_args));
                p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeCell;
            });
        });
    });
    part.pos_x_array().map(|pos_x_array| {
        pos_x_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|pos_x| {
            p_args.position_x = pos_x.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX;
        });
    });
    part.pos_y_array().map(|pos_y_array| {
        pos_y_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|pos_y| {
            p_args.position_y = pos_y.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosY;
        });
    });
    part.pos_z_array().map(|pos_z_array| {
        pos_z_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|pos_z| {
            p_args.position_z = pos_z.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosZ;
        });
    });
    part.rot_x_array().map(|rotation_x_array| {
        rotation_x_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|rotation_x| {
            p_args.rotation_x = rotation_x.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotX;
        });
    });
    part.rot_y_array().map(|rotation_y_array| {
        rotation_y_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|rotation_y| {
            p_args.rotation_y = rotation_y.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotY;
        });
    });
    part.rot_z_array().map(|rotation_z_array| {
        rotation_z_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|rotation_z| {
            p_args.rotation_z = rotation_z.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotZ;
        });
    });
    part.scl_x_array().map(|scale_x_array| {
        scale_x_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|scale_x| {
            p_args.scale_x = scale_x.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSclX;
        });
    });
    part.scl_y_array().map(|scale_y_array| {
        scale_y_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|scale_y| {
            p_args.scale_y = scale_y.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSclY;
        });
    });
    part.loscl_x_array().map(|local_scale_x_array| {
        local_scale_x_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|local_scale_x| {
            p_args.local_scale_x = local_scale_x.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLosclX;
        });
    });
    part.loscl_y_array().map(|local_scale_y_array| {
        local_scale_y_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|local_scale_y| {
            p_args.local_scale_y = local_scale_y.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLosclY;
        });
    });
    part.alpha_array().map(|alpha_array| {
        alpha_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|alpha| {
            p_args.opacity = alpha.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAlpha;
        });
    });
    part.lo_alpha_array().map(|local_alpha_array| {
        local_alpha_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|local_alpha| {
            p_args.localopacity = local_alpha.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLoAlpha;
        });
    });
    part.prio_array().map(|priority_array| {
        priority_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|priority| {
            p_args.priority = priority.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePrio;
        });
    });
    part.flip_v_array().map(|flip_v_array| {
        flip_v_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|flip_v| {
            p_args.flip_v = flip_v.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipV;
        });
    });
    part.flip_h_array().map(|flip_h_array| {
        flip_h_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|flip_h| {
            p_args.flip_h = flip_h.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipH;
        });
    });
    part.hide_array().map(|hide_array| {
        hide_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|hide| {
            p_args.hide = hide.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeHide;
        });
    });
    part.parts_color_array().map(|parts_color_array| {
        parts_color_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|parts_color| {
            p_args.parts_color = parts_color.value().map(|color| {
                let mut c_args = runtime::framedata::ss::runtime::PartAttributePartColorArgs::default();
                c_args.blend_type = match color.blend_type() {
                    ssab::ssab::BlendType::Mix => runtime::framedata::ss::runtime::BlendType::Mix,
                    ssab::ssab::BlendType::Mul => runtime::framedata::ss::runtime::BlendType::Mul,
                    ssab::ssab::BlendType::Add => runtime::framedata::ss::runtime::BlendType::Add,
                    ssab::ssab::BlendType::Sub => runtime::framedata::ss::runtime::BlendType::Sub,
                    ssab::ssab::BlendType::Mulalpha => runtime::framedata::ss::runtime::BlendType::Mulalpha,
                    ssab::ssab::BlendType::Screen => runtime::framedata::ss::runtime::BlendType::Screen,
                    ssab::ssab::BlendType::Exclusion => runtime::framedata::ss::runtime::BlendType::Exclusion,
                    ssab::ssab::BlendType::Invert => runtime::framedata::ss::runtime::BlendType::Invert,
                    _ => runtime::framedata::ss::runtime::BlendType::Mix,
                };
                fn convert_u8rect(rgba: &ssab::ssab::U8Rect) -> runtime::framedata::ss::runtime::U8Rect {
                    runtime::framedata::ss::runtime::U8Rect::new(rgba.a(), rgba.r(), rgba.g(), rgba.b())
                }
                let lt = color.lt().map(|lt|
                    lt.rgba().map(|rgba| runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&rgba), lt.rate()))
                ).flatten();
                c_args.lt = lt.as_ref();
                let rt = color.rt().map(|rt|
                    rt.rgba().map(|rgba| runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&rgba), rt.rate()))
                ).flatten();
                c_args.rt = rt.as_ref();
                let lb = color.lb().map(|lb|
                    lb.rgba().map(|rgba| runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&rgba), lb.rate()))
                ).flatten();
                c_args.lb = lb.as_ref();
                let rb = color.rb().map(|rb|
                    rb.rgba().map(|rgba| runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&rgba), rb.rate()))
                ).flatten();
                c_args.rb = rb.as_ref();

                p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePartColor;

                runtime::framedata::ss::runtime::PartAttributePartColor::create(fb, &c_args)
            });

        })
    });
    part.shader_array().map(|shader_array| {
        shader_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|shader| {
            p_args.shader = shader.value().map(|shader| {
                let mut s_args = runtime::framedata::ss::runtime::PartAttributeShaderArgs::default();
                s_args.id = shader.id().map(|id| fb.create_string(&id));
                s_args.param0 = shader.param0();
                s_args.param1 = shader.param1();
                s_args.param2 = shader.param2();
                s_args.param3 = shader.param3();
                s_args.param4 = shader.param4();
                s_args.param5 = shader.param5();
                s_args.param6 = shader.param6();
                s_args.param7 = shader.param7();
                s_args.map0_id = shader.map0_id();
                s_args.map0_name = shader.map0_name().map(|name| fb.create_string(&name));
                s_args.map1_id = shader.map1_id();
                s_args.map1_name = shader.map1_name().map(|name| fb.create_string(&name));

                p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeShader;

                runtime::framedata::ss::runtime::PartAttributeShader::create(fb, &s_args)
            });
        });
    });
    part.vertex_array().map(|vertex_array| {
        // TODO: implement
        /*
        vertex_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|vertex| {
            let v = vertex.value().map(|vertex| {
                let lt = vertex.lt().map(|lt| runtime::framedata::ss::runtime::Vec2::new(lt.v1(), lt.v2())).unwrap_or_default();
                let rt = vertex.rt().map(|rt| runtime::framedata::ss::runtime::Vec2::new(rt.v1(), rt.v2())).unwrap_or_default();
                let lb = vertex.lb().map(|lb| runtime::framedata::ss::runtime::Vec2::new(lb.v1(), lb.v2())).unwrap_or_default();
                let rb = vertex.rb().map(|rb| runtime::framedata::ss::runtime::Vec2::new(rb.v1(), rb.v2())).unwrap_or_default();

                p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeVertex;
                runtime::framedata::ss::runtime::PartAttributeVertex::new(&lt, &rt, &lb, &rb)
            });
        });
         */
    });
    part.pivot_x_array().map(|pivot_x_array| {
        pivot_x_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|pivot_x| {
            p_args.pivot_x = pivot_x.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotX;
        });
    });
    part.pivot_y_array().map(|pivot_y_array| {
        pivot_y_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|pivot_y| {
            p_args.pivot_y = pivot_y.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotY;
        });
    });
    part.anchor_x_array().map(|anchor_x_array| {
        anchor_x_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|anchor_x| {
            p_args.anchor_x = anchor_x.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAnchorX;
        });
    });
    part.anchor_y_array().map(|anchor_y_array| {
        anchor_y_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|anchor_y| {
            p_args.anchor_y = anchor_y.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAnchorY;
        });
    });
    part.size_x_array().map(|size_x_array| {
        size_x_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|size_x| {
            p_args.size_x = size_x.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeX;
        });
    });
    part.size_y_array().map(|size_y_array| {
        size_y_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|size_y| {
            p_args.size_y = size_y.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeY;
        });
    });
    part.img_flip_h_array().map(|img_flip_h_array| {
        img_flip_h_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|img_flip_h| {
            p_args.img_flip_h = img_flip_h.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeImgFlipH;
        });
    });
    part.img_flip_v_array().map(|img_flip_v_array| {
        img_flip_v_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|img_flip_v| {
            p_args.img_flip_v = img_flip_v.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeImgFlipV;
        });
    });
    part.uvt_x_array().map(|uvt_x_array| {
        uvt_x_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|uvt_x| {
            p_args.uv_move_x = uvt_x.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvtX;
        });
    });
    part.uvt_y_array().map(|uvt_y_array| {
        uvt_y_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|uvt_y| {
            p_args.uv_move_y = uvt_y.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvtY;
        });
    });
    part.uvr_z_array().map(|uvr_z_array| {
        uvr_z_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|uvr_z| {
            p_args.uv_move_z = uvr_z.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvrZ;
        });
    });
    part.uvs_x_array().map(|uvs_x_array| {
        uvs_x_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|uvs_x| {
            p_args.uv_scale_x = uvs_x.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvsX;
        });
    });
    part.uvs_y_array().map(|uvs_y_array| {
        uvs_y_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|uvs_y| {
            p_args.uv_scale_y = uvs_y.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvsY;
        });
    });
    part.boundr_array().map(|boundr_array| {
        boundr_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|boundr| {
            p_args.bounding_radius = boundr.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeBoundr;
        });
    });
    part.mask_array().map(|mask_array| {
        mask_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|mask| {
            p_args.mask = mask.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeMask;
        });
    });
    part.skew_array().map(|skew_array| {
        skew_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|skew| {
            skew.value().map(|skew| {
                p_args.skew_x = skew.v1();
                p_args.skew_y = skew.v2();
                p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSkew;
            });
        });
    });
    part.deform_array().map(|deform_array| {
        deform_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|deform| {
            deform.value().map(|deform| {
                let mut d_args = runtime::framedata::ss::runtime::PartAttributeDeformArgs::default();
                d_args.coordinate = deform.coordinate().map(|coordinate| {
                    let v = coordinate.iter().map(|v| runtime::framedata::ss::runtime::Vec2::new(v.v1(), v.v2())).collect::<Vec<_>>();
                    fb.create_vector(&v)
                });
                d_args.index = deform.index().map(|index| {
                    let v = index.iter().map(|v| v).collect::<Vec<_>>();
                    fb.create_vector(&v)
                });
                p_args.deform = Some(runtime::framedata::ss::runtime::PartAttributeDeform::create(fb, &d_args));
                p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeDeform;
            });
        });
    });
    part.texture_array().map(|texture_array| {
        texture_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|texture| {
            p_args.texture = texture.value();
            p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeTexture;
        });
    });

    runtime::framedata::ss::runtime::PartState::create(fb, &p_args)
}

fn get_parts<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, p: &flatbuffers::Vector<::flatbuffers::ForwardsUOffset<ssab::ssab::PartAnimationData>>, frame_no: i32) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<runtime::framedata::ss::runtime::PartState<'a>>>> {
    let parts = p.iter().enumerate().map(|(index, part)| {
        get_part(fb, index as u16, &part, frame_no)
    }).collect::<Vec<_>>();
    fb.create_vector(&parts)
}

pub fn get_frame_data<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, animation: &ssab::ssab::AnimationData, frame_no: i32) -> flatbuffers::WIPOffset<runtime::framedata::ss::runtime::FrameData<'a>> {
    let mut args = runtime::framedata::ss::runtime::FrameDataArgs::default();
    args.parts = animation.parts_animation_data().map( |parts_animation_data| get_parts(fb, &parts_animation_data, frame_no));

    runtime::framedata::ss::runtime::FrameData::create(fb, &args)
}
