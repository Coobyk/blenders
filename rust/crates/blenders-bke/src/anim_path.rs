//! Port of `intern/anim_path.cc` (`BKE_anim_path_*`/`BKE_where_on_path`)
//! plus `key_curve_{position,tangent}_weights` from `intern/key.cc`.
//!
//! `Object`/`Curve`/`CurveCache` are handles — the bevlist and accumulated
//! lengths arrive as slices; everything else is verbatim.

use blenders_dna::curve_types::{BevList, BevPoint};
use blenders_math::vector::Vec3;
use blenders_math::vector::Vec4;

/// `KeyInterpolationType` (DNA_key_types.h).
pub const KEY_LINEAR: i32 = 0;
pub const KEY_CARDINAL: i32 = 1;
pub const KEY_BSPLINE: i32 = 2;
pub const KEY_CATMULL_ROM: i32 = 3;

/// `key_curve_position_weights`.
pub fn key_curve_position_weights(t: f32, type_: i32) -> [f32; 4] {
    match type_ {
        KEY_LINEAR => [0.0, -t + 1.0, t, 0.0],
        KEY_CARDINAL | KEY_CATMULL_ROM => {
            let t2 = t * t;
            let t3 = t2 * t;
            let fc = if type_ == KEY_CARDINAL { 0.71 } else { 0.5 };
            [
                -fc * t3 + 2.0 * fc * t2 - fc * t,
                (2.0 - fc) * t3 + (fc - 3.0) * t2 + 1.0,
                (fc - 2.0) * t3 + (3.0 - 2.0 * fc) * t2 + fc * t,
                fc * t3 - fc * t2,
            ]
        }
        _ => {
            /* KEY_BSPLINE */
            let t2 = t * t;
            let t3 = t2 * t;
            [
                -0.16666666 * t3 + 0.5 * t2 - 0.5 * t + 0.16666666,
                0.5 * t3 - t2 + 0.66666666,
                -0.5 * t3 + 0.5 * t2 + 0.5 * t + 0.16666666,
                0.16666666 * t3,
            ]
        }
    }
}

/// `key_curve_tangent_weights`.
pub fn key_curve_tangent_weights(t: f32, type_: i32) -> [f32; 4] {
    match type_ {
        KEY_LINEAR => [0.0, -1.0, 1.0, 0.0],
        KEY_CARDINAL | KEY_CATMULL_ROM => {
            let t2 = t * t;
            let fc = if type_ == KEY_CARDINAL { 0.71 } else { 0.5 };
            [
                -3.0 * fc * t2 + 4.0 * fc * t - fc,
                3.0 * (2.0 - fc) * t2 + 2.0 * (fc - 3.0) * t,
                3.0 * (fc - 2.0) * t2 + 2.0 * (3.0 - 2.0 * fc) * t + fc,
                3.0 * fc * t2 - 2.0 * fc * t,
            ]
        }
        _ => {
            let t2 = t * t;
            [
                -0.5 * t2 + t - 0.5,
                1.5 * t2 - t * 2.0,
                -1.5 * t2 + t + 0.5,
                0.5 * t2,
            ]
        }
    }
}

/// `get_bevlist_seg_array_size` — segments = `nr` cyclic / `nr-1` open.
pub fn get_bevlist_seg_array_size(bl: &BevList) -> i32 {
    if bl.poly >= 0 {
        bl.nr
    } else {
        bl.nr - 1
    }
}

/// `BKE_anim_path_get_array_size` — `bl` = `curve_cache->bev.first()`.
pub fn bke_anim_path_get_array_size(bl: &BevList) -> i32 {
    debug_assert!(bl.nr > 1);
    get_bevlist_seg_array_size(bl)
}

/// `BKE_anim_path_get_length` — total path length from the accum array.
pub fn bke_anim_path_get_length(anim_path_accum_length: &[f32]) -> f32 {
    *anim_path_accum_length.last().unwrap_or(&0.0)
}

/// `BKE_anim_path_calc_data` — accumulated segment lengths
/// (`anim_path_accum_length[0..seg_size]`). `bevpoints` = `bl->bevpoints`.
pub fn bke_anim_path_calc_data(bl: &BevList, bevpoints: &[BevPoint]) -> Vec<f32> {
    let seg_size = get_bevlist_seg_array_size(bl);
    let mut len_data = Vec::with_capacity(seg_size as usize);
    let mut prev_len = 0.0f32;
    for i in 0..(bl.nr - 1) as usize {
        prev_len += (Vec3::from(bevpoints[i + 1].vec) - Vec3::from(bevpoints[i].vec)).length();
        len_data.push(prev_len);
    }
    if bl.poly >= 0 {
        /* Cyclic — wrap last→first. */
        prev_len += (Vec3::from(bevpoints[0].vec)
            - Vec3::from(bevpoints[bl.nr as usize - 1].vec))
        .length();
        len_data.push(prev_len);
    }
    len_data
}

/// `get_curve_points_from_idx` — 4 curve points around segment `idx`
/// (endpoint duplication rules verbatim).
fn get_curve_points_from_idx<'a>(
    idx: i32,
    bl: &BevList,
    bp_arr: &'a [BevPoint],
) -> (&'a BevPoint, &'a BevPoint, &'a BevPoint, &'a BevPoint) {
    let is_cyclic = bl.poly >= 0;
    let nr = bl.nr as usize;
    let idx = idx as usize;
    if idx == 0 {
        let p0 = if is_cyclic { &bp_arr[nr - 1] } else { &bp_arr[0] };
        let p3 = if nr > 2 { &bp_arr[2] } else { &bp_arr[1] };
        return (p0, &bp_arr[0], &bp_arr[1], p3);
    }
    if idx == nr - 2 {
        let p0 = &bp_arr[nr - 3];
        let p3 = if is_cyclic { &bp_arr[0] } else { &bp_arr[nr - 1] };
        return (p0, &bp_arr[nr - 2], &bp_arr[nr - 1], p3);
    }
    if idx == nr - 1 {
        return (&bp_arr[nr - 3], &bp_arr[nr - 2], &bp_arr[nr - 1], &bp_arr[0]);
    }
    (&bp_arr[idx - 1], &bp_arr[idx], &bp_arr[idx + 1], &bp_arr[idx + 2])
}

/// `binary_search_anim_path` — segment index + fraction for `goal_len`.
fn binary_search_anim_path(accum_len_arr: &[f32], seg_size: i32, goal_len: f32) -> Option<(i32, f32)> {
    if seg_size == 1 {
        return Some((0, goal_len / accum_len_arr[0]));
    }
    let mut cur_base = 0i32;
    let mut cur_step = seg_size - 1;
    loop {
        let cur_idx = cur_base + cur_step / 2;
        let left_len = accum_len_arr[cur_idx as usize];
        let right_len = accum_len_arr[cur_idx as usize + 1];
        if left_len <= goal_len && right_len > goal_len {
            return Some((cur_idx + 1, (goal_len - left_len) / (right_len - left_len)));
        }
        if cur_idx == 0 {
            return Some((0, goal_len / accum_len_arr[0]));
        }
        if cur_step == 0 {
            debug_assert!(false, "no valid point on anim path");
            return None;
        }
        if left_len < goal_len {
            cur_base = cur_idx + 1;
            cur_step -= 1;
        }
        cur_step /= 2;
    }
}

/// `BKE_where_on_path` — returns (vec4 dir+tilt, dir, quat, radius, weight)
/// tuple of what the C writes into its `r_*` outs; `None` = the failure
/// branches (missing cache/bevlist/`nr < 2`).
///
/// `nurb_type` = first `Nurb`->type from `BKE_curve_editNurbs_get` or
/// `cu->nurb` (caller resolves — edit-mode dependent).
pub fn bke_where_on_path(
    bl: &BevList,
    bevpoints: &[BevPoint],
    anim_path_accum_length: &[f32],
    nurb_type: i32,
    mut ctime: f32,
) -> Option<(Vec4, Vec3, Vec4, f32, f32)> {
    if bl.nr < 2 {
        return None;
    }
    let is_cyclic = bl.poly >= 0;
    if is_cyclic && (ctime < 0.0 || ctime > 1.0) {
        ctime -= ctime.floor();
    }
    let seg_size = get_bevlist_seg_array_size(bl);
    let accum_len_arr = anim_path_accum_length;
    let goal_len = ctime * accum_len_arr[seg_size as usize - 1];

    let (idx, frac);
    if ctime <= 0.0 || ctime >= 1.0 {
        let clamp_time = ctime.clamp(0.0, 1.0);
        idx = (clamp_time * (seg_size - 1) as f32) as i32;
        frac = if idx == 0 {
            goal_len / accum_len_arr[0]
        } else {
            (goal_len - accum_len_arr[idx as usize - 1])
                / (accum_len_arr[idx as usize] - accum_len_arr[idx as usize - 1])
        };
    } else {
        let found = binary_search_anim_path(accum_len_arr, seg_size, goal_len)?;
        idx = found.0;
        frac = found.1;
    }
    let (p0, p1, p2, p3) = get_curve_points_from_idx(idx, bl, bevpoints);

    /* Direction — tangent weights, negated for `vec_to_quat` compat. */
    let wt = key_curve_tangent_weights(frac, KEY_BSPLINE);
    let dir = -(Vec3::from(p0.vec) * wt[0]
        + Vec3::from(p1.vec) * wt[1]
        + Vec3::from(p2.vec) * wt[2]
        + Vec3::from(p3.vec) * wt[3]);

    /* Position weights. */
    const CU_POLY: i32 = 0;
    const CU_BEZIER: i32 = 1;
    const CU_NURBS: i32 = 4;
    let mut w = if matches!(nurb_type, CU_POLY | CU_BEZIER | CU_NURBS) {
        key_curve_position_weights(frac, KEY_LINEAR)
    } else if core::ptr::eq(p2, p3) {
        key_curve_position_weights(frac, KEY_CARDINAL)
    } else {
        key_curve_position_weights(frac, KEY_BSPLINE)
    };

    let v = Vec3::new(
        w[0] * p0.vec[0] + w[1] * p1.vec[0] + w[2] * p2.vec[0] + w[3] * p3.vec[0],
        w[0] * p0.vec[1] + w[1] * p1.vec[1] + w[2] * p2.vec[1] + w[3] * p3.vec[1],
        w[0] * p0.vec[2] + w[1] * p1.vec[2] + w[2] * p2.vec[2] + w[3] * p3.vec[2],
    );

    /* Clamp weights to 0-1 (no extrapolation for tilt/quat/radius/weight). */
    for wi in &mut w {
        *wi = wi.clamp(0.0, 1.0);
    }
    let vec = Vec4::new(v.x, v.y, v.z, w[0] * p0.tilt + w[1] * p1.tilt + w[2] * p2.tilt + w[3] * p3.tilt);

    /* Quaternion — two-level slerp (`interp_qt_qtqt`). */
    let quat = {
        let totfac = w[0] + w[3];
        let q1 = if totfac > f32::EPSILON {
            blenders_math::quaternion::interp_qt_qtqt(
                Vec4::from(p0.quat), Vec4::from(p3.quat), w[3] / totfac)
        } else {
            Vec4::from(p1.quat)
        };
        let totfac = w[1] + w[2];
        let q2 = if totfac > f32::EPSILON {
            blenders_math::quaternion::interp_qt_qtqt(
                Vec4::from(p1.quat), Vec4::from(p2.quat), w[2] / totfac)
        } else {
            Vec4::from(p3.quat)
        };
        let totfac = w[0] + w[1] + w[2] + w[3];
        if totfac > f32::EPSILON {
            blenders_math::quaternion::interp_qt_qtqt(q1, q2, (w[1] + w[2]) / totfac)
        } else {
            q2
        }
    };

    let radius = w[0] * p0.radius + w[1] * p1.radius + w[2] * p2.radius + w[3] * p3.radius;
    let weight = w[0] * p0.weight + w[1] * p1.weight + w[2] * p2.weight + w[3] * p3.weight;
    Some((vec, dir, quat, radius, weight))
}
