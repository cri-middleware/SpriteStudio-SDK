use super::simulator::*;
use super::state::*;
use super::types::*;
use crate::runtime::effect::ss::runtime as fbs;
use alloc::vec::Vec;

pub struct EffectEmitterInstance {
    pub emitter_params: EmitterParameter,
    pub particle_params: ParticleParameter,
    pub emit_patterns: Vec<EmitPattern>,
    pub offset_patterns: Vec<i32>,
    pub seed_list: Vec<u32>,
    pub emitter_seed: i32,
    pub parent_index: i32,
    pub uid: i32,
    pub cell_map_name_hash: u32,
    pub cell_name_hash: u32,
    pub blend_type: i32,
}

impl EffectEmitterInstance {
    pub fn from_fbs(node: &fbs::EffectNode, my_seed: u32) -> Self {
        let fbs_emitter = node.emitter();
        let fbs_particle = node.particle();

        let emitter_params = EmitterParameter {
            life: fbs_emitter.life(),
            interval: fbs_emitter.interval(),
            emitnum: fbs_emitter.emitnum(),
            emitmax: fbs_emitter.emitmax(),
            particle_life: fbs_emitter.particle_life(),
            particle_life2: fbs_emitter.particle_life2(),
            is_infinite: fbs_emitter.is_infinite(),
            loop_start: fbs_emitter.loop_start(),
            loop_end: fbs_emitter.loop_end(),
            loop_len: fbs_emitter.loop_len(),
            loop_gen: fbs_emitter.loop_gen(),
        };

        let particle_params = ParticleParameter {
            scale: Vector2::new(fbs_particle.scale_x(), fbs_particle.scale_y()),
            startcolor: Color::new(fbs_particle.start_color().a(), fbs_particle.start_color().r(), fbs_particle.start_color().g(), fbs_particle.start_color().b()),
            endcolor: Color::new(fbs_particle.end_color().a(), fbs_particle.end_color().r(), fbs_particle.end_color().g(), fbs_particle.end_color().b()),
            speed: fbs_particle.speed(),
            speed2: fbs_particle.speed2(),
            angle: fbs_particle.angle(),
            angle_variance: fbs_particle.angle_variance(),
            use_gravity: fbs_particle.use_gravity(),
            gravity: Vector2::new(fbs_particle.gravity_x(), fbs_particle.gravity_y()),
            use_offset: fbs_particle.use_offset(),
            offset: Vector2::new(fbs_particle.offset_x(), fbs_particle.offset_y()),
            offset2: Vector2::new(fbs_particle.offset2_x(), fbs_particle.offset2_y()),
            use_rotation: fbs_particle.use_rotation(),
            rotation: fbs_particle.rotation(),
            rotation2: fbs_particle.rotation2(),
            rotation_add: fbs_particle.rotation_add(),
            rotation_add2: fbs_particle.rotation_add2(),
            use_rotation_trans: fbs_particle.use_rotation_trans(),
            rotation_factor: fbs_particle.rotation_factor(),
            end_life_time_per: fbs_particle.end_life_time_per(),
            use_tan_accel: fbs_particle.use_tan_accel(),
            tangential_accel: fbs_particle.tangential_accel(),
            tangential_accel2: fbs_particle.tangential_accel2(),
            use_color: fbs_particle.use_color(),
            init_color: Color::new(fbs_particle.init_color().a(), fbs_particle.init_color().r(), fbs_particle.init_color().g(), fbs_particle.init_color().b()),
            init_color2: Color::new(fbs_particle.init_color2().a(), fbs_particle.init_color2().r(), fbs_particle.init_color2().g(), fbs_particle.init_color2().b()),
            use_trans_color: fbs_particle.use_trans_color(),
            trans_color: Color::new(fbs_particle.trans_color().a(), fbs_particle.trans_color().r(), fbs_particle.trans_color().g(), fbs_particle.trans_color().b()),
            trans_color2: Color::new(fbs_particle.trans_color2().a(), fbs_particle.trans_color2().r(), fbs_particle.trans_color2().g(), fbs_particle.trans_color2().b()),
            use_init_scale: fbs_particle.use_init_scale(),
            scale_range: Vector2::new(fbs_particle.scale_range_x(), fbs_particle.scale_range_y()),
            scale_factor: fbs_particle.scale_factor(),
            scale_factor2: fbs_particle.scale_factor2(),
            use_trans_scale: fbs_particle.use_trans_scale(),
            transscale: Vector2::new(fbs_particle.trans_scale_x(), fbs_particle.trans_scale_y()),
            scale_range2: Vector2::new(fbs_particle.trans_scale_range_x(), fbs_particle.trans_scale_range_y()),
            transscale_factor: fbs_particle.trans_scale_factor(),
            transscale_factor2: fbs_particle.trans_scale_factor2(),
            delay: fbs_particle.delay(),
            use_p_gravity: fbs_particle.use_p_gravity(),
            gravity_pos: Vector2::new(fbs_particle.gravity_pos_x(), fbs_particle.gravity_pos_y()),
            gravity_power: fbs_particle.gravity_power(),
            use_alpha_fade: fbs_particle.use_alpha_fade(),
            alpha_fade: fbs_particle.alpha_fade(),
            alpha_fade2: fbs_particle.alpha_fade2(),
            use_trans_speed: fbs_particle.use_trans_speed(),
            trans_speed: fbs_particle.trans_speed(),
            trans_speed2: fbs_particle.trans_speed2(),
            use_turn_direc: fbs_particle.use_turn_direc(),
            direc_rot_add: fbs_particle.direc_rot_add(),
            user_override_r_seed: fbs_particle.user_override_r_seed(),
            override_r_seed: fbs_particle.override_r_seed(),
        };

        let emitter_seed = if particle_params.user_override_r_seed { particle_params.override_r_seed } else { my_seed as i32 };

        let (emit_patterns, offset_patterns, seed_list) = EffectSimulator::precalculate_emitter(&emitter_params, &particle_params, emitter_seed);

        Self {
            emitter_params,
            particle_params,
            emit_patterns,
            offset_patterns,
            seed_list,
            emitter_seed,
            parent_index: node.parent_index(),
            uid: 0,
            cell_map_name_hash: node.cell_map_name_hash(),
            cell_name_hash: node.cell_name_hash(),
            blend_type: node.blend_type(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EmitterDrawCommand {
    pub cell_map_name_hash: u32,
    pub cell_name_hash: u32,
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
    pub fn from_fbs(model: &fbs::EffectModel, my_seed: u32) -> Self {
        let nodes = model.nodes().unwrap();
        let mut emitters: Vec<EffectEmitterInstance> = Vec::with_capacity(nodes.len());
        let mut emitter_states: Vec<EmitterState> = Vec::with_capacity(nodes.len());

        let mut is_infinite = false;
        let mut effect_time_length = 0;

        for (i, node) in nodes.iter().enumerate() {
            let mut instance = EffectEmitterInstance::from_fbs(node, my_seed);
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

        Self {
            emitters,
            state: RenderState { emitter_states, is_playing: true, ..Default::default() },
            is_infinite,
            layout_scale: Vector2::new(model.layout_scale_x() as f32 / 100.0, model.layout_scale_y() as f32 / 100.0),
            effect_time_length,
            fps: if model.fps() == 0 { 30 } else { model.fps() },
            draw_commands: Vec::new(),
            draw_particles: Vec::new(),
        }
    }

    pub fn update(&mut self, frame: f32) {
        if !self.state.is_playing || self.state.is_pausing {
            return;
        }

        self.state.current_frame = frame;
        self.state.target_frame = frame;

        if !self.is_infinite && self.state.is_loop && self.effect_time_length > 0 {
            if self.state.current_frame > self.effect_time_length as f32 {
                self.state.target_frame = libm::fmodf(self.state.current_frame, self.effect_time_length as f32);
                self.state.seed_offset = (self.state.current_frame / self.effect_time_length as f32) as i32;
            }
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
                let parent_born_info: Vec<(i32, f32)> = {
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

                for (parent_particle_id, p_ptime) in parent_born_info {
                    self.collect_emitter_particles(i, p_ptime, pi, parent_particle_id);
                }
            } else {
                self.collect_emitter_particles(i, self.state.target_frame, -1, 0);
            }

            let end_idx = self.draw_particles.len();
            if end_idx > start_idx {
                let e = &self.emitters[i];
                self.draw_commands.push(EmitterDrawCommand { cell_map_name_hash: e.cell_map_name_hash, cell_name_hash: e.cell_name_hash, blend_type: e.blend_type, particle_start: start_idx, particle_end: end_idx });
            }
        }

        (&self.draw_commands, &self.draw_particles)
    }

    fn collect_emitter_particles(&mut self, emitter_idx: usize, time: f32, parent_idx: i32, parent_particle_id: i32) {
        let e = &self.emitters[emitter_idx];
        let emitter_state = &mut self.state.emitter_states[emitter_idx];

        let slide = if parent_idx >= 0 { parent_particle_id } else { 0 };

        EffectSimulator::update_emitter(emitter_state, &e.emitter_params, &e.emit_patterns, &e.offset_patterns, time, slide);

        let mut active_indices = Vec::with_capacity(e.offset_patterns.len());
        for id in 0..e.offset_patterns.len() {
            if emitter_state.particle_state.exist[id] {
                active_indices.push(id);
            }
        }

        // SIMD batch processing
        let mut chunks = active_indices.chunks_exact(4);
        for chunk in &mut chunks {
            let pids = [chunk[0] as i32, chunk[1] as i32, chunk[2] as i32, chunk[3] as i32];
            let stimes = [emitter_state.particle_state.stime[chunk[0]], emitter_state.particle_state.stime[chunk[1]], emitter_state.particle_state.stime[chunk[2]], emitter_state.particle_state.stime[chunk[3]]];
            let endtimes = [emitter_state.particle_state.endtime[chunk[0]], emitter_state.particle_state.endtime[chunk[1]], emitter_state.particle_state.endtime[chunk[2]], emitter_state.particle_state.endtime[chunk[3]]];

            let mut ds_batch = EffectSimulator::calculate_state_batch(
                time,
                stimes,
                endtimes,
                pids,
                &e.emitter_params,
                &e.particle_params,
                e.emitter_seed,
                self.state.seed_offset,
                &e.seed_list,
                emitter_state.position,
                if parent_idx >= 0 { [parent_particle_id; 4] } else { [0; 4] },
            );

            for ds in &mut ds_batch {
                ds.x *= self.layout_scale.x;
                ds.y *= self.layout_scale.y;
                self.draw_particles.push(*ds);
            }
        }

        // Remainder processing
        for &id in chunks.remainder() {
            let mut ds = EffectSimulator::calculate_state(
                time,
                emitter_state.particle_state.stime[id],
                emitter_state.particle_state.endtime[id],
                id as i32,
                &e.emitter_params,
                &e.particle_params,
                e.emitter_seed,
                self.state.seed_offset,
                &e.seed_list,
                emitter_state.position,
                if parent_idx >= 0 { parent_particle_id } else { 0 },
            );

            ds.x *= self.layout_scale.x;
            ds.y *= self.layout_scale.y;

            self.draw_particles.push(ds);
        }
    }
}
