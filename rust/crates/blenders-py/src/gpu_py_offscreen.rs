/// Port of `python/gpu_py_offscreen.cc` — BPY
/// `gpu_py_offscreen`.
/// `GpuPyOffscreen` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyOffscreen {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
