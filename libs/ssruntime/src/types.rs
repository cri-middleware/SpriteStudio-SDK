//! POD return types shared across the runtime API surface.
//!
//! These are plain `#[repr(C)]` structs designed for return-by-value across
//! the FFI boundary, but the definitions live at the crate root (not under
//! `ffi/`) so they are also reachable from the `Context` API and from the
//! `web` feature without dragging in the C extern "C" surface.
//!
//! They are intentionally separate from ssab/effect_state FlatBuffers data —
//! FB is for persistent or zero-copy bulk data, POD is for ephemeral per-call
//! return values.

#![allow(non_camel_case_types)]

/// Resolved per-slot Instance event state, returned by
/// `ss_runtime_get_active_event_instance`.
///
/// The runtime forward-scans the parent's `EventsPerFrame` and reports the
/// most recent `EventInstance` whose `frame_index <= current parent frame`
/// for `slot_index`. When no event is active, the function returns SS6
/// default semantics: `is_synthetic_default = true`, `loop_num = 1`,
/// `speed = 1.0`, all label hashes 0 (caller falls back), all flags false.
///
/// `start_label_hash` / `end_label_hash` are returned raw — resolve them
/// against the *child* runtime context with `ss_runtime_resolve_label_time`.
/// This keeps late binding intact across dynamic-swap of the child ssab.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ss_event_instance_info {
    /// Parent `frame_index` this `EventInstance` was emitted at; 0 for synthetic default.
    pub event_frame: i32,
    /// 0 = unset → caller should fall back to 0.
    pub start_label_hash: u32,
    pub start_offset: i32,
    /// 0 = unset → caller should fall back to (child_total_frames - 1).
    pub end_label_hash: u32,
    pub end_offset: i32,
    /// -1 = infinite, >=1 = finite count.
    pub loop_num: i32,
    pub speed: f32,
    pub independent: bool,
    pub reverse: bool,
    pub pingpong: bool,
    /// `true` = SS6 default (no real event active), `false` = real `EventInstance`.
    pub is_synthetic_default: bool,
}

impl ss_event_instance_info {
    pub const fn synthetic_default() -> Self {
        Self { event_frame: 0, start_label_hash: 0, start_offset: 0, end_label_hash: 0, end_offset: 0, loop_num: 1, speed: 1.0, independent: false, reverse: false, pingpong: false, is_synthetic_default: true }
    }
}

impl Default for ss_event_instance_info {
    fn default() -> Self {
        Self::synthetic_default()
    }
}

/// Resolved per-slot Effect event state, returned by
/// `ss_runtime_get_active_effect_event`.
///
/// The runtime forward-scans the parent's `EventsPerFrame` and reports the
/// most recent `EventEffect` whose `frame_index <= current parent frame`
/// for `slot_index`. When no event is active, the runtime falls back to the
/// part's `InitialEvents.effect` (if present); if neither exists, returns
/// SS6 default semantics: `is_synthetic_default = true`, `start_time = 0`,
/// `speed = 1.0`, `independent = false`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ss_effect_event_info {
    /// Parent `frame_index` this `EventEffect` was emitted at; 0 for synthetic default
    /// or `InitialEvents.effect` fallback.
    pub event_frame: i32,
    pub start_time: i32,
    pub speed: f32,
    pub independent: bool,
    /// `true` = no real `EventEffect` matched (either `InitialEvents.effect` fallback
    /// or full SS6 default), `false` = real `EventEffect` matched.
    pub is_synthetic_default: bool,
}

impl ss_effect_event_info {
    pub const fn synthetic_default() -> Self {
        Self { event_frame: 0, start_time: 0, speed: 1.0, independent: false, is_synthetic_default: true }
    }
}

impl Default for ss_effect_event_info {
    fn default() -> Self {
        Self::synthetic_default()
    }
}

/// Result of `EffectSlot::step` / `ss_effect_slot_step`.
///
/// `state_buf` / `state_len` are valid only when `visible == true`, and only
/// until the next `step` call on the same slot — callers must consume the
/// state within the same frame and not retain pointers across calls.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ss_effect_step_result {
    pub visible: bool,
    pub state_buf: *const u8,
    pub state_len: usize,
}

impl ss_effect_step_result {
    pub const fn not_visible() -> Self {
        Self { visible: false, state_buf: core::ptr::null(), state_len: 0 }
    }
}

impl Default for ss_effect_step_result {
    fn default() -> Self {
        Self::not_visible()
    }
}
