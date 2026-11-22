//! Port of `intern/driver.cc` — driver evaluation
//! entry (path resolution + eval).

/// `evaluate_driver_from_rna_pointer` (verbatim —
/// resolve path then evaluate; 0.0 on failure).
pub fn evaluate_driver_from_rna_pointer(
    path_resolved_ok: bool,
    evaluate_driver: &mut dyn FnMut() -> f32,
) -> f32 {
    if !path_resolved_ok {
        return 0.0;
    }
    (evaluate_driver)()
}
