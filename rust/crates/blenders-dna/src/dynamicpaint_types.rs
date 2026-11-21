//! Port of `DNA_dynamicpaint_types.h` — dynamic paint canvas/brush settings.

use crate::id::NIL;
use blenders_bli::listbase::ListBase;

/// `eDynamicPaint_SurfaceType` (short).
pub type DynamicPaintSurfaceType = i16;
pub const MOD_DPAINT_SURFACE_T_PAINT: DynamicPaintSurfaceType = 0;
pub const MOD_DPAINT_SURFACE_T_DISPLACE: DynamicPaintSurfaceType = 1;
pub const MOD_DPAINT_SURFACE_T_WEIGHT: DynamicPaintSurfaceType = 2;
pub const MOD_DPAINT_SURFACE_T_WAVE: DynamicPaintSurfaceType = 3;

/// `eDynamicPaint_SurfaceFlags`.
pub type DynamicPaintSurfaceFlags = i32;
pub const MOD_DPAINT_ACTIVE: DynamicPaintSurfaceFlags = 1 << 0;
pub const MOD_DPAINT_ANTIALIAS: DynamicPaintSurfaceFlags = 1 << 1;
pub const MOD_DPAINT_DISSOLVE: DynamicPaintSurfaceFlags = 1 << 2;
pub const MOD_DPAINT_MULALPHA: DynamicPaintSurfaceFlags = 1 << 3;
pub const MOD_DPAINT_DISSOLVE_LOG: DynamicPaintSurfaceFlags = 1 << 4;
pub const MOD_DPAINT_DRY_LOG: DynamicPaintSurfaceFlags = 1 << 5;
pub const MOD_DPAINT_WAVE_OPEN_BORDERS: DynamicPaintSurfaceFlags = 1 << 7;
pub const MOD_DPAINT_DISP_INCREMENTAL: DynamicPaintSurfaceFlags = 1 << 8;
pub const MOD_DPAINT_USE_DRYING: DynamicPaintSurfaceFlags = 1 << 9;
pub const MOD_DPAINT_OUT1: DynamicPaintSurfaceFlags = 1 << 10;
pub const MOD_DPAINT_OUT2: DynamicPaintSurfaceFlags = 1 << 11;

/// `eDynamicPaint_ImageFormat` (short).
pub type DynamicPaintImageFormat = i16;
pub const MOD_DPAINT_IMGFORMAT_PNG: DynamicPaintImageFormat = 0;
pub const MOD_DPAINT_IMGFORMAT_OPENEXR: DynamicPaintImageFormat = 1;

/// `eDynamicPaint_DispType` (short).
pub type DynamicPaintDispType = i16;
pub const MOD_DPAINT_DISP_DISPLACE: DynamicPaintDispType = 0;
pub const MOD_DPAINT_DISP_DEPTH: DynamicPaintDispType = 1;

/// `eDynamicPaint_EffectFlags`.
pub type DynamicPaintEffectFlags = i32;
pub const MOD_DPAINT_EFFECT_DO_SPREAD: DynamicPaintEffectFlags = 1 << 0;
pub const MOD_DPAINT_EFFECT_DO_DRIP: DynamicPaintEffectFlags = 1 << 1;
pub const MOD_DPAINT_EFFECT_DO_SHRINK: DynamicPaintEffectFlags = 1 << 2;

/// `eDynamicPaint_InitColorType` (short).
pub type DynamicPaintInitColorType = i16;
pub const MOD_DPAINT_INITIAL_NONE: DynamicPaintInitColorType = 0;
pub const MOD_DPAINT_INITIAL_COLOR: DynamicPaintInitColorType = 1;
pub const MOD_DPAINT_INITIAL_TEXTURE: DynamicPaintInitColorType = 2;
pub const MOD_DPAINT_INITIAL_VERTEXCOLOR: DynamicPaintInitColorType = 3;

/// `eDynamicPaint_CanvasFlags` (short).
pub type DynamicPaintCanvasFlags = i16;
pub const MOD_DPAINT_BAKING: DynamicPaintCanvasFlags = 1 << 1;

/// `eDynamicPaint_BrushFlags`.
pub type DynamicPaintBrushFlags = i32;
pub const MOD_DPAINT_PART_RAD: DynamicPaintBrushFlags = 1 << 0;
pub const MOD_DPAINT_ABS_ALPHA: DynamicPaintBrushFlags = 1 << 2;
pub const MOD_DPAINT_ERASE: DynamicPaintBrushFlags = 1 << 3;
pub const MOD_DPAINT_RAMP_ALPHA: DynamicPaintBrushFlags = 1 << 4;
pub const MOD_DPAINT_PROX_PROJECT: DynamicPaintBrushFlags = 1 << 5;
pub const MOD_DPAINT_INVERSE_PROX: DynamicPaintBrushFlags = 1 << 6;
pub const MOD_DPAINT_NEGATE_VOLUME: DynamicPaintBrushFlags = 1 << 7;
pub const MOD_DPAINT_DO_SMUDGE: DynamicPaintBrushFlags = 1 << 8;
pub const MOD_DPAINT_VELOCITY_ALPHA: DynamicPaintBrushFlags = 1 << 9;
pub const MOD_DPAINT_VELOCITY_COLOR: DynamicPaintBrushFlags = 1 << 10;
pub const MOD_DPAINT_VELOCITY_DEPTH: DynamicPaintBrushFlags = 1 << 11;
pub const MOD_DPAINT_USES_VELOCITY: DynamicPaintBrushFlags = MOD_DPAINT_DO_SMUDGE
    | MOD_DPAINT_VELOCITY_ALPHA
    | MOD_DPAINT_VELOCITY_COLOR
    | MOD_DPAINT_VELOCITY_DEPTH;

/// `eDynamicPaint_CollisionType`.
pub type DynamicPaintCollisionType = i32;
pub const MOD_DPAINT_COL_VOLUME: DynamicPaintCollisionType = 0;
pub const MOD_DPAINT_COL_DIST: DynamicPaintCollisionType = 1;
pub const MOD_DPAINT_COL_VOLDIST: DynamicPaintCollisionType = 2;
pub const MOD_DPAINT_COL_PSYS: DynamicPaintCollisionType = 3;
pub const MOD_DPAINT_COL_POINT: DynamicPaintCollisionType = 4;

/// `eDynamicPaint_ProximityFalloff` (short).
pub type DynamicPaintProximityFalloff = i16;
pub const MOD_DPAINT_PRFALL_CONSTANT: DynamicPaintProximityFalloff = 0;
pub const MOD_DPAINT_PRFALL_SMOOTH: DynamicPaintProximityFalloff = 1;
pub const MOD_DPAINT_PRFALL_RAMP: DynamicPaintProximityFalloff = 2;

/// `eDynamicPaint_WaveBrushType` (short).
pub type DynamicPaintWaveBrushType = i16;
pub const MOD_DPAINT_WAVEB_DEPTH: DynamicPaintWaveBrushType = 0;
pub const MOD_DPAINT_WAVEB_FORCE: DynamicPaintWaveBrushType = 1;
pub const MOD_DPAINT_WAVEB_REFLECT: DynamicPaintWaveBrushType = 2;
pub const MOD_DPAINT_WAVEB_CHANGE: DynamicPaintWaveBrushType = 3;

/// `eDynamicPaint_RayDir` (short).
pub type DynamicPaintRayDir = i16;
pub const MOD_DPAINT_RAY_CANVAS: DynamicPaintRayDir = 0;
pub const MOD_DPAINT_RAY_BRUSH_AVG: DynamicPaintRayDir = 1;
pub const MOD_DPAINT_RAY_ZPLUS: DynamicPaintRayDir = 2;

/// `eDynamicPaint_SurfaceFormat` (short).
pub type DynamicPaintSurfaceFormat = i16;
pub const MOD_DPAINT_SURFACE_F_PTEX: DynamicPaintSurfaceFormat = 0;
pub const MOD_DPAINT_SURFACE_F_VERTEX: DynamicPaintSurfaceFormat = 1;
pub const MOD_DPAINT_SURFACE_F_IMAGESEQ: DynamicPaintSurfaceFormat = 2;

/// `DynamicPaintSurface`.
#[derive(Clone)]
pub struct DynamicPaintSurface {
    pub next: usize,
    pub prev: usize,
    /// `DynamicPaintCanvasSettings *canvas`.
    pub canvas: usize,
    /// `PaintSurfaceData *data`.
    pub data: usize,
    /// `Collection *brush_group`.
    pub brush_group: usize,
    /// `EffectorWeights *effector_weights`.
    pub effector_weights: usize,
    /// `PointCache *pointcache`.
    pub pointcache: usize,
    /// `ListBaseT<PointCache>`.
    pub ptcaches: ListBase,
    pub current_frame: i32,
    /// `char name[64]`.
    pub name: [u8; 64],
    pub format: DynamicPaintSurfaceFormat,
    pub type_: DynamicPaintSurfaceType,
    pub disp_type: DynamicPaintDispType,
    pub image_fileformat: DynamicPaintImageFormat,
    pub effect_ui: i16,
    pub init_color_type: DynamicPaintInitColorType,
    pub flags: DynamicPaintSurfaceFlags,
    pub effect: DynamicPaintEffectFlags,
    pub image_resolution: i32,
    pub substeps: i32,
    pub start_frame: i32,
    pub end_frame: i32,
    pub init_color: [f32; 4],
    /// `Tex *init_texture`.
    pub init_texture: usize,
    /// `char init_layername[68]`.
    pub init_layername: [u8; 68],
    pub dry_speed: i32,
    pub diss_speed: i32,
    pub color_dry_threshold: f32,
    pub depth_clamp: f32,
    pub disp_factor: f32,
    pub spread_speed: f32,
    pub color_spread_speed: f32,
    pub shrink_speed: f32,
    pub drip_vel: f32,
    pub drip_acc: f32,
    pub influence_scale: f32,
    pub radius_scale: f32,
    pub wave_damping: f32,
    pub wave_speed: f32,
    pub wave_timescale: f32,
    pub wave_spring: f32,
    pub wave_smoothness: f32,
    /// `char uvlayer_name[68]`, `image_output_path[1024]`, `output_name[68]`, `output_name2[68]`.
    pub uvlayer_name: [u8; 68],
    pub image_output_path: [u8; 1024],
    pub output_name: [u8; 68],
    pub output_name2: [u8; 68],
}
impl Default for DynamicPaintSurface {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            canvas: NIL,
            data: NIL,
            brush_group: NIL,
            effector_weights: NIL,
            pointcache: NIL,
            ptcaches: ListBase::default(),
            current_frame: 0,
            name: [0; 64],
            format: MOD_DPAINT_SURFACE_F_PTEX,
            type_: MOD_DPAINT_SURFACE_T_PAINT,
            disp_type: MOD_DPAINT_DISP_DISPLACE,
            image_fileformat: MOD_DPAINT_IMGFORMAT_PNG,
            effect_ui: 0,
            init_color_type: MOD_DPAINT_INITIAL_NONE,
            flags: 0,
            effect: 0,
            image_resolution: 0,
            substeps: 0,
            start_frame: 0,
            end_frame: 0,
            init_color: [0.0; 4],
            init_texture: NIL,
            init_layername: [0; 68],
            dry_speed: 0,
            diss_speed: 0,
            color_dry_threshold: 0.0,
            depth_clamp: 0.0,
            disp_factor: 0.0,
            spread_speed: 0.0,
            color_spread_speed: 0.0,
            shrink_speed: 0.0,
            drip_vel: 0.0,
            drip_acc: 0.0,
            influence_scale: 0.0,
            radius_scale: 0.0,
            wave_damping: 0.0,
            wave_speed: 0.0,
            wave_timescale: 0.0,
            wave_spring: 0.0,
            wave_smoothness: 0.0,
            uvlayer_name: [0; 68],
            image_output_path: [0; 1024],
            output_name: [0; 68],
            output_name2: [0; 68],
        }
    }
}

/// `DynamicPaintCanvasSettings`.
#[derive(Clone)]
pub struct DynamicPaintCanvasSettings {
    /// `DynamicPaintModifierData *pmd`.
    pub pmd: usize,
    /// `ListBaseT<DynamicPaintSurface>`.
    pub surfaces: ListBase,
    pub active_sur: i16,
    pub flags: DynamicPaintCanvasFlags,
    /// `char error[64]`.
    pub error: [u8; 64],
}
impl Default for DynamicPaintCanvasSettings {
    fn default() -> Self {
        Self {
            pmd: NIL,
            surfaces: ListBase::default(),
            active_sur: 0,
            flags: 0,
            error: [0; 64],
        }
    }
}

/// `DynamicPaintBrushSettings`.
#[derive(Clone)]
pub struct DynamicPaintBrushSettings {
    /// `DynamicPaintModifierData *pmd`.
    pub pmd: usize,
    /// `ParticleSystem *psys`.
    pub psys: usize,
    pub flags: DynamicPaintBrushFlags,
    pub collision: DynamicPaintCollisionType,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub alpha: f32,
    pub wetness: f32,
    pub particle_radius: f32,
    pub particle_smooth: f32,
    pub paint_distance: f32,
    /// `ColorBand *paint_ramp`, `*vel_ramp`.
    pub paint_ramp: usize,
    pub vel_ramp: usize,
    pub proximity_falloff: DynamicPaintProximityFalloff,
    pub wave_type: DynamicPaintWaveBrushType,
    pub ray_dir: DynamicPaintRayDir,
    pub wave_factor: f32,
    pub wave_clamp: f32,
    pub max_velocity: f32,
    pub smudge_strength: f32,
}
impl Default for DynamicPaintBrushSettings {
    fn default() -> Self {
        Self {
            pmd: NIL,
            psys: NIL,
            flags: 0,
            collision: MOD_DPAINT_COL_VOLUME,
            r: 0.0,
            g: 0.0,
            b: 0.0,
            alpha: 0.0,
            wetness: 0.0,
            particle_radius: 0.0,
            particle_smooth: 0.0,
            paint_distance: 0.0,
            paint_ramp: NIL,
            vel_ramp: NIL,
            proximity_falloff: MOD_DPAINT_PRFALL_CONSTANT,
            wave_type: MOD_DPAINT_WAVEB_DEPTH,
            ray_dir: MOD_DPAINT_RAY_CANVAS,
            wave_factor: 0.0,
            wave_clamp: 0.0,
            max_velocity: 0.0,
            smudge_strength: 0.0,
        }
    }
}
