//! Port of `intern/implicit_input_operation.cc` —
//! input that re-emits a closure value as an
//! operation output.

/// `ImplicitInputOperation` verbatim.
#[derive(Default)]
pub struct ImplicitInputOperation {
    /// `closure` handle.
    pub closure: usize,
    /// `prop`/`field` selector.
    pub prop: String,
}
