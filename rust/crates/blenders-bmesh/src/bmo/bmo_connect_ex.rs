//! Port of `bmo_connect_concave.cc` + `bmo_connect_nonplanar.cc`.

use crate::operators::*;
use crate::{BMesh, BM_NIL, BM_EDGE, BM_FACE, BM_VERT};
use blenders_math::vector::Vec3;

const EDGE_OUT: u8 = 1;
const FACE_OUT: u8 = 2;

/* `bmo_connect_concave.cc` */

/// `bm_face_split_by_concave`.
fn bm_face_split_by_concave(bm: &mut BMesh, f_base: usize, eps: f32) -> bool {
    let normal = bm.face(f_base).no;
    let (faces_array, mut edges_array, mut faces_double) =
        crate::polygon::bm_face_triangulate(bm, f_base, 0, 0, false);
    for &f in &faces_array {
        bmo_elem_flag_enable(bm, (BM_FACE, f), FACE_OUT);
    }
    bmo_elem_flag_enable(bm, (BM_FACE, f_base), FACE_OUT);
    edges_array.sort_by(|&a, &b| {
        let ca = (bm.vert(bm.edge(a).v1).head.hflag & bm.vert(bm.edge(a).v2).head.hflag
            & crate::BM_ELEM_TAG) != 0;
        let cb = (bm.vert(bm.edge(b).v1).head.hflag & bm.vert(bm.edge(b).v2).head.hflag
            & crate::BM_ELEM_TAG) != 0;
        if ca != cb {
            return cb.cmp(&ca); /* non-concave first? C: e_a_concave < e_b_concave sorts false first */
        }
        let la = crate::query::bm_edge_calc_length_squared(bm, a);
        let lb = crate::query::bm_edge_calc_length_squared(bm, b);
        lb.partial_cmp(&la).unwrap() /* longest first */
    });
    for &e in &edges_array {
        bmo_elem_flag_enable(bm, (BM_EDGE, e), EDGE_OUT);
        if crate::query::bm_edge_is_contiguous(bm, e) {
            if let Some((l0, l1)) = crate::query::bm_edge_loop_pair(bm, e) {
                let mut ok = true;
                for &l in &[l0, l1] {
                    let p1 = bm.vert(bm.loo(l).v).co;
                    let l_rn = bm.loo(bm.loo(bm.loo(l).radial_next).next).next;
                    let p2 = bm.vert(bm.loo(l_rn).v).co;
                    let p3 = bm.vert(bm.loo(bm.loo(l).prev).v).co;
                    let cross = (p2 - p1).cross(p3 - p1);
                    if cross.dot(normal) <= eps {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    let (f_a, f_b) = (bm.loo(l0).f, bm.loo(l1).f);
                    let mut f_double = BM_NIL;
                    let f_new = crate::core::bm_faces_join(
                        bm, &[f_a, f_b], true, Some(&mut f_double));
                    if f_double != BM_NIL {
                        faces_double.push(f_new);
                    } else if f_new != BM_NIL {
                        bmo_elem_flag_enable(bm, (BM_FACE, f_new), FACE_OUT);
                    }
                }
            }
        }
    }
    for fd in faces_double {
        crate::core::bm_face_kill(bm, fd);
    }
    true
}

/// `bm_face_convex_tag_verts`.
fn bm_face_convex_tag_verts(bm: &mut BMesh, f: usize) -> bool {
    let mut is_concave = false;
    if bm.face(f).len > 3 {
        for l in crate::iterators::LoopsOfFace::new(bm, f).collect::<Vec<_>>() {
            let v = bm.loo(l).v;
            if !crate::query::bm_loop_is_convex(bm, l) {
                is_concave = true;
                bm.vpool[v].head.hflag |= crate::BM_ELEM_TAG;
            } else {
                bm.vpool[v].head.hflag &= !crate::BM_ELEM_TAG;
            }
        }
    }
    is_concave
}

/// `bmo_connect_verts_concave_exec`.
pub fn bmo_connect_verts_concave_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let mut changed = false;
    let faces: Vec<usize> = bmo_slot_buffer(op, false, "faces")
        .iter().filter(|(ht, _)| *ht == BM_FACE).map(|&(_, i)| i).collect();
    for f in faces {
        if bm.face(f).len > 3 && bm_face_convex_tag_verts(bm, f) {
            if bm_face_split_by_concave(bm, f, f32::EPSILON) {
                changed = true;
            }
        }
    }
    if changed {
        bmo_slot_buffer_from_enabled_flag(bm, op, true, "edges.out", BM_EDGE, EDGE_OUT);
        bmo_slot_buffer_from_enabled_flag(bm, op, true, "faces.out", BM_FACE, FACE_OUT);
    }
}

/* `bmo_connect_nonplanar.cc` */

/// `bm_face_split_find`.
fn bm_face_split_find(bm: &mut BMesh, f: usize) -> Option<(usize, usize, f32)> {
    let f_len = bm.face(f).len as usize;
    let l_arr = crate::iterators::LoopsOfFace::new(bm, f).collect::<Vec<_>>();
    let mut err_best = f32::MAX;
    let mut found: Option<(usize, usize, f32)> = None;
    for i_a in 0..f_len {
        for i_b in i_a + 2..f_len {
            let (l_a, l_b) = (l_arr[i_a], l_arr[i_b]);
            if !crate::mods::bm_loop_is_adjacent(bm, l_a, l_b) {
                let no_a = crate::polygon::bm_face_calc_normal_subset(bm, l_a, l_b);
                let no_b = crate::polygon::bm_face_calc_normal_subset(bm, l_b, l_a);
                if no_a.length() != 0.0 && no_b.length() != 0.0 {
                    let err = crate::polygon::bm_face_subset_calc_planar(bm, l_a, l_b, no_a)
                        + crate::polygon::bm_face_subset_calc_planar(bm, l_b, l_a, no_b);
                    if err < err_best {
                        let mut splits = [(l_a, l_b)];
                        crate::polygon::bm_face_splits_check_legal(bm, f, &mut splits);
                        if splits[0] != (BM_NIL, BM_NIL) {
                            err_best = err;
                            found = Some((l_a, l_b, no_a.dot(no_b)));
                        }
                    }
                }
            }
        }
    }
    found
}

/// `bm_face_split_by_angle`.
fn bm_face_split_by_angle(
    bm: &mut BMesh, f: usize, angle_limit_cos: f32) -> Option<(usize, usize)>
{
    if let Some((l_a, l_b, angle_cos)) = bm_face_split_find(bm, f) {
        if angle_cos < angle_limit_cos {
            let mut l_new = BM_NIL;
            let f_new = crate::mods::bm_face_split(
                bm, f, l_a, l_b, Some(&mut l_new), None, false);
            if f_new != BM_NIL {
                bmo_elem_flag_enable(bm, (BM_FACE, f), FACE_OUT);
                bmo_elem_flag_enable(bm, (BM_FACE, f_new), FACE_OUT);
                bmo_elem_flag_enable(bm, (BM_EDGE, bm.loo(l_new).e), EDGE_OUT);
                return Some((f, f_new));
            }
        }
    }
    None
}

/// `bmo_connect_verts_nonplanar_exec`.
pub fn bmo_connect_verts_nonplanar_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let mut changed = false;
    let angle_limit_cos = bmo_slot_float_get(op, false, "angle_limit").cos();
    let mut fstack: Vec<usize> = bmo_slot_buffer(op, false, "faces")
        .iter().filter(|(ht, _)| *ht == BM_FACE).map(|&(_, i)| i)
        .filter(|&f| bm.face(f).len > 3).collect();
    while let Some(f) = fstack.pop() {
        if let Some((f0, f1)) = bm_face_split_by_angle(bm, f, angle_limit_cos) {
            for fp in [f0, f1] {
                crate::normals::bm_face_calc_normals(bm, fp);
                if bm.face(fp).len > 3 {
                    fstack.push(fp);
                }
            }
            changed = true;
        }
    }
    if changed {
        bmo_slot_buffer_from_enabled_flag(bm, op, true, "edges.out", BM_EDGE, EDGE_OUT);
        bmo_slot_buffer_from_enabled_flag(bm, op, true, "faces.out", BM_FACE, FACE_OUT);
    }
}
