/// Port of `engines/eevee/eevee_depth_of_field.cc` — EEVEE
/// `depth_of_field` module.
/// `EeveeDepthOfField` (verbatim —
/// pass/pipeline state).
#[derive(Default)]
pub struct EeveeDepthOfField {
    /// `handle` (engine state).
    pub handle: usize,
}
