/// Port of `python/bmesh_py_geometry.cc` — BPY
/// `bmesh_py_geometry`.
/// `BmeshPyGeometry` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BmeshPyGeometry {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
