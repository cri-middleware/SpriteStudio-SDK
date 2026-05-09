use super::simulator::*;
use super::state::*;
use super::types::*;
use crate::core::math::*;
use alloc::vec;
use alloc::vec::Vec;
use ssab::ssab::ss::format as ssab_fbs;

pub struct EffectEmitterInstance {
    pub emitter_params: EmitterParameter,
    pub particle_params: ParticleParameter,
    pub emit_patterns: Vec<EmitPattern>,
    pub offset_patterns: Vec<i32>,
    pub seed_list: Vec<u32>,
    pub emitter_seed: i32,
    pub parent_index: i32,
    pub uid: i32,
    pub cell_name_hash: u32,
    pub cell_map_name_hash: u32,
    pub blend_type: i32,
}

fn color_range_min_diff(min_argb: u32, max_argb: u32) -> (Color, Color) {
    let chan = |c: u32, shift: u32| -> u8 { ((c >> shift) & 0xFF) as u8 };
    let split = |a: u8, b: u8| -> (u8, u8) {
        let lo = a.min(b);
        let hi = a.max(b);
        (lo, hi - lo)
    };
    let (a_lo, a_d) = split(chan(min_argb, 24), chan(max_argb, 24));
    let (r_lo, r_d) = split(chan(min_argb, 16), chan(max_argb, 16));
    let (g_lo, g_d) = split(chan(min_argb, 8), chan(max_argb, 8));
    let (b_lo, b_d) = split(chan(min_argb, 0), chan(max_argb, 0));
    (Color::new(a_lo, r_lo, g_lo, b_lo), Color::new(a_d, r_d, g_d, b_d))
}

impl EffectEmitterInstance {
    pub fn from_ssab(node: &ssab_fbs::EffectNode, my_seed: u32) -> Self {
        let mut emitter_params = EmitterParameter::default();
        let mut particle_params = ParticleParameter::default();

        if let Some(behaviors) = node.behavior() {
            for behavior_wrapper in behaviors {
                match behavior_wrapper.value_type() {
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementBasic => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_basic() {
                            emitter_params.life = p.lifetime();
                            emitter_params.interval = p.interval();
                            emitter_params.emitnum = p.attime_create();
                            emitter_params.emitmax = p.maximum_particle();
                            emitter_params.particle_life = p.lifespan_min_value();
                            emitter_params.particle_life2 = p.lifespan_max_value() - p.lifespan_min_value();

                            particle_params.speed = p.speed_min_value();
                            particle_params.speed2 = p.speed_max_value() - p.speed_min_value();
                            const DEG_TO_RAD: f32 = core::f32::consts::PI / 180.0;
                            particle_params.angle = (p.angle() + 90.0) * DEG_TO_RAD;
                            particle_params.angle_variance = p.angle_variance() * DEG_TO_RAD;
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementRndSeedChange => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_rnd_seed_change() {
                            particle_params.user_override_r_seed = true;
                            particle_params.override_r_seed = p.value();
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementDelay => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_delay() {
                            particle_params.delay = p.value() as f32;
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementGravity => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_gravity() {
                            particle_params.use_gravity = true;
                            if let Some(g) = p.value() {
                                particle_params.gravity = Vector2::new(g.v1(), g.v2());
                            }
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementPosition => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_position() {
                            particle_params.use_offset = true;
                            if let Some(rect) = p.value() {
                                particle_params.offset = Vector2::new(rect.x1(), rect.y1());
                                particle_params.offset2 = Vector2::new(rect.x2() - rect.x1(), rect.y2() - rect.y1());
                            }
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementRotation => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_rotation() {
                            particle_params.use_rotation = true;
                            if let Some(rot) = p.rotation_value() {
                                particle_params.rotation = rot.v1();
                                particle_params.rotation2 = rot.v2() - rot.v1();
                            }
                            if let Some(rot_add) = p.rotation_add_value() {
                                particle_params.rotation_add = rot_add.v1();
                                particle_params.rotation_add2 = rot_add.v2() - rot_add.v1();
                            }
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementRotationTrans => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_rotation_trans() {
                            particle_params.use_rotation_trans = true;
                            particle_params.rotation_factor = p.rotation_factor();
                            particle_params.end_life_time_per = p.end_life_time_per();
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementTransSpeed => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_trans_speed() {
                            particle_params.use_trans_speed = true;
                            if let Some(v) = p.value() {
                                particle_params.trans_speed = v.v1();
                                particle_params.trans_speed2 = v.v2() - v.v1();
                            }
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementTangentialAcceleration => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_tangential_acceleration() {
                            particle_params.use_tan_accel = true;
                            if let Some(v) = p.value() {
                                particle_params.tangential_accel = v.v1();
                                particle_params.tangential_accel2 = v.v2() - v.v1();
                            }
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementInitColor => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_init_color() {
                            particle_params.use_color = true;
                            let (mn, df) = color_range_min_diff(p.color_min_value(), p.color_max_value());
                            particle_params.init_color = mn;
                            particle_params.init_color2 = df;
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementTransColor => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_trans_color() {
                            particle_params.use_trans_color = true;
                            let (mn, df) = color_range_min_diff(p.color_min_value(), p.color_max_value());
                            particle_params.trans_color = mn;
                            particle_params.trans_color2 = df;
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementAlphaFade => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_alpha_fade() {
                            particle_params.use_alpha_fade = true;
                            if let Some(v) = p.value() {
                                particle_params.alpha_fade = v.v1();
                                particle_params.alpha_fade2 = v.v2();
                            }
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementSize => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_size() {
                            particle_params.use_init_scale = true;
                            if let Some(size) = p.size() {
                                particle_params.scale = Vector2::new(size.x1(), size.y1());
                                particle_params.scale_range = Vector2::new(size.x2() - size.x1(), size.y2() - size.y1());
                            }
                            if let Some(scale) = p.scale_factor_value() {
                                particle_params.scale_factor = scale.v1();
                                particle_params.scale_factor2 = scale.v2() - scale.v1();
                            }
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementTransSize => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_trans_size() {
                            particle_params.use_trans_scale = true;
                            if let Some(size) = p.size() {
                                particle_params.transscale = Vector2::new(size.x1(), size.y1());
                                particle_params.scale_range2 = Vector2::new(size.x2() - size.x1(), size.y2() - size.y1());
                            }
                            if let Some(scale) = p.scale_factor_value() {
                                particle_params.transscale_factor = scale.v1();
                                particle_params.transscale_factor2 = scale.v2() - scale.v1();
                            }
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticlePointGravity => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_point_gravity() {
                            particle_params.use_p_gravity = true;
                            if let Some(pos) = p.position() {
                                particle_params.gravity_pos = Vector2::new(pos.v1(), pos.v2());
                            }
                            particle_params.gravity_power = p.power();
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleTurnToDirectionEnabled => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_turn_to_direction_enabled() {
                            particle_params.use_turn_direc = true;
                            particle_params.direc_rot_add = p.value();
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleInfiniteEmitEnabled => {
                        emitter_params.is_infinite = true;
                    }
                    _ => {}
                }
            }
        }

        let emitter_seed = if particle_params.user_override_r_seed { particle_params.override_r_seed } else { my_seed as i32 };

        let (emit_patterns, offset_patterns, seed_list) = EffectSimulator::precalculate_emitter(&emitter_params, &particle_params, emitter_seed);

        Self {
            emitter_params,
            particle_params,
            emit_patterns,
            offset_patterns,
            seed_list,
            emitter_seed,
            parent_index: node.parent_index() as i32,
            uid: 0,
            cell_name_hash: node.cell_name_hash(),
            cell_map_name_hash: node.cell_map_name_hash(),
            blend_type: node.blend_type().0 as i32,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EmitterDrawCommand {
    pub cell_name_hash: u32,
    pub cell_map_name_hash: u32,
    pub blend_type: i32,
    pub particle_start: usize,
    pub particle_end: usize,
}

pub struct EffectInstance {
    pub emitters: Vec<EffectEmitterInstance>,
    pub state: RenderState,
    pub is_infinite: bool,
    pub layout_scale: Vector2,
    pub effect_time_length: i32,
    pub fps: i32,

    pub draw_commands: Vec<EmitterDrawCommand>,
    pub draw_particles: Vec<ParticleDrawState>,
}

impl EffectInstance {
    pub fn from_ssab(effect: &ssab_fbs::Effect, my_seed: u32) -> Option<Self> {
        let nodes = effect.effect_node()?;
        let node_count = nodes.len();

        let mut node_to_emitter: Vec<Option<usize>> = vec![None; node_count];
        let mut emitter_count = 0usize;
        for (i, node) in nodes.iter().enumerate() {
            if node.effect_node_type() == ssab_fbs::EffectNodeType::Emitter {
                node_to_emitter[i] = Some(emitter_count);
                emitter_count += 1;
            }
        }

        let resolve_emitter_parent = |start_pi: i32| -> i32 {
            let mut pi = start_pi;
            while pi >= 0 && (pi as usize) < node_count {
                if let Some(emitter_idx) = node_to_emitter[pi as usize] {
                    return emitter_idx as i32;
                }
                pi = nodes.get(pi as usize).parent_index() as i32;
            }
            -1
        };

        let mut emitters: Vec<EffectEmitterInstance> = Vec::with_capacity(emitter_count);
        let mut emitter_states: Vec<EmitterState> = Vec::with_capacity(emitter_count);

        let mut is_infinite = false;
        let mut effect_time_length = 0;

        let final_seed = if effect.is_lock_rand_seed() {
            (effect.lock_rand_seed() as u32).wrapping_add(1).wrapping_mul(SEED_MAGIC as u32)
        } else {
            my_seed.wrapping_mul(SEED_MAGIC as u32)
        };

        for (node_idx, node) in nodes.iter().enumerate() {
            if node.effect_node_type() != ssab_fbs::EffectNodeType::Emitter {
                continue;
            }

            let mut instance = EffectEmitterInstance::from_ssab(&node, final_seed);
            instance.uid = node_idx as i32;
            instance.parent_index = resolve_emitter_parent(node.parent_index() as i32);

            let emitter_state = EmitterState::new(instance.emitter_params.emitmax);

            let my_length = instance.emitter_params.life + instance.emitter_params.particle_life + instance.emitter_params.particle_life2;
            let pi = instance.parent_index;
            let global_time = if pi >= 0 && (pi as usize) < emitters.len() {
                let parent = &emitters[pi as usize];
                let parent_length = parent.emitter_params.life + parent.emitter_params.particle_life + parent.emitter_params.particle_life2;
                my_length + parent_length
            } else {
                my_length
            };

            if global_time > effect_time_length {
                effect_time_length = global_time;
            }
            if instance.emitter_params.is_infinite {
                is_infinite = true;
            }

            emitters.push(instance);
            emitter_states.push(emitter_state);
        }

        let layout_scale = if let Some(ls) = effect.layout_scale() { Vector2::new(ls.v1() / 100.0, ls.v2() / 100.0) } else { Vector2::new(1.0, 1.0) };

        Some(Self {
            emitters,
            state: RenderState { emitter_states, is_playing: true, ..Default::default() },
            is_infinite,
            layout_scale,
            effect_time_length,
            fps: if effect.fps() == 0 { 30 } else { effect.fps() as i32 },
            draw_commands: Vec::new(),
            draw_particles: Vec::new(),
        })
    }

    pub fn reset(&mut self) {
        self.state.reset();
        self.draw_commands.clear();
        self.draw_particles.clear();
    }

    pub fn set_loop(&mut self, loop_flag: bool) {
        self.state.is_loop = loop_flag;
    }

    pub fn update(&mut self, frame: f32) {
        if !self.state.is_playing || self.state.is_pausing {
            return;
        }

        self.state.current_frame = frame;
        self.state.target_frame = frame;

        if !self.is_infinite && self.state.is_loop && self.effect_time_length > 0 && self.state.current_frame > self.effect_time_length as f32 {
            self.state.target_frame = fmodf(self.state.current_frame, self.effect_time_length as f32);
            self.state.seed_offset = (self.state.current_frame / self.effect_time_length as f32) as i32;
        }
    }

    pub fn get_draw_commands(&mut self) -> (&[EmitterDrawCommand], &[ParticleDrawState]) {
        self.draw_commands.clear();
        self.draw_particles.clear();

        if self.state.current_frame < 0.0 {
            return (&self.draw_commands, &self.draw_particles);
        }

        let num_emitters = self.emitters.len();

        for i in 0..num_emitters {
            let pi = self.emitters[i].parent_index;
            let start_idx = self.draw_particles.len();

            if pi >= 0 && (pi as usize) < i {
                let parent_info: Vec<(i32, f32)> = {
                    let parent = &self.emitters[pi as usize];
                    let parent_state = &self.state.emitter_states[pi as usize];
                    let mut info = Vec::new();
                    for n in 0..parent.offset_patterns.len() {
                        if parent_state.particle_state.born[n] {
                            let p_stime = parent_state.particle_state.stime[n];
                            let p_ptime = self.state.target_frame - p_stime as f32;
                            info.push((n as i32, p_ptime));
                        }
                    }
                    info
                };
                for (n, p_ptime) in parent_info {
                    self.collect_emitter_particles(i, p_ptime, pi, n);
                }
            } else {
                self.collect_emitter_particles(i, self.state.target_frame, -1, 0);
            }

            let end_idx = self.draw_particles.len();
            if end_idx > start_idx {
                let e = &self.emitters[i];
                self.draw_commands.push(EmitterDrawCommand { cell_name_hash: e.cell_name_hash, cell_map_name_hash: e.cell_map_name_hash, blend_type: e.blend_type, particle_start: start_idx, particle_end: end_idx });
            }
        }

        (&self.draw_commands, &self.draw_particles)
    }

    fn collect_emitter_particles(&mut self, emitter_idx: usize, time: f32, parent_idx: i32, parent_particle_id: i32) {
        let e = &self.emitters[emitter_idx];

        let mut slide = 0;
        if parent_idx >= 0 {
            let parent_s = &self.state.emitter_states[parent_idx as usize];
            let pp_id = parent_particle_id as usize;
            slide = parent_s.particle_state.id[pp_id] + parent_s.particle_state.cycle[pp_id];
        }

        EffectSimulator::update_emitter(&mut self.state.emitter_states[emitter_idx], &e.emitter_params, &e.emit_patterns, &e.offset_patterns, time, slide);

        let mut active_indices = Vec::with_capacity(e.offset_patterns.len());
        for id in 0..e.offset_patterns.len() {
            if self.state.emitter_states[emitter_idx].particle_state.exist[id] {
                active_indices.push(id);
            }
        }

        // SIMD batch processing
        let mut chunks = active_indices.chunks_exact(4);
        for chunk in &mut chunks {
            let p_ids_with_cycle = [
                self.state.emitter_states[emitter_idx].particle_state.id[chunk[0]] + self.state.emitter_states[emitter_idx].particle_state.cycle[chunk[0]],
                self.state.emitter_states[emitter_idx].particle_state.id[chunk[1]] + self.state.emitter_states[emitter_idx].particle_state.cycle[chunk[1]],
                self.state.emitter_states[emitter_idx].particle_state.id[chunk[2]] + self.state.emitter_states[emitter_idx].particle_state.cycle[chunk[2]],
                self.state.emitter_states[emitter_idx].particle_state.id[chunk[3]] + self.state.emitter_states[emitter_idx].particle_state.cycle[chunk[3]],
            ];
            let stimes = [
                self.state.emitter_states[emitter_idx].particle_state.stime[chunk[0]],
                self.state.emitter_states[emitter_idx].particle_state.stime[chunk[1]],
                self.state.emitter_states[emitter_idx].particle_state.stime[chunk[2]],
                self.state.emitter_states[emitter_idx].particle_state.stime[chunk[3]],
            ];
            let endtimes = [
                self.state.emitter_states[emitter_idx].particle_state.endtime[chunk[0]],
                self.state.emitter_states[emitter_idx].particle_state.endtime[chunk[1]],
                self.state.emitter_states[emitter_idx].particle_state.endtime[chunk[2]],
                self.state.emitter_states[emitter_idx].particle_state.endtime[chunk[3]],
            ];

            let mut emitter_positions = [self.state.emitter_states[emitter_idx].position; 4];
            if parent_idx >= 0 {
                let parent_e = &self.emitters[parent_idx as usize];
                let parent_s = &self.state.emitter_states[parent_idx as usize];
                let pp_id = parent_particle_id as usize;

                let p_stime = parent_s.particle_state.stime[pp_id];
                let p_endtime = parent_s.particle_state.endtime[pp_id];
                let p_ptime = self.state.target_frame - p_stime as f32;

                let ds = EffectSimulator::calculate_state(
                    p_ptime + p_stime as f32,
                    p_stime,
                    p_endtime,
                    parent_s.particle_state.id[pp_id] + parent_s.particle_state.cycle[pp_id],
                    &parent_e.emitter_params,
                    &parent_e.particle_params,
                    parent_e.emitter_seed,
                    self.state.seed_offset,
                    &parent_e.seed_list,
                    parent_s.position,
                    parent_s.particle_state.pid[pp_id],
                );
                emitter_positions = [Vector2::new(ds.x, ds.y); 4];
            }

            let p_draw_batch = EffectSimulator::calculate_state_batch(
                time,
                stimes,
                endtimes,
                p_ids_with_cycle,
                &e.emitter_params,
                &e.particle_params,
                e.emitter_seed,
                self.state.seed_offset,
                &e.seed_list,
                emitter_positions,
                if parent_idx >= 0 { [slide; 4] } else { [0; 4] },
            );
            for mut ds in p_draw_batch {
                ds.x *= self.layout_scale.x;
                ds.y *= self.layout_scale.y;
                self.draw_particles.push(ds);
            }
        }

        for &id in chunks.remainder() {
            let p_id_with_cycle = self.state.emitter_states[emitter_idx].particle_state.id[id] + self.state.emitter_states[emitter_idx].particle_state.cycle[id];
            let stime = self.state.emitter_states[emitter_idx].particle_state.stime[id];
            let endtime = self.state.emitter_states[emitter_idx].particle_state.endtime[id];

            let mut emitter_position = self.state.emitter_states[emitter_idx].position;
            if parent_idx >= 0 {
                let parent_e = &self.emitters[parent_idx as usize];
                let parent_s = &self.state.emitter_states[parent_idx as usize];
                let pp_id = parent_particle_id as usize;

                let p_stime = parent_s.particle_state.stime[pp_id];
                let p_endtime = parent_s.particle_state.endtime[pp_id];
                let p_ptime = self.state.target_frame - p_stime as f32;

                let ds = EffectSimulator::calculate_state(
                    p_ptime + p_stime as f32,
                    p_stime,
                    p_endtime,
                    parent_s.particle_state.id[pp_id] + parent_s.particle_state.cycle[pp_id],
                    &parent_e.emitter_params,
                    &parent_e.particle_params,
                    parent_e.emitter_seed,
                    self.state.seed_offset,
                    &parent_e.seed_list,
                    parent_s.position,
                    parent_s.particle_state.pid[pp_id],
                );
                emitter_position = Vector2::new(ds.x, ds.y);
            }

            let mut ds = EffectSimulator::calculate_state(time, stime, endtime, p_id_with_cycle, &e.emitter_params, &e.particle_params, e.emitter_seed, self.state.seed_offset, &e.seed_list, emitter_position, if parent_idx >= 0 { slide } else { 0 });
            ds.x *= self.layout_scale.x;
            ds.y *= self.layout_scale.y;
            self.draw_particles.push(ds);
        }
    }
}
