/// Port of `opengl/gl_shader_log.cc` — GL backend
/// `shader_log`.
/// `GLShaderLog` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlShaderLog {
    /// `id` (GL object name).
    pub id: u32,
}
