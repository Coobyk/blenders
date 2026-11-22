/// Port of `python/gpu_py_capabilities.cc` — BPY
/// `gpu_py_capabilities`.
/// `GpuPyCapabilities` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyCapabilities {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
