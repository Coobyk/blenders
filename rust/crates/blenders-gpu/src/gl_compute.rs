/// Port of `opengl/gl_compute.cc` — GL backend
/// `compute`.
/// `GLCompute` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlCompute {
    /// `id` (GL object name).
    pub id: u32,
}
