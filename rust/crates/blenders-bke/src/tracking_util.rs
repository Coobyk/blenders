//! Port of `intern/tracking_util.cc` — `TracksMap`, marker
//! unified/pixel coordinate transforms, distortion-model ⇄
//! intrinsics-options conversion, and `tracking_get_keyframed_marker`.
//! The `TrackingImageAccessor` (ImBuf/libmv) is dropped here — it is
//! a thin FFI shell; callers inject images.

use std::collections::HashMap;

use blenders_dna::tracking_types::{
    MovieTracking, MovieTrackingCamera, MovieTrackingMarker, MovieTrackingTrack, MARKER_DISABLED,
    MARKER_TRACKED, TRACKING_DISTORTION_MODEL_BROWN, TRACKING_DISTORTION_MODEL_DIVISION,
    TRACKING_DISTORTION_MODEL_NUKE, TRACKING_DISTORTION_MODEL_POLYNOMIAL,
};

/// `TracksMap` — temp track array + old↔new track mapping.
#[derive(Default)]
pub struct TracksMap {
    /// `object_name` (NUL-free).
    pub object_name: String,
    /// `num_tracks`.
    pub num_tracks: usize,
    /// `tracks` — owned copies.
    pub tracks: Vec<MovieTrackingTrack>,
    /// `hash` — tracks index → source track index in the caller's
    /// list (`None` slots not used).
    pub hash: HashMap<usize, usize>,
    /// `ptr` — insert cursor.
    pub ptr: usize,
}

/// `tracks_map_new`.
pub fn tracks_map_new(object_name: &str, num_tracks: usize) -> TracksMap {
    TracksMap {
        object_name: object_name.to_string(),
        num_tracks,
        tracks: Vec::with_capacity(num_tracks),
        hash: HashMap::new(),
        ptr: 0,
    }
}

/// `tracks_map_get_size`.
pub fn tracks_map_get_size(map: &TracksMap) -> usize {
    map.num_tracks
}

/// `tracks_map_insert` — `track_index` is the index of `track` in
/// the caller's source track list.
pub fn tracks_map_insert(map: &mut TracksMap, track: &MovieTrackingTrack, track_index: usize) {
    let new_track = track.clone();
    map.tracks.push(new_track);
    map.hash.insert(map.tracks.len() - 1, track_index);
    map.ptr += 1;
}

/// `tracks_map_merge` — `old_tracks` is the destination track list
/// of the resolved `MovieTrackingObject`; `uniquename` resolves
/// name collisions (C: `BLI_uniquename` over the track list).
pub fn tracks_map_merge(
    map: &mut TracksMap,
    old_tracks: &mut Vec<MovieTrackingTrack>,
    uniquename: &dyn Fn(&str, &[String]) -> String,
) {
    let mut tracks: Vec<MovieTrackingTrack> = Vec::new();

    for a in 0..map.num_tracks {
        let mut mapped_to_old = false;
        let track_idx = a;

        if let Some(&src_idx) = map.hash.get(&track_idx) {
            if src_idx < old_tracks.len() {
                let mut old_track = old_tracks.remove(src_idx);
                map.tracks[track_idx].flag = old_track.flag;
                map.tracks[track_idx].pat_flag = old_track.pat_flag;
                map.tracks[track_idx].search_flag = old_track.search_flag;
                old_track.markers = map.tracks[track_idx].markers.clone();
                tracks.push(old_track);
                mapped_to_old = true;
            }
        }

        if !mapped_to_old {
            let new_track = map.tracks[track_idx].clone();
            map.hash.insert(track_idx, tracks.len());
            tracks.push(new_track);
        }
    }

    let mut new_tracks: Vec<MovieTrackingTrack> = std::mem::take(old_tracks);

    for track in tracks {
        let names: Vec<String> = new_tracks
            .iter()
            .map(|t| {
                let end = t.name.iter().position(|&b| b == 0).unwrap_or(64);
                String::from_utf8_lossy(&t.name[..end]).into_owned()
            })
            .collect();
        let cur_name = {
            let end = track.name.iter().position(|&b| b == 0).unwrap_or(64);
            String::from_utf8_lossy(&track.name[..end]).into_owned()
        };
        let unique = uniquename(&cur_name, &names);
        let mut t = track;
        let bytes = unique.as_bytes();
        let n = bytes.len().min(63);
        t.name = [0; 64];
        t.name[..n].copy_from_slice(&bytes[..n]);
        new_tracks.push(t);
    }

    *old_tracks = new_tracks;
}

/// `unified_to_pixel`.
fn unified_to_pixel(frame_w: i32, frame_h: i32, unified: [f32; 2]) -> [f32; 2] {
    [unified[0] * frame_w as f32, unified[1] * frame_h as f32]
}

fn marker_to_frame_unified(marker: &MovieTrackingMarker, marker_unified: [f32; 2]) -> [f32; 2] {
    [
        marker_unified[0] + marker.pos[0],
        marker_unified[1] + marker.pos[1],
    ]
}

fn marker_unified_to_frame_pixel(
    frame_w: i32,
    frame_h: i32,
    marker: &MovieTrackingMarker,
    marker_unified: [f32; 2],
) -> [f32; 2] {
    unified_to_pixel(frame_w, frame_h, marker_to_frame_unified(marker, marker_unified))
}

/// `tracking_get_search_origin_frame_pixel`.
pub fn tracking_get_search_origin_frame_pixel(
    frame_w: i32,
    frame_h: i32,
    marker: &MovieTrackingMarker,
) -> [f32; 2] {
    let p = marker_unified_to_frame_pixel(frame_w, frame_h, marker, marker.search_min);
    [p[0] as i32 as f32, p[1] as i32 as f32]
}

fn pixel_to_unified(frame_w: i32, frame_h: i32, pixel: [f32; 2]) -> [f32; 2] {
    [pixel[0] / frame_w as f32, pixel[1] / frame_h as f32]
}

fn marker_unified_to_search_pixel(
    frame_w: i32,
    frame_h: i32,
    marker: &MovieTrackingMarker,
    marker_unified: [f32; 2],
) -> [f32; 2] {
    let frame_pixel = marker_unified_to_frame_pixel(frame_w, frame_h, marker, marker_unified);
    let origin = tracking_get_search_origin_frame_pixel(frame_w, frame_h, marker);
    [frame_pixel[0] - origin[0], frame_pixel[1] - origin[1]]
}

fn search_pixel_to_marker_unified(
    frame_w: i32,
    frame_h: i32,
    marker: &MovieTrackingMarker,
    search_pixel: [f32; 2],
) -> [f32; 2] {
    let origin = tracking_get_search_origin_frame_pixel(frame_w, frame_h, marker);
    let frame_unified = pixel_to_unified(
        frame_w,
        frame_h,
        [search_pixel[0] + origin[0], search_pixel[1] + origin[1]],
    );
    [
        frame_unified[0] - marker.pos[0],
        frame_unified[1] - marker.pos[1],
    ]
}

/// `tracking_get_marker_coords_for_tracking` — fills `x[5]`/`y[5]`
/// (4 pattern corners + center).
pub fn tracking_get_marker_coords_for_tracking(
    frame_w: i32,
    frame_h: i32,
    marker: &MovieTrackingMarker,
    search_pixel_x: &mut [f64; 5],
    search_pixel_y: &mut [f64; 5],
) {
    for i in 0..4 {
        let p = marker_unified_to_search_pixel(frame_w, frame_h, marker, marker.pattern_corners[i]);
        search_pixel_x[i] = p[0] as f64 - 0.5;
        search_pixel_y[i] = p[1] as f64 - 0.5;
    }
    let p = marker_unified_to_search_pixel(frame_w, frame_h, marker, [0.0, 0.0]);
    search_pixel_x[4] = p[0] as f64 - 0.5;
    search_pixel_y[4] = p[1] as f64 - 0.5;
}

/// `tracking_set_marker_coords_from_tracking`.
pub fn tracking_set_marker_coords_from_tracking(
    frame_w: i32,
    frame_h: i32,
    marker: &mut MovieTrackingMarker,
    search_pixel_x: &[f64; 5],
    search_pixel_y: &[f64; 5],
) {
    for i in 0..4 {
        let sp = [search_pixel_x[i] as f32 + 0.5, search_pixel_y[i] as f32 + 0.5];
        marker.pattern_corners[i] =
            search_pixel_to_marker_unified(frame_w, frame_h, marker, sp);
    }
    let sp = [search_pixel_x[4] as f32 + 0.5, search_pixel_y[4] as f32 + 0.5];
    let marker_unified = search_pixel_to_marker_unified(frame_w, frame_h, marker, sp);
    for i in 0..4 {
        marker.pattern_corners[i][0] -= marker_unified[0];
        marker.pattern_corners[i][1] -= marker_unified[1];
    }
    marker.pos[0] += marker_unified[0];
    marker.pos[1] += marker_unified[1];
}

/// `tracking_principal_point_normalized_to_pixel`.
pub fn tracking_principal_point_normalized_to_pixel(
    principal_point_normalized: [f32; 2],
    frame_w: i32,
    frame_h: i32,
) -> [f32; 2] {
    let cx = frame_w as f32 / 2.0;
    let cy = frame_h as f32 / 2.0;
    [
        cx + principal_point_normalized[0] * cx,
        cy + principal_point_normalized[1] * cy,
    ]
}

/// `tracking_principal_point_pixel_to_normalized`.
pub fn tracking_principal_point_pixel_to_normalized(
    principal_point_pixel: [f32; 2],
    frame_w: i32,
    frame_h: i32,
) -> [f32; 2] {
    let cx = frame_w as f32 / 2.0;
    let cy = frame_h as f32 / 2.0;
    [
        (principal_point_pixel[0] - cx) / cx,
        (principal_point_pixel[1] - cy) / cy,
    ]
}

/// `tracking_marker_insert_disabled` — `insert` is
/// `BKE_tracking_marker_insert` (sorted insert); `has_marker`
/// is `BKE_tracking_track_has_marker_at_frame`.
pub fn tracking_marker_insert_disabled(
    track: &MovieTrackingTrack,
    ref_marker: &MovieTrackingMarker,
    before: bool,
    overwrite: bool,
    has_marker: &dyn Fn(&MovieTrackingTrack, i32) -> bool,
    insert: &mut dyn FnMut(&mut MovieTrackingMarker),
) {
    let mut marker_new = ref_marker.clone();
    marker_new.flag &= !MARKER_DISABLED.wrapping_neg(); /* = ~MARKER_TRACKED below */
    marker_new.flag &= !MARKER_TRACKED;
    marker_new.flag |= MARKER_DISABLED;

    if before {
        marker_new.framenr -= 1;
    } else {
        marker_new.framenr += 1;
    }

    if overwrite || !has_marker(track, marker_new.framenr) {
        insert(&mut marker_new);
    }
}

/// `libmv_CameraIntrinsicsOptions`.
#[derive(Clone, Default)]
pub struct CameraIntrinsicsOptions {
    /// `distortion_model` — LIBMV model tag (same numbering as
    /// `TRACKING_DISTORTION_MODEL_*`).
    pub distortion_model: i16,
    /// `focal_length`.
    pub focal_length: f64,
    /// `principal_point_x`.
    pub principal_point_x: f64,
    /// `principal_point_y`.
    pub principal_point_y: f64,
    /// `polynomial_*`.
    pub polynomial_k1: f64,
    /// `polynomial_k2`.
    pub polynomial_k2: f64,
    /// `polynomial_k3`.
    pub polynomial_k3: f64,
    /// `polynomial_p1`.
    pub polynomial_p1: f64,
    /// `polynomial_p2`.
    pub polynomial_p2: f64,
    /// `division_k1`.
    pub division_k1: f64,
    /// `division_k2`.
    pub division_k2: f64,
    /// `nuke_*`.
    pub nuke_k1: f64,
    /// `nuke_k2`.
    pub nuke_k2: f64,
    /// `nuke_p1`.
    pub nuke_p1: f64,
    /// `nuke_p2`.
    pub nuke_p2: f64,
    /// `brown_*`.
    pub brown_k1: f64,
    /// `brown_k2`.
    pub brown_k2: f64,
    /// `brown_k3`.
    pub brown_k3: f64,
    /// `brown_k4`.
    pub brown_k4: f64,
    /// `brown_p1`.
    pub brown_p1: f64,
    /// `brown_p2`.
    pub brown_p2: f64,
    /// `image_width`.
    pub image_width: i32,
    /// `image_height`.
    pub image_height: i32,
}

fn distortion_model_parameters_from_tracking(
    camera: &MovieTrackingCamera,
    opts: &mut CameraIntrinsicsOptions,
) {
    match camera.distortion_model as i32 {
        m if m == TRACKING_DISTORTION_MODEL_POLYNOMIAL as i32 => {
            opts.distortion_model = TRACKING_DISTORTION_MODEL_POLYNOMIAL as i16;
            opts.polynomial_k1 = camera.k1 as f64;
            opts.polynomial_k2 = camera.k2 as f64;
            opts.polynomial_k3 = camera.k3 as f64;
            opts.polynomial_p1 = 0.0;
            opts.polynomial_p2 = 0.0;
        }
        m if m == TRACKING_DISTORTION_MODEL_DIVISION as i32 => {
            opts.distortion_model = TRACKING_DISTORTION_MODEL_DIVISION as i16;
            opts.division_k1 = camera.division_k1 as f64;
            opts.division_k2 = camera.division_k2 as f64;
        }
        m if m == TRACKING_DISTORTION_MODEL_NUKE as i32 => {
            opts.distortion_model = TRACKING_DISTORTION_MODEL_NUKE as i16;
            opts.nuke_k1 = camera.nuke_k1 as f64;
            opts.nuke_k2 = camera.nuke_k2 as f64;
            opts.nuke_p1 = camera.nuke_p1 as f64;
            opts.nuke_p2 = camera.nuke_p2 as f64;
        }
        m if m == TRACKING_DISTORTION_MODEL_BROWN as i32 => {
            opts.distortion_model = TRACKING_DISTORTION_MODEL_BROWN as i16;
            opts.brown_k1 = camera.brown_k1 as f64;
            opts.brown_k2 = camera.brown_k2 as f64;
            opts.brown_k3 = camera.brown_k3 as f64;
            opts.brown_k4 = camera.brown_k4 as f64;
            opts.brown_p1 = camera.brown_p1 as f64;
            opts.brown_p2 = camera.brown_p2 as f64;
        }
        _ => {
            opts.distortion_model = TRACKING_DISTORTION_MODEL_POLYNOMIAL as i16;
            opts.polynomial_k1 = 0.0;
            opts.polynomial_k2 = 0.0;
            opts.polynomial_k3 = 0.0;
            opts.polynomial_p1 = 0.0;
            opts.polynomial_p2 = 0.0;
        }
    }
}

fn distortion_model_parameters_from_options(
    opts: &CameraIntrinsicsOptions,
    camera: &mut MovieTrackingCamera,
) {
    match opts.distortion_model as i32 {
        m if m == TRACKING_DISTORTION_MODEL_POLYNOMIAL as i32 => {
            camera.distortion_model = TRACKING_DISTORTION_MODEL_POLYNOMIAL as i16;
            camera.k1 = opts.polynomial_k1 as f32;
            camera.k2 = opts.polynomial_k2 as f32;
            camera.k3 = opts.polynomial_k3 as f32;
        }
        m if m == TRACKING_DISTORTION_MODEL_DIVISION as i32 => {
            camera.distortion_model = TRACKING_DISTORTION_MODEL_DIVISION as i16;
            camera.division_k1 = opts.division_k1 as f32;
            camera.division_k2 = opts.division_k2 as f32;
        }
        m if m == TRACKING_DISTORTION_MODEL_NUKE as i32 => {
            camera.distortion_model = TRACKING_DISTORTION_MODEL_NUKE as i16;
            camera.nuke_k1 = opts.nuke_k1 as f32;
            camera.nuke_k2 = opts.nuke_k2 as f32;
            camera.nuke_p1 = opts.nuke_p1 as f32;
            camera.nuke_p2 = opts.nuke_p2 as f32;
        }
        m if m == TRACKING_DISTORTION_MODEL_BROWN as i32 => {
            camera.distortion_model = TRACKING_DISTORTION_MODEL_BROWN as i16;
            camera.brown_k1 = opts.brown_k1 as f32;
            camera.brown_k2 = opts.brown_k2 as f32;
            camera.brown_k3 = opts.brown_k3 as f32;
            camera.brown_k4 = opts.brown_k4 as f32;
            camera.brown_p1 = opts.brown_p1 as f32;
            camera.brown_p2 = opts.brown_p2 as f32;
        }
        _ => debug_assert!(false, "Unknown distortion model"),
    }
}

/// `tracking_cameraIntrinsicsOptionsFromTracking`.
pub fn tracking_camera_intrinsics_options_from_tracking(
    tracking: &MovieTracking,
    calibration_width: i32,
    calibration_height: i32,
) -> CameraIntrinsicsOptions {
    let camera = &tracking.camera;
    let aspy = 1.0 / tracking.camera.pixel_aspect;
    let principal_px = tracking_principal_point_normalized_to_pixel(
        camera.principal_point,
        calibration_width,
        calibration_height,
    );
    let mut opts = CameraIntrinsicsOptions {
        focal_length: camera.focal as f64,
        principal_point_x: principal_px[0] as f64,
        principal_point_y: (principal_px[1] * aspy) as f64,
        image_width: calibration_width,
        image_height: (calibration_height as f32 * aspy) as i32,
        ..Default::default()
    };
    distortion_model_parameters_from_tracking(camera, &mut opts);
    opts
}

/// `tracking_trackingCameraFromIntrinsicsOptions`.
pub fn tracking_tracking_camera_from_intrinsics_options(
    tracking: &mut MovieTracking,
    opts: &CameraIntrinsicsOptions,
) {
    let camera = &mut tracking.camera;
    camera.focal = opts.focal_length as f32;
    camera.principal_point = tracking_principal_point_pixel_to_normalized(
        [opts.principal_point_x as f32, opts.principal_point_y as f32],
        opts.image_width,
        opts.image_height,
    );
    distortion_model_parameters_from_options(opts, camera);
}

/// `tracking_get_keyframed_marker` — `marker_get` is
/// `BKE_tracking_marker_get(track, framenr)` → marker index.
pub fn tracking_get_keyframed_marker(
    track: &MovieTrackingTrack,
    current_frame: i32,
    backwards: bool,
    marker_get: &dyn Fn(&MovieTrackingTrack, i32) -> Option<usize>,
) -> Option<usize> {
    let mut marker_keyed: Option<usize> = None;
    let mut marker_keyed_fallback: Option<usize> = None;
    let mut a = marker_get(track, current_frame).unwrap_or(0) as i64;
    let markersnr = track.markers.len() as i64;

    while a >= 0 && a < markersnr {
        let next = if backwards { a + 1 } else { a - 1 };
        let mut is_keyframed = false;
        let cur = &track.markers[a as usize];
        let next_marker = if next >= 0 && next < markersnr {
            Some(&track.markers[next as usize])
        } else {
            None
        };

        if cur.flag & MARKER_DISABLED == 0 {
            match next_marker {
                None => {
                    if marker_keyed_fallback.is_none() {
                        marker_keyed_fallback = Some(a as usize);
                    }
                }
                Some(nm) if nm.flag & MARKER_DISABLED != 0 => {
                    if marker_keyed_fallback.is_none() {
                        marker_keyed_fallback = Some(a as usize);
                    }
                }
                _ => {}
            }
            is_keyframed |= cur.flag & MARKER_TRACKED == 0;
        }
        if is_keyframed {
            marker_keyed = Some(a as usize);
            break;
        }
        a = next;
    }

    marker_keyed.or(marker_keyed_fallback)
}
