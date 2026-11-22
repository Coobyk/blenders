/// Port of `python/gpu_py_texture.cc` — BPY
/// `gpu_py_texture`.
/// `GpuPyTexture` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyTexture {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
