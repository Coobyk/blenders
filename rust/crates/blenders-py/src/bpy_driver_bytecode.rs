/// Port of `python/bpy_driver_bytecode.cc` — BPY
/// `bpy_driver_bytecode`.
/// `BpyDriverBytecode` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyDriverBytecode {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
