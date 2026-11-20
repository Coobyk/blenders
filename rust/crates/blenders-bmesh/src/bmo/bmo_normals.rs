//! Port of `bmo_normals.cc` — normal-recalculation operator
//! (`bmo_recalc_face_normals_exec`); `bmo_smooth*` remain in later files.
use crate::iterators::LoopsOfFace;
use crate::{BMesh, BM_FACE};
use crate::operators::BMOperator;
use crate::operators::*;

const FACE_FLAG: u8 = 0;
const FACE_FLIP: u8 = 1;
const FACE_TEMP: u8 = 2;

#[inline]
fn face_flag(bm: &BMesh, f: usize, bit: u8) -> bool {
    bm.face(f).head.api_flag & (1 << bit) != 0
}
#[inline]
fn face_flag_set(bm: &mut BMesh, f: usize, bit: u8, v: bool) {
    bm.fpool[f].head.api_flag &= !(1 << bit);
    if v { bm.fpool[f].head.api_flag |= 1 << bit; }
}

/// `bmo_recalc_normal_loop_filter_cb` — manifold edges only.
fn loop_filter_cb(bm: &BMesh, l: usize) -> bool {
    crate::query::bm_edge_is_manifold(bm, bm.loo(l).e)
}

/// `recalc_face_normals_find_index` — find the outer-most loop's face and
/// whether its winding points inward (must flip).
fn recalc_face_normals_find_index(bm: &BMesh, faces: &[usize], r_is_flip: &mut bool) -> usize {
    let eps = f32::EPSILON;
    let mut cent = crate::Vec3::ZERO;
    let mut cent_area_accum = 0.0f32;
    let faces_len = faces.len();
    let cent_fac = 1.0 / faces_len as f32;
    struct Best { dist_sq: f32, edge_dot: f32, loop_dot: f32 }
    let mut best = Best { dist_sq: eps, edge_dot: f32::MIN, loop_dot: f32::MIN };
    let mut is_flip = false;
    let mut f_start_index = 0;
    for &f in faces {
        let f_area = crate::polygon::bm_face_calc_area(bm, f);
        let f_cent = crate::polygon::bm_face_calc_center_median_weighted(bm, f);
        cent = cent + f_cent * (cent_fac * f_area);
        cent_area_accum += f_area;
    }
    if cent_area_accum != 0.0 {
        cent = cent * (1.0 / cent_area_accum);
    }
    for (i, &f) in faces.iter().enumerate() {
        for l_iter in LoopsOfFace::new(bm, f) {
            let v = bm.loo(l_iter).v;
            let dir_raw = bm.vert(v).co - cent;
            let test_dist_sq = dir_raw.length_squared();
            let is_best_dist_sq = test_dist_sq > best.dist_sq;
            if !(is_best_dist_sq || test_dist_sq == best.dist_sq) {
                continue;
            }
            let dir = dir_raw * (1.0 / test_dist_sq.sqrt());
            let e0 = bm.vert(bm.loo(bm.loo(l_iter).next).v).co - bm.vert(v).co;
            let e1 = bm.vert(bm.loo(bm.loo(l_iter).prev).v).co - bm.vert(v).co;
            let (n0, n1) = (e0.length(), e1.length());
            if !(n0 > eps && n1 > eps) { continue; }
            let edge_dir_pair = [e0 * (1.0 / n0), e1 * (1.0 / n1)];
            let test_edge_dot = dir.dot(edge_dir_pair[0]).max(dir.dot(edge_dir_pair[1]));
            let is_best_edge_dot = test_edge_dot > best.edge_dot;
            if !(is_best_dist_sq || is_best_edge_dot || test_edge_dot == best.edge_dot) {
                continue;
            }
            let loop_dir_raw = edge_dir_pair[0].cross(edge_dir_pair[1]);
            let n = loop_dir_raw.length();
            if n <= eps { continue; }
            let mut loop_dir = loop_dir_raw * (1.0 / n);
            /* non-planar concave loop guard */
            if loop_dir.dot(bm.face(f).no) < 0.0 {
                loop_dir = -loop_dir;
            }
            let loop_dir_dot = dir.dot(loop_dir);
            let test_loop_dot = loop_dir_dot.abs();
            if is_best_dist_sq || is_best_edge_dot || test_loop_dot > best.loop_dot {
                best = Best { dist_sq: test_dist_sq, edge_dot: test_edge_dot, loop_dot: test_loop_dot };
                f_start_index = i;
                is_flip = loop_dir_dot < 0.0;
            }
        }
    }
    *r_is_flip = is_flip;
    f_start_index
}

/// `bmo_recalc_face_normals_array` — set consistent winding over `faces`.
fn bmo_recalc_face_normals_array(bm: &mut BMesh, faces: &[usize], oflag: u8) {
    let oflag_flip = oflag | FACE_FLIP;
    let mut is_flip = false;
    let f_start_index = recalc_face_normals_find_index(bm, faces, &mut is_flip);
    if is_flip {
        face_flag_set(bm, faces[f_start_index], FACE_FLIP, true);
    }
    let mut fstack: Vec<usize> = vec![faces[f_start_index]];
    face_flag_set(bm, faces[f_start_index], FACE_TEMP, true);
    while let Some(f) = fstack.pop() {
        let flip_state = face_flag(bm, f, FACE_FLIP);
        let loops: Vec<usize> = LoopsOfFace::new(bm, f).collect();
        for l_iter in loops {
            let l_other = bm.loo(l_iter).radial_next;
            if l_other != l_iter && loop_filter_cb(bm, l_iter) {
                let f_other = bm.loo(l_other).f;
                if !face_flag(bm, f_other, FACE_TEMP) {
                    face_flag_set(bm, f_other, FACE_TEMP, true);
                    let same_v = bm.loo(l_other).v == bm.loo(l_iter).v;
                    face_flag_set(bm, f_other, FACE_FLIP, same_v != flip_state);
                    fstack.push(f_other);
                }
            }
        }
    }
    /* apply flipping to oflag'd faces */
    for &f in faces {
        if face_flag(bm, f, oflag) && face_flag(bm, f, FACE_FLIP) {
            crate::polygon::bm_face_normal_flip(bm, f);
        }
        face_flag_set(bm, f, FACE_TEMP, false);
        face_flag_set(bm, f, FACE_FLIP, false);
    }
}

/// `bmo_recalc_face_normals_exec`.
pub fn bmo_recalc_face_normals_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let (group_tot, group_index) = crate::query::bm_mesh_calc_face_groups(
        bm,
        &|bm: &BMesh, e: usize| crate::query::bm_edge_is_manifold(bm, e),
    );
    bmo_slot_buffer_flag_enable(bm, op, false, "faces", FACE_FLAG);
    /* group faces by their computed group index */
    let mut groups: Vec<Vec<usize>> = vec![Vec::new(); group_tot as usize];
    for f in 0..bm.totface as usize {
        let g = group_index[f];
        if g >= 0 {
            groups[g as usize].push(f);
        }
    }
    for faces in groups {
        if faces.iter().any(|&f| face_flag(bm, f, FACE_FLAG)) {
            bmo_recalc_face_normals_array(bm, &faces, FACE_FLAG);
        }
    }
}
