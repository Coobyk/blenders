/// Port of `python/bpy_app_timers.cc` — BPY
/// `bpy_app_timers`.
/// `BpyAppTimers` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyAppTimers {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
