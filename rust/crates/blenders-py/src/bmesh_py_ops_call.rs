/// Port of `python/bmesh_py_ops_call.cc` — BPY
/// `bmesh_py_ops_call`.
/// `BmeshPyOpsCall` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BmeshPyOpsCall {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
