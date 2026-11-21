//! Port of `intern/attribute_math.cc` — `bke::attribute_math` generic
//! mixing (`mix2`/`mix3`/`mix4`/`mix_indices`/`mix_groups` + the
//! ColorGeometry/float4x4 mixers).
//!
//! `GSpan` → generics with a `Mix` trait; `IndexMask` →
//! `blenders_bli::index_mask::IndexMask`.

use blenders_math::matrix::{Mat3, Mat4};
use blenders_math::quaternion::Quaternion;
use blenders_math::vector::{Vec2, Vec3, Vec4};

/// `mix` capability used by `attribute_math` — `mix2`/`mix3`/`mix4`.
pub trait AttrMix: Copy {
    fn mix2(factor: f32, a: Self, b: Self) -> Self;
    fn mix3(weights: Vec3, v0: Self, v1: Self, v2: Self) -> Self {
        Self::mix2(weights.x, Self::mix2(weights.y / (weights.y + weights.z).max(1e-9), v1, v2), v0)
            /* fallback; concrete impls override with the verbatim formula */
    }
    fn mix4(weights: Vec4, v0: Self, v1: Self, v2: Self, v3: Self) -> Self {
        Self::mix3(Vec3::new(weights.x, weights.y, weights.z), v0, v1, v2)
    }
}

impl AttrMix for f32 {
    fn mix2(factor: f32, a: Self, b: Self) -> Self {
        a + (b - a) * factor
    }
    fn mix3(w: Vec3, v0: Self, v1: Self, v2: Self) -> Self {
        w.x * v0 + w.y * v1 + w.z * v2
    }
    fn mix4(w: Vec4, v0: Self, v1: Self, v2: Self, v3: Self) -> Self {
        w.x * v0 + w.y * v1 + w.z * v2 + w.w * v3
    }
}

impl AttrMix for Vec2 {
    fn mix2(factor: f32, a: Self, b: Self) -> Self {
        a + (b - a) * factor
    }
    fn mix3(w: Vec3, v0: Self, v1: Self, v2: Self) -> Self {
        v0 * w.x + v1 * w.y + v2 * w.z
    }
    fn mix4(w: Vec4, v0: Self, v1: Self, v2: Self, v3: Self) -> Self {
        v0 * w.x + v1 * w.y + v2 * w.z + v3 * w.w
    }
}

impl AttrMix for Vec3 {
    fn mix2(factor: f32, a: Self, b: Self) -> Self {
        a + (b - a) * factor
    }
    fn mix3(w: Vec3, v0: Self, v1: Self, v2: Self) -> Self {
        v0 * w.x + v1 * w.y + v2 * w.z
    }
    fn mix4(w: Vec4, v0: Self, v1: Self, v2: Self, v3: Self) -> Self {
        v0 * w.x + v1 * w.y + v2 * w.z + v3 * w.w
    }
}

impl AttrMix for Vec4 {
    fn mix2(factor: f32, a: Self, b: Self) -> Self {
        a + (b - a) * factor
    }
    fn mix3(w: Vec3, v0: Self, v1: Self, v2: Self) -> Self {
        v0 * w.x + v1 * w.y + v2 * w.z
    }
    fn mix4(w: Vec4, v0: Self, v1: Self, v2: Self, v3: Self) -> Self {
        v0 * w.x + v1 * w.y + v2 * w.z + v3 * w.w
    }
}

/// `mix2<Quaternion>` — `math::interpolate`.
pub fn quat_mix2(factor: f32, a: Quaternion, b: Quaternion) -> Quaternion {
    a.slerp(b, factor)
}

/// `mix3<Quaternion>` — expmap mixing.
pub fn quat_mix3(weights: Vec3, v0: Quaternion, v1: Quaternion, v2: Quaternion) -> Quaternion {
    let e = v0.expmap() * weights.x + v1.expmap() * weights.y + v2.expmap() * weights.z;
    Quaternion::from_expmap(e)
}

/// `mix4<Quaternion>` — expmap mixing.
pub fn quat_mix4(weights: Vec4, v0: Quaternion, v1: Quaternion, v2: Quaternion, v3: Quaternion) -> Quaternion {
    let e = v0.expmap() * weights.x + v1.expmap() * weights.y
        + v2.expmap() * weights.z + v3.expmap() * weights.w;
    Quaternion::from_expmap(e)
}

/// `mix2<float4x4>` — `math::interpolate` (polar decompose).
pub fn mat4_mix2(factor: f32, a: Mat4, b: Mat4) -> Mat4 {
    Mat4::interpolate(a, b, factor)
}

/// `mix3<float4x4>` — location/rotation/scale split (verbatim).
pub fn mat4_mix3(weights: Vec3, v0: Mat4, v1: Mat4, v2: Mat4) -> Mat4 {
    let location = v0.location() * weights.x + v1.location() * weights.y + v2.location() * weights.z;
    let rotation = quat_mix3(
        weights,
        mat3_normalized_to_quat_safe(Mat3::from(v0).normalized()),
        mat3_normalized_to_quat_safe(Mat3::from(v1).normalized()),
        mat3_normalized_to_quat_safe(Mat3::from(v2).normalized()),
    );
    let scale = Mat3::from(v0).to_scale() * weights.x + Mat3::from(v1).to_scale() * weights.y + Mat3::from(v2).to_scale() * weights.z;
    Mat4::from_loc_rot_scale(location, rotation.to_mat3(), scale)
}

/// `mix4<float4x4>` — same split with 4 weights.
pub fn mat4_mix4(weights: Vec4, v0: Mat4, v1: Mat4, v2: Mat4, v3: Mat4) -> Mat4 {
    let location = v0.location() * weights.x + v1.location() * weights.y
        + v2.location() * weights.z + v3.location() * weights.w;
    let rotation = quat_mix4(
        weights,
        mat3_normalized_to_quat_safe(Mat3::from(v0).normalized()),
        mat3_normalized_to_quat_safe(Mat3::from(v1).normalized()),
        mat3_normalized_to_quat_safe(Mat3::from(v2).normalized()),
        mat3_normalized_to_quat_safe(Mat3::from(v3).normalized()),
    );
    let scale = Mat3::from(v0).to_scale() * weights.x + Mat3::from(v1).to_scale() * weights.y
        + Mat3::from(v2).to_scale() * weights.z + Mat3::from(v3).to_scale() * weights.w;
    Mat4::from_loc_rot_scale(location, rotation.to_mat3(), scale)
}

/// `math::normalized_to_quaternion_safe(math::normalize(float3x3))`.
fn mat3_normalized_to_quat_safe(m: Mat3) -> Quaternion {
    Quaternion::from_mat3(m)
}

/// `mix_indices<T>` — weighted sum of `src[i]` for `i` in `indices` with
/// optional `weights` (equal weights when `None`).
pub fn mix_indices(src: &[Vec3], indices: &[i32], weights: Option<&[f32]>) -> Vec3 {
    if indices.is_empty() {
        return Vec3::ZERO;
    }
    let mut total = Vec3::ZERO;
    let mut total_w = 0.0f32;
    for (i, &idx) in indices.iter().enumerate() {
        let w = weights.map(|ws| ws[i]).unwrap_or(1.0);
        total += src[idx as usize] * w;
        total_w += w;
    }
    if total_w != 0.0 { total / total_w } else { Vec3::ZERO }
}

/// `mix_indices<float4x4>` — loc/rot/scale averaged (verbatim).
pub fn mat4_mix_indices(src: &[Mat4], indices: &[i32], weights: Option<&[f32]>) -> Mat4 {
    if indices.is_empty() {
        return Mat4::IDENTITY;
    }
    let mut location_accum = Vec3::ZERO;
    let mut rotation_accum = Vec3::ZERO; /* expmap accumulation */
    let mut scale_accum = Vec3::ZERO;
    let mut total_weight = 0.0f32;
    for (i, &idx) in indices.iter().enumerate() {
        let w = weights.map(|ws| ws[i]).unwrap_or(1.0);
        let m = src[idx as usize];
        location_accum += m.location() * w;
        rotation_accum += Quaternion::from_mat3(Mat3::from(m).normalized()).expmap() * w;
        scale_accum += Mat3::from(m).to_scale() * w;
        total_weight += w;
    }
    if total_weight == 0.0 {
        return Mat4::IDENTITY;
    }
    let weight_inv = 1.0 / total_weight;
    Mat4::from_loc_rot_scale(
        location_accum * weight_inv,
        Quaternion::from_expmap(rotation_accum * weight_inv).to_mat3(),
        scale_accum * weight_inv,
    )
}

/// `ColorGeometry4f` — float RGBA color attribute value.
pub type ColorGeometry4f = Vec4;
/// `ColorGeometry4b` — byte RGBA color attribute value.
pub type ColorGeometry4b = [u8; 4];

/// `DefaultMixer<T>` — per-index weighted accumulator with default fill.
pub struct DefaultMixer<T: AttrMix> {
    pub buffer: Vec<T>,
    pub total_weights: Vec<f32>,
    pub default_value: T,
}

impl<T: AttrMix> DefaultMixer<T> {
    pub fn new(buffer: Vec<T>, default_value: T) -> Self {
        let n = buffer.len();
        Self { buffer, total_weights: vec![0.0; n], default_value }
    }
    /// `mix_in(index, value, weight)` — `mix2` accumulate.
    pub fn mix_in(&mut self, index: usize, value: T, weight: f32) {
        let w0 = self.total_weights[index];
        let w1 = w0 + weight;
        let factor = if w1 != 0.0 { weight / w1 } else { 0.0 };
        self.buffer[index] = T::mix2(factor, self.buffer[index], value);
        self.total_weights[index] = w1;
    }
    /// `finalize` — untouched indices get `default_value`.
    pub fn finalize(&mut self) {
        for i in 0..self.buffer.len() {
            if self.total_weights[i] == 0.0 {
                self.buffer[i] = self.default_value;
            }
        }
    }
}

/// `ColorGeometry4bMixer` — byte color accumulator (channels averaged as
/// floats, rounded back).
pub struct ColorGeometry4bMixer {
    pub buffer: Vec<ColorGeometry4b>,
    pub total_weights: Vec<f32>,
    pub color_accum: Vec<[f32; 4]>,
    pub default_color: ColorGeometry4b,
}

impl ColorGeometry4bMixer {
    pub fn new(buffer: Vec<ColorGeometry4b>, default_color: ColorGeometry4b) -> Self {
        let n = buffer.len();
        Self {
            buffer,
            total_weights: vec![0.0; n],
            color_accum: vec![[0.0; 4]; n],
            default_color,
        }
    }
    /// `mix_in` — accumulate `color * weight` per channel.
    pub fn mix_in(&mut self, index: usize, color: ColorGeometry4b, weight: f32) {
        for c in 0..4 {
            self.color_accum[index][c] += color[c] as f32 * weight;
        }
        self.total_weights[index] += weight;
    }
    /// `finalize` — write averaged colors back.
    pub fn finalize(&mut self) {
        for i in 0..self.buffer.len() {
            let tw = self.total_weights[i];
            if tw > 0.0 {
                let mut out = [0u8; 4];
                for c in 0..4 {
                    out[c] = (self.color_accum[i][c] / tw).round().clamp(0.0, 255.0) as u8;
                }
                self.buffer[i] = out;
            } else {
                self.buffer[i] = self.default_color;
            }
        }
    }
}

/// `float4x4Mixer` — loc + expmap-rotation + scale accumulation
/// (`float4x4Mixer` in the C file).
pub struct Float4x4Mixer {
    pub buffer: Vec<Mat4>,
    pub total_weights: Vec<f32>,
    pub location_buffer: Vec<Vec3>,
    pub expmap_buffer: Vec<Vec3>,
    pub scale_buffer: Vec<Vec3>,
}

impl Float4x4Mixer {
    pub fn new(buffer: Vec<Mat4>) -> Self {
        let n = buffer.len();
        Self {
            buffer,
            total_weights: vec![0.0; n],
            location_buffer: vec![Vec3::ZERO; n],
            expmap_buffer: vec![Vec3::ZERO; n],
            scale_buffer: vec![Vec3::ZERO; n],
        }
    }
    /// `set` — overwrite accumulation state for `index`.
    pub fn set(&mut self, index: usize, value: Mat4, weight: f32) {
        self.total_weights[index] = weight;
        self.location_buffer[index] = value.location() * weight;
        self.expmap_buffer[index] =
            Quaternion::from_mat3(Mat3::from(value).normalized()).expmap() * weight;
        self.scale_buffer[index] = Mat3::from(value).to_scale() * weight;
    }
    /// `mix_in` — accumulate.
    pub fn mix_in(&mut self, index: usize, value: Mat4, weight: f32) {
        self.total_weights[index] += weight;
        self.location_buffer[index] += value.location() * weight;
        self.expmap_buffer[index] +=
            Quaternion::from_mat3(Mat3::from(value).normalized()).expmap() * weight;
        self.scale_buffer[index] += Mat3::from(value).to_scale() * weight;
    }
    /// `finalize` — indices with weight get their composed matrix;
    /// untouched indices keep `buffer` as is.
    pub fn finalize(&mut self) {
        for i in 0..self.buffer.len() {
            let weight = self.total_weights[i];
            if weight > 0.0 {
                let w_inv = 1.0 / weight;
                self.buffer[i] = Mat4::from_loc_rot_scale(
                    self.location_buffer[i] * w_inv,
                    Quaternion::from_expmap(self.expmap_buffer[i] * w_inv).to_mat3(),
                    self.scale_buffer[i] * w_inv,
                );
            }
        }
    }
}
