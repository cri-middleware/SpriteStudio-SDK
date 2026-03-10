
pub fn required_precalculation(interpolation:ssab::ssab::ss::format::InterpolationType) -> bool {
    // Determines whether to calculate in runtime or pre-interpolation.
    match interpolation {
        ssab::ssab::ss::format::InterpolationType::None => false,
        ssab::ssab::ss::format::InterpolationType::Linear => false,
        ssab::ssab::ss::format::InterpolationType::Hermite => true,
        ssab::ssab::ss::format::InterpolationType::Bezier => true,
        ssab::ssab::ss::format::InterpolationType::Acceleration => false,
        ssab::ssab::ss::format::InterpolationType::Deceleration => false,
        ssab::ssab::ss::format::InterpolationType::EaseIn => true,
        ssab::ssab::ss::format::InterpolationType::EaseOut => true,
        ssab::ssab::ss::format::InterpolationType::EaseInOut => true,
        ssab::ssab::ss::format::InterpolationType::EaseExponentialIn => false,
        ssab::ssab::ss::format::InterpolationType::EaseExponentialOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseExponentialInOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseSineIn => false,
        ssab::ssab::ss::format::InterpolationType::EaseSineOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseSineInOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseElasticIn => true,
        ssab::ssab::ss::format::InterpolationType::EaseElasticOut => true,
        ssab::ssab::ss::format::InterpolationType::EaseElasticInOut => true,
        ssab::ssab::ss::format::InterpolationType::EaseBounceIn => false,
        ssab::ssab::ss::format::InterpolationType::EaseBounceOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseBounceInOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseBackIn => false,
        ssab::ssab::ss::format::InterpolationType::EaseBackOut => false,
        ssab::ssab::ss::format::InterpolationType::EaseBackInOut => false,
        _ => false,
    }
}

pub fn omittable_precalculation<T>(interpolation:ssab::ssab::ss::format::InterpolationType, value_start:&T, value_end:&T) -> bool {
    // Determine whether the value changes during range with start-value, end-value and formula.
    match interpolation {
        ssab::ssab::ss::format::InterpolationType::None => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::Linear => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::Hermite => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::Bezier => {
            false
        }
        ssab::ssab::ss::format::InterpolationType::Acceleration => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::Deceleration => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseIn => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseOut => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseInOut => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseExponentialIn => {
            false
        }
        ssab::ssab::ss::format::InterpolationType::EaseExponentialOut => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseExponentialInOut => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseSineIn => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseSineOut => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseSineInOut => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseElasticIn => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseElasticOut => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseElasticInOut => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseBounceIn => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseBounceOut => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseBounceInOut => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseBackIn => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseBackOut => {
            false
        },
        ssab::ssab::ss::format::InterpolationType::EaseBackInOut => {
            false
        },
        _ => false,
    }
}
