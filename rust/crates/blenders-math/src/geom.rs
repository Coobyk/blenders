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

/* -------------------------------------------------------------------- */
/* Ray → AABB distance (`dist_squared_ray_to_aabb_v3*` family)            */
/* -------------------------------------------------------------------- */

use crate::matrix::Mat4;

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
