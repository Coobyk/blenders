//! Port of `bmesh_polygon.cc` — face geometry: areas, centers, tangents,
//! point-in-face tests, best-fit normals, triangulation legality checks.
//! UV-area functions are PORT_PENDING (`MLoopUV` CustomData layer).

use crate::structure::{bmesh_disk_edge_exists, bmesh_disk_edge_next};
use crate::{BMesh, BMFace, BM_NIL};
use blenders_math::geom;
use blenders_math::vector::{project_plane_normalized, Vec2, Vec3};

fn angle_signed_v2v2_pos(v1: Vec2, v2: Vec2) -> f32 {
    /* angle in [0, 2π) */
    let perp = Vec2::new(-v1.y, v1.x);
    let a = perp.dot(v2).atan2(v1.dot(v2));
    if a < 0.0 {
        a + std::f32::consts::TAU
    } else {
        a
    }
}

/// `bm_face_calc_poly_normal` — Newell normal.
pub fn bm_face_calc_poly_normal(bm: &BMesh, f: usize) -> Vec3 {
    let mut n = Vec3::ZERO;
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    loop {
        let a = bm.vert(bm.loo(l).v).co;
        let b = bm.vert(bm.loo(bm.loo(l).next).v).co;
        n.x += (a.y - b.y) * (a.z + b.z);
        n.y += (a.z - b.z) * (a.x + b.x);
        n.z += (a.x - b.x) * (a.y + b.y);
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
    n
}

/// `bm_face_calc_poly_normal_vertex_cos` — Newell normal from `vertex_cos`.
pub fn bm_face_calc_poly_normal_vertex_cos(
    bm: &BMesh,
    f: usize,
    vertex_cos: &[Vec3],
) -> Vec3 {
    let mut n = Vec3::ZERO;
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    loop {
        let a = vertex_cos[bm.loo(l).v];
        let b = vertex_cos[bm.loo(bm.loo(l).next).v];
        n.x += (a.y - b.y) * (a.z + b.z);
        n.y += (a.z - b.z) * (a.x + b.x);
        n.z += (a.x - b.x) * (a.y + b.y);
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
    n
}

/// `bm_face_calc_poly_center_median_vertex_cos`.
pub fn bm_face_calc_poly_center_median_vertex_cos(
    bm: &BMesh,
    f: usize,
    vertex_cos: &[Vec3],
) -> Vec3 {
    let mut cent = Vec3::ZERO;
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    loop {
        cent = cent + vertex_cos[bm.loo(l).v];
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
    cent / bm.face(f).len as f32
}

/// `BM_face_calc_point_in_face` — a point guaranteed inside the face
/// (center of the largest tessellation triangle for concave faces).
pub fn bm_face_calc_point_in_face(bm: &mut BMesh, f: usize) -> Vec3 {
    let f_len = bm.face(f).len;
    if f_len == 3 {
        let l = bm.face(f).l_first;
        let (l0, l1, l2) = (l, bm.loo(l).next, bm.loo(l).prev);
        return (bm.vert(bm.loo(l0).v).co
            + bm.vert(bm.loo(l1).v).co
            + bm.vert(bm.loo(l2).v).co)
            / 3.0;
    }
    /* tessellate, pick largest-area tri */
    let mut looptris: Vec<[usize; 3]> = Vec::new();
    crate::tessellate::bmesh_calc_tessellation_for_face_impl(bm, &mut looptris, f, false);
    let mut best = looptris[0];
    let mut best_area = -1.0f32;
    for tri in &looptris {
        let a = bm.vert(bm.loo(tri[0]).v).co;
        let b = bm.vert(bm.loo(tri[1]).v).co;
        let c = bm.vert(bm.loo(tri[2]).v).co;
        let area = geom::area_squared_tri(a, b, c);
        if area > best_area {
            best_area = area;
            best = *tri;
        }
    }
    (bm.vert(bm.loo(best[0]).v).co
        + bm.vert(bm.loo(best[1]).v).co
        + bm.vert(bm.loo(best[2]).v).co)
        / 3.0
}

/// `BM_face_calc_area`.
pub fn bm_face_calc_area(bm: &BMesh, f: usize) -> f32 {
    bm_face_calc_poly_normal(bm, f).length() * 0.5
}

/// `BM_face_calc_area_with_mat3`.
pub fn bm_face_calc_area_with_mat3(bm: &BMesh, f: usize, mat3: &blenders_math::matrix::Mat3) -> f32 {
    /* project verts by mat, measure in 2D of dominant axis */
    let mut n = Vec3::ZERO;
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    loop {
        let a = *mat3 * bm.vert(bm.loo(l).v).co;
        let b = *mat3 * bm.vert(bm.loo(bm.loo(l).next).v).co;
        n.x += (a.y - b.y) * (a.z + b.z);
        n.y += (a.z - b.z) * (a.x + b.x);
        n.z += (a.x - b.x) * (a.y + b.y);
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
    n.length() * 0.5
}

/// `BM_face_calc_perimeter`.
pub fn bm_face_calc_perimeter(bm: &BMesh, f: usize) -> f32 {
    let mut perimeter = 0.0;
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    loop {
        let a = bm.vert(bm.loo(l).v).co;
        let b = bm.vert(bm.loo(bm.loo(l).next).v).co;
        perimeter += (a - b).length();
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
    perimeter
}

/// `BM_face_calc_perimeter_with_mat3`.
pub fn bm_face_calc_perimeter_with_mat3(
    bm: &BMesh,
    f: usize,
    mat3: &blenders_math::matrix::Mat3,
) -> f32 {
    let mut perimeter = 0.0;
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    loop {
        let a = *mat3 * bm.vert(bm.loo(l).v).co;
        let b = *mat3 * bm.vert(bm.loo(bm.loo(l).next).v).co;
        perimeter += (a - b).length();
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
    perimeter
}

/// `bm_vert_tri_find_unique_edge` — index of the vert shared by
/// exactly one unique edge of the tri (0..2, or -1).
fn bm_vert_tri_find_unique_edge(bm: &BMesh, verts: [usize; 3]) -> i32 {
    for i in 0..3 {
        let e_test = bmesh_disk_edge_exists(bm, verts[i], verts[(i + 1) % 3]);
        let e_other = bmesh_disk_edge_exists(bm, verts[i], verts[(i + 2) % 3]);
        if e_test != BM_NIL && e_other != BM_NIL {
            /* unique corner = vert whose two edges differ from the others */
            return i as i32;
        }
    }
    -1
}

/// `BM_vert_tri_calc_tangent_from_edge` — tangent along the unique edge.
pub fn bm_vert_tri_calc_tangent_from_edge(bm: &BMesh, verts: [usize; 3]) -> Vec3 {
    let i = bm_vert_tri_find_unique_edge(bm, verts);
    debug_assert_ne!(i, -1);
    let i = i as usize;
    (bm.vert(verts[(i + 1) % 3]).co - bm.vert(verts[i]).co).normalized()
}

/// `BM_vert_tri_calc_tangent_pair_from_edge`.
pub fn bm_vert_tri_calc_tangent_pair_from_edge(
    bm: &BMesh,
    verts: [usize; 3],
) -> (Vec3, Vec3) {
    let i = bm_vert_tri_find_unique_edge(bm, verts);
    debug_assert_ne!(i, -1);
    let i = i as usize;
    let ta = (bm.vert(verts[(i + 1) % 3]).co - bm.vert(verts[i]).co).normalized();
    let tb = (bm.vert(verts[i]).co - bm.vert(verts[(i + 2) % 3]).co).normalized();
    (ta, tb)
}

/// `BM_face_calc_tangent_from_edge` — quad tangent from unique edges.
pub fn bm_face_calc_tangent_from_edge(bm: &BMesh, f: usize) -> Vec3 {
    debug_assert!(bm.face(f).len == 3 || bm.face(f).len == 4);
    if bm.face(f).len == 4 {
        /* use the two unique edges (a->b and c->d sides) */
        let l_first = bm.face(f).l_first;
        let mut l = l_first;
        let mut dir = Vec3::ZERO;
        loop {
            let e = bm.loo(l).e;
            let mut unique = true;
            let l0 = bm.edge(e).l;
            if l0 != BM_NIL {
                let mut li = l0;
                loop {
                    if bm.loo(li).f != f {
                        unique = false;
                    }
                    li = bm.loo(li).radial_next;
                    if li == l0 {
                        break;
                    }
                }
            }
            if unique {
                let a = bm.vert(bm.loo(l).v).co;
                let b = bm.vert(bm.loo(bm.loo(l).next).v).co;
                dir = dir + (b - a);
            }
            l = bm.loo(l).next;
            if l == l_first {
                break;
            }
        }
        let mut t = dir.normalized();
        /* ensure parity with quad ordering: use diagonal of opposite side */
        if t.length_squared() == 0.0 {
            t = Vec3::new(0.0, 0.0, 1.0);
        }
        t
    } else {
        let l0 = bm.face(f).l_first;
        let l1 = bm.loo(l0).next;
        let l2 = bm.loo(l1).next;
        bm_vert_tri_calc_tangent_from_edge(
            bm,
            [bm.loo(l0).v, bm.loo(l1).v, bm.loo(l2).v],
        )
    }
}

/// `BM_face_calc_tangent_pair_from_edge` for a quad: tangents of the
/// two unique edges (only meaningful for a quad face).
pub fn bm_face_calc_tangent_pair_from_edge(bm: &BMesh, f: usize) -> (Vec3, Vec3) {
    debug_assert_eq!(bm.face(f).len, 4);
    let mut t_a = Vec3::ZERO;
    let mut t_b = Vec3::ZERO;
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    let mut i = 0;
    loop {
        let e = bm.loo(l).e;
        let l0 = bm.edge(e).l;
        let mut unique = true;
        if l0 != BM_NIL {
            let mut li = l0;
            loop {
                if bm.loo(li).f != f {
                    unique = false;
                }
                li = bm.loo(li).radial_next;
                if li == l0 {
                    break;
                }
            }
        }
        if unique {
            let a = bm.vert(bm.loo(l).v).co;
            let b = bm.vert(bm.loo(bm.loo(l).next).v).co;
            if i == 0 {
                t_a = (b - a).normalized();
            } else {
                t_b = (b - a).normalized();
            }
            i += 1;
        }
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
    (t_a, t_b)
}

/// `BM_face_calc_tangent_from_edge_diagonal` — quad diagonal as tangent.
pub fn bm_face_calc_tangent_from_edge_diagonal(bm: &BMesh, f: usize) -> Vec3 {
    let l0 = bm.face(f).l_first;
    let l2 = bm.loo(bm.loo(l0).next).next;
    (bm.vert(bm.loo(l2).v).co - bm.vert(bm.loo(l0).v).co).normalized()
}

/// `BM_face_calc_tangent_from_vert_diagonal` — longest diagonal.
pub fn bm_face_calc_tangent_from_vert_diagonal(bm: &BMesh, f: usize) -> Vec3 {
    let mut best = Vec3::ZERO;
    let mut best_len = 0.0f32;
    let verts: Vec<usize> = {
        let mut out = Vec::new();
        let l_first = bm.face(f).l_first;
        let mut l = l_first;
        loop {
            out.push(bm.loo(l).v);
            l = bm.loo(l).next;
            if l == l_first {
                break;
            }
        }
        out
    };
    for i in 0..verts.len() {
        for j in (i + 2)..verts.len() {
            let d = bm.vert(verts[j]).co - bm.vert(verts[i]).co;
            let l = d.length_squared();
            if l > best_len {
                best_len = l;
                best = d;
            }
        }
    }
    best.normalized()
}

/// `BM_face_calc_tangent_auto` — from edge if quad w/ unique edges,
/// else from vert diagonal.
pub fn bm_face_calc_tangent_auto(bm: &BMesh, f: usize) -> Vec3 {
    if bm.face(f).len == 4 {
        bm_face_calc_tangent_from_edge(bm, f)
    } else {
        bm_face_calc_tangent_from_vert_diagonal(bm, f)
    }
}

/// `BM_face_calc_bounds_expand` — grow min/max by face verts.
pub fn bm_face_calc_bounds_expand(bm: &BMesh, f: usize, min: &mut Vec3, max: &mut Vec3) {
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    loop {
        let co = bm.vert(bm.loo(l).v).co;
        *min = Vec3::new(min.x.min(co.x), min.y.min(co.y), min.z.min(co.z));
        *max = Vec3::new(max.x.max(co.x), max.y.max(co.y), max.z.max(co.z));
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
}

/// `BM_face_calc_center_bounds` — bounds center.
pub fn bm_face_calc_center_bounds(bm: &BMesh, f: usize) -> Vec3 {
    let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
    bm_face_calc_bounds_expand(bm, f, &mut min, &mut max);
    (min + max) * 0.5
}

/// `BM_face_calc_center_bounds_vcos`.
pub fn bm_face_calc_center_bounds_vcos(bm: &BMesh, f: usize, vcos: &[Vec3]) -> Vec3 {
    let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    loop {
        let co = vcos[bm.loo(l).v];
        min = Vec3::new(min.x.min(co.x), min.y.min(co.y), min.z.min(co.z));
        max = Vec3::new(max.x.max(co.x), max.y.max(co.y), max.z.max(co.z));
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
    (min + max) * 0.5
}

/// `BM_face_calc_center_median_vcos` — median center over explicit coords.
pub fn bm_face_calc_center_median_vcos(bm: &BMesh, f: usize, vertex_cos: &[Vec3]) -> Vec3 {
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    let mut cent = Vec3::ZERO;
    let mut n = 0i32;
    loop {
        cent = cent + vertex_cos[bm.loo(l).v];
        n += 1;
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
    if n > 0 {
        cent / n as f32
    } else {
        cent
    }
}

/// `BM_face_calc_center_median_weighted` — center weighted by edge length.
pub fn bm_face_calc_center_median_weighted(bm: &BMesh, f: usize) -> Vec3 {
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    let mut co = Vec3::ZERO;
    let mut totw = 0.0f32;
    loop {
        let w = (bm.vert(bm.loo(l).v).co - bm.vert(bm.loo(bm.loo(l).next).v).co).length();
        co = co + bm.vert(bm.loo(l).v).co * w;
        totw += w;
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
    if totw != 0.0 {
        co * (1.0 / totw)
    } else {
        co
    }
}

/// `poly_rotate_plane` — rotate verts so `normal` becomes +Z.
pub fn poly_rotate_plane(normal: Vec3, verts: &mut [Vec3]) {
    let mat = geom::axis_dominant_to_m3(normal);
    for v in verts.iter_mut() {
        *v = mat * *v;
    }
}

/// `BM_face_calc_normal_vcos`.
pub fn bm_face_calc_normal_vcos(bm: &BMesh, f: usize, vertex_cos: &[Vec3]) -> Vec3 {
    match bm.face(f).len {
        4 => {
            let l = bm.face(f).l_first;
            let l1 = bm.loo(l).next;
            let l2 = bm.loo(l1).next;
            let l3 = bm.loo(l2).next;
            geom::normal_quad(
                vertex_cos[bm.loo(l).v],
                vertex_cos[bm.loo(l1).v],
                vertex_cos[bm.loo(l2).v],
                vertex_cos[bm.loo(l3).v],
            )
            .0
        }
        3 => {
            let l = bm.face(f).l_first;
            let l1 = bm.loo(l).next;
            let l2 = bm.loo(l1).next;
            geom::normal_tri(
                vertex_cos[bm.loo(l).v],
                vertex_cos[bm.loo(l1).v],
                vertex_cos[bm.loo(l2).v],
            )
            .0
        }
        _ => {
            let mut n = bm_face_calc_poly_normal_vertex_cos(bm, f, vertex_cos);
            n.normalize();
            n
        }
    }
}

/// `BM_verts_calc_normal_from_cloud_ex` — best-fit normal for a vert cloud.
pub fn bm_verts_calc_normal_from_cloud_ex(
    bm: &BMesh,
    varr: &[usize],
) -> (Vec3, Vec3, usize) {
    let varr_len = varr.len();
    let inv = 1.0 / varr_len as f32;
    let mut center = Vec3::ZERO;
    for &v in varr {
        center = center + bm.vert(v).co * inv;
    }
    /* farthest point from center */
    let mut co_a_index = 0usize;
    let mut co_a = Vec3::ZERO;
    let mut dist_sq_max = -1.0f32;
    for (i, &v) in varr.iter().enumerate() {
        let d = (bm.vert(v).co - center).length_squared();
        if d > dist_sq_max {
            co_a = bm.vert(v).co;
            co_a_index = i;
            dist_sq_max = d;
        }
    }
    let mut dir_a = co_a - center;
    dir_a.normalize();
    /* point farthest perpendicular to dir_a */
    let mut co_b = Vec3::ZERO;
    let mut dir_b = Vec3::ZERO;
    dist_sq_max = -1.0;
    for &v in varr {
        let co = bm.vert(v).co;
        if co == co_a {
            continue;
        }
        let d = co - center;
        let d = project_plane_normalized(d, dir_a);
        let ds = d.length_squared();
        if ds > dist_sq_max {
            co_b = co;
            dir_b = d;
            dist_sq_max = ds;
        }
    }

    let mut r_normal = Vec3::ZERO;
    if varr_len <= 3 {
        r_normal = geom::normal_tri(center, co_a, co_b).0;
    } else {
        dir_b.normalize();
        /* opposite ends along each dir */
        let mut co_a_opp = Vec3::ZERO;
        let mut co_b_opp = Vec3::ZERO;
        let mut dot_a_min = f32::MAX;
        let mut dot_b_min = f32::MAX;
        let mut has_a = false;
        let mut has_b = false;
        for &v in varr {
            let co = bm.vert(v).co;
            if co != co_a {
                let d = dir_a.dot(co);
                if d < dot_a_min || !has_a {
                    dot_a_min = d;
                    co_a_opp = co;
                    has_a = true;
                }
            }
            if co != co_b {
                let d = dir_b.dot(co);
                if d < dot_b_min || !has_b {
                    dot_b_min = d;
                    co_b_opp = co;
                    has_b = true;
                }
            }
        }
        let (qn, qarea) = geom::normal_quad(co_a, co_b, co_a_opp, co_b_opp);
        if qarea != 0.0 {
            /* refine via radial-ordered Newell accumulation */
            let mut order: Vec<usize> = (0..varr_len).collect();
            let mut angles: Vec<f32> = Vec::with_capacity(varr_len);
            for &v in varr {
                let d = bm.vert(v).co - center;
                let angle = blenders_math::vector::angle_signed_on_axis(dir_a, d, qn);
                angles.push(if angle.is_finite() { angle } else { 0.0 });
            }
            order.sort_by(|&a, &b| angles[b].partial_cmp(&angles[a]).unwrap());
            let mut normal_refine = Vec3::ZERO;
            let mut v_prev = bm.vert(varr[order[varr_len - 1]]).co;
            for &i in &order {
                let v_curr = bm.vert(varr[i]).co;
                normal_refine.x += (v_prev.y - v_curr.y) * (v_prev.z + v_curr.z);
                normal_refine.y += (v_prev.z - v_curr.z) * (v_prev.x + v_curr.x);
                normal_refine.z += (v_prev.x - v_curr.x) * (v_prev.y + v_curr.y);
                v_prev = v_curr;
            }
            if normal_refine.length_squared() != 0.0 {
                normal_refine.normalize();
                /* recompute tangent index */
                dist_sq_max = -1.0;
                for (i, &v) in varr.iter().enumerate() {
                    let d = bm.vert(v).co - center;
                    let d = project_plane_normalized(d, normal_refine);
                    let ds = d.length_squared();
                    if ds > dist_sq_max {
                        co_a_index = i;
                        dist_sq_max = ds;
                    }
                }
                r_normal = normal_refine;
            } else {
                r_normal = qn;
            }
        } else {
            r_normal = qn;
        }
    }
    (r_normal, center, co_a_index)
}

/// `BM_face_calc_normal_subset` — partial Newell over l_first..=l_last.
pub fn bm_face_calc_normal_subset(bm: &BMesh, l_first: usize, l_last: usize) -> Vec3 {
    let mut n = Vec3::ZERO;
    let mut l = l_first;
    loop {
        let a = bm.vert(bm.loo(l).v).co;
        let b = bm.vert(bm.loo(bm.loo(l).next).v).co;
        n.x += (a.y - b.y) * (a.z + b.z);
        n.y += (a.z - b.z) * (a.x + b.x);
        n.z += (a.x - b.x) * (a.y + b.y);
        if l == l_last {
            break;
        }
        l = bm.loo(l).next;
    }
    n.normalize();
    n
}

/// `BM_face_normal_flip` (`_ex` without mdisp).
pub fn bm_face_normal_flip(bm: &mut BMesh, f: usize) {
    crate::core::bmesh_kernel_loop_reverse(bm, f);
    bm.fpool[f].no = bm.face(f).no * -1.0;
}

/// `BM_face_point_inside_test`.
pub fn bm_face_point_inside_test(bm: &BMesh, f: usize, co: Vec3) -> bool {
    let axis_mat = geom::axis_dominant_to_m3(bm.face(f).no);
    let co_3 = axis_mat * co;
    let co_2d = Vec2::new(co_3.x, co_3.y);
    let mut projverts: Vec<Vec2> = Vec::with_capacity(bm.face(f).len as usize);
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    loop {
        let p = axis_mat * bm.vert(bm.loo(l).v).co;
        projverts.push(Vec2::new(p.x, p.y));
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
    geom::isect_point_poly_v2(co_2d, &projverts)
}

/// `BM_face_splits_check_legal` — null out splits that would exit the face
/// boundary or cross other splits. `loops` = pairs of loop indices.
pub fn bm_face_splits_check_legal(bm: &mut BMesh, f: usize, loops: &mut [(usize, usize)]) {
    let f_len = bm.face(f).len;
    let axis_mat = geom::axis_dominant_to_m3(bm.face(f).no);
    let mut projverts: Vec<Vec2> = Vec::with_capacity(f_len as usize);
    let mut center = Vec2::ZERO;
    {
        let l_first = bm.face(f).l_first;
        let mut l = l_first;
        loop {
            let p = axis_mat * bm.vert(bm.loo(l).v).co;
            let p2 = Vec2::new(p.x, p.y);
            projverts.push(p2);
            center = center + p2;
            l = bm.loo(l).next;
            if l == l_first {
                break;
            }
        }
    }
    if geom::is_poly_convex_v2(&projverts) {
        return;
    }
    center = center / f_len as f32;
    /* index the loops of the face, center the projection */
    {
        let l_first = bm.face(f).l_first;
        let mut l = l_first;
        let mut i = 0usize;
        loop {
            bm.lpool[l].head.index = i as i32;
            projverts[i] = projverts[i] - center;
            i += 1;
            l = bm.loo(l).next;
            if l == l_first {
                break;
            }
        }
        bm.elem_index_dirty = true;
    }
    let li = |l: usize| bm.loo(l).head.index as usize;
    let edgeverts: Vec<(Vec2, Vec2)> = loops
        .iter()
        .map(|&(a, b)| (projverts[li(a)], projverts[li(b)]))
        .collect();

    /* inside-face test at both ends */
    for i in 0..loops.len() {
        let (la, lb) = loops[i];
        let co_pair = (projverts[li(la)], projverts[li(lb)]);
        if (co_pair.0 - co_pair.1).length_squared() < 1e-12 {
            continue;
        }
        let pair_dir = (co_pair.1 - co_pair.0).normalized();
        for side in 0..2 {
            let l_side = if side == 0 { la } else { lb };
            let co = if side == 0 { co_pair.0 } else { co_pair.1 };
            let limit_init = f_len - 3;
            let mut l_prev = bm.loo(l_side).prev;
            let mut l_next = bm.loo(l_side).next;
            let mut limit = limit_init;
            while (projverts[li(l_prev)] - co).length_squared() < 1e-12 && limit > 0 {
                l_prev = bm.loo(l_prev).prev;
                limit -= 1;
            }
            limit = limit_init;
            while (projverts[li(l_next)] - co).length_squared() < 1e-12 && limit > 0 {
                l_next = bm.loo(l_next).next;
                limit -= 1;
            }
            let co_prev = projverts[li(l_prev)];
            let co_next = projverts[li(l_next)];
            let dir_other = if side == 0 { pair_dir } else { pair_dir * -1.0 };
            let dir_prev = (co_prev - co).normalized();
            let dir_next = (co_next - co).normalized();
            if angle_signed_v2v2_pos(dir_prev, dir_other)
                > angle_signed_v2v2_pos(dir_prev, dir_next)
            {
                loops[i] = (BM_NIL, BM_NIL);
                break;
            }
        }
    }

    let share_vert = |a: &(Vec2, Vec2), b: &(Vec2, Vec2)| {
        (a.0 - b.0).length_squared() < 1e-12
            || (a.0 - b.1).length_squared() < 1e-12
            || (a.1 - b.0).length_squared() < 1e-12
            || (a.1 - b.1).length_squared() < 1e-12
    };

    /* boundary crossing tests */
    let n = projverts.len();
    for i in 0..n {
        let i_prev = if i == 0 { n - 1 } else { i - 1 };
        let f_edge = (projverts[i_prev], projverts[i]);
        for j in 0..loops.len() {
            if loops[j].0 != BM_NIL && !share_vert(&f_edge, &edgeverts[j]) {
                if geom::isect_seg_seg_v2(f_edge.0, f_edge.1, edgeverts[j].0, edgeverts[j].1)
                    == geom::Isect::Cross
                {
                    loops[j] = (BM_NIL, BM_NIL);
                }
            }
        }
    }
    /* self-intersect tests */
    for i in 0..loops.len() {
        if loops[i].0 == BM_NIL {
            continue;
        }
        for j in (i + 1)..loops.len() {
            if loops[j].0 != BM_NIL && !share_vert(&edgeverts[i], &edgeverts[j]) {
                if geom::isect_seg_seg_v2(
                    edgeverts[i].0,
                    edgeverts[i].1,
                    edgeverts[j].0,
                    edgeverts[j].1,
                ) == geom::Isect::Cross
                {
                    loops[i] = (BM_NIL, BM_NIL);
                    break;
                }
            }
        }
    }
}

/// `BM_face_splits_check_optimal` — keep only pairs sharing the face.
pub fn bm_face_splits_check_optimal(bm: &BMesh, f: usize, loops: &mut [(usize, usize)]) {
    for pair in loops.iter_mut() {
        let (va, vb) = (bm.loo(pair.0).v, bm.loo(pair.1).v);
        let f2 = crate::query::bm_vert_pair_share_face_by_angle(bm, va, vb, false);
        if f2 != f {
            *pair = (BM_NIL, BM_NIL);
        }
    }
}

/// `BM_face_as_array_vert_tri`.
pub fn bm_face_as_array_vert_tri(bm: &BMesh, f: usize) -> [usize; 3] {
    debug_assert_eq!(bm.face(f).len, 3);
    let l = bm.face(f).l_first;
    let l1 = bm.loo(l).next;
    let l2 = bm.loo(l1).next;
    [bm.loo(l).v, bm.loo(l1).v, bm.loo(l2).v]
}

/// `BM_face_as_array_vert_quad`.
pub fn bm_face_as_array_vert_quad(bm: &BMesh, f: usize) -> [usize; 4] {
    debug_assert_eq!(bm.face(f).len, 4);
    let l = bm.face(f).l_first;
    let l1 = bm.loo(l).next;
    let l2 = bm.loo(l1).next;
    let l3 = bm.loo(l2).next;
    [
        bm.loo(l).v,
        bm.loo(l1).v,
        bm.loo(l2).v,
        bm.loo(l3).v,
    ]
}

/// `BM_face_as_array_loop_tri`.
pub fn bm_face_as_array_loop_tri(bm: &BMesh, f: usize) -> [usize; 3] {
    debug_assert_eq!(bm.face(f).len, 3);
    let l = bm.face(f).l_first;
    [l, bm.loo(l).next, bm.loo(bm.loo(l).next).next]
}

/// `BM_face_as_array_loop_quad`.
pub fn bm_face_as_array_loop_quad(bm: &BMesh, f: usize) -> [usize; 4] {
    debug_assert_eq!(bm.face(f).len, 4);
    let l = bm.face(f).l_first;
    let l1 = bm.loo(l).next;
    let l2 = bm.loo(l1).next;
    let l3 = bm.loo(l2).next;
    [l, l1, l2, l3]
}

/// keep face type referenced.
const _: Option<fn(&BMFace)> = None;

/* `MOD_TRIANGULATE_QUAD_*` / `MOD_TRIANGULATE_NGON_*` (DNA_modifier_types.h). */
pub const MOD_TRIANGULATE_NGON_BEAUTY: i32 = 0;
pub const MOD_TRIANGULATE_QUAD_FIXED: i32 = 1;
pub const MOD_TRIANGULATE_QUAD_ALTERNATE: i32 = 2;
pub const MOD_TRIANGULATE_QUAD_SHORTEDGE: i32 = 3;
pub const MOD_TRIANGULATE_QUAD_LONGEDGE: i32 = 4;

/// `BM_verts_calc_rotate_beauty` — edge-rotation beauty heuristic on the
/// quad's dominant-axis 2D projection (C computes in 3D with area weights).
pub fn bm_verts_calc_rotate_beauty(
    bm: &BMesh, v1: usize, v2: usize, v3: usize, v4: usize) -> f32
{
    let no = (bm.vert(v2).co - bm.vert(v1).co)
        .cross(bm.vert(v4).co - bm.vert(v3).co);
    let m = blenders_math::geom::axis_dominant_to_m3_negate(no);
    let p = |v: usize| {
        let q = m * bm.vert(v).co;
        [q.x, q.y]
    };
    let mut area = None;
    blenders_bli::polyfill_2d_beautify::polyfill_beautify_quad_rotate_calc_ex(
        p(v1), p(v2), p(v3), p(v4), false, &mut area)
}

/// `BM_face_triangulate` — split `f` into triangles in place; the last
/// triangle replaces `f` via data-swap (callers rely on `f` surviving).
/// Returns `(faces_new, edges_new, faces_double)`.
pub fn bm_face_triangulate(
    bm: &mut BMesh,
    f: usize,
    quad_method: i32,
    ngon_method: i32,
    use_tag: bool,
) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
    let use_beauty = ngon_method == MOD_TRIANGULATE_NGON_BEAUTY;
    let mut faces_new: Vec<usize> = Vec::new();
    let mut edges_new: Vec<usize> = Vec::new();
    let mut faces_double: Vec<usize> = Vec::new();
    debug_assert!(bm.face(f).len > 3);
    let f_len = bm.face(f).len as usize;
    let mut loops: Vec<usize> = Vec::with_capacity(f_len);
    let mut tris: Vec<[u32; 3]> = Vec::with_capacity(f_len - 2);
    let totfilltri = f_len - 2;
    let last_tri = f_len - 3;
    if f_len == 4 {
        let l_first = bm.face(f).l_first;
        let (l_v1, l_v2) = match quad_method {
            MOD_TRIANGULATE_QUAD_FIXED => {
                (l_first, bm.loo(bm.loo(l_first).next).next)
            }
            MOD_TRIANGULATE_QUAD_ALTERNATE => {
                (bm.loo(l_first).next, bm.loo(l_first).prev)
            }
            _ => {
                let mut l_v1 = bm.loo(l_first).next;
                let mut l_v2 = bm.loo(bm.loo(l_first).next).next;
                let l_v3 = bm.loo(l_first).prev;
                let l_v4 = l_first;
                let split_24 = match quad_method {
                    MOD_TRIANGULATE_QUAD_SHORTEDGE => {
                        let d1 = (bm.vert(bm.loo(l_v4).v).co - bm.vert(bm.loo(l_v2).v).co)
                            .length_squared();
                        let d2 = (bm.vert(bm.loo(l_v1).v).co - bm.vert(bm.loo(l_v3).v).co)
                            .length_squared();
                        d2 - d1 > 0.0
                    }
                    MOD_TRIANGULATE_QUAD_LONGEDGE => {
                        let d1 = (bm.vert(bm.loo(l_v4).v).co - bm.vert(bm.loo(l_v2).v).co)
                            .length_squared();
                        let d2 = (bm.vert(bm.loo(l_v1).v).co - bm.vert(bm.loo(l_v3).v).co)
                            .length_squared();
                        d2 - d1 < 0.0
                    }
                    _ => {
                        let flip = blenders_math::geom::is_quad_flip(
                            bm.vert(bm.loo(l_v1).v).co, bm.vert(bm.loo(l_v2).v).co,
                            bm.vert(bm.loo(l_v3).v).co, bm.vert(bm.loo(l_v4).v).co);
                        if flip & 1 != 0 { true }
                        else if flip & 2 != 0 { false }
                        else {
                            bm_verts_calc_rotate_beauty(
                                bm, bm.loo(l_v1).v, bm.loo(l_v2).v,
                                bm.loo(l_v3).v, bm.loo(l_v4).v) > 0.0
                        }
                    }
                };
                if split_24 {
                    l_v1 = l_v4;
                } else {
                    l_v2 = l_v3;
                }
                (l_v1, l_v2)
            }
        };
        loops = vec![l_v1, bm.loo(l_v1).next, l_v2, bm.loo(l_v2).next];
        tris.push([0, 1, 2]);
        tris.push([0, 2, 3]);
    } else {
        let axis_mat = blenders_math::geom::axis_dominant_to_m3_negate(bm.face(f).no);
        let mut projverts: Vec<[f32; 2]> = Vec::with_capacity(f_len);
        let mut l_iter = bm.face(f).l_first;
        for _ in 0..f_len {
            loops.push(l_iter);
            let p = axis_mat * bm.vert(bm.loo(l_iter).v).co;
            projverts.push([p.x, p.y]);
            l_iter = bm.loo(l_iter).next;
        }
        tris = blenders_bli::polyfill_2d::polyfill_calc_pub(&projverts, 1)
            .iter().map(|t| [t[0] as u32, t[1] as u32, t[2] as u32]).collect();
        if use_beauty {
            blenders_bli::polyfill_2d_beautify::polyfill_beautify(&projverts, &mut tris);
        }
    }
    let mut f_new = BM_NIL;
    for i in 0..totfilltri {
        let t = tris[i];
        let ltri = [loops[t[0] as usize], loops[t[1] as usize], loops[t[2] as usize]];
        let v_tri = [bm.loo(ltri[0]).v, bm.loo(ltri[1]).v, bm.loo(ltri[2]).v];
        f_new = crate::core::bm_face_create_verts(bm, &v_tri, Some(f), 0, true, true);
        let l_new = bm.face(f_new).l_first;
        /* duplicate-face check */
        if bm.loo(l_new).radial_next != l_new {
            let mut l_iter = bm.loo(l_new).radial_next;
            loop {
                if bm.face(bm.loo(l_iter).f).len == 3
                    && bm.loo(bm.loo(l_new).prev).v == bm.loo(bm.loo(l_iter).prev).v
                {
                    faces_double.push(if i != last_tri { f_new } else { f });
                    break;
                }
                l_iter = bm.loo(l_iter).radial_next;
                if l_iter == l_new { break; }
            }
        }
        /* CustomData loop-attr copy: PORT_PENDING */
        if i != last_tri {
            if use_tag {
                bm.fpool[f_new].head.hflag |= crate::BM_ELEM_TAG;
            }
            faces_new.push(f_new);
        }
        if use_tag || true {
            let mut l_iter = l_new;
            loop {
                let e = bm.loo(l_iter).e;
                let is_new_edge = l_iter == bm.loo(l_iter).radial_next;
                if is_new_edge {
                    if use_tag {
                        bm.epool[e].head.hflag |= crate::BM_ELEM_TAG;
                    }
                    edges_new.push(e);
                }
                l_iter = bm.loo(l_iter).next;
                if l_iter == l_new { break; }
            }
        }
    }
    crate::core::bmesh_face_swap_data(bm, f, f_new);
    crate::core::bm_face_kill(bm, f_new);
    bm.elem_index_dirty = true;
    (faces_new, edges_new, faces_double)
}

/// `bm_face_subset_calc_planar` (bmo_connect_nonplanar.cc) — summed |Δz|
/// of the chain `l_first..=l_last` against the plane `no`.
pub fn bm_face_subset_calc_planar(
    bm: &BMesh, l_first: usize, l_last: usize, no: Vec3) -> f32
{
    let axis_mat = blenders_math::geom::axis_dominant_to_m3(no);
    let mut delta_z = 0.0;
    let mut l_iter = l_first;
    let l_term = bm.loo(l_last).next;
    let z = |co: Vec3| axis_mat.cols[0].z * co.x + axis_mat.cols[1].z * co.y
        + axis_mat.cols[2].z * co.z;
    let mut z_prev = z(bm.vert(bm.loo(l_last).v).co);
    loop {
        let z_curr = z(bm.vert(bm.loo(l_iter).v).co);
        delta_z += (z_curr - z_prev).abs();
        z_prev = z_curr;
        l_iter = bm.loo(l_iter).next;
        if l_iter == l_term { break; }
    }
    delta_z
}
