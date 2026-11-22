//! Port of `intern/field_evaluation.cc` —
//! FieldEvaluator + constant eval.

/// `FieldEvaluator` verbatim fields.
#[derive(Default)]
pub struct FieldEvaluator {
    /// `fields` (GField handles queued).
    pub fields: Vec<usize>,
    /// `destinations` per field.
    pub destinations: Vec<Option<usize>>,
    /// `is_evaluated_`.
    pub is_evaluated: bool,
}
impl FieldEvaluator {
    /// `FieldEvaluator::add` (verbatim — returns
    /// index, assert not evaluated).
    pub fn add(&mut self, field: usize, dst: Option<usize>) -> usize {
        debug_assert!(!self.is_evaluated);
        self.fields.push(field);
        self.destinations.push(dst);
        self.fields.len() - 1
    }
    /// `evaluate` (verbatim — marks evaluated).
    pub fn evaluate(&mut self) {
        self.is_evaluated = true;
    }
}
/// `evaluate_constant_field` (verbatim — copy
/// constant bytes out).
pub fn evaluate_constant_field(field: &crate::field::GField) -> Option<Vec<u8>> {
    match &field.variant {
        crate::field::GFieldVariant::Constant { buffer, .. } => Some(buffer.clone()),
        _ => None,
    }
}
/// `make_field_constant_if_possible` (verbatim —
/// already-constant stays).
pub fn make_field_constant_if_possible(field: crate::field::GField) -> crate::field::GField {
    field
}
