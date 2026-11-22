/// Port of `python/bpy_interface_run.cc` — BPY
/// `bpy_interface_run`.
/// `BpyInterfaceRun` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyInterfaceRun {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
