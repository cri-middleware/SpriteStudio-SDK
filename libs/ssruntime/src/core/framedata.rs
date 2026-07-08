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
    if ssab.cellmaps().is_none() {
        return alloc::vec::Vec::new();
    }

    let mut out = alloc::vec::Vec::with_capacity(cell_array.len());
    for key in cell_array {
        let value = key.value();
        let meta = resolve_single_cell(ssab, value.cellmap_name_hash(), value.name_hash()).unwrap_or_default();
        out.push(meta);
    }
    out
}

/// Resolve one `(cellmap_name_hash, cell_name_hash)` against the SSAB cellmaps
/// to a [`ResolvedCellMeta`], or `None` when either lookup fails. Used by the
/// Override Layer to cache the resolved cell at set time; shares the exact
/// resolution logic with [`resolve_cell_array_meta`].
pub(crate) fn resolve_single_cell(ssab: &ssab::ssab::ss::format::SsAnimeBinary, cellmap_name_hash: u32, cell_name_hash: u32) -> Option<ResolvedCellMeta> {
    let cellmaps = ssab.cellmaps()?;
    let cellmap = crate::core::ssab_lookup::find_cellmap(&cellmaps, cellmap_name_hash)?;
    let cells = cellmap.cells()?;
    let cell = crate::core::ssab_lookup::find_cell(&cells, cell_name_hash)?;

    let p = cell.pivot();
    let (pivot_x, pivot_y) = (p.v1(), p.v2());
    let r = cell.rectangle();
    let (rect_left, rect_top, size_w, size_h) = (r.x1(), r.y1(), r.x2(), r.y2());
    // Mirror compute_local_uvs's fallback: if the cellmap doesn't carry
    // size_original (or it's zero), treat the cell rect dimensions as the size.
    let v = cellmap.size_original();
    let (w, h) = (v.v1(), v.v2());
    let (tex_w, tex_h) = if w > 0.0 && h > 0.0 { (w, h) } else { (size_w, size_h) };
    Some(ResolvedCellMeta { pivot_x, pivot_y, size_w, size_h, rect_left, rect_top, tex_w, tex_h, rotated: cell.rotated(), texture_hash: cellmap.name_hash() })
}

/// Pre-decoded skinned mesh bind-pose data for a single part.
///
/// Built once at SSAB load time from `PartData.mesh_binding()` (AoS in the
/// FlatBuffer) and stored on `CachedPartData`. CSR layout for influence
/// offsets + SoA for weights / bind coords so the per-frame skinning pass
/// (`compute_skinned_mesh_vertices`) can stream the data SIMD-friendly.
///
/// Bone indices are part-resolved: ssconverter has already mapped each
/// `TableIDPartsBone` slot to its owning part index, so the runtime can
/// directly index `bone_world_matrices[bone_indices[i]]`.
#[derive(Clone, Default)]
pub struct MeshBindingCache {
    pub influence_offsets: alloc::vec::Vec<u32>,
    pub bone_indices: alloc::vec::Vec<u16>,
    pub weights: alloc::vec::Vec<f32>,
    pub coords_x: alloc::vec::Vec<f32>,
    pub coords_y: alloc::vec::Vec<f32>,
}

impl MeshBindingCache {
    #[inline]
    pub fn num_vertices(&self) -> usize {
        self.influence_offsets.len().saturating_sub(1)
    }
}

/// Builds a [`MeshBindingCache`] from the SSAB-side `PartMeshBinding` table.
/// Returns `None` when the part has no mesh binding or the offset vector is
/// empty.
pub fn build_mesh_binding_cache(mesh_binding: Option<ssab::ssab::ss::format::PartMeshBinding<'_>>) -> Option<MeshBindingCache> {
    let binding = mesh_binding?;
    let offsets = binding.vertex_influence_offset()?;
    if offsets.len() < 2 {
        return None;
    }
    let bones = binding.influence_bone()?;
    let weights = binding.influence_weight()?;
    let coords_x = binding.influence_coord_x()?;
    let coords_y = binding.influence_coord_y()?;

    Some(MeshBindingCache { influence_offsets: offsets.iter().collect(), bone_indices: bones.iter().collect(), weights: weights.iter().collect(), coords_x: coords_x.iter().collect(), coords_y: coords_y.iter().collect() })
}

/// Per-part static mesh geometry resolved from the part's Cell.
///
/// Mesh parts may not change their cell after frame 0 (SS spec), so this is
/// resolved once at SSAB load time. `uvs` are interleaved `[u, v, ...]` in
/// cellmap texture-pixel space (the cell rectangle origin is already added —
/// divide by the texture size to normalize). `rest_x` / `rest_y` are SoA
/// rest-pose positions (cell coordinate minus the cell pivot's pixel
/// position, so the mesh is centred; Y-down native — same texture-pixel
/// space as `uvs`), consumed by the non-skeletal (Deform-only) mesh path.
/// SoA so the per-frame `W · rest` transform vectorizes with contiguous
/// loads. `indices` are triangle indices widened from the SSAB-side `u16`
/// `table_index_vertex`.
#[derive(Clone, Default)]
pub struct MeshCellData {
    pub uvs: alloc::vec::Vec<f32>,
    pub rest_x: alloc::vec::Vec<f32>,
    pub rest_y: alloc::vec::Vec<f32>,
    pub indices: alloc::vec::Vec<i32>,
}

impl MeshCellData {
    #[inline]
    pub fn num_vertices(&self) -> usize {
        self.uvs.len() / 2
    }
}

/// Per-frame mesh deform for one part, retained from Phase 1's `evaluate_deform`
/// so the Phase 2 mesh-vertex pass can consume it. `indices` selects which mesh
/// vertices are deformed; `dx`/`dy` are the matching SoA deltas in part-local
/// space (Y-down flip already applied). All three empty when the part has no
/// deform this frame.
#[derive(Clone, Default)]
pub struct PartDeformFrame {
    pub indices: alloc::vec::Vec<i16>,
    pub dx: alloc::vec::Vec<f32>,
    pub dy: alloc::vec::Vec<f32>,
}

impl PartDeformFrame {
    #[inline]
    pub fn clear(&mut self) {
        self.indices.clear();
        self.dx.clear();
        self.dy.clear();
    }
}

#[derive(Default, Copy, Clone)]
pub struct RawVertexDeform {
    pub present: bool,
    pub dx: [f32; 4],
    pub dy: [f32; 4],
}

/// Resolves a mesh part's Cell — fixed at frame 0 per the SS mesh spec, so the
/// first `cell_array` keyframe is authoritative — and extracts its triangulated
/// geometry. Returns `None` when the part has no cell or the cell carries no
/// mesh tables (i.e. it is not a mesh cell).
pub fn build_mesh_cell_data<'a>(ssab: &ssab::ssab::ss::format::SsAnimeBinary<'a>, cell_array: Option<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataCellValue<'a>>>>) -> Option<MeshCellData> {
    let cell_array = cell_array?;
    if cell_array.is_empty() {
        return None;
    }
    let value = cell_array.get(0).value();

    let cellmaps = ssab.cellmaps()?;
    let cellmap = crate::core::ssab_lookup::find_cellmap(&cellmaps, value.cellmap_name_hash())?;
    let cells = cellmap.cells()?;
    let cell = crate::core::ssab_lookup::find_cell(&cells, value.name_hash())?;

    let coord_x = cell.table_coord_x()?;
    let coord_y = cell.table_coord_y()?;
    let tri = cell.table_index_vertex()?;
    if coord_x.is_empty() || coord_y.is_empty() || tri.is_empty() {
        return None;
    }
    let n = coord_x.len().min(coord_y.len());

    // Cell-local mesh coordinates are pixel offsets within the cell. For UVs
    // add the cell rectangle origin (→ cellmap texture-pixel space); for rest
    // positions subtract the cell pivot so the mesh sits centred on the part
    // origin. `table_coord_x` / `table_coord_y` are the `.ssce` meshPointList
    // verbatim, i.e. texture-pixel space (Y-down native); the Deform-only mesh
    // path flips it only for a Y-up target. `cell.pivot()` is normalized
    // (offset from the cell centre, 0 = centre) and Y-up; convert to a Y-down
    // texture-pixel position via `(pivot.x + 0.5) * size_w` and
    // `(-pivot.y + 0.5) * size_h` — the Y pivot is negated to mirror
    // compute_local_vertices (`cpy`) so it matches the Y-down coordinates.
    let r = cell.rectangle();
    let (rect_left, rect_top, size_w, size_h) = (r.x1(), r.y1(), r.x2(), r.y2());
    let p = cell.pivot();
    let (pivot_norm_x, pivot_norm_y) = (p.v1(), p.v2());
    let pivot_px_x = (pivot_norm_x + 0.5) * size_w;
    let pivot_px_y = (-pivot_norm_y + 0.5) * size_h;
    let mut uvs = alloc::vec::Vec::with_capacity(n * 2);
    let mut rest_x = alloc::vec::Vec::with_capacity(n);
    let mut rest_y = alloc::vec::Vec::with_capacity(n);
    for i in 0..n {
        let x = coord_x.get(i);
        let y = coord_y.get(i);
        uvs.push(x + rect_left);
        uvs.push(y + rect_top);
        rest_x.push(x - pivot_px_x);
        rest_y.push(y - pivot_px_y);
    }
    let indices: alloc::vec::Vec<i32> = tri.iter().map(|i| i as i32).collect();

    Some(MeshCellData { uvs, indices, rest_x, rest_y })
}

#[derive(Clone)]
pub struct CachedPartData<'a> {
    pub part_index: i16,
    pub parent_index: i16,
    /// `fnv1a_hash_str(PartData.name)`, cached for the Override Layer's
    /// `name -> part_index` resolver. Stable within an SSAB (part identity is
    /// AnimationPack-scoped and 1:1 with `parts_animation_data`).
    pub name_hash: u32,
    pub resolved_cells: alloc::vec::Vec<ResolvedCellMeta>,
    pub shape_kind: Option<crate::core::geometry::ShapeType>,
    pub mesh_binding: Option<MeshBindingCache>,
    pub mesh_cell: Option<MeshCellData>,

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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        part: ssab::ssab::ss::format::PartAnimationData<'a>,
        part_index: i16,
        parent_index: i16,
        name_hash: u32,
        resolved_cells: alloc::vec::Vec<ResolvedCellMeta>,
        shape_kind: Option<crate::core::geometry::ShapeType>,
        mesh_binding: Option<MeshBindingCache>,
        mesh_cell: Option<MeshCellData>,
        blend_type: ssab::ssab::ss::format::BlendType,
        part_type_disc: ssab::ssab::ss::format::PartType,
        mask_influence: bool,
        is_mask_contributor: bool,
    ) -> Self {
        Self {
            part_index,
            parent_index,
            name_hash,
            resolved_cells,
            shape_kind,
            mesh_binding,
            mesh_cell,
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
    /// Set when `evaluate_deform` produced a key. Coordinates / indices
    /// live in the caller-owned workspaces — they are valid until the
    /// next `evaluate_part` overwrites them, so the caller must consume
    /// them within the same loop iteration.
    pub deform_present: bool,
}

#[allow(clippy::too_many_arguments)]
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
    deform_coord_workspace: &mut Vec<runtime::framedata::ss::runtime::Vec2>,
    deform_index_workspace: &mut Vec<i16>,
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
    // A part with no Hide keys at all is rendered as hidden by default.
    // hide_array=None leaves this initial state intact; hide_array=Some(..)
    // overrides it via evaluate_bool below. This is the standard SS authoring
    // model: a part has to opt in to being shown with a Hide=false key.
    part_state.set_hide(true);

    let mut update_flag = runtime::framedata::ss::runtime::UpdateAttributeFlags::empty();

    let mut result = PartEvaluationResult { state: runtime::framedata::ss::runtime::PartState::default(), cell: None, cell_meta: None, part_color: None, shader: None, vertex: None, deform_present: false };

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
    if let Some(mask) = cached_part.deform_array.and_then(|arr| evaluate_deform(arr, frame_no, &mut part_cursors[IDX_ATTRIBUTEDEFORM], start_workspace, end_workspace, out_workspace, deform_coord_workspace, deform_index_workspace)) {
        // Deform offsets are an ssae animation attribute — Y-up native like
        // position / pivot / anchor — so they flip for a Y-down target, the
        // SAME condition as those attributes. The mesh rest, by contrast,
        // comes from the .ssce cell (`table_coordinate`) and is Y-down native,
        // so it flips on the opposite (YUp) condition in compute_mesh_vertices.
        // rest and deform are deliberately mirror-imaged.
        if coordinate_system == SsCoordinateSystem::YDown {
            for v in deform_coord_workspace.iter_mut() {
                v.set_y(-v.y());
            }
        }
        result.deform_present = true;
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

/// Pre-multiply one PartColor corner's alpha by the part's hierarchical alpha
/// (RGB and per-corner `rate` are preserved). Shared by keyframe decode
/// ([`extract_pose`]) and the Override Layer so both honor one alpha contract.
pub(crate) fn premultiply_part_color_corner(c: &crate::runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor, alpha: f32) -> crate::runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor {
    let rgba = c.rgba();
    let new_a = ((rgba.a() as f32 / 255.0) * alpha * 255.0).clamp(0.0, 255.0) as u8;
    crate::runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&crate::runtime::framedata::ss::runtime::U8Rect::new(new_a, rgba.r(), rgba.g(), rgba.b()), c.rate())
}

#[allow(clippy::too_many_arguments)]
pub fn extract_pose<'a>(
    fb: &mut flatbuffers::FlatBufferBuilder<'a>,
    cached_parts: &[CachedPartData<'a>],
    frame_no: f32,
    parts_vec: &mut Vec<runtime::framedata::ss::runtime::PartState>,
    cell_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeCell>,
    parts_color_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
    shader_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
    deform_vec: &mut Vec<flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>,
    raw_vertex_deform: &mut [RawVertexDeform],
    cursors: &mut [usize],
    cell_meta_per_part: &mut [f32],
    cell_texture_hashes: &mut [u32],
    mesh_deform: &mut Vec<PartDeformFrame>,
    coordinate_system: SsCoordinateSystem,
    origin: SsOrigin,
    unit: SsUnit,
    canvas_width: f32,
    canvas_height: f32,
    start_workspace: &mut Vec<f32>,
    end_workspace: &mut Vec<f32>,
    out_workspace: &mut Vec<f32>,
    deform_coord_workspace: &mut Vec<runtime::framedata::ss::runtime::Vec2>,
    deform_index_workspace: &mut Vec<i16>,
    inheritance_alphas_workspace: &mut Vec<f32>,
) {
    if mesh_deform.len() != cached_parts.len() {
        mesh_deform.resize(cached_parts.len(), PartDeformFrame::default());
    }
    for d in mesh_deform.iter_mut() {
        d.clear();
    }
    parts_vec.clear();
    cell_vec.clear();
    parts_color_vec.clear();
    shader_vec.clear();
    deform_vec.clear();
    for slot in cell_meta_per_part.iter_mut() {
        *slot = 0.0;
    }
    for slot in cell_texture_hashes.iter_mut() {
        *slot = 0;
    }
    for slot in raw_vertex_deform.iter_mut() {
        *slot = RawVertexDeform::default();
    }

    inheritance_alphas_workspace.clear();
    inheritance_alphas_workspace.reserve(cached_parts.len());

    for (i, part) in cached_parts.iter().enumerate() {
        let part_cursors = &mut cursors[i * NUM_PART_ATTRIBUTES..(i + 1) * NUM_PART_ATTRIBUTES];
        let mut result = evaluate_part(part, frame_no, part_cursors, coordinate_system, origin, unit, canvas_width, canvas_height, start_workspace, end_workspace, out_workspace, deform_coord_workspace, deform_index_workspace);

        let cell_meta = result.cell_meta.unwrap_or_default();
        let (cell_size_w, cell_size_h) = (cell_meta.size_w, cell_meta.size_h);

        let off = i * 6;
        if off + 6 <= cell_meta_per_part.len() {
            cell_meta_per_part[off] = cell_meta.pivot_x;
            cell_meta_per_part[off + 1] = cell_meta.pivot_y;
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

        let inh_alpha = result.state.alpha() * parent_inh_alpha;
        inheritance_alphas_workspace.push(inh_alpha);

        let has_local_alpha = result.state.update_flag().contains(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeLoAlpha);
        let effective_alpha = if has_local_alpha { result.state.local_alpha() } else { inh_alpha };
        result.state.set_alpha(effective_alpha);

        // Hide is per-part in SS7: a hidden parent does NOT hide its children.
        // The hierarchy is a transform hierarchy, and hidden parts are routinely
        // used as pure transform anchors with visible children (e.g. a hidden
        // hand part anchoring a visible weapon). Only alpha inherits down the
        // tree; `hide` must not be propagated.

        // Pre-multiply per-vertex parts color alpha with the hierarchical alpha.
        if let Some(pc) = result.part_color {
            let alpha = result.state.alpha();
            let blend_type = pc.blend_type();
            let lt = premultiply_part_color_corner(pc.lt(), alpha);
            let rt = premultiply_part_color_corner(pc.rt(), alpha);
            let lb = premultiply_part_color_corner(pc.lb(), alpha);
            let rb = premultiply_part_color_corner(pc.rb(), alpha);

            result.part_color = Some(crate::runtime::framedata::ss::runtime::PartAttributePartColor::new(blend_type, &lt, &rt, &lb, &rb));
        }

        if let Some(v) = result.vertex.as_ref()
            && i < raw_vertex_deform.len()
        {
            let lt = v.lt();
            let rt = v.rt();
            let lb = v.lb();
            let rb = v.rb();
            raw_vertex_deform[i] = RawVertexDeform { present: true, dx: [lt.x(), rt.x(), lb.x(), rb.x()], dy: [lt.y(), rt.y(), lb.y(), rb.y()] };
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
        if result.deform_present {
            let mut args = runtime::framedata::ss::runtime::PartAttributeDeformArgs::default();
            if !deform_coord_workspace.is_empty() {
                args.coordinate = Some(fb.create_vector(deform_coord_workspace));
            }
            if !deform_index_workspace.is_empty() {
                args.index = Some(fb.create_vector(deform_index_workspace));
            }
            let deform_off = runtime::framedata::ss::runtime::PartAttributeDeform::create(fb, &args);
            deform_vec.push(deform_off);
            result.state.set_deform((deform_vec.len() - 1) as i16);
            // Retain per-part deform so the Phase 2 mesh-vertex pass can apply it.
            mesh_deform[i].indices.extend_from_slice(deform_index_workspace.as_slice());
            for c in deform_coord_workspace.iter() {
                mesh_deform[i].dx.push(c.x());
                mesh_deform[i].dy.push(c.y());
            }
        }
        parts_vec.push(result.state);
    }
}

pub fn compute_local_vertices_for_parts(
    parts_vec: &mut [runtime::framedata::ss::runtime::PartState],
    cell_meta_per_part: &[f32],
    raw_vertex_deform: &[RawVertexDeform],
    vertex_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
    local_vertices: &mut [f32],
    coordinate_system: SsCoordinateSystem,
) {
    vertex_vec.clear();
    for slot in local_vertices.iter_mut() {
        *slot = 0.0;
    }

    for (i, state) in parts_vec.iter_mut().enumerate() {
        if state.cell() < 0 {
            continue;
        }

        let cell_pivot_x = cell_meta_per_part.get(i * 6).copied().unwrap_or(0.0);
        let cell_pivot_y = cell_meta_per_part.get(i * 6 + 1).copied().unwrap_or(0.0);
        let pivot = (cell_pivot_x, cell_pivot_y);

        let deform = raw_vertex_deform.get(i).filter(|d| d.present).map(|d| (d.dx, d.dy));

        let mut pivot_y = state.pivot_y();
        if coordinate_system == SsCoordinateSystem::YDown {
            pivot_y = -pivot_y;
        }

        let (vx, mut vy) = crate::core::geometry::compute_local_vertices((state.size_x(), state.size_y()), pivot, (state.pivot_x(), pivot_y), (state.img_flip_h(), state.img_flip_v()), deform.as_ref().map(|(x, y)| (x, y)));

        if coordinate_system == SsCoordinateSystem::YDown {
            for y in vy.iter_mut() {
                *y = -*y;
            }
        }

        let lt = crate::runtime::framedata::ss::runtime::Vec2::new(vx[0], vy[0]);
        let rt = crate::runtime::framedata::ss::runtime::Vec2::new(vx[1], vy[1]);
        let lb = crate::runtime::framedata::ss::runtime::Vec2::new(vx[2], vy[2]);
        let rb = crate::runtime::framedata::ss::runtime::Vec2::new(vx[3], vy[3]);
        let needs_center = state.update_flag().contains(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeVertex) || state.update_flag().contains(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePartColor);
        let center = if needs_center { crate::runtime::framedata::ss::runtime::Vec2::new(vx[4], vy[4]) } else { crate::runtime::framedata::ss::runtime::Vec2::default() };
        vertex_vec.push(crate::runtime::framedata::ss::runtime::PartAttributeVertex::new(&lt, &rt, &lb, &rb, &center));
        state.set_vertex((vertex_vec.len() - 1) as i16);

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
}

type PoseFbOffsets<'a> = (
    Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeCell>>>,
    Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributePartColor>>>,
    Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeShader>>>,
    Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, runtime::framedata::ss::runtime::PartAttributeVertex>>>,
    Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>>>,
);

fn serialize_pose_to_fb<'a>(
    fb: &mut flatbuffers::FlatBufferBuilder<'a>,
    cell_vec: &[runtime::framedata::ss::runtime::PartAttributeCell],
    parts_color_vec: &[runtime::framedata::ss::runtime::PartAttributePartColor],
    shader_vec: &[runtime::framedata::ss::runtime::PartAttributeShader],
    vertex_vec: &[runtime::framedata::ss::runtime::PartAttributeVertex],
    deform_vec: &[flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>],
) -> PoseFbOffsets<'a> {
    let cells_off = (!cell_vec.is_empty()).then(|| fb.create_vector(cell_vec));
    let parts_color_off = (!parts_color_vec.is_empty()).then(|| fb.create_vector(parts_color_vec));
    let shaders_off = (!shader_vec.is_empty()).then(|| fb.create_vector(shader_vec));
    let vertices_off = (!vertex_vec.is_empty()).then(|| fb.create_vector(vertex_vec));
    let deforms_off = (!deform_vec.is_empty()).then(|| fb.create_vector(deform_vec));
    (cells_off, parts_color_off, shaders_off, vertices_off, deforms_off)
}

#[allow(clippy::too_many_arguments)]
fn classify_for_draw_batch(disc: ssab::ssab::ss::format::PartType) -> Option<(runtime::framedata::ss::runtime::DrawBatchKind, bool)> {
    use runtime::framedata::ss::runtime::DrawBatchKind as K;
    use ssab::ssab::ss::format::PartType as P;
    if disc == P::PartTypeNormal {
        return Some((K::Normal, true));
    }
    if disc == P::PartTypeShape {
        return Some((K::Shape, true));
    }
    if disc == P::PartTypeMesh {
        // Modern renderers support dynamic index buffer concatenation for arbitrary meshes
        // within a single DrawBatch, so they are fully batchable.
        return Some((K::Mesh, true));
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
        // Masks honour Hide too: a hidden mask must NOT contribute coverage
        // (SS mask opt-out is mask-value==0 OR Hide==true). Previously pure
        // masks were force-emitted regardless of Hide (harmless when masks were
        // never rendered); now that the player rasterises mask coverage, a
        // hidden mask would wrongly clip its targets, so respect Hide here.
        let drawn_as_self = !part_state.hide() && !normal_no_cell;

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
pub fn compose_frame_data<'a>(
    fb: &mut flatbuffers::FlatBufferBuilder<'a>,
    cached_parts: &[CachedPartData<'a>],
    z_order: &mut crate::core::z_order::ZOrderTable,
    parts_vec: &mut [runtime::framedata::ss::runtime::PartState],
    cell_vec: &[crate::runtime::framedata::ss::runtime::PartAttributeCell],
    parts_color_vec: &[crate::runtime::framedata::ss::runtime::PartAttributePartColor],
    shader_vec: &[crate::runtime::framedata::ss::runtime::PartAttributeShader],
    vertex_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
    deform_vec: &[flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>],
    raw_vertex_deform: &[RawVertexDeform],
    cell_meta_per_part: &[f32],
    cell_texture_hashes: &[u32],
    local_vertices: &mut [f32],
    coordinate_system: SsCoordinateSystem,
    render_mode: crate::core::RenderMode,
    draw_order_workspace: &mut Vec<u16>,
    batches_workspace: &mut Vec<runtime::framedata::ss::runtime::DrawBatch>,
    mask_indices_workspace: &mut Vec<u16>,
) -> flatbuffers::WIPOffset<runtime::framedata::ss::runtime::FrameData<'a>> {
    compute_local_vertices_for_parts(parts_vec, cell_meta_per_part, raw_vertex_deform, vertex_vec, local_vertices, coordinate_system);

    let (cells_off, parts_color_off, shaders_off, vertices_off, deforms_off) = if render_mode == crate::core::RenderMode::Direct {
        let pc = (!parts_color_vec.is_empty()).then(|| fb.create_vector(parts_color_vec));
        let s = (!shader_vec.is_empty()).then(|| fb.create_vector(shader_vec));
        (None, pc, s, None, None)
    } else {
        serialize_pose_to_fb(fb, cell_vec, parts_color_vec, shader_vec, vertex_vec, deform_vec)
    };

    let mut args = runtime::framedata::ss::runtime::FrameDataArgs { cells: cells_off, parts_color: parts_color_off, shaders: shaders_off, vertices: vertices_off, deforms: deforms_off, ..Default::default() };
    args.parts = Some(fb.create_vector(parts_vec));

    // z_order と batch 計算は parts_vec / cell_texture_hashes が満たされた後で行う。
    // FrameData::create より前に args.draw_order / args.draw_batches を設定する必要があるため
    // ここで実行する (lib.rs から z_order.update を移譲)。
    z_order.update(parts_vec);
    let (draw_order_off, draw_batches_off, mask_indices_off) = build_draw_order_and_batches(fb, z_order.indices(), cached_parts, parts_vec, cell_texture_hashes, draw_order_workspace, batches_workspace, mask_indices_workspace);
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

        let result = PartEvaluationResult { state: part_state, cell: None, cell_meta: None, part_color: None, shader: None, vertex: None, deform_present: false };

        assert_eq!(result.state.part_index(), 5);
        assert!(result.cell.is_none());
        assert!(result.part_color.is_none());
    }

    fn make_stage_b_state(cell_index: i16, size: (f32, f32), pivot: (f32, f32), flag: UpdateAttributeFlags) -> runtime::framedata::ss::runtime::PartState {
        let mut s = runtime::framedata::ss::runtime::PartState::default();
        s.set_cell(cell_index);
        s.set_vertex(-1);
        s.set_size_x(size.0);
        s.set_size_y(size.1);
        s.set_pivot_x(pivot.0);
        s.set_pivot_y(pivot.1);
        s.set_update_flag(flag);
        s
    }

    #[test]
    fn stage_b_skips_cellless_parts() {
        let mut parts_vec = vec![make_stage_b_state(-1, (10.0, 20.0), (0.0, 0.0), UpdateAttributeFlags::empty())];
        let cell_meta_per_part = vec![0.0_f32; 6];
        let raw_vertex_deform = vec![RawVertexDeform::default()];
        let mut vertex_vec: Vec<runtime::framedata::ss::runtime::PartAttributeVertex> = Vec::new();
        let mut local_vertices = vec![0.0_f32; LOCAL_VERTICES_STRIDE];

        compute_local_vertices_for_parts(&mut parts_vec, &cell_meta_per_part, &raw_vertex_deform, &mut vertex_vec, &mut local_vertices, crate::core::SsCoordinateSystem::YUp);

        assert!(vertex_vec.is_empty(), "vertex_vec should not gain entries for cell-less parts");
        assert_eq!(parts_vec[0].vertex(), -1, "vertex index should remain -1");
        assert!(local_vertices.iter().all(|&v| v == 0.0), "local_vertices for cell-less parts should stay zero");
    }

    #[test]
    fn stage_b_resolves_no_deform_box() {
        // size (10, 20), cell pivot (0, 0), state pivot (0, 0), Y-Up:
        // 5-corner box centered at origin: LT(-5,10), RT(5,10), LB(-5,-10), RB(5,-10), center(0,0).
        let mut parts_vec = vec![make_stage_b_state(0, (10.0, 20.0), (0.0, 0.0), UpdateAttributeFlags::AttributeCell)];
        let cell_meta_per_part = vec![0.0_f32; 6];
        let raw_vertex_deform = vec![RawVertexDeform::default()];
        let mut vertex_vec: Vec<runtime::framedata::ss::runtime::PartAttributeVertex> = Vec::new();
        let mut local_vertices = vec![0.0_f32; LOCAL_VERTICES_STRIDE];

        compute_local_vertices_for_parts(&mut parts_vec, &cell_meta_per_part, &raw_vertex_deform, &mut vertex_vec, &mut local_vertices, crate::core::SsCoordinateSystem::YUp);

        assert_eq!(vertex_vec.len(), 1);
        assert_eq!(parts_vec[0].vertex(), 0);
        let v = &vertex_vec[0];
        assert_eq!((v.lt().x(), v.lt().y()), (-5.0, 10.0));
        assert_eq!((v.rt().x(), v.rt().y()), (5.0, 10.0));
        assert_eq!((v.lb().x(), v.lb().y()), (-5.0, -10.0));
        assert_eq!((v.rb().x(), v.rb().y()), (5.0, -10.0));
        assert_eq!(&local_vertices[..], &[-5.0, 10.0, 5.0, 10.0, -5.0, -10.0, 5.0, -10.0, 0.0, 0.0]);
    }

    #[test]
    fn stage_b_applies_deform_offsets() {
        // Same box as above + deform offsets per corner. AttributeVertex flag is set so
        // the packed `PartAttributeVertex.center` carries the Varignon-mean center.
        let mut parts_vec = vec![make_stage_b_state(0, (10.0, 20.0), (0.0, 0.0), UpdateAttributeFlags::AttributeCell | UpdateAttributeFlags::AttributeVertex)];
        let cell_meta_per_part = vec![0.0_f32; 6];
        let raw_vertex_deform = vec![RawVertexDeform { present: true, dx: [1.0, 2.0, 3.0, 4.0], dy: [10.0, 20.0, 30.0, 40.0] }];
        let mut vertex_vec: Vec<runtime::framedata::ss::runtime::PartAttributeVertex> = Vec::new();
        let mut local_vertices = vec![0.0_f32; LOCAL_VERTICES_STRIDE];

        compute_local_vertices_for_parts(&mut parts_vec, &cell_meta_per_part, &raw_vertex_deform, &mut vertex_vec, &mut local_vertices, crate::core::SsCoordinateSystem::YUp);

        let v = &vertex_vec[0];
        assert_eq!((v.lt().x(), v.lt().y()), (-4.0, 20.0)); // base (-5,10) + (1,10)
        assert_eq!((v.rt().x(), v.rt().y()), (7.0, 30.0)); //      (5,10) + (2,20)
        assert_eq!((v.lb().x(), v.lb().y()), (-2.0, 20.0)); // (-5,-10) + (3,30)
        assert_eq!((v.rb().x(), v.rb().y()), (9.0, 30.0)); //  (5,-10) + (4,40)
        // Varignon center: ((-4+7-2+9)/4, (20+30+20+30)/4) = (2.5, 25)
        assert_eq!((v.center().x(), v.center().y()), (2.5, 25.0));
    }

    #[test]
    fn stage_b_y_down_flips_vertical() {
        // Same setup as stage_b_resolves_no_deform_box, but Y-Down flips all vy.
        let mut parts_vec = vec![make_stage_b_state(0, (10.0, 20.0), (0.0, 0.0), UpdateAttributeFlags::AttributeCell | UpdateAttributeFlags::AttributeVertex)];
        let cell_meta_per_part = vec![0.0_f32; 6];
        let raw_vertex_deform = vec![RawVertexDeform::default()];
        let mut vertex_vec: Vec<runtime::framedata::ss::runtime::PartAttributeVertex> = Vec::new();
        let mut local_vertices = vec![0.0_f32; LOCAL_VERTICES_STRIDE];

        compute_local_vertices_for_parts(&mut parts_vec, &cell_meta_per_part, &raw_vertex_deform, &mut vertex_vec, &mut local_vertices, crate::core::SsCoordinateSystem::YDown);

        let v = &vertex_vec[0];
        // vx unchanged, vy negated.
        assert_eq!((v.lt().x(), v.lt().y()), (-5.0, -10.0));
        assert_eq!((v.rt().x(), v.rt().y()), (5.0, -10.0));
        assert_eq!((v.lb().x(), v.lb().y()), (-5.0, 10.0));
        assert_eq!((v.rb().x(), v.rb().y()), (5.0, 10.0));
        assert_eq!((v.center().x(), v.center().y()), (0.0, 0.0));
    }

    #[test]
    fn stage_c_empty_vecs_yield_none() {
        let mut fb = flatbuffers::FlatBufferBuilder::new();
        let cell_vec: Vec<runtime::framedata::ss::runtime::PartAttributeCell> = Vec::new();
        let parts_color_vec: Vec<runtime::framedata::ss::runtime::PartAttributePartColor> = Vec::new();
        let shader_vec: Vec<runtime::framedata::ss::runtime::PartAttributeShader> = Vec::new();
        let vertex_vec: Vec<runtime::framedata::ss::runtime::PartAttributeVertex> = Vec::new();
        let deform_vec: Vec<flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform>> = Vec::new();

        let (c, pc, s, v, d) = serialize_pose_to_fb(&mut fb, &cell_vec, &parts_color_vec, &shader_vec, &vertex_vec, &deform_vec);

        assert!(c.is_none());
        assert!(pc.is_none());
        assert!(s.is_none());
        assert!(v.is_none());
        assert!(d.is_none());
    }

    #[test]
    fn stage_c_populated_vecs_yield_some() {
        let mut fb = flatbuffers::FlatBufferBuilder::new();
        let deform_off = runtime::framedata::ss::runtime::PartAttributeDeform::create(&mut fb, &runtime::framedata::ss::runtime::PartAttributeDeformArgs::default());

        let cell_vec = vec![runtime::framedata::ss::runtime::PartAttributeCell::default()];
        let parts_color_vec = vec![runtime::framedata::ss::runtime::PartAttributePartColor::default()];
        let shader_vec = vec![runtime::framedata::ss::runtime::PartAttributeShader::default()];
        let vertex_vec = vec![runtime::framedata::ss::runtime::PartAttributeVertex::default()];
        let deform_vec = vec![deform_off];

        let (c, pc, s, v, d) = serialize_pose_to_fb(&mut fb, &cell_vec, &parts_color_vec, &shader_vec, &vertex_vec, &deform_vec);

        assert!(c.is_some());
        assert!(pc.is_some());
        assert!(s.is_some());
        assert!(v.is_some());
        assert!(d.is_some());
    }

    #[test]
    fn stage_c_emits_only_populated_attributes() {
        // Only deform_vec is populated; the remaining four attributes are empty
        // and must yield None — never an empty vector offset.
        let mut fb = flatbuffers::FlatBufferBuilder::new();
        let deform_off = runtime::framedata::ss::runtime::PartAttributeDeform::create(&mut fb, &runtime::framedata::ss::runtime::PartAttributeDeformArgs::default());

        let cell_vec: Vec<runtime::framedata::ss::runtime::PartAttributeCell> = Vec::new();
        let parts_color_vec: Vec<runtime::framedata::ss::runtime::PartAttributePartColor> = Vec::new();
        let shader_vec: Vec<runtime::framedata::ss::runtime::PartAttributeShader> = Vec::new();
        let vertex_vec: Vec<runtime::framedata::ss::runtime::PartAttributeVertex> = Vec::new();
        let deform_vec = vec![deform_off];

        let (c, pc, s, v, d) = serialize_pose_to_fb(&mut fb, &cell_vec, &parts_color_vec, &shader_vec, &vertex_vec, &deform_vec);

        assert!(c.is_none());
        assert!(pc.is_none());
        assert!(s.is_none());
        assert!(v.is_none());
        assert!(d.is_some());
    }

    #[test]
    fn raw_vertex_deform_default_is_inert() {
        // `extract_pose` zero-fills `raw_vertex_deform` at the top of each frame
        // and Stage B reads `present` to decide whether to apply offsets, so the
        // Default impl must produce `present == false` and zeroed offsets.
        let d = RawVertexDeform::default();
        assert!(!d.present);
        assert_eq!(d.dx, [0.0; 4]);
        assert_eq!(d.dy, [0.0; 4]);
    }

    #[test]
    fn stage_b_center_is_gated_by_update_flag() {
        // Cell pivot (0.1, 0.2) produces a non-symmetric box whose Varignon center is (-1, -4).
        let cell_meta_per_part = vec![0.1_f32, 0.2, 10.0, 20.0, 0.0, 0.0];
        let raw_vertex_deform = vec![RawVertexDeform::default()];

        // Case A: only AttributeCell — center field in vertex_vec stays at default (0, 0),
        // but local_vertices always carries the computed center.
        {
            let mut parts_vec = vec![make_stage_b_state(0, (10.0, 20.0), (0.0, 0.0), UpdateAttributeFlags::AttributeCell)];
            let mut vertex_vec: Vec<runtime::framedata::ss::runtime::PartAttributeVertex> = Vec::new();
            let mut local_vertices = vec![0.0_f32; LOCAL_VERTICES_STRIDE];
            compute_local_vertices_for_parts(&mut parts_vec, &cell_meta_per_part, &raw_vertex_deform, &mut vertex_vec, &mut local_vertices, crate::core::SsCoordinateSystem::YUp);
            assert_eq!((vertex_vec[0].center().x(), vertex_vec[0].center().y()), (0.0, 0.0));
            assert_eq!((local_vertices[8], local_vertices[9]), (-1.0, -4.0));
        }

        // Case B: AttributeVertex also set — center is populated with the computed value.
        {
            let mut parts_vec = vec![make_stage_b_state(0, (10.0, 20.0), (0.0, 0.0), UpdateAttributeFlags::AttributeCell | UpdateAttributeFlags::AttributeVertex)];
            let mut vertex_vec: Vec<runtime::framedata::ss::runtime::PartAttributeVertex> = Vec::new();
            let mut local_vertices = vec![0.0_f32; LOCAL_VERTICES_STRIDE];
            compute_local_vertices_for_parts(&mut parts_vec, &cell_meta_per_part, &raw_vertex_deform, &mut vertex_vec, &mut local_vertices, crate::core::SsCoordinateSystem::YUp);
            assert_eq!((vertex_vec[0].center().x(), vertex_vec[0].center().y()), (-1.0, -4.0));
        }
    }
}
