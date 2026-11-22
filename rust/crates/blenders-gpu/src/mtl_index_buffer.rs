/// Port of `metal/mtl_index_buffer.cc` — Metal backend
/// `index_buffer`.
/// `MTLIndexBuffer` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlIndexBuffer {
    /// `mtl_` object handle.
    pub handle: u64,
}
