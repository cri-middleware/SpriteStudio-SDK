use crate::model_t;
fn convert_cellt(sscell: &sspj::ssce::SsCell) -> ssab::ssab::CellT {
    let mut cell_t = ssab::ssab::CellT::default();
    cell_t.name_hash = crate::utils::fnv1a_hash_str(&sscell.name);
    cell_t.name = if sscell.name.is_empty() { None } else { Some(sscell.name.to_string()) };
    cell_t.rectangle = Some(Box::new(model_t::common::convert_rectt(sscell.pos.x, sscell.pos.y, sscell.size.x, sscell.size.y, )));
    cell_t.pivot = Some(Box::new(model_t::common::convert_vec2t(&sscell.pivot)));
    cell_t.rotated = sscell.rotated;
    cell_t.table_coordinate = if sscell.mesh_point_list.is_empty() { None } else { Some(model_t::common::convert_vec2t_list(&sscell.mesh_point_list)) };
    cell_t.table_index_vertex = if sscell.mesh_tri_list.is_empty() { None } else { Some(model_t::common::convert_flat_list_from_vec3t_list(&sscell.mesh_tri_list)) };

    cell_t
}
fn convert_cellt_list(cells: &[sspj::ssce::SsCell]) -> Vec<ssab::ssab::CellT> {
    let mut v = cells.iter().map(|cell| convert_cellt(cell)).collect::<Vec<ssab::ssab::CellT>>();
    v.sort_by_key(|cell| cell.name_hash);
    v
}

fn convert_cellmapt(cellmap: &sspj::ssce::SsCellMap, sspj: &sspj::sspj::SsProject, sspj_parent_path: &std::path::Path) -> Result<ssab::ssab::CellMapT, crate::ConverterError> {
    let mut cell_map_t = ssab::ssab::CellMapT::default();
    cell_map_t.name_hash = crate::utils::fnv1a_hash_str(&cellmap.name);
    cell_map_t.name = if cellmap.name.is_empty() { None } else { Some(cellmap.name.to_string()) };

    let r = crate::utils::validate_file_path(&cellmap.image_path, crate::utils::get_sspj_image_dir_path(sspj_parent_path, &sspj).as_path());
    match r {
        Ok(_) => (),
        Err(e) => return Err(e),
    }
    cell_map_t.image_path = if cellmap.image_path.is_empty() { None } else { Some(cellmap.image_path.to_string()) };
    cell_map_t.size_original = Some(Box::new(model_t::common::convert_vec2t(&cellmap.pixel_size)));
    cell_map_t.wrap_mode = match &cellmap.wrap_mode {
        sspj::sstypes::SsTexWrapMode::Clamp => ssab::ssab::TexWrapMode::Clamp,
        sspj::sstypes::SsTexWrapMode::Repeat => ssab::ssab::TexWrapMode::Repeat,
        sspj::sstypes::SsTexWrapMode::Mirror => ssab::ssab::TexWrapMode::Mirror,
    };
    cell_map_t.filter_mode = match &cellmap.filter_mode {
        sspj::sstypes::SsTexFilterMode::Nearest => ssab::ssab::TexFilterMode::Nearest,
        sspj::sstypes::SsTexFilterMode::Linear => ssab::ssab::TexFilterMode::Linear,
    };
    cell_map_t.cells = if cellmap.cells.is_empty() { None } else { Some(convert_cellt_list(&cellmap.cells)) };

    Ok(cell_map_t)
}
pub fn convert_cellmapt_list(sspj: &sspj::sspj::SsProject, cellmap_names: &[&sspj::ssce::SsCellMap], sspj_parent_path: &std::path::Path) -> Result<Vec<ssab::ssab::CellMapT>, crate::ConverterError> {
    let mut v = cellmap_names.iter().map(|cellmap| convert_cellmapt(cellmap, sspj, sspj_parent_path)).collect::<Result<Vec<ssab::ssab::CellMapT>, _>>()?;
    v.sort_by_key(|cellmap| cellmap.name_hash);
    Ok(v)
}
