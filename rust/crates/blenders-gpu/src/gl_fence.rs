/// Port of `opengl/gl_fence.cc` — GL backend
/// `fence`.
/// `GLFence` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlFence {
    /// `id` (GL object name).
    pub id: u32,
}
