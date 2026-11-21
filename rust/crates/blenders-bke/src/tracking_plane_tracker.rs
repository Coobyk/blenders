//! Port of `intern/tracking_plane_tracker.cc` — `BKE_tracking_track_plane_
//! from_existing_motion`, retrack, `BKE_tracking_homography_between_two_quads`.
//!
//! Marker access (`BKE_tracking_(plane_)marker_get(_exact)`/`_insert`) and
//! `libmv_homography2DFromCorrespondencesEuc` are injected — tracking store
//! and libmv are PORT_PENDING.

use blenders_dna::tracking_types::{
    MovieTrackingPlaneMarker, MovieTrackingPlaneTrack, PLANE_MARKER_TRACKED, PLANE_TRACK_AUTOKEY,
};
use blenders_math::matrix::Mat3;

/// Marker store access injected by the caller (the plane track's marker +
/// point-track marker lists live in the tracking store).
pub struct PlaneTrackOps<'a> {
    /// `BKE_tracking_plane_marker_get` — marker at `frame` (exact or
    /// interpolated, per C semantics).
    pub plane_marker_get: &'a mut dyn FnMut(&mut MovieTrackingPlaneTrack, i32) -> Option<usize>,
    /// `BKE_tracking_plane_marker_get_exact`.
    pub plane_marker_get_exact: &'a mut dyn FnMut(&mut MovieTrackingPlaneTrack, i32) -> Option<usize>,
    /// `BKE_tracking_plane_marker_insert`.
    pub plane_marker_insert: &'a mut dyn FnMut(&mut MovieTrackingPlaneTrack, &MovieTrackingPlaneMarker),
    /// `BKE_tracking_marker_get_exact` on a point track (handle arg is the
    /// `point_tracks[i]` value) → marker `pos`.
    pub point_marker_pos_exact: &'a mut dyn FnMut(usize, i32) -> Option<[f32; 2]>,
    /// `libmv_homography2DFromCorrespondencesEuc` → 3x3 H.
    pub homography: &'a mut dyn FnMut(&[[f64; 2]], &[[f64; 2]]) -> Option<Mat3>,
    /// Marker storage (plane_track->markers array) — indices from
    /// `plane_marker_get*` index into it.
    pub markers: &'a mut Vec<MovieTrackingPlaneMarker>,
}

/// `point_markers_correspondences_on_both_image` — `(x1, x2)` pairs for
/// markers present on both frames.
fn point_markers_correspondences_on_both_image(
    plane_track: &mut MovieTrackingPlaneTrack,
    frame1: i32,
    frame2: i32,
    ops: &mut PlaneTrackOps,
) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
    let mut x1 = Vec::new();
    let mut x2 = Vec::new();
    let n = plane_track.point_tracksnr as usize;
    for i in 0..n {
        let point_track = plane_track.point_tracks[i];
        if let (Some(p1), Some(p2)) = (
            (ops.point_marker_pos_exact)(point_track, frame1),
            (ops.point_marker_pos_exact)(point_track, frame2),
        ) {
            x1.push([p1[0] as f64, p1[1] as f64]);
            x2.push([p2[0] as f64, p2[1] as f64]);
        }
    }
    (x1, x2)
}

/// `track_plane_from_existing_motion`.
fn track_plane_from_existing_motion(
    plane_track: &mut MovieTrackingPlaneTrack,
    start_frame: i32,
    direction: i32,
    retrack: bool,
    ops: &mut PlaneTrackOps,
) {
    let Some(start_idx) = (ops.plane_marker_get)(plane_track, start_frame) else {
        return;
    };
    let start_plane_marker_framenr = ops.markers[start_idx].framenr;
    let mut keyframe_plane_marker_framenr: Option<i32> = None;
    let frame_delta = if direction > 0 { 1 } else { -1 };
    if plane_track.flag & PLANE_TRACK_AUTOKEY != 0 {
        let mut current_frame = start_frame;
        loop {
            current_frame += frame_delta;
            let Some(next_idx) = (ops.plane_marker_get_exact)(plane_track, current_frame + frame_delta) else {
                break;
            };
            if ops.markers[next_idx].flag & PLANE_MARKER_TRACKED == 0 {
                keyframe_plane_marker_framenr = Some(ops.markers[next_idx].framenr);
                break;
            }
        }
    } else {
        ops.markers[start_idx].flag |= PLANE_MARKER_TRACKED;
    }
    let mut new_plane_marker = ops.markers[start_idx].clone();
    new_plane_marker.flag |= PLANE_MARKER_TRACKED;
    let mut current_frame = start_frame;
    loop {
        current_frame += frame_delta;
        let next_idx = (ops.plane_marker_get_exact)(plane_track, current_frame + frame_delta);
        if let Some(ni) = next_idx {
            if ops.markers[ni].flag & PLANE_MARKER_TRACKED == 0
                && plane_track.flag & PLANE_TRACK_AUTOKEY != 0
            {
                break;
            }
        }
        let (x1, x2) = point_markers_correspondences_on_both_image(
            plane_track, current_frame, current_frame + frame_delta, ops,
        );
        if x1.len() < 4 {
            break;
        }
        let Some(h) = (ops.homography)(&x1, &x2) else {
            break;
        };
        for i in 0..4 {
            let vec = [new_plane_marker.corners[i][0], new_plane_marker.corners[i][1], 1.0f32];
            let v = h * blenders_math::vector::Vec3::new(vec[0], vec[1], vec[2]);
            new_plane_marker.corners[i] = [v.x / v.z, v.y / v.z];
        }
        new_plane_marker.framenr = current_frame + frame_delta;
        if !retrack {
            if let (Some(kf), Some(ni)) = (keyframe_plane_marker_framenr, next_idx) {
                if plane_track.flag & PLANE_TRACK_AUTOKEY != 0 {
                    let mut fac = (ops.markers[ni].framenr - start_plane_marker_framenr) as f32
                        / (kf - start_plane_marker_framenr) as f32;
                    fac = 3.0 * fac * fac - 2.0 * fac * fac * fac;
                    for i in 0..4 {
                        let a = new_plane_marker.corners[i];
                        let b = ops.markers[ni].corners[i];
                        new_plane_marker.corners[i] = [
                            a[0] + (b[0] - a[0]) * fac,
                            a[1] + (b[1] - a[1]) * fac,
                        ];
                    }
                }
            }
        }
        (ops.plane_marker_insert)(plane_track, &new_plane_marker);
    }
}

/// `BKE_tracking_track_plane_from_existing_motion`.
pub fn bke_tracking_track_plane_from_existing_motion(
    plane_track: &mut MovieTrackingPlaneTrack,
    start_frame: i32,
    ops: &mut PlaneTrackOps,
) {
    track_plane_from_existing_motion(plane_track, start_frame, 1, false, ops);
    track_plane_from_existing_motion(plane_track, start_frame, -1, false, ops);
}

/// `find_plane_keyframe` — walk `markers` from `start_frame` until the first
/// non-TRACKED marker in `direction`.
fn find_plane_keyframe(
    plane_track: &mut MovieTrackingPlaneTrack,
    start_frame: i32,
    direction: i32,
    ops: &mut PlaneTrackOps,
) -> Option<usize> {
    let start_idx = (ops.plane_marker_get)(plane_track, start_frame)?;
    let frame_delta = if direction > 0 { 1i64 } else { -1i64 };
    let mut index = start_idx as i64;
    while index >= 0 && (index as usize) < plane_track.markersnr as usize {
        let m = &ops.markers[index as usize];
        if m.flag & PLANE_MARKER_TRACKED == 0 {
            return Some(index as usize);
        }
        index += frame_delta;
    }
    None
}

/// `BKE_tracking_retrack_plane_from_existing_motion_at_segment`.
pub fn bke_tracking_retrack_plane_from_existing_motion_at_segment(
    plane_track: &mut MovieTrackingPlaneTrack,
    start_frame: i32,
    ops: &mut PlaneTrackOps,
) {
    let prev = find_plane_keyframe(plane_track, start_frame, -1, ops);
    let next = find_plane_keyframe(plane_track, start_frame, 1, ops);
    match (prev, next) {
        (Some(p), Some(n)) => {
            track_plane_from_existing_motion(plane_track, ops.markers[p].framenr, 1, true, ops);
            track_plane_from_existing_motion(plane_track, ops.markers[n].framenr, -1, false, ops);
        }
        (Some(p), None) => {
            track_plane_from_existing_motion(plane_track, ops.markers[p].framenr, 1, true, ops);
        }
        (None, Some(n)) => {
            track_plane_from_existing_motion(plane_track, ops.markers[n].framenr, -1, true, ops);
        }
        (None, None) => {}
    }
}

/// `BKE_tracking_homography_between_two_quads` — H mapping `reference` →
/// `corners` via 4 correspondences.
pub fn bke_tracking_homography_between_two_quads(
    reference_corners: [[f32; 2]; 4],
    corners: [[f32; 2]; 4],
    homography: &mut dyn FnMut(&[[f64; 2]], &[[f64; 2]]) -> Option<Mat3>,
) -> Option<Mat3> {
    let x1: Vec<[f64; 2]> = reference_corners.iter().map(|c| [c[0] as f64, c[1] as f64]).collect();
    let x2: Vec<[f64; 2]> = corners.iter().map(|c| [c[0] as f64, c[1] as f64]).collect();
    homography(&x1, &x2)
}
