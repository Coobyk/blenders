/// Port of `python/bpy_rna_text.cc` — BPY
/// `bpy_rna_text`.
/// `BpyRnaText` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyRnaText {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
