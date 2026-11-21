//! Port of `intern/type_conversions.cc` — implicit conversions
//! between field/geometry data types (`float ↔ vec/int/color/quat…`).
//!
//! Values use `CValue` (variant standing in for `CPPType` slots);
//! `DataTypeConversions` mirrors `conversions_by_type_index_`.

use std::collections::HashMap;

use blenders_math::color::{linearrgb_to_srgb_v4, srgb_to_linearrgb_v4};
use blenders_math::quaternion::Quaternion;
use blenders_math::vector::{Vec2, Vec3};

/// `CPPType::type_index` stand-in for supported single types.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DataKind {
    /// `float`.
    Float,
    /// `float2`.
    Float2,
    /// `float3`.
    Float3,
    /// `float4`.
    Float4,
    /// `int32`.
    Int,
    /// `int2`.
    Int2,
    /// `int3`.
    Int3,
    /// `short2`.
    Short2,
    /// `int8`.
    Int8,
    /// `bool`.
    Bool,
    /// `ColorGeometry4f`.
    Color4f,
    /// `ColorGeometry4b`.
    Color4b,
    /// `math::Quaternion`.
    Quaternion,
    /// `float4x4`.
    Mat4,
}

/// `CValue` — one value of a supported type.
#[derive(Clone, Copy, Debug)]
pub enum CValue {
    /// `float`.
    Float(f32),
    /// `float2`.
    Float2([f32; 2]),
    /// `float3`.
    Float3([f32; 3]),
    /// `float4`.
    Float4([f32; 4]),
    /// `int32`.
    Int(i32),
    /// `int2`.
    Int2([i32; 2]),
    /// `int3`.
    Int3([i32; 3]),
    /// `short2`.
    Short2([i16; 2]),
    /// `int8`.
    Int8(i8),
    /// `bool`.
    Bool(bool),
    /// `ColorGeometry4f`.
    Color4f([f32; 4]),
    /// `ColorGeometry4b`.
    Color4b([u8; 4]),
    /// `Quaternion`.
    Quaternion(Quaternion),
    /// `float4x4` column-major.
    Mat4([[f32; 4]; 4]),
}

fn f32_to_i8(a: f32) -> i8 {
    a.clamp(i8::MIN as f32, i8::MAX as f32) as i8
}
fn luminance(c: [f32; 4]) -> f32 {
    /* `IMB_colormanagement_get_luminance`. */
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}
fn color_encode(c: [f32; 4]) -> [u8; 4] {
    let e = linearrgb_to_srgb_v4(c);
    [
        (e[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (e[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (e[2].clamp(0.0, 1.0) * 255.0).round() as u8,
        (e[3].clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}
fn color_decode(c: [u8; 4]) -> [f32; 4] {
    srgb_to_linearrgb_v4([
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        c[3] as f32 / 255.0,
    ])
}
fn midpoint(a: i32, b: i32) -> i32 {
    a.wrapping_add(b) / 2
}

/* --- Per-source conversion impls (C `static T x_to_y`). --- */

fn convert(v: &CValue, to: DataKind) -> Option<CValue> {
    Some(match (v, to) {
        /* float source. */
        (CValue::Float(a), DataKind::Float2) => CValue::Float2([*a, *a]),
        (CValue::Float(a), DataKind::Float3) => CValue::Float3([*a, *a, *a]),
        (CValue::Float(a), DataKind::Float4) => CValue::Float4([*a; 4]),
        (CValue::Float(a), DataKind::Int) => CValue::Int(*a as i32),
        (CValue::Float(a), DataKind::Short2) => CValue::Short2([*a as i16; 2]),
        (CValue::Float(a), DataKind::Int2) => CValue::Int2([*a as i32; 2]),
        (CValue::Float(a), DataKind::Int3) => CValue::Int3([*a as i32; 3]),
        (CValue::Float(a), DataKind::Bool) => CValue::Bool(*a > 0.0),
        (CValue::Float(a), DataKind::Int8) => CValue::Int8(f32_to_i8(*a)),
        (CValue::Float(a), DataKind::Color4f) => CValue::Color4f([*a, *a, *a, 1.0]),
        (CValue::Float(a), DataKind::Color4b) => CValue::Color4b(color_encode([*a, *a, *a, 1.0])),
        (CValue::Float(a), DataKind::Quaternion) => CValue::Quaternion(
            blenders_math::euler::EulerXYZ::new(*a, *a, *a).to_quaternion(),
        ),

        /* float2 source. */
        (CValue::Float2(a), DataKind::Float3) => CValue::Float3([a[0], a[1], 0.0]),
        (CValue::Float2(a), DataKind::Float4) => CValue::Float4([a[0], a[1], 0.0, 0.0]),
        (CValue::Float2(a), DataKind::Float) => CValue::Float((a[0] + a[1]) / 2.0),
        (CValue::Float2(a), DataKind::Int) => CValue::Int(((a[0] + a[1]) / 2.0) as i32),
        (CValue::Float2(a), DataKind::Short2) => {
            CValue::Short2([a[0] as i16, a[1] as i16])
        }
        (CValue::Float2(a), DataKind::Int2) => CValue::Int2([a[0] as i32, a[1] as i32]),
        (CValue::Float2(a), DataKind::Int3) => CValue::Int3([a[0] as i32, a[1] as i32, 0]),
        (CValue::Float2(a), DataKind::Bool) => CValue::Bool(*a != [0.0; 2]),
        (CValue::Float2(a), DataKind::Int8) => {
            CValue::Int8(f32_to_i8((a[0] + a[1]) / 2.0))
        }
        (CValue::Float2(a), DataKind::Color4f) => CValue::Color4f([a[0], a[1], 0.0, 1.0]),
        (CValue::Float2(a), DataKind::Color4b) => {
            CValue::Color4b(color_encode([a[0], a[1], 0.0, 1.0]))
        }
        (CValue::Float2(a), DataKind::Quaternion) => CValue::Quaternion(
            blenders_math::euler::EulerXYZ::new(a[0], a[1], 0.0).to_quaternion(),
        ),

        /* float3 source. */
        (CValue::Float3(a), DataKind::Bool) => CValue::Bool(*a != [0.0; 3]),
        (CValue::Float3(a), DataKind::Int8) => {
            CValue::Int8(f32_to_i8((a[0] + a[1] + a[2]) / 3.0))
        }
        (CValue::Float3(a), DataKind::Float) => CValue::Float((a[0] + a[1] + a[2]) / 3.0),
        (CValue::Float3(a), DataKind::Int) => CValue::Int(((a[0] + a[1] + a[2]) / 3.0) as i32),
        (CValue::Float3(a), DataKind::Short2) => CValue::Short2([a[0] as i16, a[1] as i16]),
        (CValue::Float3(a), DataKind::Int2) => CValue::Int2([a[0] as i32, a[1] as i32]),
        (CValue::Float3(a), DataKind::Int3) => {
            CValue::Int3([a[0] as i32, a[1] as i32, a[2] as i32])
        }
        (CValue::Float3(a), DataKind::Float2) => CValue::Float2([a[0], a[1]]),
        (CValue::Float3(a), DataKind::Float4) => CValue::Float4([a[0], a[1], a[2], 0.0]),
        (CValue::Float3(a), DataKind::Color4f) => CValue::Color4f([a[0], a[1], a[2], 1.0]),
        (CValue::Float3(a), DataKind::Color4b) => {
            CValue::Color4b(color_encode([a[0], a[1], a[2], 1.0]))
        }
        (CValue::Float3(a), DataKind::Quaternion) => {
            CValue::Quaternion(blenders_math::euler::EulerXYZ::new(a[0], a[1], a[2]).to_quaternion())
        }

        /* float4 source. */
        (CValue::Float4(a), DataKind::Bool) => CValue::Bool(*a != [0.0; 4]),
        (CValue::Float4(a), DataKind::Int8) => {
            CValue::Int8(f32_to_i8((a[0] + a[1] + a[2] + a[3]) / 4.0))
        }
        (CValue::Float4(a), DataKind::Float) => {
            CValue::Float((a[0] + a[1] + a[2] + a[3]) / 4.0)
        }
        (CValue::Float4(a), DataKind::Int) => {
            CValue::Int(((a[0] + a[1] + a[2] + a[3]) / 4.0) as i32)
        }
        (CValue::Float4(a), DataKind::Short2) => CValue::Short2([a[0] as i16, a[1] as i16]),
        (CValue::Float4(a), DataKind::Int2) => CValue::Int2([a[0] as i32, a[1] as i32]),
        (CValue::Float4(a), DataKind::Int3) => {
            CValue::Int3([a[0] as i32, a[1] as i32, a[2] as i32])
        }
        (CValue::Float4(a), DataKind::Float2) => CValue::Float2([a[0], a[1]]),
        (CValue::Float4(a), DataKind::Float3) => CValue::Float3([a[0], a[1], a[2]]),
        (CValue::Float4(a), DataKind::Color4f) => CValue::Color4f(*a),
        (CValue::Float4(a), DataKind::Color4b) => CValue::Color4b(color_encode(*a)),
        (CValue::Float4(a), DataKind::Quaternion) => CValue::Quaternion(
            blenders_math::euler::EulerXYZ::new(a[0], a[1], a[2]).to_quaternion(),
        ),

        /* int32 source. */
        (CValue::Int(a), DataKind::Bool) => CValue::Bool(*a != 0),
        (CValue::Int(a), DataKind::Int8) => {
            CValue::Int8((*a).clamp(i8::MIN as i32, i8::MAX as i32) as i8)
        }
        (CValue::Int(a), DataKind::Short2) => CValue::Short2([*a as i16; 2]),
        (CValue::Int(a), DataKind::Int2) => CValue::Int2([*a; 2]),
        (CValue::Int(a), DataKind::Int3) => CValue::Int3([*a; 3]),
        (CValue::Int(a), DataKind::Float) => CValue::Float(*a as f32),
        (CValue::Int(a), DataKind::Float2) => CValue::Float2([*a as f32; 2]),
        (CValue::Int(a), DataKind::Float3) => CValue::Float3([*a as f32; 3]),
        (CValue::Int(a), DataKind::Float4) => CValue::Float4([*a as f32; 4]),
        (CValue::Int(a), DataKind::Color4f) => {
            CValue::Color4f([*a as f32, *a as f32, *a as f32, 1.0])
        }
        (CValue::Int(a), DataKind::Color4b) => {
            CValue::Color4b(color_encode([*a as f32, *a as f32, *a as f32, 1.0]))
        }

        /* short2 source. */
        (CValue::Short2(a), DataKind::Bool) => CValue::Bool(*a != [0; 2]),
        (CValue::Short2(a), DataKind::Int8) => {
            let m = midpoint(a[0] as i32, a[1] as i32);
            CValue::Int8(m.clamp(i8::MIN as i32, i8::MAX as i32) as i8)
        }
        (CValue::Short2(a), DataKind::Int) => CValue::Int(midpoint(a[0] as i32, a[1] as i32)),
        (CValue::Short2(a), DataKind::Int2) => CValue::Int2([a[0] as i32, a[1] as i32]),
        (CValue::Short2(a), DataKind::Int3) => CValue::Int3([a[0] as i32, a[1] as i32, 0]),
        (CValue::Short2(a), DataKind::Float) => {
            CValue::Float((a[0] as f32 + a[1] as f32) / 2.0)
        }
        (CValue::Short2(a), DataKind::Float2) => {
            CValue::Float2([a[0] as f32, a[1] as f32])
        }
        (CValue::Short2(a), DataKind::Float3) => {
            CValue::Float3([a[0] as f32, a[1] as f32, 0.0])
        }
        (CValue::Short2(a), DataKind::Float4) => {
            CValue::Float4([a[0] as f32, a[1] as f32, 0.0, 0.0])
        }
        (CValue::Short2(a), DataKind::Color4f) => {
            CValue::Color4f([a[0] as f32, a[1] as f32, 0.0, 1.0])
        }
        (CValue::Short2(a), DataKind::Color4b) => {
            CValue::Color4b([a[0] as u8, a[1] as u8, 0, 255])
        }

        /* int2 source. */
        (CValue::Int2(a), DataKind::Bool) => CValue::Bool(*a != [0; 2]),
        (CValue::Int2(a), DataKind::Int8) => {
            let m = midpoint(a[0], a[1]);
            CValue::Int8(m.clamp(i8::MIN as i32, i8::MAX as i32) as i8)
        }
        (CValue::Int2(a), DataKind::Int) => CValue::Int(midpoint(a[0], a[1])),
        (CValue::Int2(a), DataKind::Int3) => CValue::Int3([a[0], a[1], 0]),
        (CValue::Int2(a), DataKind::Short2) => CValue::Short2([a[0] as i16, a[1] as i16]),
        (CValue::Int2(a), DataKind::Float) => CValue::Float((a[0] + a[1]) as f32 / 2.0),
        (CValue::Int2(a), DataKind::Float2) => CValue::Float2([a[0] as f32, a[1] as f32]),
        (CValue::Int2(a), DataKind::Float3) => {
            CValue::Float3([a[0] as f32, a[1] as f32, 0.0])
        }
        (CValue::Int2(a), DataKind::Float4) => {
            CValue::Float4([a[0] as f32, a[1] as f32, 0.0, 0.0])
        }
        (CValue::Int2(a), DataKind::Color4f) => {
            CValue::Color4f([a[0] as f32, a[1] as f32, 0.0, 1.0])
        }
        (CValue::Int2(a), DataKind::Color4b) => {
            CValue::Color4b(color_encode([a[0] as f32, a[1] as f32, 0.0, 1.0]))
        }

        /* int3 source. */
        (CValue::Int3(a), DataKind::Bool) => CValue::Bool(*a != [0; 3]),
        (CValue::Int3(a), DataKind::Int2) => CValue::Int2([a[0], a[1]]),
        (CValue::Int3(a), DataKind::Short2) => CValue::Short2([a[0] as i16, a[1] as i16]),
        (CValue::Int3(a), DataKind::Int8) => {
            let m = (a[0] + a[1] + a[2]) / 3;
            CValue::Int8(m.clamp(i8::MIN as i32, i8::MAX as i32) as i8)
        }
        (CValue::Int3(a), DataKind::Int) => CValue::Int((a[0] + a[1] + a[2]) / 3),
        (CValue::Int3(a), DataKind::Float) => {
            CValue::Float((a[0] + a[1] + a[2]) as f32 / 3.0)
        }
        (CValue::Int3(a), DataKind::Float2) => CValue::Float2([a[0] as f32, a[1] as f32]),
        (CValue::Int3(a), DataKind::Float3) => {
            CValue::Float3([a[0] as f32, a[1] as f32, a[2] as f32])
        }
        (CValue::Int3(a), DataKind::Float4) => {
            CValue::Float4([a[0] as f32, a[1] as f32, a[2] as f32, 0.0])
        }
        (CValue::Int3(a), DataKind::Color4f) => {
            CValue::Color4f([a[0] as f32, a[1] as f32, a[2] as f32, 1.0])
        }
        (CValue::Int3(a), DataKind::Color4b) => {
            CValue::Color4b(color_encode([a[0] as f32, a[1] as f32, a[2] as f32, 1.0]))
        }

        /* int8 source. */
        (CValue::Int8(a), DataKind::Bool) => CValue::Bool(*a != 0),
        (CValue::Int8(a), DataKind::Int) => CValue::Int(*a as i32),
        (CValue::Int8(a), DataKind::Short2) => CValue::Short2([*a as i16; 2]),
        (CValue::Int8(a), DataKind::Int2) => CValue::Int2([*a as i32; 2]),
        (CValue::Int8(a), DataKind::Int3) => CValue::Int3([*a as i32; 3]),
        (CValue::Int8(a), DataKind::Float) => CValue::Float(*a as f32),
        (CValue::Int8(a), DataKind::Float2) => CValue::Float2([*a as f32; 2]),
        (CValue::Int8(a), DataKind::Float3) => CValue::Float3([*a as f32; 3]),
        (CValue::Int8(a), DataKind::Float4) => CValue::Float4([*a as f32; 4]),
        (CValue::Int8(a), DataKind::Color4f) => {
            CValue::Color4f([*a as f32, *a as f32, *a as f32, 1.0])
        }
        (CValue::Int8(a), DataKind::Color4b) => {
            CValue::Color4b(color_encode([*a as f32, *a as f32, *a as f32, 1.0]))
        }

        /* bool source. */
        (CValue::Bool(a), DataKind::Float) => CValue::Float(*a as i32 as f32),
        (CValue::Bool(a), DataKind::Int8) => CValue::Int8(*a as i8),
        (CValue::Bool(a), DataKind::Int) => CValue::Int(*a as i32),
        (CValue::Bool(a), DataKind::Short2) => CValue::Short2([*a as i16; 2]),
        (CValue::Bool(a), DataKind::Int2) => CValue::Int2([*a as i32; 2]),
        (CValue::Bool(a), DataKind::Int3) => CValue::Int3([*a as i32; 3]),
        (CValue::Bool(a), DataKind::Float2) => CValue::Float2([*a as i32 as f32; 2]),
        (CValue::Bool(a), DataKind::Float3) => CValue::Float3([*a as i32 as f32; 3]),
        (CValue::Bool(a), DataKind::Float4) => CValue::Float4([*a as i32 as f32; 4]),
        (CValue::Bool(a), DataKind::Color4f) => {
            let f = *a as i32 as f32;
            CValue::Color4f([f, f, f, 1.0])
        }
        (CValue::Bool(a), DataKind::Color4b) => {
            let f = *a as i32 as f32;
            CValue::Color4b(color_encode([f, f, f, 1.0]))
        }

        /* ColorGeometry4f source. */
        (CValue::Color4f(a), DataKind::Bool) => CValue::Bool(luminance(*a) > 0.0),
        (CValue::Color4f(a), DataKind::Int8) => CValue::Int8(f32_to_i8(luminance(*a))),
        (CValue::Color4f(a), DataKind::Float) => CValue::Float(luminance(*a)),
        (CValue::Color4f(a), DataKind::Int) => CValue::Int(luminance(*a) as i32),
        (CValue::Color4f(a), DataKind::Short2) => {
            CValue::Short2([a[0] as i16, a[1] as i16])
        }
        (CValue::Color4f(a), DataKind::Int2) => CValue::Int2([a[0] as i32, a[1] as i32]),
        (CValue::Color4f(a), DataKind::Int3) => {
            CValue::Int3([a[0] as i32, a[1] as i32, a[2] as i32])
        }
        (CValue::Color4f(a), DataKind::Float2) => CValue::Float2([a[0], a[1]]),
        (CValue::Color4f(a), DataKind::Float3) => CValue::Float3([a[0], a[1], a[2]]),
        (CValue::Color4f(a), DataKind::Float4) => CValue::Float4(*a),
        (CValue::Color4f(a), DataKind::Color4b) => CValue::Color4b(color_encode(*a)),

        /* ColorGeometry4b source. */
        (CValue::Color4b(a), DataKind::Bool) => {
            CValue::Bool(a[0] > 0 || a[1] > 0 || a[2] > 0)
        }
        (CValue::Color4b(a), DataKind::Int8) => {
            CValue::Int8(f32_to_i8(luminance(color_decode(*a))))
        }
        (CValue::Color4b(a), DataKind::Float) => CValue::Float(luminance(color_decode(*a))),
        (CValue::Color4b(a), DataKind::Int) => {
            CValue::Int(luminance(color_decode(*a)) as i32)
        }
        (CValue::Color4b(a), DataKind::Short2) => CValue::Short2([a[0] as i16, a[1] as i16]),
        (CValue::Color4b(a), DataKind::Int2) => CValue::Int2([a[0] as i32, a[1] as i32]),
        (CValue::Color4b(a), DataKind::Int3) => {
            CValue::Int3([a[0] as i32, a[1] as i32, a[2] as i32])
        }
        (CValue::Color4b(a), DataKind::Float2) => {
            let c = color_decode(*a);
            CValue::Float2([c[0], c[1]])
        }
        (CValue::Color4b(a), DataKind::Float3) => {
            let c = color_decode(*a);
            CValue::Float3([c[0], c[1], c[2]])
        }
        (CValue::Color4b(a), DataKind::Float4) => CValue::Float4(color_decode(*a)),
        (CValue::Color4b(a), DataKind::Color4f) => CValue::Color4f(color_decode(*a)),

        /* Quaternion / Mat4. */
        (CValue::Mat4(m), DataKind::Quaternion) => {
            /* `normalized_to_quaternion_safe(normalize(float3x3))`. */
            let m3 = blenders_math::matrix::Mat3::from_cols(
                Vec3::new(m[0][0], m[0][1], m[0][2]).normalized(),
                Vec3::new(m[1][0], m[1][1], m[1][2]).normalized(),
                Vec3::new(m[2][0], m[2][1], m[2][2]).normalized(),
            );
            CValue::Quaternion(blenders_math::matrix::to_quaternion_legacy(m3))
        }
        (CValue::Quaternion(q), DataKind::Float2) => {
            let e = blenders_math::euler::quat_to_euler_xyz(*q);
            CValue::Float2([e.x, e.y])
        }
        (CValue::Quaternion(q), DataKind::Float3) => {
            let e = blenders_math::euler::quat_to_euler_xyz(*q);
            CValue::Float3([e.x, e.y, e.z])
        }
        (CValue::Quaternion(q), DataKind::Float4) => CValue::Float4([q.w, q.x, q.y, q.z]),
        (CValue::Quaternion(q), DataKind::Mat4) => {
            let m3 = q.to_mat3();
            CValue::Mat4([
                [m3[0].x, m3[0].y, m3[0].z, 0.0],
                [m3[1].x, m3[1].y, m3[1].z, 0.0],
                [m3[2].x, m3[2].y, m3[2].z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ])
        }

        _ => return None,
    })
}

/// `ConversionFunctions` — name + single-value converter.
pub struct ConversionFunctions {
    /// `from`/`to`.
    pub from: DataKind,
    /// `to`.
    pub to: DataKind,
    /// `convert_single_to_uninitialized`.
    pub convert: fn(&CValue, DataKind) -> Option<CValue>,
}

/// `DataTypeConversions`.
#[derive(Default)]
pub struct DataTypeConversions {
    conversions: HashMap<(DataKind, DataKind), ConversionFunctions>,
}

impl DataTypeConversions {
    /// `get_implicit_conversions` — the C global.
    pub fn implicit() -> Self {
        let mut c = Self::default();
        /* Register every (from,to) pair the C lists. */
        let pairs = [
            (DataKind::Float, DataKind::Float2),
            (DataKind::Float, DataKind::Float3),
            (DataKind::Float, DataKind::Float4),
            (DataKind::Float, DataKind::Int),
            (DataKind::Float, DataKind::Short2),
            (DataKind::Float, DataKind::Int2),
            (DataKind::Float, DataKind::Int3),
            (DataKind::Float, DataKind::Bool),
            (DataKind::Float, DataKind::Int8),
            (DataKind::Float, DataKind::Color4f),
            (DataKind::Float, DataKind::Color4b),
            (DataKind::Float, DataKind::Quaternion),
            (DataKind::Float2, DataKind::Float3),
            (DataKind::Float2, DataKind::Float4),
            (DataKind::Float2, DataKind::Float),
            (DataKind::Float2, DataKind::Int),
            (DataKind::Float2, DataKind::Short2),
            (DataKind::Float2, DataKind::Int2),
            (DataKind::Float2, DataKind::Int3),
            (DataKind::Float2, DataKind::Bool),
            (DataKind::Float2, DataKind::Int8),
            (DataKind::Float2, DataKind::Color4f),
            (DataKind::Float2, DataKind::Color4b),
            (DataKind::Float2, DataKind::Quaternion),
            (DataKind::Float3, DataKind::Bool),
            (DataKind::Float3, DataKind::Int8),
            (DataKind::Float3, DataKind::Float),
            (DataKind::Float3, DataKind::Int),
            (DataKind::Float3, DataKind::Short2),
            (DataKind::Float3, DataKind::Int2),
            (DataKind::Float3, DataKind::Int3),
            (DataKind::Float3, DataKind::Float2),
            (DataKind::Float3, DataKind::Float4),
            (DataKind::Float3, DataKind::Color4f),
            (DataKind::Float3, DataKind::Color4b),
            (DataKind::Float3, DataKind::Quaternion),
            (DataKind::Float4, DataKind::Bool),
            (DataKind::Float4, DataKind::Int8),
            (DataKind::Float4, DataKind::Float),
            (DataKind::Float4, DataKind::Int),
            (DataKind::Float4, DataKind::Short2),
            (DataKind::Float4, DataKind::Int2),
            (DataKind::Float4, DataKind::Int3),
            (DataKind::Float4, DataKind::Float2),
            (DataKind::Float4, DataKind::Float3),
            (DataKind::Float4, DataKind::Color4f),
            (DataKind::Float4, DataKind::Color4b),
            (DataKind::Float4, DataKind::Quaternion),
            (DataKind::Int, DataKind::Bool),
            (DataKind::Int, DataKind::Int8),
            (DataKind::Int, DataKind::Short2),
            (DataKind::Int, DataKind::Int2),
            (DataKind::Int, DataKind::Int3),
            (DataKind::Int, DataKind::Float),
            (DataKind::Int, DataKind::Float2),
            (DataKind::Int, DataKind::Float3),
            (DataKind::Int, DataKind::Float4),
            (DataKind::Int, DataKind::Color4f),
            (DataKind::Int, DataKind::Color4b),
            (DataKind::Short2, DataKind::Bool),
            (DataKind::Short2, DataKind::Int8),
            (DataKind::Short2, DataKind::Int),
            (DataKind::Short2, DataKind::Int2),
            (DataKind::Short2, DataKind::Int3),
            (DataKind::Short2, DataKind::Float),
            (DataKind::Short2, DataKind::Float2),
            (DataKind::Short2, DataKind::Float3),
            (DataKind::Short2, DataKind::Float4),
            (DataKind::Short2, DataKind::Color4f),
            (DataKind::Short2, DataKind::Color4b),
            (DataKind::Int2, DataKind::Bool),
            (DataKind::Int2, DataKind::Int8),
            (DataKind::Int2, DataKind::Int),
            (DataKind::Int2, DataKind::Int3),
            (DataKind::Int2, DataKind::Short2),
            (DataKind::Int2, DataKind::Float),
            (DataKind::Int2, DataKind::Float2),
            (DataKind::Int2, DataKind::Float3),
            (DataKind::Int2, DataKind::Float4),
            (DataKind::Int2, DataKind::Color4f),
            (DataKind::Int2, DataKind::Color4b),
            (DataKind::Int3, DataKind::Bool),
            (DataKind::Int3, DataKind::Int2),
            (DataKind::Int3, DataKind::Short2),
            (DataKind::Int3, DataKind::Int8),
            (DataKind::Int3, DataKind::Int),
            (DataKind::Int3, DataKind::Float),
            (DataKind::Int3, DataKind::Float2),
            (DataKind::Int3, DataKind::Float3),
            (DataKind::Int3, DataKind::Float4),
            (DataKind::Int3, DataKind::Color4f),
            (DataKind::Int3, DataKind::Color4b),
            (DataKind::Int8, DataKind::Bool),
            (DataKind::Int8, DataKind::Int),
            (DataKind::Int8, DataKind::Short2),
            (DataKind::Int8, DataKind::Int2),
            (DataKind::Int8, DataKind::Int3),
            (DataKind::Int8, DataKind::Float),
            (DataKind::Int8, DataKind::Float2),
            (DataKind::Int8, DataKind::Float3),
            (DataKind::Int8, DataKind::Float4),
            (DataKind::Int8, DataKind::Color4f),
            (DataKind::Int8, DataKind::Color4b),
            (DataKind::Bool, DataKind::Float),
            (DataKind::Bool, DataKind::Int8),
            (DataKind::Bool, DataKind::Int),
            (DataKind::Bool, DataKind::Short2),
            (DataKind::Bool, DataKind::Int2),
            (DataKind::Bool, DataKind::Int3),
            (DataKind::Bool, DataKind::Float2),
            (DataKind::Bool, DataKind::Float3),
            (DataKind::Bool, DataKind::Float4),
            (DataKind::Bool, DataKind::Color4f),
            (DataKind::Bool, DataKind::Color4b),
            (DataKind::Color4f, DataKind::Bool),
            (DataKind::Color4f, DataKind::Int8),
            (DataKind::Color4f, DataKind::Float),
            (DataKind::Color4f, DataKind::Int),
            (DataKind::Color4f, DataKind::Short2),
            (DataKind::Color4f, DataKind::Int2),
            (DataKind::Color4f, DataKind::Int3),
            (DataKind::Color4f, DataKind::Float2),
            (DataKind::Color4f, DataKind::Float3),
            (DataKind::Color4f, DataKind::Float4),
            (DataKind::Color4f, DataKind::Color4b),
            (DataKind::Color4b, DataKind::Bool),
            (DataKind::Color4b, DataKind::Int8),
            (DataKind::Color4b, DataKind::Float),
            (DataKind::Color4b, DataKind::Int),
            (DataKind::Color4b, DataKind::Short2),
            (DataKind::Color4b, DataKind::Int2),
            (DataKind::Color4b, DataKind::Int3),
            (DataKind::Color4b, DataKind::Float2),
            (DataKind::Color4b, DataKind::Float3),
            (DataKind::Color4b, DataKind::Float4),
            (DataKind::Color4b, DataKind::Color4f),
            (DataKind::Mat4, DataKind::Quaternion),
            (DataKind::Quaternion, DataKind::Float2),
            (DataKind::Quaternion, DataKind::Float3),
            (DataKind::Quaternion, DataKind::Float4),
            (DataKind::Quaternion, DataKind::Mat4),
        ];
        for (from, to) in pairs {
            c.conversions.insert(
                (from, to),
                ConversionFunctions {
                    from,
                    to,
                    convert,
                },
            );
        }
        c
    }

    /// `is_convertible` — `get_conversion_functions != nullptr`.
    pub fn is_convertible(&self, from: DataKind, to: DataKind) -> bool {
        from == to || self.conversions.contains_key(&(from, to))
    }

    /// `get_conversion_functions`.
    pub fn get_conversion_functions(&self, from: DataKind, to: DataKind) -> Option<&ConversionFunctions> {
        self.conversions.get(&(from, to))
    }

    /// `convert` — direct identity or via table.
    pub fn convert(&self, from: DataKind, to: DataKind, v: &CValue) -> Option<CValue> {
        if from == to {
            return Some(*v);
        }
        (self.get_conversion_functions(from, to)?.convert)(v, to)
    }
}
