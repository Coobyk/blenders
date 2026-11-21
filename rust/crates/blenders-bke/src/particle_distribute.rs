//! Port of `intern/particle_distribute.cc` — distribution math:
//! `distribute_grid` bounds/sizing, `hammersley_create`,
//! `init_mv_jit`, `psys_uv_to_w`, `distribute_binary_search`.
//! Task-pool execs injected.

use blenders_bli::jitter_2d::{jitterate1, jitterate2};
use blenders_bli::rand::{hammersley_1d, Rng};
use blenders_math::geom::interp_weights_poly;
use blenders_math::vector::Vec3;

/// `PART_FROM_*`.
pub const PART_FROM_VERT: i32 = 0;
/// See above.
pub const PART_FROM_FACE: i32 = 1;
/// See above.
pub const PART_FROM_VOLUME: i32 = 2;
/// `PARS_UNEXIST`.
pub const PARS_UNEXIST: i32 = 1 << 9;

/// `axis_dominant_v3_single`.
pub fn axis_dominant_v3_single(axis: Vec3) -> usize {
    let x = axis.x.abs();
    let y = axis.y.abs();
    let z = axis.z.abs();
    if x > y && x > z {
        0
    } else if y > z {
        1
    } else {
        2
    }
}

/// `distribute_grid` — fills `fuv`/`flag` for `res³` grid particles
/// spanning the mesh bounds; returns `(fuvs, flags)` and marks
/// `PARS_UNEXIST` where the grid cell has no element (the vert/face
/// occupancy passes are in `distribute_grid_mark_*`).
#[allow(clippy::too_many_arguments)]
pub fn distribute_grid_fill(
    positions: &[[f32; 3]],
    res: i32,
) -> (Vec<[f32; 3]>, Vec<i32>, [i32; 3], [f32; 3], [f32; 3], f32) {
    let (mut min, mut max) = ([f32::MAX; 3], [f32::MIN; 3]);
    if positions.is_empty() {
        min = [0.0; 3];
        max = [0.0; 3];
    } else {
        for p in positions {
            for a in 0..3 {
                min[a] = min[a].min(p[a]);
                max[a] = max[a].max(p[a]);
            }
        }
    }
    let delta = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
    let axis = axis_dominant_v3_single(Vec3::from(delta));
    let d = delta[axis] / res as f32;
    let mut size = [0i32; 3];
    size[axis] = res;
    size[(axis + 1) % 3] = (delta[(axis + 1) % 3] / d).ceil() as i32;
    size[(axis + 2) % 3] = (delta[(axis + 2) % 3] / d).ceil() as i32;
    size[(axis + 1) % 3] = size[(axis + 1) % 3].min(res);
    size[(axis + 2) % 3] = size[(axis + 2) % 3].min(res);
    size[0] = size[0].max(1);
    size[1] = size[1].max(1);
    size[2] = size[2].max(1);
    for a in 0..3 {
        min[a] += if d < delta[a] { d / 2.0 } else { delta[a] / 2.0 };
    }

    let mut fuvs = Vec::with_capacity((res * res * res) as usize);
    let mut flags = Vec::with_capacity((res * res * res) as usize);
    for i in 0..res {
        for j in 0..res {
            for k in 0..res {
                fuvs.push([
                    min[0] + i as f32 * d,
                    min[1] + j as f32 * d,
                    min[2] + k as f32 * d,
                ]);
                flags.push(PARS_UNEXIST);
            }
        }
    }
    (fuvs, flags, size, min, delta, d)
}

/// `distribute_grid` vert pass — clears `PARS_UNEXIST` at the grid
/// cells containing mesh verts.
pub fn distribute_grid_mark_verts(
    positions: &[[f32; 3]],
    flags: &mut [i32],
    res: i32,
    size: [i32; 3],
    min: [f32; 3],
    delta: [f32; 3],
    d: f32,
) {
    let mut min = min;
    for a in 0..3 {
        min[a] -= d / 2.0;
    }
    for p in positions {
        let vec = [
            (p[0] - min[0]) / delta[0],
            (p[1] - min[1]) / delta[1],
            (p[2] - min[2]) / delta[2],
        ];
        let idx = ((vec[0] * (size[0] - 1) as f32) as i32 * res
            + (vec[1] * (size[1] - 1) as f32) as i32)
            * res
            + (vec[2] * (size[2] - 1) as f32) as i32;
        if idx >= 0 && (idx as usize) < flags.len() {
            flags[idx as usize] &= !PARS_UNEXIST;
        }
    }
}

/// `hammersley_create`.
pub fn hammersley_create(n: i32, seed: i32, amount: f32) -> Vec<f32> {
    let mut rng = Rng::new(31415926 + n as u32 + seed as u32);
    let ofs0 = rng.get_double() + amount as f64;
    let ofs1 = rng.get_double() + amount as f64;
    let mut out = vec![0.0f32; (n * 2) as usize];
    for k in 0..n {
        let mut t = 0.0f64;
        hammersley_1d(k as u32, &mut t);
        out[2 * k as usize] = ((k as f64 / n as f64 + ofs0) % 1.0) as f32;
        out[2 * k as usize + 1] = ((t + ofs1) % 1.0) as f32;
    }
    out
}

/// `init_mv_jit` — multi-level jitter offsets for `num` samples.
pub fn init_mv_jit(num: i32, seed2: i32, amount: f32) -> Vec<f32> {
    if num == 0 {
        return Vec::new();
    }
    let rad1 = 1.0 / (num as f32).sqrt();
    let rad2 = 1.0 / num as f32;
    let rad3 = (num as f32).sqrt() / num as f32;
    let mut rng = Rng::new(31415926 + num as u32 + seed2 as u32);
    let mut jit = vec![0.0f32; (num * 2) as usize];
    let mut x = 0.0f32;
    let num2 = 2 * num as usize;
    let mut i = 0;
    while i < num2 {
        jit[i] = x + amount * rad1 * (0.5 - rng.get_float());
        jit[i + 1] =
            i as f32 / (2.0 * num as f32) + amount * rad1 * (0.5 - rng.get_float());
        jit[i] -= jit[i].floor();
        jit[i + 1] -= jit[i + 1].floor();
        x += rad3;
        x -= x.floor();
        i += 2;
    }
    /* 4× jitterate1×2 + jitterate2. */
    let mut jit_arr: Vec<[f32; 2]> = jit.chunks(2).map(|c| [c[0], c[1]]).collect();
    for _ in 0..4 {
        jitterate1(&mut jit_arr, rad1);
        jitterate1(&mut jit_arr, rad1);
        jitterate2(&mut jit_arr, rad2);
    }
    jit_arr.into_iter().flatten().collect()
}

/// `psys_uv_to_w` — unit-square (u,v) → tri/quad barycentric weights.
pub fn psys_uv_to_w(u: f32, v: f32, quad: bool) -> [f32; 4] {
    let (mut u, mut v) = (u, v);
    if !quad {
        if u + v > 1.0 {
            v = 1.0 - v;
        } else {
            u = 1.0 - u;
        }
    }
    let vert = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    ];
    let co = Vec3::new(u, v, 0.0);
    let mut w = [0.0f32; 4];
    if quad {
        interp_weights_poly(&mut w, &vert, co);
    } else {
        interp_weights_poly(&mut w[..3], &vert[..3], co);
        w[3] = 0.0;
    }
    w
}

/// `distribute_binary_search`.
pub fn distribute_binary_search(sum: &[f32], value: f32) -> usize {
    let n = sum.len();
    let mut low = 0usize;
    let mut high = n - 1;
    if high == low {
        return low;
    }
    if sum[low] >= value {
        return low;
    }
    if sum[high - 1] < value {
        return high;
    }
    while low < high {
        let mid = (low + high) / 2;
        if sum[mid] >= value && sum[mid - 1] < value {
            return mid;
        }
        if sum[mid] > value {
            high = mid - 1;
        } else {
            low = mid + 1;
        }
    }
    low
}
