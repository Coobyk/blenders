//! Port of `intern/effect.cc` — `EffectorCache`/`EffectorData`/
//! `EffectedPoint`, `wind_func`, `falloff_func(_dist/_rad)`,
//! `effector_falloff`, `get_effector_data`, `do_physical_effector`.
//! Scene/depsgraph/mesh access injected via closures + handles.

use blenders_math::vector::Vec3;

/// `PFIELD_*` forcefield types.
pub const PFIELD_NULL: i32 = 0;
/// See above.
pub const PFIELD_FORCE: i32 = 1;
/// See above.
pub const PFIELD_VORTEX: i32 = 2;
/// See above.
pub const PFIELD_MAGNET: i32 = 3;
/// See above.
pub const PFIELD_WIND: i32 = 4;
/// See above.
pub const PFIELD_GUIDE: i32 = 5;
/// See above.
pub const PFIELD_TEXTURE: i32 = 6;
/// See above.
pub const PFIELD_HARMONIC: i32 = 7;
/// See above.
pub const PFIELD_CHARGE: i32 = 8;
/// See above.
pub const PFIELD_LENNARDJ: i32 = 9;
/// See above.
pub const PFIELD_BOID: i32 = 10;
/// See above.
pub const PFIELD_TURBULENCE: i32 = 11;
/// See above.
pub const PFIELD_DRAG: i32 = 12;
/// See above.
pub const PFIELD_FLUIDFLOW: i32 = 13;
/// `PFIELD_SHAPE_*`.
pub const PFIELD_SHAPE_POINT: i32 = 0;
/// See above.
pub const PFIELD_SHAPE_PLANE: i32 = 1;
/// See above.
pub const PFIELD_SHAPE_SURFACE: i32 = 2;
/// See above.
pub const PFIELD_SHAPE_POINTS: i32 = 3;
/// See above.
pub const PFIELD_SHAPE_LINE: i32 = 4;
/// `PFIELD_Z_*`.
pub const PFIELD_Z_BOTH: i32 = 0;
/// See above.
pub const PFIELD_Z_POS: i32 = 1;
/// See above.
pub const PFIELD_Z_NEG: i32 = 2;
/// `PFIELD_FALL_*`.
pub const PFIELD_FALL_SPHERE: i32 = 0;
/// See above.
pub const PFIELD_FALL_TUBE: i32 = 1;
/// See above.
pub const PFIELD_FALL_CONE: i32 = 2;
/// `PFIELD_USEMIN`/`PFIELD_USEMAX`/`PFIELD_USEMINR`/`PFIELD_USEMAXR`.
pub const PFIELD_USEMIN: i32 = 1 << 0;
/// See above.
pub const PFIELD_USEMAX: i32 = 1 << 1;
/// See above.
pub const PFIELD_USEMINR: i32 = 1 << 2;
/// See above.
pub const PFIELD_USEMAXR: i32 = 1 << 3;
/// `PFIELD_GRAVITATION`.
pub const PFIELD_GRAVITATION: i32 = 1 << 4;
/// `PFIELD_DO_LOCATION`.
pub const PFIELD_DO_LOCATION: i32 = 1 << 5;
/// `PFIELD_DO_ROTATION`.
pub const PFIELD_DO_ROTATION: i32 = 1 << 6;
/// `PFIELD_GLOBAL_CO`.
pub const PFIELD_GLOBAL_CO: i32 = 1 << 7;
/// `PFIELD_MULTIPLE_SPRINGS`.
pub const PFIELD_MULTIPLE_SPRINGS: i32 = 1 << 8;
/// `PFIELD_SMOKE_DENSITY`.
pub const PFIELD_SMOKE_DENSITY: i32 = 1 << 9;
/// `PE_USE_NORMAL_DATA`.
pub const PE_USE_NORMAL_DATA: i32 = 1 << 0;

/// `PartDeflect` fields used.
#[derive(Clone, Default)]
pub struct PartDeflect {
    /// `flag` (PFIELD_USEMIN etc).
    pub flag: i32,
    /// `forcefield` type.
    pub forcefield: i32,
    /// `falloff` shape.
    pub falloff: i32,
    /// `shape` (PFIELD_SHAPE_*).
    pub shape: i32,
    /// `zdir`.
    pub zdir: i32,
    /// `f_strength`.
    pub f_strength: f32,
    /// `f_damp`.
    pub f_damp: f32,
    /// `f_flow`.
    pub f_flow: f32,
    /// `f_noise`.
    pub f_noise: f32,
    /// `f_power`.
    pub f_power: f32,
    /// `f_power_r`.
    pub f_power_r: f32,
    /// `mindist`, `maxdist`.
    pub mindist: f32,
    /// See `mindist`.
    pub maxdist: f32,
    /// `minrad`, `maxrad`.
    pub minrad: f32,
    /// See `minrad`.
    pub maxrad: f32,
    /// `f_size`.
    pub f_size: f32,
    /// `f_source` (fluid handle).
    pub f_source: usize,
}

/// `EffectorData`.
#[derive(Clone, Default)]
pub struct EffectorData {
    /// `loc`.
    pub loc: Vec3,
    /// `nor`.
    pub nor: Vec3,
    /// `vel`.
    pub vel: Vec3,
    /// `size`.
    pub size: f32,
    /// `vec_to_point`.
    pub vec_to_point: Vec3,
    /// `distance`.
    pub distance: f32,
    /// `vec_to_point2`/`nor2` (object-based, PE_USE_NORMAL_DATA off).
    pub vec_to_point2: Vec3,
    /// See `vec_to_point2`.
    pub nor2: Vec3,
    /// `index` (particle/point index).
    pub index: i32,
    /// `charge`.
    pub charge: f32,
    /// `falloff` (computed by `effector_falloff`).
    pub falloff: f32,
}

/// `EffectedPoint`.
#[derive(Clone, Default)]
pub struct EffectedPoint {
    /// `loc`.
    pub loc: Vec3,
    /// `vel`.
    pub vel: Vec3,
    /// `vel_to_sec`/`vel_to_frame` multipliers.
    pub vel_to_sec: f32,
    /// See `vel_to_sec`.
    pub vel_to_frame: f32,
    /// `index`.
    pub index: i32,
    /// `charge`.
    pub charge: f32,
    /// `size`.
    pub size: f32,
    /// `psys` self-check handle.
    pub psys: usize,
    /// `ave` angular velocity out (rotation path).
    pub ave: Option<Vec3>,
    /// `rot` quaternion (rotation path).
    pub rot: Option<[f32; 4]>,
}

/// `wind_func` — `rng` supplies (int, float) pairs.
pub fn wind_func(rng: &mut dyn FnMut() -> (i32, f32), strength: f32) -> f32 {
    let (rand_i, f) = rng();
    let random = (rand_i + 1).rem_euclid(128);
    let force = f + 1.0;
    let sign = if random as f32 > 64.0 { 1.0f32 } else { -1.0f32 };
    sign * (random as f32 / force) * strength / 128.0
}

/// `falloff_func`.
pub fn falloff_func(
    fac: f32,
    usemin: bool,
    mindist: f32,
    usemax: bool,
    maxdist: f32,
    power: f32,
) -> f32 {
    if usemax && fac > maxdist {
        return 0.0;
    }
    if usemin && fac < mindist {
        return 1.0;
    }
    let mindist = if usemin { mindist } else { 0.0 };
    (1.0 + fac - mindist).powf(-power)
}

/// `falloff_func_dist`.
pub fn falloff_func_dist(pd: &PartDeflect, fac: f32) -> f32 {
    falloff_func(
        fac,
        pd.flag & PFIELD_USEMIN != 0,
        pd.mindist,
        pd.flag & PFIELD_USEMAX != 0,
        pd.maxdist,
        pd.f_power,
    )
}

/// `falloff_func_rad`.
pub fn falloff_func_rad(pd: &PartDeflect, fac: f32) -> f32 {
    falloff_func(
        fac,
        pd.flag & PFIELD_USEMINR != 0,
        pd.minrad,
        pd.flag & PFIELD_USEMAXR != 0,
        pd.maxrad,
        pd.f_power_r,
    )
}

/// `effector_falloff` — `weights` = optional per-type weight table.
pub fn effector_falloff(
    pd: &PartDeflect,
    efd: &EffectorData,
    weight0: f32,
    weight_type: f32,
) -> f32 {
    let mut falloff = weight0 * weight_type;
    let fac = efd.nor.dot(efd.vec_to_point2);
    if pd.zdir == PFIELD_Z_POS && fac < 0.0 {
        falloff = 0.0;
    } else if pd.zdir == PFIELD_Z_NEG && fac > 0.0 {
        falloff = 0.0;
    } else {
        match pd.falloff {
            PFIELD_FALL_SPHERE => {
                falloff *= falloff_func_dist(pd, efd.distance);
            }
            PFIELD_FALL_TUBE => {
                falloff *= falloff_func_dist(pd, fac.abs());
                if falloff != 0.0 {
                    let temp = efd.vec_to_point2 - efd.nor * fac;
                    falloff *= falloff_func_rad(pd, temp.length());
                }
            }
            PFIELD_FALL_CONE => {
                falloff *= falloff_func_dist(pd, fac.abs());
                if falloff != 0.0 {
                    let len = efd.vec_to_point2.length();
                    let r_fac = if len > 0.0 {
                        (fac / len).clamp(-1.0, 1.0).acos().to_degrees()
                    } else {
                        0.0
                    };
                    falloff *= falloff_func_rad(pd, r_fac);
                }
            }
            _ => {}
        }
    }
    falloff
}

/// `closest_point_on_surface` — `nearest` injected
/// `(position, index)`; positions + corner_tris + corner_verts +
/// edges + velocities supply the surface.
#[allow(clippy::too_many_arguments)]
pub fn closest_point_on_surface(
    nearest: Option<(Vec3, usize)>,
    positions: &[Vec3],
    corner_tris: &[[i32; 3]],
    corner_verts: &[i32],
    edges: &[[i32; 2]],
    velocities: Option<&[[f32; 3]]>,
) -> Option<(Vec3, Vec3, Vec3)> {
    let (co, index) = nearest?;
    let mut surface_nor = Vec3::ZERO;
    let mut surface_vel = Vec3::ZERO;
    if !corner_tris.is_empty() {
        let tri = corner_tris[index];
        let verts = [
            corner_verts[tri[0] as usize],
            corner_verts[tri[1] as usize],
            corner_verts[tri[2] as usize],
        ];
        let a = positions[verts[0] as usize];
        let b = positions[verts[1] as usize];
        let c = positions[verts[2] as usize];
        surface_nor = (b - a).cross(c - a).normalized();
        if let Some(v) = velocities {
            surface_vel = (Vec3::from(v[verts[0] as usize])
                + Vec3::from(v[verts[1] as usize])
                + Vec3::from(v[verts[2] as usize]))
                / 3.0;
        }
    } else {
        let edge = edges[index];
        let a = positions[edge[0] as usize];
        let b = positions[edge[1] as usize];
        surface_nor = (b - a).normalized();
        if let Some(v) = velocities {
            surface_vel = (Vec3::from(v[edge[0] as usize])
                + Vec3::from(v[edge[1] as usize]))
                / 2.0;
        }
    }
    Some((co, surface_nor, surface_vel))
}

/// `get_effector_data` — the non-particle/non-surface branches are
/// verbatim; surface/points/particle sources injected.
#[allow(clippy::too_many_arguments)]
pub fn get_effector_data(
    pd: &PartDeflect,
    ob_to_world: &blenders_math::matrix::Mat4,
    eff_flag: i32,
    point: &EffectedPoint,
    real_velocity: bool,
    surface_closest: Option<(Vec3, Vec3, Vec3)>,
    points_source: Option<(Vec3, Vec3)>, /* (loc, nor) */
) -> Option<EffectorData> {
    let mut efd = EffectorData::default();
    let ret;
    if pd.shape == PFIELD_SHAPE_SURFACE && surface_closest.is_some() {
        let mut vec = point.vel * point.vel_to_frame;
        vec = vec + point.loc;
        let (loc, nor, vel) = surface_closest.unwrap();
        efd.loc = loc;
        efd.nor = nor;
        if real_velocity {
            efd.vel = vel;
        }
        efd.size = 0.0;
        ret = true;
    } else if pd.shape == PFIELD_SHAPE_POINTS && points_source.is_some() {
        let (mut loc, mut nor) = points_source.unwrap();
        loc = ob_to_world.transform_point(loc);
        let m3 = blenders_math::matrix::Mat3::from_cols(
            Vec3::new(ob_to_world[0][0], ob_to_world[0][1], ob_to_world[0][2]),
            Vec3::new(ob_to_world[1][0], ob_to_world[1][1], ob_to_world[1][2]),
            Vec3::new(ob_to_world[2][0], ob_to_world[2][1], ob_to_world[2][2]),
        );
        nor = (m3 * nor).normalized();
        efd.loc = loc;
        efd.nor = nor;
        efd.size = 0.0;
        ret = true;
    } else {
        let ob_loc = Vec3::new(ob_to_world[3][0], ob_to_world[3][1], ob_to_world[3][2]);
        let ob_z = Vec3::new(ob_to_world[2][0], ob_to_world[2][1], ob_to_world[2][2])
            .normalized();
        efd.nor = ob_z;
        if pd.shape == PFIELD_SHAPE_PLANE || pd.shape == PFIELD_SHAPE_LINE {
            let temp = point.loc - ob_loc;
            let translate = ob_z * temp.dot(ob_z);
            if pd.forcefield == PFIELD_VORTEX || pd.shape == PFIELD_SHAPE_LINE {
                efd.loc = ob_loc + translate;
            } else {
                efd.loc = point.loc - translate;
            }
        } else {
            efd.loc = ob_loc;
        }
        efd.vel = Vec3::ZERO;
        efd.size = 0.0;
        ret = true;
    }
    if !ret {
        return None;
    }
    efd.vec_to_point = point.loc - efd.loc;
    efd.distance = efd.vec_to_point.length();
    if pd.forcefield == PFIELD_HARMONIC && pd.f_size != 0.0 && efd.distance != 0.0 {
        efd.vec_to_point = efd.vec_to_point * ((efd.distance - pd.f_size) / efd.distance);
    }
    if eff_flag & PE_USE_NORMAL_DATA != 0 {
        efd.vec_to_point2 = efd.vec_to_point;
        efd.nor2 = efd.nor;
    } else {
        efd.vec_to_point2 = point.loc
            - Vec3::new(ob_to_world[3][0], ob_to_world[3][1], ob_to_world[3][2]);
        efd.nor2 = Vec3::new(ob_to_world[2][0], ob_to_world[2][1], ob_to_world[2][2])
            .normalized();
    }
    Some(efd)
}

/// `do_physical_effector` — the core force computation, verbatim.
/// `turbulence_fn`/`fluid_velocity_fn` injected.
#[allow(clippy::too_many_arguments)]
pub fn do_physical_effector(
    pd: &PartDeflect,
    efd: &mut EffectorData,
    point: &mut EffectedPoint,
    total_force: &mut Vec3,
    rng: &mut dyn FnMut() -> (i32, f32),
    turbulence_fn: &dyn Fn(f32, [f32; 3], i32) -> f32,
    fluid_velocity_fn: &dyn Fn(Vec3) -> Option<(Vec3, f32)>,
) {
    let mut force = Vec3::ZERO;
    let mut strength = pd.f_strength;
    let mut damp = pd.f_damp;
    let noise_factor = pd.f_noise;
    let mut flow_falloff = efd.falloff;

    if noise_factor > 0.0 {
        strength += wind_func(rng, noise_factor);
        if pd.forcefield == PFIELD_HARMONIC || pd.forcefield == PFIELD_DRAG {
            damp += wind_func(rng, noise_factor);
        }
    }
    force = efd.vec_to_point;
    match pd.forcefield {
        PFIELD_WIND => {
            force = efd.nor * (strength * efd.falloff);
        }
        PFIELD_FORCE => {
            force = force.normalized();
            if pd.flag & PFIELD_GRAVITATION != 0 {
                if efd.distance < f32::EPSILON {
                    strength = 0.0;
                } else {
                    strength *= efd.distance.powf(-2.0);
                }
            }
            force = force * (strength * efd.falloff);
        }
        PFIELD_VORTEX => {
            if pd.shape == PFIELD_SHAPE_POINT {
                force = efd.nor.cross(efd.vec_to_point).normalized();
                force = force * (strength * efd.distance * efd.falloff);
            } else {
                let mut temp = efd.nor2.cross(efd.vec_to_point2);
                temp = temp * (strength * efd.falloff);
                force = efd.nor2.cross(temp);
                force = force * (strength * efd.falloff);
                let temp2 = temp + point.vel * (-point.vel_to_sec);
                force = force + temp2;
            }
        }
        PFIELD_MAGNET => {
            let mut temp = if pd.shape == PFIELD_SHAPE_POINT || pd.shape == PFIELD_SHAPE_LINE {
                efd.nor.cross(efd.vec_to_point)
            } else {
                efd.nor
            };
            temp = temp.normalized() * (strength * efd.falloff);
            force = point.vel.cross(temp) * point.vel_to_sec;
        }
        PFIELD_HARMONIC => {
            force = force * (-strength * efd.falloff);
            let temp = point.vel * (-damp * 2.0 * strength.abs().sqrt() * point.vel_to_sec);
            force = force + temp;
        }
        PFIELD_CHARGE => {
            force = force * (point.charge * strength * efd.falloff);
        }
        PFIELD_LENNARDJ => {
            let mut fac = ((efd.size + point.size) / efd.distance).powf(6.0);
            fac = -fac * (1.0 - fac) / efd.distance;
            fac = fac.min(2.0);
            force = force * (strength * fac);
        }
        PFIELD_BOID => return,
        PFIELD_TURBULENCE => {
            let temp = if pd.flag & PFIELD_GLOBAL_CO != 0 {
                point.loc
            } else {
                efd.vec_to_point2 + efd.nor2
            };
            force = Vec3::new(
                -1.0 + 2.0 * turbulence_fn(pd.f_size, [temp.x, temp.y, temp.z], 2),
                -1.0 + 2.0 * turbulence_fn(pd.f_size, [temp.y, temp.z, temp.x], 2),
                -1.0 + 2.0 * turbulence_fn(pd.f_size, [temp.z, temp.x, temp.y], 2),
            );
            force = force * (strength * efd.falloff);
        }
        PFIELD_DRAG => {
            force = point.vel;
            let fac = force.length() * point.vel_to_sec;
            let strength = strength.min(2.0);
            let damp = damp.min(2.0);
            force = force.normalized() * (-efd.falloff * fac * (strength * fac + damp));
        }
        PFIELD_FLUIDFLOW => {
            force = Vec3::ZERO;
            flow_falloff = 0.0;
            if pd.f_source != 0 {
                if let Some((vel, density)) = fluid_velocity_fn(point.loc) {
                    let mut influence = strength * efd.falloff;
                    if pd.flag & PFIELD_SMOKE_DENSITY != 0 {
                        influence *= density;
                    }
                    force = vel * influence;
                    flow_falloff = influence;
                }
            }
        }
        _ => {}
    }

    if pd.flag & PFIELD_DO_LOCATION != 0 {
        *total_force = *total_force + force * (1.0 / point.vel_to_sec);
        if pd.forcefield != PFIELD_HARMONIC
            && pd.forcefield != PFIELD_DRAG
            && pd.f_flow != 0.0
        {
            *total_force = *total_force + point.vel * (-pd.f_flow * flow_falloff);
        }
    }

    if pd.flag & PFIELD_DO_ROTATION != 0 {
        if let (Some(ave), Some(rot)) = (&mut point.ave, point.rot) {
            let xvec = Vec3::new(1.0, 0.0, 0.0);
            let q = blenders_math::quaternion::Quaternion { w: rot[0], x: rot[1], y: rot[2], z: rot[3] };
            let xvec_r = q.rotate_vec3(xvec);
            let mut dave = xvec_r.cross(force);
            if pd.f_flow != 0.0 {
                dave = dave + *ave * (-pd.f_flow * efd.falloff);
            }
            *ave = *ave + dave;
        }
    }
}
