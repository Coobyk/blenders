/// Port of `metal/mtl_storage_buffer.cc` — Metal backend
/// `storage_buffer`.
/// `MTLStorageBuffer` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlStorageBuffer {
    /// `mtl_` object handle.
    pub handle: u64,
}
