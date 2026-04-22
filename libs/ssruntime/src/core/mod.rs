pub mod effect;
pub mod framedata;
pub mod interpolation;
pub mod matrix;
pub mod playback_controller;
pub mod util;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SsCoordinateSystem {
    #[default]
    YUp,
    YDown,
}

pub use matrix::{SsMatrix4x4, SsVector3};
