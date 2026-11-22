/// Port of `python/bpy_msgbus.cc` — BPY
/// `bpy_msgbus`.
/// `BpyMsgbus` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyMsgbus {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
