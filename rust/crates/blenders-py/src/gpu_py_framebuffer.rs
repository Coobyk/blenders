/// Port of `python/gpu_py_framebuffer.cc` — BPY
/// `gpu_py_framebuffer`.
/// `GpuPyFramebuffer` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyFramebuffer {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
