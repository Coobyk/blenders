/// Port of `python/gpu_py_element.cc` — BPY
/// `gpu_py_element`.
/// `GpuPyElement` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct GpuPyElement {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
