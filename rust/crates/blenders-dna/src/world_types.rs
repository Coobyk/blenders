//! Port of `DNA_world_types.h` — `World` ID block.

use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;

/// `eWorld_Mode` (short).
pub type WorldMode = i16;
pub const WO_MIST: WorldMode = 1 << 0;
pub const WO_MODE_UNUSED_1: WorldMode = 1 << 1;
pub const WO_MODE_UNUSED_2: WorldMode = 1 << 2;
pub const WO_MODE_UNUSED_3: WorldMode = 1 << 3;
pub const WO_MODE_UNUSED_4: WorldMode = 1 << 4;
pub const WO_MODE_UNUSED_5: WorldMode = 1 << 5;
pub const WO_MODE_UNUSED_6: WorldMode = 1 << 6;
pub const WO_MODE_UNUSED_7: WorldMode = 1 << 7;

/// `eWorld_MistType` (short).
pub type WorldMistType = i16;
pub const WO_MIST_QUADRATIC: WorldMistType = 0;
pub const WO_MIST_LINEAR: WorldMistType = 1;
pub const WO_MIST_INVERSE_QUADRATIC: WorldMistType = 2;

/// `eWorld_Flag` (short).
pub type WorldFlag = i16;
pub const WO_DS_EXPAND: WorldFlag = 1 << 0;
pub const WO_DS_SHOW_TEXS: WorldFlag = 1 << 2;
pub const WO_USE_EEVEE_FINITE_VOLUME: WorldFlag = 1 << 3;
pub const WO_USE_SUN_SHADOW: WorldFlag = 1 << 4;
pub const WO_USE_SUN_SHADOW_JITTER: WorldFlag = 1 << 5;

/// `eLightProbeResolution`.
pub type LightProbeResolution = i32;
pub const LIGHT_PROBE_RESOLUTION_128: LightProbeResolution = 7;
pub const LIGHT_PROBE_RESOLUTION_256: LightProbeResolution = 8;
pub const LIGHT_PROBE_RESOLUTION_512: LightProbeResolution = 9;
pub const LIGHT_PROBE_RESOLUTION_1024: LightProbeResolution = 10;
pub const LIGHT_PROBE_RESOLUTION_2048: LightProbeResolution = 11;
pub const LIGHT_PROBE_RESOLUTION_4096: LightProbeResolution = 12;

/// `World`.
#[derive(Clone)]
pub struct World {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    pub texact: i16,
    pub mistype: WorldMistType,
    pub horr: f32,
    pub horg: f32,
    pub horb: f32,
    pub exposure: f32,
    pub exp: f32,
    pub range: f32,
    pub mode: WorldMode,
    pub flag: WorldFlag,
    pub misi: f32,
    pub miststa: f32,
    pub mistdist: f32,
    pub misthi: f32,
    pub aodist: f32,
    pub aoenergy: f32,
    pub probe_resolution: LightProbeResolution,
    pub sun_threshold: f32,
    pub sun_angle: f32,
    pub sun_shadow_maximum_resolution: f32,
    pub sun_shadow_jitter_overblur: f32,
    pub sun_shadow_filter_radius: f32,
    pub pr_texture: i16,
    /// `DNA_DEPRECATED short use_nodes`.
    pub use_nodes: i16,
    /// `PreviewImage *preview`.
    pub preview: usize,
    /// `bNodeTree *nodetree`.
    pub nodetree: usize,
    /// `LightgroupMembership *lightgroup`.
    pub lightgroup: usize,
    /// `ListBaseT<LinkData> gpumaterial`.
    pub gpumaterial: ListBase,
    pub last_update: u64,
}
impl Default for World {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            texact: 0,
            mistype: WO_MIST_QUADRATIC,
            horr: 0.05,
            horg: 0.05,
            horb: 0.05,
            exposure: 0.0,
            exp: 0.0,
            range: 0.0,
            mode: 0,
            flag: WO_USE_SUN_SHADOW,
            misi: 0.0,
            miststa: 5.0,
            mistdist: 25.0,
            misthi: 0.0,
            aodist: 10.0,
            aoenergy: 1.0,
            probe_resolution: LIGHT_PROBE_RESOLUTION_1024,
            sun_threshold: 10.0,
            sun_angle: 0.526_f32.to_radians(),
            sun_shadow_maximum_resolution: 0.001,
            sun_shadow_jitter_overblur: 0.0,
            sun_shadow_filter_radius: 1.0,
            pr_texture: 0,
            use_nodes: 0,
            preview: NIL,
            nodetree: NIL,
            lightgroup: NIL,
            gpumaterial: ListBase::default(),
            last_update: 0,
        }
    }
}
