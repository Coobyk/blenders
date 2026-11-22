//! Port of `intern/simple_operation.cc` —
//! base Operation that maps input results to
//! output (eval loop shell).

/// `SimpleOperation` verbatim fields.
#[derive(Default)]
pub struct SimpleOperation {
    /// `identifier`.
    pub identifier: i32,
    /// `is_active`/`is_renderer_active`.
    pub is_active: bool,
    /// `inputs`/`outputs` sockets (extern ids).
    pub inputs: Vec<usize>,
    pub outputs: Vec<usize>,
}
/// `SimpleOperation::get_identifier`/`execute`
/// dispatch (verbatim — overridden per op).
pub fn simple_operation_get_identifier(op: &SimpleOperation) -> i32 {
    op.identifier
}
