//! Port of `intern/mask_evaluate.cc` — mask spline resolution,
//! differentiation, feather points (even/smooth offset, inner-loop
//! collapse), per-point helpers used here (`segment_co`/`normal`/
//! `weight`/`weight_scalar`/`next_bezt`), and layer evaluation.
//!
//! Depsgraph/shape/parent parts are injected via closures.

use blenders_dna::curve_enums::{HD_AUTO, HD_VECT};
use blenders_dna::curve_types::BezTriple;
use blenders_dna::mask_types::{
    MaskSpline, MaskSplinePoint, MASK_SPLINE_CYCLIC, MASK_SPLINE_INTERP_EASE,
    MASK_SPLINE_NOINTERSECT, MASK_SPLINE_OFFSET_EVEN,
};
use blenders_math::vector::Vec2;

/// `MASK_RESOL_MAX`.
const MASK_RESOL_MAX: i32 = 128;

fn bzt2(b: &BezTriple, i: usize) -> Vec2 {
    Vec2::new(b.vec[i][0], b.vec[i][1])
}

/// `BKE_mask_spline_point_next_bezt` — index-based.
fn next_bezt_index(spline: &MaskSpline, point_index: usize) -> Option<usize> {
    if point_index == spline.tot_point as usize - 1 {
        if spline.flag & MASK_SPLINE_CYCLIC != 0 {
            return Some(0);
        }
        return None;
    }
    Some(point_index + 1)
}

/// `mask_spline_point_next` / `_prev` (cyclic).
fn point_next_index(spline: &MaskSpline, point_index: usize) -> Option<usize> {
    next_bezt_index(spline, point_index)
}
fn point_prev_index(spline: &MaskSpline, point_index: usize) -> Option<usize> {
    if point_index == 0 {
        if spline.flag & MASK_SPLINE_CYCLIC != 0 {
            return Some(spline.tot_point as usize - 1);
        }
        return None;
    }
    Some(point_index - 1)
}

fn cubic_interp_v2(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let it = 1.0 - t;
    let a = it * it * it;
    let b = 3.0 * it * it * t;
    let c = 3.0 * it * t * t;
    let d = t * t * t;
    p0 * a + p1 * b + p2 * c + p3 * d
}

/// `BKE_mask_point_segment_co`.
pub fn bke_mask_point_segment_co(spline: &MaskSpline, point_index: usize, u: f32) -> Vec2 {
    let point = &spline.points[point_index];
    let Some(next) = next_bezt_index(spline, point_index) else {
        return bzt2(&point.bezt, 1);
    };
    let bezt_next = &spline.points[next].bezt;
    cubic_interp_v2(
        bzt2(&point.bezt, 1),
        bzt2(&point.bezt, 2),
        bzt2(bezt_next, 0),
        bzt2(bezt_next, 1),
        u,
    )
}

fn orthogonal_direction_get(vec: Vec2) -> Vec2 {
    Vec2::new(-vec.y, vec.x).normalized()
}

/// `BKE_mask_point_normal`.
pub fn bke_mask_point_normal(spline: &MaskSpline, point_index: usize, u: f32) -> Vec2 {
    let du = 0.05f32;
    let point_prev = point_prev_index(spline, point_index);
    let point_next = point_next_index(spline, point_index);
    let bezt = &spline.points[point_index].bezt;

    if u - du < 0.0 && point_prev.is_none() {
        let co = bke_mask_point_segment_co(spline, point_index, u + du);
        orthogonal_direction_get(co - bzt2(bezt, 1))
    } else if u + du > 1.0 && point_next.is_none() {
        let co = bke_mask_point_segment_co(spline, point_index, u - du);
        orthogonal_direction_get(bzt2(bezt, 1) - co)
    } else {
        let prev_co = if u - du < 0.0 {
            bke_mask_point_segment_co(spline, point_prev.unwrap(), 1.0 + (u - du))
        } else {
            bke_mask_point_segment_co(spline, point_index, u - du)
        };
        let co = bke_mask_point_segment_co(spline, point_index, u);
        let next_co = if u + du > 1.0 {
            bke_mask_point_segment_co(spline, point_next.unwrap(), u + du - 1.0)
        } else {
            bke_mask_point_segment_co(spline, point_index, u + du)
        };
        let dir = (co - prev_co).normalized() + (next_co - co).normalized();
        orthogonal_direction_get(dir)
    }
}

fn mask_point_interp_weight(bezt: &BezTriple, bezt_next: &BezTriple, u: f32) -> f32 {
    bezt.weight * (1.0 - u) + bezt_next.weight * u
}

/// `BKE_mask_point_weight_scalar`.
pub fn bke_mask_point_weight_scalar(spline: &MaskSpline, point_index: usize, u: f32) -> f32 {
    let bezt = &spline.points[point_index].bezt;
    let Some(next) = next_bezt_index(spline, point_index) else {
        return bezt.weight;
    };
    if u <= 0.0 {
        return bezt.weight;
    }
    if u >= 1.0 {
        return spline.points[next].bezt.weight;
    }
    mask_point_interp_weight(bezt, &spline.points[next].bezt, u)
}

/// `BKE_mask_point_weight`.
pub fn bke_mask_point_weight(spline: &MaskSpline, point_index: usize, u: f32) -> f32 {
    let point = &spline.points[point_index];
    let bezt = &point.bezt;
    let Some(next) = next_bezt_index(spline, point_index) else {
        return bezt.weight;
    };
    if u <= 0.0 {
        return bezt.weight;
    }
    let bezt_next = &spline.points[next].bezt;
    if u >= 1.0 {
        return bezt_next.weight;
    }

    let tot_uw = point.tot_uw as usize;
    let mut cur_u = 0.0f32;
    let mut cur_w = 0.0f32;
    let mut next_u = 0.0f32;
    let mut next_w = 0.0f32;
    for i in 0..=tot_uw {
        if i == 0 {
            cur_u = 0.0;
            cur_w = 1.0;
        } else {
            cur_u = point.uw[i - 1].u;
            cur_w = point.uw[i - 1].w;
        }
        if i == tot_uw {
            next_u = 1.0;
            next_w = 1.0;
        } else {
            next_u = point.uw[i].u;
            next_w = point.uw[i].w;
        }
        if u >= cur_u && u <= next_u {
            break;
        }
    }
    let fac = (u - cur_u) / (next_u - cur_u);
    cur_w *= mask_point_interp_weight(bezt, bezt_next, cur_u);
    next_w *= mask_point_interp_weight(bezt, bezt_next, next_u);
    if spline.weight_interp == MASK_SPLINE_INTERP_EASE {
        return cur_w + (next_w - cur_w) * (3.0 * fac * fac - 2.0 * fac * fac * fac);
    }
    (1.0 - fac) * cur_w + fac * next_w
}

/// `BKE_curve_forward_diff_bezier` — writes `it+1` points at
/// `stride`-spaced positions in `p`.
pub fn bke_curve_forward_diff_bezier(
    q0: f32,
    q1: f32,
    q2: f32,
    q3: f32,
    p: &mut [f32],
    start: usize,
    it: i32,
    stride: usize,
) {
    let f = it as f32;
    let rt0 = q0;
    let rt1 = 3.0 * (q1 - q0) / f;
    let f2 = f * f;
    let rt2 = 3.0 * (q0 - 2.0 * q1 + q2) / f2;
    let f3 = f2 * f;
    let rt3 = (q3 - q0 + 3.0 * (q1 - q2)) / f3;

    let mut q0 = rt0;
    let mut q1 = rt1 + rt2 + rt3;
    let mut q2 = 2.0 * rt2 + 6.0 * rt3;
    let q3 = 6.0 * rt3;

    let mut i = start;
    for _ in 0..=it {
        p[i] = q0;
        i += stride;
        q0 += q1;
        q1 += q2;
        q2 += q3;
    }
}

/// `BKE_mask_spline_resolution`.
pub fn bke_mask_spline_resolution(spline: &MaskSpline, width: i32, height: i32) -> i32 {
    let mut max_segment = 0.01f32;
    let mut resol = 1i32;
    if width != 0 && height != 0 {
        max_segment = 1.0 / width.max(height) as f32;
    }
    for i in 0..spline.tot_point as usize {
        let bezt_curr = &spline.points[i].bezt;
        let Some(next) = next_bezt_index(spline, i) else {
            break;
        };
        let bezt_next = &spline.points[next].bezt;
        let a = (bzt2(bezt_curr, 1) - bzt2(bezt_curr, 2)).length();
        let b = (bzt2(bezt_curr, 2) - bzt2(bezt_next, 0)).length();
        let c = (bzt2(bezt_next, 0) - bzt2(bezt_next, 1)).length();
        let cur_resol = ((a + b + c) / max_segment) as i32;
        resol = resol.max(cur_resol);
        if resol >= MASK_RESOL_MAX {
            break;
        }
    }
    resol.clamp(1, MASK_RESOL_MAX)
}

/// `BKE_mask_spline_feather_resolution`.
pub fn bke_mask_spline_feather_resolution(spline: &MaskSpline, width: i32, height: i32) -> i32 {
    let max_segment = 0.005f32;
    let resol = bke_mask_spline_resolution(spline, width, height);
    if resol >= MASK_RESOL_MAX {
        return MASK_RESOL_MAX;
    }
    let mut max_jump = 0.0f32;
    for point in &spline.points {
        let mut prev_u = 0.0f32;
        let mut prev_w = point.bezt.weight;
        for uw in &point.uw {
            let w_diff = uw.w - prev_w;
            let u_diff = uw.u - prev_u;
            if u_diff > f32::EPSILON {
                max_jump = max_jump.max((w_diff / u_diff).abs());
            }
            prev_u = uw.u;
            prev_w = uw.w;
        }
    }
    (resol + (max_jump / max_segment) as i32).clamp(1, MASK_RESOL_MAX)
}

/// `BKE_mask_spline_differentiate_calc_total`.
pub fn bke_mask_spline_differentiate_calc_total(spline: &MaskSpline, resol: u32) -> usize {
    if spline.flag & MASK_SPLINE_CYCLIC != 0 {
        return spline.tot_point as usize * resol as usize;
    }
    (spline.tot_point as usize - 1) * resol as usize + 1
}

/// `BKE_mask_spline_differentiate_with_resolution` — returns flat
/// `[x,y,...]` diff points.
pub fn bke_mask_spline_differentiate_with_resolution(
    spline: &MaskSpline,
    resol: u32,
) -> Vec<f32> {
    let tot = bke_mask_spline_differentiate_calc_total(spline, resol);
    if spline.tot_point <= 1 {
        return Vec::new();
    }
    let mut diff_points = vec![0.0f32; 2 * (tot + 1)];
    let mut a = spline.tot_point as usize - 1;
    if spline.flag & MASK_SPLINE_CYCLIC != 0 {
        a += 1;
    }
    let mut fp = 0usize;
    let mut point_prev = 0usize;
    let mut point_curr = 1usize;
    while a > 0 {
        a -= 1;
        if a == 0 && spline.flag & MASK_SPLINE_CYCLIC != 0 {
            point_curr = 0;
        }
        let bezt_prev = &spline.points[point_prev].bezt;
        let bezt_curr = &spline.points[point_curr].bezt;
        for j in 0..2 {
            bke_curve_forward_diff_bezier(
                bezt_prev.vec[1][j],
                bezt_prev.vec[2][j],
                bezt_curr.vec[0][j],
                bezt_curr.vec[1][j],
                &mut diff_points,
                fp + j,
                resol as i32,
                2,
            );
        }
        fp += 2 * resol as usize;
        if a == 0 && spline.flag & MASK_SPLINE_CYCLIC == 0 {
            diff_points[fp] = bezt_curr.vec[1][0];
            diff_points[fp + 1] = bezt_curr.vec[1][1];
        }
        point_prev = point_curr;
        point_curr += 1;
    }
    diff_points
}

/// `BKE_mask_spline_differentiate`.
pub fn bke_mask_spline_differentiate(
    spline: &MaskSpline,
    width: i32,
    height: i32,
) -> Vec<f32> {
    let resol = bke_mask_spline_resolution(spline, width, height);
    bke_mask_spline_differentiate_with_resolution(spline, resol as u32)
}

/// `FeatherEdgesBucket`.
#[derive(Default)]
struct FeatherEdgesBucket {
    /// `segments` — (start, end) pairs.
    segments: Vec<[i32; 2]>,
}

fn feather_bucket_check_intersect(
    feather_points: &mut [Vec2],
    bucket: &FeatherEdgesBucket,
    cur_a: i32,
    cur_b: i32,
) {
    let tot = feather_points.len() as i32;
    let v1 = feather_points[cur_a as usize];
    let v2 = feather_points[cur_b as usize];
    for seg in &bucket.segments {
        let check_a = seg[0];
        let check_b = seg[1];
        let v3 = feather_points[check_a as usize];
        let v4 = feather_points[check_b as usize];
        if check_a >= cur_a - 1 || cur_b == check_a {
            continue;
        }
        if let Some(p) = isect_seg_seg_v2(v1, v2, v3, v4) {
            let mut min_a = Vec2::splat(f32::MAX);
            let mut max_a = Vec2::splat(f32::MIN);
            let mut min_b = min_a;
            let mut max_b = max_a;
            for k in 0..tot {
                let fp = feather_points[k as usize];
                if k >= check_b && k <= cur_a {
                    min_a = min_a.min(fp);
                    max_a = max_a.max(fp);
                } else {
                    min_b = min_b.min(fp);
                    max_b = max_b.max(fp);
                }
            }
            if max_a.x - min_a.x < max_b.x - min_b.x || max_a.y - min_a.y < max_b.y - min_b.y {
                for k in check_b..=cur_a {
                    feather_points[k as usize] = p;
                }
            } else {
                for k in 0..=check_a {
                    feather_points[k as usize] = p;
                }
                if cur_b != 0 {
                    for k in cur_b..tot {
                        feather_points[k as usize] = p;
                    }
                }
            }
        }
    }
}

/// `isect_seg_seg_v2_simple` + `isect_seg_seg_v2_point` → Option.
fn isect_seg_seg_v2(v1: Vec2, v2: Vec2, v3: Vec2, v4: Vec2) -> Option<Vec2> {
    let div = (v2.x - v1.x) * (v4.y - v3.y) - (v2.y - v1.y) * (v4.x - v3.x);
    if div.abs() < f32::EPSILON {
        return None;
    }
    let labda = ((v1.y - v3.y) * (v4.x - v3.x) - (v1.x - v3.x) * (v4.y - v3.y)) / -div;
    let mu = ((v1.y - v3.y) * (v2.x - v1.x) - (v1.x - v3.x) * (v2.y - v1.y)) / -div;
    if labda >= 0.0 && labda <= 1.0 && mu >= 0.0 && mu <= 1.0 {
        Some(v1 + (v2 - v1) * labda)
    } else {
        None
    }
}

fn feather_bucket_index_from_coord(
    co: Vec2,
    min: Vec2,
    bucket_scale: Vec2,
    buckets_per_side: i32,
) -> i32 {
    let mut x = ((co.x - min.x) * bucket_scale.x) as i32;
    let mut y = ((co.y - min.y) * bucket_scale.y) as i32;
    if x == buckets_per_side {
        x -= 1;
    }
    if y == buckets_per_side {
        y -= 1;
    }
    y * buckets_per_side + x
}

fn feather_bucket_get_diagonal(
    start_bucket_index: i32,
    end_bucket_index: i32,
    buckets_per_side: i32,
) -> (i32, i32) {
    let sx = start_bucket_index % buckets_per_side;
    let sy = start_bucket_index / buckets_per_side;
    let ex = end_bucket_index % buckets_per_side;
    let ey = end_bucket_index / buckets_per_side;
    (
        sy * buckets_per_side + ex,
        ey * buckets_per_side + sx,
    )
}

/// `BKE_mask_spline_feather_collapse_inner_loops`.
pub fn bke_mask_spline_feather_collapse_inner_loops(
    spline: &MaskSpline,
    feather_points: &mut [Vec2],
) {
    let tot = feather_points.len();
    if tot < 4 {
        return;
    }
    let mut min = Vec2::splat(f32::MAX);
    let mut max = Vec2::splat(f32::MIN);
    let mut max_delta_x = -1.0f32;
    let mut max_delta_y = -1.0f32;
    for i in 0..tot {
        min = min.min(feather_points[i]);
        max = max.max(feather_points[i]);
        let next = if i + 1 == tot {
            if spline.flag & MASK_SPLINE_CYCLIC != 0 {
                0
            } else {
                break;
            }
        } else {
            i + 1
        };
        max_delta_x = max_delta_x.max((feather_points[i].x - feather_points[next].x).abs());
        max_delta_y = max_delta_y.max((feather_points[i].y - feather_points[next].y).abs());
    }
    if max.x - min.x < f32::EPSILON {
        max.x += 0.01;
        min.x -= 0.01;
    }
    if max.y - min.y < f32::EPSILON {
        max.y += 0.01;
        min.y -= 0.01;
    }
    max_delta_x /= max.x - min.x;
    max_delta_y /= max.y - min.y;
    let max_delta = max_delta_x.max(max_delta_y);

    let mut buckets_per_side = (512.0f32).min(0.9 / max_delta) as i32;
    if buckets_per_side == 0 {
        buckets_per_side = 1;
    }
    let tot_bucket = (buckets_per_side * buckets_per_side) as usize;
    let bucket_size = 1.0 / buckets_per_side as f32;
    let bucket_scale = Vec2::new(
        1.0 / ((max.x - min.x) * bucket_size),
        1.0 / ((max.y - min.y) * bucket_size),
    );
    let mut buckets: Vec<FeatherEdgesBucket> =
        (0..tot_bucket).map(|_| FeatherEdgesBucket::default()).collect();
    let bucket_index =
        |co: Vec2| feather_bucket_index_from_coord(co, min, bucket_scale, buckets_per_side);

    for i in 0..tot {
        let start = i as i32;
        let mut end = i as i32 + 1;
        if end as usize == tot {
            if spline.flag & MASK_SPLINE_CYCLIC != 0 {
                end = 0;
            } else {
                break;
            }
        }
        let sbi = bucket_index(feather_points[start as usize]);
        let ebi = bucket_index(feather_points[end as usize]);
        buckets[sbi as usize].segments.push([start, end]);
        if sbi != ebi {
            let (da, db) = feather_bucket_get_diagonal(sbi, ebi, buckets_per_side);
            buckets[ebi as usize].segments.push([start, end]);
            buckets[da as usize].segments.push([start, end]);
            buckets[db as usize].segments.push([start, end]);
        }
    }

    for i in 0..tot {
        let cur_a = i as i32;
        let mut cur_b = i as i32 + 1;
        if cur_b as usize == tot {
            cur_b = 0;
        }
        let sbi = bucket_index(feather_points[cur_a as usize]);
        let ebi = bucket_index(feather_points[cur_b as usize]);
        /* Split buckets out to satisfy the borrow rules. */
        let start_bucket = std::mem::take(&mut buckets[sbi as usize]);
        feather_bucket_check_intersect(feather_points, &start_bucket, cur_a, cur_b);
        buckets[sbi as usize] = start_bucket;
        if sbi != ebi {
            let (da, db) = feather_bucket_get_diagonal(sbi, ebi, buckets_per_side);
            for bi in [ebi, da, db] {
                let b = std::mem::take(&mut buckets[bi as usize]);
                feather_bucket_check_intersect(feather_points, &b, cur_a, cur_b);
                buckets[bi as usize] = b;
            }
        }
    }
}

fn dist_ensure_v2_v2fl(v: &mut Vec2, to: Vec2, dist: f32) {
    let d = *v - to;
    let l = d.length();
    if l > 0.0 {
        *v = to + d * (dist / l);
    }
}

/// `mask_spline_feather_differentiated_points_with_resolution__even`.
fn feather_diff_even(spline: &MaskSpline, resol: u32, do_feather_isect: bool) -> Vec<Vec2> {
    let tot = bke_mask_spline_differentiate_calc_total(spline, resol);
    let mut feather = vec![Vec2::ZERO; tot + 1];
    let mut a = spline.tot_point as usize - 1;
    if spline.flag & MASK_SPLINE_CYCLIC != 0 {
        a += 1;
    }
    let mut fp = 0usize;
    let mut point_prev = 0usize;
    let mut point_curr = 1usize;
    while a > 0 {
        a -= 1;
        if a == 0 && spline.flag & MASK_SPLINE_CYCLIC != 0 {
            point_curr = 0;
        }
        for j in 0..resol {
            let u = j as f32 / resol as f32;
            let co = bke_mask_point_segment_co(spline, point_prev, u);
            let n = bke_mask_point_normal(spline, point_prev, u);
            let weight = bke_mask_point_weight(spline, point_prev, u);
            feather[fp] = co + n * weight;
            fp += 1;
        }
        if a == 0 && spline.flag & MASK_SPLINE_CYCLIC == 0 {
            let u = 1.0f32;
            let co = bke_mask_point_segment_co(spline, point_prev, u);
            let n = bke_mask_point_normal(spline, point_prev, u);
            let weight = bke_mask_point_weight(spline, point_prev, u);
            feather[fp] = co + n * weight;
        }
        point_prev = point_curr;
        point_curr += 1;
    }
    feather.truncate(tot);
    if spline.flag & MASK_SPLINE_NOINTERSECT != 0 && do_feather_isect {
        bke_mask_spline_feather_collapse_inner_loops(spline, &mut feather);
    }
    feather
}

/// `mask_spline_feather_differentiated_points_with_resolution__double`.
fn feather_diff_double(spline: &MaskSpline, resol: u32, do_feather_isect: bool) -> Vec<Vec2> {
    let tot = bke_mask_spline_differentiate_calc_total(spline, resol);
    if spline.tot_point <= 1 {
        return Vec::new();
    }
    let mut feather = vec![Vec2::ZERO; tot + 1];
    let mut a = spline.tot_point as usize - 1;
    if spline.flag & MASK_SPLINE_CYCLIC != 0 {
        a += 1;
    }
    let mut fp = 0usize;
    let mut point_prev = 0usize;
    let mut point_curr = 1usize;
    while a > 0 {
        a -= 1;
        if a == 0 && spline.flag & MASK_SPLINE_CYCLIC != 0 {
            point_curr = 0;
        }
        let mut bezt_prev = spline.points[point_prev].bezt.clone();
        let mut bezt_curr = spline.points[point_curr].bezt.clone();

        let tvec = (bzt2(&bezt_prev, 1) - bzt2(&bezt_prev, 0)).normalized();
        let mut point_prev_n = Vec2::new(-tvec.y, tvec.x);
        let tvec = (bzt2(&bezt_curr, 1) - bzt2(&bezt_curr, 0)).normalized();
        let mut point_curr_n = Vec2::new(-tvec.y, tvec.x);

        let weight_prev = bezt_prev.weight;
        let weight_curr = bezt_curr.weight;
        point_prev_n *= weight_prev;
        point_curr_n *= weight_curr;

        let len_base = (bzt2(&bezt_prev, 1) - bzt2(&bezt_curr, 1)).length();
        bezt_prev.vec[1][0] += point_prev_n.x;
        bezt_prev.vec[1][1] += point_prev_n.y;
        bezt_prev.vec[2][0] += point_prev_n.x;
        bezt_prev.vec[2][1] += point_prev_n.y;
        bezt_curr.vec[0][0] += point_curr_n.x;
        bezt_curr.vec[0][1] += point_curr_n.y;
        bezt_curr.vec[1][0] += point_curr_n.x;
        bezt_curr.vec[1][1] += point_curr_n.y;

        let len_feather = (bzt2(&bezt_prev, 1) - bzt2(&bezt_curr, 1)).length();
        let len_scalar = if len_base != 0.0 { len_feather / len_base } else { 1.0 };
        let mut v = bzt2(&bezt_prev, 2);
        dist_ensure_v2_v2fl(
            &mut v,
            bzt2(&bezt_prev, 1),
            len_scalar * (bzt2(&bezt_prev, 2) - bzt2(&bezt_prev, 1)).length(),
        );
        bezt_prev.vec[2][0] = v.x;
        bezt_prev.vec[2][1] = v.y;
        let mut v = bzt2(&bezt_curr, 0);
        dist_ensure_v2_v2fl(
            &mut v,
            bzt2(&bezt_curr, 1),
            len_scalar * (bzt2(&bezt_curr, 0) - bzt2(&bezt_curr, 1)).length(),
        );
        bezt_curr.vec[0][0] = v.x;
        bezt_curr.vec[0][1] = v.y;

        /* forward-diff the feathered bezier. */
        let mut buf = vec![0.0f32; 2 * ((resol + 1) as usize)];
        for j in 0..2 {
            bke_curve_forward_diff_bezier(
                bezt_prev.vec[1][j],
                bezt_prev.vec[2][j],
                bezt_curr.vec[0][j],
                bezt_curr.vec[1][j],
                &mut buf,
                j,
                resol as i32,
                2,
            );
        }
        for j in 0..resol as usize {
            feather[fp + j] = Vec2::new(buf[2 * j], buf[2 * j + 1]);
        }

        if spline.points[point_prev].tot_uw > 0 {
            for j in 0..resol {
                let u = j as f32 / resol as f32;
                let co = bke_mask_point_segment_co(spline, point_prev, u);
                let weight_uw = bke_mask_point_weight(spline, point_prev, u);
                let weight_scalar = bke_mask_point_weight_scalar(spline, point_prev, u);
                let mut fpv = feather[fp];
                dist_ensure_v2_v2fl(
                    &mut fpv,
                    co,
                    (feather[fp] - co).length() * (weight_uw / weight_scalar),
                );
                feather[fp] = fpv;
                fp += 1;
            }
        } else {
            fp += resol as usize;
        }

        if a == 0 && spline.flag & MASK_SPLINE_CYCLIC == 0 {
            feather[fp] = bzt2(&bezt_curr, 1);
        }
        point_prev = point_curr;
        point_curr += 1;
    }
    feather.truncate(tot);
    if spline.flag & MASK_SPLINE_NOINTERSECT != 0 && do_feather_isect {
        bke_mask_spline_feather_collapse_inner_loops(spline, &mut feather);
    }
    feather
}

/// `mask_spline_feather_differentiated_points_with_resolution`.
pub fn mask_spline_feather_differentiated_points_with_resolution(
    spline: &MaskSpline,
    resol: u32,
    do_feather_isect: bool,
) -> Vec<Vec2> {
    match spline.offset_mode {
        m if m == MASK_SPLINE_OFFSET_EVEN => feather_diff_even(spline, resol, do_feather_isect),
        _ => feather_diff_double(spline, resol, do_feather_isect),
    }
}

/// `BKE_mask_spline_feather_points`.
pub fn bke_mask_spline_feather_points(spline: &MaskSpline) -> Vec<Vec2> {
    let mut feather = Vec::new();
    for (i, point) in spline.points.iter().enumerate() {
        let n = bke_mask_point_normal(spline, i, 0.0);
        let weight = bke_mask_point_weight(spline, i, 0.0);
        feather.push(bzt2(&point.bezt, 1) + n * weight);
        for uw in &point.uw {
            let co = bke_mask_point_segment_co(spline, i, uw.u);
            let n = bke_mask_point_normal(spline, i, uw.u);
            let weight = bke_mask_point_weight(spline, i, uw.u);
            feather.push(co + n * weight);
        }
    }
    feather
}

/// `BKE_mask_point_segment_feather_diff`.
pub fn bke_mask_point_segment_feather_diff(
    spline: &MaskSpline,
    point_index: usize,
    width: i32,
    height: i32,
) -> Vec<Vec2> {
    let resol = bke_mask_spline_feather_resolution(spline, width, height) as u32;
    let mut feather = Vec::with_capacity(resol as usize);
    for i in 0..resol {
        let u = i as f32 / resol as f32;
        let co = bke_mask_point_segment_co(spline, point_index, u);
        let n = bke_mask_point_normal(spline, point_index, u);
        let weight = bke_mask_point_weight(spline, point_index, u);
        feather.push(co + n * weight);
    }
    feather
}

/// `BKE_mask_point_segment_diff`.
pub fn bke_mask_point_segment_diff(
    spline: &MaskSpline,
    point_index: usize,
    width: i32,
    height: i32,
) -> Vec<Vec2> {
    let bezt = &spline.points[point_index].bezt;
    let Some(next) = next_bezt_index(spline, point_index) else {
        return Vec::new();
    };
    let bezt_next = &spline.points[next].bezt;
    let resol = bke_mask_spline_resolution(spline, width, height) as usize;
    let mut diff = vec![0.0f32; 2 * (resol + 1)];
    for j in 0..2 {
        bke_curve_forward_diff_bezier(
            bezt.vec[1][j],
            bezt.vec[2][j],
            bezt_next.vec[0][j],
            bezt_next.vec[1][j],
            &mut diff,
            j,
            resol as i32,
            2,
        );
    }
    diff[2 * resol] = bezt_next.vec[1][0];
    diff[2 * resol + 1] = bezt_next.vec[1][1];
    diff.chunks_exact(2).map(|c| Vec2::new(c[0], c[1])).collect()
}

/// `mask_evaluate_apply_point_parent` — `parent_matrix` injected
/// (`BKE_mask_point_parent_matrix_get`).
pub fn mask_evaluate_apply_point_parent(point: &mut MaskSplinePoint, parent_matrix: [[f32; 3]; 3]) {
    for i in 0..3 {
        let v = point.bezt.vec[i];
        point.bezt.vec[i][0] =
            parent_matrix[0][0] * v[0] + parent_matrix[0][1] * v[1] + parent_matrix[0][2];
        point.bezt.vec[i][1] =
            parent_matrix[1][0] * v[0] + parent_matrix[1][1] * v[1] + parent_matrix[1][2];
    }
}

/// `BKE_mask_layer_evaluate_deform` — per-point: copy uw, apply
/// parent matrix (injected), recalc auto/vect handles via
/// `calc_handle` (`BKE_mask_calc_handle_point`).
pub fn bke_mask_layer_evaluate_deform(
    splines: &mut [MaskSpline],
    points_deform: &mut [Vec<MaskSplinePoint>],
    parent_matrix: &dyn Fn(&MaskSplinePoint, f32) -> [[f32; 3]; 3],
    calc_handle: &mut dyn FnMut(&MaskSpline, &mut MaskSplinePoint, usize),
    ctime: f32,
) {
    for (si, spline) in splines.iter_mut().enumerate() {
        let deform = &mut points_deform[si];
        deform.resize(spline.tot_point as usize, MaskSplinePoint::default());
        let mut need_handle_recalc = false;
        for i in 0..spline.tot_point as usize {
            let point_deform = &mut deform[i];
            point_deform.uw = spline.points[i].uw.clone();
            point_deform.bezt = spline.points[i].bezt.clone();
            point_deform.parent = spline.points[i].parent.clone();
            let m = parent_matrix(&spline.points[i], ctime);
            mask_evaluate_apply_point_parent(point_deform, m);
            if point_deform.bezt.h1 == HD_AUTO || point_deform.bezt.h1 == HD_VECT {
                need_handle_recalc = true;
            }
        }
        if need_handle_recalc {
            for i in 0..spline.tot_point as usize {
                if deform[i].bezt.h1 == HD_AUTO || deform[i].bezt.h1 == HD_VECT {
                    calc_handle(spline, &mut deform[i], i);
                }
            }
        }
    }
}
