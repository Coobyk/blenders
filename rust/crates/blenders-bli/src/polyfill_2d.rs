//! Port of `polyfill_2d.cc` — ear-clipping polygon tessellation with
//! the dynamic `KDTree2D` accelerator. All upstream feature flags on:
//! USE_CLIP_EVEN, USE_CONVEX_SKIP, USE_CLIP_SWEEP, USE_KDTREE,
//! USE_KDTREE_INDEX_CACHE, USE_PRECOMPUTED_ISECT.
//!
//! `PolyIndex*` links become `u32` vertex-slot indices.

const KDNODE_UNSET: u32 = u32::MAX;
const KDNODE_FLAG_REMOVED: u16 = 1 << 0;
const KDTREE_INDEX_CACHE_UNSET: u32 = u32::MAX;

const CONCAVE: i8 = -1;
const TANGENTIAL: i8 = 0;
const CONVEX: i8 = 1;

/// `eSign`.
pub type ESign = i8;

#[derive(Default, Clone, Copy)]
struct KDTreeNode2D {
    neg: u32,
    pos: u32,
    index: u32,
    axis: u8,
    flag: u16,
    parent: u32,
}

#[derive(Default)]
struct KDTree2D {
    nodes: Vec<KDTreeNode2D>,
    root: u32,
    node_num: u32,
    nodes_map: Vec<u32>, // index -> node slot
}

#[derive(Clone, Copy)]
struct KDRange2D {
    min: f32,
    max: f32,
}

/// `PolyIndex` — circular doubly-linked vertex entry.
#[derive(Clone, Copy, Default)]
struct PolyIndex {
    next: u32,
    prev: u32,
    index: u32,
    sign: ESign,
    flag_dirty: bool,
    index_last_hit: u32,
}

struct PolyFill<'a> {
    indices: Vec<PolyIndex>,
    head: u32,
    coords: &'a [[f32; 2]],
    coords_num: u32,
    coords_num_concave: u32,
    tris: Vec<[u32; 3]>,
    kdtree: KDTree2D,
}

/// `TriIsectPrecomputed`.
struct TriIsectPrecomputed {
    edge: [[f32; 2]; 3],
    c: [f32; 3],
}

fn tri_isect_precomputed_init(vs: &[[f32; 2]; 3]) -> TriIsectPrecomputed {
    let e = |a: [f32; 2], b: [f32; 2]| [b[0] - a[0], b[1] - a[1]];
    let edge = [
        e(vs[0], vs[1]),
        e(vs[1], vs[2]),
        e(vs[2], vs[0]),
    ];
    TriIsectPrecomputed {
        edge,
        c: [
            edge[0][0] * vs[0][1] - vs[0][0] * edge[0][1],
            edge[1][0] * vs[1][1] - vs[1][0] * edge[1][1],
            edge[2][0] * vs[2][1] - vs[2][0] * edge[2][1],
        ],
    }
}

fn tri_isect_precomputed_test(t: &TriIsectPrecomputed, co: [f32; 2]) -> bool {
    (t.edge[1][1] * co[0] - t.edge[1][0] * co[1] + t.c[1] >= 0.0)
        && (t.edge[2][1] * co[0] - t.edge[2][0] * co[1] + t.c[2] >= 0.0)
        && (t.edge[0][1] * co[0] - t.edge[0][0] * co[1] + t.c[0] >= 0.0)
}

fn signum_enum(a: f32) -> ESign {
    if a > 0.0 {
        CONVEX
    } else if a == 0.0 {
        TANGENTIAL
    } else {
        CONCAVE
    }
}

fn area_tri_signed_v2_alt_2x(v1: [f32; 2], v2: [f32; 2], v3: [f32; 2]) -> f32 {
    let d2 = [v2[0] - v1[0], v2[1] - v1[1]];
    let d3 = [v3[0] - v1[0], v3[1] - v1[1]];
    d2[0] * d3[1] - d3[0] * d2[1]
}

/// `span_tri_v2_sign`.
fn span_tri_v2_sign(v1: [f32; 2], v2: [f32; 2], v3: [f32; 2]) -> ESign {
    signum_enum(area_tri_signed_v2_alt_2x(v1, v2, v3))
}

/// `cross_poly_v2` — polygon signed area x2.
fn cross_poly_v2(coords: &[[f32; 2]]) -> f32 {
    let n = coords.len();
    let mut area = 0.0f32;
    for i in 0..n {
        let a = coords[i];
        let b = coords[(i + 1) % n];
        area += a[0] * b[1] - b[0] * a[1];
    }
    area
}

// ---------------- KDTree2D ----------------

fn kdtree2d_balance_recursive(
    nodes: &mut [KDTreeNode2D],
    axis: u8,
    coords: &[[f32; 2]],
    ofs: u32,
) -> u32 {
    let node_num = nodes.len();
    if node_num == 0 {
        return KDNODE_UNSET;
    }
    if node_num == 1 {
        return ofs;
    }

    let mut neg = 0usize;
    let mut pos = node_num - 1;
    let median = node_num / 2;

    while pos > neg {
        let co = coords[nodes[pos].index as usize][axis as usize];
        let mut i = neg as i64 - 1;
        let mut j = pos;
        loop {
            loop {
                i += 1;
                if coords[nodes[i as usize].index as usize][axis as usize] >= co {
                    break;
                }
            }
            loop {
                j = j.wrapping_sub(1);
                if !(coords[nodes[j].index as usize][axis as usize] > co && j > neg) {
                    break;
                }
            }
            if i >= j as i64 {
                break;
            }
            nodes.swap(i as usize, j);
        }
        nodes.swap(i as usize, pos);
        if i as usize >= median {
            pos = (i - 1) as usize;
        }
        if (i as usize) <= median {
            neg = (i + 1) as usize;
        }
    }

    nodes[median].axis = axis;
    let next_axis = 1 - axis;
    let (left, right) = nodes.split_at_mut(median + 1);
    let neg_r = kdtree2d_balance_recursive(&mut left[..median], next_axis, coords, ofs);
    let pos_r = kdtree2d_balance_recursive(right, next_axis, coords, (median + 1) as u32 + ofs);
    nodes[median].neg = neg_r;
    nodes[median].pos = pos_r;
    (median as u32) + ofs
}

fn kdtree2d_init_mapping(tree: &mut KDTree2D) {
    for i in 0..tree.node_num as usize {
        let node = tree.nodes[i];
        if node.neg != KDNODE_UNSET {
            tree.nodes[node.neg as usize].parent = i as u32;
        }
        if node.pos != KDNODE_UNSET {
            tree.nodes[node.pos as usize].parent = i as u32;
        }
        tree.nodes_map[node.index as usize] = i as u32;
    }
    if tree.root != KDNODE_UNSET {
        tree.nodes[tree.root as usize].parent = KDNODE_UNSET;
    }
}

fn kdtree2d_node_remove(tree: &mut KDTree2D, index: u32) {
    let mut node_index = tree.nodes_map[index as usize];
    if node_index == KDNODE_UNSET {
        return;
    }
    tree.nodes_map[index as usize] = KDNODE_UNSET;
    tree.nodes[node_index as usize].flag |= KDNODE_FLAG_REMOVED;
    tree.node_num -= 1;

    while tree.nodes[node_index as usize].parent != KDNODE_UNSET {
        let node = tree.nodes[node_index as usize];
        let node_child = if node.neg == KDNODE_UNSET {
            node.pos
        } else if node.pos == KDNODE_UNSET {
            node.neg
        } else {
            break;
        };
        let parent = node.parent;
        if tree.nodes[parent as usize].neg == node_index {
            tree.nodes[parent as usize].neg = node_child;
        } else {
            tree.nodes[parent as usize].pos = node_child;
        }
        if node_child != KDNODE_UNSET {
            tree.nodes[node_child as usize].parent = parent;
        }
        if tree.nodes[parent as usize].flag & KDNODE_FLAG_REMOVED == 0 {
            break;
        }
        node_index = parent;
    }
}

fn kdtree2d_isect_tri_recursive(
    tree: &KDTree2D,
    tri_index: [u32; 3],
    tri_isect: &TriIsectPrecomputed,
    tri_center: [f32; 2],
    bounds: &[KDRange2D; 2],
    node_i: u32,
    coords: &[[f32; 2]],
) -> u32 {
    let node = &tree.nodes[node_i as usize];
    let co = coords[node.index as usize];

    if node.flag & KDNODE_FLAG_REMOVED == 0 {
        if tri_isect_precomputed_test(tri_isect, co)
            && !tri_index.contains(&node.index)
        {
            return node.index;
        }
    }

    let axis = node.axis as usize;
    let recurse_neg = |tree: &KDTree2D| -> u32 {
        if node.neg != KDNODE_UNSET && co[axis] >= bounds[axis].min {
            kdtree2d_isect_tri_recursive(
                tree,
                tri_index,
                tri_isect,
                tri_center,
                bounds,
                node.neg,
                coords,
            )
        } else {
            KDNODE_UNSET
        }
    };
    let recurse_pos = |tree: &KDTree2D| -> u32 {
        if node.pos != KDNODE_UNSET && co[axis] <= bounds[axis].max {
            kdtree2d_isect_tri_recursive(
                tree,
                tri_index,
                tri_isect,
                tri_center,
                bounds,
                node.pos,
                coords,
            )
        } else {
            KDNODE_UNSET
        }
    };

    if tri_center[axis] > co[axis] {
        let r = recurse_pos(tree);
        if r != KDNODE_UNSET {
            r
        } else {
            recurse_neg(tree)
        }
    } else {
        let r = recurse_neg(tree);
        if r != KDNODE_UNSET {
            r
        } else {
            recurse_pos(tree)
        }
    }
}

fn kdtree2d_isect_tri_single(
    tree: &KDTree2D,
    tri_index: [u32; 3],
    test_index: u32,
    coords: &[[f32; 2]],
) -> bool {
    if tree.nodes_map[test_index as usize] != KDNODE_UNSET && !tri_index.contains(&test_index) {
        let vs = [
            coords[tri_index[0] as usize],
            coords[tri_index[1] as usize],
            coords[tri_index[2] as usize],
        ];
        let t = tri_isect_precomputed_init(&vs);
        if tri_isect_precomputed_test(&t, coords[test_index as usize]) {
            return true;
        }
    }
    false
}

fn kdtree2d_isect_tri(tree: &KDTree2D, ind: [u32; 3], coords: &[[f32; 2]]) -> u32 {
    let vs = [
        coords[ind[0] as usize],
        coords[ind[1] as usize],
        coords[ind[2] as usize],
    ];
    let bounds = [
        KDRange2D {
            min: vs[0][0].min(vs[1][0]).min(vs[2][0]),
            max: vs[0][0].max(vs[1][0]).max(vs[2][0]),
        },
        KDRange2D {
            min: vs[0][1].min(vs[1][1]).min(vs[2][1]),
            max: vs[0][1].max(vs[1][1]).max(vs[2][1]),
        },
    ];
    let tri_center = [
        (vs[0][0] + vs[1][0] + vs[2][0]) / 3.0,
        (vs[0][1] + vs[1][1] + vs[2][1]) / 3.0,
    ];
    let tri_isect = tri_isect_precomputed_init(&vs);
    kdtree2d_isect_tri_recursive(
        tree,
        ind,
        &tri_isect,
        tri_center,
        &bounds,
        tree.root,
        coords,
    )
}

// ---------------- PolyFill ----------------

impl<'a> PolyFill<'a> {
    fn tri_add(&mut self) -> usize {
        self.tris.push([0; 3]);
        self.tris.len() - 1
    }

    /// `pf_coord_remove`.
    fn coord_remove(&mut self, pi: u32) {
        if self.kdtree.node_num > 0 {
            kdtree2d_node_remove(&mut self.kdtree, self.indices[pi as usize].index);
        }
        let (next, prev) = (
            self.indices[pi as usize].next,
            self.indices[pi as usize].prev,
        );
        self.indices[next as usize].prev = prev;
        self.indices[prev as usize].next = next;
        if self.head == pi {
            self.head = next;
        }
        self.coords_num -= 1;
    }

    /// `pf_coord_sign_calc`.
    fn coord_sign_calc(&mut self, pi: u32) {
        let p = self.indices[pi as usize];
        let (a, b, c) = (
            self.coords[self.indices[p.prev as usize].index as usize],
            self.coords[p.index as usize],
            self.coords[self.indices[p.next as usize].index as usize],
        );
        self.indices[pi as usize].sign = span_tri_v2_sign(a, b, c);
    }

    /// `pf_ear_tip_find`.
    fn ear_tip_find(&mut self, pi_ear_init: u32, reverse: bool) -> u32 {
        let coords_num = self.coords_num;
        for sign_accept in (TANGENTIAL..=CONVEX).rev() {
            let mut pi_ear = pi_ear_init;
            let mut i = coords_num;
            while i > 0 {
                i -= 1;
                if self.ear_tip_check_mut(pi_ear, sign_accept) {
                    return pi_ear;
                }
                pi_ear = if reverse {
                    self.indices[pi_ear as usize].prev
                } else {
                    self.indices[pi_ear as usize].next
                };
            }
        }
        let mut pi_ear = pi_ear_init;
        let mut i = coords_num;
        while i > 0 {
            i -= 1;
            if self.indices[pi_ear as usize].sign != CONCAVE {
                return pi_ear;
            }
            pi_ear = self.indices[pi_ear as usize].next;
        }
        pi_ear
    }

    /// `pf_ear_tip_check` (KD-tree path) — `&mut` because the index
    /// cache writes back `index_last_hit`/`flag_dirty`.
    fn ear_tip_check_mut(&mut self, pi_ear_tip: u32, sign_accept: ESign) -> bool {
        if self.coords_num_concave == 0 {
            return true;
        }
        if self.indices[pi_ear_tip as usize].sign != sign_accept {
            return false;
        }
        let pi = self.indices[pi_ear_tip as usize];
        let ind = [
            pi.index,
            self.indices[pi.next as usize].index,
            self.indices[pi.prev as usize].index,
        ];
        // Index cache.
        let cached_hit = pi.index_last_hit;
        if cached_hit != KDTREE_INDEX_CACHE_UNSET {
            if !pi.flag_dirty {
                if self.kdtree.nodes_map[cached_hit as usize] != KDNODE_UNSET {
                    return false;
                }
            } else if kdtree2d_isect_tri_single(
                &self.kdtree,
                ind,
                cached_hit,
                self.coords,
            ) {
                self.indices[pi_ear_tip as usize].flag_dirty = false;
                return false;
            }
        }
        let hit = kdtree2d_isect_tri(&self.kdtree, ind, self.coords);
        if hit != KDNODE_UNSET {
            self.indices[pi_ear_tip as usize].index_last_hit = hit;
            self.indices[pi_ear_tip as usize].flag_dirty = false;
            return false;
        }
        true
    }

    /// `pf_ear_tip_cut`.
    fn ear_tip_cut(&mut self, pi_ear_tip: u32) {
        let t = self.tri_add();
        let pi = self.indices[pi_ear_tip as usize];
        self.tris[t] = [
            self.indices[pi.prev as usize].index,
            pi.index,
            self.indices[pi.next as usize].index,
        ];
        self.coord_remove(pi_ear_tip);
    }

    /// `pf_triangulate`.
    fn triangulate(&mut self) {
        let mut pi_ear_init = self.head;
        let mut reverse = false;

        while self.coords_num > 3 {
            let pi_ear = self.ear_tip_find(pi_ear_init, reverse);

            if self.indices[pi_ear as usize].sign != CONVEX {
                self.coords_num_concave -= 1;
            }

            let pi_prev = self.indices[pi_ear as usize].prev;
            let pi_next = self.indices[pi_ear as usize].next;

            self.ear_tip_cut(pi_ear);

            self.indices[pi_prev as usize].flag_dirty = true;
            self.indices[pi_next as usize].flag_dirty = true;

            let sign_orig_prev = self.indices[pi_prev as usize].sign;
            let sign_orig_next = self.indices[pi_next as usize].sign;

            if sign_orig_prev != CONVEX {
                self.coord_sign_calc(pi_prev);
                if self.indices[pi_prev as usize].sign == CONVEX {
                    self.coords_num_concave -= 1;
                    let idx = self.indices[pi_prev as usize].index;
                    kdtree2d_node_remove(&mut self.kdtree, idx);
                }
            }
            if sign_orig_next != CONVEX {
                self.coord_sign_calc(pi_next);
                if self.indices[pi_next as usize].sign == CONVEX {
                    self.coords_num_concave -= 1;
                    let idx = self.indices[pi_next as usize].index;
                    kdtree2d_node_remove(&mut self.kdtree, idx);
                }
            }

            pi_ear_init = if reverse {
                self.indices[pi_prev as usize].prev
            } else {
                self.indices[pi_next as usize].next
            };
            if self.indices[pi_ear_init as usize].sign != CONVEX {
                pi_ear_init = if reverse {
                    self.indices[pi_ear_init as usize].prev
                } else {
                    self.indices[pi_ear_init as usize].next
                };
                reverse = !reverse;
            }
        }

        if self.coords_num == 3 {
            let t = self.tri_add();
            let mut pi_ear = self.head;
            self.tris[t][0] = self.indices[pi_ear as usize].index;
            pi_ear = self.indices[pi_ear as usize].next;
            self.tris[t][1] = self.indices[pi_ear as usize].index;
            pi_ear = self.indices[pi_ear as usize].next;
            self.tris[t][2] = self.indices[pi_ear as usize].index;
        }
    }
}

/// `polyfill_prepare`.
fn polyfill_prepare<'a>(
    coords: &'a [[f32; 2]],
    coords_sign: i32,
) -> PolyFill<'a> {
    let coords_num = coords.len() as u32;
    let mut indices = vec![PolyIndex::default(); coords_num as usize];

    let sign = if coords_sign == 0 {
        if cross_poly_v2(coords) <= 0.0 {
            1
        } else {
            -1
        }
    } else {
        coords_sign
    };

    if sign == 1 {
        for i in 0..coords_num {
            indices[i as usize].next = i + 1;
            indices[i as usize].prev = i.wrapping_sub(1);
            indices[i as usize].index = i;
        }
    } else {
        let n = coords_num - 1;
        for i in 0..coords_num {
            indices[i as usize].next = i + 1;
            indices[i as usize].prev = i.wrapping_sub(1);
            indices[i as usize].index = n - i;
        }
    }
    indices[0].prev = coords_num - 1;
    indices[coords_num as usize - 1].next = 0;

    let mut pf = PolyFill {
        indices,
        head: 0,
        coords,
        coords_num,
        coords_num_concave: 0,
        tris: Vec::new(),
        kdtree: KDTree2D::default(),
    };
    for i in 0..coords_num {
        pf.coord_sign_calc(i);
        if pf.indices[i as usize].sign != CONVEX {
            pf.coords_num_concave += 1;
        }
        pf.indices[i as usize].flag_dirty = false;
        pf.indices[i as usize].index_last_hit = KDTREE_INDEX_CACHE_UNSET;
    }
    pf
}

/// `polyfill_calc` — build the KD-tree (over concave verts) and clip.
fn polyfill_calc(pf: &mut PolyFill) {
    if pf.coords_num_concave > 0 {
        let n = pf.coords_num_concave as usize;
        pf.kdtree.node_num = n as u32;
        pf.kdtree.root = KDNODE_UNSET;
        pf.kdtree.nodes_map = vec![KDNODE_UNSET; pf.coords.len()];
        pf.kdtree.nodes = vec![KDTreeNode2D::default(); n];
        let mut w = 0usize;
        for i in 0..pf.coords_num {
            let pi = pf.indices[i as usize];
            if pi.sign != CONVEX {
                pf.kdtree.nodes[w].neg = KDNODE_UNSET;
                pf.kdtree.nodes[w].pos = KDNODE_UNSET;
                pf.kdtree.nodes[w].index = pi.index;
                pf.kdtree.nodes[w].axis = 0;
                pf.kdtree.nodes[w].flag = 0;
                w += 1;
            }
        }
        pf.kdtree.root =
            kdtree2d_balance_recursive(&mut pf.kdtree.nodes, 0, pf.coords, 0);
        kdtree2d_init_mapping(&mut pf.kdtree);
    } else {
        pf.kdtree.node_num = 0;
    }
    pf.triangulate();
}

/// `BLI_polyfill_calc` — tessellate an ngon into triangles.
/// `coords_sign` 0 = auto, 1 = forward winding, -1 = reverse.
/// Returns `n - 2` triangles as index triples.
pub fn polyfill_calc_pub(coords: &[[f32; 2]], coords_sign: i32) -> Vec<[u32; 3]> {
    let mut pf = polyfill_prepare(coords, coords_sign);
    polyfill_calc(&mut pf);
    pf.tris
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn square_and_concave() {
        let sq = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let tris = polyfill_calc_pub(&sq, 0);
        assert_eq!(tris.len(), 2);
        // Concave "arrow" pentagon.
        let ar = [
            [0.0, 0.0],
            [2.0, 0.0],
            [2.0, 1.0],
            [1.0, 0.5],
            [0.0, 1.0],
        ];
        let tris = polyfill_calc_pub(&ar, 0);
        assert_eq!(tris.len(), 3);
        // All emitted verts are original indices.
        for t in &tris {
            for &i in t {
                assert!((i as usize) < ar.len());
            }
        }
    }
}
