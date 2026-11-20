//! Port of `convexhull_2d.cc` — `BLI_convexhull_2d` (sorted Andrew
//! monotone-chain variant) and `BLI_convexhull_aabb_fit_points_2d`
//! (rotating calipers minimum-area bounding-box angle).
//!
//! `USE_CONVEX_CROSS_PRODUCT_ENSURE` is defined upstream (the
//! cross-product safety pops are ported); `USE_BRUTE_FORCE_ASSERT`
//! is not — the debug-only brute-force check is omitted.

use blenders_math::vector::Vec2;

fn sincos_rotate_cw_x(sincos: Vec2, p: Vec2) -> f32 {
    sincos.x * p.x + sincos.y * p.y
}
fn sincos_rotate_cw_y(sincos: Vec2, p: Vec2) -> f32 {
    sincos.y * p.x - sincos.x * p.y
}
fn is_left(p0: Vec2, p1: Vec2, p2: Vec2) -> f32 {
    (p1.x - p0.x) * (p2.y - p0.y) - (p2.x - p0.x) * (p1.y - p0.y)
}

/// `convexhull_2d_sorted_impl` — monotone-chain hull over presorted
/// (y,x) points; returns stack top index.
/// `convexhull_2d_stack_finalize` — trailing cross-product cleanup
/// (USE_CONVEX_CROSS_PRODUCT_ENSURE).
fn convexhull_2d_stack_finalize(points: &[Vec2], r_points: &[i32], range: &mut [i32; 2]) {
    let mut bot = range[0];
    let mut top = range[1];
    while top - bot >= 2 {
        if is_left(
            points[r_points[(top - 1) as usize] as usize],
            points[r_points[bot as usize] as usize],
            points[r_points[top as usize] as usize],
        ) >= 0.0
        {
            top -= 1;
            continue;
        }
        if is_left(
            points[r_points[top as usize] as usize],
            points[r_points[(bot + 1) as usize] as usize],
            points[r_points[bot as usize] as usize],
        ) >= 0.0
        {
            bot += 1;
            continue;
        }
        break;
    }
    range[0] = bot;
    range[1] = top;
}

/// `convexhull_2d_stack_push` — push `index` with the
/// cross-product ensure pops.
fn stack_push(points: &[Vec2], r_points: &mut [i32], top: &mut i32, index: i32) {
    while *top > 0
        && is_left(
            points[r_points[(*top - 1) as usize] as usize],
            points[index as usize],
            points[r_points[*top as usize] as usize],
        ) >= 0.0
    {
        *top -= 1;
    }
    *top += 1;
    r_points[*top as usize] = index;
}

fn convexhull_2d_sorted_impl(points: &[Vec2], r_points: &mut [i32]) -> i32 {
    let points_num = points.len();
    debug_assert!(points_num >= 2);
    let mut top: i32 = -1;
    let minmin = 0usize;
    let maxmax = points_num - 1;

    let xmin = points[0].x;
    let mut i = 1usize;
    while i <= maxmax && points[i].x == xmin {
        i += 1;
    }
    let minmax = i - 1;
    if minmax == maxmax {
        stack_push(points, r_points, &mut top, minmin as i32);
        if points[minmax].y != points[minmin].y {
            stack_push(points, r_points, &mut top, minmax as i32);
        }
        return top;
    }

    let xmax = points[maxmax].x;
    let mut i = maxmax as i64 - 1;
    while i >= 0 && points[i as usize].x == xmax {
        i -= 1;
    }
    let maxmin = (i + 1) as usize;

    stack_push(points, r_points, &mut top, minmin as i32);

    let mut i = minmax;
    loop {
        i += 1;
        if i > maxmin {
            break;
        }
        if i < maxmin && is_left(points[minmin], points[maxmin], points[i]) >= 0.0 {
            continue;
        }
        while top > 0 {
            if is_left(
                points[r_points[(top - 1) as usize] as usize],
                points[r_points[top as usize] as usize],
                points[i],
            ) > 0.0
            {
                break;
            }
            top -= 1;
        }
        stack_push(points, r_points, &mut top, i as i32);
    }

    if maxmax != maxmin {
        stack_push(points, r_points, &mut top, maxmax as i32);
    }

    let bot = top;
    let mut i = maxmin as i64;
    loop {
        i -= 1;
        if i < minmax as i64 {
            break;
        }
        let i = i as usize;
        if i > minmax && is_left(points[maxmax], points[minmax], points[i]) >= 0.0 {
            continue;
        }
        while top > bot {
            if is_left(
                points[r_points[(top - 1) as usize] as usize],
                points[r_points[top as usize] as usize],
                points[i],
            ) > 0.0
            {
                break;
            }
            top -= 1;
        }
        if points[i] == points[r_points[0] as usize] {
            return top;
        }
        stack_push(points, r_points, &mut top, i as i32);
    }

    if minmax != minmin && r_points[0] != minmin as i32 {
        stack_push(points, r_points, &mut top, minmin as i32);
    }
    top
}

/// `BLI_convexhull_2d` — hull of `points` in CCW order; returns
/// `(hull_indices, hull_len)`; indices reference `points`.
pub fn convexhull_2d(points: &[Vec2]) -> Vec<i32> {
    let points_num = points.len();
    if points_num < 2 {
        return (0..points_num as i32).collect();
    }
    let mut points_map: Vec<i32> = (0..points_num as i32).collect();
    points_map.sort_by(|&a, &b| {
        let (pa, pb) = (points[a as usize], points[b as usize]);
        pa.y.partial_cmp(&pb.y)
            .unwrap()
            .then(pa.x.partial_cmp(&pb.x).unwrap())
    });
    let points_sort: Vec<Vec2> = points_map.iter().map(|&i| points[i as usize]).collect();
    let mut r_points = vec![0i32; points_num];
    let top = convexhull_2d_sorted_impl(&points_sort, &mut r_points);
    // `convexhull_2d_sorted` — finalize trims bot/top.
    let mut range = [0i32, top];
    convexhull_2d_stack_finalize(&points_sort, &r_points, &mut range);
    let hull_num = (range[1] - range[0] + 1) as usize;
    let mut out = Vec::with_capacity(hull_num);
    for i in 0..hull_num {
        out.push(points_map[r_points[i] as usize]);
    }
    out
}

fn sincos_canonical(sincos: Vec2) -> Vec2 {
    if sincos.x < 0.0 {
        if sincos.y < 0.0 {
            Vec2::new(-sincos.x, -sincos.y)
        } else if sincos.x == -1.0 && sincos.y == 0.0 {
            Vec2::new(-sincos.x, sincos.y)
        } else {
            Vec2::new(sincos.y, -sincos.x)
        }
    } else {
        if sincos.y < 0.0 {
            Vec2::new(-sincos.y, sincos.x)
        } else if sincos.x == 0.0 && sincos.y == 1.0 {
            Vec2::new(sincos.y, sincos.x)
        } else {
            sincos
        }
    }
}

#[derive(Clone, Copy, Default)]
struct AngleCanonical {
    sincos: Vec2,
    sincos_canonical: Vec2,
    index: i32,
}

fn hull_angle_canonical_cmp(a: &AngleCanonical, b: &AngleCanonical) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    if a.sincos_canonical.x < b.sincos_canonical.x {
        return Less;
    }
    if a.sincos_canonical.x > b.sincos_canonical.x {
        return Greater;
    }
    if a.sincos_canonical.y > b.sincos_canonical.y {
        return Less;
    }
    if a.sincos_canonical.y < b.sincos_canonical.y {
        return Greater;
    }
    b.index.cmp(&a.index)
}

#[derive(Clone, Copy)]
struct HullAngleStep {
    next: Option<usize>,
    angle: AngleCanonical,
    index: i32,
    index_max: i32,
}

/// `HullAngleIter` — linked list over the 4 axis steps by index.
struct HullAngleIter {
    axis_ordered: Option<usize>,
    axis: [[HullAngleStep; 2]; 2],
    next_slot: usize, // free-list of detached steps (steps stay in `axis` when reinserted)
    points_hull: Vec<Vec2>,
}

impl HullAngleIter {
    /// `hull_angle_insert_ordered` — insert step (by its slot id)
    /// into the ordered list.
    fn insert_ordered(&mut self, insert_id: usize) {
        // id = axis*2 + i
        let insert_angle = self.axis[insert_id / 2][insert_id % 2].angle;
        let mut prev: Option<usize> = None;
        let mut iter = self.axis_ordered;
        while let Some(id) = iter {
            if hull_angle_canonical_cmp(&self.axis[id / 2][id % 2].angle, &insert_angle)
                != std::cmp::Ordering::Greater
            {
                break;
            }
            prev = iter;
            iter = self.axis[id / 2][id % 2].next;
        }
        match prev {
            None => {
                self.axis[insert_id / 2][insert_id % 2].next = iter;
                self.axis_ordered = Some(insert_id);
            }
            Some(p) => {
                self.axis[insert_id / 2][insert_id % 2].next = iter;
                self.axis[p / 2][p % 2].next = Some(insert_id);
            }
        }
    }

    /// `convexhull_2d_angle_iter_step_on_axis`.
    fn step_on_axis(&mut self, slot: usize) -> bool {
        let n = self.points_hull.len();
        loop {
            let (a, i) = (slot / 2, slot % 2);
            if self.axis[a][i].index == self.axis[a][i].index_max {
                self.axis[a][i].index = -1;
                return false;
            }
            let i_curr = self.axis[a][i].index as usize;
            let i_next = (i_curr + 1) % n;
            let dir = self.points_hull[i_next] - self.points_hull[i_curr];
            self.axis[a][i].index = i_next as i32;
            let len = (dir.x * dir.x + dir.y * dir.y).sqrt();
            if len != 0.0 {
                let sincos = dir / len;
                self.axis[a][i].angle.sincos = sincos;
                self.axis[a][i].angle.sincos_canonical = sincos_canonical(sincos);
                self.axis[a][i].angle.index = i_curr as i32;
                return true;
            }
        }
    }

    /// `convexhull_2d_angle_iter_init`.
    fn init(points_hull: &[Vec2]) -> HullAngleIter {
        let n = points_hull.len();
        let mut hiter = HullAngleIter {
            axis_ordered: None,
            axis: [[HullAngleStep {
                next: None,
                angle: AngleCanonical::default(),
                index: 0,
                index_max: 0,
            }; 2]; 2],
            next_slot: 0,
            points_hull: points_hull.to_vec(),
        };
        let mut range = [[points_hull[0].x; 2], [points_hull[0].y; 2]];
        for i in 1..n {
            let p = points_hull[i];
            for axis in 0..2 {
                let v = if axis == 0 { p.x } else { p.y };
                if range[axis][0] < v {
                    range[axis][0] = v;
                    hiter.axis[axis][0].index = i as i32;
                }
                if range[axis][1] > v {
                    range[axis][1] = v;
                    hiter.axis[axis][1].index = i as i32;
                }
            }
        }
        for axis in 0..2 {
            for i in 0..2 {
                let i_orig = hiter.axis[axis][i].index;
                let mut i_curr = i_orig;
                loop {
                    let i_prev = ((i_curr + n as i32 - 1) % n as i32) ;
                    if i_prev == i_orig {
                        break;
                    }
                    let dir = points_hull[i_curr as usize] - points_hull[i_prev as usize];
                    let len = (dir.x * dir.x + dir.y * dir.y).sqrt();
                    if len != 0.0 {
                        let sincos = dir / len;
                        let av = if axis == 0 { sincos.x } else { sincos.y };
                        if av.abs() > 0.5 {
                            break;
                        }
                        if sincos_canonical(sincos).x != 1.0 {
                            break;
                        }
                    }
                    i_curr = i_prev;
                    hiter.axis[axis][i].index = i_curr;
                }
            }
        }
        hiter.axis[0][0].index_max = hiter.axis[1][0].index; // West to south.
        hiter.axis[1][0].index_max = hiter.axis[0][1].index; // South to east.
        hiter.axis[0][1].index_max = hiter.axis[1][1].index; // East to north.
        hiter.axis[1][1].index_max = hiter.axis[0][0].index; // North to west.
        for axis in 0..2 {
            for i in 0..2 {
                let slot = axis * 2 + i;
                hiter.axis[axis][i].angle.index = hiter.axis[axis][i].index;
                if hiter.step_on_axis(slot) {
                    hiter.insert_ordered(slot);
                }
            }
        }
        hiter
    }

    /// `convexhull_2d_angle_iter_step`.
    fn step(&mut self) {
        let Some(slot) = self.axis_ordered else {
            return;
        };
        self.axis_ordered = self.axis[slot / 2][slot % 2].next;
        if self.step_on_axis(slot) {
            self.insert_ordered(slot);
        }
    }
}

/// `convexhull_2d_compute_extent_on_axis` — walk hull indices until
/// the rotated extent stops improving.
fn compute_extent_on_axis(
    points_hull: &[Vec2],
    sincos: Vec2,
    axis: usize,
    axis_sign: i32,
    index_p: &mut i32,
) -> f32 {
    let n = points_hull.len();
    let index_init = *index_p;
    let f = |p: Vec2| {
        if axis == 0 {
            sincos_rotate_cw_x(sincos, p)
        } else {
            sincos_rotate_cw_y(sincos, p)
        }
    };
    let mut index_best = index_init;
    let mut value_best = f(points_hull[index_best as usize]);
    for count in 1..n {
        let index_test = ((index_init + count as i32) % n as i32) as usize;
        let value_test = f(points_hull[index_test]);
        if (axis_sign == -1 && value_test > value_best)
            || (axis_sign == 1 && value_test < value_best)
        {
            break;
        }
        value_best = value_test;
        index_best = index_test as i32;
    }
    *index_p = index_best;
    value_best
}

/// `convexhull_aabb_fit_hull_2d` — rotating-calipers min-area fit.
fn convexhull_aabb_fit_hull_2d(points_hull: &[Vec2]) -> Vec2 {
    let mut area_best = f32::MAX;
    let mut sincos_best = Vec2::new(0.0, 1.0);
    let mut index_best = i32::MAX;
    let mut bounds_index_min = [0i32; 2];
    let mut bounds_index_max = [0i32; 2];

    let mut hiter = HullAngleIter::init(points_hull);
    bounds_index_min[0] = hiter.axis[0][1].angle.index;
    bounds_index_max[0] = hiter.axis[0][0].angle.index;
    bounds_index_min[1] = hiter.axis[1][0].angle.index;
    bounds_index_max[1] = hiter.axis[1][1].angle.index;

    while let Some(slot) = hiter.axis_ordered {
        let hstep = hiter.axis[slot / 2][slot % 2];
        let sc = hstep.angle.sincos_canonical;
        let b0min = compute_extent_on_axis(points_hull, sc, 0, -1, &mut bounds_index_min[0]);
        let b0max = compute_extent_on_axis(points_hull, sc, 0, 1, &mut bounds_index_max[0]);
        let b1min = compute_extent_on_axis(points_hull, sc, 1, -1, &mut bounds_index_min[1]);
        let b1max = compute_extent_on_axis(points_hull, sc, 1, 1, &mut bounds_index_max[1]);
        let area_test = (b0max - b0min) * (b1max - b1min);
        if area_test < area_best || (area_test == area_best && hstep.angle.index < index_best) {
            area_best = area_test;
            sincos_best = hstep.angle.sincos;
            index_best = hstep.angle.index;
        }
        hiter.step();
    }
    sincos_best
}

/// `BLI_convexhull_aabb_fit_points_2d_as_sincos`.
pub fn convexhull_aabb_fit_points_2d_as_sincos(points: &[Vec2]) -> Vec2 {
    let mut sincos = Vec2::new(0.0, 1.0);
    let index_map = convexhull_2d(points);
    if index_map.len() > 1 {
        let hull: Vec<Vec2> = index_map.iter().map(|&i| points[i as usize]).collect();
        sincos = convexhull_aabb_fit_hull_2d(&hull);
    }
    sincos
}

/// `BLI_convexhull_aabb_fit_points_2d` — the min-area box angle.
pub fn convexhull_aabb_fit_points_2d(points: &[Vec2]) -> f32 {
    let sincos = convexhull_aabb_fit_points_2d_as_sincos(points);
    sincos.x.atan2(sincos.y)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn square() {
        let pts = [
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(0.5, 0.5),
        ];
        let hull = convexhull_2d(&pts);
        assert_eq!(hull.len(), 4);
        let ang = convexhull_aabb_fit_points_2d(&pts);
        assert!(ang.abs() < 0.01 || (ang - std::f32::consts::FRAC_PI_2).abs() < 0.01);
    }
}
