/// Port of `python/gpu_py_batch.cc` — BPY
/// `gpu_py_batch`.
/// `GpuPyBatch` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyBatch {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
