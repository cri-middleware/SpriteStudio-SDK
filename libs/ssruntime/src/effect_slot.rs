//! [`EffectContext`] and [`EffectSlot`]: an effect part's particles, and the plan for drawing them.

use crate::error;
use crate::resource::AnimationResource;

pub struct EffectContext<'a> {
    pub eval: crate::eval::effect::instance::EffectEvaluationContext<'a>,
    pub instance: crate::eval::effect::instance::EffectInstanceState,
    seed: u32,
}

impl<'a> EffectContext<'a> {
    pub fn new(resource: &'a AnimationResource<'a>, effect_name_hash: u32, seed: u32) -> Result<Self, error::SSRuntimeError> {
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(resource.data) };
        let Some(effects) = ssab.effects() else {
            return Err(error::SSRuntimeError::AnimationNotFound);
        };
        let Some(effect) = crate::eval::ssab_lookup::find_effect(&effects, effect_name_hash) else {
            return Err(error::SSRuntimeError::AnimationNotFound);
        };
        // The effect was located by name_hash, but its data is malformed
        // (e.g. missing required `effect_node` table).
        let Some((eval, instance)) = crate::eval::effect::instance::EffectEvaluationContext::from_ssab(&effect, seed) else {
            return Err(error::SSRuntimeError::MalformedEffect);
        };
        Ok(Self { eval, instance, seed })
    }

    pub fn get_seed(&self) -> u32 {
        self.seed
    }

    pub fn reset(&mut self) {
        self.instance.reset();
    }

    pub fn set_loop(&mut self, loop_flag: bool) {
        self.instance.set_loop(loop_flag);
    }

    pub fn get_lifetime(&self) -> i32 {
        self.eval.effect_time_length
    }

    pub fn is_infinite(&self) -> bool {
        self.eval.is_infinite
    }

    pub fn update(&mut self, frame: f32) {
        self.instance.update(&self.eval, frame);
    }

    pub fn get_draw_commands(&mut self) -> (&[crate::eval::effect::instance::EmitterDrawCommand], &[crate::eval::effect::types::ParticleDrawState]) {
        self.instance.get_draw_commands(&self.eval)
    }
}

/// Resolved emitter render resources, cached on first encounter and reused
/// across frames. Identified by (cellmap_hash, cell_hash, blend_in) — the
/// effect simulator may reorder emitters when siblings go silent, so the
/// cache is keyed by content not slot index.
struct EmitterResourceEntry {
    cellmap_hash: u32,
    cell_hash: u32,
    blend_in: i32,
    resolved_ok: bool,
    blend_byte: u8,
    quad_left: f32,
    quad_top: f32,
    sw: f32,
    sh: f32,
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
}

// Lifted out of `build_draw_plan` so the row buffer can be a cached field.
#[derive(Default, Clone, Copy)]
struct ResolvedEmitter {
    ok: bool,
    blend_byte: u8,
    quad_left: f32,
    quad_top: f32,
    sw: f32,
    sh: f32,
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
    cellmap_hash: u32,
}

/// A particle quad's corners as `(xs, ys)`, in the plan's UV order: left-top,
/// right-top, left-bottom, right-bottom. `left` and `top` are in the cell's Y-up
/// space; `y_sign` mirrors the quad for a Y-down target, as it does the particle's
/// position and rotation, so the texture's top stays on top for either target.
pub(crate) fn particle_quad(left: f32, top: f32, width: f32, height: f32, y_sign: f32) -> ([f32; 4], [f32; 4]) {
    let (t, b) = (y_sign * top, y_sign * (top - height));
    ([left, left + width, left, left + width], [t, t, b, b])
}

/// High-level effect slot: wraps `EffectContext` with the per-slot lifecycle
/// state that previously lived in each Player's effect-slot table. Callers
/// resolve the active effect event externally (via `Context::get_active_effect_event`)
/// and feed it to `step()` along with frame timing — `step` owns transition
/// detection, accumulator, dead-effect skip, simulator update, and draw-plan
/// build in one call.
///
/// The cross-language contract is identical in Rust / C / Web: 1) construct
/// once per effect part, 2) call `step()` per frame with the resolved event
/// and timing, 3) read `draw_plan_buf` only when `visible == true` and only
/// before the next `step()`.
pub struct EffectSlot<'a> {
    ctx: EffectContext<'a>,
    // SSAB root, kept for cellmap lookup during draw-plan build. Borrows from
    // the same resource as `ctx`; lifetime tied to the surrounding `'a`.
    ssab: ssab::ssab::ss::format::SsAnimeBinary<'a>,
    // FlatBufferBuilder owned by the slot for `EffectDrawPlan` output.
    // Reset each `step`; buffer pointer stays valid until the next call.
    fb: flatbuffers::FlatBufferBuilder<'a>,
    emitter_resource_cache: alloc::vec::Vec<EmitterResourceEntry>,
    // Per-frame scratch for `build_draw_plan`: clear()+reserve()'d each tick
    // so capacities stabilise at the peak and steady-state allocation is zero.
    scratch_cmd_keys: alloc::vec::Vec<(u32, u32, i32, usize, usize)>,
    scratch_entries: alloc::vec::Vec<(usize, usize, usize)>,
    scratch_verts: alloc::vec::Vec<f32>,
    scratch_uvs: alloc::vec::Vec<f32>,
    scratch_colors: alloc::vec::Vec<f32>,
    scratch_indices: alloc::vec::Vec<i32>,
    scratch_resolved: alloc::vec::Vec<ResolvedEmitter>,
    scratch_command_meta: alloc::vec::Vec<(u32, u8, u32, u32)>,
    scratch_cmds: alloc::vec::Vec<crate::runtime::effect_draw_plan::ss::runtime::EffectDrawCommand>,
    // Cached at create time — never changes for a given effect resource.
    lifetime: i32,
    is_infinite: bool,
    // Transition detection: any change in (event_frame, is_synthetic, independent)
    // triggers reset/set_loop and re-seeds `accumulated_time`.
    last_event_frame: i32,
    last_is_synthetic: bool,
    last_independent: bool,
    // Speed of the last real EventEffect — reused while the implicit frame-0
    // key is being ignored (independent playback across a parent loop).
    last_speed: f32,
    // Independent-mode time accumulator (frame units, includes start_time bias).
    accumulated_time: f32,
}

impl<'a> EffectSlot<'a> {
    pub fn new(resource: &'a AnimationResource<'a>, effect_name_hash: u32, seed: u32) -> Result<Self, error::SSRuntimeError> {
        let ctx = EffectContext::new(resource, effect_name_hash, seed)?;
        let ssab = unsafe { ssab::ssab::ss::format::root_as_ss_anime_binary_unchecked(resource.data) };
        let lifetime = ctx.get_lifetime();
        let is_infinite = ctx.is_infinite();
        Ok(Self {
            ctx,
            ssab,
            fb: flatbuffers::FlatBufferBuilder::new(),
            emitter_resource_cache: alloc::vec::Vec::new(),
            scratch_cmd_keys: alloc::vec::Vec::new(),
            scratch_entries: alloc::vec::Vec::new(),
            scratch_verts: alloc::vec::Vec::new(),
            scratch_uvs: alloc::vec::Vec::new(),
            scratch_colors: alloc::vec::Vec::new(),
            scratch_indices: alloc::vec::Vec::new(),
            scratch_resolved: alloc::vec::Vec::new(),
            scratch_command_meta: alloc::vec::Vec::new(),
            scratch_cmds: alloc::vec::Vec::new(),
            lifetime,
            is_infinite,
            last_event_frame: -1,
            last_is_synthetic: false,
            last_independent: false,
            last_speed: 1.0,
            accumulated_time: 0.0,
        })
    }

    pub fn get_seed(&self) -> u32 {
        self.ctx.get_seed()
    }

    /// Whether the slot is currently running independently of the parent's
    /// frame. Reflects the most recently seen `EventEffect.independent`;
    /// used by callers to decide whether to schedule continuous tick updates.
    pub fn is_independent(&self) -> bool {
        self.last_independent
    }

    /// Reset the slot's playback / "last started key" memory to its
    /// just-created state. The host calls this on an explicit head-out
    /// (`play` / replay) so a re-played animation re-fires the effect from
    /// scratch instead of debouncing the first key as the same one it last
    /// started. The resource binding, emitter caches and simulator
    /// are left intact — forcing `last_event_frame = -1` makes the next `step`
    /// transition, which re-seeds the simulator via the existing `ctx.reset()`
    /// path.
    pub fn reset(&mut self) {
        self.last_event_frame = -1;
        self.last_is_synthetic = false;
        self.last_independent = false;
        self.last_speed = 1.0;
        self.accumulated_time = 0.0;
    }

    /// Resolve `(cellmap_hash, cell_hash, blend_in)` to render geometry/UV,
    /// caching the result in `emitter_resource_cache`. Linear search — typical
    /// effects have 1-3 unique tuples and the cache survives across frames.
    /// Free-standing form so callers can split-borrow `cache` and `ssab`
    /// independently of other `self` fields (the simulator output borrow lives
    /// on `self.ctx.instance` simultaneously inside `build_draw_plan`).
    fn ensure_emitter_resource_in(cache: &mut alloc::vec::Vec<EmitterResourceEntry>, ssab: &ssab::ssab::ss::format::SsAnimeBinary<'a>, cellmap_hash: u32, cell_hash: u32, blend_in: i32) -> usize {
        if let Some(idx) = cache.iter().position(|c| c.cellmap_hash == cellmap_hash && c.cell_hash == cell_hash && c.blend_in == blend_in) {
            return idx;
        }
        let mut entry = EmitterResourceEntry { cellmap_hash, cell_hash, blend_in, resolved_ok: false, blend_byte: 0, quad_left: 0.0, quad_top: 0.0, sw: 0.0, sh: 0.0, u0: 0.0, v0: 0.0, u1: 0.0, v1: 0.0 };

        if cellmap_hash != 0
            && let Some(cellmaps) = ssab.cellmaps()
            && let Some(cellmap) = crate::eval::ssab_lookup::find_cellmap(&cellmaps, cellmap_hash)
            && let Some(cells) = cellmap.cells()
            && let Some(cell) = crate::eval::ssab_lookup::find_cell(&cells, cell_hash)
        {
            // SS6-style blend mapping: 0=Mix, 1=Add. Anything else falls back to Mix.
            entry.blend_byte = if blend_in == 1 { 1 } else { 0 };

            let region = cell.region();
            let (src_x, src_y, sw, sh) = (region.x(), region.y(), region.w(), region.h());
            // The box a Normal part draws this cell in, in the cell's Y-up space.
            let p = cell.pivot();
            let (vx, vy) = crate::eval::geometry::compute_local_vertices((sw, sh), (p.v1(), p.v2()), (0.0, 0.0), (false, false), None);
            entry.sw = sw;
            entry.sh = sh;
            entry.quad_left = vx[0];
            entry.quad_top = vy[0];

            // UV uses cellmap.size_original — the texture dimensions at SSAB build time.
            // Atlas / scaled-texture pipelines must remap UVs engine-side.
            let s = cellmap.size_original();
            let (tex_w, tex_h) = (s.v1(), s.v2());
            let tex_w = if tex_w > 0.0 { tex_w } else { 1.0 };
            let tex_h = if tex_h > 0.0 { tex_h } else { 1.0 };
            entry.u0 = src_x / tex_w;
            entry.v0 = src_y / tex_h;
            entry.u1 = (src_x + sw) / tex_w;
            entry.v1 = (src_y + sh) / tex_h;
            entry.resolved_ok = true;
        }

        let idx = cache.len();
        cache.push(entry);
        idx
    }

    /// Advance the slot one frame and, when `draw` is set, build its draw plan.
    ///
    /// `draw` is whether the Player draws the part this frame, that is whether the
    /// part is in the frame's `draw_order`. A hidden part is not, and its slot
    /// still has to step every frame so an independently playing effect keeps
    /// time; with `draw == false` the step moves only that (the clock and the
    /// transition state) and returns not visible. The particles are a function of
    /// the effect's time alone, so the next drawn step builds exactly the plan it
    /// would have built had every step drawn.
    pub fn step(&mut self, ev: crate::types::ss_effect_event_info, parent_frame: f32, delta_seconds: f32, fps: f32, parent_looped: bool, y_flip: bool, vert_stride: u8, draw: bool) -> crate::types::ss_effect_step_result {
        // While the slot is playing independently, the SpriteStudio implicit frame-0
        // playback key (synthetic default) is ignored — the independent effect
        // keeps running across the parent's loop instead of being reset to it.
        // Only a real authored EventEffect re-transitions the slot.
        let ignore_implicit = self.last_independent && ev.is_synthetic_default;

        // Parent loop edge re-arms transition detection so a *synced* effect
        // re-fires its authored EventEffect on the next playthrough (without this
        // reset the equality check below would treat the loop edge as a no-op,
        // freezing the effect at the end of the previous cycle).
        //
        // Independent playback follows the opposite rule: the slot
        // remembers "the last key that started playback" and a re-crossing of the
        // *same* key must NOT restart it — finite single-key effects play once and
        // stay dead across loops ("fire-once"), only a *different* key re-fires.
        // So when the last started key was independent, keep the memory intact:
        // don't re-arm. `ignore_implicit` already skips the synthetic-key case;
        // `!self.last_independent` additionally protects the real-key case (e.g.
        // a partial-section loop whose first active key is the play key).
        if parent_looped && !ignore_implicit && !self.last_independent {
            self.last_event_frame = -1;
            self.last_is_synthetic = false;
        }

        // When ignoring the implicit key, keep stepping with the last real
        // event's identity / mode / speed rather than the synthetic default's.
        let event_frame = if ignore_implicit { self.last_event_frame } else { ev.event_frame };
        let start_time = ev.start_time;
        let speed = if ignore_implicit { self.last_speed } else { ev.speed };
        let independent = if ignore_implicit { true } else { ev.independent };
        let is_synthetic = ev.is_synthetic_default;

        let transitioned = !ignore_implicit && (self.last_event_frame != event_frame || self.last_is_synthetic != is_synthetic || self.last_independent != independent);

        if transitioned {
            self.ctx.reset();
            // `independent` controls the *clock* (own delta-time vs parent-synced
            // frame), not whether the effect loops. Looping is a property of the
            // effect resource itself: only genuinely infinite effects loop; finite
            // ones play once and are culled by the dead-effect early-return below
            // (which now applies regardless of `independent`). The earlier SS6
            // carry-over `set_loop(independent)` wrongly looped finite independent
            // effects forever, so they never died at their lifetime.
            self.ctx.set_loop(self.is_infinite);
            self.last_event_frame = event_frame;
            self.last_is_synthetic = is_synthetic;
            self.last_independent = independent;
            self.last_speed = speed;
            self.accumulated_time = start_time as f32;
        }

        let relative_frame: f32 = if independent {
            // Skip the first tick after transition — `play()` snaps to start_time
            // and stepping on the same tick would visibly overshoot frame 1.
            if !transitioned && delta_seconds > 0.0 {
                let delta_frames = delta_seconds * fps;
                self.accumulated_time += delta_frames * speed;
            }
            self.accumulated_time
        } else {
            let elapsed = parent_frame - event_frame as f32;
            if elapsed < 0.0 {
                return crate::types::ss_effect_step_result::not_visible();
            }
            elapsed * speed + start_time as f32
        };

        if relative_frame < 0.0 {
            return crate::types::ss_effect_step_result::not_visible();
        }

        // Dead-effect skip: a finite effect past its last particle's death is
        // gone — independent or not. Independent only changes how `relative_frame`
        // is advanced (own accumulator vs parent-synced); both clocks must still
        // honour the effect's finite lifetime.
        if !self.is_infinite && self.lifetime > 0 && relative_frame > self.lifetime as f32 {
            return crate::types::ss_effect_step_result::not_visible();
        }

        if !draw {
            return crate::types::ss_effect_step_result::not_visible();
        }

        self.ctx.update(relative_frame);
        let buf = self.build_draw_plan(y_flip, vert_stride);
        if buf.is_empty() { crate::types::ss_effect_step_result::not_visible() } else { crate::types::ss_effect_step_result { visible: true, draw_plan_buf: buf.as_ptr(), draw_plan_len: buf.len() } }
    }

    /// Build the `EffectDrawPlan` FlatBuffer from the simulator's current
    /// draw_commands + draw_particles. Resolves per-emitter resources via
    /// `ensure_emitter_resource_in` (cached) and writes verts/uvs/colors/indices
    /// concatenated for all emitters in one pass. Returns the FlatBuffer slice;
    /// valid until the next call on this slot.
    fn build_draw_plan(&mut self, y_flip: bool, vert_stride: u8) -> &[u8] {
        // π/180.
        const DEG_TO_RAD: f32 = 0.017_453_292;
        let stride = if vert_stride == 3 { 3usize } else { 2usize };
        let y_sign: f32 = if y_flip { -1.0 } else { 1.0 };

        // Narrow the borrow on `self.ctx.get_draw_commands` so other self
        // fields stay touchable afterwards.
        self.scratch_cmd_keys.clear();
        {
            let (commands, _particles) = self.ctx.get_draw_commands();
            self.scratch_cmd_keys.reserve(commands.len());
            for c in commands.iter() {
                self.scratch_cmd_keys.push((c.cell_map_name_hash, c.cell_name_hash, c.blend_type, c.particle_start, c.particle_end));
            }
        }

        // Resolve each command's resources, split-borrowing disjoint self fields.
        self.scratch_entries.clear();
        self.scratch_entries.reserve(self.scratch_cmd_keys.len());
        {
            let cache = &mut self.emitter_resource_cache;
            let ssab = &self.ssab;
            for &(cmh, ch, bi, ps, pe) in self.scratch_cmd_keys.iter() {
                self.scratch_entries.push((Self::ensure_emitter_resource_in(cache, ssab, cmh, ch, bi), ps, pe));
            }
        }

        let total_quads: usize = self.scratch_entries.iter().map(|(_, s, e)| e - s).sum();
        if total_quads == 0 {
            return &[];
        }

        self.scratch_verts.clear();
        self.scratch_uvs.clear();
        self.scratch_colors.clear();
        self.scratch_indices.clear();
        self.scratch_verts.reserve(total_quads * 4 * stride);
        self.scratch_uvs.reserve(total_quads * 4 * 2);
        self.scratch_colors.reserve(total_quads * 4 * 4);
        self.scratch_indices.reserve(total_quads * 6);

        // Snapshot resolved entries so self.emitter_resource_cache is free
        // to be re-borrowed when we later write into self.fb.
        self.scratch_resolved.clear();
        self.scratch_resolved.reserve(self.scratch_entries.len());
        for (i, _, _) in self.scratch_entries.iter() {
            let e = &self.emitter_resource_cache[*i];
            self.scratch_resolved.push(ResolvedEmitter { ok: e.resolved_ok, blend_byte: e.blend_byte, quad_left: e.quad_left, quad_top: e.quad_top, sw: e.sw, sh: e.sh, u0: e.u0, v0: e.v0, u1: e.u1, v1: e.v1, cellmap_hash: e.cellmap_hash });
        }

        self.scratch_command_meta.clear();
        self.scratch_command_meta.reserve(self.scratch_entries.len());

        let particles = &self.ctx.instance.draw_particles;

        for (cmd_idx, (_, p_start, p_end)) in self.scratch_entries.iter().enumerate() {
            let r = &self.scratch_resolved[cmd_idx];
            let particle_offset = (self.scratch_verts.len() / (4 * stride)) as u32;
            let particle_count = (p_end - p_start) as u32;

            if !r.ok {
                self.scratch_command_meta.push((r.cellmap_hash, r.blend_byte, 0, particle_offset));
                continue;
            }

            let (lx, ly) = particle_quad(r.quad_left, r.quad_top, r.sw, r.sh, y_sign);
            let uvs_corner = [r.u0, r.v0, r.u1, r.v0, r.u0, r.v1, r.u1, r.v1];

            for (i, p) in particles[*p_start..*p_end].iter().enumerate() {
                let angle = y_sign * (p.rotation * DEG_TO_RAD + p.direction);
                let c = libm::cosf(angle);
                let s = libm::sinf(angle);
                let py = y_sign * p.y;

                for v in 0..4 {
                    let xx = lx[v] * p.scale_x;
                    let yy = ly[v] * p.scale_y;
                    let wx = c * xx - s * yy + p.x;
                    let wy = s * xx + c * yy + py;
                    self.scratch_verts.push(wx);
                    self.scratch_verts.push(wy);
                    if stride == 3 {
                        self.scratch_verts.push(0.0);
                    }
                }
                self.scratch_uvs.extend_from_slice(&uvs_corner);
                let r_f = p.color.r as f32 / 255.0;
                let g_f = p.color.g as f32 / 255.0;
                let b_f = p.color.b as f32 / 255.0;
                let a_f = p.color.a as f32 / 255.0;
                for _ in 0..4 {
                    self.scratch_colors.extend_from_slice(&[r_f, g_f, b_f, a_f]);
                }
                // Quad-local indices: command consumers slice [off*6 .. (off+qc)*6]
                // and pair it with the verts slice [off*4*stride .. (off+qc)*4*stride],
                // so values must be 0..qc*4 within the command, not global.
                let v = (i * 4) as i32;
                self.scratch_indices.extend_from_slice(&[v, v + 1, v + 2, v + 2, v + 1, v + 3]);
            }

            self.scratch_command_meta.push((r.cellmap_hash, r.blend_byte, particle_count, particle_offset));
        }

        // Commands are inlined fixed-size structs; collect as POD then create
        // the vector in one shot (no per-element vtable/offset overhead).
        self.fb.reset();
        self.scratch_cmds.clear();
        self.scratch_cmds.reserve(self.scratch_command_meta.len());
        for &(hash, blend, qc, off) in self.scratch_command_meta.iter() {
            self.scratch_cmds.push(crate::runtime::effect_draw_plan::ss::runtime::EffectDrawCommand::new(hash, qc, off, blend));
        }
        let commands_off = self.fb.create_vector(&self.scratch_cmds);
        let verts_off = self.fb.create_vector(&self.scratch_verts);
        let uvs_off = self.fb.create_vector(&self.scratch_uvs);
        let colors_off = self.fb.create_vector(&self.scratch_colors);
        let indices_off = self.fb.create_vector(&self.scratch_indices);

        let mut plan_b = crate::runtime::effect_draw_plan::ss::runtime::EffectDrawPlanBuilder::new(&mut self.fb);
        plan_b.add_commands(commands_off);
        plan_b.add_vert_stride(stride as u8);
        plan_b.add_verts(verts_off);
        plan_b.add_uvs(uvs_off);
        plan_b.add_colors(colors_off);
        plan_b.add_indices(indices_off);
        let root = plan_b.finish();
        self.fb.finish_minimal(root);
        self.fb.finished_data()
    }
}
