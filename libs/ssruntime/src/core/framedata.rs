use crate::runtime;
use crate::core::interpolation::*;
use alloc::vec::Vec;

pub const NUM_PART_ATTRIBUTES: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::all().bits().count_ones() as usize;

const IDX_ATTRIBUTECELL: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeCell.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEPOSX: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEPOSY: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosY.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEPOSZ: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosZ.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEROTX: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotX.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEROTY: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotY.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEROTZ: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotZ.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTESCLX: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSclX.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTESCLY: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSclY.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTELOSCLX: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLosclX.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTELOSCLY: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLosclY.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEALPHA: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAlpha.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTELOALPHA: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLoAlpha.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEPRIO: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePrio.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEFLIPH: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipH.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEFLIPV: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipV.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEHIDE: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeHide.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEPARTCOLOR: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePartColor.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTESHADER: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeShader.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEVERTEX: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeVertex.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEPIVOTX: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotX.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEPIVOTY: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotY.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEANCHORX: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAnchorX.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEANCHORY: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAnchorY.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTESIZEX: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeX.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTESIZEY: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeY.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEIMGFLIPH: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeImgFlipH.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEIMGFLIPV: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeImgFlipV.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEUVTX: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvtX.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEUVTY: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvtY.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEUVRZ: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvrZ.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEUVSX: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvsX.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEUVSY: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvsY.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEBOUNDR: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeBoundr.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEMASK: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeMask.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTESKEW: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSkew.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEDEFORM: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeDeform.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTETEXTURE: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeTexture.bits().trailing_zeros() as usize;

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

    part.cell_array().and_then(|arr| evaluate_cell(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTECELL]))).map(|value| {
        cells.push(runtime::framedata::ss::runtime::PartAttributeCell::new(value.map_id(), value.name_hash()));
        part_state.set_cell((cells.len() - 1) as i16);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeCell;
    });
    part.position_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEPOSX]))).map(|(val, mask)| {
        part_state.set_position_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX & mask;
    });
    part.position_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEPOSY]))).map(|(val, mask)| {
        part_state.set_position_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosY & mask;
    });
    part.position_z_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEPOSZ]))).map(|(val, mask)| {
        part_state.set_position_z(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosZ & mask;
    });
    part.rotation_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEROTX]))).map(|(val, mask)| {
        part_state.set_rotation_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotX & mask;
    });
    part.rotation_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEROTY]))).map(|(val, mask)| {
        part_state.set_rotation_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotY & mask;
    });
    part.rotation_z_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEROTZ]))).map(|(val, mask)| {
        part_state.set_rotation_z(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotZ & mask;
    });
    part.scale_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTESCLX]))).map(|(val, mask)| {
        part_state.set_scale_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSclX & mask;
    });
    part.scale_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTESCLY]))).map(|(val, mask)| {
        part_state.set_scale_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSclY & mask;
    });
    part.local_scale_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTELOSCLX]))).map(|(val, mask)| {
        part_state.set_local_scale_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLosclX & mask;
    });
    part.local_scale_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTELOSCLY]))).map(|(val, mask)| {
        part_state.set_local_scale_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLosclY & mask;
    });
    part.alpha_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEALPHA]))).map(|(val, mask)| {
        part_state.set_alpha(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAlpha & mask;
    });
    part.local_alpha_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTELOALPHA]))).map(|(val, mask)| {
        part_state.set_local_alpha(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLoAlpha & mask;
    });
    part.priority_array().and_then(|arr| evaluate_int(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEPRIO]))).map(|(val, mask)| {
        part_state.set_priority(val as f32);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePrio & mask;
    });
    part.flip_h_array().and_then(|arr| evaluate_bool(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEFLIPH]))).map(|val| {
        part_state.set_flip_h(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipH;
    });
    part.flip_v_array().and_then(|arr| evaluate_bool(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEFLIPV]))).map(|val| {
        part_state.set_flip_v(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipV;
    });
    part.hide_array().and_then(|arr| evaluate_bool(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEHIDE]))).map(|val| {
        part_state.set_hide(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeHide;
    });

    part.part_color_array().and_then(|arr| evaluate_partcolor(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEPARTCOLOR]))).map(|(val, mask)| {
        parts_color.push(val);
        part_state.set_part_color((parts_color.len() - 1) as i16);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePartColor & mask;
    });
    part.shader_array().and_then(|arr| evaluate_shader(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTESHADER]))).map(|(val, mask)| {
        shaders.push(val);
        part_state.set_shader((shaders.len() - 1) as i16);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeShader & mask;
    });
    part.vertex_array().and_then(|arr| evaluate_vertex(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEVERTEX]))).map(|(val, mask)| {
        vertices.push(val);
        part_state.set_vertex((vertices.len() - 1) as i16);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeVertex & mask;
    });

    part.pivot_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEPIVOTX]))).map(|(val, mask)| {
        part_state.set_pivot_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotX & mask;
    });
    part.pivot_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEPIVOTY]))).map(|(val, mask)| {
        part_state.set_pivot_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotY & mask;
    });
    part.anchor_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEANCHORX]))).map(|(val, mask)| {
        part_state.set_anchor_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAnchorX & mask;
    });
    part.anchor_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEANCHORY]))).map(|(val, mask)| {
        part_state.set_anchor_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAnchorY & mask;
    });
    part.size_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTESIZEX]))).map(|(val, mask)| {
        part_state.set_size_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeX & mask;
    });
    part.size_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTESIZEY]))).map(|(val, mask)| {
        part_state.set_size_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeY & mask;
    });
    part.img_flip_h_array().and_then(|arr| evaluate_bool(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEIMGFLIPH]))).map(|val| {
        part_state.set_img_flip_h(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeImgFlipH;
    });
    part.img_flip_v_array().and_then(|arr| evaluate_bool(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEIMGFLIPV]))).map(|val| {
        part_state.set_img_flip_v(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeImgFlipV;
    });
    part.uv_translation_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEUVTX]))).map(|(val, mask)| {
        part_state.set_uv_translation_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvtX & mask;
    });
    part.uv_translation_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEUVTY]))).map(|(val, mask)| {
        part_state.set_uv_translation_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvtY & mask;
    });
    part.uv_rotation_z_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEUVRZ]))).map(|(val, mask)| {
        part_state.set_uv_rotation_z(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvrZ & mask;
    });
    part.uv_scale_x_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEUVSX]))).map(|(val, mask)| {
        part_state.set_uv_scale_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvsX & mask;
    });
    part.uv_scale_y_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEUVSY]))).map(|(val, mask)| {
        part_state.set_uv_scale_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvsY & mask;
    });
    part.bounding_radius_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEBOUNDR]))).map(|(val, mask)| {
        part_state.set_bounding_radius(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeBoundr & mask;
    });
    part.mask_array().and_then(|arr| evaluate_float(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEMASK]))).map(|(val, mask)| {
        part_state.set_mask(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeMask & mask;
    });

    part.skew_array().and_then(|arr| evaluate_skew(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTESKEW]))).map(|(val_x, val_y, mask)| {
        part_state.set_skew_x(val_x);
        part_state.set_skew_y(val_y);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSkew & mask;
    });
    part.deform_array().and_then(|arr| evaluate_deform(fb, arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTEDEFORM]))).map(|(val, mask)| {
        deforms.push(val);
        part_state.set_deform((deforms.len() - 1) as i16);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeDeform & mask;
    });
    part.texture_array().and_then(|arr| evaluate_texture(arr, frame_no, Some(&mut part_cursors[IDX_ATTRIBUTETEXTURE]))).map(|val| {
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
        let part_cursors = &mut cursors[i * NUM_PART_ATTRIBUTES .. (i + 1) * NUM_PART_ATTRIBUTES];
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
