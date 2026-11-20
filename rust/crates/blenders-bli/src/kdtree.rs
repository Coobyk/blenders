//! Port of `kdtree.cc` + `BLI_kdtree_new.hh` — `KDTree<T>` nearest-
//! neighbor tree (float2/3/4 → `KDTree<D>` over `[[f32; D]]`).
//! PORT_NOTE: TBB parallel build → serial; node arena → `Vec`.

pub const LEAF_SIZE_DEFAULT: usize = 16;

#[derive(Clone, Copy, Default)]
struct Node {
    /// Two children as indices into `nodes`, `u32::MAX` for a leaf.
    children: u32,
    leaf: (i32, i32),
    split: (f32, f32),
    dim: i32,
}

#[derive(Clone, Copy)]
pub struct Nearest {
    pub index: i32,
    pub distance_sq: f32,
}

pub struct KDTree<const D: usize> {
    nodes: Vec<Node>,
    tree_indices: Vec<i32>,
    bounds_min: [f32; D],
    bounds_max: [f32; D],
}

fn dist_sq<const D: usize>(a: &[f32; D], b: &[f32; D]) -> f32 {
    let mut s = 0.0;
    for i in 0..D {
        s += (a[i] - b[i]) * (a[i] - b[i]);
    }
    s
}

/// Query abstraction matching `kdtree::*Query` (`inclusive` = radius
/// queries use <=, nearest use <).
trait Query {
    const INCLUSIVE: bool;
    fn add_point(&mut self, distance_sq: f32, index: i32) -> bool;
    fn max_distance_sq(&self) -> f32;
}

fn keep_distance<Q: Query>(d: f32, max: f32) -> bool {
    if Q::INCLUSIVE { d <= max } else { d < max }
}

struct NearestQuery {
    index: i32,
    distance_sq: f32,
}
impl Query for NearestQuery {
    const INCLUSIVE: bool = false;
    fn add_point(&mut self, d: f32, i: i32) -> bool {
        self.distance_sq = d;
        self.index = i;
        true
    }
    fn max_distance_sq(&self) -> f32 {
        self.distance_sq
    }
}

struct FilteredNearestQuery<'a, F: Fn(i32) -> bool> {
    filter: &'a F,
    index: i32,
    distance_sq: f32,
}
impl<F: Fn(i32) -> bool> Query for FilteredNearestQuery<'_, F> {
    const INCLUSIVE: bool = false;
    fn add_point(&mut self, d: f32, i: i32) -> bool {
        if (self.filter)(i) {
            self.distance_sq = d;
            self.index = i;
        }
        true
    }
    fn max_distance_sq(&self) -> f32 {
        self.distance_sq
    }
}

struct NearestNQuery<'a> {
    nearest: &'a mut [Nearest],
    distance_sq: f32,
    count: usize,
}
impl Query for NearestNQuery<'_> {
    const INCLUSIVE: bool = false;
    fn add_point(&mut self, d: f32, index: i32) -> bool {
        let capacity = self.nearest.len();
        let mut i = self.count;
        while i > 0 {
            if self.nearest[i - 1].distance_sq <= d {
                break;
            }
            if i < capacity {
                self.nearest[i] = self.nearest[i - 1];
            }
            i -= 1;
        }
        if i < capacity {
            self.nearest[i] = Nearest {
                index,
                distance_sq: d,
            };
        }
        if self.count < capacity {
            self.count += 1;
        }
        true
    }
    fn max_distance_sq(&self) -> f32 {
        if self.count < self.nearest.len() {
            self.distance_sq
        } else {
            self.nearest[self.count - 1].distance_sq
        }
    }
}

struct RadiusQuery<'a, F: FnMut(i32, f32)> {
    radius_sq: f32,
    f: &'a mut F,
    count: usize,
}
impl<F: FnMut(i32, f32)> Query for RadiusQuery<'_, F> {
    const INCLUSIVE: bool = true;
    fn add_point(&mut self, d: f32, i: i32) -> bool {
        self.count += 1;
        (self.f)(i, d);
        true
    }
    fn max_distance_sq(&self) -> f32 {
        self.radius_sq
    }
}

// BuildPoint: D floats + bit-cast index.
type BuildPoint<const D: usize> = [f32; D];

impl<const D: usize> KDTree<D> {
    /// `KDTree(points, leaf_size)`.
    pub fn new(points: &[[f32; D]], leaf_size: usize) -> Self {
        Self::build(points, None, leaf_size.max(1))
    }
    /// `KDTree(points, indices, leaf_size)`.
    pub fn with_indices(points: &[[f32; D]], indices: &[i32], leaf_size: usize) -> Self {
        Self::build_subset(points, indices, leaf_size.max(1))
    }

    fn build(points: &[[f32; D]], indices: Option<&[i32]>, leaf_size: usize) -> Self {
        match indices {
            Some(ix) => Self::build_subset(points, ix, leaf_size),
            None => {
                let all: Vec<i32> = (0..points.len() as i32).collect();
                Self::build_subset(points, &all, leaf_size)
            }
        }
    }

    fn build_subset(points: &[[f32; D]], indices: &[i32], leaf_size: usize) -> Self {
        // Same as build but source indices come from `indices`.
        let n = indices.len();
        let mut order: Vec<u32> = (0..n as u32).collect();
        let mut pts: Vec<[f32; D]> = indices.iter().map(|&i| points[i as usize]).collect();
        let mut nodes: Vec<Node> = Vec::new();
        let mut bounds_min = [f32::INFINITY; D];
        let mut bounds_max = [f32::NEG_INFINITY; D];
        if n > 0 {
            for d in 0..D {
                bounds_min[d] = pts.iter().map(|p| p[d]).fold(f32::INFINITY, f32::min);
                bounds_max[d] = pts.iter().map(|p| p[d]).fold(f32::NEG_INFINITY, f32::max);
            }
            Self::build_node(
                &mut nodes, &mut pts, &mut order, 0, n, bounds_min, bounds_max, leaf_size,
            );
        }
        Self {
            nodes,
            tree_indices: order.iter().map(|&o| indices[o as usize]).collect(),
            bounds_min,
            bounds_max,
        }
    }

    fn partition(pts: &mut [[f32; D]], order: &mut [u32], start: usize, end: usize, dim: usize, split: f32) -> usize {
        // Stable two-sided partition matching `partition_points_small`
        // semantics (the block path in the C is an optimization).
        let count = end - start;
        let mut buf_p: Vec<[f32; D]> = Vec::with_capacity(count);
        let mut buf_o: Vec<u32> = Vec::with_capacity(count);
        let mut l = 0usize;
        for i in start..end {
            if pts[i][dim] < split {
                buf_p.push(pts[i]);
                buf_o.push(order[i]);
                l += 1;
            }
        }
        for i in start..end {
            if pts[i][dim] >= split {
                buf_p.push(pts[i]);
                buf_o.push(order[i]);
            }
        }
        pts[start..end].copy_from_slice(&buf_p);
        order[start..end].copy_from_slice(&buf_o);
        start + l
    }

    fn compute_bounds(pts: &[[f32; D]], start: usize, end: usize) -> ([f32; D], [f32; D]) {
        let mut lo = [f32::INFINITY; D];
        let mut hi = [f32::NEG_INFINITY; D];
        for p in &pts[start..end] {
            for d in 0..D {
                lo[d] = lo[d].min(p[d]);
                hi[d] = hi[d].max(p[d]);
            }
        }
        (lo, hi)
    }

    fn build_node(
        nodes: &mut Vec<Node>,
        pts: &mut [[f32; D]],
        order: &mut [u32],
        start: usize,
        end: usize,
        bounds_min: [f32; D],
        bounds_max: [f32; D],
        leaf_size: usize,
    ) -> u32 {
        let ni = nodes.len() as u32;
        nodes.push(Node::default());
        if end - start <= leaf_size {
            nodes[ni as usize].children = u32::MAX;
            nodes[ni as usize].leaf = (start as i32, end as i32);
            return ni;
        }
        let mut dim = 0;
        let mut extent_max = 0.0f32;
        for d in 0..D {
            let e = bounds_max[d] - bounds_min[d];
            if e > extent_max {
                extent_max = e;
                dim = d;
            }
        }
        let split_value = bounds_min[dim] + extent_max / 2.0;
        let mut middle = Self::partition(pts, order, start, end, dim, split_value);
        if middle == start || middle == end {
            middle = start + (end - start) / 2;
        }
        let (lb_min, lb_max) = Self::compute_bounds(pts, start, middle);
        let (rb_min, rb_max) = Self::compute_bounds(pts, middle, end);
        nodes[ni as usize].dim = dim as i32;
        let c0 = Self::build_node(nodes, pts, order, start, middle, lb_min, lb_max, leaf_size);
        let c1 = Self::build_node(nodes, pts, order, middle, end, rb_min, rb_max, leaf_size);
        // children indices may have shifted (Vec realloc) — store at
        // the end (matches allocate(2) semantics loosely).
        nodes[ni as usize].children = c0; // c1 = c0's sibling stored implicitly
        nodes[ni as usize].split = (lb_max[dim], rb_min[dim]);
        // Store sibling in an aux field? Upstream children[2] — encode
        // as (c0, c1) inside leaf pair when non-leaf.
        nodes[ni as usize].leaf = (c0 as i32, c1 as i32);
        ni
    }

    pub fn is_empty(&self) -> bool {
        self.tree_indices.is_empty()
    }
    pub fn size(&self) -> usize {
        self.tree_indices.len()
    }
    pub fn tree_indices(&self) -> &[i32] {
        &self.tree_indices
    }

    fn search_node<Q: Query>(
        &self,
        q: &mut Q,
        position: &[f32; D],
        points: &[[f32; D]],
        ni: u32,
        distance_sq: f32,
        offsets: &mut [f32; D],
    ) -> bool {
        let node = self.nodes[ni as usize];
        if node.children == u32::MAX {
            for i in node.leaf.0..node.leaf.1 {
                let index = self.tree_indices[i as usize];
                let pd = dist_sq(position, &points[index as usize]);
                if keep_distance::<Q>(pd, q.max_distance_sq()) && !q.add_point(pd, index) {
                    return false;
                }
            }
            return true;
        }
        let (c0, c1) = (node.leaf.0 as u32, node.leaf.1 as u32);
        let dim = node.dim as usize;
        let value = position[dim];
        let to_low = value - node.split.0;
        let to_high = value - node.split.1;
        let (far_child, far_offset);
        if to_low + to_high < 0.0 {
            if !self.search_node(q, position, points, c0, distance_sq, offsets) {
                return false;
            }
            far_child = c1;
            far_offset = to_high * to_high;
        } else {
            if !self.search_node(q, position, points, c1, distance_sq, offsets) {
                return false;
            }
            far_child = c0;
            far_offset = to_low * to_low;
        }
        let previous = offsets[dim];
        let far_distance_sq = distance_sq + far_offset - previous;
        if !keep_distance::<Q>(far_distance_sq, q.max_distance_sq()) {
            return true;
        }
        offsets[dim] = far_offset;
        let keep = self.search_node(q, position, points, far_child, far_distance_sq, offsets);
        offsets[dim] = previous;
        keep
    }

    fn search<Q: Query>(&self, q: &mut Q, position: &[f32; D], points: &[[f32; D]]) {
        let mut offsets = [0.0f32; D];
        let mut distance_sq = 0.0;
        for d in 0..D {
            if position[d] < self.bounds_min[d] {
                offsets[d] = (self.bounds_min[d] - position[d]).powi(2);
            } else if position[d] > self.bounds_max[d] {
                offsets[d] = (position[d] - self.bounds_max[d]).powi(2);
            }
            distance_sq += offsets[d];
        }
        self.search_node(q, position, points, 0, distance_sq, &mut offsets);
    }

    /// `find_nearest` → (index, distance_sq).
    pub fn find_nearest(&self, points: &[[f32; D]], position: &[f32; D], max_distance_sq: f32) -> (i32, f32) {
        if self.is_empty() {
            return (-1, max_distance_sq);
        }
        let mut q = NearestQuery {
            index: -1,
            distance_sq: max_distance_sq,
        };
        self.search(&mut q, position, points);
        (q.index, q.distance_sq)
    }

    /// `find_nearest_filtered`.
    pub fn find_nearest_filtered(
        &self,
        points: &[[f32; D]],
        position: &[f32; D],
        filter: impl Fn(i32) -> bool,
        max_distance_sq: f32,
    ) -> (i32, f32) {
        if self.is_empty() {
            return (-1, max_distance_sq);
        }
        let mut q = FilteredNearestQuery {
            filter: &filter,
            index: -1,
            distance_sq: max_distance_sq,
        };
        self.search(&mut q, position, points);
        (q.index, q.distance_sq)
    }

    /// `find_nearest_n` → count written into `nearest` (sorted).
    pub fn find_nearest_n(
        &self,
        points: &[[f32; D]],
        position: &[f32; D],
        max_distance_sq: f32,
        nearest: &mut [Nearest],
    ) -> usize {
        if self.is_empty() || nearest.is_empty() {
            return 0;
        }
        let mut q = NearestNQuery {
            nearest,
            distance_sq: max_distance_sq,
            count: 0,
        };
        self.search(&mut q, position, points);
        q.count
    }

    /// `foreach_in_radius`.
    pub fn foreach_in_radius(
        &self,
        points: &[[f32; D]],
        position: &[f32; D],
        radius: f32,
        mut f: impl FnMut(i32, f32),
    ) {
        if self.is_empty() {
            return;
        }
        let mut q = RadiusQuery {
            radius_sq: radius * radius,
            f: &mut f,
            count: 0,
        };
        self.search(&mut q, position, points);
    }

    /// `find_in_radius` → results sorted by (distance_sq, index).
    pub fn find_in_radius(&self, points: &[[f32; D]], position: &[f32; D], radius: f32) -> Vec<Nearest> {
        let mut out = Vec::new();
        self.foreach_in_radius(points, position, radius, |index, distance_sq| {
            out.push(Nearest { index, distance_sq });
        });
        out.sort_by(|a, b| {
            a.distance_sq
                .partial_cmp(&b.distance_sq)
                .unwrap()
                .then(a.index.cmp(&b.index))
        });
        out
    }

    /// `count_in_radius`.
    pub fn count_in_radius(&self, points: &[[f32; D]], position: &[f32; D], radius: f32) -> usize {
        let mut count = 0usize;
        self.foreach_in_radius(points, position, radius, |_, _| count += 1);
        count
    }

    /// `kdtree::calc_duplicates` — mark each duplicate with its
    /// first-seen index; returns found count.
    pub fn calc_duplicates(&self, points: &[[f32; D]], range: f32, duplicates: &mut [i32]) -> usize {
        if range < 0.0 {
            return 0;
        }
        let mut found = 0usize;
        for i in 0..self.tree_indices.len() {
            let index = self.tree_indices[i];
            if duplicates[index as usize] != -1 && duplicates[index as usize] != index {
                continue;
            }
            let found_before = found;
            self.foreach_in_radius(points, &points[index as usize], range, |other, _| {
                if other != index && duplicates[other as usize] == -1 {
                    duplicates[other as usize] = index;
                    found += 1;
                }
            });
            if found != found_before {
                duplicates[index as usize] = index;
            }
        }
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nearest_and_radius() {
        let pts: Vec<[f32; 2]> = vec![
            [0.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
            [5.0, 5.0],
            [5.1, 5.0],
        ];
        let tree = KDTree::<2>::new(&pts, 2);
        let (i, d) = tree.find_nearest(&pts, &[0.9, 0.1], f32::MAX);
        assert_eq!(i, 1);
        assert!(d < 0.05);
        let r = tree.find_in_radius(&pts, &[0.0, 0.0], 1.5);
        assert_eq!(r.len(), 3);
        assert_eq!(tree.count_in_radius(&pts, &[5.0, 5.0], 0.5), 2);
    }
}
