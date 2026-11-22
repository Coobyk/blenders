/// Port of `python/bmesh_py_ops.cc` — BPY
/// `bmesh_py_ops`.
/// `BmeshPyOps` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BmeshPyOps {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
