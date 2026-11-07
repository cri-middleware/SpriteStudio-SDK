use crate::eval::interpolation::*;
use crate::eval::keyframes::{KeyColumns, KeyRun};
use crate::eval::{SsCoordinateSystem, SsOrigin, SsUnit};
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
pub(crate) const IDX_ATTRIBUTEPRIO: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePrio.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEFLIPH: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipH.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEFLIPV: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeFlipV.bits().trailing_zeros() as usize;
const IDX_ATTRIBUTEHIDE: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeHide.bits().trailing_zeros() as usize;
pub(crate) const IDX_ATTRIBUTEPARTCOLOR: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePartColor.bits().trailing_zeros() as usize;
pub(crate) const IDX_ATTRIBUTESHADER: usize = crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeShader.bits().trailing_zeros() as usize;
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

/// Stride (number of `f32`s) per part in the `shape_vertices` and
/// `shape_vertex_box_coords` buffers (`ss_runtime_get_shape_vertices` and
/// `ss_runtime_get_shape_vertex_box_coords` in the C-API): 12 vertices of two
/// floats each, the most a shape part has (a star).
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

/// Resolve one `(cellmap_name_hash, cell_name_hash)` against the SSAB cellmaps
/// to a [`ResolvedCellMeta`], or `None` when either lookup fails. Both
/// cellmaps and cells are stored in `name_hash`-sorted order (FlatBuffers
/// `key`), so each lookup is a binary search. Resolves every cell the pack's
/// keys show, once, at load ([`KeyColumns::new`]), and the Override Layer's
/// cell at set time, so the two cannot disagree.
pub(crate) fn resolve_cell(cellmaps: &flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::CellMap<'_>>>, cellmap_name_hash: u32, cell_name_hash: u32) -> Option<ResolvedCellMeta> {
    let cellmap = crate::eval::ssab_lookup::find_cellmap(cellmaps, cellmap_name_hash)?;
    let cells = cellmap.cells()?;
    let cell = crate::eval::ssab_lookup::find_cell(&cells, cell_name_hash)?;

    let p = cell.pivot();
    let (pivot_x, pivot_y) = (p.v1(), p.v2());
    let r = cell.region();
    let (rect_left, rect_top, size_w, size_h) = (r.x(), r.y(), r.w(), r.h());
    // A zero `size_original` takes the cell rect's size as the texture size.
    let v = cellmap.size_original();
    let (w, h) = (v.v1(), v.v2());
    let (tex_w, tex_h) = if w > 0.0 && h > 0.0 { (w, h) } else { (size_w, size_h) };
    Some(ResolvedCellMeta { pivot_x, pivot_y, size_w, size_h, rect_left, rect_top, tex_w, tex_h, rotated: cell.rotated(), texture_hash: cellmap.name_hash() })
}

/// Pre-decoded skinned mesh bind-pose data for a single part.
///
/// Built once per part at SSAB load time from `PartData.mesh_binding()` (AoS
/// in the FlatBuffer), and shared by that part's `CachedPartData` in every
/// animation: the binding belongs to the part, not to an animation. CSR layout for influence
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
/// resolved once per cell at SSAB load time and shared by every part and
/// animation that draws that cell. `uvs` are interleaved `[u, v, ...]` in
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

/// One part's entry in a per-part deform work area (`RawVertexDeform`,
/// `PartDeformFrame`): nothing until the part first carries a deform, then a
/// box that stays allocated and is cleared every frame. Most parts never
/// deform, so they cost a pointer rather than the whole entry.
pub type DeformSlot<T> = Option<alloc::boxed::Box<T>>;

/// Part `i`'s vertex deform this frame, if it has one.
#[inline]
pub fn vertex_deform_at(raw_vertex_deform: &[DeformSlot<RawVertexDeform>], i: usize) -> Option<&RawVertexDeform> {
    raw_vertex_deform.get(i).and_then(|d| d.as_deref()).filter(|d| d.present)
}

/// A mesh part's Cell, as an index for [`KeyColumns::cell`] — fixed at frame 0
/// per the SS mesh spec, so the first cell keyframe is authoritative. `None`
/// when the part has no cell keys.
pub fn mesh_part_cell(keys: &KeyColumns<'_>, cell_keys: Option<KeyRun>) -> Option<u32> {
    let cell_keys = cell_keys?;
    if cell_keys.count() == 0 {
        return None;
    }
    Some(keys.key(cell_keys.first() as usize).cell())
}

/// Extracts a mesh cell's triangulated geometry. Returns `None` when the cell
/// cannot be resolved or carries no mesh tables (i.e. it is not a mesh cell).
pub fn build_mesh_cell_data(ssab: &ssab::ssab::ss::format::SsAnimeBinary<'_>, value: &runtime::framedata::ss::runtime::PartAttributeCell) -> Option<MeshCellData> {
    let cellmaps = ssab.cellmaps()?;
    let cellmap = crate::eval::ssab_lookup::find_cellmap(&cellmaps, value.cellmap_name_hash())?;
    let cells = cellmap.cells()?;
    let cell = crate::eval::ssab_lookup::find_cell(&cells, value.name_hash())?;

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
    let r = cell.region();
    let (rect_left, rect_top, size_w, size_h) = (r.x(), r.y(), r.w(), r.h());
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

/// Per-part static Nine-Slice authoring, read once from `PartTypeNines`.
/// Margins are `(left, top, right, bottom)` in source-cell pixels.
#[derive(Clone, Copy)]
pub struct NinesPartData {
    pub margins: (f32, f32, f32, f32),
    pub fill_mode: crate::eval::geometry::NinesFillMode,
}

/// Builds a [`NinesPartData`] from the SSAB-side `PartTypeNines` table.
/// Returns `None` for every other part type.
pub fn build_nines_part_data(part_meta: &ssab::ssab::ss::format::PartData<'_>) -> Option<NinesPartData> {
    let n = part_meta.part_type_as_part_type_nines()?;
    Some(NinesPartData { margins: (n.nines_margin_l() as f32, n.nines_margin_t() as f32, n.nines_margin_r() as f32, n.nines_margin_b() as f32), fill_mode: crate::eval::geometry::NinesFillMode::from_i32(n.nines_fill_mode()) })
}

/// Per-part Nine-Slice geometry for the current frame, concatenated across
/// parts in CSR layout. Non-nines parts occupy an empty range.
///
/// `vertices` / `box_coords` / `uvs` are interleaved pairs sliced by
/// `vertex_offsets`; `indices` is sliced by `index_offsets` and its values are
/// local to each part's vertex range.
#[derive(Default)]
pub struct NinesBuffers {
    pub vertices: alloc::vec::Vec<f32>,
    pub uvs: alloc::vec::Vec<f32>,
    pub box_coords: alloc::vec::Vec<f32>,
    pub indices: alloc::vec::Vec<i32>,
    pub vertex_offsets: alloc::vec::Vec<u32>,
    pub index_offsets: alloc::vec::Vec<u32>,
}

impl NinesBuffers {
    fn begin(&mut self, num_parts: usize) {
        self.vertices.clear();
        self.uvs.clear();
        self.box_coords.clear();
        self.indices.clear();
        self.vertex_offsets.clear();
        self.vertex_offsets.reserve(num_parts + 1);
        self.index_offsets.clear();
        self.index_offsets.reserve(num_parts + 1);
    }

    /// Vertex / index count of part `i`, `(0, 0)` for parts without geometry.
    #[inline]
    pub fn counts(&self, i: usize) -> (u32, u32) {
        let span = |o: &[u32]| match (o.get(i), o.get(i + 1)) {
            (Some(start), Some(end)) => end.saturating_sub(*start),
            _ => 0,
        };
        (span(&self.vertex_offsets), span(&self.index_offsets))
    }
}

#[derive(Clone)]
pub struct CachedPartData {
    pub part_index: i16,
    pub parent_index: i16,
    /// Which continuous attributes (`MIXED_ATTRIBUTES`) this animation actually
    /// keys on this part. Decided **per animation** (the run exists at all),
    /// never per frame — a per-frame signal would flip at the first key and
    /// make the part pop in and out of the mix mid-playback.
    ///
    /// The mix consults this to keep a source that does not animate an
    /// attribute from voting `evaluate_part`'s hardcoded default into the
    /// weighted sum. Note the converter seeds every animation with the setup
    /// pose, so "has a run" means "authored by the animator **or** by the
    /// setup pose" — both are real poses, and only the genuinely absent case
    /// is excluded.
    pub coverage: crate::runtime::framedata::ss::runtime::UpdateAttributeFlags,
    /// `fnv1a_hash_str(PartData.name)`, cached for the Override Layer's
    /// `name -> part_index` resolver. Stable within an SSAB (part identity is
    /// AnimationPack-scoped and 1:1 with `parts_animation_data`).
    pub name_hash: u32,
    /// One bit per attribute this part has keys for, at the attribute's
    /// `IDX_ATTRIBUTE*` position. An instance keeps a key cursor for each of
    /// these attributes only, in bit order (see [`cursor_slot`]).
    pub key_mask: u64,
    /// Where this part's key cursors start in an instance's cursor buffer:
    /// the number of cursors every earlier part of the animation keeps.
    pub cursor_base: u32,
    /// The keyed attributes that read the same on every frame of the
    /// animation — a run of one key, which the animation reaches at its start
    /// — as [`key_mask`](Self::key_mask) bits. They are evaluated once, into
    /// the animation's part bases, rather than on every frame.
    pub constant_mask: u64,
    pub shape_kind: Option<crate::eval::geometry::ShapeType>,
    pub nines: Option<NinesPartData>,
    pub mesh_binding: Option<alloc::sync::Arc<MeshBindingCache>>,
    pub mesh_cell: Option<alloc::sync::Arc<MeshCellData>>,

    // Batch-key metadata: read once from the SSAB `PartData` at setup, used by the
    // per-frame `DrawBatch` pass.
    pub blend_type: ssab::ssab::ss::format::BlendType,
    pub part_type_disc: ssab::ssab::ss::format::PartType,
    pub mask_influence: bool,
    pub is_mask_contributor: bool,
    /// The key runs of the attributes in [`key_mask`](Self::key_mask), in bit
    /// order: attribute `idx` is `runs[cursor_slot(key_mask, idx)]`, the same
    /// packing as an instance's cursors. A part keys a handful of its
    /// attributes, so this holds a few runs. Read one with [`run`](Self::run).
    pub runs: alloc::boxed::Box<[KeyRun]>,
}

/// The weighted (continuous) attributes, the ones the mix blends: discrete ones
/// come from the primary and never take part in a weighted sum. A part's
/// [`coverage`](CachedPartData::coverage) is the ones of these it keys.
///
/// `skew_x` / `skew_y` share one key run and therefore one flag, matching how
/// `evaluate_part` sets `AttributeSkew`.
pub(crate) const MIXED_ATTRIBUTES: crate::runtime::framedata::ss::runtime::UpdateAttributeFlags = {
    use crate::runtime::framedata::ss::runtime::UpdateAttributeFlags as F;
    F::AttributePosX
        .union(F::AttributePosY)
        .union(F::AttributePosZ)
        .union(F::AttributeRotX)
        .union(F::AttributeRotY)
        .union(F::AttributeRotZ)
        .union(F::AttributeSclX)
        .union(F::AttributeSclY)
        .union(F::AttributeLosclX)
        .union(F::AttributeLosclY)
        .union(F::AttributeAlpha)
        .union(F::AttributeLoAlpha)
        .union(F::AttributePivotX)
        .union(F::AttributePivotY)
        .union(F::AttributeAnchorX)
        .union(F::AttributeAnchorY)
        .union(F::AttributeSizeX)
        .union(F::AttributeSizeY)
        .union(F::AttributeUvtX)
        .union(F::AttributeUvtY)
        .union(F::AttributeUvrZ)
        .union(F::AttributeUvsX)
        .union(F::AttributeUvsY)
        .union(F::AttributeBoundr)
        .union(F::AttributeMask)
        .union(F::AttributeSkew)
};

impl CachedPartData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        part: ssab::ssab::ss::format::PartAnimationData<'_>,
        keys: &KeyColumns,
        part_index: i16,
        parent_index: i16,
        name_hash: u32,
        shape_kind: Option<crate::eval::geometry::ShapeType>,
        nines: Option<NinesPartData>,
        mesh_binding: Option<alloc::sync::Arc<MeshBindingCache>>,
        mesh_cell: Option<alloc::sync::Arc<MeshCellData>>,
        blend_type: ssab::ssab::ss::format::BlendType,
        part_type_disc: ssab::ssab::ss::format::PartType,
        mask_influence: bool,
        is_mask_contributor: bool,
        start_frame: i32,
    ) -> Self {
        let all: [(Option<KeyRun>, usize); NUM_PART_ATTRIBUTES] = [
            (keys.run(part.cell_keys()), IDX_ATTRIBUTECELL),
            (keys.run(part.position_x_keys()), IDX_ATTRIBUTEPOSX),
            (keys.run(part.position_y_keys()), IDX_ATTRIBUTEPOSY),
            (keys.run(part.position_z_keys()), IDX_ATTRIBUTEPOSZ),
            (keys.run(part.rotation_x_keys()), IDX_ATTRIBUTEROTX),
            (keys.run(part.rotation_y_keys()), IDX_ATTRIBUTEROTY),
            (keys.run(part.rotation_z_keys()), IDX_ATTRIBUTEROTZ),
            (keys.run(part.scale_x_keys()), IDX_ATTRIBUTESCLX),
            (keys.run(part.scale_y_keys()), IDX_ATTRIBUTESCLY),
            (keys.run(part.local_scale_x_keys()), IDX_ATTRIBUTELOSCLX),
            (keys.run(part.local_scale_y_keys()), IDX_ATTRIBUTELOSCLY),
            (keys.run(part.alpha_keys()), IDX_ATTRIBUTEALPHA),
            (keys.run(part.local_alpha_keys()), IDX_ATTRIBUTELOALPHA),
            (keys.run(part.priority_keys()), IDX_ATTRIBUTEPRIO),
            (keys.run(part.flip_h_keys()), IDX_ATTRIBUTEFLIPH),
            (keys.run(part.flip_v_keys()), IDX_ATTRIBUTEFLIPV),
            (keys.run(part.hide_keys()), IDX_ATTRIBUTEHIDE),
            (keys.run(part.part_color_keys()), IDX_ATTRIBUTEPARTCOLOR),
            (keys.run(part.shader_keys()), IDX_ATTRIBUTESHADER),
            (keys.run(part.vertex_keys()), IDX_ATTRIBUTEVERTEX),
            (keys.run(part.pivot_x_keys()), IDX_ATTRIBUTEPIVOTX),
            (keys.run(part.pivot_y_keys()), IDX_ATTRIBUTEPIVOTY),
            (keys.run(part.anchor_x_keys()), IDX_ATTRIBUTEANCHORX),
            (keys.run(part.anchor_y_keys()), IDX_ATTRIBUTEANCHORY),
            (keys.run(part.size_x_keys()), IDX_ATTRIBUTESIZEX),
            (keys.run(part.size_y_keys()), IDX_ATTRIBUTESIZEY),
            (keys.run(part.img_flip_h_keys()), IDX_ATTRIBUTEIMGFLIPH),
            (keys.run(part.img_flip_v_keys()), IDX_ATTRIBUTEIMGFLIPV),
            (keys.run(part.uv_translation_x_keys()), IDX_ATTRIBUTEUVTX),
            (keys.run(part.uv_translation_y_keys()), IDX_ATTRIBUTEUVTY),
            (keys.run(part.uv_rotation_z_keys()), IDX_ATTRIBUTEUVRZ),
            (keys.run(part.uv_scale_x_keys()), IDX_ATTRIBUTEUVSX),
            (keys.run(part.uv_scale_y_keys()), IDX_ATTRIBUTEUVSY),
            (keys.run(part.bounding_radius_keys()), IDX_ATTRIBUTEBOUNDR),
            (keys.run(part.mask_keys()), IDX_ATTRIBUTEMASK),
            (keys.run(part.skew_keys()), IDX_ATTRIBUTESKEW),
            (keys.run(part.deform_keys()), IDX_ATTRIBUTEDEFORM),
            (keys.run(part.texture_keys()), IDX_ATTRIBUTETEXTURE),
        ];
        // `all` lists the attributes in bit order, so the keyed runs gather
        // straight into `key_mask` order, in one allocation of their exact
        // size. This runs for every part of every animation set up.
        debug_assert!(all.iter().enumerate().all(|(bit, (_, idx))| bit == *idx));
        let key_mask = all.iter().filter(|(run, _)| run.is_some()).fold(0u64, |mask, (_, idx)| mask | 1 << idx);
        let mut runs = alloc::vec::Vec::with_capacity(key_mask.count_ones() as usize);
        runs.extend(all.iter().filter_map(|(run, _)| *run));
        let mut cached = Self {
            part_index,
            parent_index,
            name_hash,
            key_mask,
            runs: runs.into_boxed_slice(),
            cursor_base: 0,
            constant_mask: 0,
            coverage: crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::from_bits_retain(key_mask) & MIXED_ATTRIBUTES,
            shape_kind,
            nines,
            mesh_binding,
            mesh_cell,
            blend_type,
            part_type_disc,
            mask_influence,
            is_mask_contributor,
        };
        cached.constant_mask = cached.constant_attributes(keys, start_frame);
        cached
    }

    /// The attributes whose value is the same on every frame of an animation
    /// starting at `start_frame`, as [`constant_mask`](Self::constant_mask) bits:
    /// a run of one key. A float-like run reads its key on every frame. A
    /// bool, cell or texture run reads nothing before its key, so it counts
    /// only when the key sits at or before the start. Deform is left out: its
    /// result lives in per-frame workspaces, not in the part state.
    fn constant_attributes(&self, keys: &KeyColumns, start_frame: i32) -> u64 {
        const READS_NOTHING_BEFORE_ITS_KEY: [usize; 7] = [IDX_ATTRIBUTECELL, IDX_ATTRIBUTEFLIPH, IDX_ATTRIBUTEFLIPV, IDX_ATTRIBUTEHIDE, IDX_ATTRIBUTEIMGFLIPH, IDX_ATTRIBUTEIMGFLIPV, IDX_ATTRIBUTETEXTURE];
        let mut mask = 0;
        for (idx, run) in self.keyed_runs() {
            if run.count() != 1 || idx == IDX_ATTRIBUTEDEFORM {
                continue;
            }
            if READS_NOTHING_BEFORE_ITS_KEY.contains(&idx) && keys.key(run.first() as usize).index() as i32 > start_frame {
                continue;
            }
            mask |= 1 << idx;
        }
        mask
    }

    /// The keyed attributes' `IDX_ATTRIBUTE*` positions with their runs, in bit order.
    fn keyed_runs(&self) -> impl Iterator<Item = (usize, KeyRun)> + '_ {
        let mut bits = self.key_mask;
        self.runs.iter().map(move |run| {
            let idx = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            (idx, *run)
        })
    }

    /// This part's key run for attribute `idx` (an `IDX_ATTRIBUTE*`), or `None`
    /// when the part has no keys for it.
    #[inline]
    pub fn run(&self, idx: usize) -> Option<KeyRun> {
        (self.key_mask >> idx & 1 == 1).then(|| self.runs[cursor_slot(self.key_mask, idx)])
    }

    /// This part's key cursors within an instance's cursor buffer.
    #[inline]
    pub fn cursors<'c>(&self, all: &'c mut [u32]) -> &'c mut [u32] {
        let base = self.cursor_base as usize;
        &mut all[base..base + self.key_mask.count_ones() as usize]
    }
}

/// Where attribute `idx`'s cursor sits among a part's cursors: the part keeps
/// one for each attribute in its `key_mask`, in bit order.
#[inline]
pub fn cursor_slot(key_mask: u64, idx: usize) -> usize {
    (key_mask & ((1u64 << idx) - 1)).count_ones() as usize
}

/// How many key cursors an instance needs to play an animation with
/// `cached_parts`, whose `cursor_base`s run up to it.
pub fn cursor_count(cached_parts: &[CachedPartData]) -> usize {
    cached_parts.last().map_or(0, |p| p.cursor_base as usize + p.key_mask.count_ones() as usize)
}

/// Test-only stub of a [`CachedPartData`] with no keyframe tracks, for unit
/// tests that exercise per-part logic (e.g. the Override Layer) without loading
/// a real SSAB. Only the fields those tests read are parameterized.
#[cfg(test)]
pub(crate) fn cached_part_stub(part_index: i16, parent_index: i16, part_type_disc: ssab::ssab::ss::format::PartType, is_mask_contributor: bool) -> CachedPartData {
    CachedPartData {
        part_index,
        parent_index,
        name_hash: 0,
        key_mask: 0,
        cursor_base: 0,
        constant_mask: 0,
        // The stub has no keyframe tracks at all, but callers use it to exercise
        // per-part logic rather than the mix; claiming full coverage keeps it from
        // silently disabling attributes if one of them ever runs a blend.
        coverage: crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::all(),
        shape_kind: None,
        nines: None,
        mesh_binding: None,
        mesh_cell: None,
        blend_type: ssab::ssab::ss::format::BlendType::Mix,
        part_type_disc,
        mask_influence: false,
        is_mask_contributor,
        runs: alloc::boxed::Box::new([]),
    }
}

pub struct PartEvaluationResult {
    pub state: runtime::framedata::ss::runtime::PartState,
    /// The cell the part shows, as an index for [`KeyColumns::cell`].
    pub cell: Option<u32>,
    pub part_color: Option<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
    pub shader: Option<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
    pub vertex: Option<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
    /// Set when `evaluate_deform` produced a key. Coordinates / indices
    /// live in the caller-owned workspaces — they are valid until the
    /// next `evaluate_part` overwrites them, so the caller must consume
    /// them within the same loop iteration.
    pub deform_present: bool,
}

/// A float attribute other than position X / Y, which also take the root
/// part's origin offset and unit normalization ([`place`]). `evaluate_part`
/// writes a key's value with `set`, and `setup_pose_part` writes the setup value
/// it reads with `setup` the same way: one table, so the setup pose lands in the
/// space the keyed pose does.
struct FloatAttribute {
    idx: usize,
    set: fn(&mut runtime::framedata::ss::runtime::PartState, f32),
    setup: fn(&ssab::ssab::ss::format::InitialPartAttributes<'_>) -> f32,
    /// Negated for a Y-down target.
    flips: bool,
}

const FLOAT_ATTRIBUTES: [FloatAttribute; 23] = {
    use runtime::framedata::ss::runtime::PartState as S;
    [
        FloatAttribute { idx: IDX_ATTRIBUTEPOSZ, set: S::set_position_z, setup: |a| a.position_z(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTEROTX, set: S::set_rotation_x, setup: |a| a.rotation_x(), flips: true },
        FloatAttribute { idx: IDX_ATTRIBUTEROTY, set: S::set_rotation_y, setup: |a| a.rotation_y(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTEROTZ, set: S::set_rotation_z, setup: |a| a.rotation_z(), flips: true },
        FloatAttribute { idx: IDX_ATTRIBUTESCLX, set: S::set_scale_x, setup: |a| a.scale_x(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTESCLY, set: S::set_scale_y, setup: |a| a.scale_y(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTELOSCLX, set: S::set_local_scale_x, setup: |a| a.local_scale_x(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTELOSCLY, set: S::set_local_scale_y, setup: |a| a.local_scale_y(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTEALPHA, set: S::set_alpha, setup: |a| a.alpha(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTELOALPHA, set: S::set_local_alpha, setup: |a| a.local_alpha(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTEPIVOTX, set: S::set_pivot_x, setup: |a| a.pivot_x(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTEPIVOTY, set: S::set_pivot_y, setup: |a| a.pivot_y(), flips: true },
        FloatAttribute { idx: IDX_ATTRIBUTEANCHORX, set: S::set_anchor_x, setup: |a| a.anchor_x(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTEANCHORY, set: S::set_anchor_y, setup: |a| a.anchor_y(), flips: true },
        FloatAttribute { idx: IDX_ATTRIBUTESIZEX, set: S::set_size_x, setup: |a| a.size_x(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTESIZEY, set: S::set_size_y, setup: |a| a.size_y(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTEUVTX, set: S::set_uv_translation_x, setup: |a| a.uv_translation_x(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTEUVTY, set: S::set_uv_translation_y, setup: |a| a.uv_translation_y(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTEUVRZ, set: S::set_uv_rotation_z, setup: |a| a.uv_rotation_z(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTEUVSX, set: S::set_uv_scale_x, setup: |a| a.uv_scale_x(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTEUVSY, set: S::set_uv_scale_y, setup: |a| a.uv_scale_y(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTEBOUNDR, set: S::set_bounding_radius, setup: |a| a.bounding_radius(), flips: false },
        FloatAttribute { idx: IDX_ATTRIBUTEMASK, set: S::set_mask, setup: |a| a.mask(), flips: false },
    ]
};

/// A bool attribute, and how `evaluate_part` writes its key.
struct BoolAttribute {
    idx: usize,
    set: fn(&mut runtime::framedata::ss::runtime::PartState, bool),
}

const BOOL_ATTRIBUTES: [BoolAttribute; 5] = {
    use runtime::framedata::ss::runtime::PartState as S;
    [
        BoolAttribute { idx: IDX_ATTRIBUTEFLIPH, set: S::set_flip_h },
        BoolAttribute { idx: IDX_ATTRIBUTEFLIPV, set: S::set_flip_v },
        BoolAttribute { idx: IDX_ATTRIBUTEHIDE, set: S::set_hide },
        BoolAttribute { idx: IDX_ATTRIBUTEIMGFLIPH, set: S::set_img_flip_h },
        BoolAttribute { idx: IDX_ATTRIBUTEIMGFLIPV, set: S::set_img_flip_v },
    ]
};

/// A part's position in the target's space: a root part moved by the origin
/// offset, then both axes normalized when the unit asks for it. `y` comes in
/// already negated for a Y-down target.
#[allow(clippy::too_many_arguments)]
fn place(mut x: f32, mut y: f32, parent_index: i16, coordinate_system: SsCoordinateSystem, origin: SsOrigin, unit: SsUnit, canvas_width: f32, canvas_height: f32) -> (f32, f32) {
    if origin == SsOrigin::TopLeft && parent_index == -1 {
        x += canvas_width / 2.0;
        if coordinate_system == SsCoordinateSystem::YDown {
            y += canvas_height / 2.0;
        } else {
            y -= canvas_height / 2.0;
        }
    }
    if unit == SsUnit::Normalized && canvas_width != 0.0 && canvas_height != 0.0 {
        x /= canvas_width;
        y /= canvas_height;
    }
    (x, y)
}

/// The setup pose of one part, in the same space `evaluate_part` produces, or
/// the stack's floor lands in a different space than the poses folded onto it
/// (a track at `alpha < 1` would then drift toward a mis-transformed rest
/// pose). Both go through [`place`] and [`FLOAT_ATTRIBUTES`]; skew, the one
/// other attribute the mix blends, is written raw by both.
#[allow(clippy::too_many_arguments)]
pub(crate) fn setup_pose_part(
    a: &ssab::ssab::ss::format::InitialPartAttributes<'_>,
    parent_index: i16,
    coordinate_system: SsCoordinateSystem,
    origin: SsOrigin,
    unit: SsUnit,
    canvas_width: f32,
    canvas_height: f32,
) -> runtime::framedata::ss::runtime::PartState {
    let mut s = runtime::framedata::ss::runtime::PartState::default();
    let y_down = coordinate_system == SsCoordinateSystem::YDown;

    let (pos_x, pos_y) = place(a.position_x(), if y_down { -a.position_y() } else { a.position_y() }, parent_index, coordinate_system, origin, unit, canvas_width, canvas_height);
    s.set_position_x(pos_x);
    s.set_position_y(pos_y);
    for f in &FLOAT_ATTRIBUTES {
        let value = (f.setup)(a);
        (f.set)(&mut s, if f.flips && y_down { -value } else { value });
    }
    s.set_skew_x(a.skew_x());
    s.set_skew_y(a.skew_y());
    s
}

/// A part's state before any key is applied: the defaults `evaluate_part`
/// starts from when it builds a part base.
fn default_part_evaluation(cached_part: &CachedPartData) -> (runtime::framedata::ss::runtime::PartState, PartEvaluationResult) {
    let mut part_state = runtime::framedata::ss::runtime::PartState::default();

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
    // A part with no Hide keys at all is rendered as hidden by default; a Hide
    // run overrides it in `evaluate_part`. This is the standard SS authoring
    // model: a part has to opt in to being shown with a Hide=false key.
    part_state.set_hide(true);

    let result = PartEvaluationResult { state: runtime::framedata::ss::runtime::PartState::default(), cell: None, part_color: None, shader: None, vertex: None, deform_present: false };

    part_state.set_part_index(cached_part.part_index);
    (part_state, result)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn evaluate_part(
    cached_part: &CachedPartData,
    keys: &KeyColumns,
    frame_no: f32,
    part_cursors: &mut [u32],
    base: Option<PartBaseRef>,
    evaluate: u64,
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
    use runtime::framedata::ss::runtime::UpdateAttributeFlags as F;
    // `evaluate` picks the attributes to evaluate here: every keyed one when
    // building a part base, the ones that change during the animation when
    // starting from one.
    let on = |idx: usize| evaluate & (1u64 << idx) != 0;
    let from_base = base.is_some();
    let (mut part_state, mut update_flag, mut result) = match base {
        Some(base) => {
            let result = base.result();
            // Most parts change nothing during an animation: the base is the answer.
            if evaluate == 0 {
                return result;
            }
            (result.state, result.state.update_flag(), result)
        }
        None => {
            let (state, result) = default_part_evaluation(cached_part);
            (state, F::empty(), result)
        }
    };
    let y_down = coordinate_system == SsCoordinateSystem::YDown;

    if let Some(cell) = evaluated(cached_part, evaluate, part_cursors, IDX_ATTRIBUTECELL).and_then(|(run, cursor)| evaluate_cell(keys, &run, frame_no, cursor)) {
        result.cell = Some(cell);
        update_flag |= F::AttributeCell;
    }

    let mut pos_x = 0.0;
    if let Some(value) = evaluated(cached_part, evaluate, part_cursors, IDX_ATTRIBUTEPOSX).and_then(|(run, cursor)| evaluate_float(keys, &run, frame_no, cursor)) {
        pos_x = value;
        update_flag |= F::AttributePosX;
    }
    let mut pos_y = 0.0;
    if let Some(value) = evaluated(cached_part, evaluate, part_cursors, IDX_ATTRIBUTEPOSY).and_then(|(run, cursor)| evaluate_float(keys, &run, frame_no, cursor)) {
        pos_y = if y_down { -value } else { value };
        update_flag |= F::AttributePosY;
    }
    let (pos_x, pos_y) = place(pos_x, pos_y, cached_part.parent_index, coordinate_system, origin, unit, canvas_width, canvas_height);
    // From a part base, an axis that does not change keeps the base's value.
    if !from_base || on(IDX_ATTRIBUTEPOSX) {
        part_state.set_position_x(pos_x);
    }
    if !from_base || on(IDX_ATTRIBUTEPOSY) {
        part_state.set_position_y(pos_y);
    }

    for f in &FLOAT_ATTRIBUTES {
        if let Some(value) = evaluated(cached_part, evaluate, part_cursors, f.idx).and_then(|(run, cursor)| evaluate_float(keys, &run, frame_no, cursor)) {
            (f.set)(&mut part_state, if f.flips && y_down { -value } else { value });
            update_flag |= F::from_bits_retain(1 << f.idx);
        }
    }
    if let Some(value) = evaluated(cached_part, evaluate, part_cursors, IDX_ATTRIBUTEPRIO).and_then(|(run, cursor)| evaluate_int(keys, &run, frame_no, cursor)) {
        part_state.set_priority(value);
        update_flag |= F::AttributePrio;
    }
    for b in &BOOL_ATTRIBUTES {
        if let Some(value) = evaluated(cached_part, evaluate, part_cursors, b.idx).and_then(|(run, cursor)| evaluate_bool(keys, &run, frame_no, cursor)) {
            (b.set)(&mut part_state, value);
            update_flag |= F::from_bits_retain(1 << b.idx);
        }
    }

    if let Some(value) = evaluated(cached_part, evaluate, part_cursors, IDX_ATTRIBUTEPARTCOLOR).and_then(|(run, cursor)| evaluate_partcolor(keys, &run, frame_no, cursor)) {
        result.part_color = Some(value);
        update_flag |= F::AttributePartColor;
    }
    if let Some(value) = evaluated(cached_part, evaluate, part_cursors, IDX_ATTRIBUTESHADER).and_then(|(run, cursor)| evaluate_shader(keys, &run, frame_no, cursor)) {
        result.shader = Some(value);
        update_flag |= F::AttributeShader;
    }
    if let Some(value) = evaluated(cached_part, evaluate, part_cursors, IDX_ATTRIBUTEVERTEX).and_then(|(run, cursor)| evaluate_vertex(keys, &run, frame_no, cursor)) {
        // Raw deform Y is kept in the source (Y-Up) orientation here; the always-on
        // local-vertex calc consumes it as deform offsets and the result is flipped
        // to YDown only at the very end. The intermediate raw deform value is never
        // exposed publicly, so flipping/unflipping it would be wasted work.
        result.vertex = Some(value);
        update_flag |= F::AttributeVertex;
    }
    if let Some((skew_x, skew_y)) = evaluated(cached_part, evaluate, part_cursors, IDX_ATTRIBUTESKEW).and_then(|(run, cursor)| evaluate_skew(keys, &run, frame_no, cursor)) {
        // Skew is stored raw here (no coordinate-system flip). It is a
        // Vertex-deform-mode attribute, so the Y-down conversion happens where
        // the shear is applied to local vertices — the same authored-Y-up path
        // the 4-corner Vertex deform takes.
        part_state.set_skew_x(skew_x);
        part_state.set_skew_y(skew_y);
        update_flag |= F::AttributeSkew;
    }
    if let Some((run, cursor)) = evaluated(cached_part, evaluate, part_cursors, IDX_ATTRIBUTEDEFORM)
        && evaluate_deform(keys, &run, frame_no, cursor, start_workspace, end_workspace, out_workspace, deform_coord_workspace, deform_index_workspace)
    {
        // Deform offsets are an ssae animation attribute — Y-up native like
        // position / pivot / anchor — so they flip for a Y-down target, the
        // SAME condition as those attributes. The mesh rest, by contrast,
        // comes from the .ssce cell (`table_coord_x` / `table_coord_y`) and is
        // Y-down native, so it flips on the opposite (YUp) condition in
        // compute_mesh_vertices. rest and deform are deliberately mirror-imaged.
        if y_down {
            for v in deform_coord_workspace.iter_mut() {
                v.set_y(-v.y());
            }
        }
        result.deform_present = true;
        update_flag |= F::AttributeDeform;
    }
    if let Some(value) = evaluated(cached_part, evaluate, part_cursors, IDX_ATTRIBUTETEXTURE).and_then(|(run, cursor)| evaluate_texture(keys, &run, frame_no, cursor)) {
        part_state.set_texture(value);
        update_flag |= F::AttributeTexture;
    }

    part_state.set_update_flag(update_flag);
    result.state = part_state;
    result
}

/// Attribute `idx`'s key run and cursor, when the part keys it and `evaluate`
/// asks for it.
#[inline]
fn evaluated<'c>(cached_part: &CachedPartData, evaluate: u64, part_cursors: &'c mut [u32], idx: usize) -> Option<(KeyRun, &'c mut u32)> {
    if evaluate & cached_part.key_mask & 1 << idx == 0 {
        return None;
    }
    let slot = cursor_slot(cached_part.key_mask, idx);
    Some((cached_part.runs[slot], &mut part_cursors[slot]))
}

/// Pre-multiply one PartColor corner's alpha by the part's hierarchical alpha
/// (RGB and per-corner `rate` are preserved). Shared by keyframe decode
/// ([`extract_pose`]) and the Override Layer so both honor one alpha contract.
pub(crate) fn premultiply_part_color_corner(c: &crate::runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor, alpha: f32) -> crate::runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor {
    let rgba = c.rgba();
    let new_a = ((rgba.a() as f32 / 255.0) * alpha * 255.0).clamp(0.0, 255.0) as u8;
    crate::runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&crate::runtime::framedata::ss::runtime::U8Color::new(new_a, rgba.r(), rgba.g(), rgba.b()), c.rate())
}

/// Every part's base for an animation under one coordinate setup: its state
/// with the attributes that never change during the animation
/// ([`CachedPartData::constant_mask`]) already applied. `extract_pose` starts
/// each part from its base and evaluates only the rest.
pub struct PartBases {
    parts: Vec<PartBase>,
    /// The constant part color, shader and vertex deform of the few parts that
    /// have one, kept apart so the common part does not carry their room.
    extras: Vec<PartBaseExtras>,
}

#[derive(Clone, Copy)]
struct PartBase {
    state: runtime::framedata::ss::runtime::PartState,
    cell: Option<u32>,
    /// Index into `PartBases::extras`, or `u32::MAX`.
    extras: u32,
}

#[derive(Clone, Copy)]
struct PartBaseExtras {
    part_color: Option<runtime::framedata::ss::runtime::PartAttributePartColor>,
    shader: Option<runtime::framedata::ss::runtime::PartAttributeShader>,
    vertex: Option<runtime::framedata::ss::runtime::PartAttributeVertex>,
}

impl PartBases {
    #[allow(clippy::too_many_arguments)]
    pub fn build(cached_parts: &[CachedPartData], keys: &KeyColumns, start_frame: f32, coordinate_system: SsCoordinateSystem, origin: SsOrigin, unit: SsUnit, canvas_width: f32, canvas_height: f32) -> Self {
        // Constant attributes never search, and deform is never constant, so these stay as they are.
        let (mut cursors, mut start, mut end, mut out, mut coords, mut indices) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut bases = PartBases { parts: Vec::with_capacity(cached_parts.len()), extras: Vec::new() };
        for part in cached_parts {
            cursors.clear();
            cursors.resize(part.key_mask.count_ones() as usize, 0);
            let r = evaluate_part(part, keys, start_frame, &mut cursors, None, part.constant_mask, coordinate_system, origin, unit, canvas_width, canvas_height, &mut start, &mut end, &mut out, &mut coords, &mut indices);
            let extras = if r.part_color.is_some() || r.shader.is_some() || r.vertex.is_some() {
                bases.extras.push(PartBaseExtras { part_color: r.part_color, shader: r.shader, vertex: r.vertex });
                bases.extras.len() as u32 - 1
            } else {
                u32::MAX
            };
            bases.parts.push(PartBase { state: r.state, cell: r.cell, extras });
        }
        bases
    }

    /// Part `i`'s base, for `evaluate_part` to start from.
    pub fn get(&self, i: usize) -> Option<PartBaseRef<'_>> {
        let base = self.parts.get(i)?;
        Some(PartBaseRef { base, extras: self.extras.get(base.extras as usize) })
    }
}

/// One part's base, as [`PartBases::get`] hands it to `evaluate_part`.
#[derive(Clone, Copy)]
pub struct PartBaseRef<'b> {
    base: &'b PartBase,
    extras: Option<&'b PartBaseExtras>,
}

impl PartBaseRef<'_> {
    /// The part as its base leaves it: what `evaluate_part` returns for a part
    /// with nothing to evaluate.
    fn result(self) -> PartEvaluationResult {
        let extras = self.extras;
        PartEvaluationResult { state: self.base.state, cell: self.base.cell, part_color: extras.and_then(|e| e.part_color), shader: extras.and_then(|e| e.shader), vertex: extras.and_then(|e| e.vertex), deform_present: false }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn extract_pose<'a>(
    fb: &mut flatbuffers::FlatBufferBuilder<'a>,
    cached_parts: &[CachedPartData],
    keys: &KeyColumns<'a>,
    frame_no: f32,
    parts_vec: &mut Vec<runtime::framedata::ss::runtime::PartState>,
    cell_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeCell>,
    parts_color_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributePartColor>,
    shader_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeShader>,
    deform_vec: &mut Vec<flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>>,
    raw_vertex_deform: &mut [DeformSlot<RawVertexDeform>],
    cursors: &mut Vec<u32>,
    bases: Option<&PartBases>,
    cell_meta_per_part: &mut [f32],
    cell_texture_hashes: &mut [u32],
    cell_ids: &mut Vec<Option<u32>>,
    mesh_deform: &mut Vec<DeformSlot<PartDeformFrame>>,
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
        mesh_deform.resize_with(cached_parts.len(), || None);
    }
    for d in mesh_deform.iter_mut().flatten() {
        d.clear();
    }
    parts_vec.clear();
    cell_vec.clear();
    cell_ids.clear();
    parts_color_vec.clear();
    shader_vec.clear();
    deform_vec.clear();
    for slot in cell_meta_per_part.iter_mut() {
        *slot = 0.0;
    }
    for slot in cell_texture_hashes.iter_mut() {
        *slot = 0;
    }
    for slot in raw_vertex_deform.iter_mut().flatten() {
        slot.present = false;
    }

    inheritance_alphas_workspace.clear();
    inheritance_alphas_workspace.reserve(cached_parts.len());
    // Sized by `setup_animation`; a layer that last played another animation
    // gets a buffer of this one's size. The values are search hints, valid in
    // any state, so a resized buffer only costs the first frame a longer search.
    let cursor_len = cursor_count(cached_parts);
    if cursors.len() != cursor_len {
        cursors.clear();
        cursors.resize(cursor_len, 0);
    }

    for (i, part) in cached_parts.iter().enumerate() {
        let part_cursors = part.cursors(cursors);
        // Start from the part's base and evaluate only what changes during the
        // animation; without a base, evaluate every keyed attribute. Most parts
        // change nothing, and for them the base is the answer without the call.
        let base = bases.and_then(|b| b.get(i));
        let evaluate = if base.is_some() { part.key_mask & !part.constant_mask } else { part.key_mask };
        let mut result = match base {
            Some(base) if evaluate == 0 => base.result(),
            _ => evaluate_part(part, keys, frame_no, part_cursors, base, evaluate, coordinate_system, origin, unit, canvas_width, canvas_height, start_workspace, end_workspace, out_workspace, deform_coord_workspace, deform_index_workspace),
        };

        let cell = result.cell.map(|c| keys.cell(c));
        let cell_meta = cell.map(|c| c.meta).unwrap_or_default();
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

        // Hide is per-part at runtime: the converter bakes each part's HIDE
        // inheritance (`ineheritRates/HIDE`) into its own keys. The hierarchy is a
        // transform hierarchy, and hidden parts are routinely used as pure
        // transform anchors with visible children (e.g. a hidden hand part
        // anchoring a visible weapon). Only alpha inherits down the tree; `hide`
        // must not be propagated.

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
            **raw_vertex_deform[i].get_or_insert_with(Default::default) = RawVertexDeform { present: true, dx: [lt.x(), rt.x(), lb.x(), rb.x()], dy: [lt.y(), rt.y(), lb.y(), rb.y()] };
        }

        if let Some(cell) = cell {
            cell_vec.push(cell.cell);
            result.state.set_cell((cell_vec.len() - 1) as i16);
        }
        cell_ids.push(result.cell);
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
            let retained = mesh_deform[i].get_or_insert_with(Default::default);
            retained.indices.extend_from_slice(deform_index_workspace.as_slice());
            for c in deform_coord_workspace.iter() {
                retained.dx.push(c.x());
                retained.dy.push(c.y());
            }
        }
        parts_vec.push(result.state);
    }
}

/// Builds this frame's Nine-Slice geometry for every nines part — the runtime
/// side of `SsNines`, driven by the part's authored margins / fill mode and the
/// pose's size and Cell.
///
/// Positions land in the same local-space box the quad path produces (pivot,
/// image flip and Y-Up/Y-Down already applied), so a Player only multiplies
/// them by the world matrix. UVs are cellmap texture-pixel coordinates, and
/// `box_coords` carry each vertex's fraction inside the part box so per-corner
/// PartColor interpolates the same way `shape_vertex_box_coords` drives it.
///
/// A part flip mirrors the box coordinates rather than the UVs: a nine-slice
/// has asymmetric margins, so mirroring the sampling alone would squeeze the
/// far margin into the near slice instead of mirroring the drawn box.
pub fn compute_nines_geometry_for_parts(
    parts_vec: &[runtime::framedata::ss::runtime::PartState],
    cached_parts: &[CachedPartData],
    cell_meta_per_part: &[f32],
    raw_vertex_deform: &[DeformSlot<RawVertexDeform>],
    coordinate_system: SsCoordinateSystem,
    out: &mut NinesBuffers,
) {
    out.begin(parts_vec.len());

    for (i, state) in parts_vec.iter().enumerate() {
        out.vertex_offsets.push((out.box_coords.len() / 2) as u32);
        out.index_offsets.push(out.indices.len() as u32);

        let Some(nines) = cached_parts.get(i).and_then(|c| c.nines) else {
            continue;
        };
        let (w, h) = (state.size_x(), state.size_y());
        if state.cell() < 0 || w <= 0.0 || h <= 0.0 {
            continue;
        }

        let off = i * 6;
        let meta = |k: usize| cell_meta_per_part.get(off + k).copied().unwrap_or(0.0);
        let (cell_w, cell_h) = (meta(2), meta(3));
        let (rect_left, rect_top) = (meta(4), meta(5));
        if cell_w <= 0.0 || cell_h <= 0.0 {
            continue;
        }

        let start = out.box_coords.len() / 2;
        let index_start = out.indices.len();
        let (vertex_count, _) = crate::eval::geometry::emit_nines_geometry((w, h), (rect_left, rect_top, rect_left + cell_w, rect_top + cell_h), nines.margins, nines.fill_mode, &mut out.box_coords, &mut out.uvs, &mut out.indices);

        let mut pivot_y = state.pivot_y();
        if coordinate_system == SsCoordinateSystem::YDown {
            pivot_y = -pivot_y;
        }
        let (bx, by) = crate::eval::geometry::compute_local_vertices((w, h), (meta(0), meta(1)), (state.pivot_x(), pivot_y), (state.img_flip_h(), state.img_flip_v()), None);
        let (left, top, right, bottom) = (bx[0], by[0], bx[3], by[3]);

        let mirror_u = state.flip_h() != state.img_flip_h();
        let mirror_v = state.flip_v() != state.img_flip_v();
        let deform = vertex_deform_at(raw_vertex_deform, i);
        // Skew shares the Vertex-deform slot; applied as a shear about the pivot
        // origin in Y-up, before the Y-down flip, like the 4-corner deform.
        let skew = crate::eval::geometry::skew_local_tans(state.skew_x(), state.skew_y());

        for v in start..start + vertex_count as usize {
            let (mut fx, mut fy) = (out.box_coords[v * 2], out.box_coords[v * 2 + 1]);
            if mirror_u {
                fx = 1.0 - fx;
            }
            if mirror_v {
                fy = 1.0 - fy;
            }
            out.box_coords[v * 2] = fx;
            out.box_coords[v * 2 + 1] = fy;

            let mut x = left + fx * (right - left);
            let mut y = top + fy * (bottom - top);
            if let Some(d) = deform {
                let (dx, dy) = crate::eval::geometry::bilinear_box_deform(&d.dx, &d.dy, fx, fy);
                x += dx;
                y += dy;
            }
            if let Some((tan_x, tan_y)) = skew {
                let (nx, ny) = crate::eval::geometry::skew_point_yup(x, y, tan_x, tan_y);
                x = nx;
                y = ny;
            }
            if coordinate_system == SsCoordinateSystem::YDown {
                y = -y;
            }
            out.vertices.push(x);
            out.vertices.push(y);
        }

        // Mirroring one axis reverses each triangle's orientation; restore it
        // so Players that cull back faces still see the grid.
        if mirror_u != mirror_v {
            for tri in out.indices[index_start..].as_chunks_mut::<3>().0 {
                tri.swap(1, 2);
            }
        }
    }

    out.vertex_offsets.push((out.box_coords.len() / 2) as u32);
    out.index_offsets.push(out.indices.len() as u32);
}

pub fn compute_local_vertices_for_parts(
    parts_vec: &mut [runtime::framedata::ss::runtime::PartState],
    cell_meta_per_part: &[f32],
    raw_vertex_deform: &[DeformSlot<RawVertexDeform>],
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

        let deform = vertex_deform_at(raw_vertex_deform, i).map(|d| (d.dx, d.dy));

        let mut pivot_y = state.pivot_y();
        if coordinate_system == SsCoordinateSystem::YDown {
            pivot_y = -pivot_y;
        }

        let (mut vx, mut vy) = crate::eval::geometry::compute_local_vertices((state.size_x(), state.size_y()), pivot, (state.pivot_x(), pivot_y), (state.img_flip_h(), state.img_flip_v()), deform.as_ref().map(|(x, y)| (x, y)));

        // Skew: a Vertex-deform-mode shear about the pivot origin, applied in
        // the authored Y-up space before the Y-down flip below (mutually
        // exclusive with the 4-corner deform, so `vx`/`vy` are the plain box).
        if let Some((tan_x, tan_y)) = crate::eval::geometry::skew_local_tans(state.skew_x(), state.skew_y()) {
            for k in 0..vx.len() {
                let (nx, ny) = crate::eval::geometry::skew_point_yup(vx[k], vy[k], tan_x, tan_y);
                vx[k] = nx;
                vy[k] = ny;
            }
        }

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
    if disc == P::PartTypeNines {
        // Same story as Mesh: the per-part grids are concatenated in the Nines
        // SoA buffers, so consecutive nines parts merge into one batch.
        return Some((K::Nines, true));
    }
    if disc == P::PartTypeText {
        // Glyph layout and rasterisation belong to the Player, so the batch
        // carries neither geometry nor a texture — it exists to give the part
        // its slot in the draw order. Not batchable: the runtime cannot key the
        // font a part draws with, so merging two text parts would hide a font
        // change inside one batch.
        return Some((K::Text, false));
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
    cached_parts: &[CachedPartData],
    parts_vec: &[runtime::framedata::ss::runtime::PartState],
    cell_texture_hashes: &[u32],
    nines: &NinesBuffers,
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

        // A text part authored as a pure mask is dropped outright. The runtime
        // produces no glyphs, so it can neither write coverage nor be honoured
        // as a mask — and surfacing it as an ordinary batch would draw text the
        // author meant to be invisible.
        if kind == DrawBatchKind::Text && cached.is_mask_contributor {
            continue;
        }

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
        let cell_texture_hash = cell_texture_hashes.get(part_idx).copied().unwrap_or(0);
        // Both kinds draw the part's Cell, so without one there is nothing to
        // batch — a nines part also has an empty geometry range in that case.
        // Culled on the *cell* hash: a TCHG override names a texture but never gives a
        // cell-less part geometry to draw, so it must not resurrect one here.
        let cell_backed_no_cell = matches!(kind, DrawBatchKind::Normal | DrawBatchKind::Nines) && cell_texture_hash == 0;
        // An authored TextureChange (TCHG) rebinds this part's texture, so it has to take part
        // in the batch key — otherwise parts sharing a cellmap merge into one batch and the
        // override cannot be bound. `texture` is 0 both by schema default and for the authored
        // "no override" (empty TextureName), which falls back to the cell's own texture. Only a
        // part that resolved a cell honours the override: a textureless kind (`Shape`, `Text`)
        // keeps `texture_hash` at 0, and a cell-less part is culled below anyway.
        let texture_override = part_state.texture();
        let texture_hash = if cell_texture_hash != 0 && part_state.update_flag().contains(UpdateAttributeFlags::AttributeTexture) && texture_override != 0 { texture_override } else { cell_texture_hash };
        // Masks honour Hide too: a hidden mask must NOT contribute coverage
        // (SS mask opt-out is mask-value==0 OR Hide==true) — otherwise a hidden
        // mask would wrongly clip its targets once the player rasterises mask
        // coverage.
        let drawn_as_self = !part_state.hide() && !cell_backed_no_cell;

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
        } else if kind == DrawBatchKind::Nines {
            // Known exactly: the grid was built before the draw plan.
            nines.counts(part_idx)
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
    cached_parts: &[CachedPartData],
    z_order: &mut crate::eval::z_order::ZOrderTable,
    parts_vec: &mut [runtime::framedata::ss::runtime::PartState],
    cell_vec: &[crate::runtime::framedata::ss::runtime::PartAttributeCell],
    parts_color_vec: &[crate::runtime::framedata::ss::runtime::PartAttributePartColor],
    shader_vec: &[crate::runtime::framedata::ss::runtime::PartAttributeShader],
    vertex_vec: &mut Vec<crate::runtime::framedata::ss::runtime::PartAttributeVertex>,
    deform_vec: &[flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>],
    raw_vertex_deform: &[DeformSlot<RawVertexDeform>],
    cell_meta_per_part: &[f32],
    cell_texture_hashes: &[u32],
    local_vertices: &mut [f32],
    nines: &mut NinesBuffers,
    coordinate_system: SsCoordinateSystem,
    render_mode: crate::eval::RenderMode,
    draw_order_workspace: &mut Vec<u16>,
    batches_workspace: &mut Vec<runtime::framedata::ss::runtime::DrawBatch>,
    mask_indices_workspace: &mut Vec<u16>,
) -> flatbuffers::WIPOffset<runtime::framedata::ss::runtime::FrameData<'a>> {
    compute_local_vertices_for_parts(parts_vec, cell_meta_per_part, raw_vertex_deform, vertex_vec, local_vertices, coordinate_system);
    // Before the draw plan: its Nines batches carry the exact vertex / index counts.
    compute_nines_geometry_for_parts(parts_vec, cached_parts, cell_meta_per_part, raw_vertex_deform, coordinate_system, nines);

    let (cells_off, parts_color_off, shaders_off, vertices_off, deforms_off) = if render_mode == crate::eval::RenderMode::Direct {
        let pc = (!parts_color_vec.is_empty()).then(|| fb.create_vector(parts_color_vec));
        let s = (!shader_vec.is_empty()).then(|| fb.create_vector(shader_vec));
        (None, pc, s, None, None)
    } else {
        serialize_pose_to_fb(fb, cell_vec, parts_color_vec, shader_vec, vertex_vec, deform_vec)
    };

    let mut args = runtime::framedata::ss::runtime::FrameDataArgs { cells: cells_off, parts_color: parts_color_off, shaders: shaders_off, vertices: vertices_off, deforms: deforms_off, ..Default::default() };
    args.parts = Some(fb.create_vector(parts_vec));

    // The draw order, batches and mask indices need the finished `parts_vec` and
    // `cell_texture_hashes`, and must be in `args` before `FrameData::create`.
    z_order.update(parts_vec);
    let (draw_order_off, draw_batches_off, mask_indices_off) = build_draw_order_and_batches(fb, z_order.indices(), cached_parts, parts_vec, cell_texture_hashes, nines, draw_order_workspace, batches_workspace, mask_indices_workspace);
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
        assert_eq!(NUM_PART_ATTRIBUTES, UpdateAttributeFlags::all().bits().count_ones() as usize);
    }

    #[test]
    fn test_attribute_flags_no_gaps() {
        let all_bits = UpdateAttributeFlags::all().bits();
        let num_bits = all_bits.count_ones();
        // Check for gaps: If there are no gaps, the value of all_bits should be (1 << num_bits) - 1.
        // The bit positions are the `IDX_ATTRIBUTE*` constants, which place each
        // attribute's bit in a part's `key_mask` and so its key cursor.
        let expected_mask = (1u64 << num_bits) - 1;
        assert_eq!(all_bits, expected_mask, "UpdateAttributeFlags must not have gaps in bit values to avoid array out-of-bounds when using bit positions as indices.");
    }

    /// The setup pose is the floor the mix folds tracks onto, so it has to land
    /// in the space `evaluate_part` produces: a part keyed, once, with its setup
    /// values evaluates to its setup pose, under every coordinate setup.
    #[test]
    fn setup_pose_matches_a_pose_keyed_with_the_setup_values() {
        use crate::eval::{SsCoordinateSystem, SsOrigin, SsUnit};
        use flatbuffers::{FlatBufferBuilder, WIPOffset};
        use ssab::ssab::ss::format::{
            InitialPartAttributes, InitialPartAttributesArgs, InterpolationType, KeyCurve, Keyframes, KeyframesArgs, PartAnimationData, PartAnimationDataArgs, PartAttributeCell, PartAttributeDeform, PartAttributePartColor, PartAttributeShader,
            PartAttributeVertex, Vec2, Vec2Args,
        };

        // Every attribute the mix blends, a distinct value each.
        let setup = InitialPartAttributesArgs {
            position_x: 3.0,
            position_y: 5.0,
            position_z: 7.0,
            rotation_x: 11.0,
            rotation_y: 13.0,
            rotation_z: 17.0,
            scale_x: 1.5,
            scale_y: 2.5,
            local_scale_x: 0.5,
            local_scale_y: 0.75,
            alpha: 0.25,
            local_alpha: 0.125,
            pivot_x: 0.1,
            pivot_y: 0.2,
            anchor_x: 0.3,
            anchor_y: 0.4,
            size_x: 19.0,
            size_y: 23.0,
            uv_translation_x: 0.01,
            uv_translation_y: 0.02,
            uv_rotation_z: 29.0,
            uv_scale_x: 1.25,
            uv_scale_y: 1.75,
            bounding_radius: 31.0,
            mask: 37.0,
            skew_x: 41.0,
            skew_y: -43.0,
            ..Default::default()
        };

        // The same values as one key each, at frame 0.
        let mut values = Vec::new();
        let mut part = PartAnimationDataArgs::default();
        let mut key = |run: &mut u32, value: f32| {
            *run = values.len() as u32;
            values.push(value.to_bits());
        };
        key(&mut part.position_x_keys, setup.position_x);
        key(&mut part.position_y_keys, setup.position_y);
        key(&mut part.position_z_keys, setup.position_z);
        key(&mut part.rotation_x_keys, setup.rotation_x);
        key(&mut part.rotation_y_keys, setup.rotation_y);
        key(&mut part.rotation_z_keys, setup.rotation_z);
        key(&mut part.scale_x_keys, setup.scale_x);
        key(&mut part.scale_y_keys, setup.scale_y);
        key(&mut part.local_scale_x_keys, setup.local_scale_x);
        key(&mut part.local_scale_y_keys, setup.local_scale_y);
        key(&mut part.alpha_keys, setup.alpha);
        key(&mut part.local_alpha_keys, setup.local_alpha);
        key(&mut part.pivot_x_keys, setup.pivot_x);
        key(&mut part.pivot_y_keys, setup.pivot_y);
        key(&mut part.anchor_x_keys, setup.anchor_x);
        key(&mut part.anchor_y_keys, setup.anchor_y);
        key(&mut part.size_x_keys, setup.size_x);
        key(&mut part.size_y_keys, setup.size_y);
        key(&mut part.uv_translation_x_keys, setup.uv_translation_x);
        key(&mut part.uv_translation_y_keys, setup.uv_translation_y);
        key(&mut part.uv_rotation_z_keys, setup.uv_rotation_z);
        key(&mut part.uv_scale_x_keys, setup.uv_scale_x);
        key(&mut part.uv_scale_y_keys, setup.uv_scale_y);
        key(&mut part.bounding_radius_keys, setup.bounding_radius);
        key(&mut part.mask_keys, setup.mask);
        // Skew's key is an index into `skews`, which holds the one pair.
        part.skew_keys = values.len() as u32;
        values.push(0);

        let mut fb = FlatBufferBuilder::new();
        let skew = Vec2::create(&mut fb, &Vec2Args { v1: setup.skew_x, v2: setup.skew_y });
        let n = values.len();
        let args = KeyframesArgs {
            runs: Some(fb.create_vector(&(0..=n as u32).collect::<Vec<_>>())),
            frame: Some(fb.create_vector(&alloc::vec![0u16; n])),
            interpolation: Some(fb.create_vector(&alloc::vec![InterpolationType::None; n])),
            curve: Some(fb.create_vector(&alloc::vec![ssab::NO_CURVE; n])),
            value: Some(fb.create_vector(&values)),
            curves: Some(fb.create_vector::<KeyCurve>(&[])),
            easing_rates: Some(fb.create_vector::<f32>(&[])),
            cells: Some(fb.create_vector::<WIPOffset<PartAttributeCell>>(&[])),
            part_colors: Some(fb.create_vector::<WIPOffset<PartAttributePartColor>>(&[])),
            shaders: Some(fb.create_vector::<WIPOffset<PartAttributeShader>>(&[])),
            vertices: Some(fb.create_vector::<WIPOffset<PartAttributeVertex>>(&[])),
            skews: Some(fb.create_vector(&[skew])),
            deforms: Some(fb.create_vector::<WIPOffset<PartAttributeDeform>>(&[])),
        };
        let root = Keyframes::create(&mut fb, &args);
        fb.finish(root, None);
        let keys = KeyColumns::new(flatbuffers::root::<Keyframes>(fb.finished_data()).unwrap(), None);

        let mut part_fb = FlatBufferBuilder::new();
        let root = PartAnimationData::create(&mut part_fb, &part);
        part_fb.finish(root, None);
        let part = flatbuffers::root::<PartAnimationData>(part_fb.finished_data()).unwrap();
        let cached = CachedPartData::new(part, &keys, 0, -1, 0, None, None, None, None, ssab::ssab::ss::format::BlendType::Mix, ssab::ssab::ss::format::PartType::PartTypeNormal, false, false, 0);

        let mut setup_fb = FlatBufferBuilder::new();
        let root = InitialPartAttributes::create(&mut setup_fb, &setup);
        setup_fb.finish(root, None);
        let setup = flatbuffers::root::<InitialPartAttributes>(setup_fb.finished_data()).unwrap();

        type Read = fn(&runtime::framedata::ss::runtime::PartState) -> f32;
        let mixed: [(&str, Read); 27] = [
            ("position_x", |s| s.position_x()),
            ("position_y", |s| s.position_y()),
            ("position_z", |s| s.position_z()),
            ("rotation_x", |s| s.rotation_x()),
            ("rotation_y", |s| s.rotation_y()),
            ("rotation_z", |s| s.rotation_z()),
            ("scale_x", |s| s.scale_x()),
            ("scale_y", |s| s.scale_y()),
            ("local_scale_x", |s| s.local_scale_x()),
            ("local_scale_y", |s| s.local_scale_y()),
            ("alpha", |s| s.alpha()),
            ("local_alpha", |s| s.local_alpha()),
            ("pivot_x", |s| s.pivot_x()),
            ("pivot_y", |s| s.pivot_y()),
            ("anchor_x", |s| s.anchor_x()),
            ("anchor_y", |s| s.anchor_y()),
            ("size_x", |s| s.size_x()),
            ("size_y", |s| s.size_y()),
            ("uv_translation_x", |s| s.uv_translation_x()),
            ("uv_translation_y", |s| s.uv_translation_y()),
            ("uv_rotation_z", |s| s.uv_rotation_z()),
            ("uv_scale_x", |s| s.uv_scale_x()),
            ("uv_scale_y", |s| s.uv_scale_y()),
            ("bounding_radius", |s| s.bounding_radius()),
            ("mask", |s| s.mask()),
            ("skew_x", |s| s.skew_x()),
            ("skew_y", |s| s.skew_y()),
        ];
        let (canvas_width, canvas_height) = (200.0, 100.0);
        for coordinate_system in [SsCoordinateSystem::YUp, SsCoordinateSystem::YDown] {
            for origin in [SsOrigin::Center, SsOrigin::TopLeft] {
                for unit in [SsUnit::Pixel, SsUnit::Normalized] {
                    let floor = setup_pose_part(&setup, -1, coordinate_system, origin, unit, canvas_width, canvas_height);
                    let mut cursors = alloc::vec![0; cached.key_mask.count_ones() as usize];
                    let (mut a, mut b, mut c, mut d, mut e) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
                    let keyed = evaluate_part(&cached, &keys, 0.0, &mut cursors, None, cached.key_mask, coordinate_system, origin, unit, canvas_width, canvas_height, &mut a, &mut b, &mut c, &mut d, &mut e).state;
                    for (name, read) in mixed {
                        assert_eq!(read(&floor), read(&keyed), "{name} under {coordinate_system:?} / {origin:?} / {unit:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn test_part_evaluation_result_init() {
        let mut part_state = runtime::framedata::ss::runtime::PartState::default();
        part_state.set_part_index(5);

        let result = PartEvaluationResult { state: part_state, cell: None, part_color: None, shader: None, vertex: None, deform_present: false };

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
        let raw_vertex_deform: alloc::vec::Vec<DeformSlot<RawVertexDeform>> = vec![None];
        let mut vertex_vec: Vec<runtime::framedata::ss::runtime::PartAttributeVertex> = Vec::new();
        let mut local_vertices = vec![0.0_f32; LOCAL_VERTICES_STRIDE];

        compute_local_vertices_for_parts(&mut parts_vec, &cell_meta_per_part, &raw_vertex_deform, &mut vertex_vec, &mut local_vertices, crate::eval::SsCoordinateSystem::YUp);

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
        let raw_vertex_deform: alloc::vec::Vec<DeformSlot<RawVertexDeform>> = vec![None];
        let mut vertex_vec: Vec<runtime::framedata::ss::runtime::PartAttributeVertex> = Vec::new();
        let mut local_vertices = vec![0.0_f32; LOCAL_VERTICES_STRIDE];

        compute_local_vertices_for_parts(&mut parts_vec, &cell_meta_per_part, &raw_vertex_deform, &mut vertex_vec, &mut local_vertices, crate::eval::SsCoordinateSystem::YUp);

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
        let raw_vertex_deform = vec![Some(alloc::boxed::Box::new(RawVertexDeform { present: true, dx: [1.0, 2.0, 3.0, 4.0], dy: [10.0, 20.0, 30.0, 40.0] }))];
        let mut vertex_vec: Vec<runtime::framedata::ss::runtime::PartAttributeVertex> = Vec::new();
        let mut local_vertices = vec![0.0_f32; LOCAL_VERTICES_STRIDE];

        compute_local_vertices_for_parts(&mut parts_vec, &cell_meta_per_part, &raw_vertex_deform, &mut vertex_vec, &mut local_vertices, crate::eval::SsCoordinateSystem::YUp);

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
        let raw_vertex_deform: alloc::vec::Vec<DeformSlot<RawVertexDeform>> = vec![None];
        let mut vertex_vec: Vec<runtime::framedata::ss::runtime::PartAttributeVertex> = Vec::new();
        let mut local_vertices = vec![0.0_f32; LOCAL_VERTICES_STRIDE];

        compute_local_vertices_for_parts(&mut parts_vec, &cell_meta_per_part, &raw_vertex_deform, &mut vertex_vec, &mut local_vertices, crate::eval::SsCoordinateSystem::YDown);

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
        // Stage B applies a slot's offsets only when `present` is set, so a slot
        // made with `Default` must read as no deform.
        let d = RawVertexDeform::default();
        assert!(!d.present);
        assert_eq!(d.dx, [0.0; 4]);
        assert_eq!(d.dy, [0.0; 4]);
    }

    #[test]
    fn stage_b_center_is_gated_by_update_flag() {
        // Cell pivot (0.1, 0.2) produces a non-symmetric box whose Varignon center is (-1, -4).
        let cell_meta_per_part = vec![0.1_f32, 0.2, 10.0, 20.0, 0.0, 0.0];
        let raw_vertex_deform: alloc::vec::Vec<DeformSlot<RawVertexDeform>> = vec![None];

        // Case A: only AttributeCell — center field in vertex_vec stays at default (0, 0),
        // but local_vertices always carries the computed center.
        {
            let mut parts_vec = vec![make_stage_b_state(0, (10.0, 20.0), (0.0, 0.0), UpdateAttributeFlags::AttributeCell)];
            let mut vertex_vec: Vec<runtime::framedata::ss::runtime::PartAttributeVertex> = Vec::new();
            let mut local_vertices = vec![0.0_f32; LOCAL_VERTICES_STRIDE];
            compute_local_vertices_for_parts(&mut parts_vec, &cell_meta_per_part, &raw_vertex_deform, &mut vertex_vec, &mut local_vertices, crate::eval::SsCoordinateSystem::YUp);
            assert_eq!((vertex_vec[0].center().x(), vertex_vec[0].center().y()), (0.0, 0.0));
            assert_eq!((local_vertices[8], local_vertices[9]), (-1.0, -4.0));
        }

        // Case B: AttributeVertex also set — center is populated with the computed value.
        {
            let mut parts_vec = vec![make_stage_b_state(0, (10.0, 20.0), (0.0, 0.0), UpdateAttributeFlags::AttributeCell | UpdateAttributeFlags::AttributeVertex)];
            let mut vertex_vec: Vec<runtime::framedata::ss::runtime::PartAttributeVertex> = Vec::new();
            let mut local_vertices = vec![0.0_f32; LOCAL_VERTICES_STRIDE];
            compute_local_vertices_for_parts(&mut parts_vec, &cell_meta_per_part, &raw_vertex_deform, &mut vertex_vec, &mut local_vertices, crate::eval::SsCoordinateSystem::YUp);
            assert_eq!((vertex_vec[0].center().x(), vertex_vec[0].center().y()), (-1.0, -4.0));
        }
    }

    /// One nines part with 10 px margins over a 64 px cell.
    fn make_nines_part(fill_mode: crate::eval::geometry::NinesFillMode) -> Vec<CachedPartData> {
        let mut part = cached_part_stub(0, -1, ssab::ssab::ss::format::PartType::PartTypeNines, false);
        part.nines = Some(NinesPartData { margins: (10.0, 10.0, 10.0, 10.0), fill_mode });
        vec![part]
    }

    /// `[pivot_x, pivot_y, size_w, size_h, rect_left, rect_top]` for a 64x64
    /// cell at the texture's origin.
    fn nines_cell_meta() -> Vec<f32> {
        vec![0.0, 0.0, 64.0, 64.0, 0.0, 0.0]
    }

    fn run_nines(parts_vec: &[runtime::framedata::ss::runtime::PartState], cached: &[CachedPartData], deform: &[RawVertexDeform], coordinate_system: crate::eval::SsCoordinateSystem) -> NinesBuffers {
        let mut out = NinesBuffers::default();
        let deform: Vec<DeformSlot<RawVertexDeform>> = deform.iter().map(|d| Some(alloc::boxed::Box::new(*d))).collect();
        compute_nines_geometry_for_parts(parts_vec, cached, &nines_cell_meta(), &deform, coordinate_system, &mut out);
        out
    }

    #[test]
    fn nines_stretch_fills_the_part_box() {
        let parts_vec = vec![make_stage_b_state(0, (100.0, 100.0), (0.0, 0.0), UpdateAttributeFlags::AttributeCell)];
        let out = run_nines(&parts_vec, &make_nines_part(crate::eval::geometry::NinesFillMode::Stretch), &[RawVertexDeform::default()], crate::eval::SsCoordinateSystem::YUp);

        assert_eq!(out.vertex_offsets, vec![0, 16]);
        assert_eq!(out.index_offsets, vec![0, 54]);
        assert_eq!(out.counts(0), (16, 54));

        // The grid's corners are the part's box corners, Y-Up and pivot-centred.
        assert_eq!((out.vertices[0], out.vertices[1]), (-50.0, 50.0));
        assert_eq!((out.vertices[30], out.vertices[31]), (50.0, -50.0));
        // Box coordinates run 0 -> 1 across it, UVs stay in cell pixels.
        assert_eq!((out.box_coords[0], out.box_coords[1]), (0.0, 0.0));
        assert_eq!((out.box_coords[30], out.box_coords[31]), (1.0, 1.0));
        assert_eq!((out.uvs[0], out.uvs[1]), (0.0, 0.0));
        assert_eq!((out.uvs[30], out.uvs[31]), (64.0, 64.0));
        // The slice line sits one margin in, in both spaces.
        assert_eq!((out.vertices[10], out.vertices[11]), (-40.0, 40.0));
        assert_eq!((out.uvs[10], out.uvs[11]), (10.0, 10.0));
    }

    #[test]
    fn nines_tile_repeats_the_middle_slice() {
        // A 44 px middle slice covers the part's 80 px middle in 2 repeats.
        let parts_vec = vec![make_stage_b_state(0, (100.0, 100.0), (0.0, 0.0), UpdateAttributeFlags::AttributeCell)];
        let out = run_nines(&parts_vec, &make_nines_part(crate::eval::geometry::NinesFillMode::Tile), &[RawVertexDeform::default()], crate::eval::SsCoordinateSystem::YUp);

        let quads = 4 + 2 * 2 + 2 * 2 + 2 * 2;
        assert_eq!(out.counts(0), (quads * 4, quads * 6));
        // Independent quads, so the vertex and index buffers stay in lockstep.
        assert_eq!(out.vertices.len(), out.box_coords.len());
        assert_eq!(out.uvs.len(), out.box_coords.len());
    }

    #[test]
    fn nines_flip_mirrors_the_box_and_keeps_winding() {
        let mut state = make_stage_b_state(0, (100.0, 100.0), (0.0, 0.0), UpdateAttributeFlags::AttributeCell);
        state.set_flip_h(true);
        let out = run_nines(&[state], &make_nines_part(crate::eval::geometry::NinesFillMode::Stretch), &[RawVertexDeform::default()], crate::eval::SsCoordinateSystem::YUp);

        // The first grid vertex still samples the cell's top-left, but it is now
        // drawn at the box's right edge — mirroring the box, not the sampling,
        // is what keeps asymmetric margins mirrored instead of squeezed.
        assert_eq!((out.uvs[0], out.uvs[1]), (0.0, 0.0));
        assert_eq!((out.box_coords[0], out.box_coords[1]), (1.0, 0.0));
        assert_eq!((out.vertices[0], out.vertices[1]), (50.0, 50.0));

        // Mirroring a single axis would reverse each triangle, so
        // `compute_nines_geometry_for_parts` swaps the last two indices back.
        assert_eq!(&out.indices[0..6], &[0, 4, 1, 4, 5, 1]);

        // Mirroring both axes cancels out and leaves the original winding.
        let mut state = make_stage_b_state(0, (100.0, 100.0), (0.0, 0.0), UpdateAttributeFlags::AttributeCell);
        state.set_flip_h(true);
        state.set_flip_v(true);
        let out = run_nines(&[state], &make_nines_part(crate::eval::geometry::NinesFillMode::Stretch), &[RawVertexDeform::default()], crate::eval::SsCoordinateSystem::YUp);
        assert_eq!((out.box_coords[0], out.box_coords[1]), (1.0, 1.0));
        assert_eq!(&out.indices[0..6], &[0, 1, 4, 4, 1, 5]);
    }

    #[test]
    fn nines_spreads_vertex_deform_across_the_grid() {
        let parts_vec = vec![make_stage_b_state(0, (100.0, 100.0), (0.0, 0.0), UpdateAttributeFlags::AttributeCell | UpdateAttributeFlags::AttributeVertex)];
        // Move the LT corner 10 px right; the offset fades out across the grid.
        let deform = vec![RawVertexDeform { present: true, dx: [10.0, 0.0, 0.0, 0.0], dy: [0.0; 4] }];
        let out = run_nines(&parts_vec, &make_nines_part(crate::eval::geometry::NinesFillMode::Stretch), &deform, crate::eval::SsCoordinateSystem::YUp);

        assert_eq!(out.vertices[0], -40.0); // LT takes the whole offset
        assert_eq!(out.vertices[30], 50.0); // RB takes none of it
        // Interior vertex (0.1, 0.1): 10 * 0.9 * 0.9.
        assert!((out.vertices[10] - (-40.0 + 8.1)).abs() < 1e-4, "got {}", out.vertices[10]);
    }

    #[test]
    fn nines_honours_y_down() {
        let parts_vec = vec![make_stage_b_state(0, (100.0, 100.0), (0.0, 0.0), UpdateAttributeFlags::AttributeCell)];
        let out = run_nines(&parts_vec, &make_nines_part(crate::eval::geometry::NinesFillMode::Stretch), &[RawVertexDeform::default()], crate::eval::SsCoordinateSystem::YDown);

        // Same box coordinate, mirrored Y — the grid's first vertex is the box's
        // top-left in both systems.
        assert_eq!((out.box_coords[0], out.box_coords[1]), (0.0, 0.0));
        assert_eq!((out.vertices[0], out.vertices[1]), (-50.0, -50.0));
        assert_eq!((out.vertices[30], out.vertices[31]), (50.0, 50.0));
    }

    #[test]
    fn nines_geometry_is_empty_without_drawable_input() {
        let stretch = crate::eval::geometry::NinesFillMode::Stretch;
        let deform = [RawVertexDeform::default()];

        // Not a nines part.
        let normal = vec![cached_part_stub(0, -1, ssab::ssab::ss::format::PartType::PartTypeNormal, false)];
        let parts_vec = vec![make_stage_b_state(0, (100.0, 100.0), (0.0, 0.0), UpdateAttributeFlags::AttributeCell)];
        let out = run_nines(&parts_vec, &normal, &deform, crate::eval::SsCoordinateSystem::YUp);
        assert_eq!(out.counts(0), (0, 0));
        assert_eq!(out.vertex_offsets, vec![0, 0]);

        // Nines part with no Cell resolved this frame.
        let parts_vec = vec![make_stage_b_state(-1, (100.0, 100.0), (0.0, 0.0), UpdateAttributeFlags::empty())];
        let out = run_nines(&parts_vec, &make_nines_part(stretch), &deform, crate::eval::SsCoordinateSystem::YUp);
        assert_eq!(out.counts(0), (0, 0));

        // Nines part collapsed to zero size.
        let parts_vec = vec![make_stage_b_state(0, (0.0, 100.0), (0.0, 0.0), UpdateAttributeFlags::AttributeCell)];
        let out = run_nines(&parts_vec, &make_nines_part(stretch), &deform, crate::eval::SsCoordinateSystem::YUp);
        assert_eq!(out.counts(0), (0, 0));
        assert!(out.vertices.is_empty() && out.indices.is_empty());
    }

    #[test]
    fn text_parts_take_a_draw_slot_but_never_a_mask_slot() {
        // Three text parts; the middle one is authored as a pure mask.
        let cached =
            vec![cached_part_stub(0, -1, ssab::ssab::ss::format::PartType::PartTypeText, false), cached_part_stub(1, -1, ssab::ssab::ss::format::PartType::PartTypeText, true), cached_part_stub(2, -1, ssab::ssab::ss::format::PartType::PartTypeText, false)];
        let parts_vec: Vec<_> = (0..3).map(|_| make_stage_b_state(-1, (10.0, 10.0), (0.0, 0.0), UpdateAttributeFlags::empty())).collect();

        let mut fb = flatbuffers::FlatBufferBuilder::new();
        let (mut draw_order, mut batches, mut mask_indices) = (Vec::new(), Vec::new(), Vec::new());
        build_draw_order_and_batches(&mut fb, &[0, 1, 2], &cached, &parts_vec, &[0, 0, 0], &NinesBuffers::default(), &mut draw_order, &mut batches, &mut mask_indices);

        // Text takes part in the draw order, but a text mask is left out
        // entirely, since the runtime has no glyphs to write coverage with.
        assert_eq!(draw_order, vec![0, 2]);
        assert!(mask_indices.is_empty(), "a text mask must not claim coverage it cannot write");

        // One batch per text part: the runtime cannot key the font, so merging
        // would hide a font change inside a single batch.
        assert_eq!(batches.len(), 2);
        for (batch, rank) in batches.iter().zip([0, 1]) {
            assert_eq!(batch.kind(), runtime::framedata::ss::runtime::DrawBatchKind::Text);
            assert_eq!(batch.start_rank(), rank);
            assert_eq!(batch.count(), 1);
            // The Player owns the glyphs, so the batch carries no geometry and
            // no texture — the font comes from the ssab.
            assert_eq!((batch.vertex_count(), batch.index_count()), (0, 0));
            assert_eq!(batch.texture_hash(), 0);
        }
    }

    #[test]
    fn hidden_text_parts_stay_out_of_the_draw_plan() {
        let cached = vec![cached_part_stub(0, -1, ssab::ssab::ss::format::PartType::PartTypeText, false)];
        let mut state = make_stage_b_state(-1, (10.0, 10.0), (0.0, 0.0), UpdateAttributeFlags::empty());
        state.set_hide(true);
        let mut fb = flatbuffers::FlatBufferBuilder::new();
        let (mut draw_order, mut batches, mut mask_indices) = (Vec::new(), Vec::new(), Vec::new());
        build_draw_order_and_batches(&mut fb, &[0], &cached, &[state], &[0], &NinesBuffers::default(), &mut draw_order, &mut batches, &mut mask_indices);

        assert!(draw_order.is_empty() && batches.is_empty() && mask_indices.is_empty());
    }
}
