//! Port of `intern/node_group_operation.cc` —
//! nested node-group op (input/output socket map).

/// `NodeGroupOperation` verbatim fields.
#[derive(Default)]
pub struct NodeGroupOperation {
    /// `group_tree` handle.
    pub group_tree: usize,
    /// `input_map`/`output_map` socket↔identifier.
    pub input_map: Vec<(i32, String)>,
    pub output_map: Vec<(i32, String)>,
}
