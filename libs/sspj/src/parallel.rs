//! Parallelism abstraction.
//!
//! Production code does `use crate::parallel::*;` and then uses [`par_iter!`]
//! and `join` instead of touching rayon directly. With the `parallel` feature
//! (the default for native builds) these map onto rayon — the glob re-export
//! also brings rayon's prelude traits into scope so the downstream `map` /
//! `collect` / `zip` adapters resolve. Without the feature they fall back to
//! sequential iteration, which lets this crate compile for
//! `wasm32-unknown-unknown`, where rayon cannot spawn threads.

#[cfg(feature = "parallel")]
pub(crate) use rayon::join;
#[cfg(feature = "parallel")]
pub(crate) use rayon::prelude::*;

/// Sequential fallback for [`rayon::join`]: runs the two closures in order.
#[cfg(not(feature = "parallel"))]
pub(crate) fn join<A, B, RA, RB>(oper_a: A, oper_b: B) -> (RA, RB)
where
    A: FnOnce() -> RA,
    B: FnOnce() -> RB,
{
    (oper_a(), oper_b())
}

/// `par_iter!(collection)` yields `collection.par_iter()` under the `parallel`
/// feature and `collection.iter()` otherwise. The downstream adapters used in
/// this crate exist on both iterator kinds, so call sites are identical
/// regardless of the feature.
#[cfg(feature = "parallel")]
macro_rules! par_iter {
    ($collection:expr) => {{ ($collection).par_iter() }};
}

#[cfg(not(feature = "parallel"))]
macro_rules! par_iter {
    ($collection:expr) => {{ ($collection).iter() }};
}

pub(crate) use par_iter;
