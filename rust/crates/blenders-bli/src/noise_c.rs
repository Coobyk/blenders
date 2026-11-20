//! Port of `noise_c.cc` — legacy `BLI_noise_*` noise suite
//! (Perlin variants, Voronoi, turbulence, Musgrave fractals, cell).
//!
//! Static tables live in `noise_tables.rs` (extracted verbatim).

use crate::noise_tables::{G_PERLIN_DATA_UB, G_PERLIN_DATA_V3, HASHPNTF, HASHVECTF, NOISE_HASH_UCHAR_512};

type HashIdx = usize;
#[inline]
fn hash(i: i32) -> HashIdx {
    NOISE_HASH_UCHAR_512[(i & 511) as usize] as usize
}
#[inline]
fn hashpnt(x: i32, y: i32, z: i32) -> [f32; 3] {
    let i = hash(hash(hash(z & 255) as i32 + (y & 255)) as i32 + (x & 255));
    let o = 3 * i;
    [HASHPNTF[o], HASHPNTF[o + 1], HASHPNTF[o + 2]]
}

/* Improved Perlin (new) */
#[inline]
fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}
#[inline]
fn npfade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}
#[inline]
fn grad(hash_val: i32, x: f32, y: f32, z: f32) -> f32 {
    let h = hash_val & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 { y } else if h == 12 || h == 14 { x } else { z };
    (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
}
fn new_perlin(x: f32, y: f32, z: f32) -> f32 {
    let mut x = x;
    let mut y = y;
    let mut z = z;
    let mut u = x.floor();
    let mut v = y.floor();
    let mut w = z.floor();
    let xx = (u as i32) & 255;
    let yy = (v as i32) & 255;
    let zz = (w as i32) & 255;
    x -= u;
    y -= v;
    z -= w;
    u = npfade(x);
    v = npfade(y);
    w = npfade(z);
    let h = |i: i32| NOISE_HASH_UCHAR_512[(i & 511) as usize] as i32;
    let a = h(xx) + yy;
    let aa = h(a) + zz;
    let ab = h(a + 1) + zz;
    let b = h(xx + 1) + yy;
    let ba = h(b) + zz;
    let bb = h(b + 1) + zz;
    lerp(
        w,
        lerp(
            v,
            lerp(u, grad(h(aa), x, y, z), grad(h(ba), x - 1.0, y, z)),
            lerp(u, grad(h(ab), x, y - 1.0, z), grad(h(bb), x - 1.0, y - 1.0, z)),
        ),
        lerp(
            v,
            lerp(u, grad(h(aa + 1), x, y, z - 1.0), grad(h(ba + 1), x - 1.0, y, z - 1.0)),
            lerp(
                u,
                grad(h(ab + 1), x, y - 1.0, z - 1.0),
                grad(h(bb + 1), x - 1.0, y - 1.0, z - 1.0),
            ),
        ),
    )
}
fn new_perlin_u(x: f32, y: f32, z: f32) -> f32 {
    0.5 + 0.5 * new_perlin(x, y, z)
}

/* Original Blender noise */
fn org_blender_noise(x: f32, y: f32, z: f32) -> f32 {
    let fx = x.floor();
    let fy = y.floor();
    let fz = z.floor();
    let ox = x - fx;
    let oy = y - fy;
    let oz = z - fz;
    let ix = fx as i32;
    let iy = fy as i32;
    let iz = fz as i32;
    let jx = ox - 1.0;
    let jy = oy - 1.0;
    let jz = oz - 1.0;
    let mut cn1 = ox * ox;
    let mut cn2 = oy * oy;
    let mut cn3 = oz * oz;
    let mut cn4 = jx * jx;
    let mut cn5 = jy * jy;
    let mut cn6 = jz * jz;
    cn1 = 1.0 - 3.0 * cn1 + 2.0 * cn1 * ox;
    cn2 = 1.0 - 3.0 * cn2 + 2.0 * cn2 * oy;
    cn3 = 1.0 - 3.0 * cn3 + 2.0 * cn3 * oz;
    cn4 = 1.0 - 3.0 * cn4 - 2.0 * cn4 * jx;
    cn5 = 1.0 - 3.0 * cn5 - 2.0 * cn5 * jy;
    cn6 = 1.0 - 3.0 * cn6 - 2.0 * cn6 * jz;
    let b00 = hash(hash(ix & 255) as i32 + (iy & 255));
    let b10 = hash(hash((ix + 1) & 255) as i32 + (iy & 255));
    let b01 = hash(hash(ix & 255) as i32 + ((iy + 1) & 255));
    let b11 = hash(hash((ix + 1) & 255) as i32 + ((iy + 1) & 255));
    let b20 = (iz & 255) as usize;
    let b21 = ((iz + 1) & 255) as usize;
    let hv = |i: usize| -> [f32; 3] {
        let o = 3 * i;
        [HASHVECTF[o], HASHVECTF[o + 1], HASHVECTF[o + 2]]
    };
    let mut n = 0.5f32;
    let mut i = cn1 * cn2 * cn3;
    let mut h = hv(hash(b20 as i32 + b00 as i32));
    n += i * (h[0] * ox + h[1] * oy + h[2] * oz);
    i = cn1 * cn2 * cn6;
    h = hv(hash(b21 as i32 + b00 as i32));
    n += i * (h[0] * ox + h[1] * oy + h[2] * jz);
    i = cn1 * cn5 * cn3;
    h = hv(hash(b20 as i32 + b01 as i32));
    n += i * (h[0] * ox + h[1] * jy + h[2] * oz);
    i = cn1 * cn5 * cn6;
    h = hv(hash(b21 as i32 + b01 as i32));
    n += i * (h[0] * ox + h[1] * jy + h[2] * jz);
    i = cn4 * cn2 * cn3;
    h = hv(hash(b20 as i32 + b10 as i32));
    n += i * (h[0] * jx + h[1] * oy + h[2] * oz);
    i = cn4 * cn2 * cn6;
    h = hv(hash(b21 as i32 + b10 as i32));
    n += i * (h[0] * jx + h[1] * oy + h[2] * jz);
    i = cn4 * cn5 * cn3;
    h = hv(hash(b20 as i32 + b11 as i32));
    n += i * (h[0] * jx + h[1] * jy + h[2] * oz);
    i = cn4 * cn5 * cn6;
    h = hv(hash(b21 as i32 + b11 as i32));
    n += i * (h[0] * jx + h[1] * jy + h[2] * jz);
    n.clamp(0.0, 1.0)
}
fn org_blender_noise_s(x: f32, y: f32, z: f32) -> f32 {
    2.0 * org_blender_noise(x, y, z) - 1.0
}

/// `BLI_noise_hnoise`.
pub fn noise_hnoise(noisesize: f32, x: f32, y: f32, z: f32) -> f32 {
    if noisesize == 0.0 {
        return 0.0;
    }
    let (x, y, z) = ((1.0 + x) / noisesize, (1.0 + y) / noisesize, (1.0 + z) / noisesize);
    org_blender_noise(x, y, z)
}
/// `BLI_noise_turbulence`.
pub fn noise_turbulence(noisesize: f32, x: f32, y: f32, z: f32, nr: i32) -> f32 {
    let mut s = noise_hnoise(noisesize, x, y, z);
    let mut d = 0.5f32;
    let mut div = 1.0f32;
    let mut nr = nr;
    while nr > 0 {
        s += d * noise_hnoise(noisesize * d, x, y, z);
        div += d;
        d *= 0.5;
        nr -= 1;
    }
    s / div
}

/* Perlin-himself noise3 */
fn noise3_perlin(vec: [f32; 3]) -> f32 {
    let p = &G_PERLIN_DATA_UB;
    let g = &G_PERLIN_DATA_V3;
    macro_rules! setup {
        ($val:expr) => {{
            let t = $val + 10000.0;
            let b0 = (t as i32) & 255;
            let b1 = (b0 + 1) & 255;
            let r0 = t - t.floor();
            (b0 as usize, b1 as usize, r0, r0 - 1.0)
        }};
    }
    let (bx0, bx1, rx0, rx1) = setup!(vec[0]);
    let (by0, by1, ry0, ry1) = setup!(vec[1]);
    let (bz0, bz1, rz0, rz1) = setup!(vec[2]);
    let i = p[bx0] as usize;
    let j = p[bx1] as usize;
    let b00 = p[i + by0] as usize;
    let b10 = p[j + by0] as usize;
    let b01 = p[i + by1] as usize;
    let b11 = p[j + by1] as usize;
    let surve = |t: f32| t * t * (3.0 - 2.0 * t);
    let (sx, sy, sz) = (surve(rx0), surve(ry0), surve(rz0));
    let vat = |q: [f32; 3], rx: f32, ry: f32, rz: f32| rx * q[0] + ry * q[1] + rz * q[2];
    let u = vat(g[b00 + bz0], rx0, ry0, rz0);
    let v = vat(g[b10 + bz0], rx1, ry0, rz0);
    let a = lerp(sx, u, v);
    let u = vat(g[b01 + bz0], rx0, ry1, rz0);
    let v = vat(g[b11 + bz0], rx1, ry1, rz0);
    let b = lerp(sx, u, v);
    let c = lerp(sy, a, b);
    let u = vat(g[b00 + bz1], rx0, ry0, rz1);
    let v = vat(g[b10 + bz1], rx1, ry0, rz1);
    let a = lerp(sx, u, v);
    let u = vat(g[b01 + bz1], rx0, ry1, rz1);
    let v = vat(g[b11 + bz1], rx1, ry1, rz1);
    let b = lerp(sx, u, v);
    let d = lerp(sy, a, b);
    1.5 * lerp(sz, c, d)
}
fn org_perlin_noise(x: f32, y: f32, z: f32) -> f32 {
    noise3_perlin([x, y, z])
}
fn org_perlin_noise_u(x: f32, y: f32, z: f32) -> f32 {
    0.5 + 0.5 * noise3_perlin([x, y, z])
}
/// `BLI_noise_hnoisep`.
pub fn noise_hnoisep(noisesize: f32, x: f32, y: f32, z: f32) -> f32 {
    noise3_perlin([x / noisesize, y / noisesize, z / noisesize])
}

/* Voronoi */
fn dist_squared(x: f32, y: f32, z: f32, _e: f32) -> f32 {
    x * x + y * y + z * z
}
fn dist_real(x: f32, y: f32, z: f32, _e: f32) -> f32 {
    (x * x + y * y + z * z).sqrt()
}
fn dist_manhattan(x: f32, y: f32, z: f32, _e: f32) -> f32 {
    x.abs() + y.abs() + z.abs()
}
fn dist_chebychev(x: f32, y: f32, z: f32, _e: f32) -> f32 {
    let t = x.abs().max(y.abs());
    t.max(z.abs())
}
fn dist_minkovsky_h(x: f32, y: f32, z: f32, _e: f32) -> f32 {
    let d = x.abs().sqrt() + y.abs().sqrt() + z.abs().sqrt();
    d * d
}
fn dist_minkovsky4(x: f32, y: f32, z: f32, _e: f32) -> f32 {
    let (x, y, z) = (x * x, y * y, z * z);
    (x * x + y * y + z * z).sqrt().sqrt()
}
fn dist_minkovsky(x: f32, y: f32, z: f32, e: f32) -> f32 {
    (x.abs().powf(e) + y.abs().powf(e) + z.abs().powf(e)).powf(1.0 / e)
}
type DistFn = fn(f32, f32, f32, f32) -> f32;
fn dist_fn(dtype: i32) -> DistFn {
    match dtype {
        1 => dist_squared,
        2 => dist_manhattan,
        3 => dist_chebychev,
        4 => dist_minkovsky_h,
        5 => dist_minkovsky4,
        6 => dist_minkovsky,
        _ => dist_real,
    }
}
/// `BLI_noise_voronoi` → (da[4], pa[12]).
pub fn noise_voronoi(x: f32, y: f32, z: f32, me: f32, dtype: i32) -> ([f32; 4], [f32; 12]) {
    let distfunc = dist_fn(dtype);
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let zi = z.floor() as i32;
    let mut da = [1e10f32; 4];
    let mut pa = [0.0f32; 12];
    for xx in xi - 1..=xi + 1 {
        for yy in yi - 1..=yi + 1 {
            for zz in zi - 1..=zi + 1 {
                let p = hashpnt(xx, yy, zz);
                let xd = x - (p[0] + xx as f32);
                let yd = y - (p[1] + yy as f32);
                let zd = z - (p[2] + zz as f32);
                let d = distfunc(xd, yd, zd, me);
                if d < da[0] {
                    da[3] = da[2];
                    da[2] = da[1];
                    da[1] = da[0];
                    da[0] = d;
                    for k in (3..12).rev() {
                        pa[k] = pa[k - 3];
                    }
                    pa[0] = p[0] + xx as f32;
                    pa[1] = p[1] + yy as f32;
                    pa[2] = p[2] + zz as f32;
                } else if d < da[1] {
                    da[3] = da[2];
                    da[2] = da[1];
                    da[1] = d;
                    for k in (6..12).rev() {
                        pa[k] = pa[k - 3];
                    }
                    pa[3] = p[0] + xx as f32;
                    pa[4] = p[1] + yy as f32;
                    pa[5] = p[2] + zz as f32;
                } else if d < da[2] {
                    da[3] = da[2];
                    da[2] = d;
                    for k in (9..12).rev() {
                        pa[k] = pa[k - 3];
                    }
                    pa[6] = p[0] + xx as f32;
                    pa[7] = p[1] + yy as f32;
                    pa[8] = p[2] + zz as f32;
                } else if d < da[3] {
                    da[3] = d;
                    pa[9] = p[0] + xx as f32;
                    pa[10] = p[1] + yy as f32;
                    pa[11] = p[2] + zz as f32;
                }
            }
        }
    }
    (da, pa)
}
fn voronoi_f1(x: f32, y: f32, z: f32) -> f32 {
    noise_voronoi(x, y, z, 1.0, 0).0[0]
}
fn voronoi_f2(x: f32, y: f32, z: f32) -> f32 {
    noise_voronoi(x, y, z, 1.0, 0).0[1]
}
fn voronoi_f3(x: f32, y: f32, z: f32) -> f32 {
    noise_voronoi(x, y, z, 1.0, 0).0[2]
}
fn voronoi_f4(x: f32, y: f32, z: f32) -> f32 {
    noise_voronoi(x, y, z, 1.0, 0).0[3]
}
fn voronoi_f1f2(x: f32, y: f32, z: f32) -> f32 {
    let (da, _) = noise_voronoi(x, y, z, 1.0, 0);
    da[1] - da[0]
}
fn voronoi_cr(x: f32, y: f32, z: f32) -> f32 {
    (10.0 * voronoi_f1f2(x, y, z)).min(1.0)
}
fn voronoi_f1s(x: f32, y: f32, z: f32) -> f32 {
    2.0 * noise_voronoi(x, y, z, 1.0, 0).0[0] - 1.0
}
fn voronoi_f2s(x: f32, y: f32, z: f32) -> f32 {
    2.0 * noise_voronoi(x, y, z, 1.0, 0).0[1] - 1.0
}
fn voronoi_f3s(x: f32, y: f32, z: f32) -> f32 {
    2.0 * noise_voronoi(x, y, z, 1.0, 0).0[2] - 1.0
}
fn voronoi_f4s(x: f32, y: f32, z: f32) -> f32 {
    2.0 * noise_voronoi(x, y, z, 1.0, 0).0[3] - 1.0
}
fn voronoi_f1f2s(x: f32, y: f32, z: f32) -> f32 {
    let (da, _) = noise_voronoi(x, y, z, 1.0, 0);
    2.0 * (da[1] - da[0]) - 1.0
}
fn voronoi_crs(x: f32, y: f32, z: f32) -> f32 {
    2.0 * voronoi_cr(x, y, z) - 1.0
}

/* Cell noise */
fn cell_noise_u(x: f32, y: f32, z: f32) -> f32 {
    let x = (x + 0.000001) * 1.00001;
    let y = (y + 0.000001) * 1.00001;
    let z = (z + 0.000001) * 1.00001;
    let (xi, yi, zi) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
    let mut n = (xi as u32)
        .wrapping_add((yi as u32).wrapping_mul(1301))
        .wrapping_add((zi as u32).wrapping_mul(314159));
    n ^= n << 13;
    let f = (n.wrapping_mul(n.wrapping_mul(n).wrapping_mul(15731).wrapping_add(789221))
        .wrapping_add(1376312589)) as f32;
    f / 4294967296.0
}
/// `BLI_noise_cell`.
pub fn noise_cell(x: f32, y: f32, z: f32) -> f32 {
    2.0 * cell_noise_u(x, y, z) - 1.0
}
/// `BLI_noise_cell_v3`.
pub fn noise_cell_v3(x: f32, y: f32, z: f32) -> [f32; 3] {
    let x = (x + 0.000001) * 1.00001;
    let y = (y + 0.000001) * 1.00001;
    let z = (z + 0.000001) * 1.00001;
    hashpnt(x.floor() as i32, y.floor() as i32, z.floor() as i32)
}

type NoiseFn = fn(f32, f32, f32) -> f32;
fn noise_func(noisebasis: i32, signed: bool) -> (NoiseFn, bool) {
    // returns (fn, shift_xyz) — shift when basis 0 and !signed
    let f: NoiseFn = match noisebasis {
        1 => {
            if signed {
                org_perlin_noise
            } else {
                org_perlin_noise_u
            }
        }
        2 => {
            if signed {
                new_perlin
            } else {
                new_perlin_u
            }
        }
        3 => {
            if signed {
                voronoi_f1s
            } else {
                voronoi_f1
            }
        }
        4 => {
            if signed {
                voronoi_f2s
            } else {
                voronoi_f2
            }
        }
        5 => {
            if signed {
                voronoi_f3s
            } else {
                voronoi_f3
            }
        }
        6 => {
            if signed {
                voronoi_f4s
            } else {
                voronoi_f4
            }
        }
        7 => {
            if signed {
                voronoi_f1f2s
            } else {
                voronoi_f1f2
            }
        }
        8 => {
            if signed {
                voronoi_crs
            } else {
                voronoi_cr
            }
        }
        14 => {
            if signed {
                noise_cell
            } else {
                cell_noise_u
            }
        }
        _ => {
            if signed {
                org_blender_noise_s
            } else {
                org_blender_noise
            }
        }
    };
    (f, noisebasis == 0 && !signed)
}
/// `BLI_noise_generic_noise`.
pub fn noise_generic_noise(noisesize: f32, x: f32, y: f32, z: f32, hard: bool, noisebasis: i32) -> f32 {
    let (f, shift) = noise_func(noisebasis, false);
    let (mut x, mut y, mut z) = (x, y, z);
    if shift {
        x += 1.0;
        y += 1.0;
        z += 1.0;
    }
    if noisesize != 0.0 {
        let inv = 1.0 / noisesize;
        x *= inv;
        y *= inv;
        z *= inv;
    }
    if hard {
        (2.0 * f(x, y, z) - 1.0).abs()
    } else {
        f(x, y, z)
    }
}
/// `BLI_noise_generic_turbulence`.
pub fn noise_generic_turbulence(
    noisesize: f32,
    x: f32,
    y: f32,
    z: f32,
    oct: i32,
    hard: bool,
    noisebasis: i32,
) -> f32 {
    let (f, shift) = noise_func(noisebasis, false);
    let (mut x, mut y, mut z) = (x, y, z);
    if shift {
        x += 1.0;
        y += 1.0;
        z += 1.0;
    }
    if noisesize != 0.0 {
        let inv = 1.0 / noisesize;
        x *= inv;
        y *= inv;
        z *= inv;
    }
    let mut sum = 0.0f32;
    let mut amp = 1.0f32;
    let mut fscale = 1.0f32;
    for _ in 0..=oct {
        let mut t = f(fscale * x, fscale * y, fscale * z);
        if hard {
            t = (2.0 * t - 1.0).abs();
        }
        sum += t * amp;
        amp *= 0.5;
        fscale *= 2.0;
    }
    sum * ((1 << oct) as f32 / ((1 << (oct + 1)) - 1) as f32)
}

/* Musgrave-style */
/// `BLI_noise_mg_fbm`.
pub fn noise_mg_fbm(x: f32, y: f32, z: f32, h: f32, lacunarity: f32, octaves: f32, noisebasis: i32) -> f32 {
    let (f, _) = noise_func(noisebasis, true);
    let (mut x, mut y, mut z) = (x, y, z);
    let mut value = 0.0;
    let mut pwr = 1.0;
    let pwhl = lacunarity.powf(-h);
    for _ in 0..octaves as i32 {
        value += f(x, y, z) * pwr;
        pwr *= pwhl;
        x *= lacunarity;
        y *= lacunarity;
        z *= lacunarity;
    }
    let rmd = octaves - octaves.floor();
    if rmd != 0.0 {
        value += rmd * f(x, y, z) * pwr;
    }
    value
}
/// `BLI_noise_mg_multi_fractal`.
pub fn noise_mg_multi_fractal(
    x: f32,
    y: f32,
    z: f32,
    h: f32,
    lacunarity: f32,
    octaves: f32,
    noisebasis: i32,
) -> f32 {
    let (f, _) = noise_func(noisebasis, true);
    let (mut x, mut y, mut z) = (x, y, z);
    let mut value = 1.0;
    let mut pwr = 1.0;
    let pwhl = lacunarity.powf(-h);
    for _ in 0..octaves as i32 {
        value *= pwr * f(x, y, z) + 1.0;
        pwr *= pwhl;
        x *= lacunarity;
        y *= lacunarity;
        z *= lacunarity;
    }
    let rmd = octaves - octaves.floor();
    if rmd != 0.0 {
        value *= rmd * f(x, y, z) * pwr + 1.0;
    }
    value
}
/// `BLI_noise_mg_hetero_terrain`.
pub fn noise_mg_hetero_terrain(
    x: f32,
    y: f32,
    z: f32,
    h: f32,
    lacunarity: f32,
    octaves: f32,
    offset: f32,
    noisebasis: i32,
) -> f32 {
    let (f, _) = noise_func(noisebasis, true);
    let (mut x, mut y, mut z) = (x, y, z);
    let mut value = offset + f(x, y, z);
    x *= lacunarity;
    y *= lacunarity;
    z *= lacunarity;
    let pwhl = lacunarity.powf(-h);
    let mut pwr = pwhl;
    for _ in 1..octaves as i32 {
        value += (f(x, y, z) + offset) * pwr * value;
        pwr *= pwhl;
        x *= lacunarity;
        y *= lacunarity;
        z *= lacunarity;
    }
    let rmd = octaves - octaves.floor();
    if rmd != 0.0 {
        value += rmd * (f(x, y, z) + offset) * pwr * value;
    }
    value
}
/// `BLI_noise_mg_hybrid_multi_fractal`.
pub fn noise_mg_hybrid_multi_fractal(
    x: f32,
    y: f32,
    z: f32,
    h: f32,
    lacunarity: f32,
    octaves: f32,
    offset: f32,
    gain: f32,
    noisebasis: i32,
) -> f32 {
    let (f, _) = noise_func(noisebasis, true);
    let (mut x, mut y, mut z) = (x, y, z);
    let mut result = f(x, y, z) + offset;
    let mut weight = gain * result;
    x *= lacunarity;
    y *= lacunarity;
    z *= lacunarity;
    let pwhl = lacunarity.powf(-h);
    let mut pwr = pwhl;
    let mut i = 1;
    while weight > 0.001 && i < octaves as i32 {
        weight = weight.min(1.0);
        let signal = (f(x, y, z) + offset) * pwr;
        pwr *= pwhl;
        result += weight * signal;
        weight *= gain * signal;
        x *= lacunarity;
        y *= lacunarity;
        z *= lacunarity;
        i += 1;
    }
    let rmd = octaves - octaves.floor();
    if rmd != 0.0 {
        result += rmd * (f(x, y, z) + offset) * pwr;
    }
    result
}
/// `BLI_noise_mg_ridged_multi_fractal`.
pub fn noise_mg_ridged_multi_fractal(
    x: f32,
    y: f32,
    z: f32,
    h: f32,
    lacunarity: f32,
    octaves: f32,
    offset: f32,
    gain: f32,
    noisebasis: i32,
) -> f32 {
    let (f, _) = noise_func(noisebasis, true);
    let (mut x, mut y, mut z) = (x, y, z);
    let mut signal = (offset - f(x, y, z).abs()).powi(2);
    let mut result = signal;
    let pwhl = lacunarity.powf(-h);
    let mut pwr = pwhl;
    for _ in 1..octaves as i32 {
        x *= lacunarity;
        y *= lacunarity;
        z *= lacunarity;
        let mut weight = signal * gain;
        weight = weight.clamp(0.0, 1.0);
        signal = offset - f(x, y, z).abs();
        signal *= signal;
        signal *= weight;
        result += signal * pwr;
        pwr *= pwhl;
    }
    result
}
/// `BLI_noise_mg_variable_lacunarity`.
pub fn noise_mg_variable_lacunarity(x: f32, y: f32, z: f32, distortion: f32, nbas1: i32, nbas2: i32) -> f32 {
    let (f1, _) = noise_func(nbas1, true);
    let (f2, _) = noise_func(nbas2, true);
    let rv = [
        f1(x + 13.5, y + 13.5, z + 13.5) * distortion,
        f1(x, y, z) * distortion,
        f1(x - 13.5, y - 13.5, z - 13.5) * distortion,
    ];
    f2(x + rv[0], y + rv[1], z + rv[2])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deterministic_and_bounded() {
        assert_eq!(noise_hnoise(1.0, 0.5, 0.5, 0.5), noise_hnoise(1.0, 0.5, 0.5, 0.5));
        let v = noise3_perlin([0.3, 0.7, 1.2]);
        assert!(v.abs() <= 1.6);
        let (da, _) = noise_voronoi(0.4, 0.5, 0.6, 1.0, 0);
        assert!(da[0] <= da[1] && da[1] <= da[2] && da[2] <= da[3]);
        let n = noise_generic_noise(0.5, 0.1, 0.2, 0.3, false, 1);
        assert!((0.0..=1.0).contains(&n));
        let m = noise_mg_fbm(0.1, 0.2, 0.3, 1.0, 2.0, 3.0, 0);
        assert!(m.is_finite());
    }
}
