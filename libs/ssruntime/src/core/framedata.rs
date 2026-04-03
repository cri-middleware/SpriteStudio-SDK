use crate::runtime;
use crate::core::interpolation::*;
use alloc::vec::Vec;

fn get_part<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>,
                part: &ssab::ssab::ss::format::PartAnimationData,
                part_index: i16,
                frame_no: i32,
                cells: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeCell>,
                parts_color: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
                shaders: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
                vertices: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
                deforms: &mut Vec<flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>, part_cursors: &mut [usize]) -> Option<runtime::framedata::ss::runtime::PartState> {
    let mut part_state = runtime::framedata::ss::runtime::PartState::default();
    let mut update_flag = runtime::framedata::ss::runtime::UpdateAttributeFlags::empty();

    part_state.set_part_index(part_index);

    part.cell_array().and_then(|arr| evaluate_cell(arr, frame_no, Some(&mut part_cursors[0]))).map(|value| {
        cells.push(runtime::framedata::ss::runtime::PartAttributeCell::new(value.map_id(), value.name_hash()));
        part_state.set_cell((cells.len() - 1) as i16);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeCell;
    });
    part.position_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[1]))).map(|(val, mask)| {
        part_state.set_position_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX & mask;
    });
    part.position_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[2]))).map(|(val, mask)| {
        part_state.set_position_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosY & mask;
    });
    part.position_z_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[3]))).map(|(val, mask)| {
        part_state.set_position_z(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosZ & mask;
    });
    part.rotation_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[4]))).map(|(val, mask)| {
        part_state.set_rotation_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotX & mask;
    });
    part.rotation_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[5]))).map(|(val, mask)| {
        part_state.set_rotation_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotY & mask;
    });
    part.rotation_z_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[6]))).map(|(val, mask)| {
        part_state.set_rotation_z(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotZ & mask;
    });
    part.scale_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[7]))).map(|(val, mask)| {
        part_state.set_scale_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSclX & mask;
    });
    part.scale_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[8]))).map(|(val, mask)| {
        part_state.set_scale_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSclY & mask;
    });
    part.local_scale_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[9]))).map(|(val, mask)| {
        part_state.set_local_scale_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLosclX & mask;
    });
    part.local_scale_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[10]))).map(|(val, mask)| {
        part_state.set_local_scale_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLosclY & mask;
    });
    part.alpha_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[11]))).map(|(val, mask)| {
        part_state.set_alpha(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAlpha & mask;
    });
    part.local_alpha_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[12]))).map(|(val, mask)| {
        part_state.set_local_alpha(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLoAlpha & mask;
    });
    part.priority_array().and_then(|arr| evaluate_int(arr, frame_no, Some(&mut part_cursors[13]))).map(|(val, mask)| {
        part_state.set_priority(val as f32);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePrio & mask;
    });
    part.flip_h_array().and_then(|arr| evaluate_bool(arr, frame_no, Some(&mut part_cursors[14]))).map(|val| {
        part_state.set_flip_h(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipH;
    });
    part.flip_v_array().and_then(|arr| evaluate_bool(arr, frame_no, Some(&mut part_cursors[15]))).map(|val| {
        part_state.set_flip_v(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipV;
    });
    part.hide_array().and_then(|arr| evaluate_bool(arr, frame_no, Some(&mut part_cursors[16]))).map(|val| {
        part_state.set_hide(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeHide;
    });

    part.part_color_array().and_then(|arr| evaluate_partcolor(arr, frame_no, Some(&mut part_cursors[17]))).map(|(val, mask)| {
        parts_color.push(val);
        part_state.set_part_color((parts_color.len() - 1) as i16);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePartColor & mask;
    });
    part.shader_array().and_then(|arr| evaluate_shader(arr, frame_no, Some(&mut part_cursors[18]))).map(|(val, mask)| {
        shaders.push(val);
        part_state.set_shader((shaders.len() - 1) as i16);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeShader & mask;
    });
    part.vertex_array().and_then(|arr| evaluate_vertex(arr, frame_no, Some(&mut part_cursors[19]))).map(|(val, mask)| {
        vertices.push(val);
        part_state.set_vertex((vertices.len() - 1) as i16);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeVertex & mask;
    });

    part.pivot_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[20]))).map(|(val, mask)| {
        part_state.set_pivot_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotX & mask;
    });
    part.pivot_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[21]))).map(|(val, mask)| {
        part_state.set_pivot_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotY & mask;
    });
    part.anchor_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[22]))).map(|(val, mask)| {
        part_state.set_anchor_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAnchorX & mask;
    });
    part.anchor_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[23]))).map(|(val, mask)| {
        part_state.set_anchor_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAnchorY & mask;
    });
    part.size_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[24]))).map(|(val, mask)| {
        part_state.set_size_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeX & mask;
    });
    part.size_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[25]))).map(|(val, mask)| {
        part_state.set_size_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeY & mask;
    });
    part.img_flip_h_array().and_then(|arr| evaluate_bool(arr, frame_no, Some(&mut part_cursors[26]))).map(|val| {
        part_state.set_img_flip_h(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeImgFlipH;
    });
    part.img_flip_v_array().and_then(|arr| evaluate_bool(arr, frame_no, Some(&mut part_cursors[27]))).map(|val| {
        part_state.set_img_flip_v(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeImgFlipV;
    });
    part.uv_translation_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[28]))).map(|(val, mask)| {
        part_state.set_uv_translation_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvtX & mask;
    });
    part.uv_translation_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[29]))).map(|(val, mask)| {
        part_state.set_uv_translation_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvtY & mask;
    });
    part.uv_rotation_z_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[30]))).map(|(val, mask)| {
        part_state.set_uv_rotation_z(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvrZ & mask;
    });
    part.uv_scale_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[31]))).map(|(val, mask)| {
        part_state.set_uv_scale_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvsX & mask;
    });
    part.uv_scale_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[32]))).map(|(val, mask)| {
        part_state.set_uv_scale_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvsY & mask;
    });
    part.bounding_radius_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[33]))).map(|(val, mask)| {
        part_state.set_bounding_radius(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeBoundr & mask;
    });
    part.mask_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[34]))).map(|(val, mask)| {
        part_state.set_mask(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeMask & mask;
    });

    part.skew_array().and_then(|arr| evaluate_skew(arr, frame_no, Some(&mut part_cursors[35]))).map(|(val_x, val_y, mask)| {
        part_state.set_skew_x(val_x);
        part_state.set_skew_y(val_y);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSkew & mask;
    });
    part.deform_array().and_then(|arr| evaluate_deform(fb, arr, frame_no, Some(&mut part_cursors[36]))).map(|(val, mask)| {
        deforms.push(val);
        part_state.set_deform((deforms.len() - 1) as i16);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeDeform & mask;
    });
    part.texture_array().and_then(|arr| evaluate_texture(arr, frame_no, Some(&mut part_cursors[37]))).map(|val| {
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
                 parts_vec: &mut Vec<runtime::framedata::ss::runtime::PartState>,
                 cell_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeCell>,
                 parts_color_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
                 shader_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
                 vertice_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
                 deform_vec: &mut Vec<flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>,
                 cells_off: &mut Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeCell>>>,
                 parts_color_off: &mut Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributePartColor>>>,
                 shaders_off: &mut Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeShader>>>,
                 vertices_off: &mut Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeVertex>>>,
                 deforms_off: &mut Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>>>,
                 cursors: &mut [usize]
) {
    parts_vec.clear();
    cell_vec.clear();
    parts_color_vec.clear();
    shader_vec.clear();
    vertice_vec.clear();
    deform_vec.clear();

    for (i, part) in p.iter().enumerate() {
        let part_cursors = &mut cursors[i * 38 .. (i + 1) * 38];
        if let Some(state) = get_part(fb, &part, i as i16, frame_no, cell_vec, parts_color_vec, shader_vec, vertice_vec, deform_vec, part_cursors) {
            parts_vec.push(state);
        }
    }

    if !cell_vec.is_empty() {
        *cells_off = Some(fb.create_vector(&cell_vec));
    } else { *cells_off = None; }

    if !parts_color_vec.is_empty() {
        *parts_color_off = Some(fb.create_vector(&parts_color_vec));
    } else { *parts_color_off = None; }

    if !shader_vec.is_empty() {
        *shaders_off = Some(fb.create_vector(&shader_vec));
    } else { *shaders_off = None; }

    if !vertice_vec.is_empty() {
        *vertices_off = Some(fb.create_vector(&vertice_vec));
    } else { *vertices_off = None; }

    if !deform_vec.is_empty() {
        *deforms_off = Some(fb.create_vector(&deform_vec));
    } else { *deforms_off = None; }
}


pub fn get_frame_data<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>,
                          animation: &ssab::ssab::ss::format::AnimationData,
                          frame_no: i32,
                          parts_vec: &mut Vec<runtime::framedata::ss::runtime::PartState>,
                          cell_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeCell>,
                          parts_color_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
                          shader_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
                          vertice_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
                          deform_vec: &mut Vec<flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>,
                          cursors: &mut [usize]
) -> flatbuffers::WIPOffset<runtime::framedata::ss::runtime::FrameData<'a>> {
    let mut args = runtime::framedata::ss::runtime::FrameDataArgs::default();
    let mut cells_off: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeCell>>> = None;
    let mut parts_color_off: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributePartColor>>> = None;
    let mut shaders_off: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeShader>>> = None;
    let mut vertices_off: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeVertex>>> = None;
    let mut deforms_off: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>>> = None;

    if let Some(parts_animation_data) = animation.parts_animation_data() {
        get_parts(fb, &parts_animation_data, frame_no, parts_vec, cell_vec, parts_color_vec, shader_vec, vertice_vec, deform_vec, &mut cells_off, &mut parts_color_off, &mut shaders_off, &mut vertices_off, &mut deforms_off, cursors);
        if !parts_vec.is_empty() {
            args.parts = Some(fb.create_vector(&parts_vec));
        }
    }

    args.cells = cells_off;
    args.parts_color = parts_color_off;
    args.shaders = shaders_off;
    args.vertices = vertices_off;
    args.deforms = deforms_off;
    runtime::framedata::ss::runtime::FrameData::create(fb, &args)
}
