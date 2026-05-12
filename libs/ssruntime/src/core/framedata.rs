use crate::core::interpolation::*;
use crate::core::{SsCoordinateSystem, SsOrigin, SsUnit};
use crate::runtime;
use alloc::vec::Vec;

pub const NUM_PART_ATTRIBUTES: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::all().bits().count_ones() as usize;

pub const IDX_ATTRIBUTECELL: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeCell.bits().trailing_zeros() as usize;
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

pub const SHAPE_VERTICES_STRIDE: usize = 24;

/// Stride (number of `f32`s) per part in the `local_vertices` SoA buffer.
/// Layout: `[lt.x, lt.y, rt.x, rt.y, lb.x, lb.y, rb.x, rb.y, center.x, center.y]`
/// in part-local space, with `coordinate_system` (Y-Up/Y-Down) flip already applied.
pub const LOCAL_VERTICES_STRIDE: usize = 10;

#[derive(Clone, Copy, Default)]
pub struct ResolvedCellMeta {
    pub pivot_x: f32,
    pub pivot_y: f32,
    pub size_w: f32,
    pub size_h: f32,

    pub rect_left: f32,
    pub rect_top: f32,
    pub tex_w: f32,
    pub tex_h: f32,
    pub rotated: bool,

    pub texture_hash: u32,
}

/// Walks a part's `cell_array` keyframes once and resolves each
/// `(cellmap_name_hash, name_hash)` against the SSAB cellmaps to a
/// `ResolvedCellMeta`. The returned vector is index-aligned with `cell_array`
/// so the per-frame path can do an O(1) parallel-index lookup.
///
/// Both cellmaps and cells are stored in `name_hash`-sorted order (FlatBuffers
/// `key`); we exploit that with binary search. Keyframes whose
/// `(cellmap_name_hash, name_hash)` cannot be resolved fall back to a
/// zero-initialized entry.
pub fn resolve_cell_array_meta<'a>(ssab: &ssab::ssab::ss::format::SsAnimeBinary<'a>, cell_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataCellValue<'a>>>>) -> alloc::vec::Vec<ResolvedCellMeta> {
    let Some(cell_array) = cell_array else { return alloc::vec::Vec::new() };
    let Some(cellmaps) = ssab.cellmaps() else { return alloc::vec::Vec::new() };

    let mut out = alloc::vec::Vec::with_capacity(cell_array.len());
    for key in cell_array {
        let mut meta = ResolvedCellMeta::default();
        if let Some(value) = key.value() {
            let cellmap_name_hash = value.cellmap_name_hash();
            let name_hash = value.name_hash();
            if let Some(cellmap) = crate::core::ssab_lookup::find_cellmap(&cellmaps, cellmap_name_hash)
                && let Some(cells) = cellmap.cells()
                && let Some(cell) = crate::core::ssab_lookup::find_cell(&cells, name_hash)
            {
                let (pivot_x, pivot_y) = cell.pivot().map(|p| (p.v1(), p.v2())).unwrap_or((0.0, 0.0));
                let (rect_left, rect_top, size_w, size_h) = cell.rectangle().map(|r| (r.x1(), r.y1(), r.x2(), r.y2())).unwrap_or((0.0, 0.0, 0.0, 0.0));
                // Mirror compute_local_uvs's previous fallback: if the
                // cellmap doesn't carry size_original (or it's zero),
                // treat the cell rect dimensions as the texture size.
                let (tex_w, tex_h) = cellmap.size_original().map(|v| (v.v1(), v.v2())).filter(|(w, h)| *w > 0.0 && *h > 0.0).unwrap_or((size_w, size_h));
                meta = ResolvedCellMeta { pivot_x, pivot_y, size_w, size_h, rect_left, rect_top, tex_w, tex_h, rotated: cell.rotated(), texture_hash: cellmap.name_hash() };
            }
        }
        out.push(meta);
    }
    out
}

#[derive(Clone)]
pub struct CachedPartData<'a> {
    pub part_index: i16,
    pub parent_index: i16,
    pub resolved_cells: alloc::vec::Vec<ResolvedCellMeta>,
    pub shape_kind: Option<crate::core::geometry::ShapeType>,

    // バッチキー用の静的メタデータ (setup 時に SSAB PartData から1回だけ読む)。
    // フレームごとの DrawBatch 計算で参照される。
    pub blend_type: ssab::ssab::ss::format::BlendType,
    pub part_type_disc: ssab::ssab::ss::format::PartType,
    pub mask_influence: bool,
    pub is_mask_contributor: bool,
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
    pub fn new(
        part: ssab::ssab::ss::format::PartAnimationData<'a>,
        part_index: i16,
        parent_index: i16,
        resolved_cells: alloc::vec::Vec<ResolvedCellMeta>,
        shape_kind: Option<crate::core::geometry::ShapeType>,
        blend_type: ssab::ssab::ss::format::BlendType,
        part_type_disc: ssab::ssab::ss::format::PartType,
        mask_influence: bool,
        is_mask_contributor: bool,
    ) -> Self {
        Self {
            part_index,
            parent_index,
            resolved_cells,
            shape_kind,
            blend_type,
            part_type_disc,
            mask_influence,
            is_mask_contributor,
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
    pub cell_meta: Option<ResolvedCellMeta>,
    pub part_color: Option<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
    pub shader: Option<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
    pub vertex: Option<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
    pub deform: Option<DeformEvaluationData>,
}

pub(crate) fn evaluate_part(
    cached_part: &CachedPartData,
    frame_no: f32,
    part_cursors: &mut [usize],
    coordinate_system: SsCoordinateSystem,
    origin: SsOrigin,
    unit: SsUnit,
    canvas_width: f32,
    canvas_height: f32,
    start_workspace: &mut Vec<f32>,
    end_workspace: &mut Vec<f32>,
    out_workspace: &mut Vec<f32>,
) -> PartEvaluationResult {
    let mut part_state = runtime::framedata::ss::runtime::PartState::default();

    // default values
    part_state.set_alpha(1.0);
    part_state.set_local_alpha(1.0);
    part_state.set_scale_x(1.0);
    part_state.set_scale_y(1.0);
    part_state.set_local_scale_x(1.0);
    part_state.set_local_scale_y(1.0);
    part_state.set_size_x(1.0);
    part_state.set_size_y(1.0);
    part_state.set_uv_scale_x(1.0);
    part_state.set_uv_scale_y(1.0);
    part_state.set_cell(-1);
    part_state.set_part_color(-1);
    part_state.set_shader(-1);
    part_state.set_vertex(-1);
    part_state.set_deform(-1);

    let mut update_flag = runtime::framedata::ss::runtime::UpdateAttributeFlags::empty();

    let mut result = PartEvaluationResult { state: runtime::framedata::ss::runtime::PartState::default(), cell: None, cell_meta: None, part_color: None, shader: None, vertex: None, deform: None };

    part_state.set_part_index(cached_part.part_index);

    if let Some(value) = cached_part.cell_array.and_then(|arr| evaluate_cell(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTECELL])) {
        result.cell = Some(runtime::framedata::ss::runtime::PartAttributeCell::new(value.cellmap_name_hash(), value.name_hash()));
        result.cell_meta = cached_part.resolved_cells.get(part_cursors[IDX_ATTRIBUTECELL]).copied();
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeCell;
    }

    let mut pos_x = 0.0;
    if let Some((val, mask)) = cached_part.position_x_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEPOSX])) {
        pos_x = val;
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX & mask;
    }
    let mut pos_y = 0.0;
    if let Some((mut val, mask)) = cached_part.position_y_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEPOSY])) {
        if coordinate_system == SsCoordinateSystem::YDown {
            val = -val;
        }
        pos_y = val;
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosY & mask;
    }

    // Apply origin offset for root part
    if origin == SsOrigin::TopLeft && cached_part.parent_index == -1 {
        pos_x += canvas_width / 2.0;
        if coordinate_system == SsCoordinateSystem::YDown {
            pos_y += canvas_height / 2.0;
        } else {
            pos_y -= canvas_height / 2.0;
        }
    }

    // Apply unit normalization
    if unit == SsUnit::Normalized && canvas_width != 0.0 && canvas_height != 0.0 {
        pos_x /= canvas_width;
        pos_y /= canvas_height;
    }

    part_state.set_position_x(pos_x);
    part_state.set_position_y(pos_y);

    if let Some((val, mask)) = cached_part.position_z_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEPOSZ])) {
        part_state.set_position_z(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosZ & mask;
    }
    if let Some((mut val, mask)) = cached_part.rotation_x_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEROTX])) {
        if coordinate_system == SsCoordinateSystem::YDown {
            val = -val;
        }
        part_state.set_rotation_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotX & mask;
    }
    if let Some((val, mask)) = cached_part.rotation_y_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEROTY])) {
        part_state.set_rotation_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotY & mask;
    }
    if let Some((mut val, mask)) = cached_part.rotation_z_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEROTZ])) {
        if coordinate_system == SsCoordinateSystem::YDown {
            val = -val;
        }
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
        // Raw deform Y is kept in the source (Y-Up) orientation here; the always-on
        // local-vertex calc consumes it as deform offsets and the result is flipped
        // to YDown only at the very end. The intermediate raw deform value is never
        // exposed publicly, so flipping/unflipping it would be wasted work.
        result.vertex = Some(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeVertex & mask;
    }

    if let Some((val, mask)) = cached_part.pivot_x_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEPIVOTX])) {
        part_state.set_pivot_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotX & mask;
    }
    if let Some((mut val, mask)) = cached_part.pivot_y_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEPIVOTY])) {
        if coordinate_system == SsCoordinateSystem::YDown {
            val = -val;
        }
        part_state.set_pivot_y(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePivotY & mask;
    }
    if let Some((val, mask)) = cached_part.anchor_x_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEANCHORX])) {
        part_state.set_anchor_x(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeAnchorX & mask;
    }
    if let Some((mut val, mask)) = cached_part.anchor_y_array.and_then(|arr| evaluate_float(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEANCHORY])) {
        if coordinate_system == SsCoordinateSystem::YDown {
            val = -val;
        }
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
    if let Some((mut val, mask)) = cached_part.deform_array.and_then(|arr| {
        evaluate_deform(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEDEFORM], start_workspace, end_workspace, out_workspace)
    }) {
        if coordinate_system == SsCoordinateSystem::YDown {
            for v in val.coordinate.iter_mut() {
                v.set_y(-v.y());
            }
        }
        result.deform = Some(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeDeform & mask;
    }
    if let Some(val) = cached_part.texture_array.and_then(|arr| evaluate_texture(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTETEXTURE])) {
        part_state.set_texture(val);
        update_flag |= runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeTexture;
    }

    part_state.set_update_flag(update_flag);
    result.state = part_state;
    result
}

#[allow(clippy::too_many_arguments)]
fn get_parts<'a>(
    fb: &mut flatbuffers::FlatBufferBuilder<'a>,
    cached_parts: &[CachedPartData<'a>],
    frame_no: f32,
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
    cell_meta_per_part: &mut [f32],
    cell_texture_hashes: &mut [u32],
    local_vertices: &mut [f32],
    coordinate_system: SsCoordinateSystem,
    origin: SsOrigin,
    unit: SsUnit,
    canvas_width: f32,
    canvas_height: f32,
    start_workspace: &mut Vec<f32>,
    end_workspace: &mut Vec<f32>,
    out_workspace: &mut Vec<f32>,
    inheritance_alphas_workspace: &mut Vec<f32>,
) {
    parts_vec.clear();
    cell_vec.clear();
    parts_color_vec.clear();
    shader_vec.clear();
    vertice_vec.clear();
    deform_vec.clear();
    for slot in cell_meta_per_part.iter_mut() {
        *slot = 0.0;
    }
    for slot in cell_texture_hashes.iter_mut() {
        *slot = 0;
    }
    for slot in local_vertices.iter_mut() {
        *slot = 0.0;
    }

    inheritance_alphas_workspace.clear();
    inheritance_alphas_workspace.reserve(cached_parts.len());

    for (i, part) in cached_parts.iter().enumerate() {
        let part_cursors = &mut cursors[i * NUM_PART_ATTRIBUTES..(i + 1) * NUM_PART_ATTRIBUTES];
        let mut result = evaluate_part(
            part,
            frame_no,
            part_cursors,
            coordinate_system,
            origin,
            unit,
            canvas_width,
            canvas_height,
            start_workspace,
            end_workspace,
            out_workspace,
        );

        let cell_meta = result.cell_meta.unwrap_or_default();
        let (cell_pivot_x, cell_pivot_y, cell_size_w, cell_size_h) = (cell_meta.pivot_x, cell_meta.pivot_y, cell_meta.size_w, cell_meta.size_h);

        let off = i * 6;
        if off + 6 <= cell_meta_per_part.len() {
            cell_meta_per_part[off] = cell_pivot_x;
            cell_meta_per_part[off + 1] = cell_pivot_y;
            cell_meta_per_part[off + 2] = cell_size_w;
            cell_meta_per_part[off + 3] = cell_size_h;
            cell_meta_per_part[off + 4] = cell_meta.rect_left;
            cell_meta_per_part[off + 5] = cell_meta.rect_top;
        }
        if i < cell_texture_hashes.len() {
            cell_texture_hashes[i] = cell_meta.texture_hash;
        }

        let flag = result.state.update_flag();
        if !flag.contains(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeX) {
            result.state.set_size_x(cell_size_w);
        }
        if !flag.contains(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeSizeY) {
            result.state.set_size_y(cell_size_h);
        }

        let parent_idx = part.parent_index;
        let valid_parent_idx = (parent_idx >= 0 && (parent_idx as usize) < i).then_some(parent_idx as usize);

        let parent_inh_alpha = valid_parent_idx.and_then(|p_idx| inheritance_alphas_workspace.get(p_idx).copied()).unwrap_or(1.0);
        let parent_hide = valid_parent_idx.and_then(|p_idx| parts_vec.get(p_idx).map(|p| p.hide())).unwrap_or(false);

        let inh_alpha = result.state.alpha() * parent_inh_alpha;
        inheritance_alphas_workspace.push(inh_alpha);

        let has_local_alpha = result.state.update_flag().contains(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLoAlpha);
        let effective_alpha = if has_local_alpha { result.state.local_alpha() } else { inh_alpha };
        result.state.set_alpha(effective_alpha);

        if parent_hide {
            result.state.set_hide(true);
        }

        // Pre-multiply per-vertex parts color alpha with the hierarchical alpha.
        if let Some(pc) = result.part_color {
            let alpha = result.state.alpha();
            let apply_alpha = |c: &crate::runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor| {
                let rgba = c.rgba();
                let new_a = ((rgba.a() as f32 / 255.0) * alpha * 255.0).clamp(0.0, 255.0) as u8;
                crate::runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&crate::runtime::framedata::ss::runtime::U8Rect::new(new_a, rgba.r(), rgba.g(), rgba.b()), c.rate())
            };

            let blend_type = pc.blend_type();
            let lt = apply_alpha(pc.lt());
            let rt = apply_alpha(pc.rt());
            let lb = apply_alpha(pc.lb());
            let rb = apply_alpha(pc.rb());

            result.part_color = Some(crate::runtime::framedata::ss::runtime::PartAttributePartColor::new(blend_type, &lt, &rt, &lb, &rb));
        }

        if result.cell.is_none() {
            // Drop any raw deform we might have evaluated; nothing will read it.
            result.vertex = None;
        } else {
            let pivot = (cell_pivot_x, cell_pivot_y);

            let deform = result.vertex.as_ref().map(|v| {
                let lt = v.lt();
                let rt = v.rt();
                let lb = v.lb();
                let rb = v.rb();
                ([lt.x(), rt.x(), lb.x(), rb.x()], [lt.y(), rt.y(), lb.y(), rb.y()])
            });

            let mut pivot_y = result.state.pivot_y();
            if coordinate_system == SsCoordinateSystem::YDown {
                pivot_y = -pivot_y;
            }

            let (vx, mut vy) =
                crate::core::geometry::compute_local_vertices((result.state.size_x(), result.state.size_y()), pivot, (result.state.pivot_x(), pivot_y), (result.state.img_flip_h(), result.state.img_flip_v()), deform.as_ref().map(|(x, y)| (x, y)));

            if coordinate_system == SsCoordinateSystem::YDown {
                for y in vy.iter_mut() {
                    *y = -*y;
                }
            }

            let lt = crate::runtime::framedata::ss::runtime::Vec2::new(vx[0], vy[0]);
            let rt = crate::runtime::framedata::ss::runtime::Vec2::new(vx[1], vy[1]);
            let lb = crate::runtime::framedata::ss::runtime::Vec2::new(vx[2], vy[2]);
            let rb = crate::runtime::framedata::ss::runtime::Vec2::new(vx[3], vy[3]);
            let needs_center = result.state.update_flag().contains(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeVertex) || result.state.update_flag().contains(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePartColor);
            let center = if needs_center { crate::runtime::framedata::ss::runtime::Vec2::new(vx[4], vy[4]) } else { crate::runtime::framedata::ss::runtime::Vec2::default() };
            result.vertex = Some(crate::runtime::framedata::ss::runtime::PartAttributeVertex::new(&lt, &rt, &lb, &rb, &center));

            let off = i * LOCAL_VERTICES_STRIDE;
            if off + LOCAL_VERTICES_STRIDE <= local_vertices.len() {
                local_vertices[off] = vx[0];
                local_vertices[off + 1] = vy[0];
                local_vertices[off + 2] = vx[1];
                local_vertices[off + 3] = vy[1];
                local_vertices[off + 4] = vx[2];
                local_vertices[off + 5] = vy[2];
                local_vertices[off + 6] = vx[3];
                local_vertices[off + 7] = vy[3];
                local_vertices[off + 8] = vx[4];
                local_vertices[off + 9] = vy[4];
            }
        }

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
fn classify_for_draw_batch(disc: ssab::ssab::ss::format::PartType) -> Option<(runtime::framedata::ss::runtime::DrawBatchKind, bool)> {
    use runtime::framedata::ss::runtime::DrawBatchKind as K;
    use ssab::ssab::ss::format::PartType as P;
    if disc == P::PartTypeNormal {
        return Some((K::Normal, true));
    }
    if disc == P::PartTypeShape {
        return Some((K::Shape, false));
    }
    if disc == P::PartTypeInstance {
        return Some((K::Instance, false));
    }
    if disc == P::PartTypeEffect {
        return Some((K::Effect, false));
    }
    if disc == P::PartTypeMask {
        return Some((K::Mask, false));
    }
    None
}

const FLAG_MASK_INFLUENCE: u8 = 0x01;

#[inline]
fn ssab_blend_to_runtime(b: ssab::ssab::ss::format::BlendType) -> runtime::framedata::ss::runtime::BlendType {
    runtime::framedata::ss::runtime::BlendType(b.0)
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct BatchKey {
    kind: runtime::framedata::ss::runtime::DrawBatchKind,
    texture_hash: u32,
    blend_type: runtime::framedata::ss::runtime::BlendType,
    flags: u8,
}

struct InProgressBatch {
    start_rank: u16,
    count: u16,
    vertex_count: u32,
    index_count: u32,
    key: BatchKey,
}

fn finalize_batch(ip: InProgressBatch, out: &mut Vec<runtime::framedata::ss::runtime::DrawBatch>) {
    out.push(runtime::framedata::ss::runtime::DrawBatch::new(ip.start_rank, ip.count, ip.key.texture_hash, ip.vertex_count.min(u16::MAX as u32) as u16, ip.index_count.min(u16::MAX as u32) as u16, ip.key.blend_type, ip.key.kind, ip.key.flags));
}

type DrawOrderAndBatchesOffsets<'a> = (flatbuffers::WIPOffset<flatbuffers::Vector<'a, u16>>, flatbuffers::WIPOffset<flatbuffers::Vector<'a, runtime::framedata::ss::runtime::DrawBatch>>, flatbuffers::WIPOffset<flatbuffers::Vector<'a, u16>>);

fn build_draw_order_and_batches<'a>(
    fb: &mut flatbuffers::FlatBufferBuilder<'a>,
    z_indices: &[usize],
    cached_parts: &[CachedPartData<'_>],
    parts_vec: &[runtime::framedata::ss::runtime::PartState],
    cell_texture_hashes: &[u32],
    draw_order_workspace: &mut Vec<u16>,
    batches_workspace: &mut Vec<runtime::framedata::ss::runtime::DrawBatch>,
    mask_indices_workspace: &mut Vec<u16>,
) -> DrawOrderAndBatchesOffsets<'a> {
    use runtime::framedata::ss::runtime::{DrawBatchKind, UpdateAttributeFlags};

    draw_order_workspace.clear();
    draw_order_workspace.reserve(z_indices.len());
    batches_workspace.clear();
    mask_indices_workspace.clear();

    let mut in_progress: Option<InProgressBatch> = None;

    for &part_idx in z_indices.iter() {
        if part_idx >= cached_parts.len() || part_idx >= parts_vec.len() {
            continue;
        }
        let cached = &cached_parts[part_idx];
        let part_state = &parts_vec[part_idx];

        let Some((kind, batchable)) = classify_for_draw_batch(cached.part_type_disc) else {
            continue;
        };

        if cached.is_mask_contributor {
            mask_indices_workspace.push(part_idx as u16);
        }

        let blend_type = ssab_blend_to_runtime(cached.blend_type);
        let mut flags: u8 = 0;
        if cached.mask_influence {
            flags |= FLAG_MASK_INFLUENCE;
        }

        let is_pure_mask = kind == DrawBatchKind::Mask;
        let needs_mask_twin = cached.is_mask_contributor && !is_pure_mask;
        let texture_hash = cell_texture_hashes.get(part_idx).copied().unwrap_or(0);
        let normal_no_cell = kind == DrawBatchKind::Normal && texture_hash == 0;
        let drawn_as_self = (!part_state.hide() || is_pure_mask) && !normal_no_cell;

        if !needs_mask_twin && !drawn_as_self {
            continue;
        }

        let rank = draw_order_workspace.len() as u16;
        draw_order_workspace.push(part_idx as u16);

        if needs_mask_twin {
            if let Some(ip) = in_progress.take() {
                finalize_batch(ip, batches_workspace);
            }
            batches_workspace.push(runtime::framedata::ss::runtime::DrawBatch::new(rank, 1, 0, 0, 0, blend_type, DrawBatchKind::Mask, flags));
        }

        if !drawn_as_self {
            continue;
        }

        let (vc, ic): (u32, u32) = if kind == DrawBatchKind::Normal {
            let f = part_state.update_flag();
            let needs_center = f.contains(UpdateAttributeFlags::AttributeVertex) || f.contains(UpdateAttributeFlags::AttributePartColor);
            if needs_center { (5, 12) } else { (4, 6) }
        } else {
            (0, 0)
        };

        let key = BatchKey { kind, texture_hash, blend_type, flags };

        if let Some(ip) = in_progress.as_mut() {
            if batchable && ip.key == key {
                ip.count = ip.count.saturating_add(1);
                ip.vertex_count = ip.vertex_count.saturating_add(vc);
                ip.index_count = ip.index_count.saturating_add(ic);
                continue;
            }
            finalize_batch(in_progress.take().unwrap(), batches_workspace);
        }

        in_progress = Some(InProgressBatch { start_rank: rank, count: 1, vertex_count: vc, index_count: ic, key });
    }

    if let Some(ip) = in_progress {
        finalize_batch(ip, batches_workspace);
    }

    let draw_order_off = fb.create_vector(draw_order_workspace);
    let batches_off = fb.create_vector(batches_workspace);
    let mask_indices_off = fb.create_vector(mask_indices_workspace);
    (draw_order_off, batches_off, mask_indices_off)
}

#[allow(clippy::too_many_arguments)]
pub fn get_frame_data<'a>(
    fb: &mut flatbuffers::FlatBufferBuilder<'a>,
    cached_parts: &[CachedPartData<'a>],
    frame_no: f32,
    z_order: &mut crate::core::z_order::ZOrderTable,
    parts_vec: &mut Vec<runtime::framedata::ss::runtime::PartState>,
    cell_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeCell>,
    parts_color_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
    shader_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
    vertice_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
    deform_vec: &mut Vec<flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>,
    cursors: &mut [usize],
    cell_meta_per_part: &mut [f32],
    cell_texture_hashes: &mut [u32],
    local_vertices: &mut [f32],
    coordinate_system: SsCoordinateSystem,
    origin: SsOrigin,
    unit: SsUnit,
    canvas_width: f32,
    canvas_height: f32,
    draw_order_workspace: &mut Vec<u16>,
    batches_workspace: &mut Vec<runtime::framedata::ss::runtime::DrawBatch>,
    mask_indices_workspace: &mut Vec<u16>,
    deform_start_workspace: &mut Vec<f32>,
    deform_end_workspace: &mut Vec<f32>,
    deform_out_workspace: &mut Vec<f32>,
    inheritance_alphas_workspace: &mut Vec<f32>,
) -> flatbuffers::WIPOffset<runtime::framedata::ss::runtime::FrameData<'a>> {
    let mut args = runtime::framedata::ss::runtime::FrameDataArgs::default();
    let mut cells_off: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeCell>>> = None;
    let mut parts_color_off: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributePartColor>>> = None;
    let mut shaders_off: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeShader>>> = None;
    let mut vertices_off: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeVertex>>> = None;
    let mut deforms_off: Option<::flatbuffers::WIPOffset<::flatbuffers::Vector<'a, ::flatbuffers::ForwardsUOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>>> = None;

    get_parts(
        fb,
        cached_parts,
        frame_no,
        parts_vec,
        cell_vec,
        parts_color_vec,
        shader_vec,
        vertice_vec,
        deform_vec,
        &mut cells_off,
        &mut parts_color_off,
        &mut shaders_off,
        &mut vertices_off,
        &mut deforms_off,
        cursors,
        cell_meta_per_part,
        cell_texture_hashes,
        local_vertices,
        coordinate_system,
        origin,
        unit,
        canvas_width,
        canvas_height,
        deform_start_workspace,
        deform_end_workspace,
        deform_out_workspace,
        inheritance_alphas_workspace,
    );
    args.parts = Some(fb.create_vector(parts_vec));

    args.cells = cells_off;
    args.parts_color = parts_color_off;
    args.shaders = shaders_off;
    args.vertices = vertices_off;
    args.deforms = deforms_off;

    // z_order と batch 計算は parts_vec / cell_texture_hashes が満たされた後で行う。
    // FrameData::create より前に args.draw_order / args.draw_batches を設定する必要があるため
    // ここで実行する (lib.rs から z_order.update を移譲)。
    z_order.update(parts_vec);
    let (draw_order_off, draw_batches_off, mask_indices_off) = build_draw_order_and_batches(
        fb,
        z_order.indices(),
        cached_parts,
        parts_vec,
        cell_texture_hashes,
        draw_order_workspace,
        batches_workspace,
        mask_indices_workspace,
    );
    args.draw_order = Some(draw_order_off);
    args.draw_batches = Some(draw_batches_off);
    args.mask_indices = Some(mask_indices_off);

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

        let result = PartEvaluationResult { state: part_state, cell: None, cell_meta: None, part_color: None, shader: None, vertex: None, deform: None };

        assert_eq!(result.state.part_index(), 5);
        assert!(result.cell.is_none());
        assert!(result.part_color.is_none());
    }
}
