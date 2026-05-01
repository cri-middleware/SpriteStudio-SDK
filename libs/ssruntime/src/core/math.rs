#[cfg(not(feature = "std"))]
pub use libm::{atan2f, cosf, fabsf, floorf, fmodf, roundf, sinf, sqrtf, tanf};

#[cfg(feature = "std")]
#[inline]
pub fn tanf(f: f32) -> f32 {
    f.tan()
}

#[cfg(feature = "std")]
#[inline]
pub fn cosf(f: f32) -> f32 {
    f.cos()
}

#[cfg(feature = "std")]
#[inline]
pub fn sinf(f: f32) -> f32 {
    f.sin()
}

#[cfg(feature = "std")]
#[inline]
pub fn fabsf(f: f32) -> f32 {
    f.abs()
}

#[cfg(feature = "std")]
#[inline]
pub fn roundf(f: f32) -> f32 {
    f.round()
}

#[cfg(feature = "std")]
#[inline]
pub fn floorf(f: f32) -> f32 {
    f.floor()
}

#[cfg(feature = "std")]
#[inline]
pub fn fmodf(f: f32, side: f32) -> f32 {
    f % side
}

#[cfg(feature = "std")]
#[inline]
pub fn sqrtf(f: f32) -> f32 {
    f.sqrt()
}

#[cfg(feature = "std")]
#[inline]
pub fn atan2f(y: f32, x: f32) -> f32 {
    y.atan2(x)
}
