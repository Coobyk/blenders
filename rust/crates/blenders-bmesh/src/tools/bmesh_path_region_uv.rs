//! Port of `bmesh_path_region_uv.cc` — bidirectional-BFS path-region between
//! elements over UV-connected loops (`BM_mesh_calc_path_uv_region_*`).
//! `cd_loop_uv_offset` CustomData access is replaced by a
//! `uv_at: impl Fn(&BMesh, usize) -> Vec2` accessor (PORT_PENDING CustomData).

use crate::iterators::*;
use crate::operators::BMElemRef;
use crate::{BMesh, BM_NIL};
use crate::{BM_EDGE, BM_ELEM_TAG, BM_FACE, BM_LOOP, BM_VERT};
use blenders_math::vector::Vec2;

/* USE_EDGE_CHAIN is enabled in C. */

/// `bm_loop_pair_ends`.
fn bm_loop_pair_ends(bm: &BMesh, l_pivot: usize) -> Option<[usize; 2]> {
    let mut l_end_pair = [BM_NIL; 2];
    for j in 0..2 {
        let mut l_other = if j == 1 {
            bm.loo(l_pivot).next
        } else {
            bm.loo(l_pivot).prev
        };
        while crate::query::bm_vert_is_edge_pair_manifold(bm, bm.loo(l_other).v) {
            l_other = if j == 1 {
                bm.loo(l_other).next
            } else {
                bm.loo(l_other).prev
            };
            if l_other == l_pivot {
                return None;
            }
        }
        l_end_pair[j] = l_other;
    }
    Some(l_end_pair)
}

/// `bm_loop_region_test`.
fn bm_loop_region_test(bm: &BMesh, l: usize, depths: [&Vec<i32>; 2], pass: i32) -> bool {
    let index = bm.loo(l).head.index as usize;
    depths[0][index] != -1
        && depths[1][index] != -1
        && depths[0][index] + depths[1][index] < pass
}

/// `bm_loop_region_test_chain` (USE_EDGE_CHAIN).
fn bm_loop_region_test_chain(
    bm: &BMesh,
    l: usize,
    depths: [&Vec<i32>; 2],
    pass: i32,
) -> bool {
    if bm_loop_region_test(bm, l, depths, pass) {
        return true;
    }
    if crate::query::bm_vert_is_edge_pair_manifold(bm, bm.loo(l).v) {
        if let Some(l_end_pair) = bm_loop_pair_ends(bm, l) {
            if bm_loop_region_test(bm, l_end_pair[0], depths, pass)
                && bm_loop_region_test(bm, l_end_pair[1], depths, pass)
            {
                return true;
            }
        }
    }
    false
}

/// `mesh_calc_path_region_elem` — bidirectional BFS; returns path as a vec of
/// element indices (loops for BM_VERT/BM_EDGE path types, faces for BM_FACE).
fn mesh_calc_path_region_elem(
    bm: &mut BMesh,
    ele_src: BMElemRef,
    ele_dst: BMElemRef,
    uv_at: &impl Fn(&BMesh, usize) -> Vec2,
    path_htype: u8,
) -> Vec<usize> {
    let mut ele_loops: [Vec<usize>; 2] = [Vec::new(), Vec::new()];
    for side in 0..2 {
        let ele = if side == 1 { ele_dst } else { ele_src };
        if ele.0 == BM_FACE {
            let f = ele.1;
            let l_first = bm.face(f).l_first;
            let mut l_iter = l_first;
            loop {
                ele_loops[side].push(l_iter);
                l_iter = bm.loo(l_iter).next;
                if l_iter == l_first {
                    break;
                }
            }
        } else if ele.0 == BM_LOOP {
            if path_htype == BM_EDGE {
                let l = ele.1;
                ele_loops[side].push(l);
                ele_loops[side].push(bm.loo(l).next);
            } else if path_htype == BM_VERT {
                ele_loops[side].push(ele.1);
            }
        }
    }
    let totloop = bm.totloop as usize;
    let mut depths: [Vec<i32>; 2] = [vec![-1; totloop], vec![-1; totloop]];
    let mut pass = 0;
    let mut stack: Vec<usize> = Vec::with_capacity(totloop);
    let mut stack_other: Vec<usize> = Vec::with_capacity(totloop);
    crate::mesh::bm_mesh_elem_index_ensure(bm, BM_LOOP);
    let mut found_all = false;
    for side in 0..2 {
        let side_other = 1 - side;
        stack.clear();
        stack_other.clear();
        for &l in &ele_loops[side] {
            depths[side][bm.loo(l).head.index as usize] = 0;
            if bm.loo(l).head.hflag & BM_ELEM_TAG == 0 {
                stack.push(l);
            }
        }
        /* USE_EDGE_CHAIN */
        if path_htype == BM_VERT || path_htype == BM_EDGE {
            for &l in &ele_loops[side] {
                if crate::query::bm_vert_is_edge_pair_manifold(bm, bm.loo(l).v) {
                    if let Some(l_end_pair) = bm_loop_pair_ends(bm, l) {
                        for j in 0..2 {
                            let l_end_index = bm.loo(l_end_pair[j]).head.index as usize;
                            if depths[side][l_end_index] == -1 {
                                depths[side][l_end_index] = 0;
                                if bm.loo(l_end_pair[j]).head.hflag & BM_ELEM_TAG == 0 {
                                    stack.push(l_end_pair[j]);
                                }
                            }
                        }
                    }
                }
            }
        }
        found_all = false;
        pass = 1;
        while !stack.is_empty() {
            while let Some(l_a) = stack.pop() {
                let l_a_index = bm.loo(l_a).head.index as usize;
                for l_iter in
                    LoopsOfVert::new(bm, bm.loo(l_a).v).collect::<Vec<_>>()
                {
                    if bm.loo(l_iter).head.hflag & BM_ELEM_TAG != 0 {
                        continue;
                    }
                    if !crate::query_uv::bm_loop_uv_share_vert_check(
                        bm, l_a, l_iter, uv_at,
                    ) {
                        continue;
                    }
                    let l_iter_index = bm.loo(l_iter).head.index as usize;
                    if depths[side][l_iter_index] == -1 {
                        depths[side][l_iter_index] = depths[side][l_a_index];
                    }
                    for j in 0..2 {
                        let mut l_b = if j == 1 {
                            bm.loo(l_iter).next
                        } else {
                            bm.loo(l_iter).prev
                        };
                        let mut l_b_index = bm.loo(l_b).head.index as usize;
                        if depths[side][l_b_index] == -1 {
                            /* USE_EDGE_CHAIN */
                            let l_stop = if j == 1 {
                                bm.loo(l_iter).prev
                            } else {
                                bm.loo(l_iter).next
                            };
                            while crate::query::bm_vert_is_edge_pair_manifold(
                                bm,
                                bm.loo(l_b).v,
                            ) && depths[side][l_b_index] == -1
                                && l_b != l_stop
                            {
                                depths[side][l_b_index] = pass;
                                l_b = if j == 1 {
                                    bm.loo(l_b).next
                                } else {
                                    bm.loo(l_b).prev
                                };
                                l_b_index = bm.loo(l_b).head.index as usize;
                            }
                            if depths[side][l_b_index] == -1 {
                                debug_assert!(
                                    pass == depths[side][l_a_index] + 1
                                );
                                depths[side][l_b_index] = pass;
                                if bm.loo(l_b).head.hflag & BM_ELEM_TAG == 0 {
                                    stack_other.push(l_b);
                                }
                            }
                        }
                    }
                }
            }
            found_all = true;
            for &l in &ele_loops[side_other] {
                if depths[side][bm.loo(l).head.index as usize] == -1 {
                    found_all = false;
                    break;
                }
            }
            if found_all {
                pass += 1;
                break;
            }
            std::mem::swap(&mut stack, &mut stack_other);
            pass += 1;
        }
        if !found_all {
            break;
        }
    }
    let mut path = Vec::new();
    if !found_all {
        /* empty */
    } else if path_htype == BM_FACE {
        for f in FacesOfMesh::new(bm) {
            if bm.face(f).head.hflag & BM_ELEM_TAG == 0 {
                let l_first = bm.face(f).l_first;
                let mut l_iter = l_first;
                let mut ok = true;
                let mut ok_tests = if bm.face(f).len > 3 { 1 } else { 0 };
                loop {
                    if !bm_loop_region_test_chain(bm, l_iter, [&depths[0], &depths[1]], pass)
                    {
                        if ok_tests == 0 {
                            ok = false;
                            break;
                        }
                        ok_tests -= 1;
                    }
                    l_iter = bm.loo(l_iter).next;
                    if l_iter == l_first {
                        break;
                    }
                }
                if ok {
                    path.push(f);
                }
            }
        }
    } else if path_htype == BM_EDGE {
        for f in FacesOfMesh::new(bm) {
            let l_first = bm.face(f).l_first;
            let mut l_in_chain_next =
                bm_loop_region_test_chain(bm, l_first, [&depths[0], &depths[1]], pass);
            let mut l = l_first;
            loop {
                let l_in_chain = l_in_chain_next;
                l_in_chain_next = bm_loop_region_test_chain(
                    bm,
                    bm.loo(l).next,
                    [&depths[0], &depths[1]],
                    pass,
                );
                if l_in_chain && l_in_chain_next {
                    path.push(l);
                }
                l = bm.loo(l).next;
                if l == l_first {
                    break;
                }
            }
        }
    } else if path_htype == BM_VERT {
        for f in FacesOfMesh::new(bm) {
            let l_first = bm.face(f).l_first;
            let mut l = l_first;
            loop {
                if bm_loop_region_test_chain(bm, l, [&depths[0], &depths[1]], pass) {
                    path.push(l);
                }
                l = bm.loo(l).next;
                if l == l_first {
                    break;
                }
            }
        }
    }
    path
}

/// `BM_mesh_calc_path_uv_region_vert`.
pub fn bm_mesh_calc_path_uv_region_vert(
    bm: &mut BMesh,
    ele_src: BMElemRef,
    ele_dst: BMElemRef,
    filter_fn: &dyn Fn(&BMesh, usize) -> bool,
    uv_at: &impl Fn(&BMesh, usize) -> Vec2,
) -> Vec<usize> {
    let mut i = 0i32;
    for f in FacesOfMesh::new(bm).collect::<Vec<_>>() {
        for l in LoopsOfFace::new(bm, f).collect::<Vec<_>>() {
            let ok = filter_fn(bm, l);
            if ok {
                bm.lpool[l].head.hflag &= !BM_ELEM_TAG;
            } else {
                bm.lpool[l].head.hflag |= BM_ELEM_TAG;
            }
            bm.lpool[l].head.index = i;
            i += 1;
        }
    }
    mesh_calc_path_region_elem(bm, ele_src, ele_dst, uv_at, BM_VERT)
}

/// `BM_mesh_calc_path_uv_region_edge`.
pub fn bm_mesh_calc_path_uv_region_edge(
    bm: &mut BMesh,
    ele_src: BMElemRef,
    ele_dst: BMElemRef,
    filter_fn: &dyn Fn(&BMesh, usize) -> bool,
    uv_at: &impl Fn(&BMesh, usize) -> Vec2,
) -> Vec<usize> {
    let mut i = 0i32;
    for f in FacesOfMesh::new(bm).collect::<Vec<_>>() {
        for l in LoopsOfFace::new(bm, f).collect::<Vec<_>>() {
            let ok = filter_fn(bm, l);
            if ok {
                bm.lpool[l].head.hflag &= !BM_ELEM_TAG;
            } else {
                bm.lpool[l].head.hflag |= BM_ELEM_TAG;
            }
            bm.lpool[l].head.index = i;
            i += 1;
        }
    }
    mesh_calc_path_region_elem(bm, ele_src, ele_dst, uv_at, BM_EDGE)
}

/// `BM_mesh_calc_path_uv_region_face`.
pub fn bm_mesh_calc_path_uv_region_face(
    bm: &mut BMesh,
    ele_src: BMElemRef,
    ele_dst: BMElemRef,
    filter_fn: &dyn Fn(&BMesh, usize) -> bool,
    uv_at: &impl Fn(&BMesh, usize) -> Vec2,
) -> Vec<usize> {
    let mut i = 0i32;
    crate::marking::bm_mesh_elem_hflag_enable_all(bm, BM_VERT, BM_ELEM_TAG);
    for f in FacesOfMesh::new(bm).collect::<Vec<_>>() {
        let ok = filter_fn(bm, f);
        if ok {
            bm.fpool[f].head.hflag &= !BM_ELEM_TAG;
        } else {
            bm.fpool[f].head.hflag |= BM_ELEM_TAG;
        }
        bm.fpool[f].head.index = i;
        i += 1;
        if ok {
            /* face passes filter: clear tags on its verts */
            let l_first = bm.face(f).l_first;
            let mut l_iter = l_first;
            loop {
                let v = bm.loo(l_iter).v;
                bm.vpool[v].head.hflag &= !BM_ELEM_TAG;
                l_iter = bm.loo(l_iter).next;
                if l_iter == l_first {
                    break;
                }
            }
        }
    }
    mesh_calc_path_region_elem(bm, ele_src, ele_dst, uv_at, BM_FACE)
}
