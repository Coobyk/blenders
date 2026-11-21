//! Port of `DNA_light_types.h` — `Light`.
#![allow(non_camel_case_types)]
use crate::id::{ID, NIL};

const DEG2RADF: fn(f32) -> f32 = |deg| deg * (std::f32::consts::PI / 180.0);
pub const MAX_MTEX: usize = 18;

/// `eLight_Flag`.
pub const LA_DS_EXPAND: i16 = 1 << 0;
pub const LA_DS_SHOW_TEXS: i16 = 1 << 2;

/// `eLightType`.
pub const LA_LOCAL: i16 = 0;
pub const LA_SUN: i16 = 1;
pub const LA_SPOT: i16 = 2;
pub const LA_AREA: i16 = 4;

/// `eLight_Mode`.
pub type LightMode = i32;
pub const LA_SHADOW: LightMode = 1 << 0;
pub const LA_SQUARE: LightMode = 1 << 7;
pub const LA_SHAD_RAY: LightMode = 1 << 13;
pub const LA_SHOW_CONE: LightMode = 1 << 17;
pub const LA_CUSTOM_ATTENUATION: LightMode = 1 << 20;
pub const LA_USE_SOFT_FALLOFF: LightMode = 1 << 21;
pub const LA_SHAD_RES_ABSOLUTE: LightMode = 1 << 22;
pub const LA_SHADOW_JITTER: LightMode = 1 << 23;
pub const LA_USE_TEMPERATURE: LightMode = 1 << 24;
pub const LA_UNNORMALIZED: LightMode = 1 << 25;

/// `eLightFalloffType`.
pub const LA_FALLOFF_CONSTANT: i16 = 0;
pub const LA_FALLOFF_INVLINEAR: i16 = 1;
pub const LA_FALLOFF_INVSQUARE: i16 = 2;
pub const LA_FALLOFF_CURVE: i16 = 3;
pub const LA_FALLOFF_SLIDERS: i16 = 4;
pub const LA_FALLOFF_INVCOEFFICIENTS: i16 = 5;

/// `eLightAreaShape`.
pub const LA_AREA_SQUARE: i16 = 0;
pub const LA_AREA_RECT: i16 = 1;
pub const LA_AREA_DISK: i16 = 4;
pub const LA_AREA_ELLIPSE: i16 = 5;

/// `Light`.
#[derive(Clone)]
pub struct Light {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    pub type_: i16,
    pub flag: i16,
    pub mode: LightMode,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub temperature: f32,
    pub energy: f32,
    pub exposure: f32,
    pub radius: f32,
    pub spotsize: f32,
    pub spotblend: f32,
    pub area_shape: i16,
    pub area_size: f32,
    pub area_sizey: f32,
    pub area_sizez: f32,
    pub area_spread: f32,
    pub sun_angle: f32,
    pub pr_texture: i16,
    /// DNA_DEPRECATED `use_nodes`.
    pub use_nodes: i16,
    pub clipsta: f32,
    pub clipend_deprecated: f32,
    pub cascade_max_dist: f32,
    pub cascade_exponent: f32,
    pub cascade_fade: f32,
    pub cascade_count: i32,
    pub diff_fac: f32,
    pub spec_fac: f32,
    pub transmission_fac: f32,
    pub volume_fac: f32,
    pub att_dist: f32,
    pub shadow_filter_radius: f32,
    pub shadow_maximum_resolution: f32,
    pub shadow_jitter_overblur: f32,
    /// `PreviewImage *preview`.
    pub preview: usize,
    /// `bNodeTree *nodetree`.
    pub nodetree: usize,
    /// DNA_DEPRECATED `energy_deprecated`.
    pub energy_deprecated: f32,
}
impl Default for Light {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            type_: LA_LOCAL,
            flag: 0,
            mode: LA_SHADOW | LA_USE_SOFT_FALLOFF,
            r: 1.0,
            g: 1.0,
            b: 1.0,
            temperature: 6500.0,
            energy: 10.0,
            exposure: 0.0,
            radius: 0.0,
            spotsize: DEG2RADF(45.0),
            spotblend: 0.15,
            area_shape: LA_AREA_SQUARE,
            area_size: 0.25,
            area_sizey: 0.25,
            area_sizez: 0.25,
            area_spread: DEG2RADF(180.0),
            sun_angle: DEG2RADF(0.526),
            pr_texture: 0,
            use_nodes: 0,
            clipsta: 0.05,
            clipend_deprecated: 0.0,
            cascade_max_dist: 200.0,
            cascade_exponent: 0.8,
            cascade_fade: 0.1,
            cascade_count: 4,
            diff_fac: 1.0,
            spec_fac: 1.0,
            transmission_fac: 1.0,
            volume_fac: 1.0,
            att_dist: 40.0,
            shadow_filter_radius: 1.0,
            shadow_maximum_resolution: 0.001,
            shadow_jitter_overblur: 10.0,
            preview: NIL,
            nodetree: NIL,
            energy_deprecated: 10.0,
        }
    }
}
