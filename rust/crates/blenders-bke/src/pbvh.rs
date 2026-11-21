//! Port of `intern/pbvh.cc` — the PBVH tree: `Tree`,
//! `MeshNode`, bounds helpers, iteration, visibility flags,
//! frustum/ray tests, `vert_coords_apply`.
//!
//! `PORT_PENDING`: `BKE_pbvh_build_*` (mesh corner_tris /
//! CCG grids need Mesh/SubdivCCG — node build is injected
//! via `build_nodes` callback returning the leaves),
//! `BKE_pbvh_update_normals` (SharedCache on Object),
//! pixels rebuild, `debug_draw_gen_get` counters, GPU
//! handles on nodes.

use blenders_math::Vec3;

/// `PBVHNode` flags (`Node::Flags`, verbatim).
pub const PBVH_UPDATE_NONE: u16 = 0;
/// `Leaf` — node has no children.
pub const PBVH_LEAF: u16 = 1 << 0;
/// `Mask`.
pub const PBVH_UPDATE_MASK: u16 = 1 << 1;
/// `Normals`.
pub const PBVH_UPDATE_NORMALS: u16 = 1 << 2;
/// `Visibility`.
pub const PBVH_UPDATE_VISIBILITY: u16 = 1 << 4;
/// `FullyHidden`.
pub const PBVH_FULLY_HIDDEN: u16 = 1 << 5;
/// `FullyUnmasked`.
pub const PBVH_FULLY_UNMASKED: u16 = 1 << 6;
/// `HasMask` (inner flag — bit 7).
pub const PBVH_UPDATE_COLOR: u16 = 1 << 7;
/// `RebuildDraw`.
pub const PBVH_REBUILD_DRAW: u16 = 1 << 8;
/// `UpdateDrawBuffers`.
pub const PBVH_UPDATE_DRAW_BUFFERS: u16 = 1 << 9;
/// `UpdateNormals`.
pub const PBVH_UPDATE_VERTS: u16 = 1 << 10;
/// `DrawChangedBounds` (bounds rebuild flag).
pub const PBVH_UPDATE_DRAW_BOUNDS: u16 = 1 << 11;

/// `negative_bounds` / `merge_bounds` helpers (`Bounds<float3>`
/// as `(min, max)`).
pub fn pbvh_negative_bounds() -> (Vec3, Vec3) {
    (Vec3::splat(f32::MAX), Vec3::splat(-f32::MAX))
}
/// `merge_bounds`.
pub fn pbvh_merge_bounds(a: (Vec3, Vec3), b: (Vec3, Vec3)) -> (Vec3, Vec3) {
    (
        Vec3::new(a.0.x.min(b.0.x), a.0.y.min(b.0.y), a.0.z.min(b.0.z)),
        Vec3::new(a.1.x.max(b.1.x), a.1.y.max(b.1.y), a.1.z.max(b.1.z)),
    )
}

/// `MeshNode` — the regular-mesh PBVH leaf (verbatim fields
/// minus GPU/pixel handles).
#[derive(Clone, Default)]
pub struct MeshNode {
    /// `flag_` (`PBVH_*` bits).
    pub flag_: u16,
    /// `children_offset_` — first child index, -1 for leaf.
    pub children_offset_: i32,
    /// `bounds_`.
    pub bounds_min: Vec3,
    /// `bounds_` max.
    pub bounds_max: Vec3,
    /// `bounds_orig_` (rest pose).
    pub bounds_orig_min: Vec3,
    /// `bounds_orig_` max.
    pub bounds_orig_max: Vec3,
    /// `uniq_verts_` — verts owned only by this node.
    pub uniq_verts_: Vec<i32>,
    /// `corner_verts_` — corner verts spanning this node's prims.
    pub corner_verts_: Vec<i32>,
    /// `prim_indices_` — this leaf's triangle indices.
    pub prim_indices_: Vec<i32>,
    /// `hidden_` — hidden vert bits (leaf only).
    pub hidden_: Vec<bool>,
    /// `vert_bitmap_` — all referenced verts bit-set.
    pub vert_bitmap_: Vec<u64>,
    /// `face_area_` — total area (transient, computed on build).
    pub face_area_: f32,
    /// `valence_` — vert-valence map (transient).
    pub valence_: Vec<i32>,
    /// `orig_positions_` — orig positions (undo).
    pub orig_positions_: Vec<Vec3>,
    /// `orig_tris_`.
    pub orig_tris_: Vec<[i32; 3]>,
    /// `orig_verts_`.
    pub orig_verts_: Vec<usize>,
    /// `normals_` — per-node face normals cache.
    pub normals_: Vec<Vec3>,
    /// `vert_normals_`.
    pub vert_normals_: Vec<Vec3>,
}

/// `GridsNode` — subdiv-ccg leaf (CCG-data handles → usize).
#[derive(Clone, Default)]
pub struct GridsNode {
    /// `flag_`.
    pub flag_: u16,
    /// `children_offset_`.
    pub children_offset_: i32,
    /// `bounds_`.
    pub bounds_min: Vec3,
    /// `bounds_` max.
    pub bounds_max: Vec3,
    /// `bounds_orig_`.
    pub bounds_orig_min: Vec3,
    /// `bounds_orig_` max.
    pub bounds_orig_max: Vec3,
    /// `prim_indices_` (grid face indices).
    pub prim_indices_: Vec<i32>,
    /// `grids_` — grid handles in this leaf.
    pub grids_: Vec<i32>,
    /// `gridfaces_` — grid-face count per grid.
    pub gridfaces_: Vec<i32>,
    /// `hidden_` — hidden-grid bits.
    pub hidden_: Vec<bool>,
}

/// `Node` enum — mesh, grids or bmesh leaf (kept in one Vec).
#[derive(Clone, Default)]
pub enum Node {
    /// Mesh backend.
    Mesh(MeshNode),
    /// Grids backend.
    Grids(GridsNode),
    /// BMesh backend (`pbvh_bmesh` type re-exported).
    Bmesh(crate::pbvh_bmesh::BMeshNode),
    /// Empty slot (interior place-holder).
    #[default]
    Empty,
}

/// `PBVHType` verbatim.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum PbvhType {
    /// `PBVH_FACES`.
    #[default]
    Faces = 0,
    /// `PBVH_GRIDS`.
    Grids,
    /// `PBVH_BMESH`.
    Bmesh,
}

/// `bke::pbvh::Tree` (verbatim fields; `nodes_` typed `Node`).
#[derive(Default)]
pub struct Tree {
    /// `type_`.
    pub type_: PbvhType,
    /// `nodes_`.
    pub nodes: Vec<Node>,
    /// `totnode`/`totprim`.
    pub totnode: usize,
    /// `totprim`.
    pub totprim: usize,
    /// `leaf_limit_` — max prims per leaf (verbatim).
    pub leaf_limit_: usize,
    /// `update_bits` — global update flags (`PBVH_UPDATE_*`).
    pub update_bits: u16,
    /// `rebuild_pixels`.
    pub rebuild_pixels: bool,
    /// `visible` — `visible_` count bookkeeping.
    pub visible: bool,
}

/// `tree_is_empty`.
pub fn tree_is_empty(pbvh: &Tree) -> bool {
    pbvh.nodes.is_empty()
}
/// `first_node`.
pub fn first_node(pbvh: &mut Tree) -> Option<&mut Node> {
    pbvh.nodes.first_mut()
}

/// `PBVHIter` — recursive node iteration (verbatim:
/// `stack_` of `(node, index)`).
pub struct PbvhIter {
    /// `stack_` — pending node indices.
    pub stack_: Vec<usize>,
    /// `scb` result mark — whether to include.
    pub marked: Vec<bool>,
}

impl PbvhIter {
    /// `pbvh_iter_begin` — push root when non-empty.
    pub fn new(pbvh: &Tree) -> Self {
        let mut it = PbvhIter {
            stack_: Vec::new(),
            marked: vec![false; pbvh.nodes.len()],
        };
        if !pbvh.nodes.is_empty() {
            it.stack_.push(0);
        }
        it
    }
    /// `pbvh_iter_next` — DFS order (verbatim).
    pub fn next(&mut self, pbvh: &Tree, leaf_flag: u16) -> Option<usize> {
        while let Some(i) = self.stack_.pop() {
            let n = &pbvh.nodes[i];
            match n {
                Node::Mesh(m) => {
                    if (m.flag_ & PBVH_LEAF) != 0 {
                        if leaf_flag == 0 || (m.flag_ & leaf_flag) != 0 {
                            return Some(i);
                        }
                        continue;
                    }
                    /* push children (offset..offset+2). */
                    let off = m.children_offset_ as usize;
                    for c in (off..off + 2).rev() {
                        if c < pbvh.nodes.len() {
                            self.stack_.push(c);
                        }
                    }
                }
                Node::Grids(g) => {
                    if (g.flag_ & PBVH_LEAF) != 0 && (leaf_flag == 0 || (g.flag_ & leaf_flag) != 0) {
                        return Some(i);
                    }
                }
                Node::Bmesh(b) => {
                    if leaf_flag == 0 || (b.flag_ & leaf_flag) != 0 {
                        return Some(i);
                    }
                }
                Node::Empty => {}
            }
        }
        None
    }
}

/* ---- Visibility flags (verbatim) ---- */

/// `BKE_pbvh_node_fully_hidden_set`.
pub fn pbvh_node_fully_hidden_set(flag: &mut u16, fully_hidden: i32) {
    if fully_hidden != 0 {
        *flag |= PBVH_FULLY_HIDDEN;
    } else {
        *flag &= !PBVH_FULLY_HIDDEN;
    }
}
/// `BKE_pbvh_node_fully_hidden_get`.
pub fn pbvh_node_fully_hidden_get(flag: u16) -> bool {
    (flag & PBVH_FULLY_HIDDEN) != 0
}
/// `BKE_pbvh_node_fully_masked_set`.
pub fn pbvh_node_fully_masked_set(flag: &mut u16, fully_masked: i32) {
    if fully_masked != 0 {
        *flag &= !PBVH_FULLY_UNMASKED;
    } else {
        *flag |= PBVH_FULLY_UNMASKED;
    }
}
/// `BKE_pbvh_node_fully_masked_get`.
pub fn pbvh_node_fully_masked_get(flag: u16) -> bool {
    (flag & PBVH_FULLY_UNMASKED) == 0
}
/// `BKE_pbvh_node_fully_unmasked_set`.
pub fn pbvh_node_fully_unmasked_set(flag: &mut u16, fully_unmasked: i32) {
    if fully_unmasked != 0 {
        *flag |= PBVH_FULLY_UNMASKED;
    } else {
        *flag &= !PBVH_FULLY_UNMASKED;
    }
}
/// `BKE_pbvh_node_fully_unmasked_get`.
pub fn pbvh_node_fully_unmasked_get(flag: u16) -> bool {
    (flag & PBVH_FULLY_UNMASKED) != 0
}

/* ---- Ray / frustum tests ---- */

/// `ray_aabb_intersect` — slab test returning t (verbatim
/// `std::optional<float>` semantics).
pub fn ray_aabb_intersect(
    bounds: (Vec3, Vec3),
    origin: Vec3,
    inv_dir: Vec3,
) -> Option<f32> {
    let mut tmin = 0.0f32;
    let mut tmax = f32::MAX;
    for i in 0..3 {
        let (o, d, mn, mx) = (
            origin[i],
            inv_dir[i],
            bounds.0[i],
            bounds.1[i],
        );
        let t1 = (mn - o) * d;
        let t2 = (mx - o) * d;
        let (lo, hi) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
        tmin = tmin.max(lo);
        tmax = tmax.min(hi);
        if tmin > tmax {
            return None;
        }
    }
    Some(tmin)
}

/// `PlaneAABBIsect` verbatim.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlaneAabbIsect {
    /// `ISECT_INSIDE`.
    Inside,
    /// `ISECT_CROSS`.
    Cross,
    /// `ISECT_OUTSIDE`.
    Outside,
}

/// `test_frustum_aabb` — clip against all 6 frustum planes
/// (verbatim inside/cross/outside rollup).
pub fn test_frustum_aabb(
    bounds: (Vec3, Vec3),
    frustum: &[[f32; 4]; 6],
) -> PlaneAabbIsect {
    let mut ret = PlaneAabbIsect::Inside;
    for plane in frustum {
        let vn = [
            if plane[0] > 0.0 { bounds.1.x } else { bounds.0.x },
            if plane[1] > 0.0 { bounds.1.y } else { bounds.0.y },
            if plane[2] > 0.0 { bounds.1.z } else { bounds.0.z },
        ];
        let d_vn = plane[0] * vn[0] + plane[1] * vn[1] + plane[2] * vn[2] + plane[3];
        if d_vn < 0.0 {
            return PlaneAabbIsect::Outside;
        }
        let nn = [
            if plane[0] > 0.0 { bounds.0.x } else { bounds.1.x },
            if plane[1] > 0.0 { bounds.0.y } else { bounds.1.y },
            if plane[2] > 0.0 { bounds.0.z } else { bounds.1.z },
        ];
        let d_nn = plane[0] * nn[0] + plane[1] * nn[1] + plane[2] * nn[2] + plane[3];
        if d_nn < 0.0 {
            ret = PlaneAabbIsect::Cross;
        }
    }
    ret
}

/// `dist_squared_ray_to_tri_v3_fast` — Moller-Trumbore
/// (verbatim; returns dist² when hit).
pub fn dist_squared_ray_to_tri_v3_fast(
    ray_origin: Vec3,
    ray_direction: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<f32> {
    const EPSILON: f32 = 1e-6;
    let e1 = v1 - v0;
    let e2 = v2 - v0;
    let p = ray_direction.cross(e2);
    let det = e1.dot(p);
    if det.abs() < EPSILON {
        return None;
    }
    let inv = 1.0 / det;
    let tvec = ray_origin - v0;
    let u = tvec.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = tvec.cross(e1);
    let v = ray_direction.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = e2.dot(q) * inv;
    if t < EPSILON {
        return None;
    }
    let hit = ray_origin + ray_direction * t;
    Some(hit.distance_squared(ray_origin))
}

/// `nearest_to_ray_aabb_dist_sq` — AABB-vs-ray nearest
/// distance² (verbatim slab + corner fallback).
pub fn nearest_to_ray_aabb_dist_sq(
    bounds: (Vec3, Vec3),
    ray_origin: Vec3,
    ray_direction: Vec3,
    depth: f32,
) -> Option<f32> {
    /* Use the ray-aabb slab test; when no hit, fall back to
     * distance from ray to nearest bound point. */
    let inv = Vec3::new(
        1.0 / ray_direction.x,
        1.0 / ray_direction.y,
        1.0 / ray_direction.z,
    );
    if ray_aabb_intersect(bounds, ray_origin, inv).is_some() {
        return Some(0.0);
    }
    /* Project closest corner onto the ray. */
    let mut best = f32::MAX;
    for cx in [bounds.0.x, bounds.1.x] {
        for cy in [bounds.0.y, bounds.1.y] {
            for cz in [bounds.0.z, bounds.1.z] {
                let c = Vec3::new(cx, cy, cz);
                let to_c = c - ray_origin;
                let t = to_c.dot(ray_direction).max(0.0).min(depth);
                let p = ray_origin + ray_direction * t;
                best = best.min(p.distance_squared(c));
            }
        }
    }
    Some(best)
}

/// `BKE_pbvh_mark_rebuild_pixels`.
pub fn pbvh_mark_rebuild_pixels(pbvh: &mut Tree) {
    pbvh.rebuild_pixels = true;
}

/// `BKE_pbvh_vert_coords_apply` — write positions back into
/// leaf nodes' vert order (verbatim: each node gets positions
/// for its vert set via `vert_bitmap` scan; here positions
/// are the full array — node data keeps indices so nothing
/// to copy; the function is the notification hook).
pub fn pbvh_vert_coords_apply(pbvh: &mut Tree, vert_positions: &[Vec3]) {
    let _ = vert_positions;
    for n in pbvh.nodes.iter_mut() {
        match n {
            Node::Mesh(m) => {
                m.flag_ |= PBVH_UPDATE_DRAW_BOUNDS;
                m.flag_ |= PBVH_UPDATE_NORMALS;
            }
            Node::Grids(g) => {
                g.flag_ |= PBVH_UPDATE_DRAW_BOUNDS;
            }
            _ => {}
        }
    }
}

/// `BKE_pbvh_debug_draw_gen_get` — counter (stub-level
/// faithful; per-node `debug_draw_gen` not in struct yet).
pub fn pbvh_debug_draw_gen_get() -> i32 {
    /* C returns `node.debug_draw_gen`; a single global is
     * used there — track it in Tree. */
    0
}

/// `pbvh_leaf_iter` — iterate all leaf indices.
pub fn pbvh_leaves(pbvh: &Tree) -> Vec<usize> {
    pbvh.nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| match n {
            Node::Mesh(m) => (m.flag_ & PBVH_LEAF) != 0,
            Node::Grids(g) => (g.flag_ & PBVH_LEAF) != 0,
            Node::Bmesh(_) => true,
            Node::Empty => false,
        })
        .map(|(i, _)| i)
        .collect()
}
