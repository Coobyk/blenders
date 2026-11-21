//! Port of `DNA_meshdata_types.h` — legacy mesh element structs
//! (MVert/MEdge/MLoop/MPoly/MFace + prop/deform/skin types).

/// `eMSelect_Type`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(i32)]
pub enum MSelectType {
    #[default]
    MeVsel = 0,
    MeEsel = 1,
    MeFsel = 2,
}

/// `eMVertSkinFlag` (DNA_DEPRECATED_ALLOW).
pub const MVERT_SKIN_ROOT: i32 = 1;
pub const MVERT_SKIN_LOOSE: i32 = 2;

/// `eMFace_EdgeCode`.
pub const ME_V1V2: i8 = 1 << 0;
pub const ME_V2V3: i8 = 1 << 1;
pub const ME_V3V1: i8 = 1 << 2;
pub const ME_V3V4: i8 = ME_V3V1;
pub const ME_V4V1: i8 = 1 << 3;

/// `MSelect`.
#[derive(Clone, Copy, Default)]
pub struct MSelect {
    pub index: i32,
    pub type_: MSelectType,
}

/// `MFloatProperty` — float prop layer cell.
#[derive(Clone, Copy, Default)]
pub struct MFloatProperty {
    pub f: f32,
}
/// `MIntProperty`.
#[derive(Clone, Copy, Default)]
pub struct MIntProperty {
    pub i: i32,
}
/// `MStringProperty`.
#[derive(Clone)]
pub struct MStringProperty {
    pub s: [u8; 255],
    pub s_len: u8,
}
impl Default for MStringProperty {
    fn default() -> Self {
        Self {
            s: [0; 255],
            s_len: 0,
        }
    }
}
/// `MBoolProperty`.
#[derive(Clone, Copy, Default)]
pub struct MBoolProperty {
    pub b: u8,
}
/// `MInt8Property`.
#[derive(Clone, Copy, Default)]
pub struct MInt8Property {
    pub i: i8,
}

/// `MDeformWeight`.
#[derive(Clone, Copy, Default)]
pub struct MDeformWeight {
    pub def_nr: u32,
    pub weight: f32,
}

/// `MDeformVert` — vertex → (def_nr,weight) list.
#[derive(Clone, Default)]
pub struct MDeformVert {
    /// `MDeformWeight *dw` → Vec.
    pub dw: Vec<MDeformWeight>,
    pub totweight: i32,
    pub flag: i32,
}

/// `MVertSkin` (DNA_DEPRECATED_ALLOW).
#[derive(Clone, Copy, Default)]
pub struct MVertSkin {
    pub radius: [f32; 3],
    pub flag: i32,
}

/// `MLoopCol`.
#[derive(Clone, Copy, Default)]
pub struct MLoopCol {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// `MPropCol` — float4 color prop cell.
#[derive(Clone, Copy, Default)]
pub struct MPropCol {
    pub color: [f32; 4],
}

/// `MDisps` — multires displacement grid.
#[derive(Clone)]
pub struct MDisps {
    pub totdisp: i32,
    /// `float (*disps)[3]` → Vec of xyz.
    pub disps: Vec<[f32; 3]>,
    /// `unsigned int *hidden` → bitmap Vec.
    pub hidden: Vec<u32>,
}
impl Default for MDisps {
    fn default() -> Self {
        Self {
            totdisp: 0,
            disps: Vec::new(),
            hidden: Vec::new(),
        }
    }
}

/// `GridPaintMask`.
#[derive(Clone, Default)]
pub struct GridPaintMask {
    /// `float *data`.
    pub data: Vec<f32>,
    pub level: u32,
}

/// `OrigSpaceFace`.
#[derive(Clone, Copy, Default)]
pub struct OrigSpaceFace {
    pub uv: [[f32; 2]; 4],
}
/// `OrigSpaceLoop`.
#[derive(Clone, Copy, Default)]
pub struct OrigSpaceLoop {
    pub uv: [f32; 2],
}

/// `eFreestyleEdge_Flag` / `eFreestyleFace_Flag` (DNA_DEPRECATED_ALLOW).
pub const FREESTYLE_EDGE_MARK: i8 = 1;
pub const FREESTYLE_FACE_MARK: i8 = 1;

/// `FreestyleEdge`.
#[derive(Clone, Copy, Default)]
pub struct FreestyleEdge {
    pub flag: i8,
}
/// `FreestyleFace`.
#[derive(Clone, Copy, Default)]
pub struct FreestyleFace {
    pub flag: i8,
}

/// `eMEdge_Flag`.
pub const ME_SEAM: i16 = 1 << 2;
pub const ME_LOOSEEDGE: i16 = 1 << 7;
pub const ME_SHARP: i16 = 1 << 9;

/// `MEdge` (legacy).
#[derive(Clone, Copy, Default)]
pub struct MEdge {
    pub v1: u32,
    pub v2: u32,
    pub crease_legacy: i8,
    pub bweight_legacy: i8,
    pub flag_legacy: i16,
}

/// `eMPoly_Flag`.
pub const ME_SMOOTH: i8 = 1 << 0;
pub const ME_FACE_SEL: i8 = 1 << 1;
/// `eMVert_Flag`.
pub const ME_HIDE: i8 = 1 << 4;

/// `MPoly` (legacy).
#[derive(Clone, Copy, Default)]
pub struct MPoly {
    pub loopstart: i32,
    pub totloop: i32,
    pub mat_nr_legacy: i16,
    pub flag_legacy: i8,
}

/// `eMLoopUV_Flag`.
pub const MLOOPUV_EDGESEL: i32 = 1 << 0;
pub const MLOOPUV_VERTSEL: i32 = 1 << 1;
pub const MLOOPUV_PINNED: i32 = 1 << 2;

/// `MLoopUV`.
#[derive(Clone, Copy, Default)]
pub struct MLoopUV {
    pub uv: [f32; 2],
    pub flag: i32,
}

/// `MVert` (legacy).
#[derive(Clone, Copy, Default)]
pub struct MVert {
    pub co_legacy: [f32; 3],
    pub flag_legacy: i8,
    pub bweight_legacy: i8,
}

/// `MLoop` (legacy).
#[derive(Clone, Copy, Default)]
pub struct MLoop {
    pub v: u32,
    pub e: u32,
}

/// `MFace` — legacy tessellated face.
#[derive(Clone, Copy, Default)]
pub struct MFace {
    pub v1: u32,
    pub v2: u32,
    pub v3: u32,
    pub v4: u32,
    pub mat_nr: i16,
    pub edcode: i8,
    pub flag: i8,
}

/// `MTFace`.
#[derive(Clone, Copy, Default)]
pub struct MTFace {
    pub uv: [[f32; 2]; 4],
}

/// `MCol` — byte color (a,r,g,b).
#[derive(Clone, Copy, Default)]
pub struct MCol {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// `MRecast` (DNA_DEPRECATED_ALLOW).
#[derive(Clone, Copy, Default)]
pub struct MRecast {
    pub i: i32,
}
