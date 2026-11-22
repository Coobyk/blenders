/// Port of `python/bpy_props.cc` — BPY
/// `bpy_props`.
/// `BpyProps` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyProps {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
