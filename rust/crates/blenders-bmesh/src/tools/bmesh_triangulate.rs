//! Port of `tools/bmesh_triangulate.cc` — `BM_mesh_triangulate`.

use crate::operators::*;
use crate::{BMesh, BM_FACE};

/// `BM_mesh_triangulate` — triangulate all faces with `len >= min_vertices`.
/// With `op`+slot names, also fills face_map.out/face_map_double.out;
/// else kills double faces directly.
pub fn bm_mesh_triangulate(
    bm: &mut BMesh,
    quad_method: i32,
    ngon_method: i32,
    min_vertices: i32,
    tag_only: bool,
    op: Option<&mut BMOperator>,
    slot_facemap: Option<&str>,
    slot_facemap_double: Option<&str>,
) {
    let faces: Vec<usize> = crate::iterators::FacesOfMesh::new(bm)
        .filter(|&f| bm.face(f).len >= min_vertices
            && (!tag_only || bm.face(f).head.hflag & crate::BM_ELEM_TAG != 0))
        .collect();
    match op {
        Some(op) => {
            for f in faces {
                let (faces_array, _edges, faces_double) = crate::polygon::bm_face_triangulate(
                    bm, f, quad_method, ngon_method, tag_only);
                if !faces_array.is_empty() || !faces_double.is_empty() {
                    if let Some(name) = slot_facemap {
                        bmo_slot_map_insert(op, true, name, (BM_FACE, f), (BM_FACE, f));
                        for &nf in &faces_array {
                            bmo_slot_map_insert(op, true, name, (BM_FACE, nf), (BM_FACE, f));
                        }
                    }
                    if let Some(name) = slot_facemap_double {
                        for &fd in &faces_double {
                            bmo_slot_map_insert(op, true, name, (BM_FACE, fd), (BM_FACE, f));
                        }
                    }
                }
            }
        }
        None => {
            let mut faces_double: Vec<usize> = Vec::new();
            for f in faces {
                let (_, _, fd) = crate::polygon::bm_face_triangulate(
                    bm, f, quad_method, ngon_method, tag_only);
                faces_double.extend(fd);
            }
            for f in faces_double {
                crate::core::bm_face_kill(bm, f);
            }
        }
    }
}
