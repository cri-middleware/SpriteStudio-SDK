
pub fn convert_ss_anime_binary_t(anime_pack: &sspj::ssae::SsAnimePack, sspj: &sspj::sspj::SsProject, sspj_parent_path: &std::path::Path, embedded: bool) -> Result<ssab::ssab::ss::format::SsAnimeBinaryT, crate::ConverterError> {
    crate::model_t::ssabt::convert_ss_anime_binary_t(anime_pack, sspj, sspj_parent_path, embedded)
}