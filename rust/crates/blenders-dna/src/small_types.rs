//! Small DNA headers: `DNA_packedFile_types.h`, `DNA_uuid_types.h`,
//! `DNA_image_enums.h`, `DNA_gpu_types.h`.

/// `PackedFile`.
#[derive(Clone, Default)]
pub struct PackedFile {
    pub size: i32,
    pub seek: i32,
    /// `const void *data` → owned bytes.
    pub data: Vec<u8>,
    /// `const ImplicitSharingInfoHandle *sharing_info` — opaque.
    pub sharing_info: usize,
}

/// `bUUID` — RFC4122 field layout.
#[derive(Clone, Copy, Default)]
pub struct BUuid {
    pub time_low: u32,
    pub time_mid: u16,
    pub time_hi_and_version: u16,
    pub clock_seq_hi_and_reserved: u8,
    pub clock_seq_low: u8,
    pub node: [u8; 6],
}

/// `ImColorMode`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i8)]
pub enum ImColorMode {
    Bw = 8,
    BwA = 16,
    Rgb = 24,
    Rgba = 32,
}

/// `GPUDOFSettings`.
#[derive(Clone, Copy, Default)]
pub struct GpuDofSettings {
    pub focus_distance: f32,
    pub fstop: f32,
    pub focal_length: f32,
    pub sensor: f32,
    pub rotation: f32,
    pub ratio: f32,
    pub num_blades: i32,
    pub high_quality: i32,
}

/* ===== DNA_session_uid_types.h ===== */

/// `SessionUID`.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct SessionUid {
    pub uid: u64,
}
pub const SESSION_UID_NONE: SessionUid = SessionUid { uid: 0 };
impl SessionUid {
    pub fn hash(&self) -> u64 {
        self.uid
    }
    pub fn is_generated(&self) -> bool {
        *self != SESSION_UID_NONE
    }
}

/* ===== DNA_view3d_enums.h ===== */

/// `eV3DOffscreenDrawFlag`.
pub const V3D_OFSDRAW_NONE: i32 = 0;
pub const V3D_OFSDRAW_SHOW_ANNOTATION: i32 = 1 << 0;
pub const V3D_OFSDRAW_OVERRIDE_SCENE_SETTINGS: i32 = 1 << 1;
pub const V3D_OFSDRAW_SHOW_GRIDFLOOR: i32 = 1 << 2;
pub const V3D_OFSDRAW_SHOW_SELECTION: i32 = 1 << 3;
pub const V3D_OFSDRAW_XR_SHOW_CONTROLLERS: i32 = 1 << 4;
pub const V3D_OFSDRAW_XR_SHOW_CUSTOM_OVERLAYS: i32 = 1 << 5;
pub const V3D_OFSDRAW_SHOW_OBJECT_EXTRAS: i32 = 1 << 6;
pub const V3D_OFSDRAW_XR_SHOW_PASSTHROUGH: i32 = 1 << 7;
pub const V3D_OFSDRAW_NO_WORLD_BACKGROUND_OVERRIDE: i32 = 1 << 8;

/// `eV3DShadingLightingMode`.
pub const V3D_LIGHTING_FLAT: i8 = 0;
pub const V3D_LIGHTING_STUDIO: i8 = 1;
pub const V3D_LIGHTING_MATCAP: i8 = 2;

/// `eV3DShadingColorType`.
pub const V3D_SHADING_MATERIAL_COLOR: i8 = 0;
pub const V3D_SHADING_RANDOM_COLOR: i8 = 1;
pub const V3D_SHADING_SINGLE_COLOR: i8 = 2;
pub const V3D_SHADING_TEXTURE_COLOR: i8 = 3;
pub const V3D_SHADING_OBJECT_COLOR: i8 = 4;
pub const V3D_SHADING_VERTEX_COLOR: i8 = 5;

/// `eV3DShadingBackgroundType`.
pub const V3D_SHADING_BACKGROUND_THEME: i8 = 0;
pub const V3D_SHADING_BACKGROUND_WORLD: i8 = 1;
pub const V3D_SHADING_BACKGROUND_VIEWPORT: i8 = 2;

/* ===== DNA_attribute_types.h ===== */

/// `AttributeArray`.
#[derive(Clone, Default)]
pub struct AttributeArray {
    /// `void *data` → owned bytes.
    pub data: Vec<u8>,
    /// `const ImplicitSharingInfoHandle *sharing_info`.
    pub sharing_info: usize,
    pub size: i64,
    pub is_single: i8,
}

/// `AttributeSingle`.
#[derive(Clone, Default)]
pub struct AttributeSingle {
    /// `void *data` → owned bytes.
    pub data: Vec<u8>,
    /// `const ImplicitSharingInfoHandle *sharing_info`.
    pub sharing_info: usize,
}

/// `Attribute`.
#[derive(Clone, Default)]
pub struct Attribute {
    /// `const char *name`.
    pub name: Option<String>,
    pub data_type: i16,
    pub domain: i8,
    pub storage_type: i8,
    /// `void *data` → owned bytes.
    pub data: Vec<u8>,
}

/* ===== DNA_world_types.h ===== */

pub const MAX_MTEX: usize = 18;

/// `eWorld_Mode`.
pub const WO_MIST: i16 = 1 << 0;

/// `eWorld_MistType`.
pub const WO_MIST_QUADRATIC: i16 = 0;
pub const WO_MIST_LINEAR: i16 = 1;
pub const WO_MIST_INVERSE_QUADRATIC: i16 = 2;

/// `eWorld_Flag`.
pub const WO_DS_EXPAND: i16 = 1 << 0;
pub const WO_DS_SHOW_TEXS: i16 = 1 << 2;
pub const WO_USE_EEVEE_FINITE_VOLUME: i16 = 1 << 3;
pub const WO_USE_SUN_SHADOW: i16 = 1 << 4;
pub const WO_USE_SUN_SHADOW_JITTER: i16 = 1 << 5;

/// `eLightProbeResolution`.
pub const LIGHT_PROBE_RESOLUTION_128: i32 = 7;
pub const LIGHT_PROBE_RESOLUTION_256: i32 = 8;
pub const LIGHT_PROBE_RESOLUTION_512: i32 = 9;
pub const LIGHT_PROBE_RESOLUTION_1024: i32 = 10;
pub const LIGHT_PROBE_RESOLUTION_2048: i32 = 11;
pub const LIGHT_PROBE_RESOLUTION_4096: i32 = 12;

/// `DEG2RADF(0.526)`.
pub const WORLD_SUN_ANGLE_DEFAULT: f32 = 0.526 * (std::f32::consts::PI / 180.0);

/// `World`.
#[derive(Clone)]
pub struct World {
    pub id: crate::id::ID,
    /// `AnimData *adt`.
    pub adt: usize,
    pub texact: i16,
    pub mistype: i16,
    pub horr: f32,
    pub horg: f32,
    pub horb: f32,
    pub exposure: f32,
    pub exp: f32,
    pub range: f32,
    pub mode: i16,
    pub flag: i16,
    pub misi: f32,
    pub miststa: f32,
    pub mistdist: f32,
    pub misthi: f32,
    pub aodist: f32,
    pub aoenergy: f32,
    pub probe_resolution: i32,
    pub sun_threshold: f32,
    pub sun_angle: f32,
    pub sun_shadow_maximum_resolution: f32,
    pub sun_shadow_jitter_overblur: f32,
    pub sun_shadow_filter_radius: f32,
    pub pr_texture: i16,
    /// DNA_DEPRECATED `use_nodes`.
    pub use_nodes: i16,
    /// `PreviewImage *preview`.
    pub preview: usize,
    /// `bNodeTree *nodetree`.
    pub nodetree: usize,
    /// `LightgroupMembership *lightgroup`.
    pub lightgroup: usize,
}
impl Default for World {
    fn default() -> Self {
        Self {
            id: crate::id::ID::default(),
            adt: crate::id::NIL,
            texact: 0,
            mistype: WO_MIST_QUADRATIC,
            horr: 0.05,
            horg: 0.05,
            horb: 0.05,
            exposure: 0.0,
            exp: 0.0,
            range: 0.0,
            mode: 0,
            flag: WO_USE_SUN_SHADOW,
            misi: 0.0,
            miststa: 5.0,
            mistdist: 25.0,
            misthi: 0.0,
            aodist: 10.0,
            aoenergy: 1.0,
            probe_resolution: LIGHT_PROBE_RESOLUTION_1024,
            sun_threshold: 10.0,
            sun_angle: WORLD_SUN_ANGLE_DEFAULT,
            sun_shadow_maximum_resolution: 0.001,
            sun_shadow_jitter_overblur: 0.0,
            sun_shadow_filter_radius: 1.0,
            pr_texture: 0,
            use_nodes: 0,
            preview: crate::id::NIL,
            nodetree: crate::id::NIL,
            lightgroup: crate::id::NIL,
        }
    }
}

/* ===== DNA_colorband_types.h ===== */

/// `eColorBand_ColorMode`.
pub const COLBAND_BLEND_RGB: i8 = 0;
pub const COLBAND_BLEND_HSV: i8 = 1;
pub const COLBAND_BLEND_HSL: i8 = 2;

/// `eColorBand_Interp`.
pub const COLBAND_INTERP_LINEAR: i8 = 0;
pub const COLBAND_INTERP_EASE: i8 = 1;
pub const COLBAND_INTERP_B_SPLINE: i8 = 2;
pub const COLBAND_INTERP_CARDINAL: i8 = 3;
pub const COLBAND_INTERP_CONSTANT: i8 = 4;

/// `eColorBand_HueInterp`.
pub const COLBAND_HUE_NEAR: i8 = 0;
pub const COLBAND_HUE_FAR: i8 = 1;
pub const COLBAND_HUE_CW: i8 = 2;
pub const COLBAND_HUE_CCW: i8 = 3;

/// `CBData`.
#[derive(Clone, Copy, Default)]
pub struct CbData {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub pos: f32,
    pub cur: i32,
}

/// `ColorBand` — `data[32]` fixed array.
#[derive(Clone)]
pub struct ColorBand {
    pub tot: i16,
    pub cur: i16,
    pub ipotype: i8,
    pub ipotype_hue: i8,
    pub color_mode: i8,
    pub data: [CbData; 32],
}
impl Default for ColorBand {
    fn default() -> Self {
        Self {
            tot: 0,
            cur: 0,
            ipotype: COLBAND_INTERP_LINEAR,
            ipotype_hue: COLBAND_HUE_NEAR,
            color_mode: COLBAND_BLEND_RGB,
            data: [CbData::default(); 32],
        }
    }
}

/* ===== DNA_text_types.h ===== */

/// `eText_Flag`.
pub const TXT_ISDIRTY: i32 = 1 << 0;
pub const TXT_ISMEM: i32 = 1 << 2;
pub const TXT_ISEXT: i32 = 1 << 3;
pub const TXT_ISSCRIPT: i32 = 1 << 4;
pub const TXT_TABSTOSPACES: i32 = 1 << 10;
pub const TXT_TABSIZE: usize = 4;

/// `TextLine`.
#[derive(Clone, Default)]
pub struct TextLine {
    pub next: usize,
    pub prev: usize,
    /// `char *line`.
    pub line: Option<String>,
    /// `char *format`.
    pub format: Option<String>,
    pub len: i32,
}

/// `Text` — the text data-block.
#[derive(Clone)]
pub struct Text {
    pub id: crate::id::ID,
    /// `char *filepath`.
    pub filepath: Option<String>,
    /// `void *compiled` — opaque.
    pub compiled: usize,
    pub flags: i32,
    /// `ListBaseT<TextLine>` — embedded lines (Rust: `Vec`).
    pub lines: Vec<TextLine>,
    /// `TextLine *curl`, `*sell` — line indices
    /// (`usize::MAX` = none).
    pub curl: usize,
    pub sell: usize,
    pub curc: i32,
    pub selc: i32,
    pub mtime: f64,
}
impl Default for Text {
    fn default() -> Self {
        Self {
            id: crate::id::ID::default(),
            filepath: None,
            compiled: crate::id::NIL,
            flags: 0,
            lines: Vec::new(),
            curl: crate::id::NIL,
            sell: crate::id::NIL,
            curc: 0,
            selc: 0,
            mtime: 0.0,
        }
    }
}
