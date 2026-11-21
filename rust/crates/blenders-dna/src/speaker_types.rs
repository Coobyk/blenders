//! Port of `DNA_speaker_types.h` — `Speaker` ID.

use crate::id::{ID, NIL};

/// `eSpeaker_Flag` (short).
pub type SpeakerFlag = i16;
pub const SPK_DS_EXPAND: SpeakerFlag = 1 << 0;
pub const SPK_MUTED: SpeakerFlag = 1 << 1;

/// `Speaker` — speaker ID block.
#[derive(Clone)]
pub struct Speaker {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    /// `bSound *sound`.
    pub sound: usize,
    pub volume_max: f32,
    pub volume_min: f32,
    pub distance_max: f32,
    pub distance_reference: f32,
    pub attenuation: f32,
    pub cone_angle_outer: f32,
    pub cone_angle_inner: f32,
    pub cone_volume_outer: f32,
    pub volume: f32,
    pub pitch: f32,
    pub flag: SpeakerFlag,
}
impl Default for Speaker {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            sound: NIL,
            volume_max: 1.0,
            volume_min: 0.0,
            distance_max: f32::MAX,
            distance_reference: 1.0,
            attenuation: 1.0,
            cone_angle_outer: 360.0,
            cone_angle_inner: 360.0,
            cone_volume_outer: 1.0,
            volume: 1.0,
            pitch: 1.0,
            flag: 0,
        }
    }
}
