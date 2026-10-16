//! The Override Layer API: per-part visibility, color and cell overrides set at runtime.

use super::Context;
use crate::error;
use crate::resource::fnv1a_hash_str;

impl<'a> Context<'a> {
    /// Resolve a part name to its `part_index` in the current primary animation,
    /// or `-1` if not loaded / not found. Indices are stable within an SSAB, so
    /// callers can resolve once and reuse the index across animation changes.
    pub fn find_part_index(&self, name: &str) -> i32 {
        let Some(resource) = self.eval.resource else {
            return -1;
        };
        let anim_idx = self.instance.primary_source().animation_idx;
        if anim_idx < 0 {
            return -1;
        }
        let hash = fnv1a_hash_str(name);
        resource.animations[anim_idx as usize].cached_parts().iter().position(|p| p.name_hash == hash).map(|i| i as i32).unwrap_or(-1)
    }

    /// The integer frame the primary playhead is on right now — baked into a NON
    /// override at set time so its own set-frame keyframe can be skipped through
    /// the update of its first evaluation (see `PriorityMode::OverwriteOnNextKeyframe`).
    fn override_set_frame(&self) -> i32 {
        crate::core::math::floorf(self.instance.primary_source().controller.get_frame_no()) as i32
    }

    /// The primary animation's part count — the range an override's part index
    /// must fall in — or 0 before an animation is set up.
    pub(crate) fn override_part_count(&self) -> usize {
        let anim_idx = self.instance.primary_source().animation_idx;
        match self.eval.resource {
            Some(resource) if anim_idx >= 0 => resource.animations[anim_idx as usize].cached_parts().len(),
            _ => 0,
        }
    }

    /// The override slot of `part`, creating the table the first time an
    /// override is set.
    fn override_slot(&mut self, part: u32) -> Result<&mut Option<crate::core::override_layer::PartOverride>, error::SSRuntimeError> {
        let (idx, num_parts) = (part as usize, self.override_part_count());
        if idx >= num_parts {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        if self.instance.part_overrides.is_empty() {
            self.instance.part_overrides.resize(num_parts, None);
        }
        Ok(&mut self.instance.part_overrides[idx])
    }

    /// Clear one kind of override on `part`. Nothing to do when the table was
    /// never created.
    fn clear_override(&mut self, part: u32, clear: impl FnOnce(&mut crate::core::override_layer::PartOverride)) -> Result<(), error::SSRuntimeError> {
        let idx = part as usize;
        if idx >= self.override_part_count() {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        if let Some(entry) = self.instance.part_overrides.get_mut(idx).and_then(|e| e.as_mut()) {
            clear(entry);
        }
        self.instance.compact_override(idx);
        Ok(())
    }

    /// Set or clear a per-part visibility override.
    /// `force`: `0` = clear/revert to animation, `1` = force-hide,
    /// `2` = force-visible (reserved, not implemented — rejected).
    /// `cascade` propagates a force-hide to descendants.
    pub fn set_part_visibility_override(&mut self, part: u32, force: u8, cascade: bool) -> Result<(), error::SSRuntimeError> {
        match force {
            0 => self.clear_override(part, |entry| entry.visibility = None)?,
            1 => {
                let entry = self.override_slot(part)?.get_or_insert_with(Default::default);
                entry.visibility = Some(crate::core::override_layer::VisibilityOverride { cascade });
            }
            _ => return Err(error::SSRuntimeError::InvalidParameter),
        }
        Ok(())
    }

    /// Clear the visibility override on a part (revert to animation).
    pub fn clear_part_visibility_override(&mut self, part: u32) -> Result<(), error::SSRuntimeError> {
        self.set_part_visibility_override(part, 0, false)
    }

    /// Set a per-part color override with a single RGBA (fills all four corners).
    /// `blend_op`: `0=Mix 1=Mul 2=Add 3=Sub`. `rgba`: packed `0xRRGGBBAA`.
    /// `priority`: `0=NON 1=NOW_ANIMATION 2=PERMANENT`. Normal parts only.
    pub fn set_part_color_override(&mut self, part: u32, blend_op: u8, rgba: u32, priority: u8) -> Result<(), error::SSRuntimeError> {
        let corner = rgba.to_be_bytes(); // 0xRRGGBBAA -> [r, g, b, a]
        self.set_part_color_override_corners(part, blend_op, [corner; 4], priority)
    }

    /// Set a per-part color override with distinct corners in ssruntime order
    /// `lt, rt, lb, rb`, each `[r, g, b, a]`.
    pub fn set_part_color_override_corners(&mut self, part: u32, blend_op: u8, corners_rgba: [[u8; 4]; 4], priority: u8) -> Result<(), error::SSRuntimeError> {
        if blend_op >= 4 {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        let Some(priority) = crate::core::override_layer::PriorityMode::from_u8(priority) else {
            return Err(error::SSRuntimeError::InvalidParameter);
        };
        // Anchor the NON one-shot to "from the moment of override": remember the
        // set frame (its own keyframe is skipped through the update of the first
        // evaluation).
        let set_frame = self.override_set_frame();
        let entry = self.override_slot(part)?.get_or_insert_with(Default::default);
        entry.color = Some(crate::core::override_layer::ColorOverride { blend_op, corners_rgba, rate: 1.0, priority, set_frame, first_eval_update: None });
        Ok(())
    }

    /// Clear the color override on a part.
    pub fn clear_part_color_override(&mut self, part: u32) -> Result<(), error::SSRuntimeError> {
        self.clear_override(part, |entry| entry.color = None)
    }

    /// Set a per-part cell override, identified by `(cellmap_name_hash,
    /// cell_name_hash)` (FNV-1a of the cellmap name without `.ssce` and the cell
    /// name). `(0, 0)` reverts to the animation. `priority`: `0=NON 1=NOW 2=
    /// PERMANENT`. Normal + Mask parts only. The cell is resolved against the
    /// bound SSAB now and cached; errors if it cannot be resolved.
    pub fn set_part_cell_override(&mut self, part: u32, cellmap_name_hash: u32, cell_name_hash: u32, priority: u8) -> Result<(), error::SSRuntimeError> {
        if cellmap_name_hash == 0 && cell_name_hash == 0 {
            return self.clear_part_cell_override(part);
        }
        let Some(priority) = crate::core::override_layer::PriorityMode::from_u8(priority) else {
            return Err(error::SSRuntimeError::InvalidParameter);
        };
        if part as usize >= self.override_part_count() {
            return Err(error::SSRuntimeError::InvalidParameter);
        }
        let Some(resource) = self.eval.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(resource.get_ssab_data()) };
        let Some(resolved) = ssab.cellmaps().and_then(|cellmaps| crate::core::framedata::resolve_cell(&cellmaps, cellmap_name_hash, cell_name_hash)) else {
            return Err(error::SSRuntimeError::InvalidParameter);
        };
        let set_frame = self.override_set_frame();
        let entry = self.override_slot(part)?.get_or_insert_with(Default::default);
        entry.cell = Some(crate::core::override_layer::CellOverride { cellmap_name_hash, cell_name_hash, resolved, priority, set_frame, first_eval_update: None });
        Ok(())
    }

    /// Clear the cell override on a part.
    pub fn clear_part_cell_override(&mut self, part: u32) -> Result<(), error::SSRuntimeError> {
        self.clear_override(part, |entry| entry.cell = None)
    }

    /// Clear every per-part override.
    pub fn clear_all_part_overrides(&mut self) {
        for entry in self.instance.part_overrides.iter_mut() {
            *entry = None;
        }
    }
}
