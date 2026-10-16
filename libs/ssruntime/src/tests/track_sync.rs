//! `TrackSync` tests: where the animations laid on a track start.

use super::*;

/// Where track 0's main animation stands in its section, as a fraction.
fn primary_phase(ctx: &Context<'_>) -> f32 {
    let c = &ctx.instance().primary_source().controller;
    (c.get_frame_no() - c.get_start_frame() as f32) / (c.get_end_frame() - c.get_start_frame() + 1) as f32
}

/// The frame `phase` of the way through `handle`'s section.
fn frame_at_phase(ctx: &Context<'_>, handle: u32, phase: f32) -> f32 {
    let c = &ctx.instance().sources[handle as usize].controller;
    c.get_start_frame() as f32 + phase * (c.get_end_frame() - c.get_start_frame() + 1) as f32
}

#[test]
fn a_free_track_starts_its_animation_from_the_head() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut ctx = playing_ringo(&resource);
    assert_eq!(ctx.get_track_sync(0), Some(TrackSync::Free), "Free is the default");

    ctx.set_track_animation(1, "run").unwrap();
    let handle = ctx.get_track_main_source(1).unwrap();
    assert_eq!(ctx.get_track_frame(1), Some(frame_at_phase(&ctx, handle, 0.0)));
}

#[test]
fn a_start_synced_track_starts_at_track_zeros_phase() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut ctx = playing_ringo(&resource);
    tick(&mut ctx, 250.0);
    let phase = primary_phase(&ctx);
    assert!(phase > 0.05, "fixture sanity: track 0 is part-way through, at {phase}");

    // Set before anything is on the track: the setting is what the
    // animation laid there next starts by.
    ctx.set_track_sync(1, TrackSync::Start).unwrap();
    ctx.set_track_animation(1, "run").unwrap();
    let handle = ctx.get_track_main_source(1).unwrap();
    let expected = frame_at_phase(&ctx, handle, phase);
    let frame = ctx.get_track_frame(1).unwrap();
    assert!((frame - expected).abs() < 1e-3, "run starts at {frame}, not at {expected} ({phase} of its section)");
}

#[test]
fn a_start_synced_crossfade_on_track_zero_starts_where_the_outgoing_one_stands() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut ctx = playing_ringo(&resource);
    tick(&mut ctx, 250.0);
    let phase = primary_phase(&ctx);

    ctx.set_track_sync(0, TrackSync::Start).unwrap();
    ctx.crossfade(0, "run", 400.0).unwrap();
    let incoming = ctx.instance().sources.len() as u32 - 1;
    let expected = frame_at_phase(&ctx, incoming, phase);
    let frame = ctx.get_source_frame(incoming).unwrap();
    assert!((frame - expected).abs() < 1e-3, "run starts at {frame}, not at {expected}");
}

#[test]
fn a_start_synced_source_added_to_a_track_starts_at_track_zeros_phase() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut ctx = playing_ringo(&resource);
    tick(&mut ctx, 250.0);
    let phase = primary_phase(&ctx);

    ctx.set_track_sync(0, TrackSync::Start).unwrap();
    let handle = ctx.add_track_source(0, "run", 0.5).unwrap();
    let expected = frame_at_phase(&ctx, handle, phase);
    let frame = ctx.get_source_frame(handle).unwrap();
    assert!((frame - expected).abs() < 1e-3, "run starts at {frame}, not at {expected}");
}

#[test]
fn track_sync_belongs_to_the_track_and_goes_with_a_new_setup() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let mut ctx = Context::new();
    ctx.bind_resource(&resource);
    assert!(ctx.set_track_sync(1, TrackSync::Start).is_err(), "track 1 needs an animation on track 0");

    ctx.setup_animation("walk").unwrap();
    ctx.set_track_sync(1, TrackSync::Start).unwrap();
    assert_eq!(ctx.get_track_sync(1), Some(TrackSync::Start));
    assert_eq!(ctx.get_track_sync(0), Some(TrackSync::Free), "the setting is per track");

    ctx.setup_animation("run").unwrap();
    assert_eq!(ctx.get_track_sync(1), None, "a new setup collapses the stack");
    assert_eq!(ctx.get_track_sync(0), Some(TrackSync::Free), "and track 0 starts over from Free");
}
