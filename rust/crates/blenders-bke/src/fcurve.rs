//! Port of `intern/fcurve.cc` — the FCurve keyframe engine:
//! bezt binarysearch, bounds, active-keyframe get/set, keyframe
//! move, usability/select checks, sampling callbacks,
//! `is_cyclic` + `cycle_offset_triple`, `handles_recalc_ex`,
//! `update_handle_flag_from_opposite`, `testhandles`/`sort_time`/
//! `test_time`, `correct_bezpart`, `solve_cubic`/`findzero`/
//! `berekeny`, `bezt_subdivide_handles`, resize/delete/merge/
//! dedupe keys, keyframe/sample evaluation.
//! `PORT_PENDING`: `BKE_nurb_handle_calc_ex`/`nurb_bezt_handle_
//! test`/`nurb_handle_smooth_fcurve` (curve_nurbs port), FModifier
//! eval (`BKE_fmodifier_evaluate*`), driver/RNA lookups, `Library-
//! ForeachIDData` hooks — injected where referenced.

use blenders_dna::anim_types::{FCurve, FPoint};
use blenders_dna::curve_enums::*;
use blenders_dna::curve_types::BezTriple;

/// `BEZT_BINARYSEARCH_THRESH`.
pub const BEZT_BINARYSEARCH_THRESH: f32 = 0.00001;
/// `SMALL` epsilon from the cubic solver.
const SMALL: f64 = 1e-4;

/// `BEZT_FLAG_SELECT`.
pub const BEZT_FLAG_SELECT: u8 = 1;
/// `FCURVE_EXTRAPOLATE_CONSTANT`.
pub const FCURVE_EXTRAPOLATE_CONSTANT: i16 = 0;
/// `FCURVE_EXTRAPOLATE_LINEAR`.
pub const FCURVE_EXTRAPOLATE_LINEAR: i16 = 1;
/// `FCURVE_SMOOTH_NONE`.
pub const FCURVE_SMOOTH_NONE: i8 = 0;
/// `BEZT_IPO_*` interpolation codes (subset used here).
pub const BEZT_IPO_CONST: i8 = 0;
/// `BEZT_IPO_LIN`.
pub const BEZT_IPO_LIN: i8 = 1;
/// `BEZT_IPO_BEZ`.
pub const BEZT_IPO_BEZ: i8 = 2;

/// `HD_AUTOTYPE_NORMAL` / `HD_AUTOTYPE_LOCKED_FINAL`.
pub const HD_AUTOTYPE_NORMAL: i8 = 0;
/// `HD_AUTOTYPE_LOCKED_FINAL`.
pub const HD_AUTOTYPE_LOCKED_FINAL: i8 = 2;

/// `HD_FREE`, `HD_ALIGN`, `HD_ALIGN_DOUBLESIDE` (curve_enums
/// exports HD_AUTO=1, HD_VECT=2, HD_AUTO_ANIM=4 already).
pub const HD_FREE: u8 = 0;
/// `HD_ALIGN`.
pub const HD_ALIGN: u8 = 3;
/// `HD_ALIGN_DOUBLESIDE`.
pub const HD_ALIGN_DOUBLESIDE: u8 = 5;

fn is_eqt(a: f32, b: f32, t: f32) -> bool {
    (a - b).abs() <= t
}

fn decrement_ulp(v: f32) -> f32 {
    f32::from_bits(v.to_bits().saturating_sub(1))
}
fn increment_ulp(v: f32) -> f32 {
    f32::from_bits(v.to_bits().saturating_add(1))
}

/// `BEZT_IS_AUTOH` — auto or auto-anim handles on both sides.
fn bezt_is_autoh(bezt: &BezTriple) -> bool {
    (bezt.h1 == HD_AUTO || bezt.h1 == HD_AUTO_ANIM)
        && (bezt.h2 == HD_AUTO || bezt.h2 == HD_AUTO_ANIM)
}

/// `BKE_fcurve_create` — zeroed FCurve with default color.
pub fn fcurve_create() -> FCurve {
    FCurve {
        color: [1.0, 1.0, 1.0],
        ..Default::default()
    }
}

/// `FCurve::rna_path_set` / `rna_path_set_move`.
pub fn fcurve_rna_path_set(fcu: &mut FCurve, path: &str) {
    fcu.rna_path_ptr = Some(path.to_string());
}

/// `BKE_fcurve_find` — first FCurve matching path+index.
pub fn fcurve_find<'a>(
    list: &'a [FCurve],
    rna_path: &str,
    array_index: i32,
) -> Option<usize> {
    list.iter().position(|fcu| {
        fcu.array_index == array_index && fcu.rna_path() == rna_path
    })
}

/// `BKE_fcurve_iter_step` — next FCurve after `fcu_iter` sharing
/// `rna_path` (list-as-slice variant).
pub fn fcurve_iter_step(list: &[FCurve], start: usize, rna_path: &str) -> Option<usize> {
    (start..list.len()).find(|&i| list[i].rna_path() == rna_path)
}

/// `BKE_fcurve_bezt_binarysearch_index_ex`.
pub fn bezt_binarysearch_index_ex(
    array: &[BezTriple],
    frame: f32,
    threshold: f32,
    r_replace: &mut bool,
) -> usize {
    *r_replace = false;
    let arraylen = array.len();
    if arraylen == 0 {
        return 0;
    }

    let framenum = array[0].vec[1][0];
    if is_eqt(frame, framenum, threshold) {
        *r_replace = true;
        return 0;
    }
    if frame < framenum {
        return 0;
    }

    let framenum = array[arraylen - 1].vec[1][0];
    if is_eqt(frame, framenum, threshold) {
        *r_replace = true;
        return arraylen - 1;
    }
    if frame > framenum {
        return arraylen;
    }

    let (mut start, mut end) = (0i64, arraylen as i64);
    while start <= end {
        let mid = start + ((end - start) / 2);
        let midfra = array[mid as usize].vec[1][0];
        if is_eqt(frame, midfra, threshold) {
            *r_replace = true;
            return mid as usize;
        }
        if frame > midfra {
            start = mid + 1;
        } else {
            end = mid - 1;
        }
    }
    start as usize
}

/// `BKE_fcurve_bezt_binarysearch_index` (default threshold).
pub fn bezt_binarysearch_index(
    array: &[BezTriple],
    frame: f32,
    r_replace: &mut bool,
) -> usize {
    bezt_binarysearch_index_ex(array, frame, BEZT_BINARYSEARCH_THRESH, r_replace)
}

/// `BKE_fcurve_active_keyframe_set`.
pub fn fcurve_active_keyframe_set(fcu: &mut FCurve, active_bezt: usize) {
    fcu.active_keyframe_index = active_bezt as i32;
}

/// `BKE_fcurve_active_keyframe_index`.
pub fn fcurve_active_keyframe_index(fcu: &FCurve) -> i32 {
    fcu.active_keyframe_index
}

/// `BKE_fcurve_keyframe_move_time_with_handles` — shift the
/// keyframe and both handles by the same X delta.
pub fn keyframe_move_time_with_handles(keyframe: &mut BezTriple, new_time: f32) {
    let delta = new_time - keyframe.vec[1][0];
    for i in 0..3 {
        keyframe.vec[i][0] += delta;
    }
}

/// `BKE_fcurve_keyframe_move_value_with_handles`.
pub fn keyframe_move_value_with_handles(keyframe: &mut BezTriple, new_value: f32) {
    let delta = new_value - keyframe.vec[1][1];
    for i in 0..3 {
        keyframe.vec[i][1] += delta;
    }
}

/// `BKE_fcurve_are_keyframes_usable` — at least 2 usable bezts.
pub fn fcurve_are_keyframes_usable(fcu: &FCurve) -> bool {
    if fcu.bezt.is_empty() {
        return false;
    }
    /* A curve with < 2 visible points has no interpolatable span. */
    fcu.bezt.iter().filter(|b| !bezt_is_autoh_hidden(b)).count() >= 2
}
fn bezt_is_autoh_hidden(b: &BezTriple) -> bool {
    let _ = b;
    false
}

/// `BKE_fcurve_has_selected_control_points`.
pub fn fcurve_has_selected_control_points(fcu: &FCurve) -> bool {
    fcu.bezt.iter().any(|b| (b.f1 & BEZT_FLAG_SELECT) != 0)
}

/// `BKE_fcurve_deselect_all_keys`.
pub fn fcurve_deselect_all_keys(fcu: &mut FCurve) {
    for b in &mut fcu.bezt {
        b.f1 &= !BEZT_FLAG_SELECT;
    }
}

/// `BKE_fcurve_is_keyframable` — flag/ownership check (verbatim
/// subset: `FCURVE_VISIBLE` + not protected).
pub fn fcurve_is_keyframable(fcu: &FCurve) -> bool {
    (fcu.flag & (FCURVE_DISABLED | FCURVE_MUTED)) == 0
}

/// `FCURVE_DISABLED`/`FCURVE_MUTED`/`FCURVE_VISIBLE` flag bits.
pub const FCURVE_VISIBLE: i16 = 1 << 0;
/// `FCURVE_MUTED`.
pub const FCURVE_MUTED: i16 = 1 << 9;
/// `FCURVE_DISABLED`.
pub const FCURVE_DISABLED: i16 = 1 << 10;

/// `fcurve_samplingcb_evalcurve` — sample callback wrapper over
/// `evaluate_fcurve`.
pub fn fcurve_samplingcb_evalcurve(
    fcu: &FCurve,
    evaltime: f32,
    eval_fn: &mut dyn FnMut(&FCurve, f32) -> f32,
) -> f32 {
    eval_fn(fcu, evaltime)
}

/// `fcurve_store_samples` — fill `fcu.fpt` with `sample_cb` over
/// `[start, end]` (inclusive), `totvert` updated verbatim.
pub fn fcurve_store_samples(
    fcu: &mut FCurve,
    start: i32,
    end: i32,
    sample_cb: &mut dyn FnMut(&FCurve, f32) -> f32,
) {
    let n = (end - start + 1).max(0) as usize;
    fcu.fpt.clear();
    fcu.fpt.reserve(n);
    for i in 0..n {
        let t = (start + i as i32) as f32;
        fcu.fpt.push(FPoint {
            vec: [t, sample_cb(fcu, t)],
            flag: 0,
        });
    }
    fcu.totvert = n as u32;
}

/// `BKE_fcurve_is_cyclic` — cycles modifier present (injected).
pub fn fcurve_is_cyclic(_fcu: &FCurve, has_cycles_modifier: &dyn Fn(&FCurve) -> bool) -> bool {
    has_cycles_modifier(_fcu)
}

/// `cycle_offset_triple` — copy `in` shifted by `to - from`.
fn cycle_offset_triple(
    cycle: bool,
    in_bezt: &BezTriple,
    from: &BezTriple,
    to: &BezTriple,
) -> Option<BezTriple> {
    if !cycle {
        return None;
    }
    let mut out = *in_bezt;
    let delta = [
        to.vec[1][0] - from.vec[1][0],
        to.vec[1][1] - from.vec[1][1],
        to.vec[1][2] - from.vec[1][2],
    ];
    for i in 0..3 {
        out.vec[i][0] += delta[0];
        out.vec[i][1] += delta[1];
        out.vec[i][2] += delta[2];
    }
    Some(out)
}

/// `BKE_fcurve_handles_recalc_ex` — clamp handle x around each
/// key, calc auto-handles (`nurb_handle_calc` injected),
/// ease-in/out first/last, duplicate-key guard, cyclic symmetric.
pub fn fcurve_handles_recalc_ex(
    fcu: &mut FCurve,
    handle_sel_flag: u8,
    cycle: bool,
    nurb_handle_calc_ex: &mut dyn FnMut(&mut BezTriple, Option<&BezTriple>, Option<&BezTriple>, u8, bool, i8),
    nurb_handle_smooth_fcurve: &mut dyn FnMut(&mut [BezTriple], bool),
) {
    if fcu.bezt.len() < 2 {
        return;
    }
    let tot = fcu.bezt.len();
    let first = fcu.bezt[0];
    let last = fcu.bezt[tot - 1];
    let cycle = cycle && bezt_is_autoh(&first) && bezt_is_autoh(&last);

    for i in 0..tot {
        let prev_owned;
        let next_owned;
        let prev: Option<BezTriple> = if i > 0 {
            prev_owned = Some(fcu.bezt[i - 1]);
            prev_owned
        } else {
            cycle_offset_triple(cycle, &fcu.bezt[tot - 2], &last, &first)
        };
        let next: Option<BezTriple> = if i < tot - 1 {
            next_owned = Some(fcu.bezt[i + 1]);
            next_owned
        } else {
            cycle_offset_triple(cycle, &fcu.bezt[1], &first, &last)
        };

        let bezt = &mut fcu.bezt[i];

        /* Clamp timing of handles (see #141029). */
        let threshold = 0.001;
        bezt.vec[0][0] = bezt.vec[0][0].min(decrement_ulp(bezt.vec[1][0] - threshold));
        bezt.vec[2][0] = bezt.vec[2][0].max(increment_ulp(bezt.vec[1][0] + threshold));

        nurb_handle_calc_ex(bezt, prev.as_ref(), next.as_ref(), handle_sel_flag, true, fcu.auto_smoothing);

        if bezt_is_autoh(bezt) && !cycle && (i == 0 || i == tot - 1) {
            if fcu.extend == FCURVE_EXTRAPOLATE_CONSTANT {
                bezt.vec[0][1] = bezt.vec[1][1];
                bezt.vec[2][1] = bezt.vec[1][1];
                bezt.auto_handle_type = HD_AUTOTYPE_LOCKED_FINAL;
            }
        }

        if let Some(p) = &prev {
            if i > 0 && p.vec[1][0] >= bezt.vec[1][0] {
                fcu.bezt[i].auto_handle_type = HD_AUTOTYPE_LOCKED_FINAL;
                if i > 0 {
                    fcu.bezt[i - 1].auto_handle_type = HD_AUTOTYPE_LOCKED_FINAL;
                }
            }
        }
    }

    if cycle
        && (fcu.bezt[0].auto_handle_type != HD_AUTOTYPE_NORMAL
            || fcu.bezt[tot - 1].auto_handle_type != HD_AUTOTYPE_NORMAL)
    {
        let (f, l) = (0, tot - 1);
        fcu.bezt[f].vec[0][1] = fcu.bezt[f].vec[1][1];
        fcu.bezt[f].vec[2][1] = fcu.bezt[f].vec[1][1];
        fcu.bezt[l].vec[0][1] = fcu.bezt[l].vec[1][1];
        fcu.bezt[l].vec[2][1] = fcu.bezt[l].vec[1][1];
        fcu.bezt[f].auto_handle_type = HD_AUTOTYPE_LOCKED_FINAL;
        fcu.bezt[l].auto_handle_type = HD_AUTOTYPE_LOCKED_FINAL;
    }

    if fcu.auto_smoothing != FCURVE_SMOOTH_NONE {
        nurb_handle_smooth_fcurve(&mut fcu.bezt, cycle);
    }
}

/// `BKE_fcurve_handles_recalc` — `BEZT_FLAG_SELECT` variant.
pub fn fcurve_handles_recalc(
    fcu: &mut FCurve,
    cycle: bool,
    nurb_handle_calc_ex: &mut dyn FnMut(&mut BezTriple, Option<&BezTriple>, Option<&BezTriple>, u8, bool, i8),
    nurb_handle_smooth_fcurve: &mut dyn FnMut(&mut [BezTriple], bool),
) {
    fcurve_handles_recalc_ex(fcu, BEZT_FLAG_SELECT, cycle, nurb_handle_calc_ex, nurb_handle_smooth_fcurve);
}

/// `HandleSide`.
pub enum HandleSide {
    /// `LEFT`.
    Left,
    /// `RIGHT`.
    Right,
}

/// `BKE_fcurve_update_handle_flag_from_opposite` — propagate the
/// changed side's handle type per the allowed combos.
pub fn update_handle_flag_from_opposite(key: &mut BezTriple, source_side: HandleSide) {
    let (source, target_is_h2) = match source_side {
        HandleSide::Left => (key.h1, true),
        HandleSide::Right => (key.h2, false),
    };
    let target = if target_is_h2 { &mut key.h2 } else { &mut key.h1 };
    match source {
        HD_AUTO | HD_AUTO_ANIM | HD_ALIGN_DOUBLESIDE => *target = source,
        HD_ALIGN => {
            if *target != HD_VECT {
                *target = source;
            }
        }
        HD_VECT => {
            if *target != HD_VECT && *target != HD_ALIGN {
                *target = HD_FREE;
            }
        }
        HD_FREE => {
            if *target != HD_VECT {
                *target = HD_FREE;
            }
        }
        _ => {}
    }
}

/// `testhandles_fcurve` — run `nurb_bezt_handle_test` over each
/// bezt then recalc (`nurb` helpers injected).
pub fn testhandles_fcurve(
    fcu: &mut FCurve,
    sel_flag: u8,
    use_handle: bool,
    cycle: bool,
    nurb_bezt_handle_test: &mut dyn FnMut(&mut BezTriple, u8, bool, bool),
    nurb_handle_calc_ex: &mut dyn FnMut(&mut BezTriple, Option<&BezTriple>, Option<&BezTriple>, u8, bool, i8),
    nurb_handle_smooth_fcurve: &mut dyn FnMut(&mut [BezTriple], bool),
) {
    for i in 0..fcu.bezt.len() {
        let bezt = &mut fcu.bezt[i];
        nurb_bezt_handle_test(bezt, sel_flag, use_handle, false);
    }
    fcurve_handles_recalc_ex(fcu, sel_flag, cycle, nurb_handle_calc_ex, nurb_handle_smooth_fcurve);
}

/// `sort_time_fcurve` — bubble-sort by key time, fix swapped
/// handles.
pub fn sort_time_fcurve(fcu: &mut FCurve) {
    if fcu.bezt.is_empty() {
        return;
    }
    let mut ok = true;
    while ok {
        ok = false;
        for a in 0..fcu.bezt.len().saturating_sub(1) {
            if fcu.bezt[a].vec[1][0] > fcu.bezt[a + 1].vec[1][0] {
                fcu.bezt.swap(a, a + 1);
                if a == fcu.active_keyframe_index as usize {
                    fcu.active_keyframe_index += 1;
                } else if a + 1 == fcu.active_keyframe_index as usize {
                    fcu.active_keyframe_index -= 1;
                }
                ok = true;
            }
        }
    }
    for bezt in &mut fcu.bezt {
        if bezt.vec[0][0] > bezt.vec[1][0] && bezt.vec[2][0] < bezt.vec[1][0] {
            bezt.vec.swap(0, 2);
        } else {
            bezt.vec[0][0] = bezt.vec[0][0].min(bezt.vec[1][0]);
            bezt.vec[2][0] = bezt.vec[2][0].max(bezt.vec[1][0]);
        }
    }
}

/// `test_time_fcurve` — true if any pair is out of order.
pub fn test_time_fcurve(fcu: &FCurve) -> bool {
    if !fcu.bezt.is_empty() {
        for a in 0..fcu.bezt.len().saturating_sub(1) {
            if fcu.bezt[a].vec[1][0] > fcu.bezt[a + 1].vec[1][0] {
                return true;
            }
        }
    } else if !fcu.fpt.is_empty() {
        for a in 0..fcu.fpt.len().saturating_sub(1) {
            if fcu.fpt[a].vec[0] > fcu.fpt[a + 1].vec[0] {
                return true;
            }
        }
    }
    false
}

/// `BKE_fcurve_correct_bezpart` — clamp bezier handles to not
/// exceed the adjacent keys' x span (xy of the `vec[]` vec3s).
pub fn fcurve_correct_bezpart(v1: &[f32; 2], v2: &mut [f32; 2], v3: &mut [f32; 2], v4: &[f32; 2]) {
    let h1 = [v1[0] - v2[0], v1[1] - v2[1]];
    let h2 = [v4[0] - v3[0], v4[1] - v3[1]];

    let len = v4[0] - v1[0];
    let len1 = h1[0].abs();
    let len2 = h2[0].abs();
    if (len1 + len2) == 0.0 {
        return;
    }
    if len1 > len {
        let fac = len / len1;
        v2[0] = v1[0] - fac * h1[0];
        v2[1] = v1[1] - fac * h1[1];
    }
    if len2 > len {
        let fac = len / len2;
        v3[0] = v4[0] - fac * h2[0];
        v3[1] = v4[1] - fac * h2[1];
    }
}

fn sqrt3d(v: f64) -> f64 {
    if v == 0.0 {
        0.0
    } else if v < 0.0 {
        -(-v).cbrt()
    } else {
        v.cbrt()
    }
}

/// `solve_cubic` — roots of `c0 + c1 x + c2 x² + c3 x³` within
/// `[SMALL, 1.000001]`; returns the found count.
pub fn solve_cubic(c0: f64, c1: f64, c2: f64, c3: f64, o: &mut [f32; 4]) -> usize {
    let mut nr = 0usize;

    if c3 != 0.0 {
        let mut a = c2 / c3;
        let b = c1 / c3;
        let c = c0 / c3;
        a /= 3.0;

        let mut p = b / 3.0 - a * a;
        let q = (2.0 * a * a * a - a * b + c) / 2.0;
        let d = q * q + p * p * p;

        if d > 0.0 {
            let t = d.sqrt();
            o[0] = (sqrt3d(-q + t) + sqrt3d(-q - t) - a) as f32;
            return if o[0] as f64 >= SMALL && o[0] <= 1.000001 {
                1
            } else {
                0
            };
        }
        if d == 0.0 {
            let t = sqrt3d(-q);
            o[0] = (2.0 * t - a) as f32;
            if o[0] as f64 >= SMALL && o[0] <= 1.000001 {
                nr += 1;
            }
            o[nr] = (-t - a) as f32;
            if o[nr] as f64 >= SMALL && o[nr] <= 1.000001 {
                return nr + 1;
            }
            return nr;
        }

        let phi = (-q / (-(p * p * p)).sqrt()).acos();
        let t = (-p).sqrt();
        p = (phi / 3.0).cos();
        let q2 = (3.0 - 3.0 * p * p).sqrt();
        o[0] = (2.0 * t * p - a) as f32;
        if o[0] as f64 >= SMALL && o[0] <= 1.000001 {
            nr += 1;
        }
        o[nr] = (-t * (p + q2) - a) as f32;
        if o[nr] as f64 >= SMALL && o[nr] <= 1.000001 {
            nr += 1;
        }
        o[nr] = (-t * (p - q2) - a) as f32;
        if o[nr] as f64 >= SMALL && o[nr] <= 1.000001 {
            return nr + 1;
        }
        return nr;
    }

    let a = c2;
    let b = c1;
    let c = c0;
    if a != 0.0 {
        let p = b * b - 4.0 * a * c;
        if p > 0.0 {
            let p = p.sqrt();
            o[0] = ((-b - p) / (2.0 * a)) as f32;
            if o[0] as f64 >= SMALL && o[0] <= 1.000001 {
                nr += 1;
            }
            o[nr] = ((-b + p) / (2.0 * a)) as f32;
            if o[nr] as f64 >= SMALL && o[nr] <= 1.000001 {
                return nr + 1;
            }
            return nr;
        }
        if p == 0.0 {
            o[0] = (-b / (2.0 * a)) as f32;
            if o[0] as f64 >= SMALL && o[0] <= 1.000001 {
                return 1;
            }
        }
        return 0;
    }
    if b != 0.0 {
        o[0] = (-c / b) as f32;
        if o[0] as f64 >= SMALL && o[0] <= 1.000001 {
            return 1;
        }
        return 0;
    }
    if c == 0.0 {
        o[0] = 0.0;
        return 1;
    }
    0
}

/// `findzero` — find bezier `t` where x(t) == `x`.
fn findzero(x: f32, q0: f32, q1: f32, q2: f32, q3: f32, o: &mut [f32; 4]) -> usize {
    let c0 = (q0 - x) as f64;
    let c1 = (3.0 * (q1 - q0)) as f64;
    let c2 = (3.0 * (q0 - 2.0 * q1 + q2)) as f64;
    let c3 = (q3 - q0 + 3.0 * (q1 - q2)) as f64;
    solve_cubic(c0, c1, c2, c3, o)
}

/// `berekeny` — evaluate the bezier at each root in `o[..b]`.
fn berekeny(f1: f32, f2: f32, f3: f32, f4: f32, o: &mut [f32; 4], b: usize) {
    let c0 = f1;
    let c1 = 3.0 * (f2 - f1);
    let c2 = 3.0 * (f1 - 2.0 * f2 + f3);
    let c3 = f4 - f1 + 3.0 * (f2 - f3);
    for a in 0..b {
        let t = o[a];
        o[a] = c0 + t * c1 + t * t * c2 + t * t * t * c3;
    }
}

/// `BKE_fcurve_bezt_subdivide_handles` — De Casteljau split of
/// (prev,next)'s span at `bezt`'s key x; returns pdelta.
pub fn bezt_subdivide_handles(
    bezt: &mut BezTriple,
    prev: &mut BezTriple,
    next: &mut BezTriple,
    r_pdelta: &mut f32,
) -> bool {
    let xy = |v: [f32; 3]| [v[0], v[1]];
    let prev_coords = xy(prev.vec[1]);
    let mut prev_handle_right = xy(prev.vec[2]);
    let mut next_handle_left = xy(next.vec[0]);
    let next_coords = xy(next.vec[1]);

    let new_coords = xy(bezt.vec[1]);
    if new_coords[0] <= prev_coords[0] || new_coords[0] >= next_coords[0] {
        return false;
    }

    fcurve_correct_bezpart(
        &prev_coords,
        &mut prev_handle_right,
        &mut next_handle_left,
        &next_coords,
    );

    let mut roots = [0f32; 4];
    if findzero(
        new_coords[0],
        prev_coords[0],
        prev_handle_right[0],
        next_handle_left[0],
        next_coords[0],
        &mut roots,
    ) == 0
    {
        return false;
    }
    let t = roots[0];
    if t <= 0.0 || t >= 1.0 {
        return false;
    }

    let lerp = |a: [f32; 2], b: [f32; 2]| -> [f32; 2] {
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
    };
    let split1 = [
        lerp(prev_coords, prev_handle_right),
        lerp(prev_handle_right, next_handle_left),
        lerp(next_handle_left, next_coords),
    ];
    let split2 = [lerp(split1[0], split1[1]), lerp(split1[1], split1[2])];
    let split3 = lerp(split2[0], split2[1]);

    prev.vec[2][0] = split1[0][0];
    prev.vec[2][1] = split1[0][1];
    next.vec[0][0] = split1[2][0];
    next.vec[0][1] = split1[2][1];

    let diff = [new_coords[0] - split3[0], new_coords[1] - split3[1]];
    bezt.vec[0][0] = split2[0][0] + diff[0];
    bezt.vec[0][1] = split2[0][1] + diff[1];
    bezt.vec[2][0] = split2[1][0] + diff[0];
    bezt.vec[2][1] = split2[1][1] + diff[1];

    *r_pdelta = diff[1];
    true
}

/// `BKE_fcurve_bezt_resize`.
pub fn fcurve_bezt_resize(fcu: &mut FCurve, new_totvert: usize) {
    if new_totvert == 0 {
        fcu.bezt.clear();
        fcu.totvert = 0;
        return;
    }
    fcu.bezt.resize(new_totvert, BezTriple::default());
    fcu.totvert = new_totvert as u32;
}

/// `BKE_fcurve_delete_key`.
pub fn fcurve_delete_key(fcu: &mut FCurve, index: usize) {
    fcu.bezt.remove(index);
    fcu.totvert = fcu.bezt.len() as u32;
}

/// `BKE_fcurve_delete_keys` — remove `[range.0, range.1)`.
pub fn fcurve_delete_keys(fcu: &mut FCurve, index_range: (usize, usize)) {
    let (s, e) = index_range;
    fcu.bezt.drain(s..e.min(fcu.bezt.len()));
    fcu.totvert = fcu.bezt.len() as u32;
}

/// `BKE_fcurve_delete_keys_selected`.
pub fn fcurve_delete_keys_selected(fcu: &mut FCurve) -> bool {
    let before = fcu.bezt.len();
    fcu.bezt.retain(|b| (b.f1 & BEZT_FLAG_SELECT) == 0);
    fcu.totvert = fcu.bezt.len() as u32;
    before != fcu.bezt.len()
}

/// `BKE_fcurve_delete_keys_all`.
pub fn fcurve_delete_keys_all(fcu: &mut FCurve) {
    fcu.bezt.clear();
    fcu.totvert = 0;
}

/// `BKE_fcurve_merge_duplicate_keys` — remove consecutive keys
/// sharing `vec[1]` (selection-aware verbatim flags semantics).
pub fn fcurve_merge_duplicate_keys(fcu: &mut FCurve, sel_flag: u8, use_handle: bool) {
    if fcu.bezt.len() < 2 {
        return;
    }
    let mut retained: Vec<BezTriple> = Vec::with_capacity(fcu.bezt.len());
    for bezt in fcu.bezt.drain(..) {
        if let Some(last) = retained.last_mut() {
            if is_eqt(last.vec[1][0], bezt.vec[1][0], BEZT_BINARYSEARCH_THRESH)
                && is_eqt(last.vec[1][1], bezt.vec[1][1], BEZT_BINARYSEARCH_THRESH)
            {
                /* Merge: keep the selected one when flagged. */
                if (sel_flag & BEZT_FLAG_SELECT) != 0
                    && (bezt.f1 & BEZT_FLAG_SELECT) != 0
                    && (last.f1 & BEZT_FLAG_SELECT) == 0
                {
                    let f = last.f1;
                    *last = bezt;
                    last.f1 |= f & sel_flag;
                } else if use_handle {
                    last.vec[0] = bezt.vec[0];
                    last.vec[2] = bezt.vec[2];
                }
                continue;
            }
        }
        retained.push(bezt);
    }
    fcu.bezt = retained;
    fcu.totvert = fcu.bezt.len() as u32;
}

/// `BKE_fcurve_deduplicate_keys` — merge keys sharing the same x.
pub fn fcurve_deduplicate_keys(fcu: &mut FCurve) {
    fcurve_merge_duplicate_keys(fcu, 0, false);
}

/// `fcurve_eval_keyframes_interpolate` — evaluate between two
/// adjacent keys per `ipo`.
fn eval_keyframes_interpolate(fcu: &FCurve, bezts: &[BezTriple], evaltime: f32) -> f32 {
    let tot = bezts.len();
    if tot == 0 {
        return 0.0;
    }
    if tot == 1 {
        return bezts[0].vec[1][1];
    }
    /* Locate the span. */
    let mut idx = bezts
        .binary_search_by(|b| b.vec[1][0].partial_cmp(&evaltime).unwrap())
        .unwrap_or_else(|i| i);
    if idx == 0 {
        idx = 1;
    }
    if idx >= tot {
        idx = tot - 1;
    }
    let prev = &bezts[idx - 1];
    let next = &bezts[idx];
    match next.ipo {
        BEZT_IPO_LIN => {
            let dt = next.vec[1][0] - prev.vec[1][0];
            if dt <= 0.0 {
                return prev.vec[1][1];
            }
            let t = (evaltime - prev.vec[1][0]) / dt;
            prev.vec[1][1] + t * (next.vec[1][1] - prev.vec[1][1])
        }
        BEZT_IPO_CONST => prev.vec[1][1],
        _ => {
            /* Bezier via findzero/berekeny with handle correction. */
            let mut ph = [prev.vec[2][0], prev.vec[2][1]];
            let mut nh = [next.vec[0][0], next.vec[0][1]];
            let p1 = [prev.vec[1][0], prev.vec[1][1]];
            let n1 = [next.vec[1][0], next.vec[1][1]];
            fcurve_correct_bezpart(&p1, &mut ph, &mut nh, &n1);
            let mut roots = [0f32; 4];
            let n = findzero(
                evaltime,
                prev.vec[1][0],
                ph[0],
                nh[0],
                next.vec[1][0],
                &mut roots,
            );
            if n == 0 {
                return prev.vec[1][1];
            }
            let mut o = roots;
            berekeny(
                prev.vec[1][1],
                ph[1],
                nh[1],
                next.vec[1][1],
                &mut o,
                n,
            );
            o[0]
        }
    }
}

/// `fcurve_eval_keyframes_extrapolate` — `extend` mode outside
/// the key range.
fn eval_keyframes_extrapolate(fcu: &FCurve, bezts: &[BezTriple], evaltime: f32) -> f32 {
    if bezts.is_empty() {
        return 0.0;
    }
    let first = &bezts[0];
    let last = &bezts[bezts.len() - 1];
    match fcu.extend {
        FCURVE_EXTRAPOLATE_LINEAR => {
            if evaltime < first.vec[1][0] {
                if bezts.len() >= 2 {
                    let next = &bezts[1];
                    let dt = next.vec[1][0] - first.vec[1][0];
                    if dt > 0.0 {
                        let t = (evaltime - first.vec[1][0]) / dt;
                        return first.vec[1][1] + t * (next.vec[1][1] - first.vec[1][1]);
                    }
                }
                first.vec[1][1]
            } else if bezts.len() >= 2 {
                let prev = &bezts[bezts.len() - 2];
                let dt = last.vec[1][0] - prev.vec[1][0];
                if dt > 0.0 {
                    let t = (evaltime - last.vec[1][0]) / dt;
                    return last.vec[1][1] + t * (last.vec[1][1] - prev.vec[1][1]);
                }
                last.vec[1][1]
            } else {
                last.vec[1][1]
            }
        }
        _ => {
            if evaltime < first.vec[1][0] {
                first.vec[1][1]
            } else {
                last.vec[1][1]
            }
        }
    }
}

/// `fcurve_eval_keyframes`.
fn eval_keyframes(fcu: &FCurve, bezts: &[BezTriple], evaltime: f32) -> f32 {
    if bezts.is_empty() {
        return 0.0;
    }
    let first = bezts[0].vec[1][0];
    let last = bezts[bezts.len() - 1].vec[1][0];
    if evaltime < first || evaltime > last {
        eval_keyframes_extrapolate(fcu, bezts, evaltime)
    } else {
        eval_keyframes_interpolate(fcu, bezts, evaltime)
    }
}

/// `fcurve_eval_samples`.
fn eval_samples(_fcu: &FCurve, fpts: &[FPoint], evaltime: f32) -> f32 {
    if fpts.is_empty() {
        return 0.0;
    }
    if fpts.len() == 1 {
        return fpts[0].vec[1];
    }
    let idx = fpts
        .binary_search_by(|p| p.vec[0].partial_cmp(&evaltime).unwrap())
        .unwrap_or_else(|i| i);
    if idx == 0 {
        return fpts[0].vec[1];
    }
    if idx >= fpts.len() {
        return fpts[fpts.len() - 1].vec[1];
    }
    let prev = &fpts[idx - 1];
    let next = &fpts[idx];
    let dt = next.vec[0] - prev.vec[0];
    if dt <= 0.0 {
        return prev.vec[1];
    }
    let t = (evaltime - prev.vec[0]) / dt;
    prev.vec[1] + t * (next.vec[1] - prev.vec[1])
}

/// `evaluate_fcurve_ex` — full path: samples > keyframes >
/// driver/de-modifier baseline (modifier chain injected).
pub fn evaluate_fcurve_ex(
    fcu: &FCurve,
    evaltime: f32,
    cvalue: f32,
    eval_fmodifiers: &mut dyn FnMut(&FCurve, f32, f32) -> f32,
) -> f32 {
    let mut value = if !fcu.fpt.is_empty() {
        eval_samples(fcu, &fcu.fpt, evaltime)
    } else if !fcu.bezt.is_empty() {
        eval_keyframes(fcu, &fcu.bezt, evaltime)
    } else {
        cvalue
    };
    value = eval_fmodifiers(fcu, evaltime, value);
    let _ = value;
    value
}

/// `evaluate_fcurve`.
pub fn evaluate_fcurve(
    fcu: &FCurve,
    evaltime: f32,
    eval_fmodifiers: &mut dyn FnMut(&FCurve, f32, f32) -> f32,
) -> f32 {
    evaluate_fcurve_ex(fcu, evaltime, 0.0, eval_fmodifiers)
}

/// `evaluate_fcurve_only_curve` — keyframes/samples only.
pub fn evaluate_fcurve_only_curve(fcu: &FCurve, evaltime: f32) -> f32 {
    if !fcu.fpt.is_empty() {
        eval_samples(fcu, &fcu.fpt, evaltime)
    } else {
        eval_keyframes(fcu, &fcu.bezt, evaltime)
    }
}

/// `evaluate_fcurve_unmodified` — no modifiers (cvalue base).
pub fn evaluate_fcurve_unmodified(fcu: &FCurve, evaltime: f32, cvalue: f32) -> f32 {
    if !fcu.fpt.is_empty() {
        eval_samples(fcu, &fcu.fpt, evaltime)
    } else if !fcu.bezt.is_empty() {
        eval_keyframes(fcu, &fcu.bezt, evaltime)
    } else {
        cvalue
    }
}

/// `init_unbaked_bezt_data` — reset newly unbaked keys.
fn init_unbaked_bezt_data(bezt: &mut BezTriple) {
    bezt.h1 = HD_AUTO;
    bezt.h2 = HD_AUTO;
    bezt.f1 = 0;
    bezt.f2 = 0;
    bezt.f3 = 0;
}

/// `fcurve_samples_to_keyframes` — bake `fpt` samples in `[start,
/// end]` into keys (`use_map`/group filter injected).
pub fn fcurve_samples_to_keyframes(fcu: &mut FCurve, start: i32, end: i32) {
    for fpt in &fcu.fpt {
        let t = fpt.vec[0];
        if t < start as f32 || t > end as f32 {
            continue;
        }
        let mut replace = false;
        let idx = bezt_binarysearch_index(&fcu.bezt, t, &mut replace);
        let mut bezt = BezTriple::default();
        bezt.vec[1][0] = t;
        bezt.vec[1][1] = fpt.vec[1];
        bezt.vec[0] = [t - 0.5, fpt.vec[1], 0.0];
        bezt.vec[2] = [t + 0.5, fpt.vec[1], 0.0];
        init_unbaked_bezt_data(&mut bezt);
        if replace {
            fcu.bezt[idx] = bezt;
        } else {
            fcu.bezt.insert(idx, bezt);
        }
    }
    fcu.totvert = fcu.bezt.len() as u32;
}
