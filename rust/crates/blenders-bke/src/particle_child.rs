//! Port of `intern/particle_child.cc` — child-particle path
//! modifiers: kink (curl/radial/wave/braid/spiral), clump (curve /
//! noise / factor), roughness (turbulence/end/curve), twist, and
//! `psys_apply_child_modifiers`.
//!
//! Injected: `psys_frand`/`psys_frand_vec` (per-psys RNG),
//! `curvemapping_evaluate`, `do_guides`, `object_to_world` matrix,
//! and material color — all supplied as a `ChildEnv` of closures.

use std::f32::consts::PI;

use blenders_bli::noise_c::{noise_generic_turbulence, noise_voronoi};
use blenders_dna::color_types::CurveMapping;
use blenders_dna::particle_types::{
    ParticleSettings, PART_CHILD_EFFECT, PART_CHILD_FACES, PART_CHILD_PARTICLES,
    PART_CHILD_USE_CLUMP_CURVE, PART_CHILD_USE_CLUMP_NOISE, PART_CHILD_USE_ROUGH_CURVE,
    PART_CHILD_USE_TWIST_CURVE, PART_DRAW_COL_MAT, PART_KINK_BRAID, PART_KINK_CURL,
    PART_KINK_NO, PART_KINK_RADIAL, PART_KINK_SPIRAL, PART_KINK_WAVE,
};
use blenders_math::matrix::{Mat3, Mat4};
use blenders_math::quaternion::{
    axis_angle_normalized_to_mat3, axis_angle_to_quat, interp_qt, Quaternion,
};
use blenders_math::vector::{project_v3, Vec3};

/// `ParticleCacheKey`.
#[derive(Clone)]
pub struct ParticleCacheKey {
    /// `float co[3]`.
    pub co: Vec3,
    /// `float vel[3]`.
    pub vel: Vec3,
    /// `float rot[4]`.
    pub rot: Quaternion,
    /// `float col[3]`.
    pub col: Vec3,
    /// `float time`.
    pub time: f32,
    /// `int segments`.
    pub segments: i32,
}

impl Default for ParticleCacheKey {
    fn default() -> Self {
        Self {
            co: Vec3::ZERO,
            vel: Vec3::ZERO,
            rot: Quaternion::IDENTITY,
            col: Vec3::ZERO,
            time: 0.0,
            segments: 0,
        }
    }
}

/// `ParticleTexture`.
#[derive(Clone, Default)]
pub struct ParticleTexture {
    /// `float length`.
    pub length: f32,
    /// `float clump`.
    pub clump: f32,
    /// `float kink_freq`.
    pub kink_freq: f32,
    /// `float kink_amp`.
    pub kink_amp: f32,
    /// `float rough1`.
    pub rough1: f32,
    /// `float rough2`.
    pub rough2: f32,
    /// `float roughe`.
    pub roughe: f32,
    /// `float twist`.
    pub twist: f32,
}

/// External services the C file reaches into (`psys`, `sim`, `ctx`,
/// curve maps, effectors, material, `ob->object_to_world`).
pub struct ChildEnv<'a> {
    /// `psys->part`.
    pub part: &'a ParticleSettings,
    /// `psys_frand(psys, n)` — deterministic per-seed [0,1).
    pub frand: &'a dyn Fn(i32) -> f32,
    /// `psys_frand_vec(psys, n, vec)` — deterministic [0,1) per comp.
    pub frand_vec: &'a dyn Fn(i32) -> Vec3,
    /// `BKE_curvemapping_evaluateF(curve, comp, x)`.
    pub curve_eval: &'a dyn Fn(&CurveMapping, i32, f32) -> f32,
    /// `ctx->clumpcurve` / `part->clumpcurve` resolver.
    pub clumpcurve: Option<&'a CurveMapping>,
    /// `ctx->roughcurve` / `part->roughcurve`.
    pub roughcurve: Option<&'a CurveMapping>,
    /// `ctx->twistcurve` / `part->twistcurve`.
    pub twistcurve: Option<&'a CurveMapping>,
    /// `sim->ob->object_to_world()`.
    pub object_to_world: Mat4,
    /// `do_guides(...)` → nonzero when a guide handled the point.
    /// Args: (cpa_parent, time) → guided.
    pub do_guides: &'a dyn Fn(i32, f32) -> i32,
    /// `ctx->segments` (None → `part->draw_step` for twist segments).
    pub segments: i32,
    /// `ctx->extra_segments` (spiral kink).
    pub extra_segments: i32,
    /// `ma` color `[r,g,b]` when `draw_col == PART_DRAW_COL_MAT`.
    pub material_color: Option<Vec3>,
    /// `psys->child_seed`.
    pub child_seed: i32,
}

/// `ParticleChildModifierContext`.
pub struct ParticleChildModifierContext<'a> {
    /// `env` — sim/thread/texture services.
    pub env: &'a ChildEnv<'a>,
    /// `ptex` (None = defaults of 1.0 clump etc.).
    pub ptex: Option<ParticleTexture>,
    /// `cpa` — child index + `parent` field used.
    pub cpa_index: i32,
    /// `cpa->parent`.
    pub cpa_parent: i32,
    /// `orco`.
    pub orco: Vec3,
    /// `parent_keys`.
    pub parent_keys: Option<&'a [ParticleCacheKey]>,
    /// `par_co` (current parent point).
    pub par_co: Vec3,
    /// `par_vel`.
    pub par_vel: Vec3,
    /// `par_rot`.
    pub par_rot: Quaternion,
    /// `par_orco`.
    pub par_orco: Vec3,
}

/// `ParticlePathIterator` state — folded into
/// [`psys_path_iter_get`].
#[derive(Clone, Copy)]
pub struct ParticlePathIter {
    /// `index`.
    pub index: usize,
    /// `time`.
    pub time: f32,
    /// `parent_rotation`.
    pub parent_rotation: Quaternion,
}

/// `psys_path_iter_get` — returns (key index, time, parent
/// rotation). Parent key at same index drives the rotation chain.
pub fn psys_path_iter_get(
    keys: &[ParticleCacheKey],
    parent: Option<&[ParticleCacheKey]>,
    index: usize,
) -> ParticlePathIter {
    let totkeys = keys.len();
    debug_assert!(index < totkeys);
    let (time, parent_rotation) = if let Some(p) = parent {
        if index > 0 {
            (index as f32 / (totkeys - 1) as f32, p[index].rot * p[0].rot)
        } else {
            (
                index as f32 / (totkeys - 1) as f32,
                p[0].rot,
            )
        }
    } else {
        (index as f32 / (totkeys - 1) as f32, Quaternion::IDENTITY)
    };
    ParticlePathIter {
        index,
        time,
        parent_rotation,
    }
}

fn do_kink_spiral_deform(
    state: &mut ParticleCacheKey,
    dir: Vec3,
    kink: Vec3,
    mut time: f32,
    freq: f32,
    shape: f32,
    amplitude: f32,
    spiral_start: Vec3,
) {
    time = time.clamp(0.0, 1.0);
    let b = shape * (1.0 + 5.0f32.sqrt()) / PI * 0.25;
    let start_angle = if b != 0.0 {
        (1.0 / b).atan()
    } else {
        -PI / 2.0
    } + if b > 0.0 { -PI / 2.0 } else { PI / 2.0 };

    let mut theta = freq * time * 2.0 * PI;
    let radius = amplitude * (b * theta).exp();
    if amplitude < 0.0 {
        theta = -theta;
    }
    let spiral_axis = dir.cross(kink).normalized();
    let mut vec = kink * -radius;
    vec = axis_angle_normalized_to_mat3(spiral_axis, theta) * vec;
    vec += kink * amplitude;
    vec = axis_angle_normalized_to_mat3(spiral_axis, -start_angle) * vec;
    state.co = spiral_start + vec;
}

#[allow(clippy::too_many_arguments)]
fn do_kink_spiral(
    env: &ChildEnv,
    ptex: &ParticleTexture,
    parent_orco: Vec3,
    cpa_index: i32,
    orco: Vec3,
    hairmat: Mat4,
    keys: &mut [ParticleCacheKey],
    parent_keys: &[ParticleCacheKey],
    r_max_length: &mut f32,
) {
    let part = env.part;
    let seed = env.child_seed + cpa_index;
    let totkeys = env.segments + 1;
    let extrakeys = env.extra_segments;

    let kink_amp_random = part.kink_amp_random;
    let mut kink_amp =
        part.kink_amp * (1.0 - kink_amp_random * (env.frand)(93541 + seed));
    let mut kink_freq = part.kink_freq;
    let kink_shape = part.kink_shape;
    let kink_axis_random = part.kink_axis_random;

    let mut spiral_start = Vec3::ZERO;
    let mut spiral_start_time = 0.0f32;
    let mut spiral_par_co = Vec3::ZERO;
    let mut spiral_par_vel = Vec3::ZERO;
    let mut spiral_par_rot = Quaternion::IDENTITY;
    let mut start_index = 0usize;
    let mut end_index = 0usize;

    kink_amp *= ptex.kink_amp;
    kink_freq *= ptex.kink_freq;

    let cut_time = (totkeys - 1) as f32 * ptex.length;

    for k in 0..(totkeys - 1) as usize {
        if k as f32 + 1.0 >= cut_time {
            let fac = cut_time - k as f32;
            start_index = k + 1;
            end_index = start_index + extrakeys as usize;
            spiral_start_time = (k as f32 + fac) / (totkeys - 1) as f32;
            spiral_start = keys[k].co.lerp(keys[k + 1].co, fac);
            spiral_par_co = parent_keys[k].co.lerp(parent_keys[k + 1].co, fac);
            spiral_par_vel = parent_keys[k].vel.lerp(parent_keys[k + 1].vel, fac);
            spiral_par_rot = interp_qt(parent_keys[k].rot, parent_keys[k + 1].rot, fac);
            break;
        }
    }

    let mut dir = Vec3::ZERO;
    let mut kink_base = Vec3::ZERO;
    kink_base[part.kink_axis as usize] = 1.0;
    kink_base = env.object_to_world.transform_direction(kink_base);

    let mut modifier_ctx = ParticleChildModifierContext {
        env,
        ptex: Some(ptex.clone()),
        cpa_index,
        cpa_parent: 0,
        orco,
        parent_keys: Some(parent_keys),
        par_co: Vec3::ZERO,
        par_vel: Vec3::ZERO,
        par_rot: Quaternion::IDENTITY,
        par_orco: parent_orco,
    };

    for k in 0..end_index.min(keys.len()) {
        let (par_time, par_co, par_vel, par_rot);
        if k < start_index {
            dir = (keys[k + 1].co - keys[k].co).normalized();
            par_time = k as f32 / (totkeys - 1) as f32;
            par_co = parent_keys[k].co;
            par_vel = parent_keys[k].vel;
            par_rot = parent_keys[k].rot;
        } else {
            let spiral_time = (k - start_index) as f32 / (extrakeys - 1) as f32;
            par_time = spiral_start_time;
            par_co = spiral_par_co;
            par_vel = spiral_par_vel;
            par_rot = spiral_par_rot;

            let proj = project_v3(kink_base, dir);
            let mut kink = (kink_base - proj).normalized();
            if kink_axis_random > 0.0 {
                let a = kink_axis_random
                    * ((env.frand)(7112 + seed) * 2.0 - 1.0)
                    * PI;
                kink = axis_angle_normalized_to_mat3(dir, a) * kink;
            }
            do_kink_spiral_deform(
                &mut keys[k],
                dir,
                kink,
                spiral_time,
                kink_freq,
                kink_shape,
                kink_amp,
                spiral_start,
            );
        }
        modifier_ctx.par_co = par_co;
        modifier_ctx.par_vel = par_vel;
        modifier_ctx.par_rot = par_rot;
        do_child_modifiers(&modifier_ctx, hairmat, &mut keys[k], par_time);
    }

    *r_max_length = 0.0;
    for k in 0..end_index.saturating_sub(1).min(keys.len()) {
        *r_max_length += (keys[k + 1].co - keys[k].co).length();
    }
}

/// `check_path_length` — returns false when exceeding max_length
/// (clamps the point and writes segments).
fn check_path_length(
    k: i32,
    keys: &mut [ParticleCacheKey],
    max_length: f32,
    step_length: f32,
    cur_length: &mut f32,
) -> bool {
    let ku = k as usize;
    if *cur_length + step_length > max_length {
        let dvec = (keys[ku].co - keys[ku - 1].co)
            * ((max_length - *cur_length) / step_length);
        keys[ku].co = keys[ku - 1].co + dvec;
        keys[0].segments = k;
        return false;
    }
    true
}

/// `psys_apply_child_modifiers`.
pub fn psys_apply_child_modifiers(
    env: &ChildEnv,
    cpa_index: i32,
    cpa_parent: i32,
    ptex: &ParticleTexture,
    orco: Vec3,
    hairmat: Mat4,
    keys: &mut [ParticleCacheKey],
    parent_keys: Option<&[ParticleCacheKey]>,
    parent_orco: Vec3,
) {
    let part = env.part;
    let draw_col_ma = part.draw_col == PART_DRAW_COL_MAT;
    let use_length_check = part.kink != PART_KINK_SPIRAL;
    let mut totkeys;
    let mut max_length;

    if part.kink == PART_KINK_SPIRAL {
        let mut ml = 0.0;
        do_kink_spiral(
            env,
            ptex,
            parent_orco,
            cpa_index,
            orco,
            hairmat,
            keys,
            parent_keys.unwrap_or(&[]),
            &mut ml,
        );
        max_length = ml;
        totkeys = env.segments + 1;
        keys[0].segments = totkeys - 1;
    } else {
        let mut modifier_ctx = ParticleChildModifierContext {
            env,
            ptex: Some(ptex.clone()),
            cpa_index,
            cpa_parent,
            orco,
            parent_keys,
            par_co: Vec3::ZERO,
            par_vel: Vec3::ZERO,
            par_rot: Quaternion::IDENTITY,
            par_orco: parent_orco,
        };
        totkeys = env.segments + 1;
        max_length = ptex.length;

        for k in 0..totkeys.min(keys.len() as i32) as usize {
            let iter = psys_path_iter_get(keys, parent_keys, k);
            let pk = parent_keys.unwrap_or(&[]);
            modifier_ctx.par_co = pk.get(k).map(|k2| k2.co).unwrap_or_default();
            modifier_ctx.par_vel = pk.get(k).map(|k2| k2.vel).unwrap_or_default();
            modifier_ctx.par_rot = iter.parent_rotation;
            do_child_modifiers(&modifier_ctx, hairmat, &mut keys[k], iter.time);
        }
    }

    let step_length = 1.0 / (totkeys - 1).max(1) as f32;
    let mut cur_length = 0.0f32;

    if max_length <= 0.0 {
        keys[0].segments = -1;
        totkeys = 0;
    }

    for k in 0..totkeys.min(keys.len() as i32) as usize {
        if k >= 2 {
            keys[k - 1].vel = (keys[k].co - keys[k - 2].co) * 0.5;
        }
        if use_length_check && k > 0 {
            let (prev_co, cur_co) = (keys[k - 1].co, keys[k].co);
            cur_length += (cur_co - prev_co).length();
            if !check_path_length(k as i32, keys, max_length, step_length, &mut cur_length) {
                keys[k].vel = keys[k].co - keys[k - 1].co;
                if let Some(c) = env.material_color {
                    if draw_col_ma {
                        keys[k].col = c;
                    }
                }
                break;
            }
        }
        if k as i32 == totkeys - 1 {
            keys[k].vel = keys[k].co - keys[k - 1].co;
        }
        if let Some(c) = env.material_color {
            if draw_col_ma {
                keys[k].col = c;
            }
        }
    }
}

/// `do_kink` — kink deform for curl/radial/wave/braid.
#[allow(clippy::too_many_arguments)]
pub fn do_kink(
    state: &mut ParticleCacheKey,
    par_co: Vec3,
    par_vel: Vec3,
    par_rot: Quaternion,
    mut time: f32,
    freq: f32,
    mut shape: f32,
    amplitude: f32,
    flat: f32,
    type_: i16,
    axis: i16,
    obmat: Option<Mat4>,
    smooth_start: bool,
) {
    let mut kink = Vec3::new(1.0, 0.0, 0.0);
    let mut dt = 1.0f32;

    if type_ == PART_KINK_NO as i16 || type_ == PART_KINK_SPIRAL as i16 {
        return;
    }
    time = time.clamp(0.0, 1.0);
    if shape != 0.0 && type_ != PART_KINK_BRAID as i16 {
        if shape < 0.0 {
            time = time.powf(1.0 + shape);
        } else {
            time = time.powf(1.0 / (1.0 - shape));
        }
    }
    let t = time * freq * PI;
    if smooth_start {
        dt = t.abs().clamp(0.0, PI);
        dt = (dt / 2.0).sin();
    }
    if type_ != PART_KINK_RADIAL as i16 {
        kink = Vec3::ZERO;
        kink[axis as usize] = 1.0;
        if let Some(m) = obmat {
            kink = m.transform_direction(kink);
        }
        kink = par_rot * kink;
        kink -= project_v3(kink, par_vel);
        kink = kink.normalized();
    }

    let mut result = state.co;
    let mut par_vec = par_co - state.co;

    match type_ {
        x if x == PART_KINK_CURL as i16 => {
            let curl_offset = axis_angle_to_quat(par_vel, t) * (kink * amplitude);
            par_vec = state.co.lerp(par_co, flat);
            result = par_vec + curl_offset;
        }
        x if x == PART_KINK_RADIAL as i16 => {
            if flat > 0.0 {
                let proj = project_v3(par_vec, par_vel);
                result += proj * flat;
            }
            result += par_vec * -amplitude * t.sin();
        }
        x if x == PART_KINK_WAVE as i16 => {
            result += kink * amplitude * t.sin();
            if flat > 0.0 {
                let mut proj = project_v3(par_vec, kink);
                result += proj * flat;
                proj = project_v3(par_vec, par_vel);
                result += proj * flat;
            }
        }
        x if x == PART_KINK_BRAID as i16 => {
            let mut y_vec = par_rot * Vec3::new(0.0, 1.0, 0.0);
            let mut z_vec = par_rot * Vec3::new(0.0, 0.0, 1.0);
            let vec_one = (-par_vec).normalized();
            let inp_y = y_vec.dot(vec_one);
            let inp_z = z_vec.dot(vec_one);
            let mut state_co;
            if inp_y > 0.5 {
                state_co = y_vec;
                y_vec *= amplitude * t.cos();
                z_vec *= amplitude / 2.0 * (2.0 * t).sin();
            } else if inp_z > 0.0 {
                state_co = z_vec * (PI / 3.0).sin() + y_vec * -0.5;
                y_vec *= -amplitude * (t + PI / 3.0).cos();
                z_vec *= amplitude / 2.0 * (2.0 * t + PI / 6.0).cos();
            } else {
                state_co = z_vec * -(PI / 3.0).sin() + y_vec * -0.5;
                y_vec *= amplitude * -(t + PI / 6.0).sin();
                z_vec *= amplitude / 2.0 * -(2.0 * t + PI / 3.0).sin();
            }
            state_co *= amplitude;
            state_co += par_co;
            let pv = state.co - state_co;
            let length = pv.length();
            let pv = pv.normalized() * length.min(amplitude / 2.0);
            state_co += par_co + y_vec + z_vec + pv;

            let shape2 = 2.0 * PI * (1.0 + shape);
            if t < shape2 {
                let s = (t / shape2).sqrt();
                result = result.lerp(state_co, s);
            } else {
                result = state_co;
            }
        }
        _ => {}
    }
    let _ = &mut par_vec;
    let _ = shape;

    if dt < 1.0 {
        state.co = state.co.lerp(result, dt);
    } else {
        state.co = result;
    }
}

#[allow(clippy::too_many_arguments)]
fn do_clump_level(
    co: &mut Vec3,
    par_co: Vec3,
    time: f32,
    clumpfac: f32,
    clumppow: f32,
    pa_clump: f32,
    clumpcurve: Option<(&CurveMapping, &dyn Fn(&CurveMapping, i32, f32) -> f32)>,
) -> f32 {
    let mut clump = 0.0f32;
    if let Some((curve, eval)) = clumpcurve {
        clump = pa_clump * (1.0 - eval(curve, 0, time).clamp(0.0, 1.0));
        *co = co.lerp(par_co, clump);
    } else if clumpfac != 0.0 {
        let cpow = if clumppow < 0.0 {
            1.0 + clumppow
        } else {
            1.0 + 9.0 * clumppow
        };
        clump = if clumpfac < 0.0 {
            -clumpfac * pa_clump * (1.0 - time).powf(cpow)
        } else {
            clumpfac * pa_clump * time.powf(cpow)
        };
        *co = co.lerp(par_co, clump);
    }
    clump
}

/// `do_clump`.
#[allow(clippy::too_many_arguments)]
pub fn do_clump(
    state: &mut ParticleCacheKey,
    par_co: Vec3,
    time: f32,
    orco_offset: Vec3,
    clumpfac: f32,
    clumppow: f32,
    pa_clump: f32,
    use_clump_noise: bool,
    clump_noise_size: f32,
    clumpcurve: Option<(&CurveMapping, &dyn Fn(&CurveMapping, i32, f32) -> f32)>,
) -> f32 {
    if use_clump_noise && clump_noise_size != 0.0 {
        let noisevec = orco_offset / clump_noise_size;
        let (_da, pa) = noise_voronoi(noisevec.x, noisevec.y, noisevec.z, 1.0, 0);
        let center = par_co + Vec3::new(pa[0], pa[1], pa[2]) * clump_noise_size;
        do_clump_level(
            &mut state.co,
            center,
            time,
            clumpfac,
            clumppow,
            pa_clump,
            clumpcurve,
        );
    }
    do_clump_level(
        &mut state.co,
        par_co,
        time,
        clumpfac,
        clumppow,
        pa_clump,
        clumpcurve,
    )
}

fn do_rough(loc: Vec3, mat: Mat4, t: f32, fac: f32, size: f32, thres: f32, state: &mut ParticleCacheKey) {
    if thres != 0.0 && (-1.5 + loc.x + loc.y + loc.z).abs() < 1.5 * thres {
        return;
    }
    let rco = loc * t;
    let rough = Vec3::new(
        -1.0 + 2.0 * noise_generic_turbulence(size, rco.x, rco.y, rco.z, 2, false, 2),
        -1.0 + 2.0 * noise_generic_turbulence(size, rco.y, rco.z, rco.x, 2, false, 2),
        -1.0 + 2.0 * noise_generic_turbulence(size, rco.z, rco.x, rco.y, 2, false, 2),
    );
    state.co += Vec3::new(mat[0].x, mat[0].y, mat[0].z) * fac * rough.x;
    state.co += Vec3::new(mat[1].x, mat[1].y, mat[1].z) * fac * rough.y;
    state.co += Vec3::new(mat[2].x, mat[2].y, mat[2].z) * fac * rough.z;
}

fn do_rough_end(loc: Vec3, mat: Mat4, t: f32, fac: f32, shape: f32, state: &mut ParticleCacheKey) {
    let roughfac = fac * t.powf(shape);
    let rx = (-1.0 + 2.0 * loc.x) * roughfac;
    let ry = (-1.0 + 2.0 * loc.y) * roughfac;
    state.co += Vec3::new(mat[0].x, mat[0].y, mat[0].z) * rx;
    state.co += Vec3::new(mat[1].x, mat[1].y, mat[1].z) * ry;
}

fn do_rough_curve(
    loc: Vec3,
    mat: Mat4,
    time: f32,
    fac: f32,
    size: f32,
    roughcurve: &CurveMapping,
    eval: &dyn Fn(&CurveMapping, i32, f32) -> f32,
    state: &mut ParticleCacheKey,
) {
    let fac = fac * eval(roughcurve, 0, time).clamp(0.0, 1.0);
    let rco = loc * time;
    let rough = Vec3::new(
        -1.0 + 2.0 * noise_generic_turbulence(size, rco.x, rco.y, rco.z, 2, false, 2),
        -1.0 + 2.0 * noise_generic_turbulence(size, rco.y, rco.z, rco.x, 2, false, 2),
        -1.0 + 2.0 * noise_generic_turbulence(size, rco.z, rco.x, rco.y, 2, false, 2),
    );
    state.co += Vec3::new(mat[0].x, mat[0].y, mat[0].z) * fac * rough.x;
    state.co += Vec3::new(mat[1].x, mat[1].y, mat[1].z) * fac * rough.y;
    state.co += Vec3::new(mat[2].x, mat[2].y, mat[2].z) * fac * rough.z;
}

fn twist_num_segments(ctx: &ParticleChildModifierContext) -> i32 {
    ctx.env.segments
}

fn twist_get_axis(ctx: &ParticleChildModifierContext, time: f32) -> Vec3 {
    let num_segments = twist_num_segments(ctx);
    let index = ((time * num_segments as f32) as i32).clamp(0, num_segments) as usize;
    let pk = ctx.parent_keys.unwrap_or(&[]);
    if index > 0 {
        pk[index].co - pk[index - 1].co
    } else {
        pk[index + 1].co - pk[index].co
    }
}

fn curvemapping_integrate_clamped(
    curve: &CurveMapping,
    eval: &dyn Fn(&CurveMapping, i32, f32) -> f32,
    start: f32,
    end: f32,
    step: f32,
) -> f32 {
    let mut integral = 0.0;
    let mut x = start;
    while x < end {
        integral += eval(curve, 0, x).clamp(0.0, 1.0) * step;
        x += step;
    }
    integral
}

fn do_twist(ctx: &ParticleChildModifierContext, state: &mut ParticleCacheKey, time: f32) {
    let env = ctx.env;
    let part = env.part;
    let Some(parent_keys) = ctx.parent_keys else {
        return;
    };
    let _ = parent_keys;
    if part.childtype != PART_CHILD_PARTICLES as i16 {
        return;
    }
    if part.twist == 0.0 {
        return;
    }
    let twist_curve = if part.child_flag & PART_CHILD_USE_TWIST_CURVE != 0 {
        env.twistcurve
    } else {
        None
    };
    let axis = twist_get_axis(ctx, time);
    let mut angle = part.twist;
    if let Some(ptex) = &ctx.ptex {
        angle *= (ptex.twist - 0.5) * 2.0;
    }
    if let Some(curve) = twist_curve {
        let n = twist_num_segments(ctx);
        angle *= curvemapping_integrate_clamped(curve, env.curve_eval, 0.0, time, 1.0 / n as f32);
    } else {
        angle *= time;
    }
    let vec = state.co - ctx.par_co;
    state.co = blenders_math::quaternion::axis_angle_to_quat(axis.normalized(), angle * 2.0 * PI) * vec
        + ctx.par_co;
}

/// `do_child_modifiers`.
pub fn do_child_modifiers(
    ctx: &ParticleChildModifierContext,
    mat: Mat4,
    state: &mut ParticleCacheKey,
    t: f32,
) {
    let env = ctx.env;
    let part = env.part;
    let clumpcurve = if part.child_flag & PART_CHILD_USE_CLUMP_CURVE != 0 {
        env.clumpcurve.map(|c| (c, env.curve_eval))
    } else {
        None
    };
    let roughcurve = if part.child_flag & PART_CHILD_USE_ROUGH_CURVE != 0 {
        env.roughcurve
    } else {
        None
    };

    let mut kink_amp = part.kink_amp;
    let kink_amp_clump = part.kink_amp_clump;
    let mut kink_freq = part.kink_freq;
    let mut rough1 = part.rough1;
    let mut rough2 = part.rough2;
    let mut rough_end = part.rough_end;
    let smooth_start = part.childtype == PART_CHILD_FACES as i16;

    if let Some(ptex) = &ctx.ptex {
        kink_amp *= ptex.kink_amp;
        kink_freq *= ptex.kink_freq;
        rough1 *= ptex.rough1;
        rough2 *= ptex.rough2;
        rough_end *= ptex.roughe;
    }

    do_twist(ctx, state, t);

    let mut guided = 0;
    if part.flag & PART_CHILD_EFFECT != 0 {
        guided = (env.do_guides)(ctx.cpa_parent, t);
    }

    if guided == 0 {
        let orco_offset = ctx.orco - ctx.par_orco;
        let pa_clump = ctx.ptex.as_ref().map(|p| p.clump).unwrap_or(1.0);
        let clump = do_clump(
            state,
            ctx.par_co,
            t,
            orco_offset,
            part.clumpfac,
            part.clumppow,
            pa_clump,
            part.child_flag & PART_CHILD_USE_CLUMP_NOISE != 0,
            part.clump_noise_size,
            clumpcurve,
        );

        if kink_freq != 0.0 {
            kink_amp *= 1.0 - kink_amp_clump * clump;
            do_kink(
                state,
                ctx.par_co,
                ctx.par_vel,
                ctx.par_rot,
                t,
                kink_freq,
                part.kink_shape,
                kink_amp,
                part.kink_flat,
                part.kink,
                part.kink_axis,
                Some(env.object_to_world),
                smooth_start,
            );
        }
    }

    if let Some(rc) = roughcurve {
        do_rough_curve(ctx.orco, mat, t, rough1, part.rough1_size, rc, env.curve_eval, state);
    } else {
        if rough1 > 0.0 {
            do_rough(ctx.orco, mat, t, rough1, part.rough1_size, 0.0, state);
        }
        if rough2 > 0.0 {
            let vec = (env.frand_vec)(ctx.cpa_index + 27);
            do_rough(vec, mat, t, rough2, part.rough2_size, part.rough2_thres, state);
        }
        if rough_end > 0.0 {
            let vec = (env.frand_vec)(ctx.cpa_index + 27);
            do_rough_end(vec, mat, t, rough_end, part.rough_end_shape, state);
        }
    }
}
