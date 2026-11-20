//! Port of `tools/bmesh_bisect_plane.cc` — `BM_mesh_bisect_plane`.
//! C abuses `v->head.index`/`v->no` for scratch; Rust uses local side tables
//! keyed by vert pool index.

use crate::iterators::{EdgesOfMesh, FacesOfMesh, VertsOfMesh};
use crate::{BMesh, BM_ELEM_TAG, BM_NIL};
use blenders_math::geom::{plane_point_side, closest_to_plane};
use blenders_math::vector::Vec4;

/// `plane_point_test_v3`.
fn plane_point_test_v3(plane: Vec4, co: blenders_math::vector::Vec3, eps: f32, r_depth: &mut f32) -> i8 {
    let f = plane_point_side(plane, co);
    *r_depth = f;
    if f <= -eps { -1 } else if f >= eps { 1 } else { 0 }
}

struct VertState {
    dir: Vec<i8>,       /* BM_VERT_DIR */
    skip: Vec<bool>,    /* BM_VERT_SKIP */
    dist: Vec<f32>,     /* BM_VERT_DIST */
    sortval: Vec<f32>,  /* BM_VERT_SORTVAL */
    loopindex: Vec<u32>,/* BM_VERT_LOOPINDEX */
}

impl VertState {
    fn new(bm: &BMesh) -> Self {
        let n = bm.vpool.len();
        Self {
            dir: vec![0; n],
            skip: vec![false; n],
            dist: vec![0.0; n],
            sortval: vec![0.0; n],
            loopindex: vec![0; n],
        }
    }
}

fn vert_is_center_test(bm: &BMesh, v: usize) -> bool {
    bm.vert(v).head.hflag & BM_ELEM_TAG != 0
}

fn vert_pair_adjacent_in_orig_face(st: &VertState, v_a: usize, v_b: usize, f_len_orig: u32) -> bool {
    let d = (st.loopindex[v_a] as i64 - st.loopindex[v_b] as i64).unsigned_abs() as u32;
    d == 1 || d == f_len_orig - 1
}

/// `bm_face_bisect_verts` — split a face (which crosses the plane) along
/// the segment through its center verts.
fn bm_face_bisect_verts(
    bm: &mut BMesh,
    f: usize,
    plane: Vec4,
    oflag_center: i8,
    oflag_new: i8,
    st: &mut VertState,
) {
    use crate::operators::*;
    let f_len_orig = bm.face(f).len as u32;
    let mut vert_split_arr: Vec<usize> = Vec::with_capacity(f_len_orig as usize);
    let mut use_dirs = [false; 3];
    let mut face_has_center_edge = false;

    let l_first = bm.face(f).l_first;
    let mut l_iter = l_first;
    loop {
        let lv = bm.loo(l_iter).v;
        if vert_is_center_test(bm, lv) {
            debug_assert_eq!(st.dir[lv], 0);
            st.skip[lv] = (st.dir[bm.loo(bm.loo(l_iter).prev).v]
                ^ st.dir[bm.loo(bm.loo(l_iter).next).v]) == 0;
            vert_split_arr.push(lv);
            if !face_has_center_edge && vert_is_center_test(bm, bm.loo(bm.loo(l_iter).prev).v) {
                face_has_center_edge = true;
            }
        }
        use_dirs[(st.dir[lv] + 1) as usize] = true;
        l_iter = bm.loo(l_iter).next;
        if l_iter == l_first { break; }
    }

    if vert_split_arr.len() > 1 && use_dirs[0] && use_dirs[2] {
        if vert_split_arr.len() == 2 {
            let l_a = crate::query::bm_face_vert_share_loop(bm, f, vert_split_arr[0]);
            let l_b = crate::query::bm_face_vert_share_loop(bm, f, vert_split_arr[1]);
            let mut l_new = BM_NIL;
            let _f_new = crate::mods::bm_face_split(
                bm, f, l_a, l_b, Some(&mut l_new), true);
            if l_new != BM_NIL {
                if (oflag_center | oflag_new) != 0 {
                    bmo_elem_flag_enable(bm, (crate::BM_EDGE, bm.loo(l_new).e),
                        (oflag_center | oflag_new) as u8);
                }
                if oflag_new != 0 {
                    bmo_elem_flag_enable(bm, (crate::BM_FACE, bm.loo(l_new).f), oflag_new as u8);
                }
            }
        } else {
            let mut i: u32 = 0;
            if face_has_center_edge {
                let mut l = l_first;
                loop {
                    st.loopindex[bm.loo(l).v] = i;
                    i += 1;
                    l = bm.loo(l).next;
                    if l == l_first { break; }
                }
                let mut l_first_non_center = l_first;
                while vert_is_center_test(bm, bm.loo(l_first_non_center).v) {
                    l_first_non_center = bm.loo(l_first_non_center).next;
                }
                l_iter = l_first_non_center;
                loop {
                    if !st.skip[bm.loo(l_iter).v]
                        && vert_is_center_test(bm, bm.loo(l_iter).v)
                        && vert_is_center_test(bm, bm.loo(bm.loo(l_iter).next).v)
                    {
                        let l_prev = bm.loo(l_iter).prev;
                        let mut l_next = bm.loo(bm.loo(l_iter).next).next;
                        while vert_is_center_test(bm, bm.loo(l_next).v) {
                            l_next = bm.loo(l_next).next;
                        }
                        if (st.dir[bm.loo(l_prev).v] ^ st.dir[bm.loo(l_next).v]) == 0 {
                            let mut l_run = bm.loo(l_prev).next;
                            while l_run != l_next {
                                st.skip[bm.loo(l_run).v] = true;
                                l_run = bm.loo(l_run).next;
                            }
                        }
                        l_iter = bm.loo(l_next).prev;
                    }
                    l_iter = bm.loo(l_iter).next;
                    if l_iter == l_first_non_center { break; }
                }
            }
            let mut face_split_arr: Vec<usize> = Vec::with_capacity(vert_split_arr.len());
            /* sort_dir = face_no x plane_no */
            let mut sort_dir = bm.face(f).no.cross(plane.xyz());
            let sort_len = sort_dir.length();
            if sort_len == 0.0 {
                let mut found = false;
                for i2 in 0..vert_split_arr.len() {
                    if bm.vert(vert_split_arr[0]).co != bm.vert(vert_split_arr[i2]).co {
                        sort_dir = (bm.vert(vert_split_arr[0]).co
                            - bm.vert(vert_split_arr[i2]).co).normalized();
                        found = true;
                        break;
                    }
                }
                if !found { return; }
            } else {
                sort_dir = sort_dir / sort_len;
            }
            for &v in &vert_split_arr {
                st.sortval[v] = sort_dir.dot(bm.vert(v).co);
            }
            vert_split_arr.sort_by(|&a, &b| st.sortval[a].partial_cmp(&st.sortval[b]).unwrap());
            face_split_arr.push(f);
            let mut is_inside = false;
            for i2 in 0..vert_split_arr.len() - 1 {
                let v_a = vert_split_arr[i2];
                let v_b = vert_split_arr[i2 + 1];
                if face_has_center_edge
                    && vert_pair_adjacent_in_orig_face(st, v_a, v_b, f_len_orig)
                {
                    continue;
                }
                if !st.skip[v_a] {
                    is_inside = !is_inside;
                }
                if is_inside {
                    let mut found_j = usize::MAX;
                    let mut l_a = BM_NIL;
                    let mut l_b = BM_NIL;
                    for (j, &fs) in face_split_arr.iter().enumerate() {
                        let la = crate::query::bm_face_vert_share_loop(bm, fs, v_a);
                        let lb = crate::query::bm_face_vert_share_loop(bm, fs, v_b);
                        if la != BM_NIL && lb != BM_NIL {
                            found_j = j;
                            l_a = la;
                            l_b = lb;
                            break;
                        }
                    }
                    if found_j != usize::MAX && !crate::mods::bm_loop_is_adjacent(bm, l_a, l_b) {
                        let mut l_new = BM_NIL;
                        let f_tmp = crate::mods::bm_face_split(
                            bm, face_split_arr[found_j], l_a, l_b, Some(&mut l_new), true);
                        if l_new != BM_NIL {
                            if (oflag_center | oflag_new) != 0 {
                                bmo_elem_flag_enable(bm, (crate::BM_EDGE, bm.loo(l_new).e),
                                    (oflag_center | oflag_new) as u8);
                            }
                            if oflag_new != 0 {
                                bmo_elem_flag_enable(bm, (crate::BM_FACE, bm.loo(l_new).f), oflag_new as u8);
                            }
                        }
                        if f_tmp != BM_NIL && f_tmp != face_split_arr[found_j] {
                            face_split_arr.push(f_tmp);
                        }
                    }
                }
            }
        }
    }
}

/// `BM_mesh_bisect_plane` — split all (optionally tagged) edges/faces
/// crossing `plane`; new elements get `oflag_new`, center ones `oflag_center`.
pub fn bm_mesh_bisect_plane(
    bm: &mut BMesh,
    plane: Vec4,
    use_snap_center: bool,
    use_tag: bool,
    oflag_center: i8,
    oflag_new: i8,
    eps: f32,
) {
    use crate::operators::*;
    let mut st = VertState::new(bm);

    /* gather edges to cut */
    let mut edges_arr: Vec<usize> = Vec::with_capacity(bm.totedge as usize);
    if use_tag {
        for e in EdgesOfMesh::new(bm).collect::<Vec<_>>() {
            if bm.edge(e).head.hflag & BM_ELEM_TAG != 0 {
                edges_arr.push(e);
                let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
                bm.vpool[v1].head.hflag |= BM_ELEM_TAG;
                bm.vpool[v2].head.hflag |= BM_ELEM_TAG;
            }
        }
    } else {
        for e in EdgesOfMesh::new(bm).collect::<Vec<_>>() {
            bm.epool[e].head.hflag |= BM_ELEM_TAG;
            edges_arr.push(e);
        }
        for f in FacesOfMesh::new(bm).collect::<Vec<_>>() {
            bm.fpool[f].head.hflag |= BM_ELEM_TAG; /* face_in_stack_disable */
        }
    }

    /* classify verts */
    let mut faces_deferred_normal_update: Vec<usize> = Vec::new();
    let mut deferred_set = std::collections::HashSet::new();
    for v in VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if use_tag && bm.vert(v).head.hflag & BM_ELEM_TAG == 0 {
            bm.vpool[v].head.hflag &= !BM_ELEM_TAG;
            st.dir[v] = 0;
            st.dist[v] = 0.0;
            continue;
        }
        bm.vpool[v].head.hflag &= !BM_ELEM_TAG; /* vert_is_center_disable */
        let mut depth = 0.0f32;
        st.dir[v] = plane_point_test_v3(plane, bm.vert(v).co, eps, &mut depth);
        st.dist[v] = depth;
        if st.dir[v] == 0 {
            if oflag_center != 0 {
                bmo_elem_flag_enable(bm, (crate::BM_VERT, v), oflag_center as u8);
            }
            if use_snap_center {
                let co_center = closest_to_plane(plane, bm.vert(v).co);
                if co_center != bm.vert(v).co {
                    bm.vpool[v].co = co_center;
                    for f in crate::iterators::FacesOfVert::new(bm, v) {
                        if deferred_set.insert(f) {
                            faces_deferred_normal_update.push(f);
                        }
                    }
                }
            }
        }
    }
    for f in faces_deferred_normal_update {
        crate::normals::bm_face_calc_normals(bm, f);
    }

    /* split crossing edges, queue faces */
    let mut face_stack: Vec<usize> = Vec::new();
    let mut in_stack: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for &e in &edges_arr {
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        let side = [st.dir[v1], st.dir[v2]];
        let dist = [st.dist[v1], st.dist[v2]];
        if side[0] != 0 && side[1] != 0 && side[0] != side[1] {
            let e_fac = dist[0] / (dist[0] - dist[1]);
            if bm.edge(e).l != BM_NIL {
                let l_first = bm.edge(e).l;
                let mut l_iter = l_first;
                loop {
                    let f = bm.loo(l_iter).f;
                    if in_stack.insert(f) {
                        face_stack.push(f);
                    }
                    l_iter = bm.loo(l_iter).radial_next;
                    if l_iter == l_first { break; }
                }
            }
            let mut e_new = BM_NIL;
            let v_new = crate::mods::bm_edge_split(bm, e, v1, Some(&mut e_new), e_fac);
            if oflag_new != 0 {
                bmo_elem_flag_enable(bm, (crate::BM_EDGE, e_new), oflag_new as u8);
            }
            bm.vpool[v_new].head.hflag |= BM_ELEM_TAG;
            if (oflag_new | oflag_center) != 0 {
                bmo_elem_flag_enable(bm, (crate::BM_VERT, v_new), (oflag_new | oflag_center) as u8);
            }
            /* grow state arrays if pool grew */
            st.dir.resize(bm.vpool.len(), 0);
            st.skip.resize(bm.vpool.len(), false);
            st.dist.resize(bm.vpool.len(), 0.0);
            st.sortval.resize(bm.vpool.len(), 0.0);
            st.loopindex.resize(bm.vpool.len(), 0);
            st.dir[v_new] = 0;
            st.dist[v_new] = 0.0;
        } else if side[0] == 0 || side[1] == 0 {
            for (j, v) in [v1, v2].into_iter().enumerate() {
                if side[j] == 0 && !vert_is_center_test(bm, v) {
                    bm.vpool[v].head.hflag |= BM_ELEM_TAG;
                    for l in crate::iterators::LoopsOfVert::new(bm, v) {
                        let f = bm.loo(l).f;
                        if in_stack.insert(f) {
                            face_stack.push(f);
                        }
                    }
                }
            }
            if oflag_center != 0 && side[0] == 0 && side[1] == 0 {
                bmo_elem_flag_enable(bm, (crate::BM_EDGE, e), oflag_center as u8);
            }
        }
    }

    while let Some(f) = face_stack.pop() {
        bm_face_bisect_verts(bm, f, plane, oflag_center, oflag_new, &mut st);
    }
    bm.elem_index_dirty = true;
}
