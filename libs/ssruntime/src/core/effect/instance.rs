use super::simulator::*;
use super::state::*;
use super::types::*;
use crate::core::math::*;
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
    pub cell_index: i16,
    pub blend_type: i32,
}

fn convert_color(c: u32) -> Color {
    Color::new(((c >> 24) & 0xFF) as u8, ((c >> 16) & 0xFF) as u8, ((c >> 8) & 0xFF) as u8, (c & 0xFF) as u8)
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
                            emitter_params.particle_life2 = p.lifespan_max_value();

                            particle_params.speed = p.speed_min_value();
                            particle_params.speed2 = p.speed_max_value();
                            particle_params.angle = p.angle();
                            particle_params.angle_variance = p.angle_variance();
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
                                particle_params.offset2 = Vector2::new(rect.x2(), rect.y2());
                            }
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementRotation => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_rotation() {
                            particle_params.use_rotation = true;
                            if let Some(rot) = p.rotation_value() {
                                particle_params.rotation = rot.v1();
                                particle_params.rotation2 = rot.v2();
                            }
                            if let Some(rot_add) = p.rotation_add_value() {
                                particle_params.rotation_add = rot_add.v1();
                                particle_params.rotation_add2 = rot_add.v2();
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
                                particle_params.trans_speed2 = v.v2();
                            }
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementTangentialAcceleration => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_tangential_acceleration() {
                            particle_params.use_tan_accel = true;
                            if let Some(v) = p.value() {
                                particle_params.tangential_accel = v.v1();
                                particle_params.tangential_accel2 = v.v2();
                            }
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementInitColor => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_init_color() {
                            particle_params.use_color = true;
                            particle_params.init_color = convert_color(p.color_min_value());
                            particle_params.init_color2 = convert_color(p.color_max_value());
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementTransColor => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_trans_color() {
                            particle_params.use_trans_color = true;
                            particle_params.trans_color = convert_color(p.color_min_value());
                            particle_params.trans_color2 = convert_color(p.color_max_value());
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
                                particle_params.scale_range = Vector2::new(size.x2(), size.y2());
                            }
                            if let Some(scale) = p.scale_factor_value() {
                                particle_params.scale_factor = scale.v1();
                                particle_params.scale_factor2 = scale.v2();
                            }
                        }
                    }
                    ssab_fbs::EffectNodeBehavior::EffectParticleElementTransSize => {
                        if let Some(p) = behavior_wrapper.value_as_effect_particle_element_trans_size() {
                            particle_params.use_trans_scale = true;
                            if let Some(size) = p.size() {
                                particle_params.transscale = Vector2::new(size.x1(), size.y1());
                                particle_params.scale_range2 = Vector2::new(size.x2(), size.y2());
                            }
                            if let Some(scale) = p.scale_factor_value() {
                                particle_params.transscale_factor = scale.v1();
                                particle_params.transscale_factor2 = scale.v2();
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

        Self { emitter_params, particle_params, emit_patterns, offset_patterns, seed_list, emitter_seed, parent_index: node.parent_index() as i32, uid: 0, cell_index: node.cell_index(), blend_type: node.blend_type().0 as i32 }
    }
}

#[derive(Clone, Debug)]
pub struct EmitterDrawCommand {
    pub cell_index: i16,
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
        let mut emitters: Vec<EffectEmitterInstance> = Vec::with_capacity(nodes.len());
        let mut emitter_states: Vec<EmitterState> = Vec::with_capacity(nodes.len());

        let mut is_infinite = false;
        let mut effect_time_length = 0;

        let final_seed = if effect.is_lock_rand_seed() { effect.lock_rand_seed() as u32 } else { my_seed };

        for (i, node) in nodes.iter().enumerate() {
            let mut instance = EffectEmitterInstance::from_ssab(&node, final_seed);
            instance.uid = i as i32;

            let emitter_state = EmitterState::new(instance.emitter_params.emitmax);

            let my_length = instance.emitter_params.life + instance.emitter_params.particle_life + instance.emitter_params.particle_life2;
            let pi = instance.parent_index;
            let global_time = if pi >= 0 && (pi as usize) < i {
                let parent_length = emitters[pi as usize].emitter_params.life + emitters[pi as usize].emitter_params.particle_life + emitters[pi as usize].emitter_params.particle_life2;
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
                self.draw_commands.push(EmitterDrawCommand { cell_index: e.cell_index, blend_type: e.blend_type, particle_start: start_idx, particle_end: end_idx });
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

                for (i, &stime) in stimes.iter().enumerate() {
                    let parent_ds = EffectSimulator::calculate_state(
                        stime as f32 + parent_s.particle_state.stime[pp_id] as f32,
                        parent_s.particle_state.stime[pp_id],
                        parent_s.particle_state.endtime[pp_id],
                        slide,
                        &parent_e.emitter_params,
                        &parent_e.particle_params,
                        parent_e.emitter_seed,
                        self.state.seed_offset,
                        &parent_e.seed_list,
                        parent_s.position,
                        parent_s.particle_state.pid[pp_id],
                    );
                    emitter_positions[i] = Vector2::new(parent_ds.x, parent_ds.y);
                }
            }

            let mut ds_batch = EffectSimulator::calculate_state_batch(
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

            for ds in &mut ds_batch {
                ds.x *= self.layout_scale.x;
                ds.y *= self.layout_scale.y;
                self.draw_particles.push(*ds);
            }
        }

        // Remainder processing
        for &id in chunks.remainder() {
            let mut ep_pos = self.state.emitter_states[emitter_idx].position;
            if parent_idx >= 0 {
                let parent_e = &self.emitters[parent_idx as usize];
                let parent_s = &self.state.emitter_states[parent_idx as usize];
                let pp_id = parent_particle_id as usize;
                let stime = self.state.emitter_states[emitter_idx].particle_state.stime[id];
                let parent_ds = EffectSimulator::calculate_state(
                    stime as f32 + parent_s.particle_state.stime[pp_id] as f32,
                    parent_s.particle_state.stime[pp_id],
                    parent_s.particle_state.endtime[pp_id],
                    parent_s.particle_state.id[pp_id] + parent_s.particle_state.cycle[pp_id],
                    &parent_e.emitter_params,
                    &parent_e.particle_params,
                    parent_e.emitter_seed,
                    self.state.seed_offset,
                    &parent_e.seed_list,
                    parent_s.position,
                    parent_s.particle_state.pid[pp_id],
                );
                ep_pos = Vector2::new(parent_ds.x, parent_ds.y);
            }

            let particle_id_with_cycle = self.state.emitter_states[emitter_idx].particle_state.id[id] + self.state.emitter_states[emitter_idx].particle_state.cycle[id];
            let mut ds = EffectSimulator::calculate_state(
                time,
                self.state.emitter_states[emitter_idx].particle_state.stime[id],
                self.state.emitter_states[emitter_idx].particle_state.endtime[id],
                particle_id_with_cycle,
                &e.emitter_params,
                &e.particle_params,
                e.emitter_seed,
                self.state.seed_offset,
                &e.seed_list,
                ep_pos,
                if parent_idx >= 0 { slide } else { 0 },
            );

            ds.x *= self.layout_scale.x;
            ds.y *= self.layout_scale.y;

            self.draw_particles.push(ds);
        }
    }
}
