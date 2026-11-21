//! Port of `intern/fluid.cc` — Mantaflow domain setup:
//! `manta_pos_to_cell`/`manta_set_domain_from_mesh`,
//! `update_final_gravity`, `clamp_bounds_in_domain`,
//! `FluidObjectBB` map ops, `is_static_object`,
//! `apply_effector_fields`, `manta_get_index`.
//!
//! `PORT_PENDING`: `manta_step`/`fluid_modifier_process*`
//! (the Mantaflow solver loop — FFI to `manta::` calls
//! injected), `update_flows*`, `update_obstacles*`,
//! `emit_from_mesh`/`emit_from_particles` (Mesh sampling +
//! BVHTree), `create_*_geometry` (VDB→Mesh), cache
//! management (`BKE_fluid_cache_*` — pointcache files).

/// `FLUID_DOMAIN_USE_*` verbatim.
pub const FLUID_DOMAIN_USE_NOISE: i32 = 1 << 1;
/// `FLUID_DOMAIN_USE_DISSOLVE`.
pub const FLUID_DOMAIN_USE_DISSOLVE: i32 = 1 << 2;
/// `FLUID_DOMAIN_USE_DISSOLVE_LOG`.
pub const FLUID_DOMAIN_USE_DISSOLVE_LOG: i32 = 1 << 3;
/// `FLUID_DOMAIN_USE_ADAPTIVE_DOMAIN`.
pub const FLUID_DOMAIN_USE_ADAPTIVE_DOMAIN: i32 = 1 << 7;
/// `FLUID_DOMAIN_USE_ADAPTIVE_TIME`.
pub const FLUID_DOMAIN_USE_ADAPTIVE_TIME: i32 = 1 << 8;
/// `FLUID_DOMAIN_USE_MESH`.
pub const FLUID_DOMAIN_USE_MESH: i32 = 1 << 9;
/// `FLUID_DOMAIN_USE_GUIDE`.
pub const FLUID_DOMAIN_USE_GUIDE: i32 = 1 << 10;
/// `FLUID_DOMAIN_USE_SPEED_VECTORS`.
pub const FLUID_DOMAIN_USE_SPEED_VECTORS: i32 = 1 << 11;
/// `FLUID_DOMAIN_EXPORT_MANTA_SCRIPT`.
pub const FLUID_DOMAIN_EXPORT_MANTA_SCRIPT: i32 = 1 << 12;
/// `FLUID_DOMAIN_USE_FRACTIONS`.
pub const FLUID_DOMAIN_USE_FRACTIONS: i32 = 1 << 13;
/// `FLUID_DOMAIN_DELETE_IN_OBSTACLE`.
pub const FLUID_DOMAIN_DELETE_IN_OBSTACLE: i32 = 1 << 14;
/// `FLUID_DOMAIN_USE_DIFFUSION`.
pub const FLUID_DOMAIN_USE_DIFFUSION: i32 = 1 << 15;
/// `FLUID_DOMAIN_USE_RESUMABLE_CACHE`.
pub const FLUID_DOMAIN_USE_RESUMABLE_CACHE: i32 = 1 << 16;
/// `FLUID_DOMAIN_USE_VISCOSITY`.
pub const FLUID_DOMAIN_USE_VISCOSITY: i32 = 1 << 17;

/// `FLUID_DOMAIN_BORDER_*` verbatim.
pub const FLUID_DOMAIN_BORDER_FRONT: i32 = 1 << 1;
/// `FLUID_DOMAIN_BORDER_BACK`.
pub const FLUID_DOMAIN_BORDER_BACK: i32 = 1 << 2;
/// `FLUID_DOMAIN_BORDER_RIGHT`.
pub const FLUID_DOMAIN_BORDER_RIGHT: i32 = 1 << 3;
/// `FLUID_DOMAIN_BORDER_LEFT`.
pub const FLUID_DOMAIN_BORDER_LEFT: i32 = 1 << 4;
/// `FLUID_DOMAIN_BORDER_TOP`.
pub const FLUID_DOMAIN_BORDER_TOP: i32 = 1 << 5;
/// `FLUID_DOMAIN_BORDER_BOTTOM`.
pub const FLUID_DOMAIN_BORDER_BOTTOM: i32 = 1 << 6;

/// `manta_get_index` — `x + y * x_res + z * x_res * y_res`
/// (verbatim helper: z-major linearization).
pub fn manta_get_index(x: i32, x_res: i32, y: i32, y_res: i32, z: i32) -> usize {
    (x + y * x_res + z * x_res * y_res) as usize
}

/// `manta_pos_to_cell` (verbatim — `imat * pos - p0` scaled
/// by `1/cell_size`).
pub fn manta_pos_to_cell(
    imat: &blenders_math::Mat4,
    p0: [f32; 3],
    cell_size: [f32; 3],
    pos: [f32; 3],
) -> [f32; 3] {
    let p = blenders_math::matrix_c::mul_v3_m4v3(*imat, blenders_math::Vec3::new(pos[0], pos[1], pos[2]));
    [
        (p.x - p0[0]) / cell_size[0],
        (p.y - p0[1]) / cell_size[1],
        (p.z - p0[2]) / cell_size[2],
    ]
}

/// `manta_set_domain_from_mesh` — fill `p0`,`p1`,`dx`,
/// `global_size`,`dp0`,`imat`,`base_res`,`scale`,
/// `cell_size` (verbatim).
pub struct FluidDomainOut {
    /// `p0`/`p1` bounds.
    pub p0: [f32; 3],
    /// `p1`.
    pub p1: [f32; 3],
    /// `dx`.
    pub dx: f32,
    /// `base_res`.
    pub base_res: [i32; 3],
    /// `cell_size`.
    pub cell_size: [f32; 3],
    /// `global_size`.
    pub global_size: [f32; 3],
    /// `dp0`.
    pub dp0: [f32; 3],
    /// `scale`.
    pub scale: f32,
}
/// `manta_set_domain_from_mesh` (verbatim).
pub fn manta_set_domain_from_mesh(
    vert_positions: &[[f32; 3]],
    ob_scale: [f32; 3],
    maxres: i32,
    init_resolution: bool,
) -> FluidDomainOut {
    let mut min = [f32::MAX; 3];
    let mut max = [-f32::MAX; 3];
    for p in vert_positions {
        for i in 0..3 {
            min[i] = min[i].min(p[i]);
            max[i] = max[i].max(p[i]);
        }
    }
    let mut size = [0f32; 3];
    for i in 0..3 {
        size[i] = max[i] - min[i];
        let s = ob_scale[i];
        size[i] = (size[i] * if s.is_finite() { s } else { 1.0 }).abs();
        if !size[i].is_finite() {
            size[i] = 1.0;
        }
    }
    let mut out = FluidDomainOut {
        p0: min,
        p1: max,
        dx: 1.0 / maxres as f32,
        base_res: [0; 3],
        cell_size: size,
        global_size: size,
        dp0: min,
        scale: 0.0,
    };
    if !init_resolution || size[0] < f32::EPSILON || size[1] < f32::EPSILON || size[2] < f32::EPSILON {
        return out;
    }
    let res = maxres as f32;
    if size[0] >= size[1].max(size[2]) {
        let scale = res / size[0];
        out.scale = size[0] / ob_scale[0].abs();
        out.base_res = [
            maxres,
            ((size[1] * scale + 0.5) as i32).max(4),
            ((size[2] * scale + 0.5) as i32).max(4),
        ];
    } else if size[1] >= size[0].max(size[2]) {
        let scale = res / size[1];
        out.scale = size[1] / ob_scale[1].abs();
        out.base_res = [
            ((size[0] * scale + 0.5) as i32).max(4),
            maxres,
            ((size[2] * scale + 0.5) as i32).max(4),
        ];
    } else {
        let scale = res / size[2];
        out.scale = size[2] / ob_scale[2].abs();
        out.base_res = [
            ((size[0] * scale + 0.5) as i32).max(4),
            ((size[1] * scale + 0.5) as i32).max(4),
            maxres,
        ];
    }
    for i in 0..3 {
        out.cell_size[i] /= out.base_res[i] as f32;
    }
    out
}

/// `update_final_gravity` (verbatim).
pub fn update_final_gravity(
    phys_global_gravity: bool,
    scene_gravity: [f32; 3],
    fds_gravity: [f32; 3],
    effector_global_gravity: f32,
) -> [f32; 3] {
    let g = if phys_global_gravity {
        scene_gravity
    } else {
        fds_gravity
    };
    [g[0] * effector_global_gravity, g[1] * effector_global_gravity, g[2] * effector_global_gravity]
}

/// `clamp_bounds_in_domain` (verbatim).
pub fn clamp_bounds_in_domain(
    use_adaptive: bool,
    adapt_res: i32,
    base_res: [i32; 3],
    min: &mut [i32; 3],
    max: &mut [i32; 3],
    min_vel: Option<[f32; 3]>,
    max_vel: Option<[f32; 3]>,
    margin: i32,
    dt: f32,
) {
    for i in 0..3 {
        let adapt = if use_adaptive { adapt_res } else { 0 };
        min[i] -= margin;
        max[i] += margin;
        if let Some(v) = min_vel {
            if v[i] < 0.0 {
                min[i] += (v[i] * dt).floor() as i32;
            }
        }
        if let Some(v) = max_vel {
            if v[i] > 0.0 {
                max[i] += (v[i] * dt).ceil() as i32;
            }
        }
        min[i] = min[i].clamp(-adapt, base_res[i] + adapt);
        max[i] = max[i].clamp(-adapt, base_res[i] + adapt);
    }
}

/// `FluidObjectBB` verbatim.
#[derive(Default)]
pub struct FluidObjectBB {
    /// `influence`, `velocity`, `distances`, `numobjs` grids.
    pub influence: Vec<f32>,
    /// `velocity` (3× cells).
    pub velocity: Vec<f32>,
    /// `distances`.
    pub distances: Vec<f32>,
    /// `numobjs`.
    pub numobjs: Vec<f32>,
    /// `min`,`max`,`res`,`hmin`,`hmax`,`hres`.
    pub min: [i32; 3],
    /// `max`.
    pub max: [i32; 3],
    /// `res`.
    pub res: [i32; 3],
    /// `hmin`/`hmax`/`hres`.
    pub hmin: [i32; 3],
    /// `hmax`.
    pub hmax: [i32; 3],
    /// `hres`.
    pub hres: [i32; 3],
    /// `total_cells`.
    pub total_cells: i32,
    /// `valid`.
    pub valid: bool,
}

/// `bb_boundInsert` (verbatim).
pub fn bb_bound_insert(bb: &mut FluidObjectBB, point: [f32; 3]) {
    if !bb.valid {
        for i in 0..3 {
            bb.min[i] = point[i].floor() as i32;
            bb.max[i] = point[i].ceil() as i32;
        }
        bb.valid = true;
    } else {
        for i in 0..3 {
            if point[i] < bb.min[i] as f32 {
                bb.min[i] = point[i].floor() as i32;
            }
            if point[i] > bb.max[i] as f32 {
                bb.max[i] = point[i].ceil() as i32;
            }
        }
    }
}

/// `bb_allocateData` (verbatim).
pub fn bb_allocate_data(bb: &mut FluidObjectBB, use_velocity: bool, use_influence: bool) {
    let mut res = [0i32; 3];
    for i in 0..3 {
        res[i] = bb.max[i] - bb.min[i];
        if res[i] <= 0 {
            return;
        }
    }
    bb.total_cells = res[0] * res[1] * res[2];
    bb.res = res;
    let n = bb.total_cells as usize;
    bb.numobjs = vec![0.0; n];
    if use_influence {
        bb.influence = vec![0.0; n];
    }
    if use_velocity {
        bb.velocity = vec![0.0; 3 * n];
    }
    bb.distances = vec![f32::MAX; n];
    bb.valid = true;
}

/// `bb_combineMaps` (verbatim — merge bb2 into output).
pub fn bb_combine_maps(output: &mut FluidObjectBB, bb2: &FluidObjectBB, additive: bool, sample_size: f32) {
    let bb1 = std::mem::take(output);
    for i in 0..3 {
        if bb1.valid {
            output.min[i] = bb1.min[i].min(bb2.min[i]);
            output.max[i] = bb1.max[i].max(bb2.max[i]);
        } else {
            output.min[i] = bb2.min[i];
            output.max[i] = bb2.max[i];
        }
    }
    bb_allocate_data(output, !bb1.velocity.is_empty() || !bb2.velocity.is_empty(),
                     !bb1.influence.is_empty() || !bb2.influence.is_empty());
    for x in output.min[0]..output.max[0] {
        for y in output.min[1]..output.max[1] {
            for z in output.min[2]..output.max[2] {
                let index_out = ((x - output.min[0])
                    + (y - output.min[1]) * output.res[0]
                    + (z - output.min[2]) * output.res[0] * output.res[1])
                    as usize;
                if x >= bb1.min[0] && x < bb1.max[0] && y >= bb1.min[1] && y < bb1.max[1]
                    && z >= bb1.min[2] && z < bb1.max[2]
                {
                    let index_in = ((x - bb1.min[0])
                        + (y - bb1.min[1]) * bb1.res[0]
                        + (z - bb1.min[2]) * bb1.res[0] * bb1.res[1])
                        as usize;
                    output.numobjs[index_out] = bb1.numobjs[index_in];
                    if !output.influence.is_empty() && !bb1.influence.is_empty() {
                        output.influence[index_out] = bb1.influence[index_in];
                    }
                    output.distances[index_out] = bb1.distances[index_in];
                    if !output.velocity.is_empty() && !bb1.velocity.is_empty() {
                        output.velocity[index_out * 3] = bb1.velocity[index_in * 3];
                        output.velocity[index_out * 3 + 1] = bb1.velocity[index_in * 3 + 1];
                        output.velocity[index_out * 3 + 2] = bb1.velocity[index_in * 3 + 2];
                    }
                }
                if x >= bb2.min[0] && x < bb2.max[0] && y >= bb2.min[1] && y < bb2.max[1]
                    && z >= bb2.min[2] && z < bb2.max[2]
                {
                    let index_in = ((x - bb2.min[0])
                        + (y - bb2.min[1]) * bb2.res[0]
                        + (z - bb2.min[2]) * bb2.res[0] * bb2.res[1])
                        as usize;
                    output.numobjs[index_out] = bb2.numobjs[index_in].max(output.numobjs[index_out]);
                    if !output.influence.is_empty() && !bb2.influence.is_empty() {
                        if additive {
                            output.influence[index_out] += bb2.influence[index_in] * sample_size;
                        } else {
                            output.influence[index_out] =
                                bb2.influence[index_in].max(output.influence[index_out]);
                        }
                    }
                    output.distances[index_out] =
                        bb2.distances[index_in].min(output.distances[index_out]);
                    if !output.velocity.is_empty() && !bb2.velocity.is_empty() {
                        for a in 0..3 {
                            if bb2.velocity[index_in * 3 + a] > output.velocity[index_out * 3 + a] {
                                output.velocity[index_out * 3 + a] = bb2.velocity[index_in * 3 + a];
                            }
                        }
                    }
                }
            }
        }
    }
}

/// `is_static_object` — no dynamic modifiers, no active
/// rigidbody, no animation (verbatim; injected flags).
pub fn is_static_object(has_dynamic_modifier: bool, rbo_is_active: bool, moves_in_time: bool) -> bool {
    !has_dynamic_modifier && !rbo_is_active && !moves_in_time
}

/// `apply_effector_fields` (verbatim).
pub fn apply_effector_fields(
    index: usize,
    src_distance_value: f32,
    dest_phi_in: Option<&mut [f32]>,
    src_numobjs_value: f32,
    dest_numobjs: Option<&mut [f32]>,
    src_vel_value: [f32; 3],
    dest_vel_x: Option<&mut [f32]>,
    dest_vel_y: Option<&mut [f32]>,
    dest_vel_z: Option<&mut [f32]>,
) {
    if let Some(phi) = dest_phi_in {
        phi[index] = src_distance_value.min(phi[index]);
    }
    if let Some(n) = dest_numobjs {
        if src_numobjs_value > 0.0 {
            n[index] += 1.0;
        }
    }
    if src_numobjs_value > 0.0 {
        if let (Some(vx), Some(vy), Some(vz)) = (dest_vel_x, dest_vel_y, dest_vel_z) {
            vx[index] += src_vel_value[0];
            vy[index] += src_vel_value[1];
            vz[index] += src_vel_value[2];
        }
    }
}
