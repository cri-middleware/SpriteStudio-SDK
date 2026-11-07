//! Event tests: which sources fire the events a frame passes.

use super::*;

#[test]
fn test_event_collection_integration() {
    prepare();
    let data = std::fs::read(fixture("overall/UserData.ssab")).expect("Failed to read test ssab file");

    let ssab_data = SsabData::load_copy(&data).unwrap();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.unwrap();

    let mut ctx = Context::new();
    ctx.bind_resource(&resource);

    let anim_name = resource.get_animation_names().first().cloned().expect("No animations found in SSAB");
    ctx.setup_animation(&anim_name).unwrap();
    ctx.play();

    // Advance by 15 frames (0 -> 15)
    let interval = 1000.0 / 30.0;
    ctx.update(interval * 15.0);

    let events = ctx.get_passed_event_indices();
    assert!(!events.is_empty(), "Should have collected at least one event");

    // Advance further to cover the whole animation
    ctx.update(interval * 100.0);
    let events = ctx.get_passed_event_indices();
    assert!(!events.is_empty(), "Should have collected events");
    assert!(events.iter().any(|&e| e > 0), "Should have collected at least one event with index > 0");
}

/// Only track 0's animation fires by default, even when another track holds
/// a single source at full weight; that source fires once it opts in.
#[test]
fn an_overlay_fires_events_only_once_it_opts_in() {
    prepare();
    let data = std::fs::read(fixture("overall/UserData.ssab")).expect("Failed to read UserData.ssab");
    let ssab_data = SsabData::load_copy(&data).unwrap();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.unwrap();
    let anim_name = resource.get_animation_names().first().cloned().expect("No animations found in SSAB");

    let run = |opt_in: bool| -> (u32, Vec<u32>) {
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation(&anim_name).unwrap();
        ctx.play();
        ctx.set_track_animation(1, &anim_name).unwrap();
        let overlay = ctx.get_track_main_source(1).unwrap();
        if opt_in {
            ctx.set_source_event_threshold(overlay, 1.0).unwrap();
        }
        let mut fired = Vec::new();
        for _ in 0..120 {
            ctx.update(1000.0 / 30.0);
            fired.extend_from_slice(ctx.get_passed_event_source_handles());
        }
        (overlay, fired)
    };

    let (overlay, fired) = run(false);
    assert!(!fired.is_empty(), "fixture sanity: track 0 fires events");
    assert!(!fired.contains(&overlay), "the overlay fired without opting in");

    let (overlay, fired) = run(true);
    assert!(fired.contains(&overlay), "the overlay did not fire after opting in");
}

/// The outgoing side of a crossfade is silent by default from the moment
/// the fade begins, and with the track's outgoing threshold at 0.5 it fires
/// through the first half of the fade and no further.
#[test]
fn the_outgoing_side_fires_by_its_track_s_threshold() {
    prepare();
    let data = std::fs::read(fixture("overall/UserData.ssab")).expect("Failed to read UserData.ssab");
    let ssab_data = SsabData::load_copy(&data).unwrap();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.unwrap();
    let anim_name = resource.get_animation_names().first().cloned().expect("No animations found in SSAB");

    // The progress at each tick the outgoing source fired, and whether the
    // incoming one fired at all.
    let run = |threshold: Option<f32>| -> (Vec<f32>, bool) {
        let mut ctx = Context::new();
        ctx.bind_resource(&resource);
        ctx.setup_animation(&anim_name).unwrap();
        if let Some(t) = threshold {
            ctx.set_track_outgoing_event_threshold(0, t).unwrap();
        }
        ctx.play();
        let outgoing = ctx.get_track_main_source(0).unwrap();
        ctx.crossfade(0, &anim_name, 4000.0).unwrap();
        let incoming = ctx.get_track_main_source(0).unwrap();
        let (mut out_at, mut incoming_fired) = (Vec::new(), false);
        for _ in 0..119 {
            ctx.update(1000.0 / 30.0);
            let progress = ctx.get_track_fade_progress(0).unwrap();
            if ctx.get_passed_event_source_handles().contains(&outgoing) {
                out_at.push(progress);
            }
            incoming_fired |= ctx.get_passed_event_source_handles().contains(&incoming);
        }
        (out_at, incoming_fired)
    };

    let (out_at, incoming_fired) = run(None);
    assert!(incoming_fired, "fixture sanity: the incoming animation fires");
    assert!(out_at.is_empty(), "the outgoing side fired by default, at {out_at:?}");

    let (out_at, _) = run(Some(0.5));
    assert!(!out_at.is_empty(), "the outgoing side never fired at 0.5");
    assert!(out_at.iter().all(|&p| p <= 0.5 + 1e-4), "it fired past half way: {out_at:?}");
}

#[test]
fn track_outgoing_event_threshold_round_trips() {
    let ssab = load_ringo_resource();
    let resource = unsafe { AnimationResource::new(ssab.as_slice()) }.expect("Failed to load resource");
    let (mut ctx, _) = ringo_ctx(&resource, "walk");
    assert_eq!(ctx.get_track_outgoing_event_threshold(0), Some(f32::INFINITY), "silent by default");
    ctx.set_track_outgoing_event_threshold(0, 0.5).unwrap();
    ctx.setup_animation("run").unwrap();
    assert_eq!(ctx.get_track_outgoing_event_threshold(0), Some(0.5), "track 0 keeps it across a setup");
    assert_eq!(ctx.get_track_outgoing_event_threshold(3), None);
}
