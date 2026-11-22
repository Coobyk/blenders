/// Port of `opengl/gl_debug.cc` — GL backend
/// `debug`.
/// `GLDebug` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlDebug {
    /// `id` (GL object name).
    pub id: u32,
}
