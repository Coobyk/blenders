/// Port of `opengl/gl_index_buffer.cc` — GL backend
/// `index_buffer`.
/// `GLIndexBuffer` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlIndexBuffer {
    /// `id` (GL object name).
    pub id: u32,
}
