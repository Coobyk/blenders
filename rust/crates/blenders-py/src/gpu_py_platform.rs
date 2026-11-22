/// Port of `python/gpu_py_platform.cc` — BPY
/// `gpu_py_platform`.
/// `GpuPyPlatform` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyPlatform {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
