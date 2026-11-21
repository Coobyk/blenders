//! Port of `intern/mesh_tessellate.cc` — `bke::mesh::corner_tris_calc*`
//! (corner-triangle tessellation).
//!
//! `threading::parallel_for` → serial loops; the per-thread `MemArena`
//! becomes `Vec` scratch (polyfill already returns `Vec<[u32;3]>`).

use blenders_bli::polyfill_2d::polyfill_calc_pub;
use blenders_math::geom::{
    add_newell_cross, axis_dominant_to_m3_negate, is_quad_flip_first_third_fast, poly_to_tri_count,
};
use blenders_math::matrix_c::mul_v2_m3v3;
use blenders_math::vector::Vec3;

/// `face_triangles_num` (mesh.hh inline): `face_size - 2`.
fn face_triangles_num(face_size: i32) -> i32 {
    face_size - 2
}

/// `mesh_calc_tessellation_for_face_impl` — appends tris to `out`.
#[allow(clippy::too_many_arguments)]
fn mesh_calc_tessellation_for_face_impl(
    corner_verts: &[i32],
    positions: &[Vec3],
    face_start: usize,
    face_size: usize,
    out: &mut Vec<[i32; 3]>,
    normal_precalc: Option<Vec3>,
) {
    match face_size {
        3 => {
            out.push([face_start as i32, face_start as i32 + 1, face_start as i32 + 2]);
        }
        4 => {
            let mut tri_a = [face_start as i32, face_start as i32 + 1, face_start as i32 + 2];
            let mut tri_b = [face_start as i32, face_start as i32 + 2, face_start as i32 + 3];
            if is_quad_flip_first_third_fast(
                positions[corner_verts[tri_a[0] as usize] as usize],
                positions[corner_verts[tri_a[1] as usize] as usize],
                positions[corner_verts[tri_a[2] as usize] as usize],
                positions[corner_verts[tri_b[2] as usize] as usize],
            ) {
                tri_a[2] = tri_b[2];
                tri_b[0] = tri_a[1];
            }
            out.push(tri_a);
            out.push(tri_b);
        }
        _ => {
            let normal = match normal_precalc {
                Some(n) => n,
                None => {
                    let mut normal = Vec3::ZERO;
                    let mut co_prev = positions[corner_verts[face_start + face_size - 1] as usize];
                    for j in 0..face_size {
                        let co_curr = positions[corner_verts[face_start + j] as usize];
                        add_newell_cross(&mut normal, co_prev, co_curr);
                        co_prev = co_curr;
                    }
                    if normal.length() == 0.0 {
                        Vec3::new(0.0, 0.0, 1.0)
                    } else {
                        normal.normalized()
                    }
                }
            };
            let axis_mat = axis_dominant_to_m3_negate(normal);
            let totfilltri = face_size - 2;
            let mut projverts = Vec::with_capacity(face_size);
            for j in 0..face_size {
                let p = mul_v2_m3v3(axis_mat, positions[corner_verts[face_start + j] as usize]);
                projverts.push([p.x, p.y]);
            }
            let tris = polyfill_calc_pub(&projverts, 1);
            debug_assert_eq!(tris.len(), totfilltri);
            for t in tris {
                out.push([
                    face_start as i32 + t[0] as i32,
                    face_start as i32 + t[1] as i32,
                    face_start as i32 + t[2] as i32,
                ]);
            }
        }
    }
}

/// `corner_tris_calc_impl` — `faces` as ranges over `corner_verts`.
fn corner_tris_calc_impl(
    positions: &[Vec3],
    faces: &[std::ops::Range<i64>],
    corner_verts: &[i32],
    face_normals: &[Vec3],
    corner_tris: &mut [[i32; 3]],
) {
    let mut out_idx = 0usize;
    for (i, face) in faces.iter().enumerate() {
        let face_start = face.start as usize;
        let face_size = (face.end - face.start) as usize;
        let tris_start = poly_to_tri_count(i as i32, face_start as i32) as usize;
        debug_assert_eq!(tris_start, out_idx);
        let mut tris = Vec::new();
        mesh_calc_tessellation_for_face_impl(
            corner_verts,
            positions,
            face_start,
            face_size,
            &mut tris,
            if face_normals.is_empty() { None } else { Some(face_normals[i]) },
        );
        corner_tris[out_idx..out_idx + tris.len()].copy_from_slice(&tris);
        out_idx += tris.len();
    }
}

/// `corner_tris_calc`.
pub fn corner_tris_calc(
    vert_positions: &[Vec3],
    faces: &[std::ops::Range<i64>],
    corner_verts: &[i32],
    corner_tris: &mut [[i32; 3]],
) {
    corner_tris_calc_impl(vert_positions, faces, corner_verts, &[], corner_tris);
}

/// `corner_tris_calc_face_indices`.
pub fn corner_tris_calc_face_indices(faces: &[std::ops::Range<i64>], tri_faces: &mut [i32]) {
    for (i, face) in faces.iter().enumerate() {
        let start = poly_to_tri_count(i as i32, face.start as i32) as usize;
        let num = face_triangles_num((face.end - face.start) as i32) as usize;
        for slot in &mut tri_faces[start..start + num] {
            *slot = i as i32;
        }
    }
}

/// `corner_tris_calc_with_normals`.
pub fn corner_tris_calc_with_normals(
    vert_positions: &[Vec3],
    faces: &[std::ops::Range<i64>],
    corner_verts: &[i32],
    face_normals: &[Vec3],
    corner_tris: &mut [[i32; 3]],
) {
    debug_assert!(!face_normals.is_empty() || faces.is_empty());
    corner_tris_calc_impl(vert_positions, faces, corner_verts, face_normals, corner_tris);
}

/// `vert_tris_from_corner_tris` — gather verts per corner tri.
pub fn vert_tris_from_corner_tris(
    corner_verts: &[i32],
    corner_tris: &[[i32; 3]],
    vert_tris: &mut [[i32; 3]],
) {
    for (i, tri) in corner_tris.iter().enumerate() {
        vert_tris[i] = [
            corner_verts[tri[0] as usize],
            corner_verts[tri[1] as usize],
            corner_verts[tri[2] as usize],
        ];
    }
}

/// `corner_tri_get_real_edges`.
pub fn corner_tri_get_real_edges(
    edges: &[[i32; 2]],
    corner_verts: &[i32],
    corner_edges: &[i32],
    corner_tri: &[i32; 3],
) -> [i32; 3] {
    let mut real_edges = [0i32; 3];
    let mut i = 2usize;
    for i_next in 0..3 {
        let corner_1 = corner_tri[i] as usize;
        let corner_2 = corner_tri[i_next] as usize;
        let vert_1 = corner_verts[corner_1];
        let vert_2 = corner_verts[corner_2];
        let edge_i = corner_edges[corner_1] as usize;
        let edge = edges[edge_i];
        let is_real = (vert_1 == edge[0] && vert_2 == edge[1]) || (vert_1 == edge[1] && vert_2 == edge[0]);
        real_edges[i] = if is_real { edge_i as i32 } else { -1 };
        i = i_next;
    }
    real_edges
}
