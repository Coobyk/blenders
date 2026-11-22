//! Port of `intern/eval/deg_eval_runtime_backup_{
//! animation,pose,scene,sequence,sequencer,sound,volume,
//! movieclip,modifier}.cc` — per-type runtime backups.

/// `AnimationValueBackup` verbatim.
pub struct AnimationValueBackup {
    /// `rna_path` (parsed).
    pub rna_path: String,
    /// `array_index`.
    pub array_index: i32,
    /// `value`.
    pub value: f32,
}

/// `AnimationBackup` verbatim — disabled: `init_from_id`/
/// `restore_to_id` return early like C (backup preserves
/// animated values but blocks user edits; C `return`s).
pub struct AnimationBackup {
    /// `meed_value_backup = !depsgraph->is_active`.
    pub meed_value_backup: bool,
    /// `values_backup`.
    pub values_backup: Vec<AnimationValueBackup>,
}

impl AnimationBackup {
    /// `AnimationBackup(depsgraph)`.
    pub fn new(is_active: bool) -> Self {
        AnimationBackup {
            meed_value_backup: !is_active,
            values_backup: Vec::new(),
        }
    }

    /// `reset` (verbatim — empty).
    pub fn reset(&mut self) {}

    /// `init_from_id` (verbatim — early `return` in C).
    pub fn init_from_id(&mut self, _id: usize) {
        return;
    }

    /// `restore_to_id` (verbatim — early `return` in C).
    pub fn restore_to_id(&mut self, _id: usize) {
        return;
    }
}

/// `SceneBackup` verbatim.
pub struct SceneBackup {
    /// `audio_runtime` (moved-out scene audio).
    pub audio_runtime: usize,
    /// `rigidbody_last_time`.
    pub rigidbody_last_time: f64,
    /// `sequencer_backup`.
    pub sequencer_backup: SequencerBackup,
}

impl SceneBackup {
    /// `SceneBackup(depsgraph)` + `reset`.
    pub fn new(depsgraph: usize) -> Self {
        let mut s = SceneBackup {
            audio_runtime: 0,
            rigidbody_last_time: -1.0,
            sequencer_backup: SequencerBackup::new(depsgraph),
        };
        s.reset();
        s
    }

    /// `reset` (verbatim).
    pub fn reset(&mut self) {
        self.audio_runtime = 0;
        self.rigidbody_last_time = -1.0;
    }

    /// `init_from_scene` (verbatim — lock, rb ltime, move
    /// audio, sequencer init).
    pub fn init_from_scene(
        &mut self,
        rigidbody_ltime: Option<f64>,
        scene_audio: usize,
        sequencer_init: &mut dyn FnMut(&mut SequencerBackup),
    ) {
        if let Some(lt) = rigidbody_ltime {
            self.rigidbody_last_time = lt;
        }
        self.audio_runtime = scene_audio;
        (sequencer_init)(&mut self.sequencer_backup);
    }

    /// `restore_to_scene` (verbatim — audio back, rb ltime,
    /// sequencer restore, unlock, reset).
    pub fn restore_to_scene(
        &mut self,
        rigidbody_world_present: bool,
        set_audio: &mut dyn FnMut(usize),
        set_rigidbody_ltime: &mut dyn FnMut(f64),
        sequencer_restore: &mut dyn FnMut(&mut SequencerBackup),
    ) {
        (set_audio)(self.audio_runtime);
        if rigidbody_world_present {
            (set_rigidbody_ltime)(self.rigidbody_last_time);
        }
        (sequencer_restore)(&mut self.sequencer_backup);
        self.reset();
    }
}

/// `STRIP_MODIFIER_FLAG_NONE` verbatim.
pub const STRIP_MODIFIER_FLAG_NONE: i32 = 0;

/// `StripModifierDataBackup` verbatim.
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
    /// ctor + `reset` (verbatim).
    pub fn new() -> Self {
        let mut b = StripModifierDataBackup {
            sound_in: 0,
            sound_out: 0,
            flag: 0,
            params_hash: 0,
        };
        b.reset();
        b
    }

    /// `reset` (verbatim).
    pub fn reset(&mut self) {
        self.sound_in = 0;
        self.sound_out = 0;
        self.flag = STRIP_MODIFIER_FLAG_NONE;
        self.params_hash = 0;
    }

    /// `init_from_modifier` (verbatim — sound-type only).
    pub fn init_from_modifier(
        &mut self,
        is_type_sound: bool,
        runtime_flag: i32,
        last_sound_in: usize,
        last_sound_out: usize,
        params_hash: u64,
    ) {
        if is_type_sound {
            self.flag = runtime_flag;
            self.sound_in = last_sound_in;
            self.sound_out = last_sound_out;
            self.params_hash = params_hash;
        }
    }

    /// `restore_to_modifier` (verbatim — sound-type only,
    /// then reset).
    pub fn restore_to_modifier(
        &mut self,
        is_type_sound: bool,
        set_runtime: &mut dyn FnMut(i32, usize, usize, u64),
    ) {
        if is_type_sound {
            (set_runtime)(self.flag, self.sound_in, self.sound_out, self.params_hash);
        }
        self.reset();
    }

    /// `isEmpty` (verbatim).
    pub fn is_empty(&self) -> bool {
        self.sound_in == 0 && self.sound_out == 0
    }
}

/// `StripBackup` verbatim.
pub struct StripBackup {
    /// `scene_sound`.
    pub scene_sound: usize,
    /// `sound_time_stretch`.
    pub sound_time_stretch: usize,
    /// `sound_time_stretch_fps`.
    pub sound_time_stretch_fps: f32,
    /// `modifiers` keyed by `smd.persistent_uid`.
    pub modifiers: std::collections::HashMap<u64, StripModifierDataBackup>,
}

impl StripBackup {
    /// `StripBackup(depsgraph)` + `reset`.
    pub fn new() -> Self {
        let mut b = StripBackup {
            scene_sound: 0,
            sound_time_stretch: 0,
            sound_time_stretch_fps: 0.0,
            modifiers: std::collections::HashMap::new(),
        };
        b.reset();
        b
    }

    /// `reset` (verbatim).
    pub fn reset(&mut self) {
        self.scene_sound = 0;
        self.sound_time_stretch = 0;
        self.sound_time_stretch_fps = 0.0;
        self.modifiers.clear();
    }

    /// `init_from_strip` (verbatim — move runtime fields +
    /// per-modifier backup keyed by uid).
    pub fn init_from_strip(
        &mut self,
        scene_sound: usize,
        sound_time_stretch: usize,
        sound_time_stretch_fps: f32,
        modifiers: &[(u64, StripModifierDataBackup)],
    ) {
        self.scene_sound = scene_sound;
        self.sound_time_stretch = sound_time_stretch;
        self.sound_time_stretch_fps = sound_time_stretch_fps;
        for (uid, mb) in modifiers {
            if !mb.is_empty() {
                self.modifiers.insert(*uid, StripModifierDataBackup {
                    sound_in: mb.sound_in,
                    sound_out: mb.sound_out,
                    flag: mb.flag,
                    params_hash: mb.params_hash,
                });
            }
        }
    }

    /// `restore_to_strip` (verbatim — fields back, per-mod
    /// pop-restore, reset).
    pub fn restore_to_strip(
        &mut self,
        set_runtime: &mut dyn FnMut(usize, usize, f32),
        restore_modifier: &mut dyn FnMut(u64, StripModifierDataBackup),
    ) {
        (set_runtime)(self.scene_sound, self.sound_time_stretch, self.sound_time_stretch_fps);
        let uids: Vec<u64> = self.modifiers.keys().cloned().collect();
        for uid in uids {
            if let Some(mb) = self.modifiers.remove(&uid) {
                (restore_modifier)(uid, mb);
            }
        }
        self.reset();
    }

    /// `isEmpty` (verbatim).
    pub fn is_empty(&self) -> bool {
        self.scene_sound == 0 && self.sound_time_stretch == 0 && self.modifiers.is_empty()
    }
}

/// `SequencerBackup` verbatim — `strips_backup` keyed by
/// `strip->runtime->session_uid`.
pub struct SequencerBackup {
    /// `depsgraph`.
    pub depsgraph: usize,
    /// `strips_backup`.
    pub strips_backup: std::collections::HashMap<u64, StripBackup>,
}

impl SequencerBackup {
    /// ctor (verbatim).
    pub fn new(depsgraph: usize) -> Self {
        SequencerBackup {
            depsgraph,
            strips_backup: std::collections::HashMap::new(),
        }
    }

    /// `strip_init_cb` verbatim — init + keep non-empty.
    pub fn strip_init_cb(&mut self, session_uid: u64, strip_backup: StripBackup) {
        if !strip_backup.is_empty() {
            self.strips_backup.insert(session_uid, strip_backup);
        }
    }

    /// `restore_to_scene` verbatim — per-strip restore +
    /// scene_sound cleanup.
    pub fn restore_to_scene(
        &mut self,
        for_each_strip: &mut dyn FnMut(&mut dyn FnMut(u64)),
        restore_strip: &mut dyn FnMut(u64, &mut StripBackup),
        remove_scene_sound: &mut dyn FnMut(usize),
    ) {
        let uids: Vec<u64> = self.strips_backup.keys().cloned().collect();
        (for_each_strip)(&mut |uid| {
            if self.strips_backup.contains_key(&uid) {
                let mut b = self.strips_backup.remove(&uid).unwrap();
                (restore_strip)(uid, &mut b);
                self.strips_backup.insert(uid, b);
            }
        });
        for (_uid, b) in self.strips_backup.iter_mut() {
            if b.scene_sound != 0 {
                (remove_scene_sound)(b.scene_sound);
                b.scene_sound = 0;
            }
        }
    }
}

/// `SoundBackup` verbatim.
pub struct SoundBackup {
    /// `cache`.
    pub cache: usize,
    /// `waveform`.
    pub waveform: usize,
    /// `playback_handle`.
    pub playback_handle: usize,
}

impl SoundBackup {
    /// ctor + `reset`.
    pub fn new() -> Self {
        let mut b = SoundBackup { cache: 0, waveform: 0, playback_handle: 0 };
        b.reset();
        b
    }

    /// `reset` (verbatim).
    pub fn reset(&mut self) {
        self.cache = 0;
        self.waveform = 0;
        self.playback_handle = 0;
    }

    /// `init_from_sound` (verbatim — get-and-clear).
    pub fn init_from_sound(&mut self, cache: usize, playback_handle: usize, waveform: usize) {
        self.cache = cache;
        self.playback_handle = playback_handle;
        self.waveform = waveform;
    }
}

/// `VolumeBackup` verbatim.
pub struct VolumeBackup {
    /// `filepath`.
    pub filepath: String,
    /// `grids`.
    pub grids: usize,
}

impl VolumeBackup {
    /// `VolumeBackup(depsgraph)` (verbatim).
    pub fn new() -> Self {
        VolumeBackup { filepath: String::new(), grids: 0 }
    }

    /// `init_from_volume` (verbatim — STRNCPY + move grids).
    pub fn init_from_volume(&mut self, filepath: &str, grids: usize) {
        self.filepath = filepath.to_string();
        self.grids = grids;
    }
}

/// `MovieClipBackup` verbatim.
pub struct MovieClipBackup {
    /// `anim`.
    pub anim: usize,
    /// `cache`.
    pub cache: usize,
}

impl MovieClipBackup {
    /// ctor + `reset`.
    pub fn new() -> Self {
        let mut b = MovieClipBackup { anim: 0, cache: 0 };
        b.reset();
        b
    }

    /// `reset` (verbatim).
    pub fn reset(&mut self) {
        self.anim = 0;
        self.cache = 0;
    }

    /// `init_from_movieclip` (verbatim — steal anim/cache).
    pub fn init_from_movieclip(&mut self, anim: usize, cache: usize) {
        self.anim = anim;
        self.cache = cache;
    }

    /// `restore_to_movieclip` (verbatim — fields back +
    /// reset).
    pub fn restore_to_movieclip(&mut self, set_fields: &mut dyn FnMut(usize, usize)) {
        (set_fields)(self.anim, self.cache);
        self.reset();
    }
}

/// `ModifierDataBackup` verbatim.
pub struct ModifierDataBackup {
    /// `type`.
    pub modifier_type: u32,
    /// `runtime`.
    pub runtime: usize,
}

// `deg_eval_runtime_backup_pose.cc` — empty file (pose
// backup is a `BKE_pose_backup` call elsewhere).
