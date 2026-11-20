//! Port of `bmesh_query_uv.cc` — UV topology queries.
//! CustomData UV layers are PORT_PENDING, so every function takes a
//! `uv_at: impl Fn(&BMesh, usize) -> Vec2` accessor (loop index -> UV).
//! `BM_uv_map_offsets*`/`BMUVOffsets` are omitted until CustomData lands.

use crate::{BMesh, BM_NIL};
use blenders_math::vector::Vec2;

fn face_loops(bm: &BMesh, f: usize) -> Vec<usize> {
    let mut out = Vec::with_capacity(bm.face(f).len as usize);
    let first = bm.face(f).l_first;
    let mut l = first;
    loop {
        out.push(l);
        l = bm.loo(l).next;
        if l == first {
            break;
        }
    }
    out
}

/// `BM_face_uv_calc_center_median_weighted` — edge-length weighted UV centroid.
pub fn bm_face_uv_calc_center_median_weighted(
    bm: &BMesh,
    f: usize,
    aspect: [f32; 2],
    uv_at: impl Fn(&BMesh, usize) -> Vec2,
) -> Vec2 {
    let loops = face_loops(bm, f);
    let n = loops.len();
    let mut cent = Vec2::ZERO;
    let mut totw = 0.0f32;
    let mut uv_prev = uv_at(bm, loops[n - 1]) * Vec2::new(aspect[0], aspect[1]);
    let mut uv_curr = uv_at(bm, loops[0]) * Vec2::new(aspect[0], aspect[1]);
    let mut w_prev = (uv_prev - uv_curr).length();
    for i in 0..n {
        let uv_next = uv_at(bm, loops[(i + 1) % n]) * Vec2::new(aspect[0], aspect[1]);
        let w_curr = (uv_curr - uv_next).length();
        let w = w_curr + w_prev;
        cent += uv_curr * w;
        totw += w;
        w_prev = w_curr;
        uv_curr = uv_next;
        uv_prev = uv_curr;
    }
    let _ = uv_prev;
    if totw != 0.0 {
        cent *= 1.0 / totw;
    }
    Vec2::new(cent.x / aspect[0], cent.y / aspect[1])
}

/// `BM_face_uv_calc_center_median` — unweighted UV centroid.
pub fn bm_face_uv_calc_center_median(
    bm: &BMesh,
    f: usize,
    uv_at: impl Fn(&BMesh, usize) -> Vec2,
) -> Vec2 {
    let loops = face_loops(bm, f);
    let mut cent = Vec2::ZERO;
    for &l in &loops {
        cent += uv_at(bm, l);
    }
    cent * (1.0 / loops.len() as f32)
}

/// `BM_face_uv_calc_cross` — signed UV area of the face.
pub fn bm_face_uv_calc_cross(bm: &BMesh, f: usize, uv_at: impl Fn(&BMesh, usize) -> Vec2) -> f32 {
    let uvs: Vec<Vec2> = face_loops(bm, f).iter().map(|&l| uv_at(bm, l)).collect();
    blenders_math::geom::cross_poly_v2(&uvs)
}

/// `BM_face_uv_minmax` — UV bounding box of the face.
pub fn bm_face_uv_minmax(
    bm: &BMesh,
    f: usize,
    uv_at: impl Fn(&BMesh, usize) -> Vec2,
) -> (Vec2, Vec2) {
    let mut min = Vec2::new(f32::MAX, f32::MAX);
    let mut max = Vec2::new(-f32::MAX, -f32::MAX);
    for l in face_loops(bm, f) {
        let uv = uv_at(bm, l);
        min = min.min(uv);
        max = max.max(uv);
    }
    (min, max)
}

/// `BM_loop_uv_share_edge_check` — two loops on the same edge share UVs?
pub fn bm_loop_uv_share_edge_check(
    bm: &BMesh,
    l_a: usize,
    l_b: usize,
    uv_at: impl Fn(&BMesh, usize) -> Vec2,
) -> bool {
    debug_assert_eq!(bm.loo(l_a).e, bm.loo(l_b).e);
    let (luv_a_curr, luv_a_next) = (uv_at(bm, l_a), uv_at(bm, bm.loo(l_a).next));
    let (mut luv_b_curr, mut luv_b_next) = (uv_at(bm, l_b), uv_at(bm, bm.loo(l_b).next));
    if bm.loo(l_a).v != bm.loo(l_b).v {
        std::mem::swap(&mut luv_b_curr, &mut luv_b_next);
    }
    luv_a_curr == luv_b_curr && luv_a_next == luv_b_next
}

/// `BM_loop_uv_share_vert_check` — two loops on the same vert share UV?
pub fn bm_loop_uv_share_vert_check(
    bm: &BMesh,
    l_a: usize,
    l_b: usize,
    uv_at: impl Fn(&BMesh, usize) -> Vec2,
) -> bool {
    debug_assert_eq!(bm.loo(l_a).v, bm.loo(l_b).v);
    uv_at(bm, l_a) == uv_at(bm, l_b)
}

/// `BM_edge_uv_share_vert_check` — loops share UV and so do the adjacent
/// loops reaching the edge's other end.
pub fn bm_edge_uv_share_vert_check(
    bm: &BMesh,
    e: usize,
    l_a: usize,
    l_b: usize,
    uv_at: impl Fn(&BMesh, usize) -> Vec2,
) -> bool {
    debug_assert_eq!(bm.loo(l_a).v, bm.loo(l_b).v);
    if !bm_loop_uv_share_vert_check(bm, l_a, l_b, &uv_at) {
        return false;
    }
    let v_other = if bm.edge(e).v1 == bm.loo(l_a).v {
        bm.edge(e).v2
    } else {
        bm.edge(e).v1
    };
    let l_other_a = loop_other_vert_loop_by_edge(bm, l_a, v_other);
    let l_other_b = loop_other_vert_loop_by_edge(bm, l_b, v_other);
    if l_other_a == BM_NIL || l_other_b == BM_NIL {
        return false;
    }
    uv_at(bm, l_other_a) == uv_at(bm, l_other_b)
}

/// `BM_loop_other_vert_loop_by_edge` — loop after `l` whose vert is `v`.
fn loop_other_vert_loop_by_edge(bm: &BMesh, l: usize, v_other: usize) -> usize {
    let l_next = bm.loo(l).next;
    if bm.loo(l_next).v == v_other {
        l_next
    } else {
        let l_prev = bm.loo(l).prev;
        if bm.loo(l_prev).v == v_other {
            l_prev
        } else {
            BM_NIL
        }
    }
}

/// `BM_face_uv_point_inside_test` — is `co` inside the face's UV polygon?
pub fn bm_face_uv_point_inside_test(
    bm: &BMesh,
    f: usize,
    co: Vec2,
    uv_at: impl Fn(&BMesh, usize) -> Vec2,
) -> bool {
    let projverts: Vec<Vec2> = face_loops(bm, f).iter().map(|&l| uv_at(bm, l)).collect();
    blenders_math::geom::isect_point_poly_v2(co, &projverts)
}
