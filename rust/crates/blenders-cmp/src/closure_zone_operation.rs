//! Port of `intern/closure_zone_operation.cc` —
//! zone operation that records closure
//! item→socket bindings.

/// `ClosureZoneOperation` verbatim.
#[derive(Default)]
pub struct ClosureZoneOperation {
    /// `input_bindings`/`output_bindings`
    /// (closure item → extern socket id).
    pub input_bindings: Vec<(usize, usize)>,
    pub output_bindings: Vec<(usize, usize)>,
}
/// `add_input_binding`/`add_output_binding`.
pub fn closure_zone_add_input_binding(
    op: &mut ClosureZoneOperation,
    item: usize,
    socket: usize,
) {
    op.input_bindings.push((item, socket));
}
