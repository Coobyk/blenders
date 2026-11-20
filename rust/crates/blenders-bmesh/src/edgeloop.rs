//! Port of `bmesh_edgeloop.cc` — find path/loops of tagged edges.
//! `BMEdgeLoopStore` becomes an index-based struct (`verts: VecDeque<usize>`).

use crate::core::*;
use crate::iterators::*;
use crate::structure::*;
use crate::{mods, query};
use crate::{BMesh, BM_NIL, BM_ELEM_INTERNAL_TAG};
use blenders_math::vector::Vec3;
use std::collections::{HashSet, VecDeque};

pub const BM_EDGELOOP_IS_CLOSED: u8 = 1 << 0;
pub const BM_EDGELOOP_IS_CLOSED_JUNCTION: u8 = 1 << 1;

/// `BMEdgeLoopStore`.
pub struct BMEdgeLoopStore {
    pub verts: VecDeque<usize>,
    pub len: i32,
    pub flag: u8,
    pub co: Vec3,
    pub no: Vec3,
}

/// `BMEdgeLoopFind_Params`.
pub struct BMEdgeLoopFindParams {
    pub use_vert_junction: bool,
}

fn bm_vert_other_tag(bm: &BMesh, v: usize, v_prev: usize) -> (i32, usize) {
    let mut e_next = BM_NIL;
    let mut count = 0;
    for e in EdgesOfVert::new(bm, v) {
        if (bm.edge(e).head.hflag & BM_ELEM_INTERNAL_TAG) != 0 {
            let v_other = if bm.edge(e).v1 == v {
                bm.edge(e).v2
            } else {
                bm.edge(e).v1
            };
            if v_other != v_prev {
                e_next = e;
                count += 1;
            }
        }
    }
    (count, e_next)
}

fn bm_vert_is_junction(bm: &BMesh, v: usize) -> bool {
    let mut count = 0;
    for e in EdgesOfVert::new(bm, v) {
        if (bm.edge(e).head.hflag & BM_ELEM_INTERNAL_TAG) != 0 {
            count += 1;
            if count > 2 {
                return true;
            }
        }
    }
    false
}

fn bm_loop_build(
    bm: &mut BMesh,
    el_store: &mut BMEdgeLoopStore,
    mut v_prev: usize,
    mut v: usize,
    dir: i32,
    vert_junctions: Option<&HashSet<usize>>,
) -> bool {
    debug_assert!(dir == 1 || dir == -1);
    if (bm.vert(v).head.hflag & BM_ELEM_INTERNAL_TAG) == 0 {
        return true;
    }
    let v_first = v;
    while v != BM_NIL {
        if dir == 1 {
            el_store.verts.push_front(v);
        } else {
            el_store.verts.push_back(v);
        }
        el_store.len += 1;

        if let Some(jset) = vert_junctions {
            if jset.contains(&v) {
                break;
            }
        }
        bm.vpool[v].head.hflag &= !BM_ELEM_INTERNAL_TAG;

        let (count, e_next) = bm_vert_other_tag(bm, v, v_prev);
        let mut v_next = BM_NIL;
        if count == 1 {
            v_next = if bm.edge(e_next).v1 == v {
                bm.edge(e_next).v2
            } else {
                bm.edge(e_next).v1
            };
            bm.epool[e_next].head.hflag &= !BM_ELEM_INTERNAL_TAG;
            if v_next == v_first {
                el_store.flag |= BM_EDGELOOP_IS_CLOSED;
                v_next = BM_NIL;
            }
        } else if count == 0 {
            /* pass */
        } else {
            return false;
        }
        v_prev = v;
        v = v_next;
    }
    true
}

/// `BM_mesh_edgeloops_find` — extract closed/open edge loops of edges where
/// `test_fn` returns true. Returns the found loops.
pub fn bm_mesh_edgeloops_find(
    bm: &mut BMesh,
    test_fn: &dyn Fn(&BMesh, usize) -> bool,
    params: Option<&BMEdgeLoopFindParams>,
) -> Vec<BMEdgeLoopStore> {
    let mut result = Vec::new();
    let use_vert_junction = params.map(|p| p.use_vert_junction).unwrap_or(false);

    for i in 0..bm.vpool.len() {
        bm.vpool[i].head.hflag &= !BM_ELEM_INTERNAL_TAG;
    }

    let mut edges: Vec<usize> = Vec::new();
    for i in 0..bm.epool.len() {
        if bm.edge(i).head.htype != crate::BM_EDGE {
            continue;
        }
        if test_fn(bm, i) {
            bm.epool[i].head.hflag |= BM_ELEM_INTERNAL_TAG;
            let (v1, v2) = (bm.edge(i).v1, bm.edge(i).v2);
            bm.vpool[v1].head.hflag |= BM_ELEM_INTERNAL_TAG;
            bm.vpool[v2].head.hflag |= BM_ELEM_INTERNAL_TAG;
            edges.push(i);
        } else {
            bm.epool[i].head.hflag &= !BM_ELEM_INTERNAL_TAG;
        }
    }

    let mut vert_junction_set: HashSet<usize> = HashSet::new();
    if use_vert_junction {
        for &e in &edges {
            for v_end in [bm.edge(e).v1, bm.edge(e).v2] {
                if !vert_junction_set.contains(&v_end) && bm_vert_is_junction(bm, v_end) {
                    vert_junction_set.insert(v_end);
                }
            }
        }
    }
    let vert_junctions = if use_vert_junction && !vert_junction_set.is_empty() {
        Some(&vert_junction_set)
    } else {
        None
    };

    for &e in &edges {
        if (bm.edge(e).head.hflag & BM_ELEM_INTERNAL_TAG) != 0 {
            let mut el_store = BMEdgeLoopStore {
                verts: VecDeque::new(),
                len: 0,
                flag: 0,
                co: Vec3::default(),
                no: Vec3::default(),
            };
            let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
            let built = bm_loop_build(bm, &mut el_store, v1, v2, 1, vert_junctions)
                && bm_loop_build(bm, &mut el_store, v2, v1, -1, vert_junctions);

            if vert_junctions.is_some() && built && el_store.len >= 2 {
                if el_store.verts.front() == el_store.verts.back() {
                    el_store.verts.pop_back();
                    el_store.len -= 1;
                    el_store.flag |= BM_EDGELOOP_IS_CLOSED_JUNCTION;
                }
            }

            if built && el_store.len > 1 {
                result.push(el_store);
            }
        }
    }

    for &e in &edges {
        bm.epool[e].head.hflag &= !BM_ELEM_INTERNAL_TAG;
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        bm.vpool[v1].head.hflag &= !BM_ELEM_INTERNAL_TAG;
        bm.vpool[v2].head.hflag &= !BM_ELEM_INTERNAL_TAG;
    }
    result
}

/// `BM_mesh_edgeloops_find_path` — bidirectional search for an open path of
/// `test_fn`-matching edges between v_src and v_dst. Returns the loop if found.
pub fn bm_mesh_edgeloops_find_path(
    bm: &mut BMesh,
    test_fn: Option<&dyn Fn(&BMesh, usize) -> bool>,
    v_src: usize,
    v_dst: usize,
) -> Option<BMEdgeLoopStore> {
    debug_assert_ne!(v_src, v_dst);

    for i in 0..bm.vpool.len() {
        bm.vpool[i].head.index = 0;
        bm.vpool[i].head.hflag &= !BM_ELEM_INTERNAL_TAG;
    }
    bm.elem_index_dirty = true;

    let mut edges: Vec<usize> = Vec::new();
    for i in 0..bm.epool.len() {
        if bm.edge(i).head.htype != crate::BM_EDGE {
            continue;
        }
        let tagged = match test_fn {
            Some(f) => f(bm, i),
            None => true,
        };
        if tagged {
            bm.epool[i].head.hflag |= BM_ELEM_INTERNAL_TAG;
            let (v1, v2) = (bm.edge(i).v1, bm.edge(i).v2);
            bm.vpool[v1].head.hflag |= BM_ELEM_INTERNAL_TAG;
            bm.vpool[v2].head.hflag |= BM_ELEM_INTERNAL_TAG;
            edges.push(i);
        } else {
            bm.epool[i].head.hflag &= !BM_ELEM_INTERNAL_TAG;
        }
    }

    /* bidirectional BFS; `head.index` stores step-from-end + direction sign */
    let mut lb_src: VecDeque<usize> = VecDeque::new();
    let mut lb_dst: VecDeque<usize> = VecDeque::new();
    bm.vpool[v_src].head.index = 1;
    bm.vpool[v_dst].head.index = -1;
    bm.vpool[v_src].e = bm.vert(v_src).e; /* keep edge ref for backtrack */
    lb_src.push_back(v_src);
    lb_dst.push_back(v_dst);
    let mut v_match = BM_NIL;
    let mut found_match = false;
    /* back-pointer via vert .e reuse is fragile; use explicit parent map */
    let mut parent: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    parent.insert(v_src, v_src);
    parent.insert(v_dst, v_dst);

    'outer: loop {
        for (lb, dir) in [(&mut lb_src, 1i32), (&mut lb_dst, -1i32)] {
            let mut lb_next: VecDeque<usize> = VecDeque::new();
            while let Some(vs) = lb.pop_front() {
                let vs_iter_next = bm.vert(vs).head.index + dir;
                let v_edges: Vec<usize> = EdgesOfVert::new(bm, vs).collect();
                for e in v_edges {
                    if (bm.edge(e).head.hflag & BM_ELEM_INTERNAL_TAG) != 0 {
                        let v_next = if bm.edge(e).v1 == vs {
                            bm.edge(e).v2
                        } else {
                            bm.edge(e).v1
                        };
                        bm.epool[e].head.hflag &= !BM_ELEM_INTERNAL_TAG;
                        let v_next_index = bm.vert(v_next).head.index;
                        if v_next_index == 0 {
                            bm.vpool[v_next].head.index = vs_iter_next;
                            parent.insert(v_next, vs);
                            lb_next.push_back(v_next);
                        } else if (dir < 0) == (v_next_index < 0) {
                            /* same side */
                        } else {
                            /* met! store both end verts */
                            let (a, b) = if dir == 1 { (vs, v_next) } else { (v_next, vs) };
                            parent.insert(b, a);
                            v_match = a;
                            /* build path */
                            let mut verts: VecDeque<usize> = VecDeque::new();
                            let mut v = a;
                            loop {
                                verts.push_front(v);
                                if v == v_src {
                                    break;
                                }
                                v = parent[&v];
                            }
                            let mut v = b;
                            loop {
                                verts.push_back(v);
                                if v == v_dst {
                                    break;
                                }
                                v = parent[&v];
                            }
                            found_match = true;
                            for &e in &edges {
                                bm.epool[e].head.hflag &= !BM_ELEM_INTERNAL_TAG;
                                let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
                                bm.vpool[v1].head.hflag &= !BM_ELEM_INTERNAL_TAG;
                                bm.vpool[v2].head.hflag &= !BM_ELEM_INTERNAL_TAG;
                            }
                            return Some(BMEdgeLoopStore {
                                len: verts.len() as i32,
                                verts,
                                flag: 0,
                                co: Vec3::default(),
                                no: Vec3::default(),
                            });
                        }
                    }
                }
            }
            *lb = lb_next;
        }
        if lb_src.is_empty() || lb_dst.is_empty() {
            break 'outer;
        }
    }

    let _ = (v_match, found_match);
    for &e in &edges {
        bm.epool[e].head.hflag &= !BM_ELEM_INTERNAL_TAG;
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        bm.vpool[v1].head.hflag &= !BM_ELEM_INTERNAL_TAG;
        bm.vpool[v2].head.hflag &= !BM_ELEM_INTERNAL_TAG;
    }
    None
}

/// `BM_edgeloop_copy`.
pub fn bm_edgeloop_copy(src: &BMEdgeLoopStore) -> BMEdgeLoopStore {
    BMEdgeLoopStore {
        verts: src.verts.clone(),
        len: src.len,
        flag: src.flag,
        co: src.co,
        no: src.no,
    }
}

/// `BM_edgeloop_from_verts`.
pub fn bm_edgeloop_from_verts(v_arr: &[usize], is_closed: bool) -> BMEdgeLoopStore {
    BMEdgeLoopStore {
        verts: v_arr.iter().copied().collect(),
        len: v_arr.len() as i32,
        flag: if is_closed { BM_EDGELOOP_IS_CLOSED } else { 0 },
        co: Vec3::default(),
        no: Vec3::default(),
    }
}

/// `BM_edgeloop_is_closed`.
pub fn bm_edgeloop_is_closed(el: &BMEdgeLoopStore) -> bool {
    (el.flag & BM_EDGELOOP_IS_CLOSED) != 0
}

/// `BM_edgeloop_is_closed_junction`.
pub fn bm_edgeloop_is_closed_junction(el: &BMEdgeLoopStore) -> bool {
    (el.flag & BM_EDGELOOP_IS_CLOSED_JUNCTION) != 0
}

/// `BM_edgeloop_verts_get`.
pub fn bm_edgeloop_verts_get(el: &BMEdgeLoopStore) -> &VecDeque<usize> {
    &el.verts
}

/// `BM_edgeloop_length_get`.
pub fn bm_edgeloop_length_get(el: &BMEdgeLoopStore) -> i32 {
    el.len
}

/// `BM_edgeloop_normal_get` / `BM_edgeloop_center_get`.
pub fn bm_edgeloop_normal_get(el: &BMEdgeLoopStore) -> Vec3 {
    el.no
}
pub fn bm_edgeloop_center_get(el: &BMEdgeLoopStore) -> Vec3 {
    el.co
}

/// `BM_edgeloop_edges_get`.
pub fn bm_edgeloop_edges_get(bm: &BMesh, el: &BMEdgeLoopStore) -> Vec<usize> {
    let verts: Vec<usize> = el.verts.iter().copied().collect();
    let mut edges = Vec::with_capacity(verts.len());
    for w in verts.windows(2) {
        let e = bmesh_disk_edge_exists(bm, w[0], w[1]);
        debug_assert_ne!(e, BM_NIL);
        edges.push(e);
    }
    if (el.flag & BM_EDGELOOP_IS_CLOSED) != 0 {
        let e = bmesh_disk_edge_exists(bm, verts[0], verts[verts.len() - 1]);
        debug_assert_ne!(e, BM_NIL);
        edges.push(e);
    }
    edges
}

const EDGELOOP_EPS: f32 = 1e-5;

/// `BM_edgeloop_calc_center` — edge-length weighted centroid.
pub fn bm_edgeloop_calc_center(bm: &BMesh, el_store: &mut BMEdgeLoopStore) {
    let verts: Vec<usize> = el_store.verts.iter().copied().collect();
    let n = verts.len();
    if n < 2 {
        return;
    }
    el_store.co = Vec3::default();
    let mut totw = 0.0f32;
    let get = |i: usize| -> Vec3 { bm.vert(verts[i]).co };
    let mut w_prev = (get(n - 2) - get(n - 1)).length();
    let mut v_curr = get(n - 1);
    for i in 0..n {
        let v_next = get(i);
        let w_curr = (v_curr - v_next).length();
        let w = w_curr + w_prev;
        el_store.co = el_store.co + v_curr * w;
        totw += w;
        w_prev = w_curr;
        v_curr = v_next;
    }
    if totw != 0.0 {
        el_store.co = el_store.co * (1.0 / totw);
    }
}

/// `BM_edgeloop_calc_normal` — Newell's method.
pub fn bm_edgeloop_calc_normal(bm: &BMesh, el_store: &mut BMEdgeLoopStore) -> bool {
    let verts: Vec<usize> = el_store.verts.iter().copied().collect();
    let n = verts.len();
    if n < 2 {
        return false;
    }
    el_store.no = Vec3::default();
    for i in 0..n {
        let v_prev = bm.vert(verts[(i + n - 1) % n]).co;
        let v_curr = bm.vert(verts[i]).co;
        /* add_newell_cross_v3_v3v3 */
        el_store.no.x += (v_prev.y - v_curr.y) * (v_prev.z + v_curr.z);
        el_store.no.y += (v_prev.z - v_curr.z) * (v_prev.x + v_curr.x);
        el_store.no.z += (v_prev.x - v_curr.x) * (v_prev.y + v_curr.y);
    }
    if el_store.no.length() < EDGELOOP_EPS {
        el_store.no = Vec3::new(0.0, 0.0, 1.0);
        return false;
    }
    el_store.no.normalize();
    true
}

/// `BM_edgeloop_calc_normal_aligned`.
pub fn bm_edgeloop_calc_normal_aligned(
    bm: &BMesh,
    el_store: &mut BMEdgeLoopStore,
    no_align: Vec3,
) -> bool {
    let verts: Vec<usize> = el_store.verts.iter().copied().collect();
    let n = verts.len();
    if n < 2 {
        return false;
    }
    el_store.no = Vec3::default();
    for i in 0..n {
        let v_prev = bm.vert(verts[(i + n - 1) % n]).co;
        let v_curr = bm.vert(verts[i]).co;
        let dir = v_curr - v_prev;
        let cross = no_align.cross(dir);
        el_store.no = el_store.no + dir.cross(cross);
    }
    if el_store.no.length() < EDGELOOP_EPS {
        el_store.no = Vec3::new(0.0, 0.0, 1.0);
        return false;
    }
    el_store.no.normalize();
    true
}

/// `BM_edgeloop_flip`.
pub fn bm_edgeloop_flip(el_store: &mut BMEdgeLoopStore) {
    el_store.no = el_store.no * -1.0;
    el_store.verts.make_contiguous().reverse();
}

/// `BM_edgeloop_overlap_check` — do the two loops share a vert?
pub fn bm_edgeloop_overlap_check(
    bm: &mut BMesh,
    a: &BMEdgeLoopStore,
    b: &BMEdgeLoopStore,
) -> bool {
    let (a, b) = if a.len > b.len { (b, a) } else { (a, b) };
    for &v in &a.verts {
        bm.vpool[v].head.hflag |= BM_ELEM_INTERNAL_TAG;
    }
    for &v in &b.verts {
        bm.vpool[v].head.hflag &= !BM_ELEM_INTERNAL_TAG;
    }
    let mut overlap = false;
    for &v in &a.verts {
        if (bm.vert(v).head.hflag & BM_ELEM_INTERNAL_TAG) == 0 {
            overlap = true;
        }
        bm.vpool[v].head.hflag &= !BM_ELEM_INTERNAL_TAG;
    }
    overlap
}

/// `divide_floor_i` (BLI_math_base.h).
#[inline]
fn divide_floor_i(a: i32, b: i32) -> i32 {
    a.div_euclid(b)
}

/// `BM_mesh_edgeloops_calc_order` — reorder loops by greedy nearest-next.
pub fn bm_mesh_edgeloops_calc_order(eloops: &mut Vec<BMEdgeLoopStore>, use_normals: bool) {
    let mut cent = Vec3::ZERO;
    let mut tot = 0;
    for el in eloops.iter() {
        if el.co.is_finite() {
            cent += el.co;
            tot += 1;
        }
    }
    if tot > 0 {
        cent *= 1.0 / tot as f32;
        if !cent.is_finite() {
            cent = Vec3::ZERO;
        }
    }
    /* pick the furthest-out loop */
    let mut ordered: Vec<BMEdgeLoopStore> = Vec::with_capacity(eloops.len());
    {
        let mut best = usize::MAX;
        let mut len_best_sq = -1.0f32;
        for (i, el) in eloops.iter().enumerate() {
            let len_sq = (cent - el.co).length_squared();
            if len_sq > len_best_sq || best == usize::MAX {
                len_best_sq = len_sq;
                best = i;
            }
        }
        ordered.push(eloops.remove(best));
    }
    while !eloops.is_empty() {
        let co = ordered.last().unwrap().co;
        let no = ordered.last().unwrap().no;
        if use_normals {
            debug_assert!((no.length() - 1.0).abs() < 1e-4);
        }
        let mut best = usize::MAX;
        let mut len_best_sq = f32::MAX;
        for (i, el) in eloops.iter().enumerate() {
            let len_sq = if use_normals {
                let dir = (co - el.co).normalized();
                let l = (co - el.co).length();
                l * ((1.0 - dir.dot(no).abs()) + (1.0 - dir.dot(el.no).abs()))
            } else {
                (co - el.co).length_squared()
            };
            if len_sq < len_best_sq || best == usize::MAX {
                len_best_sq = len_sq;
                best = i;
            }
        }
        ordered.push(eloops.remove(best));
    }
    *eloops = ordered;
}

/// `BM_edgeloop_expand` — grow the loop to `el_store_len` verts, optionally
/// splitting edges (recording new edges in `split_edges`).
pub fn bm_edgeloop_expand(
    bm: &mut BMesh,
    el_store: &mut BMEdgeLoopStore,
    el_store_len: i32,
    split: bool,
    split_edges: &mut HashSet<usize>,
) {
    let mut split_swap = true;
    /* EDGE_SPLIT: split edge (copy_vert — other_vert), copy becomes v_split. */
    macro_rules! edge_split {
        ($copy_v:expr, $other_v:expr) => {{
            let e_other = crate::query::bm_edge_exists(bm, $copy_v, $other_v);
            let mut e_split = BM_NIL;
            let v_from = if split_swap { $copy_v } else { $other_v };
            let v_split = mods::bm_edge_split(bm, e_other, v_from, Some(&mut e_split), 0.0);
            bm.vpool[v_split].e = e_split;
            debug_assert_eq!(v_split, bm.edge(e_split).v2);
            split_edges.insert(e_split);
            v_split
        }};
    }
    /* first double until more than half as big */
    while (el_store.len * 2) < el_store_len {
        let mut i = 0usize;
        let n = el_store.verts.len();
        let mut processed = 0usize;
        while processed < n && i < el_store.verts.len() {
            let node_v = el_store.verts[i];
            if !split {
                el_store.verts.insert(i + 1, node_v);
                i += 2;
            } else {
                if i + 1 < el_store.verts.len()
                    || (el_store.flag & BM_EDGELOOP_IS_CLOSED) != 0
                {
                    let other_v = if i + 1 < el_store.verts.len() {
                        el_store.verts[i + 1]
                    } else {
                        el_store.verts[0]
                    };
                    let v_split = edge_split!(node_v, other_v);
                    el_store.verts.insert(i + 1, v_split);
                    i += 2;
                } else {
                    let other_v = el_store.verts[i - 1];
                    let v_split = edge_split!(node_v, other_v);
                    el_store.verts.insert(i, v_split);
                    i += 2;
                }
                split_swap = !split_swap;
            }
            el_store.len += 1;
            processed += 1;
        }
        split_swap = !split_swap;
    }
    if el_store.len < el_store_len {
        /* evenly spread the rest (BLI_FOREACH_SPARSE_RANGE) */
        let src = el_store.len;
        let dst = el_store_len - el_store.len;
        let src2 = src * 2;
        let dst2 = dst * 2;
        let mut error = dst2 - src;
        let mut iter = 0i32;
        let mut i = 0usize; /* node index cursor */
        let mut iter_prev = 0i32;
        loop {
            let delta = divide_floor_i(error, dst2);
            iter -= delta;
            if iter >= src {
                break;
            }
            while iter_prev < iter {
                i += 1;
                iter_prev += 1;
            }
            let node_v = el_store.verts[i];
            if !split {
                el_store.verts.insert(i + 1, node_v);
                i += 1;
            } else {
                if i + 1 < el_store.verts.len()
                    || (el_store.flag & BM_EDGELOOP_IS_CLOSED) != 0
                {
                    let other_v = if i + 1 < el_store.verts.len() {
                        el_store.verts[i + 1]
                    } else {
                        el_store.verts[0]
                    };
                    let v_split = edge_split!(node_v, other_v);
                    el_store.verts.insert(i + 1, v_split);
                    i += 1;
                } else {
                    let other_v = el_store.verts[i - 1];
                    let v_split = edge_split!(node_v, other_v);
                    el_store.verts.insert(i, v_split);
                    i += 1;
                }
                split_swap = !split_swap;
            }
            el_store.len += 1;
            iter_prev += 1;
            i += 1;
            error -= delta * dst2 + src2;
        }
    }
    debug_assert_eq!(el_store.len, el_store_len);
}
