//! Port of `intern/node_tree_input_node_operation.cc`
//! — forwards a tree input socket as op output.

/// `NodeTreeInputNodeOperation` verbatim.
#[derive(Default)]
pub struct NodeTreeInputNodeOperation {
    /// `input_index` in the tree.
    pub input_index: usize,
    /// `is_single_value`.
    pub is_single_value: bool,
}
