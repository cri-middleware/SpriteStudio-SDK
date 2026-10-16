//! [`Context::get_frame_data`]: the frame pipeline that evaluates the pose and builds the frame data.

use super::{Context, InstanceState, NO_MESH_LAYOUT};
use crate::mix::mix_tracks;
use crate::resource::AnimationMetadata;
use crate::{error, eval};

impl<'a> Context<'a> {
    pub fn get_frame_data(&mut self, frame_no: f32) -> Result<&[u8], error::SSRuntimeError> {
        let Some(resource) = self.eval.resource else {
            return Err(error::SSRuntimeError::NotLoaded);
        };

        let primary_anim_idx = self.instance.primary_source().animation_idx;
        if primary_anim_idx < 0 {
            return Err(error::SSRuntimeError::AnimationNotFound);
        }

        let primary_metadata = &resource.animations[primary_anim_idx as usize];
        if !((primary_metadata.start_frame as f32) <= frame_no && frame_no < (primary_metadata.end_frame as f32 + 1.0)) {
            return Err(error::SSRuntimeError::InvalidParameter);
        }

        if self.eval.fb_need > self.eval.fb_capacity {
            // The last frame outgrew the builder, which doubled. Size it to the
            // largest frame so far, with a sixteenth to spare for frames that
            // carry a little more; the old buffer is released. The builder
            // writes back to front from the end of its buffer, so the size sets
            // where the frame data starts: a multiple of 8 keeps it on the
            // boundary `PartState` needs, as the doubled sizes did.
            self.eval.fb_capacity = (self.eval.fb_need + self.eval.fb_need / 16).next_multiple_of(8);
            self.eval.fb = flatbuffers::FlatBufferBuilder::with_capacity(self.eval.fb_capacity);
        }
        self.eval.fb.reset();
        self.eval.frame_data_ready = false;

        let coordinate_system = self.instance.coordinate_system;
        let origin = self.instance.origin;
        let unit = self.instance.unit;

        // Stage A: per-source pose extraction. Each source uses its own
        // controller-driven frame; track 0's main source uses the explicit
        // `frame_no` argument to preserve single-animation behavior. That source
        // is always extracted regardless of its weight — `mix_tracks` uses it as
        // the discrete-attribute / structure source even when its continuous
        // contribution is zero.
        let primary_idx = self.instance.primary_idx();
        let primary_num_parts = primary_metadata.cached_parts().len();
        let num_sources = self.instance.sources.len();
        // One audible source at full alpha resolves to its own pose verbatim, so
        // it is extracted straight into the resolved layer and the two-level fold
        // is skipped. The floor is unobservable here: an attribute nothing keys
        // reads the same from the setup pose as from the extracted pose.
        let single = self.instance.is_single_full_track();
        for i in 0..num_sources {
            let source_anim_idx = self.instance.sources[i].animation_idx;
            if source_anim_idx < 0 {
                continue;
            }
            if i != primary_idx && self.instance.sources[i].weight <= 0.0 {
                continue;
            }
            let frame_for_source = if i == primary_idx { frame_no } else { self.instance.sources[i].controller.get_frame_no() };
            let metadata = &resource.animations[source_anim_idx as usize];
            let into_resolved = single && i == primary_idx;
            let layer = if into_resolved { &mut self.instance.resolved_layer } else { &mut self.instance.layers[i] };
            layer.fit(metadata);
            eval::framedata::extract_pose(
                &mut self.eval.fb,
                metadata.cached_parts(),
                &resource.source.keys,
                frame_for_source,
                &mut layer.parts_vec,
                &mut layer.cell_vec,
                &mut layer.parts_color_vec,
                &mut layer.shader_vec,
                &mut layer.deform_vec,
                &mut layer.raw_vertex_deform,
                &mut layer.cursors,
                Some(metadata.part_bases(coordinate_system, origin, unit)),
                &mut layer.cell_meta_per_part,
                &mut layer.cell_texture_hashes,
                &mut layer.cell_ids,
                &mut layer.mesh_deform,
                coordinate_system,
                origin,
                unit,
                metadata.canvas_width,
                metadata.canvas_height,
                &mut self.eval.deform_interpolation_start_workspace,
                &mut self.eval.deform_interpolation_end_workspace,
                &mut self.eval.deform_interpolation_out_workspace,
                &mut self.eval.deform_result_coordinate_workspace,
                &mut self.eval.deform_result_index_workspace,
                &mut self.eval.inheritance_alphas_workspace,
            );

            // Record which attributes this animation actually carries, per part,
            // for the mix's coverage rule. Animation-level and therefore constant
            // for the source's lifetime; copied per frame because a source can be
            // repointed at another animation at any time. The resolved layer is
            // never mixed from, so it needs none.
            if !into_resolved {
                layer.coverage_vec.clear();
                layer.coverage_vec.extend(metadata.cached_parts().iter().map(|p| p.coverage));
            }
        }

        if !single {
            mix_tracks(&self.instance.sources, &self.instance.layers, &self.instance.tracks, primary_idx, primary_metadata.setup_pose(coordinate_system, origin, unit), &mut self.instance.resolved_layer);
        }

        // ⟵ SEAM: apply the per-part runtime Override Layer on the resolved
        // pose — after the mix, before compose — so overrides win over both
        // keyframe values and blend results.
        if self.instance.has_any_override() {
            // (Re)size + clear the cell-UV meta so the seam records only the
            // parts it overrides this frame; compute_local_uvs consults it.
            if self.instance.override_cell_uv_meta.len() != primary_num_parts {
                self.instance.override_cell_uv_meta.resize(primary_num_parts, None);
            }
            for slot in self.instance.override_cell_uv_meta.iter_mut() {
                *slot = None;
            }
            // NON handover is decided from the frames the primary actually
            // traversed this update (section / loop / reverse / ping-pong
            // correct). Copy them out before the mutable borrows below so the
            // controller borrow is released.
            self.eval.override_passed_frames_workspace.clear();
            self.eval.override_passed_frames_workspace.extend_from_slice(self.instance.primary_source().controller.get_passed_frames());
            crate::eval::override_layer::apply_part_overrides(
                &mut self.instance.part_overrides,
                primary_metadata.cached_parts(),
                &resource.source.keys,
                &mut self.instance.resolved_layer.parts_vec,
                &mut self.instance.resolved_layer.parts_color_vec,
                &mut self.instance.resolved_layer.cell_vec,
                &mut self.instance.resolved_layer.cell_meta_per_part,
                &mut self.instance.resolved_layer.cell_texture_hashes,
                &mut self.instance.override_cell_uv_meta,
                &mut self.eval.override_hide_workspace,
                &self.eval.override_passed_frames_workspace,
                self.instance.updates,
            );
        } else {
            // No overrides active: leave the buffer empty so compute_local_uvs
            // falls back to the keyframe cell for every part.
            self.instance.override_cell_uv_meta.clear();
        }

        // Stage B (`compute_local_vertices_for_parts`), Stage C
        // (`serialize_pose_to_fb`) and the draw plan, on the resolved pose.
        let nines = &mut self.instance.globals.nines;
        let resolved = &mut self.instance.resolved_layer;
        // A track-0 change can hand the primary role to another animation.
        self.instance.z_order.fit(primary_anim_idx, primary_metadata.cached_parts());
        let offset = eval::framedata::compose_frame_data(
            &mut self.eval.fb,
            primary_metadata.cached_parts(),
            &mut self.instance.z_order,
            &mut resolved.parts_vec,
            &resolved.cell_vec,
            &resolved.parts_color_vec,
            &resolved.shader_vec,
            &mut resolved.vertex_vec,
            &resolved.deform_vec,
            &resolved.raw_vertex_deform,
            &resolved.cell_meta_per_part,
            &resolved.cell_texture_hashes,
            &mut resolved.local_vertices,
            nines,
            coordinate_system,
            self.instance.render_mode,
            &mut self.eval.draw_order_workspace,
            &mut self.eval.batches_workspace,
            &mut self.eval.mask_indices_workspace,
        );

        // Phase 2: resolve global state (matrices / UVs / shape vertices)
        // from the (post-mix) local pose.
        Self::resolve_global_state(&mut self.instance, primary_metadata);

        self.eval.fb.finish_minimal(offset);
        self.eval.frame_data_ready = true;
        self.eval.fb_need = self.eval.fb_need.max(self.eval.fb.finished_data().len());
        Ok(self.eval.fb.finished_data())
    }

    /// Phase 2 of the frame pipeline. Consumes the (post-mix) local pose
    /// in `instance.resolved_layer` and writes per-instance global outputs
    /// in `instance.globals` — world / inheritance matrices, local UVs,
    /// and shape vertex buffers.
    fn resolve_global_state(instance: &mut InstanceState<'a>, metadata: &AnimationMetadata<'a>) {
        if instance.render_mode == crate::eval::RenderMode::Direct {
            Self::compute_matrices(instance, metadata.cached_parts());
            // Skinned mesh skinning depends on world matrices, so it lives in the
            // Direct path. Mesh parts in SceneGraph mode get empty per-part ranges.
            Self::compute_mesh_vertices(instance, metadata);
        } else {
            Self::clear_mesh_vertices(instance, metadata.cached_parts().len());
        }
        Self::compute_local_uvs(instance, metadata);
        Self::compute_shape_vertices(instance, metadata.cached_parts());
    }

    /// Writes per-part mesh world-space vertex positions to
    /// `globals.mesh_vertices_x` / `_y` for the current frame — skeletal
    /// (bone-weighted) and Deform-only meshes alike, with the frame's deform
    /// deltas applied.
    ///
    /// Layout: x/y are SoA, sliced per part by `mesh_vertex_offsets` (a CSR
    /// prefix sum of length `num_parts + 1`; non-mesh parts get an empty
    /// range). The CSR offsets and the static UV / index buffers are the
    /// animation's (`StaticMeshBuffers`); only the vertex positions are
    /// recomputed each frame.
    #[allow(clippy::needless_range_loop)]
    fn compute_mesh_vertices(instance: &mut InstanceState<'a>, metadata: &AnimationMetadata<'a>) {
        let num_parts = instance.resolved_layer.parts_vec.len();
        let cached_parts = metadata.cached_parts();

        // The CSR offsets and the static UV / index buffers depend only on the
        // animation's per-part Cells (fixed by the SS mesh spec). Take the
        // offsets and size the SoA position buffers once per layout;
        // `setup_animation` resets `mesh_layout` so a new animation sets up again.
        if instance.globals.mesh_layout != metadata.mesh_layout {
            Self::setup_static_mesh_buffers(instance, metadata);
        }

        let layer = &instance.resolved_layer;
        // SSAB bind coordinates are authored Y-up; flip when the instance
        // (hence the bone world matrices) runs Y-down.
        let flip_y = instance.coordinate_system == crate::eval::SsCoordinateSystem::YDown;
        // The Deform-only mesh rest comes from the cell's `table_coord_x` /
        // `table_coord_y`, which ssconverter copies verbatim from the `.ssce`
        // meshPointList — texture-pixel space, i.e. already Y-down. So it must be
        // flipped for a Y-up target, the OPPOSITE of the Y-up-native bind
        // coordinates above.
        let flip_rest = instance.coordinate_system == crate::eval::SsCoordinateSystem::YUp;
        let globals = &mut instance.globals;

        let total_vertices = globals.mesh_vertices_x.len();
        if total_vertices == 0 {
            return;
        }

        for i in 0..num_parts {
            let Some(cached) = cached_parts.get(i) else {
                continue;
            };
            let start = globals.mesh_vertex_offsets[i] as usize;
            let end = globals.mesh_vertex_offsets[i + 1] as usize;
            if end <= start {
                continue; // not a mesh part
            }

            // Skeletal mesh: a bone-weighted binding drives the vertices.
            let skeletal = cached.mesh_binding.as_ref().map(|b| !b.bone_indices.is_empty()).unwrap_or(false);
            if skeletal {
                let binding = cached.mesh_binding.as_ref().unwrap();
                // The SpriteStudio `.ssae` meshList carries the N rendered mesh vertices
                // followed by 4 trailing placeholder entries (the cell-quad
                // corners, bound to no bone), so a well-formed binding has
                // `cell_count + 4` vertices. `compute_skinned_mesh_vertices`
                // skins exactly `out_x.len()` (= cell count) vertices from the
                // front of the binding, so a longer binding is fine — the
                // trailing extras are simply never read. Only a binding with
                // *fewer* vertices than the cell is unusable.
                if binding.num_vertices() < end - start {
                    continue; // bind has fewer vertices than the cell — cannot skin
                }
                let deform = layer.mesh_deform.get(i).and_then(|d| d.as_deref()).filter(|d| !d.indices.is_empty()).map(|d| (d.indices.as_slice(), d.dx.as_slice(), d.dy.as_slice(), &globals.world_matrices[i]));
                let (out_x, out_y) = (&mut globals.mesh_vertices_x[start..end], &mut globals.mesh_vertices_y[start..end]);
                crate::eval::geometry::compute_skinned_mesh_vertices(&binding.influence_offsets, &binding.bone_indices, &binding.weights, &binding.coords_x, &binding.coords_y, &globals.world_matrices, deform, flip_y, out_x, out_y);
                continue;
            }

            // Deform-only mesh: no bones — rigidly transform the cell's rest
            // mesh by the part's own world matrix.
            let Some(mesh_cell) = cached.mesh_cell.as_ref() else {
                continue;
            };
            if mesh_cell.rest_x.len() != (end - start) || i >= globals.world_matrices.len() {
                continue;
            }
            // Rest transform (W · rest) — SIMD over 4 vertices at a time.
            {
                let (out_x, out_y) = (&mut globals.mesh_vertices_x[start..end], &mut globals.mesh_vertices_y[start..end]);
                crate::eval::geometry::compute_rigid_mesh_vertices(&mesh_cell.rest_x, &mesh_cell.rest_y, &globals.world_matrices[i], flip_rest, out_x, out_y);
            }
            // Apply deform deltas as-is — evaluate_part has already applied any
            // coordinate-system Y conversion. Sparse scatter, kept scalar.
            if let Some(deform) = layer.mesh_deform.get(i).and_then(|d| d.as_deref()) {
                let vert_count = end - start;
                let mat = &globals.world_matrices[i];
                let (m0, m1, m4, m5) = (mat.m[0], mat.m[1], mat.m[4], mat.m[5]);
                for k in 0..deform.indices.len() {
                    let v = deform.indices[k] as usize;
                    if v < vert_count {
                        globals.mesh_vertices_x[start + v] += m0 * deform.dx[k] + m4 * deform.dy[k];
                        globals.mesh_vertices_y[start + v] += m1 * deform.dx[k] + m5 * deform.dy[k];
                    }
                }
            }
        }
    }

    /// Points the instance at the animation's static mesh buffers: copies their
    /// vertex offsets, and sizes and zeroes the SoA position buffers. A
    /// statically malformed mesh part keeps the zeros written here, so the
    /// per-frame pass needs no clear.
    fn setup_static_mesh_buffers(instance: &mut InstanceState<'a>, metadata: &AnimationMetadata<'a>) {
        let globals = &mut instance.globals;
        let mesh = metadata.static_mesh();
        globals.mesh_vertex_offsets.clone_from(&mesh.vertex_offsets);
        let total_vertices = *mesh.vertex_offsets.last().unwrap_or(&0) as usize;
        globals.mesh_vertices_x.clear();
        globals.mesh_vertices_x.resize(total_vertices, 0.0);
        globals.mesh_vertices_y.clear();
        globals.mesh_vertices_y.resize(total_vertices, 0.0);
        globals.mesh_layout = metadata.mesh_layout;
    }

    fn clear_mesh_vertices(instance: &mut InstanceState<'a>, num_parts: usize) {
        let globals = &mut instance.globals;
        if globals.mesh_vertex_offsets.len() != num_parts + 1 {
            globals.mesh_vertex_offsets.resize(num_parts + 1, 0);
        }
        for slot in globals.mesh_vertex_offsets.iter_mut() {
            *slot = 0;
        }
        globals.mesh_vertices_x.clear();
        globals.mesh_vertices_y.clear();
        globals.mesh_layout = NO_MESH_LAYOUT;
    }

    #[allow(clippy::needless_range_loop)]
    fn compute_matrices(instance: &mut InstanceState<'a>, cached_parts: &[crate::eval::framedata::CachedPartData]) {
        let layer = &instance.resolved_layer;
        let globals = &mut instance.globals;
        for i in 0..layer.parts_vec.len() {
            let parent_idx = cached_parts[i].parent_index;
            let valid_parent_idx = (parent_idx >= 0 && (parent_idx as usize) < i).then_some(parent_idx as usize);

            let parent_world = valid_parent_idx.map(|p_idx| &globals.inheritance_matrices[p_idx]);
            let parent_size = valid_parent_idx.map(|p_idx| {
                let p_state = &layer.parts_vec[p_idx];
                (p_state.size_x(), p_state.size_y())
            });

            let part_state = &layer.parts_vec[i];
            let (inh_m, draw_m) = crate::eval::SsMatrix4x4::compute_world_matrix(part_state, parent_world, parent_size);
            globals.inheritance_matrices[i] = inh_m;
            globals.world_matrices[i] = draw_m;
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn compute_local_uvs(instance: &mut InstanceState<'a>, metadata: &AnimationMetadata<'a>) {
        let keys = &metadata.source.keys;
        let layer = &instance.resolved_layer;
        let override_cell_uv_meta = &instance.override_cell_uv_meta;
        let globals = &mut instance.globals;
        let num_parts = layer.parts_vec.len();
        if globals.local_uvs.len() != num_parts * 10 {
            globals.local_uvs.resize(num_parts * 10, 0.0);
        }
        for slot in globals.local_uvs.iter_mut() {
            *slot = 0.0;
        }

        for i in 0..num_parts {
            let state = &layer.parts_vec[i];
            if state.cell() < 0 {
                continue;
            }
            // A cell override recorded at the seam wins for UVs too, so the
            // sampled image (not just the quad size/texture) reflects the swap.
            let meta = match override_cell_uv_meta.get(i).and_then(|m| m.as_ref()) {
                Some(m) => m,
                None => match layer.cell_ids.get(i).copied().flatten() {
                    Some(cell) => &keys.cell(cell).meta,
                    None => continue,
                },
            };
            if meta.size_w == 0.0 && meta.size_h == 0.0 {
                continue;
            }

            let rect_right = meta.rect_left + meta.size_w;
            let rect_bottom = meta.rect_top + meta.size_h;

            let (tu, tv, rot_deg, su, sv, fh, fv, ifh, ifv) =
                (state.uv_translation_x() * meta.tex_w, state.uv_translation_y() * meta.tex_h, state.uv_rotation_z(), state.uv_scale_x(), state.uv_scale_y(), state.flip_h(), state.flip_v(), state.img_flip_h(), state.img_flip_v());

            let (us, vs) = crate::eval::geometry::compute_local_uvs((meta.rect_left, meta.rect_top, rect_right, rect_bottom), (tu, tv, rot_deg, su, sv), (fh, fv), (ifh, ifv), meta.rotated);

            let offset = i * 10;
            for k in 0..5 {
                globals.local_uvs[offset + k * 2] = us[k];
                globals.local_uvs[offset + k * 2 + 1] = vs[k];
            }
        }
    }

    #[allow(clippy::needless_range_loop)]
    fn compute_shape_vertices(instance: &mut InstanceState<'a>, cached_parts: &[crate::eval::framedata::CachedPartData]) {
        let coordinate_system = instance.coordinate_system;
        let layer = &instance.resolved_layer;
        let globals = &mut instance.globals;
        let num_parts = layer.parts_vec.len();
        let stride = crate::eval::framedata::SHAPE_VERTICES_STRIDE;
        if globals.shape_vertex_counts.len() != num_parts {
            globals.shape_vertex_counts.resize(num_parts, 0);
        }
        globals.shape_vertex_counts.fill(0);
        // `setup_animation` leaves the vertex buffers empty when no part is a shape.
        if globals.shape_vertices.len() != num_parts * stride {
            return;
        }
        globals.shape_vertices.fill(0.0);
        globals.shape_vertex_box_coords.fill(0.0);

        for i in 0..num_parts {
            let Some(shape_kind) = cached_parts[i].shape_kind else { continue };
            let state = &layer.parts_vec[i];
            let size_x = state.size_x();
            let size_y = state.size_y();
            if size_x <= 0.0 || size_y <= 0.0 {
                continue;
            }

            let (count, vx, vy) = crate::eval::geometry::compute_shape_vertices(shape_kind, (size_x, size_y));
            if count < 3 {
                continue;
            }

            let mut effective_pivot_y = state.pivot_y();
            if coordinate_system == crate::eval::SsCoordinateSystem::YDown {
                effective_pivot_y = -effective_pivot_y;
            }
            let tx = -state.pivot_x() * size_x;
            let ty = -effective_pivot_y * size_y;
            let hw = size_x * 0.5;
            let hh = size_y * 0.5;

            // Vertex deform: the 4 box-corner offsets ([LT, RT, LB, RB], in the
            // part's Y-up local space) are spread across every shape vertex by
            // bilinear interpolation over its box coordinates (fx, fy) — the same
            // weighting the renderer uses for per-corner PartColor. Offsets are
            // added in Y-up space, before the YDown negation, so the dy sign
            // matches the 4-corner deform path in `compute_local_vertices`.
            let deform = crate::eval::framedata::vertex_deform_at(&layer.raw_vertex_deform, i);
            // Skew shares the Vertex-deform slot; applied as a shear about the
            // pivot origin in Y-up, before the Y-down flip, like the deform.
            let skew = crate::eval::geometry::skew_local_tans(state.skew_x(), state.skew_y());

            let offset = i * stride;
            for v in 0..count {
                let fx = (vx[v] + hw) / size_x;
                let fy = (hh - vy[v]) / size_y;

                let (mut dvx, mut dvy) = (vx[v] + tx, vy[v] + ty);
                if let Some(d) = deform {
                    let (ddx, ddy) = crate::eval::geometry::bilinear_box_deform(&d.dx, &d.dy, fx, fy);
                    dvx += ddx;
                    dvy += ddy;
                }
                if let Some((tan_x, tan_y)) = skew {
                    let (nx, ny) = crate::eval::geometry::skew_point_yup(dvx, dvy, tan_x, tan_y);
                    dvx = nx;
                    dvy = ny;
                }

                let mut y = dvy;
                if coordinate_system == crate::eval::SsCoordinateSystem::YDown {
                    y = -y;
                }
                globals.shape_vertices[offset + 2 * v] = dvx;
                globals.shape_vertices[offset + 2 * v + 1] = y;

                globals.shape_vertex_box_coords[offset + 2 * v] = fx;
                globals.shape_vertex_box_coords[offset + 2 * v + 1] = fy;
            }
            globals.shape_vertex_counts[i] = count as i32;
        }
    }
}
