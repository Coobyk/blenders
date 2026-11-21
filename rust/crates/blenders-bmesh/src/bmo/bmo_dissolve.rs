//! Port of `bmo_dissolve.cc` — dissolve_faces / dissolve_verts / dissolve_limit.
//! `dissolve_edges` + `dissolve_degenerate*` remain PORT_PENDING.
//! `BMO_*_flag` oflags → `head.api_flag`; BMW_ISLAND_MANIFOLD walk via walkers.

use crate::core::*;
use crate::operators::*;
use crate::query::*;
use crate::structure::bm_edge_other_vert;
use crate::walkers::{BMWType, BMWalker};
use crate::{BMesh, BM_EDGE, BM_FACE, BM_VERT, BM_NIL};

const EDGE_MARK: u8 = 1;
const EDGE_ISGC: u8 = 8;
const VERT_MARK: u8 = 1;
const VERT_MARK_PAIR: u8 = 4;
const VERT_TAG: u8 = 2;
const VERT_ISGC: u8 = 8;
const VERT_MARK_TEAR: u8 = 16;
const FACE_MARK: u8 = 1;
const FACE_ORIG: u8 = 2;
const FACE_NEW: u8 = 4;
const FACE_TAG: u8 = 8;
const DEL_FACES: i32 = 3;

/// `bm_vert_collapse_edge_and_merge` — merge hidden flags, collapse,
/// never leave visible edge on hidden verts.
fn bm_vert_collapse_edge_and_merge(bm: &mut BMesh, v: usize, do_del: bool) -> usize {
    let Some((e0, e1)) = bm_vert_edge_pair(bm, v) else { return BM_NIL };
    /* BM_elem_flag_merge_ex(e0, e1, HIDDEN): e0 |= e1 & HIDDEN */
    bm.epool[e0].head.hflag |= bm.edge(e1).head.hflag & crate::BM_ELEM_HIDDEN;
    let e = bm.vert(v).e;
    let e_new = crate::mods::bm_vert_collapse_edge(bm, e, v, do_del, true, true);
    if e_new != BM_NIL {
        if bm.edge(e_new).head.hflag & crate::BM_ELEM_HIDDEN == 0
            && (bm.vert(bm.edge(e_new).v1).head.hflag & crate::BM_ELEM_HIDDEN != 0
                || bm.vert(bm.edge(e_new).v2).head.hflag & crate::BM_ELEM_HIDDEN != 0)
        {
            if bm.edge(e_new).head.hflag & crate::BM_ELEM_SELECT != 0 {
                crate::marking::bm_edge_select_set_noflush(bm, e_new, false);
            }
            bm.epool[e_new].head.hflag |= crate::BM_ELEM_HIDDEN;
        }
    }
    e_new
}

/// `bm_face_split` — split tagged-vert faces on untagged diagonals,
/// optionally killing the vert's remaining edges after.
fn bm_face_split_op(bm: &mut BMesh, oflag: u8, use_edge_delete: bool) {
    let mut edge_delete_verts: Vec<usize> = Vec::new();
    for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.vert(v).head.api_flag & oflag == 0 { continue; }
        if !bm_vert_is_edge_pair(bm, v) {
            for l in crate::iterators::LoopsOfVert::new(bm, v).collect::<Vec<_>>() {
                if bm.face(bm.loo(l).f).len > 3 {
                    let ln = bm.loo(l).next;
                    let lp = bm.loo(l).prev;
                    if bm.vert(bm.loo(ln).v).head.api_flag & oflag == 0
                        && bm.vert(bm.loo(lp).v).head.api_flag & oflag == 0
                    {
                        crate::mods::bm_face_split(bm, bm.loo(l).f, ln, lp, None, true);
                    }
                }
            }
            if use_edge_delete {
                edge_delete_verts.push(v);
            }
        }
    }
    if use_edge_delete {
        for v in edge_delete_verts {
            while bm.vert(v).e != BM_NIL {
                let e = bm.vert(v).e;
                bm_edge_kill(bm, e);
            }
        }
    }
}

/// `bmo_dissolve_faces_exec`.
pub fn bmo_dissolve_faces_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let use_verts = bmo_slot_bool_get(op, false, "use_verts");
    if use_verts {
        for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
            if !bm_vert_is_edge_pair(bm, v) {
                bm.vpool[v].head.api_flag |= VERT_MARK;
            } else {
                bm.vpool[v].head.api_flag &= !VERT_MARK;
            }
        }
    }
    bmo_slot_buffer_flag_enable(bm, op, false, "faces", FACE_MARK | FACE_TAG);
    let faces_in: Vec<usize> = op.slots_in.iter().find(|s| s.name == "faces")
        .map(|s| s.buf.iter().filter(|(ht, _)| *ht == BM_FACE).map(|(_, i)| *i).collect())
        .unwrap_or_default();
    let mut regions: Vec<Vec<usize>> = Vec::new();
    for f in faces_in {
        if bm.face(f).head.api_flag & FACE_TAG == 0 { continue; }
        /* BMW_ISLAND_MANIFOLD over FACE_MARK-masked faces */
        let mut w = BMWalker::new(BMWType::IslandManifold, 0, 0);
        w.mask_face = FACE_MARK;
        w.begin(bm, BM_FACE, f);
        let faces: Vec<usize> = crate::walkers::bmw_walk_all(bm, &mut w)
            .into_iter().filter(|(ht, _)| *ht == BM_FACE).map(|(_, i)| i).collect();
        if faces.len() < 2 { continue; }
        for &face in &faces {
            bm.fpool[face].head.api_flag &= !FACE_TAG;
            bm.fpool[face].head.api_flag |= FACE_ORIG;
        }
        regions.push(faces);
    }
    let mut totface_target = bm.totface;
    for faces in regions {
        let faces_len = faces.len() as i32;
        let mut f_double = BM_NIL;
        let mut f_new = bm_faces_join(bm, &faces, true, Some(&mut f_double));
        if f_new != BM_NIL {
            totface_target -= faces_len - 1;
            if f_double != BM_NIL {
                bm_face_kill(bm, f_new);
                totface_target -= 1;
                f_new = f_double;
            }
            bm.fpool[f_new].head.api_flag &= !FACE_ORIG;
            bm.fpool[f_new].head.api_flag |= FACE_NEW;
        } else {
            for face in faces {
                bm.fpool[face].head.api_flag &= !FACE_ORIG;
            }
        }
    }
    if totface_target != bm.totface {
        crate::operators::bmo_op_callf(
            bm, op.flag, "delete",
            &[CallfArg::OFlag("geom".into(), 1 << BM_FACE, FACE_ORIG),
              CallfArg::Int("context".into(), DEL_FACES)]);
    }
    if use_verts {
        for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
            if bm.vert(v).head.api_flag & VERT_MARK == 0 { continue; }
            if bm_vert_is_edge_pair(bm, v) {
                bm_vert_collapse_edge_and_merge(bm, v, true);
            }
        }
    }
    bmo_slot_buffer_from_enabled_flag(
        bm, op, true, "region.out", 1 << BM_FACE, FACE_NEW);
}

/// `bmo_dissolve_verts_exec`.
pub fn bmo_dissolve_verts_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let use_face_split = bmo_slot_bool_get(op, false, "use_face_split");
    let use_boundary_tear = bmo_slot_bool_get(op, false, "use_boundary_tear");
    let verts_in: Vec<usize> = op.slots_in.iter().find(|s| s.name == "verts")
        .map(|s| s.buf.iter().filter(|(ht, _)| *ht == BM_VERT).map(|(_, i)| *i).collect())
        .unwrap_or_default();
    for &v in &verts_in {
        bm.vpool[v].head.api_flag |= VERT_MARK | VERT_ISGC;
    }
    if use_face_split {
        bm_face_split_op(bm, VERT_MARK, false);
    }
    if use_boundary_tear {
        for &v in &verts_in {
            if bm.vert(v).e == BM_NIL { continue; }
            if !bm_vert_is_edge_pair(bm, v) {
                for e in crate::iterators::EdgesOfVert::new(bm, v) {
                    if crate::delete::bm_edge_is_boundary(bm, e) {
                        bm.vpool[v].head.api_flag |= VERT_MARK_TEAR;
                        break;
                    }
                }
            }
        }
        bm_face_split_op(bm, VERT_MARK_TEAR, true);
    }
    for &v in &verts_in {
        if bm.vert(v).e == BM_NIL { continue; }
        let mut e_first = BM_NIL;
        for l_first in crate::iterators::LoopsOfVert::new(bm, v).collect::<Vec<_>>() {
            let mut l_iter = l_first;
            loop {
                let (lv, le) = (bm.loo(l_iter).v, bm.loo(l_iter).e);
                bm.vpool[lv].head.api_flag |= VERT_ISGC;
                bm.epool[le].head.api_flag |= EDGE_ISGC;
                l_iter = bm.loo(l_iter).next;
                if l_iter == l_first { break; }
            }
            e_first = bm.loo(l_first).e;
        }
        if e_first != BM_NIL {
            let mut e = e_first;
            loop {
                let e_next = crate::structure::bmesh_disk_edge_next(bm, e, v);
                if crate::edgenet::edge_is_wire(bm, e) {
                    bm_edge_kill(bm, e);
                }
                e = e_next;
                if e == e_first { break; }
            }
        }
    }
    for &v in &verts_in {
        if bm.vert(v).e != BM_NIL && bm_vert_is_edge_pair(bm, v) {
            bm.vpool[v].head.api_flag |= VERT_MARK_PAIR;
        }
    }
    for &v in &verts_in {
        if bm.vert(v).e == BM_NIL { continue; }
        if bm.vert(v).head.api_flag & VERT_MARK_PAIR == 0 {
            for e in crate::iterators::EdgesOfVert::new(bm, v).collect::<Vec<_>>() {
                if let Some((l_a, l_b)) = bm_edge_loop_pair(bm, e) {
                    crate::mods::bm_faces_join_pair(bm, l_a, l_b, false, None);
                }
            }
        }
    }
    /* kill loop-less ISGC edges */
    for e in crate::iterators::EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.edge(e).l == BM_NIL && bm.edge(e).head.api_flag & EDGE_ISGC != 0 {
            bm_edge_kill(bm, e);
        }
    }
    for &v in &verts_in {
        if bm.vert(v).e != BM_NIL && bm_vert_is_edge_pair(bm, v) {
            bm_vert_collapse_edge_and_merge(bm, v, false);
        }
    }
    for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.vert(v).e == BM_NIL && bm.vert(v).head.api_flag & VERT_ISGC != 0 {
            bm_vert_kill(bm, v);
        }
    }
}

/// `bmo_dissolve_limit_exec`.
pub fn bmo_dissolve_limit_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let einput: Vec<usize> = op.slots_in.iter().find(|s| s.name == "edges")
        .map(|s| s.buf.iter().filter(|(ht, _)| *ht == BM_EDGE).map(|(_, i)| *i).collect())
        .unwrap_or_default();
    let mut vinput: Vec<Option<usize>> = op.slots_in.iter().find(|s| s.name == "verts")
        .map(|s| s.buf.iter().filter(|(ht, _)| *ht == BM_VERT)
            .map(|(_, i)| Some(*i)).collect())
        .unwrap_or_default();
    let angle_max = std::f32::consts::FRAC_PI_2;
    let angle_limit = angle_max.min(bmo_slot_float_get(op, false, "angle_limit"));
    let do_dissolve_boundaries = bmo_slot_bool_get(op, false, "use_dissolve_boundaries");
    let delimit = bmo_slot_int_get(op, false, "delimit") as u32;
    crate::tools::bmesh_decimate_dissolve::bm_mesh_decimate_dissolve_ex(
        bm, angle_limit, do_dissolve_boundaries, delimit,
        &mut vinput, &einput, FACE_NEW);
    bmo_slot_buffer_from_enabled_flag(
        bm, op, true, "region.out", 1 << BM_FACE, FACE_NEW);
}

const EDGE_TAG: u8 = 2;
const EDGE_CHAIN: u8 = 16;

/// `bmo_vert_calc_edge_angle_blended` — edge-pair angle blended with the
/// normal-angled axis angle when the vert has a manifold edge.
fn bmo_vert_calc_edge_angle_blended(bm: &BMesh, v: usize) -> f32 {
    let Some((e0, e1)) = bm_vert_edge_pair(bm, v) else {
        return std::f32::consts::PI;
    };
    let v_a = bm_edge_other_vert(bm, e0, v);
    let v_b = bm_edge_other_vert(bm, e1, v);
    let mut angle = std::f32::consts::PI
        - blenders_math::vector::angle_v3v3v3(bm.vert(v_a).co, bm.vert(v).co, bm.vert(v_b).co);
    if let Some((f_a, f_b)) = bm_edge_face_pair(bm, bm.vert(v).e) {
        let no_a = bm.face(f_a).no;
        let no_b = bm.face(f_b).no;
        let raw_factor = (-no_a.dot(no_b)).clamp(0.0, 1.0);
        let normal_angle = std::f32::consts::PI
            - blenders_math::vector::angle_on_axis_v3v3v3(
                bm.vert(v_a).co, bm.vert(v).co, bm.vert(v_b).co, bm.vert(v).no);
        angle = normal_angle + (angle - normal_angle) * raw_factor;
    }
    angle
}

/// `bmo_find_end_of_chain` — walk edge-pair verts, tagging edges; returns
/// the chain end, or BM_NIL on cycles/already-tagged.
fn bmo_find_end_of_chain(bm: &mut BMesh, mut e: usize, mut v: usize, edge_oflag: u8) -> usize {
    let v_init = v;
    while bm_vert_is_edge_pair(bm, v) {
        e = crate::structure::bmesh_disk_edge_next(bm, e, v);
        v = bm_edge_other_vert(bm, e, v);
        if bm.edge(e).head.api_flag & edge_oflag != 0 { return BM_NIL; }
        bm.epool[e].head.api_flag |= edge_oflag;
        debug_assert_ne!(v, v_init);
        if v == v_init { return BM_NIL; }
    }
    v
}

/// `bmo_vert_touches_unselected_face`.
fn bmo_vert_touches_unselected_face(bm: &BMesh, v: usize) -> bool {
    if bm.vert(v).head.api_flag & VERT_MARK != 0 { return false; }
    for l_a in crate::iterators::LoopsOfVert::new(bm, v) {
        let l_b = bm_loop_other_edge_loop(bm, l_a, v);
        if bm.edge(bm.loo(l_a).e).head.api_flag & EDGE_TAG == 0
            && bm.edge(bm.loo(l_b).e).head.api_flag & EDGE_TAG == 0
        { return true; }
    }
    false
}

/// `bmo_vert_tagged_edges_count_at_most`.
fn bmo_vert_tagged_edges_count_at_most(bm: &BMesh, v: usize, edge_oflag: u8, max: i32) -> i32 {
    let mut n = 0;
    for e in crate::iterators::EdgesOfVert::new(bm, v) {
        if bm.edge(e).head.api_flag & edge_oflag != 0 { n += 1; }
        if n == max { return n; }
    }
    n
}

/// `bmo_dissolve_edges_init` — sets default angle_threshold = PI.
pub fn bmo_dissolve_edges_init(op: &mut BMOperator) {
    bmo_slot_float_set(op, false, "angle_threshold", std::f32::consts::PI);
}

/// `bmo_dissolve_edges_exec`.
pub fn bmo_dissolve_edges_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let angle_epsilon = 0.0001f32.to_degrees();
    let angle_threshold = bmo_slot_float_get(op, false, "angle_threshold");
    let use_verts = bmo_slot_bool_get(op, false, "use_verts")
        && angle_threshold > angle_epsilon;
    let dissolve_all = angle_threshold > std::f32::consts::PI - angle_epsilon;
    let use_face_split = bmo_slot_bool_get(op, false, "use_face_split");
    let use_preserve_quads = bmo_slot_bool_get(op, false, "use_preserve_quads");
    if use_face_split || use_verts {
        bmo_slot_buffer_flag_enable(bm, op, false, "edges", EDGE_TAG);
    }
    let edges_in: Vec<usize> = op.slots_in.iter().find(|s| s.name == "edges")
        .map(|s| s.buf.iter().filter(|(ht, _)| *ht == BM_EDGE).map(|(_, i)| *i).collect())
        .unwrap_or_default();
    for &e in &edges_in {
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        if bm_vert_is_edge_pair(bm, v1) || bm_vert_is_edge_pair(bm, v2) {
            bm.epool[e].head.api_flag |= EDGE_CHAIN;
        }
        if let Some((f0, f1)) = bm_edge_face_pair(bm, e) {
            for f in [f0, f1] {
                let l_first = bm.face(f).l_first;
                let mut l_iter = l_first;
                loop {
                    let (lv, le) = (bm.loo(l_iter).v, bm.loo(l_iter).e);
                    bm.vpool[lv].head.api_flag |= VERT_ISGC;
                    bm.epool[le].head.api_flag |= EDGE_ISGC;
                    l_iter = bm.loo(l_iter).next;
                    if l_iter == l_first { break; }
                }
            }
            if use_verts && bm.edge(e).head.api_flag & EDGE_CHAIN != 0 {
                bmo_find_end_of_chain(bm, e, v1, EDGE_TAG);
                bmo_find_end_of_chain(bm, e, v2, EDGE_TAG);
            }
        }
    }
    if use_verts {
        for &e in &edges_in {
            let Some((f0, f1)) = bm_edge_face_pair(bm, e) else { continue };
            let (len0, len1) = (bm.face(f0).len, bm.face(f1).len);
            for i in 0..2 {
                let mut v_edge = if i == 0 { bm.edge(e).v1 } else { bm.edge(e).v2 };
                if use_preserve_quads && len0 == 3 && len1 == 3
                    && bmo_vert_tagged_edges_count_at_most(bm, v_edge, EDGE_TAG, 2) == 1
                { continue; }
                if bm_vert_is_edge_pair(bm, v_edge) {
                    v_edge = bmo_find_end_of_chain(bm, e, v_edge, EDGE_CHAIN);
                    if v_edge == BM_NIL { continue; }
                }
                if bmo_vert_touches_unselected_face(bm, v_edge)
                    && bmo_vert_tagged_edges_count_at_most(bm, v_edge, EDGE_TAG, 2) != 1
                { continue; }
                bm.vpool[v_edge].head.api_flag |= VERT_MARK;
            }
        }
    }
    if use_face_split {
        for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
            let mut untag_count = 0;
            for e in crate::iterators::EdgesOfVert::new(bm, v) {
                if bm.edge(e).head.api_flag & EDGE_TAG == 0 { untag_count += 1; }
            }
            if untag_count <= 2 {
                bm.vpool[v].head.api_flag |= VERT_TAG;
            }
        }
        bm_face_split_op(bm, VERT_TAG, false);
    }
    for &e in &edges_in {
        if bm.edge(e).l != BM_NIL {
            if let Some((l_a, l_b)) = bm_edge_loop_pair(bm, e) {
                crate::mods::bm_faces_join_pair(bm, l_a, l_b, false, None);
            }
        }
    }
    for e in crate::iterators::EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.edge(e).l == BM_NIL && bm.edge(e).head.api_flag & EDGE_ISGC != 0 {
            bm_edge_kill(bm, e);
        }
    }
    for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.vert(v).e == BM_NIL && bm.vert(v).head.api_flag & VERT_ISGC != 0 {
            bm_vert_kill(bm, v);
        }
    }
    if use_verts {
        for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
            if bm.vert(v).head.api_flag & VERT_MARK == 0 { continue; }
            let Some((e0, e1)) = bm_vert_edge_pair(bm, v) else {
                bm.vpool[v].head.api_flag &= !VERT_MARK;
                continue;
            };
            if dissolve_all { continue; }
            if bm.edge(e0).head.api_flag & EDGE_CHAIN != 0
                || bm.edge(e1).head.api_flag & EDGE_CHAIN != 0
            { continue; }
            if bmo_vert_calc_edge_angle_blended(bm, v) > angle_threshold - angle_epsilon {
                bm.vpool[v].head.api_flag &= !VERT_MARK;
            }
        }
        for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
            if bm.vert(v).head.api_flag & VERT_MARK == 0 { continue; }
            if !bm_vert_is_edge_pair(bm, v) { continue; }
            bm_vert_collapse_edge_and_merge(bm, v, true);
        }
    }
}

const EDGE_MARK2: u8 = 1;
const EDGE_COLLAPSE: u8 = 2;

/// `bm_mesh_edge_collapse_flagged` — collapse EDGE_COLLAPSE-flagged edges.
fn bm_mesh_edge_collapse_flagged(bm: &mut BMesh, flag: i32, oflag: u8) {
    let mut cop = bmo_op_init(bm, &crate::opdefines::OPDEF_COLLAPSE, flag);
    if let Some(si) = cop.slots_in.iter().position(|s| s.name == "edges") {
        cop.slots_in[si].buf = crate::iterators::EdgesOfMesh::new(bm)
            .filter(|&e| bm.edge(e).head.api_flag & oflag != 0)
            .map(|e| (crate::BM_EDGE, e))
            .collect();
    }
    if let Some(si) = cop.slots_in.iter().position(|s| s.name == "uvs") {
        cop.slots_in[si].b = true;
    }
    bmo_op_exec(bm, &mut cop, crate::bmo::bmo_removedoubles::bmo_collapse_exec);
    bmo_op_finish(bm, cop);
}

/// `bmo_dissolve_degenerate_exec` — collapse zero-length edges + clip degenerate ears.
pub fn bmo_dissolve_degenerate_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let dist = bmo_slot_float_get(op, false, "dist");
    let dist_sq = dist * dist;

    bmo_slot_buffer_flag_enable(bm, op, false, "edges", EDGE_MARK2);

    /* collapse zero length edges (also catches zero-area faces) */
    let mut found = false;
    for e in crate::iterators::EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.edge(e).head.api_flag & EDGE_MARK2 != 0 {
            let len_sq =
                (bm.vert(bm.edge(e).v1).co - bm.vert(bm.edge(e).v2).co).length_squared();
            if len_sq < dist_sq {
                bm.epool[e].head.api_flag |= EDGE_COLLAPSE;
                found = true;
            }
        }
        /* clear all loop tags (checked later) */
        if bm.edge(e).l != BM_NIL {
            let l_first = bm.edge(e).l;
            let mut l_iter = l_first;
            loop {
                bm.lpool[l_iter].head.hflag &= !crate::BM_ELEM_TAG;
                l_iter = bm.loo(l_iter).radial_next;
                if l_iter == l_first {
                    break;
                }
            }
        }
    }
    if found {
        bm_mesh_edge_collapse_flagged(bm, op.flag, EDGE_COLLAPSE);
    }

    /* clip degenerate ears from the face */
    let mut found = false;
    for e in crate::iterators::EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.edge(e).l == BM_NIL || bm.edge(e).head.api_flag & EDGE_MARK2 == 0 {
            continue;
        }
        let mut l_first = bm.edge(e).l;
        let mut l_iter = l_first;
        loop {
            let (lp_v, lp_e, lp_f) = (bm.loo(l_iter).v, bm.loo(l_iter).e, bm.loo(l_iter).f);
            if bm.loo(l_iter).head.hflag & crate::BM_ELEM_TAG == 0 {
                bm.lpool[l_iter].head.hflag |= crate::BM_ELEM_TAG;
                let l_prev = bm.loo(l_iter).prev;
                let l_next = bm.loo(l_iter).next;
                if bm.edge(bm.loo(l_prev).e).head.api_flag & EDGE_MARK2 != 0
                    && bm.edge(bm.loo(l_iter).e).head.api_flag & EDGE_COLLAPSE == 0
                    && bm.edge(bm.loo(l_prev).e).head.api_flag & EDGE_COLLAPSE == 0
                {
                    /* test if the face's loop (ear) is degenerate */
                    let co_v = bm.vert(lp_v).co;
                    let dir_prev = bm.vert(bm.loo(l_prev).v).co - co_v;
                    let dir_next = bm.vert(bm.loo(l_next).v).co - co_v;
                    let len_prev = dir_prev.length();
                    let len_next = dir_next.length();
                    if len_prev > f32::EPSILON && len_next > f32::EPSILON {
                        let dir_prev_n = dir_prev / len_prev;
                        let dir_next_n = dir_next / len_next;
                        if (dir_prev_n - dir_next_n).length() * len_prev.min(len_next) <= dist {
                            let mut reset = false;
                            if (len_prev - len_next).abs() <= dist {
                                /* both edges the same length */
                                if bm.face(lp_f).len == 3 {
                                    let en = bm.loo(l_next).e;
                                    bm.epool[en].head.api_flag |= EDGE_COLLAPSE;
                                    found = true;
                                } else {
                                    /* add a joining edge and tag for removal */
                                    let mut l_split = BM_NIL;
                                    if crate::mods::bm_face_split(
                                        bm, lp_f, l_prev, l_next,
                                        Some(&mut l_split), true,
                                    ) != BM_NIL
                                    {
                                        let es = bm.loo(l_split).e;
                                        bm.epool[es].head.api_flag |= EDGE_COLLAPSE;
                                        found = true;
                                        reset = true;
                                    }
                                }
                            } else if len_prev < len_next {
                                /* split 'l_iter->e', then join the vert with next */
                                let mut _e_new = BM_NIL;
                                crate::mods::bm_edge_split(
                                    bm, lp_e, lp_v, Some(&mut _e_new),
                                    len_prev / len_next);
                                let mut l_split = BM_NIL;
                                if crate::mods::bm_face_split(
                                    bm, lp_f, l_prev, l_next,
                                    Some(&mut l_split), true,
                                ) != BM_NIL
                                {
                                    let es = bm.loo(l_split).e;
                                    bm.epool[es].head.api_flag |= EDGE_COLLAPSE;
                                    found = true;
                                }
                                reset = true;
                            } else {
                                /* len_next < len_prev: split 'l_iter->prev->e' */
                                let mut _e_new = BM_NIL;
                                crate::mods::bm_edge_split(
                                    bm, bm.loo(l_prev).e, lp_v, Some(&mut _e_new),
                                    len_next / len_prev);
                                let mut l_split = BM_NIL;
                                if crate::mods::bm_face_split(
                                    bm, lp_f, l_prev, l_next,
                                    Some(&mut l_split), true,
                                ) != BM_NIL
                                {
                                    let es = bm.loo(l_split).e;
                                    bm.epool[es].head.api_flag |= EDGE_COLLAPSE;
                                    found = true;
                                }
                                reset = true;
                            }
                            if reset {
                                /* can't track position on the radial edge — restart. */
                                l_first = l_iter;
                            }
                        }
                    }
                }
            }
            l_iter = bm.loo(l_iter).radial_next;
            if l_iter == l_first {
                break;
            }
        }
    }
    if found {
        bm_mesh_edge_collapse_flagged(bm, op.flag, EDGE_COLLAPSE);
    }
}
