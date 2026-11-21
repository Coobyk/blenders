//! Port of `intern/sound.cc` — `bSound`/`SoundRuntime`
//! management, scene audio-animation classification, sorted
//! frame-range ops, strip audio predicates.
//!
//! `PORT_PENDING`: `AUD_*` device/sequence calls (audaspace
//! engine — injected), `sound_evaluate_scene_audio_
//! animation` (depsgraph walking + RNA property animation),
//! `sound_scene_animated_paths` (PointerRNA paths),
//! `blend_write/read`, `foreach_cache/path` (ID callbacks).

/// `Bounds<int>` — `[min,max]` inclusive frame range.
pub type FrameBounds = (i32, i32);

/// `sound_frame_ranges_contain` — any range fully covering
/// `frames` (verbatim).
pub fn sound_frame_ranges_contain(ranges: &[FrameBounds], frames: FrameBounds) -> bool {
    ranges
        .iter()
        .any(|&(lo, hi)| lo <= frames.0 && hi >= frames.1)
}

/// `sound_frame_ranges_add` — insert + merge overlapping or
/// adjacent ranges (verbatim).
pub fn sound_frame_ranges_add(ranges: &mut Vec<FrameBounds>, frames: FrameBounds) {
    ranges.push(frames);
    ranges.sort_by_key(|&(lo, _)| lo);
    let mut last = 0usize;
    for i in 1..ranges.len() {
        if i64::from(ranges[i].0) <= i64::from(ranges[last].1) + 1 {
            ranges[last].1 = ranges[last].1.max(ranges[i].1);
        } else {
            last += 1;
            ranges[last] = ranges[i];
        }
    }
    ranges.truncate(last + 1);
}

/// `sound_frame_ranges_missing` — parts of `frames` not
/// covered (verbatim).
pub fn sound_frame_ranges_missing(
    ranges: &[FrameBounds],
    frames: FrameBounds,
) -> Vec<FrameBounds> {
    let mut missing = Vec::new();
    let mut start = i64::from(frames.0);
    for &(lo, hi) in ranges {
        if i64::from(hi) < start {
            continue;
        }
        if lo > frames.1 {
            break;
        }
        if i64::from(lo) > start {
            missing.push((start as i32, lo - 1));
        }
        start = i64::from(hi) + 1;
    }
    if start <= i64::from(frames.1) {
        missing.push((start as i32, frames.1));
    }
    missing
}

/// `STRIP_TYPE_*` (DNA_sequence_types verbatim).
pub const STRIP_TYPE_SOUND: i8 = 3;
/// `STRIP_TYPE_SCENE`.
pub const STRIP_TYPE_SCENE: i8 = 4;
/// `SEQ_SCENE_STRIPS` (strip flag).
pub const SEQ_SCENE_STRIPS: i32 = 1 << 1;

/// `sound_scene_strip_uses_sequencer` (verbatim).
pub fn sound_scene_strip_uses_sequencer(
    strip_type: i8,
    strip_flag: i32,
    has_scene: bool,
    scene_has_ed: bool,
) -> bool {
    strip_type == STRIP_TYPE_SCENE && has_scene && (strip_flag & SEQ_SCENE_STRIPS) != 0 && scene_has_ed
}

/// `sound_strip_has_audio` (verbatim).
pub fn sound_strip_has_audio(strip_type: i8, has_sound: bool, has_scene: bool) -> bool {
    if strip_type == STRIP_TYPE_SOUND {
        return has_sound;
    }
    if strip_type == STRIP_TYPE_SCENE {
        return has_scene;
    }
    false
}

/// `AudioAnimationCacheType` verbatim.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AudioAnimationCacheType {
    /// `None` — no caching.
    #[default]
    None = 0,
    /// `Automatic` — grow to cover requested frames.
    Automatic = 1,
}

/// `SceneAudioRuntime` (verbatim state fields).
#[derive(Default)]
pub struct SceneAudioRuntime {
    /// `cached_animation_frames` — sorted covered ranges.
    pub cached_animation_frames: Vec<FrameBounds>,
    /// `animation_cache_dirty`.
    pub animation_cache_dirty: bool,
    /// `animation_cache_type`.
    pub animation_cache_type: AudioAnimationCacheType,
    /// `animation_cache_frame_change_handlers_hash`.
    pub animation_cache_frame_change_handlers_hash: Option<u64>,
    /// `recording`/`animation_use` readers (weak handles —
    /// `Option<usize>`; `None` = expired).
    pub recording: Option<usize>,
    /// `animation_use`.
    pub animation_use: Option<usize>,
}

/// `sound_scene_audio_animation_cache_has_data`.
pub fn sound_scene_audio_animation_cache_has_data(audio: &SceneAudioRuntime) -> bool {
    !audio.cached_animation_frames.is_empty()
}
/// `sound_scene_audio_animation_is_read` (verbatim).
pub fn sound_scene_audio_animation_is_read(audio: &SceneAudioRuntime) -> bool {
    audio.recording.is_some() || audio.animation_use.is_some()
}
/// `sound_clear_scene_audio_animation_cache_state` (verbatim).
pub fn sound_clear_scene_audio_animation_cache_state(audio: &mut SceneAudioRuntime) {
    audio.animation_cache_dirty = true;
    audio.cached_animation_frames.clear();
    audio.animation_cache_type = AudioAnimationCacheType::None;
    audio.animation_cache_frame_change_handlers_hash = None;
}

/// `sound_scene_audio_animation_cache_is_compatible`.
pub fn sound_scene_audio_animation_cache_is_compatible(
    audio: &SceneAudioRuntime,
    frame_change_handlers_hash: Option<u64>,
    frames: &[FrameBounds],
) -> bool {
    if audio.animation_cache_dirty {
        return false;
    }
    match audio.animation_cache_type {
        AudioAnimationCacheType::Automatic => {
            audio.animation_cache_frame_change_handlers_hash == frame_change_handlers_hash
                && frames
                    .iter()
                    .all(|&f| sound_frame_ranges_contain(&audio.cached_animation_frames, f))
        }
        AudioAnimationCacheType::None => frames.is_empty(),
    }
}

/// `sound_scene_audio_animation_cache_can_extend` (verbatim).
pub fn sound_scene_audio_animation_cache_can_extend(
    audio: &SceneAudioRuntime,
    frame_change_handlers_hash: Option<u64>,
) -> bool {
    !audio.animation_cache_dirty
        && audio.animation_cache_type == AudioAnimationCacheType::Automatic
        && audio.animation_cache_frame_change_handlers_hash == frame_change_handlers_hash
}

/// `SoundAnimationBakeRequest` (verbatim).
pub struct SoundAnimationBakeRequest {
    /// `scene` handle.
    pub scene: usize,
    /// `start_frame`/`end_frame`.
    pub start_frame: i32,
    /// `end_frame`.
    pub end_frame: i32,
    /// `append` (extend vs reset).
    pub append: bool,
    /// `reset`.
    pub reset: bool,
}

/// `sound_scene_audio_animation_requests` — build bake
/// requests covering cache misses (verbatim logic; the
/// depsgraph-evaluated check is a precondition flag).
pub fn sound_scene_audio_animation_requests(
    is_evaluated: bool,
    audio: &SceneAudioRuntime,
    frames: &[FrameBounds],
    frame_change_handlers_hash: Option<u64>,
    scene: usize,
) -> Vec<SoundAnimationBakeRequest> {
    let mut requests = Vec::new();
    if !is_evaluated {
        return requests;
    }
    if sound_scene_audio_animation_cache_is_compatible(
        audio,
        frame_change_handlers_hash,
        frames,
    ) {
        return requests;
    }
    if !sound_scene_audio_animation_cache_can_extend(audio, frame_change_handlers_hash) {
        let reset = !sound_scene_audio_animation_is_read(audio);
        for (i, &(lo, hi)) in frames.iter().enumerate() {
            requests.push(SoundAnimationBakeRequest {
                scene,
                start_frame: lo,
                end_frame: hi,
                append: i > 0,
                reset: reset && i == 0,
            });
        }
        return requests;
    }
    for &range in frames {
        for (lo, hi) in sound_frame_ranges_missing(&audio.cached_animation_frames, range) {
            requests.push(SoundAnimationBakeRequest {
                scene,
                start_frame: lo,
                end_frame: hi,
                append: true,
                reset: false,
            });
        }
    }
    requests
}

/// `SoundSceneAnimatedPaths` — `(path, sub_path)` property
/// pointer rows (verbatim).
#[derive(Default)]
pub struct SoundSceneAnimatedPaths {
    /// RNA path strings per animated property.
    pub paths: Vec<String>,
}

/// `SoundAudioPropertyAnimation` — `path` + `property`
/// classification row (verbatim).
pub struct SoundAudioPropertyAnimation {
    /// `path`.
    pub path: String,
    /// `property`.
    pub property: String,
    /// `scene_strip` — applies to a scene strip.
    pub scene_strip: Option<usize>,
}
