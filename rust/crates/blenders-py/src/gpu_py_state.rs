/// Port of `python/gpu_py_state.cc` — BPY
/// `gpu_py_state`.
/// `GpuPyState` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyState {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
