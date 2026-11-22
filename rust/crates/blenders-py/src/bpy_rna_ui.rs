/// Port of `python/bpy_rna_ui.cc` — BPY
/// `bpy_rna_ui`.
/// `BpyRnaUi` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyRnaUi {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
