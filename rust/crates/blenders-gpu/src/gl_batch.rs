/// Port of `opengl/gl_batch.cc` — GL backend
/// `batch`.
/// `GLBatch` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlBatch {
    /// `id` (GL object name).
    pub id: u32,
}
