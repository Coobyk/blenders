//! Port of `bmesh_boolean.cc` (`WITH_GMP` build) — BMesh glue around
//! `blenders_bli::mesh_boolean`/`mesh_intersect` (IMesh/IMeshArena).

use crate::iterators::*;
use crate::BMesh;
use crate::{BM_EDGE, BM_ELEM_HIDDEN, BM_ELEM_TAG, BM_FACE, BM_NIL, BM_VERT};
use blenders_bli::mesh_boolean::BoolOpType;
use blenders_bli::mesh_intersect::{IMesh, IMeshArena, NO_INDEX};
use blenders_math::mpq::{mpq, Mpq3};
use blenders_math::vector::Vec3;

const KEEP_FLAG: u8 = 1 << 6; /* BM_ELEM_TAG_ALT */

/// `mesh_from_bm` — build an #IMesh from the mesh; also the triangulated
/// version (in `r_triangulated`).
fn mesh_from_bm(
    bm: &mut BMesh,
    looptris: &[[usize; 3]],
    r_triangulated: &mut IMesh,
    arena: &mut IMeshArena,
) -> IMesh {
    crate::mesh::bm_mesh_elem_index_ensure(bm, BM_VERT | BM_EDGE | BM_FACE);
    /* Account for triangulation and intersects. */
    arena.reserve(3 * bm.totvert as usize, 4 * bm.totface as usize);

    /* vert table: index -> vert arena idx */
    let mut vert: Vec<usize> = vec![BM_NIL; bm.totvert as usize];
    for v in VertsOfMesh::new(bm) {
        let c = bm.vert(v).co;
        let co = Mpq3::new(
            mpq(c.x as f64),
            mpq(c.y as f64),
            mpq(c.z as f64),
        );
        vert[bm.vert(v).head.index as usize] =
            arena.add_or_find_vert_mpq(&co, bm.vert(v).head.index);
    }
    let mut face: Vec<usize> = vec![BM_NIL; bm.totface as usize];
    let mut face_vert: Vec<usize> = Vec::with_capacity(100);
    let mut face_edge_orig: Vec<i32> = Vec::with_capacity(100);
    for f in FacesOfMesh::new(bm) {
        face_vert.clear();
        face_edge_orig.clear();
        let mut l = bm.face(f).l_first;
        let l_first = l;
        loop {
            face_vert.push(vert[bm.vert(bm.loo(l).v).head.index as usize]);
            face_edge_orig.push(bm.edge(bm.loo(l).e).head.index);
            l = bm.loo(l).next;
            if l == l_first {
                break;
            }
        }
        face[bm.face(f).head.index as usize] = arena.add_face_eo(
            face_vert.clone(),
            bm.face(f).head.index,
            face_edge_orig.clone(),
        );
    }
    /* Now do the triangulation mesh.
     * The loop_tris have accurate v and f members for the triangles,
     * but their next and e pointers are not correct for the loops
     * that start added-diagonal edges. */
    let mut tri_face: Vec<usize> = Vec::with_capacity(looptris.len());
    face_vert.clear();
    face_edge_orig.clear();
    for lt in looptris {
        let f_index = bm.face(bm.loo(lt[0]).f).head.index;
        face_vert.clear();
        face_edge_orig.clear();
        for j in 0..3 {
            let l = lt[j];
            let v_index = bm.vert(bm.loo(l).v).head.index;
            let e_index = if bm.loo(bm.loo(l).next).v == bm.loo(lt[(j + 1) % 3]).v {
                bm.edge(bm.loo(l).e).head.index
            } else {
                NO_INDEX
            };
            face_vert.push(vert[v_index as usize]);
            face_edge_orig.push(e_index);
        }
        tri_face.push(arena.add_face_eo(
            face_vert.clone(),
            f_index,
            face_edge_orig.clone(),
        ));
    }
    r_triangulated.set_faces(tri_face);
    IMesh::from_faces(face)
}

/// `bmvert_attached_to_wire`.
fn bmvert_attached_to_wire(bm: &BMesh, v: usize) -> bool {
    crate::query::bm_vert_is_wire(bm, v)
}

/// `bmvert_attached_to_hidden_face`.
fn bmvert_attached_to_hidden_face(bm: &BMesh, v: usize) -> bool {
    for f in FacesOfVert::new(bm, v) {
        if bm.face(f).head.hflag & BM_ELEM_HIDDEN != 0 {
            return true;
        }
    }
    false
}

/// `apply_mesh_output_to_bmesh` — change `bm` to match `m_out`.
/// Kills unused geometry except wire edges and hidden geometry (when
/// `keep_hidden`); `BM_ELEM_TAG` marks edges created by intersections.
fn apply_mesh_output_to_bmesh(
    bm: &mut BMesh,
    m_out: &mut IMesh,
    arena: &IMeshArena,
    keep_hidden: bool,
) -> bool {
    let mut any_change = false;
    m_out.populate_vert(arena);
    /* Initially mark all existing verts as "don't keep", except hidden verts
     * (if keep_hidden) and verts attached to wire edges. */
    for v in VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if (keep_hidden
            && (bm.vert(v).head.hflag & BM_ELEM_HIDDEN != 0
                || bmvert_attached_to_hidden_face(bm, v)))
            || bmvert_attached_to_wire(bm, v)
        {
            bm.vpool[v].head.hflag |= KEEP_FLAG;
        } else {
            bm.vpool[v].head.hflag &= !KEEP_FLAG;
        }
    }
    /* vert index -> bm vert (table by elem index; index ensured below) */
    crate::mesh::bm_mesh_elem_index_ensure(bm, BM_VERT | BM_FACE | BM_EDGE);
    let mut vert_tab = vec![BM_NIL; bm.totvert as usize];
    for v in VertsOfMesh::new(bm) {
        vert_tab[bm.vert(v).head.index as usize] = v;
    }
    /* Reuse old or make new #BMVert's, depending on if there's an orig or not. */
    let mut new_bmvs: Vec<usize> = vec![BM_NIL; m_out.vert.len()];
    for v in 0..m_out.vert.len() {
        let vertp = &arena.verts[m_out.vert[v]];
        let orig = vertp.orig;
        if orig != NO_INDEX {
            let bmv = vert_tab[orig as usize];
            new_bmvs[v] = bmv;
            bm.vpool[bmv].head.hflag |= KEEP_FLAG;
        }
    }
    for v in 0..m_out.vert.len() {
        if new_bmvs[v] == BM_NIL {
            let d_co = arena.verts[m_out.vert[v]].co;
            let bmv = crate::core::bm_vert_create(
                bm,
                Vec3::new(d_co[0] as f32, d_co[1] as f32, d_co[2] as f32),
                None,
                0,
            );
            new_bmvs[v] = bmv;
            bm.vpool[bmv].head.hflag |= KEEP_FLAG;
            any_change = true;
        }
    }
    /* Save current faces (by index) so creating faces won't disturb the table. */
    let mut old_bmfs = vec![BM_NIL; bm.totface as usize];
    for f in FacesOfMesh::new(bm).collect::<Vec<_>>() {
        old_bmfs[bm.face(f).head.index as usize] = f;
        if keep_hidden && bm.face(f).head.hflag & BM_ELEM_HIDDEN != 0 {
            bm.fpool[f].head.hflag |= KEEP_FLAG;
        } else {
            bm.fpool[f].head.hflag &= !KEEP_FLAG;
        }
    }
    /* Save the original edges so we can use them as examples. */
    let mut old_edges = vec![BM_NIL; bm.totedge as usize];
    for e in EdgesOfMesh::new(bm) {
        old_edges[bm.edge(e).head.index as usize] = e;
    }
    /* Reuse or make new #BMFace's. */
    for fi in 0..m_out.face_size() {
        let f_id = m_out.faces()[fi];
        let face = arena.faces[f_id].clone();
        let flen = face.vert.len();
        let mut face_bmverts = vec![BM_NIL; flen];
        for (i, &v) in face.vert.iter().enumerate() {
            let v_index = m_out.lookup_vert(v) as usize;
            face_bmverts[i] = new_bmvs[v_index];
        }
        let bmf = crate::query::bm_face_exists(bm, &face_bmverts);
        /* Never allow any duplicates (either winding), see: 160437. */
        if bmf != BM_NIL {
            bm.fpool[bmf].head.hflag |= KEEP_FLAG;
        } else {
            let orig = face.orig;
            let orig_face = if orig != NO_INDEX { old_bmfs[orig as usize] } else { BM_NIL };
            /* Make or find #BMEdge's. */
            let mut face_bmedges = vec![BM_NIL; flen];
            for i in 0..flen {
                let bmv1 = face_bmverts[i];
                let bmv2 = face_bmverts[(i + 1) % flen];
                let mut bme = crate::query::bm_edge_exists(bm, bmv1, bmv2);
                if bme == BM_NIL {
                    let orig_edge = if face.edge_orig[i] != NO_INDEX {
                        old_edges[face.edge_orig[i] as usize]
                    } else {
                        BM_NIL
                    };
                    bme = crate::core::bm_edge_create(
                        bm,
                        bmv1,
                        bmv2,
                        if orig_edge != BM_NIL { Some(orig_edge) } else { None },
                        0,
                    );
                    if orig_edge != BM_NIL {
                        let flags = (
                            bm.edge(orig_edge).head.api_flag,
                            bm.edge(orig_edge).head.hflag,
                        );
                        crate::construct::bm_elem_select_copy(
                            bm, BM_EDGE, bme, flags);
                    }
                }
                face_bmedges[i] = bme;
                if face.is_intersect.get(i).copied().unwrap_or(false) {
                    bm.epool[bme].head.hflag |= BM_ELEM_TAG;
                } else {
                    bm.epool[bme].head.hflag &= !BM_ELEM_TAG;
                }
            }
            let bmf = crate::core::bm_face_create(
                bm,
                &face_bmverts,
                &face_bmedges,
                if orig_face != BM_NIL { Some(orig_face) } else { None },
                0,
            );
            if orig_face != BM_NIL {
                let flags = (
                    bm.face(orig_face).head.api_flag,
                    bm.face(orig_face).head.hflag,
                );
                crate::construct::bm_elem_select_copy(bm, BM_FACE, bmf, flags);
            }
            bm.fpool[bmf].head.hflag |= KEEP_FLAG;
            /* Now do interpolation of loop data (e.g., UVs) using the example face. */
            if orig_face != BM_NIL {
                let l_first = bm.face(bmf).l_first;
                let mut l = l_first;
                loop {
                    crate::interp::bm_loop_interp_from_face(bm, l, orig_face, false, true);
                    l = bm.loo(l).next;
                    if l == l_first {
                        break;
                    }
                }
            }
            any_change = true;
        }
    }
    /* Now kill the unused faces and verts, and clear flags for kept ones. */
    for bmf in FacesOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.face(bmf).head.hflag & KEEP_FLAG != 0 {
            bm.fpool[bmf].head.hflag &= !KEEP_FLAG;
        } else {
            /* BM_face_kill_loose: kill face but leave its loose edges. */
            crate::core::bm_face_kill_loose(bm, bmf);
            any_change = true;
        }
    }
    for bmv in VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.vert(bmv).head.hflag & KEEP_FLAG != 0 {
            bm.vpool[bmv].head.hflag &= !KEEP_FLAG;
        } else {
            crate::core::bm_vert_kill(bm, bmv);
            any_change = true;
        }
    }
    bm.elem_index_dirty = true;
    any_change
}

/// `meshintersect::bmesh_boolean`.
#[allow(clippy::too_many_arguments)]
fn bmesh_boolean(
    bm: &mut BMesh,
    looptris: &[[usize; 3]],
    test_fn: &dyn Fn(&BMesh, usize) -> i32,
    nshapes: i32,
    use_self: bool,
    use_separate_all: bool,
    keep_hidden: bool,
    hole_tolerant: bool,
    boolean_mode: BoolOpType,
) -> bool {
    let mut arena = IMeshArena::new();
    let mut m_triangulated = IMesh::new();
    let m_in = mesh_from_bm(bm, looptris, &mut m_triangulated, &mut arena);

    let shape_fn = |f: i32| -> i32 {
        let bmf = crate::mesh::bm_face_at_index_find(bm, f);
        let test_val = test_fn(bm, bmf);
        if use_self && boolean_mode == BoolOpType::None {
            /* Unary knife op: every face where test_fn doesn't return -1. */
            if test_val != -1 {
                0
            } else {
                -1
            }
        } else if test_val >= 0 {
            test_val
        } else {
            -1
        }
    };
    let mut m_out = blenders_bli::mesh_boolean::boolean_mesh(
        &mut arena,
        &m_in,
        Some(&m_triangulated),
        boolean_mode,
        nshapes,
        &shape_fn,
        use_self,
        hole_tolerant,
    );
    let any_change = apply_mesh_output_to_bmesh(bm, &mut m_out, &arena, keep_hidden);
    if use_separate_all {
        /* separate all faces incident on intersection edges */
        crate::tools::bmesh_edgesplit::bm_mesh_edgesplit(bm, false, true, false);
    }
    any_change
}

/// `BM_mesh_boolean`.
#[allow(clippy::too_many_arguments)]
pub fn bm_mesh_boolean(
    bm: &mut BMesh,
    looptris: &[[usize; 3]],
    test_fn: &dyn Fn(&BMesh, usize) -> i32,
    nshapes: i32,
    use_self: bool,
    keep_hidden: bool,
    hole_tolerant: bool,
    boolean_mode: i32,
) -> bool {
    bmesh_boolean(
        bm,
        looptris,
        test_fn,
        nshapes,
        use_self,
        false,
        keep_hidden,
        hole_tolerant,
        match boolean_mode {
            0 => BoolOpType::Intersect,
            1 => BoolOpType::Union,
            2 => BoolOpType::Difference,
            _ => BoolOpType::None,
        },
    )
}

/// `BM_mesh_boolean_knife`.
#[allow(clippy::too_many_arguments)]
pub fn bm_mesh_boolean_knife(
    bm: &mut BMesh,
    looptris: &[[usize; 3]],
    test_fn: &dyn Fn(&BMesh, usize) -> i32,
    nshapes: i32,
    use_self: bool,
    use_separate_all: bool,
    hole_tolerant: bool,
    keep_hidden: bool,
) -> bool {
    bmesh_boolean(
        bm,
        looptris,
        test_fn,
        nshapes,
        use_self,
        use_separate_all,
        keep_hidden,
        hole_tolerant,
        BoolOpType::None,
    )
}
