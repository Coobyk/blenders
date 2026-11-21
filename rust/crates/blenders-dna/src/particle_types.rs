//! Port of `DNA_particle_types.h` — particle system settings.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::boid_types::BoidData;
use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;
/// `eSPH_Flag`.
pub type SphFlag = i32;
pub const SPH_VISCOELASTIC_SPRINGS: SphFlag = 1 << 0;
pub const SPH_CURRENT_REST_LENGTH: SphFlag = 1 << 1;
pub const SPH_FAC_REPULSION: SphFlag = 1 << 2;
pub const SPH_FAC_DENSITY: SphFlag = 1 << 3;
pub const SPH_FAC_RADIUS: SphFlag = 1 << 4;
pub const SPH_FAC_VISCOSITY: SphFlag = 1 << 5;
pub const SPH_FAC_REST_LENGTH: SphFlag = 1 << 6;

/// `eSPH_Solver`.
pub type SphSolver = i16;
pub const SPH_SOLVER_DDR: SphSolver = 0;
pub const SPH_SOLVER_CLASSICAL: SphSolver = 1;

/// `eParticleDrawFlag`.
pub type Particledrawflag = i32;
pub const PART_DRAW_VEL: Particledrawflag = 1 << 0;
pub const PART_DRAW_GLOBAL_OB: Particledrawflag = 1 << 1;
pub const PART_DRAW_SIZE: Particledrawflag = 1 << 2;
pub const PART_DRAW_EMITTER: Particledrawflag = 1 << 3;
pub const PART_DRAW_HEALTH: Particledrawflag = 1 << 4;
pub const PART_ABS_PATH_TIME: Particledrawflag = 1 << 5;
pub const PART_DRAW_COUNT_GR: Particledrawflag = 1 << 6;
pub const PART_DRAW_ROTATE_OB: Particledrawflag = 1 << 7;
pub const PART_DRAW_PARENT: Particledrawflag = 1 << 8;
pub const PART_DRAW_NUM: Particledrawflag = 1 << 9;
pub const PART_DRAW_RAND_GR: Particledrawflag = 1 << 10;
pub const PART_DRAW_REN_ADAPT: Particledrawflag = 1 << 11;
pub const PART_DRAW_VEL_LENGTH: Particledrawflag = 1 << 12;
pub const PART_DRAW_MAT_COL: Particledrawflag = 1 << 13;
pub const PART_DRAW_WHOLE_GR: Particledrawflag = 1 << 14;
pub const PART_DRAW_REN_STRAND: Particledrawflag = 1 << 15;
pub const PART_DRAW_NO_SCALE_OB: Particledrawflag = 1 << 16;
pub const PART_DRAW_GUIDE_HAIRS: Particledrawflag = 1 << 17;
pub const PART_DRAW_HAIR_GRID: Particledrawflag = 1 << 18;

/// `eParticleType`.
pub type Particletype = i16;
pub const PART_EMITTER: Particletype = 0;
pub const PART_HAIR: Particletype = 2;
pub const PART_FLUID: Particletype = 3;
pub const PART_FLUID_FLIP: Particletype = 4;
pub const PART_FLUID_SPRAY: Particletype = 5;
pub const PART_FLUID_BUBBLE: Particletype = 6;
pub const PART_FLUID_FOAM: Particletype = 7;
pub const PART_FLUID_TRACER: Particletype = 8;
pub const PART_FLUID_SPRAYFOAM: Particletype = 9;
pub const PART_FLUID_SPRAYBUBBLE: Particletype = 10;
pub const PART_FLUID_FOAMBUBBLE: Particletype = 11;
pub const PART_FLUID_SPRAYFOAMBUBBLE: Particletype = 12;

/// `eMantaflowParticleType`.
pub type Mantaflowparticletype = i32;
pub const PARTICLE_TYPE_SPRAY: Mantaflowparticletype = 1 << 1;
pub const PARTICLE_TYPE_BUBBLE: Mantaflowparticletype = 1 << 2;
pub const PARTICLE_TYPE_FOAM: Mantaflowparticletype = 1 << 3;
pub const PARTICLE_TYPE_TRACER: Mantaflowparticletype = 1 << 4;
pub const PARTICLE_TYPE_DELETE: Mantaflowparticletype = 1 << 10;

/// `eParticleFlag`.
pub type Particleflag = i32;
pub const PART_REACT_STA_END: Particleflag = 1 << 0;
pub const PART_REACT_MULTIPLE: Particleflag = 1 << 1;
pub const PART_DS_EXPAND: Particleflag = 1 << 3;
pub const PART_HAIR_REGROW: Particleflag = 1 << 4;
pub const PART_UNBORN: Particleflag = 1 << 5;
pub const PART_DIED: Particleflag = 1 << 6;
pub const PART_TRAND: Particleflag = 1 << 7;
pub const PART_EDISTR: Particleflag = 1 << 8;
pub const PART_ROTATIONS: Particleflag = 1 << 9;
pub const PART_HAIR_BSPLINE: Particleflag = 1 << 10;
pub const PART_DIE_ON_COL: Particleflag = 1 << 12;
pub const PART_SIZE_DEFL: Particleflag = 1 << 13;
pub const PART_ROT_DYN: Particleflag = 1 << 14;
pub const PART_HIDE_ADVANCED_HAIR: Particleflag = 1 << 15;
pub const PART_SIZEMASS: Particleflag = 1 << 16;
pub const PART_BOIDS_2D: Particleflag = 1 << 19;
pub const PART_SELF_EFFECT: Particleflag = 1 << 22;
pub const PART_GRID_HEXAGONAL: Particleflag = 1 << 24;
pub const PART_GRID_INVERT: Particleflag = 1 << 26;
pub const PART_CHILD_EFFECT: Particleflag = 1 << 27;
pub const PART_CHILD_LONG_HAIR: Particleflag = 1 << 28;
pub const PART_CHILD_GUIDE: Particleflag = 1 << 30;

/// `eParticleFrom`.
pub type Particlefrom = i16;
pub const PART_FROM_VERT: Particlefrom = 0;
pub const PART_FROM_FACE: Particlefrom = 1;
pub const PART_FROM_VOLUME: Particlefrom = 2;
pub const PART_FROM_CHILD: Particlefrom = 4;

/// `eParticleDistribution`.
pub type Particledistribution = i16;
pub const PART_DISTR_JIT: Particledistribution = 0;
pub const PART_DISTR_RAND: Particledistribution = 1;
pub const PART_DISTR_GRID: Particledistribution = 2;

/// `eParticlePhysicsType`.
pub type Particlephysicstype = i16;
pub const PART_PHYS_NO: Particlephysicstype = 0;
pub const PART_PHYS_NEWTON: Particlephysicstype = 1;
pub const PART_PHYS_KEYED: Particlephysicstype = 2;
pub const PART_PHYS_BOIDS: Particlephysicstype = 3;
pub const PART_PHYS_FLUID: Particlephysicstype = 4;

/// `eParticleKink`.
pub type Particlekink = i16;
pub const PART_KINK_NO: Particlekink = 0;
pub const PART_KINK_CURL: Particlekink = 1;
pub const PART_KINK_RADIAL: Particlekink = 2;
pub const PART_KINK_WAVE: Particlekink = 3;
pub const PART_KINK_BRAID: Particlekink = 4;
pub const PART_KINK_SPIRAL: Particlekink = 5;

/// `eParticleChildFlag`.
pub type Particlechildflag = i32;
pub const PART_CHILD_USE_CLUMP_NOISE: Particlechildflag = 1 << 0;
pub const PART_CHILD_USE_CLUMP_CURVE: Particlechildflag = 1 << 1;
pub const PART_CHILD_USE_ROUGH_CURVE: Particlechildflag = 1 << 2;
pub const PART_CHILD_USE_TWIST_CURVE: Particlechildflag = 1 << 3;

/// `eParticleShapeFlag`.
pub type Particleshapeflag = i16;
pub const PART_SHAPE_CLOSE_TIP: Particleshapeflag = 1 << 0;

/// `eParticleDrawCol`.
pub type Particledrawcol = i16;
pub const PART_DRAW_COL_NONE: Particledrawcol = 0;
pub const PART_DRAW_COL_MAT: Particledrawcol = 1;
pub const PART_DRAW_COL_VEL: Particledrawcol = 2;
pub const PART_DRAW_COL_ACC: Particledrawcol = 3;

/// `eParticleTimeFlag`.
pub type Particletimeflag = i16;
pub const PART_TIME_AUTOSF: Particletimeflag = 1 << 0;

/// `eParticleDrawAs`.
pub type Particledrawas = i16;
pub const PART_DRAW_NOT: Particledrawas = 0;
pub const PART_DRAW_DOT: Particledrawas = 1;
pub const PART_DRAW_HALO: Particledrawas = 1;
pub const PART_DRAW_CIRC: Particledrawas = 2;
pub const PART_DRAW_CROSS: Particledrawas = 3;
pub const PART_DRAW_AXIS: Particledrawas = 4;
pub const PART_DRAW_LINE: Particledrawas = 5;
pub const PART_DRAW_PATH: Particledrawas = 6;
pub const PART_DRAW_OB: Particledrawas = 7;
pub const PART_DRAW_GR: Particledrawas = 8;
pub const PART_DRAW_BB: Particledrawas = 9;
pub const PART_DRAW_REND: Particledrawas = 10;

/// `eParticleIntegrator`.
pub type Particleintegrator = i16;
pub const PART_INT_EULER: Particleintegrator = 0;
pub const PART_INT_MIDPOINT: Particleintegrator = 1;
pub const PART_INT_RK4: Particleintegrator = 2;
pub const PART_INT_VERLET: Particleintegrator = 3;

/// `eParticleRotationMode`.
pub type Particlerotationmode = i16;
pub const PART_ROT_NOR: Particlerotationmode = 1;
pub const PART_ROT_VEL: Particlerotationmode = 2;
pub const PART_ROT_GLOB_X: Particlerotationmode = 3;
pub const PART_ROT_GLOB_Y: Particlerotationmode = 4;
pub const PART_ROT_GLOB_Z: Particlerotationmode = 5;
pub const PART_ROT_OB_X: Particlerotationmode = 6;
pub const PART_ROT_OB_Y: Particlerotationmode = 7;
pub const PART_ROT_OB_Z: Particlerotationmode = 8;
pub const PART_ROT_NOR_TAN: Particlerotationmode = 9;

/// `eParticleAngularVelocityMode`.
pub type Particleangularvelocitymode = i16;
pub const PART_AVE_VELOCITY: Particleangularvelocitymode = 1;
pub const PART_AVE_RAND: Particleangularvelocitymode = 2;
pub const PART_AVE_HORIZONTAL: Particleangularvelocitymode = 3;
pub const PART_AVE_VERTICAL: Particleangularvelocitymode = 4;
pub const PART_AVE_GLOBAL_X: Particleangularvelocitymode = 5;
pub const PART_AVE_GLOBAL_Y: Particleangularvelocitymode = 6;
pub const PART_AVE_GLOBAL_Z: Particleangularvelocitymode = 7;

/// `eParticleReactEvent`.
pub type Particlereactevent = i16;
pub const PART_EVENT_DEATH: Particlereactevent = 0;
pub const PART_EVENT_COLLIDE: Particlereactevent = 1;
pub const PART_EVENT_NEAR: Particlereactevent = 2;

/// `eParticleChildType`.
pub type Particlechildtype = i16;
pub const PART_CHILD_PARTICLES: Particlechildtype = 1;
pub const PART_CHILD_FACES: Particlechildtype = 2;

/// `eParticleSystem_Flag`.
pub type ParticlesystemFlag = i32;
pub const PSYS_CURRENT: ParticlesystemFlag = 1 << 0;
pub const PSYS_GLOBAL_HAIR: ParticlesystemFlag = 1 << 1;
pub const PSYS_HAIR_DYNAMICS: ParticlesystemFlag = 1 << 2;
pub const PSYS_KEYED_TIMING: ParticlesystemFlag = 1 << 3;
pub const PSYS_HAIR_UPDATED: ParticlesystemFlag = 1 << 5;
pub const PSYS_DELETE: ParticlesystemFlag = 1 << 8;
pub const PSYS_HAIR_DONE: ParticlesystemFlag = 1 << 9;
pub const PSYS_KEYED: ParticlesystemFlag = 1 << 10;
pub const PSYS_EDITED: ParticlesystemFlag = 1 << 11;
pub const PSYS_DISABLED: ParticlesystemFlag = 1 << 13;
pub const PSYS_OB_ANIM_RESTORE: ParticlesystemFlag = 1 << 14;
pub const PSYS_SHARED_CACHES: ParticlesystemFlag = 1 << 15;

/// `eParticle_Flag`.
pub type ParticleFlag = i16;
pub const PARS_UNEXIST: ParticleFlag = 1 << 0;
pub const PARS_NO_DISP: ParticleFlag = 1 << 1;
pub const PARS_REKEY: ParticleFlag = 1 << 3;

/// `eParticle_AliveState`.
pub type ParticleAlivestate = i16;
pub const PARS_KILLED: ParticleAlivestate = 0;
pub const PARS_DEAD: ParticleAlivestate = 1;
pub const PARS_UNBORN: ParticleAlivestate = 2;
pub const PARS_ALIVE: ParticleAlivestate = 3;
pub const PARS_DYING: ParticleAlivestate = 4;

/// `eParticleDupliWeight_Flag`.
pub type ParticledupliweightFlag = i16;
pub const PART_DUPLIW_CURRENT: ParticledupliweightFlag = 1;

/// `eParticleSystem_Vgroup`.
pub type ParticlesystemVgroup = i16;
pub const PSYS_VG_DENSITY: ParticlesystemVgroup = 0;
pub const PSYS_VG_VEL: ParticlesystemVgroup = 1;
pub const PSYS_VG_LENGTH: ParticlesystemVgroup = 2;
pub const PSYS_VG_CLUMP: ParticlesystemVgroup = 3;
pub const PSYS_VG_KINK: ParticlesystemVgroup = 4;
pub const PSYS_VG_ROUGH1: ParticlesystemVgroup = 5;
pub const PSYS_VG_ROUGH2: ParticlesystemVgroup = 6;
pub const PSYS_VG_ROUGHE: ParticlesystemVgroup = 7;
pub const PSYS_VG_SIZE: ParticlesystemVgroup = 8;
pub const PSYS_VG_TAN: ParticlesystemVgroup = 9;
pub const PSYS_VG_ROT: ParticlesystemVgroup = 10;
pub const PSYS_VG_EFFECTOR: ParticlesystemVgroup = 11;
pub const PSYS_VG_TWIST: ParticlesystemVgroup = 12;
pub const PSYS_TOT_VG: ParticlesystemVgroup = 13;

/// `eParticleTarget_Flag`.
pub type ParticletargetFlag = i16;
pub const PTARGET_CURRENT: ParticletargetFlag = 1;
pub const PTARGET_VALID: ParticletargetFlag = 2;

/// `eParticleTarget_Mode`.
pub type ParticletargetMode = i16;
pub const PTARGET_MODE_NEUTRAL: ParticletargetMode = 0;
pub const PTARGET_MODE_FRIEND: ParticletargetMode = 1;
pub const PTARGET_MODE_ENEMY: ParticletargetMode = 2;

/// `eParticleTextureInfluence`.
pub type Particletextureinfluence = i32;
pub const PAMAP_TIME: Particletextureinfluence = 1 << 0;
pub const PAMAP_LIFE: Particletextureinfluence = 1 << 1;
pub const PAMAP_DENS: Particletextureinfluence = 1 << 2;
pub const PAMAP_SIZE: Particletextureinfluence = 1 << 3;
pub const PAMAP_INIT: Particletextureinfluence = PAMAP_TIME | PAMAP_LIFE | PAMAP_DENS | PAMAP_SIZE;
pub const PAMAP_IVEL: Particletextureinfluence = 1 << 5;
pub const PAMAP_FIELD: Particletextureinfluence = 1 << 6;
pub const PAMAP_GRAVITY: Particletextureinfluence = 1 << 10;
pub const PAMAP_DAMP: Particletextureinfluence = 1 << 11;
pub const PAMAP_PHYSICS: Particletextureinfluence = PAMAP_FIELD | PAMAP_GRAVITY | PAMAP_DAMP;
pub const PAMAP_CLUMP: Particletextureinfluence = 1 << 7;
pub const PAMAP_KINK_FREQ: Particletextureinfluence = 1 << 8;
pub const PAMAP_KINK_AMP: Particletextureinfluence = 1 << 12;
pub const PAMAP_ROUGH: Particletextureinfluence = 1 << 9;
pub const PAMAP_LENGTH: Particletextureinfluence = 1 << 4;
pub const PAMAP_TWIST: Particletextureinfluence = 1 << 13;
pub const PAMAP_CHILD: Particletextureinfluence = PAMAP_CLUMP | PAMAP_KINK_FREQ | PAMAP_KINK_AMP | PAMAP_ROUGH | PAMAP_LENGTH |
                 PAMAP_TWIST;


/// `HairKey`.
#[derive(Clone)]
pub struct HairKey {
    /// `float co[3]`
    pub co: [f32; 3],
    /// `float time`
    pub time: f32,
    /// `float weight`
    pub weight: f32,
    /// `short editflag`
    pub editflag: i16,
    /// `float world_co[3]`
    pub world_co: [f32; 3],
}
impl Default for HairKey {
    fn default() -> Self {
        Self {
            co: [0.0; 3],
            time: Default::default(),
            weight: Default::default(),
            editflag: Default::default(),
            world_co: [0.0; 3],
        }
    }
}

/// `ParticleKey`.
#[derive(Clone)]
pub struct ParticleKey {
    /// `float co[3]`
    pub co: [f32; 3],
    /// `float vel[3]`
    pub vel: [f32; 3],
    /// `float rot[4]`
    pub rot: [f32; 4],
    /// `float ave[3]`
    pub ave: [f32; 3],
    /// `float time`
    pub time: f32,
}
impl Default for ParticleKey {
    fn default() -> Self {
        Self {
            co: [0.0; 3],
            vel: [0.0; 3],
            rot: [0.0; 4],
            ave: [0.0; 3],
            time: Default::default(),
        }
    }
}

/// `BoidParticle`.
#[derive(Clone)]
pub struct BoidParticle {
    /// `Object *ground`
    pub ground: usize,
    pub data: BoidData,
    /// `float gravity[3]`
    pub gravity: [f32; 3],
    /// `float wander[3]`
    pub wander: [f32; 3],
}
impl Default for BoidParticle {
    fn default() -> Self {
        Self {
            ground: NIL,
            data: Default::default(),
            gravity: [0.0; 3],
            wander: [0.0; 3],
        }
    }
}

/// `ParticleSpring`.
#[derive(Clone)]
pub struct ParticleSpring {
    /// `float rest_length`
    pub rest_length: f32,
    /// `unsigned_int particle_index[2]`
    pub particle_index: [u32; 2],
    /// `unsigned_int delete_flag`
    pub delete_flag: u32,
}
impl Default for ParticleSpring {
    fn default() -> Self {
        Self {
            rest_length: Default::default(),
            particle_index: [0; 2],
            delete_flag: Default::default(),
        }
    }
}

/// `ChildParticle`.
#[derive(Clone)]
pub struct ChildParticle {
    /// `int num`
    pub num: i32,
    /// `int parent`
    pub parent: i32,
    /// `int pa[4]`
    pub pa: [i32; 4],
    /// `float w[4]`
    pub w: [f32; 4],
    /// `float fuv[4]`
    pub fuv: [f32; 4],
    /// `float foffset`
    pub foffset: f32,
}
impl Default for ChildParticle {
    fn default() -> Self {
        Self {
            num: Default::default(),
            parent: Default::default(),
            pa: [0; 4],
            w: [0.0; 4],
            fuv: [0.0; 4],
            foffset: Default::default(),
        }
    }
}

/// `ParticleTarget`.
#[derive(Clone)]
pub struct ParticleTarget {
    /// `ParticleTarget *next`
    pub next: usize,
    /// `ParticleTarget *prev`
    pub prev: usize,
    /// `Object *ob`
    pub ob: usize,
    /// `int psys`
    pub psys: i32,
    /// `eParticleTarget_Flag flag`
    pub flag: ParticletargetFlag,
    /// `eParticleTarget_Mode mode`
    pub mode: ParticletargetMode,
    /// `float time`
    pub time: f32,
    /// `float duration`
    pub duration: f32,
}
impl Default for ParticleTarget {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            ob: NIL,
            psys: 0,
            flag: Default::default(),
            mode: Default::default(),
            time: 0.0,
            duration: 0.0,
        }
    }
}

/// `ParticleDupliWeight`.
#[derive(Clone)]
pub struct ParticleDupliWeight {
    /// `ParticleDupliWeight *next`
    pub next: usize,
    /// `ParticleDupliWeight *prev`
    pub prev: usize,
    /// `Object *ob`
    pub ob: usize,
    /// `short count`
    pub count: i16,
    /// `eParticleDupliWeight_Flag flag`
    pub flag: ParticledupliweightFlag,
    /// `short index`
    pub index: i16,
}
impl Default for ParticleDupliWeight {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            ob: NIL,
            count: 0,
            flag: Default::default(),
            index: 0,
        }
    }
}

/// `ParticleData`.
#[derive(Clone)]
pub struct ParticleData {
    pub state: ParticleKey,
    pub prev_state: ParticleKey,
    /// `HairKey *hair`
    pub hair: usize,
    /// `ParticleKey *keys`
    pub keys: usize,
    /// `BoidParticle *boid`
    pub boid: usize,
    /// `int totkey`
    pub totkey: i32,
    /// `float time`
    pub time: f32,
    /// `float lifetime`
    pub lifetime: f32,
    /// `float dietime`
    pub dietime: f32,
    /// `int num`
    pub num: i32,
    /// `int num_dmcache`
    pub num_dmcache: i32,
    /// `float fuv[4]`
    pub fuv: [f32; 4],
    /// `float foffset`
    pub foffset: f32,
    /// `float size`
    pub size: f32,
    /// `float sphdensity`
    pub sphdensity: f32,
    /// `int hair_index`
    pub hair_index: i32,
    /// `eParticle_Flag flag`
    pub flag: ParticleFlag,
    /// `eParticle_AliveState alive`
    pub alive: ParticleAlivestate,
}
impl Default for ParticleData {
    fn default() -> Self {
        Self {
            state: Default::default(),
            prev_state: Default::default(),
            hair: NIL,
            keys: NIL,
            boid: NIL,
            totkey: 0,
            time: 0.0,
            lifetime: 0.0,
            dietime: 0.0,
            num: 0,
            num_dmcache: 0,
            fuv: [0.0; 4],
            foffset: 0.0,
            size: 0.0,
            sphdensity: 0.0,
            hair_index: 0,
            flag: Default::default(),
            alive: Default::default(),
        }
    }
}

/// `SPHFluidSettings`.
#[derive(Clone)]
pub struct SPHFluidSettings {
    /// `float radius`
    pub radius: f32,
    /// `float spring_k`
    pub spring_k: f32,
    /// `float rest_length`
    pub rest_length: f32,
    /// `float plasticity_constant`
    pub plasticity_constant: f32,
    /// `float yield_ratio`
    pub yield_ratio: f32,
    /// `float plasticity_balance`
    pub plasticity_balance: f32,
    /// `float yield_balance`
    pub yield_balance: f32,
    /// `float viscosity_omega`
    pub viscosity_omega: f32,
    /// `float viscosity_beta`
    pub viscosity_beta: f32,
    /// `float stiffness_k`
    pub stiffness_k: f32,
    /// `float stiffness_knear`
    pub stiffness_knear: f32,
    /// `float rest_density`
    pub rest_density: f32,
    /// `float buoyancy`
    pub buoyancy: f32,
    /// `eSPH_Flag flag`
    pub flag: SphFlag,
    /// `int spring_frames`
    pub spring_frames: i32,
    /// `eSPH_Solver solver`
    pub solver: SphSolver,
}
impl Default for SPHFluidSettings {
    fn default() -> Self {
        Self {
            radius: 0.0,
            spring_k: 0.0,
            rest_length: 0.0,
            plasticity_constant: 0.0,
            yield_ratio: 0.0,
            plasticity_balance: 0.0,
            yield_balance: 0.0,
            viscosity_omega: 0.0,
            viscosity_beta: 0.0,
            stiffness_k: 0.0,
            stiffness_knear: 0.0,
            rest_density: 0.0,
            buoyancy: 0.0,
            flag: Default::default(),
            spring_frames: 0,
            solver: Default::default(),
        }
    }
}

/// `ParticleSettings`.
#[derive(Clone)]
pub struct ParticleSettings {
    pub id: ID,
    /// `AnimData *adt`
    pub adt: usize,
    /// `BoidSettings *boids`
    pub boids: usize,
    /// `SPHFluidSettings *fluid`
    pub fluid: usize,
    /// `EffectorWeights *effector_weights`
    pub effector_weights: usize,
    /// `Collection *collision_group`
    pub collision_group: usize,
    /// `eParticleFlag flag`
    pub flag: Particleflag,
    /// `eParticleType type`
    pub type_: Particletype,
    /// `eParticleFrom from`
    pub from: Particlefrom,
    /// `eParticleDistribution distr`
    pub distr: Particledistribution,
    /// `short texact`
    pub texact: i16,
    /// `eParticlePhysicsType phystype`
    pub phystype: Particlephysicstype,
    /// `eParticleRotationMode rotmode`
    pub rotmode: Particlerotationmode,
    /// `eParticleAngularVelocityMode avemode`
    pub avemode: Particleangularvelocitymode,
    /// `eParticleReactEvent reactevent`
    pub reactevent: Particlereactevent,
    /// `eParticleDrawFlag draw`
    pub draw: Particledrawflag,
    /// `float draw_size`
    pub draw_size: f32,
    /// `eParticleDrawAs draw_as`
    pub draw_as: Particledrawas,
    /// `eParticleChildType childtype`
    pub childtype: Particlechildtype,
    /// `eParticleDrawAs ren_as`
    pub ren_as: Particledrawas,
    /// `short subframes`
    pub subframes: i16,
    /// `eParticleDrawCol draw_col`
    pub draw_col: Particledrawcol,
    /// `short draw_step`
    pub draw_step: i16,
    /// `short ren_step`
    pub ren_step: i16,
    /// `short hair_step`
    pub hair_step: i16,
    /// `short keys_step`
    pub keys_step: i16,
    /// `short adapt_angle`
    pub adapt_angle: i16,
    /// `short adapt_pix`
    pub adapt_pix: i16,
    /// `short disp`
    pub disp: i16,
    /// `short omat`
    pub omat: i16,
    /// `short interpolation`
    pub interpolation: i16,
    /// `eParticleIntegrator integrator`
    pub integrator: Particleintegrator,
    /// `short rotfrom`
    pub rotfrom: i16,
    /// `eParticleKink kink`
    pub kink: Particlekink,
    /// `short kink_axis`
    pub kink_axis: i16,
    /// `short bb_align`
    pub bb_align: i16,
    /// `short bb_uv_split`
    pub bb_uv_split: i16,
    /// `short bb_anim`
    pub bb_anim: i16,
    /// `short bb_split_offset`
    pub bb_split_offset: i16,
    /// `float bb_tilt`
    pub bb_tilt: f32,
    /// `float bb_rand_tilt`
    pub bb_rand_tilt: f32,
    /// `float bb_offset[2]`
    pub bb_offset: [f32; 2],
    /// `float bb_size[2]`
    pub bb_size: [f32; 2],
    /// `float bb_vel_head`
    pub bb_vel_head: f32,
    /// `float bb_vel_tail`
    pub bb_vel_tail: f32,
    /// `float color_vec_max`
    pub color_vec_max: f32,
    /// `float sta`
    pub sta: f32,
    /// `float end`
    pub end: f32,
    /// `float lifetime`
    pub lifetime: f32,
    /// `float randlife`
    pub randlife: f32,
    /// `float timetweak`
    pub timetweak: f32,
    /// `float courant_target`
    pub courant_target: f32,
    /// `float jitfac`
    pub jitfac: f32,
    /// `float eff_hair`
    pub eff_hair: f32,
    /// `float grid_rand`
    pub grid_rand: f32,
    /// `float ps_offset[1]`
    pub ps_offset: [f32; 1],
    /// `int totpart`
    pub totpart: i32,
    /// `int userjit`
    pub userjit: i32,
    /// `int grid_res`
    pub grid_res: i32,
    /// `int effector_amount`
    pub effector_amount: i32,
    /// `eParticleTimeFlag time_flag`
    pub time_flag: Particletimeflag,
    /// `float ob_vel[3]`
    pub ob_vel: [f32; 3],
    /// `float avefac`
    pub avefac: f32,
    /// `float phasefac`
    pub phasefac: f32,
    /// `float randrotfac`
    pub randrotfac: f32,
    /// `float randphasefac`
    pub randphasefac: f32,
    /// `float mass`
    pub mass: f32,
    /// `float size`
    pub size: f32,
    /// `float randsize`
    pub randsize: f32,
    /// `float acc[3]`
    pub acc: [f32; 3],
    /// `float dragfac`
    pub dragfac: f32,
    /// `float brownfac`
    pub brownfac: f32,
    /// `float dampfac`
    pub dampfac: f32,
    /// `float randlength`
    pub randlength: f32,
    /// `eParticleChildFlag child_flag`
    pub child_flag: Particlechildflag,
    /// `int child_percent`
    pub child_percent: i32,
    /// `int child_render_percent`
    pub child_render_percent: i32,
    /// `float parents`
    pub parents: f32,
    /// `float childsize`
    pub childsize: f32,
    /// `float childrandsize`
    pub childrandsize: f32,
    /// `float childrad`
    pub childrad: f32,
    /// `float childflat`
    pub childflat: f32,
    /// `float clumpfac`
    pub clumpfac: f32,
    /// `float clumppow`
    pub clumppow: f32,
    /// `float kink_amp`
    pub kink_amp: f32,
    /// `float kink_freq`
    pub kink_freq: f32,
    /// `float kink_shape`
    pub kink_shape: f32,
    /// `float kink_flat`
    pub kink_flat: f32,
    /// `float kink_amp_clump`
    pub kink_amp_clump: f32,
    /// `int kink_extra_steps`
    pub kink_extra_steps: i32,
    /// `float kink_axis_random`
    pub kink_axis_random: f32,
    /// `float kink_amp_random`
    pub kink_amp_random: f32,
    /// `float rough1`
    pub rough1: f32,
    /// `float rough1_size`
    pub rough1_size: f32,
    /// `float rough2`
    pub rough2: f32,
    /// `float rough2_size`
    pub rough2_size: f32,
    /// `float rough2_thres`
    pub rough2_thres: f32,
    /// `float rough_end`
    pub rough_end: f32,
    /// `float rough_end_shape`
    pub rough_end_shape: f32,
    /// `float clength`
    pub clength: f32,
    /// `float clength_thres`
    pub clength_thres: f32,
    /// `float parting_fac`
    pub parting_fac: f32,
    /// `float parting_min`
    pub parting_min: f32,
    /// `float parting_max`
    pub parting_max: f32,
    /// `float branch_thres`
    pub branch_thres: f32,
    /// `float path_start`
    pub path_start: f32,
    /// `float path_end`
    pub path_end: f32,
    /// `int trail_count`
    pub trail_count: i32,
    /// `int keyed_loops`
    pub keyed_loops: i32,
    /// `CurveMapping *clumpcurve`
    pub clumpcurve: usize,
    /// `CurveMapping *roughcurve`
    pub roughcurve: usize,
    /// `float clump_noise_size`
    pub clump_noise_size: f32,
    /// `float bending_random`
    pub bending_random: f32,
    /// `MTex *mtex[]`
    pub mtex: [usize; 18],
    /// `Collection *instance_collection`
    pub instance_collection: usize,
    /// `ListBaseT<ParticleDupliWeight>`
    pub instance_weights: ListBase,
    /// `Collection *force_group`
    pub force_group: usize,
    /// `Object *instance_object`
    pub instance_object: usize,
    /// `Object *bb_ob`
    pub bb_ob: usize,
    /// `PartDeflect *pd`
    pub pd: usize,
    /// `PartDeflect *pd2`
    pub pd2: usize,
    /// `short use_modifier_stack`
    pub use_modifier_stack: i16,
    /// `eParticleShapeFlag shape_flag`
    pub shape_flag: Particleshapeflag,
    /// `float twist`
    pub twist: f32,
    /// `float shape`
    pub shape: f32,
    /// `float rad_root`
    pub rad_root: f32,
    /// `float rad_tip`
    pub rad_tip: f32,
    /// `float rad_scale`
    pub rad_scale: f32,
    /// `CurveMapping *twistcurve`
    pub twistcurve: usize,
}
impl Default for ParticleSettings {
    fn default() -> Self {
        Self {
            id: Default::default(),
            adt: NIL,
            boids: NIL,
            fluid: NIL,
            effector_weights: NIL,
            collision_group: NIL,
            flag: PART_EDISTR | PART_TRAND | PART_HIDE_ADVANCED_HAIR,
            type_: PART_EMITTER,
            from: PART_FROM_FACE,
            distr: PART_DISTR_JIT,
            texact: 0,
            phystype: PART_PHYS_NEWTON,
            rotmode: PART_ROT_VEL,
            avemode: PART_AVE_VELOCITY,
            reactevent: PART_EVENT_DEATH,
            draw: Default::default(),
            draw_size: 0.1,
            draw_as: PART_DRAW_REND,
            childtype: Default::default(),
            ren_as: PART_DRAW_HALO,
            subframes: 0,
            draw_col: PART_DRAW_COL_MAT,
            draw_step: 2,
            ren_step: 3,
            hair_step: 5,
            keys_step: 5,
            adapt_angle: 5,
            adapt_pix: 3,
            disp: 100,
            omat: 1,
            interpolation: 0,
            integrator: PART_INT_MIDPOINT,
            rotfrom: 0,
            kink: PART_KINK_NO,
            kink_axis: 2,
            bb_align: 0,
            bb_uv_split: 1,
            bb_anim: 0,
            bb_split_offset: 0,
            bb_tilt: 0.0,
            bb_rand_tilt: 0.0,
            bb_offset: [0.0; 2],
            bb_size: [0.0; 2],
            bb_vel_head: 0.0,
            bb_vel_tail: 0.0,
            color_vec_max: 1.0,
            sta: 1.0,
            end: 200.0,
            lifetime: 50.0,
            randlife: 0.0,
            timetweak: 1.0,
            courant_target: 0.2,
            jitfac: 1.0,
            eff_hair: 0.0,
            grid_rand: 0.0,
            ps_offset: [0.0; 1],
            totpart: 1000,
            userjit: 0,
            grid_res: 10,
            effector_amount: 0,
            time_flag: Default::default(),
            ob_vel: [0.0; 3],
            avefac: 0.0,
            phasefac: 0.0,
            randrotfac: 0.0,
            randphasefac: 0.0,
            mass: 1.0,
            size: 0.05,
            randsize: 0.0,
            acc: [0.0; 3],
            dragfac: 0.0,
            brownfac: 0.0,
            dampfac: 0.0,
            randlength: 0.0,
            child_flag: Default::default(),
            child_percent: 10,
            child_render_percent: 100,
            parents: 0.0,
            childsize: 1.0,
            childrandsize: 0.0,
            childrad: 0.2,
            childflat: 0.0,
            clumpfac: 0.0,
            clumppow: 0.0,
            kink_amp: 0.2,
            kink_freq: 2.0,
            kink_shape: 0.0,
            kink_flat: 0.0,
            kink_amp_clump: 1.0,
            kink_extra_steps: 4,
            kink_axis_random: 0.0,
            kink_amp_random: 0.0,
            rough1: 0.0,
            rough1_size: 1.0,
            rough2: 0.0,
            rough2_size: 1.0,
            rough2_thres: 0.0,
            rough_end: 0.0,
            rough_end_shape: 1.0,
            clength: 1.0,
            clength_thres: 0.0,
            parting_fac: 0.0,
            parting_min: 0.0,
            parting_max: 0.0,
            branch_thres: 0.0,
            path_start: 0.0,
            path_end: 1.0,
            trail_count: 0,
            keyed_loops: 1,
            clumpcurve: NIL,
            roughcurve: NIL,
            clump_noise_size: 1.0,
            bending_random: 0.0,
            mtex: [NIL; 18],
            instance_collection: NIL,
            instance_weights: ListBase::default(),
            force_group: NIL,
            instance_object: NIL,
            bb_ob: NIL,
            pd: NIL,
            pd2: NIL,
            use_modifier_stack: 0,
            shape_flag: PART_SHAPE_CLOSE_TIP,
            twist: 0.0,
            shape: 0.0,
            rad_root: 1.0,
            rad_tip: 0.0,
            rad_scale: 0.01,
            twistcurve: NIL,
        }
    }
}

/// `ParticleSystem`.
#[derive(Clone)]
pub struct ParticleSystem {
    /// `ParticleSystem *next`
    pub next: usize,
    /// `ParticleSystem *prev`
    pub prev: usize,
    /// `ParticleSettings *part`
    pub part: usize,
    /// `ParticleData *particles`
    pub particles: usize,
    /// `ChildParticle *child`
    pub child: usize,
    /// `PTCacheEdit *edit`
    pub edit: usize,
    /// `ParticleCacheKey *pathcache`
    pub pathcache: usize,
    /// `ParticleCacheKey *childcache`
    pub childcache: usize,
    /// `ListBaseT<LinkData>`
    pub pathcachebufs: ListBase,
    /// `ClothModifierData *clmd`
    pub clmd: usize,
    /// `Mesh *hair_in_mesh`
    pub hair_in_mesh: usize,
    /// `Mesh *hair_out_mesh`
    pub hair_out_mesh: usize,
    /// `Object *target_ob`
    pub target_ob: usize,
    /// `LatticeDeformData *lattice_deform_data`
    pub lattice_deform_data: usize,
    /// `Object *parent`
    pub parent: usize,
    /// `ListBaseT<ParticleTarget>`
    pub targets: ListBase,
    /// `char name[64]`
    pub name: [u8; 64],
    /// `float imat[4][4]`
    pub imat: [[f32; 4]; 4],
    /// `float cfra`
    pub cfra: f32,
    /// `float tree_frame`
    pub tree_frame: f32,
    /// `float bvhtree_frame`
    pub bvhtree_frame: f32,
    /// `int seed`
    pub seed: i32,
    /// `int child_seed`
    pub child_seed: i32,
    /// `eParticleSystem_Flag flag`
    pub flag: ParticlesystemFlag,
    /// `int totpart`
    pub totpart: i32,
    /// `int totunexist`
    pub totunexist: i32,
    /// `int totchild`
    pub totchild: i32,
    /// `int totcached`
    pub totcached: i32,
    /// `int totchildcache`
    pub totchildcache: i32,
    /// `int recalc`
    pub recalc: i32,
    /// `short target_psys`
    pub target_psys: i16,
    /// `short totkeyed`
    pub totkeyed: i16,
    /// `short bakespace`
    pub bakespace: i16,
    /// `char bb_uvname[3][68]`
    pub bb_uvname: [[u8; 68]; 3],
    /// `short vgroup[13]`
    pub vgroup: [i16; 13],
    /// `short vg_neg`
    pub vg_neg: i16,
    /// `short rt3`
    pub rt3: i16,
    /// `PointCache *pointcache`
    pub pointcache: usize,
    /// `ListBaseT<PointCache>`
    pub ptcaches: ListBase,
    /// `ParticleSpring *fluid_springs`
    pub fluid_springs: usize,
    /// `int tot_fluidsprings`
    pub tot_fluidsprings: i32,
    /// `int alloc_fluidsprings`
    pub alloc_fluidsprings: i32,
    /// `bke::ParticleKDTree *tree`
    pub tree: usize,
    /// `BVHTree *bvhtree`
    pub bvhtree: usize,
    /// `ParticleDrawData *pdd`
    pub pdd: usize,
    /// `float dt_frac`
    pub dt_frac: f32,
    /// `float lattice_strength`
    pub lattice_strength: f32,
    /// `draw::ParticleBatchCache *batch_cache`
    pub batch_cache: usize,
    /// `ParticleSystem *orig_psys`
    pub orig_psys: usize,
}
impl Default for ParticleSystem {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            part: NIL,
            particles: NIL,
            child: NIL,
            edit: NIL,
            pathcache: NIL,
            childcache: NIL,
            pathcachebufs: ListBase::default(),
            clmd: NIL,
            hair_in_mesh: NIL,
            hair_out_mesh: NIL,
            target_ob: NIL,
            lattice_deform_data: NIL,
            parent: NIL,
            targets: ListBase::default(),
            name: [0; 64],
            imat: [[0.0; 4]; 4],
            cfra: 0.0,
            tree_frame: 0.0,
            bvhtree_frame: 0.0,
            seed: 0,
            child_seed: 0,
            flag: Default::default(),
            totpart: 0,
            totunexist: 0,
            totchild: 0,
            totcached: 0,
            totchildcache: 0,
            recalc: 0,
            target_psys: 0,
            totkeyed: 0,
            bakespace: 0,
            bb_uvname: [[0; 68]; 3],
            vgroup: [0; 13],
            vg_neg: 0,
            rt3: 0,
            pointcache: NIL,
            ptcaches: ListBase::default(),
            fluid_springs: NIL,
            tot_fluidsprings: 0,
            alloc_fluidsprings: 0,
            tree: NIL,
            bvhtree: NIL,
            pdd: NIL,
            dt_frac: 0.0,
            lattice_strength: 0.0,
            batch_cache: NIL,
            orig_psys: NIL,
        }
    }
}
