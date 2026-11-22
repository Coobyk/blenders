/// Port of `metal/mtl_buffer.cc` — Metal backend
/// `buffer`.
/// `MTLBuffer` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlBuffer {
    /// `mtl_` object handle.
    pub handle: u64,
}
