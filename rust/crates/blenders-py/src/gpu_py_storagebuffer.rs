/// Port of `python/gpu_py_storagebuffer.cc` — BPY
/// `gpu_py_storagebuffer`.
/// `GpuPyStoragebuffer` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyStoragebuffer {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
