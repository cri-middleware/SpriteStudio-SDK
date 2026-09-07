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

/// Whether a part's Skew (shear) inherits down to its children.
///
/// SpriteStudio authors Skew as a mode of the Vertex-deform attribute — a
/// per-part quad deformation that does **not** propagate to children (verified
/// in the editor: a child of a skewed parent stays undeformed). So Skew is
/// applied at the local-vertex stage (`compute_local_vertices_for_parts`, the
/// shape pass and the nine-slice pass) and is deliberately kept out of the
/// world / inheritance matrix, which would otherwise carry it down the tree.
///
/// If a future SpriteStudio makes Skew propagate, flip this to `true`: the
/// local-vertex shear is then skipped ([`geometry::skew_local_tans`] returns
/// `None`) and the matrix path in [`matrix::SsMatrix4x4::compute_world_matrix`]
/// is re-enabled instead. Nothing else has to move.
pub const SKEW_PROPAGATES: bool = false;

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
