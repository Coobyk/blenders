/// Port of `python/bmesh_py_types_select.cc` — BPY
/// `bmesh_py_types_select`.
/// `BmeshPyTypesSelect` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BmeshPyTypesSelect {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
