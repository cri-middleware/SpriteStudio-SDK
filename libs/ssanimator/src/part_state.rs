use sspj::ssae::SsPart;
use sspj::sstypes::{SsBlendType, SsPartType, SsPoint2, SsPoint3};
use std::cell::RefCell;
use std::rc::Weak;

pub struct SsPartState {
    index: i32,

    vertices: [f32; 30],
    colors: [f32; 40],
    uvs: [f32; 20],
    matrix: [f32; 16],
    matrix_local: [f32; 16],

    parent: Option<Weak<RefCell<SsPartState>>>,

    // TODO:
    // float* 			inheritRates;	///< 継承設定の参照先。inheritType がparentだと親のを見に行く。透過的に遡るのでルートから先に設定されている必要がある。
    position: SsPoint3,
    rotation: SsPoint3,
    scale: SsPoint2,
    localscale: SsPoint2,
    alpha: f32,
    localalpha: f32,
    prio: i32,
    h_flip: bool,
    v_flip: bool,
    hide: bool,
    pivot_offset: SsPoint2,
    anchor: SsPoint2,
    size: SsPoint2,
    image_flip_h: bool,
    image_flip_v: bool,
    uv_translate: SsPoint2,
    uv_rotation: f32,
    uv_scale: SsPoint2,
    bounding_radius: f32,

    // TODO:
    /*
    SsCellValue		cellValue;		///< セルアニメの値
    SsPartsColorAnime partsColorValue;///< カラーアニメの値
    SsColorAnime	colorValue;		///< カラーアニメの値
    SsShaderAnime	shaderValue;	///< シェーダーアニメの値
    SsVertexAnime	vertexValue;	///< 頂点アニメの値
    SsEffectAttr	effectValue;	///< エフェクトの値
     */
    effect_time: i32,
    effect_time_total: f32,
    effectseed: i32,

    no_cells: bool,
    is_parts_color: bool,
    is_color_blend: bool,
    is_shader: bool,
    is_vertex_transform: bool,
    is_local_alpha: bool,
    is_deform: bool,

    // SsSignalAttr	signalValue;

    // SsInstanceAttr	instanceValue;
    alpha_blend_type: SsBlendType,

    // std::unique_ptr<SsAnimeDecoder>		refAnime;
    // std::unique_ptr<SsEffectRenderV2>	refEffect;

    //add SS7.1
    //std::unique_ptr<SsPart> partTypeAnimeVaule;
    //--
    part_type: SsPartType,

    masklimen: i32,
    mask_influence: bool,

    // std::unique_ptr<SsMeshPart>	meshPart;

    //SsDeformAttr	deformValue;
    //SsAudioAttr		audioValue;
    //SsTexChangeAttr	changeTextureValue;
    part: Option<Weak<RefCell<SsPart>>>,
}

impl SsPartState {
    pub fn default(_index: i32) -> Self {
        Self {
            index: 0,

            vertices: [0.0; 3 * 10],
            colors: [0.0; 4 * 10],
            uvs: [0.0; 2 * 10],
            matrix: [0.0; 4 * 4],
            matrix_local: [0.0; 4 * 4],

            parent: None,

            // float* 			inheritRates;	///< 継承設定の参照先。inheritType がparentだと親のを見に行く。透過的に遡るのでルートから先に設定されている必要がある。
            position: SsPoint3::default(),
            rotation: SsPoint3::default(),
            scale: SsPoint2 { x: 1.0, y: 1.0 },
            localscale: SsPoint2 { x: 1.0, y: 1.0 },
            alpha: 1.0,
            localalpha: 1.0,
            prio: 0,
            h_flip: false,
            v_flip: false,
            hide: false,

            pivot_offset: SsPoint2::default(),
            anchor: SsPoint2::default(),
            size: SsPoint2 { x: 1.0, y: 1.0 },

            image_flip_h: false,
            image_flip_v: false,
            uv_translate: SsPoint2::default(),
            uv_rotation: 0.0,
            uv_scale: SsPoint2 { x: 1.0, y: 1.0 },

            bounding_radius: 0.0,

            // TODO:
            /*
            SsCellValue		cellValue;		///< セルアニメの値
            SsPartsColorAnime partsColorValue;///< カラーアニメの値
            SsColorAnime	colorValue;		///< カラーアニメの値
            SsShaderAnime	shaderValue;	///< シェーダーアニメの値
            SsVertexAnime	vertexValue;	///< 頂点アニメの値
            SsEffectAttr	effectValue;	///< エフェクトの値
            */
            effect_time: 0,
            effect_time_total: 0.0,
            effectseed: 0,

            no_cells: false,
            is_parts_color: false,
            is_color_blend: false,
            is_shader: false,
            is_vertex_transform: false,

            is_local_alpha: false,
            is_deform: false,

            // SsSignalAttr	signalValue;

            // SsInstanceAttr	instanceValue;
            alpha_blend_type: SsBlendType::Mix,

            // std::unique_ptr<SsAnimeDecoder>		refAnime;
            // std::unique_ptr<SsEffectRenderV2>	refEffect;

            //add SS7.1
            //std::unique_ptr<SsPart> partTypeAnimeVaule;
            //--
            part_type: SsPartType::Null,

            masklimen: 0,
            mask_influence: false,

            // std::unique_ptr<SsMeshPart>	meshPart;

            //SsDeformAttr	deformValue;
            //SsAudioAttr		audioValue;
            //SsTexChangeAttr	changeTextureValue;
            part: None,
        }
    }
}
