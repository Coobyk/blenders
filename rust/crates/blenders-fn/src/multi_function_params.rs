//! Port of `intern/multi_function_params.cc` —
//! ParamsBuilder unused-output buffer.

/// `ParamsBuilder` verbatim fields.
#[derive(Default)]
pub struct ParamsBuilder {
    /// `min_array_size_`.
    pub min_array_size: usize,
    /// `mask_` active indices.
    pub mask: Vec<usize>,
}
impl ParamsBuilder {
    /// `add_unused_output_for_unsupporting_function`
    /// (verbatim — alloc span of min_array_size,
    /// record in params + destruct callback).
    pub fn add_unused_output_for_unsupporting_function(&mut self, elem_size: usize) -> Vec<u8> {
        vec![0u8; elem_size * self.min_array_size]
    }
}
