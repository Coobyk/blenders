/// Port of `metal/mtl_shader.cc` — Metal backend
/// `shader`.
/// `MTLShader` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlShader {
    /// `mtl_` object handle.
    pub handle: u64,
}
