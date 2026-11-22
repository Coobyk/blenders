/// Port of `python/bpy_rna_array.cc` — BPY
/// `bpy_rna_array`.
/// `BpyRnaArray` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyRnaArray {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
