/// Port of `metal/mtl_cow_shader.cc` — Metal backend
/// `cow_shader`.
/// `MTLCowShader` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlCowShader {
    /// `mtl_` object handle.
    pub handle: u64,
}
