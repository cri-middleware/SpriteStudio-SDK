use alloc::vec::Vec;

use crate::core::framedata::CachedPartData;
use crate::runtime::framedata::ss::runtime::PartState;

/// Table holding the draw order (z-order). `indices` is a permutation of
/// part_idx sorted by ascending rank, and is the source data for
/// `FrameData.draw_order`.
pub struct ZOrderTable {
    indices: Vec<usize>,
    is_static: bool,
    computed: bool,
}

impl Default for ZOrderTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ZOrderTable {
    pub fn new() -> Self {
        Self { indices: Vec::new(), is_static: false, computed: false }
    }

    pub fn setup(&mut self, cached_parts: &[CachedPartData]) {
        let num_parts = cached_parts.len();
        self.indices.clear();
        self.indices.extend(0..num_parts);

        self.is_static = !cached_parts.iter().any(|p| p.priority_keys.is_some());
        if self.is_static {
            // Without priority animation, priority() is uniformly 0.0 — sort reduces to part_index ascending.
            self.indices.sort_by_key(|&i| cached_parts[i].part_index);
            self.computed = true;
        } else {
            self.computed = false;
        }
    }

    pub fn update(&mut self, parts: &[PartState]) {
        if self.is_static && self.computed {
            return;
        }

        let num_parts = parts.len();
        if self.indices.len() != num_parts {
            self.indices.clear();
            self.indices.extend(0..num_parts);
        } else {
            for (i, v) in self.indices.iter_mut().enumerate() {
                *v = i;
            }
        }

        self.indices.sort_by(|&a, &b| {
            let pa = &parts[a];
            let pb = &parts[b];
            if pa.priority() == pb.priority() { pa.part_index().cmp(&pb.part_index()) } else { pa.priority().partial_cmp(&pb.priority()).unwrap_or(::core::cmp::Ordering::Equal) }
        });

        self.computed = true;
    }

    /// The part_idx column arranged in draw order (ascending rank); the
    /// source data for `FrameData.draw_order`.
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }
}
