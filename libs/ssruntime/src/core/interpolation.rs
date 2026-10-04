use crate::core::keyframes::{Key, KeyColumns, KeyRun};
use crate::interpolate::*;
use crate::runtime;
use alloc::vec::Vec;
use ssab::ssab::ss::format::InterpolationType;

const UPDATE_FLAG_ON: runtime::framedata::ss::runtime::UpdateAttributeFlags = runtime::framedata::ss::runtime::UpdateAttributeFlags::all();

#[inline]
pub fn get_interpolated_rate(interpolation: InterpolationType, frame_no: f32, prev_frame: f32, next_frame: f32, curve: Option<&ssab::ssab::ss::format::KeyCurve>, easing_rate: f32) -> f32 {
    let rate = rate_linear(frame_no, prev_frame, next_frame);
    match interpolation {
        InterpolationType::None => 0.0,
        InterpolationType::Linear => rate,
        InterpolationType::Hermite => {
            if let Some(c) = curve {
                hermite(0.0_f32, 1.0_f32, rate, c.start_value(), c.end_value())
            } else {
                rate
            }
        }
        InterpolationType::Bezier => {
            if let Some(c) = curve {
                bezier(prev_frame, 0.0_f32, next_frame, 1.0_f32, rate, c.start_time(), c.start_value(), c.end_time(), c.end_value())
            } else {
                rate
            }
        }
        InterpolationType::Acceleration => rate_accelerate(rate),
        InterpolationType::Deceleration => rate_decelerate(rate),
        InterpolationType::EaseIn => rate_ease_in(rate, easing_rate),
        InterpolationType::EaseOut => rate_ease_out(rate, easing_rate),
        InterpolationType::EaseInOut => rate_ease_inout(rate, easing_rate),
        InterpolationType::EaseExponentialIn => rate_ease_exponential_in(rate),
        InterpolationType::EaseExponentialOut => rate_ease_exponential_out(rate),
        InterpolationType::EaseExponentialInOut => rate_ease_exponential_inout(rate),
        InterpolationType::EaseSineIn => rate_ease_sine_in(rate),
        InterpolationType::EaseSineOut => rate_ease_sine_out(rate),
        InterpolationType::EaseSineInOut => rate_ease_sine_inout(rate),
        InterpolationType::EaseElasticIn => rate_ease_elastic_in(rate, easing_rate),
        InterpolationType::EaseElasticOut => rate_ease_elastic_out(rate, easing_rate),
        InterpolationType::EaseElasticInOut => rate_ease_elastic_inout(rate, easing_rate),
        InterpolationType::EaseBounceIn => rate_ease_bounce_in(rate),
        InterpolationType::EaseBounceOut => rate_ease_bounce_out(rate),
        InterpolationType::EaseBounceInOut => rate_ease_bounce_inout(rate),
        InterpolationType::EaseBackIn => rate_ease_back_in(rate),
        InterpolationType::EaseBackOut => rate_ease_back_out(rate),
        InterpolationType::EaseBackInOut => rate_ease_back_inout(rate),
        _ => rate,
    }
}

// Compute the interpolated value directly in (frame, value)-space for single-value attributes.
//
// Why: for Bezier/Hermite the curve handle Y values from `KeyCurve` are in attribute-value units
// (e.g. pixels). Mapping the Y axis to [0,1] rate space and feeding those handles unchanged
// loses both magnitude and sign of the handle relative to (next_value - prev_value), which makes
// e.g. a Y position interpolation curve flip direction when the value range is descending.
// For other interpolation types the rate is value-independent, so we use the existing rate path.
#[inline]
fn evaluate_curve_value_f32(interpolation: InterpolationType, frame_no: f32, prev_frame: f32, next_frame: f32, prev_value: f32, next_value: f32, curve: Option<&ssab::ssab::ss::format::KeyCurve>, easing_rate: f32) -> f32 {
    match interpolation {
        InterpolationType::Hermite => {
            let rate = rate_linear(frame_no, prev_frame, next_frame);
            if let Some(c) = curve { hermite(prev_value, next_value, rate, c.start_value(), c.end_value()) } else { linear(prev_value, next_value, rate) }
        }
        InterpolationType::Bezier => {
            let rate = rate_linear(frame_no, prev_frame, next_frame);
            if let Some(c) = curve { bezier(prev_frame, prev_value, next_frame, next_value, rate, c.start_time(), c.start_value(), c.end_time(), c.end_value()) } else { linear(prev_value, next_value, rate) }
        }
        _ => {
            let rate = get_interpolated_rate(interpolation, frame_no, prev_frame, next_frame, curve, easing_rate);
            linear(prev_value, next_value, rate)
        }
    }
}

// Optimized for both forward and reverse playback (O(1) fast path).
// Long distance seeks will fallback to binary search (O(log N)).
// `hint` counts keys from the start of `run`.
#[inline]
pub fn search_prev_next<'c, 'a>(keys: &'c KeyColumns<'a>, run: &KeyRun, frame_no: f32, hint: &mut usize) -> Option<(Key<'c, 'a>, Option<Key<'c, 'a>>)> {
    let first = run.first() as usize;
    let len = run.count() as usize;
    if len == 0 {
        return None;
    }
    let get = |i: usize| keys.key(first + i);

    let mut i = *hint;
    if i >= len {
        i = len - 1;
    }

    let mut curr_val = get(i);
    let curr_idx = curr_val.index() as i32;

    if (curr_idx as f32) <= frame_no {
        let mut checks = 0;
        while i + 1 < len && checks < 4 {
            let next_val = get(i + 1);
            let next_idx = next_val.index() as i32;
            if frame_no < next_idx as f32 {
                *hint = i;
                return Some((curr_val, Some(next_val)));
            }
            i += 1;
            curr_val = next_val;
            checks += 1;
        }
        if i + 1 == len {
            *hint = i;
            return Some((curr_val, None));
        }
    } else {
        let mut checks = 0;
        let mut local_i = i;
        while local_i > 0 && checks < 4 {
            local_i -= 1;
            let prev_val = get(local_i);
            let prev_idx = prev_val.index() as i32;
            if (prev_idx as f32) <= frame_no {
                *hint = local_i;
                return Some((prev_val, Some(get(local_i + 1))));
            }
            checks += 1;
        }
    }

    let mut left = 0;
    let mut right = len;
    while left < right {
        let mid = left + (right - left) / 2;
        if (get(mid).index() as i32 as f32) <= frame_no {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    let res_idx = if left == 0 { 0 } else { left - 1 };
    *hint = res_idx;

    if left == 0 {
        return Some((get(0), None));
    }

    let prev = get(left - 1);
    if (prev.index() as i32 as f32) == frame_no || left == len {
        return Some((prev, None));
    }

    let next = get(left);
    Some((prev, Some(next)))
}

// Optimized for both forward and reverse playback (O(1) fast path).
// Long distance seeks will fallback to binary search (O(log N)).
// `hint` counts keys from the start of `run`.
#[inline]
pub fn search_prev<'c, 'a>(keys: &'c KeyColumns<'a>, run: &KeyRun, frame_no: i32, hint: &mut usize) -> Option<Key<'c, 'a>> {
    let first = run.first() as usize;
    let len = run.count() as usize;
    if len == 0 {
        return None;
    }
    let get = |i: usize| keys.key(first + i);

    let mut i = *hint;
    if i >= len {
        i = len - 1;
    }

    let mut curr_val = get(i);
    let curr_idx = curr_val.index() as i32;

    if curr_idx <= frame_no {
        let mut checks = 0;
        while i + 1 < len && checks < 4 {
            let next_val = get(i + 1);
            let next_idx = next_val.index() as i32;
            if frame_no < next_idx {
                *hint = i;
                return Some(curr_val);
            }
            i += 1;
            curr_val = next_val;
            checks += 1;
        }
        if i + 1 == len {
            *hint = i;
            return Some(curr_val);
        }
    } else {
        let mut checks = 0;
        let mut local_i = i;
        while local_i > 0 && checks < 4 {
            local_i -= 1;
            let prev_val = get(local_i);
            let prev_idx = prev_val.index() as i32;
            if prev_idx <= frame_no {
                *hint = local_i;
                return Some(prev_val);
            }
            checks += 1;
        }
    }

    let mut left = 0;
    let mut right = len;
    while left < right {
        let mid = left + (right - left) / 2;
        if (get(mid).index() as i32) <= frame_no {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    let res_idx = if left == 0 { 0 } else { left - 1 };
    *hint = res_idx;

    if left == 0 { None } else { Some(get(left - 1)) }
}

#[inline]
pub fn evaluate_float(keys: &KeyColumns<'_>, run: &KeyRun, frame_no: f32, hint: &mut usize) -> Option<(f32, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(keys, run, frame_no, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32 as f32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                return (prev.float(), UPDATE_FLAG_ON);
            }

            let next_index = next.index() as i32 as f32;
            let value = evaluate_curve_value_f32(interpolation, frame_no, prev_index, next_index, prev.float(), next.float(), prev.curve(), prev.easing_rate());

            (value, UPDATE_FLAG_ON)
        } else {
            (prev.float(), UPDATE_FLAG_ON)
        }
    })
}

#[inline]
pub fn evaluate_int(keys: &KeyColumns<'_>, run: &KeyRun, frame_no: f32, hint: &mut usize) -> Option<(i32, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(keys, run, frame_no, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32 as f32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                return (prev.int(), UPDATE_FLAG_ON);
            }

            let next_index = next.index() as i32 as f32;
            let value = evaluate_curve_value_f32(interpolation, frame_no, prev_index, next_index, prev.int() as f32, next.int() as f32, prev.curve(), prev.easing_rate());

            (crate::core::math::roundf(value) as i32, UPDATE_FLAG_ON)
        } else {
            (prev.int(), UPDATE_FLAG_ON)
        }
    })
}

#[inline]
pub fn evaluate_partcolor(keys: &KeyColumns<'_>, run: &KeyRun, frame_no: f32, hint: &mut usize) -> Option<(runtime::framedata::ss::runtime::PartAttributePartColor, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(keys, run, frame_no, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32 as f32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                return (transfer_partcolor(&prev.part_color()), UPDATE_FLAG_ON);
            }

            let next_index = next.index() as i32 as f32;
            let rate = get_interpolated_rate(interpolation, frame_no, prev_index, next_index, prev.curve(), prev.easing_rate());

            let mut current = runtime::framedata::ss::runtime::PartAttributePartColor::default();
            let prev_val = prev.part_color();
            let next_val = next.part_color();

            let get_color = |c: &Option<ssab::ssab::ss::format::SsAttributePartColorKeyValueColor>| -> (f32, f32, f32, f32, f32) {
                if let Some(c) = c
                    && let Some(rgba) = c.rgba()
                {
                    return (rgba.a() as f32, rgba.r() as f32, rgba.g() as f32, rgba.b() as f32, c.rate());
                }
                (0.0, 0.0, 0.0, 0.0, 0.0)
            };

            let p_lt = get_color(&prev_val.lt());
            let p_rt = get_color(&prev_val.rt());
            let p_lb = get_color(&prev_val.lb());
            let p_rb = get_color(&prev_val.rb());

            let n_lt = get_color(&next_val.lt());
            let n_rt = get_color(&next_val.rt());
            let n_lb = get_color(&next_val.lb());
            let n_rb = get_color(&next_val.rb());

            let start_color = [p_lt.0, p_lt.1, p_lt.2, p_lt.3, p_rt.0, p_rt.1, p_rt.2, p_rt.3, p_lb.0, p_lb.1, p_lb.2, p_lb.3, p_rb.0, p_rb.1, p_rb.2, p_rb.3];
            let end_color = [n_lt.0, n_lt.1, n_lt.2, n_lt.3, n_rt.0, n_rt.1, n_rt.2, n_rt.3, n_lb.0, n_lb.1, n_lb.2, n_lb.3, n_rb.0, n_rb.1, n_rb.2, n_rb.3];
            let mut out_color = [0.0; 16];
            interpolate_color_linear(&start_color, &end_color, rate, &mut out_color);

            let start_rate = [p_lt.4, p_rt.4, p_lb.4, p_rb.4];
            let end_rate = [n_lt.4, n_rt.4, n_lb.4, n_rb.4];
            let mut out_rate = [0.0; 4];
            interpolate_array_linear(&start_rate, &end_rate, rate, &mut out_rate);

            let create_color = |idx: usize| -> runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor {
                let base = idx * 4;
                runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&runtime::framedata::ss::runtime::U8Color::new(out_color[base] as u8, out_color[base + 1] as u8, out_color[base + 2] as u8, out_color[base + 3] as u8), out_rate[idx])
            };

            current.set_lt(&create_color(0));
            current.set_rt(&create_color(1));
            current.set_lb(&create_color(2));
            current.set_rb(&create_color(3));
            current.set_blend_type(transfer_blend_type(prev_val.blend_type()));

            (current, UPDATE_FLAG_ON)
        } else {
            (transfer_partcolor(&prev.part_color()), UPDATE_FLAG_ON)
        }
    })
}

#[inline]
fn transfer_blend_type(bt: ssab::ssab::ss::format::BlendType) -> runtime::framedata::ss::runtime::BlendType {
    match bt {
        ssab::ssab::ss::format::BlendType::Mix => runtime::framedata::ss::runtime::BlendType::Mix,
        ssab::ssab::ss::format::BlendType::Mul => runtime::framedata::ss::runtime::BlendType::Mul,
        ssab::ssab::ss::format::BlendType::Add => runtime::framedata::ss::runtime::BlendType::Add,
        ssab::ssab::ss::format::BlendType::Sub => runtime::framedata::ss::runtime::BlendType::Sub,
        ssab::ssab::ss::format::BlendType::Mulalpha => runtime::framedata::ss::runtime::BlendType::Mulalpha,
        ssab::ssab::ss::format::BlendType::Screen => runtime::framedata::ss::runtime::BlendType::Screen,
        ssab::ssab::ss::format::BlendType::Exclusion => runtime::framedata::ss::runtime::BlendType::Exclusion,
        ssab::ssab::ss::format::BlendType::Invert => runtime::framedata::ss::runtime::BlendType::Invert,
        ssab::ssab::ss::format::BlendType::Mul2 => runtime::framedata::ss::runtime::BlendType::Mul2,
        ssab::ssab::ss::format::BlendType::Div2 => runtime::framedata::ss::runtime::BlendType::Div2,
        ssab::ssab::ss::format::BlendType::Screen2 => runtime::framedata::ss::runtime::BlendType::Screen2,
        ssab::ssab::ss::format::BlendType::Overlay2 => runtime::framedata::ss::runtime::BlendType::Overlay2,
        _ => runtime::framedata::ss::runtime::BlendType::Mix,
    }
}

#[inline]
fn transfer_partcolor(source: &ssab::ssab::ss::format::PartAttributePartColor) -> runtime::framedata::ss::runtime::PartAttributePartColor {
    let mut destination = runtime::framedata::ss::runtime::PartAttributePartColor::default();
    let s = source;

    destination.set_blend_type(transfer_blend_type(s.blend_type()));

    fn convert_u8color(rgba: &ssab::ssab::ss::format::U8Color) -> runtime::framedata::ss::runtime::U8Color {
        runtime::framedata::ss::runtime::U8Color::new(rgba.a(), rgba.r(), rgba.g(), rgba.b())
    }

    if let Some(lt) = s.lt()
        && let Some(rgba) = lt.rgba()
    {
        destination.set_lt(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8color(&rgba), lt.rate()));
    }
    if let Some(rt) = s.rt()
        && let Some(rgba) = rt.rgba()
    {
        destination.set_rt(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8color(&rgba), rt.rate()));
    }
    if let Some(rb) = s.rb()
        && let Some(rgba) = rb.rgba()
    {
        destination.set_rb(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8color(&rgba), rb.rate()));
    }
    if let Some(lb) = s.lb()
        && let Some(rgba) = lb.rgba()
    {
        destination.set_lb(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8color(&rgba), lb.rate()));
    }

    destination
}

#[inline]
pub fn evaluate_shader(keys: &KeyColumns<'_>, run: &KeyRun, frame_no: f32, hint: &mut usize) -> Option<(runtime::framedata::ss::runtime::PartAttributeShader, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(keys, run, frame_no, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32 as f32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                return (transfer_shader(&prev.shader()), UPDATE_FLAG_ON);
            }

            let next_index = next.index() as i32 as f32;
            let rate = get_interpolated_rate(interpolation, frame_no, prev_index, next_index, prev.curve(), prev.easing_rate());

            let prev_value = prev.shader();
            let next_value = next.shader();

            let start_params = [prev_value.param0(), prev_value.param1(), prev_value.param2(), prev_value.param3(), prev_value.param4(), prev_value.param5(), prev_value.param6(), prev_value.param7()];
            let end_params = [next_value.param0(), next_value.param1(), next_value.param2(), next_value.param3(), next_value.param4(), next_value.param5(), next_value.param6(), next_value.param7()];
            let mut out_params = [0.0f32; 8];
            interpolate_array_linear(&start_params, &end_params, rate, &mut out_params);

            (
                runtime::framedata::ss::runtime::PartAttributeShader::new(
                    prev_value.id_hash(),
                    out_params[0],
                    out_params[1],
                    out_params[2],
                    out_params[3],
                    out_params[4],
                    out_params[5],
                    out_params[6],
                    out_params[7],
                    prev_value.map0_cellmap_name_hash(),
                    prev_value.map0_name_hash(),
                    prev_value.map1_cellmap_name_hash(),
                    prev_value.map1_name_hash(),
                ),
                UPDATE_FLAG_ON,
            )
        } else {
            (transfer_shader(&prev.shader()), UPDATE_FLAG_ON)
        }
    })
}

#[inline]
fn transfer_shader(source: &ssab::ssab::ss::format::PartAttributeShader) -> runtime::framedata::ss::runtime::PartAttributeShader {
    let s = source;

    runtime::framedata::ss::runtime::PartAttributeShader::new(
        s.id_hash(),
        s.param0(),
        s.param1(),
        s.param2(),
        s.param3(),
        s.param4(),
        s.param5(),
        s.param6(),
        s.param7(),
        s.map0_cellmap_name_hash(),
        s.map0_name_hash(),
        s.map1_cellmap_name_hash(),
        s.map1_name_hash(),
    )
}

#[inline]
pub fn evaluate_vertex(keys: &KeyColumns<'_>, run: &KeyRun, frame_no: f32, hint: &mut usize) -> Option<(runtime::framedata::ss::runtime::PartAttributeVertex, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(keys, run, frame_no, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32 as f32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                return (transfer_vertex(&prev.vertex()), UPDATE_FLAG_ON);
            }

            let next_index = next.index() as i32 as f32;
            let rate = get_interpolated_rate(interpolation, frame_no, prev_index, next_index, prev.curve(), prev.easing_rate());

            let prev_value = prev.vertex();
            let next_value = next.vertex();

            let get_coords = |v: &Option<ssab::ssab::ss::format::Vec2>| -> (f32, f32) { if let Some(v) = v { (v.v1(), v.v2()) } else { (0.0, 0.0) } };

            let p_lt = get_coords(&prev_value.lt());
            let p_rt = get_coords(&prev_value.rt());
            let p_lb = get_coords(&prev_value.lb());
            let p_rb = get_coords(&prev_value.rb());

            let n_lt = get_coords(&next_value.lt());
            let n_rt = get_coords(&next_value.rt());
            let n_lb = get_coords(&next_value.lb());
            let n_rb = get_coords(&next_value.rb());

            let start_arr = [p_lt.0, p_lt.1, p_rt.0, p_rt.1, p_lb.0, p_lb.1, p_rb.0, p_rb.1];
            let end_arr = [n_lt.0, n_lt.1, n_rt.0, n_rt.1, n_lb.0, n_lb.1, n_rb.0, n_rb.1];
            let mut out_arr = [0.0; 8];
            interpolate_vertex_linear(&start_arr, &end_arr, rate, &mut out_arr);

            let r_v1 = [out_arr[0], out_arr[2], out_arr[4], out_arr[6]];
            let r_v2 = [out_arr[1], out_arr[3], out_arr[5], out_arr[7]];

            // The center vertex is filled later by get_parts after pivot/size/flip/deform are applied.
            (
                runtime::framedata::ss::runtime::PartAttributeVertex::new(
                    &runtime::framedata::ss::runtime::Vec2::new(r_v1[0], r_v2[0]),
                    &runtime::framedata::ss::runtime::Vec2::new(r_v1[1], r_v2[1]),
                    &runtime::framedata::ss::runtime::Vec2::new(r_v1[2], r_v2[2]),
                    &runtime::framedata::ss::runtime::Vec2::new(r_v1[3], r_v2[3]),
                    &runtime::framedata::ss::runtime::Vec2::default(),
                ),
                UPDATE_FLAG_ON,
            )
        } else {
            (transfer_vertex(&prev.vertex()), UPDATE_FLAG_ON)
        }
    })
}

#[inline]
fn transfer_vertex(source: &ssab::ssab::ss::format::PartAttributeVertex) -> runtime::framedata::ss::runtime::PartAttributeVertex {
    let s = source;

    let lt = s.lt().map(|lt| runtime::framedata::ss::runtime::Vec2::new(lt.v1(), lt.v2())).unwrap_or_default();
    let rt = s.rt().map(|rt| runtime::framedata::ss::runtime::Vec2::new(rt.v1(), rt.v2())).unwrap_or_default();
    let lb = s.lb().map(|lb| runtime::framedata::ss::runtime::Vec2::new(lb.v1(), lb.v2())).unwrap_or_default();
    let rb = s.rb().map(|rb| runtime::framedata::ss::runtime::Vec2::new(rb.v1(), rb.v2())).unwrap_or_default();

    // The center vertex is filled later by get_parts after pivot/size/flip/deform are applied.
    runtime::framedata::ss::runtime::PartAttributeVertex::new(&lt, &rt, &lb, &rb, &runtime::framedata::ss::runtime::Vec2::default())
}

#[inline]
pub fn evaluate_skew(keys: &KeyColumns<'_>, run: &KeyRun, frame_no: f32, hint: &mut usize) -> Option<(f32, f32, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(keys, run, frame_no, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_value = prev.skew();
            let prev_index = prev.index() as i32 as f32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                return (prev_value.v1(), prev_value.v2(), UPDATE_FLAG_ON);
            }

            let next_index = next.index() as i32 as f32;
            let rate = get_interpolated_rate(interpolation, frame_no, prev_index, next_index, prev.curve(), prev.easing_rate());

            let next_value = next.skew();
            (linear(prev_value.v1(), next_value.v1(), rate), linear(prev_value.v2(), next_value.v2(), rate), UPDATE_FLAG_ON)
        } else {
            let prev_value = prev.skew();
            (prev_value.v1(), prev_value.v2(), UPDATE_FLAG_ON)
        }
    })
}

/// Evaluates a deform attribute at `frame_no`, writing the interpolated
/// coordinates into `coord_out` and the (non-interpolated) indices into
/// `index_out`. Both output buffers are cleared at entry, so the caller
/// can reuse them across parts without manual reset.
///
/// Returns `Some(flags)` if a key was found at/before `frame_no`,
/// carrying the same update semantics as the scalar `evaluate_*` helpers.
/// Deform values are required on every key, so a found key always reports
/// `UPDATE_FLAG_ON`.
#[inline]
pub fn evaluate_deform(
    keys: &KeyColumns<'_>,
    run: &KeyRun,
    frame_no: f32,
    hint: &mut usize,
    start_workspace: &mut Vec<f32>,
    end_workspace: &mut Vec<f32>,
    out_workspace: &mut Vec<f32>,
    coord_out: &mut Vec<runtime::framedata::ss::runtime::Vec2>,
    index_out: &mut Vec<i16>,
) -> Option<runtime::framedata::ss::runtime::UpdateAttributeFlags> {
    coord_out.clear();
    index_out.clear();
    search_prev_next(keys, run, frame_no, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32 as f32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                transfer_deform(&prev.deform(), coord_out, index_out);
                return UPDATE_FLAG_ON;
            }

            let next_index = next.index() as i32 as f32;
            let rate = get_interpolated_rate(interpolation, frame_no, prev_index, next_index, prev.curve(), prev.easing_rate());

            let prev_val = prev.deform();
            let next_val = next.deform();

            if let (Some(prev_cx), Some(prev_cy), Some(next_cx), Some(next_cy), Some(prev_idx), Some(next_idx)) = (prev_val.coord_x(), prev_val.coord_y(), next_val.coord_x(), next_val.coord_y(), prev_val.index(), next_val.index()) {
                // Deform keyframes store a sparse, vertex-keyed set of offsets:
                // `index[k]` is the changed vertex and `coord_x[k]/coord_y[k]`
                // its offset. The changed-vertex set differs between keyframes
                // (e.g. an empty rest keyframe vs a deformed one), so the two
                // sides cannot be interpolated position-by-position. Merge by
                // vertex index instead — a vertex absent on one side
                // interpolates against a zero offset. Authored index arrays
                // are ascending, so a two-pointer merge keeps the output
                // ascending and allocation-free.
                let prev_n = prev_cx.len().min(prev_cy.len());
                let next_n = next_cx.len().min(next_cy.len());
                let prev_len = core::cmp::min(prev_idx.len(), prev_n);
                let next_len = core::cmp::min(next_idx.len(), next_n);

                start_workspace.clear();
                end_workspace.clear();
                index_out.reserve(prev_len + next_len);

                let mut i = 0usize;
                let mut j = 0usize;
                while i < prev_len || j < next_len {
                    let pv = if i < prev_len { prev_idx.get(i) } else { i16::MAX };
                    let nv = if j < next_len { next_idx.get(j) } else { i16::MAX };
                    if pv == nv {
                        start_workspace.push(prev_cx.get(i));
                        start_workspace.push(prev_cy.get(i));
                        end_workspace.push(next_cx.get(j));
                        end_workspace.push(next_cy.get(j));
                        index_out.push(pv);
                        i += 1;
                        j += 1;
                    } else if pv < nv {
                        start_workspace.push(prev_cx.get(i));
                        start_workspace.push(prev_cy.get(i));
                        end_workspace.push(0.0);
                        end_workspace.push(0.0);
                        index_out.push(pv);
                        i += 1;
                    } else {
                        start_workspace.push(0.0);
                        start_workspace.push(0.0);
                        end_workspace.push(next_cx.get(j));
                        end_workspace.push(next_cy.get(j));
                        index_out.push(nv);
                        j += 1;
                    }
                }

                out_workspace.clear();
                out_workspace.resize(start_workspace.len(), 0.0);
                interpolate_array_linear(start_workspace, end_workspace, rate, out_workspace);

                coord_out.reserve(index_out.len());
                for k in 0..index_out.len() {
                    coord_out.push(runtime::framedata::ss::runtime::Vec2::new(out_workspace[k * 2], out_workspace[k * 2 + 1]));
                }

                UPDATE_FLAG_ON
            } else if let (Some(prev_cx), Some(prev_cy), Some(next_cx), Some(next_cy)) = (prev_val.coord_x(), prev_val.coord_y(), next_val.coord_x(), next_val.coord_y()) {
                // No per-vertex index array on one or both keyframes: the
                // coordinate arrays are positionally aligned, so interpolate
                // vertex-by-vertex. When prev is longer than next, its
                // trailing vertices pass through unchanged.
                let prev_n = prev_cx.len().min(prev_cy.len());
                let next_n = next_cx.len().min(next_cy.len());
                let length_full = prev_n;
                let length = core::cmp::min(prev_n, next_n);

                start_workspace.clear();
                end_workspace.clear();
                for i in 0..length {
                    start_workspace.push(prev_cx.get(i));
                    start_workspace.push(prev_cy.get(i));
                    end_workspace.push(next_cx.get(i));
                    end_workspace.push(next_cy.get(i));
                }

                out_workspace.clear();
                out_workspace.resize(length * 2, 0.0);
                interpolate_array_linear(start_workspace, end_workspace, rate, out_workspace);

                coord_out.reserve(length_full);
                for i in 0..length {
                    coord_out.push(runtime::framedata::ss::runtime::Vec2::new(out_workspace[i * 2], out_workspace[i * 2 + 1]));
                }
                for i in length..length_full {
                    coord_out.push(runtime::framedata::ss::runtime::Vec2::new(prev_cx.get(i), prev_cy.get(i)));
                }
                if let Some(index) = prev_val.index() {
                    index_out.extend(index.iter());
                }

                UPDATE_FLAG_ON
            } else {
                transfer_deform(&prev.deform(), coord_out, index_out);
                UPDATE_FLAG_ON
            }
        } else {
            transfer_deform(&prev.deform(), coord_out, index_out);
            UPDATE_FLAG_ON
        }
    })
}

#[inline]
fn transfer_deform(source: &ssab::ssab::ss::format::PartAttributeDeform, coord_out: &mut Vec<runtime::framedata::ss::runtime::Vec2>, index_out: &mut Vec<i16>) {
    let s = source;

    if let (Some(cx), Some(cy)) = (s.coord_x(), s.coord_y()) {
        let n = cx.len().min(cy.len());
        coord_out.reserve(n);
        for i in 0..n {
            coord_out.push(runtime::framedata::ss::runtime::Vec2::new(cx.get(i), cy.get(i)));
        }
    }
    if let Some(index) = s.index() {
        index_out.reserve(index.len());
        index_out.extend(index.iter());
    }
}

#[inline]
pub fn evaluate_bool(keys: &KeyColumns<'_>, run: &KeyRun, frame_no: f32, hint: &mut usize) -> Option<bool> {
    let frame_no = crate::core::math::floorf(frame_no) as i32;
    search_prev(keys, run, frame_no, hint).map(|prev| prev.bool())
}

#[inline]
pub fn evaluate_cell<'a>(keys: &KeyColumns<'a>, run: &KeyRun, frame_no: f32, hint: &mut usize) -> Option<ssab::ssab::ss::format::PartAttributeCell<'a>> {
    let frame_no = crate::core::math::floorf(frame_no) as i32;
    search_prev(keys, run, frame_no, hint).map(|prev| prev.cell())
}

#[inline]
pub fn evaluate_texture(keys: &KeyColumns<'_>, run: &KeyRun, frame_no: f32, hint: &mut usize) -> Option<u32> {
    let frame_no = crate::core::math::floorf(frame_no) as i32;
    search_prev(keys, run, frame_no, hint).map(|prev| prev.uint())
}

#[cfg(test)]
mod tests {
    use super::*;
    use flatbuffers::{FlatBufferBuilder, WIPOffset};
    use ssab::ssab::ss::format::{KeyCurve, Keyframes, KeyframesArgs, PartAttributeCell, PartAttributeDeform, PartAttributePartColor, PartAttributeShader, PartAttributeVertex, Vec2};

    use ssab::NO_CURVE;

    /// One key of a test pack: its frame, interpolation, value bits and
    /// `Keyframes.curve` slot.
    struct TestKey {
        frame: u16,
        interpolation: InterpolationType,
        value: u32,
        slot: u16,
    }

    fn key(frame: u16, interpolation: InterpolationType, value: u32) -> TestKey {
        TestKey { frame, interpolation, value, slot: NO_CURVE }
    }

    fn curved(frame: u16, interpolation: InterpolationType, value: u32, slot: u16) -> TestKey {
        TestKey { frame, interpolation, value, slot }
    }

    /// A pack whose keys are `keys`, with the given curve and easing tables and
    /// whatever value tables `values` fills in; the rest are present and empty.
    fn pack(keys: &[TestKey], curves: &[KeyCurve], easing_rates: &[f32], values: impl FnOnce(&mut FlatBufferBuilder<'static>, &mut KeyframesArgs<'static>)) -> alloc::vec::Vec<u8> {
        let mut fb = FlatBufferBuilder::new();
        let mut args = KeyframesArgs::default();
        values(&mut fb, &mut args);
        let frame: alloc::vec::Vec<u16> = keys.iter().map(|k| k.frame).collect();
        let interpolation: alloc::vec::Vec<InterpolationType> = keys.iter().map(|k| k.interpolation).collect();
        let curve: alloc::vec::Vec<u16> = keys.iter().map(|k| k.slot).collect();
        let value: alloc::vec::Vec<u32> = keys.iter().map(|k| k.value).collect();
        args.runs = Some(fb.create_vector(&[0u32, keys.len() as u32]));
        args.frame = Some(fb.create_vector(&frame));
        args.interpolation = Some(fb.create_vector(&interpolation));
        args.curve = Some(fb.create_vector(&curve));
        args.value = Some(fb.create_vector(&value));
        args.curves = Some(fb.create_vector(curves));
        args.easing_rates = Some(fb.create_vector(easing_rates));
        args.cells = args.cells.or_else(|| Some(fb.create_vector::<WIPOffset<PartAttributeCell>>(&[])));
        args.part_colors = args.part_colors.or_else(|| Some(fb.create_vector::<WIPOffset<PartAttributePartColor>>(&[])));
        args.shaders = args.shaders.or_else(|| Some(fb.create_vector::<WIPOffset<PartAttributeShader>>(&[])));
        args.vertices = args.vertices.or_else(|| Some(fb.create_vector::<WIPOffset<PartAttributeVertex>>(&[])));
        args.skews = args.skews.or_else(|| Some(fb.create_vector::<WIPOffset<Vec2>>(&[])));
        args.deforms = args.deforms.or_else(|| Some(fb.create_vector::<WIPOffset<PartAttributeDeform>>(&[])));
        let root = Keyframes::create(&mut fb, &args);
        fb.finish(root, None);
        fb.finished_data().to_vec()
    }

    fn columns(buf: &[u8]) -> KeyColumns<'_> {
        KeyColumns::new(flatbuffers::root::<Keyframes>(buf).unwrap())
    }

    /// The run of the first `count` keys.
    fn run(count: usize) -> KeyRun {
        KeyRun::new(0, count as u32)
    }

    fn f(value: f32) -> u32 {
        value.to_bits()
    }

    #[test]
    fn test_linear_interpolation() {
        assert_eq!(linear(0.0, 10.0, 0.5), 5.0);
        assert_eq!(linear(10.0, 20.0, 0.0), 10.0);
        assert_eq!(linear(10.0, 20.0, 1.0), 20.0);
    }

    #[test]
    fn test_evaluate_bool() {
        let buf = pack(&[key(0, InterpolationType::None, 1), key(10, InterpolationType::None, 0)], &[], &[], |_, _| {});
        let (keys, run) = (columns(&buf), run(2));

        let mut hint = 0;
        // Exact match
        assert_eq!(evaluate_bool(&keys, &run, 0.0, &mut hint), Some(true));
        // Interpolated (should use prev)
        assert_eq!(evaluate_bool(&keys, &run, 5.0, &mut hint), Some(true));
        // Exact match
        assert_eq!(evaluate_bool(&keys, &run, 10.0, &mut hint), Some(false));
        // Beyond last key
        assert_eq!(evaluate_bool(&keys, &run, 15.0, &mut hint), Some(false));
        // Before first key
        assert_eq!(evaluate_bool(&keys, &run, -1.0, &mut hint), None);
        // Fractional: floor() applied internally — 9.9 → 9 → still uses prev (true)
        assert_eq!(evaluate_bool(&keys, &run, 9.9, &mut hint), Some(true));
        // Fractional: 10.0 → 10 → false
        assert_eq!(evaluate_bool(&keys, &run, 10.0, &mut hint), Some(false));
    }

    #[test]
    fn test_evaluate_texture() {
        let buf = pack(&[key(0, InterpolationType::None, 100), key(10, InterpolationType::None, 200)], &[], &[], |_, _| {});
        let (keys, run) = (columns(&buf), run(2));

        let mut hint = 0;
        assert_eq!(evaluate_texture(&keys, &run, 0.0, &mut hint), Some(100));
        assert_eq!(evaluate_texture(&keys, &run, 5.0, &mut hint), Some(100));
        assert_eq!(evaluate_texture(&keys, &run, 10.0, &mut hint), Some(200));
        assert_eq!(evaluate_texture(&keys, &run, 15.0, &mut hint), Some(200));
        // Fractional: floor() applied internally — 9.99 → 9 → still 100
        assert_eq!(evaluate_texture(&keys, &run, 9.99, &mut hint), Some(100));
    }

    #[test]
    fn test_evaluate_int_no_interpolation() {
        let buf = pack(&[key(0, InterpolationType::None, 100), key(10, InterpolationType::None, 200)], &[], &[], |_, _| {});
        let (keys, run) = (columns(&buf), run(2));

        let mut hint = 0;
        // At frame 0, should be ON and 100
        let (val, flags) = evaluate_int(&keys, &run, 0.0, &mut hint).unwrap();
        assert_eq!(val, 100);
        assert!(!flags.is_empty());

        // At frame 5, the value comes from a keyframe (held from frame 0
        // because InterpolationType::None doesn't tween). Per the new flag
        // semantics — ON means "value is derived from animation data" — the
        // flag is ON. Cell-default fallbacks and Player render-path checks
        // both want this signal so they don't silently overwrite the held
        // animated value.
        let (val, flags) = evaluate_int(&keys, &run, 5.0, &mut hint).unwrap();
        assert_eq!(val, 100);
        assert!(!flags.is_empty());

        // At frame 10, should be ON and 200
        let (val, flags) = evaluate_int(&keys, &run, 10.0, &mut hint).unwrap();
        assert_eq!(val, 200);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_evaluate_int_linear_interpolation_fractional() {
        let buf = pack(&[key(0, InterpolationType::Linear, 0), key(10, InterpolationType::None, 100)], &[], &[], |_, _| {});
        let (keys, run) = (columns(&buf), run(2));

        // Fractional frame: 5.5 should yield round(linear(0,100,0.55)) == 55
        let mut hint = 0;
        let (val, flags) = evaluate_int(&keys, &run, 5.5, &mut hint).unwrap();
        assert_eq!(val, 55);
        assert!(!flags.is_empty());

        // Integer frame still works at the keyframe (rate=0.5 -> 50)
        let mut hint = 0;
        let (val, _) = evaluate_int(&keys, &run, 5.0, &mut hint).unwrap();
        assert_eq!(val, 50);
    }

    #[test]
    fn test_evaluate_float_linear_interpolation_fractional() {
        let buf = pack(&[key(0, InterpolationType::Linear, f(0.0)), key(10, InterpolationType::None, f(1.0))], &[], &[], |_, _| {});
        let (keys, run) = (columns(&buf), run(2));

        // Fractional 2.5 → rate 0.25
        let mut hint = 0;
        let (val, flags) = evaluate_float(&keys, &run, 2.5, &mut hint).unwrap();
        assert!((val - 0.25).abs() < 1e-6, "expected ~0.25, got {val}");
        assert!(!flags.is_empty());

        // Integer 5 → rate 0.5
        let mut hint = 0;
        let (val, _) = evaluate_float(&keys, &run, 5.0, &mut hint).unwrap();
        assert!((val - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_float_reads_only_its_own_run() {
        // Two runs side by side in one pack: each sees only its own keys.
        let buf = pack(&[key(0, InterpolationType::None, f(1.0)), key(10, InterpolationType::None, f(2.0)), key(0, InterpolationType::Linear, f(5.0)), key(4, InterpolationType::None, f(9.0))], &[], &[], |_, _| {});
        let keys = columns(&buf);
        let (first, second) = (KeyRun::new(0, 2), KeyRun::new(2, 2));

        let mut hint = 0;
        assert_eq!(evaluate_float(&keys, &first, 20.0, &mut hint).unwrap().0, 2.0);
        let mut hint = 0;
        assert_eq!(evaluate_float(&keys, &second, 2.0, &mut hint).unwrap().0, 7.0);
        let mut hint = 0;
        assert_eq!(evaluate_float(&keys, &second, 20.0, &mut hint).unwrap().0, 9.0);
        let mut hint = 0;
        assert_eq!(evaluate_float(&keys, &KeyRun::new(2, 0), 2.0, &mut hint), None);
    }

    #[test]
    fn test_evaluate_cell() {
        let buf = pack(&[key(0, InterpolationType::None, 0), key(10, InterpolationType::None, 1)], &[], &[], |fb, args| {
            let cell_a = PartAttributeCell::create(fb, &ssab::ssab::ss::format::PartAttributeCellArgs { cellmap_name_hash: 1, name_hash: 111 });
            let cell_b = PartAttributeCell::create(fb, &ssab::ssab::ss::format::PartAttributeCellArgs { cellmap_name_hash: 2, name_hash: 222 });
            args.cells = Some(fb.create_vector(&[cell_a, cell_b]));
        });
        let (keys, run) = (columns(&buf), run(2));

        let mut hint = 0;
        let res = evaluate_cell(&keys, &run, 5.0, &mut hint).unwrap();
        assert_eq!(res.cellmap_name_hash(), 1);
        assert_eq!(res.name_hash(), 111);

        let res = evaluate_cell(&keys, &run, 10.0, &mut hint).unwrap();
        assert_eq!(res.cellmap_name_hash(), 2);
        assert_eq!(res.name_hash(), 222);

        // Fractional: floor() applied internally — 9.999 → 9 → still cell A
        let mut hint = 0;
        let res = evaluate_cell(&keys, &run, 9.999, &mut hint).unwrap();
        assert_eq!(res.cellmap_name_hash(), 1);
    }

    #[test]
    fn test_evaluate_float_hermite() {
        // start_value/end_value are absolute target values for tangents, so
        // (0, 1) → tangent (s_start - v_start) = 0 at start, (s_end - v_end) = 0 at end.
        let curve = KeyCurve::new(0.0, 0.0, 0.0, 1.0);
        let buf = pack(&[curved(0, InterpolationType::Hermite, f(0.0), 0), key(10, InterpolationType::None, f(1.0))], &[curve], &[], |_, _| {});
        let (keys, run) = (columns(&buf), run(2));

        // Hermite with zero tangents at midpoint: rate = 3*0.5^2 - 2*0.5^3 = 0.5
        let mut hint = 0;
        let (val, _) = evaluate_float(&keys, &run, 5.0, &mut hint).unwrap();
        assert!((val - 0.5).abs() < 1e-6, "expected ~0.5, got {val}");

        // Sub-frame between integer keys: smooth Hermite curve
        let mut hint = 0;
        let (val_25, _) = evaluate_float(&keys, &run, 2.5, &mut hint).unwrap();
        let (val_3, _) = evaluate_float(&keys, &run, 3.0, &mut hint).unwrap();
        assert!(val_25 < val_3, "Hermite must be monotonically increasing on this segment");
        assert!(val_25 > 0.0 && val_25 < 1.0);
    }

    #[test]
    fn test_evaluate_float_ease_in() {
        let buf = pack(&[curved(0, InterpolationType::EaseIn, f(0.0), 0), key(10, InterpolationType::None, f(1.0))], &[], &[2.0], |_, _| {});
        let (keys, run) = (columns(&buf), run(2));

        // EaseIn with power 2 at rate 0.5: 0.5^2 = 0.25
        let mut hint = 0;
        let (val, _) = evaluate_float(&keys, &run, 5.0, &mut hint).unwrap();
        assert!((val - 0.25).abs() < 1e-5, "expected ~0.25, got {val}");

        // Sub-frame: smooth, not stepped
        let mut hint = 0;
        let (val_25, _) = evaluate_float(&keys, &run, 2.5, &mut hint).unwrap();
        let (val_3, _) = evaluate_float(&keys, &run, 3.0, &mut hint).unwrap();
        assert!(val_25 < val_3);
    }

    #[test]
    fn test_evaluate_skew_linear() {
        let buf = pack(&[key(0, InterpolationType::Linear, 0), key(10, InterpolationType::None, 1)], &[], &[], |fb, args| {
            let v0 = Vec2::create(fb, &ssab::ssab::ss::format::Vec2Args { v1: 0.0, v2: 0.0 });
            let v10 = Vec2::create(fb, &ssab::ssab::ss::format::Vec2Args { v1: 10.0, v2: 20.0 });
            args.skews = Some(fb.create_vector(&[v0, v10]));
        });
        let (keys, run) = (columns(&buf), run(2));

        // Midpoint: linear interpolation of both axes
        let mut hint = 0;
        let (val_x, val_y, flags) = evaluate_skew(&keys, &run, 5.0, &mut hint).unwrap();
        assert!((val_x - 5.0).abs() < 1e-6);
        assert!((val_y - 10.0).abs() < 1e-6);
        assert!(!flags.is_empty());

        // Sub-frame: smooth interpolation
        let mut hint = 0;
        let (val_x, val_y, _) = evaluate_skew(&keys, &run, 2.5, &mut hint).unwrap();
        assert!((val_x - 2.5).abs() < 1e-6);
        assert!((val_y - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_vertex_linear() {
        use ssab::ssab::ss::format::{PartAttributeVertexArgs, Vec2Args};
        let buf = pack(&[key(0, InterpolationType::Linear, 0), key(10, InterpolationType::None, 1)], &[], &[], |fb, args| {
            let lt0 = Vec2::create(fb, &Vec2Args { v1: 0.0, v2: 0.0 });
            let rt0 = Vec2::create(fb, &Vec2Args { v1: 0.0, v2: 0.0 });
            let lb0 = Vec2::create(fb, &Vec2Args { v1: 0.0, v2: 0.0 });
            let rb0 = Vec2::create(fb, &Vec2Args { v1: 0.0, v2: 0.0 });
            let v0 = PartAttributeVertex::create(fb, &PartAttributeVertexArgs { lt: Some(lt0), rt: Some(rt0), lb: Some(lb0), rb: Some(rb0) });

            let lt1 = Vec2::create(fb, &Vec2Args { v1: 10.0, v2: 10.0 });
            let rt1 = Vec2::create(fb, &Vec2Args { v1: -10.0, v2: 10.0 });
            let lb1 = Vec2::create(fb, &Vec2Args { v1: 10.0, v2: -10.0 });
            let rb1 = Vec2::create(fb, &Vec2Args { v1: -10.0, v2: -10.0 });
            let v10 = PartAttributeVertex::create(fb, &PartAttributeVertexArgs { lt: Some(lt1), rt: Some(rt1), lb: Some(lb1), rb: Some(rb1) });
            args.vertices = Some(fb.create_vector(&[v0, v10]));
        });
        let (keys, run) = (columns(&buf), run(2));

        // Midpoint: each corner linearly interpolated
        let mut hint = 0;
        let (val, _) = evaluate_vertex(&keys, &run, 5.0, &mut hint).unwrap();
        assert!((val.lt().x() - 5.0).abs() < 1e-6);
        assert!((val.rt().x() - -5.0).abs() < 1e-6);
        assert!((val.lb().y() - -5.0).abs() < 1e-6);
        assert!((val.rb().y() - -5.0).abs() < 1e-6);
    }

    // -- evaluate_deform tests --

    /// A pack of deform keys, one value each, with given index, interpolation,
    /// coordinates and indices.
    #[allow(clippy::type_complexity)]
    fn deform_buf(keys: &[(u16, InterpolationType, &[(f32, f32)], &[i16])]) -> alloc::vec::Vec<u8> {
        deform_buf_with_curves(keys, &[])
    }

    #[allow(clippy::type_complexity)]
    fn deform_buf_with_curves(keys: &[(u16, InterpolationType, &[(f32, f32)], &[i16])], curves: &[KeyCurve]) -> alloc::vec::Vec<u8> {
        use ssab::ssab::ss::format::PartAttributeDeformArgs;
        let test_keys: alloc::vec::Vec<TestKey> = keys.iter().enumerate().map(|(i, (idx, ip, _, _))| curved(*idx, *ip, i as u32, if curves.is_empty() || *ip == InterpolationType::None { NO_CURVE } else { 0 })).collect();
        pack(&test_keys, curves, &[], |fb, args| {
            let mut values = alloc::vec::Vec::new();
            for (_, _, coords, indices) in keys {
                let xs: alloc::vec::Vec<f32> = coords.iter().map(|(x, _)| *x).collect();
                let ys: alloc::vec::Vec<f32> = coords.iter().map(|(_, y)| *y).collect();
                let cx_off = if xs.is_empty() { None } else { Some(fb.create_vector(&xs)) };
                let cy_off = if ys.is_empty() { None } else { Some(fb.create_vector(&ys)) };
                let idx_off = if indices.is_empty() { None } else { Some(fb.create_vector(indices)) };
                values.push(PartAttributeDeform::create(fb, &PartAttributeDeformArgs { coord_x: cx_off, coord_y: cy_off, index: idx_off }));
            }
            args.deforms = Some(fb.create_vector(&values));
        })
    }

    /// Owning wrapper used only by tests so assertions can read back the
    /// produced coordinate / index buffers without having to declare and
    /// thread workspace `Vec`s through every call site.
    struct DeformOut {
        coordinate: alloc::vec::Vec<runtime::framedata::ss::runtime::Vec2>,
        index: alloc::vec::Vec<i16>,
    }

    fn eval_deform_owned(buf: &[u8], frame_no: f32, hint: &mut usize) -> Option<(DeformOut, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
        let keys = columns(buf);
        let run = run(keys_len(buf));
        let mut start = alloc::vec::Vec::new();
        let mut end = alloc::vec::Vec::new();
        let mut out = alloc::vec::Vec::new();
        let mut coordinate = alloc::vec::Vec::new();
        let mut index = alloc::vec::Vec::new();
        evaluate_deform(&keys, &run, frame_no, hint, &mut start, &mut end, &mut out, &mut coordinate, &mut index).map(|flags| (DeformOut { coordinate, index }, flags))
    }

    fn keys_len(buf: &[u8]) -> usize {
        flatbuffers::root::<Keyframes>(buf).unwrap().frame().len()
    }

    #[test]
    fn test_evaluate_deform_simd_chunk_aligned() {
        // 8 vertices = exactly 2 SIMD chunks, no tail.
        let prev: alloc::vec::Vec<(f32, f32)> = (0..8).map(|i| (i as f32, i as f32 * 2.0)).collect();
        let next: alloc::vec::Vec<(f32, f32)> = (0..8).map(|i| (i as f32 * 3.0, i as f32 * 4.0)).collect();
        let buf = deform_buf(&[(0, InterpolationType::Linear, &prev, &[]), (10, InterpolationType::None, &next, &[])]);

        // At rate=0.5, lerp(prev, next, 0.5) = (prev + next) / 2.
        let mut hint = 0;
        let (data, flags) = eval_deform_owned(&buf, 5.0, &mut hint).unwrap();
        assert_eq!(data.coordinate.len(), 8);
        assert!(!flags.is_empty());
        for i in 0..8 {
            let exp_x = (prev[i].0 + next[i].0) / 2.0;
            let exp_y = (prev[i].1 + next[i].1) / 2.0;
            assert!((data.coordinate[i].x() - exp_x).abs() < 1e-6, "x mismatch at {i}: got {} want {exp_x}", data.coordinate[i].x());
            assert!((data.coordinate[i].y() - exp_y).abs() < 1e-6, "y mismatch at {i}: got {} want {exp_y}", data.coordinate[i].y());
        }
    }

    #[test]
    fn test_evaluate_deform_simd_with_tail() {
        // 7 vertices = 1 SIMD chunk (4) + tail (3). Both branches must produce identical results.
        let prev: alloc::vec::Vec<(f32, f32)> = (0..7).map(|i| (i as f32, i as f32 * 10.0)).collect();
        let next: alloc::vec::Vec<(f32, f32)> = (0..7).map(|i| ((i as f32) + 100.0, (i as f32) * 10.0 + 1000.0)).collect();
        let buf = deform_buf(&[(0, InterpolationType::Linear, &prev, &[]), (10, InterpolationType::None, &next, &[])]);

        let mut hint = 0;
        let (data, _) = eval_deform_owned(&buf, 2.5, &mut hint).unwrap();
        assert_eq!(data.coordinate.len(), 7);
        // rate = 0.25
        for i in 0..7 {
            let exp_x = prev[i].0 + (next[i].0 - prev[i].0) * 0.25;
            let exp_y = prev[i].1 + (next[i].1 - prev[i].1) * 0.25;
            assert!((data.coordinate[i].x() - exp_x).abs() < 1e-4, "x mismatch at {i}: got {} want {exp_x}", data.coordinate[i].x());
            assert!((data.coordinate[i].y() - exp_y).abs() < 1e-4, "y mismatch at {i}: got {} want {exp_y}", data.coordinate[i].y());
        }
    }

    #[test]
    fn test_evaluate_deform_simd_only_tail() {
        // 3 vertices = 0 SIMD chunks, 3 tail. Exercises the scalar path.
        let prev = [(0.0, 0.0), (1.0, 2.0), (3.0, 4.0)];
        let next = [(10.0, 20.0), (11.0, 22.0), (13.0, 24.0)];
        let buf = deform_buf(&[(0, InterpolationType::Linear, &prev, &[]), (10, InterpolationType::None, &next, &[])]);

        let mut hint = 0;
        let (data, _) = eval_deform_owned(&buf, 5.0, &mut hint).unwrap();
        assert_eq!(data.coordinate.len(), 3);
        for i in 0..3 {
            let exp_x = (prev[i].0 + next[i].0) / 2.0;
            let exp_y = (prev[i].1 + next[i].1) / 2.0;
            assert!((data.coordinate[i].x() - exp_x).abs() < 1e-6);
            assert!((data.coordinate[i].y() - exp_y).abs() < 1e-6);
        }
    }

    #[test]
    fn test_evaluate_deform_overrun_prev_longer() {
        // prev has 6 vertices, next has 4. The 2 extras must come from prev as-is.
        let prev: alloc::vec::Vec<(f32, f32)> = (0..6).map(|i| (i as f32 + 100.0, i as f32 + 200.0)).collect();
        let next: alloc::vec::Vec<(f32, f32)> = (0..4).map(|i| (i as f32, i as f32)).collect();
        let buf = deform_buf(&[(0, InterpolationType::Linear, &prev, &[]), (10, InterpolationType::None, &next, &[])]);

        let mut hint = 0;
        let (data, _) = eval_deform_owned(&buf, 5.0, &mut hint).unwrap();
        assert_eq!(data.coordinate.len(), 6, "output length should match prev_full");
        // First 4 are interpolated.
        for i in 0..4 {
            let exp_x = (prev[i].0 + next[i].0) / 2.0;
            let exp_y = (prev[i].1 + next[i].1) / 2.0;
            assert!((data.coordinate[i].x() - exp_x).abs() < 1e-4, "interpolated x mismatch at {i}");
            assert!((data.coordinate[i].y() - exp_y).abs() < 1e-4, "interpolated y mismatch at {i}");
        }
        // Indices 4 and 5: passthrough from prev.
        for (i, p) in prev.iter().enumerate().take(6).skip(4) {
            assert_eq!(data.coordinate[i].x(), p.0, "overrun x at {i}");
            assert_eq!(data.coordinate[i].y(), p.1, "overrun y at {i}");
        }
    }

    #[test]
    fn test_evaluate_deform_none_at_keyframe() {
        // None interpolation at exact keyframe → flag ON, prev value.
        let prev = [(1.0, 2.0), (3.0, 4.0)];
        let next = [(10.0, 20.0), (30.0, 40.0)];
        let buf = deform_buf(&[(0, InterpolationType::None, &prev, &[]), (10, InterpolationType::None, &next, &[])]);

        let mut hint = 0;
        let (data, flags) = eval_deform_owned(&buf, 0.0, &mut hint).unwrap();
        assert!(!flags.is_empty(), "exact keyframe match must set ON");
        assert_eq!(data.coordinate[0].x(), prev[0].0);
        assert_eq!(data.coordinate[0].y(), prev[0].1);
    }

    #[test]
    fn test_evaluate_deform_none_off_keyframe() {
        // None interpolation off-keyframe: value is held from prev key. Per the
        // updated flag semantics — ON means "value is derived from animation
        // data" — the flag is ON. Consumers (e.g. Player render-path checks)
        // need this signal so they don't silently overwrite the held value.
        let prev = [(1.0, 2.0), (3.0, 4.0)];
        let next = [(10.0, 20.0), (30.0, 40.0)];
        let buf = deform_buf(&[(0, InterpolationType::None, &prev, &[]), (10, InterpolationType::None, &next, &[])]);

        let mut hint = 0;
        let (data, flags) = eval_deform_owned(&buf, 5.0, &mut hint).unwrap();
        assert!(!flags.is_empty(), "off-keyframe None still derives from a key, must set ON");
        assert_eq!(data.coordinate[0].x(), prev[0].0);
    }

    #[test]
    fn test_evaluate_deform_after_last_key() {
        // After last key: value is held from the last key, flag must be ON
        // (the value still comes from animation data).
        let prev = [(7.0, 8.0)];
        let buf = deform_buf(&[(5, InterpolationType::Linear, &prev, &[])]);

        let mut hint = 0;
        let (data, flags) = eval_deform_owned(&buf, 5.0, &mut hint).unwrap();
        assert!(!flags.is_empty());
        assert_eq!(data.coordinate[0].x(), 7.0);

        let mut hint = 0;
        let (data, flags) = eval_deform_owned(&buf, 9.0, &mut hint).unwrap();
        assert!(!flags.is_empty(), "held value past last key must set ON");
        assert_eq!(data.coordinate[0].x(), 7.0);
    }

    #[test]
    fn test_evaluate_deform_index_merged() {
        // Deform keyframes may change different vertex sets. evaluate_deform
        // merges prev and next by vertex index; a vertex present on only one
        // side interpolates against a zero offset.
        let prev = [(6.0, 6.0), (2.0, 2.0)];
        let next = [(8.0, 8.0), (4.0, 4.0)];
        let prev_indices: [i16; 2] = [1, 3];
        let next_indices: [i16; 2] = [3, 5];
        let buf = deform_buf(&[(0, InterpolationType::Linear, &prev, &prev_indices), (10, InterpolationType::None, &next, &next_indices)]);

        let mut hint = 0;
        let (data, _) = eval_deform_owned(&buf, 5.0, &mut hint).unwrap();
        // Ascending union of {1,3} and {3,5}.
        assert_eq!(data.index, alloc::vec![1, 3, 5]);
        // rate = 0.5. v1: prev-only  -> lerp((6,6), 0)      = (3,3).
        //             v3: both       -> lerp((2,2), (8,8))  = (5,5).
        //             v5: next-only  -> lerp(0,     (4,4))  = (2,2).
        assert!((data.coordinate[0].x() - 3.0).abs() < 1e-6, "v1 x: got {}", data.coordinate[0].x());
        assert!((data.coordinate[1].x() - 5.0).abs() < 1e-6, "v3 x: got {}", data.coordinate[1].x());
        assert!((data.coordinate[2].x() - 2.0).abs() < 1e-6, "v5 x: got {}", data.coordinate[2].x());
    }

    #[test]
    fn test_evaluate_deform_hermite() {
        // Hermite interpolation with zero tangents matches midpoint at rate 0.5.
        let prev: alloc::vec::Vec<(f32, f32)> = (0..4).map(|i| (i as f32, 0.0)).collect();
        let next: alloc::vec::Vec<(f32, f32)> = (0..4).map(|i| ((i as f32) + 10.0, 0.0)).collect();
        let buf = deform_buf_with_curves(&[(0, InterpolationType::Hermite, &prev, &[]), (10, InterpolationType::None, &next, &[])], &[KeyCurve::new(0.0, 0.0, 0.0, 1.0)]);

        let mut hint = 0;
        let (data, _) = eval_deform_owned(&buf, 5.0, &mut hint).unwrap();
        // Zero-tangent Hermite at midpoint = 0.5 → x = prev + (next-prev) * 0.5
        for i in 0..4 {
            let exp = (i as f32) + 5.0;
            assert!((data.coordinate[i].x() - exp).abs() < 1e-5, "x at {i}: got {} want {exp}", data.coordinate[i].x());
        }
    }
}
