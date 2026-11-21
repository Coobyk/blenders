//! Port of `intern/curveprofile.cc` — the `CurveProfile` widget:
//! point/handle ops, path management, bezier handle calc, adaptive
//! `create_samples` sampling, table/segments-table build, update.

use blenders_bli::rct::{rctf_size_x, rctf_size_y, Rctf};
use blenders_math::geom::dist_squared_to_line_segment_v2;
use blenders_math::vector::Vec2;

/// `angle_v2v2` — angle between two 2D vectors.
fn angle_v2v2(a: [f32;2], b: [f32;2]) -> f32 {
    let la = (a[0]*a[0]+a[1]*a[1]).sqrt();
    let lb = (b[0]*b[0]+b[1]*b[1]).sqrt();
    if la == 0.0 || lb == 0.0 { return 0.0; }
    let c = ((a[0]*b[0]+a[1]*b[1]) / (la*lb)).clamp(-1.0, 1.0);
    c.acos()
}

/// `eCurveProfilePresets`.
pub const PROF_PRESET_LINE: i32 = 0;
/// `PROF_PRESET_SUPPORTS`.
pub const PROF_PRESET_SUPPORTS: i32 = 1;
/// `PROF_PRESET_CORNERS`.
pub const PROF_PRESET_CORNERS: i32 = 2;
/// `PROF_PRESET_STEPS`.
pub const PROF_PRESET_STEPS: i32 = 3;
/// `PROF_TABLE_MAX`.
pub const PROF_TABLE_MAX: i32 = 512;
/// `PROF_SELECT`.
pub const PROF_SELECT: i16 = 1 << 0;
/// `PROF_H1_SELECT`.
pub const PROF_H1_SELECT: i16 = 1 << 1;
/// `PROF_H2_SELECT`.
pub const PROF_H2_SELECT: i16 = 1 << 2;
/// `PROF_ACTIVE`.
pub const PROF_ACTIVE: i16 = 1 << 3;
/// `PROF_H1_ACTIVE`.
pub const PROF_H1_ACTIVE: i16 = 1 << 4;
/// `PROF_H2_ACTIVE`.
pub const PROF_H2_ACTIVE: i16 = 1 << 5;
/// `PROF_USE_CLIP`.
pub const PROF_USE_CLIP: i16 = 1 << 0;
/// `PROF_SAMPLE_EVEN_LENGTHS`.
pub const PROF_SAMPLE_EVEN_LENGTHS: i16 = 1 << 1;
/// `PROF_SAMPLE_STRAIGHT_EDGES`.
pub const PROF_SAMPLE_STRAIGHT_EDGES: i16 = 1 << 2;
/// `PROF_DIRTY_PRESET`.
pub const PROF_DIRTY_PRESET: i16 = 1 << 3;
/// `PROF_UPDATE_*`.
pub const PROF_UPDATE_NONE: i32 = 0;
/// See above.
pub const PROF_UPDATE_CLIP: i32 = 1 << 0;
/// See above.
pub const PROF_UPDATE_REMOVE_DOUBLES: i32 = 1 << 1;
/// `HD_*` handle types.
pub const HD_FREE: i8 = 0;
/// `HD_AUTO`.
pub const HD_AUTO: i8 = 1;
/// `HD_VECT`.
pub const HD_VECT: i8 = 2;
/// `HD_ALIGN`.
pub const HD_ALIGN: i8 = 3;

/// `CurveProfilePoint`.
#[derive(Clone, Default)]
pub struct CurveProfilePoint {
    /// `x`,`y`.
    pub x: f32,
    /// See `x`.
    pub y: f32,
    /// `h1`, `h2` (HD_* types).
    pub h1: i8,
    /// See `h1`.
    pub h2: i8,
    /// `h1_loc`, `h2_loc`.
    pub h1_loc: [f32; 2],
    /// See `h1_loc`.
    pub h2_loc: [f32; 2],
    /// `flag` (selection bits).
    pub flag: i16,
}

/// `CurveProfile`.
#[derive(Clone)]
pub struct CurveProfile {
    /// `path` points.
    pub path: Vec<CurveProfilePoint>,
    /// `table` (evaluated, `table_size + 1` points).
    pub table: Vec<CurveProfilePoint>,
    /// `segments` (segment table).
    pub segments: Vec<CurveProfilePoint>,
    /// `segments_len`.
    pub segments_len: i16,
    /// `view_rect`, `clip_rect`.
    pub view_rect: Rctf,
    /// See `view_rect`.
    pub clip_rect: Rctf,
    /// `flag` (PROF_*).
    pub flag: i16,
    /// `preset`.
    pub preset: i32,
    /// `changed_timestamp`.
    pub changed_timestamp: u64,
    /// `i_last`.
    pub i_last: i32,
}

impl Default for CurveProfile {
    fn default() -> Self {
        Self {
            path: Vec::new(),
            table: Vec::new(),
            segments: Vec::new(),
            segments_len: 0,
            view_rect: Rctf {
                xmin: 0.0,
                xmax: 1.0,
                ymin: 0.0,
                ymax: 1.0,
            },
            clip_rect: Rctf {
                xmin: 0.0,
                xmax: 1.0,
                ymin: 0.0,
                ymax: 1.0,
            },
            flag: PROF_USE_CLIP,
            preset: PROF_PRESET_LINE,
            changed_timestamp: 0,
            i_last: 0,
        }
    }
}

/// `point_init`.
fn point_init(x: f32, y: f32, h1: i8, h2: i8) -> CurveProfilePoint {
    CurveProfilePoint {
        x,
        y,
        flag: 0,
        h1,
        h2,
        ..Default::default()
    }
}

/// `BKE_curveprofile_move_handle` — `point` = path index.
pub fn bke_curveprofile_move_handle(
    point: &mut CurveProfilePoint,
    handle_1: bool,
    snap: bool,
    delta: Option<[f32; 2]>,
) -> bool {
    let handle_type = if handle_1 { point.h1 } else { point.h2 };
    let start = if handle_1 { point.h1_loc } else { point.h2_loc };
    if handle_type != HD_FREE && handle_type != HD_ALIGN {
        return false;
    }
    let loc = if handle_1 {
        &mut point.h1_loc
    } else {
        &mut point.h2_loc
    };
    loc[0] += delta.map(|d| d[0]).unwrap_or(0.0);
    loc[1] += delta.map(|d| d[1]).unwrap_or(0.0);
    if snap {
        loc[0] = 0.125 * (8.0 * loc[0]).round();
        loc[1] = 0.125 * (8.0 * loc[1]).round();
    }
    if handle_type == HD_ALIGN {
        let other_type = if handle_1 { point.h2 } else { point.h1 };
        if other_type == HD_ALIGN {
            let cur = if handle_1 { point.h1_loc } else { point.h2_loc };
            if handle_1 {
                point.h2_loc = [2.0 * point.x - cur[0], 2.0 * point.y - cur[1]];
            } else {
                point.h1_loc = [2.0 * point.x - cur[0], 2.0 * point.y - cur[1]];
            }
        }
    }
    let now = if handle_1 { point.h1_loc } else { point.h2_loc };
    now != start
}

/// `BKE_curveprofile_move_point` — `index` must not be 0 or last.
pub fn bke_curveprofile_move_point(
    profile: &mut CurveProfile,
    index: usize,
    snap: bool,
    delta: [f32; 2],
) -> bool {
    if index == 0 || index == profile.path.len() - 1 {
        return false;
    }
    let point = &mut profile.path[index];
    let (origx, origy) = (point.x, point.y);
    point.x += delta[0];
    point.y += delta[1];
    if snap {
        point.x = 0.125 * (8.0 * point.x).round();
        point.y = 0.125 * (8.0 * point.y).round();
    }
    if profile.flag & PROF_USE_CLIP != 0 {
        point.x = point.x.max(profile.clip_rect.xmin).min(profile.clip_rect.xmax);
        point.y = point.y.max(profile.clip_rect.ymin).min(profile.clip_rect.ymax);
    }
    if point.h1 == HD_FREE || point.h1 == HD_ALIGN {
        point.h1_loc[0] += point.x - origx;
        point.h1_loc[1] += point.y - origy;
    }
    if point.h2 == HD_FREE || point.h2 == HD_ALIGN {
        point.h2_loc[0] += point.x - origx;
        point.h2_loc[1] += point.y - origy;
    }
    point.x != origx || point.y != origy
}

/// `BKE_curveprofile_translate_selection`.
pub fn bke_curveprofile_translate_selection(profile: &mut CurveProfile, offset: [f32; 2]) {
    for i in 0..profile.path.len() {
        let pt_flag = profile.path[i].flag;
        if (pt_flag & PROF_SELECT) != 0
            || ((pt_flag & PROF_H1_SELECT != 0 && profile.path[i].h1 == HD_ALIGN)
                && (pt_flag & PROF_H2_SELECT != 0 && profile.path[i].h2 == HD_ALIGN))
        {
            bke_curveprofile_move_point(profile, i, false, offset);
        } else {
            if pt_flag & PROF_H1_SELECT != 0 {
                bke_curveprofile_move_handle(&mut profile.path[i], true, false, Some(offset));
            }
            if pt_flag & PROF_H2_SELECT != 0 {
                bke_curveprofile_move_handle(&mut profile.path[i], false, false, Some(offset));
            }
        }
    }
}

/// `BKE_curveprofile_remove_point`.
pub fn bke_curveprofile_remove_point(profile: &mut CurveProfile, index: usize) -> bool {
    if profile.path.len() <= 2 {
        return false;
    }
    if index == 0 || index >= profile.path.len() {
        return false;
    }
    profile.path.remove(index);
    true
}

/// `BKE_curveprofile_remove_by_flag`.
pub fn bke_curveprofile_remove_by_flag(profile: &mut CurveProfile, flag: i16) {
    profile.path.retain(|p| p.flag & flag == 0);
}

/// `BKE_curveprofile_insert` — returns new point index.
pub fn bke_curveprofile_insert(profile: &mut CurveProfile, x: f32, y: f32) -> Option<usize> {
    let new_loc = [x, y];
    if profile.path.len() as i32 == PROF_TABLE_MAX - 1 {
        return None;
    }
    let mut min_distance = f32::MAX;
    let mut i_insert = 0usize;
    for i in 0..profile.path.len() - 1 {
        let loc1 = [profile.path[i].x, profile.path[i].y];
        let loc2 = [profile.path[i + 1].x, profile.path[i + 1].y];
        let distance = dist_squared_to_line_segment_v2(Vec2::new(new_loc[0],new_loc[1]), Vec2::new(loc1[0],loc1[1]), Vec2::new(loc2[0],loc2[1]));
        if distance < min_distance {
            min_distance = distance;
            i_insert = i + 1;
        }
    }
    for p in profile.path.iter_mut() {
        p.flag &=
            !(PROF_SELECT | PROF_H1_SELECT | PROF_H2_SELECT | PROF_ACTIVE | PROF_H1_ACTIVE
                | PROF_H2_ACTIVE);
    }
    let new_handle_type = if i_insert > 0
        && profile.path[i_insert - 1].h2 == HD_VECT
        && i_insert < profile.path.len()
        && profile.path[i_insert].h1 == HD_VECT
    {
        HD_VECT
    } else {
        HD_AUTO
    };
    let mut pt = point_init(x, y, new_handle_type, new_handle_type);
    pt.flag = PROF_SELECT;
    profile.path.insert(i_insert, pt);
    Some(i_insert)
}

/// `BKE_curveprofile_selected_handle_set`.
pub fn bke_curveprofile_selected_handle_set(profile: &mut CurveProfile, type_1: i8, type_2: i8) {
    for p in profile.path.iter_mut() {
        if p.flag & (PROF_SELECT | PROF_H1_SELECT | PROF_H2_SELECT) != 0 {
            p.h1 = type_1;
            p.h2 = type_2;
        }
    }
}

/// `mirror_point`.
fn mirror_point(point: &CurveProfilePoint) -> CurveProfilePoint {
    let mut np = point.clone();
    np.x = point.y;
    np.y = point.x;
    np.h1 = point.h2;
    np.h2 = point.h1;
    np
}

/// `BKE_curveprofile_reverse`.
pub fn bke_curveprofile_reverse(profile: &mut CurveProfile) {
    if profile.path.len() == 2 {
        return;
    }
    let old: Vec<CurveProfilePoint> = profile.path.clone();
    for (i, p) in old.iter().enumerate() {
        let i_rev = old.len() - i - 1;
        profile.path[i_rev] = mirror_point(p);
        if p.h1 == HD_FREE || p.h1 == HD_ALIGN {
            profile.path[i_rev].h1_loc = [p.h2_loc[1], p.h2_loc[0]];
        }
        if p.h2 == HD_FREE || p.h2 == HD_ALIGN {
            profile.path[i_rev].h2_loc = [p.h1_loc[1], p.h1_loc[0]];
        }
    }
}

/// `curveprofile_build_supports`.
fn curveprofile_build_supports(path: &mut Vec<CurveProfilePoint>) {
    let n = path.len() as i32;
    path[0] = point_init(1.0, 0.0, HD_VECT, HD_VECT);
    path[1] = point_init(1.0, 0.5, HD_VECT, HD_VECT);
    for i in 1..n - 2 {
        let t = i as f32 / (n - 3) as f32 * std::f32::consts::FRAC_PI_2;
        let x = 1.0 - 0.5 * (1.0 - t.cos());
        let y = 0.5 + 0.5 * t.sin();
        path[i as usize] = point_init(x, y, HD_AUTO, HD_AUTO);
    }
    path[(n - 2) as usize] = point_init(0.5, 1.0, HD_VECT, HD_VECT);
    path[(n - 1) as usize] = point_init(0.0, 1.0, HD_VECT, HD_VECT);
}

/// `curveprofile_build_steps`.
fn curveprofile_build_steps(path: &mut Vec<CurveProfilePoint>) {
    let n = path.len() as i32;
    if n == 2 {
        path[0] = point_init(1.0, 0.0, HD_VECT, HD_VECT);
        path[1] = point_init(0.0, 1.0, HD_VECT, HD_VECT);
        return;
    }
    let n_steps_x = if n % 2 == 0 { n } else { n - 1 } as f32;
    let n_steps_y = if n % 2 == 0 { n - 2 } else { n - 1 } as f32;
    for i in 0..n {
        let step_x = (i + 1) / 2;
        let step_y = i / 2;
        let x = 1.0 - (2.0 * step_x as f32 / n_steps_x);
        let y = 2.0 * step_y as f32 / n_steps_y;
        path[i as usize] = point_init(x, y, HD_VECT, HD_VECT);
    }
}

/// `BKE_curveprofile_reset` — presets verbatim.
pub fn bke_curveprofile_reset(profile: &mut CurveProfile) {
    match profile.preset {
        PROF_PRESET_LINE => {
            profile.path = vec![point_init(1.0, 0.0, HD_VECT, HD_VECT),
                                point_init(0.0, 1.0, HD_VECT, HD_VECT)];
        }
        PROF_PRESET_SUPPORTS => {
            profile.path = vec![CurveProfilePoint::default(); 17];
            curveprofile_build_supports(&mut profile.path);
        }
        PROF_PRESET_CORNERS => {
            profile.path = vec![CurveProfilePoint::default(); 9];
            curveprofile_build_supports(&mut profile.path);
        }
        PROF_PRESET_STEPS => {
            profile.path = vec![CurveProfilePoint::default(); 16];
            curveprofile_build_steps(&mut profile.path);
        }
        _ => {}
    }
}

/// `is_curved_edge`.
fn is_curved_edge(path: &[CurveProfilePoint], i: usize) -> bool {
    path[i].h2 != HD_VECT || path[i + 1].h1 != HD_VECT
}

/// `point_calculate_handle`.
fn point_calculate_handle(
    path: &mut [CurveProfilePoint],
    i: usize,
    has_prev: bool,
    has_next: bool,
) {
    let (x, y) = (path[i].x, path[i].y);
    let prev_loc = if has_prev {
        [path[i - 1].x, path[i - 1].y]
    } else {
        let nl = [path[i + 1].x, path[i + 1].y];
        [2.0 * x - nl[0], 2.0 * y - nl[1]]
    };
    let next_loc = if has_next {
        [path[i + 1].x, path[i + 1].y]
    } else {
        let pl = [path[i - 1].x, path[i - 1].y];
        [2.0 * x - pl[0], 2.0 * y - pl[1]]
    };
    if path[i].h1 == HD_FREE && path[i].h2 == HD_FREE {
        return;
    }
    let dvec_a = [x - prev_loc[0], y - prev_loc[1]];
    let dvec_b = [next_loc[0] - x, next_loc[1] - y];
    let mut len_a = (dvec_a[0] * dvec_a[0] + dvec_a[1] * dvec_a[1]).sqrt();
    let mut len_b = (dvec_b[0] * dvec_b[0] + dvec_b[1] * dvec_b[1]).sqrt();
    if len_a == 0.0 {
        len_a = 1.0;
    }
    if len_b == 0.0 {
        len_b = 1.0;
    }
    if path[i].h1 == HD_AUTO || path[i].h2 == HD_AUTO {
        let tvec = [dvec_b[0] / len_b + dvec_a[0] / len_a, dvec_b[1] / len_b + dvec_a[1] / len_a];
        let len = (tvec[0] * tvec[0] + tvec[1] * tvec[1]).sqrt() * 2.5614;
        if len != 0.0 {
            if path[i].h1 == HD_AUTO {
                let la = len_a / len;
                path[i].h1_loc = [x + tvec[0] * -la, y + tvec[1] * -la];
            }
            if path[i].h2 == HD_AUTO {
                let lb = len_b / len;
                path[i].h2_loc = [x + tvec[0] * lb, y + tvec[1] * lb];
            }
        }
    }
    if path[i].h1 == HD_VECT {
        path[i].h1_loc = [x + dvec_a[0] * -1.0 / 3.0, y + dvec_a[1] * -1.0 / 3.0];
    }
    if path[i].h2 == HD_VECT {
        path[i].h2_loc = [x + dvec_b[0] / 3.0, y + dvec_b[1] / 3.0];
    }
}

/// `calculate_path_handles`.
fn calculate_path_handles(path: &mut [CurveProfilePoint]) {
    let n = path.len();
    for i in 0..n {
        point_calculate_handle(path, i, i > 0, i < n - 1);
    }
}

/// `bezt_edge_handle_angle`.
fn bezt_edge_handle_angle(path: &[CurveProfilePoint], i_edge: usize) -> f32 {
    let sd = [
        path[i_edge].h2_loc[0] - path[i_edge].x,
        path[i_edge].h2_loc[1] - path[i_edge].y,
    ];
    let ed = [
        path[i_edge + 1].x - path[i_edge + 1].h1_loc[0],
        path[i_edge + 1].y - path[i_edge + 1].h1_loc[1],
    ];
    angle_v2v2(sd, ed)
}

/// forward-diff bezier for one channel → `n+1` samples.
fn forward_diff_bezier_vec(q0: f32, q1: f32, q2: f32, q3: f32, it: i32) -> Vec<f32> {
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
    let mut out = Vec::with_capacity(it as usize + 1);
    for _ in 0..=it {
        out.push(q0);
        q0 += q1;
        q1 += q2;
        q2 += q3;
    }
    out
}

/// `create_samples` — adaptive curvature-aware sampling.
fn create_samples(
    profile: &mut CurveProfile,
    n_segments: i32,
    sample_straight_edges: bool,
) -> Vec<CurveProfilePoint> {
    let path_len = profile.path.len();
    let totedges = path_len - 1;
    let mut path = profile.path.clone();
    calculate_path_handles(&mut path);

    let mut curve_sorted: Vec<(usize, f32)> = (0..totedges)
        .map(|i| (i, bezt_edge_handle_angle(&path, i)))
        .collect();
    curve_sorted.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });

    let mut n_samples = vec![0i16; totedges];
    let mut n_left: i32;
    if n_segments >= totedges as i32 {
        if sample_straight_edges {
            let n_common = n_segments / totedges as i32;
            n_left = n_segments % totedges as i32;
            if n_common > 0 {
                for i in 0..totedges {
                    n_samples[i] = n_common as i16;
                }
            }
        } else {
            let mut n_curved_edges = (0..totedges)
                .filter(|&i| is_curved_edge(&path, i))
                .count() as i32;
            n_curved_edges = if n_curved_edges == 0 {
                totedges as i32
            } else {
                n_curved_edges
            };
            n_left = n_segments - (totedges as i32 - n_curved_edges);
            let n_common = n_left / n_curved_edges;
            if n_common > 0 {
                for i in 0..totedges {
                    if is_curved_edge(&path, i) || n_curved_edges == totedges as i32 {
                        n_samples[i] += n_common as i16;
                    } else {
                        n_samples[i] = 1;
                    }
                }
            }
            n_left -= n_common * n_curved_edges;
        }
    } else {
        n_left = n_segments;
    }
    for i in 0..n_left.min(totedges as i32) {
        n_samples[curve_sorted[i as usize].0] += 1;
    }

    let mut r_samples = vec![CurveProfilePoint::default(); n_segments as usize];
    let mut i_sample = 0usize;
    for i in 0..totedges {
        if n_samples[i] > 0 {
            r_samples[i_sample].h1 = path[i].h1;
            r_samples[i_sample].h2 = path[i].h2;
            for j in i_sample + 1..(i_sample + n_samples[i] as usize).min(n_segments as usize)
            {
                r_samples[j].flag = 0;
                r_samples[j].h1 = HD_AUTO;
                r_samples[j].h2 = HD_AUTO;
            }
            let xs = forward_diff_bezier_vec(
                path[i].x,
                path[i].h2_loc[0],
                path[i + 1].h1_loc[0],
                path[i + 1].x,
                n_samples[i] as i32,
            );
            let ys = forward_diff_bezier_vec(
                path[i].y,
                path[i].h2_loc[1],
                path[i + 1].h1_loc[1],
                path[i + 1].y,
                n_samples[i] as i32,
            );
            for k in 0..=n_samples[i] as usize {
                if i_sample + k < r_samples.len() {
                    r_samples[i_sample + k].x = xs[k];
                    r_samples[i_sample + k].y = ys[k];
                }
            }
        }
        i_sample += n_samples[i] as usize;
    }
    r_samples
}

/// `BKE_curveprofile_table_size` — 16 samples per control point,
/// clamped to [1, PROF_TABLE_MAX].
pub fn bke_curveprofile_table_size(profile: &CurveProfile) -> i32 {
    let resolution = 16;
    ((profile.path.len() as i32 - 1) * resolution + 1).clamp(1, PROF_TABLE_MAX)
}

/// `curveprofile_distance_to_next_table_point`.
fn distance_to_next_table_point(table: &[CurveProfilePoint], i: usize) -> f32 {
    let (dx, dy) = (
        table[i + 1].x - table[i].x,
        table[i + 1].y - table[i].y,
    );
    (dx * dx + dy * dy).sqrt()
}

/// `curveprofile_total_length`.
fn curveprofile_total_length(table: &[CurveProfilePoint]) -> f32 {
    (0..table.len().saturating_sub(1))
        .map(|i| distance_to_next_table_point(table, i))
        .sum()
}

/// `create_samples_even_spacing`.
fn create_samples_even_spacing(
    profile: &mut CurveProfile,
    n_segments: i32,
) -> Vec<CurveProfilePoint> {
    let table = profile.table.clone();
    let total_length = curveprofile_total_length(&table);
    let segment_length = total_length / n_segments as f32;
    let mut distance_next = distance_to_next_table_point(&table, 0);
    let mut distance_prev = 0.0f32;
    let mut i_table = 0usize;
    let mut r_samples = vec![CurveProfilePoint::default(); n_segments as usize];
    r_samples[0].x = table[0].x;
    r_samples[0].y = table[0].y;
    for i in 1..n_segments as usize {
        let mut segment_left = segment_length;
        while distance_next < segment_left {
            segment_left -= distance_next;
            i_table += 1;
            distance_next = distance_to_next_table_point(&table, i_table);
            distance_prev = 0.0;
        }
        let factor = (distance_prev + segment_left) / (distance_prev + distance_next);
        r_samples[i].x =
            table[i_table].x + (table[i_table + 1].x - table[i_table].x) * factor;
        r_samples[i].y =
            table[i_table].y + (table[i_table + 1].y - table[i_table].y) * factor;
        distance_next -= segment_left;
        distance_prev += segment_left;
    }
    r_samples
}

/// `curveprofile_make_table`.
fn curveprofile_make_table(profile: &mut CurveProfile) {
    let n_samples = bke_curveprofile_table_size(profile);
    let mut new_table = vec![CurveProfilePoint::default(); n_samples as usize + 1];
    if n_samples > 1 {
        let samples = create_samples(profile, n_samples - 1, false);
        for (i, s) in samples.into_iter().enumerate() {
            new_table[i] = s;
        }
    }
    new_table[(n_samples - 1) as usize].x = 0.0;
    new_table[(n_samples - 1) as usize].y = 1.0;
    profile.table = new_table;
}

/// `curveprofile_make_segments_table`.
fn curveprofile_make_segments_table(profile: &mut CurveProfile) {
    let n_samples = profile.segments_len as i32;
    if n_samples <= 0 {
        return;
    }
    let samples = if profile.flag & PROF_SAMPLE_EVEN_LENGTHS != 0 {
        create_samples_even_spacing(profile, n_samples)
    } else {
        create_samples(profile, n_samples, profile.flag & PROF_SAMPLE_STRAIGHT_EDGES != 0)
    };
    let mut new_table = vec![CurveProfilePoint::default(); n_samples as usize + 1];
    for (i, s) in samples.into_iter().enumerate() {
        new_table[i] = s;
    }
    profile.segments = new_table;
}

/// `BKE_curveprofile_update`.
pub fn bke_curveprofile_update(profile: &mut CurveProfile, update_flags: i32) {
    profile.changed_timestamp += 1;
    let clipr = profile.clip_rect;
    if profile.flag & PROF_USE_CLIP != 0 {
        if update_flags & PROF_UPDATE_CLIP != 0 {
            for p in profile.path.iter_mut() {
                p.x = p.x.clamp(clipr.xmin, clipr.xmax);
                p.y = p.y.clamp(clipr.ymin, clipr.ymax);
            }
        }
        if rctf_size_x(&profile.view_rect) > rctf_size_x(&profile.clip_rect) {
            profile.view_rect.xmin = profile.clip_rect.xmin;
            profile.view_rect.xmax = profile.clip_rect.xmax;
        }
        if rctf_size_y(&profile.view_rect) > rctf_size_y(&profile.clip_rect) {
            profile.view_rect.ymin = profile.clip_rect.ymin;
            profile.view_rect.ymax = profile.clip_rect.ymax;
        }
    }
    let thresh = (0.01 * rctf_size_x(&clipr)).powi(2);
    if update_flags & PROF_UPDATE_REMOVE_DOUBLES != 0 && profile.path.len() > 2 {
        for i in 0..profile.path.len() - 1 {
            let dx = profile.path[i + 1].x - profile.path[i].x;
            let dy = profile.path[i + 1].y - profile.path[i].y;
            if dx * dx + dy * dy < thresh {
                if i == 0 {
                    let sel = profile.path[1].flag & (PROF_SELECT | PROF_ACTIVE);
                    profile.path[0].flag |= sel;
                    bke_curveprofile_remove_point(profile, 1);
                } else {
                    let sel = profile.path[i].flag & (PROF_SELECT | PROF_ACTIVE);
                    profile.path[i + 1].flag |= sel;
                    bke_curveprofile_remove_point(profile, i);
                }
                break;
            }
        }
    }
    curveprofile_make_table(profile);
    if profile.segments_len > 0 {
        curveprofile_make_segments_table(profile);
    }
}

/// `BKE_curveprofile_set_defaults`.
pub fn bke_curveprofile_set_defaults(profile: &mut CurveProfile) {
    profile.flag = PROF_USE_CLIP;
    profile.view_rect = Rctf {
        xmin: 0.0,
        xmax: 1.0,
        ymin: 0.0,
        ymax: 1.0,
    };
    profile.clip_rect = profile.view_rect;
    profile.path = vec![
        CurveProfilePoint {
            x: 1.0,
            y: 0.0,
            ..Default::default()
        },
        CurveProfilePoint {
            x: 1.0,
            y: 1.0,
            ..Default::default()
        },
    ];
    profile.changed_timestamp = 0;
}
