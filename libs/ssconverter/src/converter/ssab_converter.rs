use crate::asset_provider::AssetProvider;

pub fn convert_ss_anime_binary_t(anime_pack: &sspj::ssae::SsAnimePack, sspj: &sspj::sspj::SsProject, asset_provider: &dyn AssetProvider, embedded: bool) -> Result<ssab::ssab::ss::format::SsAnimeBinaryT, crate::ConverterError> {
    crate::model_t::ssabt::convert_ss_anime_binary_t(anime_pack, sspj, asset_provider, embedded)
}