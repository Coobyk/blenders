/// Port of `python/bpy_library_load.cc` — BPY
/// `bpy_library_load`.
/// `BpyLibraryLoad` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyLibraryLoad {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
