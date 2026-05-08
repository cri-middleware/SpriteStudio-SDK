use ssab::ssab::ss::format::{Cell, CellMap, Effect};

fn find_by_name_hash<'a, T>(items: &flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<T>>, name_hash: u32, cmp: impl Fn(&T, u32) -> core::cmp::Ordering) -> Option<T>
where
    T: flatbuffers::Follow<'a, Inner = T> + 'a,
{
    let mut low = 0usize;
    let mut high = items.len();
    while low < high {
        let mid = (low + high) / 2;
        let item = items.get(mid);
        match cmp(&item, name_hash) {
            core::cmp::Ordering::Equal => return Some(item),
            core::cmp::Ordering::Less => low = mid + 1,
            core::cmp::Ordering::Greater => high = mid,
        }
    }
    None
}

#[inline]
pub fn find_cellmap<'a>(cellmaps: &flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<CellMap<'a>>>, name_hash: u32) -> Option<CellMap<'a>> {
    find_by_name_hash(cellmaps, name_hash, |c, v| c.key_compare_with_value(v))
}

#[inline]
pub fn find_cell<'a>(cells: &flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<Cell<'a>>>, name_hash: u32) -> Option<Cell<'a>> {
    find_by_name_hash(cells, name_hash, |c, v| c.key_compare_with_value(v))
}

#[inline]
pub fn find_effect<'a>(effects: &flatbuffers::Vector<'a, flatbuffers::ForwardsUOffset<Effect<'a>>>, name_hash: u32) -> Option<Effect<'a>> {
    find_by_name_hash(effects, name_hash, |e, v| e.key_compare_with_value(v))
}
