/// Port of `python/gpu_py_compute.cc` — BPY
/// `gpu_py_compute`.
/// `GpuPyCompute` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyCompute {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
