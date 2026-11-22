//! Port of `intern/animation.cc` — strip animation
//! (keyframe mgmt over Strip data).

/// `seq_anim` channel enums verbatim.
pub const SEQ_ANIM_FRAME_START: i32 = 0;
/// `StripAnimation` — per-strip FCurves verbatim
//! record.
#[derive(Default)]
pub struct StripAnimation {
    /// `fcurves` per (prop, channel) pair.
    pub fcurves: Vec<(String, i32, usize)>,
}
