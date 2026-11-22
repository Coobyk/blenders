/// Port of `python/bpy_threads.cc` — BPY
/// `bpy_threads`.
/// `BpyThreads` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyThreads {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
