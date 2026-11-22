/// Port of `opengl/gl_compilation_subprocess.cc` — GL backend
/// `compilation_subprocess`.
/// `GLCompilationSubprocess` (verbatim — GL
/// object handle + state).
#[derive(Default)]
pub struct GlCompilationSubprocess {
    /// `id` (GL object name).
    pub id: u32,
}
