/// Port of `python/gpu_py_select.cc` — BPY
/// `gpu_py_select`.
/// `GpuPySelect` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPySelect {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
