/// Port of `python/bpy_intern_string.cc` — BPY
/// `bpy_intern_string`.
/// `BpyInternString` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyInternString {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
