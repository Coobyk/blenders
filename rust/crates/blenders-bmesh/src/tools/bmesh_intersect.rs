//! Port of `bmesh/tools/bmesh_intersect.cc`.
//!
//! Tri-tri self/cross intersection for boolean & self-intersect ops:
//! BVHTree overlap → `bm_isect_tri_tri` → `edge_verts`/`face_edges` maps →
//! edge splice, wire dissolve, edgenet face splits, boolean face groups.
//! `USE_SPLICE`, `USE_NET`, `USE_SEPARATE`, `USE_DISSOLVE`,
//! `USE_NET_ISLAND_CONNECT`, `USE_BVH` all enabled (as upstream);
//! `USE_PARANOID`, `USE_DUMP`, `USE_KDOPBVH_WATERTIGHT` disabled.

use crate::iterators::*;
use crate::{BMesh, BM_NIL};
use crate::{BM_EDGE, BM_FACE, BM_VERT};
use crate::{BM_ELEM_TAG};
use blenders_bli::kdopbvh::{
    bvhtree_balance, bvhtree_insert, bvhtree_new, bvhtree_overlap_ex, bvhtree_ray_cast,
    BVHTree, BVHTreeRay, BVHTreeRayHit, BVH_OVERLAP_RETURN_PAIRS, BVH_OVERLAP_SELF,
    BVH_OVERLAP_USE_THREADING,
};
use blenders_math::geom::{
    isect_line_line_epsilon, isect_line_segment_tri_epsilon, isect_point_tri_prism_closest,
    isect_ray_tri_epsilon, line_point_factor, normal_tri,
};
use blenders_math::vector::Vec3;
use std::collections::{HashMap, HashSet};

/// `BMeshIsectMode` (`bmesh_intersect.hh`).
pub const BMESH_ISECT_BOOLEAN_NONE: i32 = -1;
pub const BMESH_ISECT_BOOLEAN_ISECT: i32 = 0;
pub const BMESH_ISECT_BOOLEAN_UNION: i32 = 1;
pub const BMESH_ISECT_BOOLEAN_DIFFERENCE: i32 = 2;

/// `api_flag` bits for `VERT_VISIT_A`/`VERT_VISIT_B` (`_FLAG_WALK`, `_FLAG_WALK_ALT`).
const VERT_VISIT_A: u8 = 1 << 3;
const VERT_VISIT_B: u8 = 1 << 4;

/// `enum ISectType`.
const IX_NONE: i32 = -1;
const IX_EDGE_TRI_EDGE0: i32 = 0;
const IX_EDGE_TRI_EDGE2: i32 = 2;
const IX_EDGE_TRI: i32 = 3;
const IX_TOT: usize = 4;

#[derive(Default)]
struct ISectEpsilon {
    eps: f32,
    eps_sq: f32,
    eps2x: f32,
    eps2x_sq: f32,
    eps_margin: f32,
    eps_margin_sq: f32,
}

#[derive(Default)]
struct ISectState {
    /// `edgetri_cache` — int[4] key → vert.
    edgetri_cache: HashMap<[i32; 4], usize>,
    /// `edge_verts` — edge → list of verts to splice in.
    edge_verts: HashMap<usize, Vec<usize>>,
    /// `face_edges` — face elem-index → list of wire edges.
    face_edges: HashMap<i32, Vec<usize>>,
    wire_edges: HashSet<usize>,
    /// `vert_dissolve` — tagged verts.
    vert_dissolve: Vec<usize>,
    epsilon: ISectEpsilon,
}

/// `tri_v3_scale` — scale a tri around its centroid.
fn tri_v3_scale(t: &mut [Vec3; 3], scale: f32) {
    let cent = (t[0] + t[1] + t[2]) / 3.0;
    for i in 0..3 {
        t[i] = cent + (t[i] - cent) * scale;
    }
}

/// `ghash_insert_link` — prepend `val` to `map[key]` list; `use_test` dedups.
fn ghash_insert_link(map: &mut HashMap<usize, Vec<usize>>, key: usize, val: usize, use_test: bool) -> bool {
    let ls = map.entry(key).or_default();
    if use_test && ls.contains(&val) {
        return false;
    }
    ls.insert(0, val);
    true
}

fn face_edges_insert(map: &mut HashMap<i32, Vec<usize>>, key: i32, val: usize, use_test: bool) -> bool {
    let ls = map.entry(key).or_default();
    if use_test && ls.contains(&val) {
        return false;
    }
    ls.insert(0, val);
    true
}

/// `edge_verts_sort` — sort splice verts by distance² along the edge.
fn edge_verts_sort(bm: &BMesh, co: Vec3, v_ls: &mut Vec<usize>) {
    v_ls.sort_by(|&a, &b| {
        (bm.vert(a).co - co)
            .length_squared()
            .partial_cmp(&(bm.vert(b).co - co).length_squared())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

fn edge_verts_add(s: &mut ISectState, e: usize, v: usize, use_test: bool) {
    ghash_insert_link(&mut s.edge_verts, e, v, use_test);
}

fn face_edges_add(s: &mut ISectState, f_index: i32, e: usize, use_test: bool) {
    face_edges_insert(&mut s.face_edges, f_index, e, use_test);
}

/// `face_edges_split` — `BM_face_split_edgenet` w/ island connect.
fn face_edges_split(
    bm: &mut BMesh,
    f: usize,
    e_ls: &[usize],
    use_island_connect: bool,
    use_partial_connect: bool,
) {
    let mut edge_arr: Vec<usize> = e_ls.to_vec();
    if use_island_connect {
        edge_arr =
            crate::edgenet::bm_face_split_edgenet_connect_islands(bm, f, &edge_arr, use_partial_connect);
    }
    crate::edgenet::bm_face_split_edgenet(bm, f, &edge_arr);
}

/// `vert_dissolve_add` — tag + register a dissolve candidate vert.
fn vert_dissolve_add(bm: &mut BMesh, s: &mut ISectState, v: usize) {
    debug_assert_eq!(bm.vpool[v].head.hflag & BM_ELEM_TAG, 0);
    debug_assert!(!s.vert_dissolve.contains(&v));
    bm.vpool[v].head.hflag |= BM_ELEM_TAG;
    s.vert_dissolve.insert(0, v);
}

/// `intersect_line_tri` — line (p0,p1) vs tri; returns ISectType and `r_ix`.
fn intersect_line_tri(
    p0: Vec3,
    p1: Vec3,
    t_cos: &[Vec3; 3],
    t_nor: Vec3,
    r_ix: &mut Vec3,
    e: &ISectEpsilon,
) -> i32 {
    let mut p_dir = p0 - p1;
    p_dir.normalize();
    for i_t0 in 0..3usize {
        let i_t1 = (i_t0 + 1) % 3;
        let mut te_dir = t_cos[i_t0] - t_cos[i_t1];
        te_dir.normalize();
        if p_dir.dot(te_dir).abs() < 1.0 - e.eps {
            let (ix_type, ix0, mut ix1) =
                isect_line_line_epsilon(p0, p1, t_cos[i_t0], t_cos[i_t1], 0.0);
            if ix_type != 0 {
                if ix_type == 1 {
                    ix1 = ix0;
                }
                if ix_type == 1 || (ix0 - ix1).length_squared() <= e.eps_margin_sq {
                    let fac = line_point_factor(ix1, t_cos[i_t0], t_cos[i_t1]);
                    if fac >= e.eps_margin && fac <= 1.0 - e.eps_margin {
                        let fac = line_point_factor(ix0, p0, p1);
                        if fac >= e.eps_margin && fac <= 1.0 - e.eps_margin {
                            *r_ix = ix0;
                            return IX_EDGE_TRI_EDGE0 + i_t0 as i32;
                        }
                    }
                }
            }
        }
    }
    if p_dir.dot(t_nor).abs() >= e.eps {
        if let Some((fac, _, _)) =
            isect_line_segment_tri_epsilon(p0, p1, t_cos[0], t_cos[1], t_cos[2], 0.0)
        {
            if fac >= e.eps_margin && fac <= 1.0 - e.eps_margin {
                *r_ix = p0 + (p1 - p0) * fac;
                let d = (t_cos[0] - *r_ix)
                    .length_squared()
                    .min((t_cos[1] - *r_ix).length_squared())
                    .min((t_cos[2] - *r_ix).length_squared());
                if d >= e.eps_margin_sq {
                    return IX_EDGE_TRI;
                }
            }
        }
    }
    IX_NONE
}

/// `bm_isect_edge_tri` — edge (e_v0,e_v1) vs tri `t`; cache-dedup'd vert create.
fn bm_isect_edge_tri(
    bm: &mut BMesh,
    s: &mut ISectState,
    mut e_v0: usize,
    mut e_v1: usize,
    t: &[usize; 3],
    t_index: i32,
    t_cos: &[Vec3; 3],
    t_nor: Vec3,
    r_side: &mut i32,
) -> usize {
    let mut k_arr = [[0i32; 4]; IX_TOT];
    let ti = [
        bm.vert(t[0]).head.index,
        bm.vert(t[1]).head.index,
        bm.vert(t[2]).head.index,
    ];
    let mut ix = Vec3::ZERO;
    if bm.vert(e_v0).head.index > bm.vert(e_v1).head.index {
        std::mem::swap(&mut e_v0, &mut e_v1);
    }
    *r_side = intersect_line_tri(bm.vert(e_v0).co, bm.vert(e_v1).co, t_cos, t_nor, &mut ix, &s.epsilon);
    k_arr[IX_EDGE_TRI as usize] = [bm.vert(e_v0).head.index, bm.vert(e_v1).head.index, t_index, -1];
    for i in 0..3 {
        let (a, b) = match i {
            0 => (ti[0], ti[1]),
            1 => (ti[1], ti[2]),
            _ => (ti[2], ti[0]),
        };
        let mut k = [bm.vert(e_v0).head.index, bm.vert(e_v1).head.index, a, b];
        /* KEY_EDGE_TRI_ORDER */
        if k[2] > k[3] {
            k.swap(2, 3);
        }
        if k[0] > k[2] {
            k.swap(0, 2);
            k.swap(1, 3);
        }
        k_arr[i] = k;
    }
    for k in k_arr.iter() {
        if let Some(&iv) = s.edgetri_cache.get(k) {
            return iv;
        }
    }
    if *r_side != IX_NONE {
        let iv = crate::core::bm_vert_create(bm, ix, None, 0);
        let mut e = bm_edge_exists_local(bm, e_v0, e_v1);
        if e != BM_NIL {
            edge_verts_add(s, e, iv, false);
        } else {
            vert_dissolve_add(bm, s, iv);
        }
        if *r_side >= IX_EDGE_TRI_EDGE0 && *r_side <= IX_EDGE_TRI_EDGE2 {
            let i = (*r_side - IX_EDGE_TRI_EDGE0) as usize;
            e = bm_edge_exists_local(bm, t[i], t[(i + 1) % 3]);
            if e != BM_NIL {
                edge_verts_add(s, e, iv, false);
            }
        }
        s.edgetri_cache.insert(k_arr[*r_side as usize], iv);
        return iv;
    }
    BM_NIL
}

fn bm_edge_exists_local(bm: &BMesh, a: usize, b: usize) -> usize {
    crate::query::bm_edge_exists(bm, a, b)
}

/// `bm_loop_filter_fn` — face-group edge filter (`use_self` grouping).
fn bm_loop_filter_fn(
    bm: &BMesh,
    l: usize,
    test_fn: &dyn Fn(&BMesh, usize) -> i32,
) -> bool {
    let e = bm.lpool[l].e;
    if bm.edge(e).head.hflag & BM_ELEM_TAG != 0 {
        return false;
    }
    if bm.lpool[l].radial_next != l {
        let face_side = test_fn(bm, bm.lpool[l].f);
        let mut l_iter = bm.lpool[l].radial_next;
        loop {
            let other = test_fn(bm, bm.lpool[l_iter].f);
            if other != -1 && other != face_side {
                return false;
            }
            l_iter = bm.lpool[l_iter].radial_next;
            if l_iter == bm.lpool[l].radial_next {
                break;
            }
        }
        return true;
    }
    false
}

/// `bm_isect_tri_tri` — one tri-tri pair (a_index/b_index = looptris slot).
fn bm_isect_tri_tri(
    bm: &mut BMesh,
    s: &mut ISectState,
    a_index: i32,
    b_index: i32,
    a: &[usize; 3],
    b: &[usize; 3],
    no_shared: bool,
) {
    let f_a = bm.lpool[a[0]].f;
    let f_b = bm.lpool[b[0]].f;
    let fv_a = [bm.lpool[a[0]].v, bm.lpool[a[1]].v, bm.lpool[a[2]].v];
    let fv_b = [bm.lpool[b[0]].v, bm.lpool[b[1]].v, bm.lpool[b[2]].v];
    let f_a_cos = [bm.vert(fv_a[0]).co, bm.vert(fv_a[1]).co, bm.vert(fv_a[2]).co];
    let f_b_cos = [bm.vert(fv_b[0]).co, bm.vert(fv_b[1]).co, bm.vert(fv_b[2]).co];

    if no_shared {
        if fv_b.contains(&fv_a[0]) || fv_b.contains(&fv_a[1]) || fv_b.contains(&fv_a[2]) {
            return;
        }
    } else if crate::query::bm_face_share_edge_check(bm, f_a, f_b) {
        return;
    }

    let mut iv_ls_a: Vec<usize> = Vec::with_capacity(8);
    let mut iv_ls_b: Vec<usize> = Vec::with_capacity(8);

    macro_rules! push_test_a {
        ($v:expr) => {{
            let v = $v;
            if bm.vpool[v].head.api_flag & VERT_VISIT_A == 0 {
                bm.vpool[v].head.api_flag |= VERT_VISIT_A;
                iv_ls_a.push(v);
            }
        }};
    }
    macro_rules! push_test_b {
        ($v:expr) => {{
            let v = $v;
            if bm.vpool[v].head.api_flag & VERT_VISIT_B == 0 {
                bm.vpool[v].head.api_flag |= VERT_VISIT_B;
                iv_ls_b.push(v);
            }
        }};
    }

    let mut skip_rest = false;

    /* vert-vert */
    for i_a in 0..3 {
        for i_b in 0..3 {
            if (fv_a_cos_dist(bm, fv_a[i_a], fv_b[i_b])) <= s.epsilon.eps2x_sq {
                push_test_a!(fv_a[i_a]);
                push_test_b!(fv_b[i_b]);
            }
        }
    }
    /* vert-edge (a onto b's edges) */
    for i_a in 0..3 {
        if bm.vert(fv_a[i_a]).head.api_flag & VERT_VISIT_A == 0 {
            for i_b_e0 in 0..3 {
                let i_b_e1 = (i_b_e0 + 1) % 3;
                if bm.vert(fv_b[i_b_e0]).head.api_flag & VERT_VISIT_B != 0
                    || bm.vert(fv_b[i_b_e1]).head.api_flag & VERT_VISIT_B != 0
                {
                    continue;
                }
                let fac = line_point_factor(f_a_cos[i_a], f_b_cos[i_b_e0], f_b_cos[i_b_e1]);
                if fac > 0.0 - s.epsilon.eps && fac < 1.0 + s.epsilon.eps {
                    let ix = f_b_cos[i_b_e0] + (f_b_cos[i_b_e1] - f_b_cos[i_b_e0]) * fac;
                    if (ix - f_a_cos[i_a]).length_squared() <= s.epsilon.eps2x_sq {
                        push_test_b!(fv_a[i_a]);
                        let e = bm_edge_exists_local(bm, fv_b[i_b_e0], fv_b[i_b_e1]);
                        if e != BM_NIL {
                            edge_verts_add(s, e, fv_a[i_a], true);
                        }
                        break;
                    }
                }
            }
        }
    }
    /* vert-edge (b onto a's edges) */
    for i_b in 0..3 {
        if bm.vert(fv_b[i_b]).head.api_flag & VERT_VISIT_B == 0 {
            for i_a_e0 in 0..3 {
                let i_a_e1 = (i_a_e0 + 1) % 3;
                if bm.vert(fv_a[i_a_e0]).head.api_flag & VERT_VISIT_A != 0
                    || bm.vert(fv_a[i_a_e1]).head.api_flag & VERT_VISIT_A != 0
                {
                    continue;
                }
                let fac = line_point_factor(f_b_cos[i_b], f_a_cos[i_a_e0], f_a_cos[i_a_e1]);
                if fac > 0.0 - s.epsilon.eps && fac < 1.0 + s.epsilon.eps {
                    let ix = f_a_cos[i_a_e0] + (f_a_cos[i_a_e1] - f_a_cos[i_a_e0]) * fac;
                    if (ix - f_b_cos[i_b]).length_squared() <= s.epsilon.eps2x_sq {
                        push_test_a!(fv_b[i_b]);
                        let e = bm_edge_exists_local(bm, fv_a[i_a_e0], fv_a[i_a_e1]);
                        if e != BM_NIL {
                            edge_verts_add(s, e, fv_b[i_b], true);
                        }
                        break;
                    }
                }
            }
        }
    }
    /* vert-tri (a into b, shrunken) */
    {
        let mut t_scale = f_b_cos;
        tri_v3_scale(&mut t_scale, 1.0 - s.epsilon.eps2x);
        for i_a in 0..3 {
            if bm.vert(fv_a[i_a]).head.api_flag & VERT_VISIT_A != 0 {
                continue;
            }
            if let Some(ix) =
                isect_point_tri_prism_closest(f_a_cos[i_a], t_scale[0], t_scale[1], t_scale[2])
            {
                if (ix - f_a_cos[i_a]).length_squared() <= s.epsilon.eps2x_sq {
                    push_test_a!(fv_a[i_a]);
                    push_test_b!(fv_a[i_a]);
                }
            }
        }
    }
    /* vert-tri (b into a, shrunken) */
    {
        let mut t_scale = f_a_cos;
        tri_v3_scale(&mut t_scale, 1.0 - s.epsilon.eps2x);
        for i_b in 0..3 {
            if bm.vert(fv_b[i_b]).head.api_flag & VERT_VISIT_B != 0 {
                continue;
            }
            if let Some(ix) =
                isect_point_tri_prism_closest(f_b_cos[i_b], t_scale[0], t_scale[1], t_scale[2])
            {
                if (ix - f_b_cos[i_b]).length_squared() <= s.epsilon.eps2x_sq {
                    push_test_a!(fv_b[i_b]);
                    push_test_b!(fv_b[i_b]);
                }
            }
        }
    }

    if iv_ls_a.len() >= 3 && iv_ls_b.len() >= 3 {
        /* OVERLAP → finally */
        skip_rest = true;
    }

    if !skip_rest {
        let f_a_nor = normal_tri(f_a_cos[0], f_a_cos[1], f_a_cos[2]).0;
        let f_b_nor = normal_tri(f_b_cos[0], f_b_cos[1], f_b_cos[2]).0;
        /* edge-tri (a's edges vs b) */
        for i_a_e0 in 0..3 {
            let i_a_e1 = (i_a_e0 + 1) % 3;
            if bm.vert(fv_a[i_a_e0]).head.api_flag & VERT_VISIT_A != 0
                || bm.vert(fv_a[i_a_e1]).head.api_flag & VERT_VISIT_A != 0
            {
                continue;
            }
            let mut side = IX_NONE;
            let iv = bm_isect_edge_tri(
                bm, s, fv_a[i_a_e0], fv_a[i_a_e1], &fv_b, b_index, &f_b_cos, f_b_nor, &mut side,
            );
            if iv != BM_NIL {
                push_test_a!(iv);
                push_test_b!(iv);
            }
        }
        /* edge-tri (b's edges vs a) */
        for i_b_e0 in 0..3 {
            let i_b_e1 = (i_b_e0 + 1) % 3;
            if bm.vert(fv_b[i_b_e0]).head.api_flag & VERT_VISIT_B != 0
                || bm.vert(fv_b[i_b_e1]).head.api_flag & VERT_VISIT_B != 0
            {
                continue;
            }
            let mut side = IX_NONE;
            let iv = bm_isect_edge_tri(
                bm, s, fv_b[i_b_e0], fv_b[i_b_e1], &fv_a, a_index, &f_a_cos, f_a_nor, &mut side,
            );
            if iv != BM_NIL {
                push_test_a!(iv);
                push_test_b!(iv);
            }
        }
        /* wire edge creation per side with 2 isect verts */
        for i in 0..2 {
            let (ie_vs, f) = if i == 0 {
                if iv_ls_a.len() != 2 {
                    continue;
                }
                ([iv_ls_a[0], iv_ls_a[1]], f_a)
            } else {
                if iv_ls_b.len() != 2 {
                    continue;
                }
                ([iv_ls_b[0], iv_ls_b[1]], f_b)
            };
            let mut ie = bm_edge_exists_local(bm, ie_vs[0], ie_vs[1]);
            let ie_exists = ie != BM_NIL;
            if !ie_exists {
                ie = crate::core::bm_edge_create(bm, ie_vs[0], ie_vs[1], None, 0);
                s.wire_edges.insert(ie);
            } else {
                s.wire_edges.insert(ie);
                if crate::query::bm_edge_in_face(bm, ie, f) {
                    continue;
                }
            }
            face_edges_add(s, bm.face(f).head.index, ie, ie_exists);
        }
    }

    /* finally: clear visit flags */
    for &v in &iv_ls_a {
        bm.vpool[v].head.api_flag &= !VERT_VISIT_A;
    }
    for &v in &iv_ls_b {
        bm.vpool[v].head.api_flag &= !VERT_VISIT_B;
    }
}

#[inline]
fn fv_a_cos_dist(bm: &BMesh, a: usize, b: usize) -> f32 {
    (bm.vert(a).co - bm.vert(b).co).length_squared()
}

/// `isect_bvhtree_point_v3` — count distinct ray-hit depths (inside test).
fn isect_bvhtree_point_v3(tree: &BVHTree, looptri_coords: &[[Vec3; 3]], co: [f32; 3]) -> i32 {
    let mut z_buffer: Vec<f32> = Vec::with_capacity(64);
    {
        let mut cb = |index: i32, ray: &BVHTreeRay, _hit: &mut BVHTreeRayHit| {
            let tri = looptri_coords[index as usize];
            let origin = Vec3::new(ray.origin[0], ray.origin[1], ray.origin[2]);
            let dir = Vec3::new(ray.direction[0], ray.direction[1], ray.direction[2]);
            if let Some((dist, _, _)) =
                isect_ray_tri_epsilon(origin, dir, tri[0], tri[1], tri[2], f32::EPSILON)
            {
                if dist >= 0.0 {
                    z_buffer.push(dist);
                }
            }
        };
        let _ = bvhtree_ray_cast(tree, co, [1.0, 0.0, 0.0], 0.0, None, Some(&mut cb));
    }
    if z_buffer.is_empty() {
        0
    } else if z_buffer.len() == 1 {
        1
    } else {
        let eps = f32::EPSILON * 10.0;
        let mut num_isect = 1; /* always count first */
        z_buffer.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut depth_last = z_buffer[0];
        for &d in &z_buffer[1..] {
            if d - depth_last > eps {
                depth_last = d;
                num_isect += 1;
            }
        }
        num_isect
    }
}

/// `bm_vert_other_edge` (`USE_DISSOLVE` helper) — the edge-pair vert's other edge.
fn bm_vert_other_edge(bm: &BMesh, v: usize, e: usize) -> usize {
    let mut e_other = bm.vert(v).e;
    if e_other == e {
        e_other = crate::structure::bmesh_disk_edge_next(bm, e_other, v);
    }
    debug_assert_ne!(e_other, e);
    e_other
}

/// `BM_mesh_calc_face_groups` with a loop filter (boolean mode path).
/// Returns `(group_tot, groups_array, group_index)` like C:
/// `groups_array` = face elem-indexes ordered by group, `group_index[g] = (start, len)`.
fn calc_face_groups_loop_filter(
    bm: &BMesh,
    loop_filter: &dyn Fn(&BMesh, usize) -> bool,
) -> (i32, Vec<i32>, Vec<[i32; 2]>) {
    /* face elem index → position in fpool order; iterate faces sorted by index */
    let mut faces: Vec<usize> = Vec::with_capacity(bm.totface as usize);
    for f in FacesOfMesh::new(bm) {
        faces.push(f);
    }
    faces.sort_by_key(|&f| bm.face(f).head.index);
    let mut group_of: HashMap<usize, i32> = HashMap::with_capacity(faces.len());
    let mut groups_array: Vec<i32> = Vec::with_capacity(faces.len());
    let mut group_index: Vec<[i32; 2]> = Vec::new();
    for &f0 in &faces {
        if group_of.contains_key(&f0) {
            continue;
        }
        let g = group_index.len() as i32;
        let start = groups_array.len() as i32;
        let mut stack = vec![f0];
        group_of.insert(f0, g);
        while let Some(f) = stack.pop() {
            groups_array.push(bm.face(f).head.index);
            for l in LoopsOfFace::new(bm, f) {
                if loop_filter(bm, l) {
                    let e = bm.lpool[l].e;
                    for lf in FacesOfEdge::new(bm, e) {
                        if let std::collections::hash_map::Entry::Vacant(en) =
                            group_of.entry(lf)
                        {
                            en.insert(g);
                            stack.push(lf);
                        }
                    }
                }
            }
        }
        group_index.push([start, groups_array.len() as i32 - start]);
    }
    (group_index.len() as i32, groups_array, group_index)
}

/// `BM_mesh_intersect` — tri-tri intersect / boolean / self-intersect driver.
///
/// `looptris`: each `[l0,l1,l2]` of loop indices; `test_fn(f) -> i32` = C
/// `test_fn` (0/1 side, -1 skip). Returns true when the mesh was edited.
#[allow(clippy::too_many_arguments)]
pub fn bm_mesh_intersect(
    bm: &mut BMesh,
    looptris: &[[usize; 3]],
    test_fn: &dyn Fn(&BMesh, usize) -> i32,
    use_self: bool,
    use_separate: bool,
    use_dissolve: bool,
    use_island_connect: bool,
    use_partial_connect: bool,
    use_edge_tag: bool,
    boolean_mode: i32,
    eps: f32,
) -> bool {
    let mut s = ISectState::default();
    let totface_orig = bm.totface;
    let mut has_edit_boolean = false;
    let mut looptri_coords: Vec<[Vec3; 3]> = Vec::new();
    /* elem-index face table; built lazily by the dissolve/net paths. */
    let mut faces_store: Option<Vec<Option<usize>>> = None;

    s.epsilon.eps = eps;
    s.epsilon.eps2x = eps * 2.0;
    s.epsilon.eps_margin = s.epsilon.eps2x * 10.0;
    s.epsilon.eps_sq = s.epsilon.eps * s.epsilon.eps;
    s.epsilon.eps2x_sq = s.epsilon.eps2x * s.epsilon.eps2x;
    s.epsilon.eps_margin_sq = s.epsilon.eps_margin * s.epsilon.eps_margin;

    crate::mesh::bm_mesh_elem_index_ensure(bm, BM_VERT | BM_EDGE | BM_FACE);

    if use_dissolve {
        crate::marking::bm_mesh_elem_hflag_disable_all(bm, BM_EDGE | BM_VERT, BM_ELEM_TAG);
    }

    if boolean_mode != BMESH_ISECT_BOOLEAN_NONE {
        looptri_coords.reserve(looptris.len());
        for lt in looptris {
            looptri_coords.push([
                bm.vert(bm.lpool[lt[0]].v).co,
                bm.vert(bm.lpool[lt[1]].v).co,
                bm.vert(bm.lpool[lt[2]].v).co,
            ]);
        }
    }

    /* BVH trees */
    let mut tree_a = bvhtree_new(looptris.len() as i32, s.epsilon.eps_margin, 8, 8);
    for (i, lt) in looptris.iter().enumerate() {
        if test_fn(bm, bm.lpool[lt[0]].f) == 0 {
            let t_cos = [
                bm.vert(bm.lpool[lt[0]].v).co,
                bm.vert(bm.lpool[lt[1]].v).co,
                bm.vert(bm.lpool[lt[2]].v).co,
            ];
            let flat = [
                t_cos[0].x, t_cos[0].y, t_cos[0].z,
                t_cos[1].x, t_cos[1].y, t_cos[1].z,
                t_cos[2].x, t_cos[2].y, t_cos[2].z,
            ];
            bvhtree_insert(&mut tree_a, i as i32, &flat, 3);
        }
    }
    bvhtree_balance(&mut tree_a);

    let mut tree_b = bvhtree_new(looptris.len() as i32, s.epsilon.eps_margin, 8, 8);
    let mut tree_b_is_a = false;
    if !use_self {
        for (i, lt) in looptris.iter().enumerate() {
            if test_fn(bm, bm.lpool[lt[0]].f) == 1 {
                let t_cos = [
                    bm.vert(bm.lpool[lt[0]].v).co,
                    bm.vert(bm.lpool[lt[1]].v).co,
                    bm.vert(bm.lpool[lt[2]].v).co,
                ];
                let flat = [
                    t_cos[0].x, t_cos[0].y, t_cos[0].z,
                    t_cos[1].x, t_cos[1].y, t_cos[1].z,
                    t_cos[2].x, t_cos[2].y, t_cos[2].z,
                ];
                bvhtree_insert(&mut tree_b, i as i32, &flat, 3);
            }
        }
        bvhtree_balance(&mut tree_b);
    } else {
        tree_b_is_a = true;
    }

    let isect_tri_tri_no_shared = boolean_mode != BMESH_ISECT_BOOLEAN_NONE;
    let mut flag = BVH_OVERLAP_USE_THREADING | BVH_OVERLAP_RETURN_PAIRS;
    if looptris.len() < 1024 {
        flag &= !BVH_OVERLAP_USE_THREADING;
    }
    /* `use_self` → same tree on both sides (C: `tree_b = tree_a`). */
    let overlap = if tree_b_is_a {
        bvhtree_overlap_ex(&tree_a, &tree_a, None, 0, flag | BVH_OVERLAP_SELF)
    } else {
        bvhtree_overlap_ex(&tree_b, &tree_a, None, 0, flag)
    };
    for ov in &overlap {
        bm_isect_tri_tri(
            bm,
            &mut s,
            ov.index_a,
            ov.index_b,
            &looptris[ov.index_a as usize],
            &looptris[ov.index_b as usize],
            isect_tri_tri_no_shared,
        );
    }

    /* USE_SPLICE: split edges at cut points, splice verts */
    {
        let edge_keys: Vec<usize> = s.edge_verts.keys().copied().collect();
        for e in edge_keys {
            let mut v_ls = s.edge_verts.remove(&e).unwrap_or_default();
            let (v_start, _v_end) = (bm.edge(e).v1, bm.edge(e).v2);
            if v_ls.len() > 1 {
                edge_verts_sort(bm, bm.vert(v_start).co, &mut v_ls);
            }
            let is_wire = s.wire_edges.contains(&e);
            let mut v_prev = v_start;
            for vi in v_ls {
                if crate::query::bm_vert_in_edge(bm, e, v_prev) {
                    let co_a = bm.vert(bm.edge(e).v1).co;
                    let co_b = bm.vert(bm.edge(e).v2).co;
                    let fac = line_point_factor(bm.vert(vi).co, co_a, co_b).clamp(0.0, 1.0);
                    let mut e_split = BM_NIL;
                    v_prev = crate::mods::bm_edge_split(bm, e, v_prev, Some(&mut e_split), fac);
                    if bm_edge_exists_local(bm, v_prev, vi) == BM_NIL
                        && !crate::query::bm_vert_pair_share_face_check(bm, v_prev, vi)
                        && !crate::core::bm_vert_splice_check_double_edge(bm, v_prev, vi)
                        && !crate::core::bm_vert_splice_check_double_face(bm, v_prev, vi)
                    {
                        crate::core::bm_vert_splice(bm, vi, v_prev);
                    } else {
                        bm.vpool[v_prev].co = bm.vert(vi).co;
                    }
                    v_prev = vi;
                    if is_wire && e_split != BM_NIL {
                        s.wire_edges.insert(e_split);
                    }
                }
            }
        }
    }

    /* USE_DISSOLVE */
    if use_dissolve && boolean_mode == BMESH_ISECT_BOOLEAN_NONE {
        /* clear TAG on verts that are no longer edge-pairs */
        for &v in &s.vert_dissolve {
            if bm.vert(v).head.hflag & BM_ELEM_TAG != 0
                && !crate::query::bm_vert_is_edge_pair(bm, v)
            {
                bm.vpool[v].head.hflag &= !BM_ELEM_TAG;
            }
        }
        let mut splice_ls: Vec<(usize, usize)> = Vec::with_capacity(s.wire_edges.len());
        for &v in &s.vert_dissolve {
            if bm.vert(v).head.hflag & BM_ELEM_TAG == 0 {
                continue;
            }
            let e_pair = [
                bm.vert(v).e,
                crate::structure::bmesh_disk_edge_next(bm, bm.vert(v).e, v),
            ];
            if bm.edge(e_pair[0]).head.hflag & BM_ELEM_TAG != 0
                || bm.edge(e_pair[1]).head.hflag & BM_ELEM_TAG != 0
            {
                continue;
            }
            if !s.wire_edges.contains(&e_pair[0]) || !s.wire_edges.contains(&e_pair[1]) {
                continue;
            }
            let mut v_a = bm_edge_other_vert_local(bm, e_pair[0], v);
            let mut v_b = bm_edge_other_vert_local(bm, e_pair[1], v);
            let tag_a = bm.vert(v_a).head.hflag & BM_ELEM_TAG != 0;
            let tag_b = bm.vert(v_b).head.hflag & BM_ELEM_TAG != 0;
            if tag_a && tag_b {
                /* do nothing */
            } else if !tag_a && !tag_b {
                /* simple case */
                bm.epool[e_pair[1]].head.hflag |= BM_ELEM_TAG;
                splice_ls.push((v, v_b));
            } else {
                /* walk the tagged chain */
                let e = if tag_a { e_pair[0] } else { e_pair[1] };
                if !tag_a {
                    std::mem::swap(&mut v_a, &mut v_b);
                }
                let mut v_step = v;
                let mut e_step = e;
                loop {
                    let v_next = bm_edge_other_vert_local(bm, e_step, v_step);
                    bm.epool[e_step].head.hflag |= BM_ELEM_TAG;
                    if bm.vert(v_next).head.hflag & BM_ELEM_TAG == 0 {
                        splice_ls.push((v, v_next));
                        break;
                    }
                    let e_next = bm_vert_other_edge(bm, v_next, e_step);
                    e_step = e_next;
                    v_step = v_next;
                    bm.epool[e_step].head.hflag |= BM_ELEM_TAG;
                }
            }
        }
        /* drop TAGGED edges from face_edges lists */
        for e_ls in s.face_edges.values_mut() {
            e_ls.retain(|&e| bm.edge(e).head.hflag & BM_ELEM_TAG == 0);
        }
        /* kill tagged edges, invalidate faces in ftable */
        let tagged_edges: Vec<usize> = EdgesOfMesh::new(bm)
            .filter(|&e| bm.edge(e).head.hflag & BM_ELEM_TAG != 0)
            .collect();
        /* faces table indexed by elem index */
        let mut faces: Vec<Option<usize>> = vec![None; totface_orig as usize];
        for f in FacesOfMesh::new(bm) {
            let idx = bm.face(f).head.index;
            if idx >= 0 && (idx as usize) < faces.len() {
                faces[idx as usize] = Some(f);
            }
        }
        for e in tagged_edges {
            let mut l_iter = bm.edge(e).l;
            while l_iter != BM_NIL {
                let f = bm.lpool[l_iter].f;
                let f_index = bm.face(f).head.index;
                if f_index >= 0 && (f_index as usize) < faces.len() {
                    faces[f_index as usize] = None;
                }
                l_iter = bm.lpool[l_iter].radial_next;
                if l_iter == bm.edge(e).l {
                    break;
                }
            }
            s.wire_edges.remove(&e);
            crate::core::bm_edge_kill(bm, e);
        }
        /* kill loose tagged verts; splice surviving pairs */
        let mut verts_invalid: HashSet<usize> = HashSet::new();
        for &v in &s.vert_dissolve {
            if bm.vert(v).head.hflag & BM_ELEM_TAG != 0 && bm.vert(v).e == BM_NIL {
                verts_invalid.insert(v);
                crate::core::bm_vert_kill(bm, v);
            }
        }
        for &(v0, v1) in &splice_ls {
            if !verts_invalid.contains(&v0) && !verts_invalid.contains(&v1) {
                if bm_edge_exists_local(bm, v0, v1) == BM_NIL
                    && !crate::core::bm_vert_splice_check_double_edge(bm, v0, v1)
                    && !crate::core::bm_vert_splice_check_double_face(bm, v0, v1)
                {
                    crate::core::bm_vert_splice(bm, v1, v0);
                }
            }
        }
        /* stash faces table for USE_NET below */
        s.face_edges.retain(|&f_index, _| {
            f_index >= 0
                && (f_index as usize) < faces.len()
                && faces[f_index as usize].is_some()
        });
        faces_store = Some(faces);
    }

    /* USE_NET: split faces by accumulated wire edge lists */
    {
        let faces = match faces_store {
            Some(f) => f,
            None => {
                let mut ftab: Vec<Option<usize>> = vec![None; totface_orig as usize];
                for f in FacesOfMesh::new(bm) {
                    let idx = bm.face(f).head.index;
                    if idx >= 0 && (idx as usize) < ftab.len() {
                        ftab[idx as usize] = Some(f);
                    }
                }
                ftab
            }
        };
        let face_keys: Vec<i32> = s.face_edges.keys().copied().collect();
        for f_index in face_keys {
            let e_ls = s.face_edges.get(&f_index).cloned().unwrap_or_default();
            if f_index < 0 || f_index as usize >= faces.len() {
                continue;
            }
            let f = match faces[f_index as usize] {
                Some(f) => f,
                None => continue,
            };
            face_edges_split(bm, f, &e_ls, use_island_connect, use_partial_connect);
        }
    }

    /* USE_SEPARATE: edgesplit the wire edges */
    if use_separate {
        crate::marking::bm_mesh_elem_hflag_disable_all(bm, BM_EDGE, BM_ELEM_TAG);
        for &e in &s.wire_edges {
            bm.epool[e].head.hflag |= BM_ELEM_TAG;
        }
        crate::tools::bmesh_edgesplit::bm_mesh_edgesplit(bm, false, true, false);
    } else if boolean_mode != BMESH_ISECT_BOOLEAN_NONE || use_edge_tag {
        for &e in &s.wire_edges {
            bm.epool[e].head.hflag |= BM_ELEM_TAG;
        }
    }

    /* boolean face-group removal / flip */
    if boolean_mode != BMESH_ISECT_BOOLEAN_NONE {
        let tree_pair: [&BVHTree; 2] = if tree_b_is_a {
            [&tree_a, &tree_a]
        } else {
            [&tree_a, &tree_b]
        };
        /* ftable (by elem index) */
        let mut ftable: Vec<usize> = vec![BM_NIL; bm.totface as usize];
        for f in FacesOfMesh::new(bm) {
            let idx = bm.face(f).head.index;
            if idx >= 0 && (idx as usize) < ftable.len() {
                ftable[idx as usize] = f;
            }
        }
        let loop_filter = |bm_: &BMesh, l: usize| bm_loop_filter_fn(bm_, l, test_fn);
        let (group_tot, groups_array, group_index) =
            calc_face_groups_loop_filter(bm, &loop_filter);
        for i in 0..group_tot as usize {
            let mut fg = group_index[i][0];
            let fg_end = group_index[i][1] + fg;
            let f = ftable[groups_array[fg as usize] as usize];
            let side_raw = test_fn(bm, f);
            if side_raw == -1 {
                continue;
            }
            /* C: `side = !side` — invert the raw side before use. */
            let side = (side_raw == 0) as i32;
            let co = crate::polygon::bm_face_calc_point_in_face(bm, f);
            let hits = isect_bvhtree_point_v3(
                tree_pair[side as usize],
                &looptri_coords,
                [co.x, co.y, co.z],
            );
            let inside = (hits & 1) == 1;
            let (do_remove, do_flip) = match boolean_mode {
                BMESH_ISECT_BOOLEAN_ISECT => (!inside, false),
                BMESH_ISECT_BOOLEAN_UNION => (inside, false),
                _ /* BMESH_ISECT_BOOLEAN_DIFFERENCE */ => (inside == (side != 0), side == 0),
            };
            if do_remove {
                while fg != fg_end {
                    bm.fpool[ftable[groups_array[fg as usize] as usize]].mat_nr = -1;
                    fg += 1;
                }
            } else if do_flip {
                while fg != fg_end {
                    crate::polygon::bm_face_normal_flip(bm, ftable[groups_array[fg as usize] as usize]);
                    fg += 1;
                }
            }
            has_edit_boolean |= do_flip || do_remove;
        }
        /* dissolve on edge-pair verts not on tris */
        if use_dissolve {
            for &v in &s.vert_dissolve {
                if crate::query::bm_vert_is_edge_pair(bm, v) {
                    let mut ok = true;
                    let e0 = bm.vert(v).e;
                    if e0 != BM_NIL && bm.edge(e0).l != BM_NIL {
                        let l0 = bm.edge(e0).l;
                        let mut l_iter = l0;
                        loop {
                            if bm.face(bm.lpool[l_iter].f).len == 3 {
                                ok = false;
                                break;
                            }
                            l_iter = bm.lpool[l_iter].radial_next;
                            if l_iter == l0 {
                                break;
                            }
                        }
                    }
                    if ok && crate::core::bm_vert_collapse_check_double_face(bm, v) {
                        ok = false;
                    }
                    if ok {
                        let e = bm.vert(v).e;
                        crate::mods::bm_vert_collapse_edge(bm, e, v, true, false, false);
                    }
                }
            }
        }
        /* kill mat_nr == -1 faces */
        let kill: Vec<usize> = ftable
            .iter()
            .copied()
            .filter(|&f| f != BM_NIL && bm.face(f).mat_nr == -1)
            .collect();
        for f in kill {
            crate::core::bm_face_kill_loose(bm, f);
        }
    }

    let has_edit_isect = !s.face_edges.is_empty();
    bm.act_face = BM_NIL;
    has_edit_isect || has_edit_boolean
}

fn bm_edge_other_vert_local(bm: &BMesh, e: usize, v: usize) -> usize {
    let e_ = bm.edge(e);
    if e_.v1 == v {
        e_.v2
    } else {
        e_.v1
    }
}
