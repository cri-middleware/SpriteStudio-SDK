pub mod effect;
pub mod framedata;
pub mod interpolation;
pub mod math;
pub mod matrix;
pub mod override_layer;
pub mod playback_controller;
pub mod ssab_lookup;
pub mod util;
pub mod z_order;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    #[default]
    Direct,
    SceneGraph,
}

impl RenderMode {
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => RenderMode::SceneGraph,
            _ => RenderMode::Direct,
        }
    }
}

pub use matrix::{SsMatrix4x4, SsVector3};
pub mod geometry;
pub use geometry::*;
