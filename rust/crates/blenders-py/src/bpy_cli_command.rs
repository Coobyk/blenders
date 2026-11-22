/// Port of `python/bpy_cli_command.cc` — BPY
/// `bpy_cli_command`.
/// `BpyCliCommand` (verbatim — PyObject
/// wrapper).
#[derive(Default)]
pub struct BpyCliCommand {
    /// `handle` (PyObject pointer).
    pub handle: usize,
}
