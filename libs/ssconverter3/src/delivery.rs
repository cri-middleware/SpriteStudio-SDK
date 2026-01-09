use std::collections::{HashMap, HashSet};

pub fn delivery_ssab_suite(ssab_map: &HashMap<String, Vec<u8>>) -> HashSet<String> {
    ssab_map.values().flat_map(|data| {
        let anime = unsafe { ssab::ssab::root_as_ss_anime_binary_unchecked(data.as_slice()) };

        let images = anime.cellmaps().into_iter().flatten().flat_map(|cellmap| cellmap.image_path().map(|s| s.to_string()));

        let sounds = anime.sound_lists().into_iter().flatten().flat_map(|soundlist| {
            soundlist.table_data().into_iter().flatten().map(|item| item.name().to_string())
        });

        // TODO: improve font bitmap path
        let fonts = anime.font_bitmaps().into_iter().flatten().flat_map(|font_bitmap| {
            font_bitmap.name_texture().into_iter().flatten().map(|name_texture| name_texture.to_string())
        });

        images.chain(sounds).chain(fonts)
    }).collect()
}
