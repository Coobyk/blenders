//! Port of `intern/multi_function_procedure_operation.cc`
//! — procedural multi-fn op building the MF
//! procedure then executing via evaluator.

/// `MultiFunctionProcedureOperation` verbatim fields.
#[derive(Default)]
pub struct MultiFunctionProcedureOperation {
    /// `procedure` (mf::Procedure handle).
    pub procedure: usize,
    /// `variables` per param.
    pub variables: Vec<usize>,
    /// `keep_results` flags.
    pub keep_results: Vec<bool>,
}
/// `get_code` (verbatim — builds procedure then
/// emits executor).
pub fn multi_function_procedure_operation_op_type() -> &'static str {
    "MultiFunctionProcedureOperation"
}
