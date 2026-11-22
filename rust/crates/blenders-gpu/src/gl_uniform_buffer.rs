/// Port of `opengl/gl_uniform_buffer.cc` — GL backend
/// `uniform_buffer`.
/// `GLUniformBuffer` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlUniformBuffer {
    /// `id` (GL object name).
    pub id: u32,
}
