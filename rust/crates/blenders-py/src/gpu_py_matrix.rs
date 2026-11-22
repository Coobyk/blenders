/// Port of `python/gpu_py_matrix.cc` — BPY
/// `gpu_py_matrix`.
/// `GpuPyMatrix` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyMatrix {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
