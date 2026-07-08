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

use crate::core::framedata::CachedPartData;
use crate::runtime::framedata::ss::runtime::PartState;

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
}

impl PartOverride {
    /// True when the entry carries no active override (all attributes `None`).
    pub fn is_empty(&self) -> bool {
        self.visibility.is_none()
    }
}

/// Apply the override table on the resolved pose at the seam.
///
/// `override_hide_workspace` is a caller-owned scratch buffer (reused frame to
/// frame to avoid hot-path allocation); its contents on entry are irrelevant.
pub fn apply_part_overrides(part_overrides: &[Option<PartOverride>], cached_parts: &[CachedPartData], parts_vec: &mut [PartState], override_hide_workspace: &mut Vec<bool>) {
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
}
