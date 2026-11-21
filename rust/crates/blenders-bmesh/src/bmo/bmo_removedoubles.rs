//! Port of `bmo_removedoubles.cc` — `weld_verts` (targetmap weld).
//! `find_doubles`/`remove_doubles` (BVH-based) and the edit-selection
//! remap are PORT_PENDING; `average_vert_data` interp is a no-op
//! (CustomData).

use std::collections::HashMap;
use crate::operators::*;
use blenders_math::vector::Vec3;
use crate::{BMesh, BMEdge, BMFace, BMVert, BM_NIL, BM_EDGE, BM_FACE, BM_VERT};

const ELE_DEL: u8 = 1;
const EDGE_COL: u8 = 2;
const VERT_IN_FACE: u8 = 4;

/// `remdoubles_splitface` — split faces whose loops jump across a
/// welded vert.
fn remdoubles_splitface(
    bm: &mut BMesh,
    f: usize,
    op: &BMOperator,
    slot_targetmap: &str,
) {
    let mut split = false;
    let mut l_double = BM_NIL;
    let mut l_tar = BM_NIL;
    for l in crate::iterators::LoopsOfFace::new(bm, f) {
        let lv = bm.loo(l).v;
        if let Some((ht, v_tar)) = bmo_slot_map_get(op, false, slot_targetmap, (BM_VERT, lv)) {
            if ht == BM_VERT {
                let lt = crate::query::bm_face_vert_share_loop(bm, f, v_tar);
                if lt != BM_NIL && lt != l && !crate::mods::bm_loop_is_adjacent(bm, lt, l) {
                    l_double = l;
                    l_tar = lt;
                    split = true;
                    break;
                }
            }
        }
    }
    if split {
        let f_new = crate::mods::bm_face_split(bm, f, l_double, l_tar, None, false);
        remdoubles_splitface(bm, f, op, slot_targetmap);
        if f_new != BM_NIL {
            remdoubles_splitface(bm, f_new, op, slot_targetmap);
        }
    }
}

/// `remdoubles_createface` — rebuild a face with welded verts swapped in.
/// Returns (face index, created).
fn remdoubles_createface(
    bm: &mut BMesh,
    f: usize,
    op: &BMOperator,
    slot_targetmap: &str,
) -> (usize, bool) {
    let f_len = bm.face(f).len as usize;
    let mut edges: Vec<usize> = Vec::with_capacity(f_len);
    let mut loops: Vec<usize> = Vec::with_capacity(f_len);
    let mut verts: Vec<usize> = Vec::with_capacity(f_len);
    let mut failed = false;

    let map_vert = |bm: &BMesh, v: usize| -> (usize, bool) {
        let is_del = bmo_elem_flag_test(bm, (BM_VERT, v), ELE_DEL);
        if is_del {
            if let Some((_, vt)) = bmo_slot_map_get(op, false, slot_targetmap, (BM_VERT, v)) {
                return (vt, true);
            }
        }
        (v, is_del)
    };

    let l_first = bm.face(f).l_first;
    let mut l_curr = l_first;
    let (mut v_curr, mut is_del_v_curr) = map_vert(bm, bm.loo(l_curr).v);
    loop {
        let l_next = bm.loo(l_curr).next;
        let (v_next, is_del_v_next) = map_vert(bm, bm.loo(l_next).v);
        let mut e_new = BM_NIL;
        if !is_del_v_curr && !is_del_v_next {
            e_new = bm.loo(l_curr).e;
        } else if v_curr == v_next {
            e_new = BM_NIL; /* skip */
        } else {
            e_new = crate::query::bm_edge_exists(bm, v_curr, v_next);
            debug_assert!(e_new != BM_NIL);
        }
        if e_new != BM_NIL {
            if bmo_elem_flag_test(bm, (BM_VERT, v_curr), VERT_IN_FACE) {
                failed = true;
                break;
            }
            bmo_elem_flag_enable(bm, (BM_VERT, v_curr), VERT_IN_FACE);
            edges.push(e_new);
            loops.push(l_curr);
            verts.push(v_curr);
        }
        v_curr = v_next;
        is_del_v_curr = is_del_v_next;
        l_curr = l_next;
        if l_curr == l_first { break; }
    }
    for &v in &verts {
        bmo_elem_flag_disable(bm, (BM_VERT, v), VERT_IN_FACE);
    }
    if failed || verts.len() < 3 {
        return (BM_NIL, false);
    }
    /* check whether the new face would equal the old one */
    if edges.len() == f_len && !failed {
        let mut identical = true;
        for (i, &l) in loops.iter().enumerate() {
            if bm.loo(l).v != verts[i] || bm.loo(l).e != edges[i] {
                identical = false;
                break;
            }
        }
        if identical {
            return (f, false);
        }
    }
    let f_new = crate::core::bm_face_create(bm, &verts, &edges, Some(f), 0);
    (f_new, f_new != BM_NIL)
}

/// `bmo_weld_verts_exec`.
pub fn bmo_weld_verts_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let slot_targetmap = "targetmap";
    let use_centroid = bmo_slot_bool_get(op, false, "use_centroid");
    let _average_vert_data = bmo_slot_bool_get(op, false, "average_vert_data") || use_centroid;

    /* mark welded-away verts */
    let mut clusters: HashMap<usize, Vec<usize>> = HashMap::new();
    for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
        let v_dst = match bmo_slot_map_get(op, false, slot_targetmap, (BM_VERT, v)) {
            Some((_, vd)) => vd,
            None => continue,
        };
        bmo_elem_flag_enable(bm, (BM_VERT, v), ELE_DEL);
        let fl = bm.vert(v).head.hflag & crate::BM_ELEM_HIDDEN;
        bm.vpool[v_dst].head.hflag |= fl;
        if _average_vert_data && v_dst != v {
            clusters.entry(v_dst).or_default().push(v);
        }
    }
    if use_centroid {
        for (v_dst, cluster) in &clusters {
            let mut centroid = bm.vert(*v_dst).co;
            let mut count = 1;
            for &v_dup in cluster {
                centroid = centroid + bm.vert(v_dup).co;
                count += 1;
            }
            bm.vpool[*v_dst].co = centroid / count as f32;
        }
    }
    /* split faces that need it */
    for f in crate::iterators::FacesOfMesh::new(bm).collect::<Vec<_>>() {
        remdoubles_splitface(bm, f, op, slot_targetmap);
    }
    /* rewire edges around welded verts */
    for e in crate::iterators::EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        let mut v1 = bm.edge(e).v1;
        let mut v2 = bm.edge(e).v2;
        let is_del_v1 = bmo_elem_flag_test(bm, (BM_VERT, v1), ELE_DEL);
        let is_del_v2 = bmo_elem_flag_test(bm, (BM_VERT, v2), ELE_DEL);
        if is_del_v1 || is_del_v2 {
            if is_del_v1 {
                if let Some((_, vt)) = bmo_slot_map_get(op, false, slot_targetmap, (BM_VERT, v1)) {
                    v1 = vt;
                }
            }
            if is_del_v2 {
                if let Some((_, vt)) = bmo_slot_map_get(op, false, slot_targetmap, (BM_VERT, v2)) {
                    v2 = vt;
                }
            }
            if v1 == v2 {
                bmo_elem_flag_enable(bm, (BM_EDGE, e), EDGE_COL);
            } else {
                let mut e_new = crate::query::bm_edge_exists(bm, v1, v2);
                if e_new == BM_NIL {
                    e_new = crate::core::bm_edge_create(bm, v1, v2, Some(e), 0);
                }
                let fl = bm.edge(e).head.hflag & crate::BM_ELEM_HIDDEN;
                bm.epool[e_new].head.hflag |= fl;
            }
            bmo_elem_flag_enable(bm, (BM_EDGE, e), ELE_DEL);
        }
    }
    /* faces: rebuild or mark deleted */
    for f in crate::iterators::FacesOfMesh::new(bm).collect::<Vec<_>>() {
        let mut vert_delete = false;
        let mut edge_collapse = 0;
        for l in crate::iterators::LoopsOfFace::new(bm, f) {
            if bmo_elem_flag_test(bm, (BM_VERT, bm.loo(l).v), ELE_DEL) {
                vert_delete = true;
            }
            if bmo_elem_flag_test(bm, (BM_EDGE, bm.loo(l).e), EDGE_COL) {
                edge_collapse += 1;
            }
        }
        if vert_delete {
            let mut use_in_place = false;
            let mut f_new = BM_NIL;
            bmo_elem_flag_enable(bm, (BM_FACE, f), ELE_DEL);
            if bm.face(f).len as i32 - edge_collapse >= 3 {
                let (nf, created) = remdoubles_createface(bm, f, op, slot_targetmap);
                f_new = nf;
                if f_new != BM_NIL {
                    if created && f_new != f {
                        /* bmesh_face_swap_data — swap geometry so the
                         * welded face keeps f's slot (oflags preserved) */
                        crate::core::bmesh_face_swap_data(bm, f_new, f);
                        bmo_elem_flag_disable(bm, (BM_FACE, f), ELE_DEL);
                        crate::core::bm_face_kill(bm, f_new);
                        use_in_place = true;
                    } else {
                        let fl = bm.face(f).head.hflag & crate::BM_ELEM_HIDDEN;
                        bm.fpool[f_new].head.hflag |= fl;
                    }
                }
            }
            if !use_in_place && f_new != BM_NIL {
                if bm.act_face == f {
                    bm.act_face = f_new;
                }
            }
        }
    }
    crate::delete::bmo_mesh_delete_oflag_context(bm, ELE_DEL, crate::delete::DEL_ONLYTAGGED, None);
}

/// `bmo_reverse_faces_exec` (from `bmo_utils.cc`).
pub fn bmo_reverse_faces_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let _use_loop_mdisp_flip = bmo_slot_bool_get(op, false, "flip_multires");
    let faces: Vec<usize> = bmo_slot_buffer(op, false, "faces")
        .iter().filter(|(ht, _)| *ht == BM_FACE).map(|&(_, i)| i).collect();
    for f in faces {
        crate::polygon::bm_face_normal_flip(bm, f);
        /* CD_MDISPS flip PORT_PENDING */
    }
}

const VERT_KEEP: u8 = 8;
const EDGE_MARK: u8 = 1;

/// `bmo_pointmerge_facedata_exec` — CustomData-bound (PORT_PENDING stub).
pub fn bmo_pointmerge_facedata_exec(_bm: &mut BMesh, _op: &mut BMOperator) {
    /* PORT_PENDING: loops over bm->ldata layers; CustomData not yet ported. */
}

/// `bmo_average_vert_facedata_exec` — CustomData-bound (PORT_PENDING stub).
pub fn bmo_average_vert_facedata_exec(_bm: &mut BMesh, _op: &mut BMOperator) {
    /* PORT_PENDING: CustomData min/max averaging. */
}

/// `bmo_pointmerge_exec` — weld all "verts" into vert_target (or merge_co).
pub fn bmo_pointmerge_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let merge_co = bmo_slot_vec_get(op, false, "merge_co");
    let verts: Vec<usize> = op
        .slots_in
        .iter()
        .find(|s| s.name == "verts")
        .map(|s| s.buf.iter().filter(|(ht, _)| *ht == BM_VERT).map(|(_, i)| *i).collect())
        .unwrap_or_default();
    let mut vert_target = op
        .slots_in
        .iter()
        .find(|s| s.name == "vert_target")
        .and_then(|s| s.buf.iter().find(|(ht, _)| *ht == BM_VERT).map(|(_, i)| *i));
    let is_explicit_snap = vert_target.is_some();

    let mut weldop = bmo_op_init(bm, &crate::opdefines::OPDEF_WELD_VERTS, op.flag);
    for v in verts {
        if vert_target.is_none() {
            vert_target = Some(v);
            bm.vpool[v].co = merge_co;
        } else if Some(v) != vert_target {
            let vt = vert_target.unwrap();
            slot_get_mut(&mut weldop.slots_in, "targetmap").map.insert((BM_VERT, v), (BM_VERT, vt));
        }
    }
    if !is_explicit_snap {
        if let Some(si) = weldop.slots_in.iter().position(|s| s.name == "average_vert_data") {
            weldop.slots_in[si].b = true;
        }
    }
    bmo_op_exec(bm, &mut weldop, bmo_weld_verts_exec);
    bmo_op_finish(bm, weldop);
}

/// `bmo_collapse_exec` — collapse each marked-edge island to a point.
pub fn bmo_collapse_exec(bm: &mut BMesh, op: &mut BMOperator) {
    if bmo_slot_bool_get(op, false, "uvs") {
        let mut cop = bmo_op_init(bm, &crate::opdefines::OPDEF_COLLAPSE_UVS, op.flag);
        if let Some(si) = cop.slots_in.iter().position(|s| s.name == "edges") {
            cop.slots_in[si].buf = op.slots_in.iter()
                .find(|s| s.name == "edges").map(|s| s.buf.clone()).unwrap_or_default();
        }
        bmo_op_exec(bm, &mut cop, bmo_collapse_uvs_exec);
        bmo_op_finish(bm, cop);
    }
    let mut weldop = bmo_op_init(bm, &crate::opdefines::OPDEF_WELD_VERTS, op.flag);
    bmo_slot_buffer_flag_enable(bm, op, false, "edges", EDGE_MARK);

    let mut w = crate::walkers::BMWalker::new(crate::walkers::BMWType::VertShell, 0, 0);
    w.mask_edge = EDGE_MARK;

    for e_seed in crate::iterators::EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.edge(e_seed).head.api_flag & EDGE_MARK == 0 {
            continue;
        }
        let mut center = Vec3::ZERO;
        let mut count = 0;
        let mut stack: Vec<usize> = Vec::new();
        w.begin(bm, BM_VERT, bm.edge(e_seed).v1);
        for (ht, idx) in crate::walkers::bmw_walk_all(bm, &mut w) {
            if ht != BM_EDGE {
                continue;
            }
            stack.push(idx);
            center = center + bm.vert(bm.edge(idx).v1).co + bm.vert(bm.edge(idx).v2).co;
            count += 2;
            /* Prevent adding to targetmap multiple times. */
            let (v1, v2) = (bm.edge(idx).v1, bm.edge(idx).v2);
            bm.vpool[v1].head.hflag &= !crate::BM_ELEM_TAG;
            bm.vpool[v2].head.hflag &= !crate::BM_ELEM_TAG;
        }
        if stack.is_empty() {
            continue;
        }
        center = center / count as f32;
        let v_tar = bm.edge(*stack.last().unwrap()).v1;
        while let Some(e) = stack.pop() {
            for v_src in [bm.edge(e).v1, bm.edge(e).v2] {
                bm.vpool[v_src].co = center;
                if v_src != v_tar && bm.vert(v_src).head.hflag & crate::BM_ELEM_TAG == 0 {
                    bm.vpool[v_src].head.hflag |= crate::BM_ELEM_TAG;
                    slot_get_mut(&mut weldop.slots_in, "targetmap")
                        .map
                        .insert((BM_VERT, v_src), (BM_VERT, v_tar));
                }
            }
        }
    }
    bmo_op_exec(bm, &mut weldop, bmo_weld_verts_exec);
    bmo_op_finish(bm, weldop);
}

/// `bmo_collapse_uvs_exec` — CustomData-bound (PORT_PENDING stub).
pub fn bmo_collapse_uvs_exec(_bm: &mut BMesh, _op: &mut BMOperator) {
    /* PORT_PENDING: UV/color data collapse needs CustomData + LOOPDATA_ISLAND walker. */
}

/* ------------------------------------------------------------------ */
/* find_doubles */

/// `calc_duplicates_with_target` — group points within `range` (kdtree version).
fn calc_duplicates_with_target(
    positions: &[[f32; 3]],
    range: f32,
    has_self_index: bool,
    duplicates: &mut [i32],
    target_fn: impl Fn(&[i32]) -> usize,
) -> i32 {
    let mut found = 0;
    let tree = blenders_bli::kdtree::KDTree::<3>::new(positions, 8);
    if has_self_index {
        let mut duplicates_dist_sq = vec![0.0f32; positions.len()];
        for index in 0..positions.len() {
            if duplicates[index] != index as i32 {
                continue;
            }
            tree.foreach_in_radius(positions, &positions[index], range,
                |neighbor, dist_sq| {
                    let neighbor = neighbor as usize;
                    let target_index = duplicates[neighbor];
                    if target_index == -1 {
                        duplicates[neighbor] = index as i32;
                        duplicates_dist_sq[neighbor] = dist_sq;
                        found += 1;
                    } else if target_index != neighbor as i32 {
                        /* Steal the target if closer (lowest index wins ties). */
                        let best = duplicates_dist_sq[neighbor];
                        if dist_sq < best || (dist_sq == best && (index as i32) < target_index) {
                            duplicates_dist_sq[neighbor] = dist_sq;
                            duplicates[neighbor] = index as i32;
                        }
                    }
                });
        }
    }
    /* Second pass: de-duplicate remaining clusters. */
    let mut cluster: Vec<i32> = Vec::new();
    for index in 0..positions.len() {
        if duplicates[index] != -1 {
            continue;
        }
        tree.foreach_in_radius(positions, &positions[index], range, |n, _| {
            if n != index as i32 && duplicates[n as usize] == -1 {
                cluster.push(n);
            }
        });
        if cluster.is_empty() {
            continue;
        }
        found += cluster.len() as i32;
        cluster.sort_unstable();
        cluster.push(index as i32);
        let target_index = cluster[target_fn(&cluster)];
        for &ci in &cluster {
            duplicates[ci as usize] = target_index;
        }
        cluster.clear();
    }
    found
}

/// `bmesh_find_doubles_by_distance_impl` — returns merged-target indices or None.
fn bmesh_find_doubles_by_distance_impl(
    bm: &BMesh,
    verts: &[usize],
    dist: f32,
    has_keep_vert: bool,
) -> Option<Vec<i32>> {
    let mut duplicates = vec![0i32; verts.len()];
    let mut has_self_index = false;
    let positions: Vec<[f32; 3]> = verts
        .iter()
        .map(|&v| { let c = bm.vert(v).co; [c.x, c.y, c.z] })
        .collect();
    for (i, &v) in verts.iter().enumerate() {
        if has_keep_vert && bm.vert(v).head.api_flag & VERT_KEEP != 0 {
            duplicates[i] = i as i32;
            has_self_index = true;
        } else {
            duplicates[i] = -1;
        }
    }
    /* pick target index within a cluster */
    let target_fn = |cluster: &[i32]| -> usize {
        if cluster.len() == 2 {
            return if cluster[0] < cluster[1] { 0 } else { 1 };
        }
        let mut centroid = [0.0f32; 3];
        for &i in cluster {
            let p = positions[i as usize];
            centroid[0] += p[0];
            centroid[1] += p[1];
            centroid[2] += p[2];
        }
        let n = cluster.len() as f32;
        centroid = [centroid[0] / n, centroid[1] / n, centroid[2] / n];
        let dist_sq = |a: [f32; 3], b: [f32; 3]| {
            (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
        };
        let cluster_end = cluster.len() - 1;
        let mut i_best = cluster_end;
        let mut dist_sq_best = dist_sq(centroid, positions[cluster[i_best] as usize]);
        for i in 0..cluster_end {
            let d = dist_sq(centroid, positions[cluster[i] as usize]);
            if d > dist_sq_best {
                continue;
            }
            if d == dist_sq_best && cluster[i] > cluster[i_best] {
                continue;
            }
            i_best = i;
            dist_sq_best = d;
        }
        i_best
    };
    let found = calc_duplicates_with_target(
        &positions,
        dist,
        has_self_index,
        &mut duplicates,
        target_fn,
    );
    if found == 0 {
        None
    } else {
        Some(duplicates)
    }
}

/// `bmesh_find_doubles_by_distance_connected_impl`.
fn bmesh_find_doubles_by_distance_connected_impl(
    bm: &BMesh,
    verts: &[usize],
    dist: f32,
    has_keep_vert: bool,
) -> Option<Vec<i32>> {
    let mut duplicates = vec![0i32; verts.len()];
    let mut found_duplicates = false;
    let mut vert_to_index: HashMap<usize, usize> = HashMap::new();
    for (i, &v) in verts.iter().enumerate() {
        if has_keep_vert && bm.vert(v).head.api_flag & VERT_KEEP != 0 {
            duplicates[i] = i as i32;
        } else {
            duplicates[i] = -1;
        }
        vert_to_index.insert(v, i);
    }
    let dist_sq = dist * dist;
    for i in 0..verts.len() {
        if !(duplicates[i] == -1 || duplicates[i] == i as i32) {
            continue;
        }
        let co_check = bm.vert(verts[i]).co;
        let mut vert_stack: Vec<usize> = Vec::new();
        let mut i_check = Some(i);
        while let Some(ic) = i_check {
            let v_check = verts[ic];
            let e_first = bm.vert(v_check).e;
            if e_first != BM_NIL {
                let mut e_iter = e_first;
                loop {
                    /* Edge stepping. */
                    let v_other = crate::structure::bm_edge_other_vert(bm, e_iter, v_check);
                    if (bm.vert(v_other).co - co_check).length_squared() < dist_sq {
                        if let Some(&i_other) = vert_to_index.get(&v_other) {
                            if duplicates[i_other] == -1 {
                                duplicates[i_other] = i as i32;
                                vert_stack.push(i_other);
                                found_duplicates = true;
                            }
                        }
                    }
                    /* Face stepping. */
                    if bm.edge(e_iter).l != BM_NIL {
                        let mut l_radial = bm.edge(e_iter).l;
                        loop {
                            let lr = bm.loo(l_radial);
                            if lr.v == v_check && bm.face(lr.f).len > 3 {
                                let mut l_iter = bm.loo(lr.next).next;
                                let l_end = lr.prev;
                                loop {
                                    let v_other = bm.loo(l_iter).v;
                                    if (bm.vert(v_other).co - co_check).length_squared() < dist_sq {
                                        if let Some(&i_other) = vert_to_index.get(&v_other) {
                                            if duplicates[i_other] == -1 {
                                                duplicates[i_other] = i as i32;
                                                vert_stack.push(i_other);
                                                found_duplicates = true;
                                            }
                                        }
                                    }
                                    l_iter = bm.loo(l_iter).next;
                                    if l_iter == l_end {
                                        break;
                                    }
                                }
                            }
                            l_radial = bm.loo(l_radial).radial_next;
                            if l_radial == bm.edge(e_iter).l {
                                break;
                            }
                        }
                    }
                    e_iter = crate::structure::bmesh_disk_edge_next(bm, e_iter, v_check);
                    if e_iter == e_first {
                        break;
                    }
                }
            }
            i_check = vert_stack.pop();
        }
    }
    if !found_duplicates {
        None
    } else {
        Some(duplicates)
    }
}

/// `bmesh_find_doubles_common`.
fn bmesh_find_doubles_common(
    bm: &mut BMesh,
    op: &mut BMOperator,
    target_map: &mut HashMap<BMElemRef, BMElemRef>,
) {
    let use_connected = bmo_slot_bool_get(op, false, "use_connected");
    let verts: Vec<usize> = op
        .slots_in
        .iter()
        .find(|s| s.name == "verts")
        .map(|s| s.buf.iter().filter(|(ht, _)| *ht == BM_VERT).map(|(_, i)| *i).collect())
        .unwrap_or_default();
    let dist = bmo_slot_float_get(op, false, "dist");

    /* Test whether keep_verts arg exists and is non-empty. */
    let has_keep_vert = op
        .slots_in
        .iter()
        .find(|s| s.name == "keep_verts")
        .map(|s| s.buf.iter().any(|(ht, _)| *ht == BM_VERT))
        .unwrap_or(false);
    if has_keep_vert {
        bmo_slot_buffer_flag_enable(bm, op, false, "keep_verts", VERT_KEEP);
    }

    let duplicates = if use_connected {
        bmesh_find_doubles_by_distance_connected_impl(bm, &verts, dist, has_keep_vert)
    } else {
        bmesh_find_doubles_by_distance_impl(bm, &verts, dist, has_keep_vert)
    };

    if let Some(dups) = duplicates {
        for (i, &d) in dups.iter().enumerate() {
            if d == -1 || d == i as i32 {
                /* NOP / keep */
            } else {
                target_map.insert((BM_VERT, verts[i]), (BM_VERT, verts[d as usize]));
            }
        }
    }
}

/// `bmo_remove_doubles_exec`.
pub fn bmo_remove_doubles_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let mut weldop = bmo_op_init(bm, &crate::opdefines::OPDEF_WELD_VERTS, op.flag);
    let mut tmap: HashMap<BMElemRef, BMElemRef> = HashMap::new();
    bmesh_find_doubles_common(bm, op, &mut tmap);
    slot_get_mut(&mut weldop.slots_in, "targetmap").map = tmap;
    bmo_op_exec(bm, &mut weldop, bmo_weld_verts_exec);
    bmo_op_finish(bm, weldop);
}

/// `bmo_find_doubles_exec`.
pub fn bmo_find_doubles_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let mut tmap: HashMap<BMElemRef, BMElemRef> = HashMap::new();
    bmesh_find_doubles_common(bm, op, &mut tmap);
    slot_get_mut(&mut op.slots_out, "targetmap.out").map = tmap;
}
