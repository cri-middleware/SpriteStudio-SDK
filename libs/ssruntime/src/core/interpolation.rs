use ssab::ssab::ss::format::InterpolationType;
use ssinterpolate::interpolate::*;

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
    frame_no: i32,
) -> Option<f32> {
    search_prev_next(array, frame_no, |v| v.index() as i32).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                return prev.value();
            }

            let prev_index = prev.index() as i32;
            let next_index = next.index() as i32;
            let rate = (frame_no - prev_index) as f32 / (next_index - prev_index) as f32;
            let eased_rate = get_interpolated_rate(interpolation, rate);
            
            linear(prev.value(), next.value(), eased_rate)
        } else {
            prev.value()
        }
    })
}

#[inline]
pub fn evaluate_int(
    array: flatbuffers::Vector<'_, flatbuffers::ForwardsUOffset<ssab::ssab::ss::format::PartAnimationDataIntValue<'_>>>,
    frame_no: i32,
) -> Option<i32> {
    search_prev_next(array, frame_no, |v| v.index() as i32).map(|(prev, next_opt)| {
        if let Some(next) = next_opt {
            let interpolation = prev.interpolation();
            if interpolation == InterpolationType::None {
                return prev.value();
            }

            let prev_index = prev.index() as i32;
            let next_index = next.index() as i32;
            let rate = (frame_no - prev_index) as f32 / (next_index - prev_index) as f32;
            let eased_rate = get_interpolated_rate(interpolation, rate);
            
            linear(prev.value() as f32, next.value() as f32, eased_rate) as i32
        } else {
            prev.value()
        }
    })
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
