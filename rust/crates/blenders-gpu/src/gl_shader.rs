/// Port of `opengl/gl_shader.cc` — GL backend
/// `shader`.
/// `GLShader` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlShader {
    /// `id` (GL object name).
    pub id: u32,
}
