/// Port of `python/bpy_rna_gizmo.cc` — BPY
/// `bpy_rna_gizmo`.
/// `BpyRnaGizmo` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyRnaGizmo {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
