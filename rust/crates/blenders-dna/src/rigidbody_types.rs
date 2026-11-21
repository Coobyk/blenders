//! Port of `DNA_rigidbody_types.h` — rigid body world/object/constraint.

use crate::id::NIL;
use blenders_bli::listbase::ListBase;

/// `eRigidBodyWorld_Flag`.
pub type RigidBodyWorldFlag = i32;
pub const RBW_FLAG_MUTED: RigidBodyWorldFlag = 1 << 0;
pub const RBW_FLAG_USE_SPLIT_IMPULSE: RigidBodyWorldFlag = 1 << 2;

/// `eRigidBodyOb_Type` (short).
pub type RigidBodyObType = i16;
pub const RBO_TYPE_ACTIVE: RigidBodyObType = 0;
pub const RBO_TYPE_PASSIVE: RigidBodyObType = 1;

/// `eRigidBodyOb_Flag`.
pub type RigidBodyObFlag = i32;
pub const RBO_FLAG_KINEMATIC: RigidBodyObFlag = 1 << 0;
pub const RBO_FLAG_NEEDS_VALIDATE: RigidBodyObFlag = 1 << 1;
pub const RBO_FLAG_NEEDS_RESHAPE: RigidBodyObFlag = 1 << 2;
pub const RBO_FLAG_USE_DEACTIVATION: RigidBodyObFlag = 1 << 3;
pub const RBO_FLAG_START_DEACTIVATED: RigidBodyObFlag = 1 << 4;
pub const RBO_FLAG_DISABLED: RigidBodyObFlag = 1 << 5;
pub const RBO_FLAG_USE_MARGIN: RigidBodyObFlag = 1 << 6;
pub const RBO_FLAG_USE_DEFORM: RigidBodyObFlag = 1 << 7;

/// `eRigidBody_Shape` (short).
pub type RigidBodyShape = i16;
pub const RB_SHAPE_BOX: RigidBodyShape = 0;
pub const RB_SHAPE_SPHERE: RigidBodyShape = 1;
pub const RB_SHAPE_CAPSULE: RigidBodyShape = 2;
pub const RB_SHAPE_CYLINDER: RigidBodyShape = 3;
pub const RB_SHAPE_CONE: RigidBodyShape = 4;
pub const RB_SHAPE_CONVEXH: RigidBodyShape = 5;
pub const RB_SHAPE_TRIMESH: RigidBodyShape = 6;
pub const RB_SHAPE_COMPOUND: RigidBodyShape = 7;

/// `eRigidBody_MeshSource` (short).
pub type RigidBodyMeshSource = i16;
pub const RBO_MESH_BASE: RigidBodyMeshSource = 0;
pub const RBO_MESH_DEFORM: RigidBodyMeshSource = 1;
pub const RBO_MESH_FINAL: RigidBodyMeshSource = 2;

/// `eRigidBodyCon_Type` (short).
pub type RigidBodyConType = i16;
pub const RBC_TYPE_POINT: RigidBodyConType = 0;
pub const RBC_TYPE_HINGE: RigidBodyConType = 1;
pub const RBC_TYPE_SLIDER: RigidBodyConType = 3;
pub const RBC_TYPE_6DOF: RigidBodyConType = 5;
pub const RBC_TYPE_6DOF_SPRING: RigidBodyConType = 6;
pub const RBC_TYPE_FIXED: RigidBodyConType = 8;
pub const RBC_TYPE_PISTON: RigidBodyConType = 9;
pub const RBC_TYPE_MOTOR: RigidBodyConType = 11;

/// `eRigidBodyCon_SpringType` (char).
pub type RigidBodyConSpringType = i8;
pub const RBC_SPRING_TYPE1: RigidBodyConSpringType = 0;
pub const RBC_SPRING_TYPE2: RigidBodyConSpringType = 1;

/// `eRigidBodyCon_Flag`.
pub type RigidBodyConFlag = i32;
pub const RBC_FLAG_ENABLED: RigidBodyConFlag = 1 << 0;
pub const RBC_FLAG_NEEDS_VALIDATE: RigidBodyConFlag = 1 << 1;
pub const RBC_FLAG_DISABLE_COLLISIONS: RigidBodyConFlag = 1 << 2;
pub const RBC_FLAG_USE_BREAKING: RigidBodyConFlag = 1 << 3;
pub const RBC_FLAG_OVERRIDE_SOLVER_ITERATIONS: RigidBodyConFlag = 1 << 4;
pub const RBC_FLAG_USE_LIMIT_LIN_X: RigidBodyConFlag = 1 << 5;
pub const RBC_FLAG_USE_LIMIT_LIN_Y: RigidBodyConFlag = 1 << 6;
pub const RBC_FLAG_USE_LIMIT_LIN_Z: RigidBodyConFlag = 1 << 7;
pub const RBC_FLAG_USE_LIMIT_ANG_X: RigidBodyConFlag = 1 << 8;
pub const RBC_FLAG_USE_LIMIT_ANG_Y: RigidBodyConFlag = 1 << 9;
pub const RBC_FLAG_USE_LIMIT_ANG_Z: RigidBodyConFlag = 1 << 10;
pub const RBC_FLAG_USE_SPRING_X: RigidBodyConFlag = 1 << 11;
pub const RBC_FLAG_USE_SPRING_Y: RigidBodyConFlag = 1 << 12;
pub const RBC_FLAG_USE_SPRING_Z: RigidBodyConFlag = 1 << 13;
pub const RBC_FLAG_USE_MOTOR_LIN: RigidBodyConFlag = 1 << 14;
pub const RBC_FLAG_USE_MOTOR_ANG: RigidBodyConFlag = 1 << 15;
pub const RBC_FLAG_USE_SPRING_ANG_X: RigidBodyConFlag = 1 << 16;
pub const RBC_FLAG_USE_SPRING_ANG_Y: RigidBodyConFlag = 1 << 17;
pub const RBC_FLAG_USE_SPRING_ANG_Z: RigidBodyConFlag = 1 << 18;

/// `RigidBodyWorld_Shared`.
#[derive(Clone, Default)]
pub struct RigidBodyWorldShared {
    /// `PointCache *pointcache`.
    pub pointcache: usize,
    /// `ListBaseT<PointCache>`.
    pub ptcaches: ListBase,
    /// `RigidBodyWorld_Runtime *runtime`.
    pub runtime: usize,
}

/// `RigidBodyWorld`.
#[derive(Clone, Default)]
pub struct RigidBodyWorld {
    /// `EffectorWeights *effector_weights`.
    pub effector_weights: usize,
    /// `Collection *group`.
    pub group: usize,
    /// `Object **objects` → Vec of refs.
    pub objects: Vec<usize>,
    /// `Collection *constraints`.
    pub constraints: usize,
    pub ltime: f32,
    /// `RigidBodyWorld_Shared *shared`.
    pub shared: usize,
    /// `DNA_DEPRECATED PointCache *pointcache`.
    pub pointcache: usize,
    /// `ListBaseT<PointCache>`.
    pub ptcaches: ListBase,
    pub numbodies: i32,
    pub substeps_per_frame: i16,
    pub num_solver_iterations: i16,
    pub flag: RigidBodyWorldFlag,
    pub time_scale: f32,
}

/// `RigidBodyOb_Shared`.
#[derive(Clone, Default)]
pub struct RigidBodyObShared {
    /// `void *physics_object`, `*physics_shape` (Bullet handles).
    pub physics_object: usize,
    pub physics_shape: usize,
}

/// `RigidBodyOb`.
#[derive(Clone)]
pub struct RigidBodyOb {
    pub type_: RigidBodyObType,
    pub shape: RigidBodyShape,
    pub flag: RigidBodyObFlag,
    pub col_groups: i32,
    pub mesh_source: RigidBodyMeshSource,
    pub mass: f32,
    pub friction: f32,
    pub restitution: f32,
    pub margin: f32,
    pub lin_damping: f32,
    pub ang_damping: f32,
    pub lin_sleep_thresh: f32,
    pub ang_sleep_thresh: f32,
    pub orn: [f32; 4],
    pub pos: [f32; 3],
    /// `RigidBodyOb_Shared *shared`.
    pub shared: usize,
}
impl Default for RigidBodyOb {
    fn default() -> Self {
        Self {
            type_: RBO_TYPE_ACTIVE,
            shape: RB_SHAPE_BOX,
            flag: 0,
            col_groups: 0,
            mesh_source: RBO_MESH_BASE,
            mass: 0.0,
            friction: 0.0,
            restitution: 0.0,
            margin: 0.0,
            lin_damping: 0.0,
            ang_damping: 0.0,
            lin_sleep_thresh: 0.0,
            ang_sleep_thresh: 0.0,
            orn: [0.0; 4],
            pos: [0.0; 3],
            shared: NIL,
        }
    }
}

/// `RigidBodyCon`.
#[derive(Clone)]
pub struct RigidBodyCon {
    /// `Object *ob1`, `*ob2`.
    pub ob1: usize,
    pub ob2: usize,
    pub type_: RigidBodyConType,
    pub num_solver_iterations: i16,
    pub flag: RigidBodyConFlag,
    pub breaking_threshold: f32,
    pub spring_type: RigidBodyConSpringType,
    pub limit_lin_x_lower: f32,
    pub limit_lin_x_upper: f32,
    pub limit_lin_y_lower: f32,
    pub limit_lin_y_upper: f32,
    pub limit_lin_z_lower: f32,
    pub limit_lin_z_upper: f32,
    pub limit_ang_x_lower: f32,
    pub limit_ang_x_upper: f32,
    pub limit_ang_y_lower: f32,
    pub limit_ang_y_upper: f32,
    pub limit_ang_z_lower: f32,
    pub limit_ang_z_upper: f32,
    pub spring_stiffness_x: f32,
    pub spring_stiffness_y: f32,
    pub spring_stiffness_z: f32,
    pub spring_stiffness_ang_x: f32,
    pub spring_stiffness_ang_y: f32,
    pub spring_stiffness_ang_z: f32,
    pub spring_damping_x: f32,
    pub spring_damping_y: f32,
    pub spring_damping_z: f32,
    pub spring_damping_ang_x: f32,
    pub spring_damping_ang_y: f32,
    pub spring_damping_ang_z: f32,
    pub motor_lin_target_velocity: f32,
    pub motor_ang_target_velocity: f32,
    pub motor_lin_max_impulse: f32,
    pub motor_ang_max_impulse: f32,
    /// `void *physics_constraint` (Bullet handle).
    pub physics_constraint: usize,
}
impl Default for RigidBodyCon {
    fn default() -> Self {
        Self {
            ob1: NIL,
            ob2: NIL,
            type_: RBC_TYPE_POINT,
            num_solver_iterations: 0,
            flag: 0,
            breaking_threshold: 0.0,
            spring_type: RBC_SPRING_TYPE1,
            limit_lin_x_lower: 0.0, limit_lin_x_upper: 0.0,
            limit_lin_y_lower: 0.0, limit_lin_y_upper: 0.0,
            limit_lin_z_lower: 0.0, limit_lin_z_upper: 0.0,
            limit_ang_x_lower: 0.0, limit_ang_x_upper: 0.0,
            limit_ang_y_lower: 0.0, limit_ang_y_upper: 0.0,
            limit_ang_z_lower: 0.0, limit_ang_z_upper: 0.0,
            spring_stiffness_x: 0.0, spring_stiffness_y: 0.0, spring_stiffness_z: 0.0,
            spring_stiffness_ang_x: 0.0, spring_stiffness_ang_y: 0.0, spring_stiffness_ang_z: 0.0,
            spring_damping_x: 0.0, spring_damping_y: 0.0, spring_damping_z: 0.0,
            spring_damping_ang_x: 0.0, spring_damping_ang_y: 0.0, spring_damping_ang_z: 0.0,
            motor_lin_target_velocity: 0.0,
            motor_ang_target_velocity: 0.0,
            motor_lin_max_impulse: 0.0,
            motor_ang_max_impulse: 0.0,
            physics_constraint: NIL,
        }
    }
}
