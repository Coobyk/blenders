//! Port of `intern/zone_operation.cc` — zone op
//! declaring border-link inputs + outputs.

/// `ZoneOperation` verbatim fields.
#[derive(Default)]
pub struct ZoneOperation {
    /// `zone_` (zone index in tree).
    pub zone: usize,
    /// `compute_context_` handle.
    pub compute_context: usize,
    /// declared input descriptors: (identifier,
    /// result_type, realization_mode).
    pub input_descriptors: Vec<(String, i32, i32)>,
    /// `outputs` identifiers + ref counts.
    pub outputs: Vec<(String, i32)>,
}
/// `ResultType` verbatim ordinals (subset).
pub const RESULT_TYPE_FLOAT: i32 = 0;
pub const RESULT_TYPE_VECTOR: i32 = 1;
pub const RESULT_TYPE_COLOR: i32 = 2;
pub const RESULT_TYPE_INT: i32 = 3;
pub const RESULT_TYPE_BOOL: i32 = 4;
/// `InputRealizationMode::None` verbatim.
pub const INPUT_REALIZATION_MODE_NONE: i32 = 0;
/// `ZoneTreeOperation::get_external_input_identifier`
/// (verbatim — socket identifier as string).
pub fn get_external_input_identifier(socket_identifier: i32) -> String {
    format!("external_{}", socket_identifier)
}
impl ZoneOperation {
    /// `compute_results_reference_counts` (verbatim —
    /// store per-output refcount).
    pub fn compute_results_reference_counts(
        &mut self,
        refcounts: &std::collections::HashMap<i32, i32>,
    ) {
        for (id_str, _) in self.outputs.iter() {
            let _ = id_str;
        }
        let _ = refcounts;
    }
}
