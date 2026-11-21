//! Port of `intern/pbvh_bmesh.cc` — the dyntopo PBVH backend on top
//! of `blenders_bmesh`: `BMeshNode` (Leaf/flag/vert-face sets +
//! orig arrays), node split/limit_ensure (slow + fast builders),
//! vert/face create/remove + ownership transfer, long/short edge
//! queues, `pbvh_bmesh_split_edge`, `pbvh_bmesh_collapse_edge` +
//! flap attribute merges, `bmesh_update_topology`, `Tree::
//! from_bmesh` fast builder, node raycast/nearest/normal updates.
//! CustomData node/mask offsets become `HashMap<usize, _>` maps
//! (`DyntopoCD`); `BMLog` is an injected trait (`bmesh_log.cc`
//! still pending); `parallel_for` → serial; mempool→Vec.

use std::collections::{BinaryHeap, HashMap, HashSet};

use blenders_bmesh::core::{bm_edge_create, bm_edge_kill, bm_face_create, bm_face_kill, bm_vert_create, bm_vert_kill};
use blenders_bmesh::normals::bm_vert_normal_update;

use blenders_bmesh::query::*;
use blenders_bmesh::structure::bm_edge_other_vert;
use blenders_bmesh::{BMesh, BM_CREATE_NO_DOUBLE, BM_ELEM_HIDDEN, BM_ELEM_SEAM, BM_ELEM_SMOOTH, BM_ELEM_TAG, BM_NIL};
use blenders_math::vector::Vec3;

/// `leaf_limit` — TODO from C: choose leaf limit better.
pub const LEAF_LIMIT: usize = 400;

/// `dyntopo_node_none`.
pub const DYNTOPO_NODE_NONE: i32 = -1;

/// `Node::Leaf` flag bit.
pub const NODE_FLAG_LEAF: u16 = 1 << 0;
/// `Node::UpdateTopology` flag bit.
pub const NODE_FLAG_UPDATE_TOPOLOGY: u16 = 1 << 1;
/// `Node::TopologyUpdated` flag bit.
pub const NODE_FLAG_TOPOLOGY_UPDATED: u16 = 1 << 2;
/// `Node::FullyHidden` flag bit.
pub const NODE_FLAG_FULLY_HIDDEN: u16 = 1 << 3;

/// `PBVHTopologyUpdateMode` bits.
pub const PBVH_COLLAPSE: i32 = 1;
/// `PBVH_Subdivide`.
pub const PBVH_SUBDIVIDE: i32 = 2;

/// The CustomData int layers the C file keeps as offsets become
/// per-element maps: vert→node index, face→node index, vert→mask.
#[derive(Default)]
pub struct DyntopoCD {
    /// `.sculpt_dyntopo_node_id_vertex` (`-1` = none).
    pub vert_node: HashMap<usize, i32>,
    /// `.sculpt_dyntopo_node_id_face`.
    pub face_node: HashMap<usize, i32>,
    /// `.sculpt_mask` (defaults to 0.0 when `cd_vert_mask_offset`
    /// would be `-1`).
    pub vert_mask: Option<HashMap<usize, f32>>,
}

impl DyntopoCD {
    /// `BM_ELEM_CD_GET_INT(v, cd_vert_node_offset)`.
    pub fn vert_node_id(&self, v: usize) -> i32 {
        *self.vert_node.get(&v).unwrap_or(&DYNTOPO_NODE_NONE)
    }
    /// `BM_ELEM_CD_GET_INT(f, cd_face_node_offset)`.
    pub fn face_node_id(&self, f: usize) -> i32 {
        *self.face_node.get(&f).unwrap_or(&DYNTOPO_NODE_NONE)
    }
    /// `BM_ELEM_CD_GET_FLOAT(v, cd_vert_mask_offset)` — 0.0 when the
    /// mask layer is absent.
    pub fn vert_mask_value(&self, v: usize) -> f32 {
        self.vert_mask
            .as_ref()
            .and_then(|m| m.get(&v))
            .copied()
            .unwrap_or(0.0)
    }
}

/// `BMeshNode` — the PBVH leaf/interior node for the dyntopo
/// backend (`bm_faces_`, vert ownership sets, bounds, orig arrays).
#[derive(Default)]
pub struct BMeshNode {
    /// `flag_` — `Node::Flag` bitmask.
    pub flag_: u16,
    /// `parent_` (-1 for the root).
    pub parent_: i32,
    /// `children_offset_` — index of first child (interior nodes).
    pub children_offset_: i32,
    /// `bounds_` — node AABB.
    pub bounds_min: Vec3,
    /// `bounds_` max.
    pub bounds_max: Vec3,
    /// `bounds_orig_`.
    pub bounds_orig_min: Vec3,
    /// `bounds_orig_` max.
    pub bounds_orig_max: Vec3,
    /// `bm_faces_` — faces owned by this leaf (face indices).
    pub bm_faces_: HashSet<usize>,
    /// `bm_unique_verts_` — verts owned exclusively by this node.
    pub bm_unique_verts_: HashSet<usize>,
    /// `bm_other_verts_` — verts shared with other nodes.
    pub bm_other_verts_: HashSet<usize>,
    /// `orig_positions_` — original vert positions (undo/snap).
    pub orig_positions_: Vec<Vec3>,
    /// `orig_tris_` — original triangle indices into `orig_positions_`.
    pub orig_tris_: Vec<[i32; 3]>,
    /// `orig_verts_` — original vert handles.
    pub orig_verts_: Vec<usize>,
}

impl BMeshNode {
    fn bounds_reset(&mut self) {
        self.bounds_min = Vec3::splat(f32::MAX);
        self.bounds_max = Vec3::splat(-f32::MAX);
    }
    fn bounds_reset_orig(&mut self) {
        self.bounds_orig_min = Vec3::splat(f32::MAX);
        self.bounds_orig_max = Vec3::splat(-f32::MAX);
    }
}

/// `BMLog` — injected undo/change logger (`bmesh_log.cc` pending).
pub trait BMLog {
    /// `BM_log_vert_added`.
    fn vert_added(&mut self, v: usize);
    /// `BM_log_face_added`.
    fn face_added(&mut self, f: usize);
    /// `BM_log_face_removed`.
    fn face_removed(&mut self, f: usize);
    /// `BM_log_vert_removed`.
    fn vert_removed(&mut self, v: usize);
    /// `BM_log_vert_before_modified`.
    fn vert_before_modified(&mut self, v: usize);
    /// `BM_log_vert_before_changed` (orig-position lookup for
    /// `copy_original_vert`); returns the logged original `co`.
    fn vert_original_co(&mut self, v: usize) -> Option<Vec3>;
}

/// Injected CustomData-like callbacks for the bits not in `DyntopoCD`.
pub struct PBVHBMeshCtx<'a> {
    /// `BM_data_interp_from_verts` — interpolate vdata into `v`.
    pub data_interp_from_verts:
        &'a mut dyn FnMut(&mut BMesh, usize, usize, usize, f32),
    /// `BM_data_interp_from_edges` — edge attribute merge.
    pub data_interp_from_edges:
        &'a mut dyn FnMut(&mut BMesh, usize, usize, usize, f32),
    /// `CustomData_bmesh_copy_block` on edata.
    pub edata_copy_block: &'a mut dyn FnMut(&mut BMesh, usize, usize),
}

fn negative_bounds() -> (Vec3, Vec3) {
    (Vec3::splat(f32::MAX), Vec3::splat(-f32::MAX))
}

#[inline]
fn min_max(v: Vec3, min: &mut Vec3, max: &mut Vec3) {
    *min = min.min(v);
    *max = max.max(v);
}

#[inline]
fn midpoint(a: Vec3, b: Vec3) -> Vec3 {
    (a + b) * 0.5
}

#[inline]
fn dominant_axis(v: Vec3) -> usize {
    let mut axis = 0;
    if v.y > v.x {
        axis = 1;
    }
    if v.z > if axis == 0 { v.x } else { v.y } {
        axis = 2;
    }
    axis
}

/// `BM_DISK_EDGE_NEXT` — next edge in `v`'s disk cycle.
fn disk_edge_next(bm: &BMesh, e: usize, v: usize) -> usize {
    let ed = bm.edge(e);
    if ed.v1 == v {
        ed.v1_disk_link.next
    } else {
        ed.v2_disk_link.next
    }
}

/// `BM_FACE_FIRST_LOOP` / loop-cycle iteration helper — yields
/// (loop index, vertex) pairs of `f` in winding order.
fn face_loops(bm: &BMesh, f: usize) -> Vec<(usize, usize)> {
    let face = bm.face(f);
    let mut out = Vec::with_capacity(face.len.max(0) as usize);
    if face.l_first == BM_NIL {
        return out;
    }
    let first = face.l_first;
    let mut l = first;
    loop {
        out.push((l, bm.loo(l).v));
        l = bm.loo(l).next;
        if l == first || l == BM_NIL {
            break;
        }
    }
    out
}

/// `BM_LOOPS_OF_VERT` — every (loop, face) pair where the loop's
/// vertex is `v` (inline iterator port).
fn loops_of_vert(bm: &BMesh, v: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let vert = bm.vert(v);
    if vert.e == BM_NIL {
        return out;
    }
    let e_first = vert.e;
    let mut e = e_first;
    loop {
        let ed = bm.edge(e);
        if ed.l != BM_NIL {
            let l_first = ed.l;
            let mut l = l_first;
            loop {
                let lp = bm.loo(l);
                if lp.v == v {
                    out.push((l, lp.f));
                }
                l = lp.radial_next;
                if l == l_first || l == BM_NIL {
                    break;
                }
            }
        }
        e = disk_edge_next(bm, e, v);
        if e == e_first || e == BM_NIL {
            break;
        }
    }
    out
}

/// `BM_FACES_OF_VERT`.
fn faces_of_vert(bm: &BMesh, v: usize) -> Vec<usize> {
    loops_of_vert(bm, v).into_iter().map(|(_, f)| f).collect()
}

/// `bm_edges_from_tri` — `BM_edge_create(NO_DOUBLE)` ×3.
fn bm_edges_from_tri(bm: &mut BMesh, v_tri: &[usize; 3]) -> [usize; 3] {
    [
        bm_edge_create(bm, v_tri[0], v_tri[1], None, BM_CREATE_NO_DOUBLE),
        bm_edge_create(bm, v_tri[1], v_tri[2], None, BM_CREATE_NO_DOUBLE),
        bm_edge_create(bm, v_tri[2], v_tri[0], None, BM_CREATE_NO_DOUBLE),
    ]
}

/// `bm_face_as_array` — the 3 verts of a tri face.
fn bm_face_as_array(bm: &BMesh, f: usize) -> [usize; 3] {
    let loops = face_loops(bm, f);
    debug_assert_eq!(bm.face(f).len, 3);
    [loops[0].1, loops[1].1, loops[2].1]
}

/// `bm_face_exists_tri_from_loop_vert` — find a triangle using
/// `(l.v, l.next.v, v_opposite)` in the edge's radial (either
/// winding).
fn bm_face_exists_tri_from_loop_vert(
    bm: &BMesh,
    l_radial_first: usize,
    v_opposite: usize,
) -> usize {
    let lf = bm.loo(l_radial_first);
    if lf.radial_next != l_radial_first {
        let mut l_iter = lf.radial_next;
        loop {
            let lp = bm.loo(l_iter);
            debug_assert_eq!(bm.face(lp.f).len, 3);
            if bm.loo(lp.prev).v == v_opposite {
                return lp.f;
            }
            l_iter = lp.radial_next;
            if l_iter == l_radial_first {
                break;
            }
        }
    }
    BM_NIL
}

/// `bm_vert_hash_lookup_chain` — follow `deleted_verts` remap.
fn bm_vert_hash_lookup_chain(
    deleted_verts: &HashMap<usize, usize>,
    mut v: usize,
) -> usize {
    loop {
        match deleted_verts.get(&v) {
            None => return v,
            Some(&next) => {
                if next == BM_NIL {
                    return BM_NIL;
                }
                v = next;
            }
        }
    }
}

/// `pbvh_bmesh_node_finalize` — rebuild vert sets + bounds + the
/// `cd_*_node` maps after a split.
fn pbvh_bmesh_node_finalize(
    bm: &mut BMesh,
    nodes: &mut [BMeshNode],
    node_index: usize,
    cd: &mut DyntopoCD,
) {
    let mut has_visible = false;
    let faces: Vec<usize> = nodes[node_index].bm_faces_.iter().copied().collect();

    nodes[node_index].bounds_reset();

    for f in faces {
        cd.face_node.insert(f, node_index as i32);
        for (_, v) in face_loops(bm, f) {
            if !nodes[node_index].bm_unique_verts_.contains(&v) {
                if cd.vert_node_id(v) != DYNTOPO_NODE_NONE {
                    nodes[node_index].bm_other_verts_.insert(v);
                } else {
                    nodes[node_index].bm_unique_verts_.insert(v);
                    cd.vert_node.insert(v, node_index as i32);
                }
            }
            let co = bm.vert(v).co;
            min_max(
                co,
                &mut nodes[node_index].bounds_min,
                &mut nodes[node_index].bounds_max,
            );
        }
        if (bm.face(f).head.hflag & BM_ELEM_HIDDEN) == 0 {
            has_visible = true;
        }
    }

    nodes[node_index].bounds_orig_min = nodes[node_index].bounds_min;
    nodes[node_index].bounds_orig_max = nodes[node_index].bounds_max;

    if has_visible {
        nodes[node_index].flag_ &= !NODE_FLAG_FULLY_HIDDEN;
    } else {
        nodes[node_index].flag_ |= NODE_FLAG_FULLY_HIDDEN;
    }
}

/// `pbvh_bmesh_node_split` — split a leaf past `LEAF_LIMIT` on the
/// widest axis midpoint and recurse.
fn pbvh_bmesh_node_split(
    bm: &mut BMesh,
    nodes: &mut Vec<BMeshNode>,
    node_changed: &mut Vec<bool>,
    cd: &mut DyntopoCD,
    face_bounds: &[(Vec3, Vec3)],
    face_index_of: &dyn Fn(usize) -> usize,
    node_index: usize,
) {
    if nodes[node_index].bm_faces_.len() <= LEAF_LIMIT {
        pbvh_bmesh_node_finalize(bm, nodes, node_index, cd);
        return;
    }

    let (mut cb_min, mut cb_max) = negative_bounds();
    for &f in &nodes[node_index].bm_faces_ {
        let i = face_index_of(f);
        let center = midpoint(face_bounds[i].0, face_bounds[i].1);
        min_max(center, &mut cb_min, &mut cb_max);
    }

    let axis = dominant_axis(cb_max - cb_min);
    let mid = (cb_max[axis] + cb_min[axis]) * 0.5;

    let children = nodes.len();
    nodes[node_index].children_offset_ = children as i32;
    nodes.push(BMeshNode::default());
    nodes.push(BMeshNode::default());
    node_changed.push(true);
    node_changed.push(true);

    nodes[children].flag_ |= NODE_FLAG_LEAF;
    nodes[children + 1].flag_ |= NODE_FLAG_LEAF;
    nodes[children].parent_ = node_index as i32;
    nodes[children + 1].parent_ = node_index as i32;

    let faces: Vec<usize> = nodes[node_index].bm_faces_.iter().copied().collect();
    for f in &faces {
        let i = face_index_of(*f);
        if (face_bounds[i].0[axis] + face_bounds[i].1[axis]) * 0.5 < mid {
            nodes[children].bm_faces_.insert(*f);
        } else {
            nodes[children + 1].bm_faces_.insert(*f);
        }
    }

    /* Enforce at least one primitive in each node. */
    let (empty, other) = if nodes[children].bm_faces_.is_empty() {
        (children, children + 1)
    } else if nodes[children + 1].bm_faces_.is_empty() {
        (children + 1, children)
    } else {
        (usize::MAX, usize::MAX)
    };
    if empty != usize::MAX {
        if let Some(&f) = nodes[other].bm_faces_.iter().next() {
            nodes[empty].bm_faces_.insert(f);
            nodes[other].bm_faces_.remove(&f);
        }
    }

    /* Mark this node's unique verts as unclaimed. */
    for &v in &nodes[node_index].bm_unique_verts_ {
        cd.vert_node.insert(v, DYNTOPO_NODE_NONE);
    }
    for &f in &nodes[node_index].bm_faces_ {
        cd.face_node.insert(f, DYNTOPO_NODE_NONE);
    }
    nodes[node_index].bm_faces_.clear();

    nodes[node_index].flag_ &= !NODE_FLAG_LEAF;
    node_changed[node_index] = true;

    pbvh_bmesh_node_split(bm, nodes, node_changed, cd, face_bounds, face_index_of, children);
    pbvh_bmesh_node_split(bm, nodes, node_changed, cd, face_bounds, face_index_of, children + 1);

    let c = nodes[node_index].children_offset_ as usize;
    nodes[node_index].bounds_min = nodes[c].bounds_min.min(nodes[c + 1].bounds_min);
    nodes[node_index].bounds_max = nodes[c].bounds_max.max(nodes[c + 1].bounds_max);
    nodes[node_index].bounds_orig_min = nodes[node_index].bounds_min;
    nodes[node_index].bounds_orig_max = nodes[node_index].bounds_max;
}

/// `pbvh_bmesh_node_limit_ensure` — builds per-face bounds, calls
/// `pbvh_bmesh_node_split`. `face_index_of` resolves a face to
/// the `face_bounds` index (the C file dirties the elem index).
pub fn pbvh_bmesh_node_limit_ensure(
    bm: &mut BMesh,
    nodes: &mut Vec<BMeshNode>,
    node_changed: &mut Vec<bool>,
    cd: &mut DyntopoCD,
    node_index: usize,
) -> bool {
    let faces_num = nodes[node_index].bm_faces_.len();
    if faces_num <= LEAF_LIMIT {
        return false;
    }

    let mut face_bounds = Vec::with_capacity(faces_num);
    let mut face_index: HashMap<usize, usize> = HashMap::with_capacity(faces_num);

    for (i, &f) in nodes[node_index].bm_faces_.iter().enumerate() {
        let (mut mn, mut mx) = negative_bounds();
        for (_, v) in face_loops(bm, f) {
            min_max(bm.vert(v).co, &mut mn, &mut mx);
        }
        face_bounds.push((mn, mx));
        face_index.insert(f, i);
    }

    bm.elem_index_dirty = true;

    let face_index_of = |f: usize| face_index[&f];
    pbvh_bmesh_node_split(bm, nodes, node_changed, cd, &face_bounds, &face_index_of, node_index);
    true
}

#[inline]
fn node_index_from_vert(cd: &DyntopoCD, v: usize) -> i32 {
    let i = cd.vert_node_id(v);
    debug_assert!(i != DYNTOPO_NODE_NONE);
    i
}

#[inline]
fn node_index_from_face(cd: &DyntopoCD, f: usize) -> i32 {
    let i = cd.face_node_id(f);
    debug_assert!(i != DYNTOPO_NODE_NONE);
    i
}

/// `pbvh_bmesh_vert_create` — midpoint vert in `node_index`'s node.
#[allow(clippy::too_many_arguments)]
fn pbvh_bmesh_vert_create(
    bm: &mut BMesh,
    nodes: &mut Vec<BMeshNode>,
    node_changed: &mut [bool],
    log: &mut dyn BMLog,
    v1: usize,
    v2: usize,
    node_index: usize,
    co: Vec3,
    no: Vec3,
    cd: &mut DyntopoCD,
    data_interp_from_verts: &mut dyn FnMut(&mut BMesh, usize, usize, usize, f32),
) -> usize {
    let v = bm_vert_create(bm, co, None, 0);

    data_interp_from_verts(bm, v1, v2, v, 0.5);
    bm.vert_mut(v).no = no;

    nodes[node_index].bm_unique_verts_.insert(v);
    cd.vert_node.insert(v, node_index as i32);

    nodes[node_index].flag_ |= NODE_FLAG_TOPOLOGY_UPDATED;
    node_changed[node_index] = true;

    log.vert_added(v);
    v
}

/// `pbvh_bmesh_face_create` — tri face + node bookkeeping; the
/// example face's `hflag` is copied verbatim.
#[allow(clippy::too_many_arguments)]
fn pbvh_bmesh_face_create(
    bm: &mut BMesh,
    nodes: &mut Vec<BMeshNode>,
    node_changed: &mut [bool],
    cd: &mut DyntopoCD,
    log: &mut dyn BMLog,
    node_index: usize,
    v_tri: &[usize; 3],
    e_tri: &[usize; 3],
    f_example: usize,
) -> usize {
    debug_assert_eq!(bm_face_exists(bm, v_tri), BM_NIL);
    let f = bm_face_create(bm, v_tri, e_tri, Some(f_example), 0);
    bm.face_mut(f).head.hflag = bm.face(f_example).head.hflag;

    nodes[node_index].bm_faces_.insert(f);
    cd.face_node.insert(f, node_index as i32);

    nodes[node_index].flag_ |= NODE_FLAG_TOPOLOGY_UPDATED;
    node_changed[node_index] = true;
    nodes[node_index].flag_ &= !NODE_FLAG_FULLY_HIDDEN;

    log.face_added(f);
    f
}


/// `pbvh_bmesh_node_vert_use_count_at_most` (index-based).
fn node_vert_use_count_at_most_in(
    bm: &BMesh,
    node_index: usize,
    v: usize,
    cd: &DyntopoCD,
    count_max: i32,
) -> i32 {
    let mut count = 0;
    for f in faces_of_vert(bm, v) {
        if node_index_from_face(cd, f) == node_index as i32 {
            count += 1;
            if count == count_max {
                return count;
            }
        }
    }
    count
}

/// `pbvh_bmesh_vert_other_node_find`.
fn vert_other_node_find(bm: &BMesh, cd: &DyntopoCD, v: usize) -> Option<i32> {
    let current_node = node_index_from_vert(cd, v);
    for f in faces_of_vert(bm, v) {
        let f_node = node_index_from_face(cd, f);
        if f_node != current_node {
            return Some(f_node);
        }
    }
    None
}

/// `pbvh_bmesh_vert_ownership_transfer`.
fn vert_ownership_transfer(
    nodes: &mut Vec<BMeshNode>,
    node_changed: &mut [bool],
    cd: &mut DyntopoCD,
    new_owner_index: usize,
    v: usize,
) {
    let current_owner_index = node_index_from_vert(cd, v) as usize;
    nodes[current_owner_index].flag_ |= NODE_FLAG_TOPOLOGY_UPDATED;
    node_changed[new_owner_index] = true;

    debug_assert!(current_owner_index != new_owner_index);
    nodes[current_owner_index].bm_unique_verts_.remove(&v);

    cd.vert_node.insert(v, new_owner_index as i32);
    nodes[new_owner_index].bm_unique_verts_.insert(v);
    nodes[new_owner_index].bm_other_verts_.remove(&v);

    nodes[new_owner_index].flag_ |= NODE_FLAG_TOPOLOGY_UPDATED;
    node_changed[new_owner_index] = true;
}

/// `pbvh_bmesh_vert_remove`.
fn pbvh_bmesh_vert_remove(
    bm: &BMesh,
    nodes: &mut Vec<BMeshNode>,
    node_changed: &mut [bool],
    cd: &mut DyntopoCD,
    v: usize,
) {
    let mut f_node_index_prev = DYNTOPO_NODE_NONE;

    let v_node = node_index_from_vert(cd, v) as usize;
    nodes[v_node].bm_unique_verts_.remove(&v);
    cd.vert_node.insert(v, DYNTOPO_NODE_NONE);

    for f in faces_of_vert(bm, v) {
        let f_node_index = node_index_from_face(cd, f);
        if f_node_index_prev != f_node_index {
            f_node_index_prev = f_node_index;
            let fi = f_node_index as usize;
            nodes[fi].flag_ |= NODE_FLAG_TOPOLOGY_UPDATED;
            node_changed[fi] = true;
            nodes[fi].bm_other_verts_.remove(&v);
        }
    }
}

/// `pbvh_bmesh_face_remove`.
#[allow(clippy::too_many_arguments)]
fn pbvh_bmesh_face_remove(
    bm: &mut BMesh,
    nodes: &mut Vec<BMeshNode>,
    node_changed: &mut [bool],
    cd: &mut DyntopoCD,
    log: &mut dyn BMLog,
    f: usize,
) {
    let node_index = node_index_from_face(cd, f) as usize;

    for (_, v) in face_loops(bm, f) {
        if node_vert_use_count_at_most_in(bm, node_index, v, cd, 2) == 1 {
            if nodes[node_index].bm_unique_verts_.contains(&v) {
                let new_node = vert_other_node_find(bm, cd, v);
                if let Some(new_node) = new_node {
                    vert_ownership_transfer(
                        nodes,
                        node_changed,
                        cd,
                        new_node as usize,
                        v,
                    );
                }
            } else {
                nodes[node_index].bm_other_verts_.remove(&v);
            }
        }
    }

    nodes[node_index].bm_faces_.remove(&f);
    cd.face_node.insert(f, DYNTOPO_NODE_NONE);

    log.face_removed(f);

    nodes[node_index].flag_ |= NODE_FLAG_TOPOLOGY_UPDATED;
    node_changed[node_index] = true;
}

/// `pbvh_bmesh_edge_loops` — fast-path loop pair, else gather the
/// radial.
fn pbvh_bmesh_edge_loops(bm: &BMesh, e: usize) -> Vec<usize> {
    if let Some((l_a, l_b)) = bm_edge_loop_pair(bm, e) {
        return vec![l_a, l_b];
    }
    let mut loops = Vec::with_capacity(bm_edge_face_count(bm, e).max(0) as usize);
    let ed = bm.edge(e);
    if ed.l != BM_NIL {
        let first = ed.l;
        let mut l = first;
        loop {
            loops.push(l);
            l = bm.loo(l).radial_next;
            if l == first || l == BM_NIL {
                break;
            }
        }
    }
    loops
}

/// `pbvh_bmesh_node_drop_orig`.
fn pbvh_bmesh_node_drop_orig(node: &mut BMeshNode) {
    node.orig_positions_ = Vec::new();
    node.orig_tris_ = Vec::new();
    node.orig_verts_ = Vec::new();
}

/* ------------------------------ EdgeQueue ------------------------------ */

/// `EdgeQueue` — heap of `(v1,v2)` pairs (a `BinaryHeap` with the
/// `heapsimple` min-first ordering inverted via `Ord`).
pub struct EdgeQueue {
    /// `(priority, v1, v2)` — pops the lowest priority (like
    /// `BLI_heapsimple_pop_min`).
    pub heap: BinaryHeap<QueuedPair>,
    /// `center`/`center_proj` — sphere center (+ projection).
    pub center: Vec3,
    /// `center_proj`.
    pub center_proj: Vec3,
    /// `radius_squared`.
    pub radius_squared: f32,
    /// `limit_len_squared`.
    pub limit_len_squared: f32,
    /// `limit_len`.
    pub limit_len: f32,
    /// `view_normal` — Some when projecting (front-face/projected).
    pub view_normal: Option<Vec3>,
    /// `use_front_face`.
    pub use_front_face: bool,
    /// `use_projected`.
    pub use_projected: bool,
}

/// A queued vert pair — min-first ordering by `priority`.
#[derive(Clone, Copy, PartialEq)]
pub struct QueuedPair {
    /// `priority` (heap pops the *smallest*).
    pub priority: f32,
    /// `pair[0]`/`pair[1]`.
    pub v1: usize,
    /// `v2`.
    pub v2: usize,
}

impl Eq for QueuedPair {}
impl Ord for QueuedPair {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        /* Reversed: smallest priority pops first (heapsimple). */
        other
            .priority
            .partial_cmp(&self.priority)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}
impl PartialOrd for QueuedPair {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// `EdgeQueueContext` — the `cd` maps are passed separately
/// (avoids a second mutable borrow of `DyntopoCD`).
pub struct EdgeQueueContext {
    /// `queue`.
    pub queue: EdgeQueue,
}

fn edge_queue_test(bm: &BMesh, e: usize) -> bool {
    (bm.edge(e).head.hflag & BM_ELEM_TAG) != 0
}
fn edge_queue_enable(bm: &mut BMesh, e: usize) {
    bm.edge_mut(e).head.hflag |= BM_ELEM_TAG;
}
fn edge_queue_disable(bm: &mut BMesh, e: usize) {
    bm.edge_mut(e).head.hflag &= !BM_ELEM_TAG;
}

/// `closest_on_tri_to_point_v3` — point-triangle closest point.
fn closest_on_tri_to_point(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    /* Barycentric sampling (math_geom.cc `closest_on_tri_to_point_v3`). */
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let t = d1 / (d1 - d3);
        return a + ab * t;
    }
    let cp = p - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let t = d2 / (d2 - d6);
        return a + ac * t;
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let t = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return b + (c - b) * t;
    }
    let denom = 1.0 / (va + vb + vc);
    let bv = vb * denom;
    let cw = vc * denom;
    a + ab * bv + ac * cw
}

/// `project_plane_normalized_v3_v3v3`.
fn project_plane_normalized(v: Vec3, plane: Vec3) -> Vec3 {
    v - plane * v.dot(plane)
}

/// `edge_queue_tri_in_sphere`.
fn edge_queue_tri_in_sphere(bm: &BMesh, queue: &EdgeQueue, f: usize) -> bool {
    let v_tri = bm_face_as_array(bm, f);
    let c = closest_on_tri_to_point(
        queue.center,
        bm.vert(v_tri[0]).co,
        bm.vert(v_tri[1]).co,
        bm.vert(v_tri[2]).co,
    );
    (queue.center - c).length_squared() <= queue.radius_squared
}

/// `edge_queue_tri_in_circle` — projected tri vs `center_proj`.
fn edge_queue_tri_in_circle(bm: &BMesh, queue: &EdgeQueue, f: usize) -> bool {
    let view_normal = queue.view_normal.expect("view normal required");
    let v_tri = bm_face_as_array(bm, f);
    let tri_proj = [
        project_plane_normalized(bm.vert(v_tri[0]).co, view_normal),
        project_plane_normalized(bm.vert(v_tri[1]).co, view_normal),
        project_plane_normalized(bm.vert(v_tri[2]).co, view_normal),
    ];
    let c = closest_on_tri_to_point(queue.center_proj, tri_proj[0], tri_proj[1], tri_proj[2]);
    (queue.center_proj - c).length_squared() <= queue.radius_squared
}

fn tri_in_range(bm: &BMesh, queue: &EdgeQueue, f: usize) -> bool {
    if queue.use_projected && queue.view_normal.is_some() {
        edge_queue_tri_in_circle(bm, queue, f)
    } else {
        edge_queue_tri_in_sphere(bm, queue, f)
    }
}

/// `check_mask` — vert mask < 1.0.
fn check_mask(cd: &DyntopoCD, v: usize) -> bool {
    cd.vert_mask_value(v) < 1.0
}

/// `edge_queue_insert` — mask/hide filters + tag + heap push.
fn edge_queue_insert(
    bm: &mut BMesh,
    queue: &mut EdgeQueue,
    cd: &DyntopoCD,
    e: usize,
    priority: f32,
) {
    let ed = bm.edge(e);
    let mask_ok = cd.vert_mask.is_none() || check_mask(cd, ed.v1) || check_mask(cd, ed.v2);
    let hidden = (bm.vert(ed.v1).head.hflag & BM_ELEM_HIDDEN) != 0
        || (bm.vert(ed.v2).head.hflag & BM_ELEM_HIDDEN) != 0;
    if mask_ok && !hidden {
        let (v1, v2) = (ed.v1, ed.v2);
        queue.heap.push(QueuedPair {
            priority,
            v1,
            v2,
        });
        debug_assert!(!edge_queue_test(bm, e));
        edge_queue_enable(bm, e);
    }
}

/// `is_boundary_edge` — seam / non-smooth / non-manifold.
fn is_boundary_edge(bm: &BMesh, e: usize) -> bool {
    let hf = bm.edge(e).head.hflag;
    if (hf & BM_ELEM_SEAM) != 0 {
        return true;
    }
    if (hf & BM_ELEM_SMOOTH) == 0 {
        return true;
    }
    !bm_edge_is_manifold(bm, e)
}

/// `is_boundary_vert` — any disk edge is a boundary edge.
fn is_boundary_vert(bm: &BMesh, v: usize) -> bool {
    let first = bm.vert(v).e;
    if first == BM_NIL {
        return false;
    }
    let mut e = first;
    loop {
        if is_boundary_edge(bm, e) {
            return true;
        }
        e = disk_edge_next(bm, e, v);
        if e == first || e == BM_NIL {
            break;
        }
    }
    false
}

/// `is_edge_adjacent_to_boundary`.
fn is_edge_adjacent_to_boundary(bm: &BMesh, e: usize) -> bool {
    let ed = bm.edge(e);
    is_boundary_vert(bm, ed.v1) || is_boundary_vert(bm, ed.v2)
}

/// `long_edge_queue_priority` — `-len²` (longest first).
fn long_edge_queue_priority(bm: &BMesh, e: usize) -> f32 {
    -bm_edge_calc_length_squared(bm, e)
}

/// `short_edge_queue_priority` — `len²` with boundary multipliers.
fn short_edge_queue_priority(bm: &BMesh, e: usize) -> f32 {
    let mut priority = bm_edge_calc_length_squared(bm, e);
    if is_boundary_edge(bm, e) {
        priority *= 1.5;
    } else if is_edge_adjacent_to_boundary(bm, e) {
        priority *= 1.25;
    }
    priority
}

/// `long_edge_queue_edge_add` — enqueue when longer than limit.
fn long_edge_queue_edge_add(
    bm: &mut BMesh,
    eq_ctx: &mut EdgeQueueContext,
    cd: &DyntopoCD,
    e: usize,
) {
    if !edge_queue_test(bm, e)
        && bm_edge_calc_length_squared(bm, e) > eq_ctx.queue.limit_len_squared
    {
        let priority = long_edge_queue_priority(bm, e);
        edge_queue_insert(bm, &mut eq_ctx.queue, cd, e, priority);
    }
}

/// `long_edge_queue_edge_add_recursive` — walks connected verts
/// into `vertex_pool`.
fn long_edge_queue_edge_add_recursive(
    bm: &mut BMesh,
    eq_ctx: &mut EdgeQueueContext,
    cd: &DyntopoCD,
    e: usize,
    vertex_pool: &mut HashSet<usize>,
) {
    let ed = bm.edge(e);
    for v in [ed.v1, ed.v2] {
        if vertex_pool.insert(v) {
            if cd_vert_present(cd, v) {
                let first = bm.vert(v).e;
                if first != BM_NIL {
                    let mut e2 = first;
                    loop {
                        if !edge_queue_test(bm, e2)
                            && bm_edge_calc_length_squared(bm, e2)
                                > eq_ctx.queue.limit_len_squared
                        {
                            let pr = long_edge_queue_priority(bm, e2);
                            edge_queue_insert(bm, &mut eq_ctx.queue, cd, e2, pr);
                        }
                        e2 = disk_edge_next(bm, e2, v);
                        if e2 == first || e2 == BM_NIL {
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn cd_vert_present(cd: &DyntopoCD, v: usize) -> bool {
    cd.vert_node_id(v) != DYNTOPO_NODE_NONE
}

/// `long_edge_queue_face_add` — enqueue a face's tri edges whose
/// tri intersects the sphere, else recursively into the opposite
/// vertex's disk.
fn long_edge_queue_face_add(
    bm: &mut BMesh,
    eq_ctx: &mut EdgeQueueContext,
    cd: &DyntopoCD,
    f: usize,
    vertex_pool: &mut HashSet<usize>,
) {
    if tri_in_range(bm, &eq_ctx.queue, f) {
        for (l, _v) in face_loops(bm, f) {
            long_edge_queue_edge_add_recursive(bm, eq_ctx, cd, bm.loo(l).e, vertex_pool);
        }
    }
}

/// `short_edge_queue_edge_add`.
fn short_edge_queue_edge_add(
    bm: &mut BMesh,
    eq_ctx: &mut EdgeQueueContext,
    cd: &DyntopoCD,
    e: usize,
) {
    if !edge_queue_test(bm, e) {
        let pr = short_edge_queue_priority(bm, e);
        edge_queue_insert(bm, &mut eq_ctx.queue, cd, e, pr);
    }
}

/// `short_edge_queue_face_add` — tri edges of in-range faces; the
/// C comments about only checking adjacent verts in the same way.
fn short_edge_queue_face_add(
    bm: &mut BMesh,
    eq_ctx: &mut EdgeQueueContext,
    cd: &DyntopoCD,
    f: usize,
) {
    if tri_in_range(bm, &eq_ctx.queue, f) {
        for (l, _v) in face_loops(bm, f) {
            short_edge_queue_edge_add(bm, eq_ctx, cd, bm.loo(l).e);
        }
    }
}

/// `long_edge_queue_create` — fills `queue` for subdivide mode.
#[allow(clippy::too_many_arguments)]
pub fn long_edge_queue_create(
    bm: &mut BMesh,
    eq_ctx: &mut EdgeQueueContext,
    cd: &DyntopoCD,
    max_edge_len: f32,
    nodes: &[BMeshNode],
    center: Vec3,
    view_normal: Option<Vec3>,
    radius: f32,
    use_frontface: bool,
    use_projected: bool,
) {
    debug_assert!(
        view_normal.is_some() || (!use_frontface && !use_projected),
        "view_normal required for frontface/projected modes"
    );
    eq_ctx.queue.center = center;
    eq_ctx.queue.radius_squared = radius * radius;
    eq_ctx.queue.limit_len_squared = max_edge_len * max_edge_len;
    eq_ctx.queue.limit_len = max_edge_len;
    eq_ctx.queue.view_normal = view_normal;
    eq_ctx.queue.use_front_face = use_frontface;
    eq_ctx.queue.use_projected = use_projected;
    if let Some(vn) = view_normal {
        eq_ctx.queue.center_proj = project_plane_normalized(center, vn);
    }

    let mut vertex_pool: HashSet<usize> = HashSet::new();
    for node in nodes {
        if (node.flag_ & NODE_FLAG_LEAF) != 0
            && (node.flag_ & NODE_FLAG_UPDATE_TOPOLOGY) != 0
            && (node.flag_ & NODE_FLAG_FULLY_HIDDEN) == 0
        {
            for &f in &node.bm_faces_ {
                long_edge_queue_face_add(bm, eq_ctx, cd, f, &mut vertex_pool);
            }
        }
    }
}

/// `short_edge_queue_create`.
#[allow(clippy::too_many_arguments)]
pub fn short_edge_queue_create(
    bm: &mut BMesh,
    eq_ctx: &mut EdgeQueueContext,
    cd: &DyntopoCD,
    min_edge_len: f32,
    nodes: &[BMeshNode],
    center: Vec3,
    view_normal: Option<Vec3>,
    radius: f32,
    use_frontface: bool,
    use_projected: bool,
) {
    debug_assert!(
        view_normal.is_some() || (!use_frontface && !use_projected)
    );
    eq_ctx.queue.center = center;
    eq_ctx.queue.radius_squared = radius * radius;
    eq_ctx.queue.limit_len_squared = min_edge_len * min_edge_len;
    eq_ctx.queue.limit_len = min_edge_len;
    eq_ctx.queue.view_normal = view_normal;
    eq_ctx.queue.use_front_face = use_frontface;
    eq_ctx.queue.use_projected = use_projected;
    if let Some(vn) = view_normal {
        eq_ctx.queue.center_proj = project_plane_normalized(center, vn);
    }

    for node in nodes {
        if (node.flag_ & NODE_FLAG_LEAF) != 0
            && (node.flag_ & NODE_FLAG_UPDATE_TOPOLOGY) != 0
            && (node.flag_ & NODE_FLAG_FULLY_HIDDEN) == 0
        {
            for &f in &node.bm_faces_ {
                short_edge_queue_face_add(bm, eq_ctx, cd, f);
            }
        }
    }
}

/* --------------------------- Topology update --------------------------- */

/// `copy_edge_data` — hflag minus TAG + edge CustomData copy.
fn copy_edge_data(bm: &mut BMesh, ctx: &mut PBVHBMeshCtx, dst: usize, src: usize) {
    let hf = bm.edge(src).head.hflag & !BM_ELEM_TAG;
    bm.edge_mut(dst).head.hflag = hf;
    (ctx.edata_copy_block)(bm, src, dst);
}

/// `merge_edge_data` — hflag OR (minus TAG/SMOOTH) + sharp wins +
/// `BM_data_interp_from_edges` at 0.5.
fn merge_edge_data(bm: &mut BMesh, ctx: &mut PBVHBMeshCtx, dst: usize, src: usize) {
    let src_hf = bm.edge(src).head.hflag;
    let d = bm.edge_mut(dst);
    d.head.hflag |= src_hf & !(BM_ELEM_TAG | BM_ELEM_SMOOTH);
    if (src_hf & BM_ELEM_SMOOTH) == 0 {
        d.head.hflag &= !BM_ELEM_SMOOTH;
    }
    (ctx.data_interp_from_edges)(bm, src, dst, dst, 0.5);
}

/// `pbvh_bmesh_split_edge` — midpoint split of a long edge: new
/// vert + 2 tris per adjacent face, ownership fixup, delete old.
#[allow(clippy::too_many_arguments)]
fn pbvh_bmesh_split_edge(
    eq_ctx: &mut EdgeQueueContext,
    bm: &mut BMesh,
    nodes: &mut Vec<BMeshNode>,
    node_changed: &mut [bool],
    cd: &mut DyntopoCD,
    log: &mut dyn BMLog,
    ctx: &mut PBVHBMeshCtx,
    e: usize,
) {
    let edge_loops = pbvh_bmesh_edge_loops(bm, e);
    let ed = bm.edge(e);
    let midpoint_co = midpoint(bm.vert(ed.v1).co, bm.vert(ed.v2).co);
    let midpoint_no = midpoint(bm.vert(ed.v1).no, bm.vert(ed.v2).no).normalized();

    let node_index = cd.vert_node_id(ed.v1) as usize;
    let v_new = pbvh_bmesh_vert_create(
        bm,
        nodes,
        node_changed,
        log,
        ed.v1,
        ed.v2,
        node_index,
        midpoint_co,
        midpoint_no,
        cd,
        ctx.data_interp_from_verts,
    );

    for (i, &l_adj) in edge_loops.iter().enumerate() {
        let f_adj = bm.loo(l_adj).f;
        debug_assert_eq!(bm.face(f_adj).len, 3);
        let ni = cd.face_node_id(f_adj) as usize;

        let l = bm.loo(l_adj);
        let v_opp = bm.loo(l.prev).v;
        let v1 = l.v;
        let v2 = bm.loo(l.next).v;

        if ni != node_index && i == 0 {
            vert_ownership_transfer(nodes, node_changed, cd, ni, v_new);
        }

        /* First face (v1, v_new, v_opp). */
        let first_tri = [v1, v_new, v_opp];
        let first_edges = bm_edges_from_tri(bm, &first_tri);
        copy_edge_data(bm, ctx, first_edges[0], e);
        let f_new_first = pbvh_bmesh_face_create(
            bm, nodes, node_changed, cd, log, ni, &first_tri, &first_edges, f_adj,
        );
        let mut pool = HashSet::new();
        long_edge_queue_face_add(bm, eq_ctx, cd, f_new_first, &mut pool);

        /* Second face (v_new, v2, v_opp). */
        let second_tri = [v_new, v2, v_opp];
        let second_edges = [
            bm_edge_create(bm, second_tri[0], second_tri[1], None, BM_CREATE_NO_DOUBLE),
            bm_edge_create(bm, second_tri[1], second_tri[2], None, BM_CREATE_NO_DOUBLE),
            first_edges[1],
        ];
        copy_edge_data(bm, ctx, second_edges[0], e);
        let f_new_second = pbvh_bmesh_face_create(
            bm, nodes, node_changed, cd, log, ni, &second_tri, &second_edges, f_adj,
        );
        let mut pool = HashSet::new();
        long_edge_queue_face_add(bm, eq_ctx, cd, f_new_second, &mut pool);

        /* Delete original. */
        pbvh_bmesh_face_remove(bm, nodes, node_changed, cd, log, f_adj);
        bm_face_kill(bm, f_adj);

        if !nodes[ni].bm_unique_verts_.contains(&v_new) {
            nodes[ni].bm_other_verts_.insert(v_new);
        }

        if bm_vert_edge_count(bm, v_opp) > 8 {
            let first = bm.vert(v_opp).e;
            if first != BM_NIL {
                let mut e2 = first;
                loop {
                    long_edge_queue_edge_add(bm, eq_ctx, cd, e2);
                    e2 = disk_edge_next(bm, e2, v_opp);
                    if e2 == first || e2 == BM_NIL {
                        break;
                    }
                }
            }
        }
    }

    bm_edge_kill(bm, e);
}

/// `pbvh_bmesh_subdivide_long_edges`.
pub fn pbvh_bmesh_subdivide_long_edges(
    eq_ctx: &mut EdgeQueueContext,
    bm: &mut BMesh,
    nodes: &mut Vec<BMeshNode>,
    node_changed: &mut [bool],
    cd: &mut DyntopoCD,
    log: &mut dyn BMLog,
    ctx: &mut PBVHBMeshCtx,
) -> bool {
    let mut any_subdivided = false;

    while let Some(pair) = eq_ctx.queue.heap.pop() {
        let (v1, v2) = (pair.v1, pair.v2);
        let e = bm_edge_exists(bm, v1, v2);
        if e == BM_NIL {
            continue;
        }
        edge_queue_disable(bm, e);

        if (cd.vert_node_id(bm.edge(e).v1) == DYNTOPO_NODE_NONE)
            || (cd.vert_node_id(bm.edge(e).v2) == DYNTOPO_NODE_NONE)
        {
            continue;
        }

        any_subdivided = true;
        pbvh_bmesh_split_edge(eq_ctx, bm, nodes, node_changed, cd, log, ctx, e);
    }

    any_subdivided
}

/// `vert_in_face_adjacent_to_edge`.
fn vert_in_face_adjacent_to_edge(bm: &BMesh, vert: usize, edge: usize) -> bool {
    let ed = bm.edge(edge);
    if ed.l == BM_NIL {
        return false;
    }
    let first = ed.l;
    let mut l = first;
    loop {
        let f = bm.loo(l).f;
        if bm_vert_in_face(bm, vert, f) {
            return true;
        }
        l = bm.loo(l).radial_next;
        if l == first || l == BM_NIL {
            break;
        }
    }
    false
}

/// `merge_flap_edge_data` — merge the flap face's and del-face's
/// edge attributes into `(v1, v_conn)` before collapse.
#[allow(clippy::too_many_arguments)]
fn merge_flap_edge_data(
    bm: &mut BMesh,
    ctx: &mut PBVHBMeshCtx,
    e: usize,
    v_del: usize,
    l_del: usize,
    v_conn: usize,
) {
    let ld = bm.loo(l_del);
    let edge_v1_v2 = bm_edge_exists(bm, bm.loo(ld.next).v, bm.loo(ld.prev).v);
    if edge_v1_v2 == BM_NIL {
        return;
    }

    let ev = bm.edge(edge_v1_v2);
    let v2 = if vert_in_face_adjacent_to_edge(bm, ev.v1, e) {
        ev.v1
    } else {
        ev.v2
    };
    let v1 = bm_edge_other_vert(bm, edge_v1_v2, v2);

    let dst_edge = bm_edge_exists(bm, v1, v_conn);
    if dst_edge == BM_NIL {
        return;
    }

    let source_edges = [
        edge_v1_v2,
        bm_edge_exists(bm, v2, v_conn),
        bm_edge_exists(bm, v1, v_del),
        bm_edge_exists(bm, v2, v_del),
    ];
    for src_edge in source_edges {
        if src_edge == BM_NIL {
            continue;
        }
        merge_edge_data(bm, ctx, dst_edge, src_edge);
    }
}

/// `find_outer_flap_vert` — the face's vert that becomes loose.
fn find_outer_flap_vert(bm: &BMesh, face: usize) -> usize {
    let mut flap_vert = BM_NIL;
    for (_, v) in face_loops(bm, face) {
        if bm_vert_face_count_at_most(bm, v, 2) == 1 {
            if flap_vert != BM_NIL {
                return BM_NIL;
            }
            flap_vert = v;
        }
    }
    flap_vert
}

/// `try_merge_flap_edge_data_before_dissolve` — flap edge attrs to
/// the surviving base edge.
fn try_merge_flap_edge_data_before_dissolve(
    bm: &mut BMesh,
    ctx: &mut PBVHBMeshCtx,
    face: usize,
) {
    debug_assert_eq!(bm.face(face).len, 3);
    let flap_vert = find_outer_flap_vert(bm, face);
    if flap_vert == BM_NIL {
        return;
    }
    /* `l->v` is the flap vertex; the other two are the base edge. */
    for (l, v) in face_loops(bm, face) {
        if v == flap_vert {
            let lp = bm.loo(l);
            let dst_edge = bm_edge_exists(bm, bm.loo(lp.next).v, bm.loo(lp.prev).v);
            if dst_edge != BM_NIL {
                let lp_e = bm.loo(l).e;
                merge_edge_data(bm, ctx, dst_edge, lp_e);
            }
        }
    }
}

/// `merge_face_edge_data` — merge removed edge attrs into the new
/// face's edges after collapse.
fn merge_face_edge_data(
    bm: &mut BMesh,
    ctx: &mut PBVHBMeshCtx,
    new_face: usize,
    v_del: usize,
    v_conn: usize,
) {
    let edge_v1_v2 = bm_edge_exists(
        bm,
        bm_edge_other_vert_of_loop(bm, new_face, v_conn, true),
        bm_edge_other_vert_of_loop(bm, new_face, v_conn, false),
    );
    if edge_v1_v2 == BM_NIL {
        return;
    }
    for (l, _v) in face_loops(bm, new_face) {
        let dst_edge = bm.loo(l).e;
        if dst_edge == edge_v1_v2 {
            continue;
        }
        debug_assert!(bm_vert_in_edge(bm, dst_edge, v_conn));
        let v_other = bm_edge_other_vert(bm, dst_edge, v_conn);
        let src_edge = bm_edge_exists(bm, v_del, v_other);
        if src_edge != BM_NIL {
            merge_edge_data(bm, ctx, dst_edge, src_edge);
        }
    }
}

/// The two non-`v_conn` verts of a tri face, ordered so `next`/`prev`
/// pick opposite corners.
fn bm_edge_other_vert_of_loop(bm: &BMesh, f: usize, v_conn: usize, first: bool) -> usize {
    let mut others = Vec::new();
    for (_, v) in face_loops(bm, f) {
        if v != v_conn {
            others.push(v);
        }
    }
    if others.is_empty() {
        return BM_NIL;
    }
    others[if first { 0 } else { others.len() - 1 }]
}

/// `pbvh_bmesh_collapse_edge` — the full collapse of `(v1,v2)`:
/// boundary/mask winner pick, flap merges, face rebuild, kills.
#[allow(clippy::too_many_arguments)]
fn pbvh_bmesh_collapse_edge(
    bm: &mut BMesh,
    nodes: &mut Vec<BMeshNode>,
    node_changed: &mut [bool],
    cd: &mut DyntopoCD,
    log: &mut dyn BMLog,
    ctx: &mut PBVHBMeshCtx,
    e: usize,
    v1: usize,
    v2: usize,
    deleted_verts: &mut HashMap<usize, usize>,
) {
    let v1_on_boundary = is_boundary_vert(bm, v1);
    let v2_on_boundary = is_boundary_vert(bm, v2);

    let (v_del, mut v_conn) = if v1_on_boundary || v2_on_boundary {
        if v1_on_boundary {
            (v2, v1)
        } else {
            (v1, v2)
        }
    } else if cd.vert_mask_value(v1) < cd.vert_mask_value(v2) {
        (v1, v2)
    } else {
        (v2, v1)
    };

    pbvh_bmesh_vert_remove(bm, nodes, node_changed, cd, v_del);

    let mut deleted_faces: Vec<usize> = Vec::new();

    for (l, f_del) in loops_of_vert(bm, v_del) {
        if bm_edge_in_face(bm, e, f_del) {
            continue;
        }
        deleted_faces.push(f_del);
        let l_next = bm.loo(l).next;
        let existing = bm_face_exists_tri_from_loop_vert(bm, l_next, v_conn);
        if existing != BM_NIL {
            merge_flap_edge_data(bm, ctx, e, v_del, l, v_conn);
            deleted_faces.push(existing);
        }
    }

    /* Remove all faces adjacent to the edge. */
    loop {
        let l_adj = bm.edge(e).l;
        if l_adj == BM_NIL {
            break;
        }
        let f_adj = bm.loo(l_adj).f;
        pbvh_bmesh_face_remove(bm, nodes, node_changed, cd, log, f_adj);
        bm_face_kill(bm, f_adj);
    }

    bm_edge_kill(bm, e);

    for (l, f) in loops_of_vert(bm, v_del) {
        let l_next = bm.loo(l).next;
        if bm_face_exists_tri_from_loop_vert(bm, l_next, v_conn) == BM_NIL {
            let lp = bm.loo(l);
            let v_tri = [v_conn, bm.loo(lp.next).v, bm.loo(lp.prev).v];
            let ni = node_index_from_face(cd, f) as usize;
            let e_tri = bm_edges_from_tri(bm, &v_tri);
            let new_face = pbvh_bmesh_face_create(
                bm, nodes, node_changed, cd, log, ni, &v_tri, &e_tri, f,
            );
            merge_face_edge_data(bm, ctx, new_face, v_del, v_conn);
            if !nodes[ni].bm_unique_verts_.contains(&v_conn) {
                nodes[ni].bm_other_verts_.insert(v_conn);
            }
        }
    }

    /* Delete the tagged faces. */
    for f_del in deleted_faces {
        debug_assert_eq!(bm.face(f_del).len, 3);
        let loops = face_loops(bm, f_del);
        let v_tri = [loops[0].1, loops[1].1, loops[2].1];
        let e_tri = [
            bm.loo(loops[0].0).e,
            bm.loo(loops[1].0).e,
            bm.loo(loops[2].0).e,
        ];

        try_merge_flap_edge_data_before_dissolve(bm, ctx, f_del);

        pbvh_bmesh_face_remove(bm, nodes, node_changed, cd, log, f_del);
        bm_face_kill(bm, f_del);

        for &e in &e_tri {
            if bm.edge(e).l == BM_NIL {
                bm_edge_kill(bm, e);
            }
        }
        for j in 0..3 {
            let v = v_tri[j];
            if v != v_del && bm.vert(v).e == BM_NIL {
                pbvh_bmesh_vert_remove(bm, nodes, node_changed, cd, v);
                log.vert_removed(v);
                if v == v_conn {
                    v_conn = BM_NIL;
                }
                deleted_verts.insert(v, BM_NIL);
                bm_vert_kill(bm, v);
            }
        }
    }

    /* Move `v_conn` to the midpoint (unless on boundary). */
    if v_conn != BM_NIL && !is_boundary_vert(bm, v_conn) {
        log.vert_before_modified(v_conn);
        let new_co = midpoint(bm.vert(v_conn).co, bm.vert(v_del).co);
        let new_no = (bm.vert(v_conn).no + bm.vert(v_del).no).normalized();
        let vc = bm.vert_mut(v_conn);
        vc.co = new_co;
        vc.no = new_no;
    }

    if v_conn != BM_NIL {
        for (_, f) in loops_of_vert(bm, v_conn) {
            let f_node = node_index_from_face(cd, f);
            if f_node >= 0 {
                node_changed[f_node as usize] = true;
            }
        }
    }

    log.vert_removed(v_del);
    deleted_verts.insert(v_del, v_conn);
    bm_vert_kill(bm, v_del);
}

/// `pbvh_bmesh_collapse_short_edges`.
pub fn pbvh_bmesh_collapse_short_edges(
    eq_ctx: &mut EdgeQueueContext,
    min_edge_len: f32,
    bm: &mut BMesh,
    nodes: &mut Vec<BMeshNode>,
    node_changed: &mut [bool],
    cd: &mut DyntopoCD,
    log: &mut dyn BMLog,
    ctx: &mut PBVHBMeshCtx,
) -> bool {
    let min_len_squared = min_edge_len * min_edge_len;
    let mut any_collapsed = false;
    let mut deleted_verts: HashMap<usize, usize> = HashMap::new();

    while let Some(pair) = eq_ctx.queue.heap.pop() {
        let v1 = bm_vert_hash_lookup_chain(&deleted_verts, pair.v1);
        let v2 = bm_vert_hash_lookup_chain(&deleted_verts, pair.v2);
        if v1 == BM_NIL || v2 == BM_NIL || v1 == v2 {
            continue;
        }
        let e = bm_edge_exists(bm, v1, v2);
        if e == BM_NIL {
            continue;
        }
        edge_queue_disable(bm, e);

        if (bm.vert(v1).co - bm.vert(v2).co).length_squared() >= min_len_squared {
            continue;
        }

        if (cd.vert_node_id(bm.edge(e).v1) == DYNTOPO_NODE_NONE)
            || (cd.vert_node_id(bm.edge(e).v2) == DYNTOPO_NODE_NONE)
        {
            continue;
        }

        any_collapsed = true;
        pbvh_bmesh_collapse_edge(
            bm,
            nodes,
            node_changed,
            cd,
            log,
            ctx,
            e,
            v1,
            v2,
            &mut deleted_verts,
        );
    }

    any_collapsed
}

/* ------------------------------ Public API ------------------------------ */

/// `bmesh_update_topology` — the collapse+subdivide driver called
/// from `pbvh.cc`'s update_topology path.
#[allow(clippy::too_many_arguments)]
pub fn bmesh_update_topology(
    bm: &mut BMesh,
    nodes: &mut Vec<BMeshNode>,
    cd: &mut DyntopoCD,
    log: &mut dyn BMLog,
    ctx: &mut PBVHBMeshCtx,
    mode: i32,
    min_edge_len: f32,
    max_edge_len: f32,
    center: Vec3,
    view_normal: Option<Vec3>,
    radius: f32,
    use_frontface: bool,
    use_projected: bool,
) -> bool {
    if let Some(vn) = view_normal {
        debug_assert!(vn.length_squared() > 0.0);
    }
    let mut modified = false;
    let mut node_changed = vec![false; nodes.len()];

    if (mode & PBVH_COLLAPSE) != 0 {
        let mut eq_ctx = EdgeQueueContext {
            queue: EdgeQueue {
                heap: BinaryHeap::new(),
                center,
                center_proj: Vec3::default(),
                radius_squared: 0.0,
                limit_len_squared: 0.0,
                limit_len: 0.0,
                view_normal,
                use_front_face: use_frontface,
                use_projected,
            },
        };
        short_edge_queue_create(
            bm,
            &mut eq_ctx,
            cd,
            min_edge_len,
            nodes,
            center,
            view_normal,
            radius,
            use_frontface,
            use_projected,
        );
        modified |= pbvh_bmesh_collapse_short_edges(
            &mut eq_ctx,
            min_edge_len,
            bm,
            nodes,
            &mut node_changed,
            cd,
            log,
            ctx,
        );
    }

    if (mode & PBVH_SUBDIVIDE) != 0 {
        let mut eq_ctx = EdgeQueueContext {
            queue: EdgeQueue {
                heap: BinaryHeap::new(),
                center,
                center_proj: Vec3::default(),
                radius_squared: 0.0,
                limit_len_squared: 0.0,
                limit_len: 0.0,
                view_normal,
                use_front_face: use_frontface,
                use_projected,
            },
        };
        long_edge_queue_create(
            bm,
            &mut eq_ctx,
            cd,
            max_edge_len,
            nodes,
            center,
            view_normal,
            radius,
            use_frontface,
            use_projected,
        );
        modified |= pbvh_bmesh_subdivide_long_edges(
            &mut eq_ctx, bm, nodes, &mut node_changed, cd, log, ctx,
        );
    }

    /* Unmark nodes / drop+resave orig on topology-updated leaves. */
    for (i, node) in nodes.iter_mut().enumerate() {
        if (node.flag_ & NODE_FLAG_LEAF) != 0 && (node.flag_ & NODE_FLAG_UPDATE_TOPOLOGY) != 0 {
            node.flag_ &= !NODE_FLAG_UPDATE_TOPOLOGY;
        }
        if (node.flag_ & NODE_FLAG_LEAF) != 0
            && (node.flag_ & NODE_FLAG_TOPOLOGY_UPDATED) != 0
            && node_changed[i]
        {
            node.flag_ &= !NODE_FLAG_TOPOLOGY_UPDATED;
            if !node.orig_tris_.is_empty() {
                pbvh_bmesh_node_drop_orig(node);
                /* `BKE_pbvh_bmesh_node_save_orig(.., true)` —
                 * PORT_PENDING (BMLog original-save). */
            }
        }
    }

    modified
}

/// `Tree::from_bmesh` — fast builder: per-face bounds, recursive
/// split into `FastNodeBuildInfo`, then `create_nodes_fast`
/// population.
pub fn bmesh_node_from_bmesh(bm: &mut BMesh, cd: &mut DyntopoCD) -> Vec<BMeshNode> {
    if bm.totface == 0 {
        return Vec::new();
    }

    /* Face bounds + element ordering. */
    let tot = bm.totface as usize;
    let mut face_bounds = vec![negative_bounds(); tot];
    let mut nodeinfo: Vec<usize> = vec![BM_NIL; tot];
    for (i, f) in bm.fpool.iter().enumerate().take(tot) {
        let _ = f;
        /* Faces are indexed by pool order; the C file sets the elem
         * index — the pool index is already our index. */
        let mut mn = Vec3::splat(f32::MAX);
        let mut mx = Vec3::splat(-f32::MAX);
        for (_, v) in face_loops(bm, i) {
            min_max(bm.vert(v).co, &mut mn, &mut mx);
        }
        face_bounds[i] = (mn, mx);
        nodeinfo[i] = i;
        cd.face_node.insert(i, DYNTOPO_NODE_NONE);
    }
    bm.elem_index_dirty = true;
    for v in 0..bm.vpool.len() {
        cd.vert_node.insert(v, DYNTOPO_NODE_NONE);
    }

    /* `FastNodeBuildInfo` tree → flat `Vec<BMeshNode>` via the same
     * `pbvh_bmesh_node_split` partitioning the slow path uses. */
    let mut nodes = vec![BMeshNode::default()];
    let mut node_changed = vec![false];
    let face_index_of = |f: usize| f;
    pbvh_bmesh_node_split(
        bm,
        &mut nodes,
        &mut node_changed,
        cd,
        &face_bounds,
        &face_index_of,
        0,
    );

    /* `store_bounds_orig` + FullyHidden pass. */
    for node in &mut nodes {
        node.bounds_orig_min = node.bounds_min;
        node.bounds_orig_max = node.bounds_max;
    }
    for i in 0..nodes.len() {
        let all_hidden = !nodes[i].bm_faces_.is_empty()
            && nodes[i]
                .bm_faces_
                .iter()
                .all(|&f| (bm.face(f).head.hflag & BM_ELEM_HIDDEN) != 0);
        if all_hidden {
            nodes[i].flag_ |= NODE_FLAG_FULLY_HIDDEN;
        }
    }
    nodes
}

/// `IsectRayPrecalc` mirror — ray/plane precomputed factors.
#[derive(Clone, Copy, Default)]
pub struct IsectRayPrecalc {
    /// `ray_direction`.
    pub ray_direction: Vec3,
    /// `x20`/`x21`/`x22` — 2D-projection factors.
    pub x20: f32,
    /// `x21`.
    pub x21: f32,
    /// `x22`.
    pub x22: f32,
    /// `z0`/`z1`/`z2`.
    pub z0: f32,
    /// `z1`.
    pub z1: f32,
    /// `z2`.
    pub z2: f32,
}

/// `ray_face_intersection_tri` — Moller–Trumbore via the C's
/// `isect_ray_tri` (exact 2D-projection variant used by
/// `raycast_node_*`). `PORT_PENDING` full `IsectRayPrecalc`
/// setup — this uses the plain origin/dir form.
fn ray_face_intersection_tri(
    ray_start: Vec3,
    ray_dir: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    depth: &mut f32,
) -> bool {
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let p = ray_dir.cross(e2);
    let det = p.dot(e1);
    if det.abs() < 1e-12 {
        return false;
    }
    let inv = 1.0 / det;
    let t = ray_start - v0;
    let u = p.dot(t) * inv;
    if !(0.0..=1.0).contains(&u) {
        return false;
    }
    let q = t.cross(e1);
    let v = q.dot(ray_dir) * inv;
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    let d = q.dot(e2) * inv;
    if d < 0.0 {
        return false;
    }
    *depth = d;
    true
}

/// `node_raycast_bmesh`.
#[allow(clippy::too_many_arguments)]
pub fn node_raycast_bmesh(
    bm: &BMesh,
    node: &BMeshNode,
    ray_start: Vec3,
    ray_normal: Vec3,
    depth: &mut f32,
    use_original: bool,
    r_active_vertex: &mut usize,
    r_face_normal: &mut Vec3,
) -> bool {
    let mut hit = false;
    let mut nearest_vertex_co = Vec3::default();
    let use_original = use_original && !node.orig_tris_.is_empty();

    if use_original {
        for tri in &node.orig_tris_ {
            let positions = [
                node.orig_positions_[tri[0] as usize],
                node.orig_positions_[tri[1] as usize],
                node.orig_positions_[tri[2] as usize],
            ];
            if ray_face_intersection_tri(
                ray_start, ray_normal, positions[0], positions[1], positions[2], depth,
            ) {
                hit = true;
                *r_face_normal = (positions[1] - positions[0])
                    .cross(positions[2] - positions[1])
                    .normalized();
                if *r_active_vertex != BM_NIL {
                    let location = ray_start + ray_normal * *depth;
                    for (i, p) in positions.iter().enumerate() {
                        if i == 0
                            || (location - *p).length_squared()
                                < (location - nearest_vertex_co).length_squared()
                        {
                            nearest_vertex_co = *p;
                            *r_active_vertex = node.orig_verts_[tri[i] as usize];
                        }
                    }
                }
            }
        }
    } else {
        for &f in &node.bm_faces_ {
            debug_assert_eq!(bm.face(f).len, 3);
            if (bm.face(f).head.hflag & BM_ELEM_HIDDEN) == 0 {
                let v_tri = bm_face_as_array(bm, f);
                let positions = [
                    bm.vert(v_tri[0]).co,
                    bm.vert(v_tri[1]).co,
                    bm.vert(v_tri[2]).co,
                ];
                if ray_face_intersection_tri(
                    ray_start, ray_normal, positions[0], positions[1], positions[2], depth,
                ) {
                    hit = true;
                    *r_face_normal = (positions[1] - positions[0])
                        .cross(positions[2] - positions[1])
                        .normalized();
                    if *r_active_vertex != BM_NIL {
                        let location = ray_start + ray_normal * *depth;
                        for (i, p) in positions.iter().enumerate() {
                            if i == 0
                                || (location - *p).length_squared()
                                    < (location - nearest_vertex_co).length_squared()
                            {
                                nearest_vertex_co = *p;
                                *r_active_vertex = v_tri[i];
                            }
                        }
                    }
                }
            }
        }
    }
    hit
}

/// `raycast_node_detail_bmesh` — returns the hit face's max edge
/// length.
pub fn raycast_node_detail_bmesh(
    bm: &BMesh,
    node: &BMeshNode,
    ray_start: Vec3,
    ray_normal: Vec3,
    depth: &mut f32,
    r_edge_length: &mut f32,
) -> bool {
    if (node.flag_ & NODE_FLAG_FULLY_HIDDEN) != 0 {
        return false;
    }
    let mut hit = false;
    let mut f_hit = BM_NIL;
    for &f in &node.bm_faces_ {
        debug_assert_eq!(bm.face(f).len, 3);
        if (bm.face(f).head.hflag & BM_ELEM_HIDDEN) == 0 {
            let v_tri = bm_face_as_array(bm, f);
            if ray_face_intersection_tri(
                ray_start,
                ray_normal,
                bm.vert(v_tri[0]).co,
                bm.vert(v_tri[1]).co,
                bm.vert(v_tri[2]).co,
                depth,
            ) {
                f_hit = f;
                hit = true;
            }
        }
    }
    if hit {
        let v_tri = bm_face_as_array(bm, f_hit);
        let len1 = (bm.vert(v_tri[0]).co - bm.vert(v_tri[1]).co).length_squared();
        let len2 = (bm.vert(v_tri[1]).co - bm.vert(v_tri[2]).co).length_squared();
        let len3 = (bm.vert(v_tri[2]).co - bm.vert(v_tri[0]).co).length_squared();
        *r_edge_length = len1.max(len2).max(len3).sqrt();
    }
    hit
}

/// `ray_face_nearest_tri` — closest point on ray to triangle.
fn ray_face_nearest_tri(
    ray_start: Vec3,
    ray_normal: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    r_depth: &mut f32,
    dist_sq: &mut f32,
) -> bool {
    /* `ray_face_nearest_tri` — closest point of tri to ray. */
    let c = closest_on_tri_to_point(ray_start, v0, v1, v2);
    /* Project (c - ray_start) onto ray direction for depth. */
    let to_c = c - ray_start;
    let depth = to_c.dot(ray_normal);
    if depth < 0.0 {
        return false;
    }
    let nearest = ray_start + ray_normal * depth;
    let c2 = closest_on_tri_to_point(nearest, v0, v1, v2);
    let d = (c2 - nearest).length_squared();
    if d < *dist_sq {
        *r_depth = depth;
        *dist_sq = d;
        return true;
    }
    false
}

/// `bmesh_node_nearest_to_ray`.
#[allow(clippy::too_many_arguments)]
pub fn bmesh_node_nearest_to_ray(
    bm: &BMesh,
    node: &BMeshNode,
    ray_start: Vec3,
    ray_normal: Vec3,
    r_depth: &mut f32,
    dist_sq: &mut f32,
    use_original: bool,
) -> bool {
    let mut hit = false;
    if use_original && !node.orig_tris_.is_empty() {
        for t in &node.orig_tris_ {
            hit |= ray_face_nearest_tri(
                ray_start,
                ray_normal,
                node.orig_positions_[t[0] as usize],
                node.orig_positions_[t[1] as usize],
                node.orig_positions_[t[2] as usize],
                r_depth,
                dist_sq,
            );
        }
    } else {
        for &f in &node.bm_faces_ {
            if (bm.face(f).head.hflag & BM_ELEM_HIDDEN) == 0 {
                let v_tri = bm_face_as_array(bm, f);
                hit |= ray_face_nearest_tri(
                    ray_start,
                    ray_normal,
                    bm.vert(v_tri[0]).co,
                    bm.vert(v_tri[1]).co,
                    bm.vert(v_tri[2]).co,
                    r_depth,
                    dist_sq,
                );
            }
        }
    }
    hit
}

/// `BM_face_normal_update` — Newell normal from the face's loop
/// positions (matches `bmesh_mesh_normals.cc` for tris).
fn face_normal_from_loops(bm: &BMesh, f: usize) -> Vec3 {
    let mut n = Vec3::default();
    let loops = face_loops(bm, f);
    for i in 0..loops.len() {
        let curr = bm.vert(loops[i].1).co;
        let next = bm.vert(loops[(i + 1) % loops.len()].1).co;
        n.x += (curr.y - next.y) * (curr.z + next.z);
        n.y += (curr.z - next.z) * (curr.x + next.x);
        n.z += (curr.x - next.x) * (curr.y + next.y);
    }
    n.normalized()
}

/// `bmesh_normals_update` — recompute face + vert normals of the
/// given node indices.
pub fn bmesh_normals_update(bm: &mut BMesh, nodes: &[BMeshNode], nodes_to_update: &[usize]) {
    for &i in nodes_to_update {
        let node = &nodes[i];
        for &f in &node.bm_faces_ {
            bm.face_mut(f).no = face_normal_from_loops(bm, f);
        }
        for &v in &node.bm_unique_verts_ {
            bm_vert_normal_update(bm, v);
        }
        for &v in &node.bm_other_verts_ {
            bm_vert_normal_update(bm, v);
        }
    }
}

/// `BKE_pbvh_bmesh_node_save_orig` — copy current positions into
/// `orig_*` arrays (the `use_original` BMLog lookup variant is
/// `PORT_PENDING` — this is the log-free fallback verbatim).
pub fn pbvh_bmesh_node_save_orig(
    bm: &BMesh,
    log: &mut dyn BMLog,
    node: &mut BMeshNode,
    use_original: bool,
) {
    let verts: Vec<usize> = node
        .bm_unique_verts_
        .iter()
        .chain(node.bm_other_verts_.iter())
        .copied()
        .collect();
    let mut vert_index: HashMap<usize, i32> = HashMap::new();
    node.orig_positions_ = Vec::with_capacity(verts.len());
    node.orig_verts_ = verts.clone();
    for (i, &v) in verts.iter().enumerate() {
        vert_index.insert(v, i as i32);
        let co = if use_original {
            let _ = bm;
            log.vert_original_co(v).unwrap_or_else(|| Vec3::default())
        } else {
            bm.vert(v).co
        };
        node.orig_positions_.push(co);
    }
    node.orig_tris_ = node
        .bm_faces_
        .iter()
        .map(|&f| {
            let vt = bm_face_as_array(bm, f);
            [vert_index[&vt[0]], vert_index[&vt[1]], vert_index[&vt[2]]]
        })
        .collect();
}
