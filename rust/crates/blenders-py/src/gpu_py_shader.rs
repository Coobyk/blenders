/// Port of `python/gpu_py_shader.cc` — BPY
/// `gpu_py_shader`.
/// `GpuPyShader` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyShader {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
