//! Port of `intern/prefetch.cc` — prefetch job
//! frame loop + cache settings.

/// `before_playhead_frames` verbatim.
pub const BEFORE_PLAYHEAD_FRAMES: i32 = 5;
/// `SEQ_CACHE_PREFETCH_ENABLE` verbatim.
pub const SEQ_CACHE_PREFETCH_ENABLE: i32 = 1 << 5;
/// `PrefetchJob` verbatim subset.
#[derive(Default)]
pub struct PrefetchJob {
    /// `cfra`/`num_frames_prefetched`/`running`/
    /// `is_waiting`/`is_scrubbing`.
    pub cfra: i32,
    pub num_frames_prefetched: i32,
    pub running: bool,
    pub is_waiting: bool,
    pub is_scrubbing: bool,
}
/// `ScenePlaybackRange` verbatim.
#[derive(Clone, Copy)]
pub struct ScenePlaybackRange {
    /// `start_frame`/`end_frame`.
    pub start_frame: i32,
    pub end_frame: i32,
}
/// `seq_prefetch_cfra` (verbatim — cfra +
/// prefetched, wrap at end).
pub fn seq_prefetch_cfra(pfjob: &PrefetchJob, playback_range: ScenePlaybackRange) -> i32 {
    let mut new_frame = pfjob.cfra + pfjob.num_frames_prefetched;
    if new_frame >= playback_range.end_frame {
        new_frame = playback_range.start_frame + new_frame - playback_range.end_frame;
    }
    new_frame
}
/// `seq_prefetch_get_time_range` (verbatim —
/// None when no job/not running/disabled).
pub fn seq_prefetch_get_time_range(
    pfjob: Option<&PrefetchJob>,
    cache_flag: i32,
) -> Option<(i32, i32)> {
    let pfjob = pfjob?;
    if (cache_flag & SEQ_CACHE_PREFETCH_ENABLE) == 0 || !pfjob.running {
        return None;
    }
    Some((pfjob.cfra, pfjob.cfra + pfjob.num_frames_prefetched))
}
