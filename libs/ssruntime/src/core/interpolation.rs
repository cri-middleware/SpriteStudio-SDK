use std::cmp::max;
use ssab::ssab::ss::format::{InterpolationType, SsAttributePartColorKeyValueColor};
use ssinterpolate::interpolate::*;
use crate::runtime;

const UPDATE_FLAG_ON: runtime::framedata::ss::runtime::UpdateAttributeFlags = runtime::framedata::ss::runtime::UpdateAttributeFlags::empty();
const UPDATE_FLAG_OFF: runtime::framedata::ss::runtime::UpdateAttributeFlags = runtime::framedata::ss::runtime::UpdateAttributeFlags::all();

#[macro_use]

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
        // Linear and baked types (Hermite, Bezier, etc.) fallback to linear rate
        // ssconverter bakes Hermite, Bezier, and standard EaseIn/Out.
        _ => rate,
    }
}

#[inline]
pub fn search_prev_next<'a, T, F>(
    array: flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<T>>,
    frame_no: i32,
    get_index: F,
) -> Option<(T::Inner, Option<T::Inner>)>
where
    T: 'a + flatbuffers::Follow<'a>,
    F: Fn(&T::Inner) -> i32,
{
    if array.is_empty() { return None; }
    
    let mut left = 0;
    let mut right = array.len();
    while left < right {
        let mid = left + (right - left) / 2;
        let mid_val = array.get(mid);
        if get_index(&mid_val) <= frame_no {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    if left == 0 {
        return Some((array.get(0), None));
    }
    
    let prev = array.get(left - 1);
    if get_index(&prev) == frame_no || left == array.len() {
        return Some((prev, None));
    }
    
    let next = array.get(left);
    Some((prev, Some(next)))
}

#[inline]
pub fn search_prev<'a, T, F>(
    array: flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<T>>,
    frame_no: i32,
    get_index: F,
) -> Option<T::Inner>
where
    T: 'a + flatbuffers::Follow<'a>,
    F: Fn(&T::Inner) -> i32,
{
    if array.is_empty() { return None; }
    
    let mut left = 0;
    let mut right = array.len();
    while left < right {
        let mid = left + (right - left) / 2;
        let mid_val = array.get(mid);
        if get_index(&mid_val) <= frame_no {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    
    if left == 0 {
        Some(array.get(0))
    } else {
        Some(array.get(left - 1))
    }
}

#[inline]
pub fn evaluate_float(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataFloatValue<'_>>>,
    frame_no: i32
) -> Option<(f32, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    // TODO: Think methods to improve. (Always be true even when called multiple times consecutively with same frame_no.)
                    return (prev.value(), UPDATE_FLAG_ON)
                }
                return (prev.value(), UPDATE_FLAG_OFF)
            }

            let next_index = next.index() as i32;
            let rate = get_interpolated_rate(interpolation, rate_linear(frame_no as f32, prev_index:f32, next_index:f32));

            (linear(prev.value(), next.value(), rate), UPDATE_FLAG_ON)
        } else {
            (prev.value(), UPDATE_FLAG_OFF)
        }
    })
}

#[inline]
pub fn evaluate_int(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataIntValue<'_>>>,
    frame_no: i32
) -> Option<(i32, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    // TODO: Think methods to improve. (Always be true even when called multiple times consecutively with same frame_no.)
                    return (prev.value(), UPDATE_FLAG_ON)
                }
                return (prev.value(), UPDATE_FLAG_OFF)
            }

            let next_index = next.index() as i32;
            let rate = get_interpolated_rate(interpolation, rate_linear(frame_no as f32, prev_index:f32, next_index:f32));

            (libm::roundf(linear(prev.value() as f32, next.value() as f32, rate)) as i32, UPDATE_FLAG_ON)
        } else {
            (prev.value(), UPDATE_FLAG_OFF)
        }
    })
}

#[inline]
pub fn evaluate_partcolor(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataPartColor<'_>>>,
    frame_no: i32
) -> Option<(runtime::framedata::ss::runtime::PartAttributePartColor, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    // TODO: Think methods to improve. (Always be true even when called multiple times consecutively with same frame_no.)
                    return (transfer_partcolor(&prev.value()), UPDATE_FLAG_ON)
                }
                return (transfer_partcolor(&prev.value()), UPDATE_FLAG_OFF)
            }

            let next_index = next.index() as i32;
            let rate = get_interpolated_rate(interpolation, rate_linear(frame_no as f32, prev_index:f32, next_index:f32));

            fn interpolate_color(color_start: &Option<SsAttributePartColorKeyValueColor>, color_end: &Option<SsAttributePartColorKeyValueColor>, rate:f32) -> runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor {
                runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(
                    &runtime::framedata::ss::runtime::U8Rect::new(  linear(color_start.a() as f32, color_end.a() as f32, rate) as u8,
                                                                    linear(color_start.r() as f32, color_end.r() as f32, rate) as u8,
                                                                    linear(color_start.g() as f32, color_end.g() as f32, rate) as u8,
                                                                    linear(color_start.b() as f32, color_end.b() as f32, rate) as u8
                    ),
                    linear(color_start.rate(), color_end.rate(), rate)
                )
            }

            let mut current = runtime::framedata::ss::runtime::PartAttributePartColor::default();
            let prev_value = &prev.value().unwrap();
            let next_value = &next.value().unwrap();
            current.set_lt(&interpolate_color(&prev_value.lt(), &next_value.lt(), rate));
            current.set_rt(&interpolate_color(&prev_value.rt(), &next_value.rt(), rate));
            current.set_rb(&interpolate_color(&prev_value.rb(), &next_value.rb(), rate));
            current.set_lb(&interpolate_color(&prev_value.lb(), &next_value.lb(), rate));

            (current, UPDATE_FLAG_ON)
        } else {
            (transfer_partcolor(&prev.value()), UPDATE_FLAG_OFF)
        }
    })
}
#[inline]
fn transfer_partcolor(source: & Option<ssab::ssab::ss::format::PartAttributePartColor>) -> runtime::framedata::ss::runtime::PartAttributePartColor {
    let mut destination = runtime::framedata::ss::runtime::PartAttributePartColor::default();
    let source_t = source.value().unwrap().unpack();

    destination.set_blend_type( match source_t.blend_type {
            ssab::ssab::ss::format::BlendType::Mix => runtime::framedata::ss::runtime::BlendType::Mix,
            ssab::ssab::ss::format::BlendType::Mul => runtime::framedata::ss::runtime::BlendType::Mul,
            ssab::ssab::ss::format::BlendType::Add => runtime::framedata::ss::runtime::BlendType::Add,
            ssab::ssab::ss::format::BlendType::Sub => runtime::framedata::ss::runtime::BlendType::Sub,
            // MEMO: Blending methods that "Part-Color" doesn't have.
//            ssab::ssab::ss::format::BlendType::Mulalpha => runtime::framedata::ss::runtime::BlendType::Mulalpha,
//            ssab::ssab::ss::format::BlendType::Screen => runtime::framedata::ss::runtime::BlendType::Screen,
//            ssab::ssab::ss::format::BlendType::Exclusion => runtime::framedata::ss::runtime::BlendType::Exclusion,
//            ssab::ssab::ss::format::BlendType::Invert => runtime::framedata::ss::runtime::BlendType::Invert,
//            ssab::ssab::ss::format::BlendType::Mul2 => runtime::framedata::ss::runtime::BlendType::Mul2,
//            ssab::ssab::ss::format::BlendType::Div2 => runtime::framedata::ss::runtime::BlendType::Div2,
//            ssab::ssab::ss::format::BlendType::Screen2 => runtime::framedata::ss::runtime::BlendType::Screen2,
//            ssab::ssab::ss::format::BlendType::Overlay2 => runtime::framedata::ss::runtime::BlendType::Overlay2,
            _ => runtime::framedata::ss::runtime::BlendType::Mix,
        });

    fn convert_u8rect(rgba: &ssab::ssab::ss::format::U8Rect) -> runtime::framedata::ss::runtime::U8Rect {
        runtime::framedata::ss::runtime::U8Rect::new(rgba.a(), rgba.r(), rgba.g(), rgba.b())
    }
    let key_color =source.lt();
    destination.set_lt(runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&key_color), key_color.unwrap().rate));
    key_color =source.rt();
    destination.set_rt(runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&key_color), key_color.unwrap().rate));
    key_color =source.rb();
    destination.set_rb(runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&key_color), key_color.unwrap().rate));
    key_color =source.lb();
    destination.set_lb(runtime::framedata::ss::runtime::SsAttributePartColorKeyValueColor::new(&convert_u8rect(&key_color), key_color.unwrap().rate));

    destination
}

#[inline]
pub fn evaluate_shader(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataShader<'_>>>,
    frame_no: i32
) -> Option<(runtime::framedata::ss::runtime::PartAttributeShader, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    // TODO: Think methods to improve. (Always be true even when called multiple times consecutively with same frame_no.)
                    return (transfer_shader(&prev.value()), UPDATE_FLAG_ON)
                }
                return (transfer_shader(&prev.value()), UPDATE_FLAG_OFF)
            }

            let next_index = next.index() as i32;
            let rate = get_interpolated_rate(interpolation, rate_linear(frame_no as f32, prev_index:f32, next_index:f32));

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
fn transfer_shader(source: & Option<ssab::ssab::ss::format::PartAttributeShader>) -> runtime::framedata::ss::runtime::PartAttributeShader {
    runtime::framedata::ss::runtime::PartAttributeShader::new(
        source.id_hash(),
        source.param0(),
        source.param1(),
        source.param2(),
        source.param3(),
        source.param4(),
        source.param5(),
        source.param6(),
        source.param7(),
        source.map0_id(),
        source.map0_name_hash(),
        source.map1_id(),
        source.map1_name_hash()
    )
}

#[inline]
pub fn evaluate_vertex(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataVertex<'_>>>,
    frame_no: i32
) -> Option<(runtime::framedata::ss::runtime::PartAttributeVertex, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    // TODO: Think methods to improve. (Always be true even when called multiple times consecutively with same frame_no.)
                    return (transfer_vertex(&prev.value()), UPDATE_FLAG_ON)
                }
                return (transfer_vertex(&prev.value()), UPDATE_FLAG_OFF)
            }

            let next_index = next.index() as i32;
            let rate = get_interpolated_rate(interpolation, rate_linear(frame_no as f32, prev_index:f32, next_index:f32));

            fn interpolate_coordinate(coordinate_start: &Option<ssab::ssab::ss::format::Vec2>, coordinate_end: &Option<ssab::ssab::ss::format::Vec2>, rate:f32) -> runtime::framedata::ss::runtime::Vec2 {
                let coordinate_start_t = coordinate_start.unwrap();
                let coordinate_end_t = coordinate_end.unwrap();
                
                runtime::framedata::ss::runtime::Vec2::new( linear(coordinate_start_t.v1(), coordinate_end_t.v1(), rate),
                                                            linear(coordinate_start_t.v2(), coordinate_end_t.v2(), rate)
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
fn transfer_vertex(source: & Option<ssab::ssab::ss::format::PartAttributeVertex>) -> runtime::framedata::ss::runtime::PartAttributeVertex {
    let lt = source.lt().map(|lt| runtime::framedata::ss::runtime::Vec2::new(lt.v1(), lt.v2())).unwrap_or_default();
    let rt = source.rt().map(|rt| runtime::framedata::ss::runtime::Vec2::new(rt.v1(), rt.v2())).unwrap_or_default();
    let lb = source.lb().map(|lb| runtime::framedata::ss::runtime::Vec2::new(lb.v1(), lb.v2())).unwrap_or_default();
    let rb = source.rb().map(|rb| runtime::framedata::ss::runtime::Vec2::new(rb.v1(), rb.v2())).unwrap_or_default();

    runtime::framedata::ss::runtime::PartAttributeVertex::new(lt, rt, lb, rb)
}

#[inline]
pub fn evaluate_skew(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataVec2<'_>>>,
    frame_no: i32
) -> Option<(f32, f32, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_value = prev.value().unwrap();
            let prev_index = prev.index() as i32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    // TODO: Think methods to improve. (Always be true even when called multiple times consecutively with same frame_no.)
                    return (prev_value.v1(), prev_value.v2(), UPDATE_FLAG_ON)
                }
                return (prev_value.v1(), prev_value.v2(), UPDATE_FLAG_OFF)
            }

            let next_index = next.index() as i32;
            let rate = get_interpolated_rate(interpolation, rate_linear(frame_no as f32, prev_index:f32, next_index:f32));

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
    frame_no: i32
) -> Option<(::flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform>, runtime::framedata::ss::runtime::UpdateAttributeFlags)> {
    search_prev_next(array, frame_no, |v| v.index() as i32).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let prev_index = prev.index() as i32;
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                if prev_index == frame_no {
                    // TODO: Think methods to improve. (Always be true even when called multiple times consecutively with same frame_no.)
                    return (transfer_deform(fb, &prev.value()), UPDATE_FLAG_ON)
                }
                return (transfer_deform(fb, &prev.value()), UPDATE_FLAG_OFF)
            }

            let next_index = next.index() as i32;
            let rate = get_interpolated_rate(interpolation, rate_linear(frame_no as f32, prev_index:f32, next_index:f32));

            let mut current = runtime::framedata::ss::runtime::PartAttributeDeformArgs::default();
            let prev_value = prev.value().unwrap();
            let next_value = next.value().unwrap();

            if let Some(prev_coodinate) = prev_value.coordinate() && let Some(next_coodinate) = next_value.coordinate() {
                // MEMO: Originally, lengths of prev_coordinate and next_coordinate are the same.
                let length_full = prev_coodinate.len();
                let mut coordinate: [runtime::framedata::ss::runtime::Vec2; length_full];
                let length = max(length_full, next_coodinate.len());
                for i in 0..length {
                    coordinate[i].set_x(linear(prev_coodinate[i].x, next_coodinate[i].x, rate));
                    coordinate[i].set_y(linear(prev_coodinate[i].y, next_coodinate[i].y, rate));
                }
                // MEMO: Overrun use the value of `prev` as is.
                for i in length..length_full {
                    coordinate[i] = prev_coodinate[i];
                }
                fb.create_vector(&coordinate);

                // MEMO: "index" is not interpolated.
                current.index = prev_value.index().map(|index| {
                    let v = index.iter().map(|v| v).collect::<Vec<_>>();
                    fb.create_vector(&v)
                });

                (runtime::framedata::ss::runtime::PartAttributeDeform::create(fb, &current), UPDATE_FLAG_ON)
            } else {
                // MEMO: (I think) Will not be run much, but duplicate prev for error trapping.
                (transfer_deform(fb, &prev.value()), UPDATE_FLAG_ON)
            }
        } else {
            (transfer_deform(fb, &prev.value()), UPDATE_FLAG_OFF)
        }
    })
}
#[inline]
fn transfer_deform<'a>(fb: &mut flatbuffers::FlatBufferBuilder<'a>, source: & Option<ssab::ssab::ss::format::PartAttributeDeform>) -> ::flatbuffers::WIPOffset<runtime::framedata::ss::runtime::PartAttributeDeform> {
    let mut destination = runtime::framedata::ss::runtime::PartAttributeDeformArgs::default();
    destination.coordinate = source.coordinate().map(|coordinate| {
        let v = coordinate.iter().map(|v| runtime::framedata::ss::runtime::Vec2::new(v.v1(), v.v2())).collect::<Vec<_>>();
        fb.create_vector(&v)
    });
    destination.index = source.index().map(|index| {
        let v = index.iter().map(|v| v).collect::<Vec<_>>();
        fb.create_vector(&v)
    });

    runtime::framedata::ss::runtime::PartAttributeDeform::create(fb, &destination)
}

// For discrete values, we just return the previous value.
#[inline]
pub fn evaluate_bool(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataBoolValue<'_>>>,
    frame_no: i32,
) -> Option<bool> {
    search_prev(array, frame_no, |v| v.index() as i32).map(|prev| prev.value())
}

#[inline]
pub fn evaluate_cell<'a>(
    array: flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataCellValue<'a>>>,
    frame_no: i32,
) -> Option<ssab::ssab::ss::format::PartAttributeCell<'a>> {
    search_prev(array, frame_no, |v| v.index() as i32).and_then(|prev| prev.value())
}

#[inline]
pub fn evaluate_texture(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataUIntValue<'_>>>,
    frame_no: i32,
) -> Option<u32> {
    search_prev(array, frame_no, |v| v.index() as i32).map(|prev| prev.value())
}

// PartColor is complex because it has 4 corners, and RGBA + rate.
// For now, if there is interpolation, we need to interpolate each color component.
// But we return an Option to the *evaluated* state. Wait, the return type of `PartColor` would be difficult because `PartAttributePartColor` is a flatbuffers table.
// It might be easier to implement the evaluation directly in `framedata.rs` where the `PartState` is constructed, or return a struct representing the evaluated value.
