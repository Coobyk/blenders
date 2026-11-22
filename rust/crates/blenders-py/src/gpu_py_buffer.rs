/// Port of `python/gpu_py_buffer.cc` — BPY
/// `gpu_py_buffer`.
/// `GpuPyBuffer` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyBuffer {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
