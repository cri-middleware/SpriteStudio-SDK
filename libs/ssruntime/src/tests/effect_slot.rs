//! `EffectSlot` tests.

use super::*;
use crate::effect_slot::particle_quad;
use crate::eval::geometry::compute_local_vertices;

/// A 100 x 200 cell whose pivot sits a quarter right and a quarter up of its
/// centre: the quad is the box a Normal part draws that cell in, and the corners
/// the texture's top maps to stay on top in either Y direction.
#[test]
fn a_particle_quad_is_the_normal_part_box_in_both_y_directions() {
    let (vx, vy) = compute_local_vertices((100.0, 200.0), (0.25, 0.25), (0.0, 0.0), (false, false), None);
    assert_eq!((vx[0], vy[0]), (-75.0, 50.0));

    let (xs, ys) = particle_quad(vx[0], vy[0], 100.0, 200.0, 1.0);
    assert_eq!(xs, [-75.0, 25.0, -75.0, 25.0]);
    assert_eq!(ys, [50.0, 50.0, -150.0, -150.0], "Y-up: the top corners are the higher ones");

    let (xs, ys) = particle_quad(vx[0], vy[0], 100.0, 200.0, -1.0);
    assert_eq!(xs, [-75.0, 25.0, -75.0, 25.0]);
    assert_eq!(ys, [-50.0, -50.0, 150.0, 150.0], "Y-down: the top corners are the lower ones");
}

/// A slot stepped with `draw == false` -- its part hidden -- moves its clock
/// and transitions as a drawn one does, so once it draws again it builds the
/// plan of a slot that drew every frame.
#[test]
fn an_undrawn_effect_step_keeps_the_slot_in_step_with_a_drawn_one() {
    prepare();
    let input_buf = std::fs::read(fixture("overall/Effect.ssab")).expect("read pack");
    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");
    let binary = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_data.as_slice()) };
    let effects: Vec<(u32, u32)> = binary.parts().iter().enumerate().filter_map(|(i, p)| p.part_type_as_part_type_effect().map(|e| (i as u32, e.ref_effect_name_hash()))).collect();
    assert!(!effects.is_empty(), "fixture sanity: Effect has effect parts");

    let mut ctx = Context::new();
    ctx.bind_resource(&resource);
    ctx.setup_animation("anime_1").unwrap();
    ctx.play();
    let fps = ctx.get_frame_rate() as f32;
    let dt = 1.0 / fps;
    let new_slots = || effects.iter().map(|&(part, hash)| EffectSlot::new(&resource, hash, part ^ hash).expect("effect slot")).collect::<Vec<_>>();
    let (mut drawn, mut hidden) = (new_slots(), new_slots());
    let (mut compared, mut independent) = (0, false);
    for f in 0..120 {
        let frame = ctx.update(dt * 1000.0);
        let looped = ctx.just_looped();
        // Hidden from frame 10 to 49, as a part keyed hidden or force-hidden is.
        let draw = !(10..50).contains(&f);
        for (k, &(part, _)) in effects.iter().enumerate() {
            let ev = ctx.get_active_effect_event(part);
            independent |= ev.independent;
            let a = drawn[k].step(ev, frame, dt, fps, looped, false, 2, true);
            let a_plan = if a.visible { unsafe { std::slice::from_raw_parts(a.draw_plan_buf, a.draw_plan_len) }.to_vec() } else { Vec::new() };
            let b = hidden[k].step(ev, frame, dt, fps, looped, false, 2, draw);
            if !draw {
                assert!(!b.visible, "an undrawn step builds nothing (frame {f}, part {part})");
                continue;
            }
            assert_eq!(a.visible, b.visible, "frame {f}, part {part}");
            if b.visible {
                assert_eq!(a_plan.as_slice(), unsafe { std::slice::from_raw_parts(b.draw_plan_buf, b.draw_plan_len) }, "frame {f}, part {part}");
                if f >= 50 {
                    compared += 1;
                }
            }
        }
    }
    assert!(independent, "fixture sanity: anime_1 plays an effect independently");
    assert!(compared > 0, "fixture sanity: effects draw after the hidden stretch");
}
