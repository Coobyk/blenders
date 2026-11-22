/// Port of `python/gpu_py_shader_create_info.cc` — BPY
/// `gpu_py_shader_create_info`.
/// `GpuPyShaderCreateInfo` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyShaderCreateInfo {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
