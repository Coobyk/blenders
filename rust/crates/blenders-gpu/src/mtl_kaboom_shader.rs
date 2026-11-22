/// Port of `metal/mtl_kaboom_shader.cc` — Metal backend
/// `kaboom_shader`.
/// `MTLKaboomShader` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlKaboomShader {
    /// `mtl_` object handle.
    pub handle: u64,
}
