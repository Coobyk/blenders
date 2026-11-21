//! Port of `bmo_bridge.cc` — `bmo_bridge_loops_exec`.

use std::collections::VecDeque;

use crate::edgeloop::*;
use crate::iterators::*;
use crate::operators::*;
use crate::{BMesh, BM_NIL};
use crate::{BM_EDGE, BM_FACE, BM_VERT};
use crate::BM_ELEM_TAG;
use blenders_math::vector::Vec3;

const EDGE_MARK: u8 = 4;
const EDGE_OUT: u8 = 8;
const FACE_OUT: u8 = 16;

fn bm_bridge_splice_loops(
    bm: &mut BMesh,
    verts_a: &VecDeque<usize>,
    verts_b: &VecDeque<usize>,
    op_flag: i32,
    merge_factor: f32,
) {
    let mut op_weld = bmo_op_init(bm, &crate::opdefines::OPDEF_WELD_VERTS, op_flag);
    for (i, (&v_a, &v_b)) in verts_a.iter().zip(verts_b.iter()).enumerate() {
        let _ = i;
        crate::interp::bm_data_interp_from_verts(bm, v_a, v_b, v_b, merge_factor);
        bm.vpool[v_b].co = bm.vpool[v_a].co.lerp(bm.vpool[v_b].co, merge_factor);
        debug_assert!(v_a != v_b);
        bmo_slot_map_insert(&mut op_weld, false, "targetmap",
                            (BM_VERT, v_a), (BM_VERT, v_b));
    }
    bmo_op_exec(bm, &mut op_weld, crate::bmo::bmo_removedoubles::bmo_weld_verts_exec);
    bmo_op_finish(bm, op_weld);
}

fn bm_vert_loop_pair(bm: &BMesh, v1: usize, v2: usize) -> (usize, usize) {
    let e = crate::query::bm_edge_exists(bm, v1, v2);
    if e == BM_NIL {
        let l1 = LoopsOfVert::new(bm, v1).next().unwrap_or(BM_NIL);
        let l2 = LoopsOfVert::new(bm, v2).next().unwrap_or(BM_NIL);
        return (l1, l2);
    }
    let l = bm.edge(e).l;
    if l == BM_NIL {
        let l1 = LoopsOfVert::new(bm, v1).next().unwrap_or(BM_NIL);
        let l2 = LoopsOfVert::new(bm, v2).next().unwrap_or(BM_NIL);
        (l1, l2)
    } else if bm.loo(l).v == v1 {
        (l, bm.loo(l).next)
    } else {
        (bm.loo(l).next, l)
    }
}

/// `bm_edgeloop_offset_length` — total paired distance for a b-rotation.
fn bm_edgeloop_offset_length(
    bm: &BMesh,
    va: &VecDeque<usize>,
    vb: &VecDeque<usize>,
    b_start: usize,
    len_max: f32,
) -> f32 {
    let mut len = 0.0;
    let mut ib = b_start;
    for &a in va.iter() {
        len += bm.vpool[a].co.distance(bm.vpool[vb[ib]].co);
        if len >= len_max {
            break;
        }
        ib = (ib + 1) % vb.len();
    }
    len
}

/// `bm_bridge_best_rotation` — rotate `vb` so the pair alignment is shortest.
fn bm_bridge_best_rotation(bm: &BMesh, el_a: &BMEdgeLoopStore, el_b: &mut BMEdgeLoopStore) {
    let va = bm_edgeloop_verts_get(el_a).clone();
    let vb_len = bm_edgeloop_verts_get(el_b).len();
    let mut best = 0usize;
    let mut len_best = f32::MAX;
    for ib in 0..vb_len {
        let len = bm_edgeloop_offset_length(
            bm, &va, bm_edgeloop_verts_get(el_b), ib, len_best);
        if len < len_best {
            best = ib;
            len_best = len;
        }
    }
    if best != 0 {
        bm_edgeloop_verts_get_mut(el_b).rotate_left(best);
    }
}

fn bm_face_edges_tag_out(bm: &mut BMesh, f: usize) {
    for l in LoopsOfFace::new(bm, f).collect::<Vec<_>>() {
        bmo_elem_flag_enable(bm, (BM_EDGE, bm.loo(l).e), EDGE_OUT);
    }
}

fn bridge_loop_pair(
    bm: &mut BMesh,
    el_store_a: &mut BMEdgeLoopStore,
    el_store_b: &mut BMEdgeLoopStore,
    use_merge: bool,
    merge_factor: f32,
    twist_offset: i32,
    op_flag: i32,
) {
    let eps = 0.00001f32;
    let is_closed =
        bm_edgeloop_is_closed(el_store_a) && bm_edgeloop_is_closed(el_store_b);
    let mut el_store_b_copy: Option<BMEdgeLoopStore> = None;

    let len_a = bm_edgeloop_length_get(el_store_a);
    let len_b = bm_edgeloop_length_get(el_store_b);

    let el_dir = bm_edgeloop_center_get(el_store_a) - bm_edgeloop_center_get(el_store_b);

    if is_closed {
        if bm.totface > 0 {
            crate::marking::bm_mesh_elem_hflag_disable_all(
                bm, BM_FACE | BM_EDGE, BM_ELEM_TAG);
        }
        bm_edgeloop_calc_normal(bm, el_store_a);
        bm_edgeloop_calc_normal(bm, el_store_b);
        if len_a != len_b {
            /* angle from the normals of a shared face-region — simplified:
             * normals already computed; nothing else needed. */
        }
    } else {
        bm_edgeloop_calc_normal(bm, el_store_a);
        bm_edgeloop_calc_normal(bm, el_store_b);
    }

    let dot_a = bm_edgeloop_normal_get(el_store_a).dot(el_dir);
    let dot_b = bm_edgeloop_normal_get(el_store_b).dot(el_dir);
    if dot_a.abs() < eps && dot_b.abs() < eps {
        if bm_edgeloop_normal_get(el_store_a)
            .dot(bm_edgeloop_normal_get(el_store_b)) < 0.0
        {
            bm_edgeloop_flip(el_store_b);
        }
    } else if dot_b.abs() < eps && dot_a > 0.0 {
        bm_edgeloop_flip(el_store_b);
    } else if dot_a.abs() < eps && dot_b < 0.0 {
        bm_edgeloop_flip(el_store_b);
    } else if dot_a > 0.0 && dot_b > 0.0 {
        bm_edgeloop_flip(el_store_b);
    } else if dot_a < 0.0 && dot_b < 0.0 {
        bm_edgeloop_flip(el_store_b);
    }

    if !use_merge {
        let no = bm_edgeloop_normal_get(el_store_a) + bm_edgeloop_normal_get(el_store_b);
        if no.dot(el_dir) < 0.0 {
            bm_edgeloop_flip(el_store_a);
            bm_edgeloop_flip(el_store_b);
        }
        if bm.totface > 0 {
            /* winding votes: adjust loop direction to match existing faces. */
            let mut winding_votes = [0i32; 2];
            let mut winding_dir = 1i32;
            for i in 0..2 {
                let verts = bm_edgeloop_verts_get(if i == 0 {
                    &*el_store_a
                } else {
                    &*el_store_b
                })
                .clone();
                for j in 0..verts.len() {
                    let k = (j + 1) % verts.len();
                    if !is_closed && k == 0 {
                        break;
                    }
                    let (v_cur, v_next) = (verts[j], verts[k]);
                    let e = crate::query::bm_edge_exists(bm, v_cur, v_next);
                    if e != BM_NIL && crate::delete::bm_edge_is_boundary(bm, e) {
                        winding_votes[i] += if bm.loo(bm.edge(e).l).v == v_cur {
                            winding_dir
                        } else {
                            -winding_dir
                        };
                    }
                }
                winding_dir = -winding_dir;
            }
            if winding_votes[0] != 0 || winding_votes[1] != 0 {
                let mut flip = [false; 2];
                if dot_a.abs() < eps && winding_votes[0] < 0 {
                    flip[0] = !flip[0];
                    winding_votes[0] *= -1;
                }
                if dot_b.abs() < eps && winding_votes[1] < 0 {
                    flip[1] = !flip[1];
                    winding_votes[1] *= -1;
                }
                if winding_votes[0] + winding_votes[1] < 0 {
                    flip[0] = !flip[0];
                    flip[1] = !flip[1];
                }
                if flip[0] {
                    bm_edgeloop_flip(el_store_a);
                }
                if flip[1] {
                    bm_edgeloop_flip(el_store_b);
                }
            }
        }
    }

    if len_a > len_b {
        let mut el_b_copy = bm_edgeloop_copy(el_store_b);
        let mut split_edges = std::collections::HashSet::new();
        bm_edgeloop_expand(bm, &mut el_b_copy, len_a, false, &mut split_edges);
        el_store_b_copy = Some(el_b_copy);
    }
    let el_store_b_ref = el_store_b_copy.as_mut().unwrap_or(&mut *el_store_b);

    if is_closed {
        bm_bridge_best_rotation(bm, el_store_a, el_store_b_ref);
        if twist_offset != 0 {
            let len = bm_edgeloop_verts_get(el_store_b_ref).len() as i32;
            let idx = twist_offset.rem_euclid(len) as usize;
            bm_edgeloop_verts_get_mut(el_store_b_ref).rotate_left(idx);
        }
    }

    let verts_a = bm_edgeloop_verts_get(el_store_a).clone();
    let verts_b = bm_edgeloop_verts_get(el_store_b_ref).clone();

    if use_merge {
        bm_bridge_splice_loops(bm, &verts_a, &verts_b, op_flag, merge_factor);
    } else {
        let use_edgeout = true; /* EDGE_OUT always collected for `edges.out`. */
        let steps = if is_closed {
            verts_a.len()
        } else {
            verts_a.len().saturating_sub(1)
        };
        for i in 0..steps {
            let i_next = i + 1;
            let (v_a, v_b) = (verts_a[i], verts_b[i]);
            let (v_a_next, v_b_next) = (verts_a[i_next % verts_a.len()],
                                        verts_b[i_next % verts_b.len()]);

            let (mut l_a, mut l_a_next) = bm_vert_loop_pair(bm, v_a, v_a_next);
            let (mut l_b, mut l_b_next) = if v_b != v_b_next {
                bm_vert_loop_pair(bm, v_b, v_b_next)
            } else {
                let any = LoopsOfVert::new(bm, v_b).next().unwrap_or(BM_NIL);
                (any, any)
            };
            if l_a != BM_NIL && l_a_next == BM_NIL {
                l_a_next = l_a;
            }
            if l_a_next != BM_NIL && l_a == BM_NIL {
                l_a = l_a_next;
            }
            if l_b != BM_NIL && l_b_next == BM_NIL {
                l_b_next = l_b;
            }
            if l_b_next != BM_NIL && l_b == BM_NIL {
                l_b = l_b_next;
            }
            let f_example = if l_a != BM_NIL {
                bm.loo(l_a).f
            } else if l_b != BM_NIL {
                bm.loo(l_b).f
            } else {
                BM_NIL
            };

            let mut f = BM_NIL;
            if v_b != v_b_next {
                if !(v_b == v_a_next || v_b == v_a || v_b_next == v_a_next || v_b_next == v_a)
                {
                    let v_arr = [v_b, v_b_next, v_a_next, v_a];
                    f = crate::query::bm_face_exists(bm, &v_arr);
                    if f == BM_NIL {
                        f = crate::core::bm_face_create_verts(
                            bm, &v_arr, None, 0, true, true);
                        let mut l_iter = bm.face(f).l_first;
                        /* PORT_PENDING: BM_elem_attrs_copy on loops
                         * (l_b, l_b_next, l_a_next, l_a) — CustomData. */
                        let _ = &mut l_iter;
                    }
                }
            } else if !(v_b == v_a_next || v_b == v_a) {
                let v_arr = [v_b, v_a_next, v_a];
                f = crate::query::bm_face_exists(bm, &v_arr);
                if f == BM_NIL {
                    f = crate::core::bm_face_create_verts(bm, &v_arr, None, 0, true, true);
                    /* PORT_PENDING: BM_elem_attrs_copy on loops. */
                }
            }

            if f != BM_NIL {
                if f_example != BM_NIL && f_example != f {
                    /* PORT_PENDING: BM_elem_attrs_copy(f_example, f). */
                }
                bmo_elem_flag_enable(bm, (BM_FACE, f), FACE_OUT);
                bm.fpool[f].head.hflag |= BM_ELEM_TAG;
                if use_edgeout {
                    bm_face_edges_tag_out(bm, f);
                }
            }
        }
    }

    if len_a != len_b {
        /* `triangulate` sub-op on TAG'ed faces. */
        let faces: Vec<usize> = FacesOfMesh::new(bm)
            .filter(|&f| bm.fpool[f].head.hflag & BM_ELEM_TAG != 0)
            .collect();
        let mut triangulated: Vec<usize> = Vec::new();
        for f in faces {
            crate::normals::bm_face_calc_normals(bm, f);
            let (f_new, _e_new, _f_dbl) =
                crate::polygon::bm_face_triangulate(bm, f, 0, 0, false);
            for nf in f_new {
                bmo_elem_flag_enable(bm, (BM_FACE, nf), FACE_OUT);
                bm.fpool[nf].head.hflag |= BM_ELEM_TAG;
                triangulated.push(nf);
            }
        }
        /* `beautify_fill` on the same faces (restrict to tagged verts). */
        for (i, store) in [&*el_store_a, &*el_store_b_ref].iter().enumerate() {
            let verts = bm_edgeloop_verts_get(store).clone();
            for v in verts {
                if i == 0 {
                    bm.vpool[v].head.hflag |= BM_ELEM_TAG;
                } else {
                    bm.vpool[v].head.hflag &= !BM_ELEM_TAG;
                }
            }
        }
        /* `beautify_fill faces=%hf edges=ae use_restrict_tag=%b method=1`:
         * edges = all edges of tagged faces. */
        let mut edges_b: Vec<usize> = Vec::new();
        let mut seen_e = std::collections::HashSet::new();
        for f in FacesOfMesh::new(bm).collect::<Vec<_>>() {
            if bm.fpool[f].head.hflag & BM_ELEM_TAG != 0 {
                for l in LoopsOfFace::new(bm, f).collect::<Vec<_>>() {
                    if seen_e.insert(bm.loo(l).e) {
                        edges_b.push(bm.loo(l).e);
                    }
                }
            }
        }
        crate::tools::bmesh_beautify::bm_mesh_beautify_fill(
            bm, &mut edges_b,
            crate::tools::bmesh_beautify::VERT_RESTRICT_TAG, 1, 0, FACE_OUT);
        for f in FacesOfMesh::new(bm).collect::<Vec<_>>() {
            if bm.fpool[f].head.hflag & BM_ELEM_TAG != 0 {
                bmo_elem_flag_enable(bm, (BM_FACE, f), FACE_OUT);
                bm_face_edges_tag_out(bm, f);
            }
        }
    }

    if use_merge == false {
        for store in [&*el_store_a, &*el_store_b_ref] {
            let verts = bm_edgeloop_verts_get(store);
            for j in 0..verts.len() {
                let k = j + 1;
                if k >= verts.len() {
                    break;
                }
                let (v_cur, v_next) = (verts[j], verts[k]);
                if v_cur != v_next {
                    let e = crate::query::bm_edge_exists(bm, v_cur, v_next);
                    if e != BM_NIL {
                        bm.epool[e].head.api_flag &= !EDGE_OUT;
                    }
                }
            }
        }
    }
}

/// `bmo_bridge_loops_exec`.
pub fn bmo_bridge_loops_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let use_pairs = bmo_slot_bool_get(op, false, "use_pairs");
    let use_merge = bmo_slot_bool_get(op, false, "use_merge");
    let merge_factor = bmo_slot_float_get(op, false, "merge_factor");
    let use_cyclic = bmo_slot_bool_get(op, false, "use_cyclic") && !use_merge;
    let twist_offset = bmo_slot_int_get(op, false, "twist_offset");

    bmo_slot_buffer_flag_enable(bm, op, false, "edges", EDGE_MARK);
    let mut eloops = bm_mesh_edgeloops_find(
        bm,
        &|bm: &BMesh, e| bmo_elem_flag_test(bm, (BM_EDGE, e), EDGE_MARK),
        None,
    );
    for el in &mut eloops {
        bm_edgeloop_calc_center(bm, el);
    }
    let count = eloops.len();

    if count < 2 {
        /* BMO_ERROR_CANCEL: "Select at least two edge loops" */
        return;
    }
    if use_pairs && count % 2 != 0 {
        /* BMO_ERROR_CANCEL: "Select an even number of loops to bridge pairs" */
        return;
    }
    if use_merge {
        let eloop_len = bm_edgeloop_length_get(&eloops[0]);
        if eloops.iter().any(|el| bm_edgeloop_length_get(el) != eloop_len) {
            /* BMO_ERROR_CANCEL: "Selected loops must have equal edge counts" */
            return;
        }
    }
    if count > 2 {
        if use_pairs {
            for el in &mut eloops {
                bm_edgeloop_calc_normal(bm, el);
            }
        }
        bm_mesh_edgeloops_calc_order(&mut eloops, use_pairs);
    }

    let mut changed = false;
    let mut i = 0usize;
    while i < eloops.len() {
        let next = if i + 1 < eloops.len() {
            i + 1
        } else if use_cyclic && count > 2 {
            0
        } else {
            break;
        };
        let (a, b) = if i < next {
            let (lo, hi) = eloops.split_at_mut(next);
            (&mut lo[i], &mut hi[0])
        } else {
            let (lo, hi) = eloops.split_at_mut(i);
            (&mut hi[0], &mut lo[next])
        };
        bridge_loop_pair(bm, a, b, use_merge, merge_factor, twist_offset, op.flag);
        if use_pairs {
            i += 1;
        }
        i += 1;
        changed = true;
    }

    if changed && !use_merge {
        bmo_slot_buffer_from_enabled_flag(bm, op, true, "faces.out", BM_FACE, FACE_OUT);
        bmo_slot_buffer_from_enabled_flag(bm, op, true, "edges.out", BM_EDGE, EDGE_OUT);
    }
}
