//! Per-part runtime **Override Layer**
//!
//! Applies per-part runtime overrides (Visibility now; Color/Cell added in later
//! stages) on the *resolved* pose at a single seam in `Context::get_frame_data`
//! — after the source mix, before `compose_frame_data`. Overriding here (rather
//! than during per-source `extract_pose`) is what lets an override win over both
//! keyframe values and blend results.
//!
//! The override table is a dense `Vec<Option<PartOverride>>` on `InstanceState`,
//! indexed by `part_index` (O(1), zero per-frame alloc, `no_std` — no `HashMap`).

use alloc::vec::Vec;

use crate::core::framedata::{CachedPartData, ResolvedCellMeta, premultiply_part_color_corner};
use crate::runtime::framedata::ss::runtime::{BlendType, PartAttributeCell, PartAttributePartColor, PartState, SsAttributePartColorKeyValueColor, U8Rect, UpdateAttributeFlags};

/// Keyframe-vs-override precedence for the attributes that conflict with the
/// animation (Color and Cell). Visibility has no priority mode.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PriorityMode {
    /// `NON`: the keyframe reclaims the attribute whenever the animation drives
    /// it this frame; the override only shows on frames the animation does not
    /// touch that attribute. Cleared on a new animation.
    OverwriteOnNextKeyframe,
    /// `NOW_ANIMATION`: wins for the current animation; cleared on a new one.
    HoldUntilNextAnimation,
    /// `PERMANENT`: wins and persists across animation changes (re-validated by
    /// index range within the same SSAB).
    Permanent,
}

impl PriorityMode {
    /// Map the FFI `priority_mode` byte (`0/1/2`) to a mode.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::OverwriteOnNextKeyframe),
            1 => Some(Self::HoldUntilNextAnimation),
            2 => Some(Self::Permanent),
            _ => None,
        }
    }
}

/// Per-part color override (mirrors SS6 `PartsColorChange`). Applied to Normal
/// parts only. Corners are stored raw (r, g, b, a) in ssruntime order
/// `lt, rt, lb, rb`; alpha is pre-multiplied at the seam against the part's
/// composited alpha, matching keyframed PartColor.
#[derive(Clone, Copy)]
pub struct ColorOverride {
    /// `0=Mix 1=Mul 2=Add 3=Sub`.
    pub blend_op: u8,
    /// Four corners `lt, rt, lb, rb`, each `[r, g, b, a]`.
    pub corners_rgba: [[u8; 4]; 4],
    /// Per-corner blend rate (single value applied to all corners; `1.0` = full).
    pub rate: f32,
    pub priority: PriorityMode,
}

/// Per-part cell override (mirrors SS6 `CellChangeParts`). Applied to Normal and
/// Mask parts only. The cell is identified by `(cellmap_name_hash,
/// cell_name_hash)` and resolved to a [`ResolvedCellMeta`] **once at set time**
/// (cached here); the seam then only writes the resolved geometry/texture and
/// repoints the identity index — no per-frame hash lookup.
#[derive(Clone, Copy)]
pub struct CellOverride {
    pub cellmap_name_hash: u32,
    pub cell_name_hash: u32,
    pub resolved: ResolvedCellMeta,
    pub priority: PriorityMode,
}

/// Force-hide a part, optionally cascading to its descendants (a faithful port
/// of SS6 `HideSet`'s `flagInvolveChildren`). Force-*visible* (overriding a
/// keyframed hide) is intentionally reserved but not implemented (handover D-5);
/// the FFI reserves `force == 2` for it.
#[derive(Clone, Copy, Default)]
pub struct VisibilityOverride {
    /// When set, the force-hide propagates to all descendants in the part tree.
    pub cascade: bool,
}

/// A per-part override entry. Absent attributes are `None`. An all-`None` entry
/// is equivalent to no override (see [`PartOverride::is_empty`]).
#[derive(Clone, Default)]
pub struct PartOverride {
    pub visibility: Option<VisibilityOverride>,
    pub color: Option<ColorOverride>,
    pub cell: Option<CellOverride>,
}

impl PartOverride {
    /// True when the entry carries no active override (all attributes `None`).
    pub fn is_empty(&self) -> bool {
        self.visibility.is_none() && self.color.is_none() && self.cell.is_none()
    }
}

/// Apply the override table on the resolved pose at the seam.
///
/// `override_hide_workspace` is a caller-owned scratch buffer (reused frame to
/// frame to avoid hot-path allocation); its contents on entry are irrelevant.
/// `parts_color_vec` is the resolved layer's per-part color side vector; a color
/// override appends a resolved entry and repoints `PartState.part_color` (the
/// side vecs are rebuilt from the primary every frame, so appends never
/// accumulate across frames). A cell override likewise appends to `cell_vec`,
/// overwrites the parallel `cell_meta_per_part` (6 floats/part) + the
/// `cell_texture_hashes` slot, and repoints `PartState.cell`.
#[allow(clippy::too_many_arguments)]
pub fn apply_part_overrides(
    part_overrides: &[Option<PartOverride>],
    cached_parts: &[CachedPartData],
    parts_vec: &mut [PartState],
    parts_color_vec: &mut Vec<PartAttributePartColor>,
    cell_vec: &mut Vec<PartAttributeCell>,
    cell_meta_per_part: &mut [f32],
    cell_texture_hashes: &mut [u32],
    override_cell_uv_meta: &mut [Option<ResolvedCellMeta>],
    override_hide_workspace: &mut Vec<bool>,
) {
    let n = parts_vec.len();

    // --- Visibility (force-hide + optional cascade) ---
    //
    // Parts are stored parent-before-child, so a single forward pass that
    // propagates an "ancestor force-hidden (cascading)" flag through
    // `parent_index` handles the cascade without building child lists. Note
    // this cascade is *override-only* behavior: keyframed `hide` is per-part in
    // SS7 and never cascades (see `core/framedata.rs`), so we must not key the
    // cascade off `parts_vec[parent].hide()` — only off overrides.
    override_hide_workspace.clear();
    override_hide_workspace.resize(n, false);

    for i in 0..n {
        if i >= cached_parts.len() {
            break;
        }
        let vis = part_overrides.get(i).and_then(|o| o.as_ref()).and_then(|o| o.visibility);
        let self_hide = vis.is_some();
        let cascade = vis.map(|v| v.cascade).unwrap_or(false);

        let parent_idx = cached_parts[i].parent_index;
        let ancestor_cascading = if parent_idx >= 0 && (parent_idx as usize) < i { override_hide_workspace[parent_idx as usize] } else { false };

        // This part is force-hidden if it has its own override or inherits a
        // cascading force-hide from an ancestor.
        let force_hidden = self_hide || ancestor_cascading;
        // The workspace flag = "force-hide cascades from here to descendants":
        // an inherited cascade continues, and an own override cascades only when
        // its `cascade` bit is set.
        override_hide_workspace[i] = ancestor_cascading || (self_hide && cascade);

        if force_hidden {
            parts_vec[i].set_hide(true);
        }
    }

    // --- Color (Normal parts only) ---
    for i in 0..n {
        if i >= cached_parts.len() {
            break;
        }
        let Some(color) = part_overrides.get(i).and_then(|o| o.as_ref()).and_then(|o| o.color.as_ref()) else {
            continue;
        };
        // SS6 restricts PartsColorChange to NORMAL parts.
        if cached_parts[i].part_type_disc != ssab::ssab::ss::format::PartType::PartTypeNormal {
            continue;
        }

        let flag = parts_vec[i].update_flag();
        // NON yields to the animation whenever a keyframe drove color this frame.
        if color.priority == PriorityMode::OverwriteOnNextKeyframe && flag.contains(UpdateAttributeFlags::AttributePartColor) {
            continue;
        }

        // Build the override color and pre-multiply alpha by the part's already
        // composited alpha, exactly as keyframed PartColor does.
        let alpha = parts_vec[i].alpha();
        let corner = |k: usize| -> SsAttributePartColorKeyValueColor {
            let [r, g, b, a] = color.corners_rgba[k];
            let raw = SsAttributePartColorKeyValueColor::new(&U8Rect::new(a, r, g, b), color.rate);
            premultiply_part_color_corner(&raw, alpha)
        };
        let (lt, rt, lb, rb) = (corner(0), corner(1), corner(2), corner(3));
        let pc = PartAttributePartColor::new(BlendType(color.blend_op), &lt, &rt, &lb, &rb);

        parts_color_vec.push(pc);
        parts_vec[i].set_part_color((parts_color_vec.len() - 1) as i16);
        // Mark the attribute present so compose emits the 5-vertex/center-color
        // form for parts that had no keyframed color.
        parts_vec[i].set_update_flag(flag | UpdateAttributeFlags::AttributePartColor);
    }

    // --- Cell (Normal + Mask parts only) ---
    for i in 0..n {
        if i >= cached_parts.len() {
            break;
        }
        let Some(cell) = part_overrides.get(i).and_then(|o| o.as_ref()).and_then(|o| o.cell.as_ref()) else {
            continue;
        };
        let cp = &cached_parts[i];
        // SS6 restricts CellChangeParts to NORMAL; SS7 masks also sample a cell.
        if cp.part_type_disc != ssab::ssab::ss::format::PartType::PartTypeNormal && !cp.is_mask_contributor {
            continue;
        }

        let flag = parts_vec[i].update_flag();
        if cell.priority == PriorityMode::OverwriteOnNextKeyframe && flag.contains(UpdateAttributeFlags::AttributeCell) {
            continue;
        }

        let m = &cell.resolved;
        // (1) geometry channel — consumed by compute_local_vertices_for_parts.
        let off = i * 6;
        if off + 6 <= cell_meta_per_part.len() {
            cell_meta_per_part[off] = m.pivot_x;
            cell_meta_per_part[off + 1] = m.pivot_y;
            cell_meta_per_part[off + 2] = m.size_w;
            cell_meta_per_part[off + 3] = m.size_h;
            cell_meta_per_part[off + 4] = m.rect_left;
            cell_meta_per_part[off + 5] = m.rect_top;
        }
        // (2) texture channel — drives the DrawBatch texture_hash.
        if i < cell_texture_hashes.len() {
            cell_texture_hashes[i] = m.texture_hash;
        }
        // (3) UV channel — compute_local_uvs samples the keyframe cell
        // (resolved_cells[cursor]) which the seam cannot touch; record the
        // resolved meta so the overridden cell's UV rect / rotation / tex size
        // is what actually gets sampled.
        if i < override_cell_uv_meta.len() {
            override_cell_uv_meta[i] = Some(*m);
        }
        // (4) identity — the emitted FrameData's cell reference.
        cell_vec.push(PartAttributeCell::new(cell.cellmap_name_hash, cell.cell_name_hash));
        parts_vec[i].set_cell((cell_vec.len() - 1) as i16);
        parts_vec[i].set_update_flag(flag | UpdateAttributeFlags::AttributeCell);

        // A changed cell re-derives the default size for parts without a SizeX/Y
        // key (mirrors extract_pose's cell-size fallback).
        if !flag.contains(UpdateAttributeFlags::AttributeSizeX) {
            parts_vec[i].set_size_x(m.size_w);
        }
        if !flag.contains(UpdateAttributeFlags::AttributeSizeY) {
            parts_vec[i].set_size_y(m.size_h);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framedata::cached_part_stub;
    use ssab::ssab::ss::format::PartType;

    /// A blank part state with a given composited alpha and dirty-flag mask.
    fn state(alpha: f32, flag: UpdateAttributeFlags) -> PartState {
        let mut s = PartState::default();
        s.set_alpha(alpha);
        s.set_cell(-1);
        s.set_part_color(-1);
        s.set_update_flag(flag);
        s
    }

    /// Run `apply_part_overrides` with freshly-sized side buffers; returns
    /// `(parts_color_vec, cell_vec, cell_meta_per_part, cell_texture_hashes,
    /// override_cell_uv_meta)`.
    #[allow(clippy::type_complexity)]
    fn run(ovr: &[Option<PartOverride>], cached: &[CachedPartData], parts: &mut [PartState]) -> (Vec<PartAttributePartColor>, Vec<PartAttributeCell>, Vec<f32>, Vec<u32>, Vec<Option<ResolvedCellMeta>>) {
        let n = parts.len();
        let mut color_vec = Vec::new();
        let mut cell_vec = Vec::new();
        let mut cell_meta = vec![0.0f32; n * 6];
        let mut tex_hashes = vec![0u32; n];
        let mut uv_meta = vec![None; n];
        let mut ws = Vec::new();
        apply_part_overrides(ovr, cached, parts, &mut color_vec, &mut cell_vec, &mut cell_meta, &mut tex_hashes, &mut uv_meta, &mut ws);
        (color_vec, cell_vec, cell_meta, tex_hashes, uv_meta)
    }

    fn vis(cascade: bool) -> PartOverride {
        PartOverride { visibility: Some(VisibilityOverride { cascade }), ..Default::default() }
    }

    fn color(blend_op: u8, rgba: [u8; 4], priority: PriorityMode) -> PartOverride {
        PartOverride { color: Some(ColorOverride { blend_op, corners_rgba: [rgba; 4], rate: 1.0, priority }), ..Default::default() }
    }

    fn cell(cellmap_name_hash: u32, cell_name_hash: u32, size: (f32, f32), texture_hash: u32, priority: PriorityMode) -> PartOverride {
        let resolved = ResolvedCellMeta { size_w: size.0, size_h: size.1, texture_hash, ..Default::default() };
        PartOverride { cell: Some(CellOverride { cellmap_name_hash, cell_name_hash, resolved, priority }), ..Default::default() }
    }

    #[test]
    fn priority_mode_from_u8() {
        assert_eq!(PriorityMode::from_u8(0), Some(PriorityMode::OverwriteOnNextKeyframe));
        assert_eq!(PriorityMode::from_u8(1), Some(PriorityMode::HoldUntilNextAnimation));
        assert_eq!(PriorityMode::from_u8(2), Some(PriorityMode::Permanent));
        assert_eq!(PriorityMode::from_u8(3), None);
    }

    // --- Visibility ---

    #[test]
    fn visibility_force_hide_single() {
        let cached = [cached_part_stub(0, -1, PartType::PartTypeNull, false), cached_part_stub(1, 0, PartType::PartTypeNormal, false)];
        let mut parts = [state(1.0, UpdateAttributeFlags::empty()), state(1.0, UpdateAttributeFlags::empty())];
        let ovr = [None, Some(vis(false))];
        run(&ovr, &cached, &mut parts);
        assert!(!parts[0].hide());
        assert!(parts[1].hide());
    }

    #[test]
    fn visibility_cascade_hides_whole_subtree() {
        // 0 = root, 1 child of 0, 2 child of 1 (parent-before-child order).
        let cached = [cached_part_stub(0, -1, PartType::PartTypeNull, false), cached_part_stub(1, 0, PartType::PartTypeNormal, false), cached_part_stub(2, 1, PartType::PartTypeNormal, false)];
        let mut parts = [state(1.0, UpdateAttributeFlags::empty()), state(1.0, UpdateAttributeFlags::empty()), state(1.0, UpdateAttributeFlags::empty())];
        let ovr = [Some(vis(true)), None, None];
        run(&ovr, &cached, &mut parts);
        assert!(parts[0].hide() && parts[1].hide() && parts[2].hide());
    }

    #[test]
    fn visibility_non_cascade_leaves_descendants() {
        let cached = [cached_part_stub(0, -1, PartType::PartTypeNull, false), cached_part_stub(1, 0, PartType::PartTypeNormal, false), cached_part_stub(2, 1, PartType::PartTypeNormal, false)];
        let mut parts = [state(1.0, UpdateAttributeFlags::empty()), state(1.0, UpdateAttributeFlags::empty()), state(1.0, UpdateAttributeFlags::empty())];
        let ovr = [Some(vis(false)), None, None];
        run(&ovr, &cached, &mut parts);
        assert!(parts[0].hide());
        assert!(!parts[1].hide() && !parts[2].hide());
    }

    #[test]
    fn keyframe_hide_does_not_cascade() {
        // A parent hidden by a *keyframe* (not an override) must not hide its
        // child — the override cascade keys off overrides only.
        let cached = [cached_part_stub(0, -1, PartType::PartTypeNormal, false), cached_part_stub(1, 0, PartType::PartTypeNormal, false)];
        let mut parts = [state(1.0, UpdateAttributeFlags::empty()), state(1.0, UpdateAttributeFlags::empty())];
        parts[0].set_hide(true); // simulate a keyframed hide on the parent
        let ovr = [None, None];
        run(&ovr, &cached, &mut parts);
        assert!(!parts[1].hide());
    }

    // --- Color ---

    #[test]
    fn color_normal_only_and_premultiply() {
        let cached = [cached_part_stub(0, -1, PartType::PartTypeNull, false), cached_part_stub(1, -1, PartType::PartTypeNormal, false)];
        let mut parts = [state(1.0, UpdateAttributeFlags::empty()), state(0.5, UpdateAttributeFlags::empty())];
        // Add color (Add op) with opaque input to both; only the Normal part takes it.
        let ovr = [Some(color(2, [0x80, 0x40, 0x20, 0xFF], PriorityMode::HoldUntilNextAnimation)), Some(color(2, [0x80, 0x40, 0x20, 0xFF], PriorityMode::HoldUntilNextAnimation))];
        let (cv, ..) = run(&ovr, &cached, &mut parts);

        assert_eq!(parts[0].part_color(), -1, "non-Normal part must be skipped");
        let idx = parts[1].part_color();
        assert!(idx >= 0);
        assert_eq!(cv.len(), 1);
        assert!(parts[1].update_flag().contains(UpdateAttributeFlags::AttributePartColor), "center-vertex flag must be set");
        let lt = cv[idx as usize].lt().rgba();
        assert_eq!((lt.r(), lt.g(), lt.b()), (0x80, 0x40, 0x20));
        assert_eq!(lt.a(), 127, "alpha 0xFF pre-multiplied by part alpha 0.5");
        assert_eq!(cv[idx as usize].blend_type().0, 2, "blend op preserved (Add)");
    }

    #[test]
    fn color_priority_non_yields_to_keyframe() {
        let cached = [cached_part_stub(0, -1, PartType::PartTypeNormal, false)];
        // Animation drove PartColor this frame → NON override yields.
        let mut parts = [state(1.0, UpdateAttributeFlags::AttributePartColor)];
        let ovr = [Some(color(0, [1, 2, 3, 255], PriorityMode::OverwriteOnNextKeyframe))];
        let (cv, ..) = run(&ovr, &cached, &mut parts);
        assert!(cv.is_empty());
        assert_eq!(parts[0].part_color(), -1);
    }

    #[test]
    fn color_priority_now_overrides_keyframe() {
        let cached = [cached_part_stub(0, -1, PartType::PartTypeNormal, false)];
        let mut parts = [state(1.0, UpdateAttributeFlags::AttributePartColor)];
        let ovr = [Some(color(0, [1, 2, 3, 255], PriorityMode::HoldUntilNextAnimation))];
        let (cv, ..) = run(&ovr, &cached, &mut parts);
        assert_eq!(cv.len(), 1);
        assert!(parts[0].part_color() >= 0);
    }

    // --- Cell ---

    #[test]
    fn cell_applies_all_channels_and_size() {
        let cached = [cached_part_stub(0, -1, PartType::PartTypeNormal, false)];
        let mut parts = [state(1.0, UpdateAttributeFlags::empty())];
        let ovr = [Some(cell(0x1111, 0x2222, (30.0, 40.0), 0xABCD, PriorityMode::HoldUntilNextAnimation))];
        let (_, cv, cmp, cth, uv) = run(&ovr, &cached, &mut parts);

        let idx = parts[0].cell();
        assert!(idx >= 0);
        assert_eq!((cv[idx as usize].cellmap_name_hash(), cv[idx as usize].name_hash()), (0x1111, 0x2222));
        assert_eq!(cth[0], 0xABCD, "texture channel");
        assert_eq!(cmp[2], 30.0, "geometry size_w at +2");
        assert_eq!(cmp[3], 40.0, "geometry size_h at +3");
        // UV channel: the resolved meta is recorded so compute_local_uvs samples
        // the overridden cell (not just the geometry/texture channels).
        let uv_meta = uv[0].expect("UV meta recorded for the overridden cell");
        assert_eq!((uv_meta.size_w, uv_meta.size_h, uv_meta.texture_hash), (30.0, 40.0, 0xABCD));
        assert!(parts[0].update_flag().contains(UpdateAttributeFlags::AttributeCell));
        assert_eq!(parts[0].size_x(), 30.0, "size re-derived when unkeyed");
        assert_eq!(parts[0].size_y(), 40.0);
    }

    #[test]
    fn cell_normal_and_mask_only() {
        let cached = [cached_part_stub(0, -1, PartType::PartTypeNull, false), cached_part_stub(1, -1, PartType::PartTypeMask, true)];
        let mut parts = [state(1.0, UpdateAttributeFlags::empty()), state(1.0, UpdateAttributeFlags::empty())];
        let ov = cell(1, 2, (8.0, 8.0), 9, PriorityMode::HoldUntilNextAnimation);
        let ovr = [Some(ov.clone()), Some(ov)];
        let (_, cv, ..) = run(&ovr, &cached, &mut parts);
        assert_eq!(parts[0].cell(), -1, "Null part skipped");
        assert!(parts[1].cell() >= 0, "Mask part applied");
        assert_eq!(cv.len(), 1);
    }

    #[test]
    fn cell_priority_non_yields_to_keyframe() {
        let cached = [cached_part_stub(0, -1, PartType::PartTypeNormal, false)];
        let mut parts = [state(1.0, UpdateAttributeFlags::AttributeCell)];
        let ovr = [Some(cell(1, 2, (8.0, 8.0), 9, PriorityMode::OverwriteOnNextKeyframe))];
        let (_, cv, ..) = run(&ovr, &cached, &mut parts);
        assert!(cv.is_empty());
        assert_eq!(parts[0].cell(), -1);
    }
}
