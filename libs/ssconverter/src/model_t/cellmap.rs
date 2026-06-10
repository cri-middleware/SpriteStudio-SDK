use crate::model_t;
fn convert_cellt(sscell: &sspj::ssce::SsCell) -> ssab::ssab::ss::format::CellT {
    let mut cell_t = ssab::ssab::ss::format::CellT::default();
    cell_t.name_hash = crate::utils::fnv1a_hash_str(&sscell.name);
    cell_t.name = sscell.name.to_string();
    cell_t.rectangle = Box::new(model_t::common::convert_rectt(sscell.pos.x, sscell.pos.y, sscell.size.x, sscell.size.y));
    cell_t.pivot = Box::new(model_t::common::convert_vec2t(&sscell.pivot));
    cell_t.rotated = sscell.rotated;
    if sscell.mesh_point_list.is_empty() {
        cell_t.table_coord_x = None;
        cell_t.table_coord_y = None;
    } else {
        cell_t.table_coord_x = Some(sscell.mesh_point_list.iter().map(|p| p.x).collect());
        cell_t.table_coord_y = Some(sscell.mesh_point_list.iter().map(|p| p.y).collect());
    }
    cell_t.table_index_vertex = if sscell.mesh_tri_list.is_empty() { None } else { Some(model_t::common::convert_flat_list_from_vec3t_list(&sscell.mesh_tri_list)) };

    cell_t
}
fn convert_cellt_list(cells: &[sspj::ssce::SsCell]) -> Vec<ssab::ssab::ss::format::CellT> {
    let mut v = cells.iter().map(convert_cellt).collect::<Vec<ssab::ssab::ss::format::CellT>>();
    v.sort_by_key(|cell| cell.name_hash);
    v
}

fn convert_cellmapt(cellmap: &sspj::ssce::SsCellMap, sspj: &sspj::sspj::SsProject, asset_provider: &dyn crate::asset_provider::AssetProvider) -> Result<ssab::ssab::ss::format::CellMapT, crate::ConverterError> {
    let mut cell_map_t = ssab::ssab::ss::format::CellMapT::default();
    cell_map_t.name_hash = crate::utils::fnv1a_hash_str(&cellmap.name);
    cell_map_t.name = cellmap.name.to_string();

    let path = std::path::PathBuf::from(&sspj.settings.image_base_directory).join(&cellmap.image_path);
    let path_str = path.to_str().ok_or_else(|| crate::converter::converter_error::ConverterError::ConversionFailed("Non-UTF8 path in cellmap".to_string()))?;
    let r = asset_provider.validate_asset(path_str);
    match r {
        Ok(_) => (),
        Err(e) => return Err(e),
    }
    cell_map_t.image_path = cellmap.image_path.to_string();
    cell_map_t.image_path_hash = crate::utils::fnv1a_hash_str(&cellmap.image_path);
    cell_map_t.size_original = Box::new(model_t::common::convert_vec2t(&cellmap.pixel_size));
    cell_map_t.wrap_mode = match &cellmap.wrap_mode {
        sspj::sstypes::SsTexWrapMode::Clamp => ssab::ssab::ss::format::TexWrapMode::Clamp,
        sspj::sstypes::SsTexWrapMode::Repeat => ssab::ssab::ss::format::TexWrapMode::Repeat,
        sspj::sstypes::SsTexWrapMode::Mirror => ssab::ssab::ss::format::TexWrapMode::Mirror,
    };
    cell_map_t.filter_mode = match &cellmap.filter_mode {
        sspj::sstypes::SsTexFilterMode::Nearest => ssab::ssab::ss::format::TexFilterMode::Nearest,
        sspj::sstypes::SsTexFilterMode::Linear => ssab::ssab::ss::format::TexFilterMode::Linear,
    };
    cell_map_t.cells = if cellmap.cells.is_empty() { None } else { Some(convert_cellt_list(&cellmap.cells)) };

    Ok(cell_map_t)
}
pub fn convert_cellmapt_list(sspj: &sspj::sspj::SsProject, cellmap_names: &[&sspj::ssce::SsCellMap], asset_provider: &dyn crate::asset_provider::AssetProvider) -> Result<Vec<ssab::ssab::ss::format::CellMapT>, crate::ConverterError> {
    let mut v = cellmap_names.iter().map(|cellmap| convert_cellmapt(cellmap, sspj, asset_provider)).collect::<Result<Vec<ssab::ssab::ss::format::CellMapT>, _>>()?;
    v.sort_by_key(|cellmap| cellmap.name_hash);
    Ok(v)
}
