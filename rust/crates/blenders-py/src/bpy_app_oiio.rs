/// Port of `python/bpy_app_oiio.cc` — BPY
/// `bpy_app_oiio`.
/// `BpyAppOiio` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyAppOiio {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
