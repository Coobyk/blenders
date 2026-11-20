//! Port of `bmo_extrude.cc` — extrude ops + solidify.
//! `use_select_history`, `skin_modifier_root`, loop CustomData copies:
//! PORT_PENDING.

use std::collections::HashSet;
use crate::operators::*;
use crate::{BMesh, BM_NIL, BM_EDGE, BM_FACE, BM_VERT};
use blenders_math::vector::Vec3;

const EXT_INPUT: u8 = 1;
const EXT_KEEP: u8 = 2;
const EXT_DEL: u8 = 4;
const EXT_TAG: u8 = 8;
const ALL_NOLOOP: u8 = (1 << BM_VERT) | (1 << BM_EDGE) | (1 << BM_FACE);

/// `bmo_extrude_discrete_faces_exec`.
pub fn bmo_extrude_discrete_faces_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let faces: Vec<usize> = bmo_slot_buffer(op, false, "faces")
        .iter().filter(|(ht, _)| *ht == BM_FACE).map(|&(_, i)| i).collect();
    for f_org in faces {
        bmo_elem_flag_enable(bm, (BM_FACE, f_org), EXT_DEL);
        let f_new = crate::core::bm_face_copy(bm, f_org, true, true);
        bmo_elem_flag_enable(bm, (BM_FACE, f_new), EXT_KEEP);
        /* side walls: connect orig loops to new loops */
        let flen = bm.face(f_org).len as usize;
        let l_org_first = bm.face(f_org).l_first;
        let l_new_first = bm.face(f_new).l_first;
        let mut l_org = l_org_first;
        let mut l_new = l_new_first;
        for _ in 0..flen {
            let f_side = crate::construct::bm_face_create_quad_tri(
                bm,
                bm.loo(bm.loo(l_org).next).v,
                bm.loo(bm.loo(l_new).next).v,
                bm.loo(l_new).v,
                bm.loo(l_org).v,
                Some(f_org), 0);
            bmo_elem_flag_enable(bm, (BM_FACE, f_side), EXT_KEEP);
            l_org = bm.loo(l_org).next;
            l_new = bm.loo(l_new).next;
        }
    }
    /* delete original faces */
    let del = bmo_op_callf(bm, op.flag, "delete", &[
        CallfArg::OFlag("geom", BM_FACE, EXT_DEL),
        CallfArg::Int("context", crate::delete::DEL_ONLYFACES),
    ]);
    bmo_op_finish(bm, del);
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "faces.out", 1 << BM_FACE, EXT_KEEP);
}

/// `bm_extrude_copy_face_loop_attributes` — PORT_PENDING no-op
/// (loop CustomData copy); keeps hidden-flag unhide behavior.
fn bm_extrude_copy_face_loop_attributes(bm: &mut BMesh, f: usize) {
    let l_first_0 = bm.face(f).l_first;
    if l_first_0 == bm.loo(l_first_0).radial_next {
        return;
    }
    let l_other_0 = crate::query::bm_edge_other_loop(bm, bm.loo(l_first_0).e, l_first_0);
    if l_other_0 == BM_NIL { return; }
    let f_other = bm.loo(l_other_0).f;
    /* attrs copy PORT_PENDING; still copy mat_nr + unhide */
    bm.fpool[f].mat_nr = bm.face(f_other).mat_nr;
    bm.fpool[f].head.hflag &= !crate::BM_ELEM_HIDDEN;
}

/// `bmo_extrude_edge_only_exec`.
pub fn bmo_extrude_edge_only_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let use_normal_flip = bmo_slot_bool_get(op, false, "use_normal_flip");
    let edges: Vec<usize> = bmo_slot_buffer(op, false, "edges")
        .iter().filter(|(ht, _)| *ht == BM_EDGE).map(|&(_, i)| i).collect();
    for e in edges {
        bmo_elem_flag_enable(bm, (BM_EDGE, e), EXT_INPUT);
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        bmo_elem_flag_enable(bm, (BM_VERT, v1), EXT_INPUT);
        bmo_elem_flag_enable(bm, (BM_VERT, v2), EXT_INPUT);
    }
    let mut dupeop = bmo_op_init(bm, &crate::opdefines::OPDEF_DUPLICATE, op.flag);
    let geom_buf: Vec<_> = (0..bm.epool.len())
        .filter(|&e| bmo_elem_flag_test(bm, (BM_EDGE, e), EXT_INPUT))
        .flat_map(|e| [(BM_EDGE, e), (BM_VERT, bm.edge(e).v1), (BM_VERT, bm.edge(e).v2)])
        .collect();
    /* C: "duplicate geom=%fve" — buffer from flag EXT_INPUT v+e */
    let mut seen_v = HashSet::new();
    let mut seen_e = HashSet::new();
    let geom_buf: Vec<_> = geom_buf.into_iter().filter(|(ht, i)| {
        if *ht == BM_VERT { seen_v.insert(*i) } else { seen_e.insert(*i) }
    }).collect();
    slot_get_mut(&mut dupeop.slots_in, "geom").buf = geom_buf;
    bmo_op_exec(bm, &mut dupeop, crate::bmo::bmo_dupe::bmo_duplicate_exec);
    /* side walls along boundary_map edges */
    let bmap: Vec<(crate::operators::BMElemRef, crate::operators::BMElemRef)> = dupeop.slots_out
        .iter().find(|s| s.name == "boundary_map.out")
        .map(|s| s.map.iter().map(|(&k, &v)| (k, v)).collect())
        .unwrap_or_default();
    for ((ht_e, e), (ht_ne, e_new)) in bmap {
        if ht_e != BM_EDGE || ht_ne != BM_EDGE { continue; }
        let edge_normal_flip = !(bm.edge(e).l != BM_NIL
            && bm.edge(e).v1 != bm.loo(bm.edge(e).l).v);
        let f_verts = if edge_normal_flip == use_normal_flip {
            [bm.edge(e).v1, bm.edge(e).v2, bm.edge(e_new).v2, bm.edge(e_new).v1]
        } else {
            [bm.edge(e).v2, bm.edge(e).v1, bm.edge(e_new).v1, bm.edge(e_new).v2]
        };
        let f = crate::core::bm_face_create_verts(bm, &f_verts, None, 0, true, true);
        bm_extrude_copy_face_loop_attributes(bm, f);
        let e_keep = if bmo_elem_flag_test(bm, (BM_EDGE, e), EXT_INPUT) { e_new } else { e };
        bmo_elem_flag_enable(bm, (BM_FACE, f), EXT_KEEP);
        bmo_elem_flag_enable(bm, (BM_EDGE, e_keep), EXT_KEEP);
        let (v1, v2) = (bm.edge(e_keep).v1, bm.edge(e_keep).v2);
        bmo_elem_flag_enable(bm, (BM_VERT, v1), EXT_KEEP);
        bmo_elem_flag_enable(bm, (BM_VERT, v2), EXT_KEEP);
    }
    bmo_op_finish(bm, dupeop);
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "geom.out", ALL_NOLOOP, EXT_KEEP);
}

/// `bmo_extrude_vert_indiv_exec`.
pub fn bmo_extrude_vert_indiv_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let verts: Vec<usize> = bmo_slot_buffer(op, false, "verts")
        .iter().filter(|(ht, _)| *ht == BM_VERT).map(|&(_, i)| i).collect();
    for v in verts {
        let co = bm.vert(v).co;
        let mut v = v;
        let mut dupev = crate::core::bm_vert_create(bm, co, Some(v), 0);
        bmo_elem_flag_enable(bm, (BM_VERT, dupev), EXT_KEEP);
        /* wire endpoint: keep original at endpoint */
        let is_wire_endpoint = bm.vert(v).e != BM_NIL
            && bm.edge(bm.vert(v).e).l == BM_NIL
            && crate::query::bm_vert_edge_count(bm, v) == 1;
        if is_wire_endpoint && bm.edge(bm.vert(v).e).v1 == v {
            std::mem::swap(&mut v, &mut dupev);
        }
        let e = crate::core::bm_edge_create(bm, v, dupev, None, 0);
        bmo_elem_flag_enable(bm, (BM_EDGE, e), EXT_KEEP);
    }
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "verts.out", 1 << BM_VERT, EXT_KEEP);
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "edges.out", 1 << BM_EDGE, EXT_KEEP);
}

/// `bm_extrude_region_edge_flag` — merge SEAM/SMOOTH hflags of manifold
/// edges around a vert.
fn bm_extrude_region_edge_flag(bm: &BMesh, v: usize, r: &mut [u8; 2]) -> bool {
    let mut ok = false;
    r[0] = 0;
    r[1] = 0xff;
    if bm.vert(v).e == BM_NIL { return false; }
    let mut e_iter = bm.vert(v).e;
    loop {
        if bm.edge(e_iter).l != BM_NIL && !crate::delete::bm_edge_is_boundary(bm, e_iter) {
            r[0] |= bm.edge(e_iter).head.hflag;
            r[1] &= bm.edge(e_iter).head.hflag;
            ok = true;
        }
        e_iter = crate::structure::bmesh_disk_edge_next(bm, e_iter, v);
        if e_iter == bm.vert(v).e { break; }
    }
    if ok {
        r[0] &= crate::BM_ELEM_SEAM;
        r[1] = crate::BM_ELEM_SMOOTH & !r[1];
    }
    ok
}

/// `bmo_extrude_face_region_exec`.
pub fn bmo_extrude_face_region_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let use_normal_flip = bmo_slot_bool_get(op, false, "use_normal_flip");
    let use_normal_from_adjacent = bmo_slot_bool_get(op, false, "use_normal_from_adjacent");
    let use_dissolve_ortho_edges = bmo_slot_bool_get(op, false, "use_dissolve_ortho_edges");
    let skip_input_flip = bmo_slot_bool_get(op, false, "skip_input_flip");
    let use_keep_orig = bmo_slot_bool_get(op, false, "use_keep_orig");
    bmo_slot_buffer_flag_enable(bm, op, false, "geom", EXT_INPUT);
    let mut delorig = false;
    if !use_keep_orig {
        for e in crate::iterators::EdgesOfMesh::new(bm).collect::<Vec<_>>() {
            if !bmo_elem_flag_test(bm, (BM_EDGE, e), EXT_INPUT) { continue; }
            let mut found = false;
            let mut edge_face_tot = 0;
            for f in crate::iterators::FacesOfEdge::new(bm, e) {
                if !bmo_elem_flag_test(bm, (BM_FACE, f), EXT_INPUT) {
                    found = true;
                    delorig = true;
                    break;
                }
                edge_face_tot += 1;
            }
            if edge_face_tot > 1 && !found {
                bmo_elem_flag_enable(bm, (BM_EDGE, e), EXT_DEL);
            }
        }
    }
    for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.vert(v).e != BM_NIL {
            let mut found = false;
            for e in crate::iterators::EdgesOfVert::new(bm, v) {
                if bmo_elem_flag_test(bm, (BM_EDGE, e), EXT_INPUT) {
                    found = true;
                    break;
                }
            }
            if found {
                bmo_elem_flag_enable(bm, (BM_VERT, v), EXT_INPUT);
            }
        }
    }
    for f in crate::iterators::FacesOfMesh::new(bm).collect::<Vec<_>>() {
        if bmo_elem_flag_test(bm, (BM_FACE, f), EXT_INPUT) {
            bmo_elem_flag_enable(bm, (BM_FACE, f), EXT_DEL);
        }
    }
    let mut dupeop = bmo_op_init(bm, &crate::opdefines::OPDEF_DUPLICATE, op.flag);
    let geom: Vec<_> = bmo_slot_buffer(op, false, "geom").to_vec();
    slot_get_mut(&mut dupeop.slots_in, "geom").buf = geom;
    bmo_op_exec(bm, &mut dupeop, crate::bmo::bmo_dupe::bmo_duplicate_exec);
    /* remap act_face */
    if bm.act_face != BM_NIL && bmo_elem_flag_test(bm, (BM_FACE, bm.act_face), EXT_INPUT) {
        if let Some((_, nf)) = bmo_slot_map_get(&dupeop, true, "face_map.out", (BM_FACE, bm.act_face)) {
            bm.act_face = nf;
        }
    }
    if delorig {
        let del = bmo_op_callf(bm, op.flag, "delete", &[
            CallfArg::OFlag("geom", (1 << BM_VERT) | (1 << BM_EDGE) | (1 << BM_FACE), EXT_DEL),
            CallfArg::Int("context", crate::delete::DEL_ONLYTAGGED),
        ]);
        bmo_op_finish(bm, del);
    }
    if !delorig && !skip_input_flip {
        for f in crate::iterators::FacesOfMesh::new(bm).collect::<Vec<_>>() {
            if bmo_elem_flag_test(bm, (BM_FACE, f), EXT_INPUT) {
                crate::polygon::bm_face_normal_flip(bm, f);
            }
        }
    }
    let mut dissolve_verts: Vec<usize> = Vec::new();
    let mut average_normal = Vec3::ZERO;
    if use_dissolve_ortho_edges {
        for (ht, f) in bmo_slot_buffer(&dupeop, true, "geom.out") {
            if *ht == BM_FACE { average_normal = average_normal + bm.face(*f).no; }
        }
        if average_normal.length() == 0.0 {
            average_normal = Vec3::new(0.0, 0.0, 1.0);
        } else {
            average_normal = average_normal.normalized();
        }
    }
    let geom_out: Vec<_> = bmo_slot_buffer(&dupeop, true, "geom.out").to_vec();
    slot_get_mut(&mut op.slots_out, "geom.out").buf = geom_out;
    let boundary: Vec<(crate::operators::BMElemRef, crate::operators::BMElemRef)> = dupeop.slots_out
        .iter().find(|s| s.name == "boundary_map.out")
        .map(|s| s.map.iter().map(|(&k, &v)| (k, v)).collect())
        .unwrap_or_default();
    for ((ht_e, e), (ht_ne, e_new)) in boundary {
        if ht_e != BM_EDGE { continue; }
        if ht_ne != BM_EDGE { continue; }
        let excluded = bmo_slot_map_contains(op, false, "edges_exclude", (BM_EDGE, e));
        if crate::edgenet::edge_is_wire(bm, e) && excluded {
            let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
            crate::core::bm_edge_kill(bm, e);
            if bm.vert(v1).e == BM_NIL { crate::core::bm_vert_kill(bm, v1); }
            if bm.vert(v2).e == BM_NIL { crate::core::bm_vert_kill(bm, v2); }
            continue;
        }
        if excluded { continue; }
        let mut join_face = BM_NIL;
        if use_dissolve_ortho_edges && crate::delete::bm_edge_is_boundary(bm, e) {
            let jf = bm.loo(bm.edge(e).l).f;
            if average_normal.dot(bm.face(jf).no).abs() <= 0.0001 {
                join_face = jf;
            }
        }
        let edge_normal_flip = if !use_normal_from_adjacent {
            !(if bm.edge(e_new).l != BM_NIL {
                bm.loo(bm.edge(e_new).l).v == bm.edge(e_new).v1
            } else {
                !(bm.edge(e).l != BM_NIL && bm.loo(bm.edge(e).l).v == bm.edge(e).v1)
            })
        } else {
            !(bm.edge(e).l != BM_NIL && bm.edge(e).v1 != bm.loo(bm.edge(e).l).v)
        };
        let f_verts = if edge_normal_flip == use_normal_flip {
            [bm.edge(e).v1, bm.edge(e).v2, bm.edge(e_new).v2, bm.edge(e_new).v1]
        } else {
            [bm.edge(e).v2, bm.edge(e).v1, bm.edge(e_new).v1, bm.edge(e_new).v2]
        };
        /* f_edges: use region edge flags for the two new side edges */
        let mut f_edges = [e, BM_NIL, e_new, BM_NIL];
        f_edges[1] = crate::query::bm_edge_exists(bm, f_verts[1], f_verts[2]);
        if f_edges[1] == BM_NIL {
            let mut e_hflag = [0u8; 2];
            let ok = bm_extrude_region_edge_flag(bm, f_verts[2], &mut e_hflag);
            f_edges[1] = crate::core::bm_edge_create(bm, f_verts[1], f_verts[2], None, 0);
            if ok {
                bm.epool[f_edges[1]].head.hflag |= e_hflag[0];
                bm.epool[f_edges[1]].head.hflag &= !e_hflag[1];
            }
        }
        f_edges[3] = crate::query::bm_edge_exists(bm, f_verts[3], f_verts[0]);
        if f_edges[3] == BM_NIL {
            let mut e_hflag = [0u8; 2];
            let ok = bm_extrude_region_edge_flag(bm, f_verts[3], &mut e_hflag);
            f_edges[3] = crate::core::bm_edge_create(bm, f_verts[3], f_verts[0], None, 0);
            if ok {
                bm.epool[f_edges[3]].head.hflag |= e_hflag[0];
                bm.epool[f_edges[3]].head.hflag &= !e_hflag[1];
            }
        }
        let f = crate::core::bm_face_create(bm, &f_verts, &f_edges, None, 0);
        bm_extrude_copy_face_loop_attributes(bm, f);
        if join_face != BM_NIL {
            let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
            for v in [v1, v2] {
                if !bmo_elem_flag_test(bm, (BM_VERT, v), EXT_TAG) {
                    bmo_elem_flag_enable(bm, (BM_VERT, v), EXT_TAG);
                    dissolve_verts.push(v);
                }
            }
            bmo_elem_flag_enable(bm, (BM_EDGE, f_edges[0]), EXT_TAG);
            bmo_elem_flag_enable(bm, (BM_EDGE, f_edges[1]), EXT_TAG);
            crate::core::bmesh_kernel_join_face_kill_edge(bm, join_face, f, e);
        }
    }
    /* isovert links */
    let iso: Vec<(crate::operators::BMElemRef, crate::operators::BMElemRef)> = dupeop.slots_out
        .iter().find(|s| s.name == "isovert_map.out")
        .map(|s| s.map.iter().map(|(&k, &v)| (k, v)).collect())
        .unwrap_or_default();
    for ((ht_v, v), (ht_v2, v2)) in iso {
        if ht_v != BM_VERT || ht_v2 != BM_VERT { continue; }
        let mut v = v;
        let mut v2 = v2;
        let is_wire_endpoint = bm.vert(v).e != BM_NIL
            && bm.edge(bm.vert(v).e).l == BM_NIL
            && crate::query::bm_vert_edge_count(bm, v) == 1;
        if is_wire_endpoint && bm.edge(bm.vert(v).e).v1 == v {
            std::mem::swap(&mut v, &mut v2);
        }
        crate::core::bm_edge_create(bm, v, v2, None, crate::BM_CREATE_NO_DOUBLE);
    }
    /* dissolve ortho verts */
    for v in dissolve_verts {
        if bm.vert(v).e == BM_NIL { continue; }
        let e = bm.vert(v).e;
        let e_other = crate::structure::bmesh_disk_edge_next(bm, e, v);
        if e_other == e || crate::structure::bmesh_disk_edge_next(bm, e_other, v) == e {
            let e_col = if bmo_elem_flag_test(bm, (BM_EDGE, e), EXT_TAG) { e } else { e_other };
            crate::mods::bm_edge_collapse(bm, e_col, v, true, true);
        }
    }
    bmo_op_finish(bm, dupeop);
}

/* Solidify (same C file). */
const VERT_MARK: u8 = 1;
const FACE_MARK: u8 = 1;
const EDGE_MARK: u8 = 1;
const VERT_NONMAN: u8 = 2;
const EDGE_NONMAN: u8 = 2;

/// `calc_solidify_normals`.
fn calc_solidify_normals(bm: &mut BMesh) {
    for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
        bm.vpool[v].head.hflag |= crate::BM_ELEM_TAG;
    }
    /* edge_face_count per edge slot index */
    let mut edge_face_count = vec![0i32; bm.epool.len()];
    for f in crate::iterators::FacesOfMesh::new(bm).collect::<Vec<_>>() {
        if !bmo_elem_flag_test(bm, (BM_FACE, f), FACE_MARK) { continue; }
        for e in crate::iterators::EdgesOfFace::new(bm, f).collect::<Vec<_>>() {
            bmo_elem_flag_enable(bm, (BM_EDGE, e), EDGE_MARK);
            let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
            bmo_elem_flag_enable(bm, (BM_VERT, v1), VERT_MARK);
            bmo_elem_flag_enable(bm, (BM_VERT, v2), VERT_MARK);
            edge_face_count[e] += 1;
        }
    }
    for e in crate::iterators::EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        if !bmo_elem_flag_test(bm, (BM_EDGE, e), EDGE_MARK) { continue; }
        let i = edge_face_count[e];
        if i == 0 || i > 2 {
            bmo_elem_flag_enable(bm, (BM_EDGE, e), EDGE_NONMAN);
            let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
            bmo_elem_flag_enable(bm, (BM_VERT, v1), VERT_NONMAN);
            bmo_elem_flag_enable(bm, (BM_VERT, v2), VERT_NONMAN);
        }
    }
    for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if !crate::query::bm_vert_is_manifold(bm, v) {
            bmo_elem_flag_enable(bm, (BM_VERT, v), VERT_NONMAN);
            continue;
        }
        if bmo_elem_flag_test(bm, (BM_VERT, v), VERT_MARK) {
            bm.vpool[v].no = Vec3::ZERO;
        }
    }
    for e in crate::iterators::EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        if !bmo_elem_flag_test(bm, (BM_EDGE, e), EDGE_MARK)
            || bmo_elem_flag_test(bm, (BM_EDGE, e), EDGE_NONMAN)
        { continue; }
        let mut f1 = BM_NIL;
        let mut f2 = BM_NIL;
        for f in crate::iterators::FacesOfEdge::new(bm, e) {
            if bmo_elem_flag_test(bm, (BM_FACE, f), FACE_MARK) {
                if f1 == BM_NIL { f1 = f; } else { debug_assert_eq!(f2, BM_NIL); f2 = f; }
            }
        }
        debug_assert_ne!(f1, BM_NIL);
        let edge_normal;
        if f2 != BM_NIL {
            let angle = blenders_math::geom::angle_normalized_v3v3(bm.face(f1).no, bm.face(f2).no);
            if angle > 0.0 {
                edge_normal = (bm.face(f1).no + bm.face(f2).no).normalized() * angle;
            } else {
                let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
                bm.vpool[v1].head.hflag &= !crate::BM_ELEM_TAG;
                bm.vpool[v2].head.hflag &= !crate::BM_ELEM_TAG;
                continue;
            }
        } else {
            edge_normal = bm.face(f1).no * std::f32::consts::FRAC_PI_2;
        }
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        bm.vpool[v1].no = bm.vert(v1).no + edge_normal;
        bm.vpool[v2].no = bm.vert(v2).no + edge_normal;
    }
    for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if !bmo_elem_flag_test(bm, (BM_VERT, v), VERT_MARK) { continue; }
        if bmo_elem_flag_test(bm, (BM_VERT, v), VERT_NONMAN) {
            crate::normals::bm_vert_normal_update(bm, v);
        } else if bm.vert(v).no.length() == 0.0
            && bm.vert(v).head.hflag & crate::BM_ELEM_TAG == 0
        {
            let mut f_hit = BM_NIL;
            for f in crate::iterators::FacesOfVert::new(bm, v) {
                if bmo_elem_flag_test(bm, (BM_FACE, f), FACE_MARK) { f_hit = f; break; }
            }
            if f_hit != BM_NIL {
                bm.vpool[v].no = bm.face(f_hit).no;
            }
        }
    }
}

/// `solidify_add_thickness`.
fn solidify_add_thickness(bm: &mut BMesh, dist: f32) {
    let n = bm.vpool.len();
    let mut vert_angles = vec![0.0f32; n];
    let mut vert_accum = vec![0.0f32; n];
    for f in crate::iterators::FacesOfMesh::new(bm).collect::<Vec<_>>() {
        if !bmo_elem_flag_test(bm, (BM_FACE, f), FACE_MARK) { continue; }
        let loops = crate::iterators::LoopsOfFace::new(bm, f).collect::<Vec<_>>();
        let vcos: Vec<Vec3> = loops.iter().map(|&l| bm.vert(bm.loo(l).v).co).collect();
        let face_angles = blenders_math::vector::angle_poly(&vcos);
        for (i, &l) in loops.iter().enumerate() {
            let v = bm.loo(l).v;
            vert_accum[v] += face_angles[i];
            vert_angles[v] += blenders_math::geom::shell_v3v3_normalized_to_dist(
                bm.vert(v).no, bm.face(f).no) * face_angles[i];
        }
    }
    for v in 0..n {
        if vert_accum[v] != 0.0 {
            bm.vpool[v].co = bm.vert(v).co + bm.vert(v).no * (dist * (vert_angles[v] / vert_accum[v]));
        }
    }
}

/// `bmo_solidify_face_region_exec`.
pub fn bmo_solidify_face_region_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let thickness = bmo_slot_float_get(op, false, "thickness");
    let mut reverseop = bmo_op_init(bm, &crate::opdefines::OPDEF_REVERSE_FACES, op.flag);
    bmo_slot_bool_set(&mut reverseop, false, "flip_multires", true);
    let geom: Vec<_> = bmo_slot_buffer(op, false, "geom").to_vec();
    slot_get_mut(&mut reverseop.slots_in, "faces").buf = geom;
    bmo_op_exec(bm, &mut reverseop, crate::bmo::bmo_removedoubles::bmo_reverse_faces_exec);
    bmo_op_finish(bm, reverseop);
    let mut extrudeop = bmo_op_init(bm, &crate::opdefines::OPDEF_EXTRUDE_FACE_REGION, op.flag);
    bmo_slot_bool_set(&mut extrudeop, false, "use_keep_orig", true);
    let geom: Vec<_> = bmo_slot_buffer(op, false, "geom").to_vec();
    slot_get_mut(&mut extrudeop.slots_in, "geom").buf = geom;
    bmo_op_exec(bm, &mut extrudeop, bmo_extrude_face_region_exec);
    bmo_slot_buffer_flag_enable(bm, &mut extrudeop, true, "geom.out", FACE_MARK);
    calc_solidify_normals(bm);
    solidify_add_thickness(bm, thickness);
    let geom_out: Vec<_> = bmo_slot_buffer(&extrudeop, true, "geom.out").to_vec();
    slot_get_mut(&mut op.slots_out, "geom.out").buf = geom_out;
    bmo_op_finish(bm, extrudeop);
}
