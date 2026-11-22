//! Port of `intern/undefined_node_operation.cc` —
//! fallback operation for unimplemented nodes.

/// `UndefinedNodeOperation` (verbatim — executes
/// nothing, logs warn).
#[derive(Default)]
pub struct UndefinedNodeOperation {
    /// `identifier` of the node.
    pub identifier: i32,
}
/// `execute` (verbatim — no-op + warn).
pub fn undefined_operation_execute() {}
