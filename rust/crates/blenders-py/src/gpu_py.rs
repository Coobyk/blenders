/// Port of `python/gpu_py.cc` — BPY
/// `gpu_py`.
/// `GpuPy` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPy {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
