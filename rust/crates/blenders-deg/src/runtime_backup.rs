//! Port of `intern/eval/deg_eval_runtime_backup*.cc` —
/// per-ID-type runtime state backups for copy-on-write eval.

/// `ModifierDataBackup`.
#[derive(Clone, Default)]
pub struct ModifierDataBackup {
    /// `type`.
    pub r#type: i32,
    /// `runtime` payload (arena index).
    pub runtime: usize,
}

/// `SoundBackup` (verbatim — `BKE_sound_runtime_state_*`
/// injected by the caller).
#[derive(Default)]
pub struct SoundBackup {
    /// `cache` pointer (arena index).
    pub cache: usize,
    /// `waveform`.
    pub waveform: usize,
    /// `playback_handle`.
    pub playback_handle: usize,
}

impl SoundBackup {
    /// `reset`.
    pub fn reset(&mut self) {
        self.cache = 0;
        self.waveform = 0;
        self.playback_handle = 0;
    }
}

/// `VolumeBackup` (verbatim — grids handle injected).
#[derive(Default)]
pub struct VolumeBackup {
    /// `filepath` (`FILE_MAX`).
    pub filepath: String,
    /// `grids` runtime handle.
    pub grids: usize,
}

/// `MovieClipBackup` (verbatim).
#[derive(Default)]
pub struct MovieClipBackup {
    /// `anim`.
    pub anim: usize,
    /// `cache`.
    pub cache: usize,
}

impl MovieClipBackup {
    /// `reset`.
    pub fn reset(&mut self) {
        self.anim = 0;
        self.cache = 0;
    }
}

/// `SceneBackup` fields (verbatim — `sequencer_backup`
/// handled by `sequencer` module).
#[derive(Default)]
pub struct SceneBackup {
    /// `audio_runtime` payload.
    pub audio_runtime: usize,
    /// `rigidbody_last_time`.
    pub rigidbody_last_time: f64,
}

impl SceneBackup {
    /// `reset`.
    pub fn reset(&mut self) {
        self.audio_runtime = 0;
        self.rigidbody_last_time = -1.0;
    }
}
