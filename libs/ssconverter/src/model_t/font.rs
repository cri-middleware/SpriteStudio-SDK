use std::collections::HashMap;

pub fn convert_font_bitmaps(charmap_dic: &HashMap<String, &sspj::sscharmap::SsCharMap>) -> Vec<ssab::ssab::ss::format::FontBitmapT> {
    let mut v = charmap_dic
        .iter()
        .map(|(key, charmap)| {
            let parent_path = crate::utils::asset_path_parent(key);

            let mut bitmap = ssab::ssab::ss::format::FontBitmapT::default();

            bitmap.name_hash = crate::utils::fnv1a_hash_str(key);
            bitmap.name = if key.is_empty() { None } else { Some(key.to_string()) };

            bitmap.name_face = if charmap.info.face.is_empty() { None } else { Some(charmap.info.face.clone()) };
            bitmap.size = 0;
            bitmap.padding_l = 0;
            bitmap.padding_r = 0;
            bitmap.padding_u = 0;
            bitmap.padding_d = 0;
            bitmap.base = charmap.common.base as i16;
            bitmap.height_line = charmap.common.line_height as i16;

            let mut scale = ssab::ssab::ss::format::Vec2T::default();
            scale.v1 = 1.0;
            scale.v2 = 1.0;
            bitmap.scale = Some(Box::new(scale));

            let mut spacing = ssab::ssab::ss::format::Vec2T::default();
            spacing.v1 = 0.0;
            spacing.v2 = 0.0;
            bitmap.spacing = Some(Box::new(spacing));

            let mut size_texture_original = ssab::ssab::ss::format::Vec2T::default();
            size_texture_original.v1 = charmap.common.scale_w;
            size_texture_original.v2 = charmap.common.scale_h;
            bitmap.size_texture_original = Some(Box::new(size_texture_original));

            let mut pages: Vec<_> = charmap.pages.values().collect();
            pages.sort_by_key(|p| p.id);
            // The `.fnt` names its page textures relative to the charmap file, so they are rebased on
            // the charmap's directory. `/` is the separator on every host — see `join_asset_path`.
            let name_textures = pages.into_iter().map(|p| crate::utils::join_asset_path(parent_path.unwrap_or_default(), &p.file)).collect::<Vec<_>>();
            if name_textures.is_empty() {
                bitmap.name_texture_hash = None;
                bitmap.name_texture = None;
            } else {
                bitmap.name_texture_hash = Some(name_textures.iter().map(|f| crate::utils::fnv1a_hash_str(f)).collect::<Vec<_>>());
                bitmap.name_texture = Some(name_textures);
            }

            let mut table_data = charmap
                .chars
                .values()
                .map(|c| {
                    let mut glyph = ssab::ssab::ss::format::FontGlyphBitmapT::default();
                    glyph.id = c.id as u16;

                    let mut atlas = ssab::ssab::ss::format::RegionT::default();
                    atlas.x = c.src_rect.x1 as f32;
                    atlas.y = c.src_rect.y1 as f32;
                    atlas.w = c.src_rect.x2 as f32;
                    atlas.h = c.src_rect.y2 as f32;
                    glyph.atlas = Some(Box::new(atlas));

                    let mut offset = ssab::ssab::ss::format::Vec2T::default();
                    offset.v1 = c.offset.x;
                    offset.v2 = c.offset.y;
                    glyph.offset = Some(Box::new(offset));

                    glyph.width_advance = c.xadvance as f32;
                    glyph.text = std::char::from_u32(c.id as u32).map(|ch| ch.to_string());
                    glyph.page = c.page as i8;
                    glyph.channel = 0;

                    glyph
                })
                .collect::<Vec<_>>();

            table_data.sort_by_key(|g| g.id);
            bitmap.table_data = if table_data.is_empty() { None } else { Some(table_data) };

            bitmap
        })
        .collect::<Vec<_>>();
    v.sort_by_key(|e| e.name_hash);
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use sspj::sscharmap::{SsCharMap, SsCharMapChar, SsCharMapCommon, SsCharMapInfo, SsCharMapPage};
    use sspj::sstypes::{SsIRect, SsPoint2};

    #[test]
    fn test_convert_font_bitmaps() {
        let mut charmap_dic = HashMap::new();

        let mut mock_charmap = SsCharMap::default();
        mock_charmap.info = SsCharMapInfo { face: "MockFont".to_string(), unicode: true, smooth: false };
        mock_charmap.common = SsCharMapCommon { line_height: 32, base: 26, scale_w: 512.0, scale_h: 512.0, pages: 1 };
        mock_charmap.pages.insert(0, SsCharMapPage { id: 0, file: "mock_0.png".to_string() });
        mock_charmap.chars.insert(32, SsCharMapChar { id: 32, src_rect: SsIRect { x1: 205, y1: 421, x2: 7, y2: 5 }, offset: SsPoint2 { x: -3.0, y: 29.0 }, xadvance: 15, page: 0 });

        charmap_dic.insert("MockFontData".to_string(), &mock_charmap);

        let result = convert_font_bitmaps(&charmap_dic);

        assert_eq!(result.len(), 1);
        let bitmap = &result[0];

        assert_eq!(bitmap.name.as_deref(), Some("MockFontData"));
        assert_eq!(bitmap.name_face.as_deref(), Some("MockFont"));
        assert_eq!(bitmap.base, 26);
        assert_eq!(bitmap.height_line, 32);

        let size_tex = bitmap.size_texture_original.as_ref().unwrap();
        assert_eq!(size_tex.v1, 512.0);
        assert_eq!(size_tex.v2, 512.0);

        let name_texture = bitmap.name_texture.as_ref().unwrap();
        assert_eq!(name_texture.len(), 1);
        assert_eq!(name_texture[0], "mock_0.png");

        let table_data = bitmap.table_data.as_ref().unwrap();
        assert_eq!(table_data.len(), 1);

        let glyph = &table_data[0];
        assert_eq!(glyph.id, 32);
        assert_eq!(glyph.text.as_deref(), Some(" "));
        assert_eq!(glyph.width_advance, 15.0);
        assert_eq!(glyph.page, 0);

        let atlas = glyph.atlas.as_ref().unwrap();
        assert_eq!(atlas.x, 205.0);
        assert_eq!(atlas.y, 421.0);
        assert_eq!(atlas.w, 7.0);
        assert_eq!(atlas.h, 5.0);

        let offset = glyph.offset.as_ref().unwrap();
        assert_eq!(offset.v1, -3.0);
        assert_eq!(offset.v2, 29.0);
    }

    #[test]
    fn test_page_textures_are_rebased_with_forward_slash() {
        let mut charmap_dic = HashMap::new();

        let mut mock_charmap = SsCharMap::default();
        mock_charmap.pages.insert(0, SsCharMapPage { id: 0, file: "mock_0.png".to_string() });
        mock_charmap.pages.insert(1, SsCharMapPage { id: 1, file: "mock_1.png".to_string() });

        // The sspj names a charmap by its `/`-separated relative path; the page textures are
        // rebased on that directory and must read the same on every host OS.
        charmap_dic.insert("font/nested/mock.fnt".to_string(), &mock_charmap);

        let result = convert_font_bitmaps(&charmap_dic);

        let name_texture = result[0].name_texture.as_ref().unwrap();
        assert_eq!(name_texture, &["font/nested/mock_0.png".to_string(), "font/nested/mock_1.png".to_string()]);

        let name_texture_hash = result[0].name_texture_hash.as_ref().unwrap();
        assert_eq!(name_texture_hash[0], crate::utils::fnv1a_hash_str("font/nested/mock_0.png"));
        assert_eq!(name_texture_hash[1], crate::utils::fnv1a_hash_str("font/nested/mock_1.png"));
    }

    #[test]
    fn test_convert_real_font_data() {
        use sspj::ssloader::SsLoader;

        let mut path = std::env::current_dir().unwrap();
        if path.ends_with("ssconverter") {
            path = path.parent().unwrap().parent().unwrap().to_path_buf();
        }
        path.push("tests/allPartsV7/font/RoundedMPlus.fnt");

        assert!(path.exists(), "Test font file not found: {:?}", path);

        let charmap = SsCharMap::load_from_file(&path).expect("Failed to load charmap");

        let mut charmap_dic = HashMap::new();
        charmap_dic.insert("RoundedMPlus".to_string(), &charmap);

        let result = convert_font_bitmaps(&charmap_dic);

        assert_eq!(result.len(), 1);
        let bitmap = &result[0];

        assert_eq!(bitmap.name.as_deref(), Some("RoundedMPlus"));
        assert_eq!(bitmap.name_face.as_deref(), Some("PixelMplus12"));
        assert_eq!(bitmap.base, 26);
        assert_eq!(bitmap.height_line, 32);

        let size_tex = bitmap.size_texture_original.as_ref().unwrap();
        assert_eq!(size_tex.v1, 512.0);
        assert_eq!(size_tex.v2, 512.0);

        let name_texture = bitmap.name_texture.as_ref().unwrap();
        assert_eq!(name_texture.len(), 1);
        assert_eq!(name_texture[0], "RoundedMPlus_0.png");

        let table_data = bitmap.table_data.as_ref().unwrap();
        assert!(!table_data.is_empty());

        let space_glyph = table_data.iter().find(|g| g.id == 32).expect("Space glyph not found");
        assert_eq!(space_glyph.text.as_deref(), Some(" "));
        assert_eq!(space_glyph.width_advance, 15.0);
        assert_eq!(space_glyph.page, 0);

        let atlas = space_glyph.atlas.as_ref().unwrap();
        assert_eq!(atlas.x, 205.0);
        assert_eq!(atlas.y, 421.0);
        assert_eq!(atlas.w, 7.0);
        assert_eq!(atlas.h, 5.0);

        let offset = space_glyph.offset.as_ref().unwrap();
        assert_eq!(offset.v1, -3.0);
        assert_eq!(offset.v2, 29.0);
    }
}
