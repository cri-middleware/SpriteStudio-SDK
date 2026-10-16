//! Draw-plan tests: text parts, Nine-Slice geometry, and batches split by texture.

use super::*;

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
    let ssab_path = fixture("allPartsV7/text.ssab");
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
    for f in ["overall/Basic.ssab", "overall/Mask.ssab", "overall/Mesh.ssab", "allPartsV7/shape.ssab", "Ringo/Ringo.ssab"] {
        let path = fixture(f);
        let buf = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
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
    let ssab_path = fixture("allPartsV7/nineslice.ssab");
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
    let ssab_path = fixture("overall/NineSlice.ssab");
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
    prepare();
    let ssab_path = fixture("overall/TextureChange.ssab");
    let input_buf = std::fs::read(&ssab_path).expect("read TextureChange.ssab");
    let ssab_data = SsabData::load_copy(&input_buf).expect("load");
    let resource = unsafe { AnimationResource::new(ssab_data.as_slice()) }.expect("resource");
    let mut context = Context::new();
    context.bind_resource(&resource);
    context.setup_animation("anime_1").expect("setup");

    let cellmap_hash = fnv1a_hash_str("common");
    let (common_png, common_2_png, box_png) = (fnv1a_hash_str("common.png"), fnv1a_hash_str("common_2.png"), fnv1a_hash_str("box_00_00.png"));

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
