/// Port of `python/gpu_py_vertex_format.cc` — BPY
/// `gpu_py_vertex_format`.
/// `GpuPyVertexFormat` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyVertexFormat {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
