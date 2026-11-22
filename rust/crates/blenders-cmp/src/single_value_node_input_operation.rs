//! Port of `intern/single_value_node_input_operation.cc`
//! — node input providing a single constant.

/// `SingleValueNodeInputOperation` verbatim.
#[derive(Default)]
pub struct SingleValueNodeInputOperation {
    /// `socket` handle.
    pub socket: usize,
    /// `value` bytes.
    pub value: Vec<u8>,
}
