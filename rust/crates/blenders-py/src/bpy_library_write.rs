/// Port of `python/bpy_library_write.cc` — BPY
/// `bpy_library_write`.
/// `BpyLibraryWrite` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyLibraryWrite {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
