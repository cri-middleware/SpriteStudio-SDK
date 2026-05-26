use alloc::vec::Vec;

use crate::core::framedata::CachedPartData;
use crate::runtime::framedata::ss::runtime::PartState;

/// 描画順 (z-order) を保持するテーブル。`indices` には part_idx を rank 昇順で
/// 並べた permutation を持ち、これが `FrameData.draw_order` の元データとなる。
/// 旧 API には rank-per-part_idx の逆引き表 (`order: Vec<i32>`) もあったが、
/// バッチ描画契約 (chapter 9) への移行に伴い廃止 (FFI も同時に削除)。
pub struct ZOrderTable {
    indices: Vec<usize>,
    // Per array-index: true for a pure Mask part (PartType::PartTypeMask).
    // Cached in `setup` (where the static part metadata is available) and used
    // as a draw-order tie-break by both `setup` and `update`.
    is_pure_mask: Vec<bool>,
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
        Self { indices: Vec::new(), is_pure_mask: Vec::new(), is_static: false, computed: false }
    }

    pub fn setup(&mut self, cached_parts: &[CachedPartData<'_>]) {
        let num_parts = cached_parts.len();
        self.indices.clear();
        self.indices.extend(0..num_parts);

        self.is_pure_mask.clear();
        self.is_pure_mask.extend(cached_parts.iter().map(|p| p.part_type_disc == ssab::ssab::ss::format::PartType::PartTypeMask));

        self.is_static = !cached_parts.iter().any(|p| p.priority_array.is_some());
        if self.is_static {
            // Without priority animation, priority() is uniformly 0.0, so order
            // reduces to the tie-break: a pure Mask sorts ahead of equal-priority
            // parts so those parts fall outside its (draw-before) scope and are
            // not masked — matching SpriteStudio. Otherwise part_index ascending.
            let is_pure_mask = &self.is_pure_mask;
            self.indices.sort_by(|&a, &b| is_pure_mask[b].cmp(&is_pure_mask[a]).then_with(|| cached_parts[a].part_index.cmp(&cached_parts[b].part_index)));
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

        // Tie-break at equal priority: pure Mask parts first (so equal-priority
        // parts stay out of their draw-before scope), then part_index ascending.
        // Falls back to part_index-only if the cached flags don't match this set.
        let is_pure_mask = if self.is_pure_mask.len() == num_parts { Some(&self.is_pure_mask) } else { None };
        self.indices.sort_by(|&a, &b| {
            let pa = &parts[a];
            let pb = &parts[b];
            pa.priority()
                .partial_cmp(&pb.priority())
                .unwrap_or(::core::cmp::Ordering::Equal)
                .then_with(|| match is_pure_mask {
                    Some(m) => m[b].cmp(&m[a]),
                    None => ::core::cmp::Ordering::Equal,
                })
                .then_with(|| pa.part_index().cmp(&pb.part_index()))
        });

        self.computed = true;
    }

    /// part_idx 列を「描画順」(rank 昇順) に並べたスライス。
    /// FrameData.draw_order の元データとして使用する。
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }
}
