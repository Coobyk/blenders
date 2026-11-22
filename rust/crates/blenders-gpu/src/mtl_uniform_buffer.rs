/// Port of `metal/mtl_uniform_buffer.cc` — Metal backend
/// `uniform_buffer`.
/// `MTLUniformBuffer` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlUniformBuffer {
    /// `mtl_` object handle.
    pub handle: u64,
}
