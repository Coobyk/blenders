/// Port of `opengl/gl_texture_pool.cc` — GL backend
/// `texture_pool`.
/// `GLTexturePool` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlTexturePool {
    /// `id` (GL object name).
    pub id: u32,
}
