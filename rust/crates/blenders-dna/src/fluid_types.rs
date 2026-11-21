//! Port of `DNA_fluid_types.h` — Mantaflow fluid domain/flow/effector settings.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::id::NIL;
use blenders_bli::listbase::ListBase;
/// `eFluidDomain_Flags`.
pub type FluiddomainFlags = i32;
pub const FLUID_DOMAIN_USE_NOISE: FluiddomainFlags = 1 << 1;
pub const FLUID_DOMAIN_USE_DISSOLVE: FluiddomainFlags = 1 << 2;
pub const FLUID_DOMAIN_USE_DISSOLVE_LOG: FluiddomainFlags = 1 << 3;
pub const FLUID_DOMAIN_USE_HIGH_SMOOTH: FluiddomainFlags = 1 << 5;
pub const FLUID_DOMAIN_FILE_LOAD: FluiddomainFlags = 1 << 6;
pub const FLUID_DOMAIN_USE_ADAPTIVE_DOMAIN: FluiddomainFlags = 1 << 7;
pub const FLUID_DOMAIN_USE_ADAPTIVE_TIME: FluiddomainFlags = 1 << 8;
pub const FLUID_DOMAIN_USE_MESH: FluiddomainFlags = 1 << 9;
pub const FLUID_DOMAIN_USE_GUIDE: FluiddomainFlags = 1 << 10;
pub const FLUID_DOMAIN_USE_SPEED_VECTORS: FluiddomainFlags = 1 << 11;
pub const FLUID_DOMAIN_EXPORT_MANTA_SCRIPT: FluiddomainFlags = 1 << 12;
pub const FLUID_DOMAIN_USE_FRACTIONS: FluiddomainFlags = 1 << 13;
pub const FLUID_DOMAIN_DELETE_IN_OBSTACLE: FluiddomainFlags = 1 << 14;
pub const FLUID_DOMAIN_USE_DIFFUSION: FluiddomainFlags = 1 << 15;
pub const FLUID_DOMAIN_USE_RESUMABLE_CACHE: FluiddomainFlags = 1 << 16;
pub const FLUID_DOMAIN_USE_VISCOSITY: FluiddomainFlags = 1 << 17;

/// `eFluidDomain_BorderFlags`.
pub type FluiddomainBorderflags = i32;
pub const FLUID_DOMAIN_BORDER_FRONT: FluiddomainBorderflags = 1 << 1;
pub const FLUID_DOMAIN_BORDER_BACK: FluiddomainBorderflags = 1 << 2;
pub const FLUID_DOMAIN_BORDER_RIGHT: FluiddomainBorderflags = 1 << 3;
pub const FLUID_DOMAIN_BORDER_LEFT: FluiddomainBorderflags = 1 << 4;
pub const FLUID_DOMAIN_BORDER_TOP: FluiddomainBorderflags = 1 << 5;
pub const FLUID_DOMAIN_BORDER_BOTTOM: FluiddomainBorderflags = 1 << 6;

/// `eFluidDomain_FileFormat`.
pub type FluiddomainFileformat = i8;
pub const FLUID_DOMAIN_FILE_UNI: FluiddomainFileformat = 1 << 0;
pub const FLUID_DOMAIN_FILE_OPENVDB: FluiddomainFileformat = 1 << 1;
pub const FLUID_DOMAIN_FILE_RAW: FluiddomainFileformat = 1 << 2;
pub const FLUID_DOMAIN_FILE_OBJECT: FluiddomainFileformat = 1 << 3;
pub const FLUID_DOMAIN_FILE_BIN_OBJECT: FluiddomainFileformat = 1 << 4;

/// `eFluidDomain_AxisSliceMethod`.
pub type FluiddomainAxisslicemethod = i8;
pub const AXIS_SLICE_FULL: FluiddomainAxisslicemethod = 0;
pub const AXIS_SLICE_SINGLE: FluiddomainAxisslicemethod = 1;

/// `eFluidDomain_SliceAxis`.
pub type FluiddomainSliceaxis = i8;
pub const SLICE_AXIS_AUTO: FluiddomainSliceaxis = 0;
pub const SLICE_AXIS_X: FluiddomainSliceaxis = 1;
pub const SLICE_AXIS_Y: FluiddomainSliceaxis = 2;
pub const SLICE_AXIS_Z: FluiddomainSliceaxis = 3;

/// `FLUID_DisplayInterpolationMethod`.
pub type FluidDisplayinterpolationmethod = i8;
pub const FLUID_DISPLAY_INTERP_LINEAR: FluidDisplayinterpolationmethod = 0;
pub const FLUID_DISPLAY_INTERP_CUBIC: FluidDisplayinterpolationmethod = 1;
pub const FLUID_DISPLAY_INTERP_CLOSEST: FluidDisplayinterpolationmethod = 2;

/// `eFluidDomain_VectorDrawType`.
pub type FluiddomainVectordrawtype = i8;
pub const VECTOR_DRAW_NEEDLE: FluiddomainVectordrawtype = 0;
pub const VECTOR_DRAW_STREAMLINE: FluiddomainVectordrawtype = 1;
pub const VECTOR_DRAW_MAC: FluiddomainVectordrawtype = 2;

/// `eFluidDomain_VectorDrawMAC`.
pub type FluiddomainVectordrawmac = i8;
pub const VECTOR_DRAW_MAC_X: FluiddomainVectordrawmac = 1 << 0;
pub const VECTOR_DRAW_MAC_Y: FluiddomainVectordrawmac = 1 << 1;
pub const VECTOR_DRAW_MAC_Z: FluiddomainVectordrawmac = 1 << 2;

/// `FLUID_DisplayVectorField`.
pub type FluidDisplayvectorfield = i8;
pub const FLUID_DOMAIN_VECTOR_FIELD_VELOCITY: FluidDisplayvectorfield = 0;
pub const FLUID_DOMAIN_VECTOR_FIELD_GUIDE_VELOCITY: FluidDisplayvectorfield = 1;
pub const FLUID_DOMAIN_VECTOR_FIELD_FORCE: FluidDisplayvectorfield = 2;

/// `eFluidDomain_SndParticleBoundary`.
pub type FluiddomainSndparticleboundary = i8;
pub const SNDPARTICLE_BOUNDARY_DELETE: FluiddomainSndparticleboundary = 0;
pub const SNDPARTICLE_BOUNDARY_PUSHOUT: FluiddomainSndparticleboundary = 1;

/// `eFluidDomain_SndParticleCombinedExport`.
pub type FluiddomainSndparticlecombinedexport = i8;
pub const SNDPARTICLE_COMBINED_EXPORT_OFF: FluiddomainSndparticlecombinedexport = 0;
pub const SNDPARTICLE_COMBINED_EXPORT_SPRAY_FOAM: FluiddomainSndparticlecombinedexport = 1;
pub const SNDPARTICLE_COMBINED_EXPORT_SPRAY_BUBBLE: FluiddomainSndparticlecombinedexport = 2;
pub const SNDPARTICLE_COMBINED_EXPORT_FOAM_BUBBLE: FluiddomainSndparticlecombinedexport = 3;
pub const SNDPARTICLE_COMBINED_EXPORT_SPRAY_FOAM_BUBBLE: FluiddomainSndparticlecombinedexport = 4;

/// `eFluidDomain_CobaField`.
pub type FluiddomainCobafield = i8;
pub const FLUID_DOMAIN_FIELD_DENSITY: FluiddomainCobafield = 0;
pub const FLUID_DOMAIN_FIELD_HEAT: FluiddomainCobafield = 1;
pub const FLUID_DOMAIN_FIELD_FUEL: FluiddomainCobafield = 2;
pub const FLUID_DOMAIN_FIELD_REACT: FluiddomainCobafield = 3;
pub const FLUID_DOMAIN_FIELD_FLAME: FluiddomainCobafield = 4;
pub const FLUID_DOMAIN_FIELD_VELOCITY_X: FluiddomainCobafield = 5;
pub const FLUID_DOMAIN_FIELD_VELOCITY_Y: FluiddomainCobafield = 6;
pub const FLUID_DOMAIN_FIELD_VELOCITY_Z: FluiddomainCobafield = 7;
pub const FLUID_DOMAIN_FIELD_COLOR_R: FluiddomainCobafield = 8;
pub const FLUID_DOMAIN_FIELD_COLOR_G: FluiddomainCobafield = 9;
pub const FLUID_DOMAIN_FIELD_COLOR_B: FluiddomainCobafield = 10;
pub const FLUID_DOMAIN_FIELD_FORCE_X: FluiddomainCobafield = 11;
pub const FLUID_DOMAIN_FIELD_FORCE_Y: FluiddomainCobafield = 12;
pub const FLUID_DOMAIN_FIELD_FORCE_Z: FluiddomainCobafield = 13;
pub const FLUID_DOMAIN_FIELD_PHI: FluiddomainCobafield = 14;
pub const FLUID_DOMAIN_FIELD_PHI_IN: FluiddomainCobafield = 15;
pub const FLUID_DOMAIN_FIELD_PHI_OUT: FluiddomainCobafield = 16;
pub const FLUID_DOMAIN_FIELD_PHI_OBSTACLE: FluiddomainCobafield = 17;
pub const FLUID_DOMAIN_FIELD_FLAGS: FluiddomainCobafield = 18;
pub const FLUID_DOMAIN_FIELD_PRESSURE: FluiddomainCobafield = 19;

/// `eFluidDomain_GridlineColorField`.
pub type FluiddomainGridlinecolorfield = i8;
pub const FLUID_GRIDLINE_COLOR_TYPE_NONE: FluiddomainGridlinecolorfield = 0;
pub const FLUID_GRIDLINE_COLOR_TYPE_FLAGS: FluiddomainGridlinecolorfield = 1;
pub const FLUID_GRIDLINE_COLOR_TYPE_RANGE: FluiddomainGridlinecolorfield = 2;

/// `eFluidDomain_GridlineCellFilter`.
pub type FluiddomainGridlinecellfilter = i8;
pub const FLUID_CELL_TYPE_NONE: FluiddomainGridlinecellfilter = 0;
pub const FLUID_CELL_TYPE_FLUID: FluiddomainGridlinecellfilter = 1 << 0;
pub const FLUID_CELL_TYPE_OBSTACLE: FluiddomainGridlinecellfilter = 1 << 1;
pub const FLUID_CELL_TYPE_EMPTY: FluiddomainGridlinecellfilter = 1 << 2;
pub const FLUID_CELL_TYPE_INFLOW: FluiddomainGridlinecellfilter = 1 << 3;
pub const FLUID_CELL_TYPE_OUTFLOW: FluiddomainGridlinecellfilter = 1 << 4;

/// `eFluidDomain_Type`.
pub type FluiddomainType = i16;
pub const FLUID_DOMAIN_TYPE_GAS: FluiddomainType = 0;
pub const FLUID_DOMAIN_TYPE_LIQUID: FluiddomainType = 1;

/// `eFluidDomain_MeshGenerator`.
pub type FluiddomainMeshgenerator = i16;
pub const FLUID_DOMAIN_MESH_IMPROVED: FluiddomainMeshgenerator = 0;
pub const FLUID_DOMAIN_MESH_UNION: FluiddomainMeshgenerator = 1;

/// `eFluidDomain_GuideSource`.
pub type FluiddomainGuidesource = i16;
pub const FLUID_DOMAIN_GUIDE_SRC_DOMAIN: FluiddomainGuidesource = 0;
pub const FLUID_DOMAIN_GUIDE_SRC_EFFECTOR: FluiddomainGuidesource = 1;

/// `eFluidDomain_ActiveFields`.
pub type FluiddomainActivefields = i32;
pub const FLUID_DOMAIN_ACTIVE_HEAT: FluiddomainActivefields = 1 << 0;
pub const FLUID_DOMAIN_ACTIVE_FIRE: FluiddomainActivefields = 1 << 1;
pub const FLUID_DOMAIN_ACTIVE_COLORS: FluiddomainActivefields = 1 << 2;
pub const FLUID_DOMAIN_ACTIVE_COLOR_SET: FluiddomainActivefields = 1 << 3;
pub const FLUID_DOMAIN_ACTIVE_OBSTACLE: FluiddomainActivefields = 1 << 4;
pub const FLUID_DOMAIN_ACTIVE_GUIDE: FluiddomainActivefields = 1 << 5;
pub const FLUID_DOMAIN_ACTIVE_INVEL: FluiddomainActivefields = 1 << 6;
pub const FLUID_DOMAIN_ACTIVE_OUTFLOW: FluiddomainActivefields = 1 << 7;

/// `eFluidDomain_ParticleTypes`.
pub type FluiddomainParticletypes = i32;
pub const FLUID_DOMAIN_PARTICLE_FLIP: FluiddomainParticletypes = 1 << 0;
pub const FLUID_DOMAIN_PARTICLE_SPRAY: FluiddomainParticletypes = 1 << 1;
pub const FLUID_DOMAIN_PARTICLE_BUBBLE: FluiddomainParticletypes = 1 << 2;
pub const FLUID_DOMAIN_PARTICLE_FOAM: FluiddomainParticletypes = 1 << 3;
pub const FLUID_DOMAIN_PARTICLE_TRACER: FluiddomainParticletypes = 1 << 4;

/// `eFluidDomain_SimMethod`.
pub type FluiddomainSimmethod = i16;
pub const FLUID_DOMAIN_METHOD_FLIP: FluiddomainSimmethod = 1 << 0;
pub const FLUID_DOMAIN_METHOD_APIC: FluiddomainSimmethod = 1 << 1;

/// `eFluidDomain_CacheFlag`.
pub type FluiddomainCacheflag = i32;
pub const FLUID_DOMAIN_BAKING_DATA: FluiddomainCacheflag = 1 << 0;
pub const FLUID_DOMAIN_BAKED_DATA: FluiddomainCacheflag = 1 << 1;
pub const FLUID_DOMAIN_BAKING_NOISE: FluiddomainCacheflag = 1 << 2;
pub const FLUID_DOMAIN_BAKED_NOISE: FluiddomainCacheflag = 1 << 3;
pub const FLUID_DOMAIN_BAKING_MESH: FluiddomainCacheflag = 1 << 4;
pub const FLUID_DOMAIN_BAKED_MESH: FluiddomainCacheflag = 1 << 5;
pub const FLUID_DOMAIN_BAKING_PARTICLES: FluiddomainCacheflag = 1 << 6;
pub const FLUID_DOMAIN_BAKED_PARTICLES: FluiddomainCacheflag = 1 << 7;
pub const FLUID_DOMAIN_BAKING_GUIDE: FluiddomainCacheflag = 1 << 8;
pub const FLUID_DOMAIN_BAKED_GUIDE: FluiddomainCacheflag = 1 << 9;
pub const FLUID_DOMAIN_OUTDATED_DATA: FluiddomainCacheflag = 1 << 10;
pub const FLUID_DOMAIN_OUTDATED_NOISE: FluiddomainCacheflag = 1 << 11;
pub const FLUID_DOMAIN_OUTDATED_MESH: FluiddomainCacheflag = 1 << 12;
pub const FLUID_DOMAIN_OUTDATED_PARTICLES: FluiddomainCacheflag = 1 << 13;
pub const FLUID_DOMAIN_OUTDATED_GUIDE: FluiddomainCacheflag = 1 << 14;

/// `eFluidDomain_GridType`.
pub type FluiddomainGridtype = i32;
pub const FLUID_DOMAIN_GRID_FLOAT: FluiddomainGridtype = 0;
pub const FLUID_DOMAIN_GRID_INT: FluiddomainGridtype = 1;
pub const FLUID_DOMAIN_GRID_VEC3F: FluiddomainGridtype = 2;

/// `eFluidDomain_CacheFilesMode`.
pub type FluiddomainCachefilesmode = i32;
pub const FLUID_DOMAIN_CACHE_FILES_SINGLE: FluiddomainCachefilesmode = 0;
pub const FLUID_DOMAIN_CACHE_FILES_COMBINED: FluiddomainCachefilesmode = 1;

/// `eFluidDomain_CacheType`.
pub type FluiddomainCachetype = i16;
pub const FLUID_DOMAIN_CACHE_REPLAY: FluiddomainCachetype = 0;
pub const FLUID_DOMAIN_CACHE_MODULAR: FluiddomainCachetype = 1;
pub const FLUID_DOMAIN_CACHE_ALL: FluiddomainCachetype = 2;

/// `eFluidDomain_OpenVDBCompression`.
pub type FluiddomainOpenvdbcompression = i32;
pub const VDB_COMPRESSION_BLOSC: FluiddomainOpenvdbcompression = 0;
pub const VDB_COMPRESSION_ZIP: FluiddomainOpenvdbcompression = 1;
pub const VDB_COMPRESSION_NONE: FluiddomainOpenvdbcompression = 2;

/// `eFluidDomain_OpenVDBDepth`.
pub type FluiddomainOpenvdbdepth = i8;
pub const VDB_PRECISION_HALF_FLOAT: FluiddomainOpenvdbdepth = 0;
pub const VDB_PRECISION_FULL_FLOAT: FluiddomainOpenvdbdepth = 1;
pub const VDB_PRECISION_MINI_FLOAT: FluiddomainOpenvdbdepth = 2;

/// `eFluidDomain_CacheComp`.
pub type FluiddomainCachecomp = i32;
pub const SM_CACHE_LIGHT: FluiddomainCachecomp = 0;
pub const SM_CACHE_HEAVY: FluiddomainCachecomp = 1;

/// `eFluidDomain_HighresSampling`.
pub type FluiddomainHighressampling = i32;
pub const SM_HRES_NEAREST: FluiddomainHighressampling = 0;
pub const SM_HRES_LINEAR: FluiddomainHighressampling = 1;
pub const SM_HRES_FULLSAMPLE: FluiddomainHighressampling = 2;

/// `eFluidFlow_Type`.
pub type FluidflowType = i16;
pub const FLUID_FLOW_TYPE_SMOKE: FluidflowType = 1;
pub const FLUID_FLOW_TYPE_FIRE: FluidflowType = 2;
pub const FLUID_FLOW_TYPE_SMOKEFIRE: FluidflowType = 3;
pub const FLUID_FLOW_TYPE_LIQUID: FluidflowType = 4;

/// `eFluidFlow_Behavior`.
pub type FluidflowBehavior = i16;
pub const FLUID_FLOW_BEHAVIOR_INFLOW: FluidflowBehavior = 0;
pub const FLUID_FLOW_BEHAVIOR_OUTFLOW: FluidflowBehavior = 1;
pub const FLUID_FLOW_BEHAVIOR_GEOMETRY: FluidflowBehavior = 2;

/// `eFluidFlow_Source`.
pub type FluidflowSource = i16;
pub const FLUID_FLOW_SOURCE_PARTICLES: FluidflowSource = 0;
pub const FLUID_FLOW_SOURCE_MESH: FluidflowSource = 1;

/// `eFluidFlow_TextureType`.
pub type FluidflowTexturetype = i16;
pub const FLUID_FLOW_TEXTURE_MAP_AUTO: FluidflowTexturetype = 0;
pub const FLUID_FLOW_TEXTURE_MAP_UV: FluidflowTexturetype = 1;

/// `eFluidFlow_Flags`.
pub type FluidflowFlags = i32;
pub const FLUID_FLOW_ABSOLUTE: FluidflowFlags = 1 << 1;
pub const FLUID_FLOW_INITVELOCITY: FluidflowFlags = 1 << 2;
pub const FLUID_FLOW_TEXTUREEMIT: FluidflowFlags = 1 << 3;
pub const FLUID_FLOW_USE_PART_SIZE: FluidflowFlags = 1 << 4;
pub const FLUID_FLOW_USE_INFLOW: FluidflowFlags = 1 << 5;
pub const FLUID_FLOW_USE_PLANE_INIT: FluidflowFlags = 1 << 6;
pub const FLUID_FLOW_NEEDS_UPDATE: FluidflowFlags = 1 << 7;

/// `eFluidEffector_Type`.
pub type FluideffectorType = i16;
pub const FLUID_EFFECTOR_TYPE_COLLISION: FluideffectorType = 0;
pub const FLUID_EFFECTOR_TYPE_GUIDE: FluideffectorType = 1;

/// `eFluidEffector_GuideMode`.
pub type FluideffectorGuidemode = i16;
pub const FLUID_EFFECTOR_GUIDE_MAX: FluideffectorGuidemode = 0;
pub const FLUID_EFFECTOR_GUIDE_MIN: FluideffectorGuidemode = 1;
pub const FLUID_EFFECTOR_GUIDE_OVERRIDE: FluideffectorGuidemode = 2;
pub const FLUID_EFFECTOR_GUIDE_AVERAGED: FluideffectorGuidemode = 3;

/// `eFluidEffector_Flags`.
pub type FluideffectorFlags = i32;
pub const FLUID_EFFECTOR_USE_EFFEC: FluideffectorFlags = 1 << 1;
pub const FLUID_EFFECTOR_USE_PLANE_INIT: FluideffectorFlags = 1 << 2;
pub const FLUID_EFFECTOR_NEEDS_UPDATE: FluideffectorFlags = 1 << 3;

/// `FluidDomainSettings`.
#[derive(Clone)]
pub struct FluidDomainSettings {
    /// `FluidModifierData *fmd`
    pub fmd: usize,
    /// `MANTA *fluid`
    pub fluid: usize,
    /// `MANTA *fluid_old`
    pub fluid_old: usize,
    /// `void *fluid_mutex`
    pub fluid_mutex: usize,
    /// `Collection *fluid_group`
    pub fluid_group: usize,
    /// `Collection *force_group`
    pub force_group: usize,
    /// `Collection *effector_group`
    pub effector_group: usize,
    /// `gpu::Texture *tex_density`
    pub tex_density: usize,
    /// `gpu::Texture *tex_color`
    pub tex_color: usize,
    /// `gpu::Texture *tex_wt`
    pub tex_wt: usize,
    /// `gpu::Texture *tex_shadow`
    pub tex_shadow: usize,
    /// `gpu::Texture *tex_flame`
    pub tex_flame: usize,
    /// `gpu::Texture *tex_flame_coba`
    pub tex_flame_coba: usize,
    /// `gpu::Texture *tex_coba`
    pub tex_coba: usize,
    /// `gpu::Texture *tex_field`
    pub tex_field: usize,
    /// `gpu::Texture *tex_velocity_x`
    pub tex_velocity_x: usize,
    /// `gpu::Texture *tex_velocity_y`
    pub tex_velocity_y: usize,
    /// `gpu::Texture *tex_velocity_z`
    pub tex_velocity_z: usize,
    /// `gpu::Texture *tex_flags`
    pub tex_flags: usize,
    /// `gpu::Texture *tex_range_field`
    pub tex_range_field: usize,
    /// `Object *guide_parent`
    pub guide_parent: usize,
    /// `EffectorWeights *effector_weights`
    pub effector_weights: usize,
    /// `float p0[3]`
    pub p0: [f32; 3],
    /// `float p1[3]`
    pub p1: [f32; 3],
    /// `float dp0[3]`
    pub dp0: [f32; 3],
    /// `float cell_size[3]`
    pub cell_size: [f32; 3],
    /// `float global_size[3]`
    pub global_size: [f32; 3],
    /// `float prev_loc[3]`
    pub prev_loc: [f32; 3],
    /// `int shift[3]`
    pub shift: [i32; 3],
    /// `float shift_f[3]`
    pub shift_f: [f32; 3],
    /// `float obj_shift_f[3]`
    pub obj_shift_f: [f32; 3],
    /// `float imat[4][4]`
    pub imat: [[f32; 4]; 4],
    /// `float obmat[4][4]`
    pub obmat: [[f32; 4]; 4],
    /// `float fluidmat[4][4]`
    pub fluidmat: [[f32; 4]; 4],
    /// `float fluidmat_wt[4][4]`
    pub fluidmat_wt: [[f32; 4]; 4],
    /// `int base_res[3]`
    pub base_res: [i32; 3],
    /// `int res_min[3]`
    pub res_min: [i32; 3],
    /// `int res_max[3]`
    pub res_max: [i32; 3],
    /// `int res[3]`
    pub res: [i32; 3],
    /// `int total_cells`
    pub total_cells: i32,
    /// `float dx`
    pub dx: f32,
    /// `float scale`
    pub scale: f32,
    /// `int boundary_width`
    pub boundary_width: i32,
    /// `int adapt_margin`
    pub adapt_margin: i32,
    /// `int adapt_res`
    pub adapt_res: i32,
    /// `float adapt_threshold`
    pub adapt_threshold: f32,
    /// `int maxres`
    pub maxres: i32,
    /// `int solver_res`
    pub solver_res: i32,
    /// `eFluidDomain_BorderFlags border_collisions`
    pub border_collisions: FluiddomainBorderflags,
    /// `float gravity[3]`
    pub gravity: [f32; 3],
    /// `eFluidDomain_ActiveFields active_fields`
    pub active_fields: FluiddomainActivefields,
    /// `eFluidDomain_Type type`
    pub type_: FluiddomainType,
    /// `float alpha`
    pub alpha: f32,
    /// `float beta`
    pub beta: f32,
    /// `int diss_speed`
    pub diss_speed: i32,
    /// `float vorticity`
    pub vorticity: f32,
    /// `float active_color[3]`
    pub active_color: [f32; 3],
    /// `eFluidDomain_HighresSampling highres_sampling`
    pub highres_sampling: FluiddomainHighressampling,
    /// `float burning_rate`
    pub burning_rate: f32,
    /// `float flame_smoke`
    pub flame_smoke: f32,
    /// `float flame_vorticity`
    pub flame_vorticity: f32,
    /// `float flame_ignition`
    pub flame_ignition: f32,
    /// `float flame_max_temp`
    pub flame_max_temp: f32,
    /// `float flame_smoke_color[3]`
    pub flame_smoke_color: [f32; 3],
    /// `float noise_strength`
    pub noise_strength: f32,
    /// `float noise_pos_scale`
    pub noise_pos_scale: f32,
    /// `float noise_time_anim`
    pub noise_time_anim: f32,
    /// `int res_noise[3]`
    pub res_noise: [i32; 3],
    /// `int noise_scale`
    pub noise_scale: i32,
    /// `float particle_randomness`
    pub particle_randomness: f32,
    /// `int particle_number`
    pub particle_number: i32,
    /// `int particle_minimum`
    pub particle_minimum: i32,
    /// `int particle_maximum`
    pub particle_maximum: i32,
    /// `float particle_radius`
    pub particle_radius: f32,
    /// `float particle_band_width`
    pub particle_band_width: f32,
    /// `float fractions_threshold`
    pub fractions_threshold: f32,
    /// `float fractions_distance`
    pub fractions_distance: f32,
    /// `float flip_ratio`
    pub flip_ratio: f32,
    /// `int sys_particle_maximum`
    pub sys_particle_maximum: i32,
    /// `eFluidDomain_SimMethod simulation_method`
    pub simulation_method: FluiddomainSimmethod,
    /// `float viscosity_value`
    pub viscosity_value: f32,
    /// `float surface_tension`
    pub surface_tension: f32,
    /// `float viscosity_base`
    pub viscosity_base: f32,
    /// `int viscosity_exponent`
    pub viscosity_exponent: i32,
    /// `float mesh_concave_upper`
    pub mesh_concave_upper: f32,
    /// `float mesh_concave_lower`
    pub mesh_concave_lower: f32,
    /// `float mesh_particle_radius`
    pub mesh_particle_radius: f32,
    /// `int mesh_smoothen_pos`
    pub mesh_smoothen_pos: i32,
    /// `int mesh_smoothen_neg`
    pub mesh_smoothen_neg: i32,
    /// `int mesh_scale`
    pub mesh_scale: i32,
    /// `eFluidDomain_MeshGenerator mesh_generator`
    pub mesh_generator: FluiddomainMeshgenerator,
    /// `eFluidDomain_ParticleTypes particle_type`
    pub particle_type: FluiddomainParticletypes,
    /// `int particle_scale`
    pub particle_scale: i32,
    /// `float sndparticle_tau_min_wc`
    pub sndparticle_tau_min_wc: f32,
    /// `float sndparticle_tau_max_wc`
    pub sndparticle_tau_max_wc: f32,
    /// `float sndparticle_tau_min_ta`
    pub sndparticle_tau_min_ta: f32,
    /// `float sndparticle_tau_max_ta`
    pub sndparticle_tau_max_ta: f32,
    /// `float sndparticle_tau_min_k`
    pub sndparticle_tau_min_k: f32,
    /// `float sndparticle_tau_max_k`
    pub sndparticle_tau_max_k: f32,
    /// `int sndparticle_k_wc`
    pub sndparticle_k_wc: i32,
    /// `int sndparticle_k_ta`
    pub sndparticle_k_ta: i32,
    /// `float sndparticle_k_b`
    pub sndparticle_k_b: f32,
    /// `float sndparticle_k_d`
    pub sndparticle_k_d: f32,
    /// `float sndparticle_l_min`
    pub sndparticle_l_min: f32,
    /// `float sndparticle_l_max`
    pub sndparticle_l_max: f32,
    /// `int sndparticle_potential_radius`
    pub sndparticle_potential_radius: i32,
    /// `int sndparticle_update_radius`
    pub sndparticle_update_radius: i32,
    /// `eFluidDomain_SndParticleBoundary sndparticle_boundary`
    pub sndparticle_boundary: FluiddomainSndparticleboundary,
    /// `float guide_alpha`
    pub guide_alpha: f32,
    /// `int guide_beta`
    pub guide_beta: i32,
    /// `float guide_vel_factor`
    pub guide_vel_factor: f32,
    /// `int guide_res[3]`
    pub guide_res: [i32; 3],
    /// `eFluidDomain_GuideSource guide_source`
    pub guide_source: FluiddomainGuidesource,
    /// `int cache_frame_start`
    pub cache_frame_start: i32,
    /// `int cache_frame_end`
    pub cache_frame_end: i32,
    /// `int cache_frame_pause_data`
    pub cache_frame_pause_data: i32,
    /// `int cache_frame_pause_noise`
    pub cache_frame_pause_noise: i32,
    /// `int cache_frame_pause_mesh`
    pub cache_frame_pause_mesh: i32,
    /// `int cache_frame_pause_particles`
    pub cache_frame_pause_particles: i32,
    /// `int cache_frame_pause_guide`
    pub cache_frame_pause_guide: i32,
    /// `int cache_frame_offset`
    pub cache_frame_offset: i32,
    /// `eFluidDomain_CacheFlag cache_flag`
    pub cache_flag: FluiddomainCacheflag,
    /// `eFluidDomain_FileFormat cache_mesh_format`
    pub cache_mesh_format: FluiddomainFileformat,
    /// `eFluidDomain_FileFormat cache_data_format`
    pub cache_data_format: FluiddomainFileformat,
    /// `eFluidDomain_FileFormat cache_particle_format`
    pub cache_particle_format: FluiddomainFileformat,
    /// `eFluidDomain_FileFormat cache_noise_format`
    pub cache_noise_format: FluiddomainFileformat,
    /// `char cache_directory[1024]`
    pub cache_directory: [u8; 1024],
    /// `char error[64]`
    pub error: [u8; 64],
    /// `eFluidDomain_CacheType cache_type`
    pub cache_type: FluiddomainCachetype,
    /// `char cache_id[4]`
    pub cache_id: [u8; 4],
    /// `float dt`
    pub dt: f32,
    /// `float time_total`
    pub time_total: f32,
    /// `float time_per_frame`
    pub time_per_frame: f32,
    /// `float frame_length`
    pub frame_length: f32,
    /// `float time_scale`
    pub time_scale: f32,
    /// `float cfl_condition`
    pub cfl_condition: f32,
    /// `int timesteps_minimum`
    pub timesteps_minimum: i32,
    /// `int timesteps_maximum`
    pub timesteps_maximum: i32,
    /// `float slice_per_voxel`
    pub slice_per_voxel: f32,
    /// `float slice_depth`
    pub slice_depth: f32,
    /// `float display_thickness`
    pub display_thickness: f32,
    /// `float grid_scale`
    pub grid_scale: f32,
    /// `ColorBand *coba`
    pub coba: usize,
    /// `float vector_scale`
    pub vector_scale: f32,
    /// `float gridlines_lower_bound`
    pub gridlines_lower_bound: f32,
    /// `float gridlines_upper_bound`
    pub gridlines_upper_bound: f32,
    /// `float gridlines_range_color[4]`
    pub gridlines_range_color: [f32; 4],
    /// `eFluidDomain_AxisSliceMethod axis_slice_method`
    pub axis_slice_method: FluiddomainAxisslicemethod,
    /// `eFluidDomain_SliceAxis slice_axis`
    pub slice_axis: FluiddomainSliceaxis,
    /// `char show_gridlines`
    pub show_gridlines: i8,
    /// `char draw_velocity`
    pub draw_velocity: i8,
    /// `eFluidDomain_VectorDrawType vector_draw_type`
    pub vector_draw_type: FluiddomainVectordrawtype,
    /// `char vector_scale_with_magnitude`
    pub vector_scale_with_magnitude: i8,
    /// `char use_coba`
    pub use_coba: i8,
    /// `FLUID_DisplayInterpolationMethod interp_method`
    pub interp_method: FluidDisplayinterpolationmethod,
    /// `eFluidDomain_GridlineCellFilter gridlines_cell_filter`
    pub gridlines_cell_filter: FluiddomainGridlinecellfilter,
    /// `float velocity_scale`
    pub velocity_scale: f32,
    /// `eFluidDomain_OpenVDBCompression openvdb_compression`
    pub openvdb_compression: FluiddomainOpenvdbcompression,
    /// `float clipping`
    pub clipping: f32,
    /// `eFluidDomain_OpenVDBDepth openvdb_data_depth`
    pub openvdb_data_depth: FluiddomainOpenvdbdepth,
    /// `int viewsettings`
    pub viewsettings: i32,
    /// `PointCache *point_cache[]`
    pub point_cache: [usize; 2],
    /// `eFluidDomain_CacheComp cache_comp`
    pub cache_comp: FluiddomainCachecomp,
    /// `eFluidDomain_CacheComp cache_high_comp`
    pub cache_high_comp: FluiddomainCachecomp,
    /// `eFluidDomain_FileFormat cache_file_format`
    pub cache_file_format: FluiddomainFileformat,
}
impl Default for FluidDomainSettings {
    fn default() -> Self {
        Self {
            fmd: NIL,
            fluid: NIL,
            fluid_old: NIL,
            fluid_mutex: NIL,
            fluid_group: NIL,
            force_group: NIL,
            effector_group: NIL,
            tex_density: NIL,
            tex_color: NIL,
            tex_wt: NIL,
            tex_shadow: NIL,
            tex_flame: NIL,
            tex_flame_coba: NIL,
            tex_coba: NIL,
            tex_field: NIL,
            tex_velocity_x: NIL,
            tex_velocity_y: NIL,
            tex_velocity_z: NIL,
            tex_flags: NIL,
            tex_range_field: NIL,
            guide_parent: NIL,
            effector_weights: NIL,
            p0: [0.0; 3],
            p1: [0.0; 3],
            dp0: [0.0; 3],
            cell_size: [0.0; 3],
            global_size: [0.0; 3],
            prev_loc: [0.0; 3],
            shift: [0; 3],
            shift_f: [0.0; 3],
            obj_shift_f: [0.0; 3],
            imat: [[0.0; 4]; 4],
            obmat: [[0.0; 4]; 4],
            fluidmat: [[0.0; 4]; 4],
            fluidmat_wt: [[0.0; 4]; 4],
            base_res: [0; 3],
            res_min: [0; 3],
            res_max: [0; 3],
            res: [0; 3],
            total_cells: 0,
            dx: 0.0,
            scale: 0.0,
            boundary_width: 1,
            adapt_margin: 4,
            adapt_res: 0,
            adapt_threshold: 0.002,
            maxres: 32,
            solver_res: 3,
            border_collisions: Default::default(),
            gravity: [0.0; 3],
            active_fields: Default::default(),
            type_: FLUID_DOMAIN_TYPE_GAS,
            alpha: 1.0,
            beta: 1.0,
            diss_speed: 5,
            vorticity: 0.0,
            active_color: [0.0; 3],
            highres_sampling: SM_HRES_FULLSAMPLE,
            burning_rate: 0.75,
            flame_smoke: 1.0,
            flame_vorticity: 0.5,
            flame_ignition: 1.5,
            flame_max_temp: 3.0,
            flame_smoke_color: [0.0; 3],
            noise_strength: 1.0,
            noise_pos_scale: 2.0,
            noise_time_anim: 0.1,
            res_noise: [0; 3],
            noise_scale: 2,
            particle_randomness: 0.1,
            particle_number: 2,
            particle_minimum: 8,
            particle_maximum: 16,
            particle_radius: 1.0,
            particle_band_width: 3.0,
            fractions_threshold: 0.05,
            fractions_distance: 0.5,
            flip_ratio: 0.97,
            sys_particle_maximum: 0,
            simulation_method: FLUID_DOMAIN_METHOD_FLIP,
            viscosity_value: 0.05,
            surface_tension: 0.0,
            viscosity_base: 1.0,
            viscosity_exponent: 6,
            mesh_concave_upper: 3.5,
            mesh_concave_lower: 0.4,
            mesh_particle_radius: 2.0,
            mesh_smoothen_pos: 1,
            mesh_smoothen_neg: 1,
            mesh_scale: 2,
            mesh_generator: FLUID_DOMAIN_MESH_IMPROVED,
            particle_type: Default::default(),
            particle_scale: 1,
            sndparticle_tau_min_wc: 2.0,
            sndparticle_tau_max_wc: 8.0,
            sndparticle_tau_min_ta: 5.0,
            sndparticle_tau_max_ta: 20.0,
            sndparticle_tau_min_k: 1.0,
            sndparticle_tau_max_k: 5.0,
            sndparticle_k_wc: 200,
            sndparticle_k_ta: 40,
            sndparticle_k_b: 0.5,
            sndparticle_k_d: 0.6,
            sndparticle_l_min: 10.0,
            sndparticle_l_max: 25.0,
            sndparticle_potential_radius: 2,
            sndparticle_update_radius: 2,
            sndparticle_boundary: SNDPARTICLE_BOUNDARY_DELETE,
            guide_alpha: 2.0,
            guide_beta: 5,
            guide_vel_factor: 2.0,
            guide_res: [0; 3],
            guide_source: FLUID_DOMAIN_GUIDE_SRC_DOMAIN,
            cache_frame_start: 1,
            cache_frame_end: 250,
            cache_frame_pause_data: 0,
            cache_frame_pause_noise: 0,
            cache_frame_pause_mesh: 0,
            cache_frame_pause_particles: 0,
            cache_frame_pause_guide: 0,
            cache_frame_offset: 0,
            cache_flag: Default::default(),
            cache_mesh_format: FLUID_DOMAIN_FILE_BIN_OBJECT,
            cache_data_format: FLUID_DOMAIN_FILE_OPENVDB,
            cache_particle_format: FLUID_DOMAIN_FILE_OPENVDB,
            cache_noise_format: FLUID_DOMAIN_FILE_OPENVDB,
            cache_directory: [0; 1024],
            error: [0; 64],
            cache_type: FLUID_DOMAIN_CACHE_REPLAY,
            cache_id: [0; 4],
            dt: 0.0,
            time_total: 0.0,
            time_per_frame: 0.0,
            frame_length: 0.0,
            time_scale: 1.0,
            cfl_condition: 2.0,
            timesteps_minimum: 1,
            timesteps_maximum: 4,
            slice_per_voxel: 5.0,
            slice_depth: 0.5,
            display_thickness: 1.0,
            grid_scale: 1.0,
            coba: NIL,
            vector_scale: 1.0,
            gridlines_lower_bound: 0.0,
            gridlines_upper_bound: 1.0,
            gridlines_range_color: [0.0; 4],
            axis_slice_method: AXIS_SLICE_FULL,
            slice_axis: SLICE_AXIS_AUTO,
            show_gridlines: 0,
            draw_velocity: 0,
            vector_draw_type: VECTOR_DRAW_NEEDLE,
            vector_scale_with_magnitude: 1,
            use_coba: 0,
            interp_method: FLUID_DISPLAY_INTERP_LINEAR,
            gridlines_cell_filter: FLUID_CELL_TYPE_NONE,
            velocity_scale: 1.0,
            openvdb_compression: VDB_COMPRESSION_BLOSC,
            clipping: 1e-6,
            openvdb_data_depth: VDB_PRECISION_HALF_FLOAT,
            viewsettings: 0,
            point_cache: [NIL; 2],
            cache_comp: SM_CACHE_LIGHT,
            cache_high_comp: SM_CACHE_LIGHT,
            cache_file_format: Default::default(),
        }
    }
}

/// `FluidFlowSettings`.
#[derive(Clone)]
pub struct FluidFlowSettings {
    /// `FluidModifierData *fmd`
    pub fmd: usize,
    /// `Mesh *mesh`
    pub mesh: usize,
    /// `ParticleSystem *psys`
    pub psys: usize,
    /// `Tex *noise_texture`
    pub noise_texture: usize,
    /// `float *verts_old`
    pub verts_old: usize,
    /// `int numverts`
    pub numverts: i32,
    /// `float vel_multi`
    pub vel_multi: f32,
    /// `float vel_normal`
    pub vel_normal: f32,
    /// `float vel_random`
    pub vel_random: f32,
    /// `float vel_coord[3]`
    pub vel_coord: [f32; 3],
    /// `float density`
    pub density: f32,
    /// `float color[3]`
    pub color: [f32; 3],
    /// `float fuel_amount`
    pub fuel_amount: f32,
    /// `float temperature`
    pub temperature: f32,
    /// `float volume_density`
    pub volume_density: f32,
    /// `float surface_distance`
    pub surface_distance: f32,
    /// `float particle_size`
    pub particle_size: f32,
    /// `int subframes`
    pub subframes: i32,
    /// `float texture_size`
    pub texture_size: f32,
    /// `float texture_offset`
    pub texture_offset: f32,
    /// `char uvlayer_name[68]`
    pub uvlayer_name: [u8; 68],
    /// `short vgroup_density`
    pub vgroup_density: i16,
    /// `eFluidFlow_Type type`
    pub type_: FluidflowType,
    /// `eFluidFlow_Behavior behavior`
    pub behavior: FluidflowBehavior,
    /// `eFluidFlow_Source source`
    pub source: FluidflowSource,
    /// `eFluidFlow_TextureType texture_type`
    pub texture_type: FluidflowTexturetype,
}
impl Default for FluidFlowSettings {
    fn default() -> Self {
        Self {
            fmd: NIL,
            mesh: NIL,
            psys: NIL,
            noise_texture: NIL,
            verts_old: NIL,
            numverts: 0,
            vel_multi: 1.0,
            vel_normal: 0.0,
            vel_random: 0.0,
            vel_coord: [0.0; 3],
            density: 1.0,
            color: [0.0; 3],
            fuel_amount: 1.0,
            temperature: 1.0,
            volume_density: 0.0,
            surface_distance: 1.0,
            particle_size: 1.0,
            subframes: 0,
            texture_size: 1.0,
            texture_offset: 0.0,
            uvlayer_name: [0; 68],
            vgroup_density: 0,
            type_: FLUID_FLOW_TYPE_SMOKE,
            behavior: FLUID_FLOW_BEHAVIOR_GEOMETRY,
            source: FLUID_FLOW_SOURCE_MESH,
            texture_type: FLUID_FLOW_TEXTURE_MAP_AUTO,
        }
    }
}

/// `FluidEffectorSettings`.
#[derive(Clone)]
pub struct FluidEffectorSettings {
    /// `FluidModifierData *fmd`
    pub fmd: usize,
    /// `Mesh *mesh`
    pub mesh: usize,
    /// `float *verts_old`
    pub verts_old: usize,
    /// `int numverts`
    pub numverts: i32,
    /// `float surface_distance`
    pub surface_distance: f32,
    /// `eFluidEffector_Flags flags`
    pub flags: FluideffectorFlags,
    /// `int subframes`
    pub subframes: i32,
    /// `eFluidEffector_Type type`
    pub type_: FluideffectorType,
    /// `float vel_multi`
    pub vel_multi: f32,
    /// `eFluidEffector_GuideMode guide_mode`
    pub guide_mode: FluideffectorGuidemode,
}
impl Default for FluidEffectorSettings {
    fn default() -> Self {
        Self {
            fmd: NIL,
            mesh: NIL,
            verts_old: NIL,
            numverts: 0,
            surface_distance: 0.0,
            flags: FLUID_EFFECTOR_USE_EFFEC,
            subframes: 0,
            type_: FLUID_EFFECTOR_TYPE_COLLISION,
            vel_multi: 1.0,
            guide_mode: FLUID_EFFECTOR_GUIDE_OVERRIDE,
        }
    }
}
