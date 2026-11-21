//! Port of `intern/tracking.cc` — marker/track array ops:
//! binary-search get/ensure/insert/delete, path clearing
//! (remained/upto/all), flag set/clear by area, pattern/
//! search clamping, accumulate/multiply averaging.
//!
//! `PORT_PENDING`: `tracking_*_free/copy` (ID list mgmt),
//! `BKE_tracking_get_projection_matrix`/`get_camera_object_
//! matrix` (needs Object + distortion model), `track_mask_
//! gpencil` rasterization, `BKE_tracking_tracks_average`
//! multi-track accumulation (builds on `accumulate_marker`/
//! `multiply_marker` — both ported), `tracks_join` (marker
//! splice + bundle interpolation), `reconstruction_*`
//! (libmv glue), `dopesheet` (coverage math ported).

use blenders_dna::tracking_types::{MovieTrackingMarker, MovieTrackingTrack};

/// `TRACK_AREA_*` (`eTrackArea` verbatim).
pub const TRACK_AREA_NONE: i32 = 0;
/// `TRACK_AREA_POINT`.
pub const TRACK_AREA_POINT: i32 = 1 << 0;
/// `TRACK_AREA_PAT`.
pub const TRACK_AREA_PAT: i32 = 1 << 1;
/// `TRACK_AREA_SEARCH`.
pub const TRACK_AREA_SEARCH: i32 = 1 << 2;
/// `TRACK_AREA_ALL`.
pub const TRACK_AREA_ALL: i32 = TRACK_AREA_POINT | TRACK_AREA_PAT | TRACK_AREA_SEARCH;

/// `MARKER_DISABLED`/`MARKER_TRACKED` (marker flag).
pub const MARKER_DISABLED: i32 = 1 << 0;
/// `MARKER_TRACKED`.
pub const MARKER_TRACKED: i32 = 1 << 1;
/// `MARKER_GRAPH_SEL_X`/`MARKER_GRAPH_SEL_Y`.
pub const MARKER_GRAPH_SEL_X: i32 = 1 << 2;
/// `MARKER_GRAPH_SEL_Y`.
pub const MARKER_GRAPH_SEL_Y: i32 = 1 << 3;

/// `eTrackClearAction` verbatim.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrackClearAction {
    /// `TRACK_CLEAR_REMAINED` — keep up to `ref_frame`.
    Remained,
    /// `TRACK_CLEAR_UPTO` — drop up to `ref_frame`.
    Upto,
    /// `TRACK_CLEAR_ALL` — single marker at `ref_frame`.
    All,
}

/// `BKE_tracking_track_flag_set`/`_clear` (verbatim).
pub fn tracking_track_flag_set(track: &mut MovieTrackingTrack, area: i32, flag: i32) {
    if area == TRACK_AREA_NONE {
        return;
    }
    if (area & TRACK_AREA_POINT) != 0 {
        track.flag |= flag;
    }
    if (area & TRACK_AREA_PAT) != 0 {
        track.pat_flag |= flag;
    }
    if (area & TRACK_AREA_SEARCH) != 0 {
        track.search_flag |= flag;
    }
}
/// `tracking_track_flag_clear`.
pub fn tracking_track_flag_clear(track: &mut MovieTrackingTrack, area: i32, flag: i32) {
    if area == TRACK_AREA_NONE {
        return;
    }
    if (area & TRACK_AREA_POINT) != 0 {
        track.flag &= !flag;
    }
    if (area & TRACK_AREA_PAT) != 0 {
        track.pat_flag &= !flag;
    }
    if (area & TRACK_AREA_SEARCH) != 0 {
        track.search_flag &= !flag;
    }
}

/// `BKE_tracking_track_has_marker_at_frame`.
pub fn tracking_track_has_marker_at_frame(track: &MovieTrackingTrack, framenr: i32) -> bool {
    tracking_marker_get_exact(track, framenr).is_some()
}
/// `BKE_tracking_track_has_enabled_marker_at_frame`.
pub fn tracking_track_has_enabled_marker_at_frame(track: &MovieTrackingTrack, framenr: i32) -> bool {
    tracking_marker_get_exact(track, framenr)
        .map(|m| (track.markers[m].flag & MARKER_DISABLED) == 0)
        .unwrap_or(false)
}

/// `BKE_tracking_marker_get` — binary-search closest
/// (verbatim; empty → `None`).
pub fn tracking_marker_get(track: &MovieTrackingTrack, framenr: i32) -> Option<usize> {
    let n = track.markersnr.max(0) as usize;
    if n == 0 {
        return None;
    }
    let mut left = 0usize;
    let mut right = n;
    while left < right {
        let median = (left + right) / 2;
        let m = &track.markers[median];
        if m.framenr == framenr {
            return Some(median);
        }
        if m.framenr < framenr {
            left = median + 1;
        } else {
            right = median.saturating_sub(1);
        }
    }
    Some(right.min(n - 1))
}

/// `BKE_tracking_marker_get_exact`.
pub fn tracking_marker_get_exact(track: &MovieTrackingTrack, framenr: i32) -> Option<usize> {
    let i = tracking_marker_get(track, framenr)?;
    if track.markers[i].framenr == framenr {
        Some(i)
    } else {
        None
    }
}

/// `BKE_tracking_marker_insert` (verbatim — sorted insert
/// or in-place replace).
pub fn tracking_marker_insert(track: &mut MovieTrackingTrack, marker: MovieTrackingMarker) -> usize {
    if track.markersnr > 0 {
        if let Some(i) = tracking_marker_get_exact(track, marker.framenr) {
            track.markers[i] = marker;
            return i;
        }
    }
    let mut a = track.markersnr.max(0) as usize;
    while a > 0 {
        a -= 1;
        if track.markers[a].framenr < marker.framenr {
            break;
        }
        if a == 0 {
            break;
        }
    }
    let insert_at = a + 1;
    track.markers.insert(insert_at.min(track.markers.len()), marker);
    track.markersnr = track.markers.len() as i32;
    insert_at
}

/// `tracking_marker_insert_disabled` (helper — insert copy
/// flagged disabled, verbatim).
pub fn tracking_marker_insert_disabled(
    track: &mut MovieTrackingTrack,
    ref_marker: &MovieTrackingMarker,
    before: bool,
    overwrite: bool,
) {
    let mut marker = ref_marker.clone();
    if before {
        marker.framenr -= 1;
    } else {
        marker.framenr += 1;
    }
    marker.flag |= MARKER_DISABLED;
    marker.flag &= !MARKER_TRACKED;
    if !overwrite && tracking_marker_get_exact(track, marker.framenr).is_some() {
        return;
    }
    tracking_marker_insert(track, marker);
}

/// `BKE_tracking_marker_delete` (verbatim).
pub fn tracking_marker_delete(track: &mut MovieTrackingTrack, framenr: i32) {
    let markers = &mut track.markers;
    let mut a = 0;
    while a < markers.len() {
        if markers[a].framenr == framenr {
            markers.remove(a);
            break;
        }
        a += 1;
    }
    track.markersnr = track.markers.len() as i32;
}

/// `path_clear_remained` (verbatim — truncate at first
/// marker after `ref_frame`, then insert disabled after).
pub fn path_clear_remained(track: &mut MovieTrackingTrack, ref_frame: i32) {
    for a in 1..track.markersnr as usize {
        if track.markers[a].framenr > ref_frame {
            track.markers.truncate(a);
            break;
        }
    }
    track.markersnr = track.markers.len() as i32;
    if track.markersnr > 0 {
        let last = track.markers[track.markersnr as usize - 1].clone();
        tracking_marker_insert_disabled(track, &last, false, true);
    }
}

/// `path_clear_up_to` (verbatim — drop markers before
/// `ref_frame`).
pub fn path_clear_up_to(track: &mut MovieTrackingTrack, ref_frame: i32) {
    for a in (0..track.markersnr as usize).rev() {
        if track.markers[a].framenr <= ref_frame {
            track.markers.drain(0..a);
            break;
        }
    }
    track.markersnr = track.markers.len() as i32;
    if track.markersnr > 0 {
        let first = track.markers[0].clone();
        tracking_marker_insert_disabled(track, &first, true, true);
    }
}

/// `path_clear_all` (verbatim — keep only a marker at
/// `ref_frame`, with disabled before/after).
pub fn path_clear_all(track: &mut MovieTrackingTrack, ref_frame: i32) {
    let Some(i) = tracking_marker_get(track, ref_frame) else {
        return;
    };
    let marker_new = track.markers[i].clone();
    track.markers.clear();
    track.markersnr = 0;
    tracking_marker_insert(track, marker_new.clone());
    tracking_marker_insert_disabled(track, &marker_new, true, true);
    tracking_marker_insert_disabled(track, &marker_new, false, true);
}

/// `BKE_tracking_track_path_clear` (verbatim).
pub fn tracking_track_path_clear(track: &mut MovieTrackingTrack, ref_frame: i32, action: TrackClearAction) {
    match action {
        TrackClearAction::Remained => path_clear_remained(track, ref_frame),
        TrackClearAction::Upto => path_clear_up_to(track, ref_frame),
        TrackClearAction::All => path_clear_all(track, ref_frame),
    }
}

/// `accumulate_marker` (verbatim).
pub fn accumulate_marker(dst: &mut MovieTrackingMarker, src: &MovieTrackingMarker) {
    debug_assert!(dst.framenr == src.framenr);
    if (src.flag & MARKER_DISABLED) != 0 {
        return;
    }
    dst.pos[0] += src.pos[0];
    dst.pos[1] += src.pos[1];
    for c in 0..4 {
        dst.pattern_corners[c][0] += src.pattern_corners[c][0];
        dst.pattern_corners[c][1] += src.pattern_corners[c][1];
    }
    dst.search_min[0] += src.search_min[0];
    dst.search_min[1] += src.search_min[1];
    dst.search_max[0] += src.search_max[0];
    dst.search_max[1] += src.search_max[1];
    dst.flag &= !MARKER_DISABLED;
    if (src.flag & MARKER_TRACKED) == 0 {
        dst.flag &= !MARKER_TRACKED;
    }
}
/// `multiply_marker` (verbatim).
pub fn multiply_marker(marker: &mut MovieTrackingMarker, multiplier: f32) {
    marker.pos[0] *= multiplier;
    marker.pos[1] *= multiplier;
    for c in 0..4 {
        marker.pattern_corners[c][0] *= multiplier;
        marker.pattern_corners[c][1] *= multiplier;
    }
    marker.search_min[0] *= multiplier;
    marker.search_min[1] *= multiplier;
    marker.search_max[0] *= multiplier;
    marker.search_max[1] *= multiplier;
}

/// `BKE_tracking_marker_pattern_minmax` — min/max of
/// `pattern_corners` (verbatim).
pub fn tracking_marker_pattern_minmax(marker: &MovieTrackingMarker) -> ([f32; 2], [f32; 2]) {
    let mut mn = [f32::MAX; 2];
    let mut mx = [f32::MIN; 2];
    for c in &marker.pattern_corners {
        for a in 0..2 {
            mn[a] = mn[a].min(c[a]);
            mx[a] = mx[a].max(c[a]);
        }
    }
    (mn, mx)
}

/// `BKE_tracking_marker_clamp_pattern_position` (verbatim).
pub fn tracking_marker_clamp_pattern_position(marker: &mut MovieTrackingMarker) {
    let (pat_min, pat_max) = tracking_marker_pattern_minmax(marker);
    for a in 0..2 {
        if pat_min[a] < marker.search_min[a] {
            for b in 0..4 {
                marker.pattern_corners[b][a] += marker.search_min[a] - pat_min[a];
            }
        }
        if pat_max[a] > marker.search_max[a] {
            for b in 0..4 {
                marker.pattern_corners[b][a] -= pat_max[a] - marker.search_max[a];
            }
        }
    }
}
/// `BKE_tracking_marker_clamp_search_size` (verbatim).
pub fn tracking_marker_clamp_search_size(marker: &mut MovieTrackingMarker) {
    let (pat_min, pat_max) = tracking_marker_pattern_minmax(marker);
    for a in 0..2 {
        marker.search_min[a] = marker.search_min[a].min(pat_min[a]);
        marker.search_max[a] = marker.search_max[a].max(pat_max[a]);
    }
}
/// `BKE_tracking_marker_clamp_search_position` (verbatim).
pub fn tracking_marker_clamp_search_position(marker: &mut MovieTrackingMarker) {
    let (pat_min, pat_max) = tracking_marker_pattern_minmax(marker);
    let dim = [
        marker.search_max[0] - marker.search_min[0],
        marker.search_max[1] - marker.search_min[1],
    ];
    for a in 0..2 {
        if marker.search_min[a] > pat_min[a] {
            marker.search_min[a] = pat_min[a];
            marker.search_max[a] = marker.search_min[a] + dim[a];
        }
        if marker.search_max[a] < pat_max[a] {
            marker.search_max[a] = pat_max[a];
            marker.search_min[a] = marker.search_max[a] - dim[a];
        }
    }
}

/// `BKE_tracking_track_first_last_frame_get`.
pub fn tracking_track_first_last_frame_get(track: &MovieTrackingTrack) -> Option<(i32, i32)> {
    if track.markersnr <= 0 {
        return None;
    }
    Some((
        track.markers[0].framenr,
        track.markers[track.markersnr as usize - 1].framenr,
    ))
}
