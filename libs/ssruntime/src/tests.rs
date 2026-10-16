use std::path::PathBuf;

use super::*;
use crate::mix::{LayerState, MIX_ATTR_FLAGS, SourceState, TrackState, mix_tracks};

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

#[test]
fn a_part_base_plus_its_changing_attributes_is_the_full_evaluation() {
    use crate::core::framedata::{IDX_ATTRIBUTEDEFORM, evaluate_part};
    use crate::core::{SsCoordinateSystem, SsOrigin, SsUnit};
    prepare();
    for pack in ["tests/Ringo/Ringo/Ringo.ssab", "tests/overall/overall/Basic.ssab"] {
        let input_buf = std::fs::read(get_test_base_path().join(pack)).expect("read pack");
        let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
        let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");
        let mut constant = 0;
        for meta in resource.animations() {
            let parts = meta.cached_parts();
            for p in parts {
                assert_eq!(p.constant_mask & !p.key_mask, 0, "only keyed attributes are constant");
                assert_eq!(p.constant_mask >> IDX_ATTRIBUTEDEFORM & 1, 0, "deform is never constant");
                constant += p.constant_mask.count_ones();
            }
            let setups = [SsCoordinateSystem::YUp, SsCoordinateSystem::YDown].into_iter().flat_map(|cs| [SsOrigin::Center, SsOrigin::TopLeft].into_iter().flat_map(move |o| [SsUnit::Pixel, SsUnit::Normalized].into_iter().map(move |u| (cs, o, u))));
            for (cs, origin, unit) in setups {
                let bases = meta.part_bases(cs, origin, unit);
                let (mut a, mut b, mut c, mut d, mut e) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
                for f in [meta.start_frame as f32, (meta.start_frame + meta.end_frame) as f32 / 2.0 + 0.25, meta.end_frame as f32] {
                    for (i, p) in parts.iter().enumerate() {
                        let n = p.key_mask.count_ones() as usize;
                        let (mut full_cursors, mut base_cursors) = (alloc::vec![0; n], alloc::vec![0; n]);
                        let full = evaluate_part(p, &resource.source.keys, f, &mut full_cursors, None, p.key_mask, cs, origin, unit, meta.canvas_width, meta.canvas_height, &mut a, &mut b, &mut c, &mut d, &mut e);
                        let from_base = evaluate_part(p, &resource.source.keys, f, &mut base_cursors, bases.get(i), p.key_mask & !p.constant_mask, cs, origin, unit, meta.canvas_width, meta.canvas_height, &mut a, &mut b, &mut c, &mut d, &mut e);
                        assert_eq!(full.state.0, from_base.state.0, "{pack} part {i} frame {f}");
                        assert_eq!(full.cell, from_base.cell);
                        assert_eq!(full.part_color.map(|v| v.0), from_base.part_color.map(|v| v.0));
                        assert_eq!(full.shader.map(|v| v.0), from_base.shader.map(|v| v.0));
                        assert_eq!(full.vertex.map(|v| v.0), from_base.vertex.map(|v| v.0));
                        assert_eq!(full.deform_present, from_base.deform_present);
                    }
                }
            }
        }
        assert!(constant > 0, "fixture sanity: {pack} holds attributes that never change");
    }
}

#[test]
fn an_instance_keeps_a_cursor_per_keyed_attribute_only() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/Ringo/Ringo/Ringo.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Ringo.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");
    for meta in resource.animations() {
        let parts = meta.cached_parts();
        let mut base = 0;
        for p in parts {
            assert_eq!(p.cursor_base, base, "cursors are packed in part order");
            assert_eq!(p.key_mask >> crate::core::framedata::IDX_ATTRIBUTECELL & 1 == 1, p.run(crate::core::framedata::IDX_ATTRIBUTECELL).is_some());
            assert_eq!(p.key_mask >> crate::core::framedata::IDX_ATTRIBUTEDEFORM & 1 == 1, p.run(crate::core::framedata::IDX_ATTRIBUTEDEFORM).is_some());
            base += p.key_mask.count_ones();
        }
        assert_eq!(crate::core::framedata::cursor_count(parts), base as usize);
        assert!((base as usize) < parts.len() * crate::core::framedata::NUM_PART_ATTRIBUTES / 2, "fixture sanity: most attributes of most parts carry no keys");
    }
}

#[test]
fn a_mixed_pose_samples_the_primary_cell_for_its_uvs() {
    // A second source at a negligible weight sends every frame through the
    // mix, while the pose stays the primary's to within 1e-6 and the cell is
    // the primary's by definition. The UVs must follow the cell the primary
    // shows on each frame, as they do when it plays alone.
    prepare();
    let ssab_path = get_test_base_path().join("tests/Ringo/Ringo/Ringo.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Ringo.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");
    let meta = &resource.animations()[resource.find_animation_index("dance").expect("fixture sanity: Ringo has dance")];

    let (mut solo, mut mixed) = (Context::new(), Context::new());
    solo.bind_resource(&resource);
    mixed.bind_resource(&resource);
    solo.setup_animation("dance").expect("setup");
    mixed.setup_animation("dance").expect("setup");
    mixed.add_track_source(0, "dance", 1e-6).expect("second source");

    let uvs = |c: &Context| unsafe { ::core::slice::from_raw_parts(c.get_local_uvs_ptr(), c.get_local_uvs_len()) }.to_vec();
    for f in meta.start_frame..=meta.end_frame {
        solo.get_frame_data(f as f32).expect("frame");
        mixed.get_frame_data(f as f32).expect("frame");
        let (a, b) = (uvs(&solo), uvs(&mixed));
        assert_eq!(a.len(), b.len());
        for (i, (x, y)) in a.iter().zip(&b).enumerate() {
            assert!((x - y).abs() < 1e-2, "frame {f}, part {}: solo uv {x}, mixed uv {y}", i / 10);
        }
    }
}

#[test]
fn frame_data_starts_aligned_after_the_builder_is_resized() {
    // A Player reads FrameData's structs where they lie, and `PartState`
    // holds a u64: every frame has to start on an 8-byte boundary,
    // including the frames after the builder is re-sized to the largest.
    prepare();
    let ssab_path = get_test_base_path().join("tests/Ringo/Ringo/Ringo.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Ringo.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");

    let mut ctx = Context::new();
    ctx.bind_resource(&resource);
    let mut resized = 0;
    for name in resource.get_animation_names() {
        let meta = &resource.animations()[resource.find_animation_index(&name).unwrap()];
        ctx.setup_animation(&name).expect("setup");
        for _ in 0..2 {
            for f in meta.start_frame..=meta.end_frame {
                let capacity = ctx.eval.fb_capacity;
                let start = ctx.get_frame_data(f as f32).expect("frame").as_ptr() as usize;
                assert_eq!(start % 8, 0, "{name} frame {f}: frame data starts off an 8-byte boundary");
                resized += usize::from(ctx.eval.fb_capacity != capacity);
            }
        }
    }
    assert!(resized > 0, "fixture sanity: the builder is re-sized");
}

#[test]
fn static_mesh_buffers_are_shared_not_copied() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/Ringo/Ringo/Ringo.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Ringo.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");
    let names = resource.get_animation_names();
    let start = |name: &str| resource.animations()[resource.find_animation_index(name).unwrap()].start_frame as f32;

    let (mut a, mut b) = (Context::new(), Context::new());
    a.bind_resource(&resource);
    b.bind_resource(&resource);
    a.setup_animation(&names[0]).expect("setup");
    assert_eq!(a.get_mesh_uvs_len(), 0, "nothing is handed out before the first frame");
    a.get_frame_data(start(&names[0])).expect("frame");
    assert!(a.get_mesh_uvs_len() > 0 && a.get_mesh_indices_len() > 0, "fixture sanity: Ringo has mesh parts");

    // A second instance hands out the very same buffers.
    b.setup_animation(&names[0]).expect("setup");
    b.get_frame_data(start(&names[0])).expect("frame");
    assert_eq!(a.get_mesh_uvs_ptr(), b.get_mesh_uvs_ptr());
    assert_eq!(a.get_mesh_indices_ptr(), b.get_mesh_indices_ptr());
    assert_eq!(a.get_mesh_index_offsets_ptr(), b.get_mesh_index_offsets_ptr());

    // So does another animation that shows the same cells on its meshes.
    let same = names.iter().skip(1).find(|n| resource.animations()[resource.find_animation_index(n).unwrap()].mesh_layout == resource.animations()[0].mesh_layout).expect("fixture sanity: Ringo's animations share their mesh cells");
    b.setup_animation(same).expect("setup");
    b.get_frame_data(start(same)).expect("frame");
    assert_eq!(a.get_mesh_uvs_ptr(), b.get_mesh_uvs_ptr());

    // SceneGraph mode skins no mesh, so it hands out none.
    b.set_render_mode(crate::core::RenderMode::SceneGraph);
    b.get_frame_data(start(same)).expect("frame");
    assert_eq!((b.get_mesh_uvs_len(), b.get_mesh_indices_len(), b.get_mesh_index_offsets_len()), (0, 0, 0));
}

/// The zero-copy pair reports the frame the last `get_frame_data` built, and
/// reports nothing when there is not one. Without the guard the accessors
/// would read an unfinished `FlatBufferBuilder`, which asserts — and under
/// `panic = "abort"` that assert ends the host process.
#[test]
fn the_frame_data_accessors_never_read_an_unfinished_buffer() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/Ringo/Ringo/Ringo.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Ringo.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");

    let mut context = Context::new();
    assert!(context.get_frame_data_ptr().is_null(), "nothing has been built yet");
    assert_eq!(context.get_frame_data_len(), 0);

    context.bind_resource(&resource);
    let names = resource.get_animation_names();
    context.setup_animation(&names[0]).expect("Failed to setup animation");
    assert!(context.get_frame_data_ptr().is_null(), "binding a resource does not build a frame");

    let (built_ptr, built_len) = {
        let frame = context.get_frame_data(0.0).expect("Failed to build frame 0");
        (frame.as_ptr(), frame.len())
    };
    assert!(built_len > 0, "fixture sanity: frame 0 of Ringo is not empty");
    assert_eq!(context.get_frame_data_ptr(), built_ptr, "the accessor must report the buffer just built");
    assert_eq!(context.get_frame_data_len(), built_len);

    // A teardown resets the builder, so there is no finished buffer to hand
    // out any more -- and the accessors have to say so rather than read it.
    context.reset();
    assert!(context.get_frame_data_ptr().is_null(), "reset drops the built frame");
    assert_eq!(context.get_frame_data_len(), 0);
}

#[test]
fn load_parts_of_ssab_files() {
    prepare();
    TARGET_SSPAB_DIRS.iter().for_each(|&dir| {
        let ssab_dir = get_test_base_path().join(dir);
        println!("{}: ", ssab_dir.file_name().unwrap().to_str().unwrap());
        for entry in ssab_dir.read_dir().unwrap() {
            let f = entry.unwrap().path();
            if !f.is_file() || f.extension().unwrap() != "ssab" {
                continue;
            }

            let input_buf = std::fs::read(&f).unwrap();
            let ssab_data = SsabData::load_copy(&input_buf).unwrap();
            let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.unwrap();
            let mut context = Context::new();
            context.bind_resource(&resource);

            let ssab_buf = context.get_ssab_data().unwrap();
            let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
            println!("  ssab: {}", f.file_name().unwrap().to_str().unwrap());
            println!("  parts");
            ssab.parts().iter().enumerate().for_each(|(idx, part)| {
                println!("    id: {}, parent_id: {}, type: {}, name: {}", idx, part.parent_index(), part.part_type_type().variant_name().unwrap(), part.name());
            });

            println!();
        }
    });
}

#[test]
fn evaluate_animation_at_frame() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("Failed to setup animation");

    let pos_xyz_idx;
    {
        let ssab_buf = context.get_ssab_data().unwrap();
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
        let parts_meta = ssab.parts();
        pos_xyz_idx = parts_meta.iter().position(|p| p.name() == "POS_XYZ").expect("Part POS_XYZ not found") as i16;
    }

    // Test frame 0
    {
        let frame_data_buf = context.get_frame_data(0.0).expect("Failed to get frame data at frame 0");
        let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
        let parts = frame_data.parts();
        let part_state = parts.iter().find(|p| p.part_index() == pos_xyz_idx).expect("Part state not found");
        assert_eq!(part_state.position_x(), 0.0);
        assert!(part_state.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
    }

    // Test frame 5 (interpolating)
    {
        let frame_data_buf = context.get_frame_data(5.0).expect("Failed to get frame data at frame 5");
        let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
        let parts = frame_data.parts();
        let part_state = parts.iter().find(|p| p.part_index() == pos_xyz_idx).expect("Part state not found");
        assert_eq!(part_state.position_x(), 16.0);
        assert!(part_state.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
    }

    // Test frame 10
    {
        let frame_data_buf = context.get_frame_data(10.0).expect("Failed to get frame data at frame 10");
        let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
        let parts = frame_data.parts();
        let part_state = parts.iter().find(|p| p.part_index() == pos_xyz_idx).expect("Part state not found");
        assert_eq!(part_state.position_x(), 32.0);
        assert!(part_state.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
    }

    // Test frame 11 (past the last keyframe — value held at last key).
    // The flag is ON: `evaluate_part` sets it whenever the value comes
    // from animation data, which still holds when the value is a held
    // last keyframe. Render-path consumers
    // (e.g. Godot's `use_advanced`) and cell-default fallbacks both rely
    // on this signal to avoid silently dropping animated state.
    {
        let frame_data_buf = context.get_frame_data(11.0).expect("Failed to get frame data at frame 11");
        let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
        let parts = frame_data.parts();
        let part_state = parts.iter().find(|p| p.part_index() == pos_xyz_idx).expect("Part state not found");
        assert_eq!(part_state.position_x(), 32.0);
        assert!(part_state.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
    }
}

// --- Override Layer: Visibility ---

/// Load Basic.ssab bytes for the override tests (each test owns its resource
/// locally so the `resource` outlives its `Context`, matching drop order).
fn load_basic_ssab() -> SsabData {
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");
    SsabData::load_copy(&input_buf).expect("Failed to load SsabData")
}

#[test]
fn override_find_part_index() {
    let ssab_data = load_basic_ssab();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);

    // Before an animation is set up, there is no primary → -1.
    assert_eq!(context.find_part_index("POS_XYZ"), -1);

    context.setup_animation("anime_1").expect("setup");
    // Indices are 1:1 with the SSAB part order (see load_parts_of_ssab_files).
    assert_eq!(context.find_part_index("POS_XYZ"), 2);
    assert_eq!(context.find_part_index("SCL_XY"), 7);
    assert_eq!(context.find_part_index("root"), 0);
    assert_eq!(context.find_part_index("does_not_exist"), -1);
}

#[test]
fn override_table_is_created_by_the_first_override() {
    let ssab_data = load_basic_ssab();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    // No animation yet: every part index is out of range.
    assert!(matches!(context.set_part_visibility_override(0, 1, false), Err(error::SSRuntimeError::InvalidParameter)));

    context.setup_animation("anime_1").expect("setup");
    let num_parts = context.override_part_count();
    assert!(num_parts > 1);
    assert!(context.instance.part_overrides.is_empty(), "setup allocates no table");

    // Clearing what was never set is a no-op, and still range-checked; a
    // rejected set allocates nothing.
    context.clear_part_color_override(0).expect("clear without a table");
    assert!(matches!(context.clear_part_color_override(num_parts as u32), Err(error::SSRuntimeError::InvalidParameter)));
    assert!(matches!(context.set_part_visibility_override(num_parts as u32, 1, false), Err(error::SSRuntimeError::InvalidParameter)));
    assert!(context.instance.part_overrides.is_empty());

    context.set_part_visibility_override(1, 1, false).expect("set");
    assert_eq!(context.instance.part_overrides.len(), num_parts);

    // A new animation drops the visibility override, and the empty table with it.
    context.setup_animation("anime_1").expect("re-setup");
    assert!(context.instance.part_overrides.is_empty());

    // A PERMANENT color override survives it, and keeps the table.
    context.set_part_color_override(1, 0, 0xff0000ff, 2).expect("permanent color");
    context.setup_animation("anime_1").expect("re-setup");
    assert_eq!(context.instance.part_overrides.len(), num_parts);
    assert!(context.instance.part_overrides[1].as_ref().is_some_and(|o| o.color.is_some()));
}

#[test]
fn override_visibility_force_hide_and_clear() {
    let ssab_data = load_basic_ssab();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("setup");

    let idx = context.find_part_index("POS_XYZ");
    assert!(idx >= 0);
    let idx = idx as usize;

    // Baseline: POS_XYZ is a visible drawing part at frame 0.
    context.get_frame_data(0.0).expect("frame 0");
    assert!(!context.instance.resolved_layer.parts_vec[idx].hide(), "POS_XYZ expected visible at baseline");

    // Force-hide → hidden.
    context.set_part_visibility_override(idx as u32, 1, false).expect("set");
    assert!(context.instance.has_any_override());
    context.get_frame_data(0.0).expect("frame 0");
    assert!(context.instance.resolved_layer.parts_vec[idx].hide());

    // Clear → reverts to the animation's value (visible again).
    context.clear_part_visibility_override(idx as u32).expect("clear");
    assert!(!context.instance.has_any_override(), "empty entry should compact to None");
    context.get_frame_data(0.0).expect("frame 0");
    assert!(!context.instance.resolved_layer.parts_vec[idx].hide());
}

#[test]
fn override_visibility_cascade() {
    let ssab_data = load_basic_ssab();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("setup");

    let parent = context.find_part_index("LSCL_XY"); // id 6
    let child = context.find_part_index("SCL_XY"); // id 7, parent 6
    assert!(parent >= 0 && child >= 0);
    let (parent, child) = (parent as u32, child as usize);

    // Cascade force-hide the parent → the child is hidden too.
    context.set_part_visibility_override(parent, 1, true).expect("set cascade");
    context.get_frame_data(0.0).expect("frame 0");
    assert!(context.instance.resolved_layer.parts_vec[parent as usize].hide());
    assert!(context.instance.resolved_layer.parts_vec[child].hide(), "cascade should hide the child");

    // Non-cascade: the child reverts to its own animated visibility
    // (keyframed hide does NOT cascade — this proves the override cascade
    // is the only thing that reached the child).
    context.clear_all_part_overrides();
    context.get_frame_data(0.0).expect("frame 0");
    let child_baseline = context.instance.resolved_layer.parts_vec[child].hide();
    context.set_part_visibility_override(parent, 1, false).expect("set no-cascade");
    context.get_frame_data(0.0).expect("frame 0");
    assert!(context.instance.resolved_layer.parts_vec[parent as usize].hide());
    assert_eq!(context.instance.resolved_layer.parts_vec[child].hide(), child_baseline, "non-cascade must not touch the child");
}

#[test]
fn override_cleared_on_setup_animation() {
    let ssab_data = load_basic_ssab();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("setup");

    let idx = context.find_part_index("POS_XYZ") as u32;
    context.set_part_visibility_override(idx, 1, false).expect("set");
    assert!(context.instance.has_any_override());

    // A new animation clears visibility overrides.
    context.setup_animation("anime_1").expect("re-setup");
    assert!(!context.instance.has_any_override());
}

// --- Override Layer: Color ---

/// Read the `lt` corner rgba of a part's resolved color, or `None` if unset.
fn part_color_lt(ctx: &Context, idx: usize) -> Option<(u8, u8, u8, u8)> {
    let pc_idx = ctx.instance.resolved_layer.parts_vec[idx].part_color();
    if pc_idx < 0 {
        return None;
    }
    let c = &ctx.instance.resolved_layer.parts_color_vec[pc_idx as usize];
    let rgba = c.lt().rgba();
    Some((rgba.r(), rgba.g(), rgba.b(), rgba.a()))
}

#[test]
fn override_color_applies_and_premultiplies() {
    let ssab_data = load_basic_ssab();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("setup");

    // POS_XYZ has no keyframed color; the override is the only color source.
    let idx = context.find_part_index("POS_XYZ") as usize;
    context.get_frame_data(0.0).expect("frame 0");
    assert!(part_color_lt(&context, idx).is_none(), "POS_XYZ expected no baseline color");

    // Mix, opaque 0x80/0x40/0x20/0xFF, NOW priority.
    context.set_part_color_override(idx as u32, 0, 0x8040_20FF, 1).expect("set color");
    context.get_frame_data(0.0).expect("frame 0");

    let part_alpha = context.instance.resolved_layer.parts_vec[idx].alpha();
    let expected_a = ((1.0 * part_alpha) * 255.0).clamp(0.0, 255.0) as u8;
    let (r, g, b, a) = part_color_lt(&context, idx).expect("override color present");
    assert_eq!((r, g, b), (0x80, 0x40, 0x20));
    assert_eq!(a, expected_a, "alpha must be pre-multiplied by the part alpha");
    // Center-vertex flag must be set so compose emits per-vertex color.
    assert!(context.instance.resolved_layer.parts_vec[idx].update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePartColor));
}

#[test]
fn override_color_normal_only() {
    let ssab_data = load_basic_ssab();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("setup");

    // root (index 0) is PartTypeNull → color override is a no-op.
    context.set_part_color_override(0, 0, 0xFFFF_FFFF, 1).expect("set");
    context.get_frame_data(0.0).expect("frame 0");
    assert!(part_color_lt(&context, 0).is_none(), "non-Normal part must not take a color override");
}

#[test]
fn override_color_priority_non_vs_now() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/PartsColor.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("read PartsColor.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("load");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    let anim = resource.get_animation_names().first().expect("an animation").clone();
    context.setup_animation(&anim).expect("setup");

    // MIX_Whole is a Normal part with keyframed PartColor.
    let idx = context.find_part_index("MIX_Whole");
    assert!(idx >= 0);
    let idx = idx as usize;

    // Baseline: a keyframed color exists at frame 0 (flag ON).
    context.get_frame_data(0.0).expect("frame 0");
    part_color_lt(&context, idx).expect("keyframed color present at frame 0");
    assert!(context.instance.resolved_layer.parts_vec[idx].update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePartColor));

    let interval = 1000.0 / context.get_frame_rate().max(1) as f32;
    context.play();

    // NON set right after play(): the first update sits on frame 0 (its own
    // keyframe), which is skipped — so the override WINS its set-frame
    // segment even though color is animated (the old bug hid it entirely).
    context.set_part_color_override(idx as u32, 0, 0x0102_03FF, 0).expect("set NON");
    let f = context.update(0.0);
    context.get_frame_data(f).expect("frame 0");
    let (r, g, b, _a) = part_color_lt(&context, idx).expect("override color present");
    assert_eq!((r, g, b), (0x01, 0x02, 0x03), "NON wins at its set frame (own keyframe skipped)");
    assert!(context.instance.part_overrides[idx].as_ref().unwrap().color.is_some(), "slot retained on the set frame");

    // Play a full loop: playback decodes a PartColor key past frame 0 (or
    // key@0 again on the wrap), so the one-shot hands over and drops.
    let end = context.get_end_frame();
    for _ in 0..(end + 2) {
        let f = context.update(interval);
        context.get_frame_data(f).expect("frame");
    }
    assert!(context.instance.part_overrides[idx].as_ref().is_none_or(|o| o.color.is_none()), "NON hands over one-shot after decoding a key");

    // NOW: wins the keyframe every frame regardless of what the animation drives.
    context.set_part_color_override(idx as u32, 0, 0x0102_03FF, 1).expect("set NOW");
    let f = context.update(interval);
    context.get_frame_data(f).expect("frame");
    let (r, g, b, _a) = part_color_lt(&context, idx).expect("override color present");
    assert_eq!((r, g, b), (0x01, 0x02, 0x03), "NOW must override the keyframe");
}

#[test]
fn override_non_reads_the_same_however_often_its_update_is_evaluated() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/PartsColor.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("read PartsColor.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("load");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    let anim = resource.get_animation_names().first().expect("an animation").clone();
    context.setup_animation(&anim).expect("setup");
    let idx = context.find_part_index("MIX_Whole") as usize;
    let (cm_hash, cell_hash, _, _) = first_cell(&context);
    context.play();

    // Both set on frame 0, which keys MIX_Whole's color and cell, so only the
    // skip of the set frame keeps them. A Player may read a frame more than
    // once in an update (a part follower, an instance child behind a mask):
    // every read must keep them, as the first does.
    context.set_part_color_override(idx as u32, 0, 0x0102_03FF, 0).expect("set NON color");
    context.set_part_cell_override(idx as u32, cm_hash, cell_hash, 0).expect("set NON cell");
    let f = context.update(0.0);
    for read in 0..3 {
        context.get_frame_data(f).expect("frame 0");
        let (r, g, b, _a) = part_color_lt(&context, idx).expect("override color present");
        assert_eq!((r, g, b), (0x01, 0x02, 0x03), "read {read}: the color override");
        let rl = &context.instance.resolved_layer;
        let cv = &rl.cell_vec[rl.parts_vec[idx].cell() as usize];
        assert_eq!((cv.cellmap_name_hash(), cv.name_hash()), (cm_hash, cell_hash), "read {read}: the cell override");
        // The cell keyed at frame 0 may be this very cell, so the slot tells a handover apart.
        let slot = context.instance.part_overrides[idx].as_ref().expect("the part's overrides");
        assert!(slot.color.is_some() && slot.cell.is_some(), "read {read}: both slots held");
    }
}

#[test]
fn override_color_no_accumulation_across_frames() {
    let ssab_data = load_basic_ssab();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("setup");
    let idx = context.find_part_index("POS_XYZ") as u32;
    context.set_part_color_override(idx, 0, 0x8040_20FF, 1).expect("set");

    context.get_frame_data(0.0).expect("frame 0");
    let len1 = context.instance.resolved_layer.parts_color_vec.len();
    context.get_frame_data(1.0).expect("frame 1");
    let len2 = context.instance.resolved_layer.parts_color_vec.len();
    assert_eq!(len1, len2, "override append must not accumulate across frames");
}

#[test]
fn override_color_permanent_survives_setup() {
    let ssab_data = load_basic_ssab();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("setup");
    let idx = context.find_part_index("POS_XYZ") as u32;

    // PERMANENT persists across a new animation...
    context.set_part_color_override(idx, 0, 0x8040_20FF, 2).expect("set permanent");
    context.setup_animation("anime_1").expect("re-setup");
    assert!(context.instance.has_any_override(), "PERMANENT must survive setup_animation");

    // ...but NOW does not.
    context.clear_all_part_overrides();
    context.set_part_color_override(idx, 0, 0x8040_20FF, 1).expect("set now");
    context.setup_animation("anime_1").expect("re-setup");
    assert!(!context.instance.has_any_override(), "NOW must clear on setup_animation");
}

// --- Override Layer: Cell ---

/// `(cellmap_name_hash, cell_name_hash, size_w, size_h)` for cellmap[0]/cell[0].
fn first_cell(ctx: &Context) -> (u32, u32, f32, f32) {
    let buf = ctx.get_ssab_data().unwrap();
    let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(buf) };
    let cm = ssab.cellmaps().expect("cellmaps").get(0);
    let c = cm.cells().expect("cells").get(0);
    // FFI contract: the stored hash equals fnv1a of the name, so a host that
    // hashes the cellmap/cell names resolves to the same entry.
    assert_eq!(cm.name_hash(), fnv1a_hash_str(cm.name()));
    assert_eq!(c.name_hash(), fnv1a_hash_str(c.name()));
    let r = c.region();
    (cm.name_hash(), c.name_hash(), r.w(), r.h())
}

#[test]
fn override_cell_applies_all_channels() {
    let ssab_data = load_basic_ssab();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("setup");

    let (cm_hash, cell_hash, size_w, size_h) = first_cell(&context);
    let idx = context.find_part_index("POS_XYZ") as usize;

    context.set_part_cell_override(idx as u32, cm_hash, cell_hash, 1).expect("set cell");
    context.get_frame_data(0.0).expect("frame 0");

    let rl = &context.instance.resolved_layer;
    let ps = &rl.parts_vec[idx];
    // Identity channel.
    assert!(ps.cell() >= 0, "cell index repointed");
    assert!(ps.update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeCell));
    let cv = &rl.cell_vec[ps.cell() as usize];
    assert_eq!((cv.cellmap_name_hash(), cv.name_hash()), (cm_hash, cell_hash));
    // Texture channel (drives DrawBatch texture).
    assert_eq!(rl.cell_texture_hashes[idx], cm_hash);
    // Geometry channel (drives local vertices): size_w/size_h at [+2]/[+3].
    assert_eq!(rl.cell_meta_per_part[idx * 6 + 2], size_w);
    assert_eq!(rl.cell_meta_per_part[idx * 6 + 3], size_h);
}

#[test]
fn override_cell_normal_and_mask_only() {
    let ssab_data = load_basic_ssab();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("setup");

    let (cm_hash, cell_hash, _, _) = first_cell(&context);
    // root (index 0) is PartTypeNull → cell override is a no-op.
    context.set_part_cell_override(0, cm_hash, cell_hash, 1).expect("set");
    context.get_frame_data(0.0).expect("frame 0");
    assert_eq!(context.instance.resolved_layer.parts_vec[0].cell(), -1, "Null part must not take a cell override");
}

#[test]
fn override_cell_priority_non_vs_now() {
    let ssab_data = load_basic_ssab();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("setup");

    // "CELL" is a Normal part with a keyframed cell.
    let cell_part = context.find_part_index("CELL") as usize;
    context.get_frame_data(0.0).expect("frame 0");
    let ci = context.instance.resolved_layer.parts_vec[cell_part].cell();
    assert!(ci >= 0, "CELL part has a keyframed cell");
    assert!(context.instance.resolved_layer.parts_vec[cell_part].update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeCell));
    let (base_cm, base_cell) = {
        let b = &context.instance.resolved_layer.cell_vec[ci as usize];
        (b.cellmap_name_hash(), b.name_hash())
    };

    // Find a different cell in the same cellmap to override with.
    let alt = {
        let buf = context.get_ssab_data().unwrap();
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(buf) };
        let cellmaps = ssab.cellmaps().unwrap();
        let cm = cellmaps.iter().find(|c| c.name_hash() == base_cm).expect("baseline cellmap");
        cm.cells().unwrap().iter().map(|c| c.name_hash()).find(|&h| h != base_cell)
    };
    let Some(alt) = alt else {
        return; // single-cell cellmap — nothing distinct to prove priority with
    };

    let interval = 1000.0 / context.get_frame_rate().max(1) as f32;
    context.play();

    // NON set right after play(): the first update sits on the frame-0 cell
    // key (skipped), so the override WINS its set-frame segment.
    context.set_part_cell_override(cell_part as u32, base_cm, alt, 0).expect("set NON");
    let f = context.update(0.0);
    context.get_frame_data(f).expect("frame 0");
    let ci2 = context.instance.resolved_layer.parts_vec[cell_part].cell();
    let got = {
        let c = &context.instance.resolved_layer.cell_vec[ci2 as usize];
        (c.cellmap_name_hash(), c.name_hash())
    };
    assert_eq!(got, (base_cm, alt), "NON wins at its set frame (own key skipped)");
    assert!(context.instance.part_overrides[cell_part].as_ref().unwrap().cell.is_some());

    // Play a full loop: a cell key is decoded → one-shot handover, dropped.
    let end = context.get_end_frame();
    for _ in 0..(end + 2) {
        let f = context.update(interval);
        context.get_frame_data(f).expect("frame");
    }
    assert!(context.instance.part_overrides[cell_part].as_ref().is_none_or(|o| o.cell.is_none()), "cell NON hands over after decoding a key");

    // NOW: wins the keyframed cell every frame.
    context.set_part_cell_override(cell_part as u32, base_cm, alt, 1).expect("set NOW");
    let f = context.update(interval);
    context.get_frame_data(f).expect("frame");
    let ci3 = context.instance.resolved_layer.parts_vec[cell_part].cell();
    let got = {
        let c = &context.instance.resolved_layer.cell_vec[ci3 as usize];
        (c.cellmap_name_hash(), c.name_hash())
    };
    assert_eq!(got, (base_cm, alt), "NOW must override the keyframed cell");
}

#[test]
fn override_cell_changes_uvs() {
    let ssab_data = load_basic_ssab();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("setup");

    let cell_part = context.find_part_index("CELL") as usize;
    context.get_frame_data(0.0).expect("frame 0");
    let ci = context.instance.resolved_layer.parts_vec[cell_part].cell();
    assert!(ci >= 0);
    let (base_cm, base_cell) = {
        let b = &context.instance.resolved_layer.cell_vec[ci as usize];
        (b.cellmap_name_hash(), b.name_hash())
    };
    let base = cell_part * 10;
    let baseline_uvs: Vec<f32> = context.instance.globals.local_uvs[base..base + 10].to_vec();

    // A different cell in the same cellmap sits at a different atlas rect.
    let alt = {
        let buf = context.get_ssab_data().unwrap();
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(buf) };
        let cm = ssab.cellmaps().unwrap().iter().find(|c| c.name_hash() == base_cm).expect("baseline cellmap");
        cm.cells().unwrap().iter().map(|c| c.name_hash()).find(|&h| h != base_cell)
    };
    let Some(alt) = alt else {
        return; // single-cell cellmap — cannot prove a UV change
    };

    // Override to the alt cell: the SAMPLED UVs must change — not just the
    // quad size/texture. (Regression: compute_local_uvs used the keyframe
    // cell and ignored the override, so only size/texture changed.)
    context.set_part_cell_override(cell_part as u32, base_cm, alt, 1).expect("set NOW");
    context.get_frame_data(0.0).expect("frame 0");
    let overridden_uvs: Vec<f32> = context.instance.globals.local_uvs[base..base + 10].to_vec();

    assert_ne!(baseline_uvs, overridden_uvs, "a cell override must change the sampled UVs");
}

#[test]
fn override_cell_permanent_survival_and_sentinel_clear() {
    let ssab_data = load_basic_ssab();
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("setup");
    let (cm_hash, cell_hash, _, _) = first_cell(&context);
    let idx = context.find_part_index("POS_XYZ") as u32;

    // PERMANENT persists across a new animation...
    context.set_part_cell_override(idx, cm_hash, cell_hash, 2).expect("set permanent");
    context.setup_animation("anime_1").expect("re-setup");
    assert!(context.instance.has_any_override(), "PERMANENT cell must survive setup_animation");

    // ...and the (0,0) sentinel reverts to the animation (clears).
    context.set_part_cell_override(idx, 0, 0, 2).expect("sentinel clear");
    assert!(!context.instance.has_any_override(), "(0,0) must clear the cell override");

    // NOW does not survive setup.
    context.set_part_cell_override(idx, cm_hash, cell_hash, 1).expect("set now");
    context.setup_animation("anime_1").expect("re-setup");
    assert!(!context.instance.has_any_override(), "NOW cell must clear on setup_animation");
}

// --- Playback settings ---

#[test]
fn track_zero_settings_read_back_before_an_animation_is_laid() {
    // A Player applies its transport when the pack is bound, before it
    // selects an animation; the setters keep it on track 0 alone then, and
    // the getters must not read the default of a controller that has none.
    prepare();
    let ssab_path = get_test_base_path().join("tests/Ringo/Ringo/Ringo.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Ringo.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");

    let mut context = Context::new();
    context.bind_resource(&resource);
    context.set_speed_scale(2.0);
    context.set_skip_frames(false);
    context.set_playback_direction(core::playback_controller::PlaybackDirection::Backward, core::playback_controller::PlaybackStyle::PingPong);
    context.set_frame_rate(24);

    assert_eq!(context.get_speed_scale(), 2.0);
    assert!(!context.get_skip_frames());
    assert!(context.get_playback_direction() == core::playback_controller::PlaybackDirection::Backward);
    assert!(context.get_playback_style() == core::playback_controller::PlaybackStyle::PingPong);
    assert_eq!(context.get_frame_rate(), 24);

    // And the animation laid afterwards plays by them, reading the same.
    let names = resource.get_animation_names();
    context.setup_animation(&names[0]).expect("Failed to setup animation");
    assert_eq!(context.get_speed_scale(), 2.0);
    assert!(!context.get_skip_frames());
    assert!(context.get_playback_direction() == core::playback_controller::PlaybackDirection::Backward);
    assert!(context.get_playback_style() == core::playback_controller::PlaybackStyle::PingPong);
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
    let ssab_path = get_test_base_path().join("tests/Ringo/Ringo/Ringo.ssab");
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
    context.set_playback_direction(core::playback_controller::PlaybackDirection::Backward, core::playback_controller::PlaybackStyle::PingPong);
    context.set_frame_rate(authored_fps + 17);
    context.set_loop_count(3);

    // A transition to another clip in the same pack.
    context.setup_animation(&names[1]).expect("Failed to setup the second animation");

    assert!(!context.get_skip_frames(), "frame skip is the host's, and must survive the transition");
    assert_eq!(context.get_frame_rate(), authored_fps + 17, "the frame-rate override must survive the transition");
    assert_eq!(context.get_speed_scale(), 2.0, "the speed is the host's, and must survive the transition");
    assert!(context.get_playback_direction() == core::playback_controller::PlaybackDirection::Backward, "the playback direction is the host's, and must survive the transition");
    assert!(context.get_playback_style() == core::playback_controller::PlaybackStyle::PingPong, "so is the ping-pong style");
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

// --- Frame evaluation ---

// Regression for the subframe-end-frame fix: `get_frame_data` accepts the
// half-open interval `[start_frame, end_frame + 1)` so a Player doing
// subframe interpolation can pass the fractional overshoot that briefly
// lands in `(end_frame, end_frame + 1)` at the end / loop boundary without
// the runtime erroring (which would blank the whole pose).
#[test]
fn get_frame_data_accepts_subframe_overshoot_at_animation_end() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("Failed to setup animation");

    let start_frame = context.get_start_frame();
    let end_frame = context.get_end_frame();
    assert!(end_frame > start_frame, "fixture sanity: anime_1 should span multiple frames");

    // Pose at exactly `end_frame` (the last valid integer frame), copied out
    // because the returned slice aliases the context's reused frame buffer.
    let pose_at_end = context.get_frame_data(end_frame as f32).expect("end_frame must be valid").to_vec();
    assert!(!pose_at_end.is_empty(), "end_frame pose should not be empty");

    // Subframe overshoot into `(end_frame, end_frame + 1)` is accepted (it
    // used to return `InvalidParameter`) and holds the `end_frame` pose
    // exactly — no extrapolation past the final keyframe.
    for overshoot in [0.001_f32, 0.5, 0.999] {
        let frame_no = end_frame as f32 + overshoot;
        let pose = context.get_frame_data(frame_no).unwrap_or_else(|_| panic!("subframe overshoot {frame_no} must be accepted")).to_vec();
        assert_eq!(pose, pose_at_end, "pose at {frame_no} must hold the end_frame pose");
    }

    // Upper bound is exclusive: `end_frame + 1.0` (the start of the
    // nonexistent next frame) is still rejected, as is anything below
    // `start_frame`.
    assert!(context.get_frame_data(end_frame as f32 + 1.0).is_err(), "end_frame + 1.0 must be rejected");
    assert!(context.get_frame_data(start_frame as f32 - 0.5).is_err(), "below start_frame must be rejected");
}

#[test]
fn test_part_hierarchy_and_visibility() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("Failed to setup animation");

    let ssab_buf = context.get_ssab_data().unwrap();
    let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
    let parts_meta = ssab.parts();

    // Hierarchy check
    let scl_xy_idx = parts_meta.iter().position(|p| p.name() == "SCL_XY").expect("Part SCL_XY not found");
    let scl_xy_part = parts_meta.get(scl_xy_idx);
    let parent_idx = scl_xy_part.parent_index();
    assert!(parent_idx >= 0);
    let parent_part = parts_meta.get(parent_idx as usize);
    assert_eq!(parent_part.name(), "LSCL_XY");

    // Visibility (Hide flag) check
    let hide_part_idx = parts_meta.iter().position(|p| p.name() == "HIDE").expect("Part HIDE not found") as i16;

    // Frame 0: Not hidden
    let frame_0_buf = context.get_frame_data(0.0).unwrap();
    let frame_0 = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_0_buf) };
    let hide_state_0 = frame_0.parts().iter().find(|p| p.part_index() == hide_part_idx).unwrap();
    assert!(!hide_state_0.hide());

    // Frame 2: Hidden
    let frame_2_buf = context.get_frame_data(2.0).unwrap();
    let frame_2 = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_2_buf) };
    let hide_state_2 = frame_2.parts().iter().find(|p| p.part_index() == hide_part_idx).unwrap();
    assert!(hide_state_2.hide());
}

#[test]
fn test_world_matrices() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("Failed to setup animation");

    let frame_no = 0.0;
    let _ = context.get_frame_data(frame_no).expect("Failed to get frame data");

    let world_matrices_ptr = context.get_world_matrices_ptr();
    let world_matrices_len = context.get_world_matrices_len();

    // Length should be 16 floats per part
    let num_parts = context.instance.resolved_layer.parts_vec.len();
    assert_eq!(world_matrices_len, num_parts * 16);

    let world_matrices = unsafe { ::core::slice::from_raw_parts(world_matrices_ptr, world_matrices_len) };

    // For simplicity, just test that matrices are non-zero/valid
    assert!(world_matrices[0] != 0.0 || world_matrices[1] != 0.0);
}

#[test]
fn test_local_uvs() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("Failed to setup animation");

    let _ = context.get_frame_data(0.0).expect("Failed to get frame data");

    let uvs_ptr = context.get_local_uvs_ptr();
    let uvs_len = context.get_local_uvs_len();
    let num_parts = context.instance.resolved_layer.parts_vec.len();
    // 5 vertices * 2 floats per part.
    assert_eq!(uvs_len, num_parts * 10);

    let uvs = unsafe { ::core::slice::from_raw_parts(uvs_ptr, uvs_len) };

    // At least one cell-bearing part must have produced non-zero UVs.
    // (Cell-less parts and parts whose cell can't be resolved keep their
    // all-zero default — we don't make a stronger per-part assertion here
    // since the test fixture's exact part roster is incidental.)
    let any_nonzero = uvs.iter().any(|f| *f != 0.0);
    assert!(any_nonzero, "Expected at least one part to have non-zero UVs");

    // Find any part whose UV slot was populated with a non-degenerate
    // quad. There should be at least one such part in a basic animation.
    let any_valid_quad = (0..num_parts).any(|i| {
        let off = i * 10;
        let lt_u = uvs[off];
        let rt_u = uvs[off + 2];
        let lt_v = uvs[off + 1];
        let lb_v = uvs[off + 5];
        (rt_u - lt_u).abs() > f32::EPSILON && (lb_v - lt_v).abs() > f32::EPSILON
    });
    assert!(any_valid_quad, "Expected at least one part to have a non-degenerate UV quad");

    // Recomputing on a different frame should still yield a sane buffer.
    let _ = context.get_frame_data(1.0).expect("Failed to get frame data on frame 1");
    let uvs_after = unsafe { ::core::slice::from_raw_parts(context.get_local_uvs_ptr(), context.get_local_uvs_len()) };
    assert_eq!(uvs_after.len(), num_parts * 10);
    assert!(uvs_after.iter().any(|f| *f != 0.0));
}

#[test]
fn test_cell_meta_per_part() {
    // Per-part cell metadata exposed via FFI lets Players drop their own
    // cellmap binary search. Layout: 6 floats per part
    // `[pivot_x, pivot_y, size_w, size_h, rect_left, rect_top]`.
    // Cell-bearing parts populate it; the rest stay zero.
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("Failed to setup animation");

    let _ = context.get_frame_data(0.0).expect("Failed to get frame data");

    let meta_ptr = context.get_cell_meta_ptr();
    let meta_len = context.get_cell_meta_len();
    let num_parts = context.instance.resolved_layer.parts_vec.len();
    assert_eq!(meta_len, num_parts * 6);

    let meta = unsafe { ::core::slice::from_raw_parts(meta_ptr, meta_len) };

    // At least one cell-bearing part must have non-zero size_w/size_h
    // (zero pivot is legal for a centered cell, so use size as the signal).
    let any_nonzero_size = (0..num_parts).any(|i| {
        let off = i * 6;
        meta[off + 2] != 0.0 || meta[off + 3] != 0.0
    });
    assert!(any_nonzero_size, "Expected at least one part to have non-zero cell size_w/size_h");

    // Recomputing should produce a sane buffer of the same shape.
    let _ = context.get_frame_data(1.0).expect("Failed to get frame data on frame 1");
    let meta_after = unsafe { ::core::slice::from_raw_parts(context.get_cell_meta_ptr(), context.get_cell_meta_len()) };
    assert_eq!(meta_after.len(), num_parts * 6);
}

#[test]
fn test_deform_animation() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/allAttributeV7/allAttributeV7/deform.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read deform.ssab");

    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");
    let mut context = Context::new();
    context.set_render_mode(crate::core::RenderMode::SceneGraph);
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("Failed to setup animation");

    let ssab_buf = context.get_ssab_data().unwrap();
    let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };

    let mut deform_part_idx = -1;
    if let Some(animation) = ssab.animations().iter().find(|a| a.name() == "anime_1") {
        let parts_anime = animation.parts_animation_data();
        for (idx, p_anime) in parts_anime.iter().enumerate() {
            if p_anime.deform_keys() != ssab::NO_KEYS {
                deform_part_idx = idx as i32;
                break;
            }
        }
    }
    assert!(deform_part_idx >= 0, "No part with deform animation found in anime_1");

    // Test frame 0
    {
        let frame_0_buf = context.get_frame_data(0.0).unwrap();
        let frame_0 = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_0_buf) };
        let parts = frame_0.parts();
        let deform_state = parts.iter().find(|p| p.part_index() == deform_part_idx as i16).expect("Deform part not found in frame 0");

        if deform_state.deform() >= 0 {
            let deforms = frame_0.deforms().unwrap();
            let deform_attr = deforms.get(deform_state.deform() as usize);
            if let Some(coords) = deform_attr.coordinate() {
                println!("Frame 0 Deform found with {} coordinates", coords.len());
            }
        }
    }

    // Test frame 10: Interpolated deform
    {
        let frame_10_buf = context.get_frame_data(10.0).unwrap();
        let frame_10 = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_10_buf) };
        let parts = frame_10.parts();
        let deform_state = parts.iter().find(|p| p.part_index() == deform_part_idx as i16).expect("Deform part not found in frame 10");

        println!("Frame 10 Deform index: {}", deform_state.deform());
        if deform_state.deform() >= 0 {
            let deforms = frame_10.deforms().unwrap();
            let deform_attr = deforms.get(deform_state.deform() as usize);
            println!("Frame 10 Coordinates: {:?}", deform_attr.coordinate().map(|c| c.len()));
            println!("Frame 10 Vertex Indices: {:?}", deform_attr.index().map(|i| i.len()));
        }
    }
}

#[test]
fn test_shape_default_size_64x64() {
    // SS6 SsAnimeDecoder::updateState assigns a (64,64) initial size to
    // shape parts before reading animation data. The fixture's shape parts
    // (`shape_1`/`shape_2`/`shape_3` in `shape.ssab`) have no SizeX/SizeY
    // keyframes, so the runtime must surface the SS6 default instead of the
    // cell-size fallback used for cell-bearing types.
    prepare();
    let ssab_path = get_test_base_path().join("tests/allPartsV7/allPartsV7/shape.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read shape.ssab");

    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("Failed to setup animation");

    let ssab_buf = context.get_ssab_data().unwrap();
    let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
    let parts_meta = ssab.parts();
    let shape_idx = parts_meta.iter().position(|p| p.name() == "shape_1").expect("shape_1 not found") as i16;

    let frame_data_buf = context.get_frame_data(0.0).expect("Failed to get frame data");
    let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(frame_data_buf) };
    let parts = frame_data.parts();
    let state = parts.iter().find(|p| p.part_index() == shape_idx).expect("shape_1 state not found");

    assert_eq!(state.size_x(), 64.0, "Shape part default size_x must be 64.0");
    assert_eq!(state.size_y(), 64.0, "Shape part default size_y must be 64.0");
}

// --- Events ---

#[test]
fn test_event_collection_integration() {
    prepare();
    let mut base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base.pop();
    base.pop();
    let path = base.join("tests/overall/overall/UserData.ssab");
    let data = std::fs::read(path).expect("Failed to read test ssab file");

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
/// The default threshold used to be `1.0`, which a lone overlay's weight
/// meets, so an overlay fired every event it passed.
#[test]
fn an_overlay_fires_events_only_once_it_opts_in() {
    prepare();
    let data = std::fs::read(get_test_base_path().join("tests/overall/overall/UserData.ssab")).expect("Failed to read UserData.ssab");
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
    let data = std::fs::read(get_test_base_path().join("tests/overall/overall/UserData.ssab")).expect("Failed to read UserData.ssab");
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

// --- SsabData and SsabResource ---

#[test]
fn test_ssab_data_and_resource_separation() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    assert_eq!(ssab_data.as_slice().len(), input_buf.len());

    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to create AnimationResource");

    let mut context = Context::new();
    context.bind_resource(&resource);

    let names = resource.get_animation_names();
    assert!(!names.is_empty());
    assert!(names.contains(&alloc::string::String::from("anime_1")));

    context.setup_animation("anime_1").expect("Failed to setup animation");
    assert_eq!(context.get_frame_no(), 0.0);
}

#[test]
fn test_ssab_resource_accessors() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let res = unsafe { SsabResource::new(ssab_data) };

    assert!(res.resource().is_some());

    assert_eq!(res.data().as_slice().len(), input_buf.len());

    let ptr = res.resource_static_ptr();
    assert!(!ptr.is_null());
    let safe_ref_addr = res.resource().unwrap() as *const _ as usize;
    assert_eq!(ptr as usize, safe_ref_addr);
}

#[test]
fn test_load_borrow_rejects_null_pointer() {
    // `load_borrow` must not construct an SsabData wrapping a null pointer,
    // because `as_slice` would then call `from_raw_parts(null, len)` (UB for len > 0).
    let result = unsafe { SsabData::load_borrow(::core::ptr::null(), 16) };
    match result {
        Err(error::SSRuntimeError::InvalidParameter) => {}
        _ => panic!("Expected InvalidParameter for null pointer"),
    }
}

#[test]
fn test_ssab_resource_arc_sharing_keeps_buffer_alive() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/Basic.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Basic.ssab");

    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let res = unsafe { SsabResource::new(ssab_data) };

    // Take a raw pointer, then drop the original Arc handle while a clone keeps
    // the allocation alive. The pointer must remain valid through the clone.
    let ptr = res.resource_static_ptr();
    assert!(!ptr.is_null());
    let clone = res.clone();
    drop(res);

    // Safety: `clone` still owns the allocation.
    let names = unsafe { (*ptr).get_animation_names() };
    assert!(names.contains(&alloc::string::String::from("anime_1")));
    drop(clone);
}

// --- mix_tracks ---

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
            Some((dx, dy)) => crate::core::framedata::RawVertexDeform { present: true, dx, dy },
            None => crate::core::framedata::RawVertexDeform::default(),
        };
        layer.raw_vertex_deform.push(Some(alloc::boxed::Box::new(rvd)));
    }
    layer
}

/// Resolve one `Mix` track at full alpha, with the primary's own pose as the
/// floor — the shape every pre-track test was written against, so those tests
/// keep asserting exactly what they did before the stack existed.
fn mix_poses<'b>(sources: &[SourceState], layers: &[LayerState<'b>], primary_idx: usize, resolved_layer: &mut LayerState<'b>) {
    mix_tracks(sources, layers, &[TrackState::new()], primary_idx, &[], resolved_layer);
}

fn mix_source(weight: f32) -> SourceState {
    let mut s = SourceState::new();
    s.animation_idx = 0; // any non-negative value makes it active
    s.weight = weight;
    s
}

#[test]
fn mix_poses_single_source_passthrough() {
    let layers = vec![build_mix_layer(&[(10.0, 20.0, 1.5, 2.0, 0.5, None)])];
    let sources = vec![mix_source(1.0)];
    let mut resolved = LayerState::new();

    mix_poses(&sources, &layers, 0, &mut resolved);

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
fn mix_poses_blends_every_continuous_attribute() {
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
    mix_poses(&sources, &layers, 0, &mut resolved);

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
fn mix_poses_renormalizes_weights_per_part() {
    let primary = build_mix_layer(&[(10.0, 20.0, 0.0, 1.0, 1.0, None), (70.0, 80.0, 0.0, 4.0, 0.25, None)]);
    let secondary = build_mix_layer(&[(50.0, 60.0, 0.0, 3.0, 0.0, None)]);
    let layers = vec![primary, secondary];
    let sources = vec![mix_source(1.0), mix_source(1.0)];
    let mut resolved = LayerState::new();

    mix_poses(&sources, &layers, 0, &mut resolved);

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
fn mix_poses_excludes_sources_that_do_not_key_the_attribute() {
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

    mix_poses(&sources, &layers, 0, &mut resolved);

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
fn mix_poses_uncovered_attribute_keeps_primary_value() {
    use runtime::framedata::ss::runtime::UpdateAttributeFlags as F;

    let mut primary = build_mix_layer(&[(10.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let mut secondary = build_mix_layer(&[(90.0, 0.0, 0.0, 1.0, 1.0, None)]);
    set_mix_layer_coverage(&mut primary, &[F::AttributePosX]);
    set_mix_layer_coverage(&mut secondary, &[F::AttributePosX]);

    let layers = vec![primary, secondary];
    let sources = vec![mix_source(1.0), mix_source(1.0)];
    let mut resolved = LayerState::new();

    mix_poses(&sources, &layers, 0, &mut resolved);

    let p = &resolved.parts_vec[0];
    assert_eq!(p.position_x(), 50.0, "covered by both: an ordinary 50/50 blend");
    // size_x is covered by neither. `build_mix_layer` sets 10.0 on both, and
    // the primary's value is what survives.
    assert_eq!(p.size_x(), 10.0);
}

/// Weights still normalize among the sources that *do* cover an attribute,
/// so an uneven split stays proportional.
#[test]
fn mix_poses_coverage_normalizes_among_covering_sources_only() {
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

    mix_poses(&sources, &layers, 0, &mut resolved);

    // 1:3 between the two covering sources => 75.0, not 3.0.
    assert_eq!(resolved.parts_vec[0].position_x(), 75.0);
}

/// When no active source covers a part, the mixer must leave the primary's
/// pose alone rather than write zeroed accumulators over it.
#[test]
fn mix_poses_keeps_primary_pose_for_uncovered_parts() {
    let primary = build_mix_layer(&[(10.0, 20.0, 0.0, 1.0, 1.0, None), (70.0, 80.0, 0.0, 4.0, 0.25, None)]);
    let secondary = build_mix_layer(&[(50.0, 60.0, 0.0, 3.0, 0.5, None)]);
    let layers = vec![primary, secondary];
    // The primary source is inactive, so only the shorter secondary blends.
    let sources = vec![mix_source(0.0), mix_source(1.0)];
    let mut resolved = LayerState::new();

    mix_poses(&sources, &layers, 0, &mut resolved);

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
fn mix_poses_50_50_blend_trsa() {
    let layer_a = build_mix_layer(&[(10.0, 20.0, 0.0, 1.0, 1.0, None)]);
    let layer_b = build_mix_layer(&[(30.0, 40.0, ::core::f32::consts::FRAC_PI_2, 2.0, 0.5, None)]);
    let sources = vec![mix_source(1.0), mix_source(1.0)];
    let layers = vec![layer_a, layer_b];
    let mut resolved = LayerState::new();

    mix_poses(&sources, &layers, 0, &mut resolved);

    let r = &resolved.parts_vec[0];
    assert!((r.position_x() - 20.0).abs() < 1e-6);
    assert!((r.position_y() - 30.0).abs() < 1e-6);
    assert!((r.rotation_z() - ::core::f32::consts::FRAC_PI_4).abs() < 1e-4);
    assert!((r.scale_x() - 1.5).abs() < 1e-6);
    assert!((r.alpha() - 0.75).abs() < 1e-6);
}

#[test]
fn mix_poses_auto_normalizes_weights() {
    let layer_a = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layer_b = build_mix_layer(&[(100.0, 100.0, 0.0, 1.0, 1.0, None)]);
    let layers = vec![layer_a, layer_b];

    let mut resolved_a = LayerState::new();
    mix_poses(&[mix_source(0.7), mix_source(0.3)], &layers, 0, &mut resolved_a);

    let mut resolved_b = LayerState::new();
    mix_poses(&[mix_source(7.0), mix_source(3.0)], &layers, 0, &mut resolved_b);

    // Same ratio → same mix outcome regardless of absolute magnitudes.
    assert!((resolved_a.parts_vec[0].position_x() - resolved_b.parts_vec[0].position_x()).abs() < 1e-5);
    assert!((resolved_a.parts_vec[0].position_x() - 30.0).abs() < 1e-5);
}

#[test]
fn mix_poses_inactive_animation_idx_skipped() {
    let layer_a = build_mix_layer(&[(10.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layer_b = build_mix_layer(&[(90.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layers = vec![layer_a, layer_b];

    let mut src_b = mix_source(1.0);
    src_b.animation_idx = -1; // removed slot — must be skipped
    let sources = vec![mix_source(1.0), src_b];

    let mut resolved = LayerState::new();
    mix_poses(&sources, &layers, 0, &mut resolved);

    // Only source A contributes; resolved == primary verbatim.
    assert_eq!(resolved.parts_vec[0].position_x(), 10.0);
}

#[test]
fn mix_poses_zero_weight_source_skipped() {
    let layer_a = build_mix_layer(&[(10.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layer_b = build_mix_layer(&[(90.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layers = vec![layer_a, layer_b];

    let sources = vec![mix_source(1.0), mix_source(0.0)];

    let mut resolved = LayerState::new();
    mix_poses(&sources, &layers, 0, &mut resolved);

    assert_eq!(resolved.parts_vec[0].position_x(), 10.0);
}

#[test]
fn mix_poses_total_weight_zero_uses_primary() {
    let layer_a = build_mix_layer(&[(7.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layer_b = build_mix_layer(&[(99.0, 0.0, 0.0, 1.0, 1.0, None)]);
    let layers = vec![layer_a, layer_b];

    // All weights zero. Fall back to primary's pose.
    let sources = vec![mix_source(0.0), mix_source(0.0)];

    let mut resolved = LayerState::new();
    mix_poses(&sources, &layers, 0, &mut resolved);

    assert_eq!(resolved.parts_vec[0].position_x(), 7.0);
}

#[test]
fn mix_poses_raw_vertex_deform_weighted() {
    let layer_a = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, Some(([1.0, 2.0, 3.0, 4.0], [10.0, 20.0, 30.0, 40.0])))]);
    let layer_b = build_mix_layer(&[(0.0, 0.0, 0.0, 1.0, 1.0, Some(([5.0, 6.0, 7.0, 8.0], [50.0, 60.0, 70.0, 80.0])))]);
    let sources = vec![mix_source(1.0), mix_source(1.0)];
    let layers = vec![layer_a, layer_b];

    let mut resolved = LayerState::new();
    mix_poses(&sources, &layers, 0, &mut resolved);

    let d = resolved.raw_vertex_deform[0].as_deref().unwrap();
    assert!(d.present);
    assert!((d.dx[0] - 3.0).abs() < 1e-6);
    assert!((d.dx[3] - 6.0).abs() < 1e-6);
    assert!((d.dy[0] - 30.0).abs() < 1e-6);
    assert!((d.dy[3] - 60.0).abs() < 1e-6);
}

#[test]
fn mix_poses_update_flag_or_merged() {
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
    mix_poses(&sources, &layers, 0, &mut resolved);

    let f = resolved.parts_vec[0].update_flag();
    assert!(f.contains(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePosX));
    assert!(f.contains(runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeRotZ));
}

// --- Tracks and crossfades ---

fn load_ringo_resource() -> SsabData {
    prepare();
    let ssab_path = get_test_base_path().join("tests/Ringo/Ringo/Ringo.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read Ringo.ssab");
    SsabData::load_copy(&input_buf).expect("Failed to load SsabData")
}

fn evaluate_ringo(resource: &AnimationResource<'_>, anim: &str) -> Vec<(f32, f32, f32, f32)> {
    let mut ctx = Context::new();
    ctx.bind_resource(resource);
    ctx.setup_animation(anim).unwrap_or_else(|_| panic!("setup_animation({anim}) failed"));
    let start = ctx.instance.primary_source().controller.get_start_frame() as f32;
    let _ = ctx.get_frame_data(start).expect("get_frame_data failed");
    ctx.instance.resolved_layer.parts_vec.iter().map(|p| (p.position_x(), p.position_y(), p.rotation_z(), p.alpha())).collect()
}

/// Sample the resolved pose of every part, for comparing two setups.
fn resolved_of(ctx: &Context<'_>) -> Vec<(f32, f32, f32, f32)> {
    ctx.instance.resolved_layer.parts_vec.iter().map(|p| (p.position_x(), p.position_y(), p.rotation_z(), p.alpha())).collect()
}

fn ringo_ctx<'a>(resource: &'a AnimationResource<'a>, anim: &str) -> (Context<'a>, f32) {
    let mut ctx = Context::new();
    ctx.bind_resource(resource);
    ctx.setup_animation(anim).unwrap();
    let start = ctx.instance.primary_source().controller.get_start_frame() as f32;
    (ctx, start)
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
    let run_start = ctx.instance.sources[handle as usize].controller.get_start_frame();
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
    assert_eq!(blended, crate::core::framedata::MIXED_ATTRIBUTES);
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
    let setup = walk.setup_pose(ctx.instance.coordinate_system, ctx.instance.origin, ctx.instance.unit).to_vec();
    for (i, part) in ctx.instance.resolved_layer.parts_vec.iter().enumerate() {
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
    let walk = ctx.instance.tracks[0].main;

    ctx.crossfade(0, "run", 1000.0).unwrap();
    assert_eq!(ctx.get_track_fade_progress(0), Some(0.0));
    assert_eq!(ctx.get_source_weight(walk), Some(1.0));

    ctx.update(500.0);
    let p = ctx.get_track_fade_progress(0).unwrap();
    assert!((p - 0.5).abs() < 1e-3, "progress {p}");
    assert!((ctx.get_source_weight(walk).unwrap() - 0.5).abs() < 1e-3);

    ctx.update(600.0);
    assert!(ctx.get_track_fade_progress(0).is_none());
    assert_ne!(ctx.instance.tracks[0].main, walk);
    assert!(ctx.get_source_weight(walk).is_none(), "the outgoing source is released");
    assert_eq!(ctx.instance.primary, ctx.instance.tracks[0].main, "track 0 still carries the structure");
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
    ctx.instance.sources.iter().enumerate().filter(|(_, s)| s.animation_idx >= 0 && s.track == track).map(|(i, s)| (i as u32, s.weight)).collect()
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
    let walk = ctx.instance.tracks[0].main;

    ctx.crossfade(0, "run", 1000.0).unwrap();
    let run = ctx.instance.sources.iter().enumerate().position(|(i, s)| i as u32 != walk && s.animation_idx >= 0).unwrap() as u32;
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
    assert_eq!(left[0], (ctx.instance.tracks[0].main, 1.0));
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
    let wait = ctx.instance.tracks[0].main;

    ctx.crossfade(0, "attack1", 1000.0).unwrap();
    let attack = ctx.get_track_main_source(0).unwrap();
    assert_ne!(attack, wait, "the incoming animation is the track's main one at once");
    assert_eq!(ctx.instance.primary, attack, "and on track 0, the primary");
    assert_eq!(ctx.get_source_animation_index(attack), Some(resource.find_animation_index("attack1").unwrap() as u32));
    assert_eq!(ctx.get_source_weight(attack), Some(0.0), "while its pose has not started to show");
    assert_eq!(ctx.get_source_weight(wait), Some(1.0));

    let frame = ctx.update(1000.0 / 60.0);
    ctx.get_frame_data(frame).expect("the frame update returns is the incoming animation's");
    assert_eq!(ctx.instance.resolved_layer.cell_ids, ctx.instance.layers[attack as usize].cell_ids, "the discrete attributes are the incoming animation's");
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
    use crate::core::playback_controller::{PlaybackDirection, PlaybackStyle};
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
    let run = &ctx.instance.sources[ctx.get_track_main_source(0).unwrap() as usize].controller;
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
    let speed_of = |ctx: &Context<'_>, track: u32| ctx.instance.sources[ctx.get_track_main_source(track).unwrap() as usize].controller.get_speed_scale();
    assert_eq!(speed_of(&ctx, 1), 0.5);

    ctx.set_track_animation(1, "dance").unwrap();
    assert_eq!(speed_of(&ctx, 1), 0.5, "a new animation on the track keeps the track's speed");

    ctx.crossfade(1, "walk", 1000.0).unwrap();
    ctx.set_track_speed(1, 0.25).unwrap();
    for (handle, _) in track_weights(&ctx, 1) {
        assert_eq!(ctx.instance.sources[handle as usize].controller.get_speed_scale(), 0.25, "source {handle} missed the change");
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
    assert_ne!(ctx.instance.primary, 0, "the crossfade moved the primary off slot 0");

    ctx.setup_animation("dance").unwrap();
    assert_eq!(ctx.get_speed_scale(), 2.0);
    assert_eq!(ctx.instance.primary_source().controller.get_speed_scale(), 2.0);
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
    let walk = ctx.instance.tracks[0].main;

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
    let walk = ctx.instance.tracks[0].main;
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
    let walk = ctx.instance.tracks[0].main;

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
    assert_eq!(ctx.instance.primary, ctx.instance.tracks[0].main);
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
    assert_eq!(ctx.instance.primary, 0);
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
    assert_eq!(ctx.instance.primary, 0);
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

// --- The character's clock ---------------------------------------------
//
// Playback state is the character's, not track 0's: every source a track
// holds advances while the character plays, and pause / stop / play act on
// the whole stack. Before this, a source attached by `set_track_animation`
// or `crossfade` was set up but never started, so an overlay sat on its
// first frame and a finished crossfade left the character frozen.

/// Step `ctx` by `ms` milliseconds in 60 fps ticks, the way a host does.
fn tick(ctx: &mut Context<'_>, ms: f32) {
    let step: f32 = 1000.0 / 60.0;
    let mut left = ms;
    while left > 0.0 {
        ctx.update(step.min(left));
        left -= step;
    }
}

/// A playing Ringo on `walk`, its clock running.
fn playing_ringo<'a>(resource: &'a AnimationResource<'a>) -> Context<'a> {
    let (mut ctx, _) = ringo_ctx(resource, "walk");
    ctx.play();
    tick(&mut ctx, 100.0);
    assert!(ctx.get_track_frame(0).unwrap() > 0.0, "fixture sanity: track 0 advances");
    ctx
}

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
    let incoming = ctx.instance.sources.len() - 1;
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
    let start = ctx.instance.primary_source().controller.get_start_frame() as f32;
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

// --- Track sync ----------------------------------------------------------

/// Where track 0's main animation stands in its section, as a fraction.
fn primary_phase(ctx: &Context<'_>) -> f32 {
    let c = &ctx.instance.primary_source().controller;
    (c.get_frame_no() - c.get_start_frame() as f32) / (c.get_end_frame() - c.get_start_frame() + 1) as f32
}

/// The frame `phase` of the way through `handle`'s section.
fn frame_at_phase(ctx: &Context<'_>, handle: u32, phase: f32) -> f32 {
    let c = &ctx.instance.sources[handle as usize].controller;
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
    let incoming = ctx.instance.sources.len() as u32 - 1;
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

// --- InstanceSlot ---

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

/// A context with `anime` set up, and the index of its nines part, for the
/// Nine-Slice tests below.
fn setup_nines_context<'a>(resource: &'a AnimationResource<'a>, anime: &str) -> (Context<'a>, usize) {
    let mut context = Context::new();
    context.bind_resource(resource);
    context.setup_animation(anime).expect("Failed to setup animation");

    let ssab_buf = context.get_ssab_data().unwrap();
    let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
    let part_index = ssab.parts().iter().position(|p| p.part_type_type() == ssab::ssab::ss::format::PartType::PartTypeNines).expect("fixture must hold a nines part");
    (context, part_index)
}

// --- Draw plan: text, Nine-Slice and texture change ---

/// What `nines_buffers` hands back: vertices, UVs and box coords, then the
/// indices and the two per-part offset tables that cut them into parts.
type NinesBuffers = (alloc::vec::Vec<f32>, alloc::vec::Vec<f32>, alloc::vec::Vec<f32>, alloc::vec::Vec<i32>, alloc::vec::Vec<u32>, alloc::vec::Vec<u32>);

fn nines_buffers(context: &Context<'_>) -> NinesBuffers {
    unsafe {
        let f = |p: *const f32, n: usize| ::core::slice::from_raw_parts(p, n).to_vec();
        (
            f(context.get_nines_vertices_ptr(), context.get_nines_vertices_len()),
            f(context.get_nines_uvs_ptr(), context.get_nines_uvs_len()),
            f(context.get_nines_box_coords_ptr(), context.get_nines_box_coords_len()),
            ::core::slice::from_raw_parts(context.get_nines_indices_ptr(), context.get_nines_indices_len()).to_vec(),
            ::core::slice::from_raw_parts(context.get_nines_vertex_offsets_ptr(), context.get_nines_vertex_offsets_len()).to_vec(),
            ::core::slice::from_raw_parts(context.get_nines_index_offsets_ptr(), context.get_nines_index_offsets_len()).to_vec(),
        )
    }
}

/// Text parts carry no runtime geometry, but they must still hold their
/// slot in the draw order so a Player can composite them by priority.
#[test]
fn test_text_parts_are_placed_in_the_draw_plan() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/allPartsV7/allPartsV7/text.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read text.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");

    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("Failed to setup animation");

    let ssab_buf = context.get_ssab_data().unwrap();
    let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(ssab_buf) };
    let text_parts: alloc::vec::Vec<usize> = ssab.parts().iter().enumerate().filter(|(_, p)| p.part_type_type() == ssab::ssab::ss::format::PartType::PartTypeText).map(|(i, _)| i).collect();
    assert!(!text_parts.is_empty(), "the fixture must hold text parts");

    let frame_buf = context.get_frame_data(1.0).expect("Failed to get frame data").to_vec();
    let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(&frame_buf) };

    let drawn: alloc::vec::Vec<usize> = frame_data.draw_order().iter().map(|slot| frame_data.parts().get(slot as usize).part_index() as usize).collect();
    for part_index in &text_parts {
        assert!(drawn.contains(part_index), "text part {part_index} must appear in draw_order");
    }

    let text_batches: alloc::vec::Vec<_> = frame_data.draw_batches().iter().filter(|b| b.kind() == crate::runtime::framedata::ss::runtime::DrawBatchKind::Text).collect();
    assert_eq!(text_batches.len(), text_parts.len(), "each text part gets its own batch");
    for batch in text_batches {
        assert_eq!(batch.count(), 1);
        // Glyphs are the Player's: no geometry, and the font is resolved
        // from the ssab rather than from the batch's texture hash.
        assert_eq!((batch.vertex_count(), batch.index_count()), (0, 0));
        assert_eq!(batch.texture_hash(), 0);
    }
    assert!(frame_data.mask_indices().is_empty(), "no text mask in this fixture");
}

/// An SSAB without nines parts must come out of the pipeline exactly as it
/// did before the Nine-Slice support: no `Nines` batches, and empty
/// geometry buffers whose CSR is flat.
#[test]
fn test_nines_leaves_ssab_without_nines_parts_alone() {
    prepare();
    for f in ["tests/overall/overall/Basic.ssab", "tests/overall/overall/Mask.ssab", "tests/overall/overall/Mesh.ssab", "tests/allPartsV7/allPartsV7/shape.ssab", "tests/Ringo/Ringo/Ringo.ssab"] {
        let path = get_test_base_path().join(f);
        let Ok(buf) = std::fs::read(&path) else { continue };
        let ssab_data = SsabData::load_copy(&buf).unwrap();
        let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.unwrap();
        for anime in resource.get_animation_names() {
            let mut context = Context::new();
            context.bind_resource(&resource);
            context.setup_animation(&anime).unwrap();
            for frame in [0.0_f32, 1.0, 3.0] {
                let fb = context.get_frame_data(frame).unwrap().to_vec();
                let fd = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(&fb) };
                assert!(fd.draw_batches().iter().all(|b| b.kind() != crate::runtime::framedata::ss::runtime::DrawBatchKind::Nines), "{f}/{anime}");
                assert_eq!(context.get_nines_vertices_len(), 0, "{f}/{anime}");
                assert_eq!(context.get_nines_indices_len(), 0, "{f}/{anime}");
                let offs = unsafe { ::core::slice::from_raw_parts(context.get_nines_vertex_offsets_ptr(), context.get_nines_vertex_offsets_len()) };
                assert_eq!(offs.len(), fd.parts().len() + 1, "{f}/{anime}");
                assert!(offs.iter().all(|&o| o == 0), "{f}/{anime}");
            }
        }
    }
}

#[test]
fn test_nines_stretch_batch_and_geometry() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/allPartsV7/allPartsV7/nineslice.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read nineslice.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");
    let (mut context, part_index) = setup_nines_context(&resource, "anime_1");

    let frame_buf = context.get_frame_data(1.0).expect("Failed to get frame data").to_vec();
    let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(&frame_buf) };

    // The part must take part in the draw plan — it used to be dropped
    // before `draw_order` was even built.
    let rank = frame_data.draw_order().iter().position(|slot| frame_data.parts().get(slot as usize).part_index() as usize == part_index).expect("the nines part must appear in draw_order");
    let batch = frame_data.draw_batches().iter().find(|b| b.kind() == crate::runtime::framedata::ss::runtime::DrawBatchKind::Nines).expect("a Nines batch must be emitted");
    assert_eq!(batch.start_rank() as usize, rank);
    assert_eq!(batch.count(), 1);
    // Stretch shares a 4x4 grid: 16 vertices, 9 quads.
    assert_eq!(batch.vertex_count(), 16);
    assert_eq!(batch.index_count(), 54);
    assert_ne!(batch.texture_hash(), 0, "the batch must bind the cell's texture");

    let (verts, uvs, box_coords, indices, vertex_offsets, index_offsets) = nines_buffers(&context);
    assert_eq!(vertex_offsets.len(), frame_data.parts().len() + 1);
    assert_eq!(index_offsets.len(), frame_data.parts().len() + 1);

    let (vs, ve) = (vertex_offsets[part_index] as usize, vertex_offsets[part_index + 1] as usize);
    let (is, ie) = (index_offsets[part_index] as usize, index_offsets[part_index + 1] as usize);
    assert_eq!(ve - vs, 16);
    assert_eq!(ie - is, 54);
    assert_eq!(verts.len(), box_coords.len());
    assert_eq!(uvs.len(), box_coords.len());

    // Indices are part-local, so they address the part's own vertex slice.
    assert!(indices[is..ie].iter().all(|&i| (0..16).contains(&i)), "indices must be local to the part's vertex range");

    // The grid spans the whole box, corner to corner.
    assert_eq!(&box_coords[vs * 2..vs * 2 + 2], &[0.0, 0.0]);
    assert_eq!(&box_coords[(ve - 1) * 2..ve * 2], &[1.0, 1.0]);

    // Its corners coincide with the plain quad the same part reports, so a
    // Player can swap between them without re-deriving the pivot.
    let local_vertices = unsafe { ::core::slice::from_raw_parts(context.get_local_vertices_ptr(), context.get_local_vertices_len()) };
    let lv = &local_vertices[part_index * 10..part_index * 10 + 10];
    assert!((verts[vs * 2] - lv[0]).abs() < 1e-4 && (verts[vs * 2 + 1] - lv[1]).abs() < 1e-4, "first grid vertex must sit on the quad's LT corner");
    assert!((verts[(ve - 1) * 2] - lv[6]).abs() < 1e-4 && (verts[(ve - 1) * 2 + 1] - lv[7]).abs() < 1e-4, "last grid vertex must sit on the quad's RB corner");

    // UVs are texture-pixel coordinates inside the cell rectangle.
    let cell_meta = unsafe { ::core::slice::from_raw_parts(context.get_cell_meta_ptr(), context.get_cell_meta_len()) };
    let (cell_w, cell_h) = (cell_meta[part_index * 6 + 2], cell_meta[part_index * 6 + 3]);
    let (rect_left, rect_top) = (cell_meta[part_index * 6 + 4], cell_meta[part_index * 6 + 5]);
    assert_eq!(&uvs[vs * 2..vs * 2 + 2], &[rect_left, rect_top]);
    assert_eq!(&uvs[(ve - 1) * 2..ve * 2], &[rect_left + cell_w, rect_top + cell_h]);
}

#[test]
fn test_nines_tile_repeats_to_fill_the_part() {
    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/NineSlice.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("Failed to read NineSlice.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("Failed to load SsabData");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("Failed to load resource");
    let (mut context, part_index) = setup_nines_context(&resource, "anime_1");

    // The fixture's part is authored with fill mode 1 (tile) and 24 px margins.
    const MARGIN: f32 = 24.0;

    let frame_buf = context.get_frame_data(1.0).expect("Failed to get frame data").to_vec();
    let frame_data = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(&frame_buf) };
    let state = frame_data.parts().iter().find(|p| p.part_index() as usize == part_index).expect("state");
    let (w, h) = (state.size_x(), state.size_y());

    let cell_meta = unsafe { ::core::slice::from_raw_parts(context.get_cell_meta_ptr(), context.get_cell_meta_len()) };
    let (cell_w, cell_h) = (cell_meta[part_index * 6 + 2], cell_meta[part_index * 6 + 3]);

    // Repeats of the middle slice needed to cover the part's middle band.
    let div = |part: f32, cell: f32| ((part - 2.0 * MARGIN) / (cell - 2.0 * MARGIN)).ceil() as u32;
    let (div_w, div_h) = (div(w, cell_w), div(h, cell_h));
    assert!(div_w > 1 && div_h > 1, "the fixture must actually repeat (got {div_w}x{div_h})");
    // 4 corners + the two horizontal edges + the two vertical + the centre.
    let quads = 4 + 2 * div_w + 2 * div_h + div_w * div_h;

    let batch = frame_data.draw_batches().iter().find(|b| b.kind() == crate::runtime::framedata::ss::runtime::DrawBatchKind::Nines).expect("a Nines batch must be emitted");
    assert_eq!(batch.vertex_count() as u32, quads * 4);
    assert_eq!(batch.index_count() as u32, quads * 6);

    let (verts, uvs, box_coords, _indices, vertex_offsets, index_offsets) = nines_buffers(&context);
    let (vs, ve) = (vertex_offsets[part_index] as usize, vertex_offsets[part_index + 1] as usize);
    assert_eq!(ve - vs, (quads * 4) as usize);
    assert_eq!(index_offsets[part_index + 1] - index_offsets[part_index], quads * 6);

    // Every repeat stays inside the part's box and inside the cell rect.
    let (rect_left, rect_top) = (cell_meta[part_index * 6 + 4], cell_meta[part_index * 6 + 5]);
    for v in vs..ve {
        let (fx, fy) = (box_coords[v * 2], box_coords[v * 2 + 1]);
        assert!((-1e-4..=1.0 + 1e-4).contains(&fx) && (-1e-4..=1.0 + 1e-4).contains(&fy), "box coord {fx},{fy} outside the part");
        let (u, v_) = (uvs[v * 2], uvs[v * 2 + 1]);
        assert!(u >= rect_left - 1e-3 && u <= rect_left + cell_w + 1e-3, "u {u} outside the cell rect");
        assert!(v_ >= rect_top - 1e-3 && v_ <= rect_top + cell_h + 1e-3, "v {v_} outside the cell rect");
    }
    assert!(verts[vs * 2..ve * 2].iter().all(|c| c.is_finite()));

    // A repeat is drawn at the source's native size, not stretched: the
    // first centre quad is exactly one middle slice wide.
    let quad_w = (box_coords[(vs + 1) * 2] - box_coords[vs * 2]) * w;
    assert!((quad_w - (cell_w - 2.0 * MARGIN)).abs() < 1e-3, "a full repeat must keep the source slice's pixel size (got {quad_w})");
}

/// TextureChange.ssae authors two parts that share the `common` cellmap: `TO_TEXTURE`
/// swaps `common.png` -> `common_2.png` at frame 1, `TO_CELLMAP` swaps an empty name
/// (no override) -> `box_00_00.png`. A TCHG override must reach `DrawBatch.texture_hash`,
/// otherwise both parts merge into one batch and only one texture can ever be bound.
#[test]
fn texture_change_splits_batches_by_overridden_texture() {
    fn fnv1a(text: &str) -> u32 {
        text.bytes().fold(2166136261u32, |hash, b| (hash ^ b as u32).wrapping_mul(16777619))
    }

    prepare();
    let ssab_path = get_test_base_path().join("tests/overall/overall/TextureChange.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("read TextureChange.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("load");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("setup");

    let cellmap_hash = fnv1a("common");
    let (common_png, common_2_png, box_png) = (fnv1a("common.png"), fnv1a("common_2.png"), fnv1a("box_00_00.png"));

    // Every texture a TCHG key names must be reversible hash -> name by the Player, including
    // names absent from the project's <ExternalTextures> list (`common.png`, `box_00_00.png`).
    {
        let buf = context.get_ssab_data().unwrap();
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(buf) };
        let externals = ssab.external_textures().expect("external_textures");
        for expected in [common_png, common_2_png, box_png] {
            assert!(externals.iter().any(|t| t.name_hash() == expected), "TCHG texture {} missing from external_textures", expected);
        }
    }

    let batches_at = |context: &mut Context, frame: f32| -> alloc::vec::Vec<(u16, u32)> {
        let fd_buf = context.get_frame_data(frame).expect("frame data");
        let fd = unsafe { crate::runtime::framedata::ss::runtime::root_as_frame_data_unchecked(fd_buf) };
        fd.draw_batches().iter().map(|b| (b.count(), b.texture_hash())).collect()
    };

    // Frame 0: TO_TEXTURE binds its authored common.png; TO_CELLMAP's empty TextureName is
    // "no override" and must fall back to the cell's own cellmap texture (not fnv1a("")).
    assert_eq!(batches_at(&mut context, 0.0), alloc::vec![(1, common_png), (1, cellmap_hash)]);

    // Frame 1: both parts override to different textures, so neither may share a batch.
    assert_eq!(batches_at(&mut context, 1.0), alloc::vec![(1, common_2_png), (1, box_png)]);

    // TCHG is a step attribute — the frame-1 values hold with no further keys.
    assert_eq!(batches_at(&mut context, 5.0), alloc::vec![(1, common_2_png), (1, box_png)]);
}

// --- EffectSlot ---

/// A slot stepped with `draw == false` -- its part hidden -- moves its clock
/// and transitions as a drawn one does, so once it draws again it builds the
/// plan of a slot that drew every frame.
#[test]
fn an_undrawn_effect_step_keeps_the_slot_in_step_with_a_drawn_one() {
    prepare();
    let input_buf = std::fs::read(get_test_base_path().join("tests/overall/overall/Effect.ssab")).expect("read pack");
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
