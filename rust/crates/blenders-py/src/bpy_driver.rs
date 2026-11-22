/// Port of `python/bpy_driver.cc` — BPY
/// `bpy_driver`.
/// `BpyDriver` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyDriver {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
