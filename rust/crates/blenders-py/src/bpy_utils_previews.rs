/// Port of `python/bpy_utils_previews.cc` — BPY
/// `bpy_utils_previews`.
/// `BpyUtilsPreviews` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyUtilsPreviews {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
