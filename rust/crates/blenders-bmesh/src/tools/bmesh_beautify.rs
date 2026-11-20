//! Port of `tools/bmesh_beautify.cc` — iterative beauty-driven edge rotation.
use std::collections::{BinaryHeap, HashSet};
use crate::{BMesh, BM_ELEM_TAG, BM_NIL};

/// `BMElemRestrict` flags (bmesh_beautify.hh).
pub const VERT_RESTRICT_TAG: i16 = 1 << 0;
/// `EDGE_RESTRICT_DEGENERATE`.
pub const EDGE_RESTRICT_DEGENERATE: i16 = 1 << 1;

/// `EdRotState` — edge rotation fingerprint for re-visit suppression.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct EdRotState {
    /// edge vert indices (ordered small -> large).
    v_pair: [i32; 2],
    /// face vert indices (small -> large).
    f_pair: [i32; 2],
}

macro_rules! edge_ord {
    ($a:expr, $b:expr) => { if $a > $b { std::mem::swap(&mut $a, &mut $b); } };
}

/// `erot_state_ex` — fill (edge-vert pair, face-vert pair).
fn erot_state_ex(bm: &BMesh, e: usize, v_index: &mut [i32; 2], f_index: &mut [i32; 2]) {
    debug_assert!(crate::query::bm_edge_is_manifold(bm, e));
    let l = bm.edge(e).l;
    let l_radial = bm.loo(l).radial_next;
    debug_assert!(!crate::structure::bm_vert_in_edge(bm, e, bm.loo(bm.loo(l).prev).v));
    debug_assert!(!crate::structure::bm_vert_in_edge(bm, e, bm.loo(bm.loo(l_radial).prev).v));
    let (mut a, mut b) = {
        let ed = bm.edge(e);
        (bm.vert(ed.v1).head.index, bm.vert(ed.v2).head.index)
    };
    edge_ord!(a, b);
    v_index[0] = a; v_index[1] = b;
    let mut c = bm.vert(bm.loo(bm.loo(l).prev).v).head.index;
    let mut d = bm.vert(bm.loo(bm.loo(l_radial).prev).v).head.index;
    edge_ord!(c, d);
    f_index[0] = c; f_index[1] = d;
}
fn erot_state_current(bm: &BMesh, e: usize) -> EdRotState {
    let mut s = EdRotState { v_pair: [0; 2], f_pair: [0; 2] };
    erot_state_ex(bm, e, &mut s.v_pair, &mut s.f_pair);
    s
}
fn erot_state_alternate(bm: &BMesh, e: usize) -> EdRotState {
    let mut s = EdRotState { v_pair: [0; 2], f_pair: [0; 2] };
    erot_state_ex(bm, e, &mut s.f_pair, &mut s.v_pair);
    s
}

/// `bm_edge_calc_rotate_beauty__angle` — angle-based improvement.
fn bm_edge_calc_rotate_beauty__angle(v1: crate::Vec3, v2: crate::Vec3, v3: crate::Vec3, v4: crate::Vec3) -> f32 {
    use blenders_math::geom::{angle_normalized_v3v3, normal_tri};
    /* edge (2-4), current state */
    let no_a = normal_tri(v2, v3, v4).0;
    let no_b = normal_tri(v2, v4, v1).0;
    let angle_24 = angle_normalized_v3v3(no_a, no_b);
    /* edge (1-3), new state — check for degenerate outcome */
    if normal_tri(v1, v2, v3).0.length_squared() == 0.0
        || normal_tri(v1, v3, v4).0.length_squared() == 0.0
    {
        return f32::MAX;
    }
    let no_a = normal_tri(v1, v2, v3).0;
    let no_b = normal_tri(v1, v3, v4).0;
    let angle_13 = angle_normalized_v3v3(no_a, no_b);
    angle_13 - angle_24
}

/// `BM_verts_calc_rotate_beauty`.
pub fn bm_verts_calc_rotate_beauty(
    bm: &BMesh, v1: usize, v2: usize, v3: usize, v4: usize,
    flag: i16, method: i16,
) -> f32 {
    if flag & VERT_RESTRICT_TAG != 0 {
        let (va, vb) = (v1, v3);
        if (bm.vert(va).head.hflag & BM_ELEM_TAG != 0) == (bm.vert(vb).head.hflag & BM_ELEM_TAG != 0) {
            return f32::MAX;
        }
    }
    if v1 == v3 {
        return f32::MAX;
    }
    match method {
        0 => blenders_bli::polyfill_2d_beautify::polyfill_edge_calc_rotate_beauty_area(
            bm.vert(v1).co.into(), bm.vert(v2).co.into(),
            bm.vert(v3).co.into(), bm.vert(v4).co.into(),
            flag & EDGE_RESTRICT_DEGENERATE != 0),
        _ => bm_edge_calc_rotate_beauty__angle(
            bm.vert(v1).co, bm.vert(v2).co, bm.vert(v3).co, bm.vert(v4).co),
    }
}

/// `bm_edge_calc_rotate_beauty` — v1..v4 from e's two triangle loops.
fn bm_edge_calc_rotate_beauty(bm: &BMesh, e: usize, flag: i16, method: i16) -> f32 {
    let l = bm.edge(e).l;
    let l_radial = bm.loo(l).radial_next;
    let v1 = bm.loo(bm.loo(l).prev).v;         /* first vert co */
    let v2 = bm.loo(l).v;                    /* e->v1 or e->v2 */
    let v3 = bm.loo(bm.loo(l_radial).prev).v;/* second vert co */
    let v4 = bm.loo(bm.loo(l).next).v;       /* e->v1 or e->v2 */
    bm_verts_calc_rotate_beauty(bm, v1, v2, v3, v4, flag, method)
}

/// `edge_in_array`.
fn edge_in_array(bm: &BMesh, e: usize, edge_array: &[usize]) -> bool {
    let index = bm.edge(e).head.index;
    index >= 0 && (index as usize) < edge_array.len() && e == edge_array[index as usize]
}

/// `bm_edge_update_beauty_cost_single`.
fn bm_edge_update_beauty_cost_single(
    bm: &BMesh, e: usize,
    eheap: &mut BinaryHeap<std::cmp::Reverse<CostEdge>>,
    eheap_table: &mut [Option<usize>],
    edge_state_arr: &[HashSet<EdRotState>],
    edge_array: &[usize],
    flag: i16, method: i16,
) {
    if edge_in_array(bm, e, edge_array) {
        let i = bm.edge(e).head.index as usize;
        let e_state_set = &edge_state_arr[i];
        /* stale entries already in heap are skipped lazily via eheap_table version —
         * we encode cost+edge; removal handled by validity flag table. */
        eheap_table[i] = None;
        debug_assert!(crate::query::bm_edge_is_manifold(bm, e));
        let e_state_alt = erot_state_alternate(bm, e);
        if e_state_set.contains(&e_state_alt) { return; }
        let cost = bm_edge_calc_rotate_beauty(bm, e, flag, method);
        if cost < 0.0 {
            eheap.push(std::cmp::Reverse(CostEdge(cost, e)));
            eheap_table[i] = Some(e);
        }
    }
}

/// `bm_edge_update_beauty_cost` — update the 4 fan edges.
fn bm_edge_update_beauty_cost(
    bm: &BMesh, e: usize,
    eheap: &mut BinaryHeap<std::cmp::Reverse<CostEdge>>,
    eheap_table: &mut [Option<usize>],
    edge_state_arr: &[HashSet<EdRotState>],
    edge_array: &[usize],
    flag: i16, method: i16,
) {
    let l = bm.edge(e).l;
    let l_radial = bm.loo(l).radial_next;
    let e_arr = [
        bm.loo(bm.loo(l).next).e,
        bm.loo(bm.loo(l).prev).e,
        bm.loo(bm.loo(l_radial).next).e,
        bm.loo(bm.loo(l_radial).prev).e,
    ];
    debug_assert_eq!(bm.face(bm.loo(l).f).len, 3);
    debug_assert_eq!(bm.face(bm.loo(l_radial).f).len, 3);
    debug_assert_eq!(crate::query::bm_edge_face_count(bm, e), 2);
    for e2 in e_arr {
        bm_edge_update_beauty_cost_single(
            bm, e2, eheap, eheap_table, edge_state_arr, edge_array, flag, method);
    }
}

/// Heap entry: (cost, edge). BinaryHeap is max-first → Reverse for min-first.
#[derive(Clone, Copy, PartialEq)]
struct CostEdge(f32, usize);
impl Eq for CostEdge {}
impl Ord for CostEdge {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&o.0).unwrap_or(std::cmp::Ordering::Equal)
            .then(self.1.cmp(&o.1))
    }
}
impl PartialOrd for CostEdge {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> { Some(self.cmp(o)) }
}

/// `BM_mesh_beautify_fill` — rotate edges to maximize beauty until no
/// improving rotation remains.
/// `oflag_edge`/`oflag_face`: api_flag bits set on rotated edges / their faces.
pub fn bm_mesh_beautify_fill(
    bm: &mut BMesh,
    edge_array: &mut Vec<usize>,
    flag: i16, method: i16,
    oflag_edge: u8, oflag_face: u8,
) {
    let edge_array_len = edge_array.len();
    let mut edge_state_arr: Vec<HashSet<EdRotState>> = (0..edge_array_len).map(|_| HashSet::new()).collect();
    let mut eheap: BinaryHeap<std::cmp::Reverse<CostEdge>> = BinaryHeap::new();
    let mut eheap_table: Vec<Option<usize>> = vec![None; edge_array_len];
    for (i, &e) in edge_array.iter().enumerate() {
        let cost = bm_edge_calc_rotate_beauty(bm, e, flag, method);
        if cost < 0.0 {
            eheap.push(std::cmp::Reverse(CostEdge(cost, e)));
            eheap_table[i] = Some(e);
        }
        bm.epool[e].head.index = i as i32; /* set_dirty */
    }
    bm.elem_index_dirty = true;
    while let Some(std::cmp::Reverse(CostEdge(_cost, e))) = eheap.pop() {
        let i = bm.edge(e).head.index as usize;
        /* skip stale heap entries (edge no longer the array's edge) */
        if eheap_table[i].is_none() || edge_array[i] != e { continue; }
        eheap_table[i] = None;
        debug_assert_eq!(crate::query::bm_edge_face_count(bm, e), 2);
        let e_new = crate::mods::bm_edge_rotate(bm, e, false, crate::mods::BM_EDGEROT_CHECK_EXISTS);
        debug_assert!(e_new == BM_NIL || crate::query::bm_edge_face_count(bm, e_new) == 2);
        if e_new != BM_NIL {
            let e_state = erot_state_current(bm, e_new);
            debug_assert!(!edge_state_arr[i].contains(&e_state));
            edge_state_arr[i].insert(e_state);
            edge_array[i] = e_new;
            bm.epool[e_new].head.index = i as i32;
            bm_edge_update_beauty_cost(bm, e_new, &mut eheap, &mut eheap_table,
                                       &edge_state_arr, edge_array, flag, method);
            if oflag_edge != u8::MAX {
                bm.epool[e_new].head.api_flag |= 1 << oflag_edge;
            }
            if oflag_face != u8::MAX {
                let l = bm.edge(e_new).l;
                let (f1, f2) = (bm.loo(l).f, bm.loo(bm.loo(l).radial_next).f);
                bm.fpool[f1].head.api_flag |= 1 << oflag_face;
                bm.fpool[f2].head.api_flag |= 1 << oflag_face;
            }
        }
    }
}
