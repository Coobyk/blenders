/// Port of `opengl/gl_backend.cc` — GL backend
/// `backend`.
/// `GLBackend` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlBackend {
    /// `id` (GL object name).
    pub id: u32,
}
