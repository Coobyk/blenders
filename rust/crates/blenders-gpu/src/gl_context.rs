/// Port of `opengl/gl_context.cc` — GL backend
/// `context`.
/// `GLContext` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlContext {
    /// `id` (GL object name).
    pub id: u32,
}
