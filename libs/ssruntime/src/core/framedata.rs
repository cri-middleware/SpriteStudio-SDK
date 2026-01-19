use crate::runtime;
use crate::runtime::framedata::ss::runtime::{PartAttributeCell, PartAttributePartColor, PartAttributeShader};

fn get_part<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, part: &ssab::ssab::PartAnimationData, frame_no: i32, deforms: &mut Vec<flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>) -> ::flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartState<'a>> {
    let mut p_args = runtime::framedata::ss::runtime::PartStateArgs::default();

    let mut cell_: Option<PartAttributeCell> = None;
    p_args.update_flag = runtime::framedata::ss::runtime::UpdateAttributeFlags::empty();
    part.cell_array().map(|cell_array| {
        cell_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).map(|cell| {
            cell.value().map(|value| {
                cell_ = Some(runtime::framedata::ss::runtime::PartAttributeCell::new(value.map_id(), value.name_hash()));

            });
        });
    });
    if cell_.is_some() {
        p_args.cell = cell_.as_ref();
        p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeCell;
    }
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

    let mut part_color: Option<PartAttributePartColor> = None;
    part.parts_color_array().and_then(|parts_color_array| {
        parts_color_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).and_then(|parts_color| {
            parts_color.value().map(|color| {
                let blend_type = match color.blend_type() {
                    ssab::ssab::BlendType::Mix => runtime::framedata::ss::runtime::BlendType::Mix,
                    ssab::ssab::BlendType::Mul => runtime::framedata::ss::runtime::BlendType::Mul,
                    ssab::ssab::BlendType::Add => runtime::framedata::ss::runtime::BlendType::Add,
                    ssab::ssab::BlendType::Sub => runtime::framedata::ss::runtime::BlendType::Sub,
                    ssab::ssab::BlendType::Mulalpha => runtime::framedata::ss::runtime::BlendType::Mulalpha,
                    ssab::ssab::BlendType::Screen => runtime::framedata::ss::runtime::BlendType::Screen,
                    ssab::ssab::BlendType::Exclusion => runtime::framedata::ss::runtime::BlendType::Exclusion,
                    ssab::ssab::BlendType::Invert => runtime::framedata::ss::runtime::BlendType::Invert,
                    // TODO: blend mode
                    _ => runtime::framedata::ss::runtime::BlendType::Mix,
                };
                fn convert_u8rect(rgba: &ssab::ssab::U8Rect) -> runtime::framedata::ss::runtime::U8Rect {
                    runtime::framedata::ss::runtime::U8Rect::new(rgba.a(), rgba.r(), rgba.g(), rgba.b())
                }
                let lt = color.lt().and_then(|lt|
                    lt.rgba().map(|rgba| runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&rgba), lt.rate()))
                ).unwrap(); // TODO: improve
                let rt = color.rt().and_then(|rt|
                    rt.rgba().map(|rgba| runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&rgba), rt.rate()))
                ).unwrap(); // TODO: improve
                let lb = color.lb().and_then(|lb|
                    lb.rgba().map(|rgba| runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&rgba), lb.rate()))
                ).unwrap(); // TODO: improve
                let rb = color.rb().and_then(|rb|
                    rb.rgba().map(|rgba| runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&rgba), rb.rate()))
                ).unwrap(); // TODO: improve
                let a = runtime::framedata::ss::runtime::PartAttributePartColor::new(
                    blend_type,
                    &lt,
                    &rt,
                    &lb,
                    &rb
                );
                part_color = Some(a);
            })
        })
    });
    if part_color.is_some() {
        p_args.part_color = part_color.as_ref();
        p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePartColor;
    }

    let mut shader_: Option<PartAttributeShader> = None;
    part.shader_array().and_then(|shader_array| {
        shader_array.lookup_by_key(frame_no, |value, key| { value.key_compare_with_value(*key as u16) }).and_then(|shader| {
            shader.value().map(|shader| {
                let s = runtime::framedata::ss::runtime::PartAttributeShader::new(
                    shader.id_hash(),
                    shader.param0(),
                    shader.param1(),
                    shader.param2(),
                    shader.param3(),
                    shader.param4(),
                    shader.param5(),
                    shader.param6(),
                    shader.param7(),
                    shader.map0_id(),
                    shader.map0_name_hash(),
                    shader.map1_id(),
                    shader.map1_name_hash()
                );
                shader_ = Some(s);
            })
        })
    });
    if shader_.is_some() {
        p_args.shader = shader_.as_ref();
        p_args.update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeShader;
    }
    part.vertex_array().map(|_vertex_array| {
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
                deforms.push(runtime::framedata::ss::runtime::PartAttributeDeform::create(fb, &d_args));
                p_args.deform = deforms.len() as i32 - 1;
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

fn get_parts<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>,
                 p: &flatbuffers::Vector<::flatbuffers::ForwardsUOffset<ssab::ssab::PartAnimationData>>,
                 frame_no: i32,
                 deforms: &mut Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>>>,
) -> flatbuffers::WIPOffset<flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<runtime::framedata::ss::runtime::PartState<'a>>>> {
    let mut deform_vec: Vec<flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform>> = Vec::new();
    let parts = p.iter().map(|part| {
        get_part(fb, &part, frame_no, &mut deform_vec)
    }).collect::<Vec<_>>();
    if !deform_vec.is_empty() {
        *deforms = Some(fb.create_vector(&deform_vec));
    }

    fb.create_vector(&parts)
}

pub fn get_frame_data<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, animation: &ssab::ssab::AnimationData, frame_no: i32) -> flatbuffers::WIPOffset<runtime::framedata::ss::runtime::FrameData<'a>> {
    let mut args = runtime::framedata::ss::runtime::FrameDataArgs::default();
    let mut deforms: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>>> = None;

    args.parts = animation.parts_animation_data().map( |parts_animation_data| get_parts(fb, &parts_animation_data, frame_no, &mut deforms));
    args.deforms = deforms;
    runtime::framedata::ss::runtime::FrameData::create(fb, &args)
}
