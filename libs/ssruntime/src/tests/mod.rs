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

/// The projects the tests play: `tests/<name>/<name>.sspj`.
static PROJECTS: [&str; 5] = ["overall", "ParticleEffect", "allAttributeV7", "allPartsV7", "Ringo"];

fn get_test_base_path() -> PathBuf {
    std::env::current_dir().expect("Failed to get the current directory").ancestors().nth(2).expect("Failed to get the base directory").to_path_buf()
}

/// Where [`prepare`] converts the projects to, one directory each: under `target/`,
/// so a test run never writes into the source tree.
fn fixtures_dir() -> PathBuf {
    get_test_base_path().join("target/ssruntime_test_fixtures")
}

/// A pack [`prepare`] converted, named `<project>/<pack>.ssab`.
fn fixture(pack: &str) -> PathBuf {
    fixtures_dir().join(pack)
}

fn prepare() {
    // Regenerate fixtures exactly once per test run, always from source, so cached `.ssab`
    // files can never go stale against the current ssab schema, which flatbuffers would panic
    // reading out of range. `Once` serializes regeneration so the many
    // parallel `prepare()` callers don't race on a half-written directory. A failed conversion
    // panics, which poisons the `Once` and so fails every test that needs a pack.
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        for project in PROJECTS {
            let out = fixtures_dir().join(project);
            let _ = std::fs::remove_dir_all(&out);
            let sspj = get_test_base_path().join("tests").join(project).join(format!("{project}.sspj"));
            let config = ssconverter::ConverterConfig { output: Some(out.into()), ..Default::default() };
            ssconverter::convert_sspj_to_ssab_files(&sspj, &config, &|_msg: &str| {}).unwrap_or_else(|e| panic!("convert {project}: {e}"));
        }
    });
}

fn load_ringo_resource() -> SsabData {
    prepare();
    let ssab_path = fixture("Ringo/Ringo.ssab");
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
