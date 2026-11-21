//! Port of `intern/curve_nurbs.cc` — NURBS knot generation,
//! breakpoint counting and basis-function evaluation cache
//! ('The NURBS Book' p. 70).
//!
//! `interpolate_to_evaluated` is generic over `f32`/`Vec3`/etc. via
//! the `Mixable` trait (`attribute_math::DefaultMixer`); untyped
//! `GSpan` dispatch lives on the caller side.

use blenders_dna::curves_types::{
    KnotsMode, NURBS_KNOT_MODE_BEZIER, NURBS_KNOT_MODE_CUSTOM, NURBS_KNOT_MODE_ENDPOINT,
    NURBS_KNOT_MODE_ENDPOINT_BEZIER,
};
use blenders_bli::index_range::IndexRange;
use std::ops::{AddAssign, Mul};

/// `knots_num` (`BKE_curves.hh`) — total knots incl. wrap.
pub fn knots_num(points_num: i32, order: i8, cyclic: bool) -> i32 {
    points_num + if cyclic { 2 * order as i32 - 1 } else { order as i32 }
}

/// `control_points_num` — wrapped (cyclic-padded) control points.
pub fn control_points_num(points_num: i32, order: i8, cyclic: bool) -> i32 {
    points_num + if cyclic { order as i32 - 1 } else { 0 }
}

/// `check_valid_eval_params`.
pub fn check_valid_eval_params(
    points_num: i32,
    order: i8,
    cyclic: bool,
    knots_mode: KnotsMode,
    resolution: i32,
) -> bool {
    if points_num < order as i32 {
        return false;
    }
    if order < 2 {
        return false;
    }
    if resolution < 1 {
        return false;
    }
    if knots_mode == NURBS_KNOT_MODE_BEZIER || knots_mode == NURBS_KNOT_MODE_ENDPOINT_BEZIER {
        if knots_mode == NURBS_KNOT_MODE_BEZIER && points_num <= order as i32 {
            return false;
        }
        return !cyclic || points_num % (order as i32 - 1) == 0;
    }
    true
}

/// `calc_nonzero_knot_spans` — analytic count for non-custom modes.
fn calc_nonzero_knot_spans(points_num: i32, mode: KnotsMode, order: i8, cyclic: bool) -> i32 {
    debug_assert!(order >= 2);
    let order = order as i32;
    let is_bezier = mode == NURBS_KNOT_MODE_BEZIER || mode == NURBS_KNOT_MODE_ENDPOINT_BEZIER;
    let is_end_point =
        mode == NURBS_KNOT_MODE_ENDPOINT || mode == NURBS_KNOT_MODE_ENDPOINT_BEZIER;
    /* Inner knots are always repeated once except on Bezier case. */
    let repeat_inner = if is_bezier { order - 1 } else { 1 };
    /* For non endpoint Bezier repeated knots are shifted by one. */
    let knots_before_geometry = order + i32::from(is_bezier && !is_end_point && order > 2);
    let knots_after_geometry = order - 1
        + if cyclic && mode == NURBS_KNOT_MODE_ENDPOINT {
            order - 2
        } else {
            0
        };

    let knots_total = knots_num(points_num, order as i8, cyclic);
    /* On these knots as parameters actual geometry is generated. */
    let geometry_knots = knots_total - knots_before_geometry - knots_after_geometry;
    /* `repeat_inner - 1` is added to `ceil`. */
    (geometry_knots + repeat_inner - 1) / repeat_inner
}

/// `is_breakpoint` — nonzero-length knot span.
fn is_breakpoint(knots: &[f32], knot_span: usize) -> bool {
    (knots[knot_span + 1] - knots[knot_span]) > 0.0
}

/// `count_nonzero_knot_spans` — count from explicit knots.
fn count_nonzero_knot_spans(points_num: i32, order: i32, cyclic: bool, knots: &[f32]) -> i32 {
    debug_assert!(points_num > 0);
    let degree = order - 1;
    let mut span_num = 0;
    let wrapped_points_num = control_points_num(points_num, order as i8, cyclic);
    for knot_span in degree..wrapped_points_num {
        span_num += i32::from(is_breakpoint(knots, knot_span as usize));
    }
    span_num
}

/// `calculate_evaluated_num`.
pub fn calculate_evaluated_num(
    points_num: i32,
    order: i8,
    cyclic: bool,
    resolution: i32,
    knots_mode: KnotsMode,
    knots: &[f32],
) -> i32 {
    if !check_valid_eval_params(points_num, order, cyclic, knots_mode, resolution) {
        return points_num;
    }
    let nonzero_span_num = if knots_mode == NURBS_KNOT_MODE_CUSTOM && !knots.is_empty() {
        count_nonzero_knot_spans(points_num, order as i32, cyclic, knots)
    } else {
        calc_nonzero_knot_spans(points_num, knots_mode, order, cyclic)
    };
    resolution * nonzero_span_num + i32::from(!cyclic)
}

/// `copy_custom_knots`.
fn copy_custom_knots(order: i8, cyclic: bool, custom_knots: &[f32], knots: &mut [f32]) {
    let order = order as usize;
    knots[..custom_knots.len()].copy_from_slice(custom_knots);
    if cyclic {
        let last_knot = *custom_knots.last().unwrap();
        let shift = last_knot - knots[order - 1];
        let n = knots.len();
        for knot in 0..order - 1 {
            knots[n - order + 1 + knot] = knots[order + knot] + shift;
        }
    }
}

/// `calculate_knots` — generated knot vector for all non-custom
/// modes.
pub fn calculate_knots(points_num: i32, mode: KnotsMode, order: i8, cyclic: bool, knots: &mut [f32]) {
    debug_assert_eq!(knots.len() as i32, knots_num(points_num, order, cyclic));

    let order = order as i32;
    let is_bezier = mode == NURBS_KNOT_MODE_BEZIER || mode == NURBS_KNOT_MODE_ENDPOINT_BEZIER;
    let is_end_point =
        mode == NURBS_KNOT_MODE_ENDPOINT || mode == NURBS_KNOT_MODE_ENDPOINT_BEZIER;
    /* Inner knots are always repeated once except on Bezier case. */
    let repeat_inner = if is_bezier { order - 1 } else { 1 };
    /* How many times to repeat 0.0 at the beginning of knot. */
    let mut r = if is_end_point {
        order - i32::from(cyclic)
    } else if is_bezier {
        repeat_inner.min(2)
    } else {
        1
    };
    /* Number of knots replicating widths of the starting knots.
     * Covers both Cyclic and EndPoint cases. */
    let tail = if cyclic {
        2 * order - 1
    } else if is_end_point {
        order
    } else {
        0
    };

    let mut current = 0.0f32;

    let offset = i32::from(is_end_point && cyclic);
    if offset != 0 {
        knots[0] = current;
        current += 1.0;
    }

    let size = knots.len() as i32;
    for i in offset..(size - offset - tail) {
        knots[i as usize] = current;
        r -= 1;
        if r == 0 {
            current += 1.0;
            r = repeat_inner;
        }
    }

    let tail_index = size - tail;
    for i in 0..tail {
        knots[(tail_index + i) as usize] = current + (knots[i as usize] - knots[0]);
    }
}

/// `load_curve_knots` — custom-copy or generate.
pub fn load_curve_knots(
    mode: KnotsMode,
    points_num: i32,
    order: i8,
    cyclic: bool,
    curve_knots: IndexRange,
    custom_knots: &[f32],
    knots: &mut [f32],
) {
    if mode == NURBS_KNOT_MODE_CUSTOM {
        debug_assert!(!custom_knots.is_empty());
        debug_assert!(curve_knots.size() > 0);
        let slice = &custom_knots
            [curve_knots.start() as usize..curve_knots.one_after_last() as usize];
        copy_custom_knots(order, cyclic, slice, knots);
    } else {
        calculate_knots(points_num, mode, order, cyclic, knots);
    }
}

/// `calculate_multiplicity_sequence`.
pub fn calculate_multiplicity_sequence(knots: &[f32]) -> Vec<i32> {
    let mut multiplicity = Vec::with_capacity(knots.len());

    let mut m = 1;
    for i in 1..knots.len() {
        /* Only consider multiplicity for exact matching values. */
        if knots[i - 1] == knots[i] {
            m += 1;
        } else {
            multiplicity.push(m);
            m = 1;
        }
    }
    multiplicity.push(m);
    multiplicity
}

/// `calculate_basis_for_point` — Cox-de Boor basis at `parameter`.
fn calculate_basis_for_point(
    knots: &[f32],
    degree: usize,
    parameter: f32,
    span_index: usize,
    r_weights: &mut [f32],
    r_start_index: &mut i32,
) {
    debug_assert!(degree >= 1);
    debug_assert!(span_index >= degree);
    debug_assert!(span_index + degree < knots.len());
    debug_assert!(knots[span_index + 1] > knots[span_index]);
    let order = degree + 1;

    *r_start_index = (span_index - degree) as i32;

    let mut left = [0.0f32; 12];
    let mut right = [0.0f32; 12];
    r_weights[0] = 1.0;

    for j in 1..=degree {
        left[j] = parameter - knots[span_index + 1 - j];
        right[j] = knots[span_index + j] - parameter;
        let mut saved = 0.0f32;
        for r in 0..j {
            let temp = r_weights[r] / (right[r + 1] + left[j - r]);
            r_weights[r] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }
        r_weights[j] = saved;
    }
    let _ = order;
}

/// `BasisCache` — per-eval-point basis weights + control start index.
#[derive(Default)]
pub struct BasisCache {
    pub weights: Vec<f32>,
    pub start_indices: Vec<i32>,
    /// Set by callers when params invalid → copy-through.
    pub invalid: bool,
}

/// `calculate_basis_cache` — fill `basis_cache` for every evaluated
/// point (span breakpoints × resolution + closing point).
pub fn calculate_basis_cache(
    points_num: i32,
    evaluated_num: i32,
    order: i8,
    resolution: i32,
    cyclic: bool,
    knots_mode: KnotsMode,
    knots: &[f32],
    basis_cache: &mut BasisCache,
) {
    debug_assert!(points_num > 0);

    let degree = order as i32 - 1;
    let wrapped_points_num = control_points_num(points_num, order, cyclic);

    basis_cache.weights = vec![0.0; (evaluated_num * order as i32) as usize];
    basis_cache.start_indices = vec![0; evaluated_num as usize];

    if evaluated_num == 0 {
        return;
    }

    if !check_valid_eval_params(points_num, order, cyclic, knots_mode, resolution) {
        basis_cache.invalid = true;
        return;
    }
    basis_cache.invalid = false;

    /* Find the 'span index' for each breakpoint that defines the
     * 'evaluated spans'. For repeated knots, with multiplicity > 1,
     * only the rightmost is considered a breakpoint as the spans
     * between repeated knot values are zero length! */
    let breakpoint_num = (evaluated_num - i32::from(!cyclic)) / resolution;
    let mut span_offsets = vec![0i32; breakpoint_num as usize];

    let mut breakpoint_count = 0usize;
    for span_index in degree..wrapped_points_num {
        if is_breakpoint(knots, span_index as usize) {
            span_offsets[breakpoint_count] = span_index;
            breakpoint_count += 1;
        }
    }
    debug_assert_eq!(breakpoint_count as i32, breakpoint_num);

    /* Build the basis cache, sampling each evaluated span at
     * intervals. */
    for (index, &span_index) in span_offsets.iter().enumerate() {
        let mut eval_point = index as i32 * resolution;
        let span_index = span_index as usize;

        let knot_delta = knots[span_index + 1] - knots[span_index];
        let knot_step = knot_delta / resolution as f32;
        debug_assert!(knot_delta > 0.0);

        for step in 0..resolution {
            let parameter = knots[span_index] + step as f32 * knot_step;
            let mut start_index = 0i32;
            let base = eval_point as usize * order as usize;
            calculate_basis_for_point(
                knots,
                degree as usize,
                parameter,
                span_index,
                &mut basis_cache.weights[base..base + order as usize],
                &mut start_index,
            );
            basis_cache.start_indices[eval_point as usize] = start_index;
            eval_point += 1;
        }
    }
    if !cyclic {
        let order_sz = order as usize;
        let n = basis_cache.weights.len();
        let mut start_index = 0i32;
        calculate_basis_for_point(
            knots,
            degree as usize,
            knots[wrapped_points_num as usize],
            *span_offsets.last().unwrap() as usize,
            &mut basis_cache.weights[n - order_sz..],
            &mut start_index,
        );
        *basis_cache.start_indices.last_mut().unwrap() = start_index;
    }
}

/// `attribute_math::DefaultMixer` interface for interpolation.
pub trait Mixable: Copy + Default + AddAssign + Mul<f32, Output = Self> {}

impl Mixable for f32 {}
impl Mixable for blenders_math::vector::Vec3 {}
impl Mixable for blenders_math::vector::Vec2 {}

/// `interpolate_to_evaluated<T>` — non-rational.
pub fn interpolate_to_evaluated<T: Mixable>(
    basis_cache: &BasisCache,
    order: i8,
    src: &[T],
    dst: &mut [T],
) {
    let order = order as usize;
    for (i, d) in dst.iter_mut().enumerate() {
        let point_weights =
            &basis_cache.weights[i * order..i * order + order];
        let mut acc = T::default();
        for (j, &w) in point_weights.iter().enumerate() {
            let point_index =
                ((basis_cache.start_indices[i] + j as i32) as usize) % src.len();
            acc += src[point_index] * w;
        }
        *d = acc;
    }
}

/// `interpolate_to_evaluated_rational<T>`.
pub fn interpolate_to_evaluated_rational<T: Mixable>(
    basis_cache: &BasisCache,
    order: i8,
    control_weights: &[f32],
    src: &[T],
    dst: &mut [T],
) {
    let order = order as usize;
    for (i, d) in dst.iter_mut().enumerate() {
        let point_weights =
            &basis_cache.weights[i * order..i * order + order];
        let mut acc = T::default();
        for (j, &w) in point_weights.iter().enumerate() {
            let point_index =
                ((basis_cache.start_indices[i] + j as i32) as usize) % src.len();
            let weight = w * control_weights[point_index];
            acc += src[point_index] * weight;
        }
        *d = acc;
    }
}

/// `interpolate_to_evaluated` dispatch — `basis_cache.invalid` →
/// copy-through; optional `control_weights` → rational path.
pub fn interpolate_to_evaluated_dispatch<T: Mixable>(
    basis_cache: &BasisCache,
    order: i8,
    control_weights: Option<&[f32]>,
    src: &[T],
    dst: &mut [T],
) {
    if basis_cache.invalid {
        dst.copy_from_slice(src);
        return;
    }
    debug_assert_eq!(dst.len(), basis_cache.start_indices.len());
    match control_weights {
        None => interpolate_to_evaluated(basis_cache, order, src, dst),
        Some(w) if w.is_empty() => interpolate_to_evaluated(basis_cache, order, src, dst),
        Some(w) => interpolate_to_evaluated_rational(basis_cache, order, w, src, dst),
    }
}
