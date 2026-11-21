//! Port of `intern/multires_reshape_apply_base.cc` — push
//! `MultiresReshapeContext` results back onto the base mesh.
//!
//! `multires_reshape_evaluate_base_mesh_limit_at_grid` /
//! `multires_reshape_orig_grid_element_for_grid_coord` /
//! `bke::subdiv::eval_refine_from_mesh` stay injected closures —
//! PORT_PENDING the subdiv-eval + reshape-math backends.

use blenders_math::vector::Vec3;

/// `v3_dist_from_plane`.
fn v3_dist_from_plane(v: Vec3, center: Vec3, no: Vec3) -> f32 {
    let s = v - center;
    s.dot(no)
}

/// `multires_reshape_apply_base_refit_base_mesh` — vert positions are
/// updated in place; `vert_to_face_map` is `GroupedSpan<int>`.
///
/// `face_normal_calc` is `bke::mesh::face_normal_calc` (PORT_PENDING).
/// `tangent_matrix`/`grid_element` results come from the reshape-eval
/// callbacks (PORT_PENDING).
#[allow(clippy::too_many_arguments)]
pub fn multires_reshape_apply_base_refit_base_mesh(
    base_positions: &mut [Vec3],
    vert_to_face_map: &[Vec<usize>],
    base_faces: &[std::ops::Range<usize>],
    base_corner_verts: &[i32],
    face_normal_calc: &mut dyn FnMut(&[Vec3], &[i32]) -> Vec3,
) {
    let origco: Vec<Vec3> = base_positions.to_vec();
    for i in 0..base_positions.len() {
        let mut avg_no = Vec3::ZERO;
        let mut center = Vec3::ZERO;
        if vert_to_face_map[i].is_empty() {
            continue;
        }
        let mut tot = 0;
        for &face in &vert_to_face_map[i] {
            for corner in base_faces[face].clone() {
                let vndx = base_corner_verts[corner];
                if vndx != i as i32 {
                    center += origco[vndx as usize];
                    tot += 1;
                }
            }
        }
        if tot == 0 {
            continue;
        }
        center *= 1.0 / tot as f32;
        for j in 0..vert_to_face_map[i].len() {
            let face = base_faces[vert_to_face_map[i][j]].clone();
            let mut face_verts = vec![0i32; face.len()];
            let mut fake_co = vec![Vec3::ZERO; face.len()];
            for k in 0..face.len() {
                let vndx = base_corner_verts[face.start + k];
                face_verts[k] = k as i32;
                if vndx == i as i32 {
                    fake_co[k] = center;
                } else {
                    fake_co[k] = origco[vndx as usize];
                }
            }
            let no = face_normal_calc(&fake_co, &face_verts);
            avg_no += no;
        }
        avg_no = avg_no.normalized();
        let dist = v3_dist_from_plane(base_positions[i], center, avg_no);
        let push = avg_no * dist;
        base_positions[i] += push;
    }
    /* `base_mesh->tag_positions_changed()` — caller. */
}

/// `multires_reshape_apply_base_update_mesh_coords` — one base vert per
/// corner evaluated at grid (u=1, v=1) + tangent-frame displacement.
#[allow(clippy::too_many_arguments)]
pub fn multires_reshape_apply_base_update_mesh_coords(
    base_positions: &mut [Vec3],
    base_corner_verts: &[i32],
    evaluate_base_mesh_limit_at_grid: &mut dyn FnMut(i32) -> (Vec3, blenders_math::matrix::Mat3),
    grid_element_for_coord: &mut dyn FnMut(i32) -> Vec3,
) {
    for loop_index in 0..base_corner_verts.len() {
        /* `GridCoord{grid_index = loop_index, u = 1, v = 1}` */
        let (p, tangent_matrix) = evaluate_base_mesh_limit_at_grid(loop_index as i32);
        let displacement = grid_element_for_coord(loop_index as i32);
        let d = tangent_matrix * displacement;
        base_positions[base_corner_verts[loop_index] as usize] = p + d;
    }
}

/// `multires_reshape_apply_base_update_shape_key` — copies base positions
/// into the basis shape key when present.
pub fn multires_reshape_apply_base_update_shape_key(
    base_positions: &[Vec3],
    basis_shape_key: &mut Option<Vec<Vec3>>,
) {
    if let Some(basis_key_data) = basis_shape_key {
        basis_key_data.clear();
        basis_key_data.extend_from_slice(base_positions);
    }
}

/// `multires_reshape_apply_base_refine_from_base` /
/// `multires_reshape_apply_base_refine_from_deform` — thin wrappers over
/// `bke::subdiv::eval_refine_from_mesh` (PORT_PENDING, injected).
pub fn multires_reshape_apply_base_refine_from_base(
    eval_refine_from_mesh: &mut dyn FnMut(Option<Vec<Vec3>>),
) {
    eval_refine_from_mesh(None);
}

/// `multires_reshape_apply_base_refine_from_deform` — `deformed_verts`
/// is the output of `BKE_multires_create_deformed_base_mesh_vert_coords`
/// (depsgraph work; caller-resolved, PORT_PENDING).
pub fn multires_reshape_apply_base_refine_from_deform(
    eval_refine_from_mesh: &mut dyn FnMut(Option<Vec<Vec3>>),
    deformed_verts: Vec<Vec3>,
) {
    eval_refine_from_mesh(Some(deformed_verts));
}
