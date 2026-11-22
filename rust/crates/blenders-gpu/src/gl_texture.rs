/// Port of `opengl/gl_texture.cc` — GL backend
/// `texture`.
/// `GLTexture` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlTexture {
    /// `id` (GL object name).
    pub id: u32,
}
