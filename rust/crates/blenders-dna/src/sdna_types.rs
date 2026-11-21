//! Port of `DNA_sdna_types.h` — runtime struct DNA tables.

/// `SDNA_RAW_DATA_STRUCT_INDEX`.
pub const SDNA_RAW_DATA_STRUCT_INDEX: i32 = 0;

/// `SDNA_StructMember`.
#[derive(Clone, Copy, Default)]
pub struct SdnaStructMember {
    pub type_index: i16,
    pub member_index: i16,
}

/// `SDNA_Struct` — header + trailing `SDNA_StructMember` array.
#[derive(Clone, Default)]
pub struct SdnaStruct {
    pub type_index: i16,
    pub members_num: i16,
    /// `SDNA_StructMember members[]` — flexible array tail.
    pub members: Vec<SdnaStructMember>,
}

/// `SDNA` — parsed struct-DNA blob (type/member tables).
/// C++ uses `Array`/`Vector`/`Map`/`LinearAllocator`; Rust: Vec/HashMap + owned Strings.
#[derive(Clone, Default)]
pub struct Sdna {
    /// `const char *data` — raw SDNA buffer.
    pub data: Vec<u8>,
    pub data_size: i32,
    pub data_alloc: bool,
    pub pointer_size: i32,
    pub types_num: i32,
    /// `Array<StringRef> types`.
    pub types: Vec<String>,
    /// `short *types_size` → Vec.
    pub types_size: Vec<i16>,
    /// `Array<int> types_alignment`.
    pub types_alignment: Vec<i32>,
    /// `Array<SDNA_Struct *> structs`.
    pub structs: Vec<SdnaStruct>,
    pub members_num: i32,
    /// `Vector<StringRef> members`.
    pub members: Vec<String>,
    /// `Vector<short> members_array_num`.
    pub members_array_num: Vec<i16>,
    /// `Map<StringRef, int> types_to_structs_map`.
    pub types_to_structs_map: std::collections::HashMap<String, i32>,
    /// `alias` sub-struct.
    pub alias_types: Vec<String>,
    pub alias_members: Vec<String>,
    pub alias_types_to_structs_map: std::collections::HashMap<String, i32>,
}
