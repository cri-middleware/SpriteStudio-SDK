//! Unit tests over the packs in `tests/`, converted once per run, and the fixtures they share.

use std::path::PathBuf;

use super::*;

mod draw_plan;
mod effect_slot;
mod events;
mod frame;
mod instance_slot;
mod mix;
mod override_layer;
mod playback;
mod resource;
mod track_sync;
mod tracks;

static TARGET_SSPJ_FILES: [&str; 5] = ["tests/overall/overall.sspj", "tests/ParticleEffect/ParticleEffect.sspj", "tests/allAttributeV7/allAttributeV7.sspj", "tests/allPartsV7/allPartsV7.sspj", "tests/Ringo/Ringo.sspj"];

static TARGET_SSPAB_DIRS: [&str; 5] = ["tests/overall/overall", "tests/ParticleEffect/ParticleEffect", "tests/allAttributeV7/allAttributeV7", "tests/allPartsV7/allPartsV7", "tests/Ringo/Ringo"];

fn get_test_base_path() -> PathBuf {
    std::env::current_dir().expect("Failed to get the current directory").ancestors().nth(2).expect("Failed to get the base directory").to_path_buf()
}

fn prepare() {
    // Regenerate fixtures exactly once per test run, always from source, so cached `.ssab`
    // files can never go stale against the current ssab schema (a stale cache previously caused
    // flatbuffers to panic on out-of-range reads). `Once` serializes regeneration so the many
    // parallel `prepare()` callers don't race on a half-written directory.
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        TARGET_SSPAB_DIRS.iter().enumerate().for_each(|(i, &dir)| {
            let ssab_dir = get_test_base_path().join(dir);
            let _ = std::fs::remove_dir_all(&ssab_dir);
            let sspj = get_test_base_path().join(TARGET_SSPJ_FILES[i]);
            let config = ssconverter::ConverterConfig::default();
            let _ = ssconverter::convert_sspj_to_ssab_files(sspj.as_path(), &config, &|_msg: &str| {});
        });
    });
}

fn load_ringo_resource() -> SsabData {
    prepare();
    let ssab_path = get_test_base_path().join("tests/Ringo/Ringo/Ringo.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Ringo.ssab");
    SsabData::load_copy(&input_buf).expect("Failed to load SsabData")
}

fn ringo_ctx<'a>(resource: &'a AnimationResource<'a>, anim: &str) -> (Context<'a>, f32) {
    let mut ctx = Context::new();
    ctx.bind_resource(resource);
    ctx.setup_animation(anim).unwrap();
    let start = ctx.instance().primary_source().controller.get_start_frame() as f32;
    (ctx, start)
}

/// A playing Ringo on `walk`, its clock running.
fn playing_ringo<'a>(resource: &'a AnimationResource<'a>) -> Context<'a> {
    let (mut ctx, _) = ringo_ctx(resource, "walk");
    ctx.play();
    tick(&mut ctx, 100.0);
    assert!(ctx.get_track_frame(0).unwrap() > 0.0, "fixture sanity: track 0 advances");
    ctx
}

/// Step `ctx` by `ms` milliseconds in 60 fps ticks, the way a host does.
fn tick(ctx: &mut Context<'_>, ms: f32) {
    let step: f32 = 1000.0 / 60.0;
    let mut left = ms;
    while left > 0.0 {
        ctx.update(step.min(left));
        left -= step;
    }
}
