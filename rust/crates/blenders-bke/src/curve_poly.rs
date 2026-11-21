//! Port of `intern/curve_poly.cc` — `bke::curves::poly` tangent/normal
//! evaluation.

use blenders_math::geom::angle_normalized_v3v3;
use blenders_math::quaternion::rotate_direction_around_axis;
use blenders_math::vector::{angle_signed_on_axis, Vec3};

/// `delta_dir` — normalized `next - pos` or false when degenerate.
fn delta_dir(pos: Vec3, next: Vec3) -> Option<Vec3> {
    const EPSILON: f32 = 1.0e-9;
    let delta = next - pos;
    let norm = delta.length();
    if norm < EPSILON {
        return None;
    }
    Some(delta / norm)
}

/// `direction_bisect` — bisector of `other_dir` and the new segment dir.
/// `(dir, is_equal)` state carried across iterations.
fn direction_bisect(pos: Vec3, next: Vec3, other_dir: &mut Vec3, is_equal: &mut bool) -> Vec3 {
    const EPSILON: f32 = 1.0e-9;
    let prev_equal = *is_equal;
    let next_delta = next - pos;
    let next_norm = next_delta.length();
    *is_equal = next_norm < EPSILON;
    if *is_equal {
        return *other_dir;
    }
    let prev_dir = *other_dir;
    *other_dir = next_delta / next_norm;
    if prev_equal {
        return *other_dir;
    }
    let tangent = prev_dir + *other_dir;
    let norm = tangent.length();
    if norm < 0.6627619 {
        /* Approximates angle between segments < 45 degrees. */
        if norm < 2e-7 {
            /* Approximately < sin(1e-5) */
            return *other_dir;
        }
        let binormal = other_dir.cross(prev_dir);
        let normal = *other_dir - prev_dir;
        return binormal.cross(normal).normalized();
    }
    tangent / norm
}

/// `poly::calculate_tangents`.
pub fn calculate_tangents(positions: &[Vec3], is_cyclic: bool, tangents: &mut [Vec3]) {
    debug_assert_eq!(positions.len(), tangents.len());
    if positions.is_empty() {
        return;
    }
    if positions.len() == 1 {
        tangents[0] = Vec3::new(0.0, 0.0, 1.0);
        return;
    }
    let mut first_valid_index: i32 = -1;
    for i in 0..positions.len() - 1 {
        if let Some(d) = delta_dir(positions[i], positions[i + 1]) {
            tangents[i] = d;
            first_valid_index = i as i32;
            break;
        }
    }
    if first_valid_index == -1 {
        tangents.fill(Vec3::new(0.0, 0.0, 1.0));
        return;
    }
    let fvi = first_valid_index as usize;
    if fvi > 0 {
        let v = tangents[fvi];
        for t in tangents.iter_mut().take(fvi) {
            *t = v;
        }
    }
    let mut prev_delta = tangents[fvi];
    let mut prev_equal = false;
    for i in fvi + 1..positions.len() - 1 {
        tangents[i] = direction_bisect(positions[i], positions[i + 1], &mut prev_delta, &mut prev_equal);
    }
    let n = positions.len();
    if is_cyclic {
        let first = positions[0];
        tangents[n - 1] = direction_bisect(positions[n - 1], first, &mut prev_delta, &mut prev_equal);
        tangents[0] = direction_bisect(first, positions[1], &mut prev_delta, &mut prev_equal);
    } else if let Some(d) = delta_dir(positions[n - 2], positions[n - 1]) {
        tangents[n - 1] = d;
    } else {
        tangents[n - 1] = prev_delta;
    }
}

/// `poly::calculate_normals_z_up`.
pub fn calculate_normals_z_up(tangents: &[Vec3], normals: &mut [Vec3]) {
    debug_assert_eq!(normals.len(), tangents.len());
    const EPSILON: f32 = 1e-4;
    for i in 0..normals.len() {
        let tangent = tangents[i];
        if tangent.x.abs() + tangent.y.abs() < EPSILON {
            normals[i] = Vec3::new(1.0, 0.0, 0.0);
        } else {
            normals[i] = Vec3::new(tangent.y, -tangent.x, 0.0).normalized();
        }
    }
}

/// `calculate_next_normal` — rotate `last_normal` into `current_tangent`'s
/// plane (minimum-twist propagation).
fn calculate_next_normal(last_normal: Vec3, last_tangent: Vec3, current_tangent: Vec3) -> Vec3 {
    if last_tangent.is_zero() || current_tangent.is_zero() {
        return last_normal;
    }
    let angle = angle_normalized_v3v3(last_tangent, current_tangent);
    if angle != 0.0 {
        let axis = last_tangent.cross(current_tangent).normalized();
        if !axis.is_zero() {
            return rotate_direction_around_axis(last_normal, axis, angle).normalized();
        }
    }
    last_normal
}

/// `poly::calculate_normals_minimum` — minimum-twist normals with cyclic
/// twist correction.
pub fn calculate_normals_minimum(tangents: &[Vec3], cyclic: bool, normals: &mut [Vec3]) {
    debug_assert_eq!(normals.len(), tangents.len());
    if normals.is_empty() {
        return;
    }
    const EPSILON: f32 = 1e-4;
    let first_tangent = tangents[0];
    if first_tangent.x.abs() + first_tangent.y.abs() < EPSILON {
        normals[0] = Vec3::new(1.0, 0.0, 0.0);
    } else {
        normals[0] = Vec3::new(first_tangent.y, -first_tangent.x, 0.0).normalized();
    }
    let n = normals.len();
    for i in 1..n {
        normals[i] = calculate_next_normal(normals[i - 1], tangents[i - 1], tangents[i]);
    }
    if !cyclic {
        return;
    }
    let uncorrected_last_normal =
        calculate_next_normal(normals[n - 1], tangents[n - 1], tangents[0]);
    let mut correction_angle =
        angle_signed_on_axis(normals[0], uncorrected_last_normal, tangents[0]);
    if correction_angle > std::f32::consts::PI {
        correction_angle -= 2.0 * std::f32::consts::PI;
    }
    let angle_step = correction_angle / n as f32;
    for i in 0..n {
        let axis = tangents[i];
        if axis.is_zero() {
            continue;
        }
        let angle = angle_step * i as f32;
        normals[i] = rotate_direction_around_axis(normals[i], axis, angle);
    }
}
