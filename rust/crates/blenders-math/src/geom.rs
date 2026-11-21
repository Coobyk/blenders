//! Port of `source/blender/blenlib/intern/math_geom.cc` +
//! `BLI_math_geom{,_c}.hh` — geometry queries: normals/areas, planes,
//! volumes, distances, closest points, intersections.
//!
//! Planes are `[normal.xyz, w]` (`Vec4`) so that `dot(normal, p) + w == 0`
//! for points on the plane — the `float plane[4]` convention upstream.

use crate::base::{max_ff, min_ff, square, FLT_EPSILON};
use crate::vector::{Vec2, Vec3, Vec4};

/* -------------------------------------------------------------------- */
/* Polygons (Newell's method / Trapezium rule)                            */
/* -------------------------------------------------------------------- */

/// `add_newell_cross_v3_v3v3` — accumulate one edge into a Newell normal.
#[inline]
pub fn add_newell_cross(n: &mut Vec3, v_prev: Vec3, v_curr: Vec3) {
    n.x += (v_prev.y - v_curr.y) * (v_prev.z + v_curr.z);
    n.y += (v_prev.z - v_curr.z) * (v_prev.x + v_curr.x);
    n.z += (v_prev.x - v_curr.x) * (v_prev.y + v_curr.y);
}

/// `cross_tri_v3` — unnormalized triangle normal `(v1-v2) x (v2-v3)`.
pub fn cross_tri(v1: Vec3, v2: Vec3, v3: Vec3) -> Vec3 {
    let n1 = v1 - v2;
    let n2 = v2 - v3;
    Vec3::new(
        n1.y * n2.z - n1.z * n2.y,
        n1.z * n2.x - n1.x * n2.z,
        n1.x * n2.y - n1.y * n2.x,
    )
}

/// `normal_tri_v3` — returns the previous length of the normal.
pub fn normal_tri(v1: Vec3, v2: Vec3, v3: Vec3) -> (Vec3, f32) {
    let mut n = cross_tri(v1, v2, v3);
    let len = n.normalize_and_get_length();
    (n, len)
}

/// `normal_quad_v3` — cross of the two diagonals `(v1-v3) x (v2-v4)`.
pub fn normal_quad(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> (Vec3, f32) {
    let mut n = (v1 - v3).cross(v2 - v4);
    let len = n.normalize_and_get_length();
    (n, len)
}

/// `cross_poly_v3` — Newell's method polygon normal.
pub fn cross_poly(verts: &[Vec3]) -> Vec3 {
    let mut n = Vec3::ZERO;
    if verts.is_empty() {
        return n;
    }
    let mut v_prev = verts[verts.len() - 1];
    for &v_curr in verts {
        add_newell_cross(&mut n, v_prev, v_curr);
        v_prev = v_curr;
    }
    n
}

/// `normal_poly_v3`.
pub fn normal_poly(verts: &[Vec3]) -> (Vec3, f32) {
    let mut n = cross_poly(verts);
    let len = n.normalize_and_get_length();
    (n, len)
}

/// `area_tri_v3`.
pub fn area_tri(v1: Vec3, v2: Vec3, v3: Vec3) -> f32 {
    cross_tri(v1, v2, v3).length() * 0.5
}

/// `area_squared_tri_v3` — `(area)^2`, avoiding a sqrt.
pub fn area_squared_tri(v1: Vec3, v2: Vec3, v3: Vec3) -> f32 {
    (cross_tri(v1, v2, v3) * 0.5).length_squared()
}

/// `area_tri_signed_v3` — negated when the triangle normal opposes `normal`.
pub fn area_tri_signed(v1: Vec3, v2: Vec3, v3: Vec3, normal: Vec3) -> f32 {
    let n = cross_tri(v1, v2, v3);
    let mut area = n.length() * 0.5;
    if n.dot(normal) < 0.0 {
        area = -area;
    }
    area
}

/// `area_quad_v3`.
pub fn area_quad(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> f32 {
    area_poly(&[v1, v2, v3, v4])
}

/// `area_squared_quad_v3`.
pub fn area_squared_quad(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> f32 {
    area_squared_poly(&[v1, v2, v3, v4])
}

/// `area_poly_v3`.
pub fn area_poly(verts: &[Vec3]) -> f32 {
    cross_poly(verts).length() * 0.5
}

/// `area_squared_poly_v3`.
pub fn area_squared_poly(verts: &[Vec3]) -> f32 {
    (cross_poly(verts) * 0.5).length_squared()
}

/// `cross_poly_v2` — the Trapezium Area Rule, twice the signed area.
pub fn cross_poly_v2(verts: &[Vec2]) -> f32 {
    let mut cross = 0.0;
    if verts.is_empty() {
        return cross;
    }
    let mut prev = verts[verts.len() - 1];
    for &curr in verts {
        cross += (prev.x - curr.x) * (curr.y + prev.y);
        prev = curr;
    }
    cross
}

/// `area_poly_v2`.
pub fn area_poly_v2(verts: &[Vec2]) -> f32 {
    (0.5 * cross_poly_v2(verts)).abs()
}

/// `area_poly_signed_v2` — negative when wound clockwise.
pub fn area_poly_signed_v2(verts: &[Vec2]) -> f32 {
    0.5 * cross_poly_v2(verts)
}

/// `area_squared_poly_v2`.
pub fn area_squared_poly_v2(verts: &[Vec2]) -> f32 {
    let area = area_poly_signed_v2(verts);
    area * area
}

/// `cotangent_tri_weight_v3` — cotangent of the angle at `v1`; 0 if degenerate.
pub fn cotangent_tri_weight(v1: Vec3, v2: Vec3, v3: Vec3) -> f32 {
    let a = v2 - v1;
    let b = v3 - v1;
    let c_len = a.cross(b).length();
    if c_len > FLT_EPSILON {
        a.dot(b) / c_len
    } else {
        0.0
    }
}

/* -------------------------------------------------------------------- */
/* Planes                                                                */
/* -------------------------------------------------------------------- */

/// `plane_from_point_normal_v3` — `[no.xyz, -dot(no, co)]`.
pub fn plane_from_point_normal(plane_co: Vec3, plane_no: Vec3) -> Vec4 {
    Vec4::new(plane_no.x, plane_no.y, plane_no.z, -plane_no.dot(plane_co))
}

/// `plane_point_side_v3` — signed distance numerator
/// `co·plane.xyz + plane.w` (scaled by |normal|).
#[inline]
pub fn plane_point_side(plane: Vec4, co: Vec3) -> f32 {
    plane.xyz().dot(co) + plane.w
}

/// `plane_to_point_vector_v3` — `(point_on_plane, normal)`; normal keeps its length.
pub fn plane_to_point_vector(plane: Vec4) -> (Vec3, Vec3) {
    let n = plane.xyz();
    let len_sq = n.length_squared();
    (n * (-plane.w / len_sq), n)
}

/// `plane_to_point_vector_v3_normalized` — `(point_on_plane, unit_normal)`.
pub fn plane_to_point_vector_normalized(plane: Vec4) -> (Vec3, Vec3) {
    let mut no = plane.xyz();
    let len = no.normalize_and_get_length();
    (no * (-plane.w / len), no)
}

/* -------------------------------------------------------------------- */
/* Volume                                                                */
/* -------------------------------------------------------------------- */

#[inline]
fn tetra_det(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> f32 {
    // determinant_m3_array of rows [v1-v2, v2-v3, v3-v4].
    // det of row-matrix [a;b;c] == a · (b × c).
    (v1 - v2).triple(v2 - v3, v3 - v4)
}

/// `volume_tetrahedron_v3`.
pub fn volume_tetrahedron(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> f32 {
    tetra_det(v1, v2, v3, v4).abs() / 6.0
}

/// `volume_tetrahedron_signed_v3`.
pub fn volume_tetrahedron_signed(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> f32 {
    tetra_det(v1, v2, v3, v4) / 6.0
}

/// `volume_tri_tetrahedron_signed_v3_6x` — 6× the signed volume (v1×v2)·v3.
pub fn volume_tri_tetrahedron_signed_6x(v1: Vec3, v2: Vec3, v3: Vec3) -> f32 {
    v1.cross(v2).dot(v3)
}

/// `volume_tri_tetrahedron_signed_v3`.
pub fn volume_tri_tetrahedron_signed(v1: Vec3, v2: Vec3, v3: Vec3) -> f32 {
    volume_tri_tetrahedron_signed_6x(v1, v2, v3) / 6.0
}

/* -------------------------------------------------------------------- */
/* Distance & closest points                                             */
/* -------------------------------------------------------------------- */

/// `closest_to_ray_v3` — closest point on the ray `orig + t*dir` to `p`;
/// returns `(point, lambda)`.
pub fn closest_to_ray(p: Vec3, ray_orig: Vec3, ray_dir: Vec3) -> (Vec3, f32) {
    if ray_dir.is_zero() {
        return (ray_orig, 0.0);
    }
    let h = p - ray_orig;
    let lambda = ray_dir.dot(h) / ray_dir.dot(ray_dir);
    (ray_orig.madd(ray_dir, lambda), lambda)
}

/// `closest_to_line_v3`.
pub fn closest_to_line(p: Vec3, l1: Vec3, l2: Vec3) -> (Vec3, f32) {
    closest_to_ray(p, l1, l2 - l1)
}

/// `closest_to_line_v2`.
pub fn closest_to_line_v2(p: Vec2, l1: Vec2, l2: Vec2) -> (Vec2, f32) {
    let u = l2 - l1;
    let h = p - l1;
    let denom = u.dot(u);
    if denom == 0.0 {
        return (l1, 0.0);
    }
    let lambda = u.dot(h) / denom;
    (l1 + u * lambda, lambda)
}

/// `closest_to_line_segment_v2` — `(point, clamped_lambda)`.
pub fn closest_to_line_segment_v2(p: Vec2, l1: Vec2, l2: Vec2) -> (Vec2, f32) {
    let (cp, lambda) = closest_to_line_v2(p, l1, l2);
    // flip checks for !finite case (when segment is a point)
    if lambda <= 0.0 {
        (l1, 0.0)
    } else if lambda >= 1.0 {
        (l2, 1.0)
    } else {
        (cp, lambda)
    }
}

/// `closest_to_line_segment_v3`.
pub fn closest_to_line_segment_v3(p: Vec3, l1: Vec3, l2: Vec3) -> (Vec3, f32) {
    let (cp, lambda) = closest_to_line(p, l1, l2);
    if lambda <= 0.0 {
        (l1, 0.0)
    } else if lambda >= 1.0 {
        (l2, 1.0)
    } else {
        (cp, lambda)
    }
}

/// `isect_ray_line_v3` — lambda along `v0..v1` of the nearest point to the ray.
pub fn isect_ray_line(
    ray_origin: Vec3,
    ray_direction: Vec3,
    v0: Vec3,
    v1: Vec3,
) -> Option<f32> {
    let a = v1 - v0;
    let t = v0 - ray_origin;
    let n = a.cross(ray_direction);
    let nlen = n.length_squared();
    if nlen == 0.0 {
        // The lines are parallel.
        return None;
    }
    // math trick — see upstream comment:
    // numerator = dot(t,dir)*dot(a,dir) - dot(t,a)*dot(dir,dir); lambda = numerator/nlen
    let c = n - t;
    let cray = c.cross(ray_direction);
    Some(cray.dot(n) / nlen)
}

/// `closest_ray_to_segment_v3` — `(point, lambda)` or `(v0, 0)` when parallel.
pub fn closest_ray_to_segment(
    ray_origin: Vec3,
    ray_direction: Vec3,
    v0: Vec3,
    v1: Vec3,
) -> (Vec3, f32) {
    let Some(lambda) = isect_ray_line(ray_origin, ray_direction, v0, v1) else {
        return (v0, 0.0);
    };
    if lambda <= 0.0 {
        (v0, 0.0)
    } else if lambda >= 1.0 {
        (v1, 1.0)
    } else {
        (v0.lerp(v1, lambda), lambda)
    }
}

/// `closest_to_plane_v3`.
pub fn closest_to_plane(plane: Vec4, pt: Vec3) -> Vec3 {
    let n = plane.xyz();
    let len_sq = n.length_squared();
    let side = plane_point_side(plane, pt);
    pt.madd(n, -side / len_sq)
}

/// `closest_to_plane_normalized_v3` — plane normal must be unit length.
pub fn closest_to_plane_normalized(plane: Vec4, pt: Vec3) -> Vec3 {
    let side = plane_point_side(plane, pt);
    pt.madd(plane.xyz(), -side)
}

/// `closest_to_plane3_v3` — plane through the origin, `plane[3]` normal only.
pub fn closest_to_plane3(plane: Vec3, pt: Vec3) -> Vec3 {
    let len_sq = plane.length_squared();
    let side = plane.dot(pt);
    pt.madd(plane, -side / len_sq)
}

/// `closest_to_plane3_normalized_v3`.
pub fn closest_to_plane3_normalized(plane: Vec3, pt: Vec3) -> Vec3 {
    pt.madd(plane, -plane.dot(pt))
}

/// `dist_signed_squared_to_plane_v3`.
pub fn dist_signed_squared_to_plane(p: Vec3, plane: Vec4) -> f32 {
    let len_sq = plane.xyz().length_squared();
    let side = plane_point_side(plane, p);
    let fac = side / len_sq;
    (len_sq * (fac * fac)).copysign(side)
}

/// `dist_squared_to_plane_v3`.
pub fn dist_squared_to_plane(p: Vec3, plane: Vec4) -> f32 {
    let len_sq = plane.xyz().length_squared();
    let side = plane_point_side(plane, p);
    let fac = side / len_sq;
    len_sq * (fac * fac)
}

/// `dist_signed_squared_to_plane3_v3`.
pub fn dist_signed_squared_to_plane3(p: Vec3, plane: Vec3) -> f32 {
    let len_sq = plane.length_squared();
    let side = plane.dot(p);
    let fac = side / len_sq;
    (len_sq * (fac * fac)).copysign(side)
}

/// `dist_squared_to_plane3_v3`.
pub fn dist_squared_to_plane3(p: Vec3, plane: Vec3) -> f32 {
    let len_sq = plane.length_squared();
    let side = plane.dot(p);
    let fac = side / len_sq;
    len_sq * (fac * fac)
}

/// `dist_signed_to_plane_v3`.
pub fn dist_signed_to_plane(p: Vec3, plane: Vec4) -> f32 {
    let len_sq = plane.xyz().length_squared();
    let side = plane_point_side(plane, p);
    let fac = side / len_sq;
    len_sq.sqrt() * fac
}

/// `dist_to_plane_v3`.
pub fn dist_to_plane(p: Vec3, plane: Vec4) -> f32 {
    dist_signed_to_plane(p, plane).abs()
}

/// `dist_signed_to_plane3_v3`.
pub fn dist_signed_to_plane3(p: Vec3, plane: Vec3) -> f32 {
    let len_sq = plane.length_squared();
    let side = plane.dot(p);
    let fac = side / len_sq;
    len_sq.sqrt() * fac
}

/// `dist_to_plane3_v3`.
pub fn dist_to_plane3(p: Vec3, plane: Vec3) -> f32 {
    dist_signed_to_plane3(p, plane).abs()
}

/// `dist_squared_to_line_segment_v2`.
pub fn dist_squared_to_line_segment_v2(p: Vec2, l1: Vec2, l2: Vec2) -> f32 {
    closest_to_line_segment_v2(p, l1, l2).0.distance_squared(p)
}

/// `dist_to_line_segment_v2`.
pub fn dist_to_line_segment_v2(p: Vec2, l1: Vec2, l2: Vec2) -> f32 {
    dist_squared_to_line_segment_v2(p, l1, l2).sqrt()
}

/// `dist_squared_to_line_v2`.
pub fn dist_squared_to_line_v2(p: Vec2, l1: Vec2, l2: Vec2) -> f32 {
    closest_to_line_v2(p, l1, l2).0.distance_squared(p)
}

/// `dist_to_line_v2`.
pub fn dist_to_line_v2(p: Vec2, l1: Vec2, l2: Vec2) -> f32 {
    dist_squared_to_line_v2(p, l1, l2).sqrt()
}

/// `dist_squared_to_line_segment_v3`.
pub fn dist_squared_to_line_segment_v3(p: Vec3, l1: Vec3, l2: Vec3) -> f32 {
    closest_to_line_segment_v3(p, l1, l2).0.distance_squared(p)
}

/// `dist_to_line_segment_v3`.
pub fn dist_to_line_segment_v3(p: Vec3, l1: Vec3, l2: Vec3) -> f32 {
    dist_squared_to_line_segment_v3(p, l1, l2).sqrt()
}

/// `dist_squared_to_line_v3`.
pub fn dist_squared_to_line_v3(p: Vec3, l1: Vec3, l2: Vec3) -> f32 {
    closest_to_line(p, l1, l2).0.distance_squared(p)
}

/// `dist_to_line_v3`.
pub fn dist_to_line_v3(p: Vec3, l1: Vec3, l2: Vec3) -> f32 {
    dist_squared_to_line_v3(p, l1, l2).sqrt()
}

/// `dist_signed_squared_to_corner_v3v3v3` — signed squared distance to the
/// corner at `v2` formed by edges to `v1` and `v3`; `axis_ref` disambiguates
/// degenerate axes and defines the concave side.
pub fn dist_signed_squared_to_corner(
    p: Vec3,
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
    axis_ref: Vec3,
) -> f32 {
    let dir_a = v1 - v2;
    let dir_b = v3 - v2;
    let mut axis = dir_a.cross(dir_b);
    let mut flip = false;
    if axis.length_squared() < FLT_EPSILON {
        axis = axis_ref;
    } else if axis.dot(axis_ref) < 0.0 {
        flip = true;
        axis = -axis;
    }
    let plane_a = dir_a.cross(axis);
    let plane_b = axis.cross(dir_b);
    let s_p_v2 = p - v2;
    let dist_a = dist_signed_squared_to_plane3(s_p_v2, plane_a);
    let dist_b = dist_signed_squared_to_plane3(s_p_v2, plane_b);
    if flip {
        min_ff(dist_a, dist_b)
    } else {
        max_ff(dist_a, dist_b)
    }
}

/// `dist_squared_to_ray_v3_normalized` — `ray_direction` must be unit length.
pub fn dist_squared_to_ray_normalized(ray_origin: Vec3, ray_direction: Vec3, co: Vec3) -> f32 {
    let origin_to_co = co - ray_origin;
    let origin_to_proj = origin_to_co.project_onto_normalized(ray_direction);
    let co_projected_on_ray = ray_origin + origin_to_proj;
    co.distance_squared(co_projected_on_ray)
}

/// `dist_squared_ray_to_seg_v3` — `(dist², r_point, depth)`.
pub fn dist_squared_ray_to_seg(
    ray_origin: Vec3,
    ray_direction: Vec3,
    v0: Vec3,
    v1: Vec3,
) -> (f32, Vec3, f32) {
    let r_point = match isect_ray_line(ray_origin, ray_direction, v0, v1) {
        Some(lambda) if lambda <= 0.0 => v0,
        Some(lambda) if lambda >= 1.0 => v1,
        Some(lambda) => v0.lerp(v1, lambda),
        None => v0,
    };
    let dvec = r_point - ray_origin;
    let depth = dvec.dot(ray_direction);
    (dvec.length_squared() - square(depth), r_point, depth)
}

/// `aabb_get_near_far_from_plane` — `(bb_near, bb_afar)` for the plane normal.
pub fn aabb_get_near_far_from_plane(plane_no: Vec3, bbmin: Vec3, bbmax: Vec3) -> (Vec3, Vec3) {
    let mut near = Vec3::ZERO;
    let mut afar = Vec3::ZERO;
    for i in 0..3 {
        if plane_no[i] < 0.0 {
            near[i] = bbmax[i];
            afar[i] = bbmin[i];
        } else {
            near[i] = bbmin[i];
            afar[i] = bbmax[i];
        }
    }
    (near, afar)
}

/// `line_point_factor_v3_ex`.
pub fn line_point_factor_ex(p: Vec3, l1: Vec3, l2: Vec3, epsilon: f32, fallback: f32) -> f32 {
    let u = l2 - l1;
    let h = p - l1;
    let dot = u.length_squared();
    if dot > epsilon {
        u.dot(h) / dot
    } else {
        fallback
    }
}

/// `line_point_factor_v3`.
pub fn line_point_factor(p: Vec3, l1: Vec3, l2: Vec3) -> f32 {
    line_point_factor_ex(p, l1, l2, 0.0, 0.0)
}

/// `ray_point_factor_v3_ex`.
pub fn ray_point_factor_ex(
    p: Vec3,
    ray_origin: Vec3,
    ray_direction: Vec3,
    epsilon: f32,
    fallback: f32,
) -> f32 {
    let p_relative = p - ray_origin;
    let dot = ray_direction.length_squared();
    if dot > epsilon {
        ray_direction.dot(p_relative) / dot
    } else {
        fallback
    }
}

/// `ray_point_factor_v3`.
pub fn ray_point_factor(p: Vec3, ray_origin: Vec3, ray_direction: Vec3) -> f32 {
    ray_point_factor_ex(p, ray_origin, ray_direction, 0.0, 0.0)
}

/* -------------------------------------------------------------------- */
/* Intersections                                                         */
/* -------------------------------------------------------------------- */

/// Result of line/segment intersection tests — `ISECT_LINE_LINE_*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum Isect {
    Colinear = -1,
    None = 0,
    Exact = 1,
    Cross = 2,
}

/// `isect_seg_seg_v2_simple` — boolean segment/segment test.
pub fn isect_seg_seg_v2_simple(v1: Vec2, v2: Vec2, v3: Vec2, v4: Vec2) -> bool {
    fn ccw(a: Vec2, b: Vec2, c: Vec2) -> bool {
        (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
    }
    ccw(v1, v3, v4) != ccw(v2, v3, v4) && ccw(v1, v2, v3) != ccw(v1, v2, v4)
}

/// `isect_line_line_v2_point` — intersection point of the two infinite lines.
pub fn isect_line_line_v2_point(v0: Vec2, v1: Vec2, v2: Vec2, v3: Vec2) -> (Isect, Vec2) {
    let s10 = v1 - v0;
    let s32 = v3 - v2;
    let div = s10.cross(s32);
    if div != 0.0 {
        let u = v1.cross(v0);
        let v = v3.cross(v2);
        (
            Isect::Cross,
            Vec2::new(
                (s32.x * u - s10.x * v) / div,
                (s32.y * u - s10.y * v) / div,
            ),
        )
    } else {
        (Isect::Colinear, Vec2::ZERO)
    }
}

/// `isect_seg_seg_v2`.
pub fn isect_seg_seg_v2(v1: Vec2, v2: Vec2, v3: Vec2, v4: Vec2) -> Isect {
    let div = (v2.x - v1.x) * (v4.y - v3.y) - (v2.y - v1.y) * (v4.x - v3.x);
    if div == 0.0 {
        return Isect::Colinear;
    }
    let lambda =
        ((v1.y - v3.y) * (v4.x - v3.x) - (v1.x - v3.x) * (v4.y - v3.y)) / div;
    let mu = ((v1.y - v3.y) * (v2.x - v1.x) - (v1.x - v3.x) * (v2.y - v1.y)) / div;
    if (0.0..=1.0).contains(&lambda) && (0.0..=1.0).contains(&mu) {
        if lambda == 0.0 || lambda == 1.0 || mu == 0.0 || mu == 1.0 {
            return Isect::Exact;
        }
        return Isect::Cross;
    }
    Isect::None
}

/// `isect_seg_seg_v2_lambda_mu_db` — double precision, `(result, lambda, mu)`.
pub fn isect_seg_seg_v2_lambda_mu_db(
    v1: [f64; 2],
    v2: [f64; 2],
    v3: [f64; 2],
    v4: [f64; 2],
) -> (Isect, f64, f64) {
    let div = (v2[0] - v1[0]) * (v4[1] - v3[1]) - (v2[1] - v1[1]) * (v4[0] - v3[0]);
    if div.abs() < f64::EPSILON {
        return (Isect::Colinear, 0.0, 0.0);
    }
    let lambda =
        ((v1[1] - v3[1]) * (v4[0] - v3[0]) - (v1[0] - v3[0]) * (v4[1] - v3[1])) / div;
    let mu = ((v1[1] - v3[1]) * (v2[0] - v1[0]) - (v1[0] - v3[0]) * (v2[1] - v1[1])) / div;
    if (0.0..=1.0).contains(&lambda) && (0.0..=1.0).contains(&mu) {
        if lambda == 0.0 || lambda == 1.0 || mu == 0.0 || mu == 1.0 {
            return (Isect::Exact, lambda, mu);
        }
        return (Isect::Cross, lambda, mu);
    }
    (Isect::None, lambda, mu)
}

/// `isect_line_plane_v3` — line through (l1,l2) vs plane given by point+normal.
pub fn isect_line_plane(l1: Vec3, l2: Vec3, plane_co: Vec3, plane_no: Vec3) -> Option<Vec3> {
    let u = l2 - l1;
    let h = l1 - plane_co;
    let dot = plane_no.dot(u);
    if dot.abs() > FLT_EPSILON {
        let lambda = -plane_no.dot(h) / dot;
        Some(l1.madd(u, lambda))
    } else {
        None // segment is parallel to plane
    }
}

/// `isect_ray_plane_v3_factor` — lambda along `ray_direction`.
pub fn isect_ray_plane_factor(
    ray_origin: Vec3,
    ray_direction: Vec3,
    plane_co: Vec3,
    plane_no: Vec3,
) -> Option<f32> {
    let dot = plane_no.dot(ray_direction);
    if dot == 0.0 {
        return None;
    }
    let h = ray_origin - plane_co;
    Some(-plane_no.dot(h) / dot)
}

/// `isect_ray_plane_v3` — lambda, `None` when parallel or (clip && behind).
pub fn isect_ray_plane(
    ray_origin: Vec3,
    ray_direction: Vec3,
    plane: Vec4,
    clip: bool,
) -> Option<f32> {
    let (plane_co, plane_no) = plane_to_point_vector(plane);
    let lambda = isect_ray_plane_factor(ray_origin, ray_direction, plane_co, plane_no)?;
    if clip && lambda < 0.0 {
        return None;
    }
    Some(lambda)
}

/// `isect_line_sphere_v3` — 0, 1 (tangent: p1) or 2 intersections.
pub fn isect_line_sphere(
    l1: Vec3,
    l2: Vec3,
    sp: Vec3,
    r: f32,
) -> (u8, Vec3, Vec3) {
    // adapted from Paul Bourke's sphere_line_intersection (see upstream)
    let ldir = l2 - l1;
    let a = ldir.length_squared();
    let b = 2.0 * (ldir.x * (l1.x - sp.x) + ldir.y * (l1.y - sp.y) + ldir.z * (l1.z - sp.z));
    let c = sp.length_squared() + l1.length_squared() - 2.0 * sp.dot(l1) - r * r;
    let i = b * b - 4.0 * a * c;
    if i < 0.0 {
        return (0, Vec3::ZERO, Vec3::ZERO);
    }
    if i == 0.0 {
        let mu = -b / (2.0 * a);
        return (1, l1.madd(ldir, mu), Vec3::ZERO);
    }
    let i_sqrt = i.sqrt();
    let mu = (-b + i_sqrt) / (2.0 * a);
    let p1 = l1.madd(ldir, mu);
    let mu = (-b - i_sqrt) / (2.0 * a);
    let p2 = l1.madd(ldir, mu);
    (2, p1, p2)
}

/// `isect_point_planes_v3` — inside when on the negative side of every plane.
pub fn isect_point_planes(planes: &[Vec4], p: Vec3) -> bool {
    planes.iter().all(|&plane| plane_point_side(plane, p) <= 0.0)
}

/// `isect_point_tri` (2D) — +1 inside CCW, -1 inside CW, 0 outside.
pub fn isect_point_tri(pt: Vec2, v1: Vec2, v2: Vec2, v3: Vec2) -> i32 {
    let s1 = (v2 - v1).cross(pt - v1);
    let s2 = (v3 - v2).cross(pt - v2);
    let s3 = (v1 - v3).cross(pt - v3);
    if s1 >= 0.0 && s2 >= 0.0 && s3 >= 0.0 {
        return 1;
    }
    if s1 <= 0.0 && s2 <= 0.0 && s3 <= 0.0 {
        return -1;
    }
    0
}

/// `isect_point_quad` (2D) — +1 inside CCW, -1 inside CW, 0 outside.
pub fn isect_point_quad(p: Vec2, v1: Vec2, v2: Vec2, v3: Vec2, v4: Vec2) -> i32 {
    let s1 = (v2 - v1).cross(p - v1);
    let s2 = (v3 - v2).cross(p - v2);
    let s3 = (v4 - v3).cross(p - v3);
    let s4 = (v1 - v4).cross(p - v4);
    if s1 >= 0.0 && s2 >= 0.0 && s3 >= 0.0 && s4 >= 0.0 {
        return 1;
    }
    if s1 <= 0.0 && s2 <= 0.0 && s3 <= 0.0 && s4 <= 0.0 {
        return -1;
    }
    0
}

/// `closest_seg_seg_v2` — `(dist², closest_a, closest_b, lambda_a, lambda_b)`.
pub fn closest_seg_seg_v2(
    a1: Vec2,
    a2: Vec2,
    b1: Vec2,
    b2: Vec2,
) -> (f32, Vec2, Vec2, f32, f32) {
    if isect_seg_seg_v2_simple(a1, a2, b1, b2) {
        let (_, intersection) = isect_line_line_v2_point(a1, a2, b1, b2);
        let (_, lambda_a) = closest_to_line_v2(intersection, a1, a2);
        let (_, lambda_b) = closest_to_line_v2(intersection, b1, b2);
        return (0.0, intersection, intersection, lambda_a, lambda_b);
    }
    let (p1, lambda1) = closest_to_line_segment_v2(a1, b1, b2);
    let (p2, lambda2) = closest_to_line_segment_v2(a2, b1, b2);
    let (p3, lambda3) = closest_to_line_segment_v2(b1, a1, a2);
    let (p4, lambda4) = closest_to_line_segment_v2(b2, a1, a2);
    let d1 = p1.distance_squared(a1);
    let d2 = p2.distance_squared(a2);
    let d3 = p3.distance_squared(b1);
    let d4 = p4.distance_squared(b2);
    let min_d = d1.min(d2).min(d3).min(d4);
    if min_d == d1 {
        (min_d, a1, p1, 0.0, lambda1)
    } else if min_d == d2 {
        (min_d, a2, p2, 1.0, lambda2)
    } else if min_d == d3 {
        (min_d, p3, b1, lambda3, 0.0)
    } else {
        (min_d, p4, b2, lambda4, 1.0)
    }
}

/// `dist_seg_seg_v2` — distance between two 2D segments.
pub fn dist_seg_seg_v2(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> f32 {
    closest_seg_seg_v2(a1, a2, b1, b2).0.sqrt()
}

/// `isect_line_sphere_v2` — 2D variant of `isect_line_sphere`.
pub fn isect_line_sphere_v2(l1: Vec2, l2: Vec2, sp: Vec2, r: f32) -> (i8, Vec2, Vec2) {
    let ldir = l2 - l1;
    let a = ldir.dot(ldir);
    let b = 2.0 * (ldir.x * (l1.x - sp.x) + ldir.y * (l1.y - sp.y));
    let c = sp.dot(sp) + l1.dot(l1) - 2.0 * sp.dot(l1) - r * r;
    let i = b * b - 4.0 * a * c;
    if i < 0.0 {
        return (0, Vec2::ZERO, Vec2::ZERO);
    }
    if i == 0.0 {
        let mu = -b / (2.0 * a);
        return (1, l1 + ldir * mu, Vec2::ZERO);
    }
    let i_sqrt = i.sqrt();
    let p1 = l1 + ldir * ((-b + i_sqrt) / (2.0 * a));
    let p2 = l1 + ldir * ((-b - i_sqrt) / (2.0 * a));
    (2, p1, p2)
}

/// `isect_point_poly_v2` — ray-casting point-in-polygon (2D).
pub fn isect_point_poly_v2(pt: Vec2, verts: &[Vec2]) -> bool {
    let nr = verts.len();
    let mut isect = false;
    let mut j = nr - 1;
    for i in 0..nr {
        let vi = verts[i];
        let vj = verts[j];
        if (vi.y > pt.y) != (vj.y > pt.y)
            && (pt.x < (vj.x - vi.x) * (pt.y - vi.y) / (vj.y - vi.y) + vi.x)
        {
            isect = !isect;
        }
        j = i;
    }
    isect
}

/// `isect_point_poly_v2_int`.
pub fn isect_point_poly_v2_int(pt: [i32; 2], verts: &[[i32; 2]]) -> bool {
    let nr = verts.len();
    let mut isect = false;
    let mut j = nr - 1;
    for i in 0..nr {
        let vi = verts[i];
        let vj = verts[j];
        if (vi[1] > pt[1]) != (vj[1] > pt[1])
            && (pt[0] < (vj[0] - vi[0]) * (pt[1] - vi[1]) / (vj[1] - vi[1]) + vi[0])
        {
            isect = !isect;
        }
        j = i;
    }
    isect
}

/// `line_point_side_v2` — side of `pt` relative to directed line `l1->l2`.
#[inline]
pub fn line_point_side_v2(l1: Vec2, l2: Vec2, pt: Vec2) -> f32 {
    (l1.x - pt.x) * (l2.y - pt.y) - (l2.x - pt.x) * (l1.y - pt.y)
}

/// `isect_point_tri_v2_cw` — boolean point-in-triangle (any winding).
pub fn isect_point_tri_v2_cw(pt: Vec2, v1: Vec2, v2: Vec2, v3: Vec2) -> bool {
    line_point_side_v2(v1, v2, pt) >= 0.0
        && line_point_side_v2(v2, v3, pt) >= 0.0
        && line_point_side_v2(v3, v1, pt) >= 0.0
}

/// `isect_line_segment_tri_v3` — segment vs triangle, Möller–Trumbore;
/// `(hit, lambda, u, v)` where lambda is the segment parameter.
pub fn isect_line_segment_tri(
    p1: Vec3,
    p2: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<(f32, f32, f32)> {
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let d = p2 - p1;
    let p = d.cross(e2);
    let a = e1.dot(p);
    if a == 0.0 {
        return None;
    }
    let f = 1.0 / a;
    let s = p1 - v0;
    let u = f * s.dot(p);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = f * d.dot(q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let lambda = f * e2.dot(q);
    if !(0.0..=1.0).contains(&lambda) {
        return None;
    }
    Some((lambda, u, v))
}

/// `isect_line_tri_v3` — infinite line vs triangle; no lambda bounds.
pub fn isect_line_tri(
    p1: Vec3,
    p2: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<(f32, f32, f32)> {
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let d = p2 - p1;
    let p = d.cross(e2);
    let a = e1.dot(p);
    if a == 0.0 {
        return None;
    }
    let f = 1.0 / a;
    let s = p1 - v0;
    let u = f * s.dot(p);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = f * d.dot(q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let lambda = f * e2.dot(q);
    Some((lambda, u, v))
}

/// `isect_line_segment_tri_epsilon_v3`.
pub fn isect_line_segment_tri_epsilon(
    p1: Vec3,
    p2: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    epsilon: f32,
) -> Option<(f32, f32, f32)> {
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let d = p2 - p1;
    let p = d.cross(e2);
    let a = e1.dot(p);
    if a == 0.0 {
        return None;
    }
    let f = 1.0 / a;
    let s = p1 - v0;
    let u = f * s.dot(p);
    if u < -epsilon || u > 1.0 + epsilon {
        return None;
    }
    let q = s.cross(e1);
    let v = f * d.dot(q);
    if v < -epsilon || u + v > 1.0 + epsilon {
        return None;
    }
    let lambda = f * e2.dot(q);
    if !(0.0..=1.0).contains(&lambda) {
        return None;
    }
    Some((lambda, u, v))
}

/// `isect_ray_tri_v3` — Möller–Trumbore ray-triangle; lambda >= 0.
pub fn isect_ray_tri(
    ray_origin: Vec3,
    ray_direction: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<(f32, f32, f32)> {
    // NOTE(ideasman42): these values were 0.000001 in 2.4x but for projection
    // snapping on a human head (1BU == 1m), subdivision level 2 gave errors.
    const EPSILON: f32 = 0.00000001;
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let p = ray_direction.cross(e2);
    let a = e1.dot(p);
    if a > -EPSILON && a < EPSILON {
        return None;
    }
    let f = 1.0 / a;
    let s = ray_origin - v0;
    let u = f * s.dot(p);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = s.cross(e1);
    let v = f * ray_direction.dot(q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let lambda = f * e2.dot(q);
    if lambda < 0.0 {
        return None;
    }
    Some((lambda, u, v))
}

/// `isect_ray_tri_epsilon_v3`.
pub fn isect_ray_tri_epsilon(
    ray_origin: Vec3,
    ray_direction: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    epsilon: f32,
) -> Option<(f32, f32, f32)> {
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let p = ray_direction.cross(e2);
    let a = e1.dot(p);
    if a == 0.0 {
        return None;
    }
    let f = 1.0 / a;
    let s = ray_origin - v0;
    let u = f * s.dot(p);
    if u < -epsilon || u > 1.0 + epsilon {
        return None;
    }
    let q = s.cross(e1);
    let v = f * ray_direction.dot(q);
    if v < -epsilon || u + v > 1.0 + epsilon {
        return None;
    }
    let lambda = f * e2.dot(q);
    if lambda < 0.0 {
        return None;
    }
    Some((lambda, u, v))
}

/// `IsectRayPrecalc` — watertight ray-tri precalc (Cycles algorithm).
#[derive(Debug, Clone, Copy)]
pub struct IsectRayPrecalc {
    pub kx: usize,
    pub ky: usize,
    pub kz: usize,
    pub sx: f32,
    pub sy: f32,
    pub sz: f32,
}

/// `isect_ray_tri_watertight_v3_precalc`.
pub fn isect_ray_tri_watertight_precalc(ray_direction: Vec3) -> IsectRayPrecalc {
    // dimension where the ray direction is maximal
    let kz = ray_direction.abs_max_axis();
    let mut kx = if kz != 2 { kz + 1 } else { 0 };
    let mut ky = if kx != 2 { kx + 1 } else { 0 };
    // swap to preserve winding
    if ray_direction[kz] < 0.0 {
        std::mem::swap(&mut kx, &mut ky);
    }
    let inv_dir_z = 1.0 / ray_direction[kz];
    IsectRayPrecalc {
        sx: ray_direction[kx] * inv_dir_z,
        sy: ray_direction[ky] * inv_dir_z,
        sz: inv_dir_z,
        kx,
        ky,
        kz,
    }
}

/// `isect_ray_tri_watertight_v3` — Woop watertight ray-triangle.
pub fn isect_ray_tri_watertight(
    ray_origin: Vec3,
    pre: &IsectRayPrecalc,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<(f32, f32, f32)> {
    let (kx, ky, kz) = (pre.kx, pre.ky, pre.kz);
    let (sx, sy, sz) = (pre.sx, pre.sy, pre.sz);

    let a = v0 - ray_origin;
    let b = v1 - ray_origin;
    let c = v2 - ray_origin;

    let (a_kx, a_ky, a_kz) = (a[kx], a[ky], a[kz]);
    let (b_kx, b_ky, b_kz) = (b[kx], b[ky], b[kz]);
    let (c_kx, c_ky, c_kz) = (c[kx], c[ky], c[kz]);

    let ax = a_kx - sx * a_kz;
    let ay = a_ky - sy * a_kz;
    let bx = b_kx - sx * b_kz;
    let by = b_ky - sy * b_kz;
    let cx = c_kx - sx * c_kz;
    let cy = c_ky - sy * c_kz;

    let u = cx * by - cy * bx;
    let v = ax * cy - ay * cx;
    let w = bx * ay - by * ax;

    if (u < 0.0 || v < 0.0 || w < 0.0) && (u > 0.0 || v > 0.0 || w > 0.0) {
        return None;
    }

    let det = u + v + w;
    if det == 0.0 || !det.is_finite() {
        return None;
    }

    // xor float sign bits with det's sign
    let sign_det = det.to_bits() & 0x8000_0000;
    let t = (u * a_kz + v * b_kz + w * c_kz) * sz;
    let sign_t = f32::from_bits(t.to_bits() ^ sign_det);
    if sign_t < 0.0 {
        return None;
    }

    let inv_det = 1.0 / det;
    Some((t * inv_det, u * inv_det, v * inv_det))
}

/// `isect_ray_tri_watertight_v3_simple`.
pub fn isect_ray_tri_watertight_simple(
    ray_origin: Vec3,
    ray_direction: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<(f32, f32, f32)> {
    let pre = isect_ray_tri_watertight_precalc(ray_direction);
    isect_ray_tri_watertight(ray_origin, &pre, v0, v1, v2)
}

/// `isect_ray_seg_v2` — 2D ray vs segment, `(t, u)` on hit.
pub fn isect_ray_seg_v2(
    ray_origin: Vec2,
    ray_direction: Vec2,
    v0: Vec2,
    v1: Vec2,
) -> Option<(f32, f32)> {
    let v0_local = v0 - ray_origin;
    let v1_local = v1 - ray_origin;
    let s10 = v1_local - v0_local;
    let det = ray_direction.cross(s10);
    if det == 0.0 {
        return None;
    }
    let v = v0_local.cross(v1_local);
    let p = ray_direction * (v / det);
    let t = p.dot(ray_direction) / ray_direction.dot(ray_direction);
    if t < 0.0 {
        return None;
    }
    let h = v1_local - p;
    let u = s10.dot(h) / s10.dot(s10);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    Some((t, u))
}

/// `isect_point_planes_v3_negated` — `p` inside all planes (inside = strict side > 0).
pub fn isect_point_planes_negated(planes: &[Vec4], p: Vec3) -> bool {
    planes.iter().all(|pl| plane_point_side(*pl, p) > 0.0)
}

/// `isect_plane_plane_plane_v3` — common point of three planes.
pub fn isect_plane_plane_plane(plane_a: Vec4, plane_b: Vec4, plane_c: Vec4) -> Option<Vec3> {
    let (a, b, c) = (plane_a.xyz(), plane_b.xyz(), plane_c.xyz());
    // determinant_m3(a, b, c) == a . (b x c)
    let det = a.dot(b.cross(c));
    if det != 0.0 {
        let mut r = c.cross(b) * plane_a.w;
        r += a.cross(c) * plane_b.w;
        r += b.cross(a) * plane_c.w;
        r *= 1.0 / det;
        Some(r)
    } else {
        None
    }
}

/* -------------------------------------------------------------------- */
/* Ray → AABB distance (`dist_squared_ray_to_aabb_v3*` family)            */
/* -------------------------------------------------------------------- */

use crate::matrix::{Mat3, Mat4};

/// `DistRayAABB_Precalc`.
#[derive(Debug, Clone, Copy)]
pub struct DistRayAabbPrecalc {
    pub ray_origin: Vec3,
    pub ray_direction: Vec3,
    pub ray_inv_dir: Vec3,
}

/// `dist_squared_ray_to_aabb_v3_precalc`.
pub fn dist_squared_ray_to_aabb_precalc(ray_origin: Vec3, ray_direction: Vec3) -> DistRayAabbPrecalc {
    let mut inv = Vec3::ZERO;
    for i in 0..3 {
        inv[i] = if ray_direction[i] != 0.0 {
            1.0 / ray_direction[i]
        } else {
            f32::MAX
        };
    }
    DistRayAabbPrecalc {
        ray_origin,
        ray_direction,
        ray_inv_dir: inv,
    }
}

/// `dist_squared_ray_to_aabb_v3` — `(dist², r_point, depth)`.
pub fn dist_squared_ray_to_aabb(
    data: &DistRayAabbPrecalc,
    bb_min: Vec3,
    bb_max: Vec3,
) -> (f32, Vec3, f32) {
    let (local_bvmin, local_bvmax) =
        aabb_get_near_far_from_plane(data.ray_direction, bb_min, bb_max);

    let tmin = [
        (local_bvmin[0] - data.ray_origin[0]) * data.ray_inv_dir[0],
        (local_bvmin[1] - data.ray_origin[1]) * data.ray_inv_dir[1],
        (local_bvmin[2] - data.ray_origin[2]) * data.ray_inv_dir[2],
    ];
    let tmax = [
        (local_bvmax[0] - data.ray_origin[0]) * data.ray_inv_dir[0],
        (local_bvmax[1] - data.ray_origin[1]) * data.ray_inv_dir[1],
        (local_bvmax[2] - data.ray_origin[2]) * data.ray_inv_dir[2],
    ];

    let mut va = Vec3::ZERO;
    let mut vb = Vec3::ZERO;
    let rtmin;
    let rtmax;
    let mut main_axis: i32;

    // first far plane the ray exits
    if tmax[0] <= tmax[1] && tmax[0] <= tmax[2] {
        rtmax = tmax[0];
        va[0] = local_bvmax[0];
        vb[0] = local_bvmax[0];
        main_axis = 3;
    } else if tmax[1] <= tmax[0] && tmax[1] <= tmax[2] {
        rtmax = tmax[1];
        va[1] = local_bvmax[1];
        vb[1] = local_bvmax[1];
        main_axis = 2;
    } else {
        rtmax = tmax[2];
        va[2] = local_bvmax[2];
        vb[2] = local_bvmax[2];
        main_axis = 1;
    }
    // last near plane the ray enters
    if tmin[0] >= tmin[1] && tmin[0] >= tmin[2] {
        rtmin = tmin[0];
        va[0] = local_bvmin[0];
        vb[0] = local_bvmin[0];
        main_axis -= 3;
    } else if tmin[1] >= tmin[0] && tmin[1] >= tmin[2] {
        rtmin = tmin[1];
        va[1] = local_bvmin[1];
        vb[1] = local_bvmin[1];
        main_axis -= 1;
    } else {
        rtmin = tmin[2];
        va[2] = local_bvmin[2];
        vb[2] = local_bvmin[2];
        main_axis -= 2;
    }
    if main_axis < 0 {
        main_axis += 3;
    }
    let main_axis = main_axis as usize;

    if rtmin <= rtmax {
        let dvec = local_bvmax - data.ray_origin;
        let depth = dvec.dot(data.ray_direction);
        return (0.0, local_bvmax, depth);
    }

    if data.ray_direction[main_axis] >= 0.0 {
        va[main_axis] = local_bvmin[main_axis];
        vb[main_axis] = local_bvmax[main_axis];
    } else {
        va[main_axis] = local_bvmax[main_axis];
        vb[main_axis] = local_bvmin[main_axis];
    }

    dist_squared_ray_to_seg(data.ray_origin, data.ray_direction, va, vb)
}

/// `dist_squared_ray_to_aabb_v3_simple`.
pub fn dist_squared_ray_to_aabb_simple(
    ray_origin: Vec3,
    ray_direction: Vec3,
    bb_min: Vec3,
    bb_max: Vec3,
) -> (f32, Vec3, f32) {
    let data = dist_squared_ray_to_aabb_precalc(ray_origin, ray_direction);
    dist_squared_ray_to_aabb(&data, bb_min, bb_max)
}

/* -------------------------------------------------------------------- */
/* Projected AABB (`dist_squared_to_projected_aabb*` family)              */
/* -------------------------------------------------------------------- */

/// `mul_project_m4_v3_zfac` — w after `M·co` (no divide).
#[inline]
pub fn mul_project_m4_v3_zfac(mat: &Mat4, co: Vec3) -> f32 {
    mat[0][3] * co.x + mat[1][3] * co.y + mat[2][3] * co.z + mat[3][3]
}

#[inline]
fn dot_m4_v3_row_x(m: &Mat4, a: Vec3) -> f32 {
    m[0][0] * a.x + m[1][0] * a.y + m[2][0] * a.z
}
#[inline]
fn dot_m4_v3_row_y(m: &Mat4, a: Vec3) -> f32 {
    m[0][1] * a.x + m[1][1] * a.y + m[2][1] * a.z
}

/// `DistProjectedAABBPrecalc`.
#[derive(Debug, Clone, Copy)]
pub struct DistProjectedAabbPrecalc {
    pub pmat: Mat4,
    pub mval: Vec2,
    pub ray_origin: Vec3,
    pub ray_direction: Vec3,
    pub ray_inv_dir: Vec3,
}

/// `isect_plane_plane_v3` — `(point, direction)` of the intersection line.
pub fn isect_plane_plane(plane_a: Vec4, plane_b: Vec4) -> Option<(Vec3, Vec3)> {
    let plane_c = plane_a.xyz().cross(plane_b.xyz());
    let det = plane_c.length_squared();
    if det != 0.0 {
        let mut r_isect_co = plane_c.cross(plane_b.xyz()) * plane_a.w;
        r_isect_co += plane_a.xyz().cross(plane_c) * plane_b.w;
        r_isect_co *= 1.0 / det;
        Some((r_isect_co, plane_c))
    } else {
        None
    }
}

/// `dist_squared_to_projected_aabb_precalc`.
pub fn dist_squared_to_projected_aabb_precalc(
    projmat: &Mat4,
    winsize: Vec2,
    mval: Vec2,
) -> DistProjectedAabbPrecalc {
    let win_half = winsize * 0.5;
    let mval_rel = mval - win_half;
    let relative_mval = Vec2::new(mval_rel.x / win_half.x, mval_rel.y / win_half.y);

    let mut pmat = *projmat;
    let mut px = Vec4::ZERO;
    let mut py = Vec4::ZERO;
    for i in 0..4 {
        px[i] = pmat[i][0] - pmat[i][3] * relative_mval.x;
        py[i] = pmat[i][1] - pmat[i][3] * relative_mval.y;
        pmat[i][0] *= win_half.x;
        pmat[i][1] *= win_half.y;
    }

    let (ray_origin, ray_direction) = match isect_plane_plane(px, py) {
        Some((co, no)) => (co, no),
        None => {
            // co-planar planes: undetermined origin
            (Vec3::ZERO, Vec3::new(pmat[0][3], pmat[1][3], pmat[2][3]))
        }
    };

    let mut ray_inv_dir = Vec3::ZERO;
    for i in 0..3 {
        ray_inv_dir[i] = if ray_direction[i] != 0.0 {
            1.0 / ray_direction[i]
        } else {
            f32::MAX
        };
    }

    DistProjectedAabbPrecalc {
        pmat,
        mval: mval_rel,
        ray_origin,
        ray_direction,
        ray_inv_dir,
    }
}

/// `dist_squared_to_projected_aabb` — `(dist², axis_closest[3])`.
pub fn dist_squared_to_projected_aabb(
    data: &DistProjectedAabbPrecalc,
    bbmin: Vec3,
    bbmax: Vec3,
) -> (f32, [bool; 3]) {
    let (local_bvmin, local_bvmax) =
        aabb_get_near_far_from_plane(data.ray_direction, bbmin, bbmax);

    let tmin = [
        (local_bvmin[0] - data.ray_origin[0]) * data.ray_inv_dir[0],
        (local_bvmin[1] - data.ray_origin[1]) * data.ray_inv_dir[1],
        (local_bvmin[2] - data.ray_origin[2]) * data.ray_inv_dir[2],
    ];
    let tmax = [
        (local_bvmax[0] - data.ray_origin[0]) * data.ray_inv_dir[0],
        (local_bvmax[1] - data.ray_origin[1]) * data.ray_inv_dir[1],
        (local_bvmax[2] - data.ray_origin[2]) * data.ray_inv_dir[2],
    ];

    let mut va = Vec3::ZERO;
    let mut vb = Vec3::ZERO;
    let rtmin;
    let rtmax;
    let mut main_axis: i32;
    let mut axis_closest = [false; 3];

    if tmax[0] <= tmax[1] && tmax[0] <= tmax[2] {
        rtmax = tmax[0];
        va[0] = local_bvmax[0];
        vb[0] = local_bvmax[0];
        main_axis = 3;
        axis_closest[0] = data.ray_direction[0] < 0.0;
    } else if tmax[1] <= tmax[0] && tmax[1] <= tmax[2] {
        rtmax = tmax[1];
        va[1] = local_bvmax[1];
        vb[1] = local_bvmax[1];
        main_axis = 2;
        axis_closest[1] = data.ray_direction[1] < 0.0;
    } else {
        rtmax = tmax[2];
        va[2] = local_bvmax[2];
        vb[2] = local_bvmax[2];
        main_axis = 1;
        axis_closest[2] = data.ray_direction[2] < 0.0;
    }

    if tmin[0] >= tmin[1] && tmin[0] >= tmin[2] {
        rtmin = tmin[0];
        va[0] = local_bvmin[0];
        vb[0] = local_bvmin[0];
        main_axis -= 3;
        axis_closest[0] = data.ray_direction[0] >= 0.0;
    } else if tmin[1] >= tmin[0] && tmin[1] >= tmin[2] {
        rtmin = tmin[1];
        va[1] = local_bvmin[1];
        vb[1] = local_bvmin[1];
        main_axis -= 1;
        axis_closest[1] = data.ray_direction[1] >= 0.0;
    } else {
        rtmin = tmin[2];
        va[2] = local_bvmin[2];
        vb[2] = local_bvmin[2];
        main_axis -= 2;
        axis_closest[2] = data.ray_direction[2] >= 0.0;
    }
    if main_axis < 0 {
        main_axis += 3;
    }
    let main_axis = main_axis as usize;

    // ray intersects the AABB
    if rtmin <= rtmax {
        return (0.0, axis_closest);
    }

    if data.ray_direction[main_axis] >= 0.0 {
        va[main_axis] = local_bvmin[main_axis];
        vb[main_axis] = local_bvmax[main_axis];
    } else {
        va[main_axis] = local_bvmax[main_axis];
        vb[main_axis] = local_bvmin[main_axis];
    }
    let scale = (local_bvmax[main_axis] - local_bvmin[main_axis]).abs();

    let mut va2d = Vec2::new(
        dot_m4_v3_row_x(&data.pmat, va) + data.pmat[3][0],
        dot_m4_v3_row_y(&data.pmat, va) + data.pmat[3][1],
    );
    let vb2d = Vec2::new(
        va2d.x + data.pmat[main_axis][0] * scale,
        va2d.y + data.pmat[main_axis][1] * scale,
    );

    let w_a = mul_project_m4_v3_zfac(&data.pmat, va);
    if w_a != 1.0 {
        // perspective projection
        let w_b = w_a + data.pmat[main_axis][3] * scale;
        va2d.x /= w_a;
        va2d.y /= w_a;
        let vb2d = Vec2::new(vb2d.x / w_b, vb2d.y / w_b);
        return projected_aabb_tail(data, va2d, vb2d, &mut axis_closest, main_axis);
    }
    projected_aabb_tail(data, va2d, vb2d, &mut axis_closest, main_axis)
}

#[inline]
fn projected_aabb_tail(
    data: &DistProjectedAabbPrecalc,
    va2d: Vec2,
    vb2d: Vec2,
    axis_closest: &mut [bool; 3],
    main_axis: usize,
) -> (f32, [bool; 3]) {
    let dvec = data.mval - va2d;
    let edge = vb2d - va2d;
    let mut lambda = dvec.dot(edge);
    let rdist_sq;
    if lambda != 0.0 {
        lambda /= edge.length_squared();
        if lambda <= 0.0 {
            rdist_sq = data.mval.distance_squared(va2d);
            axis_closest[main_axis] = true;
        } else if lambda >= 1.0 {
            rdist_sq = data.mval.distance_squared(vb2d);
            axis_closest[main_axis] = false;
        } else {
            let va2d = va2d + edge * lambda;
            rdist_sq = data.mval.distance_squared(va2d);
            axis_closest[main_axis] = lambda < 0.5;
        }
    } else {
        rdist_sq = data.mval.distance_squared(va2d);
    }
    (rdist_sq, *axis_closest)
}

/// `dist_squared_to_projected_aabb_simple`.
pub fn dist_squared_to_projected_aabb_simple(
    projmat: &Mat4,
    winsize: Vec2,
    mval: Vec2,
    bbmin: Vec3,
    bbmax: Vec3,
) -> f32 {
    let data = dist_squared_to_projected_aabb_precalc(projmat, winsize, mval);
    dist_squared_to_projected_aabb(&data, bbmin, bbmax).0
}

/* -------------------------------------------------------------------- */
/* Triangle closest point (Ericson, RTCD)                                */
/* -------------------------------------------------------------------- */

/// `closest_on_tri_to_point_v3` — `(closest_point, barycentric_weights)`.
pub fn closest_on_tri_to_point(p: Vec3, v1: Vec3, v2: Vec3, v3: Vec3) -> (Vec3, Vec3) {
    // adapted from "Real-Time Collision Detection" by Christer Ericson
    let ab = v2 - v1;
    let ac = v3 - v1;
    let ap = p - v1;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return (v1, Vec3::new(1.0, 0.0, 0.0));
    }

    let bp = p - v2;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return (v2, Vec3::new(0.0, 1.0, 0.0));
    }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let ab_squared = d1 - d3;
        if ab_squared == 0.0 {
            return (v1, Vec3::new(1.0, 0.0, 0.0));
        }
        let v = d1 / ab_squared;
        return (v1.madd(ab, v), Vec3::new(1.0 - v, v, 0.0));
    }

    let cp = p - v3;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return (v3, Vec3::new(0.0, 0.0, 1.0));
    }

    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let ac_squared = d2 - d6;
        if ac_squared == 0.0 {
            return (v1, Vec3::new(1.0, 0.0, 0.0));
        }
        let w = d2 / ac_squared;
        return (v1.madd(ac, w), Vec3::new(1.0 - w, 0.0, w));
    }

    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let bc_squared = (d4 - d3) + (d5 - d6);
        if bc_squared == 0.0 {
            return (v2, Vec3::new(0.0, 1.0, 0.0));
        }
        let w = (d4 - d3) / bc_squared;
        return (v2 + (v3 - v2) * w, Vec3::new(0.0, 1.0 - w, w));
    }

    // inside face region
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    let r = v1.madd(ab, v) + ac * w;
    (r, Vec3::new(1.0 - v - w, v, w))
}

/// `isect_seg_seg_v2_int` — integer-coordinate version of `isect_seg_seg_v2`.
pub fn isect_seg_seg_v2_int(v1: [i32; 2], v2: [i32; 2], v3: [i32; 2], v4: [i32; 2]) -> Isect {
    let div = ((v2[0] - v1[0]) * (v4[1] - v3[1]) - (v2[1] - v1[1]) * (v4[0] - v3[0])) as f32;
    if div == 0.0 {
        return Isect::Colinear;
    }
    let lambda =
        ((v1[1] - v3[1]) * (v4[0] - v3[0]) - (v1[0] - v3[0]) * (v4[1] - v3[1])) as f32 / div;
    let mu =
        ((v1[1] - v3[1]) * (v2[0] - v1[0]) - (v1[0] - v3[0]) * (v2[1] - v1[1])) as f32 / div;
    if (0.0..=1.0).contains(&lambda) && (0.0..=1.0).contains(&mu) {
        if lambda == 0.0 || lambda == 1.0 || mu == 0.0 || mu == 1.0 {
            return Isect::Exact;
        }
        return Isect::Cross;
    }
    Isect::None
}

/* -------------------------------------------------------------------- */
/* Misc                                                                  */
/* -------------------------------------------------------------------- */

/// `poly_to_tri_count` — triangles needed to tessellate `poly_count` n-gons
/// with `corner_count` total corners (each n-gon needs n-2 tris).
pub const fn poly_to_tri_count(poly_count: i32, corner_count: i32) -> i32 {
    corner_count - (poly_count * 2)
}

/// `interp_bilinear_quad` — bilinear patch: v0(u0,v0) v1(u1,v0) v2(u1,v1) v3(u0,v1).
pub fn interp_bilinear_quad(v0: Vec3, v1: Vec3, v2: Vec3, v3: Vec3, u: f32, v: f32) -> Vec3 {
    v0 * ((1.0 - u) * (1.0 - v)) + v1 * (u * (1.0 - v)) + v2 * (u * v) + v3 * ((1.0 - u) * v)
}

/// `map_to_sphere` — direction → spherical (u, v) texture coordinates.
pub fn map_to_sphere(co: Vec3) -> Vec2 {
    // blender's map_to_sphere (BLI_utildefines / math_geom area helpers)
    let len = co.length();
    let (u, v);
    if co.x == 0.0 && co.y == 0.0 {
        u = 0.0;
        v = if co.z > 0.0 { 1.0 } else { 0.0 };
    } else {
        let _ = len;
        u = (1.0 - co.x.atan2(co.y) / std::f32::consts::PI) * 0.5;
        v = 1.0 - (co.z / len).acos() / std::f32::consts::PI;
    }
    Vec2::new(u, v)
}

#[cfg(test)]
mod tests {
    use super::*;
    

    const EPS: f32 = 1e-5;
    fn close(a: f32, b: f32) {
        assert!((a - b).abs() < EPS, "{a} != {b}");
    }

    #[test]
    fn test_area_tri() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(2.0, 0.0, 0.0);
        let c = Vec3::new(0.0, 2.0, 0.0);
        close(area_tri(a, b, c), 2.0);
        close(area_tri_signed(a, b, c, Vec3::Z), 2.0);
        close(area_tri_signed(a, b, c, -Vec3::Z), -2.0);
        let (n, _) = normal_tri(a, b, c);
        close(n.z, 1.0);
    }

    #[test]
    fn test_planes() {
        let plane = plane_from_point_normal(Vec3::new(0.0, 0.0, 5.0), Vec3::Z);
        close(plane.w, -5.0);
        close(plane_point_side(plane, Vec3::new(0.0, 0.0, 7.0)), 2.0);
        let close_pt = closest_to_plane(plane, Vec3::new(1.0, 2.0, 8.0));
        close(close_pt.z, 5.0);
        let (co, no) = plane_to_point_vector_normalized(plane);
        close(co.z, 5.0);
        close(no.z, 1.0);
    }

    #[test]
    fn test_volume() {
        let a = Vec3::ZERO;
        let b = Vec3::X;
        let c = Vec3::Y;
        let d = Vec3::Z;
        close(volume_tetrahedron(a, b, c, d), 1.0 / 6.0);
    }

    #[test]
    fn test_distances() {
        let l1 = Vec3::new(0.0, 0.0, 0.0);
        let l2 = Vec3::new(1.0, 0.0, 0.0);
        let p = Vec3::new(0.5, 1.0, 0.0);
        close(dist_to_line_v3(p, l1, l2), 1.0);
        close(dist_to_line_segment_v3(p, l1, l2), 1.0);
        let (c, lam) = closest_to_line_segment_v3(Vec3::new(2.0, 1.0, 0.0), l1, l2);
        close(c.x, 1.0);
        close(lam, 1.0);
    }

    #[test]
    fn test_isect() {
        // crossing segments
        let r = isect_seg_seg_v2(
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(0.0, 2.0),
            Vec2::new(2.0, 0.0),
        );
        assert_eq!(r, Isect::Cross);
        // point in tri
        assert_eq!(
            isect_point_tri(
                Vec2::new(0.25, 0.25),
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0)
            ),
            1
        );
        // line x plane
        let isect = isect_line_plane(
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::ZERO,
            Vec3::Z,
        );
        close(isect.unwrap().z, 0.0);
        // line x sphere: through center along z, hits at +-r
        let (n, p1, p2) = isect_line_sphere(
            Vec3::new(0.0, 0.0, -5.0),
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::ZERO,
            1.0,
        );
        assert_eq!(n, 2);
        close(p1.z, 1.0);
        close(p2.z, -1.0);
    }

    #[test]
    fn test_corner() {
        // right-angle corner at origin, edges along +x and +y, axis +z
        let v2 = Vec3::ZERO;
        let v1 = Vec3::X;
        let v3 = Vec3::Y;
        // inside the wedge -> negative signed distance
        let inside = Vec3::new(0.5, 0.5, 1.0);
        assert!(dist_signed_squared_to_corner(inside, v1, v2, v3, Vec3::Z) < 0.0);
        // outside the wedge -> positive
        let outside = Vec3::new(-0.5, -0.5, 1.0);
        assert!(dist_signed_squared_to_corner(outside, v1, v2, v3, Vec3::Z) > 0.0);
    }
}

/// `isect_planes_v3_fn` — calls `cb` for each common point of plane triples
/// inside all planes; returns whether any was found.
pub fn isect_planes_fn(
    planes: &[Vec4],
    eps_coplanar: f32,
    eps_isect: f32,
    mut cb: impl FnMut(Vec3, usize, usize, usize),
) -> bool {
    let mut found = false;
    let n = planes.len();
    for i in 0..n {
        let n1 = planes[i];
        for j in i + 1..n {
            let n2 = planes[j];
            let n1n2 = n1.xyz().cross(n2.xyz());
            if n1n2.length_squared() <= eps_coplanar {
                continue;
            }
            for k in j + 1..n {
                let n3 = planes[k];
                let n2n3 = n2.xyz().cross(n3.xyz());
                if n2n3.length_squared() <= eps_coplanar {
                    continue;
                }
                let n3n1 = n3.xyz().cross(n1.xyz());
                if n3n1.length_squared() <= eps_coplanar {
                    continue;
                }
                let quotient = -n1.xyz().dot(n2n3);
                if quotient.abs() < eps_coplanar {
                    continue;
                }
                let co_test = Vec3::new(
                    (n2n3.x * n1.w + n3n1.x * n2.w + n1n2.x * n3.w) / quotient,
                    (n2n3.y * n1.w + n3n1.y * n2.w + n1n2.y * n3.w) / quotient,
                    (n2n3.z * n1.w + n3n1.z * n2.w + n1n2.z * n3.w) / quotient,
                );
                let mut ok = true;
                for (i_test, np) in planes.iter().enumerate() {
                    if np.xyz().dot(co_test) + np.w > eps_isect
                        && i_test != i && i_test != j && i_test != k
                    {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    cb(co_test, i, j, k);
                    found = true;
                }
            }
        }
    }
    found
}

fn sub_v3db(a: Vec3, b: Vec3) -> [f64; 3] {
    [
        a.x as f64 - b.x as f64,
        a.y as f64 - b.y as f64,
        a.z as f64 - b.z as f64,
    ]
}
fn cross_v3db(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn dot_v3db_v3fl(a: [f64; 3], b: Vec3) -> f64 {
    a[0] * b.x as f64 + a[1] * b.y as f64 + a[2] * b.z as f64
}

/// `isect_tri_tri_v3_ex` — `(hit, p1, p2, tri_a_edge_isect_count)`.
pub fn isect_tri_tri_ex(tri_a: &[Vec3; 3], tri_b: &[Vec3; 3]) -> Option<(Vec3, Vec3, i32)> {
    struct Range {
        min: f32,
        max: f32,
        loc: [Vec3; 2],
    }
    let mut side = [[0.0f32; 3]; 2];

    let ba = sub_v3db(tri_a[0], tri_a[1]);
    let bc = sub_v3db(tri_a[2], tri_a[1]);
    let plane_a = cross_v3db(ba, bc);
    let plane_a_w = -dot_v3db_v3fl(plane_a, tri_a[1]);
    side[1][0] = (dot_v3db_v3fl(plane_a, tri_b[0]) + plane_a_w) as f32;
    side[1][1] = (dot_v3db_v3fl(plane_a, tri_b[1]) + plane_a_w) as f32;
    side[1][2] = (dot_v3db_v3fl(plane_a, tri_b[2]) + plane_a_w) as f32;

    if side[1][0] == 0.0 && side[1][1] == 0.0 && side[1][2] == 0.0 {
        return None; // coplanar unsupported
    }
    if side[1][0] != 0.0
        && side[1][1] != 0.0
        && side[1][2] != 0.0
        && (side[1][0] < 0.0) == (side[1][1] < 0.0)
        && (side[1][0] < 0.0) == (side[1][2] < 0.0)
    {
        return None;
    }

    let ba = sub_v3db(tri_b[0], tri_b[1]);
    let bc = sub_v3db(tri_b[2], tri_b[1]);
    let plane_b = cross_v3db(ba, bc);
    let plane_b_w = -dot_v3db_v3fl(plane_b, tri_b[1]);
    side[0][0] = (dot_v3db_v3fl(plane_b, tri_a[0]) + plane_b_w) as f32;
    side[0][1] = (dot_v3db_v3fl(plane_b, tri_a[1]) + plane_b_w) as f32;
    side[0][2] = (dot_v3db_v3fl(plane_b, tri_a[2]) + plane_b_w) as f32;

    if side[0][0] != 0.0
        && side[0][1] != 0.0
        && side[0][2] != 0.0
        && (side[0][0] < 0.0) == (side[0][1] < 0.0)
        && (side[0][0] < 0.0) == (side[0][2] < 0.0)
    {
        return None;
    }

    let isect_dir = cross_v3db(plane_a, plane_b);
    let mut range = [
        Range {
            min: 0.0,
            max: 0.0,
            loc: [Vec3::ZERO; 2],
        },
        Range {
            min: 0.0,
            max: 0.0,
            loc: [Vec3::ZERO; 2],
        },
    ];

    for i in 0..2 {
        let tri = if i == 0 { tri_a } else { tri_b };
        let mut tri_i = if side[i][0] != 0.0
            && side[i][1] != 0.0
            && (side[i][0] < 0.0) == (side[i][1] < 0.0)
        {
            [1usize, 2, 0]
        } else if side[i][1] != 0.0
            && side[i][2] != 0.0
            && (side[i][1] < 0.0) == (side[i][2] < 0.0)
        {
            [2usize, 0, 1]
        } else {
            [0usize, 1, 2]
        };

        let dot_b = dot_v3db_v3fl(isect_dir, tri[tri_i[1]]);
        let sidec = side[i][tri_i[1]];
        if sidec != 0.0 {
            let dot_a = dot_v3db_v3fl(isect_dir, tri[tri_i[0]]);
            let dot_c = dot_v3db_v3fl(isect_dir, tri[tri_i[2]]);
            let mut fac0 = sidec / (sidec - side[i][tri_i[0]]);
            let mut fac1 = sidec / (sidec - side[i][tri_i[2]]);
            let mut offset0 = fac0 as f64 * (dot_a - dot_b);
            let mut offset1 = fac1 as f64 * (dot_c - dot_b);
            if offset0 > offset1 {
                std::mem::swap(&mut offset0, &mut offset1);
                std::mem::swap(&mut fac0, &mut fac1);
                tri_i.swap(0, 2);
            }
            range[i].min = (dot_b + offset0) as f32;
            range[i].max = (dot_b + offset1) as f32;
            range[i].loc[0] = tri[tri_i[1]] * (1.0 - fac0) + tri[tri_i[0]] * fac0;
            range[i].loc[1] = tri[tri_i[1]] * (1.0 - fac1) + tri[tri_i[2]] * fac1;
        } else {
            range[i].min = dot_b as f32;
            range[i].max = dot_b as f32;
            range[i].loc[0] = tri[tri_i[1]];
            range[i].loc[1] = tri[tri_i[1]];
        }
    }

    if range[0].max > range[1].min && range[0].min < range[1].max {
        let (i1, i2, cnt);
        if range[0].min >= range[1].min {
            i1 = range[0].loc[0];
            if range[0].max <= range[1].max {
                i2 = range[0].loc[1];
                cnt = 2;
            } else {
                i2 = range[1].loc[1];
                cnt = 1;
            }
        } else if range[0].max <= range[1].max {
            i1 = range[0].loc[1];
            i2 = range[1].loc[0];
            cnt = 1;
        } else {
            i1 = range[1].loc[0];
            i2 = range[1].loc[1];
            cnt = 0;
        }
        return Some((i1, i2, cnt));
    }
    None
}

/// `isect_tri_tri_v3` — segment endpoints of the tri/tri intersection.
pub fn isect_tri_tri(tri_a: &[Vec3; 3], tri_b: &[Vec3; 3]) -> Option<(Vec3, Vec3)> {
    isect_tri_tri_ex(tri_a, tri_b).map(|(a, b, _)| (a, b))
}

/* Tri-tri isect 2D (Guigue & Devillers orientation predicates). */

fn tri_tri_v2_impl_vert(a0: Vec2, a1: Vec2, a2: Vec2, b0: Vec2, b1: Vec2, b2: Vec2) -> bool {
    let s = line_point_side_v2;
    if s(b2, b0, a1) >= 0.0 {
        if s(b2, b1, a1) <= 0.0 {
            if s(a0, b0, a1) > 0.0 {
                return s(a0, b1, a1) <= 0.0;
            }
            if s(a0, b0, a2) >= 0.0 {
                return s(a1, a2, b0) >= 0.0;
            }
            return false;
        }
        if s(a0, b1, a1) <= 0.0 {
            if s(b2, b1, a2) <= 0.0 {
                return s(a1, a2, b1) >= 0.0;
            }
            return false;
        }
        return false;
    }
    if s(b2, b0, a2) >= 0.0 {
        if s(a1, a2, b2) >= 0.0 {
            return s(a0, b0, a2) >= 0.0;
        }
        if s(a1, a2, b1) >= 0.0 {
            return s(b2, a2, b1) >= 0.0;
        }
        return false;
    }
    false
}

fn tri_tri_v2_impl_edge(a0: Vec2, a1: Vec2, a2: Vec2, b0: Vec2, b2: Vec2) -> bool {
    let s = line_point_side_v2;
    if s(b2, b0, a1) >= 0.0 {
        if s(a0, b0, a1) >= 0.0 {
            return s(a0, a1, b2) >= 0.0;
        }
        if s(a1, a2, b0) >= 0.0 {
            return s(a2, a0, b0) >= 0.0;
        }
        return false;
    }
    if s(b2, b0, a2) >= 0.0 {
        if s(a0, b0, a2) >= 0.0 {
            return s(a0, a2, b2) >= 0.0 || s(a1, a2, b2) >= 0.0;
        }
        return false;
    }
    false
}

fn tri_tri_impl_ccw_v2(a0: Vec2, a1: Vec2, a2: Vec2, b0: Vec2, b1: Vec2, b2: Vec2) -> bool {
    let s = line_point_side_v2;
    if s(b0, b1, a0) >= 0.0 {
        if s(b1, b2, a0) >= 0.0 {
            if s(b2, b0, a0) >= 0.0 {
                return true;
            }
            return tri_tri_v2_impl_edge(a0, a1, a2, b0, b2);
        }
        if s(b2, b0, a0) >= 0.0 {
            return tri_tri_v2_impl_edge(a0, a1, a2, b2, b0);
        }
        return tri_tri_v2_impl_vert(a0, a1, a2, b0, b1, b2);
    }
    if s(b1, b2, a0) >= 0.0 {
        if s(b2, b0, a0) >= 0.0 {
            return tri_tri_v2_impl_edge(a0, a1, a2, b1, b2);
        }
        return tri_tri_v2_impl_vert(a0, a1, a2, b1, b2, b0);
    }
    tri_tri_v2_impl_vert(a0, a1, a2, b2, b0, b1)
}

/// `isect_tri_tri_v2`.
pub fn isect_tri_tri_v2(a0: Vec2, a1: Vec2, a2: Vec2, b0: Vec2, b1: Vec2, b2: Vec2) -> bool {
    let s = line_point_side_v2;
    if s(a0, a1, a2) < 0.0 {
        if s(b0, b1, b2) < 0.0 {
            return tri_tri_impl_ccw_v2(a0, a2, a1, b0, b2, b1);
        }
        return tri_tri_impl_ccw_v2(a0, a2, a1, b0, b1, b2);
    }
    if s(b0, b1, b2) < 0.0 {
        return tri_tri_impl_ccw_v2(a0, a1, a2, b0, b2, b1);
    }
    tri_tri_impl_ccw_v2(a0, a1, a2, b0, b1, b2)
}

/* Sphere sweep (Fauerby "Improved Collision Detection and Response"). */

fn get_lowest_root(a: f32, b: f32, c: f32, max_r: f32) -> Option<f32> {
    let determinant = b * b - 4.0 * a * c;
    if determinant < 0.0 {
        return None;
    }
    let sqrt_d = determinant.sqrt();
    let mut r1 = (-b - sqrt_d) / (2.0 * a);
    let mut r2 = (-b + sqrt_d) / (2.0 * a);
    if r1 > r2 {
        std::mem::swap(&mut r1, &mut r2);
    }
    if r1 > 0.0 && r1 < max_r {
        return Some(r1);
    }
    if r2 > 0.0 && r2 < max_r {
        return Some(r2);
    }
    None
}

/// `isect_aabb_planes_v3` — ISECT_AABB_PLANE_* result.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AabbPlaneIsect {
    /// `ISECT_AABB_PLANE_BEHIND_ANY`.
    BehindAny = 0,
    /// `ISECT_AABB_PLANE_CROSS_ANY`.
    CrossAny = 1,
    /// `ISECT_AABB_PLANE_IN_FRONT_ALL`.
    InFrontAll = 2,
}

/// `isect_aabb_planes_v3`.
pub fn isect_aabb_planes(planes: &[Vec4], bbmin: Vec3, bbmax: Vec3) -> AabbPlaneIsect {
    let mut ret = AabbPlaneIsect::InFrontAll;
    for plane in planes {
        let (bb_near, bb_far) = aabb_get_near_far_from_plane(plane.xyz(), bbmin, bbmax);
        if plane_point_side(*plane, bb_far) < 0.0 {
            return AabbPlaneIsect::BehindAny;
        }
        if ret != AabbPlaneIsect::CrossAny && plane_point_side(*plane, bb_near) < 0.0 {
            ret = AabbPlaneIsect::CrossAny;
        }
    }
    ret
}

/// `isect_sweeping_sphere_tri_v3` — `(lambda, impact_point)`.
pub fn isect_sweeping_sphere_tri(
    p1: Vec3,
    p2: Vec3,
    radius: f32,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<(f32, Vec3)> {
    let radius2 = radius * radius;
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let vel = p2 - p1;

    // test plane of tri
    let mut nor = e1.cross(e2);
    nor.normalize();
    if nor.dot(vel) > 0.0 {
        nor = -nor;
    }
    let mut a = p1.dot(nor) - v0.dot(nor);
    let nordotv = nor.dot(vel);
    if nordotv.abs() < 0.000001 {
        if a.abs() >= radius {
            return None;
        }
    } else {
        let mut t0 = (-a + radius) / nordotv;
        let mut t1 = (-a - radius) / nordotv;
        if t0 > t1 {
            std::mem::swap(&mut t0, &mut t1);
        }
        if t0 > 1.0 || t1 < 0.0 {
            return None;
        }
        let t0 = t0.clamp(0.0, 1.0);
        let point = p1 + vel * t0 - nor * radius;

        // is the point in the tri?
        a = e1.dot(e1);
        let b = e1.dot(e2);
        let c = e2.dot(e2);
        let temp = point - v0;
        let d = temp.dot(e1);
        let e = temp.dot(e2);
        let x = d * c - e * b;
        let y = e * a - d * b;
        let z = x + y - (a * c - b * b);
        if z <= 0.0 && x >= 0.0 && y >= 0.0 {
            return Some((t0, point));
        }
    }

    let mut lambda = 1.0f32;
    let mut ipoint = Vec3::ZERO;
    let mut found = false;

    // test points
    let va = vel.dot(vel);
    for v in [v0, v1, v2] {
        let temp = p1 - v;
        let b = 2.0 * vel.dot(temp);
        let c = temp.dot(temp) - radius2;
        if let Some(root) = get_lowest_root(va, b, c, lambda) {
            lambda = root;
            ipoint = v;
            found = true;
        }
    }

    // test edges
    let e3 = v2 - v1;
    // e1, e2 share bv = v0 - p1
    for (edge, origin) in [(e1, v0), (e2, v0)] {
        let bv = v0 - p1;
        let elen2 = edge.dot(edge);
        let edotv = edge.dot(vel);
        let edotbv = edge.dot(bv);
        let a = elen2 * -vel.dot(vel) + edotv * edotv;
        let b = 2.0 * (elen2 * vel.dot(bv) - edotv * edotbv);
        let c = elen2 * (radius2 - bv.dot(bv)) + edotbv * edotbv;
        if let Some(new_lambda) = get_lowest_root(a, b, c, lambda) {
            let e = (edotv * new_lambda - edotbv) / elen2;
            if (0.0..=1.0).contains(&e) {
                lambda = new_lambda;
                ipoint = edge * e + origin;
                found = true;
            }
        }
    }
    {
        let bv = v1 - p1;
        let elen2 = e3.dot(e3);
        let edotv = e3.dot(vel);
        let edotbv = e3.dot(bv);
        let a = elen2 * -vel.dot(vel) + edotv * edotv;
        let b = 2.0 * (elen2 * vel.dot(bv) - edotv * edotbv);
        let c = elen2 * (radius2 - bv.dot(bv)) + edotbv * edotbv;
        if let Some(new_lambda) = get_lowest_root(a, b, c, lambda) {
            let e = (edotv * new_lambda - edotbv) / elen2;
            if (0.0..=1.0).contains(&e) {
                lambda = new_lambda;
                ipoint = e3 * e + v1;
                found = true;
            }
        }
    }

    if found {
        Some((lambda, ipoint))
    } else {
        None
    }
}

/// `isect_axial_line_segment_tri_v3`.
pub fn isect_axial_line_segment_tri(
    axis: usize,
    p1: Vec3,
    p2: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<f32> {
    const EPSILON: f32 = 0.000001;
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let p = v0 - p1;
    let a0 = axis;
    let a1 = (axis + 1) % 3;
    let a2 = (axis + 2) % 3;

    let mut f = e2[a1] * e1[a2] - e2[a2] * e1[a1];
    if f > -EPSILON && f < EPSILON {
        return None;
    }
    let v = (p[a2] * e1[a1] - p[a1] * e1[a2]) / f;
    if !(0.0..=1.0).contains(&v) {
        return None;
    }
    f = e1[a1];
    let u;
    if f > -EPSILON && f < EPSILON {
        f = e1[a2];
        if f > -EPSILON && f < EPSILON {
            return None;
        }
        u = (-p[a2] - v * e2[a2]) / f;
    } else {
        u = (-p[a1] - v * e2[a1]) / f;
    }
    if u < 0.0 || u + v > 1.0 {
        return None;
    }
    let lambda = (p[a0] + u * e1[a0] + v * e2[a0]) / (p2[a0] - p1[a0]);
    if !(0.0..=1.0).contains(&lambda) {
        return None;
    }
    Some(lambda)
}

/// `isect_line_line_epsilon_v3` — 0 none / 1 coplanar point / 2 nearest points.
pub fn isect_line_line_epsilon(
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
    v4: Vec3,
    epsilon: f32,
) -> (i8, Vec3, Vec3) {
    let mut c = v3 - v1;
    let mut a = v2 - v1;
    let mut b = v4 - v3;
    let mut ab = a.cross(b);
    let d = c.dot(ab);
    let div = ab.dot(ab);

    // test zero length line
    if div == 0.0 {
        return (0, Vec3::ZERO, Vec3::ZERO);
    }
    // test if the two lines are coplanar
    if d.abs() <= epsilon {
        let cb = c.cross(b);
        let i1 = v1 + a * (cb.dot(ab) / div);
        return (1, i1, i1);
    }

    let mut t = v1 - v3;
    let n = a.cross(b);
    // project_v3_v3v3(t, t, n)
    t = t - n * (t.dot(n) / n.dot(n));

    let v3t = v3 + t;
    let v4t = v4 + t;
    c = v3t - v1;
    a = v2 - v1;
    b = v4t - v3t;
    ab = a.cross(b);
    let cb = c.cross(b);
    let i1 = v1 + a * (cb.dot(ab) / ab.dot(ab));
    let i2 = i1 - t;
    (2, i1, i2)
}

/// `isect_line_line_v3`.
pub fn isect_line_line(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> (i8, Vec3, Vec3) {
    isect_line_line_epsilon(v1, v2, v3, v4, 0.000001)
}

/// `isect_line_line_strict_v3` — segment-segment crossing point + lambda.
pub fn isect_line_line_strict(
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
    v4: Vec3,
) -> Option<(Vec3, f32)> {
    const EPSILON: f32 = 0.000001;
    let c = v3 - v1;
    let a = v2 - v1;
    let b = v4 - v3;
    let ab = a.cross(b);
    let d = c.dot(ab);
    let div = ab.dot(ab);
    if div == 0.0 || d.abs() < EPSILON {
        return None;
    }
    let cb = c.cross(b);
    let ca = c.cross(a);
    let f1 = cb.dot(ab) / div;
    let f2 = ca.dot(ab) / div;
    if (0.0..=1.0).contains(&f1) && (0.0..=1.0).contains(&f2) {
        return Some((v1 + a * f1, f1));
    }
    None
}

/// `isect_ray_ray_epsilon_v3` — `(lambda_a, lambda_b)`.
pub fn isect_ray_ray_epsilon(
    ray_origin_a: Vec3,
    ray_direction_a: Vec3,
    ray_origin_b: Vec3,
    ray_direction_b: Vec3,
    epsilon: f32,
) -> Option<(f32, f32)> {
    let n = ray_direction_b.cross(ray_direction_a);
    let nlen = n.length_squared();
    if nlen < epsilon {
        return None; // parallel
    }
    let t = ray_origin_b - ray_origin_a;
    let c = n - t;
    let la = c.cross(ray_direction_b).dot(n) / nlen;
    let lb = c.cross(ray_direction_a).dot(n) / nlen;
    Some((la, lb))
}

/// `isect_ray_ray_v3`.
pub fn isect_ray_ray(
    ray_origin_a: Vec3,
    ray_direction_a: Vec3,
    ray_origin_b: Vec3,
    ray_direction_b: Vec3,
) -> Option<(f32, f32)> {
    isect_ray_ray_epsilon(
        ray_origin_a,
        ray_direction_a,
        ray_origin_b,
        ray_direction_b,
        f32::MIN,
    )
}

/// `isect_aabb_aabb_v3`.
pub fn isect_aabb_aabb(min1: Vec3, max1: Vec3, min2: Vec3, max2: Vec3) -> bool {
    min1.x < max2.x
        && min1.y < max2.y
        && min1.z < max2.z
        && min2.x < max1.x
        && min2.y < max1.y
        && min2.z < max1.z
}

/// `IsectRayAABB_Precalc`.
#[derive(Debug, Clone, Copy)]
pub struct IsectRayAabbPrecalc {
    pub ray_origin: Vec3,
    pub ray_inv_dir: Vec3,
    pub sign: [usize; 3],
}

/// `isect_ray_aabb_v3_precalc`.
pub fn isect_ray_aabb_precalc(ray_origin: Vec3, ray_direction: Vec3) -> IsectRayAabbPrecalc {
    let inv = Vec3::new(
        1.0 / ray_direction.x,
        1.0 / ray_direction.y,
        1.0 / ray_direction.z,
    );
    IsectRayAabbPrecalc {
        ray_origin,
        ray_inv_dir: inv,
        sign: [
            (inv.x < 0.0) as usize,
            (inv.y < 0.0) as usize,
            (inv.z < 0.0) as usize,
        ],
    }
}

/// `isect_ray_aabb_v3` — ray-AABB slab test, returns `tmin`.
pub fn isect_ray_aabb(data: &IsectRayAabbPrecalc, bb_min: Vec3, bb_max: Vec3) -> Option<f32> {
    let bbox = [bb_min, bb_max];
    let mut tmin = (bbox[data.sign[0]][0] - data.ray_origin[0]) * data.ray_inv_dir[0];
    let mut tmax = (bbox[1 - data.sign[0]][0] - data.ray_origin[0]) * data.ray_inv_dir[0];
    let tymin = (bbox[data.sign[1]][1] - data.ray_origin[1]) * data.ray_inv_dir[1];
    let tymax = (bbox[1 - data.sign[1]][1] - data.ray_origin[1]) * data.ray_inv_dir[1];
    if tmin > tymax || tymin > tmax {
        return None;
    }
    tmin = tymin.max(tmin);
    tmax = tymax.min(tmax);
    let tzmin = (bbox[data.sign[2]][2] - data.ray_origin[2]) * data.ray_inv_dir[2];
    let tzmax = (bbox[1 - data.sign[2]][2] - data.ray_origin[2]) * data.ray_inv_dir[2];
    if tmin > tzmax || tzmin > tmax {
        return None;
    }
    tmin = tzmin.max(tmin);
    Some(tmin)
}

/// `isect_ray_aabb_v3_simple` — `(tmin, tmax)`.
pub fn isect_ray_aabb_simple(
    orig: Vec3,
    dir: Vec3,
    bb_min: Vec3,
    bb_max: Vec3,
) -> Option<(f32, f32)> {
    let invdir = |d: f32| -> f64 {
        if d > 1e-35 || d < -1e-35 {
            1.0 / d as f64
        } else {
            f64::MAX
        }
    };
    let (ix, iy, iz) = (invdir(dir.x), invdir(dir.y), invdir(dir.z));
    let t = [
        (bb_min.x - orig.x) as f64 * ix,
        (bb_max.x - orig.x) as f64 * ix,
        (bb_min.y - orig.y) as f64 * iy,
        (bb_max.y - orig.y) as f64 * iy,
        (bb_min.z - orig.z) as f64 * iz,
        (bb_max.z - orig.z) as f64 * iz,
    ];
    let hit_min = t[0].min(t[1]).max(t[2].min(t[3])).max(t[4].min(t[5])) as f32;
    let hit_max = t[0].max(t[1]).min(t[2].max(t[3])).min(t[4].max(t[5])) as f32;
    if hit_max < 0.0 || hit_min > hit_max {
        return None;
    }
    Some((hit_min, hit_max))
}

/// `closest_to_line_v2_db` — double-precision variant.
pub fn closest_to_line_v2_db(p: [f64; 2], l1: [f64; 2], l2: [f64; 2]) -> ([f64; 2], f64) {
    let u = [l2[0] - l1[0], l2[1] - l1[1]];
    let h = [p[0] - l1[0], p[1] - l1[1]];
    let denom = u[0] * u[0] + u[1] * u[1];
    if denom == 0.0 {
        return (l1, 0.0);
    }
    let lambda = (u[0] * h[0] + u[1] * h[1]) / denom;
    ([l1[0] + u[0] * lambda, l1[1] + u[1] * lambda], lambda)
}

/// `line_point_factor_v2_ex`.
pub fn line_point_factor_v2_ex(
    p: Vec2,
    l1: Vec2,
    l2: Vec2,
    epsilon: f32,
    fallback: f32,
) -> f32 {
    let u = l2 - l1;
    let h = p - l1;
    let dot = u.length_squared();
    if dot > epsilon {
        u.dot(h) / dot
    } else {
        fallback
    }
}

/// `line_point_factor_v2`.
pub fn line_point_factor_v2(p: Vec2, l1: Vec2, l2: Vec2) -> f32 {
    line_point_factor_v2_ex(p, l1, l2, 0.0, 0.0)
}

/// `line_plane_factor_v3`.
pub fn line_plane_factor(plane_co: Vec3, plane_no: Vec3, l1: Vec3, l2: Vec3) -> f32 {
    let u = l2 - l1;
    let h = l1 - plane_co;
    let dot = plane_no.dot(u);
    if dot != 0.0 {
        -plane_no.dot(h) / dot
    } else {
        0.0
    }
}

/// `limit_dist_v3` — pull `v1`, `v2` toward their midpoint until at most `dist` apart.
pub fn limit_dist(v1: &mut Vec3, v2: &mut Vec3, dist: f32) {
    let dist_old = (*v2 - *v1).length();
    if dist_old > dist {
        let fac = (dist / dist_old) * 0.5;
        let (v1_old, v2_old) = (*v1, *v2);
        *v1 = v1_old * (0.5 + fac) + v2_old * (0.5 - fac);
        *v2 = v1_old * (0.5 - fac) + v2_old * (0.5 + fac);
    }
}

/// `isect_point_tri_v2_int` — `p` inside the rect-half triangle of a rect's diagonal.
pub fn isect_point_tri_v2_int(x1: i32, y1: i32, x2: i32, y2: i32, a: i32, b: i32) -> i32 {
    isect_point_tri(
        Vec2::new(a as f32, b as f32),
        Vec2::new(x1 as f32, y1 as f32),
        Vec2::new(x1 as f32, y2 as f32),
        Vec2::new(x2 as f32, y1 as f32),
    )
}

/// `point_in_slice` — is `p` inside the slice of `v1` along line `l1->l2`.
fn point_in_slice(p: Vec3, v1: Vec3, l1: Vec3, l2: Vec3) -> bool {
    let (cp, _) = closest_to_line(v1, l1, l2);
    let q = cp - v1;
    let q_squared = q.dot(q);
    if q_squared.abs() < f32::EPSILON {
        return false;
    }
    let rp = p - v1;
    let h = q.dot(rp) / q_squared;
    (0.0..=1.0).contains(&h)
}

/// `point_in_slice_as` — slice defined by origin + (unnormalized) normal.
fn point_in_slice_as(p: Vec3, origin: Vec3, normal: Vec3) -> bool {
    let rp = p - origin;
    let h = normal.dot(rp) / normal.dot(normal);
    (0.0..=1.0).contains(&h)
}

/// `point_in_slice_seg`.
pub fn point_in_slice_seg(p: Vec3, l1: Vec3, l2: Vec3) -> bool {
    point_in_slice_as(p, l1, l2 - l1)
}

/// `isect_point_tri_prism_v3` — `p` inside the triangular prism extruded from the tri.
pub fn isect_point_tri_prism(p: Vec3, v1: Vec3, v2: Vec3, v3: Vec3) -> bool {
    point_in_slice(p, v1, v2, v3)
        && point_in_slice(p, v2, v3, v1)
        && point_in_slice(p, v3, v1, v2)
}

/// `isect_point_tri_v3` — closest point of `p` on the tri's plane when `p` is in the prism.
pub fn isect_point_tri_prism_closest(p: Vec3, v1: Vec3, v2: Vec3, v3: Vec3) -> Option<Vec3> {
    if isect_point_tri_prism(p, v1, v2, v3) {
        let no = cross_tri(v1, v2, v3);
        debug_assert!(no.length_squared() != 0.0);
        let plane = plane_from_point_normal(v1, no);
        Some(closest_to_plane(plane, p))
    } else {
        None
    }
}

/// `clip_segment_v3_plane` — clip the segment to the halfspace behind `plane`.
/// Returns `None` when fully clipped, else the (possibly shortened) segment.
pub fn clip_segment_plane(p1: Vec3, p2: Vec3, plane: Vec4) -> Option<(Vec3, Vec3)> {
    let dp = p2 - p1;
    let div = dp.dot(plane.xyz());
    if div == 0.0 {
        return Some((p1, p2)); // parallel
    }
    let t = -plane_point_side(plane, p1);
    if div > 0.0 {
        if t >= div {
            return None; // behind plane, completely clipped
        }
        if t > 0.0 {
            return Some((p1 + dp * (t / div), p2));
        }
    } else {
        if t >= 0.0 {
            return None; // behind plane, completely clipped
        }
        if t > div {
            return Some((p1, p1 + dp * (t / div)));
        }
    }
    Some((p1, p2))
}

/// `clip_segment_v3_plane_n` — clip against all `planes`.
pub fn clip_segment_plane_n(p1: Vec3, p2: Vec3, planes: &[Vec4]) -> Option<(Vec3, Vec3)> {
    let mut p1_fac = 0.0;
    let mut p2_fac = 1.0;
    let dp = p2 - p1;
    for plane in planes {
        let div = dp.dot(plane.xyz());
        if div != 0.0 {
            let mut t = -plane_point_side(*plane, p1);
            if div > 0.0 {
                if t >= div {
                    return None;
                }
                if t > 0.0 {
                    t /= div;
                    if t > p1_fac {
                        p1_fac = t;
                        if p1_fac > p2_fac {
                            return None;
                        }
                    }
                }
            } else {
                if t >= 0.0 {
                    return None;
                }
                if t > div {
                    t /= div;
                    if t < p2_fac {
                        p2_fac = t;
                        if p1_fac > p2_fac {
                            return None;
                        }
                    }
                }
            }
        }
    }
    Some((p1 + dp * p1_fac, p1 + dp * p2_fac))
}

/// `ortho_basis_v3v3_v3` — two unit vectors perpendicular to normalized `n`.
pub fn ortho_basis(n: Vec3) -> (Vec3, Vec3) {
    let f = n.x * n.x + n.y * n.y; // len_squared_v2(n)
    if f > FLT_EPSILON {
        let d = 1.0 / f.sqrt();
        let n1 = Vec3::new(n.y * d, -n.x * d, 0.0);
        let n2 = Vec3::new(-n.z * n1.y, n.z * n1.x, n.x * n1.y - n.y * n1.x);
        (n1, n2)
    } else {
        (
            Vec3::new(if n.z < 0.0 { -1.0 } else { 1.0 }, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        )
    }
}

/// `axis_dominant_v3_to_m3` — basis matrix whose rows project onto (n1, n2, normal):
/// `mul_v3_m3v3(mat, v)` = (v·n1, v·n2, v·n3). C fills rows then `transpose_m3`.
pub fn axis_dominant_to_m3(normal: Vec3) -> Mat3 {
    let (n1, n2) = ortho_basis(normal);
    Mat3::from_cols(n1, n2, normal).transposed()
}

/// `axis_dominant_v3_to_m3_negate`.
pub fn axis_dominant_to_m3_negate(normal: Vec3) -> Mat3 {
    let z = -normal;
    let (n1, n2) = ortho_basis(z);
    Mat3::from_cols(n1, n2, z).transposed()
}

fn tri_signed_area(v1: Vec3, v2: Vec3, v3: Vec3, i: usize, j: usize) -> f32 {
    0.5 * ((v1[i] - v2[i]) * (v2[j] - v3[j]) + (v1[j] - v2[j]) * (v3[i] - v2[i]))
}

/// `barycentric_weights` — returns `None` on degenerate (zero-area) triangles.
fn barycentric_weights(v1: Vec3, v2: Vec3, v3: Vec3, co: Vec3, n: Vec3) -> Option<Vec3> {
    let (i, j) = n.axis_dominant();
    let mut w = Vec3::new(
        tri_signed_area(v2, v3, co, i, j),
        tri_signed_area(v3, v1, co, i, j),
        tri_signed_area(v1, v2, co, i, j),
    );
    let wtot = w.x + w.y + w.z;
    w *= 1.0 / wtot;
    if w.is_finite() {
        Some(w)
    } else {
        None
    }
}

/// `interp_weights_tri_v3` — barycentric weights of `co` in the tri.
pub fn interp_weights_tri(v1: Vec3, v2: Vec3, v3: Vec3, co: Vec3) -> Vec3 {
    let n = normal_tri(v1, v2, v3).0;
    barycentric_weights(v1, v2, v3, co, n).unwrap_or(Vec3::new(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0))
}

/// `interp_weights_quad_v3`.
pub fn interp_weights_quad(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3, co: Vec3) -> [f32; 4] {
    let mut w = [0.0f32; 4];
    // exact match
    if co == v1 {
        w[0] = 1.0;
        return w;
    }
    if co == v2 {
        w[1] = 1.0;
        return w;
    }
    if co == v3 {
        w[2] = 1.0;
        return w;
    }
    if co == v4 {
        w[3] = 1.0;
        return w;
    }
    // barycentric weights over the two candidate triangles
    let n = (v1 - v3).cross(v2 - v4);
    match barycentric_weights(v1, v2, v4, co, n) {
        Some(w3) if w3[0] >= 0.0 => {
            w[0] = w3[0];
            w[1] = w3[1];
            w[3] = w3[2];
        }
        _ => {
            if let Some(w2) = barycentric_weights(v2, v3, v4, co, n) {
                w[1] = w2[0];
                w[2] = w2[1];
                w[3] = w2[2];
            }
        }
    }
    w
}

/// `barycentric_inside_triangle_v2` — 0 outside, 1 strictly inside, 2 inside-incl.
pub fn barycentric_inside_triangle_v2(w: Vec3) -> i32 {
    if w.x > 0.0 && w.x < 1.0 && w.y > 0.0 && w.y < 1.0 && w.z > 0.0 && w.z < 1.0 {
        return 1;
    }
    if w.x >= 0.0 && w.x <= 1.0 && w.y >= 0.0 && w.y <= 1.0 && w.z >= 0.0 && w.z <= 1.0 {
        return 2;
    }
    0
}

/// `barycentric_coords_v2` — exact barycentric coords in a 2D tri.
pub fn barycentric_coords_v2(v1: Vec2, v2: Vec2, v3: Vec2, co: Vec2) -> Option<Vec3> {
    let (x, y) = (co.x, co.y);
    let (x1, y1) = (v1.x, v1.y);
    let (x2, y2) = (v2.x, v2.y);
    let (x3, y3) = (v3.x, v3.y);
    let det = (y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3);
    let w = Vec3::new(
        ((y2 - y3) * (x - x3) + (x3 - x2) * (y - y3)) / det,
        ((y3 - y1) * (x - x3) + (x1 - x3) * (y - y3)) / det,
        0.0,
    );
    let w = Vec3::new(w.x, w.y, 1.0 - w.x - w.y);
    if w.is_finite() {
        Some(w)
    } else {
        None
    }
}

/// `cross_tri_v2`.
#[inline]
pub fn cross_tri_v2(v1: Vec2, v2: Vec2, v3: Vec2) -> f32 {
    (v1.x - v2.x) * (v2.y - v3.y) + (v1.y - v2.y) * (v3.x - v2.x)
}

/// `barycentric_weights_v2` — signed-area weights (fallback for degenerate).
pub fn barycentric_weights_v2(v1: Vec2, v2: Vec2, v3: Vec2, co: Vec2) -> Vec3 {
    let mut w = Vec3::new(
        cross_tri_v2(v2, v3, co),
        cross_tri_v2(v3, v1, co),
        cross_tri_v2(v1, v2, co),
    );
    let wtot = w.x + w.y + w.z;
    w *= 1.0 / wtot;
    if w.is_finite() {
        w
    } else {
        Vec3::new(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
    }
}

/// `barycentric_weights_v2_clamped`.
pub fn barycentric_weights_v2_clamped(v1: Vec2, v2: Vec2, v3: Vec2, co: Vec2) -> Vec3 {
    let mut w = Vec3::new(
        max_ff(cross_tri_v2(v2, v3, co), 0.0),
        max_ff(cross_tri_v2(v3, v1, co), 0.0),
        max_ff(cross_tri_v2(v1, v2, co), 0.0),
    );
    let wtot = w.x + w.y + w.z;
    w *= 1.0 / wtot;
    if w.is_finite() {
        w
    } else {
        Vec3::new(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
    }
}

/// `area_tri_signed_v2`.
#[inline]
pub fn area_tri_signed_v2(v1: Vec2, v2: Vec2, v3: Vec2) -> f32 {
    0.5 * ((v1.x - v2.x) * (v2.y - v3.y) + (v1.y - v2.y) * (v3.x - v2.x))
}

/// `area_tri_v2`.
#[inline]
pub fn area_tri_v2(v1: Vec2, v2: Vec2, v3: Vec2) -> f32 {
    area_tri_signed_v2(v1, v2, v3).abs()
}

/// `barycentric_weights_v2_persp` — perspective-correct weights (v are clip-space).
pub fn barycentric_weights_v2_persp(
    v1: Vec4,
    v2: Vec4,
    v3: Vec4,
    co: Vec2,
) -> Vec3 {
    let mut w = Vec3::new(
        cross_tri_v2(v2.xy(), v3.xy(), co) / v1.w,
        cross_tri_v2(v3.xy(), v1.xy(), co) / v2.w,
        cross_tri_v2(v1.xy(), v2.xy(), co) / v3.w,
    );
    let wtot = w.x + w.y + w.z;
    w *= 1.0 / wtot;
    if w.is_finite() {
        w
    } else {
        Vec3::new(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
    }
}

/// `barycentric_weights_v2_quad` — mean-value weights on a quad.
pub fn barycentric_weights_v2_quad(
    v1: Vec2,
    v2: Vec2,
    v3: Vec2,
    v4: Vec2,
    co: Vec2,
) -> [f32; 4] {
    let dirs = [v1 - co, v2 - co, v3 - co, v4 - co];
    let lens = [
        dirs[0].length(),
        dirs[1].length(),
        dirs[2].length(),
        dirs[3].length(),
    ];
    let mut w = [0.0f32; 4];
    for i in 0..4 {
        if lens[i] < FLT_EPSILON {
            w[i] = 1.0;
            return w;
        }
    }
    let areas = [
        dirs[0].cross(dirs[1]),
        dirs[1].cross(dirs[2]),
        dirs[2].cross(dirs[3]),
        dirs[3].cross(dirs[0]),
    ];
    let dots = [
        dirs[0].dot(dirs[1]),
        dirs[1].dot(dirs[2]),
        dirs[2].dot(dirs[3]),
        dirs[3].dot(dirs[0]),
    ];
    let lens_prod = [
        lens[0] * lens[1],
        lens[1] * lens[2],
        lens[2] * lens[3],
        lens[3] * lens[0],
    ];

    // point exactly on an edge
    for i in 0..4 {
        if areas[i].abs() < 1.0e-12 * lens_prod[i] && dots[i] <= 0.0 {
            let j = (i + 1) & 3;
            let sum = lens[i] + lens[j];
            w[i] = lens[j] / sum;
            w[j] = lens[i] / sum;
            return w;
        }
    }

    // NOTE(ideasman42): fabsf needed for concave/bow-tie quads (mask rasterizer).
    let t = [
        if areas[0] != 0.0 {
            ((lens_prod[0] - dots[0]) / areas[0]).abs()
        } else {
            0.0
        },
        if areas[1] != 0.0 {
            ((lens_prod[1] - dots[1]) / areas[1]).abs()
        } else {
            0.0
        },
        if areas[2] != 0.0 {
            ((lens_prod[2] - dots[2]) / areas[2]).abs()
        } else {
            0.0
        },
        if areas[3] != 0.0 {
            ((lens_prod[3] - dots[3]) / areas[3]).abs()
        } else {
            0.0
        },
    ];
    w[0] = (t[3] + t[0]) / lens[0];
    w[1] = (t[0] + t[1]) / lens[1];
    w[2] = (t[1] + t[2]) / lens[2];
    w[3] = (t[2] + t[3]) / lens[3];
    let wtot = w[0] + w[1] + w[2] + w[3];
    for x in w.iter_mut() {
        *x *= 1.0 / wtot;
    }
    if w.iter().all(|x| x.is_finite()) {
        w
    } else {
        [0.25; 4]
    }
}

/// `transform_point_by_tri_v3` — map `pt_src` to target tri space via barycentric weights.
pub fn transform_point_by_tri(
    pt_src: Vec3,
    tri_tar: &[Vec3; 3],
    tri_src: &[Vec3; 3],
) -> Vec3 {
    let no_tar = normal_tri(tri_tar[0], tri_tar[1], tri_tar[2]).0;
    let no_src = normal_tri(tri_src[0], tri_src[1], tri_src[2]).0;
    let mat_src = axis_dominant_to_m3(no_src);

    let pt_src_xy = mat_src * pt_src;
    let tri_xy_src: [Vec3; 3] = [
        mat_src * tri_src[0],
        mat_src * tri_src[1],
        mat_src * tri_src[2],
    ];
    let w_src = barycentric_weights_v2(
        tri_xy_src[0].xy(),
        tri_xy_src[1].xy(),
        tri_xy_src[2].xy(),
        pt_src_xy.xy(),
    );
    let mut pt_tar = tri_tar[0] * w_src.x + tri_tar[1] * w_src.y + tri_tar[2] * w_src.z;

    let area_tar = area_tri(tri_tar[0], tri_tar[1], tri_tar[2]).sqrt();
    let area_src = area_tri_v2(
        tri_xy_src[0].xy(),
        tri_xy_src[1].xy(),
        tri_xy_src[2].xy(),
    )
    .sqrt();
    let z_ofs_src = pt_src_xy.z - tri_xy_src[0].z;
    pt_tar += no_tar * ((z_ofs_src / area_src) * area_tar);
    pt_tar
}

/// `transform_point_by_seg_v3`.
pub fn transform_point_by_seg(
    p_src: Vec3,
    l_dst_p1: Vec3,
    l_dst_p2: Vec3,
    l_src_p1: Vec3,
    l_src_p2: Vec3,
) -> Vec3 {
    let t = line_point_factor(p_src, l_src_p1, l_src_p2);
    l_dst_p1 * (1.0 - t) + l_dst_p2 * t
}

/// `interp_sparse_array` — fill `skipval` entries by nearest-valid interpolation.
/// -1 = no valid entries, 0 = none invalid, 1 = interpolated.
pub fn interp_sparse_array(array: &mut [f32], skipval: f32) -> i32 {
    let list_size = array.len();
    let mut found_invalid = false;
    let mut found_valid = false;
    for &x in array.iter() {
        if x == skipval {
            found_invalid = true;
        } else {
            found_valid = true;
        }
    }
    if !found_valid {
        return -1;
    }
    if !found_invalid {
        return 0;
    }

    let mut array_up = vec![0.0f32; list_size];
    let mut array_down = vec![0.0f32; list_size];
    let mut ofs_tot_up = vec![0i32; list_size];
    let mut ofs_tot_down = vec![0i32; list_size];

    let mut valid_last = skipval;
    let mut valid_ofs = 0;
    for i in 0..list_size {
        if array[i] == skipval {
            array_up[i] = valid_last;
            valid_ofs += 1;
            ofs_tot_up[i] = valid_ofs;
        } else {
            valid_last = array[i];
            valid_ofs = 0;
        }
    }
    valid_last = skipval;
    valid_ofs = 0;
    for i in (0..list_size).rev() {
        if array[i] == skipval {
            array_down[i] = valid_last;
            valid_ofs += 1;
            ofs_tot_down[i] = valid_ofs;
        } else {
            valid_last = array[i];
            valid_ofs = 0;
        }
    }

    for i in 0..list_size {
        if array[i] == skipval {
            if array_up[i] != skipval && array_down[i] != skipval {
                array[i] = (array_up[i] * ofs_tot_down[i] as f32
                    + array_down[i] * ofs_tot_up[i] as f32)
                    / (ofs_tot_down[i] + ofs_tot_up[i]) as f32;
            } else if array_up[i] != skipval {
                array[i] = array_up[i];
            } else if array_down[i] != skipval {
                array[i] = array_down[i];
            }
        }
    }
    1
}

/* Mean-value interpolation weights for polygons (Mark Mayer et al). */

struct Float3Len {
    dir: Vec3,
    len: f32,
}
struct Double2Len {
    dir: [f64; 2],
    len: f64,
}

fn mean_value_half_tan_v3(d_curr: &Float3Len, d_next: &Float3Len) -> f32 {
    let cross = d_curr.dir.cross(d_next.dir);
    let area = cross.length();
    if area != 0.0 {
        let dot = d_curr.dir.dot(d_next.dir);
        let len = d_curr.len * d_next.len;
        let result = (len - dot) / area;
        if result.is_finite() {
            return result;
        }
    }
    0.0
}

fn mean_value_half_tan_v2_db(d_curr: &Double2Len, d_next: &Double2Len) -> f64 {
    let area = d_curr.dir[0] * d_next.dir[1] - d_curr.dir[1] * d_next.dir[0];
    if area != 0.0 {
        let dot = d_curr.dir[0] * d_next.dir[0] + d_curr.dir[1] * d_next.dir[1];
        let len = d_curr.len * d_next.len;
        let result = (len - dot) / area;
        if result.is_finite() {
            return result;
        }
    }
    0.0
}

const IS_POINT_IX: u8 = 1 << 0;
const IS_SEGMENT_IX: u8 = 1 << 1;

/// `interp_weights_poly_v3` — fills `w[i]` for each of the `n` polygon verts.
pub fn interp_weights_poly(w: &mut [f32], v: &[Vec3], co: Vec3) {
    let n = v.len();
    let mut max_value = 0.0f32;
    for vi in v {
        max_value = max_ff(max_value, (vi.x - co.x).abs());
        max_value = max_ff(max_value, (vi.y - co.y).abs());
        max_value = max_ff(max_value, (vi.z - co.z).abs());
    }
    let eps = 16.0 * FLT_EPSILON * max_value;
    let eps_sq = eps * eps;

    let mut i_curr = n - 1;
    let mut i_next = 0usize;
    let mut ix_flag = 0u8;
    let mut totweight = 0.0f32;

    let mut v_curr = v[i_curr];
    let mut v_next = v[i_next];
    let dir_len3 = |va: Vec3, vb: Vec3| -> Float3Len {
        let dir = va - vb;
        Float3Len {
            dir,
            len: dir.length(),
        }
    };
    let mut d_curr = dir_len3(v[n - 2], co);
    let mut d_next = dir_len3(v_curr, co);
    let mut ht_prev = mean_value_half_tan_v3(&d_curr, &d_next);

    while i_next < n {
        if d_next.len < eps {
            ix_flag = IS_POINT_IX;
            break;
        }
        if dist_squared_to_line_segment_v3(co, v_curr, v_next) < eps_sq {
            ix_flag = IS_SEGMENT_IX;
            break;
        }
        d_curr = d_next;
        d_next = dir_len3(v_next, co);
        let ht = mean_value_half_tan_v3(&d_curr, &d_next);
        w[i_curr] = (ht_prev + ht) / d_curr.len;
        totweight += w[i_curr];
        i_curr = i_next;
        i_next += 1;
        v_curr = v_next;
        v_next = v[i_next % n];
        ht_prev = ht;
    }

    if ix_flag != 0 {
        for x in w.iter_mut() {
            *x = 0.0;
        }
        if ix_flag & IS_POINT_IX != 0 {
            w[i_curr] = 1.0;
        } else {
            let fac = line_point_factor(co, v_curr, v_next).clamp(0.0, 1.0);
            w[i_curr] = 1.0 - fac;
            w[i_next % n] = fac;
        }
    } else if totweight != 0.0 {
        for x in w.iter_mut() {
            *x /= totweight;
        }
    }
}

/// `interp_weights_poly_v2`.
pub fn interp_weights_poly_v2(w: &mut [f32], v: &[Vec2], co: Vec2) {
    let n = v.len();
    let mut max_value = 0.0f32;
    for vi in v {
        max_value = max_ff(max_value, (vi.x - co.x).abs());
        max_value = max_ff(max_value, (vi.y - co.y).abs());
    }
    let eps = 16.0 * FLT_EPSILON * max_value;
    let eps_sq = eps * eps;

    let mut i_curr = n - 1;
    let mut i_next = 0usize;
    let mut ix_flag = 0u8;
    let mut totweight = 0.0f32;

    let mut v_curr = v[i_curr];
    let mut v_next = v[i_next];
    let dir_len2 = |va: Vec2, vb: Vec2| -> Double2Len {
        let dir = [
            va.x as f64 - vb.x as f64,
            va.y as f64 - vb.y as f64,
        ];
        Double2Len {
            dir,
            len: (dir[0] * dir[0] + dir[1] * dir[1]).sqrt(),
        }
    };
    let mut d_curr = dir_len2(v[n - 2], co);
    let mut d_next = dir_len2(v_curr, co);
    let mut ht_prev = mean_value_half_tan_v2_db(&d_curr, &d_next);

    while i_next < n {
        if d_next.len < eps as f64 {
            ix_flag = IS_POINT_IX;
            break;
        }
        if dist_squared_to_line_segment_v2(co, v_curr, v_next) < eps_sq {
            ix_flag = IS_SEGMENT_IX;
            break;
        }
        d_curr = d_next;
        d_next = dir_len2(v_next, co);
        let ht = mean_value_half_tan_v2_db(&d_curr, &d_next);
        w[i_curr] = if d_curr.len == 0.0 {
            0.0
        } else {
            ((ht_prev + ht) / d_curr.len) as f32
        };
        totweight += w[i_curr];
        i_curr = i_next;
        i_next += 1;
        v_curr = v_next;
        v_next = v[i_next % n];
        ht_prev = ht;
    }

    if ix_flag != 0 {
        for x in w.iter_mut() {
            *x = 0.0;
        }
        if ix_flag & IS_POINT_IX != 0 {
            w[i_curr] = 1.0;
        } else {
            let fac = line_point_factor_v2(co, v_curr, v_next).clamp(0.0, 1.0);
            w[i_curr] = 1.0 - fac;
            w[i_next % n] = fac;
        }
    } else if totweight != 0.0 {
        for x in w.iter_mut() {
            *x /= totweight;
        }
    }
}

/// `interp_cubic_v3` — Hermite interpolation `(x, v)` at parameter `t`.
pub fn interp_cubic(
    x1: Vec3,
    v1: Vec3,
    x2: Vec3,
    v2: Vec3,
    t: f32,
) -> (Vec3, Vec3) {
    let t2 = t * t;
    let t3 = t2 * t;
    let a = v1 + v2 + (x1 - x2) * 2.0;
    let b = v1 * -2.0 - v2 - (x1 - x2) * 3.0;
    (a * t3 + b * t2 + v1 * t + x1, a * (3.0 * t2) + b * (2.0 * t) + v1)
}

/// `resolve_tri_uv_v2`.
pub fn resolve_tri_uv_v2(st: Vec2, st0: Vec2, st1: Vec2, st2: Vec2) -> Vec2 {
    let a = (st0.x - st2.x) as f64;
    let b = (st1.x - st2.x) as f64;
    let c = (st0.y - st2.y) as f64;
    let d = (st1.y - st2.y) as f64;
    let det = a * d - c * b;
    if det > -f64::EPSILON && det < f64::EPSILON {
        return Vec2::ZERO;
    }
    let x = [
        (st.x - st2.x) as f64,
        (st.y - st2.y) as f64,
    ];
    Vec2::new(
        ((d * x[0] - b * x[1]) / det) as f32,
        ((-c * x[0] + a * x[1]) / det) as f32,
    )
}

/// `resolve_tri_uv_v3`.
pub fn resolve_tri_uv(st: Vec3, st0: Vec3, st1: Vec3, st2: Vec3) -> Vec2 {
    let v0 = st1 - st0;
    let v1 = st2 - st0;
    let v2 = st - st0;
    let d00 = v0.dot(v0) as f64;
    let d01 = v0.dot(v1) as f64;
    let d11 = v1.dot(v1) as f64;
    let d20 = v2.dot(v0) as f64;
    let d21 = v2.dot(v1) as f64;
    let det = d00 * d11 - d01 * d01;
    if det > -f64::EPSILON && det < f64::EPSILON {
        return Vec2::ZERO;
    }
    let w = ((d00 * d21 - d01 * d20) / det) as f32;
    let v = ((d11 * d20 - d01 * d21) / det) as f32;
    Vec2::new(1.0 - v - w, v)
}

fn quad_uv_coeffs(st: Vec2, st0: Vec2, st1: Vec2, st2: Vec2, st3: Vec2) -> (f64, f64, f64, f64) {
    let signed_area = ((st0.x * st1.y - st0.y * st1.x)
        + (st1.x * st2.y - st1.y * st2.x)
        + (st2.x * st3.y - st2.y * st3.x)
        + (st3.x * st0.y - st3.y * st0.x)) as f64;
    let a = ((st0.x - st.x) * (st0.y - st3.y) - (st0.y - st.y) * (st0.x - st3.x)) as f64;
    let b = 0.5 * (((st0.x - st.x) * (st1.y - st2.y) - (st0.y - st.y) * (st1.x - st2.x))
        + ((st1.x - st.x) * (st0.y - st3.y) - (st1.y - st.y) * (st0.x - st3.x)))
        as f64;
    let fc = ((st1.x - st.x) * (st1.y - st2.y) - (st1.y - st.y) * (st1.x - st2.x)) as f64;
    (signed_area, a, b, fc)
}

/// `resolve_quad_uv_v2_deriv` — `(uv, deriv)`.
pub fn resolve_quad_uv_v2_deriv(
    st: Vec2,
    st0: Vec2,
    st1: Vec2,
    st2: Vec2,
    st3: Vec2,
) -> (Vec2, [[f32; 2]; 2]) {
    let (signed_area, a, b, fc) = quad_uv_coeffs(st, st0, st1, st2, st3);
    let mut r_uv = Vec2::ZERO;
    let mut denom = a - 2.0 * b + fc;
    if denom > -f64::EPSILON && denom < f64::EPSILON {
        let f_den = a - fc;
        if f_den < -f64::EPSILON || f_den > f64::EPSILON {
            r_uv.x = (a / f_den) as f32;
        }
    } else {
        let desc_sq = b * b - a * fc;
        let desc = desc_sq.max(0.0).sqrt();
        let s = if signed_area > 0.0 { -1.0 } else { 1.0 };
        r_uv.x = (((a - b) + s * desc) / denom) as f32;
    }

    {
        let denom_s = ((1.0 - r_uv.x) * (st0.x - st3.x) + r_uv.x * (st1.x - st2.x)) as f64;
        let denom_t = ((1.0 - r_uv.x) * (st0.y - st3.y) + r_uv.x * (st1.y - st2.y)) as f64;
        let (i, d) = if denom_s.abs() < denom_t.abs() {
            (1usize, denom_t)
        } else {
            (0usize, denom_s)
        };
        denom = d;
        if denom < -f64::EPSILON || denom > f64::EPSILON {
            r_uv.y = (((1.0 - r_uv.x) * (st0[i] - st[i]) + r_uv.x * (st1[i] - st[i])) as f64
                / denom) as f32;
        }
    }

    let mut r_deriv = [[0.0f32; 2]; 2];
    {
        let tmp1 = st1 - st0;
        let tmp2 = st2 - st3;
        let s = tmp1 * (1.0 - r_uv.y) + tmp2 * r_uv.y;
        let tmp1 = st3 - st0;
        let tmp2 = st2 - st1;
        let t = tmp1 * (1.0 - r_uv.x) + tmp2 * r_uv.x;
        let denom = (t.x * s.y - t.y * s.x) as f64;
        if denom < -f64::EPSILON || denom > f64::EPSILON {
            let inv_denom = 1.0 / denom;
            r_deriv[0][0] = (-t.y as f64 * inv_denom) as f32;
            r_deriv[0][1] = (t.x as f64 * inv_denom) as f32;
            r_deriv[1][0] = (s.y as f64 * inv_denom) as f32;
            r_deriv[1][1] = (-s.x as f64 * inv_denom) as f32;
        }
    }
    (r_uv, r_deriv)
}

/// `resolve_quad_uv_v2`.
pub fn resolve_quad_uv_v2(st: Vec2, st0: Vec2, st1: Vec2, st2: Vec2, st3: Vec2) -> Vec2 {
    resolve_quad_uv_v2_deriv(st, st0, st1, st2, st3).0
}

/// `resolve_quad_u_v2` — only the u coordinate.
pub fn resolve_quad_u_v2(st: Vec2, st0: Vec2, st1: Vec2, st2: Vec2, st3: Vec2) -> f32 {
    let (signed_area, a, b, fc) = quad_uv_coeffs(st, st0, st1, st2, st3);
    let denom = a - 2.0 * b + fc;
    if denom > -f64::EPSILON && denom < f64::EPSILON {
        let f_den = a - fc;
        if f_den < -f64::EPSILON || f_den > f64::EPSILON {
            return (a / f_den) as f32;
        }
        return 0.0;
    }
    let desc_sq = b * b - a * fc;
    let desc = desc_sq.max(0.0).sqrt();
    let s = if signed_area > 0.0 { -1.0 } else { 1.0 };
    (((a - b) + s * desc) / denom) as f32
}

/// `interp_bilinear_quad_v3` — bilinear interpolation of four `Vec3` samples.
pub fn interp_bilinear_quad_v3(data: &[Vec3; 4], u: f32, v: f32) -> Vec3 {
    data[0] * ((1.0 - u) * (1.0 - v))
        + data[1] * (u * (1.0 - v))
        + data[2] * (u * v)
        + data[3] * ((1.0 - u) * v)
}

/// `interp_barycentric_tri_v3`.
pub fn interp_barycentric_tri(data: &[Vec3; 3], u: f32, v: f32) -> Vec3 {
    data[0] * u + data[1] * v + data[2] * (1.0 - u - v)
}

/* -------------------------------------------------------------------- */
/* View & Projection                                                     */
/* -------------------------------------------------------------------- */

/// `window_translate_m4` — shift a window matrix by `(x, y)`.
pub fn window_translate_m4(winmat: &mut Mat4, perspmat: &Mat4, x: f32, y: f32) {
    if winmat[2][3] == -1.0 {
        // win-matrix is always perspective here
        let v1 = Vec3::new(perspmat[0][0], perspmat[1][0], perspmat[2][0]);
        let v2 = Vec3::new(perspmat[0][1], perspmat[1][1], perspmat[2][1]);
        let len1 = 1.0 / v1.length();
        let len2 = 1.0 / v2.length();
        winmat[2][0] -= len1 * winmat[0][0] * x;
        winmat[2][1] -= len2 * winmat[1][1] * y;
    } else {
        winmat[3][0] += x;
        winmat[3][1] += y;
    }
}

/// `planes_from_projmat` — extract frustum planes; each `Option` slot is
/// filled when `Some`. Order: left, right, bottom, top, near, far.
pub fn planes_from_projmat(mat: &Mat4) -> [Vec4; 6] {
    let col3 = mat[3];
    let mut planes = [Vec4::ZERO; 6];
    for i in 0..4 {
        planes[0][i] = col3[i] + mat[i][0]; // left
        planes[1][i] = col3[i] - mat[i][0]; // right
        planes[2][i] = col3[i] + mat[i][1]; // bottom
        planes[3][i] = col3[i] - mat[i][1]; // top
        planes[4][i] = col3[i] + mat[i][2]; // near
        planes[5][i] = col3[i] - mat[i][2]; // far
    }
    planes
}

/// `projmat_dimensions` — `(left, right, bottom, top, near, far)`.
pub fn projmat_dimensions(winmat: &Mat4) -> [f32; 6] {
    if winmat[3][3] == 0.0 {
        // perspective
        let near = winmat[3][2] / (winmat[2][2] - 1.0);
        let far = if winmat[2][2] == -1.0 {
            near + 1e9 // infinite projection matrix
        } else {
            winmat[3][2] / (winmat[2][2] + 1.0)
        };
        [
            near * ((winmat[2][0] - 1.0) / winmat[0][0]),
            near * ((winmat[2][0] + 1.0) / winmat[0][0]),
            near * ((winmat[2][1] - 1.0) / winmat[1][1]),
            near * ((winmat[2][1] + 1.0) / winmat[1][1]),
            near,
            far,
        ]
    } else {
        [
            (-winmat[3][0] - 1.0) / winmat[0][0],
            (-winmat[3][0] + 1.0) / winmat[0][0],
            (-winmat[3][1] - 1.0) / winmat[1][1],
            (-winmat[3][1] + 1.0) / winmat[1][1],
            (winmat[3][2] + 1.0) / winmat[2][2],
            (winmat[3][2] - 1.0) / winmat[2][2],
        ]
    }
}

/// `projmat_dimensions_db`.
pub fn projmat_dimensions_db(winmat_fl: &Mat4) -> [f64; 6] {
    let w = |i: usize, j: usize| winmat_fl[i][j] as f64;
    if w(3, 3) == 0.0 {
        let near = w(3, 2) / (w(2, 2) - 1.0);
        [
            near * ((w(2, 0) - 1.0) / w(0, 0)),
            near * ((w(2, 0) + 1.0) / w(0, 0)),
            near * ((w(2, 1) - 1.0) / w(1, 1)),
            near * ((w(2, 1) + 1.0) / w(1, 1)),
            near,
            w(3, 2) / (w(2, 2) + 1.0),
        ]
    } else {
        [
            (-w(3, 0) - 1.0) / w(0, 0),
            (-w(3, 0) + 1.0) / w(0, 0),
            (-w(3, 1) - 1.0) / w(1, 1),
            (-w(3, 1) + 1.0) / w(1, 1),
            (w(3, 2) + 1.0) / w(2, 2),
            (w(3, 2) - 1.0) / w(2, 2),
        ]
    }
}

/// `projmat_from_subregion`.
pub fn projmat_from_subregion(
    projmat: &Mat4,
    win_size: [i32; 2],
    x_min: i32,
    x_max: i32,
    y_min: i32,
    y_max: i32,
) -> Mat4 {
    let rect_width = (x_max - x_min) as f32;
    let rect_height = (y_max - y_min) as f32;
    let x_sca = win_size[0] as f32 / rect_width;
    let y_sca = win_size[1] as f32 / rect_height;
    let x_fac = ((x_min + x_max) - win_size[0]) as f32 / rect_width;
    let y_fac = ((y_min + y_max) - win_size[1]) as f32 / rect_height;
    let mut r = *projmat;
    r[0][0] *= x_sca;
    r[1][1] *= y_sca;
    if projmat[3][3] == 0.0 {
        r[2][0] = r[2][0] * x_sca + x_fac;
        r[2][1] = r[2][1] * y_sca + y_fac;
    } else {
        r[3][0] = r[3][0] * x_sca - x_fac;
        r[3][1] = r[3][1] * y_sca - y_fac;
    }
    r
}

fn axis_angle_to_mat4(axis: usize, angle: f32) -> Mat4 {
    let (s, c) = angle.sin_cos();
    let mut m = Mat4::IDENTITY;
    match axis {
        0 => {
            m[1][1] = c;
            m[1][2] = s;
            m[2][1] = -s;
            m[2][2] = c;
        }
        1 => {
            m[0][0] = c;
            m[0][2] = -s;
            m[2][0] = s;
            m[2][2] = c;
        }
        _ => {
            m[0][0] = c;
            m[0][1] = s;
            m[1][0] = -s;
            m[1][1] = c;
        }
    }
    m
}

fn translate_m4_in_place(mat: &mut Mat4, x: f32, y: f32, z: f32) {
    mat[3][0] += mat[0][0] * x + mat[1][0] * y + mat[2][0] * z;
    mat[3][1] += mat[0][1] * x + mat[1][1] * y + mat[2][1] * z;
    mat[3][2] += mat[0][2] * x + mat[1][2] * y + mat[2][2] * z;
    mat[3][3] += mat[0][3] * x + mat[1][3] * y + mat[2][3] * z;
}

fn rotate_m4_in_place(mat: &mut Mat4, axis: usize, angle: f32) {
    let rot = axis_angle_to_mat4(axis, angle);
    *mat = *mat * rot;
}

/// `polarview_m4` — orbit camera matrix (dist, azimuth, incidence, twist).
pub fn polarview_m4(dist: f32, azimuth: f32, incidence: f32, twist: f32) -> Mat4 {
    let mut mat = Mat4::IDENTITY;
    translate_m4_in_place(&mut mat, 0.0, 0.0, -dist);
    rotate_m4_in_place(&mut mat, 2, -twist);
    rotate_m4_in_place(&mut mat, 0, -incidence);
    rotate_m4_in_place(&mut mat, 2, -azimuth);
    mat
}

fn i_multmatrix(icand: &Mat4, mat: &mut Mat4) {
    let mut temp = Mat4::IDENTITY;
    for row in 0..4 {
        for col in 0..4 {
            temp[row][col] = icand[row][0] * mat[0][col]
                + icand[row][1] * mat[1][col]
                + icand[row][2] * mat[2][col]
                + icand[row][3] * mat[3][col];
        }
    }
    *mat = temp;
}

/// `lookat_m4` — view matrix looking from `v` to `p` with `twist`.
pub fn lookat_m4(v: Vec3, p: Vec3, twist: f32) -> Mat4 {
    let mut mat1 = Mat4::IDENTITY;
    let mut mat = axis_angle_to_mat4(2, -twist);

    let dx = p.x - v.x;
    let dy = p.y - v.y;
    let dz = p.z - v.z;
    let mut hyp = dx * dx + dz * dz;
    let hyp1 = (dy * dy + hyp).sqrt();
    hyp = hyp.sqrt();

    let (sine, cosine) = if hyp1 != 0.0 {
        (-dy / hyp1, hyp / hyp1)
    } else {
        (0.0, 1.0)
    };
    mat1[1][1] = cosine;
    mat1[1][2] = sine;
    mat1[2][1] = -sine;
    mat1[2][2] = cosine;
    i_multmatrix(&mat1, &mut mat);

    mat1[1][1] = 1.0;
    mat1[2][2] = 1.0;
    mat1[1][2] = 0.0;
    mat1[2][1] = 0.0;

    let (sine, cosine) = if hyp != 0.0 {
        (dx / hyp, -dz / hyp)
    } else {
        (0.0, 1.0)
    };
    mat1[0][0] = cosine;
    mat1[0][2] = -sine;
    mat1[2][0] = sine;
    mat1[2][2] = cosine;
    i_multmatrix(&mat1, &mut mat);
    translate_m4_in_place(&mut mat, -v.x, -v.y, -v.z);
    mat
}

/// `box_clip_bounds_m4` — clip boundbox against `bounds` (or unit cube when
/// `bounds` is `None`); returns the shared out-flags (0 = visible).
pub fn box_clip_bounds_m4(
    boundbox: &[Vec3; 2],
    bounds: Option<Vec4>,
    winmat: &Mat4,
) -> i32 {
    let mut flag = -1i32;
    for a in 0..8 {
        let vec = Vec4::new(
            if a & 1 != 0 { boundbox[0].x } else { boundbox[1].x },
            if a & 2 != 0 { boundbox[0].y } else { boundbox[1].y },
            if a & 4 != 0 { boundbox[0].z } else { boundbox[1].z },
            1.0,
        );
        let vec = *winmat * vec;
        let mut fl = 0;
        match bounds {
            Some(b) => {
                if vec.x > b.y * vec.w {
                    fl |= 1;
                }
                if vec.x < b.x * vec.w {
                    fl |= 2;
                }
                if vec.y > b.w * vec.w {
                    fl |= 4;
                }
                if vec.y < b.z * vec.w {
                    fl |= 8;
                }
            }
            None => {
                if vec.x < -vec.w {
                    fl |= 1;
                }
                if vec.x > vec.w {
                    fl |= 2;
                }
                if vec.y < -vec.w {
                    fl |= 4;
                }
                if vec.y > vec.w {
                    fl |= 8;
                }
            }
        }
        if vec.z < -vec.w {
            fl |= 16;
        }
        if vec.z > vec.w {
            fl |= 32;
        }
        flag &= fl;
        if flag == 0 {
            return 0;
        }
    }
    flag
}

/// `box_minmax_bounds_m4` — expand `min`/`max` by the 8 boundbox corners
/// transformed by `mat`.
pub fn box_minmax_bounds_m4(
    min: &mut Vec3,
    max: &mut Vec3,
    boundbox: &[Vec3; 2],
    mat: &Mat4,
) {
    let mut mn = *min;
    let mut mx = *max;
    for a in 0..8 {
        let v = Vec3::new(
            if a & 1 != 0 { boundbox[0].x } else { boundbox[1].x },
            if a & 2 != 0 { boundbox[0].y } else { boundbox[1].y },
            if a & 4 != 0 { boundbox[0].z } else { boundbox[1].z },
        );
        let vec = *mat * v;
        for i in 0..3 {
            mn[i] = mn[i].min(vec[i]);
            mx[i] = mx[i].max(vec[i]);
        }
    }
    *min = mn;
    *max = mx;
}

/* -------------------------------------------------------------------- */
/* Mapping                                                               */
/* -------------------------------------------------------------------- */

/// `snap_coordinate` — snap `u` inside [0,1] with texel epsilon.
fn snap_coordinate(mut u: f32) -> f32 {
    if u < 0.0 {
        u += 1.0;
    }
    const EPSILON: f32 = 0.25 / 65536.0;
    if u < EPSILON {
        return 0.0;
    }
    if 1.0 - EPSILON < u {
        return 1.0;
    }
    u
}

/// `map_to_tube` — `(u, v, regular)`.
pub fn map_to_tube(x: f32, y: f32, z: f32) -> (f32, f32, bool) {
    let regular = x * x + y * y >= 1e-6 * 1e-6;
    let u = if regular {
        snap_coordinate(x.atan2(-y) / (2.0 * std::f32::consts::PI))
    } else {
        0.5
    };
    (u, (z + 1.0) / 2.0, regular)
}

/// `angle_normalized_v3v3` — angle between two unit vectors (accurate form).
pub fn angle_normalized_v3v3(v1: Vec3, v2: Vec3) -> f32 {
    if v1.dot(v2) >= 0.0 {
        2.0 * ((v1 - v2).length() / 2.0).clamp(-1.0, 1.0).asin()
    } else {
        std::f32::consts::PI
            - 2.0 * ((v1 + v2).length() / 2.0).clamp(-1.0, 1.0).asin()
    }
}

/// `rotate_normalized_v3_v3v3fl` — Rodrigues rotation of `p` around unit `axis`.
pub fn rotate_normalized(p: Vec3, axis: Vec3, angle: f32) -> Vec3 {
    let costheta = angle.cos();
    let sintheta = angle.sin();
    Vec3::new(
        ((costheta + (1.0 - costheta) * axis.x * axis.x) * p.x)
            + (((1.0 - costheta) * axis.x * axis.y - axis.z * sintheta) * p.y)
            + (((1.0 - costheta) * axis.x * axis.z + axis.y * sintheta) * p.z),
        (((1.0 - costheta) * axis.x * axis.y + axis.z * sintheta) * p.x)
            + ((costheta + (1.0 - costheta) * axis.y * axis.y) * p.y)
            + (((1.0 - costheta) * axis.y * axis.z - axis.x * sintheta) * p.z),
        (((1.0 - costheta) * axis.x * axis.z - axis.y * sintheta) * p.x)
            + (((1.0 - costheta) * axis.y * axis.z + axis.x * sintheta) * p.y)
            + ((costheta + (1.0 - costheta) * axis.z * axis.z) * p.z),
    )
}

/// `map_to_plane_axis_angle_v2_v3v3fl` — rotate `co` by `angle` around `axis`, keep xy.
pub fn map_to_plane_axis_angle_v2(co: Vec3, axis: Vec3, angle: f32) -> Vec2 {
    rotate_normalized(co, axis, angle).xy()
}

/// `map_to_plane_v2_v3v3` — project `co` into the plane with normal `no` (unit).
pub fn map_to_plane_v2(co: Vec3, no: Vec3) -> Vec2 {
    let target = Vec3::new(0.0, 0.0, 1.0);
    let mut axis = no.cross(target);
    axis.normalize();
    map_to_plane_axis_angle_v2(co, axis, angle_normalized_v3v3(no, target))
}

/* -------------------------------------------------------------------- */
/* Normals                                                               */
/* -------------------------------------------------------------------- */

/// `math::safe_acos_approx` — fast acos, max error 4.5e-5.
pub fn safe_acos_approx(x: f32) -> f32 {
    let f = x.abs();
    let m = if f < 1.0 { 1.0 - (1.0 - f) } else { 1.0 };
    let a = (1.0 - m).sqrt()
        * (1.5707963267 + m * (-0.213300989 + m * (0.077980478 + m * -0.02164095)));
    if x < 0.0 {
        std::f32::consts::PI - a
    } else {
        a
    }
}

/// `accumulate_vertex_normals_tri_v3` — add angle-weighted `f_no` to n1..n3.
pub fn accumulate_vertex_normals_tri(
    n: [&mut Vec3; 3],
    f_no: Vec3,
    co1: Vec3,
    co2: Vec3,
    co3: Vec3,
) {
    let mut vdiffs = [co2 - co1, co3 - co2, co1 - co3];
    for d in vdiffs.iter_mut() {
        d.normalize();
    }
    let mut prev_edge = vdiffs[2];
    for (i, vn) in n.into_iter().enumerate() {
        let cur_edge = vdiffs[i];
        let fac = safe_acos_approx(-cur_edge.dot(prev_edge));
        *vn += f_no * fac;
        prev_edge = cur_edge;
    }
}

/// `accumulate_vertex_normals_v3` — tri or quad variant.
pub fn accumulate_vertex_normals(
    n: [&mut Vec3; 4],
    f_no: Vec3,
    co1: Vec3,
    co2: Vec3,
    co3: Vec3,
    co4: Option<Vec3>,
) {
    let nverts = if co4.is_some() { 4 } else { 3 };
    let mut vdiffs = [Vec3::ZERO; 4];
    vdiffs[0] = co2 - co1;
    vdiffs[1] = co3 - co2;
    if nverts == 3 {
        vdiffs[2] = co1 - co3;
    } else {
        vdiffs[2] = co4.unwrap() - co3;
        vdiffs[3] = co1 - co4.unwrap();
        vdiffs[3].normalize();
    }
    vdiffs[0].normalize();
    vdiffs[1].normalize();
    vdiffs[2].normalize();

    let mut prev_edge = vdiffs[nverts - 1];
    for i in 0..nverts {
        let cur_edge = vdiffs[i];
        let fac = safe_acos_approx(-cur_edge.dot(prev_edge));
        // n[i] may be a null-equivalent only upstream; here all four exist.
        *n[i] += f_no * fac;
        prev_edge = cur_edge;
    }
}

/// `accumulate_vertex_normals_poly_v3` — `vertnos` accumulates per-vertex,
/// `vdiffs` is scratch of the same length.
pub fn accumulate_vertex_normals_poly(
    vertnos: &mut [Vec3],
    polyno: Vec3,
    vertcos: &[Vec3],
    vdiffs: &mut [Vec3],
) {
    let nverts = vertcos.len();
    for i in 0..nverts {
        vdiffs[i] = vertcos[(i + 1) % nverts] - vertcos[i];
        vdiffs[i].normalize();
    }
    let mut prev_edge = vdiffs[nverts - 1];
    for i in 0..nverts {
        let cur_edge = vdiffs[i];
        let fac = safe_acos_approx(-cur_edge.dot(prev_edge));
        vertnos[i] += polyno * fac;
        prev_edge = cur_edge;
    }
}

/* -------------------------------------------------------------------- */
/* Tangents                                                              */
/* -------------------------------------------------------------------- */

/// `tangent_from_uv_v3`.
pub fn tangent_from_uv(uv1: Vec2, uv2: Vec2, uv3: Vec2, co1: Vec3, co2: Vec3, co3: Vec3, n: Vec3) -> Vec3 {
    let s1 = uv2.x - uv1.x;
    let s2 = uv3.x - uv1.x;
    let t1 = uv2.y - uv1.y;
    let t2 = uv3.y - uv1.y;
    let mut det = s1 * t2 - s2 * t1;
    if det == 0.0 {
        return Vec3::ZERO;
    }
    det = 1.0 / det;
    let e1 = co1 - co2;
    let e2 = co1 - co3;
    let mut r_tang = Vec3::new(
        (t2 * e1.x - t1 * e2.x) * det,
        (t2 * e1.y - t1 * e2.y) * det,
        (t2 * e1.z - t1 * e2.z) * det,
    );
    let tangv = Vec3::new(
        (s1 * e2.x - s2 * e1.x) * det,
        (s1 * e2.y - s2 * e1.y) * det,
        (s1 * e2.z - s2 * e1.z) * det,
    );
    let ct = r_tang.cross(tangv);
    if ct.dot(n) < 0.0 {
        r_tang = -r_tang;
    }
    r_tang
}

/* -------------------------------------------------------------------- */
/* Vector clouds                                                         */
/* -------------------------------------------------------------------- */

/// `vcloud_estimate_transform_v3` — estimate transform between two point
/// clouds. Returns `(lloc, rloc, lrot, lscale)`.
pub fn vcloud_estimate_transform(
    pos: &[Vec3],
    weight: Option<&[f32]>,
    rpos: &[Vec3],
    rweight: Option<&[f32]>,
) -> (Vec3, Vec3, Mat3, Mat3) {
    let list_size = pos.len();
    let mut accu_com = Vec3::ZERO;
    let mut accu_rcom = Vec3::ZERO;
    let mut accu_weight = 0.0;
    let mut accu_rweight = 0.0;
    let eps = 1e-6f32;

    for a in 0..list_size {
        match weight {
            Some(w) => {
                accu_com += pos[a] * w[a];
                accu_weight += w[a];
            }
            None => accu_com += pos[a],
        }
        match rweight {
            Some(w) => {
                accu_rcom += rpos[a] * w[a];
                accu_rweight += w[a];
            }
            None => accu_rcom += rpos[a],
        }
    }
    if weight.is_none() || rweight.is_none() {
        accu_weight = list_size as f32;
        accu_rweight = list_size as f32;
    }
    accu_com *= 1.0 / accu_weight;
    accu_rcom *= 1.0 / accu_rweight;

    let mut lrot = Mat3::IDENTITY;
    let mut lscale = Mat3::IDENTITY;

    // polar-decompose rotation from scale (Higham's thesis)
    let mut m = Mat3::ZERO;
    let mut mr = Mat3::ZERO;
    for a in 0..list_size {
        let va = rpos[a] - accu_rcom;
        let vb = pos[a] - accu_com;
        for i in 0..3 {
            for j in 0..3 {
                m[i][j] += va[i] * vb[j];
                mr[i][j] += va[i] * va[j];
            }
        }
    }
    let mut q = m;
    let stunt = Vec3::new(q[0][0], q[1][1], q[2][2]);
    q = q * (1.0 / stunt.length());

    let mut odet = 0.0;
    let mut ndet = q.determinant();
    let mut i = 0;
    while (odet - ndet) * (odet - ndet) > eps && i < 15 {
        let mut qi = q.inverted().unwrap_or(q);
        qi = qi.transposed();
        q = (q + qi) * 0.5;
        odet = ndet;
        ndet = q.determinant();
        i += 1;
    }

    if i > 0 {
        lrot = q;
        let irot = q.inverted().unwrap_or(Mat3::IDENTITY);
        let qi = mr.inverted().unwrap_or(Mat3::IDENTITY);
        let q2 = m * qi;
        let scale = irot * q2;
        lscale = scale;
    }

    (accu_com, accu_rcom, lrot, lscale)
}

/// `is_edge_convex_v3`.
pub fn is_edge_convex(v1: Vec3, v2: Vec3, f1_no: Vec3, f2_no: Vec3) -> bool {
    if f1_no != f2_no {
        let cross = f1_no.cross(f2_no);
        let l_dir = v2 - v1;
        return l_dir.dot(cross) > 0.0;
    }
    false
}

/// `project_plane_v3_v3v3` — project `p` onto the plane through origin with
/// (unnormalized) normal `v_plane`.
pub fn project_plane(p: Vec3, v_plane: Vec3) -> Vec3 {
    p - v_plane * (p.dot(v_plane) / v_plane.length_squared())
}

/// `is_quad_convex_v3`.
pub fn is_quad_convex(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> bool {
    let plane = (v1 - v3).cross(v2 - v4);
    let eps_sq = 1e-8f32 * 1e-8;
    if plane.length_squared() < eps_sq {
        return false;
    }
    let coords = [v1, v2, v3, v4];
    let mut proj = [Vec3::ZERO; 4];
    for i in 0..4 {
        proj[i] = project_plane(coords[i], plane);
    }
    let mut dirs = [Vec3::ZERO; 4];
    let mut j = 3;
    for i in 0..4 {
        dirs[i] = proj[i] - proj[j];
        j = i;
    }
    let sign = |a: Vec3, b: Vec3| plane.dot(a.cross(b)) > 0.0;
    sign(dirs[0], dirs[1]) && sign(dirs[1], dirs[2]) && sign(dirs[2], dirs[3]) && sign(dirs[3], dirs[0])
}

/// `is_quad_convex_v2` — diagonals must intersect.
pub fn is_quad_convex_v2(v1: Vec2, v2: Vec2, v3: Vec2, v4: Vec2) -> bool {
    matches!(
        isect_seg_seg_v2(v1, v3, v2, v4),
        Isect::Exact | Isect::Cross
    )
}

/// `is_poly_convex_v2`.
pub fn is_poly_convex_v2(verts: &[Vec2]) -> bool {
    let nr = verts.len();
    let mut sign_flag = 0u32;
    let mut co_prev = verts[nr - 1];
    let mut co_curr = verts[0];
    let mut dir_prev = verts[nr - 2] - co_prev;
    for a in 0..nr {
        let dir_curr = co_prev - co_curr;
        let cross = dir_prev.cross(dir_curr);
        if cross < 0.0 {
            sign_flag |= 1;
        } else if cross > 0.0 {
            sign_flag |= 2;
        }
        if sign_flag == 3 {
            return false;
        }
        dir_prev = dir_curr;
        co_prev = co_curr;
        co_curr = verts[(a + 1) % nr];
    }
    true
}

/// `is_quad_flip_v3` — bitmask: bit0 v1v3 flip, bit1 v2v4 flip.
pub fn is_quad_flip(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> i32 {
    let d_12 = v1 - v2;
    let d_23 = v2 - v3;
    let d_34 = v3 - v4;
    let d_41 = v4 - v1;
    let mut ret = 0;
    let cross_a = d_12.cross(d_23);
    let cross_b = d_34.cross(d_41);
    ret |= ((cross_a.dot(cross_b) < 0.0) as i32) << 0;
    let cross_a = d_23.cross(d_34);
    let cross_b = d_41.cross(d_12);
    ret |= ((cross_a.dot(cross_b) < 0.0) as i32) << 1;
    ret
}

/// `is_quad_flip_v3_first_third_fast`.
pub fn is_quad_flip_first_third_fast(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> bool {
    let d_12 = v2 - v1;
    let d_13 = v3 - v1;
    let d_14 = v4 - v1;
    let cross_a = d_12.cross(d_13);
    let cross_b = d_14.cross(d_13);
    cross_a.dot(cross_b) > 0.0
}

/// `cubic_tangent_factor_circle_v3`.
pub fn cubic_tangent_factor_circle(tan_l: Vec3, tan_r: Vec3) -> f32 {
    const EPS: f32 = 1e-5;
    let tan_dot = tan_l.dot(tan_r);
    if tan_dot > 1.0 - EPS {
        return (1.0 / 3.0) * 0.75;
    }
    if tan_dot < -1.0 + EPS {
        return 1.0 / 2.0;
    }
    let angle = tan_dot.acos() / 2.0;
    let angle_sin = angle.sin();
    let angle_cos = angle.cos();
    ((1.0 - angle_cos) / (angle_sin * 2.0)) / angle_sin
}

/// `geodesic_distance_propagate_across_triangle`.
pub fn geodesic_distance_propagate_across_triangle(
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    dist1: f32,
    dist2: f32,
) -> f32 {
    let v10 = v0 - v1;
    let v12 = v2 - v1;
    if dist1 != 0.0 && dist2 != 0.0 {
        let mut u = v12;
        let d12 = u.normalize_and_get_length();
        if d12 * d12 > 0.0 {
            let mut n = v12.cross(v10);
            n.normalize();
            let v = n.cross(u);
            let v0_ = Vec2::new(v10.dot(u), v10.dot(v).abs());
            let a = 0.5 * (1.0 + (dist1 * dist1 - dist2 * dist2) / (d12 * d12));
            let hh = dist1 * dist1 - a * a * d12 * d12;
            if hh > 0.0 {
                let h = hh.sqrt();
                let s_ = Vec2::new(a * d12, -h);
                let x_intercept = s_.x + h * (v0_.x - s_.x) / (v0_.y + h);
                if (0.0..=d12).contains(&x_intercept) {
                    let dist0 = (s_ - v0_).length();
                    if dist0 >= min_ff(dist1, dist2) {
                        return dist0;
                    }
                }
            }
        }
    }
    min_ff(dist1 + v10.length(), dist2 + (v0 - v2).length())
}

/* ---- math_geom_inline.cc ---- */

/// `axis_dominant_v3` — two axes orthogonal to the dominant one.
pub fn axis_dominant_v3(axis: Vec3) -> (usize, usize) {
    let (xn, yn, zn) = (axis.x.abs(), axis.y.abs(), axis.z.abs());
    if zn >= xn && zn >= yn {
        (0, 1)
    } else if yn >= xn && yn >= zn {
        (0, 2)
    } else {
        (1, 2)
    }
}

/// `axis_dominant_v3_max` — (axis_a, axis_b, dominant |value|).
pub fn axis_dominant_v3_max(axis: Vec3) -> (usize, usize, f32) {
    let (xn, yn, zn) = (axis.x.abs(), axis.y.abs(), axis.z.abs());
    if zn >= xn && zn >= yn {
        (0, 1, zn)
    } else if yn >= xn && yn >= zn {
        (0, 2, yn)
    } else {
        (1, 2, xn)
    }
}

/// `axis_dominant_v3_ortho_single` — index of the SMALLEST |component|.
pub fn axis_dominant_v3_ortho_single(vec: Vec3) -> usize {
    let (x, y, z) = (vec.x.abs(), vec.y.abs(), vec.z.abs());
    if x < y {
        if x < z {
            0
        } else {
            2
        }
    } else if y < z {
        1
    } else {
        2
    }
}

/// `max_axis_v3` — index of largest component (signed).
pub fn max_axis_v3(vec: Vec3) -> usize {
    if vec.x > vec.y {
        if vec.x > vec.z {
            0
        } else {
            2
        }
    } else if vec.y > vec.z {
        1
    } else {
        2
    }
}

/// `min_axis_v3` — index of smallest component (signed).
pub fn min_axis_v3(vec: Vec3) -> usize {
    if vec.x < vec.y {
        if vec.x < vec.z {
            0
        } else {
            2
        }
    } else if vec.y < vec.z {
        1
    } else {
        2
    }
}

const SMALL_NUMBER: f32 = 1e-8;

/// `shell_angle_to_dist`.
pub fn shell_angle_to_dist(angle: f32) -> f32 {
    if angle < SMALL_NUMBER {
        1.0
    } else {
        (1.0 / angle.cos()).abs()
    }
}

/// `shell_v3v3_normalized_to_dist`.
pub fn shell_v3v3_normalized_to_dist(a: Vec3, b: Vec3) -> f32 {
    let angle_cos = a.dot(b).abs();
    if angle_cos < SMALL_NUMBER {
        1.0
    } else {
        1.0 / angle_cos
    }
}

/// `shell_v2v2_normalized_to_dist`.
pub fn shell_v2v2_normalized_to_dist(a: Vec2, b: Vec2) -> f32 {
    let angle_cos = a.dot(b).abs();
    if angle_cos < SMALL_NUMBER {
        1.0
    } else {
        1.0 / angle_cos
    }
}

/// `shell_v3v3_mid_normalized_to_dist` — distance to the normalized midpoint.
pub fn shell_v3v3_mid_normalized_to_dist(a: Vec3, b: Vec3) -> f32 {
    let mut ab = a + b;
    let angle_cos = if ab.normalize_and_get_length() != 0.0 {
        a.dot(ab).abs()
    } else {
        0.0
    };
    if angle_cos < SMALL_NUMBER {
        1.0
    } else {
        1.0 / angle_cos
    }
}

/// `shell_v2v2_mid_normalized_to_dist`.
pub fn shell_v2v2_mid_normalized_to_dist(a: Vec2, b: Vec2) -> f32 {
    let mut ab = a + b;
    let angle_cos = if ab.normalize_and_get_length() != 0.0 {
        a.dot(ab).abs()
    } else {
        0.0
    };
    if angle_cos < SMALL_NUMBER {
        1.0
    } else {
        1.0 / angle_cos
    }
}

/* ---- math_vec.cc ---- */

/// `isect_seg_seg<float2>` — `(kind, lambda, mu)` intersection of segments
/// `v1v2` and `v3v4`.
pub fn isect_seg_seg_v2_lambda_mu(v1: Vec2, v2: Vec2, v3: Vec2, v4: Vec2) -> (Isect, f32, f32) {
    let div = (v2.x - v1.x) * (v4.y - v3.y) - (v2.y - v1.y) * (v4.x - v3.x);
    if div == 0.0 {
        return (Isect::Colinear, 0.0, 0.0);
    }
    let lambda = ((v1.y - v3.y) * (v4.x - v3.x) - (v1.x - v3.x) * (v4.y - v3.y)) / div;
    let mu = ((v1.y - v3.y) * (v2.x - v1.x) - (v1.x - v3.x) * (v2.y - v1.y)) / div;
    let kind = if (0.0..=1.0).contains(&lambda) && (0.0..=1.0).contains(&mu) {
        if lambda == 0.0 || lambda == 1.0 || mu == 0.0 || mu == 1.0 {
            Isect::Exact
        } else {
            Isect::Cross
        }
    } else {
        Isect::None
    };
    (kind, lambda, mu)
}

/// `translate_points` — add `translation` to every point.
pub fn translate_points(points: &mut [Vec3], translation: Vec3) {
    if translation.is_zero() {
        return;
    }
    for p in points.iter_mut() {
        *p += translation;
    }
}

/// `isect_seg_seg_v2_point_ex` — 1 on hit (`r_vi`), -1 otherwise.
/// `endpoint_bias` expands the valid u/v range past the endpoints.
pub fn isect_seg_seg_v2_point_ex(
    v0: Vec2, v1: Vec2, v2: Vec2, v3: Vec2, endpoint_bias: f32, r_vi: &mut Vec2,
) -> i32 {
    let eps = 1e-6f32;
    let endpoint_min = -endpoint_bias;
    let endpoint_max = endpoint_bias + 1.0;
    let mut v0 = v0;
    let mut v1 = v1;
    let mut v2 = v2;
    let mut v3 = v3;
    let mut s10 = v1 - v0;
    let s32 = v3 - v2;
    let mut s30 = v3 - v0;
    let d = s10.x * s32.y - s10.y * s32.x;
    if d != 0.0 {
        let u = (s30.x * s32.y - s30.y * s32.x) / d;
        let v = (s10.x * s30.y - s10.y * s30.x) / d;
        if (endpoint_min..=endpoint_max).contains(&u) && (endpoint_min..=endpoint_max).contains(&v)
        {
            let vi_test = v0 + s10 * u;
            let s_vi_v2 = vi_test - v2;
            let v = s32.dot(s_vi_v2) / s32.dot(s32);
            if (endpoint_min..=endpoint_max).contains(&v) {
                *r_vi = vi_test;
                return 1;
            }
        }
        return -1;
    }
    if (s10.x * s30.y - s10.y * s30.x) == 0.0 && (s32.x * s30.y - s32.y * s30.x) == 0.0 {
        /* equal lines */
        if (v1 - v0).length_squared() <= eps * eps {
            if (v3 - v2).length_squared() > eps * eps {
                std::mem::swap(&mut v0, &mut v2);
                std::mem::swap(&mut v1, &mut v3);
                s10 = v1 - v0;
                s30 = v3 - v0;
            } else {
                if (v2 - v0).length_squared() <= eps * eps {
                    *r_vi = v0;
                    return 1;
                }
                return -1;
            }
        }
        let s20 = v2 - v0;
        let mut u_a = s20.dot(s10) / s10.dot(s10);
        let mut u_b = s30.dot(s10) / s10.dot(s10);
        if u_a > u_b {
            std::mem::swap(&mut u_a, &mut u_b);
        }
        if u_a > endpoint_max || u_b < endpoint_min {
            return -1;
        }
        if u_a.max(0.0) == u_b.min(1.0) {
            *r_vi = v0 + s10 * u_a.max(0.0);
            return 1;
        }
    }
    -1
}

/// `is_quad_flip_v3` — returns bitmask of flipped corner pairs (0 = convex quad).
pub fn is_quad_flip_v3(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> i32 {
    let d_12 = v1 - v2;
    let d_23 = v2 - v3;
    let d_34 = v3 - v4;
    let d_41 = v4 - v1;
    let mut ret = 0;
    let mut cross_a = d_12.cross(d_23);
    let mut cross_b = d_34.cross(d_41);
    ret |= ((cross_a.dot(cross_b) < 0.0) as i32) << 0;
    cross_a = d_23.cross(d_34);
    cross_b = d_41.cross(d_12);
    ret |= ((cross_a.dot(cross_b) < 0.0) as i32) << 1;
    ret
}
