/// Port of `opengl/gl_framebuffer.cc` — GL backend
/// `framebuffer`.
/// `GLFramebuffer` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlFramebuffer {
    /// `id` (GL object name).
    pub id: u32,
}
