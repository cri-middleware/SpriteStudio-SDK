//! `InstanceSlot` tests: independent children and their loops.

use super::*;

/// Drives an independent instance child for `ticks` one-frame steps and
/// returns how many of them reported `child_looped`. Mirrors a Player's host
/// loop: `play()` on the transition edge, then `step` once per frame.
fn count_independent_child_loops(ev: crate::types::ss_event_instance_info, ticks: usize) -> usize {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut child_ctx = Context::new();
    child_ctx.bind_resource(&resource);
    child_ctx.setup_animation("walk").unwrap();

    let mut slot = InstanceSlot::new();
    let delta = 1.0 / child_ctx.get_frame_rate() as f32;

    // Transition tick: the slot configures the child, the host plays it.
    let r = slot.step(ev, &mut child_ctx, 0.0, 0.0, false);
    assert!(r.transitioned, "first step must be a transition edge");
    child_ctx.play();

    let mut looped = 0;
    for _ in 0..ticks {
        if slot.step(ev, &mut child_ctx, 0.0, delta, false).child_looped {
            looped += 1;
        }
    }
    looped
}

#[test]
fn independent_child_looped_does_not_fire_every_tick_on_reverse() {
    // A backward-playing child's frame decreases on *every* tick, so any
    // "frame went backwards" inference reports a loop every tick instead of
    // once per wrap.
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut ctx = Context::new();
    ctx.bind_resource(&resource);
    ctx.setup_animation("walk").unwrap();
    let cycle = (ctx.get_end_frame() - ctx.get_start_frame() + 1) as usize;

    let ev = crate::types::ss_event_instance_info { event_frame: 0, loop_num: -1, speed: 1.0, independent: true, reverse: true, is_synthetic_default: false, ..Default::default() };

    let ticks = cycle * 3;
    let looped = count_independent_child_loops(ev, ticks);
    assert_eq!(looped, 3, "reverse child over {ticks} ticks of a {cycle}-frame animation");
}

#[test]
fn independent_child_looped_does_not_fire_every_tick_on_pingpong() {
    // A ping-pong child's frame decreases for the whole return leg, so the
    // same inference reports a loop on every tick of that leg.
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut ctx = Context::new();
    ctx.bind_resource(&resource);
    ctx.setup_animation("walk").unwrap();
    let cycle = (ctx.get_end_frame() - ctx.get_start_frame() + 1) as usize;

    let ev = crate::types::ss_event_instance_info { event_frame: 0, loop_num: -1, speed: 1.0, independent: true, pingpong: true, is_synthetic_default: false, ..Default::default() };

    // One ping-pong cycle is there-and-back, so 6 cycles' worth of ticks is ~3 loops.
    let ticks = cycle * 6;
    let looped = count_independent_child_loops(ev, ticks);
    assert_eq!(looped, 3, "pingpong child over {ticks} ticks of a {cycle}-frame animation");
}

#[test]
fn re_transition_keeps_the_offset_section_instead_of_shrinking_it() {
    // A start-offset key re-fired (a replay, or a second authored key)
    // must resolve `_end` against the animation, not against the section
    // the previous transition left behind.
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut child_ctx = Context::new();
    child_ctx.bind_resource(&resource);
    child_ctx.setup_animation("walk").unwrap();
    let last = child_ctx.get_anime_data_end_frame();

    let mut slot = InstanceSlot::new();
    let ev = crate::types::ss_event_instance_info { event_frame: 0, loop_num: -1, speed: 1.0, start_offset: 3, is_synthetic_default: false, ..Default::default() };

    slot.step(ev, &mut child_ctx, 0.0, 0.0, false);
    assert_eq!((child_ctx.get_start_frame(), child_ctx.get_end_frame()), (3, last));

    slot.reset();
    slot.step(ev, &mut child_ctx, 0.0, 0.0, false);
    assert_eq!((child_ctx.get_start_frame(), child_ctx.get_end_frame()), (3, last), "the section must not shrink when the same key re-fires");
}

#[test]
fn independent_child_finish_is_observable_on_the_child_context() {
    // An independent child is advanced through `child_ctx.update`, so the
    // child's own `is_finished` already reports its completion — no
    // dedicated field on `ss_instance_step_result` is needed for it. Synced
    // children are seeked via `set_frame_relative` and have no equivalent.
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut child_ctx = Context::new();
    child_ctx.bind_resource(&resource);
    child_ctx.setup_animation("walk").unwrap();

    let mut slot = InstanceSlot::new();
    let delta = 1.0 / child_ctx.get_frame_rate() as f32;
    let cycle = (child_ctx.get_end_frame() - child_ctx.get_start_frame() + 1) as usize;

    let ev = crate::types::ss_event_instance_info { event_frame: 0, loop_num: 1, speed: 1.0, independent: true, is_synthetic_default: false, ..Default::default() };

    let r = slot.step(ev, &mut child_ctx, 0.0, 0.0, false);
    assert!(r.transitioned);
    child_ctx.play();
    assert!(!child_ctx.is_finished(), "child must not start finished");

    for _ in 0..cycle + 2 {
        slot.step(ev, &mut child_ctx, 0.0, delta, false);
    }

    assert!(child_ctx.is_finished(), "a single-loop independent child reports `is_finished` on its own context once the loop completes");
}

#[test]
fn the_speed_a_step_used_survives_the_ignored_implicit_key() {
    // An independent key after frame 0, with a speed other than 1. Once the
    // parent loops back before it, the host resolves the implicit frame-0 key
    // (speed 1) again, and the slot ignores it and keeps the real key's speed.
    // A host that scales the child's own sub-tree by the resolved event runs
    // the grandchildren at 1 while the child runs at 2; `get_speed` is what it
    // has to read instead.
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut child_ctx = Context::new();
    child_ctx.bind_resource(&resource);
    child_ctx.setup_animation("walk").unwrap();

    let mut slot = InstanceSlot::new();
    assert_eq!(slot.get_speed(), 1.0, "a fresh slot has driven nothing yet");
    let delta = 1.0 / child_ctx.get_frame_rate() as f32;

    let key = crate::types::ss_event_instance_info { event_frame: 20, loop_num: -1, speed: 2.0, independent: true, is_synthetic_default: false, ..Default::default() };
    let r = slot.step(key, &mut child_ctx, 20.0, 0.0, false);
    assert!(r.transitioned);
    child_ctx.play();
    slot.step(key, &mut child_ctx, 21.0, delta, false);
    assert_eq!(slot.get_speed(), 2.0);

    let implicit = crate::types::ss_event_instance_info::synthetic_default();
    let before = child_ctx.get_frame_no();
    let r = slot.step(implicit, &mut child_ctx, 0.0, delta, true);
    assert!(!r.transitioned && r.independent, "the implicit key must not re-transition an independent child");
    assert_eq!(implicit.speed, 1.0);
    assert_eq!(slot.get_speed(), 2.0, "the slot keeps the real key's speed while it ignores the implicit key");
    assert!((child_ctx.get_frame_no() - before - 2.0).abs() < 1e-3, "the child advanced at the speed get_speed reports");
}

#[test]
fn the_speed_a_step_used_is_the_keys_otherwise() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut child_ctx = Context::new();
    child_ctx.bind_resource(&resource);
    child_ctx.setup_animation("walk").unwrap();

    let mut slot = InstanceSlot::new();
    let synced = crate::types::ss_event_instance_info { event_frame: 0, loop_num: -1, speed: 0.5, independent: false, is_synthetic_default: false, ..Default::default() };
    slot.step(synced, &mut child_ctx, 0.0, 0.0, false);
    assert_eq!(slot.get_speed(), 0.5);

    slot.reset();
    assert_eq!(slot.get_speed(), 1.0, "reset forgets the speed with the rest");
}
