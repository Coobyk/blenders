/// Port of `opengl/gl_storage_buffer.cc` — GL backend
/// `storage_buffer`.
/// `GLStorageBuffer` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlStorageBuffer {
    /// `id` (GL object name).
    pub id: u32,
}
