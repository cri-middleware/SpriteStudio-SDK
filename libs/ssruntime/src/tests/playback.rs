//! Playback tests: track 0's settings and the character's clock.

use super::*;

// --- Playback settings ---

#[test]
fn track_zero_settings_read_back_before_an_animation_is_laid() {
    // A Player applies its transport when the pack is bound, before it
    // selects an animation; the setters keep it on track 0 alone then, and
    // the getters must not read the default of a controller that has none.
    prepare();
    let ssab_path = fixture("Ringo/Ringo.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Ringo.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");

    let mut context = Context::new();
    context.bind_resource(&resource);
    context.set_speed_scale(2.0);
    context.set_skip_frames(false);
    context.set_playback_direction(eval::playback_controller::PlaybackDirection::Backward, eval::playback_controller::PlaybackStyle::PingPong);
    context.set_frame_rate(24);

    assert_eq!(context.get_speed_scale(), 2.0);
    assert!(!context.get_skip_frames());
    assert!(context.get_playback_direction() == eval::playback_controller::PlaybackDirection::Backward);
    assert!(context.get_playback_style() == eval::playback_controller::PlaybackStyle::PingPong);
    assert_eq!(context.get_frame_rate(), 24);

    // And the animation laid afterwards plays by them, reading the same.
    let names = resource.get_animation_names();
    context.setup_animation(&names[0]).expect("Failed to setup animation");
    assert_eq!(context.get_speed_scale(), 2.0);
    assert!(!context.get_skip_frames());
    assert!(context.get_playback_direction() == eval::playback_controller::PlaybackDirection::Backward);
    assert!(context.get_playback_style() == eval::playback_controller::PlaybackStyle::PingPong);
    assert_eq!(context.get_frame_rate(), 24);
}

#[test]
fn the_hosts_playback_policy_survives_setup_animation() {
    // Through the *public* path, which is the one that matters: a Player only
    // ever reaches `PlaybackController::setup` via `Context::setup_animation`,
    // and that collapses the track stack first — rebuilding the source, and
    // with it the controller. Asserting on the controller alone would pass
    // while the behaviour a Player sees stayed broken.
    prepare();
    let ssab_path = fixture("Ringo/Ringo.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Ringo.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");

    let names = resource.get_animation_names();
    assert!(names.len() >= 2, "fixture sanity: Ringo.ssab holds many animations");

    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation(&names[0]).expect("Failed to setup animation");

    let authored_fps = context.get_frame_rate();
    context.set_speed_scale(2.0);
    context.set_skip_frames(false);
    context.set_playback_direction(eval::playback_controller::PlaybackDirection::Backward, eval::playback_controller::PlaybackStyle::PingPong);
    context.set_frame_rate(authored_fps + 17);
    context.set_loop_count(3);

    // A transition to another clip in the same pack.
    context.setup_animation(&names[1]).expect("Failed to setup the second animation");

    assert!(!context.get_skip_frames(), "frame skip is the host's, and must survive the transition");
    assert_eq!(context.get_frame_rate(), authored_fps + 17, "the frame-rate override must survive the transition");
    assert_eq!(context.get_speed_scale(), 2.0, "the speed is the host's, and must survive the transition");
    assert!(context.get_playback_direction() == eval::playback_controller::PlaybackDirection::Backward, "the playback direction is the host's, and must survive the transition");
    assert!(context.get_playback_style() == eval::playback_controller::PlaybackStyle::PingPong, "so is the ping-pong style");
    assert_eq!(context.get_loop_count(), -1, "the loop count is per-transition and must reset");

    // Frame skip survived as `false`, which means fixed stepping: exactly one
    // frame per update, however much time is handed over. Measured as a
    // magnitude, because the retained direction walks the head down rather
    // than up -- which is itself the direction surviving, end to end.
    context.play();
    let before = context.get_frame_no();
    context.update(10_000.0);
    let stepped = context.get_frame_no() - before;
    assert!(stepped < 0.0, "the retained backward direction should walk the head down, got {stepped}");
    assert!((stepped.abs() - 1.0).abs() < 1e-3, "fixed stepping should move exactly one frame, got {stepped}");

    // The getter above says the rate was *kept*; this says it is *applied* —
    // folded into the step against the new clip's frame rate rather than the
    // old one's. That needs timed stepping, and turning it back on is the
    // host's call: it touches neither the retained rate nor the override.
    // Advance a quarter of the clip so the loop cannot wrap and hide it.
    context.set_skip_frames(true);
    let effective_fps = (authored_fps + 17) as f32 * 2.0;
    let span = (context.get_end_frame() - context.get_start_frame()) as f32;
    assert!(span >= 4.0, "fixture sanity: the clip needs a few frames to measure across");
    let quarter = span * 0.25;

    context.play();
    let before = context.get_frame_no();
    context.update(quarter / effective_fps * 1000.0);
    let advanced = (context.get_frame_no() - before).abs();
    assert!((advanced - quarter).abs() < span * 0.1, "the retained speed and override should give ~{quarter} frames, got {advanced}");
}

// --- The character's clock ---------------------------------------------
//
// Playback state is the character's, not track 0's: every source a track
// holds advances while the character plays, and pause / stop / play act on
// the whole stack. Before this, a source attached by `set_track_animation`
// or `crossfade` was set up but never started, so an overlay sat on its
// first frame and a finished crossfade left the character frozen.

#[test]
fn an_overlay_track_plays_with_the_character() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut ctx = playing_ringo(&resource);

    ctx.set_track_animation(1, "run").unwrap();
    let start = ctx.get_track_frame(1).unwrap();
    tick(&mut ctx, 100.0);
    assert_ne!(ctx.get_track_frame(1).unwrap(), start, "the overlay is stuck on its first frame");
}

#[test]
fn a_crossfade_on_an_empty_track_plays_with_the_character() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut ctx = playing_ringo(&resource);

    ctx.crossfade(1, "run", 500.0).unwrap();
    let start = ctx.get_track_frame(1).unwrap();
    tick(&mut ctx, 100.0);
    assert_ne!(ctx.get_track_frame(1).unwrap(), start, "a crossfade from nothing starts playing at once");
}

#[test]
fn the_incoming_animation_of_a_crossfade_keeps_playing() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut ctx = playing_ringo(&resource);

    ctx.crossfade(0, "run", 200.0).unwrap();
    let incoming = ctx.instance().sources.len() - 1;
    let start = ctx.get_source_frame(incoming as u32).unwrap();
    tick(&mut ctx, 100.0);
    assert!(ctx.get_track_fade_progress(0).is_some(), "fixture sanity: still mid-fade");
    assert_ne!(ctx.get_source_frame(incoming as u32).unwrap(), start, "the incoming animation is stuck while it fades in");

    tick(&mut ctx, 200.0);
    assert!(ctx.get_track_fade_progress(0).is_none(), "fixture sanity: the fade has completed");
    assert!(ctx.is_playing(), "the character stopped when the crossfade completed");
    let settled = ctx.get_track_frame(0).unwrap();
    tick(&mut ctx, 100.0);
    assert_ne!(ctx.get_track_frame(0).unwrap(), settled, "track 0 is frozen after the crossfade");
}

#[test]
fn pause_holds_every_track_and_every_fade() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut ctx = playing_ringo(&resource);
    ctx.set_track_animation(1, "run").unwrap();
    ctx.crossfade(0, "wait1", 400.0).unwrap();
    tick(&mut ctx, 100.0);

    ctx.pause();
    let frames = (ctx.get_track_frame(0), ctx.get_track_frame(1));
    let progress = ctx.get_track_fade_progress(0);
    tick(&mut ctx, 200.0);
    assert_eq!((ctx.get_track_frame(0), ctx.get_track_frame(1)), frames, "a track advanced while paused");
    assert_eq!(ctx.get_track_fade_progress(0), progress, "the crossfade advanced while paused");

    ctx.resume();
    tick(&mut ctx, 100.0);
    assert_ne!(ctx.get_track_frame(1), frames.1, "resume did not restart the overlay");
    assert_ne!(ctx.get_track_fade_progress(0), progress, "resume did not restart the crossfade");
}

#[test]
fn stop_halts_the_whole_character_where_it_stands() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut ctx = playing_ringo(&resource);
    ctx.set_track_animation(1, "run").unwrap();
    ctx.fade_track(1, 0.0, 400.0).unwrap();
    tick(&mut ctx, 100.0);

    ctx.stop();
    assert!(!ctx.is_playing());
    let frames = (ctx.get_track_frame(0), ctx.get_track_frame(1));
    let alpha = ctx.get_track_alpha(1);
    tick(&mut ctx, 200.0);
    assert_eq!((ctx.get_track_frame(0), ctx.get_track_frame(1)), frames, "a track advanced while stopped");
    assert_eq!(ctx.get_track_alpha(1), alpha, "the alpha fade advanced while stopped");

    ctx.resume();
    tick(&mut ctx, 100.0);
    assert_eq!((ctx.get_track_frame(0), ctx.get_track_frame(1)), frames, "resume is not a way out of stop");
}

/// `play` is a head-out of track 0, and only of track 0: the other tracks
/// keep their place, as they do when track 0 loops or ends on its own.
#[test]
fn play_heads_out_track_zero_and_leaves_the_other_tracks_in_place() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut ctx = playing_ringo(&resource);
    ctx.set_track_animation(1, "run").unwrap();
    ctx.set_track_animation(2, "dance").unwrap();
    ctx.set_track_speed(2, 0.0).unwrap();
    ctx.set_track_frame(2, 5.0).unwrap();
    tick(&mut ctx, 100.0);

    let overlay = ctx.get_track_frame(1).unwrap();
    assert!(overlay > 0.0, "fixture sanity: the overlay has moved off its first frame");
    ctx.stop();
    ctx.play();
    let start = ctx.instance().primary_source().controller.get_start_frame() as f32;
    assert_eq!(ctx.get_track_frame(0), Some(start), "play rewinds track 0");
    assert_eq!(ctx.get_track_frame(1), Some(overlay), "play moved the overlay");
    assert_eq!(ctx.get_track_frame(2), Some(5.0), "play moved the scrubbed track");

    tick(&mut ctx, 100.0);
    assert_ne!(ctx.get_track_frame(1), Some(overlay), "the overlay did not carry on after play");
}

#[test]
fn the_clock_runs_on_after_track_zero_finishes() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "attack1");
    ctx.set_loop_count(1);
    ctx.play();
    ctx.set_track_animation(1, "run").unwrap();
    ctx.fade_track(1, 0.0, 60_000.0).unwrap();

    let mut ticks = 0;
    while !ctx.is_finished() {
        tick(&mut ctx, 1000.0 / 60.0);
        ticks += 1;
        assert!(ticks < 60 * 60, "fixture sanity: attack1 with one loop finishes");
    }
    let overlay = ctx.get_track_frame(1);
    let alpha = ctx.get_track_alpha(1);
    tick(&mut ctx, 100.0);
    assert_ne!(ctx.get_track_frame(1), overlay, "the overlay stopped with track 0");
    assert_ne!(ctx.get_track_alpha(1), alpha, "the fade stopped with track 0");
}

/// After track 0 finishes the clock still runs, so a pause still holds the
/// other tracks. It is not reported, though: `is_pausing` implies
/// `is_playing`, and the pair keeps reading as three states.
#[test]
fn a_pause_after_track_zero_finishes_holds_the_rest_without_reporting_a_hold() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "attack1");
    ctx.set_loop_count(1);
    ctx.play();
    ctx.set_track_animation(1, "run").unwrap();
    while !ctx.is_finished() {
        tick(&mut ctx, 1000.0 / 60.0);
    }

    ctx.pause();
    assert!(!ctx.is_playing() && !ctx.is_pausing(), "still the finished state, not a fourth one");
    let overlay = ctx.get_track_frame(1);
    tick(&mut ctx, 100.0);
    assert_eq!(ctx.get_track_frame(1), overlay, "the pause did not hold the overlay");

    ctx.resume();
    tick(&mut ctx, 100.0);
    assert_ne!(ctx.get_track_frame(1), overlay, "resume did not release the overlay");
}

/// The clock's own state reads on where `is_playing` / `is_pausing` stop
/// at track 0's end, so a host gating its tick needs no copy of it.
#[test]
fn the_clock_state_reads_through_track_zero_finishing() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "attack1");
    let clock = |ctx: &Context<'_>| (ctx.is_clock_playing(), ctx.is_clock_pausing());
    assert_eq!(clock(&ctx), (false, false), "a setup leaves the clock stopped");

    ctx.pause();
    assert_eq!(clock(&ctx), (false, false), "a pause on a stopped clock held it");

    ctx.set_loop_count(1);
    ctx.play();
    assert_eq!(clock(&ctx), (true, false));
    ctx.set_track_animation(1, "run").unwrap();
    while !ctx.is_finished() {
        tick(&mut ctx, 1000.0 / 60.0);
    }
    assert!(!ctx.is_playing(), "fixture sanity: track 0 reports its end");
    assert_eq!(clock(&ctx), (true, false), "the clock read as stopped with track 0");

    ctx.pause();
    assert_eq!(clock(&ctx), (true, true), "the hold after track 0's end went unread");
    ctx.resume();
    assert_eq!(clock(&ctx), (true, false));

    ctx.stop();
    assert_eq!(clock(&ctx), (false, false));

    ctx.play_with_start_frame(0.0);
    assert_eq!(clock(&ctx), (true, false));
    assert!(ctx.setup_animation("no such animation").is_err());
    assert_eq!(clock(&ctx), (true, false), "a failed setup stopped the clock");
    ctx.setup_animation("walk").unwrap();
    assert_eq!(clock(&ctx), (false, false), "a setup left the clock running");
}

/// What a Player gating its redraw needs beyond track 0's frame: whether
/// anything else in the stack still moves the pose, read from the stack
/// itself rather than from the calls that built it.
#[test]
fn the_stack_is_in_motion_only_while_something_beyond_track_zero_moves() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut ctx = playing_ringo(&resource);
    assert!(!ctx.is_stack_in_motion(), "track 0 alone");

    ctx.set_track_animation(1, "run").unwrap();
    assert!(ctx.is_stack_in_motion(), "an overlay plays");
    ctx.pause();
    assert!(ctx.is_stack_in_motion(), "the clock was read");
    ctx.resume();
    ctx.set_track_alpha(1, 0.0).unwrap();
    assert!(!ctx.is_stack_in_motion(), "a silenced overlay");
    ctx.set_track_alpha(1, 1.0).unwrap();
    ctx.set_track_speed(1, 0.0).unwrap();
    assert!(!ctx.is_stack_in_motion(), "a scrubbed overlay");
    ctx.set_track_speed(1, 1.0).unwrap();
    ctx.clear_track(1).unwrap();
    assert_eq!(ctx.get_track_count(), 2, "fixture sanity: a cleared track keeps its slot");
    assert!(!ctx.is_stack_in_motion(), "a cleared overlay");

    ctx.set_track_animation(1, "run").unwrap();
    ctx.set_track_loop_count(1, 1).unwrap();
    for _ in 0..2000 {
        if ctx.is_track_finished(1) {
            break;
        }
        tick(&mut ctx, 1000.0 / 60.0);
    }
    assert!(ctx.is_track_finished(1), "fixture sanity: the overlay runs out");
    assert!(!ctx.is_stack_in_motion(), "a finished overlay holds its last frame");
    ctx.clear_track(1).unwrap();

    ctx.crossfade(0, "run", 100.0).unwrap();
    assert!(ctx.is_stack_in_motion(), "a crossfade on track 0");
    tick(&mut ctx, 200.0);
    assert!(!ctx.is_stack_in_motion(), "the completed crossfade retired the outgoing side");

    ctx.crossfade(0, "walk", 100.0).unwrap();
    tick(&mut ctx, 50.0);
    ctx.cancel_track_fade(0).unwrap();
    assert!(ctx.get_track_fade_progress(0).is_none(), "fixture sanity: the fade is cancelled");
    assert!(ctx.is_stack_in_motion(), "the outgoing side a cancelled crossfade left plays on");

    ctx.setup_animation("walk").unwrap();
    ctx.play();
    assert!(!ctx.is_stack_in_motion(), "a setup folds the stack back");
    ctx.add_track_source(0, "run", 0.5).unwrap();
    assert!(ctx.is_stack_in_motion(), "a source beside track 0's main one");
}

/// An event's index addresses the firing source's animation, and a
/// crossfade on track 0 hands track 0 to another animation, so a Player
/// has to be able to ask which animation a source plays.
#[test]
fn a_source_reports_the_animation_it_plays() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let walk = resource.find_animation_index("walk").unwrap() as u32;
    let run = resource.find_animation_index("run").unwrap() as u32;
    let mut ctx = playing_ringo(&resource);
    let main = ctx.get_track_main_source(0).unwrap();
    assert_eq!(ctx.get_source_animation_index(main), Some(walk));

    ctx.crossfade(0, "run", 100.0).unwrap();
    tick(&mut ctx, 200.0);
    let main = ctx.get_track_main_source(0).unwrap();
    assert_eq!(ctx.get_source_animation_index(main), Some(run), "track 0 now plays run");
    assert_eq!(resource.get_animation_names()[run as usize], "run", "the index is the pack's animation order");

    assert_eq!(ctx.get_source_animation_index(999), None);
}

/// The current animation is track 0's main one, and a crossfade or a
/// `set_track_animation` on track 0 changes it from the call on, while
/// another track's leaves it alone.
#[test]
fn the_current_animation_follows_track_zero_from_the_call_on() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let index = |name: &str| resource.find_animation_index(name).map(|i| i as u32);
    let mut ctx = Context::new();
    ctx.bind_resource(&resource);
    assert_eq!(ctx.get_current_animation_index(), None, "nothing is set up yet");

    ctx.setup_animation("walk").unwrap();
    ctx.play();
    assert_eq!(ctx.get_current_animation_index(), index("walk"));

    ctx.crossfade(0, "run", 1000.0).unwrap();
    assert_eq!(ctx.get_current_animation_index(), index("run"), "from the moment the fade begins");

    ctx.set_track_animation(1, "attack1").unwrap();
    assert_eq!(ctx.get_current_animation_index(), index("run"), "another track's animation is not the current one");

    ctx.set_track_animation(0, "dance").unwrap();
    assert_eq!(ctx.get_current_animation_index(), index("dance"));
}
