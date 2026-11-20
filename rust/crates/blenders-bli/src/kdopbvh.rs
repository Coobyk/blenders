//! Port of `BLI_kdopbvh.cc` — k-DOP bounding volume hierarchy.
//!
//! Pointer tree -> index arena: `nodearray` holds `BVHNode`s, `nodes` holds
//! indices into it (leaves first, then branches), `nodechild`/`nodebv` are flat
//! slabs addressed by `child_base`/`bv_base`. `parent`/`children` are indices
//! (`-1`/`None` for null). Threading (`BLI_task_parallel_range`) is serial.
//!
//! PORT_NOTE: `USE_SKIP_LINKS`, `USE_PRINT_TREE`, `USE_VERIFY_TREE` are off
//! upstream and not ported (skip links, tree printing, verify pass).

use blenders_math::geom::{
    aabb_get_near_far_from_plane, dist_squared_to_projected_aabb,
    dist_squared_to_projected_aabb_precalc, isect_aabb_planes, isect_ray_tri_watertight_precalc,
    plane_point_side, planes_from_projmat, AabbPlaneIsect, DistProjectedAabbPrecalc,
    IsectRayPrecalc,
};
use blenders_math::matrix::Mat4;
use blenders_math::vector::{Vec2, Vec3, Vec4};
use blenders_math::vector_inline::normalize_v3;

pub const MAX_TREETYPE: usize = 32;
/// `KDOPBVH_THREAD_LEAF_THRESHOLD` (NDEBUG value).
pub const KDOPBVH_THREAD_LEAF_THRESHOLD: usize = 1024;
/// `BVH_RAYCAST_DIST_MAX`.
pub const BVH_RAYCAST_DIST_MAX: f32 = 1e20;

pub const BVH_OVERLAP_RETURN_PAIRS: i32 = 1 << 0;
pub const BVH_OVERLAP_USE_THREADING: i32 = 1 << 1;
pub const BVH_OVERLAP_SELF: i32 = 1 << 2;

pub const BVH_RAYCAST_DEFAULT: i32 = 0;
pub const BVH_RAYCAST_WATERTIGHT: i32 = 1 << 0;

pub const BVH_NEAREST_OPTIMAL_ORDER: i32 = 1 << 0;

/// `bvhtree_kdop_axes[13][3]`.
pub const BVHTREE_KDOP_AXES: [[f32; 3]; 13] = [
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
    [1.0, 1.0, 1.0],
    [1.0, -1.0, 1.0],
    [1.0, 1.0, -1.0],
    [1.0, -1.0, -1.0],
    [1.0, 1.0, 0.0],
    [1.0, 0.0, 1.0],
    [0.0, 1.0, 1.0],
    [1.0, -1.0, 0.0],
    [1.0, 0.0, -1.0],
    [0.0, 1.0, -1.0],
];

/// `bvhtree_kdop_axes_length[13]`.
pub const BVHTREE_KDOP_AXES_LENGTH: [f32; 13] = [
    1.0,
    1.0,
    1.0,
    1.7320508075688772,
    1.7320508075688772,
    1.7320508075688772,
    1.7320508075688772,
    1.4142135623730951,
    1.4142135623730951,
    1.4142135623730951,
    1.4142135623730951,
    1.4142135623730951,
    1.4142135623730951,
];

#[derive(Clone, Copy, Default)]
struct BVHNode {
    child_base: usize,
    bv_base: usize,
    parent: i32,
    index: i32,
    node_num: u8,
    main_axis: u8,
}

/// `BVHTree`.
pub struct BVHTree {
    nodes: Vec<i32>,
    nodearray: Vec<BVHNode>,
    nodechild: Vec<i32>,
    nodebv: Vec<f32>,
    epsilon: f32,
    leaf_num: usize,
    branch_num: usize,
    nodes_num_alloc: usize,
    start_axis: u8,
    stop_axis: u8,
    axis: u8,
    tree_type: u8,
}

/// `BVHTreeOverlap`.
#[derive(Clone, Copy, Default)]
pub struct BVHTreeOverlap {
    pub index_a: i32,
    pub index_b: i32,
}

/// `BVHTreeNearest`.
#[derive(Clone, Copy)]
pub struct BVHTreeNearest {
    pub index: i32,
    pub dist_sq: f32,
    pub co: [f32; 3],
}

/// `BVHTreeRay`.
#[derive(Clone)]
pub struct BVHTreeRay {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
    pub radius: f32,
    pub isect_precalc: Option<IsectRayPrecalc>,
}

/// `BVHTreeRayHit`.
#[derive(Clone, Copy)]
pub struct BVHTreeRayHit {
    pub index: i32,
    pub dist: f32,
    pub co: [f32; 3],
    pub no: [f32; 3],
}

impl Default for BVHTreeRayHit {
    fn default() -> Self {
        BVHTreeRayHit {
            index: -1,
            dist: BVH_RAYCAST_DIST_MAX,
            co: [0.0; 3],
            no: [0.0; 3],
        }
    }
}

/// `BVHTreeAxisRange`.
#[derive(Clone, Copy)]
pub struct BVHTreeAxisRange {
    pub min: f32,
    pub max: f32,
}

type Axis = u8;

impl BVHTree {
    fn bv(&self, n: usize) -> &[f32] {
        let b = self.nodearray[n].bv_base;
        &self.nodebv[b..b + self.axis as usize]
    }
    fn bv_mut(&mut self, n: usize) -> &mut [f32] {
        let a = self.axis as usize;
        let b = self.nodearray[n].bv_base;
        &mut self.nodebv[b..b + a]
    }
    fn child(&self, n: usize, k: usize) -> Option<usize> {
        let c = self.nodechild[self.nodearray[n].child_base + k];
        if c >= 0 {
            Some(c as usize)
        } else {
            None
        }
    }
    fn set_child(&mut self, n: usize, k: usize, c: usize) {
        let base = self.nodearray[n].child_base;
        self.nodechild[base + k] = c as i32;
    }
    fn node_index(&self, n: usize) -> i32 {
        self.nodearray[n].index
    }
    fn node_num(&self, n: usize) -> usize {
        self.nodearray[n].node_num as usize
    }
}

/// `node_minmax_init`.
fn node_minmax_init(tree_start: Axis, tree_stop: Axis, bv: &mut [f32]) {
    for a in tree_start..tree_stop {
        let a = a as usize;
        bv[2 * a] = f32::MAX;
        bv[2 * a + 1] = -f32::MAX;
    }
}

/* Intro-sort on leaf indices by `bv[axis]` — insertion sort, partition, median-of-3. */

fn node_bv_axis(tree: &BVHTree, node_id: i32, axis: usize) -> f32 {
    tree.nodebv[tree.nodearray[node_id as usize].bv_base + axis]
}

fn bvh_insertionsort(tree: &BVHTree, a: &mut [i32], lo: usize, hi: usize, axis: usize) {
    for i in lo..hi {
        let mut j = i;
        let t = a[i];
        while j != lo && node_bv_axis(tree, t, axis) < node_bv_axis(tree, a[j - 1], axis) {
            a[j] = a[j - 1];
            j -= 1;
        }
        a[j] = t;
    }
}

fn bvh_partition(tree: &BVHTree, a: &mut [i32], lo: usize, hi: usize, x: i32, axis: usize) -> usize {
    let mut i = lo;
    let mut j = hi;
    loop {
        while node_bv_axis(tree, a[i], axis) < node_bv_axis(tree, x, axis) {
            i += 1;
        }
        j -= 1;
        while node_bv_axis(tree, x, axis) < node_bv_axis(tree, a[j], axis) {
            j -= 1;
        }
        if i >= j {
            return i;
        }
        a.swap(i, j);
        i += 1;
    }
}

fn bvh_medianof3(tree: &BVHTree, a: &[i32], lo: usize, mid: usize, hi: usize, axis: usize) -> i32 {
    if node_bv_axis(tree, a[mid], axis) < node_bv_axis(tree, a[lo], axis) {
        if node_bv_axis(tree, a[hi], axis) < node_bv_axis(tree, a[mid], axis) {
            return a[mid];
        }
        if node_bv_axis(tree, a[hi], axis) < node_bv_axis(tree, a[lo], axis) {
            return a[hi];
        }
        return a[lo];
    }
    if node_bv_axis(tree, a[hi], axis) < node_bv_axis(tree, a[mid], axis) {
        if node_bv_axis(tree, a[hi], axis) < node_bv_axis(tree, a[lo], axis) {
            return a[lo];
        }
        return a[hi];
    }
    a[mid]
}

/// `partition_nth_element`.
fn partition_nth_element(
    tree: &BVHTree,
    a: &mut [i32],
    mut begin: usize,
    mut end: usize,
    n: usize,
    axis: usize,
) {
    while end - begin > 3 {
        let pivot = bvh_medianof3(tree, a, begin, (begin + end) / 2, end - 1, axis);
        let cut = bvh_partition(tree, a, begin, end, pivot, axis);
        if cut <= n {
            begin = cut;
        } else {
            end = cut;
        }
    }
    bvh_insertionsort(tree, a, begin, end, axis);
}

/// `create_kdop_hull`.
fn create_kdop_hull(tree: &mut BVHTree, node: usize, co: &[f32], numpoints: usize, moving: bool) {
    if !moving {
        let (start, stop) = (tree.start_axis, tree.stop_axis);
        node_minmax_init(start, stop, tree.bv_mut(node));
    }
    for k in 0..numpoints {
        for a in tree.start_axis..tree.stop_axis {
            let axis = &BVHTREE_KDOP_AXES[a as usize];
            let p = &co[k * 3..k * 3 + 3];
            let newminmax = p[0] * axis[0] + p[1] * axis[1] + p[2] * axis[2];
            let bv = tree.bv_mut(node);
            bv[2 * a as usize] = bv[2 * a as usize].min(newminmax);
            bv[2 * a as usize + 1] = bv[2 * a as usize + 1].max(newminmax);
        }
    }
}

/// `refit_kdop_hull` — re-fit node BV from leaf BVs `nodes[start..end]`.
fn refit_kdop_hull(tree: &mut BVHTree, node: usize, start: usize, end: usize) {
    let (s, e) = (tree.start_axis, tree.stop_axis);
    node_minmax_init(s, e, tree.bv_mut(node));
    for j in start..end {
        for a in s..e {
            let a = a as usize;
            let nb = tree.bv(tree.nodes[j] as usize);
            let (newmin, newmax) = (nb[2 * a], nb[2 * a + 1]);
            let bv = tree.bv_mut(node);
            bv[2 * a] = bv[2 * a].min(newmin);
            bv[2 * a + 1] = bv[2 * a + 1].max(newmax);
        }
    }
}

/// `get_largest_axis`.
fn get_largest_axis(bv: &[f32]) -> u8 {
    let mp = [bv[1] - bv[0], bv[3] - bv[2], bv[5] - bv[4]];
    if mp[0] > mp[1] {
        if mp[0] > mp[2] {
            1
        } else {
            5
        }
    } else if mp[1] > mp[2] {
        3
    } else {
        5
    }
}

/// `node_join` — bottom-up update of node BV from children.
fn node_join(tree: &mut BVHTree, node: usize) {
    let (s, e) = (tree.start_axis, tree.stop_axis);
    node_minmax_init(s, e, tree.bv_mut(node));
    for i in 0..tree.tree_type as usize {
        if let Some(c) = tree.child(node, i) {
            for a in s..e {
                let a = a as usize;
                let (cmin, cmax) = (tree.bv(c)[2 * a], tree.bv(c)[2 * a + 1]);
                let bv = tree.bv_mut(node);
                bv[2 * a] = bv[2 * a].min(cmin);
                bv[2 * a + 1] = bv[2 * a + 1].max(cmax);
            }
        } else {
            break;
        }
    }
}

/* Implicit tree construction. */

struct BVHBuildHelper {
    tree_type: i32,
    leafs_num: i32,
    leafs_per_child: [i32; 32],
    branches_on_level: [i32; 32],
    remain_leafs: i32,
}

fn build_implicit_tree_helper(tree: &BVHTree) -> BVHBuildHelper {
    let mut data = BVHBuildHelper {
        tree_type: tree.tree_type as i32,
        leafs_num: tree.leaf_num as i32,
        leafs_per_child: [0; 32],
        branches_on_level: [0; 32],
        remain_leafs: 0,
    };
    data.leafs_per_child[0] = 1;
    while data.leafs_per_child[0] < data.leafs_num {
        data.leafs_per_child[0] *= data.tree_type;
    }
    data.branches_on_level[0] = 1;
    let mut depth = 1usize;
    while depth < 32 && data.leafs_per_child[depth - 1] != 0 {
        data.branches_on_level[depth] = data.branches_on_level[depth - 1] * data.tree_type;
        data.leafs_per_child[depth] = data.leafs_per_child[depth - 1] / data.tree_type;
        depth += 1;
    }
    let remain = data.leafs_num - data.leafs_per_child[1];
    let nnodes = (remain + data.tree_type - 2) / (data.tree_type - 1);
    data.remain_leafs = remain + nnodes;
    data
}

fn implicit_leafs_index(data: &BVHBuildHelper, depth: usize, child_index: i32) -> i32 {
    let min_leaf_index = child_index * data.leafs_per_child[depth - 1];
    if min_leaf_index <= data.remain_leafs {
        return min_leaf_index;
    }
    if data.leafs_per_child[depth] != 0 {
        return data.leafs_num
            - (data.branches_on_level[depth - 1] - child_index) * data.leafs_per_child[depth];
    }
    data.remain_leafs
}

/// `implicit_needed_branches`.
fn implicit_needed_branches(tree_type: i32, leafs: i32) -> i32 {
    1.max((leafs + tree_type - 3) / (tree_type - 1))
}

fn split_leafs(tree: &BVHTree, leafs_array: &mut Vec<i32>, nth: &[i32], partitions: usize, axis: usize) {
    for i in 0..partitions - 1 {
        if nth[i] >= nth[partitions] {
            break;
        }
        partition_nth_element(
            tree,
            leafs_array,
            nth[i] as usize,
            nth[partitions] as usize,
            nth[i + 1] as usize,
            axis,
        );
    }
}

/// `non_recursive_bvh_div_nodes` — serial port (task_cb inlined per level).
fn non_recursive_bvh_div_nodes(tree: &mut BVHTree, branches_base: usize, leafs_num: i32) {
    let tree_type = tree.tree_type as i32;
    let tree_offset = 2 - tree_type;
    let branches_num = implicit_needed_branches(tree_type, leafs_num);

    {
        let root = branches_base + 1;
        tree.nodearray[root].parent = -1;
        if leafs_num == 1 {
            refit_kdop_hull(tree, root, 0, leafs_num as usize);
            let ma = get_largest_axis(tree.bv(root)) / 2;
            tree.nodearray[root].main_axis = ma as u8;
            tree.nodearray[root].node_num = 1;
            let leaf = tree.nodes[0] as usize;
            tree.set_child(root, 0, leaf);
            tree.nodearray[leaf].parent = root as i32;
            return;
        }
    }

    let data = build_implicit_tree_helper(tree);

    let mut i: i32 = 1;
    let mut depth = 1usize;
    while i <= branches_num {
        let first_of_next_level = i * tree_type + tree_offset;
        let i_stop = first_of_next_level.min(branches_num + 1);

        for j in i..i_stop {
            /* non_recursive_bvh_div_nodes_task_cb inlined */
            let parent_level_index = j - i;
            let parent = branches_base + j as usize;
            let parent_leafs_begin =
                implicit_leafs_index(&data, depth, parent_level_index);
            let parent_leafs_end =
                implicit_leafs_index(&data, depth, parent_level_index + 1);

            refit_kdop_hull(tree, parent, parent_leafs_begin as usize, parent_leafs_end as usize);
            let split_axis = get_largest_axis(tree.bv(parent));
            tree.nodearray[parent].main_axis = split_axis / 2;

            let mut nth = [0i32; MAX_TREETYPE + 1];
            nth[0] = parent_leafs_begin;
            nth[tree_type as usize] = parent_leafs_end;
            for k in 1..tree_type {
                let child_index = j * tree_type + tree_offset + k;
                let child_level_index = child_index - first_of_next_level;
                nth[k as usize] =
                    implicit_leafs_index(&data, depth + 1, child_level_index);
            }
            let mut nodes = std::mem::take(&mut tree.nodes);
            split_leafs(tree, &mut nodes, &nth, tree_type as usize, split_axis as usize);
            tree.nodes = nodes;

            let mut k = 0i32;
            while k < tree_type {
                let child_index = j * tree_type + tree_offset + k;
                let child_level_index = child_index - first_of_next_level;
                let child_leafs_begin = implicit_leafs_index(&data, depth + 1, child_level_index);
                let child_leafs_end = implicit_leafs_index(&data, depth + 1, child_level_index + 1);

                if child_leafs_end - child_leafs_begin > 1 {
                    let cid = branches_base + child_index as usize;
                    tree.set_child(parent, k as usize, cid);
                    tree.nodearray[cid].parent = parent as i32;
                } else if child_leafs_end - child_leafs_begin == 1 {
                    let leaf = tree.nodes[child_leafs_begin as usize] as usize;
                    tree.set_child(parent, k as usize, leaf);
                    tree.nodearray[leaf].parent = parent as i32;
                } else {
                    break;
                }
                k += 1;
            }
            tree.nodearray[parent].node_num = k as u8;
        }

        i = i * tree_type + tree_offset;
        depth += 1;
    }
}

/// `bvhtree_node_inflate`.
fn bvhtree_node_inflate(tree: &mut BVHTree, node: usize, dist: f32) {
    for a in tree.start_axis..tree.stop_axis {
        let a = a as usize;
        let dc = dist * BVHTREE_KDOP_AXES_LENGTH[a];
        let bv = tree.bv_mut(node);
        bv[2 * a] -= dc;
        bv[2 * a + 1] += dc;
    }
}

/* Public API. */

/// `BLI_bvhtree_new`.
pub fn bvhtree_new(maxsize: i32, epsilon: f32, tree_type: u8, axis: u8) -> BVHTree {
    assert!((2..=MAX_TREETYPE as u8).contains(&tree_type));
    let epsilon = epsilon.max(f32::EPSILON);
    let (start_axis, stop_axis) = match axis {
        26 => (0, 13),
        18 => (7, 13),
        14 => (0, 7),
        8 => (0, 4),  /* AABB */
        6 => (0, 3),  /* OBB */
        _ => unreachable!(),
    };
    let nodes_num_alloc =
        (maxsize + implicit_needed_branches(tree_type as i32, maxsize) + tree_type as i32) as usize;
    let mut tree = BVHTree {
        nodes: vec![-1; nodes_num_alloc],
        nodearray: vec![BVHNode::default(); nodes_num_alloc],
        nodechild: vec![-1; tree_type as usize * nodes_num_alloc],
        nodebv: vec![0.0; axis as usize * nodes_num_alloc],
        epsilon,
        leaf_num: 0,
        branch_num: 0,
        nodes_num_alloc,
        start_axis,
        stop_axis,
        axis,
        tree_type,
    };
    for i in 0..nodes_num_alloc {
        tree.nodearray[i].bv_base = i * axis as usize;
        tree.nodearray[i].child_base = i * tree_type as usize;
        tree.nodearray[i].parent = -1;
        tree.nodearray[i].index = -1;
    }
    tree
}

/// `BLI_bvhtree_balance`.
pub fn bvhtree_balance(tree: &mut BVHTree) {
    assert_eq!(tree.branch_num, 0);
    let branches_base = tree.leaf_num - 1;
    let leafs_num = tree.leaf_num as i32;
    non_recursive_bvh_div_nodes(tree, branches_base, leafs_num);
    tree.branch_num = implicit_needed_branches(tree.tree_type as i32, leafs_num) as usize;
    for i in 0..tree.branch_num {
        let id = (tree.leaf_num + i) as i32;
        tree.nodes[tree.leaf_num + i] = id;
    }
}

/// `BLI_bvhtree_insert`.
pub fn bvhtree_insert(tree: &mut BVHTree, index: i32, co: &[f32], numpoints: usize) {
    assert!(tree.branch_num == 0);
    assert!(tree.leaf_num < tree.nodes_num_alloc);
    let node = tree.leaf_num;
    tree.nodes[node] = node as i32;
    tree.leaf_num += 1;
    create_kdop_hull(tree, node, co, numpoints, false);
    tree.nodearray[node].index = index;
    bvhtree_node_inflate(tree, node, tree.epsilon);
}

/// `BLI_bvhtree_update_node`.
pub fn bvhtree_update_node(
    tree: &mut BVHTree,
    index: usize,
    co: &[f32],
    co_moving: Option<&[f32]>,
    numpoints: usize,
) -> bool {
    if index > tree.leaf_num {
        return false;
    }
    create_kdop_hull(tree, index, co, numpoints, false);
    if let Some(cm) = co_moving {
        create_kdop_hull(tree, index, cm, numpoints, true);
    }
    bvhtree_node_inflate(tree, index, tree.epsilon);
    true
}

/// `BLI_bvhtree_update_tree`.
pub fn bvhtree_update_tree(tree: &mut BVHTree) {
    let root = tree.leaf_num;
    for i in (root..root + tree.branch_num).rev() {
        let node = tree.nodes[i] as usize;
        node_join(tree, node);
    }
}

/// `BLI_bvhtree_get_len`.
pub fn bvhtree_get_len(tree: &BVHTree) -> i32 {
    tree.leaf_num as i32
}
/// `BLI_bvhtree_get_tree_type`.
pub fn bvhtree_get_tree_type(tree: &BVHTree) -> i32 {
    tree.tree_type as i32
}
/// `BLI_bvhtree_get_epsilon`.
pub fn bvhtree_get_epsilon(tree: &BVHTree) -> f32 {
    tree.epsilon
}
/// `BLI_bvhtree_get_bounding_box`.
pub fn bvhtree_get_bounding_box(tree: &BVHTree) -> ([f32; 3], [f32; 3]) {
    let root = tree.nodes[tree.leaf_num];
    if root >= 0 {
        let bv = tree.bv(root as usize);
        ([bv[0], bv[2], bv[4]], [bv[1], bv[3], bv[5]])
    } else {
        ([0.0; 3], [0.0; 3])
    }
}

/* Overlap. */

fn tree_overlap_test(bv1: &[f32], bv2: &[f32], start_axis: Axis, stop_axis: Axis) -> bool {
    for a in start_axis..stop_axis {
        let a = a as usize;
        if bv1[2 * a] > bv2[2 * a + 1] || bv2[2 * a] > bv1[2 * a + 1] {
            return false;
        }
    }
    true
}

type OverlapCallback<'a> = Option<&'a mut dyn FnMut(i32, i32, i32) -> bool>;

struct OverlapCtx<'a, 'b> {
    overlaps: Option<&'b mut Vec<BVHTreeOverlap>>,
    callback: OverlapCallback<'a>,
    max_interactions: u32,
    thread: i32,
}

impl<'a, 'b> OverlapCtx<'a, 'b> {
    fn push(&mut self, ia: i32, ib: i32) {
        if let Some(o) = self.overlaps.as_deref_mut() {
            o.push(BVHTreeOverlap {
                index_a: ia,
                index_b: ib,
            });
        }
    }
}

fn tree_overlap_traverse(
    t1: &BVHTree,
    t2: &BVHTree,
    same_tree: bool,
    ctx: &mut OverlapCtx<'_, '_>,
    start: Axis,
    stop: Axis,
    n1: usize,
    n2: usize,
) {
    if tree_overlap_test(t1.bv(n1), t2.bv(n2), start, stop) {
        if t1.node_num(n1) == 0 {
            if t2.node_num(n2) == 0 {
                if same_tree && n1 == n2 {
                    return;
                }
                ctx.push(t1.node_index(n1), t2.node_index(n2));
            } else {
                for j in 0..t2.tree_type as usize {
                    if let Some(c) = t2.child(n2, j) {
                        tree_overlap_traverse(t1, t2, same_tree, ctx, start, stop, n1, c);
                    }
                }
            }
        } else {
            for j in 0..t1.tree_type as usize {
                if let Some(c) = t1.child(n1, j) {
                    tree_overlap_traverse(t1, t2, same_tree, ctx, start, stop, c, n2);
                }
            }
        }
    }
}

fn tree_overlap_traverse_cb(
    t1: &BVHTree,
    t2: &BVHTree,
    same_tree: bool,
    ctx: &mut OverlapCtx<'_, '_>,
    start: Axis,
    stop: Axis,
    n1: usize,
    n2: usize,
) {
    if tree_overlap_test(t1.bv(n1), t2.bv(n2), start, stop) {
        if t1.node_num(n1) == 0 {
            if t2.node_num(n2) == 0 {
                if same_tree && n1 == n2 {
                    return;
                }
                let cb = ctx.callback.as_deref_mut().unwrap();
                if cb(t1.node_index(n1), t2.node_index(n2), ctx.thread) {
                    ctx.push(t1.node_index(n1), t2.node_index(n2));
                }
            } else {
                for j in 0..t2.tree_type as usize {
                    if let Some(c) = t2.child(n2, j) {
                        tree_overlap_traverse_cb(t1, t2, same_tree, ctx, start, stop, n1, c);
                    }
                }
            }
        } else {
            for j in 0..t1.tree_type as usize {
                if let Some(c) = t1.child(n1, j) {
                    tree_overlap_traverse_cb(t1, t2, same_tree, ctx, start, stop, c, n2);
                }
            }
        }
    }
}

fn tree_overlap_traverse_num(
    t1: &BVHTree,
    t2: &BVHTree,
    same_tree: bool,
    ctx: &mut OverlapCtx<'_, '_>,
    start: Axis,
    stop: Axis,
    n1: usize,
    n2: usize,
) -> bool {
    if tree_overlap_test(t1.bv(n1), t2.bv(n2), start, stop) {
        if t1.node_num(n1) == 0 {
            if t2.node_num(n2) == 0 {
                if same_tree && n1 == n2 {
                    return false;
                }
                let hit = match ctx.callback.as_deref_mut() {
                    None => true,
                    Some(cb) => cb(t1.node_index(n1), t2.node_index(n2), ctx.thread),
                };
                if hit {
                    ctx.push(t1.node_index(n1), t2.node_index(n2));
                    ctx.max_interactions -= 1;
                    return ctx.max_interactions == 0;
                }
            } else {
                for j in 0..t2.node_num(n2) {
                    let c = t2.child(n2, j).unwrap();
                    if tree_overlap_traverse_num(t1, t2, same_tree, ctx, start, stop, n1, c) {
                        return true;
                    }
                }
            }
        } else {
            let max_i = ctx.max_interactions;
            for j in 0..t1.node_num(n1) {
                let c = t1.child(n1, j).unwrap();
                if tree_overlap_traverse_num(t1, t2, same_tree, ctx, start, stop, c, n2) {
                    ctx.max_interactions = max_i;
                }
            }
        }
    }
    false
}

fn tree_overlap_invoke_traverse(
    t1: &BVHTree,
    t2: &BVHTree,
    same_tree: bool,
    ctx: &mut OverlapCtx<'_, '_>,
    start: Axis,
    stop: Axis,
    n1: usize,
    n2: usize,
) {
    if ctx.max_interactions > 0 {
        tree_overlap_traverse_num(t1, t2, same_tree, ctx, start, stop, n1, n2);
    } else if ctx.callback.is_some() {
        tree_overlap_traverse_cb(t1, t2, same_tree, ctx, start, stop, n1, n2);
    } else {
        tree_overlap_traverse(t1, t2, same_tree, ctx, start, stop, n1, n2);
    }
}

fn tree_overlap_traverse_self_cb(t: &BVHTree, ctx: &mut OverlapCtx<'_, '_>, start: Axis, stop: Axis, n: usize) {
    for i in 0..t.node_num(n) {
        let ci = t.child(n, i).unwrap();
        tree_overlap_traverse_self_cb(t, ctx, start, stop, ci);
        for j in i + 1..t.node_num(n) {
            let cj = t.child(n, j).unwrap();
            tree_overlap_traverse_cb(t, t, true, ctx, start, stop, ci, cj);
        }
    }
}
fn tree_overlap_traverse_self(t: &BVHTree, ctx: &mut OverlapCtx<'_, '_>, start: Axis, stop: Axis, n: usize) {
    for i in 0..t.node_num(n) {
        let ci = t.child(n, i).unwrap();
        tree_overlap_traverse_self(t, ctx, start, stop, ci);
        for j in i + 1..t.node_num(n) {
            let cj = t.child(n, j).unwrap();
            tree_overlap_traverse(t, t, true, ctx, start, stop, ci, cj);
        }
    }
}
fn tree_overlap_invoke_traverse_self(
    t: &BVHTree,
    ctx: &mut OverlapCtx<'_, '_>,
    start: Axis,
    stop: Axis,
    n: usize,
) {
    if ctx.callback.is_some() {
        tree_overlap_traverse_self_cb(t, ctx, start, stop, n);
    } else {
        tree_overlap_traverse_self(t, ctx, start, stop, n);
    }
}

/// `BLI_bvhtree_overlap_thread_num`.
pub fn bvhtree_overlap_thread_num(tree: &BVHTree) -> i32 {
    (tree.tree_type as i32).min(tree.node_num(tree.nodes[tree.leaf_num] as usize) as i32)
}

/// `BLI_bvhtree_overlap_ex` — serial port (threading dropped).
pub fn bvhtree_overlap_ex(
    tree1: &BVHTree,
    tree2: &BVHTree,
    callback: OverlapCallback,
    max_interactions: u32,
    flag: i32,
) -> Vec<BVHTreeOverlap> {
    let overlap_pairs = flag & BVH_OVERLAP_RETURN_PAIRS != 0;
    let mut use_self = flag & BVH_OVERLAP_SELF != 0;
    debug_assert!(overlap_pairs || max_interactions > 0);
    debug_assert!(!use_self || (std::ptr::eq(tree1, tree2) && max_interactions == 0));

    if (tree1.axis != tree2.axis)
        && (tree1.axis == 14 || tree2.axis == 14)
        && (tree1.axis == 18 || tree2.axis == 18)
    {
        panic!("incompatible kdop types");
    }
    if use_self && !std::ptr::eq(tree1, tree2) {
        use_self = false;
    }

    let root1 = tree1.nodes[tree1.leaf_num] as usize;
    let root2 = tree2.nodes[tree2.leaf_num] as usize;
    let start = tree1.start_axis.min(tree2.start_axis);
    let stop = tree1.stop_axis.min(tree2.stop_axis);

    if !tree_overlap_test(tree1.bv(root1), tree2.bv(root2), start, stop) {
        return Vec::new();
    }

    let mut overlap_store = Vec::new();
    {
        let mut ctx = OverlapCtx {
            overlaps: if overlap_pairs { Some(&mut overlap_store) } else { None },
            callback,
            max_interactions: if use_self { 0 } else { max_interactions },
            thread: 0,
        };
        if use_self {
            tree_overlap_invoke_traverse_self(tree1, &mut ctx, start, stop, root1);
        } else {
            tree_overlap_invoke_traverse(tree1, tree2, false, &mut ctx, start, stop, root1, root2);
        }
    }
    overlap_store
}

/// `BLI_bvhtree_overlap`.
pub fn bvhtree_overlap(
    tree1: &BVHTree,
    tree2: &BVHTree,
    callback: OverlapCallback,
) -> Vec<BVHTreeOverlap> {
    bvhtree_overlap_ex(
        tree1,
        tree2,
        callback,
        0,
        BVH_OVERLAP_USE_THREADING | BVH_OVERLAP_RETURN_PAIRS,
    )
}
/// `BLI_bvhtree_overlap_self`.
pub fn bvhtree_overlap_self(tree: &BVHTree, callback: OverlapCallback) -> Vec<BVHTreeOverlap> {
    bvhtree_overlap_ex(
        tree,
        tree,
        callback,
        0,
        BVH_OVERLAP_USE_THREADING | BVH_OVERLAP_RETURN_PAIRS | BVH_OVERLAP_SELF,
    )
}

/* Intersect plane. */

fn tree_intersect_plane_test(bv: &[f32], plane: [f32; 4]) -> bool {
    let bb_min = Vec3::new(bv[0], bv[2], bv[4]);
    let bb_max = Vec3::new(bv[1], bv[3], bv[5]);
    let plane_v = Vec4::new(plane[0], plane[1], plane[2], plane[3]);
    let (bb_near, bb_far) = aabb_get_near_far_from_plane(plane_v.xyz(), bb_min, bb_max);
    (plane_point_side(plane_v, bb_near) > 0.0) != (plane_point_side(plane_v, bb_far) > 0.0)
}

fn bvhtree_intersect_plane_dfs(tree: &BVHTree, plane: [f32; 4], n: usize, out: &mut Vec<i32>) {
    if tree_intersect_plane_test(tree.bv(n), plane) {
        if tree.node_num(n) == 0 {
            out.push(tree.node_index(n));
        } else {
            for j in 0..tree.tree_type as usize {
                if let Some(c) = tree.child(n, j) {
                    bvhtree_intersect_plane_dfs(tree, plane, c, out);
                }
            }
        }
    }
}

/// `BLI_bvhtree_intersect_plane`.
pub fn bvhtree_intersect_plane(tree: &BVHTree, plane: [f32; 4]) -> Vec<i32> {
    let mut out = Vec::new();
    if tree.leaf_num > 0 {
        let root = tree.nodes[tree.leaf_num] as usize;
        bvhtree_intersect_plane_dfs(tree, plane, root, &mut out);
    }
    out
}

/* Find nearest. */

fn calc_nearest_point_squared(proj: &[f32; 3], bv: &[f32]) -> (f32, [f32; 3]) {
    let mut nearest = [0.0f32; 3];
    for i in 0..3 {
        let mut val = proj[i];
        val = val.max(bv[2 * i]);
        val = val.min(bv[2 * i + 1]);
        nearest[i] = val;
    }
    let d = [
        proj[0] - nearest[0],
        proj[1] - nearest[1],
        proj[2] - nearest[2],
    ];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2], nearest)
}

type NearestCallback<'a> = Option<&'a mut dyn FnMut(i32, &[f32; 3], &mut BVHTreeNearest)>;

struct NearestCtx<'a> {
    proj: [f32; 13],
    co: [f32; 3],
    callback: NearestCallback<'a>,
    nearest: BVHTreeNearest,
}

fn dfs_find_nearest_dfs(tree: &BVHTree, ctx: &mut NearestCtx, n: usize) {
    if tree.node_num(n) == 0 {
        if let Some(cb) = ctx.callback.as_deref_mut() {
            cb(tree.node_index(n), &ctx.co, &mut ctx.nearest);
        } else {
            ctx.nearest.index = tree.node_index(n);
            let (d, co) = calc_nearest_point_squared(&ctx.proj[..3].try_into().unwrap(), tree.bv(n));
            ctx.nearest.dist_sq = d;
            ctx.nearest.co = co;
        }
    } else {
        let main = tree.nodearray[n].main_axis as usize;
        let c0 = tree.child(n, 0).unwrap();
        if ctx.proj[main] <= tree.bv(c0)[main * 2 + 1] {
            for i in 0..tree.node_num(n) {
                let c = tree.child(n, i).unwrap();
                let (d, _) = calc_nearest_point_squared(&ctx.proj[..3].try_into().unwrap(), tree.bv(c));
                if d >= ctx.nearest.dist_sq {
                    continue;
                }
                dfs_find_nearest_dfs(tree, ctx, c);
            }
        } else {
            for i in (0..tree.node_num(n)).rev() {
                let c = tree.child(n, i).unwrap();
                let (d, _) = calc_nearest_point_squared(&ctx.proj[..3].try_into().unwrap(), tree.bv(c));
                if d >= ctx.nearest.dist_sq {
                    continue;
                }
                dfs_find_nearest_dfs(tree, ctx, c);
            }
        }
    }
}

fn dfs_find_nearest_begin(tree: &BVHTree, ctx: &mut NearestCtx, n: usize) {
    let (d, _) = calc_nearest_point_squared(&ctx.proj[..3].try_into().unwrap(), tree.bv(n));
    if d >= ctx.nearest.dist_sq {
        return;
    }
    dfs_find_nearest_dfs(tree, ctx, n);
}

/* Priority-queue variant: use std BinaryHeap of (dist_sq, node). */
fn heap_find_nearest_begin(tree: &BVHTree, ctx: &mut NearestCtx, root: usize) {
    use std::cmp::Ordering;
    use std::collections::BinaryHeap;
    struct Entry(f32, usize);
    impl PartialEq for Entry {
        fn eq(&self, o: &Self) -> bool {
            self.0 == o.0
        }
    }
    impl Eq for Entry {}
    impl PartialOrd for Entry {
        fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
            Some(self.cmp(o))
        }
    }
    impl Ord for Entry {
        fn cmp(&self, o: &Self) -> Ordering {
            o.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
        }
    }

    let proj3: &[f32; 3] = ctx.proj[..3].try_into().unwrap();
    let (d, _) = calc_nearest_point_squared(proj3, tree.bv(root));
    if d >= ctx.nearest.dist_sq {
        return;
    }
    let mut heap = BinaryHeap::with_capacity(32);
    fn inner(tree: &BVHTree, ctx: &mut NearestCtx, heap: &mut BinaryHeap<Entry>, n: usize) {
        if tree.node_num(n) == 0 {
            if let Some(cb) = ctx.callback.as_deref_mut() {
                cb(tree.node_index(n), &ctx.co, &mut ctx.nearest);
            } else {
                ctx.nearest.index = tree.node_index(n);
                let proj3: &[f32; 3] = ctx.proj[..3].try_into().unwrap();
                let (d, co) = calc_nearest_point_squared(proj3, tree.bv(n));
                ctx.nearest.dist_sq = d;
                ctx.nearest.co = co;
            }
        } else {
            let proj3: &[f32; 3] = ctx.proj[..3].try_into().unwrap();
            for i in 0..tree.node_num(n) {
                let c = tree.child(n, i).unwrap();
                let (d, _) = calc_nearest_point_squared(proj3, tree.bv(c));
                if d < ctx.nearest.dist_sq {
                    heap.push(Entry(d, c));
                }
            }
        }
    }
    inner(tree, ctx, &mut heap, root);
    while let Some(Entry(d, n)) = heap.pop() {
        if d >= ctx.nearest.dist_sq {
            break;
        }
        inner(tree, ctx, &mut heap, n);
    }
}

/// `BLI_bvhtree_find_nearest_ex`.
pub fn bvhtree_find_nearest_ex(
    tree: &BVHTree,
    co: [f32; 3],
    nearest: Option<BVHTreeNearest>,
    callback: NearestCallback,
    flag: i32,
) -> (i32, BVHTreeNearest) {
    let root = tree.nodes[tree.leaf_num];
    let mut ctx = NearestCtx {
        proj: [0.0; 13],
        co,
        callback,
        nearest: nearest.unwrap_or(BVHTreeNearest {
            index: -1,
            dist_sq: f32::MAX,
            co: [0.0; 3],
        }),
    };
    for a in tree.start_axis..tree.stop_axis {
        let ax = &BVHTREE_KDOP_AXES[a as usize];
        ctx.proj[a as usize] = co[0] * ax[0] + co[1] * ax[1] + co[2] * ax[2];
    }
    if root >= 0 {
        if flag & BVH_NEAREST_OPTIMAL_ORDER != 0 {
            heap_find_nearest_begin(tree, &mut ctx, root as usize);
        } else {
            dfs_find_nearest_begin(tree, &mut ctx, root as usize);
        }
    }
    (ctx.nearest.index, ctx.nearest)
}
/// `BLI_bvhtree_find_nearest`.
pub fn bvhtree_find_nearest(
    tree: &BVHTree,
    co: [f32; 3],
    nearest: Option<BVHTreeNearest>,
    callback: NearestCallback,
) -> (i32, BVHTreeNearest) {
    bvhtree_find_nearest_ex(tree, co, nearest, callback, 0)
}

/* Find nearest first (duplicate detection). */

fn isect_aabb_v3(tree: &BVHTree, n: usize, co: &[f32; 3]) -> bool {
    let bv = tree.bv(n);
    co[0] > bv[0] && co[0] < bv[1] && co[1] > bv[2] && co[1] < bv[3] && co[2] > bv[4] && co[2] < bv[5]
}

fn dfs_find_duplicate_fast_dfs(tree: &BVHTree, ctx: &mut NearestCtx, n: usize) -> bool {
    if tree.node_num(n) == 0 {
        if isect_aabb_v3(tree, n, &ctx.co) {
            if let Some(cb) = ctx.callback.as_deref_mut() {
                let d = ctx.nearest.dist_sq;
                cb(tree.node_index(n), &ctx.co, &mut ctx.nearest);
                return ctx.nearest.dist_sq < d;
            }
            ctx.nearest.index = tree.node_index(n);
            return true;
        }
    } else {
        let main = tree.nodearray[n].main_axis as usize;
        let c0 = tree.child(n, 0).unwrap();
        if ctx.proj[main] <= tree.bv(c0)[main * 2 + 1] {
            for i in 0..tree.node_num(n) {
                let c = tree.child(n, i).unwrap();
                if isect_aabb_v3(tree, c, &ctx.co) && dfs_find_duplicate_fast_dfs(tree, ctx, c) {
                    return true;
                }
            }
        } else {
            for i in (0..tree.node_num(n)).rev() {
                let c = tree.child(n, i).unwrap();
                if isect_aabb_v3(tree, c, &ctx.co) && dfs_find_duplicate_fast_dfs(tree, ctx, c) {
                    return true;
                }
            }
        }
    }
    false
}

/// `BLI_bvhtree_find_nearest_first`.
pub fn bvhtree_find_nearest_first(
    tree: &BVHTree,
    co: [f32; 3],
    dist_sq: f32,
    callback: NearestCallback,
) -> i32 {
    let root = tree.nodes[tree.leaf_num];
    let mut ctx = NearestCtx {
        proj: [0.0; 13],
        co,
        callback,
        nearest: BVHTreeNearest {
            index: -1,
            dist_sq,
            co: [0.0; 3],
        },
    };
    for a in tree.start_axis..tree.stop_axis {
        let ax = &BVHTREE_KDOP_AXES[a as usize];
        ctx.proj[a as usize] = co[0] * ax[0] + co[1] * ax[1] + co[2] * ax[2];
    }
    if root >= 0 {
        dfs_find_duplicate_fast_dfs(tree, &mut ctx, root as usize);
    }
    ctx.nearest.index
}

/* Ray cast. */

fn ray_nearest_hit(ray: &BVHTreeRay, hit_dist: f32, ray_dot_axis: &[f32; 13], bv: &[f32]) -> f32 {
    let mut low = 0.0f32;
    let mut upper = hit_dist;
    for i in 0..3 {
        let b = &bv[2 * i..2 * i + 2];
        if ray_dot_axis[i] == 0.0 {
            if ray.origin[i] < b[0] - ray.radius || ray.origin[i] > b[1] + ray.radius {
                return f32::MAX;
            }
        } else {
            let ll = (b[0] - ray.radius - ray.origin[i]) / ray_dot_axis[i];
            let lu = (b[1] + ray.radius - ray.origin[i]) / ray_dot_axis[i];
            if ray_dot_axis[i] > 0.0 {
                low = low.max(ll);
                upper = upper.min(lu);
            } else {
                low = low.max(lu);
                upper = upper.min(ll);
            }
            if low > upper {
                return f32::MAX;
            }
        }
    }
    low
}

fn fast_ray_nearest_hit(
    ray: &BVHTreeRay,
    hit_dist: f32,
    idot_axis: &[f32; 13],
    index: &[usize; 6],
    bv: &[f32],
) -> f32 {
    let t1x = (bv[index[0]] - ray.origin[0]) * idot_axis[0];
    let t2x = (bv[index[1]] - ray.origin[0]) * idot_axis[0];
    let t1y = (bv[index[2]] - ray.origin[1]) * idot_axis[1];
    let t2y = (bv[index[3]] - ray.origin[1]) * idot_axis[1];
    let t1z = (bv[index[4]] - ray.origin[2]) * idot_axis[2];
    let t2z = (bv[index[5]] - ray.origin[2]) * idot_axis[2];

    if (t1x > t2y || t2x < t1y || t1x > t2z || t2x < t1z || t1y > t2z || t2y < t1z)
        || (t2x < 0.0 || t2y < 0.0 || t2z < 0.0)
        || (t1x > hit_dist || t1y > hit_dist || t1z > hit_dist)
    {
        return f32::MAX;
    }
    t1x.max(t1y).max(t1z)
}

type RayCastCallback<'a> = Option<&'a mut dyn FnMut(i32, &BVHTreeRay, &mut BVHTreeRayHit)>;

struct RayCastCtx<'a> {
    ray: BVHTreeRay,
    callback: RayCastCallback<'a>,
    ray_dot_axis: [f32; 13],
    idot_axis: [f32; 13],
    index: [usize; 6],
    hit: BVHTreeRayHit,
}

fn raycast_dist(tree: &BVHTree, ctx: &RayCastCtx, n: usize) -> f32 {
    if ctx.ray.radius == 0.0 {
        fast_ray_nearest_hit(&ctx.ray, ctx.hit.dist, &ctx.idot_axis, &ctx.index, tree.bv(n))
    } else {
        ray_nearest_hit(&ctx.ray, ctx.hit.dist, &ctx.ray_dot_axis, tree.bv(n))
    }
}

fn dfs_raycast(tree: &BVHTree, ctx: &mut RayCastCtx, n: usize) {
    let dist = raycast_dist(tree, ctx, n);
    if dist >= ctx.hit.dist {
        return;
    }
    if tree.node_num(n) == 0 {
        if let Some(cb) = ctx.callback.as_deref_mut() {
            cb(tree.node_index(n), &ctx.ray, &mut ctx.hit);
        } else {
            ctx.hit.index = tree.node_index(n);
            ctx.hit.dist = dist;
            for i in 0..3 {
                ctx.hit.co[i] = ctx.ray.origin[i] + ctx.ray.direction[i] * dist;
            }
        }
    } else {
        let main = tree.nodearray[n].main_axis as usize;
        if ctx.ray_dot_axis[main] > 0.0 {
            for i in 0..tree.node_num(n) {
                let c = tree.child(n, i).unwrap();
                dfs_raycast(tree, ctx, c);
            }
        } else {
            for i in (0..tree.node_num(n)).rev() {
                let c = tree.child(n, i).unwrap();
                dfs_raycast(tree, ctx, c);
            }
        }
    }
}

fn dfs_raycast_all(tree: &BVHTree, ctx: &mut RayCastCtx, n: usize) {
    let dist = raycast_dist(tree, ctx, n);
    if dist >= ctx.hit.dist {
        return;
    }
    if tree.node_num(n) == 0 {
        let cb = ctx.callback.as_deref_mut().expect("raycast_all needs callback");
        let dist = ctx.hit.dist;
        cb(tree.node_index(n), &ctx.ray, &mut ctx.hit);
        ctx.hit.index = -1;
        ctx.hit.dist = dist;
    } else {
        let main = tree.nodearray[n].main_axis as usize;
        if ctx.ray_dot_axis[main] > 0.0 {
            for i in 0..tree.node_num(n) {
                let c = tree.child(n, i).unwrap();
                dfs_raycast_all(tree, ctx, c);
            }
        } else {
            for i in (0..tree.node_num(n)).rev() {
                let c = tree.child(n, i).unwrap();
                dfs_raycast_all(tree, ctx, c);
            }
        }
    }
}

fn bvhtree_ray_cast_data_precalc(ctx: &mut RayCastCtx, flag: i32) {
    for i in 0..3 {
        let ax = &BVHTREE_KDOP_AXES[i];
        ctx.ray_dot_axis[i] = ctx.ray.direction[0] * ax[0]
            + ctx.ray.direction[1] * ax[1]
            + ctx.ray.direction[2] * ax[2];
        if ctx.ray_dot_axis[i].abs() < f32::EPSILON {
            ctx.ray_dot_axis[i] = 0.0;
            ctx.idot_axis[i] = f32::MAX;
        } else {
            ctx.idot_axis[i] = 1.0 / ctx.ray_dot_axis[i];
        }
        ctx.index[2 * i] = if ctx.idot_axis[i] < 0.0 { 1 } else { 0 };
        ctx.index[2 * i + 1] = 1 - ctx.index[2 * i];
        ctx.index[2 * i] += 2 * i;
        ctx.index[2 * i + 1] += 2 * i;
    }
    if flag & BVH_RAYCAST_WATERTIGHT != 0 {
        ctx.ray.isect_precalc = Some(isect_ray_tri_watertight_precalc(Vec3::from(
            ctx.ray.direction,
        )));
    } else {
        ctx.ray.isect_precalc = None;
    }
}

/// `BLI_bvhtree_ray_cast_ex`.
pub fn bvhtree_ray_cast_ex(
    tree: &BVHTree,
    co: [f32; 3],
    dir: [f32; 3],
    radius: f32,
    hit: Option<BVHTreeRayHit>,
    callback: RayCastCallback,
    flag: i32,
) -> (i32, BVHTreeRayHit) {
    let root = tree.nodes[tree.leaf_num];
    let mut ctx = RayCastCtx {
        ray: BVHTreeRay {
            origin: co,
            direction: dir,
            radius,
            isect_precalc: None,
        },
        callback,
        ray_dot_axis: [0.0; 13],
        idot_axis: [0.0; 13],
        index: [0; 6],
        hit: hit.unwrap_or_default(),
    };
    bvhtree_ray_cast_data_precalc(&mut ctx, flag);
    if root >= 0 {
        dfs_raycast(tree, &mut ctx, root as usize);
    }
    (ctx.hit.index, ctx.hit)
}
/// `BLI_bvhtree_ray_cast`.
pub fn bvhtree_ray_cast(
    tree: &BVHTree,
    co: [f32; 3],
    dir: [f32; 3],
    radius: f32,
    hit: Option<BVHTreeRayHit>,
    callback: RayCastCallback,
) -> (i32, BVHTreeRayHit) {
    bvhtree_ray_cast_ex(tree, co, dir, radius, hit, callback, BVH_RAYCAST_DEFAULT)
}

/// `BLI_bvhtree_bb_raycast`.
pub fn bvhtree_bb_raycast(bv: &[f32; 6], light_start: [f32; 3], light_end: [f32; 3]) -> (f32, [f32; 3]) {
    let mut dir = [
        light_end[0] - light_start[0],
        light_end[1] - light_start[1],
        light_end[2] - light_start[2],
    ];
    normalize_v3(&mut dir);
    let ray = BVHTreeRay {
        origin: light_start,
        direction: dir,
        radius: 0.0,
        isect_precalc: None,
    };
    let mut rda = [0.0f32; 13];
    rda[..3].copy_from_slice(&dir);
    let dist = ray_nearest_hit(&ray, BVH_RAYCAST_DIST_MAX, &rda, bv);
    let pos = [
        light_start[0] + dir[0] * dist,
        light_start[1] + dir[1] * dist,
        light_start[2] + dir[2] * dist,
    ];
    (dist, pos)
}

/// `BLI_bvhtree_ray_cast_all_ex`.
pub fn bvhtree_ray_cast_all_ex(
    tree: &BVHTree,
    co: [f32; 3],
    dir: [f32; 3],
    radius: f32,
    hit_dist: f32,
    callback: RayCastCallback,
    flag: i32,
) {
    let root = tree.nodes[tree.leaf_num];
    assert!(callback.is_some());
    let mut ctx = RayCastCtx {
        ray: BVHTreeRay {
            origin: co,
            direction: dir,
            radius,
            isect_precalc: None,
        },
        callback,
        ray_dot_axis: [0.0; 13],
        idot_axis: [0.0; 13],
        index: [0; 6],
        hit: BVHTreeRayHit {
            index: -1,
            dist: hit_dist,
            co: [0.0; 3],
            no: [0.0; 3],
        },
    };
    bvhtree_ray_cast_data_precalc(&mut ctx, flag);
    if root >= 0 {
        dfs_raycast_all(tree, &mut ctx, root as usize);
    }
}
/// `BLI_bvhtree_ray_cast_all`.
pub fn bvhtree_ray_cast_all(
    tree: &BVHTree,
    co: [f32; 3],
    dir: [f32; 3],
    radius: f32,
    hit_dist: f32,
    callback: RayCastCallback,
) {
    bvhtree_ray_cast_all_ex(tree, co, dir, radius, hit_dist, callback, BVH_RAYCAST_DEFAULT)
}

/* Range query. */

fn dfs_range_query(
    tree: &BVHTree,
    center: &[f32; 3],
    radius_sq: f32,
    cb: &mut dyn FnMut(i32, &[f32; 3], f32),
    hits: &mut i32,
    n: usize,
) {
    if tree.node_num(n) == 0 {
        /* leaf handled by parent */
    } else {
        for i in 0..tree.node_num(n) {
            let c = tree.child(n, i).unwrap();
            let (d, _) = calc_nearest_point_squared(center, tree.bv(c));
            if d < radius_sq {
                if tree.node_num(c) == 0 {
                    *hits += 1;
                    cb(tree.node_index(c), center, d);
                } else {
                    dfs_range_query(tree, center, radius_sq, cb, hits, c);
                }
            }
        }
    }
}

/// `BLI_bvhtree_range_query`.
pub fn bvhtree_range_query(
    tree: &BVHTree,
    co: [f32; 3],
    radius: f32,
    callback: &mut dyn FnMut(i32, &[f32; 3], f32),
) -> i32 {
    let root = tree.nodes[tree.leaf_num];
    let radius_sq = radius * radius;
    let mut hits = 0;
    if root >= 0 {
        let root = root as usize;
        let (d, _) = calc_nearest_point_squared(&co, tree.bv(root));
        if d < radius_sq {
            if tree.node_num(root) == 0 {
                hits += 1;
                callback(tree.node_index(root), &co, d);
            } else {
                dfs_range_query(tree, &co, radius_sq, callback, &mut hits, root);
            }
        }
    }
    hits
}

/* Nearest projected. */

type NearestProjectedCallback<'a> = Option<
    &'a mut dyn FnMut(i32, &DistProjectedAabbPrecalc, &[[f32; 4]], &mut BVHTreeNearest),
>;

struct NearestProjectedCtx<'a> {
    precalc: DistProjectedAabbPrecalc,
    closest_axis: [bool; 3],
    callback: NearestProjectedCallback<'a>,
    nearest: BVHTreeNearest,
    clip_plane: Vec<[f32; 4]>,
}

fn node_dist_projected(ctx: &NearestProjectedCtx, bv: &[f32]) -> f32 {
    dist_squared_to_projected_aabb(
        &ctx.precalc,
        Vec3::new(bv[0], bv[2], bv[4]),
        Vec3::new(bv[1], bv[3], bv[5]),
    )
    .0
}

fn projected_leaf(tree: &BVHTree, ctx: &mut NearestProjectedCtx, n: usize) {
    if let Some(cb) = ctx.callback.as_deref_mut() {
        cb(
            tree.node_index(n),
            &ctx.precalc,
            &ctx.clip_plane,
            &mut ctx.nearest,
        );
    } else {
        ctx.nearest.index = tree.node_index(n);
        ctx.nearest.dist_sq = node_dist_projected(ctx, tree.bv(n));
    }
}

fn bvhtree_nearest_projected_dfs(tree: &BVHTree, ctx: &mut NearestProjectedCtx, n: usize) {
    if tree.node_num(n) == 0 {
        projected_leaf(tree, ctx, n);
    } else {
        let main = tree.nodearray[n].main_axis as usize;
        if ctx.closest_axis[main] {
            for i in 0..tree.node_num(n) {
                let c = tree.child(n, i).unwrap();
                if node_dist_projected(ctx, tree.bv(c)) <= ctx.nearest.dist_sq {
                    bvhtree_nearest_projected_dfs(tree, ctx, c);
                }
            }
        } else {
            for i in (0..tree.node_num(n)).rev() {
                let c = tree.child(n, i).unwrap();
                if node_dist_projected(ctx, tree.bv(c)) <= ctx.nearest.dist_sq {
                    bvhtree_nearest_projected_dfs(tree, ctx, c);
                }
            }
        }
    }
}

fn bvhtree_nearest_projected_clip_dfs(tree: &BVHTree, ctx: &mut NearestProjectedCtx, n: usize) {
    if tree.node_num(n) == 0 {
        projected_leaf(tree, ctx, n);
    } else {
        let main = tree.nodearray[n].main_axis as usize;
        let order: Vec<usize> = if ctx.closest_axis[main] {
            (0..tree.node_num(n)).collect()
        } else {
            (0..tree.node_num(n)).rev().collect()
        };
        for i in order {
            let c = tree.child(n, i).unwrap();
            let bv = tree.bv(c);
            let bb_min = Vec3::new(bv[0], bv[2], bv[4]);
            let bb_max = Vec3::new(bv[1], bv[3], bv[5]);
            let planes: Vec<Vec4> = ctx.clip_plane.iter().map(|p| Vec4::new(p[0], p[1], p[2], p[3])).collect();
            let isect = isect_aabb_planes(&planes, bb_min, bb_max);
            if isect != AabbPlaneIsect::BehindAny
                && node_dist_projected(ctx, bv) <= ctx.nearest.dist_sq
            {
                if isect == AabbPlaneIsect::CrossAny {
                    bvhtree_nearest_projected_clip_dfs(tree, ctx, c);
                } else {
                    bvhtree_nearest_projected_dfs(tree, ctx, c);
                }
            }
        }
    }
}

/// `BLI_bvhtree_find_nearest_projected`.
pub fn bvhtree_find_nearest_projected(
    tree: &BVHTree,
    projmat: &Mat4,
    winsize: [f32; 2],
    mval: [f32; 2],
    clip_plane: Option<&[[f32; 4]]>,
    nearest: Option<BVHTreeNearest>,
    callback: NearestProjectedCallback,
) -> (i32, BVHTreeNearest) {
    let root = tree.nodes[tree.leaf_num];
    if root < 0 {
        return (
            -1,
            nearest.unwrap_or(BVHTreeNearest {
                index: -1,
                dist_sq: f32::MAX,
                co: [0.0; 3],
            }),
        );
    }
    let root = root as usize;
    let precalc = dist_squared_to_projected_aabb_precalc(
        projmat,
        Vec2::new(winsize[0], winsize[1]),
        Vec2::new(mval[0], mval[1]),
    );
    let planes_all = planes_from_projmat(projmat);
    let clip: Vec<[f32; 4]> = match clip_plane {
        Some(cp) => cp.to_vec(),
        None => vec![[
            planes_all[4][0],
            planes_all[4][1],
            planes_all[4][2],
            planes_all[4][3],
        ]],
    };
    let mut ctx = NearestProjectedCtx {
        precalc,
        closest_axis: [false; 3],
        callback,
        nearest: nearest.unwrap_or(BVHTreeNearest {
            index: -1,
            dist_sq: f32::MAX,
            co: [0.0; 3],
        }),
        clip_plane: clip,
    };
    {
        let bv = tree.bv(root);
        let bb_min = Vec3::new(bv[0], bv[2], bv[4]);
        let bb_max = Vec3::new(bv[1], bv[3], bv[5]);
        let planes: Vec<Vec4> = ctx.clip_plane.iter().map(|p| Vec4::new(p[0], p[1], p[2], p[3])).collect();
        let isect = isect_aabb_planes(&planes, bb_min, bb_max);
        if isect != AabbPlaneIsect::BehindAny {
            let (d, axis) =
                dist_squared_to_projected_aabb(&ctx.precalc, bb_min, bb_max);
            ctx.closest_axis = axis;
            if d <= ctx.nearest.dist_sq {
                if isect == AabbPlaneIsect::CrossAny {
                    bvhtree_nearest_projected_clip_dfs(tree, &mut ctx, root);
                } else {
                    bvhtree_nearest_projected_dfs(tree, &mut ctx, root);
                }
            }
        }
    }
    (ctx.nearest.index, ctx.nearest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_tree(points: &[[f32; 3]]) -> BVHTree {
        let mut t = bvhtree_new(points.len() as i32, 0.0, 2, 6);
        for (i, p) in points.iter().enumerate() {
            bvhtree_insert(&mut t, i as i32, p, 1);
        }
        bvhtree_balance(&mut t);
        t
    }

    #[test]
    fn nearest_and_range() {
        let pts: Vec<[f32; 3]> = (0..64)
            .map(|i| [i as f32, (i % 8) as f32, (i % 3) as f32])
            .collect();
        let t = build_tree(&pts);
        let co = [10.2, 0.1, 0.0];
        let (idx, nearest) = bvhtree_find_nearest(&t, co, None, None);
        assert_eq!(idx, 9);
        assert_eq!(nearest.index, 9);
        let mut hits = Vec::new();
        let n = bvhtree_range_query(&t, [5.0, 5.0, 1.0], 2.0, &mut |i, _, d| {
            hits.push((i, d));
        });
        assert_eq!(n, hits.len() as i32);
        assert!(hits.iter().any(|h| h.0 == 5));
    }

    #[test]
    fn raycast_and_overlap_and_plane() {
        let mut pts = Vec::new();
        for i in 0..27 {
            pts.push([(i % 3) as f32, ((i / 3) % 3) as f32, (i / 9) as f32]);
        }
        let t = build_tree(&pts);
        let (idx, hit) = bvhtree_ray_cast(&t, [0.5, 0.5, -5.0], [0.0, 0.0, 1.0], 0.6, None, None);
        /* radius-inflated slab entry: dist = 5 - radius, first z=0 leaf in DFS order */
        assert!(idx >= 0);
        assert!((hit.dist - 4.4).abs() < 1e-4);

        let t2 = build_tree(&pts);
        let ov = bvhtree_overlap(&t, &t2, None);
        /* identical trees: every leaf pair overlapping counts twice + shared points */
        assert!(ov.len() >= 27);

        let isect = bvhtree_intersect_plane(&t, [0.0, 0.0, 1.0, -1.0]);
        assert_eq!(isect.len(), 9);
    }
}
