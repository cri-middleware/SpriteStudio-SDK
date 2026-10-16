//! [`InstanceSlot`]: an instance part's child context, driven by the parent's instance keys.

use crate::Context;

/// High-level instance-child slot: tracks the per-slot transition state
/// previously kept in each Player's instance-children table. `step()` resolves
/// labels against the *child* runtime context, configures the child's
/// playback (animation section, loop, direction), and advances the child's
/// frame — leaving only the C++/host-side `play()` event sink and the
/// redraw call to the caller.
pub struct InstanceSlot {
    last_event_frame: i32,
    last_is_synthetic: bool,
    last_independent: bool,
    // Speed of the last real EventInstance — reused while the implicit frame-0
    // key is being ignored (independent playback across a parent loop).
    last_speed: f32,
}

impl Default for InstanceSlot {
    fn default() -> Self {
        Self::new()
    }
}

impl InstanceSlot {
    pub const fn new() -> Self {
        Self { last_event_frame: -1, last_is_synthetic: false, last_independent: false, last_speed: 1.0 }
    }

    /// Whether the slot is currently driving its child independently of the
    /// parent's frame. Reflects the most recently seen `EventInstance.independent`.
    pub fn is_independent(&self) -> bool {
        self.last_independent
    }

    /// Reset the slot's "last started key" memory to its just-created state so
    /// the host can re-fire the instance child on an explicit head-out (`play`
    /// / replay) instead of debouncing the first key as the same one it last
    /// started. The child context is owned by the host and reset
    /// separately; the next `step` re-configures it on transition, and the host
    /// plays it.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Drive the slot for one tick. Configures `child_ctx` on transition
    /// edges and advances its frame either by parent-relative seek or by
    /// delta-time update depending on `independent`.
    pub fn step(&mut self, ev: crate::types::ss_event_instance_info, child_ctx: &mut Context<'_>, parent_frame: f32, delta_seconds: f32, parent_looped: bool) -> crate::types::ss_instance_step_result {
        // While the slot drives its child independently, the SpriteStudio implicit
        // frame-0 playback key (synthetic default) is ignored — the independent
        // child keeps running across the parent's loop instead of being
        // re-seeked to it. Only a real authored EventInstance re-transitions it.
        let ignore_implicit = self.last_independent && ev.is_synthetic_default;

        // Synced children re-arm on the parent loop edge so the authored
        // EventInstance re-fires each playthrough. Independent children follow the
        // "last started key" rule instead: a re-crossing of the same key must not
        // restart the child (sustained / fire-once), only a different key re-fires.
        // Keep the memory intact when the last started key was independent.
        // Mirrors EffectSlot.
        if parent_looped && !ignore_implicit && !self.last_independent {
            self.last_event_frame = -1;
            self.last_is_synthetic = false;
        }

        let transitioned = !ignore_implicit && (self.last_event_frame == -1 || self.last_event_frame != ev.event_frame || self.last_is_synthetic != ev.is_synthetic_default);

        if transitioned {
            // `_start` / `_end` are implicit — they are not in the label table,
            // so these fallbacks are what every default key resolves to. They
            // read the animation's own range, never the section: the
            // `set_animation_section` below narrows that, and reading it back
            // shrinks the section again on every re-transition.
            let start_label_time = child_ctx.resolve_label_time(ev.start_label_hash, child_ctx.get_anime_data_start_frame());
            let end_label_time = child_ctx.resolve_label_time(ev.end_label_hash, child_ctx.get_anime_data_end_frame());
            let start_frame = start_label_time + ev.start_offset;
            let mut end_frame = end_label_time + ev.end_offset;
            if end_frame < start_frame {
                end_frame = start_frame;
            }

            let dir = if ev.reverse { crate::eval::playback_controller::PlaybackDirection::Backward } else { crate::eval::playback_controller::PlaybackDirection::Forward };
            let style = if ev.pingpong { crate::eval::playback_controller::PlaybackStyle::PingPong } else { crate::eval::playback_controller::PlaybackStyle::Normal };
            child_ctx.set_playback_direction(dir, style);
            child_ctx.set_animation_section(start_frame, end_frame);
            child_ctx.set_loop_count(ev.loop_num);

            self.last_event_frame = ev.event_frame;
            self.last_is_synthetic = ev.is_synthetic_default;
            self.last_speed = ev.speed;
        }

        // When ignoring the implicit key, keep the child's playback config and
        // step it with the last real event's independent mode + speed.
        let independent = if ignore_implicit { true } else { ev.independent };
        let speed = if ignore_implicit { self.last_speed } else { ev.speed };

        // Branch on `independent`. Synced children are seeked deterministically
        // from the parent's frame each tick — same `parent_frame` always yields
        // the same `child_frame`. Independent children own their own controller
        // time: forward `delta_seconds` to `update`, except on the transition
        // tick itself — the section change above has already put the child on
        // its first frame for the direction, where the host's `play()` leaves it,
        // and a further step here would visibly skip that frame.
        let child_frame_no = if independent {
            if delta_seconds > 0.0 && !transitioned {
                let d_ms = delta_seconds * 1000.0 * speed;
                child_ctx.update(d_ms)
            } else {
                child_ctx.get_frame_no()
            }
        } else {
            let diff = (parent_frame - ev.event_frame as f32) * speed;
            child_ctx.set_frame_relative(diff);
            child_ctx.get_frame_no()
        };

        // Hosts feed this back as `parent_looped` when stepping the child's own
        // instance/effect slots, so it re-arms the grandchildren's playback keys:
        // an independent child runs on its own clock, making its loop — not the
        // parent animation's — the edge they must re-fire on.
        let child_looped = independent && delta_seconds > 0.0 && !transitioned && child_ctx.just_looped();

        self.last_independent = independent;

        crate::types::ss_instance_step_result { visible: true, transitioned, independent, child_frame_no, child_looped }
    }
}
