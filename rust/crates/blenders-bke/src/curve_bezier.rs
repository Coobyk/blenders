//! Port of `intern/curve_bezier.cc` + the `bezier::` inlines in
//! `BKE_curves.hh` — Bezier handle math, evaluated offsets, knot
//! insertion, forward-differenced segment evaluation.

use blenders_bli::index_mask::IndexMask;
use blenders_bli::offset_indices::OffsetIndices;
use blenders_dna::curves_types::{
    BEZIER_HANDLE_ALIGN, BEZIER_HANDLE_AUTO, BEZIER_HANDLE_FREE, BEZIER_HANDLE_VECTOR,
};
use blenders_math::vector::{Vec2, Vec3};

/// `point_is_sharp` inline.
pub fn point_is_sharp(handle_types_left: &[i8], handle_types_right: &[i8], index: usize) -> bool {
    [BEZIER_HANDLE_VECTOR, BEZIER_HANDLE_FREE].contains(&handle_types_left[index])
        || [BEZIER_HANDLE_VECTOR, BEZIER_HANDLE_FREE].contains(&handle_types_right[index])
}

/// `segment_is_vector(left, right)` inline.
pub fn segment_is_vector_lr(left: i8, right: i8) -> bool {
    left == BEZIER_HANDLE_VECTOR && right == BEZIER_HANDLE_VECTOR
}

/// `segment_is_vector(types, types, segment_index)`.
pub fn segment_is_vector(
    handle_types_left: &[i8],
    handle_types_right: &[i8],
    segment_index: usize,
) -> bool {
    debug_assert!(segment_index < handle_types_left.len() - 1);
    segment_is_vector_lr(
        handle_types_right[segment_index],
        handle_types_left[segment_index + 1],
    )
}

/// `last_cyclic_segment_is_vector`.
pub fn last_cyclic_segment_is_vector(handle_types_left: &[i8], handle_types_right: &[i8]) -> bool {
    segment_is_vector_lr(
        *handle_types_right.last().unwrap(),
        handle_types_left[0],
    )
}

/// `segments_num` inline.
pub fn segments_num(points_num: usize, cyclic: bool) -> usize {
    points_num.saturating_sub(!cyclic as usize)
}

/// `has_vector_handles` inline.
pub fn has_vector_handles(
    num_curve_points: usize,
    evaluated_size: usize,
    cyclic: bool,
    resolution: i32,
) -> bool {
    evaluated_size - (!cyclic as usize)
        != segments_num(num_curve_points, cyclic) * resolution.max(0) as usize
}

/// `calculate_evaluated_offsets`.
pub fn calculate_evaluated_offsets(
    handle_types_left: &[i8],
    handle_types_right: &[i8],
    cyclic: bool,
    resolution: i32,
    evaluated_offsets: &mut [i32],
) {
    let size = handle_types_left.len();
    debug_assert_eq!(evaluated_offsets.len(), size + 1);

    evaluated_offsets[0] = 0;
    if size == 1 {
        *evaluated_offsets.last_mut().unwrap() = 1;
        return;
    }

    let points_per_segment = resolution.max(1);

    let mut offset = 0i32;
    for i in 0..size - 1 {
        evaluated_offsets[i] = offset;
        offset += if segment_is_vector(handle_types_left, handle_types_right, i) {
            1
        } else {
            points_per_segment
        };
    }

    evaluated_offsets[size - 1] = offset;
    if cyclic {
        offset += if last_cyclic_segment_is_vector(handle_types_left, handle_types_right) {
            1
        } else {
            points_per_segment
        };
    } else {
        offset += 1;
    }

    *evaluated_offsets.last_mut().unwrap() = offset;
}

/// `Insertion` — knot insertion result.
pub struct Insertion {
    pub handle_prev: Vec3,
    pub left_handle: Vec3,
    pub position: Vec3,
    pub right_handle: Vec3,
    pub handle_next: Vec3,
}

/// `insert` — de Casteljau subdivision at `parameter`.
pub fn insert(
    point_prev: Vec3,
    handle_prev: Vec3,
    handle_next: Vec3,
    point_next: Vec3,
    parameter: f32,
) -> Insertion {
    debug_assert!((0.0..=1.0).contains(&parameter));
    let lerp = |a: Vec3, b: Vec3| a + (b - a) * parameter;

    let center_point = lerp(handle_prev, handle_next);
    let handle_prev = lerp(point_prev, handle_prev);
    let handle_next = lerp(handle_next, point_next);
    let left_handle = lerp(handle_prev, center_point);
    let right_handle = lerp(center_point, handle_next);
    let position = lerp(left_handle, right_handle);
    Insertion {
        handle_prev,
        left_handle,
        position,
        right_handle,
        handle_next,
    }
}

/// `calculate_vector_handle` inline.
pub fn calculate_vector_handle(point: Vec3, next_point: Vec3) -> Vec3 {
    point + (next_point - point) * (1.0 / 3.0)
}

/// `calculate_aligned_handle` — keep the opposite handle directly
/// opposite, preserving its length.
fn calculate_aligned_handle(position: Vec3, other_handle: Vec3, aligned_handle: Vec3) -> Vec3 {
    /* Keep track of the old length of the opposite handle. */
    let length = (aligned_handle - position).length();
    /* Set the other handle to directly opposite from the current
     * handle. */
    let dir = (other_handle - position).normalized();
    if dir.length_squared() == 0.0 {
        /* There is no direction to align against when the opposite
         * handle coincides with the control point, so preserve the
         * existing aligned handle. */
        return aligned_handle;
    }
    position - dir * length
}

/// `calculate_align_both_handles` → `(new_left, new_right)`; the new
/// handles stay in the plane of the old ones, lengths preserved.
fn calculate_align_both_handles(
    position: Vec3,
    left_handle: Vec3,
    right_handle: Vec3,
) -> (Vec3, Vec3) {
    let left_dir = left_handle - position;
    let right_dir = right_handle - position;

    /* Keep track of the lengths of both handles. */
    let left_length = left_dir.length();
    let right_length = right_dir.length();

    if left_length == 0.0 && right_length == 0.0 {
        /* All three points are the same, no clear way to fixed it. */
        return (left_handle, right_handle);
    }
    /* If one handle has zero length, use the other as the length and
     * direction. */
    if left_length == 0.0 {
        return (position - right_dir, right_handle);
    }
    if right_length == 0.0 {
        return (left_handle, position - left_dir);
    }

    /* Use the direction halfway between the two directions. */
    let align_dir = left_dir.normalized() + right_dir.normalized();

    let align_length = align_dir.length();
    if align_length <= 0.0001 * (left_length + right_length) {
        /* The handles are already aligned. */
        return (left_handle, right_handle);
    }

    /* Normalize. */
    let align_dir = align_dir / align_length;

    /* Project the directions onto the plane formed by `align_dir`. */
    let new_left_dir = left_dir - align_dir * left_dir.dot(align_dir);
    let new_right_dir = right_dir - align_dir * right_dir.dot(align_dir);

    /* Use the new directions with the old lengths. */
    (
        position + new_left_dir.normalized() * left_length,
        position + new_right_dir.normalized() * right_length,
    )
}

/// `calculate_point_handles`.
#[allow(clippy::too_many_arguments)]
fn calculate_point_handles(
    type_left: i8,
    type_right: i8,
    position: Vec3,
    prev_position: Vec3,
    next_position: Vec3,
    left: &mut Vec3,
    right: &mut Vec3,
) {
    if [BEZIER_HANDLE_AUTO].contains(&type_left) || [BEZIER_HANDLE_AUTO].contains(&type_right) {
        let prev_diff = position - prev_position;
        let next_diff = next_position - position;
        let mut prev_len = prev_diff.length();
        let mut next_len = next_diff.length();
        if prev_len == 0.0 {
            prev_len = 1.0;
        }
        if next_len == 0.0 {
            next_len = 1.0;
        }
        let dir = next_diff / next_len + prev_diff / prev_len;

        /* The magic number 2.5614 is derived from approximating a
         * circular arc at the control point (see C comment). */
        let len = dir.length() * 2.5614;
        if len != 0.0 {
            if type_left == BEZIER_HANDLE_AUTO {
                let prev_len_clamped = prev_len.min(next_len * 5.0);
                *left = position + dir * -(prev_len_clamped / len);
            }
            if type_right == BEZIER_HANDLE_AUTO {
                let next_len_clamped = next_len.min(prev_len * 5.0);
                *right = position + dir * (next_len_clamped / len);
            }
        }
    }

    if type_left == BEZIER_HANDLE_VECTOR {
        *left = calculate_vector_handle(position, prev_position);
    }

    if type_right == BEZIER_HANDLE_VECTOR {
        *right = calculate_vector_handle(position, next_position);
    }

    /* When one of the handles is "aligned" handle, it must be
     * aligned with the other, i.e. point in the opposite direction. */
    if type_left == BEZIER_HANDLE_ALIGN && type_right != BEZIER_HANDLE_ALIGN {
        *left = calculate_aligned_handle(position, *right, *left);
    } else if type_left != BEZIER_HANDLE_ALIGN && type_right == BEZIER_HANDLE_ALIGN {
        *right = calculate_aligned_handle(position, *left, *right);
    }
}

/// `set_handle_position`.
pub fn set_handle_position(
    position: Vec3,
    type_: i8,
    type_other: i8,
    new_handle: Vec3,
    handle: &mut Vec3,
    handle_other: &mut Vec3,
) {
    /* Don't bother when the handle positions are calculated
     * automatically anyway. */
    if [BEZIER_HANDLE_AUTO, BEZIER_HANDLE_VECTOR].contains(&type_) {
        return;
    }

    *handle = new_handle;
    if type_other == BEZIER_HANDLE_ALIGN {
        *handle_other = calculate_aligned_handle(position, *handle, *handle_other);
    }
}

/// `calculate_single_aligned_handles`.
pub fn calculate_single_aligned_handles(
    selection: &IndexMask,
    positions: &[Vec3],
    align_with: &[Vec3],
    align_handles: &mut [Vec3],
) {
    selection.iter().for_each(|point| {
        let point = point as usize;
        align_handles[point] = calculate_aligned_handle(
            positions[point],
            align_with[point],
            align_handles[point],
        );
    });
}

/// `calculate_aligned_handles`.
pub fn calculate_aligned_handles(
    selection: &IndexMask,
    positions: &[Vec3],
    handles_left: &[Vec3],
    handles_right: &[Vec3],
    align_handles_left: &mut [Vec3],
    align_handles_right: &mut [Vec3],
) {
    selection.iter().for_each(|point| {
        let point = point as usize;
        let (new_left, new_right) = calculate_align_both_handles(
            positions[point],
            handles_left[point],
            handles_right[point],
        );
        align_handles_left[point] = new_left;
        align_handles_right[point] = new_right;
    });
}

/// `calculate_auto_handles` — recalc all AUTO/VECTOR handles and
/// align ALIGN handles.
pub fn calculate_auto_handles(
    cyclic: bool,
    types_left: &[i8],
    types_right: &[i8],
    positions: &[Vec3],
    positions_left: &mut [Vec3],
    positions_right: &mut [Vec3],
) {
    let points_num = positions.len();
    if points_num == 1 {
        return;
    }

    let last = points_num - 1;
    calculate_point_handles(
        types_left[0],
        types_right[0],
        positions[0],
        if cyclic {
            positions[last]
        } else {
            positions[0] * 2.0 - positions[1]
        },
        positions[1],
        &mut positions_left[0],
        &mut positions_right[0],
    );

    for i in 1..points_num - 1 {
        calculate_point_handles(
            types_left[i],
            types_right[i],
            positions[i],
            positions[i - 1],
            positions[i + 1],
            &mut positions_left[i],
            &mut positions_right[i],
        );
    }

    calculate_point_handles(
        types_left[last],
        types_right[last],
        positions[last],
        positions[last - 1],
        if cyclic {
            positions[0]
        } else {
            positions[last] * 2.0 - positions[last - 1]
        },
        &mut positions_left[last],
        &mut positions_right[last],
    );
}

/// `evaluate_segment_ex` — forward-difference cubic eval.
pub trait BezierEval: Copy + std::ops::Add<Output = Self> + std::ops::Sub<Output = Self>
    + std::ops::Mul<f32, Output = Self> + std::ops::Div<f32, Output = Self> + std::ops::AddAssign
{
}

impl BezierEval for Vec3 {}
impl BezierEval for Vec2 {}

/// `evaluate_segment` — cubic via forward differencing.
pub fn evaluate_segment<T: BezierEval>(
    point_0: T,
    point_1: T,
    point_2: T,
    point_3: T,
    result: &mut [T],
) {
    debug_assert!(!result.is_empty());
    let inv_len = 1.0 / result.len() as f32;
    let inv_len_squared = inv_len * inv_len;
    let inv_len_cubed = inv_len_squared * inv_len;

    let rt1 = (point_1 - point_0) * (3.0 * inv_len);
    let rt2 = (point_0 - point_1 * 2.0 + point_2) * (3.0 * inv_len_squared);
    let rt3 = (point_3 - point_0 + (point_1 - point_2) * 3.0) * inv_len_cubed;

    let mut q0 = point_0;
    let mut q1 = rt1 + rt2 + rt3;
    let mut q2 = rt2 * 2.0 + rt3 * 6.0;
    let q3 = rt3 * 6.0;
    for r in result.iter_mut() {
        *r = q0;
        q0 += q1;
        q1 += q2;
        q2 += q3;
    }
}

/// `calculate_evaluated_positions`.
pub fn calculate_evaluated_positions(
    positions: &[Vec3],
    handles_left: &[Vec3],
    handles_right: &[Vec3],
    evaluated_offsets: OffsetIndices<'_>,
    evaluated_positions: &mut [Vec3],
) {
    debug_assert_eq!(
        evaluated_offsets.total_size() as usize,
        evaluated_positions.len()
    );
    if evaluated_offsets.total_size() == 1 {
        evaluated_positions[0] = positions[0];
        return;
    }

    let last = positions.len() - 1;

    /* Evaluate the first segment. */
    let r0 = evaluated_offsets.group(0);
    evaluate_segment(
        positions[0],
        handles_right[0],
        handles_left[1],
        positions[1],
        &mut evaluated_positions[r0.start() as usize..r0.one_after_last() as usize],
    );

    /* Inner segments. */
    for i in 1..last {
        let evaluated_range = evaluated_offsets.group(i);
        let s = evaluated_range.start() as usize;
        let e = evaluated_range.one_after_last() as usize;
        if evaluated_range.size() == 1 {
            evaluated_positions[s] = positions[i];
        } else {
            evaluate_segment(
                positions[i],
                handles_right[i],
                handles_left[i + 1],
                positions[i + 1],
                &mut evaluated_positions[s..e],
            );
        }
    }

    /* Evaluate the final cyclic segment if necessary. */
    let last_segment_points = evaluated_offsets.group(last);
    let s = last_segment_points.start() as usize;
    let e = last_segment_points.one_after_last() as usize;
    if last_segment_points.size() == 1 {
        *evaluated_positions.last_mut().unwrap() = positions[last];
    } else {
        evaluate_segment(
            positions[last],
            handles_right[last],
            handles_left[0],
            positions[0],
            &mut evaluated_positions[s..e],
        );
    }
}

/// `linear_interpolation` — first point then lerp.
fn linear_interpolation<T: BezierEval>(a: T, b: T, dst: &mut [T]) {
    dst[0] = a;
    let step = 1.0 / dst.len() as f32;
    for i in 1..dst.len() {
        dst[i] = a + (b - a) * (i as f32 * step);
    }
}

/// `interpolate_to_evaluated` — per-segment linear interp.
pub fn interpolate_to_evaluated<T: BezierEval>(
    src: &[T],
    evaluated_offsets: OffsetIndices<'_>,
    dst: &mut [T],
) {
    debug_assert!(!src.is_empty());
    debug_assert_eq!(evaluated_offsets.total_size() as usize, dst.len());
    if src.len() == 1 {
        debug_assert_eq!(dst.len(), 1);
        dst[0] = src[0];
        return;
    }

    let last = src.len() - 1;
    let r0 = evaluated_offsets.group(0);
    linear_interpolation(
        src[0],
        src[1],
        &mut dst[r0.start() as usize..r0.one_after_last() as usize],
    );

    for i in 1..last {
        let segment = evaluated_offsets.group(i);
        linear_interpolation(
            src[i],
            src[i + 1],
            &mut dst[segment.start() as usize..segment.one_after_last() as usize],
        );
    }

    let last_segment = evaluated_offsets.group(last);
    linear_interpolation(
        src[last],
        src[0],
        &mut dst[last_segment.start() as usize..last_segment.one_after_last() as usize],
    );
}
