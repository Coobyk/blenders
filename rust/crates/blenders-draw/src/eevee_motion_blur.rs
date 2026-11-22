/// Port of `engines/eevee/eevee_motion_blur.cc` — EEVEE
/// `motion_blur` module.
/// `EeveeMotionBlur` (verbatim —
/// pass/pipeline state).
#[derive(Default)]
pub struct EeveeMotionBlur {
    /// `handle` (engine state).
    pub handle: usize,
}
