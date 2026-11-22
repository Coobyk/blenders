/// Port of `python/bpy_interface.cc` — BPY
/// `bpy_interface`.
/// `BpyInterface` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyInterface {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
