/// Port of `shader_tool/resource.cc` — GPU
/// shader-tool `Resource`.
/// `Resource` (verbatim — tool
/// state/struct).
#[derive(Default)]
pub struct Resource {
    /// `handle` (tool-internal).
    pub handle: usize,
}
