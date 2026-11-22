/// Port of `python/gpu_py_vertex_buffer.cc` — BPY
/// `gpu_py_vertex_buffer`.
/// `GpuPyVertexBuffer` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyVertexBuffer {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
