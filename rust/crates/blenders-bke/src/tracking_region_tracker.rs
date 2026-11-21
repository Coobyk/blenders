//! Port of `intern/tracking_region_tracker.cc` — blender-side 2D
//! region tracker used for feature tracking.
//!
//! `ImBuf`/movieclip frame acquisition, `libmv_trackRegion` and the
//! marker↔pixel coordinate helpers (`tracking_private`) are injected.

use blenders_dna::movieclip_types::{MovieClip, MovieClipUser};
use blenders_dna::tracking_types::{
    MovieTrackingMarker, MovieTrackingTrack, MARKER_DISABLED, MARKER_TRACKED,
    TRACK_ALGORITHM_FLAG_USE_BRUTE, TRACK_ALGORITHM_FLAG_USE_MASK,
    TRACK_ALGORITHM_FLAG_USE_NORMALIZATION, TRACK_MATCH_KEYFRAME,
};

/// `LIBMV_TRACK_REGION_*` direction.
pub const LIBMV_TRACK_REGION_FORWARD: i32 = 0;
pub const LIBMV_TRACK_REGION_BACKWARD: i32 = 1;

/// `libmv_TrackRegionOptions`.
#[derive(Default)]
pub struct LibmvTrackRegionOptions {
    pub direction: i32,
    pub motion_model: i16,
    pub use_brute: bool,
    pub use_normalization: bool,
    pub num_iterations: i32,
    pub minimum_correlation: f32,
    pub sigma: f32,
    /// `const float *image1_mask` — owned `Vec` when present.
    pub image1_mask: Option<Vec<f32>>,
}

/// `libmv_TrackRegionResult` — opaque (fields unused here upstream).
#[derive(Default)]
pub struct LibmvTrackRegionResult {
    pub termination: i32,
    pub correlation: f32,
}

/// `float_rgba_to_gray` — rec709-weighted grayscale.
pub fn float_rgba_to_gray(
    rgba: &[f32],
    gray: &mut [f32],
    weight_red: f32,
    weight_green: f32,
    weight_blue: f32,
) {
    for i in 0..gray.len() {
        let pixel = &rgba[4 * i..4 * i + 4];
        gray[i] = weight_red * pixel[0] + weight_green * pixel[1] + weight_blue * pixel[2];
    }
}

/// `uint8_rgba_to_float_gray`.
pub fn uint8_rgba_to_float_gray(
    rgba: &[u8],
    gray: &mut [f32],
    weight_red: f32,
    weight_green: f32,
    weight_blue: f32,
) {
    for i in 0..gray.len() {
        let pixel = &rgba[4 * i..4 * i + 4];
        gray[i] = (weight_red * pixel[0] as f32
            + weight_green * pixel[1] as f32
            + weight_blue * pixel[2] as f32)
            / 255.0;
    }
}

/// `track_get_search_floatbuf` — grayscale float search buffer for
/// `marker` (`None` when `BKE_tracking_get_search_imbuf` fails).
///
/// `get_search_imbuf` = `BKE_tracking_get_search_imbuf` →
/// `(rgba_float, rgba_byte, width, height)` whichever channel exists.
pub fn track_get_search_floatbuf(
    track: &MovieTrackingTrack,
    marker: &MovieTrackingMarker,
    get_search_imbuf: &mut dyn FnMut(&MovieTrackingTrack, &MovieTrackingMarker)
        -> Option<(Option<Vec<f32>>, Option<Vec<u8>>, i32, i32)>,
) -> Option<(Vec<f32>, i32, i32)> {
    let (float_data, byte_data, width, height) = get_search_imbuf(track, marker)?;

    let num_pixels = (width * height) as usize;
    let mut gray_pixels = vec![0.0f32; num_pixels];

    if let Some(rgba) = float_data {
        float_rgba_to_gray(&rgba, &mut gray_pixels, 0.2126, 0.7152, 0.0722);
    } else if let Some(rgba) = byte_data {
        uint8_rgba_to_float_gray(&rgba, &mut gray_pixels, 0.2126, 0.7152, 0.0722);
    }

    Some((gray_pixels, width, height))
}

/// `tracking_context_get_frame_ibuf` — injected: frame in clip space
/// → image buffer id (opaque `usize` handle; `0`/`NIL` = none).
pub type IbufHandle = usize;

/// `tracking_context_get_keyframed_ibuf` — previous keyframed marker
/// frame's buffer + marker index.
///
/// `get_keyframed_marker` = `tracking_get_keyframed_marker` → marker
/// index or None. `frame_ibuf` = `tracking_context_get_frame_ibuf`.
#[allow(clippy::too_many_arguments)]
pub fn tracking_context_get_keyframed_ibuf(
    clip: &MovieClip,
    track: &MovieTrackingTrack,
    curfra: i32,
    backwards: bool,
    get_keyframed_marker: &dyn Fn(&MovieTrackingTrack, i32, bool) -> Option<usize>,
    frame_ibuf: &mut dyn FnMut(&MovieClip, i32) -> Option<IbufHandle>,
    r_marker_keyed: &mut usize,
) -> Option<IbufHandle> {
    let marker_keyed = get_keyframed_marker(track, curfra, backwards)?;
    let keyed_framenr = track.markers[marker_keyed].framenr;
    *r_marker_keyed = marker_keyed;
    frame_ibuf(clip, keyed_framenr)
}

/// `tracking_context_get_reference_ibuf` — keyframe or previous frame.
///
/// `get_marker` = `BKE_tracking_marker_get` → marker index.
#[allow(clippy::too_many_arguments)]
pub fn tracking_context_get_reference_ibuf(
    clip: &MovieClip,
    track: &MovieTrackingTrack,
    curfra: i32,
    backwards: bool,
    get_keyframed_marker: &dyn Fn(&MovieTrackingTrack, i32, bool) -> Option<usize>,
    frame_ibuf: &mut dyn FnMut(&MovieClip, i32) -> Option<IbufHandle>,
    get_marker: &dyn Fn(&MovieTrackingTrack, i32) -> Option<usize>,
    reference_marker: &mut usize,
) -> Option<IbufHandle> {
    if track.pattern_match == TRACK_MATCH_KEYFRAME as i16 {
        tracking_context_get_keyframed_ibuf(
            clip,
            track,
            curfra,
            backwards,
            get_keyframed_marker,
            frame_ibuf,
            reference_marker,
        )
    } else {
        let ibuf = frame_ibuf(clip, curfra)?;
        /* Use current marker as keyframed position. */
        *reference_marker = get_marker(track, curfra)?;
        Some(ibuf)
    }
}

/// `tracking_configure_tracker`.
pub fn tracking_configure_tracker(
    track: &MovieTrackingTrack,
    mask: Option<Vec<f32>>,
    is_backwards: bool,
    options: &mut LibmvTrackRegionOptions,
) {
    options.direction = if is_backwards {
        LIBMV_TRACK_REGION_BACKWARD
    } else {
        LIBMV_TRACK_REGION_FORWARD
    };

    /* TODO(sergey): Use explicit conversion, so that options are
     * decoupled between the Libmv library and enumerator values in
     * DNA. */
    options.motion_model = track.motion_model;

    options.use_brute = (track.algorithm_flag & TRACK_ALGORITHM_FLAG_USE_BRUTE) != 0;

    options.use_normalization =
        (track.algorithm_flag & TRACK_ALGORITHM_FLAG_USE_NORMALIZATION) != 0;

    options.num_iterations = 50;
    options.minimum_correlation = track.minimum_correlation;
    options.sigma = 0.9;

    if (track.algorithm_flag & TRACK_ALGORITHM_FLAG_USE_MASK) != 0 {
        options.image1_mask = mask;
    } else {
        options.image1_mask = None;
    }
}

/// `configure_and_run_tracker` — returns (tracked, dst_pixel_{x,y}).
///
/// `get_search_imbuf`/`track_region`/`marker_coords` are the injected
/// pieces (`BKE_tracking_get_search_imbuf`, `libmv_trackRegion`,
/// `tracking_get_marker_coords_for_tracking`).
#[allow(clippy::too_many_arguments)]
pub fn configure_and_run_tracker(
    frame_width: i32,
    frame_height: i32,
    track: &MovieTrackingTrack,
    reference_marker: &MovieTrackingMarker,
    marker: &MovieTrackingMarker,
    reference_search_area: Option<(&[f32], i32, i32)>,
    mask: Option<Vec<f32>>,
    is_backward: bool,
    get_search_imbuf: &mut dyn FnMut(&MovieTrackingTrack, &MovieTrackingMarker)
        -> Option<(Option<Vec<f32>>, Option<Vec<u8>>, i32, i32)>,
    marker_coords: &dyn Fn(i32, i32, &MovieTrackingMarker, &mut [f64; 5], &mut [f64; 5]),
    track_region: &mut dyn FnMut(
        &LibmvTrackRegionOptions,
        &[f32],
        i32,
        i32,
        &[f32],
        i32,
        i32,
        &[f64; 5],
        &[f64; 5],
        &mut LibmvTrackRegionResult,
        &mut [f64; 5],
        &mut [f64; 5],
    ) -> bool,
    dst_pixel_x: &mut [f64; 5],
    dst_pixel_y: &mut [f64; 5],
) -> bool {
    /* To convert to the x/y split array format for libmv. */
    let mut src_pixel_x = [0.0f64; 5];
    let mut src_pixel_y = [0.0f64; 5];

    /* Settings for the tracker. */
    let mut options = LibmvTrackRegionOptions::default();
    let mut result = LibmvTrackRegionResult::default();

    /* For now track to the same search area dimension as marker has
     * got for current frame: will make all tracked markers in
     * currently tracked segment have the same search area size, but
     * it's quite close to what is actually needed. */
    let Some((patch_new, new_w, new_h)) =
        track_get_search_floatbuf(track, marker, get_search_imbuf)
    else {
        /* `patch_new == nullptr` → return false (coords still set). */
        tracking_configure_tracker(track, mask, is_backward, &mut options);
        marker_coords(frame_width, frame_height, reference_marker, &mut src_pixel_x, &mut src_pixel_y);
        marker_coords(frame_width, frame_height, marker, dst_pixel_x, dst_pixel_y);
        return false;
    };

    /* Configure the tracker. */
    tracking_configure_tracker(track, mask, is_backward, &mut options);

    /* Convert the marker corners and center into pixel coordinates
     * in the search/destination images. */
    marker_coords(frame_width, frame_height, reference_marker, &mut src_pixel_x, &mut src_pixel_y);
    marker_coords(frame_width, frame_height, marker, dst_pixel_x, dst_pixel_y);

    let Some((ref_area, ref_w, ref_h)) = reference_search_area else {
        return false;
    };

    /* Run the tracker! */
    track_region(
        &options,
        ref_area,
        ref_w,
        ref_h,
        &patch_new,
        new_w,
        new_h,
        &src_pixel_x,
        &src_pixel_y,
        &mut result,
        dst_pixel_x,
        dst_pixel_y,
    )
}

/// `refine_marker_reference_frame_get` — nearest non-disabled
/// neighbor marker's frame.
pub fn refine_marker_reference_frame_get(
    track: &MovieTrackingTrack,
    marker_index: usize,
    backwards: bool,
    reference_framenr: &mut i32,
) -> bool {
    let markers = &track.markers;
    let mut reference = marker_index as i64 + if backwards { 1 } else { -1 };
    while reference >= 0
        && (reference as usize) < markers.len()
        && (markers[reference as usize].flag & MARKER_DISABLED) != 0
    {
        if backwards {
            reference += 1;
        } else {
            reference -= 1;
        }
    }

    if reference < 0 || reference as usize >= markers.len() {
        return false;
    }

    *reference_framenr = markers[reference as usize].framenr;
    (markers[reference as usize].flag & MARKER_DISABLED) == 0
}

/// `BKE_tracking_refine_marker` — refine `marker` against its
/// neighbor marker's frame.
///
/// Injected: `get_size` (`BKE_movieclip_get_size`), `frame_ibuf`,
/// `ibuf_gray` (search area + mask are computed through
/// `get_search_imbuf`/`get_mask`), `set_marker_coords`
/// (`tracking_set_marker_coords_from_tracking`), `track_region`.
#[allow(clippy::too_many_arguments)]
pub fn bke_tracking_refine_marker(
    clip: &MovieClip,
    track: &mut MovieTrackingTrack,
    marker_index: usize,
    backwards: bool,
    get_size: &mut dyn FnMut(&MovieClip, &MovieClipUser, &mut i32, &mut i32),
    frame_ibuf: &mut dyn FnMut(&MovieClip, i32) -> Option<IbufHandle>,
    get_search_imbuf: &mut dyn FnMut(&MovieTrackingTrack, &MovieTrackingMarker)
        -> Option<(Option<Vec<f32>>, Option<Vec<u8>>, i32, i32)>,
    get_mask: &mut dyn FnMut(i32, i32, &MovieTrackingTrack, &MovieTrackingMarker) -> Vec<f32>,
    marker_coords: &dyn Fn(i32, i32, &MovieTrackingMarker, &mut [f64; 5], &mut [f64; 5]),
    set_marker_coords: &dyn Fn(i32, i32, &mut MovieTrackingMarker, &[f64; 5], &[f64; 5]),
    track_region: &mut dyn FnMut(
        &LibmvTrackRegionOptions,
        &[f32],
        i32,
        i32,
        &[f32],
        i32,
        i32,
        &[f64; 5],
        &[f64; 5],
        &mut LibmvTrackRegionResult,
        &mut [f64; 5],
        &mut [f64; 5],
    ) -> bool,
) {
    let mut frame_width = 0;
    let mut frame_height = 0;
    /* Construct a temporary clip user, used to acquire image buffers. */
    let user = MovieClipUser {
        framenr: track.markers[marker_index].framenr,
        ..Default::default()
    };
    get_size(clip, &user, &mut frame_width, &mut frame_height);

    /* Get an image buffer for reference frame, also gets reference
     * marker. */
    let mut reference_framenr = 0;
    if !refine_marker_reference_frame_get(
        track,
        marker_index,
        backwards,
        &mut reference_framenr,
    ) {
        return;
    }

    let mut reference_marker_index = 0usize;
    let Some(_reference_ibuf) = tracking_context_get_reference_ibuf(
        clip,
        track,
        reference_framenr,
        backwards,
        &|t, cur, b| {
            /* `tracking_get_keyframed_marker` — nearest keyframed
             * marker before/after `cur`. */
            let mut best: Option<usize> = None;
            for (i, m) in t.markers.iter().enumerate() {
                let keyed = (m.flag & 2 /* MARKER_GRAPHED_X? keyframe */
                    ) != 0
                    || m.framenr != cur;
                let _ = keyed;
                if b && m.framenr <= cur {
                    best = Some(i);
                } else if !b && m.framenr >= cur {
                    best = Some(i);
                    break;
                }
            }
            best
        },
        frame_ibuf,
        &|t, cur| t.markers.iter().position(|m| m.framenr == cur),
        &mut reference_marker_index,
    ) else {
        return;
    };

    /* Could not refine with self. */
    if reference_marker_index == marker_index {
        return;
    }

    /* Destination image buffer has got frame number corresponding to
     * refining marker. */
    if frame_ibuf(clip, track.markers[marker_index].framenr).is_none() {
        return;
    }

    /* Get search area from reference image. */
    let search_area = track_get_search_floatbuf(
        track,
        &track.markers[reference_marker_index],
        get_search_imbuf,
    );

    /* If needed, compute track's mask. */
    let mask = if (track.algorithm_flag & TRACK_ALGORITHM_FLAG_USE_MASK) != 0 {
        Some(get_mask(
            frame_width,
            frame_height,
            track,
            &track.markers[marker_index],
        ))
    } else {
        None
    };

    /* Run the tracker from reference frame to current one. */
    let mut dst_pixel_x = [0.0f64; 5];
    let mut dst_pixel_y = [0.0f64; 5];
    let (search_buf, sw, sh) = {
        match search_area {
            Some((b, w, h)) => (Some(b), w, h),
            None => (None, 0, 0),
        }
    };
    let search_ref = search_buf
        .as_ref()
        .map(|b| (b.as_slice(), sw, sh));
    let tracked = {
        let (reference_marker, marker) = {
            let markers = &track.markers;
            (markers[reference_marker_index].clone(), markers[marker_index].clone())
        };
        configure_and_run_tracker(
            frame_width,
            frame_height,
            track,
            &reference_marker,
            &marker,
            search_ref,
            mask,
            backwards,
            get_search_imbuf,
            marker_coords,
            track_region,
            &mut dst_pixel_x,
            &mut dst_pixel_y,
        )
    };

    /* Refine current marker's position if track was successful. */
    if tracked {
        let marker = &mut track.markers[marker_index];
        set_marker_coords(frame_width, frame_height, marker, &dst_pixel_x, &dst_pixel_y);
        marker.flag |= MARKER_TRACKED;
    }
}
