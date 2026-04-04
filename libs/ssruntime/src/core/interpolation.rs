use ssab::ssab::ss::format::{InterpolationType, SsAttributePartColorKeyValueColor};
use ssinterpolate::interpolate::*;
use crate::runtime;

const UPDATE_FLAG_ON: runtime::framedata::ss::runtime::UpdateAttributeFlags = runtime::framedata::ss::runtime::UpdateAttributeFlags::empty();
const UPDATE_FLAG_OFF: runtime::framedata::ss::runtime::UpdateAttributeFlags = runtime::framedata::ss::runtime::UpdateAttributeFlags::all();

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

// Optimized for forward playback (O(1) fast path). Reverse or ping-pong playback 
// currently fallbacks to binary search (O(log N)). Bidirectional optimization 
// can be implemented if performance bottlenecks occur in those modes.
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
    
    let curr_val = array.get(i);
    let curr_idx = get_index(&curr_val);
    
    if curr_idx <= frame_no {
        if i + 1 < len {
            let next_val = array.get(i + 1);
            let next_idx = get_index(&next_val);
            if frame_no < next_idx {
                *hint = i;
                return Some((curr_val, Some(next_val)));
            }
        } else {
            *hint = i;
            return Some((curr_val, None));
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

// Optimized for forward playback (O(1) fast path). Reverse or ping-pong playback 
// currently fallbacks to binary search (O(log N)). Bidirectional optimization 
// can be implemented if performance bottlenecks occur in those modes.
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
    
    let curr_val = array.get(i);
    let curr_idx = get_index(&curr_val);
    
    if curr_idx <= frame_no {
        if i + 1 < len {
            let next_val = array.get(i + 1);
            let next_idx = get_index(&next_val);
            if frame_no < next_idx {
                *hint = i;
                return Some(curr_val);
            }
        } else {
            *hint = i;
            return Some(curr_val);
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
        Some(array.get(0))
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
            (prev.value(), UPDATE_FLAG_OFF)
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
            (prev.value(), UPDATE_FLAG_OFF)
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
            (transfer_partcolor(&prev.value()), UPDATE_FLAG_OFF)
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
            (transfer_shader(&prev.value()), UPDATE_FLAG_OFF)
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
            (transfer_vertex(&prev.value()), UPDATE_FLAG_OFF)
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
            (prev_value.v1(), prev_value.v2(), UPDATE_FLAG_OFF)
        }
    })
}

#[inline]
pub fn evaluate_deform<'a>(
    fb: &mut flatbuffers::FlatBufferBuilder<'a>,
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataDeform<'_>>>,
    frame_no: i32,
    hint: &mut usize,
) -> Option<(::flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>>, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32, hint).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    return (transfer_deform(fb, &prev.value()), UPDATE_FLAG_ON)
                }
                return (transfer_deform(fb, &prev.value()), UPDATE_FLAG_OFF)
            }

            let next_index = next.index() as i32;
            let rate = get_interpolated_rate(interpolation, rate_linear(frame_no as f32, prev_index as f32, next_index as f32));

            let mut current = runtime::framedata::ss::runtime::PartAttributeDeformArgs::default();
            let prev_val = prev.value().unwrap();
            let next_val = next.value().unwrap();

            if let (Some(prev_coodinate), Some(next_coodinate)) = (prev_val.coordinate(), next_val.coordinate()) {
                let length_full = prev_coodinate.len();
                let mut coordinate = Vec::with_capacity(length_full);
                let length = core::cmp::min(length_full, next_coodinate.len());
                for i in 0..length {
                    coordinate.push(runtime::framedata::ss::runtime::Vec2::new(
                        linear(prev_coodinate.get(i).v1(), next_coodinate.get(i).v1(), rate),
                        linear(prev_coodinate.get(i).v2(), next_coodinate.get(i).v2(), rate)
                    ));
                }
                // Overrun use the value of `prev` as is.
                for i in length..length_full {
                    coordinate.push(runtime::framedata::ss::runtime::Vec2::new(
                        prev_coodinate.get(i).v1(),
                        prev_coodinate.get(i).v2()
                    ));
                }
                current.coordinate = Some(fb.create_vector(&coordinate));

                // "index" is not interpolated.
                current.index = prev_val.index().map(|index| {
                    let v = index.iter().collect::<Vec<_>>();
                    fb.create_vector(&v)
                });

                (runtime::framedata::ss::runtime::PartAttributeDeform::create(fb, &current), UPDATE_FLAG_ON)
            } else {
                (transfer_deform(fb, &prev.value()), UPDATE_FLAG_ON)
            }
        } else {
            (transfer_deform(fb, &prev.value()), UPDATE_FLAG_OFF)
        }
    })
}

#[inline]
fn transfer_deform<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, source: &Option<ssab::ssab::ss::format::PartAttributeDeform>) -> ::flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform<'a>> {
    let mut destination = runtime::framedata::ss::runtime::PartAttributeDeformArgs::default();
    if let Some(s) = source {
        destination.coordinate = s.coordinate().map(|coordinate| {
            let v = coordinate.iter().map(|v| runtime::framedata::ss::runtime::Vec2::new(v.v1(), v.v2())).collect::<Vec<_>>();
            fb.create_vector(&v)
        });
        destination.index = s.index().map(|index| {
            let v = index.iter().collect::<Vec<_>>();
            fb.create_vector(&v)
        });
    }

    runtime::framedata::ss::runtime::PartAttributeDeform::create(fb, &destination)
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

    #[test]
    fn test_linear_interpolation() {
        assert_eq!(linear(0.0, 10.0, 0.5), 5.0);
        assert_eq!(linear(10.0, 20.0, 0.0), 10.0);
        assert_eq!(linear(10.0, 20.0, 1.0), 20.0);
    }
}
