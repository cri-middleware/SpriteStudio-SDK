//! `mix_tracks` tests: blending sources and tracks into one pose.

use super::*;
use crate::mix::{LayerState, SourceState, TrackState, mix_tracks};

/// `(pos_x, pos_y, rot_z, scale, alpha, optional (dx, dy) deform)`.
type MixLayerPart = (f32, f32, f32, f32, f32, Option<([f32; 4], [f32; 4])>);

fn build_mix_layer(parts: &[MixLayerPart]) -> LayerState<'static> {
    let mut layer = LayerState::new();
    for (i, p) in parts.iter().enumerate() {
        let mut s = runtime::framedata::ss::runtime::PartState::default();
        s.set_part_index(i as i16);
        s.set_cell(-1);
        s.set_vertex(-1);
        s.set_position_x(p.0);
        s.set_position_y(p.1);
        s.set_rotation_z(p.2);
        s.set_scale_x(p.3);
        s.set_scale_y(p.3);
        s.set_alpha(p.4);
        s.set_size_x(10.0);
        s.set_size_y(10.0);
        s.set_update_flag(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX);
        layer.parts_vec.push(s);
        let rvd = match p.5 {
            Some((dx, dy)) => crate::eval::framedata::RawVertexDeform { present: true, dx, dy },
            None => crate::eval::framedata::RawVertexDeform::default(),
        };
        layer.raw_vertex_deform.push(Some(alloc::boxed::Box::new(rvd)));
    }
    layer
}

/// Resolve one `Mix` track at full alpha, with the primary's own pose as the
/// floor — the shape every pre-track test was written against, so those tests
/// keep asserting exactly what they did before the stack existed.
fn mix_single_track<'b>(sources: &[SourceState], layers: &[LayerState<'b>], primary_idx: usize, resolved_layer: &mut LayerState<'b>) {
    mix_tracks(sources, layers, &[TrackState::new()], primary_idx, &[], resolved_layer);
}

fn mix_source(weight: f32) -> SourceState {
    let mut s = SourceState::new();
    s.animation_idx = 0; // any non-negative value makes it active
    s.weight = weight;
    s
}

#[test]
fn mix_tracks_single_source_passthrough() {
    let layers = vec![build_mix_layer(&[(10.0, 20.0, 1.5, 2.0, 0.5, None)])];
    let sources = vec![mix_source(1.0)];
    let mut resolved = LayerState::new();

    mix_single_track(&sources, &layers, 0, &mut resolved);

    assert_eq!(resolved.parts_vec.len(), 1);
    let r = &resolved.parts_vec[0];
    assert_eq!(r.position_x(), 10.0);
    assert_eq!(r.position_y(), 20.0);
    assert_eq!(r.rotation_z(), 1.5);
    assert_eq!(r.scale_x(), 2.0);
    assert_eq!(r.alpha(), 0.5);
}

/// Every continuous `PartState` attribute set to a distinct value
/// (`bias + 1.0` … `bias + 28.0`, skipping `bias + 13.0` which is the slot
/// of the discrete `priority`), so a swapped or dropped attribute cannot
/// pass a comparison by coincidence.
fn fully_attributed_part_state(bias: f32) -> runtime::framedata::ss::runtime::PartState {
    let mut ps = runtime::framedata::ss::runtime::PartState::default();
    ps.set_position_x(bias + 1.0);
    ps.set_position_y(bias + 2.0);
    ps.set_position_z(bias + 3.0);
    ps.set_rotation_x(bias + 4.0);
    ps.set_rotation_y(bias + 5.0);
    ps.set_rotation_z(bias + 6.0);
    ps.set_scale_x(bias + 7.0);
    ps.set_scale_y(bias + 8.0);
    ps.set_local_scale_x(bias + 9.0);
    ps.set_local_scale_y(bias + 10.0);
    ps.set_alpha(bias + 11.0);
    ps.set_local_alpha(bias + 12.0);
    ps.set_pivot_x(bias + 14.0);
    ps.set_pivot_y(bias + 15.0);
    ps.set_anchor_x(bias + 16.0);
    ps.set_anchor_y(bias + 17.0);
    ps.set_size_x(bias + 18.0);
    ps.set_size_y(bias + 19.0);
    ps.set_uv_translation_x(bias + 20.0);
    ps.set_uv_translation_y(bias + 21.0);
    ps.set_uv_rotation_z(bias + 22.0);
    ps.set_uv_scale_x(bias + 23.0);
    ps.set_uv_scale_y(bias + 24.0);
    ps.set_bounding_radius(bias + 25.0);
    ps.set_mask(bias + 26.0);
    ps.set_skew_x(bias + 27.0);
    ps.set_skew_y(bias + 28.0);
    ps
}

/// Pins the full continuous/discrete split: all 27 continuous attributes
/// must be weighted-summed and every discrete one must come from the
/// primary verbatim. The narrower TRSA blend tests below would not notice a
/// dropped skew or a pivot that stopped blending.
#[test]
fn mix_tracks_blends_every_continuous_attribute() {
    use runtime::framedata::ss::runtime::UpdateAttributeFlags;

    let mut primary_part = fully_attributed_part_state(0.0);
    primary_part.set_priority(13);
    primary_part.set_part_index(7);
    primary_part.set_cell(3);
    primary_part.set_update_flag(UpdateAttributeFlags::AttributePosX);

    let mut secondary_part = fully_attributed_part_state(100.0);
    secondary_part.set_priority(999);
    secondary_part.set_part_index(-1);
    secondary_part.set_cell(-1);
    secondary_part.set_update_flag(UpdateAttributeFlags::AttributeAlpha);

    let mut layers = Vec::new();
    for part in [primary_part, secondary_part] {
        let mut layer = LayerState::new();
        layer.parts_vec.push(part);
        layer.raw_vertex_deform.push(None);
        layers.push(layer);
    }

    let sources = vec![mix_source(1.0), mix_source(1.0)];
    let mut resolved = LayerState::new();
    mix_single_track(&sources, &layers, 0, &mut resolved);

    // A 50/50 blend of `n` and `n + 100` is `n + 50`, exact in f32.
    let r = &resolved.parts_vec[0];
    assert_eq!(r.position_x(), 51.0);
    assert_eq!(r.position_y(), 52.0);
    assert_eq!(r.position_z(), 53.0);
    assert_eq!(r.rotation_x(), 54.0);
    assert_eq!(r.rotation_y(), 55.0);
    assert_eq!(r.rotation_z(), 56.0);
    assert_eq!(r.scale_x(), 57.0);
    assert_eq!(r.scale_y(), 58.0);
    assert_eq!(r.local_scale_x(), 59.0);
    assert_eq!(r.local_scale_y(), 60.0);
    assert_eq!(r.alpha(), 61.0);
    assert_eq!(r.local_alpha(), 62.0);
    assert_eq!(r.pivot_x(), 64.0);
    assert_eq!(r.pivot_y(), 65.0);
    assert_eq!(r.anchor_x(), 66.0);
    assert_eq!(r.anchor_y(), 67.0);
    assert_eq!(r.size_x(), 68.0);
    assert_eq!(r.size_y(), 69.0);
    assert_eq!(r.uv_translation_x(), 70.0);
    assert_eq!(r.uv_translation_y(), 71.0);
    assert_eq!(r.uv_rotation_z(), 72.0);
    assert_eq!(r.uv_scale_x(), 73.0);
    assert_eq!(r.uv_scale_y(), 74.0);
    assert_eq!(r.bounding_radius(), 75.0);
    assert_eq!(r.mask(), 76.0);
    assert_eq!(r.skew_x(), 77.0);
    assert_eq!(r.skew_y(), 78.0);

    // Discrete attributes: primary's values, never blended.
    assert_eq!(r.priority(), 13);
    assert_eq!(r.part_index(), 7);
    assert_eq!(r.cell(), 3);
    assert_eq!(r.update_flag(), UpdateAttributeFlags::AttributePosX | UpdateAttributeFlags::AttributeAlpha);
}

/// A source shorter than the primary must not drag the parts it does not
/// cover toward the origin: weights are renormalized over the sources that
/// actually have the part.
#[test]
fn mix_tracks_renormalizes_weights_per_part() {
    let primary = build_mix_layer(&[(10.0, 20.0, 0.0, 1.0, 1.0, None), (70.0, 80.0, 0.0, 4.0, 0.25, None)]);
    let secondary = build_mix_layer(&[(50.0, 60.0, 0.0, 3.0, 0.0, None)]);
    let layers = vec![primary, secondary];
    let sources = vec![mix_source(1.0), mix_source(1.0)];
    let mut resolved = LayerState::new();

    mix_single_track(&sources, &layers, 0, &mut resolved);

    // Part 0 is covered by both: an ordinary 50/50 blend.
    let p0 = &resolved.parts_vec[0];
    assert_eq!(p0.position_x(), 30.0);
    assert_eq!(p0.position_y(), 40.0);
    assert_eq!(p0.scale_x(), 2.0);
    assert_eq!(p0.alpha(), 0.5);

    // Part 1 exists only in the primary, so it keeps the primary's pose at
    // full weight. A global normalization would halve every attribute here.
    let p1 = &resolved.parts_vec[1];
    assert_eq!(p1.position_x(), 70.0);
    assert_eq!(p1.position_y(), 80.0);
    assert_eq!(p1.scale_x(), 4.0);
    assert_eq!(p1.alpha(), 0.25);
}

/// Give `layer` explicit per-part coverage. Without this a hand-built layer
/// reports "covers everything" (see `mix::layer_coverage`), which is what keeps
/// the pre-coverage tests above meaningful.
fn set_mix_layer_coverage(layer: &mut LayerState<'static>, coverage: &[runtime::framedata::ss::runtime::UpdateAttributeFlags]) {
    layer.coverage_vec.clear();
    layer.coverage_vec.extend_from_slice(coverage);
}

fn mix_source_on(track: u16, weight: f32) -> SourceState {
    let mut s = mix_source(weight);
    s.track = track;
    s
}

fn setup_pose_of(parts: &[MixLayerPart]) -> Vec<runtime::framedata::ss::runtime::PartState> {
    build_mix_layer(parts).parts_vec
}

/// A track's `alpha` lerps its pose against everything below it — and unlike a
/// source's `weight`, it means the same thing no matter what is underneath.
#[test]
fn track_alpha_lerps_against_the_track_below() {
    let base = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let over = build_mix_layer(&[(100.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let setup = setup_pose_of(&[(0.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layers = vec![base, over];
    let sources = vec![mix_source_on(0, 1.0), mix_source_on(1, 1.0)];
    let mut tracks = vec![TrackState::new(), TrackState::new()];
    tracks[1].alpha = 0.3;
    let mut resolved = LayerState::new();

    mix_tracks(&sources, &layers, &tracks, 0, &setup, &mut resolved);

    // lerp(0, 100, 0.3); f32 rounding of 0.3 puts it a ULP off an exact 30.
    assert!((resolved.parts_vec[0].position_x() - 30.0).abs() < 1e-4);
}

/// The overlay's strength must not depend on how many sources sit underneath —
/// the failure that flat normalization has and the track stack does not.
#[test]
fn track_alpha_is_independent_of_the_sources_below() {
    let setup = setup_pose_of(&[(0.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let mut tracks = vec![TrackState::new(), TrackState::new()];
    tracks[1].alpha = 0.3;

    for extra_bases in 0..3 {
        let mut layers = vec![build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, None)])];
        let mut sources = vec![mix_source_on(0, 1.0)];
        for _ in 0..extra_bases {
            layers.push(build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, None)]));
            sources.push(mix_source_on(0, 1.0));
        }
        layers.push(build_mix_layer(&[(100.0, 0.0, 0.0, 1.0, 1.0, None)]));
        sources.push(mix_source_on(1, 1.0));

        let mut resolved = LayerState::new();
        mix_tracks(&sources, &layers, &tracks, 0, &setup, &mut resolved);
        assert!((resolved.parts_vec[0].position_x() - 30.0).abs() < 1e-4, "with {extra_bases} extra base sources");
    }
}

/// An additive track offsets what is below by its distance from the setup pose,
/// so stacking it does not replace the performance underneath.
#[test]
fn additive_track_offsets_by_its_distance_from_setup() {
    // setup pose puts the part at x=10; the overlay animates it to x=40, i.e. +30.
    let setup = setup_pose_of(&[(10.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let base = build_mix_layer(&[(70.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let over = build_mix_layer(&[(40.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layers = vec![base, over];
    let sources = vec![mix_source_on(0, 1.0), mix_source_on(1, 1.0)];
    let mut tracks = vec![TrackState::new(), TrackState::new()];
    tracks[1].blend = TrackBlend::Additive;
    let mut resolved = LayerState::new();

    mix_tracks(&sources, &layers, &tracks, 0, &setup, &mut resolved);
    assert_eq!(resolved.parts_vec[0].position_x(), 100.0, "70 + (40 - 10)");

    // Half strength halves the offset rather than pulling toward the overlay.
    tracks[1].alpha = 0.5;
    let mut resolved = LayerState::new();
    mix_tracks(&sources, &layers, &tracks, 0, &setup, &mut resolved);
    assert_eq!(resolved.parts_vec[0].position_x(), 85.0, "70 + 0.5 * (40 - 10)");
}

/// A track that animates only part of the rig passes everything else through,
/// which is what makes an overlay work without an explicit mask.
#[test]
fn track_passes_through_attributes_it_does_not_animate() {
    use runtime::framedata::ss::runtime::UpdateAttributeFlags as F;

    let setup = setup_pose_of(&[(0.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let mut base = build_mix_layer(&[(70.0, 0.0, 0.0, 2.0, 1.0, None)]);
    let mut over = build_mix_layer(&[(0.0, 0.0, 45.0, 1.0, 1.0, None)]);
    set_mix_layer_coverage(&mut base, &[F::AttributePosX | F::AttributeSclX]);
    set_mix_layer_coverage(&mut over, &[F::AttributeRotZ]); // rotation only

    let layers = vec![base, over];
    let sources = vec![mix_source_on(0, 1.0), mix_source_on(1, 1.0)];
    let tracks = vec![TrackState::new(), TrackState::new()];
    let mut resolved = LayerState::new();

    mix_tracks(&sources, &layers, &tracks, 0, &setup, &mut resolved);

    let p = &resolved.parts_vec[0];
    assert_eq!(p.rotation_z(), 45.0, "the overlay owns what it animates");
    assert_eq!(p.position_x(), 70.0, "and leaves the rest of the pose alone");
    assert_eq!(p.scale_x(), 2.0);
}

/// The coverage rule: an attribute a source does not animate must not be
/// summed in at all. Before this rule the non-covering source voted
/// `evaluate_part`'s hardcoded default and dragged the result halfway to a
/// number that means nothing.
#[test]
fn mix_tracks_excludes_sources_that_do_not_key_the_attribute() {
    use runtime::framedata::ss::runtime::UpdateAttributeFlags as F;

    let mut primary = build_mix_layer(&[(100.0, 0.0, 0.0, 1.0, 0.25, None)]);
    let mut secondary = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, None)]);
    // The primary animates position_x and alpha; the secondary animates
    // neither, and merely reports the defaults for them (0.0 / 1.0).
    set_mix_layer_coverage(&mut primary, &[F::AttributePosX | F::AttributeAlpha]);
    set_mix_layer_coverage(&mut secondary, &[F::AttributeSclX]);

    let layers = vec![primary, secondary];
    let sources = vec![mix_source(1.0), mix_source(1.0)];
    let mut resolved = LayerState::new();

    mix_single_track(&sources, &layers, 0, &mut resolved);

    let p = &resolved.parts_vec[0];
    // Only the primary covers these, so it owns them outright. A global
    // normalization would give 50.0 and 0.625 instead.
    assert_eq!(p.position_x(), 100.0);
    assert_eq!(p.alpha(), 0.25);
    // scale_x is covered only by the secondary, which holds 1.0.
    assert_eq!(p.scale_x(), 1.0);
}

/// An attribute nobody animates keeps the primary's value rather than
/// collapsing to a zero-weighted sum.
#[test]
fn mix_tracks_uncovered_attribute_keeps_primary_value() {
    use runtime::framedata::ss::runtime::UpdateAttributeFlags as F;

    let mut primary = build_mix_layer(&[(10.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let mut secondary = build_mix_layer(&[(90.0, 0.0, 0.0, 1.0, 1.0, None)]);
    set_mix_layer_coverage(&mut primary, &[F::AttributePosX]);
    set_mix_layer_coverage(&mut secondary, &[F::AttributePosX]);

    let layers = vec![primary, secondary];
    let sources = vec![mix_source(1.0), mix_source(1.0)];
    let mut resolved = LayerState::new();

    mix_single_track(&sources, &layers, 0, &mut resolved);

    let p = &resolved.parts_vec[0];
    assert_eq!(p.position_x(), 50.0, "covered by both: an ordinary 50/50 blend");
    // size_x is covered by neither. `build_mix_layer` sets 10.0 on both, and
    // the primary's value is what survives.
    assert_eq!(p.size_x(), 10.0);
}

/// Weights still normalize among the sources that *do* cover an attribute,
/// so an uneven split stays proportional.
#[test]
fn mix_tracks_coverage_normalizes_among_covering_sources_only() {
    use runtime::framedata::ss::runtime::UpdateAttributeFlags as F;

    let mut a = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let mut b = build_mix_layer(&[(100.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let mut c = build_mix_layer(&[(999.0, 0.0, 0.0, 1.0, 1.0, None)]);
    set_mix_layer_coverage(&mut a, &[F::AttributePosX]);
    set_mix_layer_coverage(&mut b, &[F::AttributePosX]);
    set_mix_layer_coverage(&mut c, &[F::AttributeAlpha]); // does not cover position_x

    let layers = vec![a, b, c];
    // c carries the largest weight, but it has no say in position_x.
    let sources = vec![mix_source(1.0), mix_source(3.0), mix_source(96.0)];
    let mut resolved = LayerState::new();

    mix_single_track(&sources, &layers, 0, &mut resolved);

    // 1:3 between the two covering sources => 75.0, not 3.0.
    assert_eq!(resolved.parts_vec[0].position_x(), 75.0);
}

/// When no active source covers a part, the mixer must leave the primary's
/// pose alone rather than write zeroed accumulators over it.
#[test]
fn mix_tracks_keeps_primary_pose_for_uncovered_parts() {
    let primary = build_mix_layer(&[(10.0, 20.0, 0.0, 1.0, 1.0, None), (70.0, 80.0, 0.0, 4.0, 0.25, None)]);
    let secondary = build_mix_layer(&[(50.0, 60.0, 0.0, 3.0, 0.5, None)]);
    let layers = vec![primary, secondary];
    // The primary source is inactive, so only the shorter secondary blends.
    let sources = vec![mix_source(0.0), mix_source(1.0)];
    let mut resolved = LayerState::new();

    mix_single_track(&sources, &layers, 0, &mut resolved);

    // Part 0: the secondary is the only contributor, at full weight.
    let p0 = &resolved.parts_vec[0];
    assert_eq!(p0.position_x(), 50.0);
    assert_eq!(p0.position_y(), 60.0);
    assert_eq!(p0.scale_x(), 3.0);
    assert_eq!(p0.alpha(), 0.5);

    // Part 1: nothing covers it, so the primary's pose survives untouched.
    let p1 = &resolved.parts_vec[1];
    assert_eq!(p1.position_x(), 70.0);
    assert_eq!(p1.position_y(), 80.0);
    assert_eq!(p1.scale_x(), 4.0);
    assert_eq!(p1.alpha(), 0.25);
}

#[test]
fn mix_tracks_50_50_blend_trsa() {
    let layer_a = build_mix_layer(&[(10.0, 20.0, 0.0, 1.0, 1.0, None)]);
    let layer_b = build_mix_layer(&[(30.0, 40.0, ::core::f32::consts::FRAC_PI_2, 2.0, 0.5, None)]);
    let sources = vec![mix_source(1.0), mix_source(1.0)];
    let layers = vec![layer_a, layer_b];
    let mut resolved = LayerState::new();

    mix_single_track(&sources, &layers, 0, &mut resolved);

    let r = &resolved.parts_vec[0];
    assert!((r.position_x() - 20.0).abs() < 1e-6);
    assert!((r.position_y() - 30.0).abs() < 1e-6);
    assert!((r.rotation_z() - ::core::f32::consts::FRAC_PI_4).abs() < 1e-4);
    assert!((r.scale_x() - 1.5).abs() < 1e-6);
    assert!((r.alpha() - 0.75).abs() < 1e-6);
}

#[test]
fn mix_tracks_auto_normalizes_weights() {
    let layer_a = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layer_b = build_mix_layer(&[(100.0, 100.0, 0.0, 1.0, 1.0, None)]);
    let layers = vec![layer_a, layer_b];

    let mut resolved_a = LayerState::new();
    mix_single_track(&[mix_source(0.7), mix_source(0.3)], &layers, 0, &mut resolved_a);

    let mut resolved_b = LayerState::new();
    mix_single_track(&[mix_source(7.0), mix_source(3.0)], &layers, 0, &mut resolved_b);

    // Same ratio → same mix outcome regardless of absolute magnitudes.
    assert!((resolved_a.parts_vec[0].position_x() - resolved_b.parts_vec[0].position_x()).abs() < 1e-5);
    assert!((resolved_a.parts_vec[0].position_x() - 30.0).abs() < 1e-5);
}

#[test]
fn mix_tracks_inactive_animation_idx_skipped() {
    let layer_a = build_mix_layer(&[(10.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layer_b = build_mix_layer(&[(90.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layers = vec![layer_a, layer_b];

    let mut src_b = mix_source(1.0);
    src_b.animation_idx = -1; // removed slot — must be skipped
    let sources = vec![mix_source(1.0), src_b];

    let mut resolved = LayerState::new();
    mix_single_track(&sources, &layers, 0, &mut resolved);

    // Only source A contributes; resolved == primary verbatim.
    assert_eq!(resolved.parts_vec[0].position_x(), 10.0);
}

#[test]
fn mix_tracks_zero_weight_source_skipped() {
    let layer_a = build_mix_layer(&[(10.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layer_b = build_mix_layer(&[(90.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layers = vec![layer_a, layer_b];

    let sources = vec![mix_source(1.0), mix_source(0.0)];

    let mut resolved = LayerState::new();
    mix_single_track(&sources, &layers, 0, &mut resolved);

    assert_eq!(resolved.parts_vec[0].position_x(), 10.0);
}

#[test]
fn mix_tracks_total_weight_zero_uses_primary() {
    let layer_a = build_mix_layer(&[(7.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layer_b = build_mix_layer(&[(99.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layers = vec![layer_a, layer_b];

    // All weights zero. Fall back to primary's pose.
    let sources = vec![mix_source(0.0), mix_source(0.0)];

    let mut resolved = LayerState::new();
    mix_single_track(&sources, &layers, 0, &mut resolved);

    assert_eq!(resolved.parts_vec[0].position_x(), 7.0);
}

#[test]
fn mix_tracks_raw_vertex_deform_weighted() {
    let layer_a = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, Some(([1.0, 2.0, 3.0, 4.0], [10.0, 20.0, 30.0, 40.0])))]);
    let layer_b = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, Some(([5.0, 6.0, 7.0, 8.0], [50.0, 60.0, 70.0, 80.0])))]);
    let sources = vec![mix_source(1.0), mix_source(1.0)];
    let layers = vec![layer_a, layer_b];

    let mut resolved = LayerState::new();
    mix_single_track(&sources, &layers, 0, &mut resolved);

    let d = resolved.raw_vertex_deform[0].as_deref().unwrap();
    assert!(d.present);
    assert!((d.dx[0] - 3.0).abs() < 1e-6);
    assert!((d.dx[3] - 6.0).abs() < 1e-6);
    assert!((d.dy[0] - 30.0).abs() < 1e-6);
    assert!((d.dy[3] - 60.0).abs() < 1e-6);
}

#[test]
fn mix_tracks_update_flag_or_merged() {
    // Source A has AttributePosX set, source B has AttributeRotZ. The mixed
    // pose must report both so Stage B's `needs_center` and any consumer
    // logic see every attribute that contributed to the value.
    let mut layer_a = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let mut layer_b = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, None)]);
    layer_a.parts_vec[0].set_update_flag(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX);
    layer_b.parts_vec[0].set_update_flag(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotZ);

    let sources = vec![mix_source(1.0), mix_source(1.0)];
    let layers = vec![layer_a, layer_b];

    let mut resolved = LayerState::new();
    mix_single_track(&sources, &layers, 0, &mut resolved);

    let f = resolved.parts_vec[0].update_flag();
    assert!(f.contains(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
    assert!(f.contains(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotZ));
}
