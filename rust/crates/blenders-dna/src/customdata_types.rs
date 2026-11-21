//! Port of `DNA_customdata_types.h` — `eCustomDataType`, `CustomDataLayer`,
//! `CustomData`, `CustomDataExternal`, `CustomData_MeshMasks`, masks.

/// `eCustomDataType`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(i32)]
pub enum CustomDataType {
    #[default]
    CdAutoFromName = -1,
    CdMDeformVert = 2,
    CdMFace = 4,
    CdMTFace = 5,
    CdMCol = 6,
    CdOrigIndex = 7,
    CdNormal = 8,
    CdPropFloat = 10,
    CdPropInt32 = 11,
    CdPropString = 12,
    CdOrigSpace = 13,
    CdOrco = 14,
    CdPropByteColor = 17,
    CdTangent = 18,
    CdMDisps = 19,
    CdPropFloat4x4 = 20,
    CdPropInt16_2d = 22,
    CdClothOrco = 23,
    CdPropFloat4 = 24,
    CdShapeKeyIndex = 27,
    CdShapeKey = 28,
    CdOrigSpaceMLoop = 31,
    CdBmElemPyptr = 33,
    CdGridPaintMask = 35,
    CdMLoopTangent = 39,
    CdTessLoopNormal = 40,
    CdPropInt8 = 45,
    CdPropInt32_2d = 46,
    CdPropColor = 47,
    CdPropFloat3 = 48,
    CdPropFloat2 = 49,
    CdPropBool = 50,
    CdPropQuaternion = 52,
}
pub const CD_NUMTYPES: i32 = 53;

/// `CD_MASK_*`.
pub const CD_MASK_MDEFORMVERT: u64 = 1 << 2;
pub const CD_MASK_MFACE: u64 = 1 << 4;
pub const CD_MASK_MTFACE: u64 = 1 << 5;
pub const CD_MASK_MCOL: u64 = 1 << 6;
pub const CD_MASK_ORIGINDEX: u64 = 1 << 7;
pub const CD_MASK_NORMAL: u64 = 1 << 8;
pub const CD_MASK_PROP_FLOAT: u64 = 1 << 10;
pub const CD_MASK_PROP_INT32: u64 = 1 << 11;
pub const CD_MASK_PROP_STRING: u64 = 1 << 12;
pub const CD_MASK_ORIGSPACE: u64 = 1 << 13;
pub const CD_MASK_ORCO: u64 = 1 << 14;
pub const CD_MASK_PROP_BYTE_COLOR: u64 = 1 << 17;
pub const CD_MASK_MDISPS: u64 = 1 << 19;
pub const CD_MASK_CLOTH_ORCO: u64 = 1 << 23;
pub const CD_MASK_SHAPE_KEYINDEX: u64 = 1 << 27;
pub const CD_MASK_SHAPEKEY: u64 = 1 << 28;
pub const CD_MASK_ORIGSPACE_MLOOP: u64 = 1 << 31;
pub const CD_MASK_BM_ELEM_PYPTR: u64 = 1 << 33;
pub const CD_MASK_GRID_PAINT_MASK: u64 = 1 << 35;
pub const CD_MASK_MLOOPTANGENT: u64 = 1 << 39;
pub const CD_MASK_TESSLOOPNORMAL: u64 = 1 << 40;
pub const CD_MASK_PROP_COLOR: u64 = 1 << 47;
pub const CD_MASK_PROP_FLOAT4: u64 = 1 << 24;
pub const CD_MASK_PROP_FLOAT3: u64 = 1 << 48;
pub const CD_MASK_PROP_FLOAT2: u64 = 1 << 49;
pub const CD_MASK_PROP_BOOL: u64 = 1 << 50;
pub const CD_MASK_PROP_INT8: u64 = 1 << 45;
pub const CD_MASK_PROP_INT16_2D: u64 = 1 << 22;
pub const CD_MASK_PROP_INT32_2D: u64 = 1 << 46;
pub const CD_MASK_PROP_QUATERNION: u64 = 1 << 52;
pub const CD_MASK_PROP_FLOAT4X4: u64 = 1 << 20;
pub const CD_MASK_MULTIRES_GRIDS: u64 = CD_MASK_MDISPS | CD_MASK_GRID_PAINT_MASK;
pub const CD_MASK_ALL: u64 = !0;
pub const CD_MASK_PROP_ALL: u64 = CD_MASK_PROP_FLOAT | CD_MASK_PROP_FLOAT2 | CD_MASK_PROP_FLOAT3
    | CD_MASK_PROP_FLOAT4 | CD_MASK_PROP_INT32 | CD_MASK_PROP_COLOR | CD_MASK_PROP_STRING
    | CD_MASK_PROP_BYTE_COLOR | CD_MASK_PROP_BOOL | CD_MASK_PROP_INT8 | CD_MASK_PROP_INT16_2D
    | CD_MASK_PROP_INT32_2D | CD_MASK_PROP_QUATERNION | CD_MASK_PROP_FLOAT4X4;
pub const CD_MASK_COLOR_ALL: u64 = CD_MASK_PROP_COLOR | CD_MASK_PROP_BYTE_COLOR;

pub const MAX_MTFACE: usize = 8;
pub const MAX_CUSTOMDATA_LAYER_NAME: usize = 68;
pub const MAX_CUSTOMDATA_LAYER_NAME_NO_PREFIX: usize = 64;

/// `eCustomDataLayer_Flag`.
pub const CD_FLAG_NOCOPY: i32 = 1 << 0;
pub const CD_FLAG_UNUSED: i32 = 1 << 1;
pub const CD_FLAG_TEMPORARY: i32 = (1 << 2) | CD_FLAG_NOCOPY;
pub const CD_FLAG_EXTERNAL: i32 = 1 << 3;
pub const CD_FLAG_IN_MEMORY: i32 = 1 << 4;
pub const CD_FLAG_RUNTIME_NOINTERP: i32 = 1 << 7;

/// `CustomDataLayer` — `data`/`sharing_info` opaque.
#[derive(Clone)]
pub struct CustomDataLayer {
    pub type_: CustomDataType,
    pub offset: i32,
    pub flag: i32,
    pub active: i32,
    pub active_rnd: i32,
    pub uid: i32,
    /// `char name[MAX_CUSTOMDATA_LAYER_NAME]`.
    pub name: [u8; MAX_CUSTOMDATA_LAYER_NAME],
    /// `void *data` → opaque arena index / 0.
    pub data: usize,
    /// `const ImplicitSharingInfoHandle *sharing_info`.
    pub sharing_info: usize,
}

impl Default for CustomDataLayer {
    fn default() -> Self {
        Self {
            type_: CustomDataType::default(),
            offset: 0,
            flag: 0,
            active: 0,
            active_rnd: 0,
            uid: 0,
            name: [0; MAX_CUSTOMDATA_LAYER_NAME],
            data: 0,
            sharing_info: 0,
        }
    }
}

/// `CustomDataExternal`.
#[derive(Clone)]
pub struct CustomDataExternal {
    /// `char filepath[FILE_MAX]`.
    pub filepath: [u8; crate::id::FILE_MAX],
}
impl Default for CustomDataExternal {
    fn default() -> Self {
        Self {
            filepath: [0; crate::id::FILE_MAX],
        }
    }
}

/// `CustomData` — per-element-type layer stack.
#[derive(Clone)]
pub struct CustomData {
    /// `CustomDataLayer *layers` → owned Vec.
    pub layers: Vec<CustomDataLayer>,
    /// `int typemap[CD_NUMTYPES]` — start index of each type.
    pub typemap: [i32; CD_NUMTYPES as usize],
    pub totlayer: i32,
    pub maxlayer: i32,
    pub totsize: i32,
    /// `BLI_mempool *pool` — opaque.
    pub pool: usize,
    /// `CustomDataExternal *external` → Box.
    pub external: Option<Box<CustomDataExternal>>,
}

/// `eCustomDataMask`.
pub type CustomDataMask = u64;

/// `CustomData_MeshMasks`.
#[derive(Clone, Copy, Default)]
pub struct CustomDataMeshMasks {
    pub vmask: u64,
    pub emask: u64,
    pub fmask: u64,
    pub pmask: u64,
    pub lmask: u64,
}

impl Default for CustomData {
    fn default() -> Self {
        Self {
            layers: Vec::new(),
            typemap: [0; CD_NUMTYPES as usize],
            totlayer: 0,
            maxlayer: 0,
            totsize: 0,
            pool: 0,
            external: None,
        }
    }
}
