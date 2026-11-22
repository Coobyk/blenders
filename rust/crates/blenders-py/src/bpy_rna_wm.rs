/// Port of `python/bpy_rna_wm.cc` — BPY
/// `bpy_rna_wm`.
/// `BpyRnaWm` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyRnaWm {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
