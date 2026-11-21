//! Port of `DNA_ID_enums.h` — icon sizes, `eIDPropertyType`,
//! `ID_Type` codes and related enums.

/// `eIconSizes`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum IconSize {
    IconSizeIcon = 0,
    IconSizePreview = 1,
    NumIconSizes,
}

pub const IDP_STRUCT_BASE: i32 = 64;

/// `eIDPropertyType` — full set (supersedes the small enum in `id.rs`).
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(i8)]
pub enum IDPropertyType {
    IdpString = 0,
    IdpInt = 1,
    IdpFloat = 2,
    IdpArray = 5,
    IdpGroup = 6,
    IdpId = 7,
    IdpDouble = 8,
    IdpIdpArray = 9,
    IdpBoolean = 10,
    IdpCurveMapping = 64,
    IdpColorBand = 65,
    IdpCurveProfile = 66,
    #[default]
    IdpNone = 127,
}
pub const IDP_NUM_BASIC_TYPES: i32 = 11;
pub const IDP_NUM_STRUCT_TYPES: i32 = 3;

/// `eIDPropertyTypeFilter`.
pub const IDP_TYPE_FILTER_STRING: i32 = 1 << IDPropertyType::IdpString as i32;
pub const IDP_TYPE_FILTER_INT: i32 = 1 << IDPropertyType::IdpInt as i32;
pub const IDP_TYPE_FILTER_FLOAT: i32 = 1 << IDPropertyType::IdpFloat as i32;
pub const IDP_TYPE_FILTER_ARRAY: i32 = 1 << IDPropertyType::IdpArray as i32;
pub const IDP_TYPE_FILTER_GROUP: i32 = 1 << IDPropertyType::IdpGroup as i32;
pub const IDP_TYPE_FILTER_ID: i32 = 1 << IDPropertyType::IdpId as i32;
pub const IDP_TYPE_FILTER_DOUBLE: i32 = 1 << IDPropertyType::IdpDouble as i32;
pub const IDP_TYPE_FILTER_IDPARRAY: i32 = 1 << IDPropertyType::IdpIdpArray as i32;
pub const IDP_TYPE_FILTER_BOOLEAN: i32 = 1 << IDPropertyType::IdpBoolean as i32;

/// `eIDPropertySubType`.
pub const IDP_STRING_SUB_UTF8: i32 = 0;
pub const IDP_STRING_SUB_BYTE: i32 = 1;

/// `eIDPropertyFlag`.
pub const IDP_FLAG_OVERRIDABLE_LIBRARY: i16 = 1 << 0;
pub const IDP_FLAG_OVERRIDELIBRARY_LOCAL: i16 = 1 << 1;
pub const IDP_FLAG_STATIC_TYPE: i16 = 1 << 4;

/// `LibraryFlag`.
pub const LIBRARY_FLAG_IS_ARCHIVE: i32 = 1 << 0;
pub const LIBRARY_FLAG_IS_EXTERNAL: i32 = 1 << 1;

/// `MAKE_ID2(c, d)`.
pub const fn make_id2(c: u8, d: u8) -> u16 {
    (d as u16) << 8 | c as u16
}

/// `ID_Type` codes.
pub const ID_SCE: u16 = make_id2(b'S', b'C');
pub const ID_LI: u16 = make_id2(b'L', b'I');
pub const ID_OB: u16 = make_id2(b'O', b'B');
pub const ID_ME: u16 = make_id2(b'M', b'E');
pub const ID_CU_LEGACY: u16 = make_id2(b'C', b'U');
pub const ID_MB: u16 = make_id2(b'M', b'B');
pub const ID_MA: u16 = make_id2(b'M', b'A');
pub const ID_TE: u16 = make_id2(b'T', b'E');
pub const ID_IM: u16 = make_id2(b'I', b'M');
pub const ID_LT: u16 = make_id2(b'L', b'T');
pub const ID_LA: u16 = make_id2(b'L', b'A');
pub const ID_CA: u16 = make_id2(b'C', b'A');
pub const ID_KE: u16 = make_id2(b'K', b'E');
pub const ID_WO: u16 = make_id2(b'W', b'O');
pub const ID_SCR: u16 = make_id2(b'S', b'R');
pub const ID_VF: u16 = make_id2(b'V', b'F');
pub const ID_TXT: u16 = make_id2(b'T', b'X');
pub const ID_SPK: u16 = make_id2(b'S', b'K');
pub const ID_SO: u16 = make_id2(b'S', b'O');
pub const ID_GR: u16 = make_id2(b'G', b'R');
pub const ID_AR: u16 = make_id2(b'A', b'R');
pub const ID_AC: u16 = make_id2(b'A', b'C');
pub const ID_NT: u16 = make_id2(b'N', b'T');
pub const ID_BR: u16 = make_id2(b'B', b'R');
pub const ID_PA: u16 = make_id2(b'P', b'A');
pub const ID_GD_LEGACY: u16 = make_id2(b'G', b'D');
pub const ID_WM: u16 = make_id2(b'W', b'M');
pub const ID_MC: u16 = make_id2(b'M', b'C');
pub const ID_MSK: u16 = make_id2(b'M', b'S');
pub const ID_LS: u16 = make_id2(b'L', b'S');
pub const ID_PAL: u16 = make_id2(b'P', b'L');
pub const ID_PC: u16 = make_id2(b'P', b'C');
pub const ID_CF: u16 = make_id2(b'C', b'F');
pub const ID_WS: u16 = make_id2(b'W', b'S');
pub const ID_LP: u16 = make_id2(b'L', b'P');
pub const ID_CV: u16 = make_id2(b'C', b'V');
pub const ID_PT: u16 = make_id2(b'P', b'T');
pub const ID_VO: u16 = make_id2(b'V', b'O');
pub const ID_GP: u16 = make_id2(b'G', b'P');
pub const ID_LINK_PLACEHOLDER: u16 = make_id2(b'I', b'D');
pub const ID_SCRN: u16 = make_id2(b'S', b'N');
pub const ID_SEQ: u16 = make_id2(b'S', b'Q');
pub const ID_CO: u16 = make_id2(b'C', b'O');
pub const ID_PO: u16 = make_id2(b'A', b'C');
pub const ID_NLA: u16 = make_id2(b'N', b'L');
pub const ID_FLUIDSIM: u16 = make_id2(b'F', b'S');
