/// Port of `python/bpy_rna_callback.cc` — BPY
/// `bpy_rna_callback`.
/// `BpyRnaCallback` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyRnaCallback {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
