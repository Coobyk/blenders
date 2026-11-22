/// Port of `python/bpy_app_usd.cc` — BPY
/// `bpy_app_usd`.
/// `BpyAppUsd` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyAppUsd {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
