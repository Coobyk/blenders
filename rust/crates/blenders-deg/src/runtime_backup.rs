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

/// `AnimationValueBackup` (verbatim).
#[derive(Clone)]
pub struct AnimationValueBackup {
    /// `rna_path` (parsed path text).
    pub rna_path: String,
    /// `array_index`.
    pub array_index: i32,
    /// `value`.
    pub value: f32,
}

/// `AnimationBackup` (verbatim — note `init_from_id`/
/// `restore_to_id` are early-return disabled upstream).
#[derive(Default)]
pub struct AnimationBackup {
    /// `meed_value_backup` (`!depsgraph->is_active`).
    pub meed_value_backup: bool,
    /// `values_backup`.
    pub values_backup: Vec<AnimationValueBackup>,
}

impl AnimationBackup {
    /// ctor (verbatim — `meed_value_backup = !is_active`).
    pub fn new(depsgraph_is_active: bool) -> Self {
        Self {
            meed_value_backup: !depsgraph_is_active,
            values_backup: Vec::new(),
        }
    }
    /// `reset` (verbatim — no-op).
    pub fn reset(&mut self) {}
}

/// `STRIP_MODIFIER_FLAG_NONE`.
pub const STRIP_MODIFIER_FLAG_NONE: i32 = 0;

/// `StripModifierDataBackup` (verbatim).
#[derive(Clone, Default)]
pub struct StripModifierDataBackup {
    /// `sound_in`.
    pub sound_in: usize,
    /// `sound_out`.
    pub sound_out: usize,
    /// `flag`.
    pub flag: i32,
    /// `params_hash`.
    pub params_hash: u64,
}

impl StripModifierDataBackup {
    /// `reset`.
    pub fn reset(&mut self) {
        self.sound_in = 0;
        self.sound_out = 0;
        self.flag = STRIP_MODIFIER_FLAG_NONE;
        self.params_hash = 0;
    }
    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.sound_in == 0 && self.sound_out == 0
    }
}

/// `StripBackup` (verbatim — `modifiers` keyed by
/// `persistent_uid`).
#[derive(Default)]
pub struct StripBackup {
    /// `scene_sound`.
    pub scene_sound: usize,
    /// `sound_time_stretch`.
    pub sound_time_stretch: usize,
    /// `sound_time_stretch_fps`.
    pub sound_time_stretch_fps: f32,
    /// `modifiers` (`persistent_uid` → backup).
    pub modifiers: std::collections::HashMap<i32, StripModifierDataBackup>,
}

impl StripBackup {
    /// `reset`.
    pub fn reset(&mut self) {
        self.scene_sound = 0;
        self.sound_time_stretch = 0;
        self.sound_time_stretch_fps = 0.0;
        self.modifiers.clear();
    }
    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.scene_sound == 0 && self.sound_time_stretch == 0 && self.modifiers.is_empty()
    }
}

/// `SequencerBackup` (verbatim — `strips_backup` keyed by
/// `SessionUID`).
#[derive(Default)]
pub struct SequencerBackup {
    /// `strips_backup` (session_uid → strip backup).
    pub strips_backup: std::collections::HashMap<u64, StripBackup>,
}
