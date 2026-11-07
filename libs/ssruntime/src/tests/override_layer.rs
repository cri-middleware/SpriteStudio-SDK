//! Override Layer tests: visibility, color and cell overrides.

use super::*;

// --- Override Layer: Visibility ---

/// Load Basic.ssab bytes for the override tests (each test owns its resource
/// locally so the `resource` outlives its `Context`, matching drop order).
fn load_basic_ssab() -> SsabData {
    prepare();
    let ssab_path = fixture("overall/Basic.ssab");
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
    assert!(context.instance().part_overrides.is_empty(), "setup allocates no table");

    // Clearing what was never set is a no-op, and still range-checked; a
    // rejected set allocates nothing.
    context.clear_part_color_override(0).expect("clear without a table");
    assert!(matches!(context.clear_part_color_override(num_parts as u32), Err(error::SSRuntimeError::InvalidParameter)));
    assert!(matches!(context.set_part_visibility_override(num_parts as u32, 1, false), Err(error::SSRuntimeError::InvalidParameter)));
    assert!(context.instance().part_overrides.is_empty());

    context.set_part_visibility_override(1, 1, false).expect("set");
    assert_eq!(context.instance().part_overrides.len(), num_parts);

    // A new animation drops the visibility override, and the empty table with it.
    context.setup_animation("anime_1").expect("re-setup");
    assert!(context.instance().part_overrides.is_empty());

    // A Permanent color override survives it, and keeps the table.
    context.set_part_color_override(1, 0, 0xff0000ff, 2).expect("permanent color");
    context.setup_animation("anime_1").expect("re-setup");
    assert_eq!(context.instance().part_overrides.len(), num_parts);
    assert!(context.instance().part_overrides[1].as_ref().is_some_and(|o| o.color.is_some()));
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
    assert!(!context.instance().resolved_layer.parts_vec[idx].hide(), "POS_XYZ expected visible at baseline");

    // Force-hide → hidden.
    context.set_part_visibility_override(idx as u32, 1, false).expect("set");
    assert!(context.instance().has_any_override());
    context.get_frame_data(0.0).expect("frame 0");
    assert!(context.instance().resolved_layer.parts_vec[idx].hide());

    // Clear → reverts to the animation's value (visible again).
    context.clear_part_visibility_override(idx as u32).expect("clear");
    assert!(!context.instance().has_any_override(), "empty entry should compact to None");
    context.get_frame_data(0.0).expect("frame 0");
    assert!(!context.instance().resolved_layer.parts_vec[idx].hide());
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
    assert!(context.instance().resolved_layer.parts_vec[parent as usize].hide());
    assert!(context.instance().resolved_layer.parts_vec[child].hide(), "cascade should hide the child");

    // Non-cascade: the child reverts to its own animated visibility
    // (keyframed hide does NOT cascade — this proves the override cascade
    // is the only thing that reached the child).
    context.clear_all_part_overrides();
    context.get_frame_data(0.0).expect("frame 0");
    let child_baseline = context.instance().resolved_layer.parts_vec[child].hide();
    context.set_part_visibility_override(parent, 1, false).expect("set no-cascade");
    context.get_frame_data(0.0).expect("frame 0");
    assert!(context.instance().resolved_layer.parts_vec[parent as usize].hide());
    assert_eq!(context.instance().resolved_layer.parts_vec[child].hide(), child_baseline, "non-cascade must not touch the child");
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
    assert!(context.instance().has_any_override());

    // A new animation clears visibility overrides.
    context.setup_animation("anime_1").expect("re-setup");
    assert!(!context.instance().has_any_override());
}

// --- Override Layer: Color ---

/// Read the `lt` corner rgba of a part's resolved color, or `None` if unset.
fn part_color_lt(ctx: &Context, idx: usize) -> Option<(u8, u8, u8, u8)> {
    let pc_idx = ctx.instance().resolved_layer.parts_vec[idx].part_color();
    if pc_idx < 0 {
        return None;
    }
    let c = &ctx.instance().resolved_layer.parts_color_vec[pc_idx as usize];
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

    // Mix, opaque 0x80/0x40/0x20/0xFF, HoldUntilNextAnimation priority.
    context.set_part_color_override(idx as u32, 0, 0x8040_20FF, 1).expect("set color");
    context.get_frame_data(0.0).expect("frame 0");

    let part_alpha = context.instance().resolved_layer.parts_vec[idx].alpha();
    let expected_a = ((1.0 * part_alpha) * 255.0).clamp(0.0, 255.0) as u8;
    let (r, g, b, a) = part_color_lt(&context, idx).expect("override color present");
    assert_eq!((r, g, b), (0x80, 0x40, 0x20));
    assert_eq!(a, expected_a, "alpha must be pre-multiplied by the part alpha");
    // Center-vertex flag must be set so compose emits per-vertex color.
    assert!(context.instance().resolved_layer.parts_vec[idx].update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePartColor));
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
    let ssab_path = fixture("overall/PartsColor.ssab");
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
    assert!(context.instance().resolved_layer.parts_vec[idx].update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributePartColor));

    let interval = 1000.0 / context.get_frame_rate().max(1) as f32;
    context.play();

    // OverwriteOnNextKeyframe set right after play(): the first update sits on frame 0 (its own
    // keyframe), which is skipped — so the override WINS its set-frame
    // segment even though color is animated.
    context.set_part_color_override(idx as u32, 0, 0x0102_03FF, 0).expect("set OverwriteOnNextKeyframe");
    let f = context.update(0.0);
    context.get_frame_data(f).expect("frame 0");
    let (r, g, b, _a) = part_color_lt(&context, idx).expect("override color present");
    assert_eq!((r, g, b), (0x01, 0x02, 0x03), "OverwriteOnNextKeyframe wins at its set frame (own keyframe skipped)");
    assert!(context.instance().part_overrides[idx].as_ref().unwrap().color.is_some(), "slot retained on the set frame");

    // Play a full loop: playback decodes a PartColor key past frame 0 (or
    // key@0 again on the wrap), so the one-shot hands over and drops.
    let end = context.get_end_frame();
    for _ in 0..(end + 2) {
        let f = context.update(interval);
        context.get_frame_data(f).expect("frame");
    }
    assert!(context.instance().part_overrides[idx].as_ref().is_none_or(|o| o.color.is_none()), "OverwriteOnNextKeyframe hands over one-shot after decoding a key");

    // HoldUntilNextAnimation: wins the keyframe every frame regardless of what the animation drives.
    context.set_part_color_override(idx as u32, 0, 0x0102_03FF, 1).expect("set HoldUntilNextAnimation");
    let f = context.update(interval);
    context.get_frame_data(f).expect("frame");
    let (r, g, b, _a) = part_color_lt(&context, idx).expect("override color present");
    assert_eq!((r, g, b), (0x01, 0x02, 0x03), "HoldUntilNextAnimation must override the keyframe");
}

#[test]
fn override_non_reads_the_same_however_often_its_update_is_evaluated() {
    prepare();
    let ssab_path = fixture("overall/PartsColor.ssab");
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
    context.set_part_color_override(idx as u32, 0, 0x0102_03FF, 0).expect("set OverwriteOnNextKeyframe color");
    context.set_part_cell_override(idx as u32, cm_hash, cell_hash, 0).expect("set OverwriteOnNextKeyframe cell");
    let f = context.update(0.0);
    for read in 0..3 {
        context.get_frame_data(f).expect("frame 0");
        let (r, g, b, _a) = part_color_lt(&context, idx).expect("override color present");
        assert_eq!((r, g, b), (0x01, 0x02, 0x03), "read {read}: the color override");
        let rl = &context.instance().resolved_layer;
        let cv = &rl.cell_vec[rl.parts_vec[idx].cell() as usize];
        assert_eq!((cv.cellmap_name_hash(), cv.name_hash()), (cm_hash, cell_hash), "read {read}: the cell override");
        // The cell keyed at frame 0 may be this very cell, so the slot tells a handover apart.
        let slot = context.instance().part_overrides[idx].as_ref().expect("the part's overrides");
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
    let len1 = context.instance().resolved_layer.parts_color_vec.len();
    context.get_frame_data(1.0).expect("frame 1");
    let len2 = context.instance().resolved_layer.parts_color_vec.len();
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

    // Permanent persists across a new animation...
    context.set_part_color_override(idx, 0, 0x8040_20FF, 2).expect("set permanent");
    context.setup_animation("anime_1").expect("re-setup");
    assert!(context.instance().has_any_override(), "Permanent must survive setup_animation");

    // ...but HoldUntilNextAnimation does not.
    context.clear_all_part_overrides();
    context.set_part_color_override(idx, 0, 0x8040_20FF, 1).expect("set now");
    context.setup_animation("anime_1").expect("re-setup");
    assert!(!context.instance().has_any_override(), "HoldUntilNextAnimation must clear on setup_animation");
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

    let rl = &context.instance().resolved_layer;
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
    assert_eq!(context.instance().resolved_layer.parts_vec[0].cell(), -1, "Null part must not take a cell override");
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
    let ci = context.instance().resolved_layer.parts_vec[cell_part].cell();
    assert!(ci >= 0, "CELL part has a keyframed cell");
    assert!(context.instance().resolved_layer.parts_vec[cell_part].update_flag().contains(crate::runtime::framedata::ss::runtime::UpdateAttributeFlags::AttributeCell));
    let (base_cm, base_cell) = {
        let b = &context.instance().resolved_layer.cell_vec[ci as usize];
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

    // OverwriteOnNextKeyframe set right after play(): the first update sits on the frame-0 cell
    // key (skipped), so the override WINS its set-frame segment.
    context.set_part_cell_override(cell_part as u32, base_cm, alt, 0).expect("set OverwriteOnNextKeyframe");
    let f = context.update(0.0);
    context.get_frame_data(f).expect("frame 0");
    let ci2 = context.instance().resolved_layer.parts_vec[cell_part].cell();
    let got = {
        let c = &context.instance().resolved_layer.cell_vec[ci2 as usize];
        (c.cellmap_name_hash(), c.name_hash())
    };
    assert_eq!(got, (base_cm, alt), "OverwriteOnNextKeyframe wins at its set frame (own key skipped)");
    assert!(context.instance().part_overrides[cell_part].as_ref().unwrap().cell.is_some());

    // Play a full loop: a cell key is decoded → one-shot handover, dropped.
    let end = context.get_end_frame();
    for _ in 0..(end + 2) {
        let f = context.update(interval);
        context.get_frame_data(f).expect("frame");
    }
    assert!(context.instance().part_overrides[cell_part].as_ref().is_none_or(|o| o.cell.is_none()), "cell OverwriteOnNextKeyframe hands over after decoding a key");

    // HoldUntilNextAnimation: wins the keyframed cell every frame.
    context.set_part_cell_override(cell_part as u32, base_cm, alt, 1).expect("set HoldUntilNextAnimation");
    let f = context.update(interval);
    context.get_frame_data(f).expect("frame");
    let ci3 = context.instance().resolved_layer.parts_vec[cell_part].cell();
    let got = {
        let c = &context.instance().resolved_layer.cell_vec[ci3 as usize];
        (c.cellmap_name_hash(), c.name_hash())
    };
    assert_eq!(got, (base_cm, alt), "HoldUntilNextAnimation must override the keyframed cell");
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
    let ci = context.instance().resolved_layer.parts_vec[cell_part].cell();
    assert!(ci >= 0);
    let (base_cm, base_cell) = {
        let b = &context.instance().resolved_layer.cell_vec[ci as usize];
        (b.cellmap_name_hash(), b.name_hash())
    };
    let base = cell_part * 10;
    let baseline_uvs: Vec<f32> = context.instance().globals.local_uvs[base..base + 10].to_vec();

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
    // quad size/texture.
    context.set_part_cell_override(cell_part as u32, base_cm, alt, 1).expect("set HoldUntilNextAnimation");
    context.get_frame_data(0.0).expect("frame 0");
    let overridden_uvs: Vec<f32> = context.instance().globals.local_uvs[base..base + 10].to_vec();

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

    // Permanent persists across a new animation...
    context.set_part_cell_override(idx, cm_hash, cell_hash, 2).expect("set permanent");
    context.setup_animation("anime_1").expect("re-setup");
    assert!(context.instance().has_any_override(), "Permanent cell must survive setup_animation");

    // ...and the (0,0) sentinel reverts to the animation (clears).
    context.set_part_cell_override(idx, 0, 0, 2).expect("sentinel clear");
    assert!(!context.instance().has_any_override(), "(0,0) must clear the cell override");

    // HoldUntilNextAnimation does not survive setup.
    context.set_part_cell_override(idx, cm_hash, cell_hash, 1).expect("set now");
    context.setup_animation("anime_1").expect("re-setup");
    assert!(!context.instance().has_any_override(), "HoldUntilNextAnimation cell must clear on setup_animation");
}
