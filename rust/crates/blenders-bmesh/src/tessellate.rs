//! Port of `bmesh_mesh_tessellate.cc` — face → triangle `looptris` output.
//! Serial only (the C version's threading is flattened). `BMPartialUpdate`
//! variants run on the partial face list when provided.

use crate::{BMesh, BM_FACE, BM_NIL};
use blenders_math::geom;
use blenders_math::matrix::Mat3;
use blenders_math::vector::{Vec2, Vec3};

/// `poly_to_tri_count` — total triangles over all faces.
pub fn poly_to_tri_count(totface: i32, totloop: i32) -> i32 {
    totloop - 2 * totface
}

fn face_loop_indices(bm: &BMesh, f: usize) -> Vec<usize> {
    let mut out = Vec::with_capacity(bm.face(f).len as usize);
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    loop {
        out.push(l);
        l = bm.loo(l).next;
        if l == l_first {
            break;
        }
    }
    out
}

fn project_verts(bm: &BMesh, f: usize, axis_mat: &Mat3) -> Vec<[f32; 2]> {
    face_loop_indices(bm, f)
        .iter()
        .map(|&l| {
            let p = *axis_mat * bm.vert(bm.loo(l).v).co;
            [p.x, p.y]
        })
        .collect()
}

/// `bmesh_calc_tessellation_for_face_impl` — fill `looptris` for one face.
/// Returns the number of triangles written.
pub fn bmesh_calc_tessellation_for_face_impl(
    bm: &mut BMesh,
    looptris: &mut Vec<[usize; 3]>,
    f: usize,
    face_normal: bool,
) -> usize {
    let f_len = bm.face(f).len;
    match f_len {
        3 => {
            let l0 = bm.face(f).l_first;
            let l1 = bm.loo(l0).next;
            let l2 = bm.loo(l1).next;
            if face_normal {
                let (n, _) = geom::normal_tri(
                    bm.vert(bm.loo(l0).v).co,
                    bm.vert(bm.loo(l1).v).co,
                    bm.vert(bm.loo(l2).v).co,
                );
                bm.fpool[f].no = n;
            }
            looptris.push([l0, l1, l2]);
            1
        }
        4 => {
            let l0 = bm.face(f).l_first;
            let l1 = bm.loo(l0).next;
            let l2 = bm.loo(l1).next;
            let l3 = bm.loo(l2).next;
            if face_normal {
                let (n, _) = geom::normal_quad(
                    bm.vert(bm.loo(l0).v).co,
                    bm.vert(bm.loo(l1).v).co,
                    bm.vert(bm.loo(l2).v).co,
                    bm.vert(bm.loo(l3).v).co,
                );
                bm.fpool[f].no = n;
            }
            if geom::is_quad_flip_first_third_fast(
                bm.vert(bm.loo(l0).v).co,
                bm.vert(bm.loo(l1).v).co,
                bm.vert(bm.loo(l2).v).co,
                bm.vert(bm.loo(l3).v).co,
            ) {
                /* flip out of degenerate 0-2 state */
                looptris.push([l0, l1, l3]);
                looptris.push([l1, l2, l3]);
            } else {
                looptris.push([l0, l1, l2]);
                looptris.push([l0, l2, l3]);
            }
            2
        }
        _ => {
            if face_normal {
                crate::normals::bm_face_calc_normals(bm, f);
            }
            let no = bm.face(f).no;
            let axis_mat = geom::axis_dominant_to_m3_negate(no);
            let l_arr = face_loop_indices(bm, f);
            let projverts = project_verts(bm, f, &axis_mat);
            let tris = blenders_bli::polyfill_2d::polyfill_calc_pub(&projverts, 1);
            for tri in &tris {
                looptris.push([
                    l_arr[tri[0] as usize],
                    l_arr[tri[1] as usize],
                    l_arr[tri[2] as usize],
                ]);
            }
            tris.len()
        }
    }
}

/// `BM_mesh_calc_tessellation_ex` — all faces; `face_normal` recomputes f->no.
pub fn bm_mesh_calc_tessellation_ex(
    bm: &mut BMesh,
    looptris: &mut Vec<[usize; 3]>,
    face_normal: bool,
) {
    for i in 0..bm.fpool.len() {
        if bm.face(i).head.htype != BM_FACE {
            continue;
        }
        bmesh_calc_tessellation_for_face_impl(bm, looptris, i, face_normal);
    }
}

/// `BM_mesh_calc_tessellation`.
pub fn bm_mesh_calc_tessellation(bm: &mut BMesh, looptris: &mut Vec<[usize; 3]>) {
    bm_mesh_calc_tessellation_ex(bm, looptris, false);
}

/// `bmesh_calc_tessellation_for_face_beauty` — beauty-pass tessellation of
/// one face (rotates quad/ngon edges to nicer splits).
pub fn bmesh_calc_tessellation_for_face_beauty(
    bm: &mut BMesh,
    looptris: &mut Vec<[usize; 3]>,
    f: usize,
) -> usize {
    let f_len = bm.face(f).len;
    match f_len {
        3 => {
            let l0 = bm.face(f).l_first;
            let l1 = bm.loo(l0).next;
            let l2 = bm.loo(l1).next;
            looptris.push([l0, l1, l2]);
            1
        }
        4 => {
            let l_v1 = bm.face(f).l_first;
            let l_v2 = bm.loo(l_v1).next;
            let l_v3 = bm.loo(l_v2).next;
            let l_v4 = bm.loo(l_v1).prev;
            let axis_mat = geom::axis_dominant_to_m3(bm.face(f).no);
            let p = |l: usize| -> Vec2 {
                let c = axis_mat * bm.vert(bm.loo(l).v).co;
                Vec2::new(c.x, c.y)
            };
            let vq = [p(l_v1), p(l_v2), p(l_v3), p(l_v4)];
            let mut r_area: Option<f32> = None;
            let split_13 = blenders_bli::polyfill_2d_beautify::polyfill_beautify_quad_rotate_calc_ex(
                [vq[0].x, vq[0].y],
                [vq[1].x, vq[1].y],
                [vq[2].x, vq[2].y],
                [vq[3].x, vq[3].y],
                false,
                &mut r_area,
            ) < 0.0;
            if split_13 {
                looptris.push([l_v1, l_v2, l_v3]);
                looptris.push([l_v1, l_v3, l_v4]);
            } else {
                looptris.push([l_v1, l_v2, l_v4]);
                looptris.push([l_v2, l_v3, l_v4]);
            }
            2
        }
        _ => {
            let axis_mat = geom::axis_dominant_to_m3_negate(bm.face(f).no);
            let l_arr = face_loop_indices(bm, f);
            let projverts = project_verts(bm, f, &axis_mat);
            let mut tris = blenders_bli::polyfill_2d::polyfill_calc_pub(&projverts, 1);
            blenders_bli::polyfill_2d_beautify::polyfill_beautify(&projverts, &mut tris);
            for tri in &tris {
                looptris.push([
                    l_arr[tri[0] as usize],
                    l_arr[tri[1] as usize],
                    l_arr[tri[2] as usize],
                ]);
            }
            tris.len()
        }
    }
}

/// `BM_mesh_calc_tessellation_beauty`.
pub fn bm_mesh_calc_tessellation_beauty(bm: &mut BMesh, looptris: &mut Vec<[usize; 3]>) {
    for i in 0..bm.fpool.len() {
        if bm.face(i).head.htype != BM_FACE {
            continue;
        }
        bmesh_calc_tessellation_for_face_beauty(bm, looptris, i);
    }
}

/// `BM_mesh_calc_tessellation_with_partial_ex` — like `_ex` but only faces
/// marked in `bmpinfo` (port: takes explicit face index list).
pub fn bm_mesh_calc_tessellation_with_partial_ex(
    bm: &mut BMesh,
    looptris: &mut Vec<[usize; 3]>,
    faces: &[usize],
    face_normal: bool,
) {
    for &f in faces {
        if bm.face(f).head.htype != BM_FACE {
            continue;
        }
        bmesh_calc_tessellation_for_face_impl(bm, looptris, f, face_normal);
    }
}

/// `BM_mesh_calc_tessellation_with_partial`.
pub fn bm_mesh_calc_tessellation_with_partial(
    bm: &mut BMesh,
    looptris: &mut Vec<[usize; 3]>,
    faces: &[usize],
) {
    bm_mesh_calc_tessellation_with_partial_ex(bm, looptris, faces, false);
}

/// keep BM_NIL referenced for parity with the C header API surface.
const _: usize = BM_NIL;
const _: Vec3 = Vec3::ZERO;
