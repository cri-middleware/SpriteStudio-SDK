pub mod effect;
pub mod framedata;
pub mod interpolation;
pub mod math;
pub mod matrix;
pub mod playback_controller;
pub mod util;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SsCoordinateSystem {
    #[default]
    YUp,
    YDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SsOrigin {
    #[default]
    Center,
    TopLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SsUnit {
    #[default]
    Pixel,
    Normalized,
}

pub use matrix::{SsMatrix4x4, SsVector3};
pub mod geometry;
pub use geometry::*;
