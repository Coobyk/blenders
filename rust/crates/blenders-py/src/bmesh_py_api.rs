/// Port of `python/bmesh_py_api.cc` — BPY
/// `bmesh_py_api`.
/// `BmeshPyApi` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BmeshPyApi {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
