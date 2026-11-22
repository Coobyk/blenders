/// Port of `python/bpy_gizmo_wrap.cc` — BPY
/// `bpy_gizmo_wrap`.
/// `BpyGizmoWrap` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyGizmoWrap {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
