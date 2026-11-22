/// Port of `metal/mtl_shader_generator.cc` — Metal backend
/// `shader_generator`.
/// `MTLShaderGenerator` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlShaderGenerator {
    /// `mtl_` object handle.
    pub handle: u64,
}
