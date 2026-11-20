//! Port of `bmo_flatten.cc` + `bmo_planar_faces.cc`.

use std::collections::{HashMap, HashSet};
use crate::operators::*;
use crate::{BMesh, BM_NIL, BM_EDGE, BM_FACE, BM_VERT};
use blenders_math::geom::{closest_to_plane_normalized, plane_from_point_normal};
use blenders_math::vector::Vec3;

const ELE_VERT_ADJUST: u8 = 1;
const ELE_FACE_ADJUST: u8 = 2;

const FLATTEN_BEST_FIT: i32 = 0;
const FLATTEN_NORMAL: i32 = 1;
const FLATTEN_VIEW: i32 = 2;

fn compute_centroid(bm: &BMesh, verts: &[usize]) -> Vec3 {
    let mut c = Vec3::ZERO;
    for &v in verts { c = c + bm.vert(v).co; }
    if !verts.is_empty() { c = c / verts.len() as f32; }
    c
}

fn compute_average_face_normal(bm: &BMesh, faces: &[usize]) -> Vec3 {
    let mut n = Vec3::ZERO;
    for &f in faces { n = n + bm.face(f).no; }
    if n.length() > 0.0 { n = n.normalized(); }
    n
}

/// `bmo_flatten_exec`.
pub fn bmo_flatten_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let factor = bmo_slot_float_get(op, false, "factor");
    let method = bmo_slot_int_get(op, false, "method");
    let lock_x = bmo_slot_bool_get(op, false, "lock_x");
    let lock_y = bmo_slot_bool_get(op, false, "lock_y");
    let lock_z = bmo_slot_bool_get(op, false, "lock_z");
    let mut view_direction = Vec3::new(0.0, 0.0, 1.0);
    if method == FLATTEN_VIEW {
        view_direction = bmo_slot_vec_get(op, false, "view_normal");
    }
    let flatten_verts = |bm: &mut BMesh, verts: &[usize], faces: &[usize]| {
        let mut actual = method;
        if actual == FLATTEN_NORMAL && faces.is_empty() {
            actual = FLATTEN_BEST_FIT;
        }
        let (normal, center) = match actual {
            FLATTEN_BEST_FIT => {
                let (n, c, _) = crate::polygon::bm_verts_calc_normal_from_cloud_ex(bm, verts);
                (n, c)
            }
            FLATTEN_NORMAL => (compute_average_face_normal(bm, faces), compute_centroid(bm, verts)),
            _ => (view_direction, compute_centroid(bm, verts)),
        };
        debug_assert!(normal.length() > 0.0);
        for &v in verts {
            let co = bm.vert(v).co;
            let mut projected = co - normal * (co - center).dot(normal);
            if lock_x { projected.x = co.x; }
            if lock_y { projected.y = co.y; }
            if lock_z { projected.z = co.z; }
            bm.vpool[v].co = co + (projected - co) * factor;
        }
    };

    crate::marking::bm_mesh_elem_hflag_disable_all(
        bm, (1 << BM_FACE) | (1 << BM_EDGE), crate::BM_ELEM_TAG);
    bmo_slot_buffer_hflag_enable(
        bm, op, false, "geom", (1 << BM_FACE) | (1 << BM_EDGE), crate::BM_ELEM_TAG, false);

    /* face groups */
    let (face_group_num, face_group_idx) = crate::query::bm_mesh_calc_face_groups(
        bm, &|bm: &BMesh, f| bm.face(f).head.hflag & crate::BM_ELEM_TAG != 0);
    let mut groups: Vec<Vec<usize>> = vec![Vec::new(); face_group_num as usize];
    for f in crate::iterators::FacesOfMesh::new(bm) {
        let g = face_group_idx[f];
        if g >= 0 { groups[g as usize].push(f); }
    }
    for faces in &groups {
        let mut verts: Vec<usize> = Vec::new();
        let mut seen = HashSet::new();
        for &f in faces {
            for e in crate::iterators::EdgesOfFace::new(bm, f).collect::<Vec<_>>() {
                bm.epool[e].head.hflag &= !crate::BM_ELEM_TAG;
            }
            for v in crate::iterators::VertsOfFace::new(bm, f) {
                if seen.insert(v) { verts.push(v); }
            }
        }
        flatten_verts(bm, &verts, faces);
    }
    /* edge groups (loose edges not in any tagged face) */
    let (edge_group_num, edge_group_idx) = crate::query::bm_mesh_calc_edge_groups(
        bm, &|bm: &BMesh, e| bm.edge(e).head.hflag & crate::BM_ELEM_TAG != 0);
    let mut egroups: Vec<Vec<usize>> = vec![Vec::new(); edge_group_num as usize];
    for e in crate::iterators::EdgesOfMesh::new(bm) {
        let g = edge_group_idx[e];
        if g >= 0 { egroups[g as usize].push(e); }
    }
    for edges in &egroups {
        let mut verts: Vec<usize> = Vec::new();
        let mut seen = HashSet::new();
        for &e in edges {
            for v in [bm.edge(e).v1, bm.edge(e).v2] {
                if seen.insert(v) { verts.push(v); }
            }
        }
        flatten_verts(bm, &verts, &[]);
    }
}

/// `bmo_planar_faces_exec`.
pub fn bmo_planar_faces_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let fac = bmo_slot_float_get(op, false, "factor");
    let iterations = bmo_slot_int_get(op, false, "iterations");
    let eps_sq = 0.00001f32 * 0.00001;
    let faces: Vec<usize> = bmo_slot_buffer(op, false, "faces")
        .iter().filter(|(ht, _)| *ht == BM_FACE).map(|&(_, i)| i).collect();
    /* per-slot-index face centers */
    let slot_faces = bmo_slot_buffer(op, false, "faces").to_vec();
    let mut faces_center: HashMap<usize, Vec3> = HashMap::new();
    let mut shared_vert_num = 0;
    for &(_, f) in &slot_faces {
        if bm.face(f).len == 3 { continue; }
        faces_center.insert(f, crate::polygon::bm_face_calc_center_median_weighted(bm, f));
        for l in crate::iterators::LoopsOfFace::new(bm, f) {
            let v = bm.loo(l).v;
            if !bmo_elem_flag_test(bm, (BM_VERT, v), ELE_VERT_ADJUST) {
                bmo_elem_flag_enable(bm, (BM_VERT, v), ELE_VERT_ADJUST);
                shared_vert_num += 1;
            }
        }
        bmo_elem_flag_enable(bm, (BM_FACE, f), ELE_FACE_ADJUST);
    }
    let _ = shared_vert_num;
    let _ = faces;
    let mut vaccum_map: HashMap<usize, (Vec3, i32)> = HashMap::new();
    for _ in 0..iterations {
        let mut changed = false;
        for &(_, f) in &slot_faces {
            if !bmo_elem_flag_test(bm, (BM_FACE, f), ELE_FACE_ADJUST) { continue; }
            bmo_elem_flag_disable(bm, (BM_FACE, f), ELE_FACE_ADJUST);
            debug_assert_ne!(bm.face(f).len, 3);
            let plane = plane_from_point_normal(faces_center[&f], bm.face(f).no);
            for l in crate::iterators::LoopsOfFace::new(bm, f).collect::<Vec<_>>() {
                let v = bm.loo(l).v;
                let co = closest_to_plane_normalized(plane, bm.vert(v).co);
                let e = vaccum_map.entry(v).or_insert((Vec3::ZERO, 0));
                e.1 += 1;
                e.0 = e.0 + (co - e.0) * (1.0 / e.1 as f32);
            }
        }
        for (&v, &(co, _)) in &vaccum_map {
            if (bm.vert(v).co - co).length_squared() > eps_sq {
                bmo_elem_flag_enable(bm, (BM_VERT, v), ELE_VERT_ADJUST);
                bm.vpool[v].co = bm.vert(v).co + (co - bm.vert(v).co) * fac;
                changed = true;
            }
            for f in crate::iterators::FacesOfVert::new(bm, v).collect::<Vec<_>>() {
                if bm.face(f).len != 3 {
                    bmo_elem_flag_enable(bm, (BM_FACE, f), ELE_FACE_ADJUST);
                }
            }
        }
        if !changed { break; }
        vaccum_map.clear();
    }
}
