#[cfg(test)]
mod tests {
    use crate::ssae::SsAnimePack;
    use crate::ssloader::SsLoader;
    use crate::sspj_error::SspjError;

    fn get_minimal_ssae_xml(part_xml: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<SpriteStudioAnimePack version="2.00.00">
    <settings>
        <fps>30</fps>
        <frameCount>1</frameCount>
        <canvasSize>0 0</canvasSize>
        <pivot>0 0</pivot>
        <startFrame>0</startFrame>
        <endFrame>0</endFrame>
    </settings>
    <name>test</name>
    <Model>
        <partList>
            {}
        </partList>
        <meshList />
        <boneList />
    </Model>
    <cellmapNames />
    <animeList>
        <anime>
            <name>Setup</name>
            <overrideSettings>0</overrideSettings>
            <isSetup>1</isSetup>
            <settings>
                <fps>30</fps>
                <frameCount>1</frameCount>
                <canvasSize>0 0</canvasSize>
                <pivot>0 0</pivot>
                <startFrame>0</startFrame>
                <endFrame>0</endFrame>
            </settings>
            <partAnimes />
            <labels />
        </anime>
    </animeList>
</SpriteStudioAnimePack>"#,
            part_xml
        )
    }

    #[test]
    fn test_ssae_robustness_minimal_valid() {
        let part_xml = r#"
            <value>
                <name>root</name>
                <arrayIndex>0</arrayIndex>
                <parentIndex>-1</parentIndex>
                <type>null</type>
                <boundsType>none</boundsType>
                <inheritType>self</inheritType>
                <ineheritRates>
                    <ALPH>1</ALPH><IFLH>0</IFLH><IFLV>0</IFLV><FLPH>0</FLPH><FLPV>0</FLPV><HIDE>0</HIDE>
                </ineheritRates>
                <alphaBlendType>mix</alphaBlendType>
                <show>1</show>
                <locked>0</locked>
                <maskInfluence>1</maskInfluence>
            </value>"#;
        let xml = get_minimal_ssae_xml(part_xml);
        let result = SsAnimePack::load(&xml);
        assert!(result.is_ok(), "Should parse with all mandatory fields. Error: {:?}", result.err());
    }

    #[test]
    fn test_ssae_robustness_invalid_xml() {
        let xml = r#"<SpriteStudioAnimePack> <invalid"#;
        let result = SsAnimePack::load(xml);
        assert!(matches!(result, Err(SspjError::XmlParseError(_))));
    }

    #[test]
    fn test_ssae_robustness_unknown_tags() {
        let part_xml = r#"
            <value>
                <name>root</name>
                <arrayIndex>0</arrayIndex>
                <parentIndex>-1</parentIndex>
                <type>null</type>
                <boundsType>none</boundsType>
                <inheritType>self</inheritType>
                <ineheritRates>
                    <ALPH>1</ALPH><IFLH>0</IFLH><IFLV>0</IFLV><FLPH>0</FLPH><FLPV>0</FLPV><HIDE>0</HIDE>
                </ineheritRates>
                <alphaBlendType>mix</alphaBlendType>
                <show>1</show>
                <locked>0</locked>
                <maskInfluence>1</maskInfluence>
                <newUnknownTag>Ignore this</newUnknownTag>
            </value>"#;
        let xml = get_minimal_ssae_xml(part_xml);
        let result = SsAnimePack::load(&xml);
        assert!(result.is_ok(), "Should ignore unknown tags inside parts. Error: {:?}", result.err());
    }

    #[test]
    fn test_sspj_robustness_minimal() {
        use crate::sspj::SsProject;
        let xml = r#"<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<SpriteStudioProject version="2.00.00">
    <name>test_project</name>
    <settings>
        <exportBaseDirectory>export</exportBaseDirectory>
        <queryExportBaseDirectory>1</queryExportBaseDirectory>
        <wrapMode>clamp</wrapMode>
        <filterMode>linear</filterMode>
    </settings>
</SpriteStudioProject>"#;
        let result = quick_xml::de::from_str::<SsProject>(xml);
        assert!(result.is_ok(), "SsProject should parse with minimal fields. Error: {:?}", result.err());
        let sspj = result.unwrap();
        assert_eq!(sspj.name, "test_project");
        assert_eq!(sspj.settings.anime_base_directory, "");
    }

    #[test]
    fn test_sspj_robustness_empty_name_lists() {
        use crate::sspj::SsProject;
        const LISTS: [&str; 6] = ["cellmapNames", "animepackNames", "effectFileNames", "sequencepackNames", "audiopackNames", "charmapNames"];
        let self_closed: String = LISTS.iter().map(|list| format!("<{list} />")).collect();
        let open_and_close: String = LISTS.iter().map(|list| format!("<{list}></{list}>")).collect();
        for lists in [self_closed, open_and_close] {
            let xml = format!(
                r#"<SpriteStudioProject version="2.00.00">
    <name>test_project</name>
    <settings>
        <exportBaseDirectory>export</exportBaseDirectory>
        <queryExportBaseDirectory>1</queryExportBaseDirectory>
        <wrapMode>clamp</wrapMode>
        <filterMode>linear</filterMode>
    </settings>
    {lists}
</SpriteStudioProject>"#
            );
            let sspj = quick_xml::de::from_str::<SsProject>(&xml).unwrap();
            assert!(sspj.cellmap_names.is_empty());
            assert!(sspj.animepack_names.is_empty());
            assert!(sspj.effect_file_names.is_empty());
            assert!(sspj.sequencepack_names.is_empty());
            assert!(sspj.audiopack_names.is_empty());
            assert!(sspj.charmap_names.is_empty());
        }
    }

    #[test]
    fn test_ssqe_robustness_empty_item_list() {
        use crate::ssqe::SsSequence;
        let xml = "<sequence><name>empty</name><index>0</index><type>LAST</type><list /></sequence>";
        assert!(quick_xml::de::from_str::<SsSequence>(xml).unwrap().list.is_empty());
    }

    #[test]
    fn test_ssse_robustness_empty_sound_list() {
        use crate::ssse::SsSoundFileList;
        let xml = "<table><id>0</id><Alias></Alias><sounds /></table>";
        assert!(quick_xml::de::from_str::<SsSoundFileList>(xml).unwrap().sounds.is_empty());
    }

    #[test]
    fn test_signal_robustness_empty_lists() {
        use crate::ssattribute::{SsAttributeSignalCommand, SsAttributeSignalKeyValue};
        let no_commands = "<value><commands /></value>";
        assert!(quick_xml::de::from_str::<SsAttributeSignalKeyValue>(no_commands).unwrap().commands.is_empty());
        let no_params = "<value><active>1</active><commandId>command</commandId><params /></value>";
        assert!(quick_xml::de::from_str::<SsAttributeSignalCommand>(no_params).unwrap().params.is_empty());
    }
}
