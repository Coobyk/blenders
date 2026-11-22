/// Port of `opengl/gl_vertex_buffer.cc` — GL backend
/// `vertex_buffer`.
/// `GLVertexBuffer` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlVertexBuffer {
    /// `id` (GL object name).
    pub id: u32,
}
