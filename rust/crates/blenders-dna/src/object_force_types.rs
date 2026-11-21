//! Port of `DNA_object_force_types.h` — effector weights, field, softbody settings.

use crate::id::NIL;
use blenders_bli::listbase::ListBase;

/// `eEffectorWeight_Flag` (short).
pub type EffectorWeightFlag = i16;
pub const EFF_WEIGHT_DO_HAIR: EffectorWeightFlag = 1;

/// `ePField_Flag`.
pub type PFieldFlag = i32;
pub const PFIELD_USEMAX: PFieldFlag = 1 << 0;
pub const PFIELD_GUIDE_PATH_ADD: PFieldFlag = 1 << 2;
pub const PFIELD_PLANAR: PFieldFlag = 1 << 3;
pub const PDEFLE_KILL_PART: PFieldFlag = 1 << 4;
pub const PFIELD_POSZ: PFieldFlag = 1 << 5;
pub const PFIELD_TEX_OBJECT: PFieldFlag = 1 << 6;
pub const PFIELD_GLOBAL_CO: PFieldFlag = 1 << 6;
pub const PFIELD_TEX_2D: PFieldFlag = 1 << 7;
pub const PFIELD_MULTIPLE_SPRINGS: PFieldFlag = 1 << 7;
pub const PFIELD_USEMIN: PFieldFlag = 1 << 8;
pub const PFIELD_USEMAXR: PFieldFlag = 1 << 9;
pub const PFIELD_USEMINR: PFieldFlag = 1 << 10;
pub const PFIELD_TEX_ROOTCO: PFieldFlag = 1 << 11;
pub const PFIELD_SURFACE: PFieldFlag = 1 << 12;
pub const PFIELD_VISIBILITY: PFieldFlag = 1 << 13;
pub const PFIELD_DO_LOCATION: PFieldFlag = 1 << 14;
pub const PFIELD_DO_ROTATION: PFieldFlag = 1 << 15;
pub const PFIELD_GUIDE_PATH_WEIGHT: PFieldFlag = 1 << 16;
pub const PFIELD_SMOKE_DENSITY: PFieldFlag = 1 << 17;
pub const PFIELD_GRAVITATION: PFieldFlag = 1 << 18;
pub const PFIELD_CLOTH_USE_CULLING: PFieldFlag = 1 << 19;
pub const PFIELD_CLOTH_USE_NORMAL: PFieldFlag = 1 << 20;

/// `ePField_Falloff` (short).
pub type PFieldFalloff = i16;
pub const PFIELD_FALL_SPHERE: PFieldFalloff = 0;
pub const PFIELD_FALL_TUBE: PFieldFalloff = 1;
pub const PFIELD_FALL_CONE: PFieldFalloff = 2;

/// `ePField_Shape` (short).
pub type PFieldShape = i16;
pub const PFIELD_SHAPE_POINT: PFieldShape = 0;
pub const PFIELD_SHAPE_PLANE: PFieldShape = 1;
pub const PFIELD_SHAPE_SURFACE: PFieldShape = 2;
pub const PFIELD_SHAPE_POINTS: PFieldShape = 3;
pub const PFIELD_SHAPE_LINE: PFieldShape = 4;

/// `ePField_TexMode` (short).
pub type PFieldTexMode = i16;
pub const PFIELD_TEX_RGB: PFieldTexMode = 0;
pub const PFIELD_TEX_GRAD: PFieldTexMode = 1;
pub const PFIELD_TEX_CURL: PFieldTexMode = 2;

/// `ePField_ZDir` (short).
pub type PFieldZDir = i16;
pub const PFIELD_Z_BOTH: PFieldZDir = 0;
pub const PFIELD_Z_POS: PFieldZDir = 1;
pub const PFIELD_Z_NEG: PFieldZDir = 2;

/// `eSoftBody_Flag`.
pub type SoftBodyFlag = i32;
pub const OB_SB_ENABLE: SoftBodyFlag = 1 << 0;
pub const OB_SB_GOAL: SoftBodyFlag = 1 << 1;
pub const OB_SB_EDGES: SoftBodyFlag = 1 << 2;
pub const OB_SB_QUADS: SoftBodyFlag = 1 << 3;
pub const OB_SB_POSTDEF: SoftBodyFlag = 1 << 4;
pub const OB_SB_SELF: SoftBodyFlag = 1 << 9;
pub const OB_SB_FACECOLL: SoftBodyFlag = 1 << 10;
pub const OB_SB_EDGECOLL: SoftBodyFlag = 1 << 11;
pub const OB_SB_AERO_ANGLE: SoftBodyFlag = 1 << 14;

/// `eSoftBody_SolverFlag` (char).
pub type SoftBodySolverFlag = i8;
pub const SBSO_MONITOR: SoftBodySolverFlag = 1 << 0;
pub const SBSO_OLDERR: SoftBodySolverFlag = 1 << 1;
pub const SBSO_ESTIMATEIPO: SoftBodySolverFlag = 1 << 2;

/// `eSoftBody_Mode` (short).
pub type SoftBodyMode = i16;
pub const SBC_MODE_MANUAL: SoftBodyMode = 0;
pub const SBC_MODE_AVG: SoftBodyMode = 1;
pub const SBC_MODE_MIN: SoftBodyMode = 2;
pub const SBC_MODE_MAX: SoftBodyMode = 3;
pub const SBC_MODE_AVGMINMAX: SoftBodyMode = 4;

/// `ePFieldType` (short).
pub type PFieldType = i16;
pub const PFIELD_NULL: PFieldType = 0;
pub const PFIELD_FORCE: PFieldType = 1;
pub const PFIELD_VORTEX: PFieldType = 2;
pub const PFIELD_MAGNET: PFieldType = 3;
pub const PFIELD_WIND: PFieldType = 4;
pub const PFIELD_GUIDE: PFieldType = 5;
pub const PFIELD_TEXTURE: PFieldType = 6;
pub const PFIELD_HARMONIC: PFieldType = 7;
pub const PFIELD_CHARGE: PFieldType = 8;
pub const PFIELD_LENNARDJ: PFieldType = 9;
pub const PFIELD_BOID: PFieldType = 10;
pub const PFIELD_TURBULENCE: PFieldType = 11;
pub const PFIELD_DRAG: PFieldType = 12;
pub const PFIELD_FLUIDFLOW: PFieldType = 13;
pub const NUM_PFIELD_TYPES: i32 = 14;

/// `PartDeflect` — particle deflector / field settings.
#[derive(Clone)]
pub struct PartDeflect {
    pub flag: PFieldFlag,
    pub deflect: i16,
    pub forcefield: PFieldType,
    pub falloff: PFieldFalloff,
    pub shape: PFieldShape,
    pub tex_mode: PFieldTexMode,
    pub kink: i16,
    pub kink_axis: i16,
    pub zdir: PFieldZDir,
    pub f_strength: f32,
    pub f_damp: f32,
    pub f_flow: f32,
    pub f_wind_factor: f32,
    pub f_size: f32,
    pub f_power: f32,
    pub maxdist: f32,
    pub mindist: f32,
    pub f_power_r: f32,
    pub maxrad: f32,
    pub minrad: f32,
    pub pdef_damp: f32,
    pub pdef_rdamp: f32,
    pub pdef_perm: f32,
    pub pdef_frict: f32,
    pub pdef_rfrict: f32,
    pub pdef_stickness: f32,
    pub absorption: f32,
    pub pdef_sbdamp: f32,
    pub pdef_sbift: f32,
    pub pdef_sboft: f32,
    pub clump_fac: f32,
    pub clump_pow: f32,
    pub kink_freq: f32,
    pub kink_shape: f32,
    pub kink_amp: f32,
    pub free_end: f32,
    pub tex_nabla: f32,
    /// `Tex *tex`.
    pub tex: usize,
    pub f_noise: f32,
    pub seed: i32,
    pub drawvec1: [f32; 4],
    pub drawvec2: [f32; 4],
    pub drawvec_falloff_min: [f32; 3],
    pub drawvec_falloff_max: [f32; 3],
    /// `Object *f_source`.
    pub f_source: usize,
    pub pdef_cfrict: f32,
}
impl Default for PartDeflect {
    fn default() -> Self {
        Self {
            flag: 0,
            deflect: 0,
            forcefield: PFIELD_NULL,
            falloff: PFIELD_FALL_SPHERE,
            shape: PFIELD_SHAPE_POINT,
            tex_mode: PFIELD_TEX_RGB,
            kink: 0,
            kink_axis: 0,
            zdir: PFIELD_Z_BOTH,
            f_strength: 0.0,
            f_damp: 0.0,
            f_flow: 0.0,
            f_wind_factor: 0.0,
            f_size: 0.0,
            f_power: 0.0,
            maxdist: 0.0,
            mindist: 0.0,
            f_power_r: 0.0,
            maxrad: 0.0,
            minrad: 0.0,
            pdef_damp: 0.0,
            pdef_rdamp: 0.0,
            pdef_perm: 0.0,
            pdef_frict: 0.0,
            pdef_rfrict: 0.0,
            pdef_stickness: 0.0,
            absorption: 0.0,
            pdef_sbdamp: 0.0,
            pdef_sbift: 0.0,
            pdef_sboft: 0.0,
            clump_fac: 0.0,
            clump_pow: 0.0,
            kink_freq: 0.0,
            kink_shape: 0.0,
            kink_amp: 0.0,
            free_end: 0.0,
            tex_nabla: 0.0,
            tex: NIL,
            f_noise: 0.0,
            seed: 0,
            drawvec1: [0.0; 4],
            drawvec2: [0.0; 4],
            drawvec_falloff_min: [0.0; 3],
            drawvec_falloff_max: [0.0; 3],
            f_source: NIL,
            pdef_cfrict: 0.0,
        }
    }
}

/// `EffectorWeights`.
#[derive(Clone, Default)]
pub struct EffectorWeights {
    /// `Collection *group`.
    pub group: usize,
    /// `float weight[NUM_PFIELD_TYPES]`.
    pub weight: [f32; 14],
    pub global_gravity: f32,
    pub flag: EffectorWeightFlag,
}

/// `SBVertex`.
#[derive(Clone, Default)]
pub struct SbVertex {
    pub vec: [f32; 4],
}

/// `SoftBody_Shared`.
#[derive(Clone, Default)]
pub struct SoftBodyShared {
    /// `PointCache *pointcache`.
    pub pointcache: usize,
    /// `ListBaseT<PointCache>`.
    pub ptcaches: ListBase,
}

/// `SoftBody`.
#[derive(Clone)]
pub struct SoftBody {
    pub totpoint: i32,
    pub totspring: i32,
    /// `BodyPoint *bpoint`, `BodySpring *bspring` → arena refs.
    pub bpoint: usize,
    pub bspring: usize,
    pub msg_lock: i8,
    pub msg_value: i16,
    pub nodemass: f32,
    /// `char namedVG_Mass[64]`.
    pub named_vg_mass: [u8; 64],
    pub grav: f32,
    pub mediafrict: f32,
    pub rklimit: f32,
    pub physics_speed: f32,
    pub goalspring: f32,
    pub goalfrict: f32,
    pub mingoal: f32,
    pub maxgoal: f32,
    pub defgoal: f32,
    pub vertgroup: i16,
    /// `char namedVG_Softgoal[64]`.
    pub named_vg_softgoal: [u8; 64],
    pub fuzzyness: i16,
    pub inspring: f32,
    pub infrict: f32,
    /// `char namedVG_Spring_K[64]`.
    pub named_vg_spring_k: [u8; 64],
    pub local: i8,
    pub solverflags: SoftBodySolverFlag,
    /// `SBVertex **keys` → Vec of Vec refs.
    pub keys: Vec<usize>,
    pub totpointkey: i32,
    pub totkey: i32,
    pub secondspring: f32,
    pub colball: f32,
    pub balldamp: f32,
    pub ballstiff: f32,
    pub sbc_mode: SoftBodyMode,
    pub aeroedge: i16,
    pub minloops: i16,
    pub maxloops: i16,
    pub choke: i16,
    pub solver_id: i16,
    pub plastic: i16,
    pub springpreload: i16,
    /// `SBScratch *scratch`.
    pub scratch: usize,
    pub shearstiff: f32,
    pub inpush: f32,
    /// `SoftBody_Shared *shared`.
    pub shared: usize,
    /// `DNA_DEPRECATED PointCache *pointcache`.
    pub pointcache: usize,
    /// `ListBaseT<PointCache>`.
    pub ptcaches: ListBase,
    /// `Collection *collision_group`.
    pub collision_group: usize,
    /// `EffectorWeights *effector_weights`.
    pub effector_weights: usize,
    pub lcom: [f32; 3],
    pub lrot: [[f32; 3]; 3],
    pub lscale: [[f32; 3]; 3],
    pub last_frame: i32,
}
impl Default for SoftBody {
    fn default() -> Self {
        Self {
            totpoint: 0,
            totspring: 0,
            bpoint: NIL,
            bspring: NIL,
            msg_lock: 0,
            msg_value: 0,
            nodemass: 0.0,
            named_vg_mass: [0; 64],
            grav: 0.0,
            mediafrict: 0.0,
            rklimit: 0.0,
            physics_speed: 0.0,
            goalspring: 0.0,
            goalfrict: 0.0,
            mingoal: 0.0,
            maxgoal: 0.0,
            defgoal: 0.0,
            vertgroup: 0,
            named_vg_softgoal: [0; 64],
            fuzzyness: 0,
            inspring: 0.0,
            infrict: 0.0,
            named_vg_spring_k: [0; 64],
            local: 0,
            solverflags: 0,
            keys: Vec::new(),
            totpointkey: 0,
            totkey: 0,
            secondspring: 0.0,
            colball: 0.0,
            balldamp: 0.0,
            ballstiff: 0.0,
            sbc_mode: SBC_MODE_MANUAL,
            aeroedge: 0,
            minloops: 0,
            maxloops: 0,
            choke: 0,
            solver_id: 0,
            plastic: 0,
            springpreload: 0,
            scratch: NIL,
            shearstiff: 0.0,
            inpush: 0.0,
            shared: NIL,
            pointcache: NIL,
            ptcaches: ListBase::default(),
            collision_group: NIL,
            effector_weights: NIL,
            lcom: [0.0; 3],
            lrot: [[0.0; 3]; 3],
            lscale: [[0.0; 3]; 3],
            last_frame: 0,
        }
    }
}
