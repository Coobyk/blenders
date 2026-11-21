//! Port of `intern/tracking_auto.cc` — the libmv marker coordinate
//! conversions, margin/trackable checks, and autotrack context
//! marker bookkeeping. libmv/track context is injected.

/// `LIBMV_MARKER_SOURCE_*`.
pub const LIBMV_MARKER_SOURCE_MANUAL: i32 = 0;
/// See above.
pub const LIBMV_MARKER_SOURCE_TRACKED: i32 = 1;
/// `LIBMV_MARKER_STATUS_UNKNOWN`.
pub const LIBMV_MARKER_STATUS_UNKNOWN: i32 = 0;
/// `LIBMV_MARKER_MODEL_TYPE_POINT`.
pub const LIBMV_MARKER_MODEL_TYPE_POINT: i32 = 0;
/// `LIBMV_MARKER_CHANNEL_*`.
pub const LIBMV_MARKER_CHANNEL_R: i32 = 1;
/// See above.
pub const LIBMV_MARKER_CHANNEL_G: i32 = 2;
/// See above.
pub const LIBMV_MARKER_CHANNEL_B: i32 = 4;
/// `MARKER_TRACKED`.
pub const MARKER_TRACKED: i32 = 1 << 0;
/// `MARKER_DISABLED`.
pub const MARKER_DISABLED: i32 = 1 << 1;
/// `TRACK_DISABLE_*`.
pub const TRACK_DISABLE_RED: i32 = 1 << 6;
/// See above.
pub const TRACK_DISABLE_GREEN: i32 = 1 << 7;
/// See above.
pub const TRACK_DISABLE_BLUE: i32 = 1 << 8;
/// `TRACK_LOCKED`.
pub const TRACK_LOCKED: i32 = 1 << 2;
/// `TRACK_HIDDEN`.
pub const TRACK_HIDDEN: i32 = 1 << 9;
/// `TRACK_MATCH_KEYFRAME`.
pub const TRACK_MATCH_KEYFRAME: i32 = 1;

/// `MovieTrackingMarker` fields used.
#[derive(Clone, Default)]
pub struct MovieTrackingMarker {
    /// `framenr`.
    pub framenr: i32,
    /// `pos` (normalized).
    pub pos: [f32; 2],
    /// `pattern_corners` (normalized, relative to pos).
    pub pattern_corners: [[f32; 2]; 4],
    /// `search_min`, `search_max` (normalized, relative).
    pub search_min: [f32; 2],
    /// See `search_min`.
    pub search_max: [f32; 2],
    /// `flag`.
    pub flag: i32,
}

/// `libmv_Marker` (struct mirror).
#[derive(Clone, Default)]
pub struct LibmvMarker {
    /// `clip`.
    pub clip: i32,
    /// `frame`.
    pub frame: i32,
    /// `track`.
    pub track: i32,
    /// `center` (libmv frame coords).
    pub center: [f32; 2],
    /// `patch` corners (libmv frame coords).
    pub patch: [[f32; 2]; 4],
    /// `search_region_min`, `search_region_max`.
    pub search_region_min: [f32; 2],
    /// See `search_region_min`.
    pub search_region_max: [f32; 2],
    /// `weight`.
    pub weight: f32,
    /// `source`.
    pub source: i32,
    /// `status`.
    pub status: i32,
    /// `model_type`.
    pub model_type: i32,
    /// `model_id`.
    pub model_id: i32,
    /// `reference_clip`, `reference_frame`.
    pub reference_clip: i32,
    /// See `reference_clip`.
    pub reference_frame: i32,
    /// `disabled_channels`.
    pub disabled_channels: i32,
}

/// `normalized_to_libmv_frame`.
pub fn normalized_to_libmv_frame(
    normalized: [f32; 2],
    frame_dimensions: [i32; 2],
) -> [f32; 2] {
    [
        normalized[0] * frame_dimensions[0] as f32 - 0.5,
        normalized[1] * frame_dimensions[1] as f32 - 0.5,
    ]
}

/// `normalized_relative_to_libmv_frame`.
pub fn normalized_relative_to_libmv_frame(
    normalized: [f32; 2],
    origin: [f32; 2],
    frame_dimensions: [i32; 2],
) -> [f32; 2] {
    [
        (normalized[0] + origin[0]) * frame_dimensions[0] as f32 - 0.5,
        (normalized[1] + origin[1]) * frame_dimensions[1] as f32 - 0.5,
    ]
}

/// `libmv_frame_to_normalized`.
pub fn libmv_frame_to_normalized(
    frame_coord: [f32; 2],
    frame_dimensions: [i32; 2],
) -> [f32; 2] {
    [
        (frame_coord[0] + 0.5) / frame_dimensions[0] as f32,
        (frame_coord[1] + 0.5) / frame_dimensions[1] as f32,
    ]
}

/// `libmv_frame_to_normalized_relative`.
pub fn libmv_frame_to_normalized_relative(
    frame_coord: [f32; 2],
    origin: [f32; 2],
    frame_dimensions: [i32; 2],
) -> [f32; 2] {
    [
        (frame_coord[0] - origin[0]) / frame_dimensions[0] as f32,
        (frame_coord[1] - origin[1]) / frame_dimensions[1] as f32,
    ]
}

/// `dna_marker_to_libmv_marker` — `track_flags`/`pattern_match` from
/// `MovieTrackingTrack`; `keyframe_framenr` injected for
/// `TRACK_MATCH_KEYFRAME`.
#[allow(clippy::too_many_arguments)]
pub fn dna_marker_to_libmv_marker(
    track_flag: i32,
    pattern_match: i32,
    marker: &MovieTrackingMarker,
    clip: i32,
    track_index: i32,
    frame_width: i32,
    frame_height: i32,
    backwards: bool,
    keyframe_framenr: i32,
) -> LibmvMarker {
    let frame_dimensions = [frame_width, frame_height];
    let mut libmv_marker = LibmvMarker {
        clip,
        frame: marker.framenr,
        track: track_index,
        center: normalized_to_libmv_frame(marker.pos, frame_dimensions),
        weight: 1.0,
        status: LIBMV_MARKER_STATUS_UNKNOWN,
        model_type: LIBMV_MARKER_MODEL_TYPE_POINT,
        model_id: 0,
        reference_clip: clip,
        source: if marker.flag & MARKER_TRACKED != 0 {
            LIBMV_MARKER_SOURCE_TRACKED
        } else {
            LIBMV_MARKER_SOURCE_MANUAL
        },
        reference_frame: if pattern_match == TRACK_MATCH_KEYFRAME {
            keyframe_framenr
        } else if backwards {
            marker.framenr - 1
        } else {
            marker.framenr
        },
        disabled_channels: (if track_flag & TRACK_DISABLE_RED != 0 {
            LIBMV_MARKER_CHANNEL_R
        } else {
            0
        }) | (if track_flag & TRACK_DISABLE_GREEN != 0 {
            LIBMV_MARKER_CHANNEL_G
        } else {
            0
        }) | (if track_flag & TRACK_DISABLE_BLUE != 0 {
            LIBMV_MARKER_CHANNEL_B
        } else {
            0
        }),
        ..Default::default()
    };
    for i in 0..4 {
        libmv_marker.patch[i] = normalized_relative_to_libmv_frame(
            marker.pattern_corners[i],
            marker.pos,
            frame_dimensions,
        );
    }
    libmv_marker.search_region_min = normalized_relative_to_libmv_frame(
        marker.search_min,
        marker.pos,
        frame_dimensions,
    );
    libmv_marker.search_region_max = normalized_relative_to_libmv_frame(
        marker.search_max,
        marker.pos,
        frame_dimensions,
    );
    libmv_marker
}

/// `libmv_marker_to_dna_marker`.
pub fn libmv_marker_to_dna_marker(
    libmv_marker: &LibmvMarker,
    frame_width: i32,
    frame_height: i32,
) -> MovieTrackingMarker {
    let frame_dimensions = [frame_width, frame_height];
    let mut marker = MovieTrackingMarker {
        framenr: libmv_marker.frame,
        pos: libmv_frame_to_normalized(libmv_marker.center, frame_dimensions),
        ..Default::default()
    };
    for i in 0..4 {
        marker.pattern_corners[i] = libmv_frame_to_normalized_relative(
            libmv_marker.patch[i],
            libmv_marker.center,
            frame_dimensions,
        );
    }
    marker.search_min = libmv_frame_to_normalized_relative(
        libmv_marker.search_region_min,
        libmv_marker.center,
        frame_dimensions,
    );
    marker.search_max = libmv_frame_to_normalized_relative(
        libmv_marker.search_region_max,
        libmv_marker.center,
        frame_dimensions,
    );
    marker.flag = if libmv_marker.source == LIBMV_MARKER_SOURCE_TRACKED {
        MARKER_TRACKED
    } else {
        0
    };
    marker
}

/// `tracking_check_marker_margin`.
pub fn tracking_check_marker_margin(
    libmv_marker: &LibmvMarker,
    margin: i32,
    frame_width: i32,
    frame_height: i32,
) -> bool {
    let mut patch_min = [f32::MAX; 2];
    let mut patch_max = [f32::MIN; 2];
    for i in 0..4 {
        for a in 0..2 {
            patch_min[a] = patch_min[a].min(libmv_marker.patch[i][a]);
            patch_max[a] = patch_max[a].max(libmv_marker.patch[i][a]);
        }
    }
    let margin = margin as f32;
    let margin_left = (libmv_marker.center[0] - patch_min[0]).max(margin);
    let margin_top = (patch_max[1] - libmv_marker.center[1]).max(margin);
    let margin_right = (patch_max[0] - libmv_marker.center[0]).max(margin);
    let margin_bottom = (libmv_marker.center[1] - patch_min[1]).max(margin);
    !(libmv_marker.center[0] < margin_left
        || libmv_marker.center[0] > frame_width as f32 - margin_right
        || libmv_marker.center[1] < margin_bottom
        || libmv_marker.center[1] > frame_height as f32 - margin_top)
}

/// `autotrack_is_marker_usable`.
pub fn autotrack_is_marker_usable(marker: &MovieTrackingMarker) -> bool {
    marker.flag & MARKER_DISABLED == 0
}

/// `autotrack_is_track_trackable` — `selected` = `TRACK_SELECTED`.
pub fn autotrack_is_track_trackable(
    selected: bool,
    track_flag: i32,
    marker: &MovieTrackingMarker,
) -> bool {
    if selected && (track_flag & (TRACK_LOCKED | TRACK_HIDDEN)) == 0 {
        return autotrack_is_marker_usable(marker);
    }
    false
}
