/// Port of `python/bpy_interface_atexit.cc` — BPY
/// `bpy_interface_atexit`.
/// `BpyInterfaceAtexit` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyInterfaceAtexit {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
