//! Port of `intern/pixel_operation.cc` —
//! PixelOperation: per-pixel op over a compile
//! unit, logging + reference counting.

/// `PixelOperation` verbatim fields.
#[derive(Default)]
pub struct PixelOperation {
    /// `compute_context_` handle.
    pub compute_context: usize,
    /// `is_single_value_`.
    pub is_single_value: bool,
    /// `output_sockets_to_output_identifiers_map_`.
    pub output_sockets_to_output_identifiers: std::collections::HashMap<usize, String>,
    /// `inputs_to_linked_outputs_map_`.
    pub inputs_to_linked_outputs: std::collections::HashMap<String, usize>,
    /// `implicit_inputs_to_input_identifiers_map_`.
    pub implicit_inputs_to_input_identifiers: std::collections::HashMap<i32, String>,
    /// `inputs_to_reference_counts_map_`.
    pub inputs_to_reference_counts: std::collections::HashMap<String, i32>,
    /// `logged_outputs_`/`preview_outputs_`.
    pub logged_outputs: std::collections::HashSet<usize>,
    pub preview_outputs: std::collections::HashSet<usize>,
}
impl PixelOperation {
    /// `get_output_identifier_from_output_socket`
    /// (verbatim).
    pub fn get_output_identifier_from_output_socket(&self, socket: usize) -> Option<&str> {
        self.output_sockets_to_output_identifiers
            .get(&socket)
            .map(String::as_str)
    }
    /// `get_internal_input_reference_count` (verbatim).
    pub fn get_internal_input_reference_count(&self, identifier: &str) -> i32 {
        self.inputs_to_reference_counts
            .get(identifier)
            .copied()
            .unwrap_or(0)
    }
    /// `compute_results_reference_counts` (verbatim —
    /// compile-unit count +1 for preview/logged).
    pub fn compute_results_reference_counts(
        &self,
        output: usize,
        base_count: i32,
    ) -> i32 {
        let mut reference_count = base_count;
        if self.preview_outputs.contains(&output) {
            reference_count += 1;
        }
        if self.logged_outputs.contains(&output) {
            reference_count += 1;
        }
        reference_count
    }
}
