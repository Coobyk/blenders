//! Port of `DNA_lightprobe_types.h` — `LightProbe` ID and light-cache structures.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::id::{ID, NIL};

/// `LIGHTCACHE_STATIC_VERSION`.
pub const LIGHTCACHE_STATIC_VERSION: i32 = 2;
/// `eLightProbeType`.
pub type Lightprobetype = i8;
pub const LIGHTPROBE_TYPE_SPHERE: Lightprobetype = 0;
pub const LIGHTPROBE_TYPE_PLANE: Lightprobetype = 1;
pub const LIGHTPROBE_TYPE_VOLUME: Lightprobetype = 2;

/// `eLightProbe_Flag`.
pub type LightprobeFlag = i8;
pub const LIGHTPROBE_FLAG_CUSTOM_PARALLAX: LightprobeFlag = 1 << 0;
pub const LIGHTPROBE_FLAG_SHOW_INFLUENCE: LightprobeFlag = 1 << 1;
pub const LIGHTPROBE_FLAG_SHOW_PARALLAX: LightprobeFlag = 1 << 2;
pub const LIGHTPROBE_FLAG_SHOW_CLIP_DIST: LightprobeFlag = 1 << 3;
pub const LIGHTPROBE_FLAG_SHOW_DATA: LightprobeFlag = 1 << 4;
pub const LIGHTPROBE_FLAG_INVERT_GROUP: LightprobeFlag = 1 << 5;
pub const LIGHTPROBE_DS_EXPAND: LightprobeFlag = 1 << 6;

/// `eLightProbe_GridFlag`.
pub type LightprobeGridflag = i8;
pub const LIGHTPROBE_GRID_CAPTURE_WORLD: LightprobeGridflag = 1 << 0;
pub const LIGHTPROBE_GRID_CAPTURE_INDIRECT: LightprobeGridflag = 1 << 1;
pub const LIGHTPROBE_GRID_CAPTURE_EMISSION: LightprobeGridflag = 1 << 2;

/// `eLightProbeDisplay`.
pub type Lightprobedisplay = i8;
pub const LIGHTPROBE_DISP_WIRE: Lightprobedisplay = 0;
pub const LIGHTPROBE_DISP_SHADED: Lightprobedisplay = 1;
pub const LIGHTPROBE_DISP_DIFFUSE: Lightprobedisplay = 2;
pub const LIGHTPROBE_DISP_REFLECTIVE: Lightprobedisplay = 3;

/// `eLightProbeShape`.
pub type Lightprobeshape = i8;
pub const LIGHTPROBE_SHAPE_ELLIPSOID: Lightprobeshape = 0;
pub const LIGHTPROBE_SHAPE_BOX: Lightprobeshape = 1;

/// `eLightCacheType`.
pub type Lightcachetype = i32;
pub const LIGHTCACHE_TYPE_STATIC: Lightcachetype = 0;

/// `eLightCache_Flag`.
pub type LightcacheFlag = i32;
pub const LIGHTCACHE_BAKED: LightcacheFlag = 1 << 0;
pub const LIGHTCACHE_BAKING: LightcacheFlag = 1 << 1;
pub const LIGHTCACHE_CUBE_READY: LightcacheFlag = 1 << 2;
pub const LIGHTCACHE_GRID_READY: LightcacheFlag = 1 << 3;
pub const LIGHTCACHE_UPDATE_CUBE: LightcacheFlag = 1 << 4;
pub const LIGHTCACHE_UPDATE_GRID: LightcacheFlag = 1 << 5;
pub const LIGHTCACHE_UPDATE_WORLD: LightcacheFlag = 1 << 6;
pub const LIGHTCACHE_UPDATE_AUTO: LightcacheFlag = 1 << 7;
pub const LIGHTCACHE_INVALID: LightcacheFlag = 1 << 8;
pub const LIGHTCACHE_NOT_USABLE: LightcacheFlag = 1 << 9;

/// `eLightCacheTexture_DataType`.
pub type LightcachetextureDatatype = i8;
pub const LIGHTCACHETEX_BYTE: LightcachetextureDatatype = 1 << 0;
pub const LIGHTCACHETEX_FLOAT: LightcachetextureDatatype = 1 << 1;
pub const LIGHTCACHETEX_UINT: LightcachetextureDatatype = 1 << 2;

/// `eLightProbeGridCacheLayout`.
pub type Lightprobegridcachelayout = i32;
pub const LIGHTPROBE_CACHE_UNIFORM_GRID: Lightprobegridcachelayout = 0;
pub const LIGHTPROBE_CACHE_ADAPTIVE_RESOLUTION: Lightprobegridcachelayout = 1;

/// `eLightProbeObjectCacheType`.
pub type Lightprobeobjectcachetype = i32;
pub const LIGHTPROBE_CACHE_TYPE_NONE: Lightprobeobjectcachetype = 0;
pub const LIGHTPROBE_CACHE_TYPE_STATIC: Lightprobeobjectcachetype = 1;


/// `LightProbe` — light probe ID block.
#[derive(Clone)]
pub struct LightProbe {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    pub type_: i8,
    pub flag: i8,
    pub attenuation_type: i8,
    pub parallax_type: i8,
    pub grid_flag: i8,
    pub distinf: f32,
    pub distpar: f32,
    pub falloff: f32,
    pub clipsta: f32,
    pub clipend: f32,
    pub vis_bias: f32,
    pub vis_bleedbias: f32,
    pub vis_blur: f32,
    pub intensity: f32,
    pub grid_resolution_x: i32,
    pub grid_resolution_y: i32,
    pub grid_resolution_z: i32,
    pub grid_bake_samples: i32,
    pub grid_surface_bias: f32,
    pub grid_escape_bias: f32,
    pub grid_normal_bias: f32,
    pub grid_view_bias: f32,
    pub grid_facing_bias: f32,
    pub grid_validity_threshold: f32,
    pub grid_dilation_threshold: f32,
    pub grid_dilation_radius: f32,
    pub grid_clamp_direct: f32,
    pub grid_clamp_indirect: f32,
    pub grid_surfel_density: i32,
    /// `Collection *visibility_grp`.
    pub visibility_grp: usize,
    pub data_display_size: f32,
}
impl Default for LightProbe {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            type_: LIGHTPROBE_TYPE_SPHERE as i8,
            flag: LIGHTPROBE_FLAG_SHOW_INFLUENCE as i8,
            attenuation_type: LIGHTPROBE_SHAPE_ELLIPSOID as i8,
            parallax_type: LIGHTPROBE_SHAPE_ELLIPSOID as i8,
            grid_flag: (LIGHTPROBE_GRID_CAPTURE_INDIRECT | LIGHTPROBE_GRID_CAPTURE_EMISSION) as i8,
            distinf: 2.5,
            distpar: 2.5,
            falloff: 0.2,
            clipsta: 0.8,
            clipend: 20.0,
            vis_bias: 1.0,
            vis_bleedbias: 0.0,
            vis_blur: 0.2,
            intensity: 1.0,
            grid_resolution_x: 4,
            grid_resolution_y: 4,
            grid_resolution_z: 4,
            grid_bake_samples: 2048,
            grid_surface_bias: 0.05,
            grid_escape_bias: 0.1,
            grid_normal_bias: 0.3,
            grid_view_bias: 0.0,
            grid_facing_bias: 0.5,
            grid_validity_threshold: 0.40,
            grid_dilation_threshold: 0.5,
            grid_dilation_radius: 1.0,
            grid_clamp_direct: 0.0,
            grid_clamp_indirect: 10.0,
            grid_surfel_density: 20,
            visibility_grp: NIL,
            data_display_size: 0.1,
        }
    }
}

/// `LightProbeCache` (16-byte aligned in C++).
#[derive(Clone, Default)]
pub struct LightProbeCache {
    pub position: [f32; 3],
    pub parallax_type: f32,
    pub attenuation_fac: f32,
    pub attenuation_type: f32,
    pub attenuationmat: [[f32; 4]; 4],
    pub parallaxmat: [[f32; 4]; 4],
}

/// `LightGridCache` (16-byte aligned in C++).
#[derive(Clone, Default)]
pub struct LightGridCache {
    pub mat: [[f32; 4]; 4],
    pub resolution: [i32; 3],
    pub offset: i32,
    pub corner: [f32; 3],
    pub attenuation_scale: f32,
    pub increment_x: [f32; 3],
    pub attenuation_bias: f32,
    pub increment_y: [f32; 3],
    pub level_bias: f32,
    pub increment_z: [f32; 3],
    pub visibility_bias: f32,
    pub visibility_bleed: f32,
    pub visibility_range: f32,
}

/// `LightCacheTexture`.
#[derive(Clone, Default)]
pub struct LightCacheTexture {
    /// `gpu::Texture *tex`.
    pub tex: usize,
    /// `char *data` → Vec.
    pub data: Vec<u8>,
    pub tex_size: [i32; 3],
    pub data_type: i8,
    pub components: i8,
}

/// `LightCache`.
#[derive(Clone)]
pub struct LightCache {
    pub flag: i32,
    pub version: i32,
    pub type_: i32,
    pub cube_len: i32,
    pub grid_len: i32,
    pub mips_len: i32,
    pub vis_res: i32,
    pub ref_res: i32,
    pub grid_tx: LightCacheTexture,
    pub cube_tx: LightCacheTexture,
    /// `LightCacheTexture *cube_mips` → Vec.
    pub cube_mips: Vec<LightCacheTexture>,
    /// `LightProbeCache *cube_data` → Vec.
    pub cube_data: Vec<LightProbeCache>,
    /// `LightGridCache *grid_data` → Vec.
    pub grid_data: Vec<LightGridCache>,
}
impl Default for LightCache {
    fn default() -> Self {
        Self {
            flag: 0,
            version: 0,
            type_: LIGHTCACHE_TYPE_STATIC,
            cube_len: 0,
            grid_len: 0,
            mips_len: 0,
            vis_res: 0,
            ref_res: 0,
            grid_tx: LightCacheTexture::default(),
            cube_tx: LightCacheTexture::default(),
            cube_mips: Vec::new(),
            cube_data: Vec::new(),
            grid_data: Vec::new(),
        }
    }
}

/// `LightProbeBakingData` — flat f32 buffers ([4]-strided arrays flattened).
#[derive(Clone, Default)]
pub struct LightProbeBakingData {
    pub l0: Vec<f32>,
    pub l1_a: Vec<f32>,
    pub l1_b: Vec<f32>,
    pub l1_c: Vec<f32>,
    pub validity: Vec<f32>,
    pub virtual_offset: Vec<f32>,
}

/// `LightProbeIrradianceData` ([3]-strided arrays flattened).
#[derive(Clone, Default)]
pub struct LightProbeIrradianceData {
    pub l0: Vec<f32>,
    pub l1_a: Vec<f32>,
    pub l1_b: Vec<f32>,
    pub l1_c: Vec<f32>,
}

/// `LightProbeVisibilityData`.
#[derive(Clone, Default)]
pub struct LightProbeVisibilityData {
    pub l0: Vec<f32>,
    pub l1_a: Vec<f32>,
    pub l1_b: Vec<f32>,
    pub l1_c: Vec<f32>,
}

/// `LightProbeConnectivityData`.
#[derive(Clone, Default)]
pub struct LightProbeConnectivityData {
    pub validity: Vec<u8>,
}

/// `LightProbeBlockData`.
#[derive(Clone, Default)]
pub struct LightProbeBlockData {
    pub offset: [i32; 3],
    pub level: i32,
}

/// `LightProbeGridCacheFrame`.
#[derive(Clone)]
pub struct LightProbeGridCacheFrame {
    pub size: [i32; 3],
    pub data_layout: i32,
    pub block_len: i32,
    pub block_size: i32,
    /// `LightProbeBlockData *block_infos` → Vec.
    pub block_infos: Vec<LightProbeBlockData>,
    pub baking: LightProbeBakingData,
    pub irradiance: LightProbeIrradianceData,
    pub visibility: LightProbeVisibilityData,
    pub connectivity: LightProbeConnectivityData,
    pub surfels_len: i32,
    /// `void *surfels`.
    pub surfels: usize,
}
impl Default for LightProbeGridCacheFrame {
    fn default() -> Self {
        Self {
            size: [0; 3],
            data_layout: LIGHTPROBE_CACHE_UNIFORM_GRID,
            block_len: 0,
            block_size: 0,
            block_infos: Vec::new(),
            baking: LightProbeBakingData::default(),
            irradiance: LightProbeIrradianceData::default(),
            visibility: LightProbeVisibilityData::default(),
            connectivity: LightProbeConnectivityData::default(),
            surfels_len: 0,
            surfels: NIL,
        }
    }
}

/// `LightProbeObjectCache`.
#[derive(Clone)]
pub struct LightProbeObjectCache {
    pub cache_type: i32,
    pub shared: i8,
    pub dirty: i8,
    /// `LightProbeGridCacheFrame *grid_static_cache`.
    pub grid_static_cache: usize,
}
impl Default for LightProbeObjectCache {
    fn default() -> Self {
        Self {
            cache_type: LIGHTPROBE_CACHE_TYPE_NONE,
            shared: 0,
            dirty: 0,
            grid_static_cache: NIL,
        }
    }
}
