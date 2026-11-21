//! Port of `DNA_sound_types.h` — `bSound` ID.

use crate::id::{ID, NIL};

/// `eSound_Flag` (short).
pub type SoundFlag = i16;
pub const SOUND_FLAGS_3D: SoundFlag = 1 << 3;
pub const SOUND_FLAGS_CACHING: SoundFlag = 1 << 4;
pub const SOUND_FLAGS_MONO: SoundFlag = 1 << 5;

/// `bSound` — sound ID block.
#[derive(Clone)]
pub struct BSound {
    pub id: ID,
    /// `char filepath[1024]`.
    pub filepath: [u8; 1024],
    /// `PackedFile *packedfile`, `*newpackedfile`.
    pub packedfile: usize,
    pub newpackedfile: usize,
    pub offset_time: f64,
    pub volume: f32,
    pub attenuation: f32,
    pub pitch: f32,
    pub min_gain: f32,
    pub max_gain: f32,
    pub distance: f32,
    pub audio_channels: i32,
    pub samplerate: i32,
    pub flags: SoundFlag,
    pub stream_index: i16,
    /// `bke::SoundRuntime *runtime`.
    pub runtime: usize,
}
impl Default for BSound {
    fn default() -> Self {
        Self {
            id: ID::default(),
            filepath: [0; 1024],
            packedfile: NIL,
            newpackedfile: NIL,
            offset_time: 0.0,
            volume: 0.0,
            attenuation: 0.0,
            pitch: 0.0,
            min_gain: 0.0,
            max_gain: 0.0,
            distance: 0.0,
            audio_channels: 0,
            samplerate: 0,
            flags: 0,
            stream_index: 0,
            runtime: NIL,
        }
    }
}
