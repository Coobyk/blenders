//! Port of `noise.cc` — the modern `blender::noise` module:
//! Jenkins lookup3 + PCG hashes, 1-4D Perlin noise (signed/positive/fbm/fractal
//! variants/distorted), 1-4D Voronoi (F1/F2/Smooth-F1/DistanceToEdge/NSphereRadius),
//! Gabor noise, and the mask/VArray batch wrappers.
//!
//! PORT_NOTE: upstream's `VArray<T>`/`SoaPtr`/`multi_eval` machinery is represented
//! by `VArrayRead<T>` (single value / slice / closure accessor) and per-index
//! element-wise evaluation; the chunked SoA fast paths and `exec_presets` don't
//! apply. `perlin_split_mask` is implemented directly over mask indices.

use blenders_math::vector::{Vec2, Vec3, Vec4};
use std::collections::HashMap;

use crate::index_mask::IndexMask;

/* -------------------------------------------------------------------- */
/** Jenkins Lookup3 hash functions. */

#[inline]
fn hash_bit_rotate(x: u32, k: u32) -> u32 {
    x.rotate_left(k)
}
#[inline]
fn hash_bit_mix(a: &mut u32, b: &mut u32, c: &mut u32) {
    *a = a.wrapping_sub(*c);
    *a ^= hash_bit_rotate(*c, 4);
    *c = c.wrapping_add(*b);
    *b = b.wrapping_sub(*a);
    *b ^= hash_bit_rotate(*a, 6);
    *a = a.wrapping_add(*c);
    *c = c.wrapping_sub(*b);
    *c ^= hash_bit_rotate(*b, 8);
    *b = b.wrapping_add(*a);
    *a = a.wrapping_sub(*c);
    *a ^= hash_bit_rotate(*c, 16);
    *c = c.wrapping_add(*b);
    *b = b.wrapping_sub(*a);
    *b ^= hash_bit_rotate(*a, 19);
    *a = a.wrapping_add(*c);
    *c = c.wrapping_sub(*b);
    *c ^= hash_bit_rotate(*b, 4);
    *b = b.wrapping_add(*a);
}
#[inline]
fn hash_bit_final(a: &mut u32, b: &mut u32, c: &mut u32) {
    *c ^= *b;
    *c = c.wrapping_sub(hash_bit_rotate(*b, 14));
    *a ^= *c;
    *a = a.wrapping_sub(hash_bit_rotate(*c, 11));
    *b ^= *a;
    *b = b.wrapping_sub(hash_bit_rotate(*a, 25));
    *c ^= *b;
    *c = c.wrapping_sub(hash_bit_rotate(*b, 16));
    *a ^= *c;
    *a = a.wrapping_sub(hash_bit_rotate(*c, 4));
    *b ^= *a;
    *b = b.wrapping_sub(hash_bit_rotate(*a, 14));
    *c ^= *b;
    *c = c.wrapping_sub(hash_bit_rotate(*b, 24));
}

pub fn hash1(kx: u32) -> u32 {
    let mut a = 0xdeadbeefu32.wrapping_add((1 << 2) + 13);
    let (mut b, mut c) = (a, a);
    a = a.wrapping_add(kx);
    hash_bit_final(&mut a, &mut b, &mut c);
    c
}
pub fn hash2(kx: u32, ky: u32) -> u32 {
    let mut a = 0xdeadbeefu32.wrapping_add((2 << 2) + 13);
    let (mut b, mut c) = (a, a);
    b = b.wrapping_add(ky);
    a = a.wrapping_add(kx);
    hash_bit_final(&mut a, &mut b, &mut c);
    c
}
pub fn hash3(kx: u32, ky: u32, kz: u32) -> u32 {
    let mut a = 0xdeadbeefu32.wrapping_add((3 << 2) + 13);
    let (mut b, mut c) = (a, a);
    c = c.wrapping_add(kz);
    b = b.wrapping_add(ky);
    a = a.wrapping_add(kx);
    hash_bit_final(&mut a, &mut b, &mut c);
    c
}
pub fn hash4(kx: u32, ky: u32, kz: u32, kw: u32) -> u32 {
    let mut a = 0xdeadbeefu32.wrapping_add((4 << 2) + 13);
    let (mut b, mut c) = (a, a);
    a = a.wrapping_add(kx);
    b = b.wrapping_add(ky);
    c = c.wrapping_add(kz);
    hash_bit_mix(&mut a, &mut b, &mut c);
    a = a.wrapping_add(kw);
    hash_bit_final(&mut a, &mut b, &mut c);
    c
}

#[inline]
fn float_as_uint(f: f32) -> u32 {
    f.to_bits()
}

/* PCG 2D/3D/4D hashes — signed arithmetic matching OSL. */

fn hash_pcg2d_i(mut v: [i32; 2]) -> [i32; 2] {
    v[0] = v[0].wrapping_mul(1664525).wrapping_add(1013904223);
    v[1] = v[1].wrapping_mul(1664525).wrapping_add(1013904223);
    v[0] = v[0].wrapping_add(v[1].wrapping_mul(1664525));
    v[1] = v[1].wrapping_add(v[0].wrapping_mul(1664525));
    v[0] ^= v[0] >> 16;
    v[1] ^= v[1] >> 16;
    v[0] = v[0].wrapping_add(v[1].wrapping_mul(1664525));
    v[1] = v[1].wrapping_add(v[0].wrapping_mul(1664525));
    v
}
fn hash_pcg3d_i(mut v: [i32; 3]) -> [i32; 3] {
    for c in v.iter_mut() {
        *c = c.wrapping_mul(1664525).wrapping_add(1013904223);
    }
    v[0] = v[0].wrapping_add(v[1].wrapping_mul(v[2]));
    v[1] = v[1].wrapping_add(v[2].wrapping_mul(v[0]));
    v[2] = v[2].wrapping_add(v[0].wrapping_mul(v[1]));
    for c in v.iter_mut() {
        *c ^= *c >> 16;
    }
    v[0] = v[0].wrapping_add(v[1].wrapping_mul(v[2]));
    v[1] = v[1].wrapping_add(v[2].wrapping_mul(v[0]));
    v[2] = v[2].wrapping_add(v[0].wrapping_mul(v[1]));
    v
}
fn hash_pcg4d_i(mut v: [i32; 4]) -> [i32; 4] {
    for c in v.iter_mut() {
        *c = c.wrapping_mul(1664525).wrapping_add(1013904223);
    }
    v[0] = v[0].wrapping_add(v[1].wrapping_mul(v[3]));
    v[1] = v[1].wrapping_add(v[2].wrapping_mul(v[0]));
    v[2] = v[2].wrapping_add(v[0].wrapping_mul(v[1]));
    v[3] = v[3].wrapping_add(v[1].wrapping_mul(v[2]));
    for c in v.iter_mut() {
        *c ^= *c >> 16;
    }
    v[0] = v[0].wrapping_add(v[1].wrapping_mul(v[3]));
    v[1] = v[1].wrapping_add(v[2].wrapping_mul(v[0]));
    v[2] = v[2].wrapping_add(v[0].wrapping_mul(v[1]));
    v[3] = v[3].wrapping_add(v[1].wrapping_mul(v[2]));
    v
}

fn hash_float_1(kx: f32) -> u32 {
    hash1(float_as_uint(kx))
}
fn hash_float_2(k: Vec2) -> u32 {
    hash2(float_as_uint(k.x), float_as_uint(k.y))
}
fn hash_float_3(k: Vec3) -> u32 {
    hash3(float_as_uint(k.x), float_as_uint(k.y), float_as_uint(k.z))
}
fn hash_float_4(k: Vec4) -> u32 {
    hash4(
        float_as_uint(k.x),
        float_as_uint(k.y),
        float_as_uint(k.z),
        float_as_uint(k.w),
    )
}

pub fn uint_to_float_01(k: u32) -> f32 {
    k as f32 / 0xFFFFFFFFu32 as f32
}
fn hash_to_float_1(kx: u32) -> f32 {
    uint_to_float_01(hash1(kx))
}
fn hash_to_float_2(kx: u32, ky: u32) -> f32 {
    uint_to_float_01(hash2(kx, ky))
}
fn hash_to_float_3(kx: u32, ky: u32, kz: u32) -> f32 {
    uint_to_float_01(hash3(kx, ky, kz))
}
fn hash_to_float_4(kx: u32, ky: u32, kz: u32, kw: u32) -> f32 {
    uint_to_float_01(hash4(kx, ky, kz, kw))
}

pub fn hash_float_to_float(k: f32) -> f32 {
    uint_to_float_01(hash_float_1(k))
}
pub fn hash_float_to_float2(k: Vec2) -> f32 {
    uint_to_float_01(hash_float_2(k))
}
pub fn hash_float_to_float3(k: Vec3) -> f32 {
    uint_to_float_01(hash_float_3(k))
}
pub fn hash_float_to_float4(k: Vec4) -> f32 {
    uint_to_float_01(hash_float_4(k))
}

pub fn hash_float2_to_float2(k: Vec2) -> Vec2 {
    Vec2::new(
        hash_float_to_float2(k),
        hash_float_to_float3(Vec3::new(k.x, k.y, 1.0)),
    )
}
pub fn hash_float3_to_float2(k: Vec3) -> Vec2 {
    Vec2::new(
        hash_float_to_float3(Vec3::new(k.x, k.y, k.z)),
        hash_float_to_float3(Vec3::new(k.z, k.x, k.y)),
    )
}
pub fn hash_float4_to_float2(k: Vec4) -> Vec2 {
    Vec2::new(
        hash_float_to_float4(Vec4::new(k.x, k.y, k.z, k.w)),
        hash_float_to_float4(Vec4::new(k.z, k.x, k.w, k.y)),
    )
}
pub fn hash_float1_to_float3(k: f32) -> Vec3 {
    Vec3::new(
        hash_float_to_float(k),
        hash_float_to_float2(Vec2::new(k, 1.0)),
        hash_float_to_float2(Vec2::new(k, 2.0)),
    )
}
pub fn hash_float2_to_float3(k: Vec2) -> Vec3 {
    Vec3::new(
        hash_float_to_float2(k),
        hash_float_to_float3(Vec3::new(k.x, k.y, 1.0)),
        hash_float_to_float3(Vec3::new(k.x, k.y, 2.0)),
    )
}
pub fn hash_float3_to_float3(k: Vec3) -> Vec3 {
    Vec3::new(
        hash_float_to_float3(k),
        hash_float_to_float4(Vec4::new(k.x, k.y, k.z, 1.0)),
        hash_float_to_float4(Vec4::new(k.x, k.y, k.z, 2.0)),
    )
}
pub fn hash_float4_to_float3(k: Vec4) -> Vec3 {
    Vec3::new(
        hash_float_to_float4(k),
        hash_float_to_float4(Vec4::new(k.z, k.x, k.w, k.y)),
        hash_float_to_float4(Vec4::new(k.w, k.z, k.y, k.x)),
    )
}
pub fn hash_float4_to_float4(k: Vec4) -> Vec4 {
    Vec4::new(
        hash_float_to_float4(k),
        hash_float_to_float4(Vec4::new(k.w, k.x, k.y, k.z)),
        hash_float_to_float4(Vec4::new(k.z, k.w, k.x, k.y)),
        hash_float_to_float4(Vec4::new(k.y, k.z, k.w, k.x)),
    )
}

fn int_to_float_01(k: i32) -> f32 {
    (k & 0x7fffffff) as f32 * (1.0 / 0x7fffffff as f32)
}
fn hash_int2_to_float2(k: [i32; 2]) -> Vec2 {
    let h = hash_pcg2d_i(k);
    Vec2::new(int_to_float_01(h[0]), int_to_float_01(h[1]))
}
fn hash_int3_to_float3(k: [i32; 3]) -> Vec3 {
    let h = hash_pcg3d_i(k);
    Vec3::new(
        int_to_float_01(h[0]),
        int_to_float_01(h[1]),
        int_to_float_01(h[2]),
    )
}
fn hash_int4_to_float4(k: [i32; 4]) -> Vec4 {
    let h = hash_pcg4d_i(k);
    Vec4::new(
        int_to_float_01(h[0]),
        int_to_float_01(h[1]),
        int_to_float_01(h[2]),
        int_to_float_01(h[3]),
    )
}
fn hash_int2_to_float3(k: [i32; 2]) -> Vec3 {
    hash_int3_to_float3([k[0], k[1], 0])
}
fn hash_int4_to_float3(k: [i32; 4]) -> Vec3 {
    let v = hash_int4_to_float4(k);
    Vec3::new(v.x, v.y, v.z)
}

/* -------------------------------------------------------------------- */
/** Perlin noise. */

#[inline]
fn mixf(v0: f32, v1: f32, x: f32) -> f32 {
    (1.0 - x) * v0 + x * v1
}
trait Mix<T> {
    fn mix(self, v1: Self, x: f32) -> Self;
}
impl Mix<f32> for f32 {
    fn mix(self, v1: Self, x: f32) -> Self {
        mixf(self, v1, x)
    }
}
macro_rules! impl_mix_vec {
    ($T:ty, $($c:ident),+) => {
        impl Mix<$T> for $T {
            fn mix(self, v1: Self, x: f32) -> Self {
                let mut out = self;
                $(out.$c = mixf(self.$c, v1.$c, x);)+
                out
            }
        }
    };
}
impl_mix_vec!(Vec2, x, y);
impl_mix_vec!(Vec3, x, y, z);
impl_mix_vec!(Vec4, x, y, z, w);
fn mixv<T: Mix<T>>(v0: T, v1: T, x: f32) -> T {
    v0.mix(v1, x)
}
#[inline]
fn mix4(v0: f32, v1: f32, v2: f32, v3: f32, x: f32, y: f32) -> f32 {
    let x1 = 1.0 - x;
    (1.0 - y) * (v0 * x1 + v1 * x) + y * (v2 * x1 + v3 * x)
}
#[allow(clippy::too_many_arguments)]
#[inline]
fn mix8(v0: f32, v1: f32, v2: f32, v3: f32, v4: f32, v5: f32, v6: f32, v7: f32, x: f32, y: f32, z: f32) -> f32 {
    let x1 = 1.0 - x;
    let y1 = 1.0 - y;
    let z1 = 1.0 - z;
    z1 * (y1 * (v0 * x1 + v1 * x) + y * (v2 * x1 + v3 * x))
        + z * (y1 * (v4 * x1 + v5 * x) + y * (v6 * x1 + v7 * x))
}
#[allow(clippy::too_many_arguments)]
#[inline]
fn mix16(v: [f32; 16], x: f32, y: f32, z: f32, w: f32) -> f32 {
    mixf(
        mix8(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], x, y, z),
        mix8(v[8], v[9], v[10], v[11], v[12], v[13], v[14], v[15], x, y, z),
        w,
    )
}
#[inline]
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}
#[inline]
fn negate_if(value: f32, condition: u32) -> f32 {
    f32::from_bits(value.to_bits() ^ (((condition != 0) as u32) << 31))
}
#[inline]
fn select_float(condition: bool, true_value: f32, false_value: f32) -> f32 {
    let c = condition as i32 as f32;
    c * true_value + (1.0 - c) * false_value
}
#[inline]
fn noise_grad1(hash: u32, x: f32) -> f32 {
    let h = hash & 15;
    let g = (1 + (h & 7)) as f32;
    negate_if(g, h & 8) * x
}
#[inline]
fn noise_grad2(hash: u32, x: f32, y: f32) -> f32 {
    let h = hash & 7;
    let u = select_float(h < 4, x, y);
    let v = 2.0 * select_float(h < 4, y, x);
    negate_if(u, h & 1) + negate_if(v, h & 2)
}
#[inline]
fn noise_grad3(hash: u32, x: f32, y: f32, z: f32) -> f32 {
    let h = hash & 15;
    let u = select_float(h < 8, x, y);
    let vt = select_float((h & 13) == 12, x, z);
    let v = select_float(h < 4, y, vt);
    negate_if(u, h & 1) + negate_if(v, h & 2)
}
#[inline]
fn noise_grad4(hash: u32, x: f32, y: f32, z: f32, w: f32) -> f32 {
    let h = hash & 31;
    let u = select_float(h < 24, x, y);
    let v = select_float(h < 16, y, z);
    let s = select_float(h < 8, z, w);
    negate_if(u, h & 1) + negate_if(v, h & 2) + negate_if(s, h & 4)
}
#[inline]
fn floor_to_int(x: f32) -> i32 {
    let i = x as i32;
    i - ((x < i as f32) as i32)
}
#[inline]
fn floor_fraction(x: f32, i: &mut i32) -> f32 {
    *i = floor_to_int(x);
    x - *i as f32
}

fn perlin_noise_1(position: f32) -> f32 {
    let mut x = 0;
    let fx = floor_fraction(position, &mut x);
    let u = fade(fx);
    mixf(
        noise_grad1(hash1(x as u32), fx),
        noise_grad1(hash1((x + 1) as u32), fx - 1.0),
        u,
    )
}
fn perlin_noise_2(position: Vec2) -> f32 {
    let (mut x, mut y) = (0, 0);
    let fx = floor_fraction(position.x, &mut x);
    let fy = floor_fraction(position.y, &mut y);
    let (u, v) = (fade(fx), fade(fy));
    mix4(
        noise_grad2(hash2(x as u32, y as u32), fx, fy),
        noise_grad2(hash2((x + 1) as u32, y as u32), fx - 1.0, fy),
        noise_grad2(hash2(x as u32, (y + 1) as u32), fx, fy - 1.0),
        noise_grad2(hash2((x + 1) as u32, (y + 1) as u32), fx - 1.0, fy - 1.0),
        u,
        v,
    )
}
fn perlin_noise_3(position: Vec3) -> f32 {
    let (mut x, mut y, mut z) = (0, 0, 0);
    let fx = floor_fraction(position.x, &mut x);
    let fy = floor_fraction(position.y, &mut y);
    let fz = floor_fraction(position.z, &mut z);
    let (u, v, w) = (fade(fx), fade(fy), fade(fz));
    let h = |dx: i32, dy: i32, dz: i32| hash3((x + dx) as u32, (y + dy) as u32, (z + dz) as u32);
    mix8(
        noise_grad3(h(0, 0, 0), fx, fy, fz),
        noise_grad3(h(1, 0, 0), fx - 1.0, fy, fz),
        noise_grad3(h(0, 1, 0), fx, fy - 1.0, fz),
        noise_grad3(h(1, 1, 0), fx - 1.0, fy - 1.0, fz),
        noise_grad3(h(0, 0, 1), fx, fy, fz - 1.0),
        noise_grad3(h(1, 0, 1), fx - 1.0, fy, fz - 1.0),
        noise_grad3(h(0, 1, 1), fx, fy - 1.0, fz - 1.0),
        noise_grad3(h(1, 1, 1), fx - 1.0, fy - 1.0, fz - 1.0),
        u,
        v,
        w,
    )
}
fn perlin_noise_4(position: Vec4) -> f32 {
    let (mut x, mut y, mut z, mut w) = (0, 0, 0, 0);
    let fx = floor_fraction(position.x, &mut x);
    let fy = floor_fraction(position.y, &mut y);
    let fz = floor_fraction(position.z, &mut z);
    let fw = floor_fraction(position.w, &mut w);
    let (u, v, t, s) = (fade(fx), fade(fy), fade(fz), fade(fw));
    let h = |dx: i32, dy: i32, dz: i32, dw: i32| {
        hash4(
            (x + dx) as u32,
            (y + dy) as u32,
            (z + dz) as u32,
            (w + dw) as u32,
        )
    };
    let g = |dx: i32, dy: i32, dz: i32, dw: i32, fx_: f32, fy_: f32, fz_: f32, fw_: f32| {
        noise_grad4(h(dx, dy, dz, dw), fx_, fy_, fz_, fw_)
    };
    mix16(
        [
            g(0, 0, 0, 0, fx, fy, fz, fw),
            g(1, 0, 0, 0, fx - 1.0, fy, fz, fw),
            g(0, 1, 0, 0, fx, fy - 1.0, fz, fw),
            g(1, 1, 0, 0, fx - 1.0, fy - 1.0, fz, fw),
            g(0, 0, 1, 0, fx, fy, fz - 1.0, fw),
            g(1, 0, 1, 0, fx - 1.0, fy, fz - 1.0, fw),
            g(0, 1, 1, 0, fx, fy - 1.0, fz - 1.0, fw),
            g(1, 1, 1, 0, fx - 1.0, fy - 1.0, fz - 1.0, fw),
            g(0, 0, 0, 1, fx, fy, fz, fw - 1.0),
            g(1, 0, 0, 1, fx - 1.0, fy, fz, fw - 1.0),
            g(0, 1, 0, 1, fx, fy - 1.0, fz, fw - 1.0),
            g(1, 1, 0, 1, fx - 1.0, fy - 1.0, fz, fw - 1.0),
            g(0, 0, 1, 1, fx, fy, fz - 1.0, fw - 1.0),
            g(1, 0, 1, 1, fx - 1.0, fy, fz - 1.0, fw - 1.0),
            g(0, 1, 1, 1, fx, fy - 1.0, fz - 1.0, fw - 1.0),
            g(1, 1, 1, 1, fx - 1.0, fy - 1.0, fz - 1.0, fw - 1.0),
        ],
        u,
        v,
        t,
        s,
    )
}

fn precision_correction_1(x: f32) -> f32 {
    let correction = 0.5 * ((x.abs() >= 1000000.0) as i32 as f32);
    x % 100000.0 + correction
}
fn precision_correction_2(p: Vec2) -> Vec2 {
    Vec2::new(
        precision_correction_1(p.x),
        precision_correction_1(p.y),
    )
}
fn precision_correction_3(p: Vec3) -> Vec3 {
    Vec3::new(
        precision_correction_1(p.x),
        precision_correction_1(p.y),
        precision_correction_1(p.z),
    )
}
fn precision_correction_4(p: Vec4) -> Vec4 {
    Vec4::new(
        precision_correction_1(p.x),
        precision_correction_1(p.y),
        precision_correction_1(p.z),
        precision_correction_1(p.w),
    )
}

/// Dimension-generic noise input (upstream `template<typename T>`).
pub trait NoisePos:
    Copy + std::ops::Add<Output = Self> + std::ops::AddAssign + std::ops::Mul<f32, Output = Self>
{
    const SCALE: f32;
    fn perlin_noise(self) -> f32;
    fn corrected(self) -> Self;
    fn distortion(self, strength: f32) -> Self;
    fn offset1() -> Self;
    fn offset2() -> Self;
}
impl NoisePos for f32 {
    const SCALE: f32 = 0.2500;
    fn perlin_noise(self) -> f32 {
        perlin_noise_1(self)
    }
    fn corrected(self) -> Self {
        precision_correction_1(self)
    }
    fn distortion(self, strength: f32) -> Self {
        self + perlin_signed_1(self + random_float_offset(0.0)) * strength
            - self // position += perlin_distortion; distortion vec added below
    }
    fn offset1() -> Self {
        random_float_offset(1.0)
    }
    fn offset2() -> Self {
        random_float_offset(2.0)
    }
}
impl NoisePos for Vec2 {
    const SCALE: f32 = 0.6616;
    fn perlin_noise(self) -> f32 {
        perlin_noise_2(self)
    }
    fn corrected(self) -> Self {
        precision_correction_2(self)
    }
    fn distortion(self, strength: f32) -> Self {
        Vec2::new(
            perlin_signed_2(self + random_float2_offset(0.0)) * strength,
            perlin_signed_2(self + random_float2_offset(1.0)) * strength,
        )
    }
    fn offset1() -> Self {
        random_float2_offset(2.0)
    }
    fn offset2() -> Self {
        random_float2_offset(3.0)
    }
}
impl NoisePos for Vec3 {
    const SCALE: f32 = 0.9820;
    fn perlin_noise(self) -> f32 {
        perlin_noise_3(self)
    }
    fn corrected(self) -> Self {
        precision_correction_3(self)
    }
    fn distortion(self, strength: f32) -> Self {
        Vec3::new(
            perlin_signed_3(self + random_float3_offset(0.0)) * strength,
            perlin_signed_3(self + random_float3_offset(1.0)) * strength,
            perlin_signed_3(self + random_float3_offset(2.0)) * strength,
        )
    }
    fn offset1() -> Self {
        random_float3_offset(3.0)
    }
    fn offset2() -> Self {
        random_float3_offset(4.0)
    }
}
impl NoisePos for Vec4 {
    const SCALE: f32 = 0.8344;
    fn perlin_noise(self) -> f32 {
        perlin_noise_4(self)
    }
    fn corrected(self) -> Self {
        precision_correction_4(self)
    }
    fn distortion(self, strength: f32) -> Self {
        Vec4::new(
            perlin_signed_4(self + random_float4_offset(0.0)) * strength,
            perlin_signed_4(self + random_float4_offset(1.0)) * strength,
            perlin_signed_4(self + random_float4_offset(2.0)) * strength,
            perlin_signed_4(self + random_float4_offset(3.0)) * strength,
        )
    }
    fn offset1() -> Self {
        random_float4_offset(4.0)
    }
    fn offset2() -> Self {
        random_float4_offset(5.0)
    }
}

fn perlin_signed_1(position: f32) -> f32 {
    perlin_noise_1(precision_correction_1(position)) * <f32 as NoisePos>::SCALE
}
fn perlin_signed_2(position: Vec2) -> f32 {
    perlin_noise_2(precision_correction_2(position)) * <Vec2 as NoisePos>::SCALE
}
fn perlin_signed_3(position: Vec3) -> f32 {
    perlin_noise_3(precision_correction_3(position)) * <Vec3 as NoisePos>::SCALE
}
fn perlin_signed_4(position: Vec4) -> f32 {
    perlin_noise_4(precision_correction_4(position)) * <Vec4 as NoisePos>::SCALE
}
pub fn perlin_signed<T: NoisePos>(position: T) -> f32 {
    position.corrected().perlin_noise() * T::SCALE
}
/// `noise::perlin` — positive [0,1] Perlin.
pub fn perlin<T: NoisePos>(position: T) -> f32 {
    perlin_signed(position) / 2.0 + 0.5
}

/// `perlin_fbm`.
pub fn perlin_fbm<T: NoisePos>(
    p: T,
    detail: f32,
    roughness: f32,
    lacunarity: f32,
    normalize: bool,
) -> f32 {
    let mut fscale = 1.0f32;
    let mut amp = 1.0f32;
    let mut maxamp = 0.0f32;
    let mut sum = 0.0f32;
    for _ in 0..=detail as i32 {
        let t = perlin_signed(p * fscale);
        sum += t * amp;
        maxamp += amp;
        amp *= roughness;
        fscale *= lacunarity;
    }
    let rmd = detail - detail.floor();
    if rmd != 0.0 {
        let t = perlin_signed(p * fscale);
        let sum2 = sum + t * amp;
        if normalize {
            mixf(0.5 * sum / maxamp + 0.5, 0.5 * sum2 / (maxamp + amp) + 0.5, rmd)
        } else {
            mixf(sum, sum2, rmd)
        }
    } else if normalize {
        0.5 * sum / maxamp + 0.5
    } else {
        sum
    }
}
/// `perlin_multi_fractal`.
pub fn perlin_multi_fractal<T: NoisePos>(
    mut p: T,
    detail: f32,
    roughness: f32,
    lacunarity: f32,
) -> f32 {
    let mut value = 1.0f32;
    let mut pwr = 1.0f32;
    for _ in 0..=detail as i32 {
        value *= pwr * perlin_signed(p) + 1.0;
        pwr *= roughness;
        p = p * lacunarity;
    }
    let rmd = detail - detail.floor();
    if rmd != 0.0 {
        value *= rmd * pwr * perlin_signed(p) + 1.0;
    }
    value
}
/// `perlin_hetero_terrain`.
pub fn perlin_hetero_terrain<T: NoisePos>(
    mut p: T,
    detail: f32,
    roughness: f32,
    lacunarity: f32,
    offset: f32,
) -> f32 {
    let mut pwr = roughness;
    let mut value = offset + perlin_signed(p);
    p = p * lacunarity;
    for _ in 1..=detail as i32 {
        let increment = (perlin_signed(p) + offset) * pwr * value;
        value += increment;
        pwr *= roughness;
        p = p * lacunarity;
    }
    let rmd = detail - detail.floor();
    if rmd != 0.0 {
        let increment = (perlin_signed(p) + offset) * pwr * value;
        value += rmd * increment;
    }
    value
}
/// `perlin_hybrid_multi_fractal`.
pub fn perlin_hybrid_multi_fractal<T: NoisePos>(
    mut p: T,
    detail: f32,
    roughness: f32,
    lacunarity: f32,
    offset: f32,
    gain: f32,
) -> f32 {
    let mut pwr = 1.0f32;
    let mut value = 0.0f32;
    let mut weight = 1.0f32;
    let mut i = 0;
    while weight > 0.001 && i <= detail as i32 {
        weight = weight.min(1.0);
        let signal = (perlin_signed(p) + offset) * pwr;
        pwr *= roughness;
        value += weight * signal;
        weight *= gain * signal;
        p = p * lacunarity;
        i += 1;
    }
    let rmd = detail - detail.floor();
    if rmd != 0.0 && weight > 0.001 {
        weight = weight.min(1.0);
        let signal = (perlin_signed(p) + offset) * pwr;
        value += rmd * weight * signal;
    }
    value
}
/// `perlin_ridged_multi_fractal`.
pub fn perlin_ridged_multi_fractal<T: NoisePos>(
    mut p: T,
    detail: f32,
    roughness: f32,
    lacunarity: f32,
    offset: f32,
    gain: f32,
) -> f32 {
    let mut pwr = roughness;
    let mut signal = offset - perlin_signed(p).abs();
    signal *= signal;
    let mut value = signal;
    for _ in 1..=detail as i32 {
        p = p * lacunarity;
        let weight = (signal * gain).clamp(0.0, 1.0);
        signal = offset - perlin_signed(p).abs();
        signal *= signal;
        signal *= weight;
        value += signal * pwr;
        pwr *= roughness;
    }
    value
}

pub const NOISE_SHD_PERLIN_MULTIFRACTAL: i32 = 0;
pub const NOISE_SHD_PERLIN_FBM: i32 = 1;
pub const NOISE_SHD_PERLIN_HYBRID_MULTIFRACTAL: i32 = 2;
pub const NOISE_SHD_PERLIN_RIDGED_MULTIFRACTAL: i32 = 3;
pub const NOISE_SHD_PERLIN_HETERO_TERRAIN: i32 = 4;

/// `perlin_select`.
pub fn perlin_select<T: NoisePos>(
    p: T,
    detail: f32,
    roughness: f32,
    lacunarity: f32,
    offset: f32,
    gain: f32,
    type_: i32,
    normalize: bool,
) -> f32 {
    match type_ {
        NOISE_SHD_PERLIN_MULTIFRACTAL => perlin_multi_fractal(p, detail, roughness, lacunarity),
        NOISE_SHD_PERLIN_FBM => perlin_fbm(p, detail, roughness, lacunarity, normalize),
        NOISE_SHD_PERLIN_HYBRID_MULTIFRACTAL => {
            perlin_hybrid_multi_fractal(p, detail, roughness, lacunarity, offset, gain)
        }
        NOISE_SHD_PERLIN_RIDGED_MULTIFRACTAL => {
            perlin_ridged_multi_fractal(p, detail, roughness, lacunarity, offset, gain)
        }
        NOISE_SHD_PERLIN_HETERO_TERRAIN => {
            perlin_hetero_terrain(p, detail, roughness, lacunarity, offset)
        }
        _ => 0.0,
    }
}

/* Random float offsets used as seeds. */

pub fn random_float_offset(seed: f32) -> f32 {
    100.0 + hash_float_to_float(seed) * 100.0
}
pub fn random_float2_offset(seed: f32) -> Vec2 {
    Vec2::new(
        100.0 + hash_float_to_float2(Vec2::new(seed, 0.0)) * 100.0,
        100.0 + hash_float_to_float2(Vec2::new(seed, 1.0)) * 100.0,
    )
}
pub fn random_float3_offset(seed: f32) -> Vec3 {
    Vec3::new(
        100.0 + hash_float_to_float2(Vec2::new(seed, 0.0)) * 100.0,
        100.0 + hash_float_to_float2(Vec2::new(seed, 1.0)) * 100.0,
        100.0 + hash_float_to_float2(Vec2::new(seed, 2.0)) * 100.0,
    )
}
pub fn random_float4_offset(seed: f32) -> Vec4 {
    Vec4::new(
        100.0 + hash_float_to_float2(Vec2::new(seed, 0.0)) * 100.0,
        100.0 + hash_float_to_float2(Vec2::new(seed, 1.0)) * 100.0,
        100.0 + hash_float_to_float2(Vec2::new(seed, 2.0)) * 100.0,
        100.0 + hash_float_to_float2(Vec2::new(seed, 3.0)) * 100.0,
    )
}

pub fn perlin_distortion<T: NoisePos>(position: T, strength: f32) -> T {
    position.distortion(strength)
}
/// `perlin_fractal_distorted`.
pub fn perlin_fractal_distorted<T: NoisePos>(
    mut position: T,
    detail: f32,
    roughness: f32,
    lacunarity: f32,
    offset: f32,
    gain: f32,
    distortion: f32,
    type_: i32,
    normalize: bool,
) -> f32 {
    position += position.distortion(distortion);
    perlin_select(position, detail, roughness, lacunarity, offset, gain, type_, normalize)
}
/// `perlin_float3_fractal_distorted`.
pub fn perlin_float3_fractal_distorted<T: NoisePos>(
    mut position: T,
    detail: f32,
    roughness: f32,
    lacunarity: f32,
    offset: f32,
    gain: f32,
    distortion: f32,
    type_: i32,
    normalize: bool,
) -> Vec3 {
    position += position.distortion(distortion);
    Vec3::new(
        perlin_select(position, detail, roughness, lacunarity, offset, gain, type_, normalize),
        perlin_select(
            position + T::offset1(),
            detail,
            roughness,
            lacunarity,
            offset,
            gain,
            type_,
            normalize,
        ),
        perlin_select(
            position + T::offset2(),
            detail,
            roughness,
            lacunarity,
            offset,
            gain,
            type_,
            normalize,
        ),
    )
}

/* -------------------------------------------------------------------- */
/** Voronoi noise. */

pub const NOISE_SHD_VORONOI_EUCLIDEAN: i32 = 0;
pub const NOISE_SHD_VORONOI_MANHATTAN: i32 = 1;
pub const NOISE_SHD_VORONOI_CHEBYCHEV: i32 = 2;
pub const NOISE_SHD_VORONOI_MINKOWSKI: i32 = 3;

pub const NOISE_SHD_VORONOI_F1: i32 = 0;
pub const NOISE_SHD_VORONOI_F2: i32 = 1;
pub const NOISE_SHD_VORONOI_SMOOTH_F1: i32 = 2;
pub const NOISE_SHD_VORONOI_DISTANCE_TO_EDGE: i32 = 3;
pub const NOISE_SHD_VORONOI_N_SPHERE_RADIUS: i32 = 4;

#[derive(Clone, Copy, Default)]
pub struct VoronoiParams {
    pub scale: f32,
    pub detail: f32,
    pub roughness: f32,
    pub lacunarity: f32,
    pub smoothness: f32,
    pub exponent: f32,
    pub randomness: f32,
    pub max_distance: f32,
    pub normalize: bool,
    pub feature: i32,
    pub metric: i32,
}
#[derive(Clone, Copy, Default)]
pub struct VoronoiOutput {
    pub distance: f32,
    pub color: Vec3,
    pub position: Vec4,
}

fn voronoi_distance_1(a: f32, b: f32) -> f32 {
    (b - a).abs()
}
fn voronoi_distance_2(a: Vec2, b: Vec2, params: &VoronoiParams) -> f32 {
    match params.metric {
        NOISE_SHD_VORONOI_EUCLIDEAN => ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt(),
        NOISE_SHD_VORONOI_MANHATTAN => (a.x - b.x).abs() + (a.y - b.y).abs(),
        NOISE_SHD_VORONOI_CHEBYCHEV => (a.x - b.x).abs().max((a.y - b.y).abs()),
        _ => ((a.x - b.x).abs().powf(params.exponent) + (a.y - b.y).abs().powf(params.exponent))
            .powf(1.0 / params.exponent),
    }
}
fn voronoi_distance_3(a: Vec3, b: Vec3, params: &VoronoiParams) -> f32 {
    match params.metric {
        NOISE_SHD_VORONOI_EUCLIDEAN => {
            ((a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)).sqrt()
        }
        NOISE_SHD_VORONOI_MANHATTAN => {
            (a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs()
        }
        NOISE_SHD_VORONOI_CHEBYCHEV => (a.x - b.x).abs().max((a.y - b.y).abs()).max((a.z - b.z).abs()),
        _ => ((a.x - b.x).abs().powf(params.exponent)
            + (a.y - b.y).abs().powf(params.exponent)
            + (a.z - b.z).abs().powf(params.exponent))
        .powf(1.0 / params.exponent),
    }
}
fn voronoi_distance_4(a: Vec4, b: Vec4, params: &VoronoiParams) -> f32 {
    match params.metric {
        NOISE_SHD_VORONOI_EUCLIDEAN => ((a.x - b.x).powi(2)
            + (a.y - b.y).powi(2)
            + (a.z - b.z).powi(2)
            + (a.w - b.w).powi(2))
        .sqrt(),
        NOISE_SHD_VORONOI_MANHATTAN => {
            (a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs() + (a.w - b.w).abs()
        }
        NOISE_SHD_VORONOI_CHEBYCHEV => (a.x - b.x)
            .abs()
            .max((a.y - b.y).abs())
            .max((a.z - b.z).abs())
            .max((a.w - b.w).abs()),
        _ => ((a.x - b.x).abs().powf(params.exponent)
            + (a.y - b.y).abs().powf(params.exponent)
            + (a.z - b.z).abs().powf(params.exponent)
            + (a.w - b.w).abs().powf(params.exponent))
        .powf(1.0 / params.exponent),
    }
}
fn voronoi_distance_bound_2(a: Vec2, b: Vec2, params: &VoronoiParams) -> f32 {
    if params.metric == NOISE_SHD_VORONOI_EUCLIDEAN {
        (a.x - b.x).powi(2) + (a.y - b.y).powi(2)
    } else {
        voronoi_distance_2(a, b, params)
    }
}
fn voronoi_distance_bound_3(a: Vec3, b: Vec3, params: &VoronoiParams) -> f32 {
    if params.metric == NOISE_SHD_VORONOI_EUCLIDEAN {
        (a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)
    } else {
        voronoi_distance_3(a, b, params)
    }
}
fn voronoi_distance_bound_4(a: Vec4, b: Vec4, params: &VoronoiParams) -> f32 {
    if params.metric == NOISE_SHD_VORONOI_EUCLIDEAN {
        (a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2) + (a.w - b.w).powi(2)
    } else {
        voronoi_distance_4(a, b, params)
    }
}

fn voronoi_position_1(coord: f32) -> Vec4 {
    Vec4::new(0.0, 0.0, 0.0, coord)
}
fn voronoi_position_2(coord: Vec2) -> Vec4 {
    Vec4::new(coord.x, coord.y, 0.0, 0.0)
}
fn voronoi_position_3(coord: Vec3) -> Vec4 {
    Vec4::new(coord.x, coord.y, coord.z, 0.0)
}
fn voronoi_position_4(coord: Vec4) -> Vec4 {
    coord
}

#[inline]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/* 1D */
pub fn voronoi_f1_1d(params: &VoronoiParams, coord: f32) -> VoronoiOutput {
    let cell_position = coord.floor();
    let local_position = coord - cell_position;
    let mut min_distance = f32::MAX;
    let mut target_offset = 0.0;
    let mut target_position = 0.0;
    for i in -1..=1 {
        let cell_offset = i as f32;
        let point_position =
            cell_offset + hash_float_to_float(cell_position + cell_offset) * params.randomness;
        let d = voronoi_distance_1(point_position, local_position);
        if d < min_distance {
            target_offset = cell_offset;
            min_distance = d;
            target_position = point_position;
        }
    }
    VoronoiOutput {
        distance: min_distance,
        color: hash_float1_to_float3(cell_position + target_offset),
        position: voronoi_position_1(target_position + cell_position),
    }
}
pub fn voronoi_smooth_f1_1d(params: &VoronoiParams, coord: f32, calc_color: bool) -> VoronoiOutput {
    let cell_position = coord.floor();
    let local_position = coord - cell_position;
    let mut smooth_distance = 0.0f32;
    let mut smooth_position = 0.0f32;
    let mut smooth_color = Vec3::new(0.0, 0.0, 0.0);
    let mut h = -1.0f32;
    for i in -2..=2 {
        let cell_offset = i as f32;
        let point_position =
            cell_offset + hash_float_to_float(cell_position + cell_offset) * params.randomness;
        let d = voronoi_distance_1(point_position, local_position);
        h = if h == -1.0 {
            1.0
        } else {
            smoothstep(0.0, 1.0, 0.5 + 0.5 * (smooth_distance - d) / params.smoothness)
        };
        let mut correction = params.smoothness * h * (1.0 - h);
        smooth_distance = mixf(smooth_distance, d, h) - correction;
        correction /= 1.0 + 3.0 * params.smoothness;
        let cell_color = hash_float1_to_float3(cell_position + cell_offset);
        if calc_color {
            smooth_color = mixv(smooth_color, cell_color, h) - Vec3::new(correction, correction, correction);
        }
        smooth_position = mixf(smooth_position, point_position, h) - correction;
    }
    VoronoiOutput {
        distance: smooth_distance,
        color: smooth_color,
        position: voronoi_position_1(cell_position + smooth_position),
    }
}
pub fn voronoi_f2_1d(params: &VoronoiParams, coord: f32) -> VoronoiOutput {
    let cell_position = coord.floor();
    let local_position = coord - cell_position;
    let (mut df1, mut df2) = (f32::MAX, f32::MAX);
    let (mut of1, mut of2) = (0.0f32, 0.0f32);
    let (mut pf1, mut pf2) = (0.0f32, 0.0f32);
    for i in -1..=1 {
        let cell_offset = i as f32;
        let point_position =
            cell_offset + hash_float_to_float(cell_position + cell_offset) * params.randomness;
        let d = voronoi_distance_1(point_position, local_position);
        if d < df1 {
            df2 = df1;
            df1 = d;
            of2 = of1;
            of1 = cell_offset;
            pf2 = pf1;
            pf1 = point_position;
        } else if d < df2 {
            df2 = d;
            of2 = cell_offset;
            pf2 = point_position;
        }
    }
    VoronoiOutput {
        distance: df2,
        color: hash_float1_to_float3(cell_position + of2),
        position: voronoi_position_1(pf2 + cell_position),
    }
}
pub fn voronoi_distance_to_edge_1d(params: &VoronoiParams, coord: f32) -> f32 {
    let cell_position = coord.floor();
    let local_position = coord - cell_position;
    let mid = hash_float_to_float(cell_position) * params.randomness;
    let left = -1.0 + hash_float_to_float(cell_position - 1.0) * params.randomness;
    let right = 1.0 + hash_float_to_float(cell_position + 1.0) * params.randomness;
    ((mid + left) / 2.0 - local_position)
        .abs()
        .min(((mid + right) / 2.0 - local_position).abs())
}
pub fn voronoi_n_sphere_radius_1d(params: &VoronoiParams, coord: f32) -> f32 {
    let cell_position = coord.floor();
    let local_position = coord - cell_position;
    let mut closest = 0.0f32;
    let mut closest_offset = 0.0f32;
    let mut min = f32::MAX;
    for i in -1..=1 {
        let cell_offset = i as f32;
        let p = cell_offset + hash_float_to_float(cell_position + cell_offset) * params.randomness;
        let d = (p - local_position).abs();
        if d < min {
            min = d;
            closest = p;
            closest_offset = cell_offset;
        }
    }
    let mut min = f32::MAX;
    let mut closest2 = 0.0f32;
    for i in -1..=1 {
        if i == 0 {
            continue;
        }
        let cell_offset = i as f32 + closest_offset;
        let p = cell_offset + hash_float_to_float(cell_position + cell_offset) * params.randomness;
        let d = (closest - p).abs();
        if d < min {
            min = d;
            closest2 = p;
        }
    }
    (closest2 - closest).abs() / 2.0
}

/* 2D/3D/4D Voronoi — same algorithm per dimension, written out explicitly. */

macro_rules! voronoi_dim_impl {
    (
        $pos:ty,
        floor_v: $floor_v:expr, to_int: $to_int:expr, from_int: $from_int:expr,
        int_color: $int_color:expr, int_hash: $int_hash:expr, dist: $dist:expr, distb: $distb:expr,
        pos4: $pos4:expr, dot: $dot:expr, dims: $dims:literal,
        f1: $f1:ident, smooth: $smooth:ident, f2: $f2:ident, dte: $dte:ident, nsr: $nsr:ident
    ) => {
        pub fn $f1(params: &VoronoiParams, coord: $pos) -> VoronoiOutput {
            let cell_f = $floor_v(coord);
            let local = coord - cell_f;
            let cell: [i32; 4] = $to_int(cell_f);
            let mut min = f32::MAX;
            let mut target_offset = [0i32; 4];
            let mut target = coord - coord;
            voronoi_offsets::<$dims>(1, |off| {
                let cell_i = iadd4(cell, off);
                let h = $int_hash(cell_i);
                let point = $from_int(off) + scale_pos::<$pos>(h, params.randomness);
                let d = $distb(point, local, params);
                if d < min {
                    target_offset = off;
                    min = d;
                    target = point;
                }
            });
            let cell_i = iadd4(cell, target_offset);
            VoronoiOutput {
                distance: $dist(target, local, params),
                color: $int_color(cell_i),
                position: $pos4(target + cell_f),
            }
        }
        pub fn $smooth(params: &VoronoiParams, coord: $pos, calc_color: bool) -> VoronoiOutput {
            let cell_f = $floor_v(coord);
            let local = coord - cell_f;
            let cell: [i32; 4] = $to_int(cell_f);
            let mut smooth_distance = 0.0f32;
            let mut smooth_color = Vec3::new(0.0, 0.0, 0.0);
            let mut smooth_position = coord - coord;
            let mut h = -1.0f32;
            voronoi_offsets::<$dims>(2, |off| {
                let cell_i = iadd4(cell, off);
                let hsh = $int_hash(cell_i);
                let point = $from_int(off) + scale_pos::<$pos>(hsh, params.randomness);
                let d = $dist(point, local, params);
                h = if h == -1.0 {
                    1.0
                } else {
                    smoothstep(0.0, 1.0, 0.5 + 0.5 * (smooth_distance - d) / params.smoothness)
                };
                let mut correction = params.smoothness * h * (1.0 - h);
                smooth_distance = mixf(smooth_distance, d, h) - correction;
                correction /= 1.0 + 3.0 * params.smoothness;
                if calc_color {
                    let cc = $int_color(cell_i);
                    smooth_color =
                        mixv(smooth_color, cc, h) - Vec3::new(correction, correction, correction);
                }
                smooth_position =
                    mixv(smooth_position, point, h) - scale_pos::<$pos>(one_pos::<$pos>(), correction);
            });
            VoronoiOutput {
                distance: smooth_distance,
                color: smooth_color,
                position: $pos4(cell_f + smooth_position),
            }
        }
        pub fn $f2(params: &VoronoiParams, coord: $pos) -> VoronoiOutput {
            let cell_f = $floor_v(coord);
            let local = coord - cell_f;
            let cell: [i32; 4] = $to_int(cell_f);
            let (mut df1, mut df2) = (f32::MAX, f32::MAX);
            let mut of1 = [0i32; 4];
            let mut of2 = [0i32; 4];
            let mut pf1 = coord - coord;
            let mut pf2 = coord - coord;
            voronoi_offsets::<$dims>(1, |off| {
                let cell_i = iadd4(cell, off);
                let hsh = $int_hash(cell_i);
                let point = $from_int(off) + scale_pos::<$pos>(hsh, params.randomness);
                let d = $dist(point, local, params);
                if d < df1 {
                    df2 = df1;
                    df1 = d;
                    of2 = of1;
                    of1 = off;
                    pf2 = pf1;
                    pf1 = point;
                } else if d < df2 {
                    df2 = d;
                    of2 = off;
                    pf2 = point;
                }
            });
            let cell_i = iadd4(cell, of2);
            VoronoiOutput {
                distance: df2,
                color: $int_color(cell_i),
                position: $pos4(pf2 + cell_f),
            }
        }
        pub fn $dte(params: &VoronoiParams, coord: $pos) -> f32 {
            let cell_f = $floor_v(coord);
            let local = coord - cell_f;
            let cell: [i32; 4] = $to_int(cell_f);
            let mut vector_to_closest = coord - coord;
            let mut min = f32::MAX;
            voronoi_offsets::<$dims>(1, |off| {
                let cell_i = iadd4(cell, off);
                let hsh = $int_hash(cell_i);
                let vp = $from_int(off) + scale_pos::<$pos>(hsh, params.randomness) - local;
                let d = $dot(vp, vp);
                if d < min {
                    min = d;
                    vector_to_closest = vp;
                }
            });
            let mut min = f32::MAX;
            voronoi_offsets::<$dims>(1, |off| {
                let cell_i = iadd4(cell, off);
                let hsh = $int_hash(cell_i);
                let vp = $from_int(off) + scale_pos::<$pos>(hsh, params.randomness) - local;
                let perp = vp - vector_to_closest;
                if $dot(perp, perp) > 0.0001 {
                    let n = perp * (1.0 / $dot(perp, perp).sqrt());
                    let mid = (vector_to_closest + vp) * 0.5;
                    let d = $dot(mid, n);
                    if d < min {
                        min = d;
                    }
                }
            });
            min
        }
        pub fn $nsr(params: &VoronoiParams, coord: $pos) -> f32 {
            let cell_f = $floor_v(coord);
            let local = coord - cell_f;
            let cell: [i32; 4] = $to_int(cell_f);
            let mut closest = coord - coord;
            let mut closest_off = [0i32; 4];
            let mut min = f32::MAX;
            voronoi_offsets::<$dims>(1, |off| {
                let cell_i = iadd4(cell, off);
                let hsh = $int_hash(cell_i);
                let p = $from_int(off) + scale_pos::<$pos>(hsh, params.randomness);
                let d = $dot(p - local, p - local);
                if d < min {
                    min = d;
                    closest = p;
                    closest_off = off;
                }
            });
            let mut min = f32::MAX;
            let mut closest2 = coord - coord;
            voronoi_offsets_skip_center::<$dims>(1, |off_base| {
                let off = iadd4(off_base, closest_off);
                let cell_i = iadd4(cell, off);
                let hsh = $int_hash(cell_i);
                let p = $from_int(off) + scale_pos::<$pos>(hsh, params.randomness);
                let d = $dot(closest - p, closest - p);
                if d < min {
                    min = d;
                    closest2 = p;
                }
            });
            $dot(closest2 - closest, closest2 - closest).sqrt() / 2.0
        }
    };
}

/* Cell offset iteration helpers — cells are [i32;4], unused dims stay 0. */
fn iadd4(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}
fn voronoi_offsets<const D: usize>(radius: i32, mut f: impl FnMut([i32; 4])) {
    let r = -radius..=radius;
    match D {
        2 => {
            for j in r.clone() {
                for i in r.clone() {
                    f([i, j, 0, 0]);
                }
            }
        }
        3 => {
            for k in r.clone() {
                for j in r.clone() {
                    for i in r.clone() {
                        f([i, j, k, 0]);
                    }
                }
            }
        }
        _ => {
            for u in r.clone() {
                for k in r.clone() {
                    for j in r.clone() {
                        for i in r.clone() {
                            f([i, j, k, u]);
                        }
                    }
                }
            }
        }
    }
}
fn voronoi_offsets_skip_center<const D: usize>(radius: i32, mut f: impl FnMut([i32; 4])) {
    voronoi_offsets::<D>(radius, |off| {
        if off != [0, 0, 0, 0] {
            f(off);
        }
    });
}

fn scale_pos<T>(v: T, s: f32) -> T
where
    T: Copy + std::ops::Mul<f32, Output = T>,
{
    v * s
}
fn one_pos<T: OnePos>() -> T {
    T::one()
}
trait OnePos {
    fn one() -> Self;
}
impl OnePos for Vec2 {
    fn one() -> Self {
        Vec2::new(1.0, 1.0)
    }
}
impl OnePos for Vec3 {
    fn one() -> Self {
        Vec3::new(1.0, 1.0, 1.0)
    }
}
impl OnePos for Vec4 {
    fn one() -> Self {
        Vec4::new(1.0, 1.0, 1.0, 1.0)
    }
}

voronoi_dim_impl! {
    Vec2,
    floor_v: |v: Vec2| Vec2::new(v.x.floor(), v.y.floor()),
    to_int: |v: Vec2| [v.x as i32, v.y as i32, 0, 0],
    from_int: |v: [i32; 4]| Vec2::new(v[0] as f32, v[1] as f32),
    int_color: |v: [i32; 4]| hash_int2_to_float3([v[0], v[1]]),
    int_hash: |v: [i32; 4]| hash_int2_to_float2([v[0], v[1]]),
    dist: voronoi_distance_2,
    distb: voronoi_distance_bound_2,
    pos4: voronoi_position_2,
    dot: |a: Vec2, b: Vec2| a.x * b.x + a.y * b.y,
    dims: 2,
    f1: voronoi_f1_2d, smooth: voronoi_smooth_f1_2d, f2: voronoi_f2_2d,
    dte: voronoi_distance_to_edge_2d, nsr: voronoi_n_sphere_radius_2d
}
voronoi_dim_impl! {
    Vec3,
    floor_v: |v: Vec3| Vec3::new(v.x.floor(), v.y.floor(), v.z.floor()),
    to_int: |v: Vec3| [v.x as i32, v.y as i32, v.z as i32, 0],
    from_int: |v: [i32; 4]| Vec3::new(v[0] as f32, v[1] as f32, v[2] as f32),
    int_color: |v: [i32; 4]| hash_int3_to_float3([v[0], v[1], v[2]]),
    int_hash: |v: [i32; 4]| hash_int3_to_float3([v[0], v[1], v[2]]),
    dist: voronoi_distance_3,
    distb: voronoi_distance_bound_3,
    pos4: voronoi_position_3,
    dot: |a: Vec3, b: Vec3| a.x * b.x + a.y * b.y + a.z * b.z,
    dims: 3,
    f1: voronoi_f1_3d, smooth: voronoi_smooth_f1_3d, f2: voronoi_f2_3d,
    dte: voronoi_distance_to_edge_3d, nsr: voronoi_n_sphere_radius_3d
}
voronoi_dim_impl! {
    Vec4,
    floor_v: |v: Vec4| Vec4::new(v.x.floor(), v.y.floor(), v.z.floor(), v.w.floor()),
    to_int: |v: Vec4| [v.x as i32, v.y as i32, v.z as i32, v.w as i32],
    from_int: |v: [i32; 4]| Vec4::new(v[0] as f32, v[1] as f32, v[2] as f32, v[3] as f32),
    int_color: |v: [i32; 4]| hash_int4_to_float3([v[0], v[1], v[2], v[3]]),
    int_hash: |v: [i32; 4]| hash_int4_to_float4([v[0], v[1], v[2], v[3]]),
    dist: voronoi_distance_4,
    distb: voronoi_distance_bound_4,
    pos4: voronoi_position_4,
    dot: |a: Vec4, b: Vec4| a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w,
    dims: 4,
    f1: voronoi_f1_4d, smooth: voronoi_smooth_f1_4d, f2: voronoi_f2_4d,
    dte: voronoi_distance_to_edge_4d, nsr: voronoi_n_sphere_radius_4d
}

/* Dimension-dispatched public Voronoi API matching upstream overloads. */
pub trait VoronoiCoord: Copy {
    fn f1(params: &VoronoiParams, coord: Self) -> VoronoiOutput;
    fn smooth_f1(params: &VoronoiParams, coord: Self, calc_color: bool) -> VoronoiOutput;
    fn f2(params: &VoronoiParams, coord: Self) -> VoronoiOutput;
    fn distance_to_edge(params: &VoronoiParams, coord: Self) -> f32;
    fn n_sphere_radius(params: &VoronoiParams, coord: Self) -> f32;
    fn scale_by(self, s: f32) -> Self;
}
impl VoronoiCoord for f32 {
    fn f1(p: &VoronoiParams, c: Self) -> VoronoiOutput {
        voronoi_f1_1d(p, c)
    }
    fn smooth_f1(p: &VoronoiParams, c: Self, cc: bool) -> VoronoiOutput {
        voronoi_smooth_f1_1d(p, c, cc)
    }
    fn f2(p: &VoronoiParams, c: Self) -> VoronoiOutput {
        voronoi_f2_1d(p, c)
    }
    fn distance_to_edge(p: &VoronoiParams, c: Self) -> f32 {
        voronoi_distance_to_edge_1d(p, c)
    }
    fn n_sphere_radius(p: &VoronoiParams, c: Self) -> f32 {
        voronoi_n_sphere_radius_1d(p, c)
    }
    fn scale_by(self, s: f32) -> Self {
        self * s
    }
}
impl VoronoiCoord for Vec2 {
    fn f1(p: &VoronoiParams, c: Self) -> VoronoiOutput {
        voronoi_f1_2d(p, c)
    }
    fn smooth_f1(p: &VoronoiParams, c: Self, cc: bool) -> VoronoiOutput {
        voronoi_smooth_f1_2d(p, c, cc)
    }
    fn f2(p: &VoronoiParams, c: Self) -> VoronoiOutput {
        voronoi_f2_2d(p, c)
    }
    fn distance_to_edge(p: &VoronoiParams, c: Self) -> f32 {
        voronoi_distance_to_edge_2d(p, c)
    }
    fn n_sphere_radius(p: &VoronoiParams, c: Self) -> f32 {
        voronoi_n_sphere_radius_2d(p, c)
    }
    fn scale_by(self, s: f32) -> Self {
        self * s
    }
}
impl VoronoiCoord for Vec3 {
    fn f1(p: &VoronoiParams, c: Self) -> VoronoiOutput {
        voronoi_f1_3d(p, c)
    }
    fn smooth_f1(p: &VoronoiParams, c: Self, cc: bool) -> VoronoiOutput {
        voronoi_smooth_f1_3d(p, c, cc)
    }
    fn f2(p: &VoronoiParams, c: Self) -> VoronoiOutput {
        voronoi_f2_3d(p, c)
    }
    fn distance_to_edge(p: &VoronoiParams, c: Self) -> f32 {
        voronoi_distance_to_edge_3d(p, c)
    }
    fn n_sphere_radius(p: &VoronoiParams, c: Self) -> f32 {
        voronoi_n_sphere_radius_3d(p, c)
    }
    fn scale_by(self, s: f32) -> Self {
        self * s
    }
}
impl VoronoiCoord for Vec4 {
    fn f1(p: &VoronoiParams, c: Self) -> VoronoiOutput {
        voronoi_f1_4d(p, c)
    }
    fn smooth_f1(p: &VoronoiParams, c: Self, cc: bool) -> VoronoiOutput {
        voronoi_smooth_f1_4d(p, c, cc)
    }
    fn f2(p: &VoronoiParams, c: Self) -> VoronoiOutput {
        voronoi_f2_4d(p, c)
    }
    fn distance_to_edge(p: &VoronoiParams, c: Self) -> f32 {
        voronoi_distance_to_edge_4d(p, c)
    }
    fn n_sphere_radius(p: &VoronoiParams, c: Self) -> f32 {
        voronoi_n_sphere_radius_4d(p, c)
    }
    fn scale_by(self, s: f32) -> Self {
        self * s
    }
}

/* Fractal Voronoi. */
/// `fractal_voronoi_x_fx`.
pub fn fractal_voronoi_x_fx<T: VoronoiCoord>(
    params: &VoronoiParams,
    coord: T,
    calc_color: bool,
) -> VoronoiOutput {
    let mut amplitude = 1.0f32;
    let mut max_amplitude = 0.0f32;
    let mut scale = 1.0f32;
    let mut output = VoronoiOutput::default();
    let zero_input = params.detail == 0.0 || params.roughness == 0.0;

    let mut i = 0;
    while i as f32 <= params.detail.ceil() {
        let octave = if params.feature == NOISE_SHD_VORONOI_F2 {
            T::f2(params, coord.scale_by(scale))
        } else if params.feature == NOISE_SHD_VORONOI_SMOOTH_F1 && params.smoothness != 0.0 {
            T::smooth_f1(params, coord.scale_by(scale), calc_color)
        } else {
            T::f1(params, coord.scale_by(scale))
        };
        if zero_input {
            output = octave;
            break;
        }
        if (i as f32) <= params.detail {
            max_amplitude += amplitude;
            output.distance += octave.distance * amplitude;
            output.color = output.color + octave.color * amplitude;
            output.position = mixv(output.position, octave.position * (1.0 / scale), amplitude);
            scale *= params.lacunarity;
            amplitude *= params.roughness;
        } else {
            let remainder = params.detail - params.detail.floor();
            if remainder != 0.0 {
                max_amplitude = mixf(max_amplitude, max_amplitude + amplitude, remainder);
                output.distance = mixf(
                    output.distance,
                    output.distance + octave.distance * amplitude,
                    remainder,
                );
                output.color = mixv(
                    output.color,
                    output.color + octave.color * amplitude,
                    remainder,
                );
                output.position = mixv(
                    output.position,
                    mixv(output.position, octave.position * (1.0 / scale), amplitude),
                    remainder,
                );
            }
        }
        i += 1;
    }
    if zero_input {
        max_amplitude = 1.0;
    }
    if params.normalize {
        output.distance /= max_amplitude * params.max_distance;
        output.color = output.color * (1.0 / max_amplitude);
    }
    output.position = if params.scale != 0.0 {
        output.position * (1.0 / params.scale)
    } else {
        Vec4::new(0.0, 0.0, 0.0, 0.0)
    };
    output
}
/// `fractal_voronoi_distance_to_edge`.
pub fn fractal_voronoi_distance_to_edge<T: VoronoiCoord>(params: &VoronoiParams, coord: T) -> f32 {
    let mut amplitude = 1.0f32;
    let mut max_amplitude = params.max_distance;
    let mut scale = 1.0f32;
    let mut distance = 8.0f32;
    let zero_input = params.detail == 0.0 || params.roughness == 0.0;

    let mut i = 0;
    while i as f32 <= params.detail.ceil() {
        let octave_distance = T::distance_to_edge(params, coord.scale_by(scale));
        if zero_input {
            distance = octave_distance;
            break;
        }
        if (i as f32) <= params.detail {
            max_amplitude = mixf(max_amplitude, params.max_distance / scale, amplitude);
            distance = mixf(distance, distance.min(octave_distance / scale), amplitude);
            scale *= params.lacunarity;
            amplitude *= params.roughness;
        } else {
            let remainder = params.detail - params.detail.floor();
            if remainder != 0.0 {
                let lerp_amplitude = mixf(max_amplitude, params.max_distance / scale, amplitude);
                max_amplitude = mixf(max_amplitude, lerp_amplitude, remainder);
                let lerp_distance =
                    mixf(distance, distance.min(octave_distance / scale), amplitude);
                distance = mixf(distance, distance.min(lerp_distance), remainder);
            }
        }
        i += 1;
    }
    if params.normalize {
        distance /= max_amplitude;
    }
    distance
}

/* -------------------------------------------------------------------- */
/** Gabor noise. */

const GABOR_IMPULSES_COUNT: i32 = 8;

fn compute_2d_gabor_kernel(position: Vec2, frequency: f32, orientation: f32) -> Vec2 {
    let distance_squared = position.x * position.x + position.y * position.y;
    let hann_window = 0.5 + 0.5 * (std::f32::consts::PI * distance_squared).cos();
    let gaussian_envelop = (-std::f32::consts::PI * distance_squared).exp();
    let windowed_gaussian_envelope = gaussian_envelop * hann_window;
    let frequency_vector = Vec2::new(orientation.cos(), orientation.sin()) * frequency;
    let angle = 2.0 * std::f32::consts::PI * (position.x * frequency_vector.x + position.y * frequency_vector.y);
    let phasor = Vec2::new(angle.cos(), angle.sin());
    phasor * windowed_gaussian_envelope
}
fn compute_2d_gabor_standard_deviation() -> f32 {
    (GABOR_IMPULSES_COUNT as f32 * 0.5 * 0.25).sqrt()
}
fn compute_2d_gabor_noise_cell(
    cell: Vec2,
    position: Vec2,
    frequency: f32,
    isotropy: f32,
    base_orientation: f32,
) -> Vec2 {
    let mut noise = Vec2::new(0.0, 0.0);
    for i in 0..GABOR_IMPULSES_COUNT {
        let seed_for_orientation = Vec3::new(cell.x, cell.y, (i * 3) as f32);
        let seed_for_kernel_center = Vec3::new(cell.x, cell.y, (i * 3 + 1) as f32);
        let seed_for_weight = Vec3::new(cell.x, cell.y, (i * 3 + 2) as f32);
        let random_orientation =
            (hash_float_to_float3(seed_for_orientation) - 0.5) * std::f32::consts::PI;
        let orientation = base_orientation + random_orientation * isotropy;
        let kernel_center = hash_float3_to_float2(seed_for_kernel_center);
        let position_in_kernel_space = position - kernel_center;
        if position_in_kernel_space.x * position_in_kernel_space.x
            + position_in_kernel_space.y * position_in_kernel_space.y
            >= 1.0
        {
            continue;
        }
        let weight = if hash_float_to_float3(seed_for_weight) < 0.5 {
            -1.0
        } else {
            1.0
        };
        noise = noise
            + compute_2d_gabor_kernel(position_in_kernel_space, frequency, orientation) * weight;
    }
    noise
}
fn compute_2d_gabor_noise(
    coordinates: Vec2,
    frequency: f32,
    isotropy: f32,
    base_orientation: f32,
) -> Vec2 {
    let cell_position = Vec2::new(coordinates.x.floor(), coordinates.y.floor());
    let local_position = coordinates - cell_position;
    let mut sum = Vec2::new(0.0, 0.0);
    for j in -1..=1 {
        for i in -1..=1 {
            let cell_offset = Vec2::new(i as f32, j as f32);
            let current_cell_position = cell_position + cell_offset;
            let position_in_cell_space = local_position - cell_offset;
            sum = sum
                + compute_2d_gabor_noise_cell(
                    current_cell_position,
                    position_in_cell_space,
                    frequency,
                    isotropy,
                    base_orientation,
                );
        }
    }
    sum
}
fn compute_3d_gabor_kernel(position: Vec3, frequency: f32, orientation: Vec3) -> Vec2 {
    let distance_squared = position.x * position.x + position.y * position.y + position.z * position.z;
    let hann_window = 0.5 + 0.5 * (std::f32::consts::PI * distance_squared).cos();
    let gaussian_envelop = (-std::f32::consts::PI * distance_squared).exp();
    let windowed_gaussian_envelope = gaussian_envelop * hann_window;
    let frequency_vector = orientation * frequency;
    let angle = 2.0
        * std::f32::consts::PI
        * (position.x * frequency_vector.x
            + position.y * frequency_vector.y
            + position.z * frequency_vector.z);
    let phasor = Vec2::new(angle.cos(), angle.sin());
    phasor * windowed_gaussian_envelope
}
fn compute_3d_gabor_standard_deviation() -> f32 {
    (GABOR_IMPULSES_COUNT as f32 * 0.5 * (1.0 / (4.0 * std::f32::consts::SQRT_2))).sqrt()
}
fn compute_3d_orientation(orientation: Vec3, isotropy: f32, seed: Vec4) -> Vec3 {
    if isotropy == 0.0 {
        return orientation;
    }
    let mut inclination = orientation.z.acos();
    let mut azimuth = orientation.y.signum()
        * (orientation.x / (orientation.x * orientation.x + orientation.y * orientation.y).sqrt()).acos();
    let random_angles = hash_float4_to_float2(seed) * std::f32::consts::PI;
    inclination += random_angles.x * isotropy;
    azimuth += random_angles.y * isotropy;
    Vec3::new(
        inclination.sin() * azimuth.cos(),
        inclination.sin() * azimuth.sin(),
        inclination.cos(),
    )
}
fn compute_3d_gabor_noise_cell(
    cell: Vec3,
    position: Vec3,
    frequency: f32,
    isotropy: f32,
    base_orientation: Vec3,
) -> Vec2 {
    let mut noise = Vec2::new(0.0, 0.0);
    for i in 0..GABOR_IMPULSES_COUNT {
        let seed_for_orientation = Vec4::new(cell.x, cell.y, cell.z, (i * 3) as f32);
        let seed_for_kernel_center = Vec4::new(cell.x, cell.y, cell.z, (i * 3 + 1) as f32);
        let seed_for_weight = Vec4::new(cell.x, cell.y, cell.z, (i * 3 + 2) as f32);
        let orientation = compute_3d_orientation(base_orientation, isotropy, seed_for_orientation);
        let kernel_center = hash_float4_to_float3(seed_for_kernel_center);
        let position_in_kernel_space = position - kernel_center;
        if position_in_kernel_space.x * position_in_kernel_space.x
            + position_in_kernel_space.y * position_in_kernel_space.y
            + position_in_kernel_space.z * position_in_kernel_space.z
            >= 1.0
        {
            continue;
        }
        let weight = if hash_float_to_float4(seed_for_weight) < 0.5 {
            -1.0
        } else {
            1.0
        };
        noise = noise
            + compute_3d_gabor_kernel(position_in_kernel_space, frequency, orientation) * weight;
    }
    noise
}
fn compute_3d_gabor_noise(
    coordinates: Vec3,
    frequency: f32,
    isotropy: f32,
    base_orientation: Vec3,
) -> Vec2 {
    let cell_position = Vec3::new(
        coordinates.x.floor(),
        coordinates.y.floor(),
        coordinates.z.floor(),
    );
    let local_position = coordinates - cell_position;
    let mut sum = Vec2::new(0.0, 0.0);
    for k in -1..=1 {
        for j in -1..=1 {
            for i in -1..=1 {
                let cell_offset = Vec3::new(i as f32, j as f32, k as f32);
                let current_cell_position = cell_position + cell_offset;
                let position_in_cell_space = local_position - cell_offset;
                sum = sum
                    + compute_3d_gabor_noise_cell(
                        current_cell_position,
                        position_in_cell_space,
                        frequency,
                        isotropy,
                        base_orientation,
                    );
            }
        }
    }
    sum
}
/// `noise::gabor` 2D → (value, phase, intensity).
pub fn gabor_2d(
    coordinates: Vec2,
    scale: f32,
    frequency: f32,
    anisotropy: f32,
    orientation: f32,
) -> (f32, f32, f32) {
    let scaled_coordinates = coordinates * scale;
    let isotropy = 1.0 - anisotropy.clamp(0.0, 1.0);
    let sanitized_frequency = frequency.max(0.001);
    let phasor = compute_2d_gabor_noise(scaled_coordinates, sanitized_frequency, isotropy, orientation);
    let standard_deviation = compute_2d_gabor_standard_deviation();
    let normalization_factor = 6.0 * standard_deviation;
    (
        (phasor.y / normalization_factor) * 0.5 + 0.5,
        (phasor.y.atan2(phasor.x) + std::f32::consts::PI) / (2.0 * std::f32::consts::PI),
        (phasor.x * phasor.x + phasor.y * phasor.y).sqrt() / normalization_factor,
    )
}
/// `noise::gabor` 3D → (value, phase, intensity).
pub fn gabor_3d(
    coordinates: Vec3,
    scale: f32,
    frequency: f32,
    anisotropy: f32,
    orientation: Vec3,
) -> (f32, f32, f32) {
    let scaled_coordinates = coordinates * scale;
    let isotropy = 1.0 - anisotropy.clamp(0.0, 1.0);
    let sanitized_frequency = frequency.max(0.001);
    let l = (orientation.x * orientation.x + orientation.y * orientation.y + orientation.z * orientation.z).sqrt();
    let normalized_orientation = orientation * (1.0 / l);
    let phasor = compute_3d_gabor_noise(
        scaled_coordinates,
        sanitized_frequency,
        isotropy,
        normalized_orientation,
    );
    let standard_deviation = compute_3d_gabor_standard_deviation();
    let normalization_factor = 6.0 * standard_deviation;
    (
        (phasor.y / normalization_factor) * 0.5 + 0.5,
        (phasor.y.atan2(phasor.x) + std::f32::consts::PI) / (2.0 * std::f32::consts::PI),
        (phasor.x * phasor.x + phasor.y * phasor.y).sqrt() / normalization_factor,
    )
}

/* -------------------------------------------------------------------- */
/** VArray-style batch wrappers.
 *
 * PORT_NOTE: upstream takes `VArray<T>` and evaluates over an `IndexMask`
 * with chunked SoA kernels. `VArrayRead` covers the same access patterns
 * (single/span/computed); evaluation is per-index. */

pub enum VArrayRead<'a, T: Copy> {
    Single(T),
    Slice(&'a [T]),
    Fn(&'a dyn Fn(i64) -> T),
}
impl<'a, T: Copy> VArrayRead<'a, T> {
    pub fn get(&self, i: i64) -> T {
        match self {
            VArrayRead::Single(v) => *v,
            VArrayRead::Slice(s) => s[i as usize],
            VArrayRead::Fn(f) => f(i),
        }
    }
    pub fn get_if_single(&self) -> Option<T> {
        match self {
            VArrayRead::Single(v) => Some(*v),
            _ => None,
        }
    }
}

pub fn white_noise_float_1d(mask: &IndexMask, ks: &VArrayRead<f32>, r_values: &mut [f32]) {
    for i in mask.iter() {
        r_values[i as usize] = hash_float_to_float(ks.get(i));
    }
}
pub fn white_noise_float_2d(mask: &IndexMask, ks: &VArrayRead<Vec3>, r_values: &mut [f32]) {
    for i in mask.iter() {
        let k = ks.get(i);
        r_values[i as usize] = hash_float_to_float2(Vec2::new(k.x, k.y));
    }
}
pub fn white_noise_float_3d(mask: &IndexMask, ks: &VArrayRead<Vec3>, r_values: &mut [f32]) {
    for i in mask.iter() {
        r_values[i as usize] = hash_float_to_float3(ks.get(i));
    }
}
pub fn white_noise_float_4d(
    mask: &IndexMask,
    vectors: &VArrayRead<Vec3>,
    ws: &VArrayRead<f32>,
    r_values: &mut [f32],
) {
    for i in mask.iter() {
        let v = vectors.get(i);
        r_values[i as usize] = hash_float_to_float4(Vec4::new(v.x, v.y, v.z, ws.get(i)));
    }
}
pub fn white_noise_color_1d(mask: &IndexMask, ks: &VArrayRead<f32>, r_values: &mut [[f32; 4]]) {
    for i in mask.iter() {
        let c = hash_float1_to_float3(ks.get(i));
        r_values[i as usize] = [c.x, c.y, c.z, 1.0];
    }
}
pub fn white_noise_color_2d(mask: &IndexMask, ks: &VArrayRead<Vec3>, r_values: &mut [[f32; 4]]) {
    for i in mask.iter() {
        let k = ks.get(i);
        let c = hash_float2_to_float3(Vec2::new(k.x, k.y));
        r_values[i as usize] = [c.x, c.y, c.z, 1.0];
    }
}
pub fn white_noise_color_3d(mask: &IndexMask, ks: &VArrayRead<Vec3>, r_values: &mut [[f32; 4]]) {
    for i in mask.iter() {
        let c = hash_float3_to_float3(ks.get(i));
        r_values[i as usize] = [c.x, c.y, c.z, 1.0];
    }
}
pub fn white_noise_color_4d(
    mask: &IndexMask,
    vectors: &VArrayRead<Vec3>,
    ws: &VArrayRead<f32>,
    r_values: &mut [[f32; 4]],
) {
    for i in mask.iter() {
        let v = vectors.get(i);
        let c = hash_float4_to_float3(Vec4::new(v.x, v.y, v.z, ws.get(i)));
        r_values[i as usize] = [c.x, c.y, c.z, 1.0];
    }
}

fn perlin_clamp_detail(detail: f32) -> f32 {
    if detail.is_nan() {
        return 0.0;
    }
    detail.clamp(0.0, 15.0)
}
fn perlin_clamp_roughness(roughness: f32) -> f32 {
    roughness.max(0.0)
}

/// `CommonPerlinVArrayParams` — shared per-element noise parameters.
pub struct CommonPerlinVArrayParams<'a> {
    pub mask: &'a IndexMask,
    pub detail: VArrayRead<'a, f32>,
    pub roughness: VArrayRead<'a, f32>,
    pub lacunarity: VArrayRead<'a, f32>,
    pub offset: VArrayRead<'a, f32>,
    pub gain: VArrayRead<'a, f32>,
    pub distortion: VArrayRead<'a, f32>,
    pub scale: VArrayRead<'a, f32>,
    pub type_: i32,
    pub normalize: bool,
}

fn perlin_float_fractal_distorted_chunk<T: NoisePos>(
    params: &CommonPerlinVArrayParams,
    positions: &VArrayRead<T>,
    dst: &mut [f32],
    distort: impl Fn(T, f32) -> T,
) {
    for i in params.mask.iter() {
        let iu = i as usize;
        let mut pos = positions.get(i) * params.scale.get(i);
        let d = params.distortion.get(i);
        if d != 0.0 {
            pos = pos + distort(pos, d);
        }
        dst[iu] = perlin_select(
            pos,
            perlin_clamp_detail(params.detail.get(i)),
            perlin_clamp_roughness(params.roughness.get(i)),
            params.lacunarity.get(i),
            params.offset.get(i),
            params.gain.get(i),
            params.type_,
            params.normalize,
        );
    }
}
/// `perlin_float_fractal_distorted_varray_1d`.
pub fn perlin_float_fractal_distorted_varray_1d(
    params: &CommonPerlinVArrayParams,
    positions: &VArrayRead<f32>,
    r_values: &mut [f32],
) {
    perlin_float_fractal_distorted_chunk(params, positions, r_values, |p, d| {
        perlin_signed_1(p + random_float_offset(0.0)) * d
    });
}
/// `perlin_float_fractal_distorted_varray_2d` — position input is float3, xy used.
pub fn perlin_float_fractal_distorted_varray_2d(
    params: &CommonPerlinVArrayParams,
    positions_3d: &VArrayRead<Vec3>,
    r_values: &mut [f32],
) {
    let positions = VArrayRead::Fn(&|i: i64| {
        let v = positions_3d.get(i);
        Vec2::new(v.x, v.y)
    });
    perlin_float_fractal_distorted_chunk(params, &positions, r_values, |p, d| {
        Vec2::new(
            perlin_signed_2(p + random_float2_offset(0.0)) * d,
            perlin_signed_2(p + random_float2_offset(1.0)) * d,
        )
    });
}
/// `perlin_float_fractal_distorted_varray_3d`.
pub fn perlin_float_fractal_distorted_varray_3d(
    params: &CommonPerlinVArrayParams,
    positions: &VArrayRead<Vec3>,
    r_values: &mut [f32],
) {
    perlin_float_fractal_distorted_chunk(params, positions, r_values, |p, d| {
        Vec3::new(
            perlin_signed_3(p + random_float3_offset(0.0)) * d,
            perlin_signed_3(p + random_float3_offset(1.0)) * d,
            perlin_signed_3(p + random_float3_offset(2.0)) * d,
        )
    });
}
/// `perlin_float_fractal_distorted_varray_4d` — float3 position + separate w input.
pub fn perlin_float_fractal_distorted_varray_4d(
    params: &CommonPerlinVArrayParams,
    positions_3d: &VArrayRead<Vec3>,
    ws: &VArrayRead<f32>,
    r_values: &mut [f32],
) {
    let positions = VArrayRead::Fn(&|i: i64| {
        let v = positions_3d.get(i);
        Vec4::new(v.x, v.y, v.z, ws.get(i))
    });
    perlin_float_fractal_distorted_chunk(params, &positions, r_values, |p, d| {
        Vec4::new(
            perlin_signed_4(p + random_float4_offset(0.0)) * d,
            perlin_signed_4(p + random_float4_offset(1.0)) * d,
            perlin_signed_4(p + random_float4_offset(2.0)) * d,
            perlin_signed_4(p + random_float4_offset(3.0)) * d,
        )
    });
}

fn perlin_color_channel_offsets<T: NoisePos>() -> [T; 3] {
    [
        T::offset1() * 0.0, /* zero offset */
        T::offset1(),
        T::offset2(),
    ]
}
fn perlin_color_fractal_distorted_chunk<T: NoisePos>(
    params: &CommonPerlinVArrayParams,
    positions: &VArrayRead<T>,
    dst: &mut [[f32; 4]],
    distort: impl Fn(T, f32) -> T,
) {
    let channel_offsets = perlin_color_channel_offsets::<T>();
    for i in params.mask.iter() {
        let iu = i as usize;
        let mut pos = positions.get(i) * params.scale.get(i);
        let d = params.distortion.get(i);
        if d != 0.0 {
            pos = pos + distort(pos, d);
        }
        for channel in 0..3 {
            dst[iu][channel] = perlin_select(
                pos + channel_offsets[channel],
                perlin_clamp_detail(params.detail.get(i)),
                perlin_clamp_roughness(params.roughness.get(i)),
                params.lacunarity.get(i),
                params.offset.get(i),
                params.gain.get(i),
                params.type_,
                params.normalize,
            );
        }
        dst[iu][3] = 1.0;
    }
}
/// `perlin_color_fractal_distorted_varray_1d`.
pub fn perlin_color_fractal_distorted_varray_1d(
    params: &CommonPerlinVArrayParams,
    positions: &VArrayRead<f32>,
    r_values: &mut [[f32; 4]],
) {
    perlin_color_fractal_distorted_chunk(params, positions, r_values, |p, d| {
        perlin_signed_1(p + random_float_offset(0.0)) * d
    });
}
/// `perlin_color_fractal_distorted_varray_2d`.
pub fn perlin_color_fractal_distorted_varray_2d(
    params: &CommonPerlinVArrayParams,
    positions_3d: &VArrayRead<Vec3>,
    r_values: &mut [[f32; 4]],
) {
    let positions = VArrayRead::Fn(&|i: i64| {
        let v = positions_3d.get(i);
        Vec2::new(v.x, v.y)
    });
    perlin_color_fractal_distorted_chunk(params, &positions, r_values, |p, d| {
        Vec2::new(
            perlin_signed_2(p + random_float2_offset(0.0)) * d,
            perlin_signed_2(p + random_float2_offset(1.0)) * d,
        )
    });
}
/// `perlin_color_fractal_distorted_varray_3d`.
pub fn perlin_color_fractal_distorted_varray_3d(
    params: &CommonPerlinVArrayParams,
    positions: &VArrayRead<Vec3>,
    r_values: &mut [[f32; 4]],
) {
    perlin_color_fractal_distorted_chunk(params, positions, r_values, |p, d| {
        Vec3::new(
            perlin_signed_3(p + random_float3_offset(0.0)) * d,
            perlin_signed_3(p + random_float3_offset(1.0)) * d,
            perlin_signed_3(p + random_float3_offset(2.0)) * d,
        )
    });
}
/// `perlin_color_fractal_distorted_varray_4d`.
pub fn perlin_color_fractal_distorted_varray_4d(
    params: &CommonPerlinVArrayParams,
    positions_3d: &VArrayRead<Vec3>,
    ws: &VArrayRead<f32>,
    r_values: &mut [[f32; 4]],
) {
    let positions = VArrayRead::Fn(&|i: i64| {
        let v = positions_3d.get(i);
        Vec4::new(v.x, v.y, v.z, ws.get(i))
    });
    perlin_color_fractal_distorted_chunk(params, &positions, r_values, |p, d| {
        Vec4::new(
            perlin_signed_4(p + random_float4_offset(0.0)) * d,
            perlin_signed_4(p + random_float4_offset(1.0)) * d,
            perlin_signed_4(p + random_float4_offset(2.0)) * d,
            perlin_signed_4(p + random_float4_offset(3.0)) * d,
        )
    });
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct PerlinSplitKey {
    detail_bits: u32,
    has_distortion: bool,
}
/// `perlin_split_mask` — group mask indices by quantized detail + distortion
/// so each sub-mask shares octave count. Returns one IndexMask per group.
pub fn perlin_split_mask(
    mask: &IndexMask,
    detail_varray: &VArrayRead<f32>,
    distortion_varray: &VArrayRead<f32>,
) -> Vec<IndexMask> {
    let detail_value = detail_varray.get_if_single();
    let distortion_value = distortion_varray.get_if_single();
    if detail_value.is_some() && distortion_value.is_some() {
        return vec![mask.clone()];
    }
    let mut indices_by_key: HashMap<PerlinSplitKey, Vec<i64>> = HashMap::new();
    for i in mask.iter() {
        let detail = perlin_clamp_detail(detail_varray.get(i));
        let detail_floor = detail.floor();
        let quantized = detail_floor + 0.5 * ((detail != detail_floor) as i32 as f32);
        let key = PerlinSplitKey {
            detail_bits: quantized.to_bits(),
            has_distortion: distortion_varray.get(i) != 0.0,
        };
        indices_by_key.entry(key).or_default().push(i);
    }
    indices_by_key
        .into_values()
        .map(|indices| IndexMask::from_indices_i64(&indices))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perlin_deterministic_and_bounded() {
        for &(p, lo, hi) in &[(perlin(0.5f32), 0.0f32, 1.0f32), (perlin(Vec3::new(0.3, 0.7, 1.2)), 0.0, 1.0)] {
            assert!(p >= lo && p <= hi);
        }
        assert_eq!(perlin(Vec2::new(0.5, 0.5)), perlin(Vec2::new(0.5, 0.5)));
        let f = perlin_fbm(Vec3::new(0.3, 0.7, 1.2), 2.0, 0.5, 2.0, true);
        assert!((0.0..=1.0).contains(&f));
        let v = fractal_voronoi_x_fx(
            &VoronoiParams {
                detail: 1.5,
                roughness: 0.5,
                lacunarity: 2.0,
                randomness: 1.0,
                max_distance: 1.0,
                feature: NOISE_SHD_VORONOI_F1,
                metric: NOISE_SHD_VORONOI_EUCLIDEAN,
                scale: 1.0,
                ..Default::default()
            },
            Vec3::new(0.3, 0.4, 0.5),
            true,
        );
        assert!(v.distance.is_finite() && v.distance >= 0.0);
        let (val, phase, inten) = gabor_2d(Vec2::new(0.4, 0.6), 2.0, 2.0, 0.5, 0.0);
        assert!(val.is_finite() && phase.is_finite() && inten.is_finite());
    }
}
