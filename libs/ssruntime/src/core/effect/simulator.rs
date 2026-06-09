use super::state::*;
use super::types::*;
use crate::core::math::*;
use alloc::vec::Vec;
use wide::f32x4;

pub fn out_quad(t: f32, total_time: f32, max: f32, min: f32) -> f32 {
    if total_time == 0.0 {
        return 0.0;
    }
    let mut t = t;
    if t > total_time {
        t = total_time;
    }
    let range = max - min;
    t /= total_time;
    -range * t * (t - 2.0) + min
}

pub fn out_quad_simd(t: f32x4, total_time: f32x4, max: f32x4, min: f32x4) -> f32x4 {
    let mut t = t;
    let t_gt_total = t.simd_gt(total_time);
    t = t_gt_total.blend(total_time, t);

    let range = max - min;
    let t_norm = t / total_time;
    let res = (-range * t_norm).mul_add(t_norm - f32x4::splat(2.0), min);

    let total_time_zero = total_time.simd_eq(f32x4::splat(0.0));
    total_time_zero.blend(f32x4::splat(0.0), res)
}

fn blend_number(a: u8, b: u8, rate: f32) -> u8 {
    (a as f32 + (b as i32 - a as i32) as f32 * rate) as u8
}

fn blend_float(a: f32, b: f32, rate: f32) -> f32 {
    a + (b - a) * rate
}

fn blend_float_simd(a: f32x4, b: f32x4, rate: f32x4) -> f32x4 {
    (b - a).mul_add(rate, a)
}

// Vectorized PRNG using wide
pub use super::prng::{XorShift32, XorShift32x4};

pub struct EffectSimulator;

impl EffectSimulator {
    pub fn precalculate_emitter(emitter_params: &EmitterParameter, particle_params: &ParticleParameter, emitter_seed: i32) -> (Vec<EmitPattern>, Vec<i32>, Vec<u32>) {
        let mut rng = XorShift32::new(emitter_seed as u32);

        let mut emit_patterns = Vec::new();
        let mut offset_patterns = Vec::new();

        let emitnum = if emitter_params.emitnum < 1 { 1 } else { emitter_params.emitnum };
        let cycle = ((emitter_params.emitmax * emitter_params.interval) as f32 / emitnum as f32 + 0.5) as i32;

        let mut extendsize = emitter_params.emitmax * LIFE_EXTEND_SCALE;
        if extendsize < LIFE_EXTEND_MIN {
            extendsize = LIFE_EXTEND_MIN;
        }

        let mut shot = 0;
        let mut offset = particle_params.delay as i32;
        for _ in 0..emitter_params.emitmax {
            if shot >= emitnum {
                shot = 0;
                offset += emitter_params.interval;
            }
            offset_patterns.push(offset);
            shot += 1;
        }

        for i in 0..extendsize {
            let life = emitter_params.particle_life + (emitter_params.particle_life2 as f32 * rng.gen_float()) as i32;
            let mut p_cycle = cycle;
            if life > cycle {
                p_cycle = life;
            }
            emit_patterns.push(EmitPattern { uid: i, life, cycle: p_cycle });
        }

        let mut rng = XorShift32::new(emitter_seed as u32);
        let seed_table_len = (emitter_params.emitmax as usize) * 3;
        let mut seed_list = Vec::with_capacity(seed_table_len);
        for _ in 0..seed_table_len {
            seed_list.push(rng.gen_u32());
        }

        (emit_patterns, offset_patterns, seed_list)
    }

    pub fn update_emitter(state: &mut EmitterState, emitter_params: &EmitterParameter, emit_patterns: &[EmitPattern], offset_patterns: &[i32], current_time: f32, slide: i32) {
        let pnum = emit_patterns.len();
        if pnum == 0 {
            return;
        }

        let slide_offset = slide * SEED_MAGIC;

        for (i, &offset) in offset_patterns.iter().enumerate() {
            let slide_num = (i as i32 + slide_offset).rem_euclid(pnum as i32) as usize;
            let target_ep = &emit_patterns[slide_num];
            let t = current_time - offset as f32;

            state.particle_state.exist[i] = false;
            state.particle_state.born[i] = false;

            if target_ep.cycle != 0 {
                let loopnum = floorf(t / target_ep.cycle as f32) as i32;
                let cycle_top = loopnum * target_ep.cycle;

                state.particle_state.id[i] = target_ep.uid;
                state.particle_state.pid[i] = slide;
                state.particle_state.cycle[i] = loopnum;
                state.particle_state.stime[i] = cycle_top + offset;
                state.particle_state.endtime[i] = state.particle_state.stime[i] + target_ep.life;

                if state.particle_state.stime[i] as f32 <= current_time && (state.particle_state.endtime[i] as f32) > current_time {
                    state.particle_state.exist[i] = true;
                    state.particle_state.born[i] = true;
                }

                if !emitter_params.is_infinite {
                    if state.particle_state.stime[i] >= emitter_params.life {
                        state.particle_state.exist[i] = false;

                        let t_limit = emitter_params.life - offset_patterns[i];
                        let loopnum_limit = t_limit / target_ep.cycle;
                        let cycle_top_limit = loopnum_limit * target_ep.cycle;

                        state.particle_state.stime[i] = cycle_top_limit + offset_patterns[i];
                        state.particle_state.endtime[i] = state.particle_state.stime[i] + target_ep.life;
                        state.particle_state.born[i] = false;
                    } else {
                        state.particle_state.born[i] = true;
                    }
                }

                if t < 0.0 {
                    state.particle_state.exist[i] = false;
                    state.particle_state.born[i] = false;
                }
            }
        }
    }

    pub fn calculate_state(
        current_time: f32,
        particle_stime: i32,
        particle_endtime: i32,
        particle_id: i32,
        _emitter_params: &EmitterParameter,
        particle_params: &ParticleParameter,
        emitter_seed: i32,
        seed_offset: i32,
        seed_list: &[u32],
        emitter_pos: Vector2,
        pid: i32,
    ) -> ParticleDrawState {
        let t = current_time - particle_stime as f32;
        let life = (particle_endtime - particle_stime) as f32;

        if life == 0.0 {
            return ParticleDrawState::default();
        }

        let lifeper = t / life;
        let seed_table_len = seed_list.len();
        let pseed = seed_list[particle_id as usize % seed_table_len];
        let seed = pseed.wrapping_add(emitter_seed as u32).wrapping_add(pid as u32).wrapping_add(seed_offset as u32);

        let mut rng = XorShift32::new(seed);
        let mut gen_rand = || rng.gen_float();

        let out_x;
        let out_y;
        {
            let rad = particle_params.angle + (gen_rand() * particle_params.angle_variance - particle_params.angle_variance / 2.0);
            let speed = particle_params.speed + (particle_params.speed2 * gen_rand());

            let mut addr = 0.0;
            if particle_params.use_tan_accel {
                let accel = particle_params.tangential_accel + (gen_rand() * particle_params.tangential_accel2);
                let _speed = if speed <= 0.0 { 0.1 } else { speed };
                let l = life * _speed * 0.2;
                let c = core::f32::consts::PI * l;
                addr = (accel / c) * t;
            }

            let mut x = cosf(rad + addr) * speed * t;
            let mut y = sinf(rad + addr) * speed * t;

            if particle_params.use_trans_speed {
                let transspeed = particle_params.trans_speed + (particle_params.trans_speed2 * gen_rand());
                let speedadd = transspeed / life;
                let addtx = cosf(rad + addr) * speed;
                let addtx_trans = cosf(rad + addr) * speedadd;
                x = ((addtx_trans * t) + addtx) * (t + 1.0) / 2.0;

                let addty = sinf(rad + addr) * speed;
                let addty_trans = sinf(rad + addr) * speedadd;
                y = ((addty_trans * t) + addty) * (t + 1.0) / 2.0;
            }

            if particle_params.use_gravity {
                let t2 = t * t;
                x += 0.5 * particle_params.gravity.x * t2;
                y += 0.5 * particle_params.gravity.y * t2;
            }
            out_x = x;
            out_y = y;
        }

        let mut ox = 0.0;
        let mut oy = 0.0;
        if particle_params.use_offset {
            ox = particle_params.offset.x + (particle_params.offset2.x * gen_rand());
            oy = particle_params.offset.y + (particle_params.offset2.y * gen_rand());
        }

        let mut rotation = 0.0;
        if particle_params.use_rotation {
            let rot = particle_params.rotation + (gen_rand() * particle_params.rotation2);
            let add = particle_params.rotation_add + (gen_rand() * particle_params.rotation_add2);

            if particle_params.use_rotation_trans {
                let lastt = life * particle_params.end_life_time_per;
                if lastt == 0.0 {
                    rotation = rot + (add * particle_params.rotation_factor) * t;
                } else {
                    let addf = (add * particle_params.rotation_factor - add) / lastt;
                    let mod_t = if t > lastt { t - lastt } else { 0.0 };
                    let nowt = if t < lastt { t } else { lastt };
                    let final_soul = add + addf * nowt;
                    let mut addrf = (final_soul + add) * (nowt + 1.0) / 2.0;
                    addrf -= add;
                    addrf += mod_t * final_soul;
                    rotation = rot + addrf;
                }
            } else {
                rotation = rot + (add * t);
            }
        }

        let mut color = Color::new(255, 255, 255, 255);
        if particle_params.use_color {
            color.a = (particle_params.init_color.a as f32 + (gen_rand() * particle_params.init_color2.a as f32)) as u8;
            color.r = (particle_params.init_color.r as f32 + (gen_rand() * particle_params.init_color2.r as f32)) as u8;
            color.g = (particle_params.init_color.g as f32 + (gen_rand() * particle_params.init_color2.g as f32)) as u8;
            color.b = (particle_params.init_color.b as f32 + (gen_rand() * particle_params.init_color2.b as f32)) as u8;
        }

        if particle_params.use_trans_color {
            let ecolor_a = (particle_params.trans_color.a as f32 + (gen_rand() * particle_params.trans_color2.a as f32)) as u8;
            let ecolor_r = (particle_params.trans_color.r as f32 + (gen_rand() * particle_params.trans_color2.r as f32)) as u8;
            let ecolor_g = (particle_params.trans_color.g as f32 + (gen_rand() * particle_params.trans_color2.g as f32)) as u8;
            let ecolor_b = (particle_params.trans_color.b as f32 + (gen_rand() * particle_params.trans_color2.b as f32)) as u8;

            color.a = blend_number(color.a, ecolor_a, lifeper);
            color.r = blend_number(color.r, ecolor_r, lifeper);
            color.g = blend_number(color.g, ecolor_g, lifeper);
            color.b = blend_number(color.b, ecolor_b, lifeper);
        }

        if particle_params.use_alpha_fade {
            let start = particle_params.alpha_fade;
            let end = particle_params.alpha_fade2;
            let per = lifeper * 100.0;
            if per < start && start > 0.0 {
                let alpha = (start - per) / start;
                color.a = (color.a as f32 * (1.0 - alpha)) as u8;
            } else if per > end {
                if end >= 100.0 {
                    color.a = 0;
                } else {
                    let alpha = if (per - end) / (100.0 - end) < 1.0 { (per - end) / (100.0 - end) } else { 1.0 };
                    color.a = (color.a as f32 * (1.0 - alpha)) as u8;
                }
            }
        }

        let mut scale_x = 1.0;
        let mut scale_y = 1.0;
        let mut scalefactor = 1.0;
        if particle_params.use_init_scale {
            scale_x = particle_params.scale.x + (gen_rand() * particle_params.scale_range.x);
            scale_y = particle_params.scale.y + (gen_rand() * particle_params.scale_range.y);
            scalefactor = particle_params.scale_factor + (gen_rand() * particle_params.scale_factor2);
        }
        if particle_params.use_trans_scale {
            let s2_x = particle_params.transscale.x + (gen_rand() * particle_params.scale_range2.x);
            let s2_y = particle_params.transscale.y + (gen_rand() * particle_params.scale_range2.y);
            let sf2 = particle_params.transscale_factor + (gen_rand() * particle_params.transscale_factor2);
            scale_x = blend_float(scale_x, s2_x, lifeper);
            scale_y = blend_float(scale_y, s2_y, lifeper);
            scalefactor = blend_float(scalefactor, sf2, lifeper);
        }
        let out_scale_x = scale_x * scalefactor;
        let out_scale_y = scale_y * scalefactor;

        let mut final_x = out_x + ox + emitter_pos.x;
        let mut final_y = out_y + oy + emitter_pos.y;

        if particle_params.use_p_gravity {
            let vx = particle_params.gravity_pos.x - (ox + emitter_pos.x);
            let vy = particle_params.gravity_pos.y - (oy + emitter_pos.y);
            let dist = sqrtf(vx * vx + vy * vy);
            let inv_dist = if dist == 0.0 { 0.0 } else { 1.0 / dist };
            let nvx = vx * inv_dist;
            let nvy = vy * inv_dist;

            let gp = particle_params.gravity_power;
            if gp > 0.0 {
                let len = if dist > 0.1 { dist } else { 0.1 };
                let et = (len / gp) * 0.90;
                let gt = if t < et * 0.90 { t } else { et * 0.90 };
                final_x += nvx * gp * gt;
                final_y += nvy * gp * gt;
                let blend = out_quad(gt, et, 0.9, 0.0) + (t / life * 0.1);
                final_x = blend_float(final_x, particle_params.gravity_pos.x, blend);
                final_y = blend_float(final_y, particle_params.gravity_pos.y, blend);
            } else {
                final_x += nvx * gp * t;
                final_y += nvy * gp * t;
            }
        }

        let mut direction = 0.0;
        if particle_params.use_turn_direc {
            let next_t = t + 1.0;
            let mut rng_next = XorShift32::new(seed);
            let mut gen_rand_next = || rng_next.gen_float();

            let rad = particle_params.angle + (gen_rand_next() * particle_params.angle_variance - particle_params.angle_variance / 2.0);
            let speed = particle_params.speed + (particle_params.speed2 * gen_rand_next());

            let mut addr = 0.0;
            if particle_params.use_tan_accel {
                let accel = particle_params.tangential_accel + (gen_rand_next() * particle_params.tangential_accel2);
                let _speed = if speed <= 0.0 { 0.1 } else { speed };
                let l = life * _speed * 0.2;
                let c = core::f32::consts::PI * l;
                addr = (accel / c) * next_t;
            }

            let mut x = cosf(rad + addr) * speed * next_t;
            let mut y = sinf(rad + addr) * speed * next_t;

            if particle_params.use_trans_speed {
                let transspeed = particle_params.trans_speed + (particle_params.trans_speed2 * gen_rand_next());
                let speedadd = transspeed / life;
                let addtx = cosf(rad + addr) * speed;
                let addtx_trans = cosf(rad + addr) * speedadd;
                x = ((addtx_trans * next_t) + addtx) * (next_t + 1.0) / 2.0;

                let addty = sinf(rad + addr) * speed;
                let addty_trans = sinf(rad + addr) * speedadd;
                y = ((addty_trans * next_t) + addty) * (next_t + 1.0) / 2.0;
            }

            if particle_params.use_gravity {
                let t2 = next_t * next_t;
                x += 0.5 * particle_params.gravity.x * t2;
                y += 0.5 * particle_params.gravity.y * t2;
            }

            let mut ox = 0.0;
            let mut oy = 0.0;
            if particle_params.use_offset {
                ox = particle_params.offset.x + (particle_params.offset2.x * gen_rand_next());
                oy = particle_params.offset.y + (particle_params.offset2.y * gen_rand_next());
            }

            let mut next_final_x = x + ox + emitter_pos.x;
            let mut next_final_y = y + oy + emitter_pos.y;

            if particle_params.use_p_gravity {
                let vx = particle_params.gravity_pos.x - (ox + emitter_pos.x);
                let vy = particle_params.gravity_pos.y - (oy + emitter_pos.y);
                let dist = sqrtf(vx * vx + vy * vy);
                let inv_dist = if dist == 0.0 { 0.0 } else { 1.0 / dist };
                let nvx = vx * inv_dist;
                let nvy = vy * inv_dist;

                let gp = particle_params.gravity_power;
                if gp > 0.0 {
                    let len = if dist > 0.1 { dist } else { 0.1 };
                    let et = (len / gp) * 0.90;
                    let gt = if next_t < et * 0.90 { next_t } else { et * 0.90 };
                    next_final_x += nvx * gp * gt;
                    next_final_y += nvy * gp * gt;
                    let blend = out_quad(gt, et, 0.9, 0.0) + (next_t / life * 0.1);
                    next_final_x = blend_float(next_final_x, particle_params.gravity_pos.x, blend);
                    next_final_y = blend_float(next_final_y, particle_params.gravity_pos.y, blend);
                } else {
                    next_final_x += nvx * gp * next_t;
                    next_final_y += nvy * gp * next_t;
                }
            }

            direction = atan2f(next_final_y - final_y, next_final_x - final_x) + core::f32::consts::FRAC_PI_2 + (particle_params.direc_rot_add * core::f32::consts::PI / 180.0);
        }

        ParticleDrawState { x: final_x, y: final_y, scale_x: out_scale_x, scale_y: out_scale_y, rotation, direction, color }
    }

    /// SIMD batch calculation for 4 particles simultaneously.
    pub fn calculate_state_batch(
        current_time: f32,
        stimes: [i32; 4],
        endtimes: [i32; 4],
        particle_ids: [i32; 4],
        _emitter_params: &EmitterParameter,
        particle_params: &ParticleParameter,
        emitter_seed: i32,
        seed_offset: i32,
        seed_list: &[u32],
        emitter_pos: [Vector2; 4],
        pids: [i32; 4],
    ) -> [ParticleDrawState; 4] {
        let stimes_f = f32x4::from([stimes[0] as f32, stimes[1] as f32, stimes[2] as f32, stimes[3] as f32]);
        let endtimes_f = f32x4::from([endtimes[0] as f32, endtimes[1] as f32, endtimes[2] as f32, endtimes[3] as f32]);
        let t = f32x4::splat(current_time) - stimes_f;
        let life = endtimes_f - stimes_f;

        let life_zero = life.simd_eq(f32x4::splat(0.0));
        let life_safe = life_zero.blend(f32x4::splat(1.0), life);

        let lifeper = t / life_safe;

        let seed_table_len = seed_list.len();
        let pseeds = [seed_list[particle_ids[0] as usize % seed_table_len], seed_list[particle_ids[1] as usize % seed_table_len], seed_list[particle_ids[2] as usize % seed_table_len], seed_list[particle_ids[3] as usize % seed_table_len]];
        let seeds = [
            pseeds[0].wrapping_add(emitter_seed as u32).wrapping_add(pids[0] as u32).wrapping_add(seed_offset as u32),
            pseeds[1].wrapping_add(emitter_seed as u32).wrapping_add(pids[1] as u32).wrapping_add(seed_offset as u32),
            pseeds[2].wrapping_add(emitter_seed as u32).wrapping_add(pids[2] as u32).wrapping_add(seed_offset as u32),
            pseeds[3].wrapping_add(emitter_seed as u32).wrapping_add(pids[3] as u32).wrapping_add(seed_offset as u32),
        ];
        let mut rng = XorShift32x4::new(seeds);

        let out_x;
        let out_y;
        {
            let rad = rng.gen_float().mul_add(f32x4::splat(particle_params.angle_variance), f32x4::splat(particle_params.angle - particle_params.angle_variance / 2.0));
            let speed = rng.gen_float().mul_add(f32x4::splat(particle_params.speed2), f32x4::splat(particle_params.speed));

            let mut addr = f32x4::splat(0.0);
            if particle_params.use_tan_accel {
                let accel = rng.gen_float().mul_add(f32x4::splat(particle_params.tangential_accel2), f32x4::splat(particle_params.tangential_accel));
                let speed_safe = speed.simd_le(f32x4::splat(0.0)).blend(f32x4::splat(0.1), speed);
                let l = life_safe * speed_safe * f32x4::splat(0.2);
                let c = f32x4::splat(core::f32::consts::PI) * l;
                addr = (accel * c.recip()) * t;
            }

            let rad_addr = rad + addr;
            let (sin_rad_addr, cos_rad_addr) = rad_addr.sin_cos();
            let mut x = cos_rad_addr * speed * t;
            let mut y = sin_rad_addr * speed * t;

            if particle_params.use_trans_speed {
                let transspeed = rng.gen_float().mul_add(f32x4::splat(particle_params.trans_speed2), f32x4::splat(particle_params.trans_speed));
                let speedadd = transspeed * life_safe.recip();
                let addtx = cos_rad_addr * speed;
                let addtx_trans = cos_rad_addr * speedadd;
                x = addtx_trans.mul_add(t, addtx) * (t + f32x4::splat(1.0)) * f32x4::splat(0.5);

                let addty = sin_rad_addr * speed;
                let addty_trans = sin_rad_addr * speedadd;
                y = addty_trans.mul_add(t, addty) * (t + f32x4::splat(1.0)) * f32x4::splat(0.5);
            }

            if particle_params.use_gravity {
                let t2 = t * t;
                x = f32x4::splat(0.5 * particle_params.gravity.x).mul_add(t2, x);
                y = f32x4::splat(0.5 * particle_params.gravity.y).mul_add(t2, y);
            }
            out_x = x;
            out_y = y;
        }

        let mut ox = f32x4::splat(0.0);
        let mut oy = f32x4::splat(0.0);
        if particle_params.use_offset {
            ox = rng.gen_float().mul_add(f32x4::splat(particle_params.offset2.x), f32x4::splat(particle_params.offset.x));
            oy = rng.gen_float().mul_add(f32x4::splat(particle_params.offset2.y), f32x4::splat(particle_params.offset.y));
        }

        let mut rotation = f32x4::splat(0.0);
        if particle_params.use_rotation {
            let rot = rng.gen_float().mul_add(f32x4::splat(particle_params.rotation2), f32x4::splat(particle_params.rotation));
            let add = rng.gen_float().mul_add(f32x4::splat(particle_params.rotation_add2), f32x4::splat(particle_params.rotation_add));

            if particle_params.use_rotation_trans {
                let lastt = life_safe * f32x4::splat(particle_params.end_life_time_per);

                let lastt_zero = lastt.simd_eq(f32x4::splat(0.0));
                let res_zero = (add * f32x4::splat(particle_params.rotation_factor)).mul_add(t, rot);

                let addf = add.mul_sub(f32x4::splat(particle_params.rotation_factor), add) * lastt.recip();
                let mod_t = t.simd_gt(lastt).blend(t - lastt, f32x4::splat(0.0));
                let nowt = t.simd_lt(lastt).blend(t, lastt);
                let final_soul = addf.mul_add(nowt, add);
                let mut addrf = (final_soul + add) * (nowt + f32x4::splat(1.0)) * f32x4::splat(0.5);
                addrf -= add;
                addrf = mod_t.mul_add(final_soul, addrf);
                let res_normal = rot + addrf;

                rotation = lastt_zero.blend(res_zero, res_normal);
            } else {
                rotation = add.mul_add(t, rot);
            }
        }

        let mut color_a = f32x4::splat(255.0);
        let mut color_r = f32x4::splat(255.0);
        let mut color_g = f32x4::splat(255.0);
        let mut color_b = f32x4::splat(255.0);

        if particle_params.use_color {
            color_a = rng.gen_float().mul_add(f32x4::splat(particle_params.init_color2.a as f32), f32x4::splat(particle_params.init_color.a as f32)).trunc();
            color_r = rng.gen_float().mul_add(f32x4::splat(particle_params.init_color2.r as f32), f32x4::splat(particle_params.init_color.r as f32)).trunc();
            color_g = rng.gen_float().mul_add(f32x4::splat(particle_params.init_color2.g as f32), f32x4::splat(particle_params.init_color.g as f32)).trunc();
            color_b = rng.gen_float().mul_add(f32x4::splat(particle_params.init_color2.b as f32), f32x4::splat(particle_params.init_color.b as f32)).trunc();
        }

        if particle_params.use_trans_color {
            let ecolor_a = rng.gen_float().mul_add(f32x4::splat(particle_params.trans_color2.a as f32), f32x4::splat(particle_params.trans_color.a as f32)).trunc();
            let ecolor_r = rng.gen_float().mul_add(f32x4::splat(particle_params.trans_color2.r as f32), f32x4::splat(particle_params.trans_color.r as f32)).trunc();
            let ecolor_g = rng.gen_float().mul_add(f32x4::splat(particle_params.trans_color2.g as f32), f32x4::splat(particle_params.trans_color.g as f32)).trunc();
            let ecolor_b = rng.gen_float().mul_add(f32x4::splat(particle_params.trans_color2.b as f32), f32x4::splat(particle_params.trans_color.b as f32)).trunc();

            color_a = blend_float_simd(color_a, ecolor_a, lifeper).trunc();
            color_r = blend_float_simd(color_r, ecolor_r, lifeper).trunc();
            color_g = blend_float_simd(color_g, ecolor_g, lifeper).trunc();
            color_b = blend_float_simd(color_b, ecolor_b, lifeper).trunc();
        }

        if particle_params.use_alpha_fade {
            let start = f32x4::splat(particle_params.alpha_fade);
            let end = f32x4::splat(particle_params.alpha_fade2);
            let per = lifeper * f32x4::splat(100.0);

            let cond_start = per.simd_lt(start) & start.simd_gt(f32x4::splat(0.0));
            let alpha_start = (start - per) / start;
            let color_a_start = color_a * (f32x4::splat(1.0) - alpha_start);

            let cond_end = per.simd_gt(end);
            let end_ge_100 = end.simd_ge(f32x4::splat(100.0));
            let alpha_end = ((per - end) / (f32x4::splat(100.0) - end)).min(f32x4::splat(1.0));
            let color_a_end = end_ge_100.blend(f32x4::splat(0.0), color_a * (f32x4::splat(1.0) - alpha_end));

            color_a = cond_start.blend(color_a_start, cond_end.blend(color_a_end, color_a)).trunc();
        }

        let mut scale_x = f32x4::splat(1.0);
        let mut scale_y = f32x4::splat(1.0);
        let mut scalefactor = f32x4::splat(1.0);
        if particle_params.use_init_scale {
            scale_x = rng.gen_float().mul_add(f32x4::splat(particle_params.scale_range.x), f32x4::splat(particle_params.scale.x));
            scale_y = rng.gen_float().mul_add(f32x4::splat(particle_params.scale_range.y), f32x4::splat(particle_params.scale.y));
            scalefactor = rng.gen_float().mul_add(f32x4::splat(particle_params.scale_factor2), f32x4::splat(particle_params.scale_factor));
        }
        if particle_params.use_trans_scale {
            let s2_x = rng.gen_float().mul_add(f32x4::splat(particle_params.scale_range2.x), f32x4::splat(particle_params.transscale.x));
            let s2_y = rng.gen_float().mul_add(f32x4::splat(particle_params.scale_range2.y), f32x4::splat(particle_params.transscale.y));
            let sf2 = rng.gen_float().mul_add(f32x4::splat(particle_params.transscale_factor2), f32x4::splat(particle_params.transscale_factor));
            scale_x = blend_float_simd(scale_x, s2_x, lifeper);
            scale_y = blend_float_simd(scale_y, s2_y, lifeper);
            scalefactor = blend_float_simd(scalefactor, sf2, lifeper);
        }
        let out_scale_x = scale_x * scalefactor;
        let out_scale_y = scale_y * scalefactor;

        let emitter_pos_x = f32x4::from([emitter_pos[0].x, emitter_pos[1].x, emitter_pos[2].x, emitter_pos[3].x]);
        let emitter_pos_y = f32x4::from([emitter_pos[0].y, emitter_pos[1].y, emitter_pos[2].y, emitter_pos[3].y]);

        let mut final_x = out_x + ox + emitter_pos_x;
        let mut final_y = out_y + oy + emitter_pos_y;

        if particle_params.use_p_gravity {
            let vx = f32x4::splat(particle_params.gravity_pos.x) - (ox + emitter_pos_x);
            let vy = f32x4::splat(particle_params.gravity_pos.y) - (oy + emitter_pos_y);
            let dist_sq = vx.mul_add(vx, vy * vy);
            let inv_dist = dist_sq.recip_sqrt();
            let inv_dist = dist_sq.simd_eq(f32x4::splat(0.0)).blend(f32x4::splat(0.0), inv_dist);
            let nvx = vx * inv_dist;
            let nvy = vy * inv_dist;
            let dist = dist_sq * inv_dist;

            let gp = f32x4::splat(particle_params.gravity_power);
            let gp_gt_zero = gp.simd_gt(f32x4::splat(0.0));

            // gp > 0 case
            let len = dist.max(f32x4::splat(0.1));
            let et = (len * gp.recip()) * f32x4::splat(0.90);
            let gt = t.min(et * f32x4::splat(0.90));
            let res_pos_x_gt = (nvx * gp).mul_add(gt, final_x);
            let res_pos_y_gt = (nvy * gp).mul_add(gt, final_y);
            let blend = out_quad_simd(gt, et, f32x4::splat(0.9), f32x4::splat(0.0)) + (t * life_safe.recip() * f32x4::splat(0.1));
            let final_x_gt = blend_float_simd(res_pos_x_gt, f32x4::splat(particle_params.gravity_pos.x), blend);
            let final_y_gt = blend_float_simd(res_pos_y_gt, f32x4::splat(particle_params.gravity_pos.y), blend);

            // gp <= 0 case
            let final_x_le = (nvx * gp).mul_add(t, final_x);
            let final_y_le = (nvy * gp).mul_add(t, final_y);

            final_x = gp_gt_zero.blend(final_x_gt, final_x_le);
            final_y = gp_gt_zero.blend(final_y_gt, final_y_le);
        }

        let mut direction = [0.0; 4];
        if particle_params.use_turn_direc {
            let next_t = t + f32x4::splat(1.0);
            let mut rng_next = XorShift32x4::new(seeds);
            let mut gen_rand_next = || rng_next.gen_float();

            let rad = gen_rand_next().mul_add(f32x4::splat(particle_params.angle_variance), f32x4::splat(particle_params.angle - particle_params.angle_variance / 2.0));
            let speed = gen_rand_next().mul_add(f32x4::splat(particle_params.speed2), f32x4::splat(particle_params.speed));

            let mut addr = f32x4::splat(0.0);
            if particle_params.use_tan_accel {
                let accel = gen_rand_next().mul_add(f32x4::splat(particle_params.tangential_accel2), f32x4::splat(particle_params.tangential_accel));
                let speed_safe = speed.simd_le(f32x4::splat(0.0)).blend(f32x4::splat(0.1), speed);
                let l = life_safe * speed_safe * f32x4::splat(0.2);
                let c = f32x4::splat(core::f32::consts::PI) * l;
                addr = (accel * c.recip()) * next_t;
            }

            let rad_addr = rad + addr;
            let (sin_rad_addr, cos_rad_addr) = rad_addr.sin_cos();
            let mut x = cos_rad_addr * speed * next_t;
            let mut y = sin_rad_addr * speed * next_t;

            if particle_params.use_trans_speed {
                let transspeed = gen_rand_next().mul_add(f32x4::splat(particle_params.trans_speed2), f32x4::splat(particle_params.trans_speed));
                let speedadd = transspeed * life_safe.recip();
                let addtx = cos_rad_addr * speed;
                let addtx_trans = cos_rad_addr * speedadd;
                x = addtx_trans.mul_add(next_t, addtx) * (next_t + f32x4::splat(1.0)) * f32x4::splat(0.5);

                let addty = sin_rad_addr * speed;
                let addty_trans = sin_rad_addr * speedadd;
                y = addty_trans.mul_add(next_t, addty) * (next_t + f32x4::splat(1.0)) * f32x4::splat(0.5);
            }

            if particle_params.use_gravity {
                let t2 = next_t * next_t;
                x = f32x4::splat(0.5 * particle_params.gravity.x).mul_add(t2, x);
                y = f32x4::splat(0.5 * particle_params.gravity.y).mul_add(t2, y);
            }

            let mut ox = f32x4::splat(0.0);
            let mut oy = f32x4::splat(0.0);
            if particle_params.use_offset {
                ox = gen_rand_next().mul_add(f32x4::splat(particle_params.offset2.x), f32x4::splat(particle_params.offset.x));
                oy = gen_rand_next().mul_add(f32x4::splat(particle_params.offset2.y), f32x4::splat(particle_params.offset.y));
            }

            let mut next_final_x = x + ox + emitter_pos_x;
            let mut next_final_y = y + oy + emitter_pos_y;

            if particle_params.use_p_gravity {
                let vx = f32x4::splat(particle_params.gravity_pos.x) - (ox + emitter_pos_x);
                let vy = f32x4::splat(particle_params.gravity_pos.y) - (oy + emitter_pos_y);
                let dist_sq = vx.mul_add(vx, vy * vy);
                let inv_dist = dist_sq.recip_sqrt();
                let inv_dist = dist_sq.simd_eq(f32x4::splat(0.0)).blend(f32x4::splat(0.0), inv_dist);
                let nvx = vx * inv_dist;
                let nvy = vy * inv_dist;
                let dist = dist_sq * inv_dist;

                let gp = f32x4::splat(particle_params.gravity_power);
                let gp_gt_zero = gp.simd_gt(f32x4::splat(0.0));

                let len = dist.max(f32x4::splat(0.1));
                let et = (len * gp.recip()) * f32x4::splat(0.90);
                let gt = next_t.min(et * f32x4::splat(0.90));
                let res_pos_x_gt = (nvx * gp).mul_add(gt, next_final_x);
                let res_pos_y_gt = (nvy * gp).mul_add(gt, next_final_y);
                let blend = out_quad_simd(gt, et, f32x4::splat(0.9), f32x4::splat(0.0)) + (next_t * life_safe.recip() * f32x4::splat(0.1));
                let final_x_gt = blend_float_simd(res_pos_x_gt, f32x4::splat(particle_params.gravity_pos.x), blend);
                let final_y_gt = blend_float_simd(res_pos_y_gt, f32x4::splat(particle_params.gravity_pos.y), blend);

                let final_x_le = (nvx * gp).mul_add(next_t, next_final_x);
                let final_y_le = (nvy * gp).mul_add(next_t, next_final_y);

                next_final_x = gp_gt_zero.blend(final_x_gt, final_x_le);
                next_final_y = gp_gt_zero.blend(final_y_gt, final_y_le);
            }

            let dy = next_final_y - final_y;
            let dx = next_final_x - final_x;
            let bias = f32x4::splat(core::f32::consts::FRAC_PI_2 + (particle_params.direc_rot_add * core::f32::consts::PI / 180.0));
            direction = (dy.atan2(dx) + bias).to_array();
        }

        let out_x_arr = final_x.to_array();
        let out_y_arr = final_y.to_array();
        let scale_x_arr = out_scale_x.to_array();
        let scale_y_arr = out_scale_y.to_array();
        let rot_arr = rotation.to_array();
        let col_a_arr = color_a.to_array();
        let col_r_arr = color_r.to_array();
        let col_g_arr = color_g.to_array();
        let col_b_arr = color_b.to_array();

        let mut results = [ParticleDrawState::default(); 4];
        let life_zero_mask = life_zero.to_bitmask();
        for i in 0..4 {
            // If the i-th bit is 1, it means life was zero
            if (life_zero_mask & (1 << i)) != 0 {
                continue;
            }
            results[i] = ParticleDrawState {
                x: out_x_arr[i],
                y: out_y_arr[i],
                scale_x: scale_x_arr[i],
                scale_y: scale_y_arr[i],
                rotation: rot_arr[i],
                direction: direction[i],
                color: Color::new(col_a_arr[i] as u8, col_r_arr[i] as u8, col_g_arr[i] as u8, col_b_arr[i] as u8),
            };
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_base_params() -> (EmitterParameter, ParticleParameter) {
        let emitter_params = EmitterParameter { life: 30, emitmax: 10, particle_life: 15, interval: 1, emitnum: 1, ..Default::default() };

        let particle_params = ParticleParameter { speed: 5.0, ..Default::default() };

        (emitter_params, particle_params)
    }

    #[test]
    fn test_precalculate_emitter_basic() {
        let (emitter_params, particle_params) = create_base_params();
        let emitter_seed = 12345;

        let (emit_patterns, offset_patterns, seed_list) = EffectSimulator::precalculate_emitter(&emitter_params, &particle_params, emitter_seed);

        assert_eq!(offset_patterns.len(), 10);
        assert_eq!(emit_patterns.len(), 10 * LIFE_EXTEND_SCALE as usize);
        assert_eq!(seed_list.len(), 30); // emitmax * 3

        for (i, &offset) in offset_patterns.iter().enumerate().take(10) {
            assert_eq!(offset, i as i32);
        }
    }

    #[test]
    fn test_update_emitter_lifecycle() {
        let (emitter_params, particle_params) = create_base_params();
        let emitter_seed = 12345;

        let (emit_patterns, offset_patterns, _) = EffectSimulator::precalculate_emitter(&emitter_params, &particle_params, emitter_seed);

        let mut state = EmitterState::new(10);

        EffectSimulator::update_emitter(&mut state, &emitter_params, &emit_patterns, &offset_patterns, 0.0, 0);
        assert!(state.particle_state.born[0]);
        assert!(state.particle_state.exist[0]);
        assert!(!state.particle_state.born[1]);

        EffectSimulator::update_emitter(&mut state, &emitter_params, &emit_patterns, &offset_patterns, 5.0, 0);
        for i in 0..=5 {
            assert!(state.particle_state.exist[i]);
        }
        assert!(!state.particle_state.exist[6]);

        EffectSimulator::update_emitter(&mut state, &emitter_params, &emit_patterns, &offset_patterns, 60.0, 0);
        assert!(!state.particle_state.exist[0]);
    }

    #[test]
    fn test_calculate_state_color_fade() {
        let (emitter_params, mut particle_params) = create_base_params();
        particle_params.use_color = true;
        particle_params.init_color = Color::new(255, 255, 0, 0);
        particle_params.init_color2 = Color::new(0, 0, 0, 0);

        particle_params.use_trans_color = true;
        particle_params.trans_color = Color::new(255, 0, 0, 255);
        particle_params.trans_color2 = Color::new(0, 0, 0, 0);

        let seed_list = vec![1, 2, 3];

        let draw_state = EffectSimulator::calculate_state(7.5, 0, 15, 0, &emitter_params, &particle_params, 12345, 0, &seed_list, Vector2::new(0.0, 0.0), 0);

        assert_eq!(draw_state.color.a, 255);
        assert!(draw_state.color.r < 255 && draw_state.color.r > 0);
        assert_eq!(draw_state.color.g, 0);
        assert!(draw_state.color.b > 0 && draw_state.color.b < 255);
    }

    #[test]
    fn test_calculate_state_gravity() {
        let (emitter_params, mut particle_params) = create_base_params();
        particle_params.speed = 0.0;
        particle_params.speed2 = 0.0;
        particle_params.use_gravity = true;
        particle_params.gravity = Vector2::new(0.0, -9.8);

        let seed_list = vec![1, 2, 3];

        let draw_state = EffectSimulator::calculate_state(1.0, 0, 15, 0, &emitter_params, &particle_params, 12345, 0, &seed_list, Vector2::new(0.0, 0.0), 0);

        assert_eq!(draw_state.x, 0.0);
        assert!(draw_state.y < 0.0);
        assert!((draw_state.y - (-4.9)).abs() < 0.001);
    }

    #[test]
    fn test_calculate_state_point_gravity() {
        let (emitter_params, mut particle_params) = create_base_params();
        particle_params.speed = 0.0;
        particle_params.speed2 = 0.0;
        particle_params.use_p_gravity = true;
        particle_params.gravity_pos = Vector2::new(10.0, 0.0);
        particle_params.gravity_power = 2.0;

        let seed_list = vec![1, 2, 3];

        let draw_state = EffectSimulator::calculate_state(1.0, 0, 15, 0, &emitter_params, &particle_params, 12345, 0, &seed_list, Vector2::new(0.0, 0.0), 0);

        assert!(draw_state.x > 0.0);
        assert_eq!(draw_state.y, 0.0);
    }

    #[test]
    fn test_xorshift32x4_simd() {
        let mut prng = XorShift32x4::new([123, 456, 789, 101112]);
        let rand_vals = prng.gen_float().to_array();

        for &val in &rand_vals {
            assert!((0.0..=1.0).contains(&val));
        }
    }

    #[test]
    fn test_calculate_state_simd_vs_scalar_equivalence() {
        let (emitter_params, mut particle_params) = create_base_params();

        // Enable various parameters to ensure all branches of SIMD calculation are tested
        particle_params.use_color = true;
        particle_params.init_color = Color::new(255, 100, 150, 200);
        particle_params.init_color2 = Color::new(50, 50, 50, 50);
        particle_params.use_trans_color = true;
        particle_params.trans_color = Color::new(255, 0, 0, 255);
        particle_params.trans_color2 = Color::new(0, 0, 0, 0);
        particle_params.use_gravity = true;
        particle_params.gravity = Vector2::new(1.0, -9.8);
        particle_params.use_rotation = true;
        particle_params.rotation = 45.0;
        particle_params.rotation2 = 10.0;
        particle_params.rotation_add = 5.0;
        particle_params.rotation_add2 = 2.0;

        let seed_list = vec![123, 456, 789, 1011, 1213];
        let emitter_seed = 9999;
        let seed_offset = 5;
        let current_time = 10.0;
        let emitter_pos = Vector2::new(100.0, 200.0);

        // Parameters for 4 particles
        let stimes = [0, 2, 4, 6];
        let endtimes = [15, 17, 19, 21];
        let particle_ids = [0, 1, 2, 3];
        let pids = [10, 20, 30, 40];

        // 1. Calculate using scalar fallback
        let mut scalar_results = [ParticleDrawState::default(); 4];
        for i in 0..4 {
            scalar_results[i] = EffectSimulator::calculate_state(current_time, stimes[i], endtimes[i], particle_ids[i], &emitter_params, &particle_params, emitter_seed, seed_offset, &seed_list, emitter_pos, pids[i]);
        }

        // 2. Calculate using SIMD batch
        let simd_results = EffectSimulator::calculate_state_batch(current_time, stimes, endtimes, particle_ids, &emitter_params, &particle_params, emitter_seed, seed_offset, &seed_list, [emitter_pos; 4], pids);

        // 3. Verify equivalence
        for i in 0..4 {
            let s = &scalar_results[i];
            let v = &simd_results[i];

            // Use a small epsilon for floating point comparison since SIMD vs Scalar math
            // can sometimes have minute precision differences, although here they should be exact.
            let epsilon = 1e-5;

            assert!((s.x - v.x).abs() < epsilon, "X mismatch at index {}: scalar {}, simd {}", i, s.x, v.x);
            assert!((s.y - v.y).abs() < epsilon, "Y mismatch at index {}: scalar {}, simd {}", i, s.y, v.y);
            assert!((s.scale_x - v.scale_x).abs() < epsilon, "Scale X mismatch at index {}", i);
            assert!((s.scale_y - v.scale_y).abs() < epsilon, "Scale Y mismatch at index {}", i);
            assert!((s.rotation - v.rotation).abs() < epsilon, "Rotation mismatch at index {}", i);

            assert_eq!(s.color.a, v.color.a, "Color A mismatch at index {}", i);
            assert_eq!(s.color.r, v.color.r, "Color R mismatch at index {}", i);
            assert_eq!(s.color.g, v.color.g, "Color G mismatch at index {}", i);
            assert_eq!(s.color.b, v.color.b, "Color B mismatch at index {}", i);
        }
    }
}
