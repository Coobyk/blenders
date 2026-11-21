//! Port of `intern/particle.cc` pure parts — particle
//! settings curve initialization (clump/rough/twist),
//! `PART_*` flag constants.
//!
//! `PORT_PENDING`: full particle simulation
//! (`psys_get_particle_on_path` state eval, keyed/boid
//! physics, effector forces, `do_guides`, hair matrices),
//! `get_particle_uv`/`BKE_particle_co_hair`/`_uv_on_emitter`
//! (need Mesh/`MTFace` legacy data), batch cache, blend IO.

use crate::colortools::{curvemapping_add, curvemapping_init};
use blenders_dna::color_types::CurveMapping;

/// `PART_TYPE_*` verbatim.
pub const PART_TYPE_EMITTER: i32 = 0;
/// `PART_TYPE_HAIR`.
pub const PART_TYPE_HAIR: i32 = 1;
/// `PART_TYPE_REACTOR` (deprecated).
pub const PART_TYPE_REACTOR: i32 = 2;

/// `PART_PHYS_*` verbatim.
pub const PART_PHYS_NO: i32 = 0;
/// `PART_PHYS_NEWTON`.
pub const PART_PHYS_NEWTON: i32 = 1;
/// `PART_PHYS_KEYED`.
pub const PART_PHYS_KEYED: i32 = 2;
/// `PART_PHYS_BOIDS`.
pub const PART_PHYS_BOIDS: i32 = 3;
/// `PART_PHYS_FLUID`.
pub const PART_PHYS_FLUID: i32 = 4;

/// `PART_DRAW_*` verbatim.
pub const PART_DRAW_VEL: i32 = 1 << 0;
/// `PART_DRAW_GLOBAL_OB`.
pub const PART_DRAW_GLOBAL_OB: i32 = 1 << 1;
/// `PART_DRAW_SIZE`.
pub const PART_DRAW_SIZE: i32 = 1 << 2;
/// `PART_DRAW_EMITTER` (deprecated).
pub const PART_DRAW_EMITTER: i32 = 1 << 3;
/// `PART_DRAW_HEALTH`.
pub const PART_DRAW_HEALTH: i32 = 1 << 4;
/// `PART_DRAW_COUNT_GR`.
pub const PART_DRAW_COUNT_GR: i32 = 1 << 6;
/// `PART_DRAW_ROTATE_OB`.
pub const PART_DRAW_ROTATE_OB: i32 = 1 << 7;
/// `PART_DRAW_PARENT`.
pub const PART_DRAW_PARENT: i32 = 1 << 8;
/// `PART_DRAW_NUM`.
pub const PART_DRAW_NUM: i32 = 1 << 9;
/// `PART_DRAW_RAND_GR`.
pub const PART_DRAW_RAND_GR: i32 = 1 << 10;
/// `PART_DRAW_REN_ADAPT`.
pub const PART_DRAW_REN_ADAPT: i32 = 1 << 11;
/// `PART_DRAW_VEL_LENGTH`.
pub const PART_DRAW_VEL_LENGTH: i32 = 1 << 12;
/// `PART_DRAW_MAT_COL` (deprecated).
pub const PART_DRAW_MAT_COL: i32 = 1 << 13;
/// `PART_DRAW_WHOLE_GR`.
pub const PART_DRAW_WHOLE_GR: i32 = 1 << 14;
/// `PART_DRAW_REN_STRAND`.
pub const PART_DRAW_REN_STRAND: i32 = 1 << 15;
/// `PART_DRAW_NO_SCALE_OB`.
pub const PART_DRAW_NO_SCALE_OB: i32 = 1 << 16;
/// `PART_DRAW_GUIDE_HAIRS`.
pub const PART_DRAW_GUIDE_HAIRS: i32 = 1 << 17;
/// `PART_DRAW_HAIR_GRID`.
pub const PART_DRAW_HAIR_GRID: i32 = 1 << 18;

/// `PART_CHILD_*` verbatim.
pub const PART_CHILD_PARTICLES: i32 = 1;
/// `PART_CHILD_FACES`.
pub const PART_CHILD_FACES: i32 = 2;

/// `PART_FROM_*` verbatim.
pub const PART_FROM_VERT: i32 = 0;
/// `PART_FROM_FACE`.
pub const PART_FROM_FACE: i32 = 1;
/// `PART_FROM_VOLUME`.
pub const PART_FROM_VOLUME: i32 = 2;
/// `PART_FROM_PARTICLE`.
pub const PART_FROM_PARTICLE: i32 = 3;

/// `PSYS_*` — `ParticleSystem.flag` verbatim (subset).
pub const PSYS_DISABLED: i32 = 1 << 0;
/// `PSYS_KEYED`.
pub const PSYS_KEYED: i32 = 1 << 1;
/// `PSYS_EDITED`.
pub const PSYS_EDITED: i32 = 1 << 2;
/// `PSYS_PROTECTED`.
pub const PSYS_PROTECTED: i32 = 1 << 3;
/// `PSYS_GLOBAL_HAIR`.
pub const PSYS_GLOBAL_HAIR: i32 = 1 << 9;
/// `PSYS_HAIR_UPDATED`.
pub const PSYS_HAIR_UPDATED: i32 = 1 << 10;
/// `PSYS_HAIR_DO_DYNAMICS`.
pub const PSYS_HAIR_DO_DYNAMICS: i32 = 1 << 11;
/// `PSYS_HAIR_SUBD`.
pub const PSYS_HAIR_SUBD: i32 = 1 << 12;
/// `PSYS_RECALC_RESET`.
pub const PSYS_RECALC_RESET: i32 = 1 << 16;

/// `PARS_*` — `ParticleData.flag` verbatim (subset).
pub const PARS_UNEXIST: i32 = 1 << 1;
/// `PARS_UNBORN`.
pub const PARS_UNBORN: i32 = 1 << 2;
/// `PARS_UNDYING`.
pub const PARS_UNDYING: i32 = 1 << 4;
/// `PARS_NO_DISP`.
pub const PARS_NO_DISP: i32 = 1 << 5;
/// `PARS_STICKY`.
pub const PARS_STICKY: i32 = 1 << 6;
/// `PARS_REKEY`.
pub const PARS_REKEY: i32 = 1 << 7;
/// `PARS_DEAD`.
pub const PARS_DEAD: i32 = 1 << 8;

fn flat_linear_curve() -> CurveMapping {
    let mut cumap = curvemapping_add(1, 0.0, 0.0, 1.0, 1.0);
    cumap.cm[0].curve[0].x = 0.0;
    cumap.cm[0].curve[0].y = 1.0;
    cumap.cm[0].curve[1].x = 1.0;
    cumap.cm[0].curve[1].y = 1.0;
    curvemapping_init(&mut cumap);
    cumap
}

/// `BKE_particlesettings_clump_curve_init` (verbatim —
/// caller assigns the handle).
pub fn particlesettings_clump_curve_init() -> CurveMapping {
    flat_linear_curve()
}
/// `BKE_particlesettings_rough_curve_init` (verbatim).
pub fn particlesettings_rough_curve_init() -> CurveMapping {
    flat_linear_curve()
}
/// `BKE_particlesettings_twist_curve_init` (verbatim).
pub fn particlesettings_twist_curve_init() -> CurveMapping {
    flat_linear_curve()
}
