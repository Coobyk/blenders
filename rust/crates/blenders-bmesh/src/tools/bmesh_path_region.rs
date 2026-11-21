//! Port of `bmesh_path_region.cc` — region between two elements via
//! bidirectional BFS depth maps (`USE_EDGE_CHAIN` variant).

use crate::iterators::*;
use crate::operators::BMElemRef;
use crate::{BMesh, BM_EDGE, BM_ELEM_TAG, BM_FACE, BM_NIL, BM_VERT};

#[inline]
fn other_vert(bm: &BMesh, e: usize, v: usize) -> usize {
    if bm.edge(e).v1 == v { bm.edge(e).v2 } else { bm.edge(e).v1 }
}

#[inline]
fn disk_next(bm: &BMesh, e: usize, v: usize) -> usize {
    crate::structure::bmesh_disk_edge_next(bm, e, v)
}

/// `bm_vert_pair_ends` — for a vert with 2 edge users, find the vertices at
/// each end-point (skipping chains of edge-pair-manifold verts).
fn bm_vert_pair_ends(bm: &BMesh, v_pivot: usize, v_end_pair: &mut [usize; 2]) -> bool {
    let mut e = bm.vert(v_pivot).e;
    let e_first = e;
    let mut j = 0;
    loop {
        let mut e_chain = e;
        let mut v_other = other_vert(bm, e_chain, v_pivot);
        while crate::query::bm_vert_is_edge_pair_manifold(bm, v_other) {
            let e_chain_next = disk_next(bm, e_chain, v_other);
            v_other = other_vert(bm, e_chain_next, v_other);
            if v_other == v_pivot {
                return false;
            }
            e_chain = e_chain_next;
        }
        v_end_pair[j] = v_other;
        j += 1;
        e = disk_next(bm, e, v_pivot);
        if e == e_first {
            break;
        }
    }
    debug_assert_eq!(j, 2);
    true
}

/// `bm_vert_region_test`.
fn bm_vert_region_test(bm: &BMesh, v: usize, depths: [&[i32]; 2], pass: i32) -> bool {
    let index = bm.vert(v).head.index as usize;
    depths[0][index] != -1
        && depths[1][index] != -1
        && depths[0][index] + depths[1][index] < pass
}

/// `bm_vert_region_test_chain` (USE_EDGE_CHAIN).
fn bm_vert_region_test_chain(bm: &BMesh, v: usize, depths: [&[i32]; 2], pass: i32) -> bool {
    if bm_vert_region_test(bm, v, depths, pass) {
        return true;
    }
    let mut v_end_pair = [BM_NIL; 2];
    if crate::query::bm_vert_is_edge_pair_manifold(bm, v)
        && bm_vert_pair_ends(bm, v, &mut v_end_pair)
        && bm_vert_region_test(bm, v_end_pair[0], depths, pass)
        && bm_vert_region_test(bm, v_end_pair[1], depths, pass)
    {
        return true;
    }
    false
}

/// `mesh_calc_path_region_elem` — bidirectional BFS; verts without
/// `BM_ELEM_TAG` are walked over.
fn mesh_calc_path_region_elem(
    bm: &mut BMesh,
    ele_src: BMElemRef,
    ele_dst: BMElemRef,
    path_htype: u8,
) -> Vec<BMElemRef> {
    let mut ele_verts: [Vec<usize>; 2] = [Vec::new(), Vec::new()];
    for side in 0..2 {
        let (ht, idx) = if side == 1 { ele_dst } else { ele_src };
        if ht == BM_FACE {
            let l_first = bm.face(idx).l_first;
            let mut l = l_first;
            loop {
                ele_verts[side].push(bm.loo(l).v);
                l = bm.loo(l).next;
                if l == l_first {
                    break;
                }
            }
        } else if ht == BM_EDGE {
            ele_verts[side].push(bm.edge(idx).v1);
            ele_verts[side].push(bm.edge(idx).v2);
        } else if ht == BM_VERT {
            ele_verts[side].push(idx);
        } else {
            debug_assert!(false);
        }
    }

    let mut depths: [Vec<i32>; 2] = [Vec::new(), Vec::new()];
    let mut pass = 0;
    let mut stack: Vec<usize> = Vec::with_capacity(bm.totvert as usize);
    let mut stack_other: Vec<usize> = Vec::with_capacity(bm.totvert as usize);
    crate::mesh::bm_mesh_elem_index_ensure(bm, BM_VERT);

    /* After exhausting all possible elements, we should have found all elements
     * on the 'side_other'. otherwise, exit early. */
    let mut found_all = false;
    for side in 0..2 {
        let side_other = 1 - side;
        /* initialize depths to -1 (un-touched) */
        depths[side] = vec![-1; bm.totvert as usize];
        stack.clear();
        stack_other.clear();
        for i in 0..ele_verts[side].len() {
            let v = ele_verts[side][i];
            depths[side][bm.vert(v).head.index as usize] = 0;
            if bm.vert(v).e != BM_NIL && bm.vert(v).head.hflag & BM_ELEM_TAG == 0 {
                stack.push(v);
            }
        }
        /* Expand initial state to end-point vertices when they only have 2x
         * edges, preventing odd behavior when src/dst are mid-chain. */
        if path_htype == BM_VERT || path_htype == BM_EDGE {
            for i in 0..ele_verts[side].len() {
                let v = ele_verts[side][i];
                let mut v_end_pair = [BM_NIL; 2];
                if crate::query::bm_vert_is_edge_pair_manifold(bm, v)
                    && bm_vert_pair_ends(bm, v, &mut v_end_pair)
                {
                    for j in 0..2 {
                        let vi = bm.vert(v_end_pair[j]).head.index as usize;
                        if depths[side][vi] == -1 {
                            depths[side][vi] = 0;
                            if bm.vert(v_end_pair[j]).head.hflag & BM_ELEM_TAG == 0 {
                                stack.push(v_end_pair[j]);
                            }
                        }
                    }
                }
            }
        }
        /* Walk until we find all verts on side_other, or no connection. */
        found_all = false;
        pass = 1;
        while !stack.is_empty() {
            while let Some(v_a) = stack.pop() {
                let mut e = bm.vert(v_a).e;
                let e_first = e;
                loop {
                    let mut v_b = other_vert(bm, e, v_a);
                    let mut v_b_index = bm.vert(v_b).head.index as usize;
                    if depths[side][v_b_index] == -1 {
                        /* Walk along the chain, fill in values until we reach a
                         * vertex with 3+ edges. */
                        let mut e_chain = e;
                        while crate::query::bm_vert_is_edge_pair_manifold(bm, v_b)
                            && depths[side][v_b_index] == -1
                        {
                            depths[side][v_b_index] = pass;
                            let e_chain_next = disk_next(bm, e_chain, v_b);
                            v_b = other_vert(bm, e_chain_next, v_b);
                            v_b_index = bm.vert(v_b).head.index as usize;
                            e_chain = e_chain_next;
                        }
                        /* Add the other vertex to the stack for the next pass. */
                        if depths[side][v_b_index] == -1 {
                            depths[side][v_b_index] = pass;
                            if bm.vert(v_b).head.hflag & BM_ELEM_TAG == 0 {
                                stack_other.push(v_b);
                            }
                        }
                    }
                    e = disk_next(bm, e, v_a);
                    if e == e_first {
                        break;
                    }
                }
            }
            /* Stop searching once there's none left. */
            found_all = true;
            for i in 0..ele_verts[side_other].len() {
                if depths[side][bm.vert(ele_verts[side_other][i]).head.index as usize] == -1 {
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

    /* select elements whose verts are within the region. */
    let mut path: Vec<BMElemRef> = Vec::new();
    if !found_all {
        /* fail! (do nothing) */
    } else if path_htype == BM_FACE {
        for f in FacesOfMesh::new(bm) {
            if bm.face(f).head.hflag & BM_ELEM_TAG == 0 {
                /* Allowing a single failure on a face gives fewer 'gaps'. */
                let mut ok = true;
                let mut ok_tests = if bm.face(f).len > 3 { 1 } else { 0 };
                let l_first = bm.face(f).l_first;
                let mut l = l_first;
                loop {
                    if !bm_vert_region_test_chain(bm, bm.loo(l).v, [&depths[0], &depths[1]], pass) {
                        if ok_tests == 0 {
                            ok = false;
                            break;
                        }
                        ok_tests -= 1;
                    }
                    l = bm.loo(l).next;
                    if l == l_first {
                        break;
                    }
                }
                if ok {
                    path.push((BM_FACE, f));
                }
            }
        }
    } else if path_htype == BM_EDGE {
        for e in EdgesOfMesh::new(bm) {
            if bm.edge(e).head.hflag & BM_ELEM_TAG == 0 {
                let mut ok = true;
                for j in 0..2 {
                    let v = if j == 0 { bm.edge(e).v1 } else { bm.edge(e).v2 };
                    if !bm_vert_region_test_chain(bm, v, [&depths[0], &depths[1]], pass) {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    path.push((BM_EDGE, e));
                }
            }
        }
    } else if path_htype == BM_VERT {
        for v in VertsOfMesh::new(bm) {
            if bm_vert_region_test_chain(bm, v, [&depths[0], &depths[1]], pass) {
                path.push((BM_VERT, v));
            }
        }
    }
    path
}

/// `BM_mesh_calc_path_region_vert`.
pub fn bm_mesh_calc_path_region_vert(
    bm: &mut BMesh,
    ele_src: BMElemRef,
    ele_dst: BMElemRef,
    filter_fn: &dyn Fn(&BMesh, usize) -> bool,
) -> Vec<BMElemRef> {
    /* BM_ELEM_TAG flag is used to store visited verts */
    for (i, v) in VertsOfMesh::new(bm).enumerate().collect::<Vec<_>>() {
        let test = filter_fn(bm, v);
        if test {
            bm.vpool[v].head.hflag &= !BM_ELEM_TAG;
        } else {
            bm.vpool[v].head.hflag |= BM_ELEM_TAG;
        }
        bm.vpool[v].head.index = i as i32; /* set_inline */
    }
    bm.elem_index_dirty = false; /* vert index just set inline */
    mesh_calc_path_region_elem(bm, ele_src, ele_dst, BM_VERT)
}

/// `BM_mesh_calc_path_region_edge`.
pub fn bm_mesh_calc_path_region_edge(
    bm: &mut BMesh,
    ele_src: BMElemRef,
    ele_dst: BMElemRef,
    filter_fn: &dyn Fn(&BMesh, usize) -> bool,
) -> Vec<BMElemRef> {
    crate::marking::bm_mesh_elem_hflag_enable_all(bm, BM_VERT, BM_ELEM_TAG);
    for e in EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        let test = filter_fn(bm, e);
        if test {
            bm.epool[e].head.hflag &= !BM_ELEM_TAG;
        } else {
            bm.epool[e].head.hflag |= BM_ELEM_TAG;
        }
        /* flush tag to verts */
        if !test {
            let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
            bm.vpool[v1].head.hflag &= !BM_ELEM_TAG;
            bm.vpool[v2].head.hflag &= !BM_ELEM_TAG;
        }
    }
    mesh_calc_path_region_elem(bm, ele_src, ele_dst, BM_EDGE)
}

/// `BM_mesh_calc_path_region_face`.
pub fn bm_mesh_calc_path_region_face(
    bm: &mut BMesh,
    ele_src: BMElemRef,
    ele_dst: BMElemRef,
    filter_fn: &dyn Fn(&BMesh, usize) -> bool,
) -> Vec<BMElemRef> {
    crate::marking::bm_mesh_elem_hflag_enable_all(bm, BM_VERT, BM_ELEM_TAG);
    for f in FacesOfMesh::new(bm).collect::<Vec<_>>() {
        let test = filter_fn(bm, f);
        if test {
            bm.fpool[f].head.hflag &= !BM_ELEM_TAG;
        } else {
            bm.fpool[f].head.hflag |= BM_ELEM_TAG;
        }
        /* flush tag to verts */
        if !test {
            let l_first = bm.face(f).l_first;
            let mut l = l_first;
            loop {
                let v = bm.loo(l).v;
                bm.vpool[v].head.hflag &= !BM_ELEM_TAG;
                l = bm.loo(l).next;
                if l == l_first {
                    break;
                }
            }
        }
    }
    mesh_calc_path_region_elem(bm, ele_src, ele_dst, BM_FACE)
}
