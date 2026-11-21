//! Port of `DNA_packedFile_types.h` — `PackedFile`.

/// `PackedFile` — an embedded file blob.
#[derive(Clone, Default)]
pub struct PackedFile {
    pub size: i32,
    pub seek: i32,
    /// `const void *data` → byte buffer (len `size`).
    pub data: Vec<u8>,
    /// `ImplicitSharingInfoHandle *sharing_info`.
    pub sharing_info: usize,
}
