use alloc::vec::Vec;

use crate::core::framedata::CachedPartData;
use crate::runtime::framedata::ss::runtime::PartState;

pub struct ZOrderTable {
    order: Vec<i32>,
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
        Self { order: Vec::new(), indices: Vec::new(), is_static: false, computed: false }
    }

    pub fn setup(&mut self, cached_parts: &[CachedPartData<'_>]) {
        let num_parts = cached_parts.len();
        self.order.clear();
        self.order.resize(num_parts, 0);
        self.indices.clear();
        self.indices.extend(0..num_parts);

        self.is_static = !cached_parts.iter().any(|p| p.priority_array.is_some());
        if self.is_static {
            // Without priority animation, priority() is uniformly 0.0 — sort reduces to part_index ascending.
            self.indices.sort_by_key(|&i| cached_parts[i].part_index);
            self.write_ranks();
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
        if self.order.len() != num_parts || self.indices.len() != num_parts {
            self.order.resize(num_parts, 0);
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
            if pa.priority() == pb.priority() {
                pa.part_index().cmp(&pb.part_index())
            } else {
                pa.priority().partial_cmp(&pb.priority()).unwrap_or(::core::cmp::Ordering::Equal)
            }
        });

        self.write_ranks();
        self.computed = true;
    }

    fn write_ranks(&mut self) {
        for (rank, &part_idx) in self.indices.iter().enumerate() {
            self.order[part_idx] = rank as i32;
        }
    }

    pub fn as_slice(&self) -> &[i32] {
        &self.order
    }

    pub fn as_ptr(&self) -> *const i32 {
        self.order.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
}
