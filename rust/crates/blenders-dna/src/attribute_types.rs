//! Port of `DNA_attribute_types.h` — `Attribute*` storage structs.

use crate::id::NIL;

/// `AttributeArray`.
#[derive(Clone, Default)]
pub struct AttributeArray {
    /// `void *data` → opaque attribute buffer ref.
    pub data: usize,
    /// `ImplicitSharingInfoHandle *sharing_info`.
    pub sharing_info: usize,
    pub size: i64,
    pub is_single: i8,
}

/// `AttributeSingle`.
#[derive(Clone, Default)]
pub struct AttributeSingle {
    /// `void *data`.
    pub data: usize,
    /// `ImplicitSharingInfoHandle *sharing_info`.
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
    /// `void *data` — tagged union payload ref (`AttributeArray`/`AttributeSingle`).
    pub data: usize,
}

/// `AttributeStorage`.
#[derive(Clone)]
pub struct AttributeStorage {
    /// `Attribute *dna_attributes` → Vec.
    pub dna_attributes: Vec<Attribute>,
    pub dna_attributes_num: i32,
    /// `bke::AttributeStorageRuntime *runtime` — opaque.
    pub runtime: usize,
    /// Payload table backing `Attribute.data` handles (the
    /// `void *data` allocations in C — array bytes or one
    /// single element). Index = `Attribute.data`.
    pub data_blobs: Vec<Vec<u8>>,
}
impl AttributeStorage {
    /// `wrap()` — accessor parity note; runtime wrapper lands with bke attributes.
    pub fn wrap(&self) -> &AttributeStorage {
        self
    }
}
impl Default for AttributeStorage {
    fn default() -> Self {
        Self {
            data_blobs: Vec::new(),
            dna_attributes: Vec::new(),
            dna_attributes_num: 0,
            runtime: NIL,
        }
    }
}
