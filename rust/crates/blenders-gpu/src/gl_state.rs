/// Port of `opengl/gl_state.cc` — GL backend
/// `state`.
/// `GLState` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlState {
    /// `id` (GL object name).
    pub id: u32,
}
