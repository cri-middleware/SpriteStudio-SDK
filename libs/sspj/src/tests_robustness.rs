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
}
