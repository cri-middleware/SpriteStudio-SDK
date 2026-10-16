//! Track tests: sources, track alpha and blend, crossfades and fades.

use super::*;
use crate::mix::MIX_ATTR_FLAGS;

fn evaluate_ringo(resource: &AnimationResource<'_>, anim: &str) -> Vec<(f32, f32, f32, f32)> {
    let mut ctx = Context::new();
    ctx.bind_resource(resource);
    ctx.setup_animation(anim).unwrap_or_else(|_| panic!("setup_animation({anim}) failed"));
    let start = ctx.instance().primary_source().controller.get_start_frame() as f32;
    let _ = ctx.get_frame_data(start).expect("get_frame_data failed");
    ctx.instance().resolved_layer.parts_vec.iter().map(|p| (p.position_x(), p.position_y(), p.rotation_z(), p.alpha())).collect()
}

/// Sample the resolved pose of every part, for comparing two setups.
fn resolved_of(ctx: &Context<'_>) -> Vec<(f32, f32, f32, f32)> {
    ctx.instance().resolved_layer.parts_vec.iter().map(|p| (p.position_x(), p.position_y(), p.rotation_z(), p.alpha())).collect()
}

/// Two sources on one track blend by weight, exactly as they did before the
/// stack existed — `add_track_source` is the locomotion path.
#[test]
fn two_sources_on_one_track_blend_by_weight() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let walk = evaluate_ringo(&resource, "walk");
    let run = evaluate_ringo(&resource, "run");

    let (mut ctx, walk_start) = ringo_ctx(&resource, "walk");
    let handle = ctx.add_track_source(0, "run", 1.0).expect("add_track_source(run)");
    let run_start = ctx.instance().sources[handle as usize].controller.get_start_frame();
    ctx.set_source_frame(handle, run_start as f32).unwrap();
    ctx.get_frame_data(walk_start).expect("get_frame_data");
    let mixed = resolved_of(&ctx);

    for i in 0..walk.len() {
        let (wx, wy, wr, wa) = walk[i];
        let (rx, ry, rr, ra) = run[i];
        let (mx, my, mr, ma) = mixed[i];
        assert!((mx - 0.5 * (wx + rx)).abs() < 1e-3, "part {i} position_x");
        assert!((my - 0.5 * (wy + ry)).abs() < 1e-3, "part {i} position_y");
        assert!((mr - 0.5 * (wr + rr)).abs() < 1e-3, "part {i} rotation_z");
        assert!((ma - 0.5 * (wa + ra)).abs() < 1e-3, "part {i} alpha");
    }
}

#[test]
fn zero_weight_source_contributes_nothing() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let walk_only = evaluate_ringo(&resource, "walk");

    let (mut ctx, walk_start) = ringo_ctx(&resource, "walk");
    ctx.add_track_source(0, "run", 0.0).unwrap();
    ctx.get_frame_data(walk_start).expect("get_frame_data");
    assert_eq!(walk_only, resolved_of(&ctx));
}

/// A second track at alpha 0 must be inaudible, and at alpha 1 must take over
/// the attributes it animates.
#[test]
fn second_track_alpha_controls_how_much_of_it_lands() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let walk_only = evaluate_ringo(&resource, "walk");

    let (mut ctx, walk_start) = ringo_ctx(&resource, "walk");
    ctx.set_track_animation(1, "run").unwrap();
    ctx.set_track_frame(1, ctx.get_track_frame(1).unwrap()).unwrap();

    ctx.set_track_alpha(1, 0.0).unwrap();
    ctx.get_frame_data(walk_start).expect("get_frame_data");
    assert_eq!(walk_only, resolved_of(&ctx), "an alpha-0 track must not be audible");

    ctx.set_track_alpha(1, 1.0).unwrap();
    ctx.get_frame_data(walk_start).expect("get_frame_data");
    assert_ne!(walk_only, resolved_of(&ctx), "at full alpha it must take over what it animates");
}

/// A part's coverage is the attributes it keys among those the mix blends,
/// so the two lists of them have to name the same ones.
#[test]
fn coverage_names_the_attributes_the_mix_blends() {
    let blended = MIX_ATTR_FLAGS.iter().fold(runtime::framedata::ss::runtime::UpdateAttributeFlags::empty(), |all, f| all | *f);
    assert_eq!(blended, crate::eval::framedata::MIXED_ATTRIBUTES);
}

/// Track 0 faded to alpha 0 resolves to the setup pose. This is what replaced
/// the empty blend source: the stack's floor *is* the setup pose, so fading
/// out needs no placeholder animation.
#[test]
fn track_zero_at_alpha_zero_resolves_to_the_setup_pose() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, start) = ringo_ctx(&resource, "walk");

    ctx.set_track_alpha(0, 0.0).unwrap();
    ctx.get_frame_data(start).expect("get_frame_data");

    let walk = &resource.animations()[resource.find_animation_index("walk").unwrap()];
    let setup = walk.setup_pose(ctx.instance().coordinate_system, ctx.instance().origin, ctx.instance().unit).to_vec();
    for (i, part) in ctx.instance().resolved_layer.parts_vec.iter().enumerate() {
        assert!((part.position_x() - setup[i].position_x()).abs() < 1e-4, "part {i} position_x: resolved={} setup={}", part.position_x(), setup[i].position_x());
        assert!((part.rotation_z() - setup[i].rotation_z()).abs() < 1e-4, "part {i} rotation_z");
        assert!((part.scale_x() - setup[i].scale_x()).abs() < 1e-4, "part {i} scale_x");
    }
}

#[test]
fn crossfade_hands_the_track_over_and_releases_the_old_source() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");
    // Fades run on the character's clock, so it has to be playing.
    ctx.play();
    let walk = ctx.instance().tracks[0].main;

    ctx.crossfade(0, "run", 1000.0).unwrap();
    assert_eq!(ctx.get_track_fade_progress(0), Some(0.0));
    assert_eq!(ctx.get_source_weight(walk), Some(1.0));

    ctx.update(500.0);
    let p = ctx.get_track_fade_progress(0).unwrap();
    assert!((p - 0.5).abs() < 1e-3, "progress {p}");
    assert!((ctx.get_source_weight(walk).unwrap() - 0.5).abs() < 1e-3);

    ctx.update(600.0);
    assert!(ctx.get_track_fade_progress(0).is_none());
    assert_ne!(ctx.instance().tracks[0].main, walk);
    assert!(ctx.get_source_weight(walk).is_none(), "the outgoing source is released");
    assert_eq!(ctx.instance().primary, ctx.instance().tracks[0].main, "track 0 still carries the structure");
}

/// `update` returns the frame of whichever source is primary when it
/// returns, and a crossfade on track 0 changes which one that is. Returning
/// the outgoing animation's frame handed the caller a number
/// `get_frame_data` then rejected — `wait1` (80 frames) to `attack1` (20)
/// failed at frame 45, which is the documented `get_frame_data(update(dt))`
/// pairing breaking on a perfectly ordinary transition.
#[test]
fn update_returns_a_frame_get_frame_data_accepts_across_a_crossfade() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "wait1");
    ctx.set_loop_count(-1);
    ctx.play();

    // Get well past `attack1`'s last frame before handing the track over.
    for _ in 0..45 {
        let frame = ctx.update(1000.0 / 60.0);
        ctx.get_frame_data(frame).expect("frame data while playing");
    }
    assert!(ctx.get_frame_no() > 20.0, "frame {} is not past attack1's range", ctx.get_frame_no());

    ctx.crossfade(0, "attack1", 400.0).unwrap();
    for tick in 0..40 {
        let frame = ctx.update(1000.0 / 60.0);
        ctx.get_frame_data(frame).unwrap_or_else(|e| panic!("tick {tick}: get_frame_data({frame}) failed: {e:?}"));
    }
    assert!(ctx.get_track_fade_progress(0).is_none(), "the crossfade should have completed");
}

/// Weights of the live sources on `track`, keyed by handle.
fn track_weights(ctx: &Context<'_>, track: u16) -> Vec<(u32, f32)> {
    ctx.instance().sources.iter().enumerate().filter(|(_, s)| s.animation_idx >= 0 && s.track == track).map(|(i, s)| (i as u32, s.weight)).collect()
}

fn weight_of(ctx: &Context<'_>, handle: u32) -> f32 {
    ctx.get_source_weight(handle).unwrap_or_else(|| panic!("source {handle} was released"))
}

/// Restarting a crossfade folds the one in flight into the new outgoing side
/// at the weights it reached. Completing it instead snapped the pose the
/// moment the restart landed.
#[test]
fn crossfade_restart_folds_the_fade_in_flight() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");
    ctx.play();
    let walk = ctx.instance().tracks[0].main;

    ctx.crossfade(0, "run", 1000.0).unwrap();
    let run = ctx.instance().sources.iter().enumerate().position(|(i, s)| i as u32 != walk && s.animation_idx >= 0).unwrap() as u32;
    ctx.update(300.0);
    ctx.crossfade(0, "dance", 1000.0).unwrap();

    assert_eq!(ctx.get_track_fade_progress(0), Some(0.0));
    assert!((weight_of(&ctx, walk) - 0.7).abs() < 1e-3, "walk keeps what it had reached");
    assert!((weight_of(&ctx, run) - 0.3).abs() < 1e-3, "run keeps what it had reached");
    assert_eq!(track_weights(&ctx, 0).len(), 3, "both sides of the first fade are now outgoing");

    ctx.update(500.0);
    assert!((weight_of(&ctx, walk) - 0.35).abs() < 1e-3);
    assert!((weight_of(&ctx, run) - 0.15).abs() < 1e-3);
    let sum: f32 = track_weights(&ctx, 0).iter().map(|(_, w)| w).sum();
    assert!((sum - 1.0).abs() < 1e-3, "the track's weights sum to {sum}");

    ctx.update(600.0);
    let left = track_weights(&ctx, 0);
    assert_eq!(left.len(), 1, "every outgoing source is released on completion: {left:?}");
    assert_eq!(left[0], (ctx.instance().tracks[0].main, 1.0));
}

/// The incoming animation takes the main role, and on track 0 the primary,
/// the moment the crossfade is called: the frame and the discrete attributes
/// are its own from the first frame of the fade, while its weight is still 0.
#[test]
fn a_crossfade_hands_the_main_role_over_when_it_begins() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "wait1");
    ctx.play();
    let wait = ctx.instance().tracks[0].main;

    ctx.crossfade(0, "attack1", 1000.0).unwrap();
    let attack = ctx.get_track_main_source(0).unwrap();
    assert_ne!(attack, wait, "the incoming animation is the track's main one at once");
    assert_eq!(ctx.instance().primary, attack, "and on track 0, the primary");
    assert_eq!(ctx.get_source_animation_index(attack), Some(resource.find_animation_index("attack1").unwrap() as u32));
    assert_eq!(ctx.get_source_weight(attack), Some(0.0), "while its pose has not started to show");
    assert_eq!(ctx.get_source_weight(wait), Some(1.0));

    let frame = ctx.update(1000.0 / 60.0);
    ctx.get_frame_data(frame).expect("the frame update returns is the incoming animation's");
    assert_eq!(ctx.instance().resolved_layer.cell_ids, ctx.instance().layers[attack as usize].cell_ids, "the discrete attributes are the incoming animation's");
}

/// An outgoing animation that runs out mid-fade does not finish the
/// character. Reporting from it until the fade completed said finished, then
/// playing again once the incoming one took over — with nothing to say it
/// had started again.
#[test]
fn an_outgoing_animation_running_out_does_not_finish_the_character() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "attack1");
    ctx.set_loop_count(1);
    ctx.play();

    ctx.crossfade(0, "wait1", 5000.0).unwrap();
    for _ in 0..120 {
        ctx.update(1000.0 / 60.0);
        assert!(!ctx.is_finished(), "the character finished with the outgoing animation");
        assert!(ctx.is_playing());
    }
    assert!(ctx.get_track_fade_progress(0).is_some(), "the fade is still running, outlasting attack1");
}

/// A crossfade's incoming animation is played the way the host plays track
/// 0, as `setup_animation` keeps it: the speed, the rate, the frame skip and
/// the direction carry over; the loop count is the animation's, and starts
/// over. Laid at the defaults instead, it reset the host's speed the moment
/// the fade began.
#[test]
fn a_crossfade_plays_the_incoming_animation_the_way_track_zero_is_played() {
    use crate::eval::playback_controller::{PlaybackDirection, PlaybackStyle};
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");
    ctx.set_speed_scale(2.0);
    ctx.set_frame_rate(30);
    ctx.set_skip_frames(false);
    ctx.set_playback_direction(PlaybackDirection::Backward, PlaybackStyle::PingPong);
    ctx.set_loop_count(3);
    ctx.play();

    ctx.crossfade(0, "run", 1000.0).unwrap();
    let run = &ctx.instance().sources[ctx.get_track_main_source(0).unwrap() as usize].controller;
    assert_eq!(run.get_speed_scale(), 2.0);
    assert_eq!(run.get_frame_rate(), 30);
    assert!(!run.get_skip_frames());
    assert_eq!(run.get_playback_direction(), PlaybackDirection::Backward);
    assert_eq!(run.get_playback_style(), PlaybackStyle::PingPong);
    assert_eq!(run.get_loop_count(), -1, "the loop count is restated per animation");
    assert_eq!(ctx.get_speed_scale(), 2.0, "track 0 reports what it plays at");
    assert_eq!(ctx.get_track_speed(0), Some(2.0));
}

/// A track's settings reach every source on it, both sides of a fade
/// included, and outlive the animation on it.
#[test]
fn a_track_keeps_its_speed_across_the_animations_laid_on_it() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");
    ctx.play();

    // Set before anything is laid there: the track keeps it until something is.
    ctx.set_track_speed(1, 0.5).unwrap();
    ctx.set_track_animation(1, "run").unwrap();
    let speed_of = |ctx: &Context<'_>, track: u32| ctx.instance().sources[ctx.get_track_main_source(track).unwrap() as usize].controller.get_speed_scale();
    assert_eq!(speed_of(&ctx, 1), 0.5);

    ctx.set_track_animation(1, "dance").unwrap();
    assert_eq!(speed_of(&ctx, 1), 0.5, "a new animation on the track keeps the track's speed");

    ctx.crossfade(1, "walk", 1000.0).unwrap();
    ctx.set_track_speed(1, 0.25).unwrap();
    for (handle, _) in track_weights(&ctx, 1) {
        assert_eq!(ctx.instance().sources[handle as usize].controller.get_speed_scale(), 0.25, "source {handle} missed the change");
    }
    assert_eq!(ctx.get_track_speed(1), Some(0.25));
    assert_eq!(ctx.get_speed_scale(), 1.0, "track 0 is untouched");

    ctx.set_track_speed(1, -1.0).unwrap();
    assert_eq!(ctx.get_track_speed(1), Some(0.0), "a negative speed is stored as a stop");
}

/// A track above 0 can be told how many times to play, and says when it is
/// done, while track 0 plays on. The track then holds its last frame.
#[test]
fn an_overlay_track_finishes_on_its_own_loop_count() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "wait1");
    ctx.play();
    ctx.set_track_animation(1, "attack1").unwrap();
    assert_eq!(ctx.get_track_loop_count(1), Some(-1), "an animation laid on a track loops forever");
    ctx.set_track_loop_count(1, 1).unwrap();
    assert_eq!(ctx.get_track_loop_count(1), Some(1));

    let mut finished_at = None;
    for tick in 0..120 {
        ctx.update(1000.0 / 60.0);
        if ctx.is_track_finished(1) {
            finished_at = Some(tick);
            break;
        }
    }
    assert!(finished_at.is_some(), "attack1 never finished on track 1");
    let held = ctx.get_track_frame(1);
    ctx.update(100.0);
    assert!(ctx.is_track_finished(1), "a finished track stays finished");
    assert_eq!(ctx.get_track_frame(1), held, "and holds its last frame");
    assert!(!ctx.is_finished() && ctx.is_playing(), "track 0 plays on");
    assert!(!ctx.is_track_finished(2), "a track with nothing on it has not finished");
    assert!(ctx.set_track_loop_count(2, 1).is_err(), "nor can it be given a loop count");
}

/// An overlay that loops reports the wrap on the update it happens, as
/// `just_looped` does for track 0.
#[test]
fn an_overlay_track_reports_its_own_loop() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "wait1");
    ctx.play();
    ctx.set_track_animation(1, "attack1").unwrap();

    let looped = (0..120).filter(|_| {
        ctx.update(1000.0 / 60.0);
        ctx.track_just_looped(1)
    });
    assert!(looped.count() > 0, "attack1 never wrapped on track 1");
}

/// `setup_animation` keeps track 0's settings even once a crossfade has
/// moved the primary off slot 0. Carried from slot 0, they came from a
/// released source at the defaults.
#[test]
fn setup_keeps_track_zero_s_settings_after_a_crossfade() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");
    ctx.set_speed_scale(2.0);
    ctx.play();
    ctx.crossfade(0, "run", 100.0).unwrap();
    ctx.update(200.0);
    assert_ne!(ctx.instance().primary, 0, "the crossfade moved the primary off slot 0");

    ctx.setup_animation("dance").unwrap();
    assert_eq!(ctx.get_speed_scale(), 2.0);
    assert_eq!(ctx.instance().primary_source().controller.get_speed_scale(), 2.0);
}

/// What a cancelled fade leaves on the track fades out with the next one,
/// in the proportions it was left at. Fading out only the main source left
/// the other one mixing at its stale weight, which jumped the pose when the
/// fade began and again when it completed and the leftover was released.
#[test]
fn a_crossfade_after_a_cancel_carries_the_leftover_out() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");
    ctx.play();
    let walk = ctx.instance().tracks[0].main;

    ctx.crossfade(0, "run", 1000.0).unwrap();
    ctx.update(250.0);
    ctx.cancel_track_fade(0).unwrap();
    let before = track_weights(&ctx, 0);

    ctx.crossfade(0, "dance", 1000.0).unwrap();
    for (handle, weight) in &before {
        assert!((weight_of(&ctx, *handle) - weight).abs() < 1e-3, "source {handle} jumped when the fade began");
    }
    assert!((weight_of(&ctx, walk) - 0.75).abs() < 1e-3);

    ctx.update(1100.0);
    let left = track_weights(&ctx, 0);
    assert_eq!(left.len(), 1, "the leftover is released with the rest: {left:?}");
}

/// A track built from several sources fades out as a whole, keeping their
/// proportions, whatever scale the host gave the weights in.
#[test]
fn a_crossfade_from_a_mixed_track_fades_every_source_out() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");
    ctx.play();
    let walk = ctx.instance().tracks[0].main;
    ctx.set_source_weight(walk, 3.0).unwrap();
    let run = ctx.add_track_source(0, "run", 1.0).unwrap();

    ctx.crossfade(0, "dance", 1000.0).unwrap();
    assert!((weight_of(&ctx, walk) - 0.75).abs() < 1e-3, "3 : 1 normalises to 0.75 : 0.25");
    assert!((weight_of(&ctx, run) - 0.25).abs() < 1e-3);

    ctx.update(500.0);
    assert!((weight_of(&ctx, walk) - 0.375).abs() < 1e-3);
    assert!((weight_of(&ctx, run) - 0.125).abs() < 1e-3);

    ctx.update(600.0);
    assert!(ctx.get_source_weight(walk).is_none());
    assert!(ctx.get_source_weight(run).is_none());
}

#[test]
fn crossfade_on_an_empty_track_just_starts_playing() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");

    ctx.crossfade(1, "run", 500.0).unwrap();
    assert!(ctx.get_track_fade_progress(1).is_none(), "nothing to fade from");
    assert_eq!(ctx.get_track_alpha(1), Some(1.0));
    assert!(ctx.get_track_frame(1).is_some());
}

#[test]
fn fade_track_drives_alpha_to_its_target() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");
    // Fades run on the character's clock, so it has to be playing.
    ctx.play();

    ctx.fade_track(0, 0.0, 400.0).unwrap();
    ctx.update(200.0);
    let a = ctx.get_track_alpha(0).unwrap();
    assert!((a - 0.5).abs() < 1e-3, "alpha {a}");

    ctx.update(300.0);
    assert_eq!(ctx.get_track_alpha(0), Some(0.0));
    assert!(ctx.get_track_fade_progress(0).is_none());
}

#[test]
fn cancel_track_fade_leaves_the_weights_alone() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");
    let walk = ctx.instance().tracks[0].main;

    ctx.crossfade(0, "run", 1000.0).unwrap();
    ctx.update(250.0);
    let held = ctx.get_source_weight(walk).unwrap();
    ctx.cancel_track_fade(0).unwrap();
    ctx.update(500.0);

    assert!(ctx.get_track_fade_progress(0).is_none());
    assert_eq!(ctx.get_source_weight(walk), Some(held), "weights freeze where the fade stopped");
}

#[test]
fn set_track_animation_replaces_what_the_track_was_playing() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");

    let extra = ctx.add_track_source(0, "run", 1.0).unwrap();
    ctx.set_track_animation(0, "dance").unwrap();

    assert!(ctx.get_source_weight(extra).is_none(), "the replaced animation's extra sources go with it");
    assert_eq!(ctx.instance().primary, ctx.instance().tracks[0].main);
}

#[test]
fn clear_track_silences_a_track_but_never_track_zero() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let walk_only = evaluate_ringo(&resource, "walk");
    let (mut ctx, start) = ringo_ctx(&resource, "walk");

    ctx.set_track_animation(1, "run").unwrap();
    ctx.clear_track(1).unwrap();
    ctx.get_frame_data(start).expect("get_frame_data");
    assert_eq!(walk_only, resolved_of(&ctx));

    assert!(matches!(ctx.clear_track(0), Err(error::SSRuntimeError::InvalidParameter)), "track 0 carries the structure");
}

#[test]
fn setup_animation_collapses_the_stack() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");

    ctx.set_track_animation(2, "run").unwrap();
    ctx.set_track_alpha(2, 0.4).unwrap();
    assert_eq!(ctx.get_track_count(), 3);

    ctx.setup_animation("dance").unwrap();
    assert_eq!(ctx.get_track_count(), 1, "a new setup starts from one track again");
    assert_eq!(ctx.get_track_alpha(0), Some(1.0));
    assert_eq!(ctx.instance().primary, 0);
}

#[test]
fn track_api_validates_state() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");

    // Layering onto an instance with no structure yet is refused.
    let mut ctx = Context::new();
    ctx.bind_resource(&resource);
    assert!(matches!(ctx.set_track_animation(1, "run"), Err(error::SSRuntimeError::AnimationNotFound)));

    ctx.setup_animation("walk").unwrap();
    assert!(matches!(ctx.set_track_animation(1, "not_a_real_anim"), Err(error::SSRuntimeError::AnimationNotFound)));

    // Tracks that do not exist, and out-of-range handles.
    assert!(matches!(ctx.set_track_alpha(9, 1.0), Err(error::SSRuntimeError::InvalidParameter)));
    assert!(ctx.get_track_alpha(9).is_none());
    assert!(matches!(ctx.set_track_frame(9, 0.0), Err(error::SSRuntimeError::InvalidParameter)));
    assert!(matches!(ctx.set_source_weight(99, 1.0), Err(error::SSRuntimeError::InvalidParameter)));
    assert!(ctx.get_source_weight(99).is_none());
    assert!(ctx.get_source_frame(99).is_none());

    // NaN alpha would poison every attribute the track touches.
    assert!(matches!(ctx.set_track_alpha(0, f32::NAN), Err(error::SSRuntimeError::InvalidParameter)));
}

#[test]
fn crossfade_validates_arguments() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");

    assert!(matches!(ctx.crossfade(0, "run", 0.0), Err(error::SSRuntimeError::InvalidParameter)));
    assert!(matches!(ctx.crossfade(0, "run", -10.0), Err(error::SSRuntimeError::InvalidParameter)));
    assert!(matches!(ctx.crossfade(0, "run", f32::NAN), Err(error::SSRuntimeError::InvalidParameter)));
    assert!(matches!(ctx.crossfade(0, "not_a_real_anim", 100.0), Err(error::SSRuntimeError::AnimationNotFound)));
    assert!(matches!(ctx.fade_track(0, f32::NAN, 100.0), Err(error::SSRuntimeError::InvalidParameter)));
    assert!(matches!(ctx.fade_track(9, 0.0, 100.0), Err(error::SSRuntimeError::InvalidParameter)));
}

#[test]
fn bind_resource_clears_track_state() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");
    ctx.set_track_animation(1, "run").unwrap();

    ctx.bind_resource(&resource);
    assert_eq!(ctx.get_track_count(), 1);
    assert_eq!(ctx.instance().primary, 0);
}

/// Only track 0's animation fires events by default; a source opts in by
/// lowering its threshold to its weight or below.
#[test]
fn source_event_threshold_round_trips() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");

    let handle = ctx.add_track_source(0, "run", 1.0).unwrap();
    assert_eq!(ctx.get_source_event_threshold(handle), Some(f32::INFINITY), "default keeps secondaries quiet");
    ctx.set_source_event_threshold(handle, 0.5).unwrap();
    assert_eq!(ctx.get_source_event_threshold(handle), Some(0.5));
    assert!(matches!(ctx.set_source_event_threshold(99, 0.5), Err(error::SSRuntimeError::InvalidParameter)));
}

/// The C ABI reads the source handles through the same index as the event
/// list, and answers `u32::MAX` — a value no live handle takes — outside it.
///
/// The accessor exists on the Rust and wasm faces already; this covers the C
/// one, which had been left out. Without it a native Player blending two
/// animations gets both of their events in one undifferentiated list.
#[cfg(feature = "ffi")]
#[test]
fn passed_event_source_handles_reach_the_c_abi() {
    use crate::ffi::*;
    let ssab = load_ringo_resource();
    let bytes = ssab.as_slice();
    let ctx = ss_runtime_create();
    assert!(unsafe { ss_runtime_load_ssab_copy(ctx, bytes.as_ptr(), bytes.len()) });
    assert!(unsafe { ss_runtime_setup_animation_by_hash(ctx, fnv1a_hash_str("walk")) });
    // Out of range in either direction is the sentinel, not a read past the
    // slice — which is the whole reason this is bounds-checked here.
    assert_eq!(unsafe { ss_runtime_get_passed_event_source_handle(ctx, -1) }, u32::MAX);
    assert_eq!(unsafe { ss_runtime_get_passed_event_source_handle(ctx, 9999) }, u32::MAX);
    assert_eq!(unsafe { ss_runtime_get_passed_event_source_handle(::core::ptr::null_mut(), 0) }, u32::MAX);
    unsafe { ss_runtime_destroy(ctx) };
}

/// The C face maps `SS_RUNTIME_TRACK_SYNC_*` both ways and refuses a value
/// outside them, rather than reading it as one of the two.
#[cfg(feature = "ffi")]
#[test]
fn track_sync_round_trips_through_the_c_abi() {
    use crate::ffi::*;
    let ssab = load_ringo_resource();
    let bytes = ssab.as_slice();
    // The C functions take the context `ss_runtime_create` hands out, not a
    // bare `Context`: they write the last error into it.
    let ctx = ss_runtime_create();
    assert!(unsafe { ss_runtime_load_ssab_copy(ctx, bytes.as_ptr(), bytes.len()) });
    assert!(unsafe { ss_runtime_setup_animation_by_hash(ctx, fnv1a_hash_str("walk")) });

    assert_eq!(unsafe { ss_runtime_get_track_sync(ctx, 0) }, SS_RUNTIME_TRACK_SYNC_FREE);
    assert!(unsafe { ss_runtime_set_track_sync(ctx, 1, SS_RUNTIME_TRACK_SYNC_START) });
    assert_eq!(unsafe { ss_runtime_get_track_sync(ctx, 1) }, SS_RUNTIME_TRACK_SYNC_START);
    assert!(!unsafe { ss_runtime_set_track_sync(ctx, 1, 2) }, "an unknown sync is refused");
    assert_eq!(unsafe { ss_runtime_get_track_sync(ctx, 1) }, SS_RUNTIME_TRACK_SYNC_START, "and leaves the setting alone");
    assert_eq!(unsafe { ss_runtime_get_track_sync(ctx, 99) }, SS_RUNTIME_INVALID_HANDLE);

    let walk = unsafe { ss_runtime_get_track_main_source(ctx, 0) };
    assert!(unsafe { ss_runtime_get_source_animation_index(ctx, walk) } >= 0);
    assert_eq!(unsafe { ss_runtime_get_source_animation_index(ctx, 999) }, -1);
    unsafe { ss_runtime_destroy(ctx) };
}

/// The C face of the track settings maps the direction and style both ways
/// and answers a sentinel no setting reads back as for a track that does
/// not exist or has no animation.
#[cfg(feature = "ffi")]
#[test]
fn track_settings_round_trip_through_the_c_abi() {
    use crate::ffi::*;
    let ssab = load_ringo_resource();
    let bytes = ssab.as_slice();
    let ctx = ss_runtime_create();
    assert_eq!(unsafe { ss_runtime_get_current_animation_index(ctx) }, -1, "nothing is set up yet");
    assert!(unsafe { ss_runtime_load_ssab_copy(ctx, bytes.as_ptr(), bytes.len()) });
    assert!(unsafe { ss_runtime_setup_animation_by_hash(ctx, fnv1a_hash_str("walk")) });
    let walk = unsafe { ss_runtime_get_source_animation_index(ctx, ss_runtime_get_track_main_source(ctx, 0)) };
    assert_eq!(unsafe { ss_runtime_get_current_animation_index(ctx) }, walk);

    assert!(unsafe { ss_runtime_set_track_speed(ctx, 1, 0.5) });
    assert_eq!(unsafe { ss_runtime_get_track_speed(ctx, 1) }, 0.5);
    assert!(unsafe { ss_runtime_set_track_frame_rate(ctx, 1, 24) });
    assert_eq!(unsafe { ss_runtime_get_track_frame_rate(ctx, 1) }, 24);
    assert!(unsafe { ss_runtime_set_track_skip_frames(ctx, 1, false) });
    assert!(!unsafe { ss_runtime_get_track_skip_frames(ctx, 1) });
    assert!(unsafe { ss_runtime_set_track_playback_direction(ctx, 1, 1, 1) });
    assert_eq!((unsafe { ss_runtime_get_track_playback_direction(ctx, 1) }, unsafe { ss_runtime_get_track_playback_style(ctx, 1) }), (1, 1));
    assert!(unsafe { ss_runtime_set_track_outgoing_event_threshold(ctx, 1, 0.5) });
    assert_eq!(unsafe { ss_runtime_get_track_outgoing_event_threshold(ctx, 1) }, 0.5);

    assert!(!unsafe { ss_runtime_set_track_loop_count(ctx, 1, 2) }, "no animation on track 1 to count the loops of");
    assert_eq!(unsafe { ss_runtime_get_track_loop_count(ctx, 1) }, 0);
    assert!(!unsafe { ss_runtime_is_track_finished(ctx, 1) } && !unsafe { ss_runtime_track_just_looped(ctx, 1) });
    assert!(unsafe { ss_runtime_set_track_animation_by_hash(ctx, 1, fnv1a_hash_str("run")) });
    assert_eq!(unsafe { ss_runtime_get_track_loop_count(ctx, 1) }, -1, "laid looping forever");
    assert!(unsafe { ss_runtime_set_track_loop_count(ctx, 1, 2) });
    assert_eq!(unsafe { ss_runtime_get_track_loop_count(ctx, 1) }, 2);

    assert!(unsafe { ss_runtime_get_track_speed(ctx, 99) }.is_nan());
    assert_eq!(unsafe { ss_runtime_get_track_frame_rate(ctx, 99) }, -1);
    assert_eq!((unsafe { ss_runtime_get_track_playback_direction(ctx, 99) }, unsafe { ss_runtime_get_track_playback_style(ctx, 99) }), (-1, -1));
    assert!(unsafe { ss_runtime_get_track_outgoing_event_threshold(ctx, 99) }.is_nan());
    unsafe { ss_runtime_destroy(ctx) };
}

#[test]
fn set_track_speed_and_frame_target_the_tracks_animation() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");
    ctx.set_track_animation(1, "run").unwrap();

    ctx.set_track_frame(1, 3.0).unwrap();
    assert_eq!(ctx.get_track_frame(1), Some(3.0));

    ctx.set_track_speed(1, 0.0).unwrap();
    ctx.update(1000.0);
    assert_eq!(ctx.get_track_frame(1), Some(3.0), "speed 0 parks the track");
}
