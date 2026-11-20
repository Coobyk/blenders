//! Port of `tools/bmesh_decimate_dissolve.cc` — heap-driven edge dissolve
//! (`BM_mesh_decimate_dissolve_ex`) + ear-collapse degenerate checks.
//! `BMO_DELIM_UV` (CustomData loop data) is PORT_PENDING — treated as unset.
//! C `elem_index` scratch → local index side-tables.

use crate::core::*;
use crate::query::*;
use crate::{BMesh, BMFace, BMVert, BM_NIL};
use blenders_bli::heap::Heap;
use blenders_math::vector::{Vec2, Vec3};

pub const BMO_DELIM_NORMAL: u32 = 1 << 0;
pub const BMO_DELIM_MATERIAL: u32 = 1 << 1;
pub const BMO_DELIM_SEAM: u32 = 1 << 2;
pub const BMO_DELIM_SHARP: u32 = 1 << 3;
pub const BMO_DELIM_UV: u32 = 1 << 4;

const COST_INVALID: f32 = f32::MAX;

/// `bm_edge_is_delimiter` (manifold edges only). UV layers → skipped.
fn bm_edge_is_delimiter(bm: &BMesh, e: usize, delimit: u32) -> bool {
    debug_assert!(bm_edge_is_manifold(bm, e));
    if delimit != 0 {
        if delimit & BMO_DELIM_SEAM != 0
            && bm.edge(e).head.hflag & crate::BM_ELEM_SEAM != 0
        { return true; }
        if delimit & BMO_DELIM_SHARP != 0
            && bm.edge(e).head.hflag & crate::BM_ELEM_SMOOTH == 0
        { return true; }
        if delimit & BMO_DELIM_MATERIAL != 0 {
            let l = bm.edge(e).l;
            let l_r = bm.loo(l).radial_next;
            if bm.face(bm.loo(l).f).mat_nr != bm.face(bm.loo(l_r).f).mat_nr {
                return true;
            }
        }
        if delimit & BMO_DELIM_NORMAL != 0 && !bm_edge_is_contiguous(bm, e) {
            return true;
        }
        /* BMO_DELIM_UV: loop customdata — PORT_PENDING, never delimits. */
    }
    false
}

/// `bm_vert_is_delimiter`.
fn bm_vert_is_delimiter(bm: &BMesh, v: usize, delimit: u32) -> bool {
    debug_assert!(bm.vert(v).e != BM_NIL);
    if delimit == 0 { return false; }
    let e_first = bm.vert(v).e;
    let mut e = e_first;
    loop {
        if bm_edge_is_manifold(bm, e) && bm_edge_is_delimiter(bm, e, delimit) {
            return true;
        }
        e = crate::structure::bmesh_disk_edge_next(bm, e, v);
        if e == e_first { break; }
    }
    false
}

/// `bm_vert_edge_face_angle` — cost for vert-collapse (radians, 90°→1.0).
fn bm_vert_edge_face_angle(bm: &BMesh, v: usize, delimit: u32) -> f32 {
    const UNIT_TO_ANGLE: f32 = std::f32::consts::FRAC_PI_2;
    const ANGLE_TO_UNIT: f32 = 1.0 / std::f32::consts::FRAC_PI_2;
    let angle = bm_vert_calc_edge_angle(bm, v);
    let e = bm.vert(v).e;
    if e != BM_NIL && bm_edge_is_manifold(bm, e) {
        if !bm_vert_is_delimiter(bm, v, delimit) {
            return angle * ANGLE_TO_UNIT * (bm_edge_calc_face_angle(bm, e) * ANGLE_TO_UNIT)
                * UNIT_TO_ANGLE;
        }
    }
    angle
}

/// `bm_edge_calc_dissolve_error` — negative-face-normal-dot cost.
fn bm_edge_calc_dissolve_error(bm: &BMesh, e: usize, delimit: u32) -> f32 {
    if bm_edge_is_manifold(bm, e) && !bm_edge_is_delimiter(bm, e, delimit) {
        let l = bm.edge(e).l;
        let l_r = bm.loo(l).radial_next;
        let mut cos_neg = bm.face(bm.loo(l).f).no.dot(bm.face(bm.loo(l_r).f).no);
        if bm_edge_is_contiguous(bm, e) {
            cos_neg *= -1.0;
        }
        return cos_neg;
    }
    COST_INVALID
}

/// `mul_v2_m3v3_center` — project around `center` into 2D basis rows.
fn mul_v2_m3v3_center(m: &blenders_math::matrix::Mat3, a: Vec3, center: Vec3) -> Vec2 {
    let co = a - center;
    Vec2::new(
        m.cols[0].x * co.x + m.cols[1].x * co.y + m.cols[2].x * co.z,
        m.cols[0].y * co.x + m.cols[1].y * co.y + m.cols[2].y * co.z,
    )
}

fn signum_i(x: f32) -> i32 {
    if x > 0.0 { 1 } else if x < 0.0 { -1 } else { 0 }
}

/// `bm_loop_collapse_is_degenerate` — would collapsing this ear flip a
/// neighbor ear or self-intersect the face?
fn bm_loop_collapse_is_degenerate(bm: &BMesh, l_ear: usize) -> bool {
    let center = bm.vert(bm.loo(l_ear).v).co;
    let axis_mat = blenders_math::geom::axis_dominant_to_m3(bm.face(bm.loo(l_ear).f).no);
    let l_prev = bm.loo(l_ear).prev;
    let l_next = bm.loo(l_ear).next;
    let mut tri_2d = [
        mul_v2_m3v3_center(&axis_mat, bm.vert(bm.loo(l_prev).v).co, center),
        Vec2::ZERO, /* l_ear->v itself */
        mul_v2_m3v3_center(&axis_mat, bm.vert(bm.loo(l_next).v).co, center),
    ];
    /* adjacent-vert flip checks */
    let v_prev = bm.loo(l_prev).v;
    let v_next = bm.loo(l_next).v;
    if !bm_vert_is_edge_pair(bm, v_prev) {
        let adj = mul_v2_m3v3_center(
            &axis_mat, bm.vert(bm.loo(bm.loo(l_prev).prev).v).co, center);
        if signum_i(blenders_math::geom::cross_tri_v2(adj, tri_2d[0], tri_2d[1]))
            != signum_i(blenders_math::geom::cross_tri_v2(adj, tri_2d[0], tri_2d[2]))
        { return true; }
    }
    if !bm_vert_is_edge_pair(bm, v_next) {
        let adj = mul_v2_m3v3_center(
            &axis_mat, bm.vert(bm.loo(bm.loo(l_next).next).v).co, center);
        if signum_i(blenders_math::geom::cross_tri_v2(adj, tri_2d[2], tri_2d[1]))
            != signum_i(blenders_math::geom::cross_tri_v2(adj, tri_2d[2], tri_2d[0]))
        { return true; }
    }
    /* point-in-ear self-intersection check */
    if blenders_math::geom::cross_tri_v2(tri_2d[0], tri_2d[1], tri_2d[2]) < 0.0 {
        tri_2d.swap(1, 2);
    }
    let l_first = l_prev;
    let mut l_iter = bm.loo(l_next).next;
    loop {
        let co_2d = mul_v2_m3v3_center(&axis_mat, bm.vert(bm.loo(l_iter).v).co, center);
        if blenders_math::geom::isect_point_tri_v2_cw(
            co_2d, tri_2d[0], tri_2d[1], tri_2d[2])
        { return true; }
        l_iter = bm.loo(l_iter).next;
        if l_iter == l_first { break; }
    }
    false
}

/// `bm_vert_collapse_is_degenerate`.
fn bm_vert_collapse_is_degenerate(bm: &BMesh, v: usize) -> bool {
    let Some((e0, e1)) = bm_vert_edge_pair(bm, v) else { return true };
    if crate::edgenet::edge_is_wire(bm, e0) || crate::edgenet::edge_is_wire(bm, e1) {
        return false;
    }
    let v0 = bm_edge_other_vert(bm, e0, v);
    let v1 = bm_edge_other_vert(bm, e1, v);
    if blenders_math::vector::cos_v3v3v3(bm.vert(v0).co, bm.vert(v).co, bm.vert(v1).co).abs()
        < 1.0 - f32::EPSILON
    {
        /* scan radial loops of e1 for non-triangle faces containing v */
        let l_first = bm.edge(e1).l;
        if l_first == BM_NIL { return false; }
        let mut l_iter = l_first;
        loop {
            if bm.face(bm.loo(l_iter).f).len > 3 {
                let l_pivot = if bm.loo(l_iter).v == v { l_iter } else { bm.loo(l_iter).next };
                debug_assert_eq!(bm.loo(l_pivot).v, v);
                if bm_loop_collapse_is_degenerate(bm, l_pivot) {
                    return true;
                }
            }
            l_iter = bm.loo(l_iter).radial_next;
            if l_iter == l_first { break; }
        }
    }
    false
}

/// `BM_mesh_decimate_dissolve_ex` — dissolve planar-connected edges/faces,
/// then collapse edge-pair verts under the angle limit. `vinput`/`einput`
/// mirror the C in/out arrays (`None` kills a vertex entry).
pub fn bm_mesh_decimate_dissolve_ex(
    bm: &mut BMesh,
    angle_limit: f32,
    do_dissolve_boundaries: bool,
    delimit: u32,
    vinput: &mut Vec<Option<usize>>,
    einput: &[usize],
    oflag_out: u8)
{
    let angle_limit_cos_neg = -angle_limit.cos();

    /* --- edge/face dissolve pass --- */
    {
        let mut eheap: Heap<usize> = Heap::new();
        let eheap_table_len = if do_dissolve_boundaries {
            einput.len()
        } else {
            einput.len().max(vinput.len())
        };
        let mut eheap_table: Vec<Option<blenders_bli::heap::HeapNode>> =
            vec![None; eheap_table_len];
        let mut e_index: std::collections::HashMap<usize, usize> =
            std::collections::HashMap::new();
        for e_iter in crate::iterators::EdgesOfMesh::new(bm).collect::<Vec<_>>() {
            let wire = crate::edgenet::edge_is_wire(bm, e_iter);
            bm.pool[bm.edge(e_iter).head.arena_index()].hflag_mut_set(
                if wire { crate::BM_ELEM_TAG } else { 0 }, crate::BM_ELEM_TAG);
        }
        for (i, &e) in einput.iter().enumerate() {
            let cost = bm_edge_calc_dissolve_error(bm, e, delimit);
            eheap_table[i] = Some(eheap.insert(cost, e));
            e_index.insert(e, i);
        }
        while let Some(&e) = eheap.top() {
            if eheap.top_cost() >= angle_limit_cos_neg { break; }
            let node = eheap.top_node().unwrap();
            let i = *e_index.get(&e).unwrap_or(&usize::MAX);
            let mut f_new: usize = BM_NIL;
            if bm_edge_is_manifold(bm, e) {
                let l = bm.edge(e).l;
                let l_r = bm.loo(l).radial_next;
                f_new = crate::mods::bm_faces_join_pair(bm, l, l_r, false, None);
                if f_new != BM_NIL {
                    eheap.remove(node);
                    if i != usize::MAX { eheap_table[i] = None; }
                    crate::normals::bm_face_calc_normals(bm);
                    if oflag_out != 0 {
                        bm.face_mut(f_new).head.oflag |= oflag_out;
                    }
                    let l_first = bm.face(f_new).l_first;
                    let mut l_iter = l_first;
                    loop {
                        let le = bm.loo(l_iter).e;
                        if let Some(&j) = e_index.get(&le) {
                            if let Some(hn) = eheap_table[j] {
                                let cost = bm_edge_calc_dissolve_error(bm, le, delimit);
                                eheap.update(hn, cost);
                            }
                        }
                        l_iter = bm.loo(l_iter).next;
                        if l_iter == l_first { break; }
                    }
                }
            }
            if f_new == BM_NIL {
                eheap.update(node, COST_INVALID);
            }
        }
        /* kill wire edges marked non-TAG, and verts left dangling */
        crate::indexing::bm_mesh_elem_index_ensure(bm, 1 << crate::BM_VERT);
        let mut vert_reverse_lookup = vec![usize::MAX; bm.totvert];
        for (i, v) in vinput.iter().enumerate() {
            if let Some(v) = v {
                vert_reverse_lookup[bm.vert(*v).head.index] = i;
            }
        }
        let earray: Vec<usize> = crate::iterators::EdgesOfMesh::new(bm).collect();
        for &e_iter in earray.iter().rev() {
            if crate::edgenet::edge_is_wire(bm, e_iter)
                && bm.edge(e_iter).head.hflag & crate::BM_ELEM_TAG == 0
            {
                let (v1, v2) = (bm.edge(e_iter).v1, bm.edge(e_iter).v2);
                bm_edge_kill(bm, e_iter);
                for v in [v1, v2] {
                    if bm.vert(v).e == BM_NIL {
                        let idx = bm.vert(v).head.index;
                        let r = vert_reverse_lookup[idx];
                        if r != usize::MAX { vinput[r] = None; }
                        bm_vert_kill(bm, v);
                    }
                }
            }
        }
    }

    if do_dissolve_boundaries {
        for v in vinput.iter() {
            if let Some(v) = v {
                if bm_vert_is_edge_pair(bm, *v) {
                    let e = bm.vert(*v).e;
                    bm_vert_collapse_edge(bm, e, *v, true, true, true);
                }
            }
        }
    } else {
        /* --- vert-collapse pass --- */
        let mut vheap: Heap<usize> = Heap::new();
        let mut vheap_table: Vec<Option<blenders_bli::heap::HeapNode>> =
            vec![None; vinput.len()];
        let mut v_index: std::collections::HashMap<usize, usize> =
            std::collections::HashMap::new();
        for (i, v) in vinput.iter().enumerate() {
            if let Some(v) = v {
                let cost = bm_vert_edge_face_angle(bm, *v, delimit);
                vheap_table[i] = Some(vheap.insert(cost, *v));
                v_index.insert(*v, i);
            }
        }
        while let Some(&v) = vheap.top() {
            if vheap.top_cost() >= angle_limit { break; }
            let node = vheap.top_node().unwrap();
            let i = *v_index.get(&v).unwrap_or(&usize::MAX);
            let mut e_new: usize = BM_NIL;
            if !bm_vert_collapse_is_degenerate(bm, v) {
                let e = bm.vert(v).e;
                e_new = bm_vert_collapse_edge(bm, e, v, true, true, true);
                if e_new != BM_NIL {
                    vheap.remove(node);
                    if i != usize::MAX { vheap_table[i] = None; }
                    if bm.edge(e_new).l != BM_NIL {
                        let l_first = bm.edge(e_new).l;
                        let mut l_iter = l_first;
                        loop {
                            l_iter = bm.loo(l_iter).radial_next;
                            if l_iter == l_first { break; }
                        }
                        crate::normals::bm_face_calc_normals(bm);
                    }
                    for v_iter in crate::iterators::VertsOfEdge::new(bm, e_new)
                        .collect::<Vec<_>>()
                    {
                        if let Some(&j) = v_index.get(&v_iter) {
                            if let Some(hn) = vheap_table[j] {
                                let cost = bm_vert_edge_face_angle(bm, v_iter, delimit);
                                vheap.update(hn, cost);
                            }
                        }
                    }
                    /* re-validate verts that were COST_INVALID inside ear range */
                    if bm.edge(e_new).l != BM_NIL {
                        let l_first = bm.edge(e_new).l;
                        let mut l_iter = l_first;
                        loop {
                            let l_cycle_first = bm.loo(l_iter).prev;
                            let mut l_cycle_iter = bm.loo(bm.loo(l_iter).next).next;
                            loop {
                                let cv = bm.loo(l_cycle_iter).v;
                                if let Some(&j) = v_index.get(&cv) {
                                    if let Some(hn) = vheap_table[j] {
                                        if vheap.node_value(hn) == COST_INVALID {
                                            let cost = bm_vert_edge_face_angle(
                                                bm, cv, delimit);
                                            vheap.update(hn, cost);
                                        }
                                    }
                                }
                                l_cycle_iter = bm.loo(l_cycle_iter).next;
                                if l_cycle_iter == l_cycle_first { break; }
                            }
                            l_iter = bm.loo(l_iter).radial_next;
                            if l_iter == l_first { break; }
                        }
                    }
                }
            }
            if e_new == BM_NIL {
                vheap.update(node, COST_INVALID);
            }
        }
    }
}
