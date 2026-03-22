use crate::runtime;
use crate::core::interpolation::*;

fn get_part<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>,
                part: &ssab::ssab::ss::format::PartAnimationData,
                part_index: i16,
                frame_no: i32,
                cells: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeCell>,
                parts_color: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
                shaders: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
                vertices: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
                deforms: &mut Vec<flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>) -> Option<runtime::framedata::ss::runtime::PartState> {
    let mut part_state = runtime::framedata::ss::runtime::PartState::default();
    let mut update_flag = runtime::framedata::ss::runtime::UpdateAttributeFlags::empty();

    part_state.set_part_index(part_index);

    part.cell_array().and_then(|arr| evaluate_cell(arr, frame_no)).map(|value| {
        cells.push(runtime::framedata::ss::runtime::PartAttributeCell::new(value.map_id(), value.name_hash()));
        part_state.set_cell((cells.len() - 1) as i16);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeCell;
    });

    part.position_x_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_position_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX;
    });
    part.position_y_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_position_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosY;
    });
    part.position_z_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_position_z(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosZ;
    });
    part.rotation_x_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_rotation_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotX;
    });
    part.rotation_y_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_rotation_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotY;
    });
    part.rotation_z_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_rotation_z(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotZ;
    });
    part.scale_x_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_scale_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSclX;
    });
    part.scale_y_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_scale_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSclY;
    });
    part.local_scale_x_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_local_scale_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLosclX;
    });
    part.local_scale_y_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_local_scale_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLosclY;
    });
    part.alpha_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_alpha(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAlpha;
    });
    part.local_alpha_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_local_alpha(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLoAlpha;
    });
    part.priority_array().and_then(|arr| evaluate_int(arr, frame_no)).map(|val| {
        part_state.set_priority(val as f32);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePrio;
    });
    part.flip_h_array().and_then(|arr| evaluate_bool(arr, frame_no)).map(|val| {
        part_state.set_flip_h(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipH;
    });
    part.flip_v_array().and_then(|arr| evaluate_bool(arr, frame_no)).map(|val| {
        part_state.set_flip_v(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipV;
    });
    part.hide_array().and_then(|arr| evaluate_bool(arr, frame_no)).map(|val| {
        part_state.set_hide(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeHide;
    });
    part.part_color_array().and_then(|arr| search_prev_next(arr, frame_no, |v| v.index() as i32)).and_then(|(prev, _)| prev.value()).map(|color| {
                let blend_type = match color.blend_type() {
                    ssab::ssab::ss::format::BlendType::Mix => runtime::framedata::ss::runtime::BlendType::Mix,
                    ssab::ssab::ss::format::BlendType::Mul => runtime::framedata::ss::runtime::BlendType::Mul,
                    ssab::ssab::ss::format::BlendType::Add => runtime::framedata::ss::runtime::BlendType::Add,
                    ssab::ssab::ss::format::BlendType::Sub => runtime::framedata::ss::runtime::BlendType::Sub,
                    ssab::ssab::ss::format::BlendType::Mulalpha => runtime::framedata::ss::runtime::BlendType::Mulalpha,
                    ssab::ssab::ss::format::BlendType::Screen => runtime::framedata::ss::runtime::BlendType::Screen,
                    ssab::ssab::ss::format::BlendType::Exclusion => runtime::framedata::ss::runtime::BlendType::Exclusion,
                    ssab::ssab::ss::format::BlendType::Invert => runtime::framedata::ss::runtime::BlendType::Invert,
                    ssab::ssab::ss::format::BlendType::Mul2 => runtime::framedata::ss::runtime::BlendType::Mul2,
                    ssab::ssab::ss::format::BlendType::Div2 => runtime::framedata::ss::runtime::BlendType::Div2,
                    ssab::ssab::ss::format::BlendType::Screen2 => runtime::framedata::ss::runtime::BlendType::Screen2,
                    ssab::ssab::ss::format::BlendType::Overlay2 => runtime::framedata::ss::runtime::BlendType::Overlay2,
                    _ => runtime::framedata::ss::runtime::BlendType::Mix,
                };
                fn convert_u8rect(rgba: &ssab::ssab::ss::format::U8Rect) -> runtime::framedata::ss::runtime::U8Rect {
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
                parts_color.push(runtime::framedata::ss::runtime::PartAttributePartColor::new(blend_type, &lt, &rt, &lb, &rb));
                part_state.set_part_color((parts_color.len() - 1) as i16);
                update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePartColor;
            });

    part.shader_array().and_then(|arr| search_prev_next(arr, frame_no, |v| v.index() as i32)).and_then(|(prev, _)| prev.value()).map(|shader| {
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
                shaders.push(s);
                part_state.set_shader((shaders.len() - 1) as i16);
                update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeShader;
            });

    part.vertex_array().and_then(|arr| search_prev_next(arr, frame_no, |v| v.index() as i32)).and_then(|(prev, _)| prev.value()).map(|vertex| {
                let lt = vertex.lt().map(|lt| runtime::framedata::ss::runtime::Vec2::new(lt.v1(), lt.v2())).unwrap_or_default();
                let rt = vertex.rt().map(|rt| runtime::framedata::ss::runtime::Vec2::new(rt.v1(), rt.v2())).unwrap_or_default();
                let lb = vertex.lb().map(|lb| runtime::framedata::ss::runtime::Vec2::new(lb.v1(), lb.v2())).unwrap_or_default();
                let rb = vertex.rb().map(|rb| runtime::framedata::ss::runtime::Vec2::new(rb.v1(), rb.v2())).unwrap_or_default();

                vertices.push(runtime::framedata::ss::runtime::PartAttributeVertex::new(&lt, &rt, &lb, &rb));
                part_state.set_vertex((vertices.len() - 1) as i16);
                update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeVertex;
            });

    part.pivot_x_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_pivot_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotX;
    });
    part.pivot_y_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_pivot_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotY;
    });
    part.anchor_x_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_anchor_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAnchorX;
    });
    part.anchor_y_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_anchor_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAnchorY;
    });
    part.size_x_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_size_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeX;
    });
    part.size_y_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_size_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeY;
    });
    part.img_flip_h_array().and_then(|arr| evaluate_bool(arr, frame_no)).map(|val| {
        part_state.set_img_flip_h(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeImgFlipH;
    });
    part.img_flip_v_array().and_then(|arr| evaluate_bool(arr, frame_no)).map(|val| {
        part_state.set_img_flip_v(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeImgFlipV;
    });
    part.uv_translation_x_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_uv_translation_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvtX;
    });
    part.uv_translation_y_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_uv_translation_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvtY;
    });
    part.uv_rotation_z_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_uv_rotation_z(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvrZ;
    });
    part.uv_scale_x_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_uv_scale_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvsX;
    });
    part.uv_scale_y_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_uv_scale_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvsY;
    });
    part.bounding_radius_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_bounding_radius(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeBoundr;
    });
    part.mask_array().and_then(|arr| evaluate_float(arr, frame_no)).map(|val| {
        part_state.set_mask(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeMask;
    });
    part.skew_array().and_then(|arr| search_prev_next(arr, frame_no, |v| v.index() as i32)).and_then(|(prev, _)| prev.value()).map(|skew| {
                part_state.set_skew_x(skew.v1());
                part_state.set_skew_y(skew.v2());
                update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSkew;
    });
    part.deform_array().and_then(|arr| search_prev_next(arr, frame_no, |v| v.index() as i32)).and_then(|(prev, _)| prev.value()).map(|deform| {
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
                part_state.set_deform((deforms.len() - 1) as i16);
                update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeDeform;
    });
    part.texture_array().and_then(|arr| evaluate_texture(arr, frame_no)).map(|val| {
        part_state.set_texture(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeTexture;
    });

    if update_flag.is_empty() {
        return None;
    }
    part_state.set_update_flag(update_flag);
    Some(part_state)
}

fn get_parts<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>,
                 p: &flatbuffers::Vector<::flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationData>>,
                 frame_no: i32,
                 cells: &mut Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeCell>>>,
                 parts_color: &mut Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributePartColor>>>,
                 shaders: &mut Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeShader>>>,
                 vertices: &mut Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeVertex>>>,
                 deforms: &mut Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>>>
) -> Vec<crate::runtime::framedata::ss::runtime::PartState> {
    let mut cell_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeCell> = Vec::new();
    let mut parts_colo_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributePartColor> = Vec::new();
    let mut shader_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeShader> = Vec::new();
    let mut vertice_vec: Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex> = Vec::new();
    let mut deform_vec: Vec<flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform>> = Vec::new();

    let parts = p.iter().enumerate().filter_map(|(i, part)| {
        get_part(fb, &part, i as i16, frame_no, &mut cell_vec, &mut parts_colo_vec, &mut shader_vec, &mut vertice_vec, &mut deform_vec)
    }).collect::<Vec<_>>();

    if !cell_vec.is_empty() {
        *cells = Some(fb.create_vector(&cell_vec));
    }
    if !parts_colo_vec.is_empty() {
        *parts_color = Some(fb.create_vector(&parts_colo_vec));
    }
    if !shader_vec.is_empty() {
        *shaders = Some(fb.create_vector(&shader_vec));
    }
    if !vertice_vec.is_empty() {
        *vertices = Some(fb.create_vector(&vertice_vec));
    }
    if !deform_vec.is_empty() {
        *deforms = Some(fb.create_vector(&deform_vec));
    }

    parts
}


pub fn get_frame_data<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, animation: &ssab::ssab::ss::format::AnimationData, frame_no: i32) -> flatbuffers::WIPOffset<runtime::framedata::ss::runtime::FrameData<'a>> {
    let mut args = runtime::framedata::ss::runtime::FrameDataArgs::default();
    let mut cells: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeCell>>> = None;
    let mut parts_color: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributePartColor>>> = None;
    let mut shaders: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeShader>>> = None;
    let mut vertices: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeVertex>>> = None;
    let mut deforms: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>>> = None;

    // args.parts = animation.parts_animation_data().map( |parts_animation_data| get_parts(fb, &parts_animation_data, frame_no, &mut deforms));
    args.parts = animation.parts_animation_data().and_then( |parts_animation_data| {
        let parts = get_parts(fb, &parts_animation_data, frame_no, &mut cells, &mut parts_color, &mut shaders, &mut vertices, &mut deforms);
        if parts.is_empty() { None } else { Some(fb.create_vector(&parts)) }
    });
    args.cells = cells;
    args.parts_color = parts_color;
    args.shaders = shaders;
    args.vertices = vertices;
    args.deforms = deforms;
    runtime::framedata::ss::runtime::FrameData::create(fb, &args)
}
