//! Port of `bmo_rotate_edges.cc`.

use crate::operators::*;
use crate::{BMesh, BM_NIL, BM_EDGE, BM_FACE};
use crate::mods::{
    bm_edge_rotate, bm_edge_rotate_check, BM_EDGEROT_CHECK_DEGENERATE, BM_EDGEROT_CHECK_EXISTS,
};
use blenders_bli::heap::Heap;

const EDGE_OUT: u8 = 1;
const FACE_MARK: u8 = 1;

/// `bm_rotate_edges_simple`.
fn bm_rotate_edges_simple(bm: &mut BMesh, op: &mut BMOperator, check_flag: i16, use_ccw: bool) {
    let edges: Vec<usize> = bmo_slot_buffer(op, false, "edges")
        .iter().filter(|(ht, _)| *ht == BM_EDGE).map(|&(_, i)| i).collect();
    for e in edges {
        if bm_edge_rotate_check(bm, e) {
            let e_rotate = bm_edge_rotate(bm, e, use_ccw, check_flag);
            if e_rotate != BM_NIL {
                bmo_elem_flag_enable(bm, (BM_EDGE, e_rotate), EDGE_OUT);
            }
        }
    }
}

/// `bm_edge_calc_rotate_cost`.
fn bm_edge_calc_rotate_cost(bm: &BMesh, e: usize) -> f32 {
    -crate::query::bm_edge_calc_length_squared(bm, e)
}

/// `bm_edge_rotate_is_boundary` — `edge_set` is the input-edge side-table
/// (replaces C's `BM_elem_index_get/set` abuse).
fn bm_edge_rotate_is_boundary(bm: &BMesh, e: usize, edge_set: &std::collections::HashMap<usize, usize>) -> bool {
    let mut count = 0;
    let mut l_radial_iter = bm.edge(e).l;
    let l_first = l_radial_iter;
    loop {
        let mut l_iter = bm.loo(l_radial_iter).next;
        while l_iter != l_radial_iter {
            let e_iter = bm.loo(l_iter).e;
            if edge_set.contains_key(&e_iter) {
                if count == 1 { return false; }
                count += 1;
                break;
            }
            l_iter = bm.loo(l_iter).next;
        }
        l_radial_iter = bm.loo(l_radial_iter).radial_next;
        if l_radial_iter == l_first { break; }
    }
    true
}

/// `bm_rotate_edges_shared`.
fn bm_rotate_edges_shared(
    bm: &mut BMesh, op: &mut BMOperator, check_flag: i16, use_ccw: bool, edges_len: usize)
{
    let mut heap: Heap<usize> = Heap::with_capacity(edges_len);
    let edges: Vec<usize> = bmo_slot_buffer(op, false, "edges")
        .iter().map(|&(_, i)| i).collect();
    let mut edges_len_rotate = 0usize;
    /* edge -> input index (replaces elem_index abuse), only manifold ones */
    let mut edge_set: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut eheap_table: Vec<Option<blenders_bli::heap::HeapNode>> = vec![None; edges_len];
    let mut freed = vec![false; edges_len];
    for (i, &e) in edges.iter().enumerate() {
        if crate::query::bm_edge_is_manifold(bm, e) {
            edge_set.insert(e, i);
        }
    }
    const PASS_BOUNDARY: u8 = 0;
    const PASS_ALL: u8 = 1;
    const PASS_DONE: u8 = 2;
    let mut pass_type = PASS_BOUNDARY;
    while pass_type != PASS_DONE && edges_len_rotate != edges_len {
        debug_assert!(heap.is_empty());
        for i in 0..edges_len {
            if freed[i] { continue; }
            let e = edges[i];
            let mut ok = edge_set.contains_key(&e) && bm_edge_rotate_check(bm, e);
            if ok && pass_type == PASS_BOUNDARY {
                ok = bm_edge_rotate_is_boundary(bm, e, &edge_set);
            }
            if ok {
                let mut cost = bm_edge_calc_rotate_cost(bm, e);
                if pass_type == PASS_BOUNDARY {
                    cost = if cost != 0.0 { -1.0 / cost } else { f32::MAX };
                }
                eheap_table[i] = Some(heap.insert(cost, e));
            }
        }
        if heap.is_empty() {
            pass_type += 1;
            continue;
        }
        let edges_len_rotate_prev = edges_len_rotate;
        while let Some(e_best) = heap.pop_min() {
            let e_best_index = edge_set[&e_best];
            eheap_table[e_best_index] = None;
            if bm_edge_rotate_check(bm, e_best) {
                let e_rotate = bm_edge_rotate(bm, e_best, use_ccw, check_flag);
                if e_rotate != BM_NIL {
                    bmo_elem_flag_enable(bm, (BM_EDGE, e_rotate), EDGE_OUT);
                    edge_set.remove(&e_rotate);
                    freed[e_best_index] = true;
                    edges_len_rotate += 1;
                    let mut l_radial_iter = bm.edge(e_rotate).l;
                    let l_first = l_radial_iter;
                    loop {
                        let mut l_iter = bm.loo(l_radial_iter).next;
                        while l_iter != l_radial_iter {
                            let e_iter = bm.loo(l_iter).e;
                            if let Some(&e_iter_index) = edge_set.get(&e_iter) {
                                if eheap_table[e_iter_index].is_none() && !freed[e_iter_index]
                                    && bm_edge_rotate_check(bm, e_iter)
                                {
                                    let cost = bm_edge_calc_rotate_cost(bm, e_iter);
                                    eheap_table[e_iter_index] = Some(heap.insert(cost, e_iter));
                                }
                            }
                            l_iter = bm.loo(l_iter).next;
                        }
                        l_radial_iter = bm.loo(l_radial_iter).radial_next;
                        if l_radial_iter == l_first { break; }
                    }
                }
            }
        }
        if edges_len_rotate == edges_len_rotate_prev {
            pass_type += 1;
        }
    }
}

/// `bmo_rotate_edges_exec`.
pub fn bmo_rotate_edges_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let edges_len = bmo_slot_buffer(op, false, "edges").len();
    let use_ccw = bmo_slot_bool_get(op, false, "use_ccw");
    let is_single = edges_len == 1;
    let check_flag = if is_single {
        BM_EDGEROT_CHECK_EXISTS
    } else {
        BM_EDGEROT_CHECK_EXISTS | BM_EDGEROT_CHECK_DEGENERATE
    };
    let mut is_simple = true;
    if !is_single {
        let edges: Vec<usize> = bmo_slot_buffer(op, false, "edges")
            .iter().filter(|(ht, _)| *ht == BM_EDGE).map(|&(_, i)| i).collect();
        'outer: for e in edges {
            if let Some((f0, f1)) = crate::query::bm_edge_face_pair(bm, e) {
                for f in [f0, f1] {
                    if bmo_elem_flag_test(bm, (BM_FACE, f), FACE_MARK) {
                        is_simple = false;
                        break;
                    }
                    bmo_elem_flag_enable(bm, (BM_FACE, f), FACE_MARK);
                }
                if !is_simple { break 'outer; }
            }
        }
    }
    if is_simple {
        bm_rotate_edges_simple(bm, op, check_flag, use_ccw);
    } else {
        bm_rotate_edges_shared(bm, op, check_flag, use_ccw, edges_len);
    }
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "edges.out", BM_EDGE, EDGE_OUT);
}
