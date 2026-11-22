/// Port of `opengl/gl_immediate.cc` — GL backend
/// `immediate`.
/// `GLImmediate` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlImmediate {
    /// `id` (GL object name).
    pub id: u32,
}
