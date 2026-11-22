/// Port of `python/bmesh_py_types.cc` — BPY
/// `bmesh_py_types`.
/// `BmeshPyTypes` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BmeshPyTypes {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
