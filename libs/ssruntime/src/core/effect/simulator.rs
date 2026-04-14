use super::types::*;
use super::state::*;

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

fn blend_number(a: u8, b: u8, rate: f32) -> u8 {
    (a as f32 + (b as i32 - a as i32) as f32 * rate) as u8
}

fn blend_float(a: f32, b: f32, rate: f32) -> f32 {
    a + (b - a) * rate
}

pub struct EffectSimulator;

impl EffectSimulator {
    /// エミッターの初期化・事前計算ロジック
    /// C++の precalculate2 に相当。放出パターンとオフセットを生成する。
    pub fn precalculate_emitter(
        emitter_params: &EmitterParameter,
        particle_params: &ParticleParameter,
        emitter_seed: i32,
    ) -> (Vec<EmitPattern>, Vec<i32>, Vec<u32>) {
        let mut rand_state = emitter_seed as u32;
        let mut gen_rand = || {
            rand_state ^= rand_state << 13;
            rand_state ^= rand_state >> 17;
            rand_state ^= rand_state << 5;
            (rand_state as f32) / core::u32::MAX as f32
        };

        let mut emit_patterns = Vec::new();
        let mut offset_patterns = Vec::new();

        if emitter_params.emitnum < 1 {
            // 安全策。本来はデータバリデーションで弾くべき
        }
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
            offset_patterns.push_back(offset);
            shot += 1;
        }

        for i in 0..extendsize {
            let life = emitter_params.particle_life + (emitter_params.particle_life2 as f32 * gen_rand()) as i32;
            let mut p_cycle = cycle;
            if life > cycle {
                p_cycle = life;
            }
            emit_patterns.push(EmitPattern {
                uid: i,
                life,
                cycle: p_cycle,
            });
        }

        // シードテーブルの作成
        let seed_table_len = (emitter_params.emitmax as usize) * 3;
        let mut seed_list = Vec::with_capacity(seed_table_len);
        for _ in 0..seed_table_len {
            rand_state ^= rand_state << 13;
            rand_state ^= rand_state >> 17;
            rand_state ^= rand_state << 5;
            seed_list.push(rand_state);
        }

        (emit_patterns, offset_patterns, seed_list)
    }

    /// 毎フレームのエミッター更新
    /// 現在の時間に基づいてパーティクルの生存状態を計算し、SoA構造体に書き込む
    pub fn update_emitter(
        state: &mut EmitterState,
        emitter_params: &EmitterParameter,
        emit_patterns: &[EmitPattern],
        offset_patterns: &[i32],
        current_time: f32,
        slide: i32,
    ) {
        let onum = offset_patterns.len();
        let pnum = emit_patterns.len();
        if pnum == 0 { return; }

        let slide_offset = slide * SEED_MAGIC;

        for i in 0..onum {
            let slide_num = (i as i32 + slide_offset).rem_euclid(pnum as i32) as usize;
            let target_ep = &emit_patterns[slide_num];
            let t = current_time - offset_patterns[i] as f32;

            state.particle_state.exist[i] = false;
            state.particle_state.born[i] = false;

            if target_ep.cycle != 0 {
                let loopnum = libm::floorf(t / target_ep.cycle as f32) as i32;
                let cycle_top = loopnum * target_ep.cycle;

                state.particle_state.cycle[i] = loopnum;
                state.particle_state.stime[i] = cycle_top + offset_patterns[i];
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

    /// パーティクル単体の描画状態計算
    pub fn calculate_state(
        current_time: f32,
        particle_stime: i32,
        particle_endtime: i32,
        particle_id: i32,
        emitter_params: &EmitterParameter,
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
            return ParticleDrawState {
                x: 0.0, y: 0.0, scale_x: 1.0, scale_y: 1.0, rotation: 0.0, direction: 0.0,
                color: Color::new(255, 255, 255, 255),
            };
        }

        let lifeper = t / life;
        let seed_table_len = seed_list.len();
        let pseed = seed_list[particle_id as usize % seed_table_len];
        let seed = (pseed as u64 + emitter_seed as u64 + pid as u64 + seed_offset as u64) as u32;

        // XorShift32 (Rust実装)
        let mut rand_state = seed;
        let mut gen_rand = || {
            rand_state ^= rand_state << 13;
            rand_state ^= rand_state >> 17;
            rand_state ^= rand_state << 5;
            (rand_state as f32) / core::u32::MAX as f32
        };

        // 1. Position and Speed
        let mut out_x = 0.0;
        let mut out_y = 0.0;
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

            let mut x = libm::cosf(rad + addr) * speed * t;
            let mut y = libm::sinf(rad + addr) * speed * t;

            if particle_params.use_trans_speed {
                let transspeed = particle_params.trans_speed + (particle_params.trans_speed2 * gen_rand());
                let speedadd = transspeed / life;
                let addtx = libm::cosf(rad + addr) * speed;
                let addtx_trans = libm::cosf(rad + addr) * speedadd;
                x = ((addtx_trans * t) + addtx) * (t + 1.0) / 2.0;

                let addty = libm::sinf(rad + addr) * speed;
                let addty_trans = libm::sinf(rad + addr) * speedadd;
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

        // 2. Offset
        let mut ox = 0.0;
        let mut oy = 0.0;
        if particle_params.use_offset {
            ox = particle_params.offset.x + (particle_params.offset2.x * gen_rand());
            oy = particle_params.offset.y + (particle_params.offset2.y * gen_rand());
        }

        // 3. Rotation
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

        // 4. Color
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

        // 5. Scale
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

        // 6. Final World Position
        let mut final_x = out_x + ox + emitter_pos.x;
        let mut final_y = out_y + oy + emitter_pos.y;

        if particle_params.use_p_gravity {
            let vx = particle_params.gravity_pos.x - (ox + emitter_pos.x);
            let vy = particle_params.gravity_pos.y - (oy + emitter_pos.y);
            let dist = libm::sqrtf(vx * vx + vy * vy);
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

        // 7. Turn to Direction
        let direction = 0.0;

        ParticleDrawState {
            x: final_x,
            y: final_y,
            scale_x: out_scale_x,
            scale_y: out_scale_y,
            rotation,
            direction,
            color,
        }
    }
}
