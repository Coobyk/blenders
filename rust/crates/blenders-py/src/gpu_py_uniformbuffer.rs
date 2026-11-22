/// Port of `python/gpu_py_uniformbuffer.cc` — BPY
/// `gpu_py_uniformbuffer`.
/// `GpuPyUniformbuffer` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyUniformbuffer {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
