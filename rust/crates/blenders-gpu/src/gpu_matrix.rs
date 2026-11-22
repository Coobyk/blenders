/// Port of `intern/gpu_matrix.cc` — matrix
/// stack (modelview/projection push/pop/
/// ortho/persp).
/// `GPU_MATRIX_STACK_DEPTH` (verbatim = 16).
pub const GPU_MATRIX_STACK_DEPTH: usize = 16;
/// `GPUMatrixStack` (verbatim — top-of-
/// stack push/pop).
pub struct GpuMatrixStack {
    /// `stack` of 4×4.
    pub stack: Vec<[[f32; 4]; 4]>,
}
impl GpuMatrixStack {
    /// `gpu_matrix_push` (verbatim).
    pub fn push(&mut self, m: [[f32; 4]; 4]) {
        self.stack.push(m);
    }
    /// `gpu_matrix_pop` (verbatim).
    pub fn pop(&mut self) -> Option<[[f32; 4]; 4]> {
        self.stack.pop()
    }
    /// `gpu_matrix_top` (verbatim).
    pub fn top(&self) -> Option<[[f32; 4]; 4]> {
        self.stack.last().copied()
    }
}
/// `gpu_matrix_ortho` (verbatim — ortho
/// projection).
pub fn gpu_matrix_ortho(l: f32, r: f32, b: f32, t: f32, n: f32, f: f32) -> [[f32; 4]; 4] {
    [
        [2.0 / (r - l), 0.0, 0.0, -(r + l) / (r - l)],
        [0.0, 2.0 / (t - b), 0.0, -(t + b) / (t - b)],
        [0.0, 0.0, -2.0 / (f - n), -(f + n) / (f - n)],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
/// `gpu_matrix_persp` (verbatim — frustum
/// projection).
pub fn gpu_matrix_persp(l: f32, r: f32, b: f32, t: f32, n: f32, f: f32) -> [[f32; 4]; 4] {
    [
        [2.0 * n / (r - l), 0.0, (r + l) / (r - l), 0.0],
        [0.0, 2.0 * n / (t - b), (t + b) / (t - b), 0.0],
        [0.0, 0.0, -(f + n) / (f - n), -2.0 * f * n / (f - n)],
        [0.0, 0.0, -1.0, 0.0],
    ]
}
/// `gpu_matrix_normalize_plane` (verbatim —
/// plane eq normalize).
pub fn gpu_matrix_normalize_plane(p: &mut [f32; 4]) {
    let l = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
    if l != 0.0 {
        p[0] /= l;
        p[1] /= l;
        p[2] /= l;
        p[3] /= l;
    }
}
