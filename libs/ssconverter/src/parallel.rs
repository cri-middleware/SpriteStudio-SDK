//! Parallelism abstraction.
//!
//! Conversion code does `use crate::parallel::*;` and then uses [`par_iter!`]
//! instead of touching rayon directly. With the `parallel` feature (the
//! default for native builds) it maps onto rayon — the glob re-export also
//! brings rayon's prelude traits into scope so the downstream `map` / `zip` /
//! `collect` / `try_for_each` adapters resolve. Without the feature it falls
//! back to sequential iteration, which lets the converter compile for
//! `wasm32-unknown-unknown`, where rayon cannot spawn threads.

#[cfg(feature = "parallel")]
pub(crate) use rayon::prelude::*;

/// `par_iter!(collection)` yields `collection.par_iter()` under the `parallel`
/// feature and `collection.iter()` otherwise. The adapters used here exist on
/// both iterator kinds (including for `HashMap`), so call sites are identical
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
