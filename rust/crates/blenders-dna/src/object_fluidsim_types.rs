//! Port of `DNA_object_fluidsim_types.h` — legacy fluid sim settings.

use crate::id::NIL;

/// `eFluidsim_Type` (short).
pub type FluidsimType = i16;
pub const OB_FLUIDSIM_ENABLE: FluidsimType = 1;
pub const OB_FLUIDSIM_DOMAIN: FluidsimType = 1 << 1;
pub const OB_FLUIDSIM_FLUID: FluidsimType = 1 << 2;
pub const OB_FLUIDSIM_OBSTACLE: FluidsimType = 1 << 3;
pub const OB_FLUIDSIM_INFLOW: FluidsimType = 1 << 4;
pub const OB_FLUIDSIM_OUTFLOW: FluidsimType = 1 << 5;
pub const OB_FLUIDSIM_PARTICLE: FluidsimType = 1 << 6;
pub const OB_FLUIDSIM_CONTROL: FluidsimType = 1 << 7;

/// `eFluidsim_Flag`.
pub type FluidsimFlag = i32;
pub const OB_FLUIDSIM_REVERSE: FluidsimFlag = 1 << 0;
pub const OB_FLUIDSIM_ACTIVE: FluidsimFlag = 1 << 1;
pub const OB_FLUIDSIM_OVERRIDE_TIME: FluidsimFlag = 1 << 2;

/// `FluidVertexVelocity`.
#[derive(Clone, Default)]
pub struct FluidVertexVelocity {
    pub vel: [f32; 3],
}

/// `FluidsimSettings`.
#[derive(Clone)]
pub struct FluidsimSettings {
    /// `FluidsimModifierData *fmd`.
    pub fmd: usize,
    pub threads: i32,
    pub type_: FluidsimType,
    pub show_advancedoptions: i16,
    pub resolutionxyz: i16,
    pub previewresxyz: i16,
    pub realsize: f32,
    pub gui_display_mode: i16,
    pub render_display_mode: i16,
    pub viscosity_value: f32,
    /// `DNA_DEPRECATED short viscosityMode`.
    pub viscosity_mode: i16,
    pub viscosity_exponent: i16,
    pub grav: [f32; 3],
    pub anim_start: f32,
    pub anim_end: f32,
    pub bake_start: i32,
    pub bake_end: i32,
    pub frame_offset: i32,
    pub gstar: f32,
    pub max_refine: i32,
    pub ini_velx: f32,
    pub ini_vely: f32,
    pub ini_velz: f32,
    /// `char surfdataPath[1024]`.
    pub surfdata_path: [u8; 1024],
    pub bb_start: [f32; 3],
    pub bb_size: [f32; 3],
    pub type_flags: i16,
    pub domain_novecgen: i8,
    pub volume_init_type: i8,
    pub part_slip_value: f32,
    pub generate_tracers: i32,
    pub generate_particles: f32,
    pub surface_smoothing: f32,
    pub surface_subdivs: i32,
    pub flag: FluidsimFlag,
    pub particle_inf_size: f32,
    pub particle_inf_alpha: f32,
    pub far_field_size: f32,
    /// `FluidVertexVelocity *meshVelocities` → Vec.
    pub mesh_velocities: Vec<FluidVertexVelocity>,
    pub totvert: i32,
    pub cps_time_start: f32,
    pub cps_time_end: f32,
    pub cps_quality: f32,
    pub attractforce_strength: f32,
    pub attractforce_radius: f32,
    pub velocityforce_strength: f32,
    pub velocityforce_radius: f32,
    pub lastgoodframe: i32,
    pub anim_rate: f32,
}
impl Default for FluidsimSettings {
    fn default() -> Self {
        Self {
            fmd: NIL,
            threads: 0,
            type_: 0,
            show_advancedoptions: 0,
            resolutionxyz: 0,
            previewresxyz: 0,
            realsize: 0.0,
            gui_display_mode: 0,
            render_display_mode: 0,
            viscosity_value: 0.0,
            viscosity_mode: 0,
            viscosity_exponent: 0,
            grav: [0.0; 3],
            anim_start: 0.0,
            anim_end: 0.0,
            bake_start: 0,
            bake_end: 0,
            frame_offset: 0,
            gstar: 0.0,
            max_refine: 0,
            ini_velx: 0.0,
            ini_vely: 0.0,
            ini_velz: 0.0,
            surfdata_path: [0; 1024],
            bb_start: [0.0; 3],
            bb_size: [0.0; 3],
            type_flags: 0,
            domain_novecgen: 0,
            volume_init_type: 0,
            part_slip_value: 0.0,
            generate_tracers: 0,
            generate_particles: 0.0,
            surface_smoothing: 0.0,
            surface_subdivs: 0,
            flag: 0,
            particle_inf_size: 0.0,
            particle_inf_alpha: 0.0,
            far_field_size: 0.0,
            mesh_velocities: Vec::new(),
            totvert: 0,
            cps_time_start: 0.0,
            cps_time_end: 0.0,
            cps_quality: 0.0,
            attractforce_strength: 0.0,
            attractforce_radius: 0.0,
            velocityforce_strength: 0.0,
            velocityforce_radius: 0.0,
            lastgoodframe: 0,
            anim_rate: 0.0,
        }
    }
}
