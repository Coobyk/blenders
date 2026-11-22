/// Port of `python/bpy_path.cc` — BPY
/// `bpy_path`.
/// `BpyPath` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyPath {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
