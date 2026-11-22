/// Port of `python/gpu_py_api.cc` — BPY
/// `gpu_py_api`.
/// `GpuPyApi` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyApi {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
