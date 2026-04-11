use ssab::ssab::ss::format::{InterpolationType, SsAttributePartColorKeyValueColor};
use ssinterpolate::interpolate::*;
use crate::runtime;

const UPDATE_FLAG_ON: runtime::framedata::ss::runtime::UpdateAttributeFlags = runtime::framedata::ss::runtime::UpdateAttributeFlags::all();
const UPDATE_FLAG_OFF: runtime::framedata::ss::runtime::UpdateAttributeFlags = runtime::framedata::ss::runtime::UpdateAttributeFlags::empty();

#[inline]
pub fn get_interpolated_rate(interpolation: InterpolationType, rate: f32) -> f32 {
    match interpolation {
        InterpolationType::Acceleration => rate_accelerate(rate),
        InterpolationType::Deceleration => rate_decelerate(rate),
        InterpolationType::EaseExponentialIn => rate_ease_exponential_in(rate),
        InterpolationType::EaseExponentialOut => rate_ease_exponential_out(rate),
        InterpolationType::EaseExponentialInOut => rate_ease_exponential_inout(rate),
        InterpolationType::EaseSineIn => rate_ease_sine_in(rate),
        InterpolationType::EaseSineOut => rate_ease_sine_out(rate),
        InterpolationType::EaseSineInOut => rate_ease_sine_inout(rate),
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
pub fn search_prev_next<'a, T, F>(
    array: flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<T>>,
    frame_no: i32,
    get_index: F,
    hint: &mut usize,
) -> Option<(T::Inner, Option<T::Inner>)>
where
    T: 'a + flatbuffers::Follow<'a>,
    F: Fn(&T::Inner) -> i32,
{
    if array.is_empty() { return None; }
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
            if prev_idx <= frame_no {
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
        if get_index(&mid_val) <= frame_no {
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
    if get_index(&prev) == frame_no || left == len {
        return Some((prev, None));
    }
    
    let next = array.get(left);
    Some((prev, Some(next)))
}

// Optimized for both forward and reverse playback (O(1) fast path).
// Long distance seeks will fallback to binary search (O(log N)).
#[inline]
pub fn search_prev<'a, T, F>(
    array: flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<T>>,
    frame_no: i32,
    get_index: F,
    hint: &mut usize,
) -> Option<T::Inner>
where
    T: 'a + flatbuffers::Follow<'a>,
    F: Fn(&T::Inner) -> i32,
{
    if array.is_empty() { return None; }
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

    if left == 0 {
        None
    } else {
        Some(array.get(left - 1))
    }
}


#[inline]
pub fn evaluate_float(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'_>>>,
    frame_no: i32,
    hint: &mut usize,
) -> Option<(f32, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    return (prev.value(), UPDATE_FLAG_ON)
                }
                return (prev.value(), UPDATE_FLAG_OFF)
            }

            let next_index = next.index() as i32;
            let rate = get_interpolated_rate(interpolation, rate_linear(frame_no as f32, prev_index as f32, next_index as f32));

            (linear(prev.value(), next.value(), rate), UPDATE_FLAG_ON)
        } else {
            if prev.index() as i32 == frame_no {
                (prev.value(), UPDATE_FLAG_ON)
            } else {
                (prev.value(), UPDATE_FLAG_OFF)
            }
        }
    })
}

#[inline]
pub fn evaluate_int(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataIntValue<'_>>>,
    frame_no: i32,
    hint: &mut usize,
) -> Option<(i32, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    return (prev.value(), UPDATE_FLAG_ON)
                }
                return (prev.value(), UPDATE_FLAG_OFF)
            }

            let next_index = next.index() as i32;
            let rate = get_interpolated_rate(interpolation, rate_linear(frame_no as f32, prev_index as f32, next_index as f32));

            (libm::roundf(linear(prev.value() as f32, next.value() as f32, rate)) as i32, UPDATE_FLAG_ON)
        } else {
            if prev.index() as i32 == frame_no {
                (prev.value(), UPDATE_FLAG_ON)
            } else {
                (prev.value(), UPDATE_FLAG_OFF)
            }
        }
    })
}

#[inline]
pub fn evaluate_partcolor(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataPartColor<'_>>>,
    frame_no: i32,
    hint: &mut usize,
) -> Option<(runtime::framedata::ss::runtime::PartAttributePartColor, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    return (transfer_partcolor(&prev.value()), UPDATE_FLAG_ON)
                }
                return (transfer_partcolor(&prev.value()), UPDATE_FLAG_OFF)
            }

            let next_index = next.index() as i32;
            let rate = get_interpolated_rate(interpolation, rate_linear(frame_no as f32, prev_index as f32, next_index as f32));

            fn interpolate_color(color_start: &Option<SsAttributePartColorKeyValueColor>, color_end: &Option<SsAttributePartColorKeyValueColor>, rate: f32) -> runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor {
                let s = color_start.as_ref().unwrap();
                let e = color_end.as_ref().unwrap();
                let s_rgba = s.rgba().unwrap();
                let e_rgba = e.rgba().unwrap();
                runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(
                    &runtime::framedata::ss::runtime::U8Rect::new(  linear(s_rgba.a() as f32, e_rgba.a() as f32, rate) as u8,
                                                                    linear(s_rgba.r() as f32, e_rgba.r() as f32, rate) as u8,
                                                                    linear(s_rgba.g() as f32, e_rgba.g() as f32, rate) as u8,
                                                                    linear(s_rgba.b() as f32, e_rgba.b() as f32, rate) as u8
                    ),
                    linear(s.rate(), e.rate(), rate)
                )
            }

            let mut current = runtime::framedata::ss::runtime::PartAttributePartColor::default();
            let prev_val = prev.value().unwrap();
            let next_val = next.value().unwrap();
            current.set_lt(&interpolate_color(&prev_val.lt(), &next_val.lt(), rate));
            current.set_rt(&interpolate_color(&prev_val.rt(), &next_val.rt(), rate));
            current.set_rb(&interpolate_color(&prev_val.rb(), &next_val.rb(), rate));
            current.set_lb(&interpolate_color(&prev_val.lb(), &next_val.lb(), rate));
            current.set_blend_type(transfer_blend_type(prev_val.blend_type()));

            (current, UPDATE_FLAG_ON)
        } else {
            if prev.index() as i32 == frame_no {
                (transfer_partcolor(&prev.value()), UPDATE_FLAG_ON)
            } else {
                (transfer_partcolor(&prev.value()), UPDATE_FLAG_OFF)
            }
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
        
        if let Some(lt) = s.lt() {
            destination.set_lt(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&lt.rgba().unwrap()), lt.rate()));
        }
        if let Some(rt) = s.rt() {
            destination.set_rt(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&rt.rgba().unwrap()), rt.rate()));
        }
        if let Some(rb) = s.rb() {
            destination.set_rb(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&rb.rgba().unwrap()), rb.rate()));
        }
        if let Some(lb) = s.lb() {
            destination.set_lb(&runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&lb.rgba().unwrap()), lb.rate()));
        }
    }
    destination
}

#[inline]
pub fn evaluate_shader(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataShader<'_>>>,
    frame_no: i32,
    hint: &mut usize,
) -> Option<(runtime::framedata::ss::runtime::PartAttributeShader, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    return (transfer_shader(&prev.value()), UPDATE_FLAG_ON)
                }
                return (transfer_shader(&prev.value()), UPDATE_FLAG_OFF)
            }

            let next_index = next.index() as i32;
            let rate = get_interpolated_rate(interpolation, rate_linear(frame_no as f32, prev_index as f32, next_index as f32));

            let prev_value = prev.value().unwrap();
            let next_value = next.value().unwrap();
            (runtime::framedata::ss::runtime::PartAttributeShader::new(
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
                    prev_value.map1_name_hash()
                ),
                UPDATE_FLAG_ON
            )
        } else {
            if prev.index() as i32 == frame_no {
                (transfer_shader(&prev.value()), UPDATE_FLAG_ON)
            } else {
                (transfer_shader(&prev.value()), UPDATE_FLAG_OFF)
            }
        }
    })
}

#[inline]
fn transfer_shader(source: &Option<ssab::ssab::ss::format::PartAttributeShader>) -> runtime::framedata::ss::runtime::PartAttributeShader {
    if let Some(s) = source {
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
            s.map0_id(),
            s.map0_name_hash(),
            s.map1_id(),
            s.map1_name_hash()
        )
    } else {
        runtime::framedata::ss::runtime::PartAttributeShader::default()
    }
}

#[inline]
pub fn evaluate_vertex(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataVertex<'_>>>,
    frame_no: i32,
    hint: &mut usize,
) -> Option<(runtime::framedata::ss::runtime::PartAttributeVertex, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    return (transfer_vertex(&prev.value()), UPDATE_FLAG_ON)
                }
                return (transfer_vertex(&prev.value()), UPDATE_FLAG_OFF)
            }

            let next_index = next.index() as i32;
            let rate = get_interpolated_rate(interpolation, rate_linear(frame_no as f32, prev_index as f32, next_index as f32));

            fn interpolate_coordinate(coordinate_start: &Option<ssab::ssab::ss::format::Vec2>, coordinate_end: &Option<ssab::ssab::ss::format::Vec2>, rate: f32) -> runtime::framedata::ss::runtime::Vec2 {
                let s = coordinate_start.as_ref().unwrap();
                let e = coordinate_end.as_ref().unwrap();
                runtime::framedata::ss::runtime::Vec2::new( linear(s.v1(), e.v1(), rate),
                                                            linear(s.v2(), e.v2(), rate)
                )
            }

            let prev_value = prev.value().unwrap();
            let next_value = next.value().unwrap();
            (runtime::framedata::ss::runtime::PartAttributeVertex::new(
                &interpolate_coordinate(&prev_value.lt(), &next_value.lt(), rate),
                &interpolate_coordinate(&prev_value.rt(), &next_value.rt(), rate),
                &interpolate_coordinate(&prev_value.lb(), &next_value.lb(), rate),
                &interpolate_coordinate(&prev_value.rb(), &next_value.rb(), rate),
            ),
             UPDATE_FLAG_ON
            )
        } else {
            if prev.index() as i32 == frame_no {
                (transfer_vertex(&prev.value()), UPDATE_FLAG_ON)
            } else {
                (transfer_vertex(&prev.value()), UPDATE_FLAG_OFF)
            }
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
pub fn evaluate_skew(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataVec2<'_>>>,
    frame_no: i32,
    hint: &mut usize,
) -> Option<(f32, f32, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_value = prev.value().unwrap();
            let prev_index = prev.index() as i32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    return (prev_value.v1(), prev_value.v2(), UPDATE_FLAG_ON)
                }
                return (prev_value.v1(), prev_value.v2(), UPDATE_FLAG_OFF)
            }

            let next_index = next.index() as i32;
            let rate = get_interpolated_rate(interpolation, rate_linear(frame_no as f32, prev_index as f32, next_index as f32));

            let next_value = next.value().unwrap();
            (   linear(prev_value.v1(), next_value.v1(), rate),
                linear(prev_value.v2(), next_value.v2(), rate),
                UPDATE_FLAG_ON
            )
        } else {
            let prev_value = prev.value().unwrap();
            if prev.index() as i32 == frame_no {
                (prev_value.v1(), prev_value.v2(), UPDATE_FLAG_ON)
            } else {
                (prev_value.v1(), prev_value.v2(), UPDATE_FLAG_OFF)
            }
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
    frame_no: i32,
    hint: &mut usize,
) -> Option<(DeformEvaluationData, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    return (transfer_deform(&prev.value()), UPDATE_FLAG_ON)
                }
                return (transfer_deform(&prev.value()), UPDATE_FLAG_OFF)
            }

            let next_index = next.index() as i32;
            let rate = get_interpolated_rate(interpolation, rate_linear(frame_no as f32, prev_index as f32, next_index as f32));

            let mut current = DeformEvaluationData::default();
            let prev_val = prev.value().unwrap();
            let next_val = next.value().unwrap();

            if let (Some(prev_coodinate), Some(next_coodinate)) = (prev_val.coordinate(), next_val.coordinate()) {
                let length_full = prev_coodinate.len();
                current.coordinate.reserve(length_full);
                let length = core::cmp::min(length_full, next_coodinate.len());
                for i in 0..length {
                    current.coordinate.push(runtime::framedata::ss::runtime::Vec2::new(
                        linear(prev_coodinate.get(i).v1(), next_coodinate.get(i).v1(), rate),
                        linear(prev_coodinate.get(i).v2(), next_coodinate.get(i).v2(), rate)
                    ));
                }
                // Overrun use the value of `prev` as is.
                for i in length..length_full {
                    current.coordinate.push(runtime::framedata::ss::runtime::Vec2::new(
                        prev_coodinate.get(i).v1(),
                        prev_coodinate.get(i).v2()
                    ));
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
            if prev.index() as i32 == frame_no {
                (transfer_deform(&prev.value()), UPDATE_FLAG_ON)
            } else {
                (transfer_deform(&prev.value()), UPDATE_FLAG_OFF)
            }
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
pub fn evaluate_bool(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataBoolValue<'_>>>,
    frame_no: i32,
    hint: &mut usize,
) -> Option<bool> {
    search_prev(array, frame_no, |v| v.index() as i32, hint).map(|prev| prev.value())
}

#[inline]
pub fn evaluate_cell<'a>(
    array: flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataCellValue<'a>>>,
    frame_no: i32,
    hint: &mut usize,
) -> Option<ssab::ssab::ss::format::PartAttributeCell<'a>> {
    search_prev(array, frame_no, |v| v.index() as i32, hint).and_then(|prev| prev.value())
}

#[inline]
pub fn evaluate_texture(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataUIntValue<'_>>>,
    frame_no: i32,
    hint: &mut usize,
) -> Option<u32> {
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
        
        let v0 = PartAnimationDataBoolValue::create(&mut builder, &PartAnimationDataBoolValueArgs {
            index: 0,
            value: true,
        });
        let v10 = PartAnimationDataBoolValue::create(&mut builder, &PartAnimationDataBoolValueArgs {
            index: 10,
            value: false,
        });
        let values_off = builder.create_vector(&[v0, v10]);
        builder.finish(values_off, None);
        
        let buf = builder.finished_data();
        let vector = flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<PartAnimationDataBoolValue>>>(buf).unwrap();

        let mut hint = 0;
        // Exact match
        assert_eq!(evaluate_bool(vector, 0, &mut hint), Some(true));
        // Interpolated (should use prev)
        assert_eq!(evaluate_bool(vector, 5, &mut hint), Some(true));
        // Exact match
        assert_eq!(evaluate_bool(vector, 10, &mut hint), Some(false));
        // Beyond last key
        assert_eq!(evaluate_bool(vector, 15, &mut hint), Some(false));
        // Before first key
        assert_eq!(evaluate_bool(vector, -1, &mut hint), None);
    }

    #[test]
    fn test_evaluate_texture() {
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        
        let v0 = PartAnimationDataUIntValue::create(&mut builder, &PartAnimationDataUIntValueArgs {
            index: 0,
            value: 100,
            interpolation: InterpolationType::None,
        });
        let v10 = PartAnimationDataUIntValue::create(&mut builder, &PartAnimationDataUIntValueArgs {
            index: 10,
            value: 200,
            interpolation: InterpolationType::None,
        });
        let values_off = builder.create_vector(&[v0, v10]);
        builder.finish(values_off, None);
        
        let buf = builder.finished_data();
        let vector = flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<PartAnimationDataUIntValue>>>(buf).unwrap();

        let mut hint = 0;
        assert_eq!(evaluate_texture(vector, 0, &mut hint), Some(100));
        assert_eq!(evaluate_texture(vector, 5, &mut hint), Some(100));
        assert_eq!(evaluate_texture(vector, 10, &mut hint), Some(200));
        assert_eq!(evaluate_texture(vector, 15, &mut hint), Some(200));
    }

    #[test]
    fn test_evaluate_int_no_interpolation() {
        use ssab::ssab::ss::format::{PartAnimationDataIntValue, PartAnimationDataIntValueArgs};
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        
        let v0 = PartAnimationDataIntValue::create(&mut builder, &PartAnimationDataIntValueArgs {
            index: 0,
            value: 100,
            interpolation: InterpolationType::None,
        });
        let v10 = PartAnimationDataIntValue::create(&mut builder, &PartAnimationDataIntValueArgs {
            index: 10,
            value: 200,
            interpolation: InterpolationType::None,
        });
        let values_off = builder.create_vector(&[v0, v10]);
        builder.finish(values_off, None);
        
        let buf = builder.finished_data();
        let vector = flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<PartAnimationDataIntValue>>>(buf).unwrap();

        let mut hint = 0;
        // At frame 0, should be ON and 100
        let (val, flags) = evaluate_int(vector, 0, &mut hint).unwrap();
        assert_eq!(val, 100);
        assert!(!flags.is_empty());

        // At frame 5, should be OFF (inherited) and 100
        let (val, flags) = evaluate_int(vector, 5, &mut hint).unwrap();
        assert_eq!(val, 100);
        assert!(flags.is_empty());

        // At frame 10, should be ON and 200
        let (val, flags) = evaluate_int(vector, 10, &mut hint).unwrap();
        assert_eq!(val, 200);
        assert!(!flags.is_empty());
    }

    #[test]
    fn test_evaluate_cell() {
        use ssab::ssab::ss::format::{PartAnimationDataCellValue, PartAnimationDataCellValueArgs, PartAttributeCell, PartAttributeCellArgs};
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        
        let cell_a = PartAttributeCell::create(&mut builder, &PartAttributeCellArgs {
            map_id: 1,
            name_hash: 111,
        });
        let v0 = PartAnimationDataCellValue::create(&mut builder, &PartAnimationDataCellValueArgs {
            index: 0,
            value: Some(cell_a),
        });
        
        let cell_b = PartAttributeCell::create(&mut builder, &PartAttributeCellArgs {
            map_id: 2,
            name_hash: 222,
        });
        let v10 = PartAnimationDataCellValue::create(&mut builder, &PartAnimationDataCellValueArgs {
            index: 10,
            value: Some(cell_b),
        });
        
        let values_off = builder.create_vector(&[v0, v10]);
        builder.finish(values_off, None);
        
        let buf = builder.finished_data();
        let vector = flatbuffers::root::<flatbuffers::Vector<flatbuffers::ForwardsUOffset<PartAnimationDataCellValue>>>(buf).unwrap();

        let mut hint = 0;
        let res = evaluate_cell(vector, 5, &mut hint).unwrap();
        assert_eq!(res.map_id(), 1);
        assert_eq!(res.name_hash(), 111);

        let res = evaluate_cell(vector, 10, &mut hint).unwrap();
        assert_eq!(res.map_id(), 2);
        assert_eq!(res.name_hash(), 222);
    }
}
