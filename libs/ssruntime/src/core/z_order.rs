use alloc::vec::Vec;

use crate::core::framedata::CachedPartData;
use crate::runtime::framedata::ss::runtime::PartState;

/// 描画順 (z-order) を保持するテーブル。`indices` には part_idx を rank 昇順で
/// 並べた permutation を持ち、これが `FrameData.draw_order` の元データとなる。
/// 旧 API には rank-per-part_idx の逆引き表 (`order: Vec<i32>`) もあったが、
/// バッチ描画契約 (chapter 9) への移行に伴い廃止 (FFI も同時に削除)。
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

    pub fn setup(&mut self, cached_parts: &[CachedPartData<'_>]) {
        let num_parts = cached_parts.len();
        self.indices.clear();
        self.indices.extend(0..num_parts);

        self.is_static = !cached_parts.iter().any(|p| p.priority_array.is_some());
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

    /// part_idx 列を「描画順」(rank 昇順) に並べたスライス。
    /// FrameData.draw_order の元データとして使用する。
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }
}
