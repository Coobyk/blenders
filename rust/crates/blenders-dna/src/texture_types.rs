//! Port of `DNA_texture_types.h` — `Tex`, `MTex`, `TexMapping`, `ColorMapping`.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::color_types::CurveMapping;
use crate::small_types::ColorBand;
use crate::id::{ID, NIL};
use crate::image_types::ImageUser;
use crate::material_types::{MAP_COL, TEXCO_UV};
/// `eTexMapping_Flag`.
pub type TexmappingFlag = i32;
pub const TEXMAP_CLIP_MIN: TexmappingFlag = 1 << 0;
pub const TEXMAP_CLIP_MAX: TexmappingFlag = 1 << 1;
pub const TEXMAP_UNIT_MATRIX: TexmappingFlag = 1 << 2;

/// `eTexMapping_Type`.
pub type TexmappingType = i32;
pub const TEXMAP_TYPE_POINT: TexmappingType = 0;
pub const TEXMAP_TYPE_TEXTURE: TexmappingType = 1;
pub const TEXMAP_TYPE_VECTOR: TexmappingType = 2;
pub const TEXMAP_TYPE_NORMAL: TexmappingType = 3;

/// `eColorMapping_Flag`.
pub type ColormappingFlag = i32;
pub const COLORMAP_USE_RAMP: ColormappingFlag = 1;

/// `eTex_Type`.
pub type TexType = i16;
pub const TEX_CLOUDS: TexType = 1;
pub const TEX_WOOD: TexType = 2;
pub const TEX_MARBLE: TexType = 3;
pub const TEX_MAGIC: TexType = 4;
pub const TEX_BLEND: TexType = 5;
pub const TEX_STUCCI: TexType = 6;
pub const TEX_NOISE: TexType = 7;
pub const TEX_IMAGE: TexType = 8;
pub const TEX_MUSGRAVE: TexType = 11;
pub const TEX_VORONOI: TexType = 12;
pub const TEX_DISTNOISE: TexType = 13;

/// `eTex_MusgraveType`.
pub type TexMusgravetype = i16;
pub const TEX_MFRACTAL: TexMusgravetype = 0;
pub const TEX_RIDGEDMF: TexMusgravetype = 1;
pub const TEX_HYBRIDMF: TexMusgravetype = 2;
pub const TEX_FBM: TexMusgravetype = 3;
pub const TEX_HTERRAIN: TexMusgravetype = 4;

/// `eTex_NoiseBasis`.
pub type TexNoisebasis = i16;
pub const TEX_BLENDER: TexNoisebasis = 0;
pub const TEX_STDPERLIN: TexNoisebasis = 1;
pub const TEX_NEWPERLIN: TexNoisebasis = 2;
pub const TEX_VORONOI_F1: TexNoisebasis = 3;
pub const TEX_VORONOI_F2: TexNoisebasis = 4;
pub const TEX_VORONOI_F3: TexNoisebasis = 5;
pub const TEX_VORONOI_F4: TexNoisebasis = 6;
pub const TEX_VORONOI_F2F1: TexNoisebasis = 7;
pub const TEX_VORONOI_CRACKLE: TexNoisebasis = 8;
pub const TEX_CELLNOISE: TexNoisebasis = 14;

/// `eTex_VoronoiDistMetric`.
pub type TexVoronoidistmetric = i16;
pub const TEX_DISTANCE: TexVoronoidistmetric = 0;
pub const TEX_DISTANCE_SQUARED: TexVoronoidistmetric = 1;
pub const TEX_MANHATTAN: TexVoronoidistmetric = 2;
pub const TEX_CHEBYCHEV: TexVoronoidistmetric = 3;
pub const TEX_MINKOVSKY_HALF: TexVoronoidistmetric = 4;
pub const TEX_MINKOVSKY_FOUR: TexVoronoidistmetric = 5;
pub const TEX_MINKOVSKY: TexVoronoidistmetric = 6;

/// `eTex_ImaFlag`.
pub type TexImaflag = i16;
pub const TEX_INTERPOL: TexImaflag = 1 << 0;
pub const TEX_USEALPHA: TexImaflag = 1 << 1;
pub const TEX_IMAROT: TexImaflag = 1 << 4;
pub const TEX_CALCALPHA: TexImaflag = 1 << 5;
pub const TEX_NORMALMAP: TexImaflag = 1 << 11;
pub const TEX_DERIVATIVEMAP: TexImaflag = 1 << 14;

/// `eTex_Flag`.
pub type TexFlag = i16;
pub const TEX_COLORBAND: TexFlag = 1 << 0;
pub const TEX_FLIPBLEND: TexFlag = 1 << 1;
pub const TEX_NEGALPHA: TexFlag = 1 << 2;
pub const TEX_CHECKER_ODD: TexFlag = 1 << 3;
pub const TEX_CHECKER_EVEN: TexFlag = 1 << 4;
pub const TEX_PRV_ALPHA: TexFlag = 1 << 5;
pub const TEX_PRV_NOR: TexFlag = 1 << 6;
pub const TEX_REPEAT_XMIR: TexFlag = 1 << 7;
pub const TEX_REPEAT_YMIR: TexFlag = 1 << 8;
pub const TEX_DS_EXPAND: TexFlag = 1 << 9;
pub const TEX_NO_CLAMP: TexFlag = 1 << 10;

/// `eTex_Extend`.
pub type TexExtend = i16;
pub const TEX_EXTEND: TexExtend = 1;
pub const TEX_CLIP: TexExtend = 2;
pub const TEX_REPEAT: TexExtend = 3;
pub const TEX_CLIPCUBE: TexExtend = 4;
pub const TEX_CHECKER: TexExtend = 5;

/// `eTex_NoiseType`.
pub type TexNoisetype = i16;
pub const TEX_NOISESOFT: TexNoisetype = 0;
pub const TEX_NOISEPERL: TexNoisetype = 1;

/// `eTex_WoodWaveform`.
pub type TexWoodwaveform = i16;
pub const TEX_SIN: TexWoodwaveform = 0;
pub const TEX_SAW: TexWoodwaveform = 1;
pub const TEX_TRI: TexWoodwaveform = 2;

/// `eTex_WoodType`.
pub type TexWoodtype = i16;
pub const TEX_BAND: TexWoodtype = 0;
pub const TEX_RING: TexWoodtype = 1;
pub const TEX_BANDNOISE: TexWoodtype = 2;
pub const TEX_RINGNOISE: TexWoodtype = 3;

/// `eTex_CloudType`.
pub type TexCloudtype = i16;
pub const TEX_DEFAULT: TexCloudtype = 0;
pub const TEX_COLOR: TexCloudtype = 1;

/// `eTex_MarbleType`.
pub type TexMarbletype = i16;
pub const TEX_SOFT: TexMarbletype = 0;
pub const TEX_SHARP: TexMarbletype = 1;
pub const TEX_SHARPER: TexMarbletype = 2;

/// `eTex_BlendType`.
pub type TexBlendtype = i16;
pub const TEX_LIN: TexBlendtype = 0;
pub const TEX_QUAD: TexBlendtype = 1;
pub const TEX_EASE: TexBlendtype = 2;
pub const TEX_DIAG: TexBlendtype = 3;
pub const TEX_SPHERE: TexBlendtype = 4;
pub const TEX_HALO: TexBlendtype = 5;
pub const TEX_RAD: TexBlendtype = 6;

/// `eTex_StucciType`.
pub type TexStuccitype = i16;
pub const TEX_PLASTIC: TexStuccitype = 0;
pub const TEX_WALLIN: TexStuccitype = 1;
pub const TEX_WALLOUT: TexStuccitype = 2;

/// `eTex_VoronoiColType`.
pub type TexVoronoicoltype = i16;
pub const TEX_INTENSITY: TexVoronoicoltype = 0;
pub const TEX_COL1: TexVoronoicoltype = 1;
pub const TEX_COL2: TexVoronoicoltype = 2;
pub const TEX_COL3: TexVoronoicoltype = 3;

/// `eTex_ReturnValue`.
pub type TexReturnvalue = i16;
pub const TEX_INT: TexReturnvalue = 0;
pub const TEX_RGB: TexReturnvalue = 1;

/// `eTex_PreviewType`.
pub type TexPreviewtype = i16;
pub const TEX_PR_TEXTURE: TexPreviewtype = 0;
pub const TEX_PR_OTHER: TexPreviewtype = 1;
pub const TEX_PR_BOTH: TexPreviewtype = 2;

/// `eTex_Projection`.
pub type TexProjection = i8;
pub const PROJ_N: TexProjection = 0;
pub const PROJ_X: TexProjection = 1;
pub const PROJ_Y: TexProjection = 2;
pub const PROJ_Z: TexProjection = 3;

/// `eMTex_Mapping`.
pub type MtexMapping = i8;
pub const MTEX_FLAT: MtexMapping = 0;
pub const MTEX_CUBE: MtexMapping = 1;
pub const MTEX_TUBE: MtexMapping = 2;
pub const MTEX_SPHERE: MtexMapping = 3;

/// `eMTex_BlendType`.
pub type MtexBlendtype = i16;
pub const MTEX_BLEND: MtexBlendtype = 0;
pub const MTEX_MUL: MtexBlendtype = 1;
pub const MTEX_ADD: MtexBlendtype = 2;
pub const MTEX_SUB: MtexBlendtype = 3;
pub const MTEX_DIV: MtexBlendtype = 4;
pub const MTEX_DARK: MtexBlendtype = 5;
pub const MTEX_DIFF: MtexBlendtype = 6;
pub const MTEX_LIGHT: MtexBlendtype = 7;
pub const MTEX_SCREEN: MtexBlendtype = 8;
pub const MTEX_OVERLAY: MtexBlendtype = 9;
pub const MTEX_BLEND_HUE: MtexBlendtype = 10;
pub const MTEX_BLEND_SAT: MtexBlendtype = 11;
pub const MTEX_BLEND_VAL: MtexBlendtype = 12;
pub const MTEX_BLEND_COLOR: MtexBlendtype = 13;
pub const MTEX_SOFT_LIGHT: MtexBlendtype = 15;
pub const MTEX_LIN_LIGHT: MtexBlendtype = 16;

/// `eMTex_BrushMapMode`.
pub type MtexBrushmapmode = i8;
pub const MTEX_MAP_MODE_VIEW: MtexBrushmapmode = 0;
pub const MTEX_MAP_MODE_TILED: MtexBrushmapmode = 1;
pub const MTEX_MAP_MODE_3D: MtexBrushmapmode = 2;
pub const MTEX_MAP_MODE_AREA: MtexBrushmapmode = 3;
pub const MTEX_MAP_MODE_RANDOM: MtexBrushmapmode = 4;
pub const MTEX_MAP_MODE_STENCIL: MtexBrushmapmode = 5;

/// `eMTex_BrushAngleMode`.
pub type MtexBrushanglemode = i8;
pub const MTEX_ANGLE_RANDOM: MtexBrushanglemode = 1;
pub const MTEX_ANGLE_RAKE: MtexBrushanglemode = 2;


/// `MTex` — a texture slot on a material/light/world.
#[derive(Clone)]
pub struct MTex {
    pub texco: i16,
    pub mapto: i16,
    pub blendtype: i16,
    /// `Object *object`.
    pub object: usize,
    /// `Tex *tex`.
    pub tex: usize,
    /// `char uvname[68]`.
    pub uvname: [u8; 68],
    pub projx: i8,
    pub projy: i8,
    pub projz: i8,
    pub mapping: i8,
    pub brush_map_mode: i8,
    pub brush_angle_mode: i8,
    pub which_output: i16,
    pub ofs: [f32; 3],
    pub size: [f32; 3],
    pub rot: f32,
    pub random_angle: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub k: f32,
    pub def_var: f32,
    pub colfac: f32,
    pub alphafac: f32,
    pub timefac: f32,
    pub lengthfac: f32,
    pub clumpfac: f32,
    pub dampfac: f32,
    pub kinkfac: f32,
    pub kinkampfac: f32,
    pub roughfac: f32,
    pub padensfac: f32,
    pub gravityfac: f32,
    pub lifefac: f32,
    pub sizefac: f32,
    pub ivelfac: f32,
    pub fieldfac: f32,
    pub twistfac: f32,
}
impl Default for MTex {
    fn default() -> Self {
        Self {
            texco: TEXCO_UV as i16,
            mapto: MAP_COL as i16,
            blendtype: MTEX_BLEND,
            object: NIL,
            tex: NIL,
            uvname: [0; 68],
            projx: PROJ_X as i8,
            projy: PROJ_Y as i8,
            projz: PROJ_Z as i8,
            mapping: MTEX_FLAT as i8,
            brush_map_mode: MTEX_MAP_MODE_VIEW as i8,
            brush_angle_mode: 0,
            which_output: 0,
            ofs: [0.0; 3],
            size: [1.0; 3],
            rot: 0.0,
            random_angle: 2.0 * std::f32::consts::PI,
            r: 1.0,
            g: 0.0,
            b: 1.0,
            k: 1.0,
            def_var: 1.0,
            colfac: 1.0,
            alphafac: 1.0,
            timefac: 1.0,
            lengthfac: 1.0,
            clumpfac: 1.0,
            dampfac: 1.0,
            kinkfac: 1.0,
            kinkampfac: 1.0,
            roughfac: 1.0,
            padensfac: 1.0,
            gravityfac: 1.0,
            lifefac: 1.0,
            sizefac: 1.0,
            ivelfac: 1.0,
            fieldfac: 1.0,
            twistfac: 1.0,
        }
    }
}

/// `Tex_Runtime`.
#[derive(Clone, Default)]
pub struct TexRuntime {
    pub last_update: u64,
}

/// `Tex` — texture ID block.
#[derive(Clone)]
pub struct Tex {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    pub noisesize: f32,
    pub turbul: f32,
    pub bright: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub rfac: f32,
    pub gfac: f32,
    pub bfac: f32,
    pub filtersize: f32,
    pub mg_h: f32,
    pub mg_lacunarity: f32,
    pub mg_octaves: f32,
    pub mg_offset: f32,
    pub mg_gain: f32,
    pub dist_amount: f32,
    pub ns_outscale: f32,
    pub vn_w1: f32,
    pub vn_w2: f32,
    pub vn_w3: f32,
    pub vn_w4: f32,
    pub vn_mexp: f32,
    pub vn_distm: i16,
    pub vn_coltype: i16,
    pub noisedepth: i16,
    pub noisetype: i16,
    pub noisebasis: i16,
    pub noisebasis2: i16,
    pub imaflag: i16,
    pub flag: i16,
    pub type_: i16,
    pub stype: i16,
    pub cropxmin: f32,
    pub cropymin: f32,
    pub cropxmax: f32,
    pub cropymax: f32,
    pub xrepeat: i16,
    pub yrepeat: i16,
    pub extend: i16,
    /// `DNA_DEPRECATED int len`.
    pub len: i32,
    /// `DNA_DEPRECATED int frames`.
    pub frames: i32,
    /// `DNA_DEPRECATED int offset`.
    pub offset: i32,
    /// `DNA_DEPRECATED int sfra`.
    pub sfra: i32,
    pub checkerdist: f32,
    pub nabla: f32,
    pub iuser: ImageUser,
    /// `bNodeTree *nodetree`.
    pub nodetree: usize,
    /// `Image *ima`.
    pub ima: usize,
    /// `ColorBand *coba`.
    pub coba: usize,
    /// `PreviewImage *preview`.
    pub preview: usize,
    pub use_nodes: i8,
    pub runtime: TexRuntime,
}
impl Default for Tex {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            noisesize: 0.25,
            turbul: 5.0,
            bright: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            rfac: 1.0,
            gfac: 1.0,
            bfac: 1.0,
            filtersize: 1.0,
            mg_h: 1.0,
            mg_lacunarity: 2.0,
            mg_octaves: 2.0,
            mg_offset: 1.0,
            mg_gain: 1.0,
            dist_amount: 1.0,
            ns_outscale: 1.0,
            vn_w1: 1.0,
            vn_w2: 0.0,
            vn_w3: 0.0,
            vn_w4: 0.0,
            vn_mexp: 2.5,
            vn_distm: 0,
            vn_coltype: 0,
            noisedepth: 2,
            noisetype: 0,
            noisebasis: 0,
            noisebasis2: 0,
            imaflag: TEX_INTERPOL | TEX_USEALPHA,
            flag: TEX_CHECKER_ODD | TEX_NO_CLAMP,
            type_: TEX_IMAGE,
            stype: 0,
            cropxmin: 0.0,
            cropymin: 0.0,
            cropxmax: 1.0,
            cropymax: 1.0,
            xrepeat: 1,
            yrepeat: 1,
            extend: TEX_REPEAT,
            len: 0,
            frames: 0,
            offset: 0,
            sfra: 1,
            checkerdist: 0.0,
            nabla: 0.025,
            iuser: ImageUser::default(),
            nodetree: NIL,
            ima: NIL,
            coba: NIL,
            preview: NIL,
            use_nodes: 0,
            runtime: TexRuntime::default(),
        }
    }
}

/// `TexMapping`.
#[derive(Clone)]
pub struct TexMapping {
    pub loc: [f32; 3],
    pub rot: [f32; 3],
    pub size: [f32; 3],
    pub flag: i32,
    pub projx: i8,
    pub projy: i8,
    pub projz: i8,
    pub mapping: i8,
    pub type_: i32,
    pub mat: [[f32; 4]; 4],
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// `Object *ob`.
    pub ob: usize,
}
impl Default for TexMapping {
    fn default() -> Self {
        Self {
            loc: [0.0; 3],
            rot: [0.0; 3],
            size: [0.0; 3],
            flag: 0,
            projx: 0,
            projy: 0,
            projz: 0,
            mapping: 0,
            type_: 0,
            mat: [[0.0; 4]; 4],
            min: [0.0; 3],
            max: [0.0; 3],
            ob: NIL,
        }
    }
}

/// `ColorMapping`.
#[derive(Clone, Default)]
pub struct ColorMapping {
    pub coba: ColorBand,
    pub bright: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub flag: i32,
    pub blend_color: [f32; 3],
    pub blend_factor: f32,
    pub blend_type: i32,
}

/// `Tex` helper (struct kept for parity with header tail section).
pub type CurveMappingRef = usize;
