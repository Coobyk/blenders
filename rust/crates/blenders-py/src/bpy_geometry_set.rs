/// Port of `python/bpy_geometry_set.cc` — BPY
/// `bpy_geometry_set`.
/// `BpyGeometrySet` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyGeometrySet {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
