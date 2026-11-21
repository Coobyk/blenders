//! Port of `bmo_dupe.cc` — `duplicate` and `split` operators.
//! CustomData attribute copies (cd_*_map) are PORT_PENDING no-ops;
//! `use_select_history` is PORT_PENDING (no edit-selection list yet).

use std::collections::HashMap;
use crate::core::*;
use crate::operators::*;
use blenders_math::vector::{Vec3, Vec4};
use crate::operators::BMElemRef;
use crate::{BMesh, BM_NIL, BM_EDGE, BM_FACE, BM_VERT};

const DUPE_INPUT: u8 = 1;
const DUPE_NEW: u8 = 2;
const DUPE_DONE: u8 = 4;

fn slot_map_insert(op: &mut BMOperator, out: bool, name: &str, k: BMElemRef, v: BMElemRef) {
    bmo_slot_map_insert(op, out, name, k, v);
}

/// `bmo_vert_copy`.
fn bmo_vert_copy(
    op: &mut BMOperator,
    bm_dst: &mut BMesh,
    v_src: usize,
    vhash: &mut HashMap<usize, usize>,
) -> usize {
    let co = bm_dst.vert(v_src).co;
    let v_dst = crate::core::bm_vert_create(bm_dst, co, None, 0);
    slot_map_insert(op, true, "vert_map.out", (BM_VERT, v_src), (BM_VERT, v_dst));
    slot_map_insert(op, true, "vert_map.out", (BM_VERT, v_dst), (BM_VERT, v_src));
    vhash.insert(v_src, v_dst);
    bmo_elem_flag_enable(bm_dst, (BM_VERT, v_dst), DUPE_NEW);
    v_dst
}

/// `bmo_edge_copy`.
fn bmo_edge_copy(
    op: &mut BMOperator,
    bm_dst: &mut BMesh,
    bm_src: &BMesh,
    e_src: usize,
    vhash: &HashMap<usize, usize>,
    ehash: &mut HashMap<usize, usize>,
    use_edge_flip_from_face: bool,
) -> usize {
    let mut rlen = 0u32;
    if bm_src.edge(e_src).l != BM_NIL {
        let l_first = bm_src.edge(e_src).l;
        let mut l_iter = l_first;
        loop {
            if bmo_elem_flag_test(bm_src, (BM_FACE, bm_src.loo(l_iter).f), DUPE_INPUT) {
                rlen += 1;
            }
            l_iter = bm_src.loo(l_iter).radial_next;
            if l_iter == l_first { break; }
        }
    }
    let e_dst_v1 = vhash[&bm_src.edge(e_src).v1];
    let e_dst_v2 = vhash[&bm_src.edge(e_src).v2];
    let e_dst = crate::core::bm_edge_create(bm_dst, e_dst_v1, e_dst_v2, None, 0);
    slot_map_insert(op, true, "edge_map.out", (BM_EDGE, e_src), (BM_EDGE, e_dst));
    slot_map_insert(op, true, "edge_map.out", (BM_EDGE, e_dst), (BM_EDGE, e_src));
    if rlen < 2 {
        slot_map_insert(op, true, "boundary_map.out", (BM_EDGE, e_src), (BM_EDGE, e_dst));
    }
    ehash.insert(e_src, e_dst);
    bmo_elem_flag_enable(bm_dst, (BM_EDGE, e_dst), DUPE_NEW);
    if use_edge_flip_from_face {
        if crate::delete::bm_edge_is_boundary(bm_src, e_src)
            && bm_src.loo(bm_src.edge(e_src).l).v == bm_src.edge(e_src).v1
        {
            crate::mods::bm_edge_verts_swap(bm_dst, e_dst);
        }
    }
    e_dst
}

/// `bmo_face_copy`.
fn bmo_face_copy(
    op: &mut BMOperator,
    bm_dst: &mut BMesh,
    bm_src: &BMesh,
    f_src: usize,
    vhash: &HashMap<usize, usize>,
    ehash: &HashMap<usize, usize>,
) -> usize {
    let f_len = bm_src.face(f_src).len as usize;
    let mut vtar: Vec<usize> = Vec::with_capacity(f_len);
    let mut edar: Vec<usize> = Vec::with_capacity(f_len);
    let l_first_src = bm_src.face(f_src).l_first;
    let mut l_iter_src = l_first_src;
    loop {
        vtar.push(vhash[&bm_src.loo(l_iter_src).v]);
        edar.push(ehash[&bm_src.loo(l_iter_src).e]);
        l_iter_src = bm_src.loo(l_iter_src).next;
        if l_iter_src == l_first_src { break; }
    }
    let f_dst = crate::core::bm_face_create(bm_dst, &vtar, &edar, None, 0);
    slot_map_insert(op, true, "face_map.out", (BM_FACE, f_src), (BM_FACE, f_dst));
    slot_map_insert(op, true, "face_map.out", (BM_FACE, f_dst), (BM_FACE, f_src));
    bmo_elem_flag_enable(bm_dst, (BM_FACE, f_dst), DUPE_NEW);
    f_dst
}

/// `bmo_mesh_copy`.
fn bmo_mesh_copy(op: &mut BMOperator, bm_dst: &mut BMesh, bm_src: &mut BMesh) {
    let use_edge_flip_from_face = bmo_slot_bool_get(op, false, "use_edge_flip_from_face");
    let mut vhash: HashMap<usize, usize> = HashMap::new();
    let mut ehash: HashMap<usize, usize> = HashMap::new();
    let same_mesh = std::ptr::eq(bm_dst, bm_src);
    /* verts */
    for v in crate::iterators::VertsOfMesh::new(bm_src).collect::<Vec<_>>() {
        if bmo_elem_flag_test(bm_src, (BM_VERT, v), DUPE_INPUT)
            && !bmo_elem_flag_test(bm_src, (BM_VERT, v), DUPE_DONE)
        {
            let mut isolated = true;
            for f in crate::iterators::FacesOfVert::new(bm_src, v) {
                if bmo_elem_flag_test(bm_src, (BM_FACE, f), DUPE_INPUT) {
                    isolated = false;
                    break;
                }
            }
            if isolated {
                for e in crate::iterators::EdgesOfVert::new(bm_src, v) {
                    if bmo_elem_flag_test(bm_src, (BM_EDGE, e), DUPE_INPUT) {
                        isolated = false;
                        break;
                    }
                }
            }
            let v2 = bmo_vert_copy(op, bm_dst, v, &mut vhash);
            if isolated {
                slot_map_insert(op, true, "isovert_map.out", (BM_VERT, v), (BM_VERT, v2));
            }
            bmo_elem_flag_enable(bm_src, (BM_VERT, v), DUPE_DONE);
        }
    }
    /* edges */
    for e in crate::iterators::EdgesOfMesh::new(bm_src).collect::<Vec<_>>() {
        if bmo_elem_flag_test(bm_src, (BM_EDGE, e), DUPE_INPUT)
            && !bmo_elem_flag_test(bm_src, (BM_EDGE, e), DUPE_DONE)
        {
            let (v1, v2) = (bm_src.edge(e).v1, bm_src.edge(e).v2);
            for v in [v1, v2] {
                if !bmo_elem_flag_test(bm_src, (BM_VERT, v), DUPE_DONE) {
                    bmo_vert_copy(op, bm_dst, v, &mut vhash);
                    bmo_elem_flag_enable(bm_src, (BM_VERT, v), DUPE_DONE);
                }
            }
            let bm_src_ref: &BMesh = bm_src;
            bmo_edge_copy(op, bm_dst, bm_src_ref, e, &vhash, &mut ehash, use_edge_flip_from_face);
            bmo_elem_flag_enable(bm_src, (BM_EDGE, e), DUPE_DONE);
        }
    }
    /* faces */
    for f in crate::iterators::FacesOfMesh::new(bm_src).collect::<Vec<_>>() {
        if bmo_elem_flag_test(bm_src, (BM_FACE, f), DUPE_INPUT) {
            for v in crate::iterators::VertsOfFace::new(bm_src, f).collect::<Vec<_>>() {
                if !bmo_elem_flag_test(bm_src, (BM_VERT, v), DUPE_DONE) {
                    bmo_vert_copy(op, bm_dst, v, &mut vhash);
                    bmo_elem_flag_enable(bm_src, (BM_VERT, v), DUPE_DONE);
                }
            }
            for e in crate::iterators::EdgesOfFace::new(bm_src, f).collect::<Vec<_>>() {
                if !bmo_elem_flag_test(bm_src, (BM_EDGE, e), DUPE_DONE) {
                    let bm_src_ref: &BMesh = bm_src;
                    bmo_edge_copy(op, bm_dst, bm_src_ref, e, &vhash, &mut ehash,
                                  use_edge_flip_from_face);
                    bmo_elem_flag_enable(bm_src, (BM_EDGE, e), DUPE_DONE);
                }
            }
            let bm_src_ref: &BMesh = bm_src;
            bmo_face_copy(op, bm_dst, bm_src_ref, f, &vhash, &ehash);
            bmo_elem_flag_enable(bm_src, (BM_FACE, f), DUPE_DONE);
        }
    }
    let _ = same_mesh; /* use_select_history remap PORT_PENDING */
}

/// `bmo_duplicate_exec`.
pub fn bmo_duplicate_exec(bm: &mut BMesh, op: &mut BMOperator) {
    bmo_slot_buffer_flag_enable(bm, op, false, "geom", DUPE_INPUT);
    /* bm_dst == bm (no external dest support yet) */
    let bm_src = unsafe { &mut *(bm as *mut BMesh) };
    bmo_mesh_copy(op, bm, bm_src);
    /* geom_orig.out = copy of input geom slot */
    let geom_in: Vec<BMElemRef> = bmo_slot_buffer(op, false, "geom").to_vec();
    crate::operators::slot_get_mut(&mut op.slots_out, "geom_orig.out").buf = geom_in;
    bmo_slot_buffer_from_enabled_flag(
        bm, op, true, "geom.out",
        (1 << BM_VERT) | (1 << BM_EDGE) | (1 << BM_FACE), DUPE_NEW);
}

const SPLIT_INPUT: u8 = 1;

/// `bmo_split_exec`.
pub fn bmo_split_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let use_only_faces = bmo_slot_bool_get(op, false, "use_only_faces");
    /* run duplicate on the same geom */
    let mut dupeop = bmo_op_init(bm, &crate::opdefines::OPDEF_DUPLICATE, op.flag);
    let geom: Vec<BMElemRef> = bmo_slot_buffer(op, false, "geom").to_vec();
    crate::operators::slot_get_mut(&mut dupeop.slots_in, "geom").buf = geom;
    bmo_op_exec(bm, &mut dupeop, crate::bmo::bmo_dupe::bmo_duplicate_exec);
    /* remap active face */
    if bm.act_face != BM_NIL {
        if let Some((ht, idx)) = bmo_slot_map_get(&dupeop, true, "face_map.out", (BM_FACE, bm.act_face)) {
            if ht == BM_FACE { bm.act_face = idx; }
        }
    }
    bmo_slot_buffer_flag_enable(bm, op, false, "geom", SPLIT_INPUT);
    if use_only_faces {
        for e in crate::iterators::EdgesOfMesh::new(bm).collect::<Vec<_>>() {
            let mut found = false;
            for f in crate::iterators::FacesOfEdge::new(bm, e) {
                if !bmo_elem_flag_test(bm, (BM_FACE, f), SPLIT_INPUT) {
                    found = true;
                    break;
                }
            }
            if !found {
                bmo_elem_flag_enable(bm, (BM_EDGE, e), SPLIT_INPUT);
            }
        }
        for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
            let mut found = false;
            for e in crate::iterators::EdgesOfVert::new(bm, v) {
                if !bmo_elem_flag_test(bm, (BM_EDGE, e), SPLIT_INPUT) {
                    found = true;
                    break;
                }
            }
            if !found {
                bmo_elem_flag_enable(bm, (BM_VERT, v), SPLIT_INPUT);
            }
        }
    }
    /* copy dupe outputs into split outputs */
    let geom_out: Vec<BMElemRef> = bmo_slot_buffer(&dupeop, true, "geom.out").to_vec();
    crate::operators::slot_get_mut(&mut op.slots_out, "geom.out").buf = geom_out;
    let iso_map = dupeop.slots_out.iter()
        .find(|s| s.name == "isovert_map.out").map(|s| s.map.clone()).unwrap_or_default();
    crate::operators::slot_get_mut(&mut op.slots_out, "isovert_map.out").map = iso_map;
    /* delete original SPLIT_INPUT faces, keeping boundary verts/edges */
    let boundary_keys: Vec<BMElemRef> = {
        let mut ks = Vec::new();
        if let Some(s) = dupeop.slots_out.iter().find(|s| s.name == "boundary_map.out") {
            ks.extend(s.map.keys().copied());
        }
        ks
    };
    for (ht, idx) in boundary_keys {
        bmo_elem_flag_enable(bm, (ht, idx), SPLIT_INPUT);
    }
    crate::delete::bmo_mesh_delete_oflag_context(bm, SPLIT_INPUT, crate::delete::DEL_FACES, None);
    bmo_op_finish(bm, dupeop);
}

const DEL_INPUT: u8 = 1;

/// `bmo_delete_exec` — delete flagged geometry by context.
pub fn bmo_delete_exec(bm: &mut BMesh, op: &mut BMOperator) {
    bmo_slot_buffer_flag_enable(bm, op, false, "geom", DEL_INPUT);
    let context = bmo_slot_int_get(op, false, "context");
    crate::delete::bmo_mesh_delete_oflag_context(bm, DEL_INPUT, context, None);
}

/// `bmo_spin_exec` — extrude or duplicate geometry over N rotated steps.
pub fn bmo_spin_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let cent = bmo_slot_vec_get(op, false, "cent");
    let axis = bmo_slot_vec_get(op, false, "axis").normalized();
    let dvec = bmo_slot_vec_get(op, false, "dvec");
    let use_dvec = !dvec.is_zero();
    let steps = bmo_slot_int_get(op, false, "steps");
    let angle_total = bmo_slot_float_get(op, false, "angle");
    let do_dupli = bmo_slot_bool_get(op, false, "use_duplicate");
    let use_normal_flip = bmo_slot_bool_get(op, false, "use_normal_flip");
    let space = op.slots_in.iter().find(|s| s.name == "space")
        .map(|s| s.mat4).unwrap_or(blenders_math::matrix::Mat4::IDENTITY);
    let use_merge = bmo_slot_bool_get(op, false, "use_merge") && steps >= 3 && !do_dupli;

    /* original-index tables (C stashes the index inside v->no[0] bits) */
    let vtable: Option<Vec<usize>> = if use_merge {
        Some(crate::iterators::VertsOfMesh::new(bm).collect())
    } else {
        None
    };
    let mut vtable_coords: Option<Vec<Vec3>> = if !do_dupli {
        Some(Vec::with_capacity(bm.totvert.max(0) as usize))
    } else {
        None
    };
    if vtable.is_some() || vtable_coords.is_some() {
        let mut i = 0usize;
        for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
            if let Some(vc) = vtable_coords.as_mut() {
                vc.push(bm.vert(v).co);
            }
            /* Evil! store original index in normal bits (duplicated into new verts). */
            bm.vpool[v].no.x = f32::from_bits(i as u32);
            i += 1;
        }
    }

    /* geom_last.out = copy of input geom slot */
    let geom_in: Vec<BMElemRef> = bmo_slot_buffer(op, false, "geom").to_vec();
    slot_get_mut(&mut op.slots_out, "geom_last.out").buf = geom_in;

    for a in 0..steps {
        let step_angle = angle_total * ((a + 1) as f32 / steps as f32);
        let rmat = blenders_math::quaternion::axis_angle_normalized_to_mat3(axis, step_angle);
        let rmat4 = {
            let mut m = blenders_math::matrix::Mat4::IDENTITY;
            for c in 0..3 {
                m.cols[c] = Vec4::new(rmat.cols[c].x, rmat.cols[c].y, rmat.cols[c].z, 0.0);
            }
            m
        };

        if do_dupli {
            /* duplicate original geometry, rotate by total angle */
            let mut dupop = bmo_op_init(bm, &crate::opdefines::OPDEF_DUPLICATE, op.flag);
            let geom: Vec<BMElemRef> = bmo_slot_buffer(op, false, "geom").to_vec();
            slot_get_mut(&mut dupop.slots_in, "geom").buf = geom;
            bmo_op_exec(bm, &mut dupop, bmo_duplicate_exec);
            let geom_out: Vec<BMElemRef> = bmo_slot_buffer(&dupop, true, "geom.out").to_vec();
            let mut rop = bmo_op_init(bm, &crate::opdefines::OPDEF_ROTATE, op.flag);
            slot_get_mut(&mut rop.slots_in, "cent").vec = cent;
            slot_get_mut(&mut rop.slots_in, "matrix").mat4 = rmat4;
            slot_get_mut(&mut rop.slots_in, "space").mat4 = space;
            slot_get_mut(&mut rop.slots_in, "verts").buf = geom_out.clone();
            bmo_op_exec(bm, &mut rop, crate::bmo::bmo_utils::bmo_rotate_exec);
            bmo_op_finish(bm, rop);
            slot_get_mut(&mut op.slots_out, "geom_last.out").buf = geom_out;
            bmo_op_finish(bm, dupop);
        } else {
            /* extrude from last geometry */
            let mut extop = bmo_op_init(bm, &crate::opdefines::OPDEF_EXTRUDE_FACE_REGION, op.flag);
            let last: Vec<BMElemRef> = bmo_slot_buffer(op, true, "geom_last.out").to_vec();
            slot_get_mut(&mut extop.slots_in, "geom").buf = last;
            slot_get_mut(&mut extop.slots_in, "use_keep_orig").b = use_merge;
            slot_get_mut(&mut extop.slots_in, "use_normal_flip").b = use_normal_flip && a == 0;
            slot_get_mut(&mut extop.slots_in, "use_normal_from_adjacent").b = a != 0;
            slot_get_mut(&mut extop.slots_in, "skip_input_flip").b = true;
            bmo_op_exec(bm, &mut extop, crate::bmo::bmo_extrude::bmo_extrude_face_region_exec);
            let mut geom_out: Vec<BMElemRef> =
                bmo_slot_buffer(&extop, true, "geom.out").to_vec();

            if !(use_merge && a == steps - 1) {
                /* Reset each new vert to un-rotated origin before rotating (#148890). */
                if a != 0 {
                    if let Some(vc) = &vtable_coords {
                        for &(ht, idx) in &geom_out {
                            if ht == BM_VERT {
                                let index = bm.vert(idx).no.x.to_bits() as usize;
                                if index < vc.len() {
                                    bm.vpool[idx].co = vc[index];
                                }
                            }
                        }
                    }
                }
                let mut rop = bmo_op_init(bm, &crate::opdefines::OPDEF_ROTATE, op.flag);
                slot_get_mut(&mut rop.slots_in, "cent").vec = cent;
                slot_get_mut(&mut rop.slots_in, "matrix").mat4 = rmat4;
                slot_get_mut(&mut rop.slots_in, "space").mat4 = space;
                slot_get_mut(&mut rop.slots_in, "verts").buf = geom_out.clone();
                bmo_op_exec(bm, &mut rop, crate::bmo::bmo_utils::bmo_rotate_exec);
                bmo_op_finish(bm, rop);
                slot_get_mut(&mut op.slots_out, "geom_last.out").buf = geom_out;
            } else {
                /* Merge first/last verts+edges in place (keeping geom.out state). */
                if let Some(vt) = &vtable {
                    let mut i = 0;
                    while i < geom_out.len() {
                        if geom_out[i].0 == BM_VERT {
                            let v_src = geom_out[i].1;
                            let index = bm.vert(v_src).no.x.to_bits() as usize;
                            if index < vt.len() {
                                let v_dst = vt[index];
                                bm_vert_splice(bm, v_dst, v_src);
                                geom_out.swap_remove(i);
                                continue;
                            }
                        }
                        i += 1;
                    }
                }
                let mut i = 0;
                while i < geom_out.len() {
                    if geom_out[i].0 == BM_EDGE {
                        let e_src = geom_out[i].1;
                        let e_dst = crate::query::bm_edge_find_double(bm, e_src);
                        if e_dst != crate::BM_NIL {
                            bm_edge_splice(bm, e_dst, e_src);
                            geom_out.swap_remove(i);
                            continue;
                        }
                    }
                    i += 1;
                }
                /* Full copies of faces may cause overlap. */
                let mut i = 0;
                while i < geom_out.len() {
                    if geom_out[i].0 == BM_FACE {
                        let f_src = geom_out[i].1;
                        let f_dst = crate::query::bm_face_find_double(bm, f_src);
                        if f_dst != crate::BM_NIL {
                            bm_face_kill(bm, f_src);
                            geom_out.swap_remove(i);
                            continue;
                        }
                    }
                    i += 1;
                }
            }
            bmo_op_finish(bm, extop);
        }

        if use_dvec {
            let dvec_step = rmat * dvec * ((a + 1) as f32);
            let mut top = bmo_op_init(bm, &crate::opdefines::OPDEF_TRANSLATE, op.flag);
            slot_get_mut(&mut top.slots_in, "vec").vec = dvec_step;
            slot_get_mut(&mut top.slots_in, "space").mat4 = space;
            let last: Vec<BMElemRef> = bmo_slot_buffer(op, true, "geom_last.out").to_vec();
            slot_get_mut(&mut top.slots_in, "verts").buf = last;
            bmo_op_exec(bm, &mut top, crate::bmo::bmo_utils::bmo_translate_exec);
            bmo_op_finish(bm, top);
        }
    }
}
