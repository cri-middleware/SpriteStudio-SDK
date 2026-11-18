//! The events a frame passes, and the event / effect keys active on a part.

use super::Context;

impl<'a> Context<'a> {
    pub(super) fn collect_events(&mut self) {
        self.instance.collected_event_indices.clear();
        self.instance.collected_event_source_handles.clear();
        let Some(resource) = self.eval.resource else {
            return;
        };
        let primary_idx = self.instance.primary_idx();
        let num_sources = self.instance.sources.len();

        for i in 0..num_sources {
            let source = &self.instance.sources[i];
            if !source.has_real_animation() {
                continue;
            }
            let is_primary = i == primary_idx;
            if !is_primary && !self.instance.source_fires_events(i) {
                continue;
            }
            let metadata = &resource.animations[source.animation_idx as usize];
            let Some(events_vec) = metadata.events else { continue };
            let source_handle = i as u32;

            for &frame_no in source.controller.get_passed_frames() {
                if frame_no < 0 || frame_no > u16::MAX as i32 {
                    continue;
                }
                let target_frame = frame_no as u16;
                let mut low = 0;
                let mut high = events_vec.len();
                while low < high {
                    let mid = low + (high - low) / 2;
                    match events_vec.get(mid).frame_index().cmp(&target_frame) {
                        ::core::cmp::Ordering::Equal => {
                            self.instance.collected_event_indices.push(mid as i32);
                            self.instance.collected_event_source_handles.push(source_handle);
                            break;
                        }
                        ::core::cmp::Ordering::Less => low = mid + 1,
                        ::core::cmp::Ordering::Greater => high = mid,
                    }
                }
            }
        }
    }

    /// The events the last [`Self::update`] passed. They stay until the next one,
    /// [`Self::setup_animation`], [`Self::bind_resource`] or [`Self::reset`].
    pub fn get_passed_event_indices(&self) -> &[i32] {
        &self.instance.collected_event_indices
    }

    /// Parallel to [`Self::get_passed_event_indices`]: source handle that fired
    /// each collected event. With every source and every track's outgoing side
    /// at the default event threshold (`+∞`), entries are uniformly
    /// [`Self::get_track_main_source`] of track 0. Diverges once secondaries opt
    /// in to event firing via [`Self::set_source_event_threshold`], or a
    /// crossfade's outgoing side via [`Self::set_track_outgoing_event_threshold`].
    pub fn get_passed_event_source_handles(&self) -> &[u32] {
        &self.instance.collected_event_source_handles
    }

    /// Resolve the most-recent `EventInstance` whose `frame_index <= current frame`
    /// for `slot_index` on the *currently selected animation*. Returns
    /// `synthetic_default()` when no event has fired yet (SS6 default semantics).
    /// Designed to live behind the `ss_runtime_get_active_event_instance` FFI.
    pub fn get_active_event_instance(&self, slot_index: u32) -> crate::types::ss_event_instance_info {
        let mut info = crate::types::ss_event_instance_info::synthetic_default();
        let Some(resource) = self.eval.resource else { return info };
        if self.instance.primary_source().animation_idx < 0 {
            return info;
        }
        let metadata = &resource.animations[self.instance.primary_source().animation_idx as usize];
        let Some(events) = metadata.events else { return info };

        let frame_no = self.instance.primary_source().controller.get_frame_no();
        let frame_int = if frame_no < 0.0 {
            -1
        } else if frame_no >= u16::MAX as f32 {
            u16::MAX as i32
        } else {
            frame_no as i32
        };

        // Forward scan: events are sorted by frame_index. Later matches override
        // earlier ones, so the loop naturally lands on the most-recent
        // EventInstance for `slot_index`.
        for i in 0..events.len() {
            let epf = events.get(i);
            if (epf.frame_index() as i32) > frame_int {
                break;
            }
            let Some(instances) = epf.instances() else { continue };
            for j in 0..instances.len() {
                let ev = instances.get(j);
                if ev.part_index() as u32 != slot_index {
                    continue;
                }
                let attr = ev.value();
                info = crate::types::ss_event_instance_info {
                    event_frame: epf.frame_index() as i32,
                    start_label_hash: attr.start_label_hash(),
                    start_offset: attr.start_offset(),
                    end_label_hash: attr.end_label_hash(),
                    end_offset: attr.end_offset(),
                    loop_num: attr.loop_num(),
                    speed: attr.speed(),
                    independent: attr.independent(),
                    reverse: attr.reverse(),
                    pingpong: attr.pingpong(),
                    is_synthetic_default: false,
                };
            }
        }
        info
    }

    /// Resolve the most-recent `EventEffect` whose `frame_index <= current frame`
    /// for `slot_index` on the *currently selected animation*. When no event has
    /// fired yet, falls back to `InitialEvents.effect` for that slot (still flagged
    /// `is_synthetic_default = true`); if neither exists returns full SS6 defaults.
    /// Designed to live behind the `ss_runtime_get_active_effect_event` FFI.
    pub fn get_active_effect_event(&self, slot_index: u32) -> crate::types::ss_effect_event_info {
        let mut info = crate::types::ss_effect_event_info::synthetic_default();
        let Some(resource) = self.eval.resource else { return info };
        if self.instance.primary_source().animation_idx < 0 {
            return info;
        }
        let metadata = &resource.animations[self.instance.primary_source().animation_idx as usize];

        // Fallback first: InitialEvents.effect for this slot. Real EventEffect
        // matches override below; the synthetic-default flag stays `true` for
        // initial-only state so callers don't treat it as an event edge.
        if let Some(inits) = metadata.initial_events
            && (slot_index as usize) < inits.len()
        {
            let entry = inits.get(slot_index as usize);
            if let Some(eff) = entry.effect() {
                info.event_frame = 0;
                info.start_time = eff.start_time();
                let s = eff.speed();
                info.speed = if s == 0.0 { 1.0 } else { s };
                // The implicit frame-0 (Setup) key is never an independent-playback
                // start point: only a real authored EventEffect may begin independent
                // playback. Forced to false so this mirrors the Instance synthetic
                // default (`ss_event_instance_info::synthetic_default`, independent =
                // false) and the two part types stay symmetric.
                info.independent = false;
                info.is_synthetic_default = true;
            }
        }

        let Some(events) = metadata.events else { return info };

        let frame_no = self.instance.primary_source().controller.get_frame_no();
        let frame_int = if frame_no < 0.0 {
            -1
        } else if frame_no >= u16::MAX as f32 {
            u16::MAX as i32
        } else {
            frame_no as i32
        };

        for i in 0..events.len() {
            let epf = events.get(i);
            if (epf.frame_index() as i32) > frame_int {
                break;
            }
            let Some(effects) = epf.effects() else { continue };
            for j in 0..effects.len() {
                let ev = effects.get(j);
                if ev.part_index() as u32 != slot_index {
                    continue;
                }
                let attr = ev.value();
                let s = attr.speed();
                info = crate::types::ss_effect_event_info { event_frame: epf.frame_index() as i32, start_time: attr.start_time(), speed: if s == 0.0 { 1.0 } else { s }, independent: attr.independent(), is_synthetic_default: false };
            }
        }
        info
    }

    /// Look up `label_hash` in the currently selected animation's `labels`
    /// table and return its `time`. Returns `fallback` when `label_hash == 0`
    /// (caller convention for "unset"), no animation is bound, or the hash
    /// is not found. Linear scan — labels per animation are typically <50.
    pub fn resolve_label_time(&self, label_hash: u32, fallback: i32) -> i32 {
        if label_hash == 0 {
            return fallback;
        }
        let Some(resource) = self.eval.resource else { return fallback };
        if self.instance.primary_source().animation_idx < 0 {
            return fallback;
        }
        let metadata = &resource.animations[self.instance.primary_source().animation_idx as usize];
        let Some(labels) = metadata.labels else { return fallback };
        for i in 0..labels.len() {
            let label = labels.get(i);
            if label.name_hash() == label_hash {
                return label.time();
            }
        }
        fallback
    }
}
