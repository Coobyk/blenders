//! Port of `bmesh_mesh.cc` — mesh lifecycle, element indexing, remap.
//!
//! Mempool slots in C are arena indices here; dead elements have `htype==0`
//! (set by kill functions). `elem_table` arrays are unnecessary since index
//! lookups scan arena slots directly. Toolflags (`_OFlag` elements),
//! `BM_mesh_remap` CustomData remap and `bmesh_edit_begin/end` operator hooks
//! are PORT_PENDING.

use crate::{BMesh, BM_EDGE, BM_FACE, BM_LOOP, BM_NIL, BM_VERT};
use blenders_math::matrix::Mat4;
use blenders_math::vector::Vec3;

/// `BM_mesh_elem_index_ensure_ex` — assign head.index for htype elements.
pub fn bm_mesh_elem_index_ensure_ex(bm: &mut BMesh, htype: u8, elem_offset: &mut [i32; 4]) {
    match htype {
        BM_VERT => {
            let mut i = elem_offset[0];
            for v in bm.vpool.iter_mut() {
                v.head.index = i;
                i += 1;
            }
            elem_offset[0] = i;
        }
        BM_EDGE => {
            let mut i = elem_offset[1];
            for e in bm.epool.iter_mut() {
                e.head.index = i;
                i += 1;
            }
            elem_offset[1] = i;
        }
        BM_FACE => {
            let mut i = elem_offset[2];
            for f in bm.fpool.iter_mut() {
                f.head.index = i;
                i += 1;
            }
            elem_offset[2] = i;
        }
        BM_LOOP => {
            let mut i = elem_offset[3];
            for l in bm.lpool.iter_mut() {
                l.head.index = i;
                i += 1;
            }
            elem_offset[3] = i;
        }
        _ => debug_assert!(false),
    }
}

/// `BM_mesh_elem_index_ensure` — htype flags like `BM_VERT|BM_FACE`.
pub fn bm_mesh_elem_index_ensure(bm: &mut BMesh, htype: u8) {
    let mut offset = [0i32; 4];
    if (htype & (1 << BM_VERT)) != 0 || htype == BM_VERT {
        bm_mesh_elem_index_ensure_ex(bm, BM_VERT, &mut offset);
    }
    if (htype & (1 << BM_EDGE)) != 0 || htype == BM_EDGE {
        bm_mesh_elem_index_ensure_ex(bm, BM_EDGE, &mut offset);
    }
    if (htype & (1 << BM_FACE)) != 0 || htype == BM_FACE {
        bm_mesh_elem_index_ensure_ex(bm, BM_FACE, &mut offset);
    }
    if (htype & (1 << BM_LOOP)) != 0 || htype == BM_LOOP {
        bm_mesh_elem_index_ensure_ex(bm, BM_LOOP, &mut offset);
    }
}

/// `BM_vert_at_index_find` — vert whose head.index == index.
pub fn bm_vert_at_index_find(bm: &BMesh, index: i32) -> usize {
    for (i, v) in bm.vpool.iter().enumerate() {
        if v.head.htype == BM_VERT && v.head.index == index {
            return i;
        }
    }
    BM_NIL
}

/// `BM_edge_at_index_find`.
pub fn bm_edge_at_index_find(bm: &BMesh, index: i32) -> usize {
    for (i, e) in bm.epool.iter().enumerate() {
        if e.head.htype == BM_EDGE && e.head.index == index {
            return i;
        }
    }
    BM_NIL
}

/// `BM_face_at_index_find`.
pub fn bm_face_at_index_find(bm: &BMesh, index: i32) -> usize {
    for (i, f) in bm.fpool.iter().enumerate() {
        if f.head.htype == BM_FACE && f.head.index == index {
            return i;
        }
    }
    BM_NIL
}

/// `BM_loop_at_index_find`.
pub fn bm_loop_at_index_find(bm: &BMesh, index: i32) -> usize {
    for (i, l) in bm.lpool.iter().enumerate() {
        if l.head.htype == BM_LOOP && l.head.index == index {
            return i;
        }
    }
    BM_NIL
}

/// `BM_mesh_elem_count`.
pub fn bm_mesh_elem_count(bm: &BMesh, htype: u8) -> i32 {
    match htype {
        BM_VERT => bm.totvert,
        BM_EDGE => bm.totedge,
        BM_FACE => bm.totface,
        BM_LOOP => bm.totloop,
        _ => 0,
    }
}

/// `BM_mesh_clear` — remove all elements.
pub fn bm_mesh_clear(bm: &mut BMesh) {
    bm.vpool.clear();
    bm.epool.clear();
    bm.lpool.clear();
    bm.fpool.clear();
    bm.totvert = 0;
    bm.totedge = 0;
    bm.totloop = 0;
    bm.totface = 0;
    bm.elem_table_dirty = true;
    bm.elem_index_dirty = true;
}

/// `BM_mesh_vert_coords_get` — fill positions from verts.
pub fn bm_mesh_vert_coords_get(bm: &BMesh, positions: &mut [Vec3]) {
    let mut i = 0;
    for v in bm.vpool.iter() {
        if v.head.htype == BM_VERT && i < positions.len() {
            positions[i] = v.co;
            i += 1;
        }
    }
}

/// `BM_mesh_vert_normals_get`.
pub fn bm_mesh_vert_normals_get(bm: &BMesh, normals: &mut [Vec3]) {
    let mut i = 0;
    for v in bm.vpool.iter() {
        if v.head.htype == BM_VERT && i < normals.len() {
            normals[i] = v.no;
            i += 1;
        }
    }
}

/// `BM_mesh_vert_coords_alloc`.
pub fn bm_mesh_vert_coords_alloc(bm: &BMesh) -> Vec<Vec3> {
    let mut positions = Vec::with_capacity(bm.totvert.max(0) as usize);
    for v in bm.vpool.iter() {
        if v.head.htype == BM_VERT {
            positions.push(v.co);
        }
    }
    positions
}

/// `BM_mesh_vert_coords_apply`.
pub fn bm_mesh_vert_coords_apply(bm: &mut BMesh, vert_coords: &[Vec3]) {
    let mut i = 0;
    for v in bm.vpool.iter_mut() {
        if v.head.htype == BM_VERT && i < vert_coords.len() {
            v.co = vert_coords[i];
            i += 1;
        }
    }
}

/// `BM_mesh_vert_coords_apply_with_mat4`.
pub fn bm_mesh_vert_coords_apply_with_mat4(
    bm: &mut BMesh,
    vert_coords: &[Vec3],
    mat: Mat4,
) {
    let mut i = 0;
    for v in bm.vpool.iter_mut() {
        if v.head.htype == BM_VERT && i < vert_coords.len() {
            v.co = mat.transform_point(vert_coords[i]);
            i += 1;
        }
    }
}

/// `BM_mesh_vert_coords_apply_to_marked` — apply only where marked.
pub fn bm_mesh_vert_coords_apply_to_marked(
    bm: &mut BMesh,
    vert_coords: &[Vec3],
    marked: &[bool],
) {
    let mut i = 0;
    for v in bm.vpool.iter_mut() {
        if v.head.htype == BM_VERT && i < vert_coords.len() {
            if marked.get(i).copied().unwrap_or(false) {
                v.co = vert_coords[i];
            }
            i += 1;
        }
    }
}

/// `BM_mesh_debug_info` — Python-dict-style debug dump of the mesh
/// (CustomData layer sections are PORT_PENDING: no CustomData).
pub fn bm_mesh_debug_info(bm: &BMesh) -> String {
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str(&format!("    'ptr': '{:p}',\n", bm as *const BMesh));
    s.push_str(&format!("    'totvert': {},\n", bm.totvert));
    s.push_str(&format!("    'totedge': {},\n", bm.totedge));
    s.push_str(&format!("    'totface': {},\n", bm.totface));
    for name in ["vert_layers", "edge_layers", "loop_layers", "poly_layers"] {
        s.push_str(&format!("    '{}': (\n", name));
        /* PORT_PENDING: CustomData_debug_info_from_layers(&bm->?data). */
        s.push_str("    ),\n");
    }
    s.push_str("}\n");
    s
}

/// `BM_mesh_debug_print`.
pub fn bm_mesh_debug_print(bm: &BMesh) {
    print!("{}", bm_mesh_debug_info(bm));
}
