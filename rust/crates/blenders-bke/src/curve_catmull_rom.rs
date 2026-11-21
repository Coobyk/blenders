//! Port of `intern/curve_catmull_rom.cc` — `bke::curves::catmull_rom`
//! uniform Catmull-Rom evaluation.

use blenders_math::vector::Vec4;
use std::ops::{Add, Mul};

/// `segments_num` — segments for `points_num` (cyclic adds closing seg).
pub fn segments_num(points_num: i32, cyclic: bool) -> i32 {
    if cyclic { points_num.max(0) } else { (points_num - 1).max(0) }
}

/// `calculate_evaluated_num` — evaluated point count for `resolution`.
pub fn calculate_evaluated_num(points_num: i32, cyclic: bool, resolution: i32) -> i32 {
    let points_per_segment = resolution.max(1);
    let eval_num = points_per_segment * segments_num(points_num, cyclic);
    if cyclic {
        eval_num.max(1)
    } else {
        eval_num + 1
    }
}

/// `calculate_basis` — uniform Catmull-Rom weights (a,b,c,d).
pub fn calculate_basis(parameter: f32) -> Vec4 {
    let t = parameter;
    let s = 1.0 - parameter;
    Vec4::new(
        -t * s * s,
        2.0 + t * t * (3.0 * t - 5.0),
        2.0 + s * s * (3.0 * s - 5.0),
        -s * t * t,
    )
}

/// `interpolate<T>` — Catmull-Rom basis weighted sum.
pub fn cr_interpolate<T>(a: T, b: T, c: T, d: T, parameter: f32) -> T
where
    T: Copy + Add<Output = T> + Mul<f32, Output = T>,
{
    let w = calculate_basis(parameter);
    a * w.x + b * w.y + c * w.z + d * w.w
}

/// `evaluate_segment` — write `dst.len()` samples of the `a..d` segment
/// (first sample is `b` verbatim).
fn evaluate_segment<T>(a: T, b: T, c: T, d: T, dst: &mut [T])
where
    T: Copy + Add<Output = T> + Mul<f32, Output = T>,
{
    let step = 1.0 / dst.len() as f32;
    dst[0] = b;
    for i in 1..dst.len() {
        dst[i] = cr_interpolate(a, b, c, d, i as f32 * step);
    }
}

/// `interpolate_to_evaluated(src, cyclic, resolution, dst)` —
/// `dst.len() == calculate_evaluated_num(src.len(), cyclic, resolution)`.
pub fn interpolate_to_evaluated<T>(src: &[T], cyclic: bool, resolution: i32, dst: &mut [T])
where
    T: Copy + Add<Output = T> + Mul<f32, Output = T>,
{
    let n = src.len();
    if n == 0 {
        return;
    }
    let points_per_segment = resolution.max(1) as usize;
    let range_fn = |segment_i: usize| segment_i * points_per_segment;
    if n == 1 {
        dst[0] = src[0];
        return;
    }
    let first = range_fn(0);
    if n == 2 {
        evaluate_segment(src[0], src[0], src[1], src[1], &mut dst[first..first + points_per_segment]);
        if cyclic {
            let last = range_fn(1);
            evaluate_segment(src[1], src[1], src[0], src[0], &mut dst[last..last + points_per_segment]);
        } else {
            *dst.last_mut().unwrap() = src[1];
        }
        return;
    }
    let second_to_last = range_fn(n - 2);
    let last = range_fn(n - 1);
    if cyclic {
        evaluate_segment(src[n - 1], src[0], src[1], src[2],
            &mut dst[first..first + points_per_segment]);
        evaluate_segment(src[n - 2], src[n - 1], src[n - 1], src[0],
            &mut dst[second_to_last..second_to_last + points_per_segment]);
        evaluate_segment(src[n - 1], src[n - 1], src[0], src[1],
            &mut dst[last..last + points_per_segment]);
    } else {
        evaluate_segment(src[0], src[0], src[1], src[2],
            &mut dst[first..first + points_per_segment]);
        evaluate_segment(src[n - 2], src[n - 1], src[n - 1], src[n - 1],
            &mut dst[second_to_last..second_to_last + points_per_segment]);
        *dst.last_mut().unwrap() = src[n - 1];
    }
    /* inner segments */
    for i in 1..n - 2 {
        let seg = range_fn(i);
        evaluate_segment(src[i - 1], src[i], src[i + 1], src[i + 2],
            &mut dst[seg..seg + points_per_segment]);
    }
}
