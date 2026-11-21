//! Port of `intern/softbody.cc` — `BodyPoint`/`BodySpring`/
//! `BodyQuad`/`SBScratch` types, goal/mass evaluation,
//! spring-list building, collision-ball sizing, force
//! scaling, `renew_softbody`/free paths.
//!
//! `PORT_PENDING`: `sb_spring_force`/`softbody_calc_forces`/
//! `apply_forces`/`predict_system`/`Heun` integrator (the
//! ODE solve — `EIG_linear_solver_*` injected via trait),
//! `ccd_*` (libccd collision), `sb_detect_*_collision`
//! (depsgraph deflectors), `vertexCos` write-back, `sbNew`/
//! `sbObjectStep`/`SB_estimate_transform` (SVD — math port
//! of weighted rigid-fit pending the svd module).

/// `BodyPoint` verbatim.
#[derive(Clone, Default)]
pub struct BodyPoint {
    /// `origS`, `origE`, `origT` (original start/end/tangent).
    pub origs: [f32; 3],
    /// `origE`.
    pub orige: [f32; 3],
    /// `origT`.
    pub origt: [f32; 3],
    /// `pos`.
    pub pos: [f32; 3],
    /// `vec` (velocity).
    pub vec: [f32; 3],
    /// `force`.
    pub force: [f32; 3],
    /// `goal`.
    pub goal: f32,
    /// `prevpos`, `prevvec`, `prevdx`, `prevdv` (Heun).
    pub prevpos: [f32; 3],
    /// `prevvec`.
    pub prevvec: [f32; 3],
    /// `prevdx`.
    pub prevdx: [f32; 3],
    /// `prevdv`.
    pub prevdv: [f32; 3],
    /// `impdv`, `impdx`.
    pub impdv: [f32; 3],
    /// `impdx`.
    pub impdx: [f32; 3],
    /// `nofsprings` + `springs` (index list).
    pub nofsprings: i32,
    /// `springs`.
    pub springs: Vec<i32>,
    /// `choke`, `choke2`, `frozen`.
    pub choke: f32,
    /// `choke2`.
    pub choke2: f32,
    /// `frozen`.
    pub frozen: f32,
    /// `colball`.
    pub colball: f32,
    /// `loc_flag`.
    pub loc_flag: i16,
    /// `mass`.
    pub mass: f32,
    /// `springweight`.
    pub springweight: f32,
}

/// `BodySpring` verbatim.
#[derive(Clone, Default)]
pub struct BodySpring {
    /// `v1`, `v2` — endpoint point indices.
    pub v1: i32,
    /// `v2`.
    pub v2: i32,
    /// `len` (rest length).
    pub len: f32,
    /// `springtype` (`SB_EDGE`/`SB_BEND`/`SB_STIFFQUAD`).
    pub springtype: i16,
    /// `flag` (`BSF_INTERSECT`).
    pub flag: i16,
    /// `order` (spring order).
    pub order: i16,
}

/// `BodyFace` verbatim.
#[derive(Clone, Default)]
pub struct BodyFace {
    /// `v1`, `v2`, `v3`.
    pub v1: i32,
    /// `v2`.
    pub v2: i32,
    /// `v3`.
    pub v3: i32,
    /// `ext_force` (faces colliding).
    pub ext_force: [f32; 3],
    /// `flag` (`BFF_*`).
    pub flag: i16,
}

/// `ReferenceVert` verbatim.
#[derive(Clone, Default)]
pub struct ReferenceVert {
    /// `pos`.
    pub pos: [f32; 3],
    /// `mass`.
    pub mass: f32,
}

/// `ReferenceState` verbatim.
#[derive(Default)]
pub struct ReferenceState {
    /// `com` — center of mass.
    pub com: [f32; 3],
    /// `ivert`.
    pub ivert: Vec<ReferenceVert>,
}

/// `SBScratch` verbatim.
#[derive(Default)]
pub struct SbScratch {
    /// `colliderhash` — `Map<Object*, ccd_Mesh*>`.
    pub colliderhash: Vec<(usize, usize)>,
    /// `needstobuildcollider`.
    pub needstobuildcollider: i16,
    /// `flag`.
    pub flag: i16,
    /// `bodyface`.
    pub bodyface: Vec<BodyFace>,
    /// `bodyface_num`.
    pub bodyface_num: i32,
    /// `aabbmin`/`aabbmax`.
    pub aabbmin: [f32; 3],
    /// `aabbmax`.
    pub aabbmax: [f32; 3],
    /// `Ref`.
    pub ref_: ReferenceState,
}

/// `MID_PRESERVE`/`SOFTGOALSNAP`/`HEUNWARNLIMIT`/`BSF_*`/
/// `SBF_*`/`BFF_*`/`CCD_SAFETY` verbatim.
pub const MID_PRESERVE: i32 = 1;
/// `SOFTGOALSNAP`.
pub const SOFTGOALSNAP: f32 = 0.999;
/// `HEUNWARNLIMIT`.
pub const HEUNWARNLIMIT: i32 = 1;
/// `BSF_INTERSECT`.
pub const BSF_INTERSECT: i16 = 1;
/// `SBF_DOFUZZY`.
pub const SBF_DOFUZZY: i16 = 1;
/// `SBF_OUTOFCOLLISION`.
pub const SBF_OUTOFCOLLISION: i16 = 2;
/// `BFF_INTERSECT`.
pub const BFF_INTERSECT: i16 = 1;
/// `BFF_CLOSEVERT`.
pub const BFF_CLOSEVERT: i16 = 2;
/// `CCD_SAFETY`.
pub const CCD_SAFETY: i32 = 190561;

/// `SB_EDGE`/`SB_BEND`/`SB_STIFFQUAD` spring types.
pub const SB_EDGE: i16 = 0;
/// `SB_BEND`.
pub const SB_BEND: i16 = 1;
/// `SB_STIFFQUAD`.
pub const SB_STIFFQUAD: i16 = 2;
/// `SB_BEND_QUAD`.
pub const SB_BEND_QUAD: i16 = 3;

/// `SBC_MODE_MANUAL`/`AVG`/`MIN`/`MAX`/`AVGMINMAX`.
pub const SBC_MODE_MANUAL: i16 = 0;
/// `SBC_MODE_AVG`.
pub const SBC_MODE_AVG: i16 = 1;
/// `SBC_MODE_MIN`.
pub const SBC_MODE_MIN: i16 = 2;
/// `SBC_MODE_MAX`.
pub const SBC_MODE_MAX: i16 = 3;
/// `SBC_MODE_AVGMINMAX`.
pub const SBC_MODE_AVGMINMAX: i16 = 4;

/// `OB_SB_GOAL` (softflag).
pub const OB_SB_GOAL: i16 = 1 << 0;

/// `SoftHeunTol` (verbatim default).
pub const SOFT_HEUN_TOL: f32 = 1.0;

/// `sb_grav_force_scale` (verbatim).
pub fn sb_grav_force_scale() -> f32 {
    0.001
}
/// `sb_fric_force_scale` (verbatim).
pub fn sb_fric_force_scale() -> f32 {
    0.01
}
/// `sb_time_scale` — `sb->physics_speed` or 1 (verbatim).
pub fn sb_time_scale(physics_speed: Option<f32>) -> f32 {
    physics_speed.unwrap_or(1.0)
}

/// `_final_goal` (verbatim — `(mingoal + goal*|maxgoal-
/// mingoal|)^4` when `OB_SB_GOAL` set).
pub fn final_goal(softflag: i16, bp_goal: f32, mingoal: f32, maxgoal: f32) -> f32 {
    if (softflag & OB_SB_GOAL) == 0 {
        return 0.0;
    }
    if bp_goal < 0.0 {
        return 0.0;
    }
    let f = mingoal + bp_goal * (maxgoal - mingoal).abs();
    f.powf(4.0)
}

/// `_final_mass` (verbatim).
pub fn final_mass(bp_mass: f32, nodemass: f32) -> f32 {
    bp_mass * nodemass
}

/// `add_bp_springlist` — append `springID` (verbatim).
pub fn add_bp_springlist(bp: &mut BodyPoint, springid: i32) {
    bp.springs.push(springid);
    bp.nofsprings = bp.springs.len() as i32;
}

/// `build_bps_springlist` (verbatim O(N²) scan).
pub fn build_bps_springlist(bpoints: &mut [BodyPoint], bsprings: &[BodySpring]) {
    for (i, bp) in bpoints.iter_mut().enumerate() {
        bp.springs.clear();
        for (j, bs) in bsprings.iter().enumerate() {
            if i as i32 == bs.v1 || i as i32 == bs.v2 {
                bp.springs.push(j as i32);
            }
        }
        bp.nofsprings = bp.springs.len() as i32;
    }
}

/// `calculate_collision_balls` (verbatim — per-point colball
/// from attached edge springs + `sbc_mode`).
pub fn calculate_collision_balls(
    bpoints: &mut [BodyPoint],
    bsprings: &[BodySpring],
    sbc_mode: i16,
    colball: f32,
) {
    for bp in bpoints.iter_mut() {
        bp.colball = 0.0;
        let mut akku = 0.0f32;
        let mut akku_count = 0i32;
        let mut min = 1e22f32;
        let mut max = -1e22f32;
        for &sid in bp.springs.iter() {
            let bs = &bsprings[sid as usize];
            if bs.springtype == SB_EDGE {
                akku += bs.len;
                akku_count += 1;
                min = bs.len.min(min);
                max = bs.len.max(max);
            }
        }
        if akku_count > 0 {
            match sbc_mode {
                SBC_MODE_MANUAL => bp.colball = colball,
                SBC_MODE_AVG => bp.colball = akku / akku_count as f32 * colball,
                SBC_MODE_MIN => bp.colball = min * colball,
                SBC_MODE_MAX => bp.colball = max * colball,
                SBC_MODE_AVGMINMAX => bp.colball = (min + max) / 2.0 * colball,
                _ => {}
            }
        }
    }
}

/// `renew_softbody` — size the bpoint/bspring arrays and
/// reset point fields (verbatim init values).
pub fn renew_softbody(totpoint: usize, totspring: usize, softflag: i16, defgoal: f32) -> (Vec<BodyPoint>, Vec<BodySpring>) {
    let bpoints = (0..totpoint)
        .map(|_| {
            let mut bp = BodyPoint::default();
            bp.goal = if (softflag & OB_SB_GOAL) != 0 {
                defgoal
            } else {
                0.0
            };
            bp.frozen = 1.0;
            bp.springweight = 1.0;
            bp.mass = 1.0;
            bp
        })
        .collect();
    (bpoints, vec![BodySpring::default(); totspring])
}

/// `dfdx_spring` — 3×3 stiffness block via `matrix_add`
/// (verbatim, `EIG_linear_solver_matrix_add` injected).
pub fn dfdx_spring(
    ia: usize,
    ic: usize,
    op: usize,
    dir: [f32; 3],
    l: f32,
    len: f32,
    factor: f32,
    matrix_add: &mut dyn FnMut(usize, usize, f32),
) {
    if l < len {
        for i in 0..3 {
            for j in 0..3 {
                let delta_ij = if i == j { 1.0 } else { 0.0 };
                let m =
                    factor * (dir[i] * dir[j] + (1.0 - l / len) * (delta_ij - dir[i] * dir[j]));
                matrix_add(ia + i, op + ic + j, m);
            }
        }
    } else {
        for i in 0..3 {
            for j in 0..3 {
                let m = factor * dir[i] * dir[j];
                matrix_add(ia + i, op + ic + j, m);
            }
        }
    }
}
/// `dfdx_goal` (verbatim).
pub fn dfdx_goal(ia: usize, ic: usize, op: usize, factor: f32, matrix_add: &mut dyn FnMut(usize, usize, f32)) {
    for i in 0..3 {
        matrix_add(ia + i, op + ic + i, factor);
    }
}
/// `dfdv_goal` (verbatim).
pub fn dfdv_goal(ia: usize, ic: usize, factor: f32, matrix_add: &mut dyn FnMut(usize, usize, f32)) {
    for i in 0..3 {
        matrix_add(ia + i, ic + i, factor);
    }
}
