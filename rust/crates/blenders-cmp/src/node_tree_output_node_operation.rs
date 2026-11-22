//! Port of `intern/node_tree_output_node_operation.cc`
//! — writes op result into tree output.

/// `NodeTreeOutputNodeOperation` verbatim.
#[derive(Default)]
pub struct NodeTreeOutputNodeOperation {
    /// `output_index`.
    pub output_index: usize,
    /// `should_compute` (active output flag).
    pub should_compute: bool,
}
