//! Port of `intern/group_node_operation.cc` —
//! `bNodeType::GROUP` op wrapping inner node
//! group evaluation.
#[derive(Default)]
pub struct GroupNodeOperation {
    /// `group_tree` handle.
    pub group_tree: usize,
}
