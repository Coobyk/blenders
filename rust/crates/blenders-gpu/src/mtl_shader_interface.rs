/// Port of `metal/mtl_shader_interface.cc` — Metal backend
/// `shader_interface`.
/// `MTLShaderInterface` (verbatim — MTL
/// object + state).
#[derive(Default)]
pub struct MtlShaderInterface {
    /// `mtl_` object handle.
    pub handle: u64,
}
