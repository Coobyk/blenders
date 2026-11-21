//! Port of `DNA_cloth_types.h` — cloth sim/collision settings.

use crate::id::NIL;

/// `CLOTH_SIMSETTINGS_FLAGS`.
pub type ClothSimSettingsFlags = i32;
pub const CLOTH_SIMSETTINGS_FLAG_COLLOBJ: ClothSimSettingsFlags = 1 << 2;
pub const CLOTH_SIMSETTINGS_FLAG_GOAL: ClothSimSettingsFlags = 1 << 3;
pub const CLOTH_SIMSETTINGS_FLAG_TEARING: ClothSimSettingsFlags = 1 << 4;
pub const CLOTH_SIMSETTINGS_FLAG_PRESSURE: ClothSimSettingsFlags = 1 << 5;
pub const CLOTH_SIMSETTINGS_FLAG_PRESSURE_VOL: ClothSimSettingsFlags = 1 << 6;
pub const CLOTH_SIMSETTINGS_FLAG_INTERNAL_SPRINGS: ClothSimSettingsFlags = 1 << 7;
pub const CLOTH_SIMSETTINGS_FLAG_SCALING: ClothSimSettingsFlags = 1 << 8;
pub const CLOTH_SIMSETTINGS_FLAG_INTERNAL_SPRINGS_NORMAL: ClothSimSettingsFlags = 1 << 9;
pub const CLOTH_SIMSETTINGS_FLAG_RESIST_SPRING_COMPRESS: ClothSimSettingsFlags = 1 << 13;
pub const CLOTH_SIMSETTINGS_FLAG_SEW: ClothSimSettingsFlags = 1 << 14;
pub const CLOTH_SIMSETTINGS_FLAG_DYNAMIC_BASEMESH: ClothSimSettingsFlags = 1 << 15;

/// `CLOTH_BENDING_MODEL` (short).
pub type ClothBendingModel = i16;
pub const CLOTH_BENDING_LINEAR: ClothBendingModel = 0;
pub const CLOTH_BENDING_ANGULAR: ClothBendingModel = 1;

/// `CLOTH_COLLISIONSETTINGS_FLAGS`.
pub type ClothCollisionSettingsFlags = i32;
pub const CLOTH_COLLSETTINGS_FLAG_ENABLED: ClothCollisionSettingsFlags = 1 << 1;
pub const CLOTH_COLLSETTINGS_FLAG_SELF: ClothCollisionSettingsFlags = 1 << 2;

/// `ClothSimSettings`.
#[derive(Clone)]
pub struct ClothSimSettings {
    /// `LinkNode *cache`.
    pub cache: usize,
    pub mingoal: f32,
    /// `DNA_DEPRECATED float Cdis`.
    pub cdis: f32,
    pub cvi: f32,
    pub gravity: [f32; 3],
    pub dt: f32,
    pub mass: f32,
    /// `DNA_DEPRECATED float structural`.
    pub structural: f32,
    pub shear: f32,
    pub bending: f32,
    pub max_bend: f32,
    /// `DNA_DEPRECATED float max_struct`.
    pub max_struct: f32,
    pub max_shear: f32,
    pub max_sewing: f32,
    pub avg_spring_len: f32,
    pub timescale: f32,
    pub time_scale: f32,
    pub maxgoal: f32,
    pub eff_force_scale: f32,
    pub eff_wind_scale: f32,
    pub sim_time_old: f32,
    pub defgoal: f32,
    pub goalspring: f32,
    pub goalfrict: f32,
    pub velocity_smooth: f32,
    pub density_target: f32,
    pub density_strength: f32,
    pub collider_friction: f32,
    /// `DNA_DEPRECATED float vel_damping`.
    pub vel_damping: f32,
    pub shrink_min: f32,
    pub shrink_max: f32,
    pub uniform_pressure_force: f32,
    pub target_volume: f32,
    pub pressure_factor: f32,
    pub fluid_density: f32,
    pub vgroup_pressure: i16,
    pub bending_damping: f32,
    pub voxel_cell_size: f32,
    pub steps_per_frame: i32,
    pub flags: ClothSimSettingsFlags,
    /// `DNA_DEPRECATED int preroll`.
    pub preroll: i32,
    pub maxspringlen: i32,
    pub solver_type: i16,
    pub vgroup_bend: i16,
    pub vgroup_mass: i16,
    pub vgroup_struct: i16,
    pub vgroup_shrink: i16,
    pub shapekey_rest: i16,
    pub presets: i16,
    pub reset: i16,
    /// `EffectorWeights *effector_weights`.
    pub effector_weights: usize,
    pub bending_model: ClothBendingModel,
    pub vgroup_shear: i16,
    pub tension: f32,
    pub compression: f32,
    pub max_tension: f32,
    pub max_compression: f32,
    pub tension_damp: f32,
    pub compression_damp: f32,
    pub shear_damp: f32,
    pub internal_spring_max_length: f32,
    pub internal_spring_max_diversion: f32,
    pub vgroup_intern: i16,
    pub internal_tension: f32,
    pub internal_compression: f32,
    pub max_internal_tension: f32,
    pub max_internal_compression: f32,
}
impl Default for ClothSimSettings {
    fn default() -> Self {
        Self {
            cache: NIL,
            mingoal: 0.0,
            cdis: 0.0,
            cvi: 1.0,
            gravity: [0.0, 0.0, -9.81],
            dt: 0.0,
            mass: 0.3,
            structural: 0.0,
            shear: 5.0,
            bending: 0.5,
            max_bend: 0.5,
            max_struct: 0.0,
            max_shear: 5.0,
            max_sewing: 0.0,
            avg_spring_len: 0.0,
            timescale: 1.0,
            time_scale: 1.0,
            maxgoal: 1.0,
            eff_force_scale: 1000.0,
            eff_wind_scale: 250.0,
            sim_time_old: 0.0,
            defgoal: 0.0,
            goalspring: 1.0,
            goalfrict: 0.0,
            velocity_smooth: 0.0,
            density_target: 0.0,
            density_strength: 0.0,
            collider_friction: 0.0,
            vel_damping: 0.0,
            shrink_min: 0.0,
            shrink_max: 0.0,
            uniform_pressure_force: 0.0,
            target_volume: 0.0,
            pressure_factor: 1.0,
            fluid_density: 0.0,
            vgroup_pressure: 0,
            bending_damping: 0.5,
            voxel_cell_size: 0.1,
            steps_per_frame: 5,
            flags: CLOTH_SIMSETTINGS_FLAG_INTERNAL_SPRINGS_NORMAL,
            preroll: 0,
            maxspringlen: 10,
            solver_type: 0,
            vgroup_bend: 0,
            vgroup_mass: 0,
            vgroup_struct: 0,
            vgroup_shrink: 0,
            shapekey_rest: 0,
            presets: 2,
            reset: 0,
            effector_weights: NIL,
            bending_model: CLOTH_BENDING_ANGULAR,
            vgroup_shear: 0,
            tension: 15.0,
            compression: 15.0,
            max_tension: 15.0,
            max_compression: 15.0,
            tension_damp: 5.0,
            compression_damp: 5.0,
            shear_damp: 5.0,
            internal_spring_max_length: 0.0,
            internal_spring_max_diversion: std::f32::consts::FRAC_PI_4,
            vgroup_intern: 0,
            internal_tension: 15.0,
            internal_compression: 15.0,
            max_internal_tension: 15.0,
            max_internal_compression: 15.0,
        }
    }
}

/// `ClothCollSettings`.
#[derive(Clone)]
pub struct ClothCollSettings {
    /// `LinkNode *collision_list`.
    pub collision_list: usize,
    pub epsilon: f32,
    pub self_friction: f32,
    pub friction: f32,
    pub damping: f32,
    pub selfepsilon: f32,
    /// `DNA_DEPRECATED float repel_force`.
    pub repel_force: f32,
    /// `DNA_DEPRECATED float distance_repel`.
    pub distance_repel: f32,
    pub flags: ClothCollisionSettingsFlags,
    /// `DNA_DEPRECATED short self_loop_count`.
    pub self_loop_count: i16,
    pub loop_count: i16,
    /// `Collection *group`.
    pub group: usize,
    pub vgroup_selfcol: i16,
    pub vgroup_objcol: i16,
    pub clamp: f32,
    pub self_clamp: f32,
}
impl Default for ClothCollSettings {
    fn default() -> Self {
        Self {
            collision_list: NIL,
            epsilon: 0.015,
            self_friction: 5.0,
            friction: 5.0,
            damping: 0.0,
            selfepsilon: 0.015,
            repel_force: 0.0,
            distance_repel: 0.0,
            flags: CLOTH_COLLSETTINGS_FLAG_ENABLED,
            self_loop_count: 0,
            loop_count: 2,
            group: NIL,
            vgroup_selfcol: 0,
            vgroup_objcol: 0,
            clamp: 0.0,
            self_clamp: 0.0,
        }
    }
}
