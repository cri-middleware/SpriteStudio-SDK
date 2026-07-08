//! Per-part runtime **Override Layer** (ROADMAP Phase 2).
//!
//! Applies per-part runtime overrides (Visibility now; Color/Cell added in later
//! stages) on the *resolved* pose at a single seam in `Context::get_frame_data`
//! — after the source mix, before `compose_frame_data`. Overriding here (rather
//! than during per-source `extract_pose`) is what lets an override win over both
//! keyframe values and blend results.
//!
//! The override table is a dense `Vec<Option<PartOverride>>` on `InstanceState`,
//! indexed by `part_index` (O(1), zero per-frame alloc, `no_std` — no `HashMap`).
//!
//! See `libs/ssruntime/OVERRIDE_LAYER_SPEC.md` for the design and the ForUnity
//! handover decisions (D-1..D-6) it is reconciled against.

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
        // (1) geometry/UV channel — consumed by compute_local_vertices_for_parts.
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
        // (3) identity — the emitted FrameData's cell reference.
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
