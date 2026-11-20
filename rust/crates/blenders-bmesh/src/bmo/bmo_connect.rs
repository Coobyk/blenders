//! Port of `bmo_connect.cc` — `connect_verts` op.

use crate::operators::*;
use crate::{BMesh, BM_NIL, BM_EDGE, BM_FACE, BM_VERT};

const VERT_INPUT: u8 = 1;
const EDGE_OUT: u8 = 1;
const EDGE_OUT_ADJ: u8 = 2;
const FACE_TAG: u8 = 2;
const FACE_EXCLUDE: u8 = 4;

/// `bm_face_connect_verts` — fan of non-adjacent splits between INPUT verts.
/// Returns 0 (nothing), 1 (ok), -1 (error).
fn bm_face_connect_verts(bm: &mut BMesh, f: usize, check_degenerate: bool) -> i32 {
    let pair_split_max = (bm.face(f).len / 2) as usize;
    let mut loops_split: Vec<[usize; 2]> = Vec::with_capacity(pair_split_max);
    let mut l_tag_prev = BM_NIL;
    let mut l_tag_first = BM_NIL;
    let mut result = 1;

    let l_first = bm.face(f).l_first;
    let mut l_iter = l_first;
    loop {
        let lv = bm.loo(l_iter).v;
        if bmo_elem_flag_test(bm, (BM_VERT, lv), VERT_INPUT)
            && (!bmo_elem_flag_test(bm, (BM_VERT, bm.loo(bm.loo(l_iter).prev).v), VERT_INPUT)
                || !bmo_elem_flag_test(bm, (BM_VERT, bm.loo(bm.loo(l_iter).next).v), VERT_INPUT))
        {
            if l_tag_prev == BM_NIL {
                l_tag_prev = l_iter;
                l_tag_first = l_iter;
                l_iter = bm.loo(l_iter).next;
                if l_iter == l_first { break; }
                continue;
            }
            if !crate::mods::bm_loop_is_adjacent(bm, l_tag_prev, l_iter) {
                let e = crate::query::bm_edge_exists(bm, bm.loo(l_tag_prev).v, lv);
                if e == BM_NIL || !bmo_elem_flag_test(bm, (BM_EDGE, e), EDGE_OUT) {
                    loops_split.push([l_tag_prev, l_iter]);
                }
            }
            l_tag_prev = l_iter;
        }
        l_iter = bm.loo(l_iter).next;
        if l_iter == l_first { break; }
    }
    if loops_split.is_empty() {
        return 0;
    }
    if !crate::mods::bm_loop_is_adjacent(bm, l_tag_first, l_tag_prev)
        && !(loops_split[0][0] == l_tag_first && loops_split[0][1] == l_tag_prev)
    {
        loops_split.push([l_tag_first, l_tag_prev]);
    }
    let mut pair_t: Vec<(usize, usize)> = loops_split.iter().map(|p| (p[0], p[1])).collect();
    if check_degenerate {
        crate::polygon::bm_face_splits_check_legal(bm, f, &mut pair_t);
    } else {
        crate::polygon::bm_face_splits_check_optimal(bm, f, &mut pair_t);
    }
    let mut verts_pair: Vec<[usize; 2]> = Vec::with_capacity(pair_t.len());
    for &(a, b) in &pair_t {
        if a != BM_NIL {
            verts_pair.push([bm.loo(a).v, bm.loo(b).v]);
        }
    }
    let mut deferred_remove: Vec<usize> = Vec::new();
    let mut f_cur = f;
    for vp in &verts_pair {
        let edge_exists = crate::query::bm_edge_exists(bm, vp[0], vp[1]) != BM_NIL;
        let l_a = crate::query::bm_face_vert_share_loop(bm, f_cur, vp[0]);
        let l_b = crate::query::bm_face_vert_share_loop(bm, f_cur, vp[1]);
        let mut l_new = BM_NIL;
        let mut f_new = BM_NIL;
        if l_a != BM_NIL && l_b != BM_NIL {
            f_new = crate::mods::bm_face_split(bm, f_cur, l_a, l_b, Some(&mut l_new), edge_exists);
            if edge_exists {
                for l in [l_a, l_b] {
                    if crate::query::bm_face_find_double(bm, bm.loo(l).f) != BM_NIL {
                        deferred_remove.push(l);
                    }
                }
            }
        }
        if l_new == BM_NIL || f_new == BM_NIL {
            result = -1;
            break;
        }
        f_cur = f_new;
        bmo_elem_flag_enable(bm, (BM_EDGE, bm.loo(l_new).e), EDGE_OUT);
    }
    for l in deferred_remove {
        crate::core::bm_face_kill(bm, bm.loo(l).f);
    }
    result
}

/// `bmo_connect_verts_exec`.
pub fn bmo_connect_verts_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let check_degenerate = bmo_slot_bool_get(op, false, "check_degenerate");
    let mut faces: Vec<usize> = Vec::new();
    bmo_slot_buffer_flag_enable(bm, op, false, "faces_exclude", FACE_EXCLUDE);
    let verts: Vec<usize> = bmo_slot_buffer(op, false, "verts")
        .iter().filter(|(ht, _)| *ht == BM_VERT).map(|&(_, i)| i).collect();
    for v in verts {
        bmo_elem_flag_enable(bm, (BM_VERT, v), VERT_INPUT);
        for l_iter in crate::iterators::LoopsOfVert::new(bm, v).collect::<Vec<_>>() {
            let f = bm.loo(l_iter).f;
            if !bmo_elem_flag_test(bm, (BM_FACE, f), FACE_EXCLUDE)
                && !bmo_elem_flag_test(bm, (BM_FACE, f), FACE_TAG)
            {
                bmo_elem_flag_enable(bm, (BM_FACE, f), FACE_TAG);
                if bm.face(f).len > 3 {
                    faces.push(f);
                }
            }
            let (lp, ln) = (bm.loo(l_iter).prev, bm.loo(l_iter).next);
            if bmo_elem_flag_test(bm, (BM_VERT, bm.loo(lp).v), VERT_INPUT) {
                bmo_elem_flag_enable(bm, (BM_EDGE, bm.loo(lp).e), EDGE_OUT_ADJ);
            }
            if bmo_elem_flag_test(bm, (BM_VERT, bm.loo(ln).v), VERT_INPUT) {
                bmo_elem_flag_enable(bm, (BM_EDGE, bm.loo(l_iter).e), EDGE_OUT_ADJ);
            }
        }
    }
    let mut err = 0;
    while let Some(f) = faces.pop() {
        if bm_face_connect_verts(bm, f, check_degenerate) == -1 {
            err = -1; /* BMO_ERROR_FATAL "Could not connect vertices" */
        }
    }
    bmo_slot_buffer_from_enabled_flag(
        bm, op, true, "edges.out", 1 << BM_EDGE, EDGE_OUT | EDGE_OUT_ADJ);
    let _ = err; /* error reporting PORT_PENDING */
}
