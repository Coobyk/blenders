/// Port of `engines/image/image_shader.cc` —
/// image engine shader binding.
/// `image_shader` (verbatim — texture + color
/// uniforms).
pub struct ImageShader {
    /// `image` tex handle.
    pub image: usize,
    /// `color` uniform.
    pub color: [f32; 4],
}
