/// Port of `opengl/gl_vertex_array.cc` — GL backend
/// `vertex_array`.
/// `GLVertexArray` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlVertexArray {
    /// `id` (GL object name).
    pub id: u32,
}
