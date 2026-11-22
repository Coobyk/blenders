/// Port of `python/gpu_py_types.cc` — BPY
/// `gpu_py_types`.
/// `GpuPyTypes` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyTypes {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
