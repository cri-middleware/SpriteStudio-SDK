
pub fn convert_ss_anime_binary_t(anime_pack: &sspj::ssae::SsAnimePack, sspj: &sspj::sspj::SsProject) -> Result<ssab::ssab::SsAnimeBinaryT, crate::ConverterError> {
    crate::model_t::ssabt::convert_ss_anime_binary_t(anime_pack, sspj)
}