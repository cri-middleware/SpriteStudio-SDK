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
    /// The animation the table was set up for, or -1.
    animation: i32,
}

impl Default for ZOrderTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ZOrderTable {
    pub fn new() -> Self {
        Self { indices: Vec::new(), is_static: false, computed: false, animation: -1 }
    }

    /// Set the table up for `animation`. The draw order only has to be sorted
    /// every frame when some part's priority changes during the animation;
    /// otherwise it is sorted once, by the first `update`.
    pub fn setup(&mut self, animation: i32, cached_parts: &[CachedPartData]) {
        let num_parts = cached_parts.len();
        self.indices.clear();
        self.indices.extend(0..num_parts);
        self.animation = animation;

        // A run of one key holds the priority for the whole animation; the
        // converter writes one for every part from the setup pose.
        self.is_static = !cached_parts.iter().any(|p| p.run(crate::core::framedata::IDX_ATTRIBUTEPRIO).is_some_and(|run| run.count() > 1));
        if self.is_static && !cached_parts.iter().any(|p| p.run(crate::core::framedata::IDX_ATTRIBUTEPRIO).is_some()) {
            // Without priority keys, priority() is uniformly 0 — sort reduces to part_index ascending.
            self.indices.sort_by_key(|&i| cached_parts[i].part_index);
            self.computed = true;
        } else {
            self.computed = false;
        }
    }

    /// Set the table up again if `animation` is not the one it was set up for:
    /// a track-0 change can hand the primary role to another animation.
    pub fn fit(&mut self, animation: i32, cached_parts: &[CachedPartData]) {
        if self.animation != animation {
            self.setup(animation, cached_parts);
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
            pa.priority().cmp(&pb.priority()).then_with(|| pa.part_index().cmp(&pb.part_index()))
        });

        self.computed = true;
    }

    /// The part_idx column arranged in draw order (ascending rank); the
    /// source data for `FrameData.draw_order`.
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framedata::cached_part_stub;
    use crate::core::keyframes::KeyRun;

    fn parts(runs: &[Option<KeyRun>]) -> Vec<CachedPartData> {
        runs.iter()
            .enumerate()
            .map(|(i, run)| {
                let mut p = cached_part_stub(i as i16, -1, ssab::ssab::ss::format::PartType::PartTypeNormal, false);
                if let Some(run) = run {
                    p.key_mask = 1 << crate::core::framedata::IDX_ATTRIBUTEPRIO;
                    p.runs = alloc::boxed::Box::new([*run]);
                }
                p
            })
            .collect()
    }

    fn states(priorities: &[i32]) -> Vec<PartState> {
        priorities
            .iter()
            .enumerate()
            .map(|(i, &p)| {
                let mut s = PartState::default();
                s.set_part_index(i as i16);
                s.set_priority(p);
                s
            })
            .collect()
    }

    #[test]
    fn a_constant_priority_is_sorted_once() {
        let one = Some(KeyRun::new(0, 1));
        let mut table = ZOrderTable::new();
        table.setup(0, &parts(&[one, one, one]));
        table.update(&states(&[2, 0, 1]));
        assert_eq!(table.indices(), &[1, 2, 0]);
        // One key a part means the priority cannot change, so later frames keep the order.
        table.update(&states(&[0, 1, 2]));
        assert_eq!(table.indices(), &[1, 2, 0]);
    }

    #[test]
    fn an_animated_priority_is_sorted_every_frame() {
        let one = Some(KeyRun::new(0, 1));
        let mut table = ZOrderTable::new();
        table.setup(0, &parts(&[one, Some(KeyRun::new(1, 2)), one]));
        table.update(&states(&[2, 0, 1]));
        assert_eq!(table.indices(), &[1, 2, 0]);
        table.update(&states(&[0, 3, 1]));
        assert_eq!(table.indices(), &[0, 2, 1]);
    }

    #[test]
    fn another_animation_is_set_up_again() {
        let one = Some(KeyRun::new(0, 1));
        let p = parts(&[one, one, one]);
        let mut table = ZOrderTable::new();
        table.setup(0, &p);
        table.update(&states(&[2, 0, 1]));
        table.fit(0, &p);
        table.update(&states(&[0, 1, 2]));
        assert_eq!(table.indices(), &[1, 2, 0], "same animation: the order stands");
        table.fit(1, &p);
        table.update(&states(&[0, 1, 2]));
        assert_eq!(table.indices(), &[0, 1, 2], "another animation: sorted again");
    }
}
