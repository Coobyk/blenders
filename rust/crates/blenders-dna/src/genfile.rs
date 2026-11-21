//! Port of `DNA_genfile.h` — SDNA read/encode/reconstruct API.

use crate::sdna_types::{Sdna, SDNA_RAW_DATA_STRUCT_INDEX};

/// `eSDNA_Type`.
pub type ESdnaType = i32;
pub const SDNA_TYPE_CHAR: ESdnaType = 0;
pub const SDNA_TYPE_UCHAR: ESdnaType = 1;
pub const SDNA_TYPE_SHORT: ESdnaType = 2;
pub const SDNA_TYPE_USHORT: ESdnaType = 3;
pub const SDNA_TYPE_INT: ESdnaType = 4;
pub const SDNA_TYPE_FLOAT: ESdnaType = 7;
pub const SDNA_TYPE_DOUBLE: ESdnaType = 8;
pub const SDNA_TYPE_INT64: ESdnaType = 9;
pub const SDNA_TYPE_UINT64: ESdnaType = 10;
pub const SDNA_TYPE_VOID: ESdnaType = 11;
pub const SDNA_TYPE_INT8: ESdnaType = 12;
pub const SDNA_TYPE_RAW_DATA: ESdnaType = 13;

/// `SdnaBuiltinType`.
#[derive(Clone)]
pub struct SdnaBuiltinType {
    /// `StringRefNull name`.
    pub name: &'static str,
    pub size: i16,
    pub type_index: i32,
}

/// `builtin_sdna_types` table (`SDNA_TYPE_RAW_DATA` entries; `long`/`ulong` get
/// indices 5 and 6 in C).
pub const BUILTIN_SDNA_TYPES: [SdnaBuiltinType; 13] = [
    SdnaBuiltinType { name: "char", size: 1, type_index: SDNA_TYPE_CHAR },
    SdnaBuiltinType { name: "uchar", size: 1, type_index: SDNA_TYPE_UCHAR },
    SdnaBuiltinType { name: "short", size: 2, type_index: SDNA_TYPE_SHORT },
    SdnaBuiltinType { name: "ushort", size: 2, type_index: SDNA_TYPE_USHORT },
    SdnaBuiltinType { name: "int", size: 4, type_index: SDNA_TYPE_INT },
    SdnaBuiltinType { name: "long", size: 4, type_index: 5 },
    SdnaBuiltinType { name: "ulong", size: 4, type_index: 6 },
    SdnaBuiltinType { name: "float", size: 4, type_index: SDNA_TYPE_FLOAT },
    SdnaBuiltinType { name: "double", size: 8, type_index: SDNA_TYPE_DOUBLE },
    SdnaBuiltinType { name: "int64_t", size: 8, type_index: SDNA_TYPE_INT64 },
    SdnaBuiltinType { name: "uint64_t", size: 8, type_index: SDNA_TYPE_UINT64 },
    SdnaBuiltinType { name: "void", size: 0, type_index: SDNA_TYPE_VOID },
    SdnaBuiltinType { name: "int8_t", size: 1, type_index: SDNA_TYPE_INT8 },
];

/// `eSDNA_StructCompare`.
pub type ESdnaStructCompare = i32;
pub const SDNA_CMP_REMOVED: ESdnaStructCompare = 0;
pub const SDNA_CMP_EQUAL: ESdnaStructCompare = 1;
pub const SDNA_CMP_NOT_EQUAL: ESdnaStructCompare = 2;
pub const SDNA_CMP_UNKNOWN: ESdnaStructCompare = 3;

/// `DNA_elem_type_size` — byte size of a builtin type (or of a struct index >=
/// `SDNA_TYPE_RAW_DATA` when resolved through `DNA_struct_size`).
pub fn dna_elem_type_size(elem_nr: ESdnaType) -> i32 {
    match elem_nr {
        SDNA_TYPE_CHAR | SDNA_TYPE_UCHAR | SDNA_TYPE_INT8 => 1,
        SDNA_TYPE_SHORT | SDNA_TYPE_USHORT => 2,
        SDNA_TYPE_INT | SDNA_TYPE_FLOAT => 4,
        SDNA_TYPE_DOUBLE | SDNA_TYPE_INT64 | SDNA_TYPE_UINT64 => 8,
        SDNA_TYPE_VOID => 0,
        _ => -1, // PORT_PENDING: struct-index lookup via DNA_struct_size
    }
}

/// `DNA_ReconstructInfo` — opaque reconstruction context (PORT_PENDING).
#[derive(Default)]
pub struct DnaReconstructInfo {
    _private: (),
}

// Remaining `DNA_sdna_*`/`DNA_struct_*`/`DNA_reconstruct_*` API lands with
// blenkernel's file read path; `SDNA` tables live in `sdna_types`.
#[allow(dead_code)]
fn _pending(_s: &Sdna) {
    let _ = SDNA_RAW_DATA_STRUCT_INDEX;
    let _ = DnaReconstructInfo::default();
}
