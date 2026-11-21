//! Port of `intern/colortools.cc` — the `CurveMapping` engine
//! (defaults/add/copy/free, point remove/insert/reset presets,
//! `calchandle_curvemap`, `curvemap_make_table` bezier→table,
//! evaluate + premultiply + film-like tone + range/slope helpers),
//! `BKE_histogram_update_sample_line`, `BKE_scopes_*` (serial
//! update with injected pixel/colormanagement hooks),
//! `BKE_color_managed_*_settings_{init,copy,free}` and
//! `BKE_curvemapping_equals`/`colorspace_settings_*`.
//! `PORT_PENDING`: `*_blend_write`/`_read` IO, `Scopes`
//! `sample_lines` parallel TLS merge (serial port here).

use blenders_dna::color_types::{
    CurveMap, CurveMapPoint, CurveMapping, Histogram, Scopes, CM_RESOL, CM_TABLE, CM_TABLEDIV,
    CUMA_ACTIVE, CUMA_DO_CLIP, CUMA_EXTEND_EXTRAPOLATE, CUMA_HANDLE_AUTO_ANIM,
    CUMA_HANDLE_VECTOR, CUMA_PREMULLED, CUMA_REMOVE, CUMA_SELECT, CUMA_USE_WRAPPING,
    CURVE_PRESET_BELL, CURVE_PRESET_CONSTANT_MEDIAN, CURVE_PRESET_GAUSS, CURVE_PRESET_LINE,
    CURVE_PRESET_MAX, CURVE_PRESET_MID8, CURVE_PRESET_ROOT, CURVE_PRESET_ROUND,
    CURVE_PRESET_SHARP, CURVE_PRESET_SMOOTH, CURVE_TONE_FILMLIKE,
};
use blenders_dna::curve_enums::{HD_AUTO, HD_AUTO_ANIM, HD_VECT};
use blenders_dna::curve_types::BezTriple;
use blenders_dna::vec_types::Rctf;
use blenders_math::color::{hsv_to_rgb, rgb_to_hsv, rgb_to_ycc};

use crate::mask_evaluate::bke_curve_forward_diff_bezier;

/// `CM_TOT` — 4 curve maps (RGB + combined).
pub const CM_TOT: usize = 4;
/// `CurveMapSlopeType` — `Negative`(default) `Positive` `PositiveNegative`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CurveMapSlopeType {
    /// Downward slope.
    Negative,
    /// Upward.
    Positive,
    /// Both halves (symmetric).
    PositiveNegative,
}

/// `eBezTriple_Handle` used by `set_defaults` — `HD_VECT` →
/// `CUMA_HANDLE_VECTOR`, `HD_AUTO_ANIM` → `CUMA_HANDLE_AUTO_ANIM`.
pub fn set_defaults(
    cumap: &mut CurveMapping,
    tot: usize,
    minx: f32,
    miny: f32,
    maxx: f32,
    maxy: f32,
    default_handle_type: u8,
) {
    cumap.flag = CUMA_DO_CLIP | CUMA_EXTEND_EXTRAPOLATE;
    if tot == 4 {
        cumap.cur = 3;
    }
    let clipminx = minx.min(maxx);
    let clipminy = miny.min(maxy);
    let clipmaxx = minx.max(maxx);
    let clipmaxy = miny.max(maxy);
    cumap.curr = Rctf { xmin: clipminx, xmax: clipmaxx, ymin: clipminy, ymax: clipmaxy };
    cumap.clipr = cumap.curr;
    cumap.white = [1.0; 3];
    cumap.bwmul = [1.0; 3];
    for a in 0..tot {
        cumap.cm[a].default_handle_type = if default_handle_type == HD_VECT {
            CUMA_HANDLE_VECTOR
        } else if default_handle_type == HD_AUTO_ANIM {
            CUMA_HANDLE_AUTO_ANIM
        } else {
            0
        };
        cumap.cm[a].totpoint = 2;
        let h = cumap.cm[a].default_handle_type;
        cumap.cm[a].curve = vec![
            CurveMapPoint { x: minx, y: miny, flag: h, shorty: 0 },
            CurveMapPoint { x: maxx, y: maxy, flag: h, shorty: 0 },
        ];
    }
    cumap.changed_timestamp = 0;
}

/// `BKE_curvemapping_add`.
pub fn curvemapping_add(tot: usize, minx: f32, miny: f32, maxx: f32, maxy: f32) -> CurveMapping {
    let mut cumap = CurveMapping::default();
    set_defaults(&mut cumap, tot, minx, miny, maxx, maxy, 0);
    cumap
}

/// `BKE_curvemapping_free_data_single` — clear one curve's arrays.
pub fn free_data_single(cumap: &mut CurveMapping, index: usize) {
    let cm = &mut cumap.cm[index];
    cm.curve.clear();
    cm.table.clear();
    cm.premultable.clear();
}

/// `BKE_curvemapping_free_data`.
pub fn free_data(cumap: &mut CurveMapping) {
    for i in 0..CM_TOT {
        free_data_single(cumap, i);
    }
}

/// `BKE_curvemapping_copy_data_single`.
pub fn copy_data_single(target: &mut CurveMapping, src: &CurveMapping, index: usize) {
    let s = &src.cm[index];
    let t = &mut target.cm[index];
    *t = s.clone();
}

/// `BKE_curvemapping_copy_data`.
pub fn copy_data(target: &mut CurveMapping, src: &CurveMapping) {
    target.flag = src.flag;
    target.cur = src.cur;
    target.changed_timestamp = src.changed_timestamp;
    target.preset = src.preset;
    target.curr = src.curr;
    target.clipr = src.clipr;
    target.black = src.black;
    target.white = src.white;
    target.bwmul = src.bwmul;
    target.sample = src.sample;
    for i in 0..CM_TOT {
        copy_data_single(target, src, i);
    }
}

/// `BKE_curvemapping_copy`.
pub fn curvemapping_copy(src: &CurveMapping) -> CurveMapping {
    let mut target = CurveMapping::default();
    copy_data(&mut target, src);
    target
}

/// `BKE_curvemapping_set_black_white_ex`.
pub fn set_black_white_ex(black: [f32; 3], white: [f32; 3]) -> [f32; 3] {
    let mut bwmul = [0.0; 3];
    for a in 0..3 {
        let delta = (white[a] - black[a]).max(1e-5);
        bwmul[a] = 1.0 / delta;
    }
    bwmul
}

/// `BKE_curvemapping_set_black_white`.
pub fn set_black_white(
    cumap: &mut CurveMapping,
    black: Option<[f32; 3]>,
    white: Option<[f32; 3]>,
) {
    if let Some(w) = white {
        cumap.white = w;
    }
    if let Some(b) = black {
        cumap.black = b;
    }
    cumap.bwmul = set_black_white_ex(cumap.black, cumap.white);
    cumap.changed_timestamp += 1;
}

/// `BKE_curvemap_remove_point` — remove by index; needs ≥3 pts.
pub fn curvemap_remove_point(cuma: &mut CurveMap, index: usize) -> bool {
    if cuma.totpoint <= 2 {
        return false;
    }
    let before = cuma.curve.len();
    cuma.curve.remove(index);
    let removed = before - cuma.curve.len();
    cuma.totpoint -= removed as i16;
    removed != 0
}

/// `BKE_curvemap_remove` — remove points with `flag`, keep outer.
pub fn curvemap_remove(cuma: &mut CurveMap, flag: i16) {
    let mut cmp = Vec::with_capacity(cuma.curve.len());
    let mut removed = 0;
    cmp.push(cuma.curve[0]);
    for i in 1..cuma.totpoint as usize - 1 {
        if cuma.curve[i].flag & flag == 0 {
            cmp.push(cuma.curve[i]);
        } else {
            removed += 1;
        }
    }
    cmp.push(cuma.curve[cuma.totpoint as usize - 1]);
    cuma.curve = cmp;
    cuma.totpoint -= removed;
}

/// `BKE_curvemap_insert` — sorted insert, returns new index.
pub fn curvemap_insert(cuma: &mut CurveMap, x: f32, y: f32) -> usize {
    cuma.totpoint += 1;
    let mut new_path = Vec::with_capacity(cuma.totpoint as usize);
    let mut new_pt = 0usize;
    let mut found = false;
    let mut i_old = 0;
    for i_new in 0..cuma.totpoint as usize {
        if !found && (i_new + 1 == cuma.totpoint as usize || x < cuma.curve[i_new].x) {
            new_path.push(CurveMapPoint {
                x,
                y,
                flag: CUMA_SELECT | cuma.default_handle_type,
                shorty: 0,
            });
            found = true;
            new_pt = i_new;
        } else {
            let old = cuma.curve[i_old];
            new_path.push(CurveMapPoint {
                x: old.x,
                y: old.y,
                flag: old.flag & !(CUMA_SELECT | CUMA_ACTIVE),
                shorty: old.shorty,
            });
            i_old += 1;
        }
    }
    cuma.curve = new_path;
    new_pt
}

/// `BKE_curvemap_reset` — preset shapes; `slope` mirrors/
/// symmetrizes.
pub fn curvemap_reset(cuma: &mut CurveMap, clipr: &Rctf, preset: i32, slope: CurveMapSlopeType) {
    cuma.curve.clear();
    cuma.totpoint = match preset {
        CURVE_PRESET_LINE | CURVE_PRESET_CONSTANT_MEDIAN | CURVE_PRESET_MAX => 2,
        CURVE_PRESET_SHARP => 5,
        CURVE_PRESET_SMOOTH | CURVE_PRESET_MID8 => 8,
        CURVE_PRESET_ROUND | CURVE_PRESET_ROOT => 6,
        CURVE_PRESET_GAUSS => 7,
        CURVE_PRESET_BELL => 3,
        _ => 2,
    };
    cuma.curve =
        vec![CurveMapPoint { x: 0.0, y: 0.0, flag: cuma.default_handle_type, shorty: 0 };
             cuma.totpoint as usize];
    let c = &mut cuma.curve;
    match preset {
        CURVE_PRESET_LINE => {
            c[0].x = clipr.xmin;
            c[0].y = clipr.ymax;
            c[1].x = clipr.xmax;
            c[1].y = clipr.ymin;
            if slope == CurveMapSlopeType::PositiveNegative {
                for i in 0..2 {
                    c[i].flag &= !CUMA_HANDLE_AUTO_ANIM;
                    c[i].flag |= CUMA_HANDLE_VECTOR;
                }
            }
        }
        CURVE_PRESET_CONSTANT_MEDIAN => {
            c[0].x = clipr.xmin;
            c[0].y = (clipr.ymin + clipr.ymax) / 2.0;
            c[1].x = clipr.xmax;
            c[1].y = (clipr.ymin + clipr.ymax) / 2.0;
        }
        CURVE_PRESET_SHARP => {
            let pts = [(0.0, 1.0), (0.25, 0.5625), (0.5, 0.25), (0.75, 0.0625), (1.0, 0.0)];
            for (i, &(x, y)) in pts.iter().enumerate() {
                c[i].x = x;
                c[i].y = y;
            }
            if slope == CurveMapSlopeType::PositiveNegative {
                c[0].flag = c[0].flag & !CUMA_HANDLE_AUTO_ANIM | CUMA_HANDLE_VECTOR;
                c[4].flag = c[4].flag & !CUMA_HANDLE_AUTO_ANIM | CUMA_HANDLE_VECTOR;
            }
        }
        CURVE_PRESET_SMOOTH => {
            let pts = [
                (0.0, 1.0),
                (0.12, 0.96),
                (0.25, 0.84),
                (0.42, 0.62),
                (0.58, 0.38),
                (0.75, 0.16),
                (0.88, 0.04),
                (1.0, 0.0),
            ];
            for (i, &(x, y)) in pts.iter().enumerate() {
                c[i].x = x;
                c[i].y = y;
            }
        }
        CURVE_PRESET_MAX => {
            c[0].x = 0.0;
            c[0].y = 1.0;
            c[1].x = 1.0;
            c[1].y = 1.0;
        }
        CURVE_PRESET_MID8 => {
            for i in 0..cuma.totpoint as usize {
                c[i].x = i as f32 / cuma.totpoint as f32;
                c[i].y = 0.5;
            }
        }
        CURVE_PRESET_ROUND => {
            let pts = [
                (0.0, 1.0),
                (0.5, 0.866),
                (0.6765, 0.7364),
                (0.8582, 0.5133),
                (0.967, 0.2547),
                (1.0, 0.0),
            ];
            for (i, &(x, y)) in pts.iter().enumerate() {
                c[i].x = x;
                c[i].y = y;
            }
        }
        CURVE_PRESET_ROOT => {
            let pts = [(0.0, 1.0), (0.25, 0.866), (0.5, 0.707), (0.75, 0.5), (0.9375, 0.25), (1.0, 0.0)];
            for (i, &(x, y)) in pts.iter().enumerate() {
                c[i].x = x;
                c[i].y = y;
            }
            if slope == CurveMapSlopeType::PositiveNegative {
                c[0].flag = c[0].flag & !CUMA_HANDLE_AUTO_ANIM | CUMA_HANDLE_VECTOR;
                c[5].flag = c[5].flag & !CUMA_HANDLE_AUTO_ANIM | CUMA_HANDLE_VECTOR;
            }
        }
        CURVE_PRESET_GAUSS => {
            let pts = [
                (0.0, 0.025),
                (0.16, 0.135),
                (0.298, 0.36),
                (0.50, 1.0),
                (0.70, 0.36),
                (0.84, 0.135),
                (1.0, 0.025),
            ];
            for (i, &(x, y)) in pts.iter().enumerate() {
                c[i].x = x;
                c[i].y = y;
            }
        }
        CURVE_PRESET_BELL => {
            let pts = [(0.0, 0.025), (0.50, 1.0), (1.0, 0.025)];
            for (i, &(x, y)) in pts.iter().enumerate() {
                c[i].x = x;
                c[i].y = y;
            }
        }
        _ => {}
    }

    if slope == CurveMapSlopeType::Positive {
        if preset == CURVE_PRESET_LINE || preset == CURVE_PRESET_CONSTANT_MEDIAN {
            cuma.curve.swap(0, 1);
            /* swap y only (x order kept) */
            cuma.curve.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
            let (y0, y1) = (cuma.curve[0].y, cuma.curve[1].y);
            cuma.curve[0].y = y1;
            cuma.curve[1].y = y0;
        } else if preset != CURVE_PRESET_MID8 {
            let last = cuma.totpoint as usize - 1;
            let mut newpoints = Vec::with_capacity(cuma.totpoint as usize);
            for i in 0..cuma.totpoint as usize {
                let mut p = cuma.curve[last - i];
                p.x = 1.0 - p.x;
                newpoints.push(p);
            }
            cuma.curve = newpoints;
        }
    } else if slope == CurveMapSlopeType::PositiveNegative {
        let num_points = cuma.totpoint as usize * 2 - 1;
        let mut new_points = vec![CurveMapPoint::default(); num_points];
        for i in 0..cuma.totpoint as usize {
            let src_last = cuma.totpoint as usize - i - 1;
            let dst_last = num_points - i - 1;
            new_points[i] = cuma.curve[src_last];
            new_points[i].x = (1.0 - cuma.curve[src_last].x) * 0.5;
            new_points[dst_last] = new_points[i];
            new_points[dst_last].x = 0.5 + cuma.curve[src_last].x * 0.5;
        }
        cuma.totpoint = num_points as i16;
        cuma.curve = new_points;
    }
    cuma.table.clear();
}

/// `BKE_curvemap_activate_nearest_point` — activate nearest
/// selected point around `i_last`.
pub fn curvemap_activate_nearest_point(cuma: &mut CurveMap, i_last: i32) {
    let n = cuma.totpoint as i32;
    for i in 1.. {
        let k = (i + 1) / 2;
        let idx = if i & 1 == 1 { i_last - k } else { i_last + k };
        if !(0..n).contains(&idx) {
            if i_last - k < 0 && i_last + k >= n {
                return;
            }
            continue;
        }
        let p = &mut cuma.curve[idx as usize];
        if p.flag & CUMA_SELECT != 0 {
            p.flag |= CUMA_ACTIVE;
            return;
        }
    }
}

/// `BKE_curvemap_handle_set` — set handle type on selected pts.
pub fn curvemap_handle_set(cuma: &mut CurveMap, type_: u8) {
    for p in &mut cuma.curve {
        if p.flag & CUMA_SELECT != 0 {
            p.flag &= !(CUMA_HANDLE_VECTOR | CUMA_HANDLE_AUTO_ANIM);
            if type_ == HD_VECT {
                p.flag |= CUMA_HANDLE_VECTOR;
            } else if type_ == HD_AUTO_ANIM {
                p.flag |= CUMA_HANDLE_AUTO_ANIM;
            }
        }
    }
}

fn sub_v2(a: [f32; 3], b: [f32; 3]) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

/// `calchandle_curvemap` — auto/vector bezier handles.
pub fn calchandle_curvemap(bezt: &mut BezTriple, prev: Option<&BezTriple>, next: Option<&BezTriple>) {
    if bezt.h1 == 0 && bezt.h2 == 0 {
        return;
    }
    let p2 = bezt.vec[1];
    let mut pt = [0.0f32; 3];
    let p1: [f32; 3] = match prev {
        None => {
            let p3n = next.unwrap().vec[1];
            pt[0] = 2.0 * p2[0] - p3n[0];
            pt[1] = 2.0 * p2[1] - p3n[1];
            pt
        }
        Some(pr) => pr.vec[1],
    };
    let p3: [f32; 3] = match next {
        None => {
            let p1p = prev.unwrap().vec[1];
            pt[0] = 2.0 * p2[0] - p1p[0];
            pt[1] = 2.0 * p2[1] - p1p[1];
            pt
        }
        Some(nx) => nx.vec[1],
    };
    let dvec_a = sub_v2(p2, p1);
    let dvec_b = sub_v2(p3, p2);
    let mut len_a = (dvec_a[0] * dvec_a[0] + dvec_a[1] * dvec_a[1]).sqrt();
    let mut len_b = (dvec_b[0] * dvec_b[0] + dvec_b[1] * dvec_b[1]).sqrt();
    if len_a == 0.0 {
        len_a = 1.0;
    }
    if len_b == 0.0 {
        len_b = 1.0;
    }
    let auto = |h: u8| h == HD_AUTO || h == HD_AUTO_ANIM;
    if auto(bezt.h1) || auto(bezt.h2) {
        let tvec = [dvec_b[0] / len_b + dvec_a[0] / len_a, dvec_b[1] / len_b + dvec_a[1] / len_a];
        let len = (tvec[0] * tvec[0] + tvec[1] * tvec[1]).sqrt() * 2.5614;
        if len != 0.0 {
            if auto(bezt.h1) {
                let la = len_a / len;
                for i in 0..3 {
                    bezt.vec[0][i] = p2[i] - tvec[i.min(1)] * la;
                }
                if bezt.h1 == HD_AUTO_ANIM && next.is_some() && prev.is_some() {
                    let pr = prev.unwrap().vec[1];
                    let nx = next.unwrap().vec[1];
                    let yd1 = pr[1] - bezt.vec[1][1];
                    let yd2 = nx[1] - bezt.vec[1][1];
                    if (yd1 <= 0.0 && yd2 <= 0.0) || (yd1 >= 0.0 && yd2 >= 0.0) {
                        bezt.vec[0][1] = bezt.vec[1][1];
                    } else if yd1 <= 0.0 {
                        bezt.vec[0][1] = bezt.vec[0][1].max(pr[1]);
                    } else {
                        bezt.vec[0][1] = bezt.vec[0][1].min(pr[1]);
                    }
                }
            }
            if auto(bezt.h2) {
                let lb = len_b / len;
                for i in 0..3 {
                    bezt.vec[2][i] = p2[i] + tvec[i.min(1)] * lb;
                }
                if bezt.h2 == HD_AUTO_ANIM && next.is_some() && prev.is_some() {
                    let pr = prev.unwrap().vec[1];
                    let nx = next.unwrap().vec[1];
                    let yd1 = pr[1] - bezt.vec[1][1];
                    let yd2 = nx[1] - bezt.vec[1][1];
                    if (yd1 <= 0.0 && yd2 <= 0.0) || (yd1 >= 0.0 && yd2 >= 0.0) {
                        bezt.vec[2][1] = bezt.vec[1][1];
                    } else if yd1 <= 0.0 {
                        bezt.vec[2][1] = bezt.vec[2][1].min(nx[1]);
                    } else {
                        bezt.vec[2][1] = bezt.vec[2][1].max(nx[1]);
                    }
                }
            }
        }
    }
    if bezt.h1 == HD_VECT {
        for i in 0..3 {
            bezt.vec[0][i] = p2[i] - dvec_a[i.min(1)] / 3.0;
        }
    }
    if bezt.h2 == HD_VECT {
        for i in 0..3 {
            bezt.vec[2][i] = p2[i] + dvec_b[i.min(1)] / 3.0;
        }
    }
}

/// `curvemap_calc_extend` — horizontal/extrapolated extension.
pub fn curvemap_calc_extend(
    cumap: &CurveMapping,
    cuma: &CurveMap,
    x: f32,
    first: [f32; 2],
    last: [f32; 2],
) -> f32 {
    if x <= first[0] {
        if cumap.flag & CUMA_EXTEND_EXTRAPOLATE == 0 {
            return first[1];
        }
        if cuma.ext_in[0] == 0.0 {
            return first[1] + cuma.ext_in[1] * 10000.0;
        }
        return first[1] + cuma.ext_in[1] * (x - first[0]) / cuma.ext_in[0];
    }
    if x >= last[0] {
        if cumap.flag & CUMA_EXTEND_EXTRAPOLATE == 0 {
            return last[1];
        }
        if cuma.ext_out[0] == 0.0 {
            return last[1] - cuma.ext_out[1] * 10000.0;
        }
        return last[1] + cuma.ext_out[1] * (x - last[0]) / cuma.ext_out[0];
    }
    0.0
}

/// `BKE_curve_correct_bezpart` — clamp overlapping handles.
pub fn curve_correct_bezpart(v1: [f32; 2], v2: &mut [f32; 2], v3: &mut [f32; 2], v4: [f32; 2]) {
    let h1 = [v1[0] - v2[0], v1[1] - v2[1]];
    let h2 = [v4[0] - v3[0], v4[1] - v3[1]];
    let len = v4[0] - v1[0];
    let len1 = h1[0].abs();
    let len2 = h2[0].abs();
    if len1 + len2 == 0.0 {
        return;
    }
    if len1 + len2 > len {
        let fac = len / (len1 + len2);
        v2[0] = v1[0] - fac * h1[0];
        v2[1] = v1[1] - fac * h1[1];
        v3[0] = v4[0] - fac * h2[0];
        v3[1] = v4[1] - fac * h2[1];
    }
}

/// `curve_eval_bezier_point` — CM_RESOL points on the segment.
fn curve_eval_bezier_point(start: [[f32; 3]; 3], end: [[f32; 3]; 3], point: &mut [f32]) {
    let mut s2 = [start[2][0], start[2][1]];
    let mut e0 = [end[0][0], end[0][1]];
    curve_correct_bezpart([start[1][0], start[1][1]], &mut s2, &mut e0, [end[1][0], end[1][1]]);
    let mut xs = vec![0.0f32; CM_RESOL];
    let mut ys = vec![0.0f32; CM_RESOL];
    bke_curve_forward_diff_bezier(
        start[1][0], s2[0], e0[0], end[1][0], &mut xs, 0, (CM_RESOL - 1) as i32, 1,
    );
    bke_curve_forward_diff_bezier(
        start[1][1], s2[1], e0[1], end[1][1], &mut ys, 0, (CM_RESOL - 1) as i32, 1,
    );
    for i in 0..CM_RESOL {
        point[2 * i] = xs[i];
        point[2 * i + 1] = ys[i];
    }
}

/// `curvemap_make_table` — build the `CM_TABLE` uniform table.
pub fn curvemap_make_table(cumap: &CurveMapping, cuma: &mut CurveMap) {
    let clipr = cumap.clipr;
    let use_wrapping = cumap.flag & CUMA_USE_WRAPPING != 0;
    if cuma.curve.is_empty() {
        return;
    }
    cuma.mintable = clipr.xmin;
    cuma.maxtable = clipr.xmax;
    let bezt_totpoint = cuma.totpoint.max(2) as usize;
    let mut bezt: Vec<BezTriple> = vec![BezTriple::default(); bezt_totpoint];
    if cuma.totpoint >= 2 {
        for a in 0..bezt_totpoint {
            let cmp = cuma.curve[a.min(cuma.curve.len() - 1)];
            cuma.mintable = cuma.mintable.min(cmp.x);
            cuma.maxtable = cuma.maxtable.max(cmp.x);
            bezt[a].vec[1][0] = cmp.x;
            bezt[a].vec[1][1] = cmp.y;
            if cmp.flag & CUMA_HANDLE_VECTOR != 0 {
                bezt[a].h1 = HD_VECT;
                bezt[a].h2 = HD_VECT;
            } else if cmp.flag & CUMA_HANDLE_AUTO_ANIM != 0 {
                bezt[a].h1 = HD_AUTO_ANIM;
                bezt[a].h2 = HD_AUTO_ANIM;
            } else {
                bezt[a].h1 = HD_AUTO;
                bezt[a].h2 = HD_AUTO;
            }
        }
    } else {
        cuma.mintable = 0.0;
        cuma.maxtable = 0.0;
        bezt[0].h1 = HD_AUTO;
        bezt[0].h2 = HD_AUTO;
        bezt[1].h1 = HD_AUTO;
        bezt[1].h2 = HD_AUTO;
    }
    let table_range = cuma.maxtable - cuma.mintable;
    let mut bezt_pre = bezt[bezt_totpoint - 1].clone();
    let mut bezt_post = bezt[0].clone();
    let mut have_pre = false;
    let mut have_post = false;
    if use_wrapping {
        bezt_pre.h1 = bezt[bezt_totpoint - 1].h2;
        bezt_pre.h2 = bezt[bezt_totpoint - 1].h2;
        bezt_pre.vec[1][0] = bezt[bezt_totpoint - 1].vec[1][0] - table_range;
        bezt_pre.vec[1][1] = bezt[bezt_totpoint - 1].vec[1][1];
        bezt_post.h1 = bezt[0].h1;
        bezt_post.h2 = bezt[0].h1;
        bezt_post.vec[1][0] = bezt[0].vec[1][0] + table_range;
        bezt_post.vec[1][1] = bezt[0].vec[1][1];
        have_pre = true;
        have_post = true;
    }
    for a in 0..bezt_totpoint {
        let prev_b = if a == 0 {
            if have_pre { Some(bezt_pre.clone()) } else { None }
        } else {
            Some(bezt[a - 1].clone())
        };
        let next_b = if a != bezt_totpoint - 1 {
            Some(bezt[a + 1].clone())
        } else if have_post {
            Some(bezt_post.clone())
        } else {
            None
        };
        let mut cur = bezt[a].clone();
        calchandle_curvemap(&mut cur, prev_b.as_ref(), next_b.as_ref());
        bezt[a] = cur;
    }
    bezt_pre.vec[0][0] = bezt[bezt_totpoint - 1].vec[0][0] - table_range;
    bezt_pre.vec[0][1] = bezt[bezt_totpoint - 1].vec[0][1];
    bezt_pre.vec[2][0] = bezt[bezt_totpoint - 1].vec[2][0] - table_range;
    bezt_pre.vec[2][1] = bezt[bezt_totpoint - 1].vec[2][1];
    bezt_post.vec[0][0] = bezt[0].vec[0][0] + table_range;
    bezt_post.vec[0][1] = bezt[0].vec[0][1];
    bezt_post.vec[2][0] = bezt[0].vec[2][0] + table_range;
    bezt_post.vec[2][1] = bezt[0].vec[2][1];

    if bezt_totpoint > 2 && !use_wrapping {
        if bezt[0].h2 == HD_AUTO {
            let hlen = len3_sub(bezt[0].vec[1], bezt[0].vec[2]);
            let mut vec = bezt[1].vec[0];
            vec[0] = vec[0].max(bezt[0].vec[1][0]);
            for i in 0..3 {
                vec[i] -= bezt[0].vec[1][i];
            }
            let nlen = (vec[0] * vec[0] + vec[1] * vec[1] + vec[2] * vec[2]).sqrt();
            if nlen > f32::EPSILON {
                for i in 0..3 {
                    vec[i] *= hlen / nlen;
                }
                for i in 0..3 {
                    bezt[0].vec[2][i] = vec[i] + bezt[0].vec[1][i];
                    bezt[0].vec[0][i] = bezt[0].vec[1][i] - vec[i];
                }
            }
        }
        let a = bezt_totpoint - 1;
        if bezt[a].h2 == HD_AUTO {
            let hlen = len3_sub(bezt[a].vec[1], bezt[a].vec[0]);
            let mut vec = bezt[a - 1].vec[2];
            vec[0] = vec[0].min(bezt[a].vec[1][0]);
            for i in 0..3 {
                vec[i] -= bezt[a].vec[1][i];
            }
            let nlen = (vec[0] * vec[0] + vec[1] * vec[1] + vec[2] * vec[2]).sqrt();
            if nlen > f32::EPSILON {
                for i in 0..3 {
                    vec[i] *= hlen / nlen;
                }
                for i in 0..3 {
                    bezt[a].vec[0][i] = vec[i] + bezt[a].vec[1][i];
                    bezt[a].vec[2][i] = bezt[a].vec[1][i] - vec[i];
                }
            }
        }
    }

    let totpoint = if use_wrapping {
        (bezt_totpoint + 1) * CM_RESOL
    } else {
        (bezt_totpoint - 1) * CM_RESOL
    };
    let mut allpoints = vec![0.0f32; totpoint * 2];
    let mut off = 0;
    if use_wrapping {
        curve_eval_bezier_point(bezt_pre.vec, bezt[0].vec, &mut allpoints[off..off + 2 * CM_RESOL]);
        off += 2 * CM_RESOL;
    }
    for a in 0..bezt_totpoint - 1 {
        curve_eval_bezier_point(
            bezt[a].vec,
            bezt[a + 1].vec,
            &mut allpoints[off..off + 2 * CM_RESOL],
        );
        off += 2 * CM_RESOL;
    }
    if use_wrapping {
        curve_eval_bezier_point(
            bezt[bezt_totpoint - 1].vec,
            bezt_post.vec,
            &mut allpoints[off..off + 2 * CM_RESOL],
        );
    }

    cuma.ext_in[0] = bezt[0].vec[0][0] - bezt[0].vec[1][0];
    cuma.ext_in[1] = bezt[0].vec[0][1] - bezt[0].vec[1][1];
    let mut ext_in_range =
        (cuma.ext_in[0] * cuma.ext_in[0] + cuma.ext_in[1] * cuma.ext_in[1]).sqrt();
    if ext_in_range == 0.0 {
        cuma.ext_in[0] = -1.0;
        cuma.ext_in[1] = 0.0;
        ext_in_range = 1.0;
    }
    cuma.ext_in[0] /= ext_in_range;
    cuma.ext_in[1] /= ext_in_range;
    let out_a = bezt_totpoint - 1;
    cuma.ext_out[0] = bezt[out_a].vec[1][0] - bezt[out_a].vec[2][0];
    cuma.ext_out[1] = bezt[out_a].vec[1][1] - bezt[out_a].vec[2][1];
    let mut ext_out_range =
        (cuma.ext_out[0] * cuma.ext_out[0] + cuma.ext_out[1] * cuma.ext_out[1]).sqrt();
    if ext_out_range == 0.0 {
        cuma.ext_out[0] = -1.0;
        cuma.ext_out[1] = 0.0;
        ext_out_range = 1.0;
    }
    cuma.ext_out[0] /= ext_out_range;
    cuma.ext_out[1] /= ext_out_range;

    let range = CM_TABLEDIV * table_range;
    cuma.range = if range != 0.0 { 1.0 / range } else { 0.0 };

    let firstpoint = 0usize;
    let lastpoint = totpoint - 1;
    let mut cmp = vec![CurveMapPoint::default(); CM_TABLE + 1];
    let mut point = firstpoint;
    for a in 0..=CM_TABLE {
        let cur_x = cuma.mintable + range * a as f32;
        cmp[a].x = cur_x;
        while cur_x >= allpoints[point * 2] && point != lastpoint {
            point += 1;
        }
        if (point == firstpoint || (point == lastpoint && cur_x >= allpoints[point * 2]))
            && !use_wrapping
        {
            if (cur_x - allpoints[point * 2]).abs() < 1e-6 {
                cmp[a].y = allpoints[point * 2 + 1];
            } else {
                cmp[a].y = curvemap_calc_extend(
                    cumap,
                    cuma,
                    cur_x,
                    [allpoints[firstpoint * 2], allpoints[firstpoint * 2 + 1]],
                    [allpoints[lastpoint * 2], allpoints[lastpoint * 2 + 1]],
                );
            }
        } else {
            let fac1 = allpoints[point * 2] - allpoints[(point - 1) * 2];
            let fac2 = allpoints[point * 2] - cur_x;
            let fac1 = if fac1 > f32::EPSILON { fac2 / fac1 } else { 0.0 };
            cmp[a].y = fac1 * allpoints[(point - 1) * 2 + 1]
                + (1.0 - fac1) * allpoints[point * 2 + 1];
        }
    }
    cuma.table = cmp;
}

fn len3_sub(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// `BKE_curvemapping_premultiply`.
pub fn curvemapping_premultiply(cumap: &mut CurveMapping, restore: bool) {
    if restore {
        if cumap.flag & CUMA_PREMULLED != 0 {
            for a in 0..3 {
                std::mem::swap(&mut cumap.cm[a].table, &mut cumap.cm[a].premultable);
                cumap.cm[a].premultable.clear();
                cumap.cm[a].ext_in = cumap.cm[a].premul_ext_in;
                cumap.cm[a].ext_out = cumap.cm[a].premul_ext_out;
                cumap.cm[a].premul_ext_in = [0.0; 2];
                cumap.cm[a].premul_ext_out = [0.0; 2];
            }
            cumap.flag &= !CUMA_PREMULLED;
        }
    } else if cumap.flag & CUMA_PREMULLED == 0 {
        for a in 0..3 {
            if cumap.cm[a].table.is_empty() {
                let cumap_copy = cumap.clone();
                curvemap_make_table(&cumap_copy, &mut cumap.cm[a]);
            }
            cumap.cm[a].premultable = cumap.cm[a].table.clone();
        }
        if cumap.cm[3].table.is_empty() {
            let cumap_copy = cumap.clone();
            curvemap_make_table(&cumap_copy, &mut cumap.cm[3]);
        }
        let cumap_copy = cumap.clone();
        for a in 0..3 {
            for b in 0..=CM_TABLE {
                cumap.cm[a].table[b].y = curvemap_evaluate_f(
                    &cumap_copy,
                    &cumap.cm[3],
                    cumap.cm[a].table[b].y,
                );
            }
            cumap.cm[a].premul_ext_in = cumap.cm[a].ext_in;
            cumap.cm[a].premul_ext_out = cumap.cm[a].ext_out;
            cumap.cm[a].ext_in[0] *= cumap.cm[3].ext_in[0];
            cumap.cm[a].ext_in[1] *= cumap.cm[3].ext_in[1];
            cumap.cm[a].ext_out[0] *= cumap.cm[3].ext_out[0];
            cumap.cm[a].ext_out[1] *= cumap.cm[3].ext_out[1];
        }
        cumap.flag |= CUMA_PREMULLED;
    }
}

/// `BKE_curvemap_active_get` — index of first selected (prefer
/// `CUMA_ACTIVE`).
pub fn curvemap_active_get(cuma: &CurveMap) -> Option<usize> {
    let mut active = None;
    for (i, pt) in cuma.curve.iter().enumerate() {
        if pt.flag & CUMA_SELECT != 0 {
            active = Some(i);
            if pt.flag & CUMA_ACTIVE != 0 {
                break;
            }
        }
    }
    active
}

/// `BKE_curvemap_translate_selection`.
pub fn curvemap_translate_selection(cuma: &mut CurveMap, offset: [f32; 2]) {
    for pt in &mut cuma.curve {
        if pt.flag & CUMA_SELECT != 0 {
            pt.x += offset[0];
            pt.y += offset[1];
        }
    }
}

/// `BKE_curvemapping_changed` — clip, sort, dedupe, rebuild table.
pub fn curvemapping_changed(cumap: &mut CurveMapping, rem_doubles: bool) {
    let cur = cumap.cur as usize;
    let clipr = cumap.clipr;
    let thresh = 0.01 * (clipr.xmax - clipr.xmin);
    let mut dx = 0.0f32;
    let mut dy = 0.0f32;
    cumap.changed_timestamp += 1;
    if cumap.flag & CUMA_DO_CLIP != 0 {
        let cuma = &cumap.cm[cur];
        for i in 0..cuma.totpoint as usize {
            let cmp = &cuma.curve[i];
            if cmp.flag & CUMA_SELECT != 0 {
                if cmp.x < clipr.xmin {
                    dx = dx.min(cmp.x - clipr.xmin);
                } else if cmp.x > clipr.xmax {
                    dx = dx.max(cmp.x - clipr.xmax);
                }
                if cmp.y < clipr.ymin {
                    dy = dy.min(cmp.y - clipr.ymin);
                } else if cmp.y > clipr.ymax {
                    dy = dy.max(cmp.y - clipr.ymax);
                }
            }
        }
        let cuma = &mut cumap.cm[cur];
        for i in 0..cuma.totpoint as usize {
            let cmp = &mut cuma.curve[i];
            if cmp.flag & CUMA_SELECT != 0 {
                cmp.x -= dx;
                cmp.y -= dy;
            }
        }
        if cumap.curr.xmax - cumap.curr.xmin > cumap.clipr.xmax - cumap.clipr.xmin {
            cumap.curr.xmin = cumap.clipr.xmin;
            cumap.curr.xmax = cumap.clipr.xmax;
        }
        if cumap.curr.ymax - cumap.curr.ymin > cumap.clipr.ymax - cumap.clipr.ymin {
            cumap.curr.ymin = cumap.clipr.ymin;
            cumap.curr.ymax = cumap.clipr.ymax;
        }
    }
    cumap.cm[cur]
        .curve
        .sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    if rem_doubles && cumap.cm[cur].totpoint > 2 {
        let cuma = &mut cumap.cm[cur];
        let mut mark = false;
        for a in 0..cuma.totpoint as usize - 1 {
            let ddx = cuma.curve[a].x - cuma.curve[a + 1].x;
            let ddy = cuma.curve[a].y - cuma.curve[a + 1].y;
            if (ddx * ddx + ddy * ddy).sqrt() < thresh {
                if a == 0 {
                    cuma.curve[a + 1].flag |= CUMA_REMOVE;
                    if cuma.curve[a + 1].flag & CUMA_SELECT != 0 {
                        cuma.curve[a].flag |= CUMA_SELECT;
                    }
                    if cuma.curve[a + 1].flag & CUMA_ACTIVE != 0 {
                        cuma.curve[a].flag |= CUMA_ACTIVE;
                    }
                } else {
                    cuma.curve[a].flag |= CUMA_REMOVE;
                    if cuma.curve[a].flag & CUMA_SELECT != 0 {
                        cuma.curve[a + 1].flag |= CUMA_SELECT;
                    }
                    if cuma.curve[a].flag & CUMA_ACTIVE != 0 {
                        cuma.curve[a + 1].flag |= CUMA_ACTIVE;
                    }
                }
                mark = true;
                break;
            }
        }
        if mark {
            curvemap_remove(cuma, CUMA_REMOVE);
        }
    }
    let cumap_copy = cumap.clone();
    curvemap_make_table(&cumap_copy, &mut cumap.cm[cur]);
}

/// `BKE_curvemapping_changed_all`.
pub fn curvemapping_changed_all(cumap: &mut CurveMapping) {
    let cur = cumap.cur;
    for a in 0..CM_TOT {
        if !cumap.cm[a].curve.is_empty() {
            cumap.cur = a as i32;
            curvemapping_changed(cumap, false);
        }
    }
    cumap.cur = cur;
}

/// `BKE_curvemapping_reset_view`.
pub fn curvemapping_reset_view(cumap: &mut CurveMapping) {
    cumap.curr = cumap.clipr;
}

/// `BKE_curvemap_evaluateF`.
pub fn curvemap_evaluate_f(cumap: &CurveMapping, cuma: &CurveMap, value: f32) -> f32 {
    let fi = (value - cuma.mintable) * cuma.range;
    let i = fi as i32;
    if fi < 0.0 || fi > CM_TABLE as f32 {
        return curvemap_calc_extend(
            cumap,
            cuma,
            value,
            [cuma.table[0].x, cuma.table[0].y],
            [cuma.table[CM_TABLE].x, cuma.table[CM_TABLE].y],
        );
    }
    if i < 0 {
        return cuma.table[0].y;
    }
    if i as usize >= CM_TABLE {
        return cuma.table[CM_TABLE].y;
    }
    let fi = fi - i as f32;
    (1.0 - fi) * cuma.table[i as usize].y + fi * cuma.table[i as usize + 1].y
}

/// `BKE_curvemapping_evaluateF` (curve index + clip).
pub fn curvemapping_evaluate_f(cumap: &CurveMapping, cur: usize, value: f32) -> f32 {
    let mut val = curvemap_evaluate_f(cumap, &cumap.cm[cur], value);
    if cumap.flag & CUMA_DO_CLIP != 0 {
        val = val.clamp(cumap.clipr.ymin, cumap.clipr.ymax);
    }
    val
}

/// `BKE_curvemapping_evaluate3F`.
pub fn curvemapping_evaluate3_f(cumap: &CurveMapping, vecin: [f32; 3]) -> [f32; 3] {
    [
        curvemap_evaluate_f(cumap, &cumap.cm[0], vecin[0]),
        curvemap_evaluate_f(cumap, &cumap.cm[1], vecin[1]),
        curvemap_evaluate_f(cumap, &cumap.cm[2], vecin[2]),
    ]
}

/// `BKE_curvemapping_evaluateRGBF`.
pub fn curvemapping_evaluate_rgbf(cumap: &CurveMapping, vecin: [f32; 3]) -> [f32; 3] {
    [
        curvemap_evaluate_f(cumap, &cumap.cm[0], curvemap_evaluate_f(cumap, &cumap.cm[3], vecin[0])),
        curvemap_evaluate_f(cumap, &cumap.cm[1], curvemap_evaluate_f(cumap, &cumap.cm[3], vecin[1])),
        curvemap_evaluate_f(cumap, &cumap.cm[2], curvemap_evaluate_f(cumap, &cumap.cm[3], vecin[2])),
    ]
}

/// `evaluate_film_like` — hue-preserving combined-curve eval.
fn evaluate_film_like(cumap: &CurveMapping, input: [f32; 3]) -> [f32; 3] {
    let curve_map = &cumap.cm[3];
    let minimum = input[0].min(input[1]).min(input[2]);
    let maximum = input[0].max(input[1]).max(input[2]);
    /* `math::max(min(x,y), min(z, max(x,y)))`. */
    let median = input[0].min(input[1]).max(input[2].min(input[0].max(input[1])));
    let new_min = curvemap_evaluate_f(cumap, curve_map, minimum);
    let new_max = curvemap_evaluate_f(cumap, curve_map, maximum);
    let scaling_ratio = (new_max - new_min) / (maximum - minimum);
    let new_median = new_min + (median - minimum) * scaling_ratio;
    [
        if input[0] == maximum {
            new_max
        } else if input[0] == minimum {
            new_min
        } else {
            new_median
        },
        if input[1] == maximum {
            new_max
        } else if input[1] == minimum {
            new_min
        } else {
            new_median
        },
        if input[2] == maximum {
            new_max
        } else if input[2] == minimum {
            new_min
        } else {
            new_median
        },
    ]
}

/// `BKE_curvemapping_evaluate_premulRGBF_ex`.
pub fn evaluate_premul_rgbf_ex(
    cumap: &CurveMapping,
    vecin: [f32; 3],
    black: [f32; 3],
    bwmul: [f32; 3],
) -> [f32; 3] {
    let balanced = [
        (vecin[0] - black[0]) * bwmul[0],
        (vecin[1] - black[1]) * bwmul[1],
        (vecin[2] - black[2]) * bwmul[2],
    ];
    match cumap.tone {
        CURVE_TONE_FILMLIKE => evaluate_film_like(cumap, balanced),
        _ => [
            curvemap_evaluate_f(cumap, &cumap.cm[0], balanced[0]),
            curvemap_evaluate_f(cumap, &cumap.cm[1], balanced[1]),
            curvemap_evaluate_f(cumap, &cumap.cm[2], balanced[2]),
        ],
    }
}

/// `BKE_curvemapping_evaluate_premulRGBF`.
pub fn evaluate_premul_rgbf(cumap: &CurveMapping, vecin: [f32; 3]) -> [f32; 3] {
    evaluate_premul_rgbf_ex(cumap, vecin, cumap.black, cumap.bwmul)
}

/// `BKE_curvemapping_evaluate_premulRGB` — byte variant.
pub fn evaluate_premul_rgb(cumap: &CurveMapping, vecin_byte: [u8; 3]) -> [u8; 3] {
    let vecin = [
        vecin_byte[0] as f32 / 255.0,
        vecin_byte[1] as f32 / 255.0,
        vecin_byte[2] as f32 / 255.0,
    ];
    let out = evaluate_premul_rgbf(cumap, vecin);
    out.map(|v| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8)
}

/// `BKE_curvemapping_RGBA_does_something`.
pub fn rgba_does_something(cumap: &CurveMapping) -> bool {
    if cumap.black.iter().any(|&b| b != 0.0) {
        return true;
    }
    if cumap.white.iter().any(|&w| w != 1.0) {
        return true;
    }
    for cm in &cumap.cm {
        if !cm.curve.is_empty() {
            if cm.totpoint != 2 {
                return true;
            }
            if cm.curve[0].x != 0.0 || cm.curve[0].y != 0.0 {
                return true;
            }
            if cm.curve[1].x != 1.0 || cm.curve[1].y != 1.0 {
                return true;
            }
        }
    }
    false
}

/// `BKE_curvemapping_get_range_minimums`.
pub fn get_range_minimums(cumap: &CurveMapping) -> [f32; CM_TOT] {
    let mut m = [0.0; CM_TOT];
    for i in 0..CM_TOT {
        m[i] = cumap.cm[i].mintable;
    }
    m
}

/// `BKE_curvemapping_compute_range_dividers`.
pub fn compute_range_dividers(cumap: &CurveMapping) -> [f32; CM_TOT] {
    let mut d = [0.0; CM_TOT];
    for i in 0..CM_TOT {
        d[i] = 1.0 / (cumap.cm[i].maxtable - cumap.cm[i].mintable).max(1e-8);
    }
    d
}

/// `BKE_curvemapping_compute_slopes`.
pub fn compute_slopes(cumap: &CurveMapping) -> ([f32; CM_TOT], [f32; CM_TOT]) {
    let range_dividers = compute_range_dividers(cumap);
    let mut start_slopes = [0.0; CM_TOT];
    let mut end_slopes = [0.0; CM_TOT];
    for i in 0..CM_TOT {
        let cm = &cumap.cm[i];
        if cumap.flag & CUMA_EXTEND_EXTRAPOLATE == 0 {
            continue;
        }
        start_slopes[i] = if cm.ext_in[0] != 0.0 {
            cm.ext_in[1] / (cm.ext_in[0] * range_dividers[i])
        } else {
            1e8
        };
        end_slopes[i] = if cm.ext_out[0] != 0.0 {
            cm.ext_out[1] / (cm.ext_out[0] * range_dividers[i])
        } else {
            1e8
        };
    }
    (start_slopes, end_slopes)
}

/// `BKE_curvemapping_is_map_identity`.
pub fn is_map_identity(cumap: &CurveMapping, index: usize) -> bool {
    if cumap.flag & CUMA_EXTEND_EXTRAPOLATE == 0 {
        return false;
    }
    let cm = &cumap.cm[index];
    if cm.maxtable - cm.mintable != 1.0 {
        return false;
    }
    if cm.ext_in[0] != cm.ext_in[1] {
        return false;
    }
    if cm.ext_out[0] != cm.ext_out[1] {
        return false;
    }
    if cm.totpoint != 2 {
        return false;
    }
    if cm.curve[0].x != 0.0 || cm.curve[0].y != 0.0 {
        return false;
    }
    if cm.curve[1].x != 0.0 || cm.curve[1].y != 0.0 {
        return false;
    }
    true
}

/// `BKE_curvemapping_init` — build missing tables.
pub fn curvemapping_init(cumap: &mut CurveMapping) {
    let cumap_copy = cumap.clone();
    for a in 0..CM_TOT {
        if cumap.cm[a].table.is_empty() {
            curvemap_make_table(&cumap_copy, &mut cumap.cm[a]);
        }
    }
}

/// `BKE_curvemapping_table_F` — 4-channel out (R only filled).
pub fn curvemapping_table_f(cumap: &CurveMapping) -> Vec<f32> {
    let size = CM_TABLE + 1;
    let mut array = vec![0.0f32; 4 * size];
    for a in 0..size {
        if !cumap.cm[0].table.is_empty() {
            array[a * 4] = cumap.cm[0].table[a].y;
        }
    }
    array
}

/// `BKE_curvemapping_table_RGBA`.
pub fn curvemapping_table_rgba(cumap: &CurveMapping) -> Vec<f32> {
    let size = CM_TABLE + 1;
    let mut array = vec![0.0f32; 4 * size];
    for a in 0..size {
        for c in 0..4 {
            if !cumap.cm[c].table.is_empty() {
                array[a * 4 + c] = cumap.cm[c].table[a].y;
            }
        }
    }
    array
}

/// `BKE_curvemapping_num_channels`.
pub fn num_channels(cumap: &CurveMapping) -> i32 {
    if cumap.cm[3].totpoint > 0 {
        return 4;
    }
    if cumap.cm[2].totpoint > 0 {
        return 3;
    }
    if cumap.cm[1].totpoint > 0 {
        return 2;
    }
    1
}

fn get_bin_float(f: f32) -> usize {
    ((f * 255.0 + 0.5) as i32).clamp(0, 255) as usize
}

/// `BKE_histogram_update_sample_line` — pixels supplied by
/// `get_pixel(x,y) -> Option<rgba>`; `cm_apply_v4` display-space
/// transform; `luminance` helper injected.
pub fn histogram_update_sample_line(
    hist: &mut Histogram,
    ibuf_w: i32,
    ibuf_h: i32,
    ibuf_channels: i32,
    is_float: bool,
    get_pixel: &mut dyn FnMut(i32, i32) -> Option<[f32; 4]>,
    cm_apply_v4: Option<&mut dyn FnMut(&mut [f32; 4])>,
    luminance: &dyn Fn(&[f32; 4]) -> f32,
) {
    let mut cm_apply_v4 = cm_apply_v4;
    let x1 = (hist.co[0][0] * ibuf_w as f32).round() as i32;
    let x2 = (hist.co[1][0] * ibuf_w as f32).round() as i32;
    let y1 = (hist.co[0][1] * ibuf_h as f32).round() as i32;
    let y2 = (hist.co[1][1] * ibuf_h as f32).round() as i32;
    hist.channels = 3;
    hist.x_resolution = 256;
    hist.xmax = 1.0;
    let _ = is_float;
    for i in 0..256 {
        let x = (0.5 + x1 as f32 + i as f32 * (x2 - x1) as f32 / 255.0) as i32;
        let y = (0.5 + y1 as f32 + i as f32 * (y2 - y1) as f32 / 255.0) as i32;
        if x < 0 || y < 0 || x >= ibuf_w || y >= ibuf_h {
            hist.data_luma[i] = 0.0;
            hist.data_r[i] = 0.0;
            hist.data_g[i] = 0.0;
            hist.data_b[i] = 0.0;
            hist.data_a[i] = 0.0;
            continue;
        }
        let mut rgba = match get_pixel(x, y) {
            Some(r) => r,
            None => {
                hist.data_luma[i] = 0.0;
                hist.data_r[i] = 0.0;
                hist.data_g[i] = 0.0;
                hist.data_b[i] = 0.0;
                hist.data_a[i] = 0.0;
                continue;
            }
        };
        match ibuf_channels {
            4 => {
                if let Some(apply) = cm_apply_v4.as_deref_mut() {
                    apply(&mut rgba);
                }
            }
            3 => {
                if let Some(apply) = cm_apply_v4.as_deref_mut() {
                    apply(&mut rgba);
                }
                rgba[3] = 1.0;
            }
            2 => {
                rgba[1] = rgba[0];
                rgba[2] = rgba[0];
                rgba[3] = rgba[1];
            }
            1 => {
                rgba[1] = rgba[0];
                rgba[2] = rgba[0];
                rgba[3] = 1.0;
            }
            _ => {}
        }
        hist.data_luma[i] = luminance(&rgba);
        hist.data_r[i] = rgba[0];
        hist.data_g[i] = rgba[1];
        hist.data_b[i] = rgba[2];
        hist.data_a[i] = rgba[3];
    }
}

/// `BKE_scopes_free`.
pub fn scopes_free(scopes: &mut Scopes) {
    scopes.waveform_1.clear();
    scopes.waveform_2.clear();
    scopes.waveform_3.clear();
    scopes.vecscope.clear();
    scopes.vecscope_rgb.clear();
}

/// `BKE_scopes_new`.
pub fn scopes_new(scopes: &mut Scopes) {
    scopes.accuracy = 30.0;
    scopes.hist.mode = 1; /* HISTO_MODE_RGB */
    scopes.wavefrm_alpha = 0.3;
    scopes.vecscope_alpha = 0.3;
    scopes.wavefrm_height = 100;
    scopes.vecscope_height = 100;
    scopes.hist.height = 100;
    scopes.ok = 0;
    scopes.waveform_1.clear();
    scopes.waveform_2.clear();
    scopes.waveform_3.clear();
    scopes.vecscope.clear();
    scopes.vecscope_rgb.clear();
}

/// `save_sample_line` — one vecscope/waveform sample.
fn save_sample_line(
    scopes: &mut Scopes,
    yuv_matrix: &[[f32; 3]; 3],
    idx: usize,
    fx: f32,
    position_rgb: [f32; 3],
    position_ycc: [f32; 3],
    color_rgb: [f32; 3],
    cm_apply_v3_colors: Option<&mut dyn FnMut(&mut [f32; 3])>,
) {
    let clamped = [
        position_rgb[0].min(1.0),
        position_rgb[1].min(1.0),
        position_rgb[2].min(1.0),
    ];
    let yuv = [
        yuv_matrix[0][0] * clamped[0] + yuv_matrix[0][1] * clamped[1] + yuv_matrix[0][2] * clamped[2],
        yuv_matrix[1][0] * clamped[0] + yuv_matrix[1][1] * clamped[1] + yuv_matrix[1][2] * clamped[2],
        yuv_matrix[2][0] * clamped[0] + yuv_matrix[2][1] * clamped[1] + yuv_matrix[2][2] * clamped[2],
    ];
    if scopes.vecscope.len() > idx + 1 {
        scopes.vecscope[idx] = yuv[1];
        scopes.vecscope[idx + 1] = yuv[2];
    }
    let color_idx = (idx / 2) * 3;
    let (mut h, mut s, _v) = rgb_to_hsv(color_rgb[0], color_rgb[1], color_rgb[2]);
    s *= 0.5;
    let rgb = hsv_to_rgb(h, s, 1.0);
    if scopes.vecscope_rgb.len() > color_idx + 2 {
        scopes.vecscope_rgb[color_idx] = rgb[0];
        scopes.vecscope_rgb[color_idx + 1] = rgb[1];
        scopes.vecscope_rgb[color_idx + 2] = rgb[2];
        if let Some(apply) = cm_apply_v3_colors {
            apply(&mut [
                scopes.vecscope_rgb[color_idx],
                scopes.vecscope_rgb[color_idx + 1],
                scopes.vecscope_rgb[color_idx + 2],
            ]);
        }
    }
    match scopes.wavefrm_mode as i32 {
        0 | 4 => {
            /* RGB / RGB_PARADE */
            if scopes.waveform_1.len() > idx + 1 {
                scopes.waveform_1[idx] = fx;
                scopes.waveform_1[idx + 1] = position_rgb[0].min(1.0);
                scopes.waveform_2[idx] = fx;
                scopes.waveform_2[idx + 1] = position_rgb[1].min(1.0);
                scopes.waveform_3[idx] = fx;
                scopes.waveform_3[idx + 1] = position_rgb[2].min(1.0);
            }
        }
        1 => {
            /* LUMA */
            if scopes.waveform_1.len() > idx + 1 {
                scopes.waveform_1[idx] = fx;
                scopes.waveform_1[idx + 1] = position_ycc[0].min(1.0);
            }
        }
        2 | 3 | 5 => {
            /* YCC JPEG/709/601 */
            if scopes.waveform_1.len() > idx + 1 {
                scopes.waveform_1[idx] = fx;
                scopes.waveform_1[idx + 1] = position_ycc[0].min(1.0);
                scopes.waveform_2[idx] = fx;
                scopes.waveform_2[idx + 1] = position_ycc[1].min(1.0);
                scopes.waveform_3[idx] = fx;
                scopes.waveform_3[idx + 1] = position_ycc[2].min(1.0);
            }
        }
        _ => {}
    }
}

/// `BKE_scopes_update` — serial port of the per-row callback +
/// reduce; pixels via `get_row(y) -> Vec<rgba>`, scope-space +
/// display-space processors injected.
#[allow(clippy::too_many_arguments)]
pub fn scopes_update(
    scopes: &mut Scopes,
    ibuf_w: i32,
    ibuf_h: i32,
    ibuf_channels: i32,
    get_row: &mut dyn FnMut(i32) -> Vec<[f32; 4]>,
    cm_apply_v4_positions: Option<&mut dyn FnMut(&mut [f32; 4])>,
    cm_apply_v3_colors: Option<&mut dyn FnMut(&mut [f32; 3])>,
    luma_coefficients: [f32; 3],
    yuv_matrix: [[f32; 3]; 3],
    ycc_mode: i32,
) {
    let mut cm_apply_v4_positions = cm_apply_v4_positions;
    let mut cm_apply_v3_colors = cm_apply_v3_colors;
    let cm_apply_v3_colors = &mut cm_apply_v3_colors;
    let mut bin_lum = [0u32; 256];
    let mut bin_r = [0u32; 256];
    let mut bin_g = [0u32; 256];
    let mut bin_b = [0u32; 256];
    let mut bin_a = [0u32; 256];
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];

    let rows_per_sample_line = (ibuf_h / scopes.sample_lines.max(1)).max(1);
    for y in 0..ibuf_h {
        let savedlines = y / rows_per_sample_line;
        let do_sample_line = savedlines < scopes.sample_lines && y % rows_per_sample_line == 0;
        let row = get_row(y);
        for (x, rgba) in row.iter().enumerate() {
            let mut position_rgba = *rgba;
            if let Some(apply) = cm_apply_v4_positions.as_deref_mut() {
                apply(&mut position_rgba);
            }
            let luma = luma_coefficients[0] * position_rgba[0]
                + luma_coefficients[1] * position_rgba[1]
                + luma_coefficients[2] * position_rgba[2];
            let mut position_ycc = [0.0f32; 3];
            if ycc_mode == -1 {
                for i in 0..3 {
                    min[i] = min[i].min(position_rgba[i]);
                    max[i] = max[i].max(position_rgba[i]);
                }
            } else {
                let (yy, cb, cr) = rgb_to_ycc(
                    position_rgba[0], position_rgba[1], position_rgba[2], ycc_mode,
                );
                position_ycc = [yy / 255.0, cb / 255.0, cr / 255.0];
                for i in 0..3 {
                    min[i] = min[i].min(position_ycc[i]);
                    max[i] = max[i].max(position_ycc[i]);
                }
            }
            bin_lum[get_bin_float(luma)] += 1;
            bin_r[get_bin_float(position_rgba[0])] += 1;
            bin_g[get_bin_float(position_rgba[1])] += 1;
            bin_b[get_bin_float(position_rgba[2])] += 1;
            bin_a[get_bin_float(position_rgba[3])] += 1;
            if do_sample_line {
                let fx = x as f32 / ibuf_w as f32;
                let idx = 2 * (ibuf_w as usize * savedlines as usize + x);
                save_sample_line(
                    scopes,
                    &yuv_matrix,
                    idx,
                    fx,
                    [position_rgba[0], position_rgba[1], position_rgba[2]],
                    position_ycc,
                    [rgba[0], rgba[1], rgba[2]],
                    match cm_apply_v3_colors {
                        Some(c) => Some(*c),
                        None => None,
                    },
                );
            }
        }
    }
    scopes.minmax[0] = [min[0], max[0]];
    scopes.minmax[1] = [min[1], max[1]];
    scopes.minmax[2] = [min[2], max[2]];

    /* normalize `scopes->hist` to max bin. */
    let max_l = *bin_lum.iter().max().unwrap_or(&0) as f32;
    let max_r = *bin_r.iter().max().unwrap_or(&0) as f32;
    let max_g = *bin_g.iter().max().unwrap_or(&0) as f32;
    let max_b = *bin_b.iter().max().unwrap_or(&0) as f32;
    let max_a = *bin_a.iter().max().unwrap_or(&0) as f32;
    let divl = if max_l != 0.0 { 1.0 / max_l } else { 0.0 };
    let divr = if max_r != 0.0 { 1.0 / max_r } else { 0.0 };
    let divg = if max_g != 0.0 { 1.0 / max_g } else { 0.0 };
    let divb = if max_b != 0.0 { 1.0 / max_b } else { 0.0 };
    let diva = if max_a != 0.0 { 1.0 / max_a } else { 0.0 };
    for a in 0..256 {
        scopes.hist.data_luma[a] = bin_lum[a] as f32 * divl;
        scopes.hist.data_r[a] = bin_r[a] as f32 * divr;
        scopes.hist.data_g[a] = bin_g[a] as f32 * divg;
        scopes.hist.data_b[a] = bin_b[a] as f32 * divb;
        scopes.hist.data_a[a] = bin_a[a] as f32 * diva;
    }
    scopes.ok = 1;
}

/* Color-managed settings. */

fn strncpy64(dst: &mut [u8; 64], src: &str) {
    let b = src.as_bytes();
    let n = b.len().min(63);
    dst[..n].copy_from_slice(&b[..n]);
    dst[n] = 0;
}

/// `BKE_color_managed_display_settings_init` — `default_name`
/// injected.
pub fn color_managed_display_settings_init(
    display_device: &mut [u8; 64],
    emulation: &mut i32,
    default_name: &str,
) {
    strncpy64(display_device, default_name);
    *emulation = 0; /* COLORMANAGE_DISPLAY_EMULATION_AUTO */
}

/// `BKE_color_managed_display_settings_copy`.
pub fn color_managed_display_settings_copy(
    display_device: &mut [u8; 64],
    emulation: &mut i32,
    from_device: &[u8; 64],
    from_emulation: i32,
) {
    *display_device = *from_device;
    *emulation = from_emulation;
}

/// `BKE_color_managed_view_settings_init` — view transform +
/// display injected; `validate` callback for
/// `IMB_colormanagement_validate_settings`.
pub fn color_managed_view_settings_init(
    view_transform: &mut [u8; 64],
    look: &mut [u8; 64],
    flag: &mut i32,
    gamma: &mut f32,
    exposure: &mut f32,
    temperature: &mut f32,
    tint: &mut f32,
    curve_mapping: &mut usize,
    transform_name: &str,
    validate: &mut dyn FnMut(),
) {
    strncpy64(view_transform, transform_name);
    strncpy64(look, "None");
    *flag = 0;
    *gamma = 1.0;
    *exposure = 0.0;
    *temperature = 6500.0;
    *tint = 10.0;
    *curve_mapping = 0;
    validate();
}

/// `BKE_color_managed_view_settings_copy_keep_curve_mapping`.
pub fn color_managed_view_settings_copy_keep_curve_mapping(
    look: &mut [u8; 64],
    view_transform: &mut [u8; 64],
    flag: &mut i32,
    exposure: &mut f32,
    gamma: &mut f32,
    temperature: &mut f32,
    tint: &mut f32,
    from: &ColorManagedViewFlat,
) {
    *look = from.look;
    *view_transform = from.view_transform;
    *flag = from.flag;
    *exposure = from.exposure;
    *gamma = from.gamma;
    *temperature = from.temperature;
    *tint = from.tint;
}

/// Flat view-settings snapshot for copy fns.
#[derive(Clone, Copy)]
pub struct ColorManagedViewFlat {
    /// `look[64]`.
    pub look: [u8; 64],
    /// `view_transform[64]`.
    pub view_transform: [u8; 64],
    /// `flag`.
    pub flag: i32,
    /// `exposure`.
    pub exposure: f32,
    /// `gamma`.
    pub gamma: f32,
    /// `temperature`.
    pub temperature: f32,
    /// `tint`.
    pub tint: f32,
}

/// `BKE_color_managed_colorspace_settings_init`.
pub fn colorspace_settings_init(name: &mut [u8; 64], interop_id: &mut [u8; 64]) {
    strncpy64(name, "");
    strncpy64(interop_id, "");
}

/// `BKE_color_managed_colorspace_settings_equals`.
pub fn colorspace_settings_equals(a: &[u8; 64], b: &[u8; 64]) -> bool {
    a == b
}

/// `BKE_curvemapping_equals`.
pub fn curvemapping_equals(a: &CurveMapping, b: &CurveMapping) -> bool {
    if (a.flag & !CUMA_PREMULLED) != (b.flag & !CUMA_PREMULLED)
        || a.tone != b.tone
        || a.preset != b.preset
    {
        return false;
    }
    if a.clipr.xmin != b.clipr.xmin
        || a.clipr.xmax != b.clipr.xmax
        || a.clipr.ymin != b.clipr.ymin
        || a.clipr.ymax != b.clipr.ymax
    {
        return false;
    }
    if a.black != b.black || a.white != b.white {
        return false;
    }
    let mask = CUMA_HANDLE_VECTOR | CUMA_HANDLE_AUTO_ANIM;
    for i in 0..CM_TOT {
        let (ca, cb) = (&a.cm[i], &b.cm[i]);
        if ca.totpoint != cb.totpoint || ca.default_handle_type != cb.default_handle_type {
            return false;
        }
        for j in 0..ca.totpoint as usize {
            let (pa, pb) = (&ca.curve[j], &cb.curve[j]);
            if pa.x != pb.x || pa.y != pb.y || (pa.flag & mask) != (pb.flag & mask) {
                return false;
            }
        }
    }
    true
}
