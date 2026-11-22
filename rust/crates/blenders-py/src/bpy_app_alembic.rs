/// Port of `python/bpy_app_alembic.cc` — BPY
/// `bpy_app_alembic`.
/// `BpyAppAlembic` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyAppAlembic {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
