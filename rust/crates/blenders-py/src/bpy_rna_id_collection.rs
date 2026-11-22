/// Port of `python/bpy_rna_id_collection.cc` — BPY
/// `bpy_rna_id_collection`.
/// `BpyRnaIdCollection` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyRnaIdCollection {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
