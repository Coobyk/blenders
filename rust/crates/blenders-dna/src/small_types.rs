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
