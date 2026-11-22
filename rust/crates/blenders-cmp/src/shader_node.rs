//! Port of `intern/shader_node.cc` — shader node
//! operation bridge (fn-graph → operation).

/// `ShaderNodeOperation` verbatim.
#[derive(Default)]
pub struct ShaderNodeOperation {
    /// `node` handle.
    pub node: usize,
    /// `constant_foldable`.
    pub constant_foldable: bool,
}
