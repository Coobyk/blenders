/// Port of `opengl/gl_shader_interface.cc` — GL backend
/// `shader_interface`.
/// `GLShaderInterface` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlShaderInterface {
    /// `id` (GL object name).
    pub id: u32,
}
