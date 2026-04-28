use crate::runtime;
use alloc::vec::Vec;
use ssab::ssab::ss::format::InterpolationType;
use ssinterpolate::interpolate::*;

const UPDATE_FLAG_ON: runtime::framedata::ss::runtime::UpdateAttributeFlags = runtime::framedata::ss::runtime::UpdateAttributeFlags::all();
const UPDATE_FLAG_OFF: runtime::framedata::ss::runtime::UpdateAttributeFlags = runtime::framedata::ss::runtime::UpdateAttributeFlags::empty();

#[inline]
pub fn get_interpolated_rate(interpolation: InterpolationType, frame_no: f32, prev_frame: f32, next_frame: f32, curve: Option<ssab::ssab::ss::format::SsCurve>, easing_rate: f32) -> f32 {
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

// Optimized for both forward and reverse playback (O(1) fast path).
// Long distance seeks will fallback to binary search (O(log N)).
#[inline]
pub fn search_prev_next<'a, T, F>(array: flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<T>>, frame_no: f32, get_index: F, hint: &mut usize) -> Option<(T::Inner, Option<T::Inner>)>
where
    T: 'a + flatbuffers::Follow<'a>,
    F: Fn(&T::Inner) -> i32,
{
    if array.is_empty() {
        return None;
    }
    let len = array.len();

    let mut i = *hint;
    if i >= len {
        i = len - 1;
    }

    let mut curr_val = array.get(i);
    let curr_idx = get_index(&curr_val);

    if (curr_idx as f32) <= frame_no {
        let mut checks = 0;
        while i + 1 < len && checks < 4 {
            let next_val = array.get(i + 1);
            let next_idx = get_index(&next_val);
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
            let prev_val = array.get(local_i);
            let prev_idx = get_index(&prev_val);
            if (prev_idx as f32) <= frame_no {
                *hint = local_i;
                return Some((prev_val, Some(array.get(local_i + 1))));
            }
            checks += 1;
        }
    }

    let mut left = 0;
    let mut right = len;
    while left < right {
        let mid = left + (right - left) / 2;
        let mid_val = array.get(mid);
        if (get_index(&mid_val) as f32) <= frame_no {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    let res_idx = if left == 0 { 0 } else { left - 1 };
    *hint = res_idx;

    if left == 0 {
        return Some((array.get(0), None));
    }

    let prev = array.get(left - 1);
    if (get_index(&prev) as f32) == frame_no || left == len {
        return Some((prev, None));
    }

    let next = array.get(left);
    Some((prev, Some(next)))
}

// Optimized for both forward and reverse playback (O(1) fast path).
// Long distance seeks will fallback to binary search (O(log N)).
#[inline]
pub fn search_prev<'a, T, F>(array: flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<T>>, frame_no: i32, get_index: F, hint: &mut usize) -> Option<T::Inner>
where
    T: 'a + flatbuffers::Follow<'a>,
    F: Fn(&T::Inner) -> i32,
{
    if array.is_empty() {
        return None;
    }
    let len = array.len();

    let mut i = *hint;
    if i >= len {
        i = len - 1;
    }

    let mut curr_val = array.get(i);
    let curr_idx = get_index(&curr_val);

    if curr_idx <= frame_no {
        let mut checks = 0;
        while i + 1 < len && checks < 4 {
            let next_val = array.get(i + 1);
            let next_idx = get_index(&next_val);
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
            let prev_val = array.get(local_i);
            let prev_idx = get_index(&prev_val);
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
        let mid_val = array.get(mid);
        if get_index(&mid_val) <= frame_no {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    let res_idx = if left == 0 { 0 } else { left - 1 };
    *hint = res_idx;

    if left == 0 { None } else { Some(array.get(left - 1)) }
}

#[inline]
pub fn evaluate_float(array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'_>>>, frame_no: f32, hint: &mut usize) -> Option<(f32, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32 as f32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    return (prev.value(), UPDATE_FLAG_ON);
                }
                return (prev.value(), UPDATE_FLAG_OFF);
            }

            let next_index = next.index() as i32 as f32;
            let rate = get_interpolated_rate(interpolation, frame_no, prev_index, next_index, prev.curve(), prev.easing_rate());

            (linear(prev.value(), next.value(), rate), UPDATE_FLAG_ON)
        } else {
            if (prev.index() as i32 as f32) == frame_no { (prev.value(), UPDATE_FLAG_ON) } else { (prev.value(), UPDATE_FLAG_OFF) }
        }
    })
}

#[inline]
pub fn evaluate_int(array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataIntValue<'_>>>, frame_no: f32, hint: &mut usize) -> Option<(i32, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32 as f32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    return (prev.value(), UPDATE_FLAG_ON);
                }
                return (prev.value(), UPDATE_FLAG_OFF);
            }

            let next_index = next.index() as i32 as f32;
            let rate = get_interpolated_rate(interpolation, frame_no, prev_index, next_index, prev.curve(), prev.easing_rate());

            (crate::core::math::roundf(linear(prev.value() as f32, next.value() as f32, rate)) as i32, UPDATE_FLAG_ON)
        } else {
            if (prev.index() as i32 as f32) == frame_no { (prev.value(), UPDATE_FLAG_ON) } else { (prev.value(), UPDATE_FLAG_OFF) }
        }
    })
}

#[inline]
pub fn evaluate_partcolor(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataPartColor<'_>>>,
    frame_no: f32,
    hint: &mut usize,
) -> Option<(runtime::framedata::ss::runtime::PartAttributePartColor, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32 as f32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    return (transfer_partcolor(&prev.value()), UPDATE_FLAG_ON);
                }
                return (transfer_partcolor(&prev.value()), UPDATE_FLAG_OFF);
            }

            let next_index = next.index() as i32 as f32;
            let rate = get_interpolated_rate(interpolation, frame_no, prev_index, next_index, prev.curve(), prev.easing_rate());

            let mut current = runtime::framedata::ss::runtime::PartAttributePartColor::default();
            let Some(prev_val) = prev.value() else {
                return (current, UPDATE_FLAG_OFF);
            };
            let Some(next_val) = next.value() else {
                return (current, UPDATE_FLAG_OFF);
            };

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

            let p_a = wide::f32x4::from([p_lt.0, p_rt.0, p_lb.0, p_rb.0]);
            let p_r = wide::f32x4::from([p_lt.1, p_rt.1, p_lb.1, p_rb.1]);
            let p_g = wide::f32x4::from([p_lt.2, p_rt.2, p_lb.2, p_rb.2]);
            let p_b = wide::f32x4::from([p_lt.3, p_rt.3, p_lb.3, p_rb.3]);
            let p_rate = wide::f32x4::from([p_lt.4, p_rt.4, p_lb.4, p_rb.4]);

            let n_a = wide::f32x4::from([n_lt.0, n_rt.0, n_lb.0, n_rb.0]);
            let n_r = wide::f32x4::from([n_lt.1, n_rt.1, n_lb.1, n_rb.1]);
            let n_g = wide::f32x4::from([n_lt.2, n_rt.2, n_lb.2, n_rb.2]);
            let n_b = wide::f32x4::from([n_lt.3, n_rt.3, n_lb.3, n_rb.3]);
            let n_rate = wide::f32x4::from([n_lt.4, n_rt.4, n_lb.4, n_rb.4]);

            let rate_simd = wide::f32x4::splat(rate);
            let res_a = (p_a + (n_a - p_a) * rate_simd).to_array();
            let res_r = (p_r + (n_r - p_r) * rate_simd).to_array();
            let res_g = (p_g + (n_g - p_g) * rate_simd).to_array();
            let res_b = (p_b + (n_b - p_b) * rate_simd).to_array();
            let res_rate = (p_rate + (n_rate - p_rate) * rate_simd).to_array();

            let create_color = |idx: usize| -> runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor {
                runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&runtime::framedata::ss::runtime::U8Rect::new(res_a[idx] as u8, res_r[idx] as u8, res_g[idx] as u8, res_b[idx] as u8), res_rate[idx])
            };

            current.set_lt(&create_color(0));
            current.set_rt(&create_color(1));
            current.set_lb(&create_color(2));
            current.set_rb(&create_color(3));
            current.set_blend_type(transfer_blend_type(prev_val.blend_type()));

            (current, UPDATE_FLAG_ON)
        } else {
            if (prev.index() as i32 as f32) == frame_no { (transfer_partcolor(&prev.value()), UPDATE_FLAG_ON) } else { (transfer_partcolor(&prev.value()), UPDATE_FLAG_OFF) }
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
fn transfer_partcolor(source: &Option<ssab::ssab::ss::format::PartAttributePartColor>) -> runtime::framedata::ss::runtime::PartAttributePartColor {
    let mut destination = runtime::framedata::ss::runtime::PartAttributePartColor::default();
    if let Some(s) = source {
        destination.set_blend_type(transfer_blend_type(s.blend_type()));

        fn convert_u8rect(rgba: &ssab::ssab::ss::format::U8Rect) -> runtime::framedata::ss::runtime::U8Rect {
            runtime::framedata::ss::runtime::U8Rect::new(rgba.a(), rgba.r(), rgba.g(), rgba.b())
        }

        if let Some(lt) = s.lt()
            && let Some(rgba) = lt.rgba()
        {
            destination.set_lt(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&rgba), lt.rate()));
        }
        if let Some(rt) = s.rt()
            && let Some(rgba) = rt.rgba()
        {
            destination.set_rt(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&rgba), rt.rate()));
        }
        if let Some(rb) = s.rb()
            && let Some(rgba) = rb.rgba()
        {
            destination.set_rb(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&rgba), rb.rate()));
        }
        if let Some(lb) = s.lb()
            && let Some(rgba) = lb.rgba()
        {
            destination.set_lb(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&rgba), lb.rate()));
        }
    }
    destination
}

#[inline]
pub fn evaluate_shader(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataShader<'_>>>,
    frame_no: f32,
    hint: &mut usize,
) -> Option<(runtime::framedata::ss::runtime::PartAttributeShader, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32 as f32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    return (transfer_shader(&prev.value()), UPDATE_FLAG_ON);
                }
                return (transfer_shader(&prev.value()), UPDATE_FLAG_OFF);
            }

            let next_index = next.index() as i32 as f32;
            let rate = get_interpolated_rate(interpolation, frame_no, prev_index, next_index, prev.curve(), prev.easing_rate());

            let Some(prev_value) = prev.value() else {
                return (runtime::framedata::ss::runtime::PartAttributeShader::default(), UPDATE_FLAG_OFF);
            };
            let Some(next_value) = next.value() else {
                return (runtime::framedata::ss::runtime::PartAttributeShader::default(), UPDATE_FLAG_OFF);
            };
            (
                runtime::framedata::ss::runtime::PartAttributeShader::new(
                    prev_value.id_hash(),
                    linear(prev_value.param0(), next_value.param0(), rate),
                    linear(prev_value.param1(), next_value.param1(), rate),
                    linear(prev_value.param2(), next_value.param2(), rate),
                    linear(prev_value.param3(), next_value.param3(), rate),
                    linear(prev_value.param4(), next_value.param4(), rate),
                    linear(prev_value.param5(), next_value.param5(), rate),
                    linear(prev_value.param6(), next_value.param6(), rate),
                    linear(prev_value.param7(), next_value.param7(), rate),
                    prev_value.map0_id(),
                    prev_value.map0_name_hash(),
                    prev_value.map1_id(),
                    prev_value.map1_name_hash(),
                ),
                UPDATE_FLAG_ON,
            )
        } else {
            if (prev.index() as i32 as f32) == frame_no { (transfer_shader(&prev.value()), UPDATE_FLAG_ON) } else { (transfer_shader(&prev.value()), UPDATE_FLAG_OFF) }
        }
    })
}

#[inline]
fn transfer_shader(source: &Option<ssab::ssab::ss::format::PartAttributeShader>) -> runtime::framedata::ss::runtime::PartAttributeShader {
    if let Some(s) = source {
        runtime::framedata::ss::runtime::PartAttributeShader::new(s.id_hash(), s.param0(), s.param1(), s.param2(), s.param3(), s.param4(), s.param5(), s.param6(), s.param7(), s.map0_id(), s.map0_name_hash(), s.map1_id(), s.map1_name_hash())
    } else {
        runtime::framedata::ss::runtime::PartAttributeShader::default()
    }
}

#[inline]
pub fn evaluate_vertex(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataVertex<'_>>>,
    frame_no: f32,
    hint: &mut usize,
) -> Option<(runtime::framedata::ss::runtime::PartAttributeVertex, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32 as f32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    return (transfer_vertex(&prev.value()), UPDATE_FLAG_ON);
                }
                return (transfer_vertex(&prev.value()), UPDATE_FLAG_OFF);
            }

            let next_index = next.index() as i32 as f32;
            let rate = get_interpolated_rate(interpolation, frame_no, prev_index, next_index, prev.curve(), prev.easing_rate());

            let Some(prev_value) = prev.value() else {
                return (runtime::framedata::ss::runtime::PartAttributeVertex::default(), UPDATE_FLAG_OFF);
            };
            let Some(next_value) = next.value() else {
                return (runtime::framedata::ss::runtime::PartAttributeVertex::default(), UPDATE_FLAG_OFF);
            };

            let get_coords = |v: &Option<ssab::ssab::ss::format::Vec2>| -> (f32, f32) { if let Some(v) = v { (v.v1(), v.v2()) } else { (0.0, 0.0) } };

            let p_lt = get_coords(&prev_value.lt());
            let p_rt = get_coords(&prev_value.rt());
            let p_lb = get_coords(&prev_value.lb());
            let p_rb = get_coords(&prev_value.rb());

            let n_lt = get_coords(&next_value.lt());
            let n_rt = get_coords(&next_value.rt());
            let n_lb = get_coords(&next_value.lb());
            let n_rb = get_coords(&next_value.rb());

            let p_v1 = wide::f32x4::from([p_lt.0, p_rt.0, p_lb.0, p_rb.0]);
            let p_v2 = wide::f32x4::from([p_lt.1, p_rt.1, p_lb.1, p_rb.1]);

            let n_v1 = wide::f32x4::from([n_lt.0, n_rt.0, n_lb.0, n_rb.0]);
            let n_v2 = wide::f32x4::from([n_lt.1, n_rt.1, n_lb.1, n_rb.1]);

            let rate_simd = wide::f32x4::splat(rate);
            let res_v1 = p_v1 + (n_v1 - p_v1) * rate_simd;
            let res_v2 = p_v2 + (n_v2 - p_v2) * rate_simd;

            let r_v1 = res_v1.to_array();
            let r_v2 = res_v2.to_array();

            (
                runtime::framedata::ss::runtime::PartAttributeVertex::new(
                    &runtime::framedata::ss::runtime::Vec2::new(r_v1[0], r_v2[0]),
                    &runtime::framedata::ss::runtime::Vec2::new(r_v1[1], r_v2[1]),
                    &runtime::framedata::ss::runtime::Vec2::new(r_v1[2], r_v2[2]),
                    &runtime::framedata::ss::runtime::Vec2::new(r_v1[3], r_v2[3]),
                ),
                UPDATE_FLAG_ON,
            )
        } else {
            if (prev.index() as i32 as f32) == frame_no { (transfer_vertex(&prev.value()), UPDATE_FLAG_ON) } else { (transfer_vertex(&prev.value()), UPDATE_FLAG_OFF) }
        }
    })
}

#[inline]
fn transfer_vertex(source: &Option<ssab::ssab::ss::format::PartAttributeVertex>) -> runtime::framedata::ss::runtime::PartAttributeVertex {
    if let Some(s) = source {
        let lt = s.lt().map(|lt| runtime::framedata::ss::runtime::Vec2::new(lt.v1(), lt.v2())).unwrap_or_default();
        let rt = s.rt().map(|rt| runtime::framedata::ss::runtime::Vec2::new(rt.v1(), rt.v2())).unwrap_or_default();
        let lb = s.lb().map(|lb| runtime::framedata::ss::runtime::Vec2::new(lb.v1(), lb.v2())).unwrap_or_default();
        let rb = s.rb().map(|rb| runtime::framedata::ss::runtime::Vec2::new(rb.v1(), rb.v2())).unwrap_or_default();

        runtime::framedata::ss::runtime::PartAttributeVertex::new(&lt, &rt, &lb, &rb)
    } else {
        runtime::framedata::ss::runtime::PartAttributeVertex::default()
    }
}

#[inline]
pub fn evaluate_skew(array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataVec2<'_>>>, frame_no: f32, hint: &mut usize) -> Option<(f32, f32, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let Some(prev_value) = prev.value() else {
                return (0.0, 0.0, UPDATE_FLAG_OFF);
            };
            let prev_index = prev.index() as i32 as f32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    return (prev_value.v1(), prev_value.v2(), UPDATE_FLAG_ON);
                }
                return (prev_value.v1(), prev_value.v2(), UPDATE_FLAG_OFF);
            }

            let next_index = next.index() as i32 as f32;
            let rate = get_interpolated_rate(interpolation, frame_no, prev_index, next_index, prev.curve(), prev.easing_rate());

            let Some(next_value) = next.value() else {
                return (prev_value.v1(), prev_value.v2(), UPDATE_FLAG_OFF);
            };
            (linear(prev_value.v1(), next_value.v1(), rate), linear(prev_value.v2(), next_value.v2(), rate), UPDATE_FLAG_ON)
        } else {
            let Some(prev_value) = prev.value() else {
                return (0.0, 0.0, UPDATE_FLAG_OFF);
            };
            if (prev.index() as i32 as f32) == frame_no { (prev_value.v1(), prev_value.v2(), UPDATE_FLAG_ON) } else { (prev_value.v1(), prev_value.v2(), UPDATE_FLAG_OFF) }
        }
    })
}

#[derive(Default, Clone, Debug)]
pub struct DeformEvaluationData {
    pub coordinate: Vec<runtime::framedata::ss::runtime::Vec2>,
    pub index: Vec<i16>,
}

#[inline]
pub fn evaluate_deform(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataDeform<'_>>>,
    frame_no: f32,
    hint: &mut usize,
) -> Option<(DeformEvaluationData, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32 as f32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    return (transfer_deform(&prev.value()), UPDATE_FLAG_ON);
                }
                return (transfer_deform(&prev.value()), UPDATE_FLAG_OFF);
            }

            let next_index = next.index() as i32 as f32;
            let rate = get_interpolated_rate(interpolation, frame_no, prev_index, next_index, prev.curve(), prev.easing_rate());

            let mut current = DeformEvaluationData::default();
            let Some(prev_val) = prev.value() else {
                return (current, UPDATE_FLAG_OFF);
            };
            let Some(next_val) = next.value() else {
                return (current, UPDATE_FLAG_OFF);
            };

            if let (Some(prev_coodinate), Some(next_coodinate)) = (prev_val.coordinate(), next_val.coordinate()) {
                let length_full = prev_coodinate.len();
                current.coordinate.reserve(length_full);
                let length = core::cmp::min(length_full, next_coodinate.len());

                // SIMD lerp 4 vertices at a time on the overlap.
                let rate_simd = wide::f32x4::splat(rate);
                let chunks = length / 4;
                for c in 0..chunks {
                    let i = c * 4;
                    let p0 = prev_coodinate.get(i);
                    let p1 = prev_coodinate.get(i + 1);
                    let p2 = prev_coodinate.get(i + 2);
                    let p3 = prev_coodinate.get(i + 3);
                    let n0 = next_coodinate.get(i);
                    let n1 = next_coodinate.get(i + 1);
                    let n2 = next_coodinate.get(i + 2);
                    let n3 = next_coodinate.get(i + 3);

                    let p_v1 = wide::f32x4::from([p0.v1(), p1.v1(), p2.v1(), p3.v1()]);
                    let p_v2 = wide::f32x4::from([p0.v2(), p1.v2(), p2.v2(), p3.v2()]);
                    let n_v1 = wide::f32x4::from([n0.v1(), n1.v1(), n2.v1(), n3.v1()]);
                    let n_v2 = wide::f32x4::from([n0.v2(), n1.v2(), n2.v2(), n3.v2()]);

                    let r_v1 = (p_v1 + (n_v1 - p_v1) * rate_simd).to_array();
                    let r_v2 = (p_v2 + (n_v2 - p_v2) * rate_simd).to_array();

                    for k in 0..4 {
                        current.coordinate.push(runtime::framedata::ss::runtime::Vec2::new(r_v1[k], r_v2[k]));
                    }
                }
                // Tail (length - chunks*4 elements remaining on the overlap).
                for i in (chunks * 4)..length {
                    current.coordinate.push(runtime::framedata::ss::runtime::Vec2::new(linear(prev_coodinate.get(i).v1(), next_coodinate.get(i).v1(), rate), linear(prev_coodinate.get(i).v2(), next_coodinate.get(i).v2(), rate)));
                }
                // Overrun use the value of `prev` as is.
                for i in length..length_full {
                    current.coordinate.push(runtime::framedata::ss::runtime::Vec2::new(prev_coodinate.get(i).v1(), prev_coodinate.get(i).v2()));
                }

                // "index" is not interpolated.
                if let Some(index) = prev_val.index() {
                    current.index = index.iter().collect();
                }

                (current, UPDATE_FLAG_ON)
            } else {
                (transfer_deform(&prev.value()), UPDATE_FLAG_ON)
            }
        } else {
            if (prev.index() as i32 as f32) == frame_no { (transfer_deform(&prev.value()), UPDATE_FLAG_ON) } else { (transfer_deform(&prev.value()), UPDATE_FLAG_OFF) }
        }
    })
}

#[inline]
fn transfer_deform(source: &Option<ssab::ssab::ss::format::PartAttributeDeform>) -> DeformEvaluationData {
    let mut destination = DeformEvaluationData::default();
    if let Some(s) = source {
        if let Some(coordinate) = s.coordinate() {
            destination.coordinate = coordinate.iter().map(|v| runtime::framedata::ss::runtime::Vec2::new(v.v1(), v.v2())).collect();
        }
        if let Some(index) = s.index() {
            destination.index = index.iter().collect();
        }
    }
    destination
}

#[inline]
pub fn evaluate_bool(array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataBoolValue<'_>>>, frame_no: f32, hint: &mut usize) -> Option<bool> {
    let frame_no = crate::core::math::floorf(frame_no) as i32;
    search_prev(array, frame_no, |v| v.index() as i32, hint).map(|prev| prev.value())
}

#[inline]
pub fn evaluate_cell<'a>(array: flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataCellValue<'a>>>, frame_no: f32, hint: &mut usize) -> Option<ssab::ssab::ss::format::PartAttributeCell<'a>> {
    let frame_no = crate::core::math::floorf(frame_no) as i32;
    search_prev(array, frame_no, |v| v.index() as i32, hint).and_then(|prev| prev.value())
}

#[inline]
pub fn evaluate_texture(array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataUIntValue<'_>>>, frame_no: f32, hint: &mut usize) -> Option<u32> {
    let frame_no = crate::core::math::floorf(frame_no) as i32;
    search_prev(array, frame_no, |v| v.index() as i32, hint).map(|prev| prev.value())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ssab::ssab::ss::format::{PartAnimationDataBoolValue, PartAnimationDataBoolValueArgs};
    use ssab::ssab::ss::format::{PartAnimationDataUIntValue, PartAnimationDataUIntValueArgs};

    #[test]
    fn test_linear_interpolation() {
        assert_eq!(linear(0.0, 10.0, 0.5), 5.0);
        assert_eq!(linear(10.0, 20.0, 0.0), 10.0);
        assert_eq!(linear(10.0, 20.0, 1.0), 20.0);
    }

    #[test]
    fn test_evaluate_bool() {
        let mut builder = flatbuffers::FlatBufferBuilder::new();

        let v0 = PartAnimationDataBoolValue::create(&mut builder, &PartAnimationDataBoolValueArgs { index: 0, value: true });
        let v10 = PartAnimationDataBoolValue::create(&mut builder, &PartAnimationDataBoolValueArgs { index: 10, value: false });
        let values_off = builder.create_vector(&[v0, v10]);
        builder.finish(values_off, None);

        let buf = builder.finished_data();
        let vector = flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<PartAnimationDataBoolValue>>>(buf).unwrap();

        let mut hint = 0;
        // Exact match
        assert_eq!(evaluate_bool(vector, 0.0, &mut hint), Some(true));
        // Interpolated (should use prev)
        assert_eq!(evaluate_bool(vector, 5.0, &mut hint), Some(true));
        // Exact match
        assert_eq!(evaluate_bool(vector, 10.0, &mut hint), Some(false));
        // Beyond last key
        assert_eq!(evaluate_bool(vector, 15.0, &mut hint), Some(false));
        // Before first key
        assert_eq!(evaluate_bool(vector, -1.0, &mut hint), None);
        // Fractional: floor() applied internally — 9.9 → 9 → still uses prev (true)
        assert_eq!(evaluate_bool(vector, 9.9, &mut hint), Some(true));
        // Fractional: 10.0 → 10 → false
        assert_eq!(evaluate_bool(vector, 10.0, &mut hint), Some(false));
    }

    #[test]
    fn test_evaluate_texture() {
        let mut builder = flatbuffers::FlatBufferBuilder::new();

        let v0 = PartAnimationDataUIntValue::create(&mut builder, &PartAnimationDataUIntValueArgs { index: 0, value: 100, interpolation: InterpolationType::None });
        let v10 = PartAnimationDataUIntValue::create(&mut builder, &PartAnimationDataUIntValueArgs { index: 10, value: 200, interpolation: InterpolationType::None });
        let values_off = builder.create_vector(&[v0, v10]);
        builder.finish(values_off, None);

        let buf = builder.finished_data();
        let vector = flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<PartAnimationDataUIntValue>>>(buf).unwrap();

        let mut hint = 0;
        assert_eq!(evaluate_texture(vector, 0.0, &mut hint), Some(100));
        assert_eq!(evaluate_texture(vector, 5.0, &mut hint), Some(100));
        assert_eq!(evaluate_texture(vector, 10.0, &mut hint), Some(200));
        assert_eq!(evaluate_texture(vector, 15.0, &mut hint), Some(200));
        // Fractional: floor() applied internally — 9.99 → 9 → still 100
        assert_eq!(evaluate_texture(vector, 9.99, &mut hint), Some(100));
    }

    #[test]
    fn test_evaluate_int_no_interpolation() {
        use ssab::ssab::ss::format::{PartAnimationDataIntValue, PartAnimationDataIntValueArgs};
        let mut builder = flatbuffers::FlatBufferBuilder::new();

        let v0 = PartAnimationDataIntValue::create(&mut builder, &PartAnimationDataIntValueArgs { index: 0, value: 100, interpolation: InterpolationType::None, ..Default::default() });
        let v10 = PartAnimationDataIntValue::create(&mut builder, &PartAnimationDataIntValueArgs { index: 10, value: 200, interpolation: InterpolationType::None, ..Default::default() });
        let values_off = builder.create_vector(&[v0, v10]);
        builder.finish(values_off, None);

        let buf = builder.finished_data();
        let vector = flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<PartAnimationDataIntValue>>>(buf).unwrap();

        let mut hint = 0;
        // At frame 0, should be ON and 100
        let (val, flags) = evaluate_int(vector, 0.0, &mut hint).unwrap();
        assert_eq!(val, 100);
        assert!(!flags.is_empty());

        // At frame 5, should be OFF (inherited) and 100
        let (val, flags) = evaluate_int(vector, 5.0, &mut hint).unwrap();
        assert_eq!(val, 100);
        assert!(flags.is_empty());

        // At frame 10, should be ON and 200
        let (val, flags) = evaluate_int(vector, 10.0, &mut hint).unwrap();
        assert_eq!(val, 200);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_evaluate_int_linear_interpolation_fractional() {
        use ssab::ssab::ss::format::{PartAnimationDataIntValue, PartAnimationDataIntValueArgs};
        let mut builder = flatbuffers::FlatBufferBuilder::new();

        let v0 = PartAnimationDataIntValue::create(&mut builder, &PartAnimationDataIntValueArgs { index: 0, value: 0, interpolation: InterpolationType::Linear, ..Default::default() });
        let v10 = PartAnimationDataIntValue::create(&mut builder, &PartAnimationDataIntValueArgs { index: 10, value: 100, interpolation: InterpolationType::None, ..Default::default() });
        let values_off = builder.create_vector(&[v0, v10]);
        builder.finish(values_off, None);

        let buf = builder.finished_data();
        let vector = flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<PartAnimationDataIntValue>>>(buf).unwrap();

        // Fractional frame: 5.5 should yield round(linear(0,100,0.55)) == 55
        let mut hint = 0;
        let (val, flags) = evaluate_int(vector, 5.5, &mut hint).unwrap();
        assert_eq!(val, 55);
        assert!(!flags.is_empty());

        // Integer frame still works at the keyframe (rate=0.5 -> 50)
        let mut hint = 0;
        let (val, _) = evaluate_int(vector, 5.0, &mut hint).unwrap();
        assert_eq!(val, 50);
    }

    #[test]
    fn test_evaluate_float_linear_interpolation_fractional() {
        use ssab::ssab::ss::format::{PartAnimationDataFloatValue, PartAnimationDataFloatValueArgs};
        let mut builder = flatbuffers::FlatBufferBuilder::new();

        let v0 = PartAnimationDataFloatValue::create(&mut builder, &PartAnimationDataFloatValueArgs { index: 0, value: 0.0, interpolation: InterpolationType::Linear, ..Default::default() });
        let v10 = PartAnimationDataFloatValue::create(&mut builder, &PartAnimationDataFloatValueArgs { index: 10, value: 1.0, interpolation: InterpolationType::None, ..Default::default() });
        let values_off = builder.create_vector(&[v0, v10]);
        builder.finish(values_off, None);

        let buf = builder.finished_data();
        let vector = flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<PartAnimationDataFloatValue>>>(buf).unwrap();

        // Fractional 2.5 → rate 0.25
        let mut hint = 0;
        let (val, flags) = evaluate_float(vector, 2.5, &mut hint).unwrap();
        assert!((val - 0.25).abs() < 1e-6, "expected ~0.25, got {val}");
        assert!(!flags.is_empty());

        // Integer 5 → rate 0.5
        let mut hint = 0;
        let (val, _) = evaluate_float(vector, 5.0, &mut hint).unwrap();
        assert!((val - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_cell() {
        use ssab::ssab::ss::format::{PartAnimationDataCellValue, PartAnimationDataCellValueArgs, PartAttributeCell, PartAttributeCellArgs};
        let mut builder = flatbuffers::FlatBufferBuilder::new();

        let cell_a = PartAttributeCell::create(&mut builder, &PartAttributeCellArgs { map_id: 1, name_hash: 111 });
        let v0 = PartAnimationDataCellValue::create(&mut builder, &PartAnimationDataCellValueArgs { index: 0, value: Some(cell_a) });

        let cell_b = PartAttributeCell::create(&mut builder, &PartAttributeCellArgs { map_id: 2, name_hash: 222 });
        let v10 = PartAnimationDataCellValue::create(&mut builder, &PartAnimationDataCellValueArgs { index: 10, value: Some(cell_b) });

        let values_off = builder.create_vector(&[v0, v10]);
        builder.finish(values_off, None);

        let buf = builder.finished_data();
        let vector = flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<PartAnimationDataCellValue>>>(buf).unwrap();

        let mut hint = 0;
        let res = evaluate_cell(vector, 5.0, &mut hint).unwrap();
        assert_eq!(res.map_id(), 1);
        assert_eq!(res.name_hash(), 111);

        let res = evaluate_cell(vector, 10.0, &mut hint).unwrap();
        assert_eq!(res.map_id(), 2);
        assert_eq!(res.name_hash(), 222);

        // Fractional: floor() applied internally — 9.999 → 9 → still cell A
        let mut hint = 0;
        let res = evaluate_cell(vector, 9.999, &mut hint).unwrap();
        assert_eq!(res.map_id(), 1);
    }

    #[test]
    fn test_evaluate_float_hermite() {
        use ssab::ssab::ss::format::{PartAnimationDataFloatValue, PartAnimationDataFloatValueArgs, SsCurve, SsCurveArgs};
        let mut builder = flatbuffers::FlatBufferBuilder::new();

        // start_value/end_value are absolute target values for tangents, so
        // (0, 1) → tangent (s_start - v_start) = 0 at start, (s_end - v_end) = 0 at end.
        let curve = SsCurve::create(&mut builder, &SsCurveArgs { start_time: 0.0, start_value: 0.0, end_time: 0.0, end_value: 1.0 });
        let v0 = PartAnimationDataFloatValue::create(&mut builder, &PartAnimationDataFloatValueArgs { index: 0, value: 0.0, interpolation: InterpolationType::Hermite, curve: Some(curve), ..Default::default() });
        let v10 = PartAnimationDataFloatValue::create(&mut builder, &PartAnimationDataFloatValueArgs { index: 10, value: 1.0, interpolation: InterpolationType::None, ..Default::default() });
        let values_off = builder.create_vector(&[v0, v10]);
        builder.finish(values_off, None);

        let buf = builder.finished_data();
        let vector = flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<PartAnimationDataFloatValue>>>(buf).unwrap();

        // Hermite with zero tangents at midpoint: rate = 3*0.5^2 - 2*0.5^3 = 0.5
        let mut hint = 0;
        let (val, _) = evaluate_float(vector, 5.0, &mut hint).unwrap();
        assert!((val - 0.5).abs() < 1e-6, "expected ~0.5, got {val}");

        // Sub-frame between integer keys: smooth Hermite curve
        let mut hint = 0;
        let (val_25, _) = evaluate_float(vector, 2.5, &mut hint).unwrap();
        let (val_3, _) = evaluate_float(vector, 3.0, &mut hint).unwrap();
        assert!(val_25 < val_3, "Hermite must be monotonically increasing on this segment");
        assert!(val_25 > 0.0 && val_25 < 1.0);
    }

    #[test]
    fn test_evaluate_float_ease_in() {
        use ssab::ssab::ss::format::{PartAnimationDataFloatValue, PartAnimationDataFloatValueArgs};
        let mut builder = flatbuffers::FlatBufferBuilder::new();

        let v0 = PartAnimationDataFloatValue::create(&mut builder, &PartAnimationDataFloatValueArgs { index: 0, value: 0.0, interpolation: InterpolationType::EaseIn, easing_rate: 2.0, ..Default::default() });
        let v10 = PartAnimationDataFloatValue::create(&mut builder, &PartAnimationDataFloatValueArgs { index: 10, value: 1.0, interpolation: InterpolationType::None, ..Default::default() });
        let values_off = builder.create_vector(&[v0, v10]);
        builder.finish(values_off, None);

        let buf = builder.finished_data();
        let vector = flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<PartAnimationDataFloatValue>>>(buf).unwrap();

        // EaseIn with power 2 at rate 0.5: 0.5^2 = 0.25
        let mut hint = 0;
        let (val, _) = evaluate_float(vector, 5.0, &mut hint).unwrap();
        assert!((val - 0.25).abs() < 1e-5, "expected ~0.25, got {val}");

        // Sub-frame: smooth, not stepped
        let mut hint = 0;
        let (val_25, _) = evaluate_float(vector, 2.5, &mut hint).unwrap();
        let (val_3, _) = evaluate_float(vector, 3.0, &mut hint).unwrap();
        assert!(val_25 < val_3);
    }

    #[test]
    fn test_evaluate_skew_linear() {
        use ssab::ssab::ss::format::{PartAnimationDataVec2, PartAnimationDataVec2Args, Vec2, Vec2Args};
        let mut builder = flatbuffers::FlatBufferBuilder::new();

        let v0_value = Vec2::create(&mut builder, &Vec2Args { v1: 0.0, v2: 0.0 });
        let v0 = PartAnimationDataVec2::create(&mut builder, &PartAnimationDataVec2Args { index: 0, value: Some(v0_value), interpolation: InterpolationType::Linear, ..Default::default() });
        let v10_value = Vec2::create(&mut builder, &Vec2Args { v1: 10.0, v2: 20.0 });
        let v10 = PartAnimationDataVec2::create(&mut builder, &PartAnimationDataVec2Args { index: 10, value: Some(v10_value), interpolation: InterpolationType::None, ..Default::default() });
        let values_off = builder.create_vector(&[v0, v10]);
        builder.finish(values_off, None);

        let buf = builder.finished_data();
        let vector = flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<PartAnimationDataVec2>>>(buf).unwrap();

        // Midpoint: linear interpolation of both axes
        let mut hint = 0;
        let (val_x, val_y, flags) = evaluate_skew(vector, 5.0, &mut hint).unwrap();
        assert!((val_x - 5.0).abs() < 1e-6);
        assert!((val_y - 10.0).abs() < 1e-6);
        assert!(!flags.is_empty());

        // Sub-frame: smooth interpolation
        let mut hint = 0;
        let (val_x, val_y, _) = evaluate_skew(vector, 2.5, &mut hint).unwrap();
        assert!((val_x - 2.5).abs() < 1e-6);
        assert!((val_y - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_vertex_linear() {
        use ssab::ssab::ss::format::{PartAnimationDataVertex, PartAnimationDataVertexArgs, PartAttributeVertex, PartAttributeVertexArgs, Vec2, Vec2Args};
        let mut builder = flatbuffers::FlatBufferBuilder::new();

        let lt0 = Vec2::create(&mut builder, &Vec2Args { v1: 0.0, v2: 0.0 });
        let rt0 = Vec2::create(&mut builder, &Vec2Args { v1: 0.0, v2: 0.0 });
        let lb0 = Vec2::create(&mut builder, &Vec2Args { v1: 0.0, v2: 0.0 });
        let rb0 = Vec2::create(&mut builder, &Vec2Args { v1: 0.0, v2: 0.0 });
        let v0_value = PartAttributeVertex::create(&mut builder, &PartAttributeVertexArgs { lt: Some(lt0), rt: Some(rt0), lb: Some(lb0), rb: Some(rb0) });
        let v0 = PartAnimationDataVertex::create(&mut builder, &PartAnimationDataVertexArgs { index: 0, value: Some(v0_value), interpolation: InterpolationType::Linear, ..Default::default() });

        let lt1 = Vec2::create(&mut builder, &Vec2Args { v1: 10.0, v2: 10.0 });
        let rt1 = Vec2::create(&mut builder, &Vec2Args { v1: -10.0, v2: 10.0 });
        let lb1 = Vec2::create(&mut builder, &Vec2Args { v1: 10.0, v2: -10.0 });
        let rb1 = Vec2::create(&mut builder, &Vec2Args { v1: -10.0, v2: -10.0 });
        let v10_value = PartAttributeVertex::create(&mut builder, &PartAttributeVertexArgs { lt: Some(lt1), rt: Some(rt1), lb: Some(lb1), rb: Some(rb1) });
        let v10 = PartAnimationDataVertex::create(&mut builder, &PartAnimationDataVertexArgs { index: 10, value: Some(v10_value), interpolation: InterpolationType::None, ..Default::default() });

        let values_off = builder.create_vector(&[v0, v10]);
        builder.finish(values_off, None);

        let buf = builder.finished_data();
        let vector = flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<PartAnimationDataVertex>>>(buf).unwrap();

        // Midpoint: each corner linearly interpolated
        let mut hint = 0;
        let (val, _) = evaluate_vertex(vector, 5.0, &mut hint).unwrap();
        assert!((val.lt().x() - 5.0).abs() < 1e-6);
        assert!((val.rt().x() - -5.0).abs() < 1e-6);
        assert!((val.lb().y() - -5.0).abs() < 1e-6);
        assert!((val.rb().y() - -5.0).abs() < 1e-6);
    }

    // -- evaluate_deform tests --

    /// Build a single PartAnimationDataDeform key with given index, interpolation, coordinates, and indices.
    fn deform_buf(keys: &[(u16, InterpolationType, &[(f32, f32)], &[i16])]) -> alloc::vec::Vec<u8> {
        use ssab::ssab::ss::format::{PartAnimationDataDeform, PartAnimationDataDeformArgs, PartAttributeDeform, PartAttributeDeformArgs, Vec2, Vec2Args};

        let mut builder = flatbuffers::FlatBufferBuilder::new();
        let mut key_offsets = alloc::vec::Vec::new();

        for (idx, ip, coords, indices) in keys {
            let vec_offsets: alloc::vec::Vec<_> = coords.iter().map(|(x, y)| Vec2::create(&mut builder, &Vec2Args { v1: *x, v2: *y })).collect();
            let coord_off = if vec_offsets.is_empty() { None } else { Some(builder.create_vector(&vec_offsets)) };
            let idx_off = if indices.is_empty() { None } else { Some(builder.create_vector(indices)) };
            let value = PartAttributeDeform::create(&mut builder, &PartAttributeDeformArgs { coordinate: coord_off, index: idx_off });
            let key = PartAnimationDataDeform::create(&mut builder, &PartAnimationDataDeformArgs { index: *idx, interpolation: *ip, value: Some(value), ..Default::default() });
            key_offsets.push(key);
        }

        let arr = builder.create_vector(&key_offsets);
        builder.finish(arr, None);
        builder.finished_data().to_vec()
    }

    fn deform_root(buf: &[u8]) -> flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataDeform<'_>>> {
        flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataDeform>>>(buf).unwrap()
    }

    #[test]
    fn test_evaluate_deform_simd_chunk_aligned() {
        // 8 vertices = exactly 2 SIMD chunks, no tail.
        let prev: alloc::vec::Vec<(f32, f32)> = (0..8).map(|i| (i as f32, i as f32 * 2.0)).collect();
        let next: alloc::vec::Vec<(f32, f32)> = (0..8).map(|i| (i as f32 * 3.0, i as f32 * 4.0)).collect();
        let buf = deform_buf(&[(0, InterpolationType::Linear, &prev, &[]), (10, InterpolationType::None, &next, &[])]);
        let vec = deform_root(&buf);

        // At rate=0.5, lerp(prev, next, 0.5) = (prev + next) / 2.
        let mut hint = 0;
        let (data, flags) = evaluate_deform(vec, 5.0, &mut hint).unwrap();
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
        let vec = deform_root(&buf);

        let mut hint = 0;
        let (data, _) = evaluate_deform(vec, 2.5, &mut hint).unwrap();
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
        let vec = deform_root(&buf);

        let mut hint = 0;
        let (data, _) = evaluate_deform(vec, 5.0, &mut hint).unwrap();
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
        let vec = deform_root(&buf);

        let mut hint = 0;
        let (data, _) = evaluate_deform(vec, 5.0, &mut hint).unwrap();
        assert_eq!(data.coordinate.len(), 6, "output length should match prev_full");
        // First 4 are interpolated.
        for i in 0..4 {
            let exp_x = (prev[i].0 + next[i].0) / 2.0;
            let exp_y = (prev[i].1 + next[i].1) / 2.0;
            assert!((data.coordinate[i].x() - exp_x).abs() < 1e-4, "interpolated x mismatch at {i}");
            assert!((data.coordinate[i].y() - exp_y).abs() < 1e-4, "interpolated y mismatch at {i}");
        }
        // Indices 4 and 5: passthrough from prev.
        for i in 4..6 {
            assert_eq!(data.coordinate[i].x(), prev[i].0, "overrun x at {i}");
            assert_eq!(data.coordinate[i].y(), prev[i].1, "overrun y at {i}");
        }
    }

    #[test]
    fn test_evaluate_deform_none_at_keyframe() {
        // None interpolation at exact keyframe → flag ON, prev value.
        let prev = [(1.0, 2.0), (3.0, 4.0)];
        let next = [(10.0, 20.0), (30.0, 40.0)];
        let buf = deform_buf(&[(0, InterpolationType::None, &prev, &[]), (10, InterpolationType::None, &next, &[])]);
        let vec = deform_root(&buf);

        let mut hint = 0;
        let (data, flags) = evaluate_deform(vec, 0.0, &mut hint).unwrap();
        assert!(!flags.is_empty(), "exact keyframe match must set ON");
        assert_eq!(data.coordinate[0].x(), prev[0].0);
        assert_eq!(data.coordinate[0].y(), prev[0].1);
    }

    #[test]
    fn test_evaluate_deform_none_off_keyframe() {
        // None interpolation off-keyframe → flag OFF, prev value held.
        let prev = [(1.0, 2.0), (3.0, 4.0)];
        let next = [(10.0, 20.0), (30.0, 40.0)];
        let buf = deform_buf(&[(0, InterpolationType::None, &prev, &[]), (10, InterpolationType::None, &next, &[])]);
        let vec = deform_root(&buf);

        let mut hint = 0;
        let (data, flags) = evaluate_deform(vec, 5.0, &mut hint).unwrap();
        assert!(flags.is_empty(), "off-keyframe None must clear flag");
        // Should still hold prev value.
        assert_eq!(data.coordinate[0].x(), prev[0].0);
    }

    #[test]
    fn test_evaluate_deform_after_last_key() {
        // After last key: exact match → ON, otherwise OFF, value held from prev.
        let prev = [(7.0, 8.0)];
        let buf = deform_buf(&[(5, InterpolationType::Linear, &prev, &[])]);
        let vec = deform_root(&buf);

        let mut hint = 0;
        let (data, flags) = evaluate_deform(vec, 5.0, &mut hint).unwrap();
        assert!(!flags.is_empty());
        assert_eq!(data.coordinate[0].x(), 7.0);

        let mut hint = 0;
        let (data, flags) = evaluate_deform(vec, 9.0, &mut hint).unwrap();
        assert!(flags.is_empty());
        assert_eq!(data.coordinate[0].x(), 7.0);
    }

    #[test]
    fn test_evaluate_deform_index_preserved() {
        // The "index" array is not interpolated; runtime must copy from prev key.
        let prev = [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 3.0)];
        let next = [(10.0, 10.0); 4];
        let prev_indices: [i16; 4] = [5, 6, 7, 8];
        let buf = deform_buf(&[(0, InterpolationType::Linear, &prev, &prev_indices), (10, InterpolationType::None, &next, &[42, 43, 44, 45])]);
        let vec = deform_root(&buf);

        let mut hint = 0;
        let (data, _) = evaluate_deform(vec, 5.0, &mut hint).unwrap();
        assert_eq!(data.index, alloc::vec![5, 6, 7, 8], "index must come from prev key, not next");
    }

    #[test]
    fn test_evaluate_deform_hermite() {
        // Hermite interpolation with zero tangents matches midpoint at rate 0.5.
        use ssab::ssab::ss::format::{PartAnimationDataDeform, PartAnimationDataDeformArgs, PartAttributeDeform, PartAttributeDeformArgs, SsCurve, SsCurveArgs, Vec2, Vec2Args};

        let mut builder = flatbuffers::FlatBufferBuilder::new();
        let curve = SsCurve::create(&mut builder, &SsCurveArgs { start_time: 0.0, start_value: 0.0, end_time: 0.0, end_value: 1.0 });

        let p_vec: alloc::vec::Vec<_> = (0..4).map(|i| Vec2::create(&mut builder, &Vec2Args { v1: i as f32, v2: 0.0 })).collect();
        let p_coord = builder.create_vector(&p_vec);
        let p_value = PartAttributeDeform::create(&mut builder, &PartAttributeDeformArgs { coordinate: Some(p_coord), index: None });
        let v0 = PartAnimationDataDeform::create(&mut builder, &PartAnimationDataDeformArgs { index: 0, interpolation: InterpolationType::Hermite, value: Some(p_value), curve: Some(curve), ..Default::default() });

        let n_vec: alloc::vec::Vec<_> = (0..4).map(|i| Vec2::create(&mut builder, &Vec2Args { v1: (i as f32) + 10.0, v2: 0.0 })).collect();
        let n_coord = builder.create_vector(&n_vec);
        let n_value = PartAttributeDeform::create(&mut builder, &PartAttributeDeformArgs { coordinate: Some(n_coord), index: None });
        let v10 = PartAnimationDataDeform::create(&mut builder, &PartAnimationDataDeformArgs { index: 10, interpolation: InterpolationType::None, value: Some(n_value), ..Default::default() });

        let arr = builder.create_vector(&[v0, v10]);
        builder.finish(arr, None);

        let buf = builder.finished_data();
        let vec = flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<PartAnimationDataDeform>>>(buf).unwrap();

        let mut hint = 0;
        let (data, _) = evaluate_deform(vec, 5.0, &mut hint).unwrap();
        // Zero-tangent Hermite at midpoint = 0.5 → x = prev + (next-prev) * 0.5
        for i in 0..4 {
            let exp = (i as f32) + 5.0;
            assert!((data.coordinate[i].x() - exp).abs() < 1e-5, "x at {i}: got {} want {exp}", data.coordinate[i].x());
        }
    }
}
