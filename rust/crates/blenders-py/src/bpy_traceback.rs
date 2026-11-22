/// Port of `python/bpy_traceback.cc` — BPY
/// `bpy_traceback`.
/// `BpyTraceback` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyTraceback {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
