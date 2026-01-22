use std::collections::HashMap;
pub fn convert_font_bitmaps(_sspj: &sspj::sspj::SsProject, charmap_dic: &HashMap<String, &sspj::sscharmap::SsCharMap>, ) -> Vec<ssab::ssab::FontBitmapT> {
    let mut v = charmap_dic.iter().map(|(key, _value)| {
        let mut bitmap = ssab::ssab::FontBitmapT::default();

        bitmap.name_hash = crate::utils::fnv1a_hash_str(key);
        bitmap.name = if key.is_empty() { None} else { Some(key.to_string()) };
        // TODO: impl
        /*
        bitmap.name_face ;
        bitmap.size;
        bitmap.padding_l;
        bitmap.padding_r;
        bitmap.padding_u;
        bitmap.padding_d;
        bitmap.base;
        bitmap.height_line;
        bitmap.scale;
        bitmap.spacing;
        bitmap.size_texture_original;
        bitmap.name_texture;
        bitmap.table_data;
         */

        bitmap
    }).collect::<Vec<_>>();
    v.sort_by_key(|e| e.name_hash);
    v
}
