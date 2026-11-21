//! Port of `intern/tracking_detect.cc` — `BKE_tracking_detect_fast/_harris`
//! feature detection entry points.
//!
//! libmv (the detector) and `BKE_tracking_track_add` stay behind closures —
//! PORT_PENDING until the tracking store + libmv backend land.

use blenders_dna::gpencil_legacy_types::{bGPDspoint, bGPDstroke};

/// `check_point_in_stroke` — winding count, odd = inside. `points` is the
/// stroke's point array (the stroke's `points` field is an arena handle).
pub fn check_point_in_stroke(stroke: &bGPDstroke, points: &[bGPDspoint], x: f32, y: f32) -> bool {
    let mut count = 0i32;
    let mut prev = stroke.totpoints as usize - 1;
    for i in 0..stroke.totpoints as usize {
        if (points[i].y < y && points[prev].y >= y) || (points[prev].y < y && points[i].y >= y) {
            let fac = (y - points[i].y) / (points[prev].y - points[i].y);
            if points[i].x + fac * (points[prev].x - points[i].x) < x {
                count += 1;
            }
        }
        prev = i;
    }
    count % 2 != 0
}

/// `check_point_in_layer` — true when `(x,y)` is inside any stroke of the
/// layer's frames. `points_of` resolves a stroke's point arena.
pub fn check_point_in_layer<'a>(
    strokes: impl IntoIterator<Item = &'a bGPDstroke>,
    points_of: &dyn Fn(&bGPDstroke) -> &[bGPDspoint],
    x: f32,
    y: f32,
) -> bool {
    for stroke in strokes {
        if check_point_in_stroke(stroke, points_of(stroke), x, y) {
            return true;
        }
    }
    false
}

/// `libmv_DetectOptions` (libmv-capi.h).
#[derive(Default)]
pub struct LibmvDetectOptions {
    /// `LIBMV_DETECTOR_*`.
    pub detector: i32,
    pub margin: i32,
    pub min_distance: i32,
    pub fast_min_trackness: i32,
    pub harris_threshold: f32,
}

pub const LIBMV_DETECTOR_FAST: i32 = 0;
pub const LIBMV_DETECTOR_HARRIS: i32 = 1;

/// One libmv feature as `libmv_getFeature` reports it.
pub struct LibmvFeature {
    pub x: f64,
    pub y: f64,
    pub score: f64,
    pub size: f64,
}

/// `detect_retrieve_libmv_features` — normalized coords + layer mask +
/// track add (`TRACK_SELECT` on flag/pat_flag/search_flag).
///
/// `check_layer`: `check_point_in_layer` bound to the layer's strokes, or
/// `None`; `add_track`: `BKE_tracking_track_add` + select flags
/// (PORT_PENDING — receives `(xu, yu, framenr, width, height)`).
pub fn detect_retrieve_libmv_features(
    features: &[LibmvFeature],
    framenr: i32,
    width: i32,
    height: i32,
    check_layer: &mut dyn FnMut(f32, f32) -> bool,
    place_outside_layer: bool,
    add_track: &mut dyn FnMut(f32, f32, i32, i32, i32),
) {
    for f in features {
        let xu = (f.x + 0.5) as f32 / width as f32;
        let yu = (f.y + 0.5) as f32 / height as f32;
        /* C: `ok = check_point_in_layer(layer,...) != place_outside_layer`
         * when a mask layer exists; callers pass a mask check or a
         * `|_,_| true` no-op when none. */
        let ok = check_layer(xu, yu) != place_outside_layer;
        if ok {
            add_track(xu, yu, framenr, width, height);
        }
    }
}
