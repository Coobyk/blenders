/// Port of `python/bpy_app_ocio.cc` — BPY
/// `bpy_app_ocio`.
/// `BpyAppOcio` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyAppOcio {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
