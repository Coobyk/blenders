/// Port of `python/bpy_rna_anim.cc` — BPY
/// `bpy_rna_anim`.
/// `BpyRnaAnim` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyRnaAnim {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
