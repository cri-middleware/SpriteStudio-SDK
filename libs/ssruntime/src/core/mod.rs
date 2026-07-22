//! The evaluation core — everything that turns `.ssab` data plus a time into drawable state.
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`playback_controller`] | Transport, loop counting, ping-pong, seek, FPS and playback range. |
//! | [`interpolation`] | Keyframe curve evaluation (delegates the maths to `ssinterpolate`). |
//! | [`matrix`] / [`geometry`] | Transform composition and vertex/mesh geometry. |
//! | [`framedata`] | Builds the per-frame FlatBuffer and the DrawBatch plan a Player consumes. |
//! | [`override_layer`] | Per-part runtime overrides of color, cell and visibility. |
//! | [`effect`] | Particle effect simulation. |
//! | [`z_order`] | Draw-order resolution. |
//! | [`ssab_lookup`] | Name/hash lookups into the loaded binary. |
//!
//! The enums defined here — [`SsCoordinateSystem`], [`SsOrigin`], [`SsUnit`] and
//! [`RenderMode`] — are the knobs that adapt output to a host engine's conventions.
//! Getting them right is usually the first thing a new Player has to do: a Y-down
//! engine fed Y-up output renders upside down.

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
