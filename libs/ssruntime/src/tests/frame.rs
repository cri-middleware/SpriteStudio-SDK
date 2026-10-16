//! Frame evaluation tests: part bases, key cursors, the frame data buffers and the evaluated pose.

use super::*;

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
                let capacity = ctx.eval().fb_capacity;
                let start = ctx.get_frame_data(f as f32).expect("frame").as_ptr() as usize;
                assert_eq!(start % 8, 0, "{name} frame {f}: frame data starts off an 8-byte boundary");
                resized += usize::from(ctx.eval().fb_capacity != capacity);
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
    let num_parts = context.instance().resolved_layer.parts_vec.len();
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
    let num_parts = context.instance().resolved_layer.parts_vec.len();
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
    let num_parts = context.instance().resolved_layer.parts_vec.len();
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
