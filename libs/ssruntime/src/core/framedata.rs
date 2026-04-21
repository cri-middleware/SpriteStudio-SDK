use crate::core::interpolation::*;
use crate::runtime;
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
pub const IDX_ATTRIBUTEDEFORM: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeDeform.bits().trailing_zeros() as usize;
pub const IDX_ATTRIBUTETEXTURE: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeTexture.bits().trailing_zeros() as usize;

#[derive(Clone)]
pub struct CachedPartData<'a> {
    pub part_index: i16,
    pub parent_index: i16,
    pub cell_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataCellValue<'a>>>>,
    pub position_x_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub position_y_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub position_z_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub rotation_x_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub rotation_y_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub rotation_z_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub scale_x_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub scale_y_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub local_scale_x_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub local_scale_y_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub alpha_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub local_alpha_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub priority_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataIntValue<'a>>>>,
    pub flip_h_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataBoolValue<'a>>>>,
    pub flip_v_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataBoolValue<'a>>>>,
    pub hide_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataBoolValue<'a>>>>,
    pub part_color_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataPartColor<'a>>>>,
    pub shader_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataShader<'a>>>>,
    pub vertex_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataVertex<'a>>>>,
    pub pivot_x_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub pivot_y_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub anchor_x_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub anchor_y_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub size_x_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub size_y_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub img_flip_h_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataBoolValue<'a>>>>,
    pub img_flip_v_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataBoolValue<'a>>>>,
    pub uv_translation_x_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub uv_translation_y_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub uv_rotation_z_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub uv_scale_x_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub uv_scale_y_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub bounding_radius_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub mask_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'a>>>>,
    pub skew_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataVec2<'a>>>>,
    pub deform_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataDeform<'a>>>>,
    pub texture_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataUIntValue<'a>>>>,
}

impl<'a> CachedPartData<'a> {
    pub fn new(part: ssab::ssab::ss::format::PartAnimationData<'a>, part_index: i16, parent_index: i16) -> Self {
        Self {
            part_index,
            parent_index,
            cell_array: part.cell_array(),
            position_x_array: part.position_x_array(),
            position_y_array: part.position_y_array(),
            position_z_array: part.position_z_array(),
            rotation_x_array: part.rotation_x_array(),
            rotation_y_array: part.rotation_y_array(),
            rotation_z_array: part.rotation_z_array(),
            scale_x_array: part.scale_x_array(),
            scale_y_array: part.scale_y_array(),
            local_scale_x_array: part.local_scale_x_array(),
            local_scale_y_array: part.local_scale_y_array(),
            alpha_array: part.alpha_array(),
            local_alpha_array: part.local_alpha_array(),
            priority_array: part.priority_array(),
            flip_h_array: part.flip_h_array(),
            flip_v_array: part.flip_v_array(),
            hide_array: part.hide_array(),
            part_color_array: part.part_color_array(),
            shader_array: part.shader_array(),
            vertex_array: part.vertex_array(),
            pivot_x_array: part.pivot_x_array(),
            pivot_y_array: part.pivot_y_array(),
            anchor_x_array: part.anchor_x_array(),
            anchor_y_array: part.anchor_y_array(),
            size_x_array: part.size_x_array(),
            size_y_array: part.size_y_array(),
            img_flip_h_array: part.img_flip_h_array(),
            img_flip_v_array: part.img_flip_v_array(),
            uv_translation_x_array: part.uv_translation_x_array(),
            uv_translation_y_array: part.uv_translation_y_array(),
            uv_rotation_z_array: part.uv_rotation_z_array(),
            uv_scale_x_array: part.uv_scale_x_array(),
            uv_scale_y_array: part.uv_scale_y_array(),
            bounding_radius_array: part.bounding_radius_array(),
            mask_array: part.mask_array(),
            skew_array: part.skew_array(),
            deform_array: part.deform_array(),
            texture_array: part.texture_array(),
        }
    }
}

pub struct PartEvaluationResult {
    pub state: runtime::framedata::ss::runtime::PartState,
    pub cell: Option<crate::runtime::framedata::ss::runtime::PartAttributeCell>,
    pub part_color: Option<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
    pub shader: Option<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
    pub vertex: Option<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
    pub deform: Option<DeformEvaluationData>,
}

pub(crate) fn evaluate_part(cached_part: &CachedPartData, frame_no: i32, part_cursors: &mut [usize]) -> Option<PartEvaluationResult> {
    let mut part_state = runtime::framedata::ss::runtime::PartState::default();
    
    // Set SpriteStudio default values
    part_state.set_alpha(1.0);
    part_state.set_local_alpha(1.0);
    part_state.set_scale_x(1.0);
    part_state.set_scale_y(1.0);
    part_state.set_local_scale_x(1.0);
    part_state.set_local_scale_y(1.0);

    let mut update_flag = runtime::framedata::ss::runtime::UpdateAttributeFlags::empty();

    let mut result = PartEvaluationResult { state: runtime::framedata::ss::runtime::PartState::default(), cell: None, part_color: None, shader: None, vertex: None, deform: None };

    part_state.set_part_index(cached_part.part_index);

    // Note: Calling `clone()` on `flatbuffers::Vector` here is extremely cheap and does NOT cause
    // memory fragmentation or heap allocation. `flatbuffers::Vector` is just a lightweight view
    // (a pointer and a length) into the underlying byte buffer. This shallow copy is necessary
    // to satisfy Rust's ownership rules when consuming the `Option` via `and_then` from a reference.
    if let Some(value) = cached_part.cell_array.and_then(|arr| evaluate_cell(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTECELL])) {
        result.cell = Some(runtime::framedata::ss::runtime::PartAttributeCell::new(value.map_id(), value.name_hash()));
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeCell;
    }
    if let Some((val, mask)) = cached_part.position_x_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEPOSX])) {
        part_state.set_position_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX & mask;
    }
    if let Some((val, mask)) = cached_part.position_y_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEPOSY])) {
        part_state.set_position_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosY & mask;
    }
    if let Some((val, mask)) = cached_part.position_z_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEPOSZ])) {
        part_state.set_position_z(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosZ & mask;
    }
    if let Some((val, mask)) = cached_part.rotation_x_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEROTX])) {
        part_state.set_rotation_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotX & mask;
    }
    if let Some((val, mask)) = cached_part.rotation_y_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEROTY])) {
        part_state.set_rotation_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotY & mask;
    }
    if let Some((val, mask)) = cached_part.rotation_z_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEROTZ])) {
        part_state.set_rotation_z(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotZ & mask;
    }
    if let Some((val, mask)) = cached_part.scale_x_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTESCLX])) {
        part_state.set_scale_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSclX & mask;
    }
    if let Some((val, mask)) = cached_part.scale_y_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTESCLY])) {
        part_state.set_scale_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSclY & mask;
    }
    if let Some((val, mask)) = cached_part.local_scale_x_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTELOSCLX])) {
        part_state.set_local_scale_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLosclX & mask;
    }
    if let Some((val, mask)) = cached_part.local_scale_y_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTELOSCLY])) {
        part_state.set_local_scale_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLosclY & mask;
    }
    if let Some((val, mask)) = cached_part.alpha_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEALPHA])) {
        part_state.set_alpha(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAlpha & mask;
    }
    if let Some((val, mask)) = cached_part.local_alpha_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTELOALPHA])) {
        part_state.set_local_alpha(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLoAlpha & mask;
    }
    if let Some((val, mask)) = cached_part.priority_array.and_then(|arr| evaluate_int(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEPRIO])) {
        part_state.set_priority(val as f32);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePrio & mask;
    }
    if let Some(val) = cached_part.flip_h_array.and_then(|arr| evaluate_bool(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEFLIPH])) {
        part_state.set_flip_h(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipH;
    }
    if let Some(val) = cached_part.flip_v_array.and_then(|arr| evaluate_bool(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEFLIPV])) {
        part_state.set_flip_v(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipV;
    }
    if let Some(val) = cached_part.hide_array.and_then(|arr| evaluate_bool(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEHIDE])) {
        part_state.set_hide(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeHide;
    }

    if let Some((val, mask)) = cached_part.part_color_array.and_then(|arr| evaluate_partcolor(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEPARTCOLOR])) {
        result.part_color = Some(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePartColor & mask;
    }
    if let Some((val, mask)) = cached_part.shader_array.and_then(|arr| evaluate_shader(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTESHADER])) {
        result.shader = Some(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeShader & mask;
    }
    if let Some((val, mask)) = cached_part.vertex_array.and_then(|arr| evaluate_vertex(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEVERTEX])) {
        result.vertex = Some(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeVertex & mask;
    }

    if let Some((val, mask)) = cached_part.pivot_x_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEPIVOTX])) {
        part_state.set_pivot_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotX & mask;
    }
    if let Some((val, mask)) = cached_part.pivot_y_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEPIVOTY])) {
        part_state.set_pivot_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotY & mask;
    }
    if let Some((val, mask)) = cached_part.anchor_x_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEANCHORX])) {
        part_state.set_anchor_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAnchorX & mask;
    }
    if let Some((val, mask)) = cached_part.anchor_y_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEANCHORY])) {
        part_state.set_anchor_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAnchorY & mask;
    }
    if let Some((val, mask)) = cached_part.size_x_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTESIZEX])) {
        part_state.set_size_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeX & mask;
    }
    if let Some((val, mask)) = cached_part.size_y_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTESIZEY])) {
        part_state.set_size_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeY & mask;
    }
    if let Some(val) = cached_part.img_flip_h_array.and_then(|arr| evaluate_bool(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEIMGFLIPH])) {
        part_state.set_img_flip_h(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeImgFlipH;
    }
    if let Some(val) = cached_part.img_flip_v_array.and_then(|arr| evaluate_bool(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEIMGFLIPV])) {
        part_state.set_img_flip_v(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeImgFlipV;
    }
    if let Some((val, mask)) = cached_part.uv_translation_x_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEUVTX])) {
        part_state.set_uv_translation_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvtX & mask;
    }
    if let Some((val, mask)) = cached_part.uv_translation_y_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEUVTY])) {
        part_state.set_uv_translation_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvtY & mask;
    }
    if let Some((val, mask)) = cached_part.uv_rotation_z_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEUVRZ])) {
        part_state.set_uv_rotation_z(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvrZ & mask;
    }
    if let Some((val, mask)) = cached_part.uv_scale_x_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEUVSX])) {
        part_state.set_uv_scale_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvsX & mask;
    }
    if let Some((val, mask)) = cached_part.uv_scale_y_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEUVSY])) {
        part_state.set_uv_scale_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeUvsY & mask;
    }
    if let Some((val, mask)) = cached_part.bounding_radius_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEBOUNDR])) {
        part_state.set_bounding_radius(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeBoundr & mask;
    }
    if let Some((val, mask)) = cached_part.mask_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEMASK])) {
        part_state.set_mask(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeMask & mask;
    }

    if let Some((val_x, val_y, mask)) = cached_part.skew_array.and_then(|arr| evaluate_skew(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTESKEW])) {
        part_state.set_skew_x(val_x);
        part_state.set_skew_y(val_y);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSkew & mask;
    }
    if let Some((val, mask)) = cached_part.deform_array.and_then(|arr| evaluate_deform(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEDEFORM])) {
        result.deform = Some(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeDeform & mask;
    }
    if let Some(val) = cached_part.texture_array.and_then(|arr| evaluate_texture(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTETEXTURE])) {
        part_state.set_texture(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeTexture;
    }

    if update_flag.is_empty() {
        return None;
    }
    part_state.set_update_flag(update_flag);
    result.state = part_state;
    Some(result)
}

#[allow(clippy::too_many_arguments)]
fn get_parts<'a>(
    fb: &mut flatbuffers::FlatBufferBuilder<'a>,
    cached_parts: &[CachedPartData<'a>],
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
    cursors: &mut [usize],
) {
    parts_vec.clear();
    cell_vec.clear();
    parts_color_vec.clear();
    shader_vec.clear();
    vertice_vec.clear();
    deform_vec.clear();

    for (i, part) in cached_parts.iter().enumerate() {
        let part_cursors = &mut cursors[i * NUM_PART_ATTRIBUTES..(i + 1) * NUM_PART_ATTRIBUTES];
        if let Some(mut result) = evaluate_part(part, frame_no, part_cursors) {
            if let Some(cell) = result.cell {
                cell_vec.push(cell);
                result.state.set_cell((cell_vec.len() - 1) as i16);
            }
            if let Some(part_color) = result.part_color {
                parts_color_vec.push(part_color);
                result.state.set_part_color((parts_color_vec.len() - 1) as i16);
            }
            if let Some(shader) = result.shader {
                shader_vec.push(shader);
                result.state.set_shader((shader_vec.len() - 1) as i16);
            }
            if let Some(vertex) = result.vertex {
                vertice_vec.push(vertex);
                result.state.set_vertex((vertice_vec.len() - 1) as i16);
            }
            if let Some(deform_data) = result.deform {
                let mut args = runtime::framedata::ss::runtime::PartAttributeDeformArgs::default();
                if !deform_data.coordinate.is_empty() {
                    args.coordinate = Some(fb.create_vector(&deform_data.coordinate));
                }
                if !deform_data.index.is_empty() {
                    args.index = Some(fb.create_vector(&deform_data.index));
                }
                let deform_off = runtime::framedata::ss::runtime::PartAttributeDeform::create(fb, &args);
                deform_vec.push(deform_off);
                result.state.set_deform((deform_vec.len() - 1) as i16);
            }
            parts_vec.push(result.state);
        }
    }

    if !cell_vec.is_empty() {
        *cells_off = Some(fb.create_vector(cell_vec));
    } else {
        *cells_off = None;
    }

    if !parts_color_vec.is_empty() {
        *parts_color_off = Some(fb.create_vector(parts_color_vec));
    } else {
        *parts_color_off = None;
    }

    if !shader_vec.is_empty() {
        *shaders_off = Some(fb.create_vector(shader_vec));
    } else {
        *shaders_off = None;
    }

    if !vertice_vec.is_empty() {
        *vertices_off = Some(fb.create_vector(vertice_vec));
    } else {
        *vertices_off = None;
    }

    if !deform_vec.is_empty() {
        *deforms_off = Some(fb.create_vector(deform_vec));
    } else {
        *deforms_off = None;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn get_frame_data<'a>(
    fb: &mut flatbuffers::FlatBufferBuilder<'a>,
    cached_parts: &[CachedPartData<'a>],
    frame_no: i32,
    parts_vec: &mut Vec<runtime::framedata::ss::runtime::PartState>,
    cell_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeCell>,
    parts_color_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
    shader_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
    vertice_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
    deform_vec: &mut Vec<flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>,
    cursors: &mut [usize],
) -> flatbuffers::WIPOffset<runtime::framedata::ss::runtime::FrameData<'a>> {
    let mut args = runtime::framedata::ss::runtime::FrameDataArgs::default();
    let mut cells_off: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeCell>>> = None;
    let mut parts_color_off: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributePartColor>>> = None;
    let mut shaders_off: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeShader>>> = None;
    let mut vertices_off: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeVertex>>> = None;
    let mut deforms_off: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>>> = None;

    get_parts(fb, cached_parts, frame_no, parts_vec, cell_vec, parts_color_vec, shader_vec, vertice_vec, deform_vec, &mut cells_off, &mut parts_color_off, &mut shaders_off, &mut vertices_off, &mut deforms_off, cursors);
    if !parts_vec.is_empty() {
        args.parts = Some(fb.create_vector(parts_vec));
    }

    args.cells = cells_off;
    args.parts_color = parts_color_off;
    args.shaders = shaders_off;
    args.vertices = vertices_off;
    args.deforms = deforms_off;
    runtime::framedata::ss::runtime::FrameData::create(fb, &args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::framedata::ss::runtime::UpdateAttributeFlags;

    #[test]
    fn test_attribute_indices_consistency() {
        // Verify that each IDX_ATTRIBUTE constant matches the trailing zeros of its flag.
        // This ensures that our indexing into the cursors array is correctly aligned with bit positions.
        assert_eq!(IDX_ATTRIBUTECELL, UpdateAttributeFlags::AttributeCell.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEPOSX, UpdateAttributeFlags::AttributePosX.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEPOSY, UpdateAttributeFlags::AttributePosY.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEPOSZ, UpdateAttributeFlags::AttributePosZ.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEROTX, UpdateAttributeFlags::AttributeRotX.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEROTY, UpdateAttributeFlags::AttributeRotY.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEROTZ, UpdateAttributeFlags::AttributeRotZ.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTESCLX, UpdateAttributeFlags::AttributeSclX.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTESCLY, UpdateAttributeFlags::AttributeSclY.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTELOSCLX, UpdateAttributeFlags::AttributeLosclX.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTELOSCLY, UpdateAttributeFlags::AttributeLosclY.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEALPHA, UpdateAttributeFlags::AttributeAlpha.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTELOALPHA, UpdateAttributeFlags::AttributeLoAlpha.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEPRIO, UpdateAttributeFlags::AttributePrio.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEFLIPH, UpdateAttributeFlags::AttributeFlipH.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEFLIPV, UpdateAttributeFlags::AttributeFlipV.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEHIDE, UpdateAttributeFlags::AttributeHide.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEPARTCOLOR, UpdateAttributeFlags::AttributePartColor.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTESHADER, UpdateAttributeFlags::AttributeShader.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEVERTEX, UpdateAttributeFlags::AttributeVertex.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEPIVOTX, UpdateAttributeFlags::AttributePivotX.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEPIVOTY, UpdateAttributeFlags::AttributePivotY.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEANCHORX, UpdateAttributeFlags::AttributeAnchorX.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEANCHORY, UpdateAttributeFlags::AttributeAnchorY.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTESIZEX, UpdateAttributeFlags::AttributeSizeX.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTESIZEY, UpdateAttributeFlags::AttributeSizeY.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEIMGFLIPH, UpdateAttributeFlags::AttributeImgFlipH.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEIMGFLIPV, UpdateAttributeFlags::AttributeImgFlipV.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEUVTX, UpdateAttributeFlags::AttributeUvtX.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEUVTY, UpdateAttributeFlags::AttributeUvtY.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEUVRZ, UpdateAttributeFlags::AttributeUvrZ.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEUVSX, UpdateAttributeFlags::AttributeUvsX.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEUVSY, UpdateAttributeFlags::AttributeUvsY.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEBOUNDR, UpdateAttributeFlags::AttributeBoundr.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEMASK, UpdateAttributeFlags::AttributeMask.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTESKEW, UpdateAttributeFlags::AttributeSkew.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTEDEFORM, UpdateAttributeFlags::AttributeDeform.bits().trailing_zeros() as usize);
        assert_eq!(IDX_ATTRIBUTETEXTURE, UpdateAttributeFlags::AttributeTexture.bits().trailing_zeros() as usize);
    }

    #[test]
    fn test_num_part_attributes_count() {
        // NUM_PART_ATTRIBUTES should exactly match the number of set bits in UpdateAttributeFlags::all().
        assert_eq!(NUM_PART_ATTRIBUTES, UpdateAttributeFlags::all().bits().count_ones() as usize);
    }

    #[test]
    fn test_attribute_flags_no_gaps() {
        let all_bits = UpdateAttributeFlags::all().bits();
        let num_bits = all_bits.count_ones();
        // Check for gaps: If there are no gaps, the value of all_bits should be (1 << num_bits) - 1.
        // This is critical because we use bit trailing_zeros as direct indices into arrays of size NUM_PART_ATTRIBUTES.
        let expected_mask = (1u64 << num_bits) - 1;
        assert_eq!(all_bits, expected_mask, "UpdateAttributeFlags must not have gaps in bit values to avoid array out-of-bounds when using bit positions as indices.");
    }

    #[test]
    fn test_part_evaluation_result_init() {
        let mut part_state = runtime::framedata::ss::runtime::PartState::default();
        part_state.set_part_index(5);

        let result = PartEvaluationResult { state: part_state, cell: None, part_color: None, shader: None, vertex: None, deform: None };

        assert_eq!(result.state.part_index(), 5);
        assert!(result.cell.is_none());
        assert!(result.part_color.is_none());
    }
}
