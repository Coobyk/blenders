/// Port of `python/bpy_audaspace.cc` — BPY
/// `bpy_audaspace`.
/// `BpyAudaspace` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyAudaspace {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
