//! Port of `intern/shader_operation.cc` —
//! ShaderOperation (fn-graph multi-input op).

/// `ShaderOperation` verbatim fields.
#[derive(Default)]
pub struct ShaderOperation {
    /// `shader_node` handle.
    pub shader_node: usize,
    /// `material`/`tree` handles.
    pub material: usize,
    /// `buffer_fn` flag (uses buffer fn path).
    pub uses_buffer_fn: bool,
}
