//! The C ABI's handles, used the way a Player uses them and in the orders the
//! header allows: a resource destroyed while a context or a slot still holds
//! it, and a context rebound, reloaded or reset while nothing else holds its
//! old resource. CI also runs this under Miri, which reports a use after free
//! or a dangling borrow that a plain run lets pass.
#![cfg(feature = "ffi")]

mod common;

use core::ffi::c_void;
use std::ptr::null;

use ssab::ssab::ss::format as fbs;
use ssruntime::ffi::*;

fn pack(name: &str) -> Vec<u8> {
    let path = common::converted("overall").join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn binary(bytes: &[u8]) -> fbs::SsAnimeBinary<'_> {
    fbs::root_as_ss_anime_binary(bytes).expect("verify pack")
}

fn first_animation(bytes: &[u8]) -> u32 {
    binary(bytes).animations().get(0).name_hash()
}

/// Reads every byte of a buffer the runtime handed out, so that Miri checks it
/// is still live.
fn touch(ptr: *const u8, len: usize) {
    assert!(!ptr.is_null() && len > 0);
    std::hint::black_box(unsafe { std::slice::from_raw_parts(ptr, len) }.iter().fold(0u8, |a, &b| a ^ b));
}

fn play(ctx: *mut c_void, animation_hash: u32, frames: usize) {
    assert!(unsafe { ss_runtime_setup_animation_by_hash(ctx, animation_hash) });
    ss_runtime_play(ctx);
    for _ in 0..frames {
        let frame = ss_runtime_update(ctx, 1000.0 / 30.0);
        let (mut data, mut len) = (null(), 0);
        unsafe { ss_runtime_get_frame_data(ctx, frame, &mut data, &mut len) };
        touch(data, len);
    }
}

#[test]
fn a_context_keeps_its_resource_after_the_handle_is_destroyed() {
    let bytes = pack("Basic.ssab");
    let animation = first_animation(&bytes);
    let res = unsafe { ss_resource_create_copy(bytes.as_ptr(), bytes.len()) };
    assert!(!res.is_null());
    drop(bytes);
    let ctx = ss_runtime_create();
    assert!(ss_runtime_bind_resource(ctx, res));
    ss_resource_destroy(res);
    play(ctx, animation, 3);
    let (mut data, mut len) = (null(), 0);
    unsafe { ss_runtime_get_ssab(ctx, &mut data, &mut len) };
    touch(data, len);
    ss_runtime_destroy(ctx);
}

#[test]
fn a_context_lets_its_old_resource_go_when_rebound_reloaded_or_reset() {
    let (a, b) = (pack("Basic.ssab"), pack("Mesh.ssab"));
    let ctx = ss_runtime_create();
    for bytes in [&a, &b, &a] {
        let res = unsafe { ss_resource_create_copy(bytes.as_ptr(), bytes.len()) };
        assert!(ss_runtime_bind_resource(ctx, res));
        // The context now holds the only reference, which the next bind releases.
        ss_resource_destroy(res);
        play(ctx, first_animation(bytes), 2);
    }
    assert!(unsafe { ss_runtime_load_ssab_copy(ctx, b.as_ptr(), b.len()) });
    play(ctx, first_animation(&b), 2);
    assert!(unsafe { ss_runtime_load_ssab_borrow(ctx, a.as_ptr(), a.len()) });
    play(ctx, first_animation(&a), 2);
    ss_runtime_reset(ctx);
    assert!(unsafe { ss_runtime_load_ssab_borrow(ctx, b.as_ptr(), b.len()) });
    play(ctx, first_animation(&b), 2);
    ss_runtime_destroy(ctx);
}

#[test]
fn an_effect_slot_keeps_its_resource_after_the_handle_and_the_context_are_destroyed() {
    let bytes = pack("Effect.ssab");
    let effects: Vec<(u32, u32)> = binary(&bytes).parts().iter().enumerate().filter_map(|(i, part)| part.part_type_as_part_type_effect().map(|effect| (i as u32, effect.ref_effect_name_hash()))).collect();
    assert!(!effects.is_empty(), "fixture sanity: Effect has effect parts");
    let res = unsafe { ss_resource_create_borrow(bytes.as_ptr(), bytes.len()) };
    let ctx = ss_runtime_create();
    assert!(ss_runtime_bind_resource(ctx, res));
    let slots: Vec<(u32, *mut c_void)> = effects.iter().map(|&(part, hash)| (part, ss_effect_slot_create(res, hash, part))).collect();
    assert!(slots.iter().all(|(_, slot)| !slot.is_null()));
    ss_resource_destroy(res);

    play(ctx, first_animation(&bytes), 0);
    let fps = ss_runtime_get_frame_rate(ctx) as f32;
    let mut drawn = 0;
    for _ in 0..6 {
        let frame = ss_runtime_update(ctx, 1000.0 / fps);
        let looped = ss_runtime_just_looped(ctx);
        for &(part, slot) in &slots {
            let r = ss_effect_slot_step(slot, ss_runtime_get_active_effect_event(ctx, part), frame, 1.0 / fps, fps, looped, false, 2, true);
            if r.visible {
                touch(r.draw_plan_buf, r.draw_plan_len);
                drawn += 1;
            }
        }
    }
    assert!(drawn > 0, "fixture sanity: an effect drew");

    ss_runtime_destroy(ctx);
    for (_, slot) in slots {
        ss_effect_slot_reset(slot);
        ss_effect_slot_destroy(slot);
    }
}

#[test]
fn an_instance_child_plays_from_a_resource_whose_handle_is_destroyed() {
    let bytes = pack("Instance.ssab");
    let instances: Vec<(u32, String, u32)> =
        binary(&bytes).parts().iter().enumerate().filter_map(|(i, part)| part.part_type_as_part_type_instance().map(|instance| (i as u32, instance.ref_anime_pack().to_string(), instance.ref_anime_name_hash()))).collect();
    assert!(!instances.is_empty(), "fixture sanity: Instance has instance parts");
    let ctx = ss_runtime_create();
    assert!(unsafe { ss_runtime_load_ssab_copy(ctx, bytes.as_ptr(), bytes.len()) });
    let children: Vec<(u32, *mut c_void, *mut c_void)> = instances
        .iter()
        .map(|(part, child_pack, animation)| {
            let child_bytes = pack(&format!("{child_pack}.ssab"));
            let res = unsafe { ss_resource_create_copy(child_bytes.as_ptr(), child_bytes.len()) };
            let child = ss_runtime_create();
            assert!(ss_runtime_bind_resource(child, res));
            ss_resource_destroy(res);
            assert!(unsafe { ss_runtime_setup_animation_by_hash(child, *animation) });
            (*part, ss_instance_slot_create(), child)
        })
        .collect();

    play(ctx, first_animation(&bytes), 0);
    let mut drawn = 0;
    for _ in 0..6 {
        let frame = ss_runtime_update(ctx, 1000.0 / 30.0);
        let looped = ss_runtime_just_looped(ctx);
        for &(part, slot, child) in &children {
            let r = ss_instance_slot_step(slot, ss_runtime_get_active_event_instance(ctx, part), child, frame, 1.0 / 30.0, looped);
            if r.transitioned {
                ss_runtime_play(child);
            }
            if r.visible {
                let (mut data, mut len) = (null(), 0);
                unsafe { ss_runtime_get_frame_data(child, r.child_frame_no, &mut data, &mut len) };
                touch(data, len);
                drawn += 1;
            }
        }
    }
    assert!(drawn > 0, "fixture sanity: an instance child drew");

    ss_runtime_destroy(ctx);
    for (_, slot, child) in children {
        ss_instance_slot_destroy(slot);
        ss_runtime_destroy(child);
    }
}
