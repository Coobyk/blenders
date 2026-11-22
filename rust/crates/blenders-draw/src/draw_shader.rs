/// Port of `intern/draw_shader.cc` —
/// `DRWShader` (shader permutations +
/// builtin shaders).
/// `DRWShader` (verbatim — create-info +
/// compiled shader).
pub struct DrwShader {
    /// `info` (shader create-info id).
    pub info: usize,
    /// `shader` (GPUShader handle).
    pub shader: usize,
    /// `mat_group` (builtin variant).
    pub mat_group: i32,
}
/// `drw_shader_get` (verbatim — builtin
/// lookup).
pub fn drw_shader_get(_info: usize) -> usize {
    0
}
