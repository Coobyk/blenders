//! Port of `intern/attribute_legacy_convert.cc` —
//! `custom_data_type_to_attr_type` ↔ `attr_type_to_custom_data_type` +
//! the domain→`CustomData`/size tables.
//!
//! `eCustomDataType` consts come from `blenders_dna::customdata_types` —
//! the full set is emitted here for completeness (the DNA file only kept
//! `CD_MASK_*`; the type codes below are verbatim from DNA_customdata_types.h).

/// `eCustomDataType` codes (DNA_customdata_types.h).
pub const CD_AUTO_FROM_NAME: i32 = -1;
pub const CD_MVERT: i32 = 0;
pub const CD_MSTICKY: i32 = 1;
pub const CD_MDEFORMVERT: i32 = 2;
pub const CD_MEDGE: i32 = 3;
pub const CD_MFACE: i32 = 4;
pub const CD_MTFACE: i32 = 5;
pub const CD_MCOL: i32 = 6;
pub const CD_ORIGINDEX: i32 = 7;
pub const CD_NORMAL: i32 = 8;
pub const CD_FACEMAP: i32 = 9;
pub const CD_PROP_FLOAT: i32 = 10;
pub const CD_PROP_INT32: i32 = 11;
pub const CD_PROP_STRING: i32 = 12;
pub const CD_ORIGSPACE: i32 = 13;
pub const CD_ORCO: i32 = 14;
pub const CD_MTEXPOLY: i32 = 15;
pub const CD_MLOOPUV: i32 = 16;
pub const CD_PROP_BYTE_COLOR: i32 = 17;
pub const CD_TANGENT: i32 = 18;
pub const CD_MDISPS: i32 = 19;
pub const CD_PROP_FLOAT4X4: i32 = 20;
pub const CD_PROP_INT16_2D: i32 = 22;
pub const CD_CLOTH_ORCO: i32 = 23;
pub const CD_PROP_FLOAT4: i32 = 24;
pub const CD_MPOLY: i32 = 25;
pub const CD_MLOOP: i32 = 26;
pub const CD_SHAPE_KEYINDEX: i32 = 27;
pub const CD_SHAPEKEY: i32 = 28;
pub const CD_BWEIGHT: i32 = 29;
pub const CD_CREASE: i32 = 30;
pub const CD_ORIGSPACE_MLOOP: i32 = 31;
pub const CD_BM_ELEM_PYPTR: i32 = 33;
pub const CD_PAINT_MASK: i32 = 34;
pub const CD_GRID_PAINT_MASK: i32 = 35;
pub const CD_MVERT_SKIN: i32 = 36;
pub const CD_FREESTYLE_EDGE: i32 = 37;
pub const CD_FREESTYLE_FACE: i32 = 38;
pub const CD_MLOOPTANGENT: i32 = 39;
pub const CD_TESSLOOPNORMAL: i32 = 40;
pub const CD_CUSTOMLOOPNORMAL: i32 = 41;
pub const CD_SCULPT_FACE_SETS: i32 = 42;
pub const CD_PROP_INT8: i32 = 45;
pub const CD_PROP_INT32_2D: i32 = 46;
pub const CD_PROP_COLOR: i32 = 47;
pub const CD_PROP_FLOAT3: i32 = 48;
pub const CD_PROP_FLOAT2: i32 = 49;
pub const CD_PROP_BOOL: i32 = 50;
pub const CD_PROP_QUATERNION: i32 = 52;
pub const CD_NUMTYPES: i32 = 53;

/// `AttrType` (BKE_attribute.hh `enum class AttrType`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttrType {
    Bool,
    Int8,
    Int16_2D,
    Int32,
    Int32_2D,
    Float,
    Float2,
    Float3,
    Float4,
    Float4x4,
    ColorByte,
    ColorFloat,
    Quaternion,
    String,
}

/// `AttrDomain` (BKE_attribute.hh `enum class AttrDomain`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttrDomain {
    Point,
    Edge,
    Face,
    Corner,
    Spline,
    Instance,
    Layer,
    Auto,
}

/// `AttrStorageType` (BKE_attribute.hh).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttrStorageType {
    Single,
    Array,
}

/// `custom_data_type_to_attr_type` — verbatim switch.
pub fn custom_data_type_to_attr_type(data_type: i32) -> Option<AttrType> {
    match data_type {
        /* Types not used for actual CustomData layers. */
        CD_NUMTYPES | CD_AUTO_FROM_NAME | CD_TANGENT => {
            debug_assert!(false, "unexpected custom data type");
            None
        }
        /* Types only used for versioning old files. */
        CD_MVERT | CD_MSTICKY | CD_MEDGE | CD_FACEMAP | CD_MTEXPOLY | CD_MLOOPUV
        | CD_MPOLY | CD_MLOOP | CD_BWEIGHT | CD_CREASE | CD_PAINT_MASK
        | CD_CUSTOMLOOPNORMAL | CD_SCULPT_FACE_SETS | CD_MTFACE | CD_TESSLOOPNORMAL
        | CD_FREESTYLE_EDGE | CD_FREESTYLE_FACE | CD_MVERT_SKIN => None,
        /* Types only used for BMesh. */
        CD_SHAPEKEY | CD_SHAPE_KEYINDEX | CD_BM_ELEM_PYPTR => None,
        /* Mesh-struct layers, not generic attributes. */
        CD_MDEFORMVERT | CD_ORCO | CD_CLOTH_ORCO => None,
        CD_NORMAL | CD_MDISPS | CD_ORIGSPACE_MLOOP | CD_GRID_PAINT_MASK => None,
        CD_ORIGINDEX => None,
        CD_MLOOPTANGENT => None,
        CD_MFACE | CD_ORIGSPACE | CD_MCOL => None,
        /* Generic property types. */
        CD_PROP_FLOAT => Some(AttrType::Float),
        CD_PROP_INT32 => Some(AttrType::Int32),
        CD_PROP_BYTE_COLOR => Some(AttrType::ColorByte),
        CD_PROP_FLOAT4X4 => Some(AttrType::Float4x4),
        CD_PROP_INT16_2D => Some(AttrType::Int16_2D),
        CD_PROP_INT8 => Some(AttrType::Int8),
        CD_PROP_INT32_2D => Some(AttrType::Int32_2D),
        CD_PROP_COLOR => Some(AttrType::ColorFloat),
        CD_PROP_FLOAT3 => Some(AttrType::Float3),
        CD_PROP_FLOAT4 => Some(AttrType::Float4),
        CD_PROP_FLOAT2 => Some(AttrType::Float2),
        CD_PROP_BOOL => Some(AttrType::Bool),
        CD_PROP_STRING => Some(AttrType::String),
        CD_PROP_QUATERNION => Some(AttrType::Quaternion),
        _ => None,
    }
}

/// `attr_type_to_custom_data_type` — verbatim switch.
pub fn attr_type_to_custom_data_type(attr_type: AttrType) -> Option<i32> {
    match attr_type {
        AttrType::Bool => Some(CD_PROP_BOOL),
        AttrType::Int8 => Some(CD_PROP_INT8),
        AttrType::Int16_2D => Some(CD_PROP_INT16_2D),
        AttrType::Int32 => Some(CD_PROP_INT32),
        AttrType::Int32_2D => Some(CD_PROP_INT32_2D),
        AttrType::Float => Some(CD_PROP_FLOAT),
        AttrType::Float2 => Some(CD_PROP_FLOAT2),
        AttrType::Float3 => Some(CD_PROP_FLOAT3),
        AttrType::Float4 => Some(CD_PROP_FLOAT4),
        AttrType::Float4x4 => Some(CD_PROP_FLOAT4X4),
        AttrType::ColorByte => Some(CD_PROP_BYTE_COLOR),
        AttrType::ColorFloat => Some(CD_PROP_COLOR),
        AttrType::Quaternion => Some(CD_PROP_QUATERNION),
        AttrType::String => Some(CD_PROP_STRING),
        _ => None,
    }
}

/// `get_domain_size` — mesh element count for a domain.
pub fn get_domain_size(domain: AttrDomain, verts_num: i32, edges_num: i32, faces_num: i32, corners_num: i32) -> i32 {
    match domain {
        AttrDomain::Point => verts_num,
        AttrDomain::Edge => edges_num,
        AttrDomain::Face => faces_num,
        AttrDomain::Corner => corners_num,
        _ => 0,
    }
}
