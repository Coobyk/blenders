/// Port of `python/bpy_rna_data.cc` — BPY
/// `bpy_rna_data`.
/// `BpyRnaData` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyRnaData {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
