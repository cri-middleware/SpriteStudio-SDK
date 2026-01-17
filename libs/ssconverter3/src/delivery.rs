use std::collections::{HashMap, HashSet};

pub fn get_assets_of_ssab_using(ssab_map: &HashMap<String, Vec<u8>>, sspj: &sspj::sspj::SsProject) -> HashMap<String, String> {
    ssab_map.values().flat_map(|data| {
        let anime = unsafe { ssab::ssab::root_as_ss_anime_binary_unchecked(data.as_slice()) };

        let images = anime.cellmaps().into_iter().flatten().flat_map(|cellmap| cellmap.image_path().map(|s| {
            (s.to_string(), std::path::PathBuf::from(&sspj.settings.image_base_directory).join(s).to_string_lossy().to_string())
        })).collect::<HashMap<_, _>>();

        let external_textures = anime.external_textures().into_iter().flatten().map(|external_texture| {
            let s = external_texture.name().unwrap();
            (s.to_string(), std::path::PathBuf::from(&sspj.settings.image_base_directory).join(external_texture.name().unwrap()).to_string_lossy().to_string())
        }).collect::<HashMap<_, _>>();

        anime.cellmaps().into_iter().flatten().flat_map(|cellmap| cellmap.image_path().map(|s| {
            std::path::PathBuf::from(&sspj.settings.image_base_directory).join(s).to_string_lossy().to_string()
        }));

        // TODO: impl
        //let sounds = anime.sound_lists().into_iter().flatten().flat_map(|soundlist| {
        //    soundlist.table_data().into_iter().flatten().map(|item| item.name().to_string())
        //});

        // TODO: improve font bitmap path
        let fonts = anime.font_bitmaps().into_iter().flatten().flat_map(|font_bitmap| {
            font_bitmap.name_texture().into_iter().flatten().map(|name_texture| name_texture.to_string())
        });

        let mut output = HashMap::<String, String>::new();
        output.extend(images);
        output.extend(external_textures);
        output
    }).collect()
}
