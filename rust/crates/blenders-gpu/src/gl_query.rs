/// Port of `opengl/gl_query.cc` — GL backend
/// `query`.
/// `GLQuery` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlQuery {
    /// `id` (GL object name).
    pub id: u32,
}
