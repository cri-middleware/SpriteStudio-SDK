//! The evaluation core — everything that turns `.ssab` data plus a time into drawable state.
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`playback_controller`] | Transport, loop counting, ping-pong, seek, FPS and playback range. |
//! | [`interpolation`] | Keyframe curve evaluation (delegates the maths to [`crate::interpolate`]). |
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
pub mod keyframes;
pub mod math;
pub mod matrix;
pub mod override_layer;
pub mod playback_controller;
pub mod ssab_lookup;
pub mod z_order;

/// The Y direction of what the runtime emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SsCoordinateSystem {
    /// Y grows upward, as SpriteStudio authors it. `0` in the C-API.
    #[default]
    YUp,
    /// Y grows downward: Y coordinates and rotation are flipped. `1` in the C-API.
    YDown,
}

/// Whether a part's Skew (shear) inherits down to its children.
/// true == inherits down to children.
/// false == does not inherit.
pub const SKEW_PROPAGATES: bool = false;

/// Where the positions the runtime emits are measured from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SsOrigin {
    /// The canvas center. `0` in the C-API.
    #[default]
    Center,
    /// The canvas's top-left corner: root parts are offset by half the canvas. `1`
    /// in the C-API.
    TopLeft,
}

/// The unit of the part positions the runtime emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SsUnit {
    /// Pixels. `0` in the C-API.
    #[default]
    Pixel,
    /// Each part's position is divided by the canvas size. Sizes, local vertices and
    /// UVs stay in pixels. `1` in the C-API.
    Normalized,
}

/// What a frame computes, by how the Player draws it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    /// World matrices and mesh vertices are computed. `0` in the C-API.
    #[default]
    Direct,
    /// World matrices are not computed, for a Player that composes parts in its own
    /// scene graph. `1` in the C-API.
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
