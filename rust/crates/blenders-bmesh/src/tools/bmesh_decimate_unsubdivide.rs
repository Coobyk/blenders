//! Port of `tools/bmesh_decimate_unsubdivide.cc` — unsubdivide decimation.
use crate::iterators::{EdgesOfVert, VertsOfMesh};
use crate::{BMesh, BM_ELEM_TAG, BM_NIL};

/// Verts with more connected edges/faces are ignored.
const VERT_DISSOLVE_MAX: usize = 4;

/// `VertDissolveMethod`.
#[derive(Clone, Copy, PartialEq)]
enum VertDissolveMethod {
    /// 3/4 manifold edges: clip ears into triangles, join them.
    InteriorFan,
    /// 2 edges on same 2 faces: remove vert, connect adjacent edges.
    InteriorChain,
    /// 1 manifold edge + 2 boundary edges.
    BoundaryFan,
    /// 2 wire edges, no faces.
    WireChain,
}

/// `bm_vert_loops_from_vert_edges` — one loop per radial face at `v`.
fn bm_vert_loops_from_vert_edges(bm: &BMesh, v: usize, v_edges: &[usize], v_loops: &mut Vec<usize>) -> usize {
    let mut v_loops_num = 0;
    for &e in v_edges {
        let l0 = bm.edge(e).l;
        if l0 != BM_NIL {
            let mut l_radial_iter = l0;
            loop {
                if bm.loo(l_radial_iter).v == v {
                    if v_loops_num == VERT_DISSOLVE_MAX {
                        debug_assert!(false, "corrupt topology");
                        return v_loops_num;
                    }
                    v_loops.push(l_radial_iter);
                    v_loops_num += 1;
                }
                l_radial_iter = bm.loo(l_radial_iter).radial_next;
                if l_radial_iter == l0 { break; }
            }
        }
    }
    v_loops_num
}

/// `bm_vert_dissolve_fan_makes_double_whole_face`.
fn bm_vert_dissolve_fan_makes_double_whole_face(bm: &BMesh, v_loops_all: &[usize]) -> bool {
    let mut edges_from_loop_ear: Vec<usize> = Vec::new();
    for &l in v_loops_all {
        let f = bm.loo(l).f;
        let e_opposite;
        if bm.face(f).len > 3 {
            let a = bm.loo(bm.loo(l).prev).v;
            let b = bm.loo(bm.loo(l).next).v;
            let eo = crate::query::bm_edge_exists(bm, a, b);
            if eo == BM_NIL {
                return false;
            }
            e_opposite = eo;
        } else {
            e_opposite = bm.loo(bm.loo(l).next).e;
        }
        if edges_from_loop_ear.len() == VERT_DISSOLVE_MAX {
            return false;
        }
        edges_from_loop_ear.push(e_opposite);
    }
    if edges_from_loop_ear.len() >= 3 {
        if crate::query::bm_face_exists_multi_edge(bm, &edges_from_loop_ear) {
            return true;
        }
    }
    false
}

/// `bm_vert_dissolve_fan_makes_double_with_existing`.
fn bm_vert_dissolve_fan_makes_double_with_existing(bm: &BMesh, v_loops_all: &[usize]) -> bool {
    for &l in v_loops_all {
        let f = bm.loo(l).f;
        if bm.face(f).len > 3 {
            if crate::core::bm_face_split_check_double_face(bm, bm.loo(l).prev, bm.loo(l).next, 3) {
                return true;
            }
        }
    }
    false
}

/// `bm_vert_dissolve_fan_makes_double_with_self` — canonicalized
/// collapsed-face comparison between the (len>3) loops.
fn bm_vert_dissolve_fan_makes_double_with_self(bm: &BMesh, v_loops_all: &[usize]) -> bool {
    debug_assert!(v_loops_all.len() <= VERT_DISSOLVE_MAX);
    if v_loops_all.len() < 2 { return false; }
    /* keep only faces with len > 3, sorted by face len */
    let mut v_loops: Vec<usize> = v_loops_all.iter().copied()
        .filter(|&l| bm.face(bm.loo(l).f).len > 3).collect();
    if v_loops.len() < 2 { return false; }
    v_loops.sort_by_key(|&l| bm.face(bm.loo(l).f).len);
    let verts_canonical_collapsed = |l_vert: usize| -> Vec<usize> {
        let f = bm.loo(l_vert).f;
        let verts_num = (bm.face(f).len - 1) as usize;
        let mut verts: Vec<usize> = Vec::with_capacity(verts_num);
        let mut l = bm.loo(l_vert).next;
        while l != l_vert {
            verts.push(bm.loo(l).v);
            l = bm.loo(l).next;
        }
        debug_assert_eq!(verts.len(), verts_num);
        /* canonicalize: start at min index, order by next-adjacent min */
        let mut i_min = 0;
        for i in 1..verts_num {
            if verts[i] < verts[i_min] { i_min = i; }
        }
        let i_next = if i_min + 1 == verts_num { 0 } else { i_min + 1 };
        let i_prev = if i_min == 0 { verts_num - 1 } else { i_min - 1 };
        let step = if verts[i_next] < verts[i_prev] { 1 } else { verts_num - 1 };
        let mut verts_ordered = Vec::with_capacity(verts_num);
        let mut j = i_min;
        for _ in 0..verts_num {
            verts_ordered.push(verts[j]);
            j += step;
            if j >= verts_num { j -= verts_num; }
        }
        verts_ordered
    };
    /* per-equal-length group, compare canonical collapsed loops */
    let mut group_beg = 0;
    while group_beg < v_loops.len() {
        let group_f_len = bm.face(bm.loo(v_loops[group_beg]).f).len;
        let mut group_end = group_beg + 1;
        while group_end < v_loops.len()
            && bm.face(bm.loo(v_loops[group_end]).f).len == group_f_len
        {
            group_end += 1;
        }
        let group_len = group_end - group_beg;
        if group_len >= 2 {
            let collapsed: Vec<Vec<usize>> = (0..group_len)
                .map(|i| verts_canonical_collapsed(v_loops[group_beg + i]))
                .collect();
            for i in 0..group_len - 1 {
                for j in i + 1..group_len {
                    if collapsed[i] == collapsed[j] {
                        return true;
                    }
                }
            }
        }
        group_beg = group_end;
    }
    false
}

/// `bm_vert_dissolve_fan_or_chain_test`.
fn bm_vert_dissolve_fan_or_chain_test(
    bm: &BMesh,
    v: usize,
    check_for_duplicates: bool,
) -> Option<(VertDissolveMethod, Vec<usize>)> {
    let mut v_edges: Vec<usize> = Vec::new();
    let mut e_boundary: [usize; 2] = [BM_NIL; 2];
    let mut e_boundary_num = 0;
    let mut tot_edge_boundary = 0;
    let mut tot_edge_manifold = 0;
    let mut tot_edge_wire = 0;
    for e in EdgesOfVert::new(bm, v) {
        if crate::delete::bm_edge_is_boundary(bm, e) {
            if e_boundary_num < 2 { e_boundary[e_boundary_num] = e; }
            e_boundary_num += 1;
            tot_edge_boundary += 1;
        } else if crate::query::bm_edge_is_manifold(bm, e) {
            tot_edge_manifold += 1;
        } else if crate::edgenet::edge_is_wire(bm, e) {
            tot_edge_wire += 1;
        } else {
            return None;
        }
        if v_edges.len() == VERT_DISSOLVE_MAX {
            return None;
        }
        v_edges.push(e);
    }
    let tot_edge = v_edges.len();
    if ((tot_edge == 4) && (tot_edge_boundary == 0) && (tot_edge_manifold == 4))
        || ((tot_edge == 3) && (tot_edge_boundary == 0) && (tot_edge_manifold == 3))
    {
        let mut v_loops = Vec::new();
        if check_for_duplicates {
            bm_vert_loops_from_vert_edges(bm, v, &v_edges, &mut v_loops);
            if bm_vert_dissolve_fan_makes_double_whole_face(bm, &v_loops) { return None; }
            if bm_vert_dissolve_fan_makes_double_with_existing(bm, &v_loops)
                || bm_vert_dissolve_fan_makes_double_with_self(bm, &v_loops)
            { return None; }
        }
        return Some((VertDissolveMethod::InteriorFan, v_loops));
    }
    if (tot_edge == 3) && (tot_edge_boundary == 2) && (tot_edge_manifold == 1) {
        let mut v_loops = Vec::new();
        if check_for_duplicates {
            debug_assert_eq!(e_boundary_num, 2);
            let a = crate::structure::bm_edge_other_vert(bm, e_boundary[0], v);
            let b = crate::structure::bm_edge_other_vert(bm, e_boundary[1], v);
            let e_span = crate::query::bm_edge_exists(bm, a, b);
            if e_span != BM_NIL && bm.edge(e_span).l != BM_NIL {
                return None;
            }
            bm_vert_loops_from_vert_edges(bm, v, &v_edges, &mut v_loops);
            if bm_vert_dissolve_fan_makes_double_with_existing(bm, &v_loops)
                || bm_vert_dissolve_fan_makes_double_with_self(bm, &v_loops)
            { return None; }
        }
        return Some((VertDissolveMethod::BoundaryFan, v_loops));
    }
    if (tot_edge == 2) && (tot_edge_wire == 2) {
        return Some((VertDissolveMethod::WireChain, Vec::new()));
    }
    if (tot_edge == 2) && (tot_edge_manifold == 2) {
        return Some((VertDissolveMethod::InteriorChain, Vec::new()));
    }
    None
}

/// `bm_vert_dissolve_fan_or_chain` — dissolve one vert by classified method.
fn bm_vert_dissolve_fan_or_chain(bm: &mut BMesh, v: usize) -> bool {
    let (dissolve, v_loops) = match bm_vert_dissolve_fan_or_chain_test(bm, v, true) {
        Some(r) => r,
        None => return false,
    };
    match dissolve {
        VertDissolveMethod::WireChain | VertDissolveMethod::InteriorChain => {
            crate::mods::bm_vert_collapse_edge(bm, bm.vert(v).e, v, true, true, true) != BM_NIL
        }
        VertDissolveMethod::BoundaryFan | VertDissolveMethod::InteriorFan => {
            for &l in &v_loops {
                let f = bm.loo(l).f;
                if bm.face(f).len > 3 {
                    let mut l_new = BM_NIL;
                    debug_assert_ne!(bm.loo(bm.loo(l).prev).v, bm.loo(bm.loo(l).next).v);
                    let (lp, ln) = (bm.loo(l).prev, bm.loo(l).next);
                    crate::mods::bm_face_split(bm, f, lp, ln, Some(&mut l_new), true);
                    let le = bm.loo(l).e;
                    let lpe = bm.loo(lp).e;
                    let lne = bm.loo(l_new).e;
                    crate::marking::bm_elem_flag_merge_into(
                        bm, (crate::BM_EDGE, lne), (crate::BM_EDGE, le), (crate::BM_EDGE, lpe));
                }
            }
            crate::mods::bm_vert_dissolve(bm, v)
        }
    }
}

const VERT_INDEX_DO_COLLAPSE: i32 = -1;
const VERT_INDEX_INIT: i32 = 0;
const VERT_INDEX_IGNORE: i32 = 1;

/// `bm_tag_untagged_neighbors` — tag INIT neighbors of `verts_start`
/// with `desired_tag`, return the newly tagged verts.
fn bm_tag_untagged_neighbors(
    bm: &mut BMesh,
    verts_start: &[usize],
    desired_tag: i32,
) -> Vec<usize> {
    let mut r_verts_tagged = Vec::new();
    for &v in verts_start {
        debug_assert_eq!(bm.vert(v).head.index, -desired_tag);
        for e in EdgesOfVert::new(bm, v).collect::<Vec<_>>() {
            let v_other = crate::structure::bm_edge_other_vert(bm, e, v);
            if bm.vert(v_other).head.index == VERT_INDEX_INIT {
                bm.vpool[v_other].head.index = desired_tag; /* set_dirty! */
                r_verts_tagged.push(v_other);
            }
        }
    }
    r_verts_tagged
}

/// `BM_mesh_decimate_unsubdivide_ex` — for `iterations` steps, dissolve
/// an independent set of eligible (fan/chain) verts.
pub fn bm_mesh_decimate_unsubdivide_ex(bm: &mut BMesh, iterations: i32, tag_only: bool) {
    if !tag_only {
        for v in 0..bm.totvert as usize {
            bm.vpool[v].head.hflag |= BM_ELEM_TAG;
        }
    }
    for _iter_step in 0..iterations {
        let mut verts_were_marked_for_dissolve = false;
        /* tag eligible verts */
        for v in VertsOfMesh::new(bm).collect::<Vec<_>>() {
            if bm.vert(v).head.hflag & BM_ELEM_TAG != 0
                && bm_vert_dissolve_fan_or_chain_test(bm, v, false).is_some()
            {
                bm.vpool[v].head.index = VERT_INDEX_INIT; /* set_dirty! */
            } else {
                bm.vpool[v].head.index = VERT_INDEX_IGNORE; /* set_dirty! */
            }
        }
        /* main loop: island-expansion alternating collapse/ignore tags */
        for v in VertsOfMesh::new(bm).collect::<Vec<_>>() {
            if bm.vert(v).head.index != VERT_INDEX_INIT { continue; }
            bm.vpool[v].head.index = VERT_INDEX_IGNORE;
            let mut verts_ignore = vec![v];
            loop {
                let verts_collapse = bm_tag_untagged_neighbors(bm, &verts_ignore, VERT_INDEX_DO_COLLAPSE);
                if verts_collapse.is_empty() { break; }
                verts_were_marked_for_dissolve = true;
                verts_ignore = bm_tag_untagged_neighbors(bm, &verts_collapse, VERT_INDEX_IGNORE);
                if verts_ignore.is_empty() { break; }
            }
        }
        if !verts_were_marked_for_dissolve { break; }
        /* remove all verts tagged for removal */
        for v in VertsOfMesh::new(bm).collect::<Vec<_>>() {
            if bm.vert(v).head.index == VERT_INDEX_DO_COLLAPSE {
                bm_vert_dissolve_fan_or_chain(bm, v);
            }
        }
    }
    /* Ensure the vert index values will be recomputed. */
    bm.elem_index_dirty = true;
}

/// `BM_mesh_decimate_unsubdivide`.
pub fn bm_mesh_decimate_unsubdivide(bm: &mut BMesh, iterations: i32) {
    bm_mesh_decimate_unsubdivide_ex(bm, iterations, false);
}
