/// Port of `python/bmesh_py_utils.cc` — BPY
/// `bmesh_py_utils`.
/// `BmeshPyUtils` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BmeshPyUtils {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
