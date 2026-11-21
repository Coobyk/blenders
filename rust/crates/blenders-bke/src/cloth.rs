//! Port of `intern/cloth.cc` — `Cloth`/`ClothVertex`/`ClothSpring`/
//! `ClothHairData`/`ClothSolverResult` runtime structs, BVH
//! build/update from cloth, `cloth_clear_cache`/`do_init_cloth`/
//! `do_step_cloth`/`clothModifier_do` skeletons, `cloth_uses_vgroup`,
//! `cloth_apply_vgroup`, `cloth_shrink_factor`, `cloth_bend_poly_dir`,
//! `cloth_spring_angle`, `cloth_hair_update_bending_(rest_)targets`,
//! `cloth_update_springs`/`_verts`/`_spring_lengths`,
//! `cloth_parallel_transport_hair_frame`, free/error-spring mgmt.
//! `PORT_PENDING`: `cloth_build_springs`/`cloth_from_object`/
//! `cloth_add_shear_bend_spring`/`find_internal_spring_target_vertex`
//! (needs mesh `edges`/BVH tree traversal — the spring *update* and
//! vgroup passes that use them are ported); `cloth_to_object`/
//! `cloth_from_mesh` (Mesh write-out needs `Mesh` builder);
//! `Implicit_Data` solver + `PointCache` IO are injected handles.

use blenders_bli::kdopbvh::{self, BVHTree};
use blenders_dna::cloth_types::{
    ClothCollSettings, ClothSimSettings, CLOTH_COLLSETTINGS_FLAG_ENABLED,
    CLOTH_COLLSETTINGS_FLAG_SELF,
};
use blenders_dna::meshdata_types::MDeformVert;
use blenders_math::vector::Vec3;

/// `eClothVertexFlag`.
pub const CLOTH_VERT_FLAG_PINNED: i32 = 1 << 0;
/// `NOSELFCOLL`.
pub const CLOTH_VERT_FLAG_NOSELFCOLL: i32 = 1 << 1;
/// `NOOBJCOLL`.
pub const CLOTH_VERT_FLAG_NOOBJCOLL: i32 = 1 << 2;
/// `SOFTGOALSNAP`.
pub const SOFTGOALSNAP: f32 = 0.999;

/// `eClothSpringTypes`.
pub const CLOTH_SPRING_TYPE_STRUCTURAL: i32 = 1 << 1;
/// `SHEAR`.
pub const CLOTH_SPRING_TYPE_SHEAR: i32 = 1 << 2;
/// `BENDING`.
pub const CLOTH_SPRING_TYPE_BENDING: i32 = 1 << 3;
/// `GOAL`.
pub const CLOTH_SPRING_TYPE_GOAL: i32 = 1 << 4;
/// `SEWING`.
pub const CLOTH_SPRING_TYPE_SEWING: i32 = 1 << 5;
/// `BENDING_HAIR`.
pub const CLOTH_SPRING_TYPE_BENDING_HAIR: i32 = 1 << 6;
/// `INTERNAL`.
pub const CLOTH_SPRING_TYPE_INTERNAL: i32 = 1 << 7;

/// `eClothSpringFlags`.
pub const CLOTH_SPRING_FLAG_DEACTIVATE: i32 = 1 << 1;
/// `NEEDED`.
pub const CLOTH_SPRING_FLAG_NEEDED: i32 = 1 << 2;

/// `CLOTH_BENDING_ANGULAR` (`ClothSimSettings::bending_model`).
pub const CLOTH_BENDING_ANGULAR: i32 = 1;

/// `ClothHairData`.
#[derive(Clone, Copy, Default)]
pub struct ClothHairData {
    /// `loc[3]`.
    pub loc: [f32; 3],
    /// `rot[3][3]`.
    pub rot: [[f32; 3]; 3],
    /// `rest_target[3]`.
    pub rest_target: [f32; 3],
    /// `radius`.
    pub radius: f32,
    /// `bending_stiffness`.
    pub bending_stiffness: f32,
}

/// `ClothSolverResult`.
#[derive(Clone, Copy, Default)]
pub struct ClothSolverResult {
    /// `status`.
    pub status: i32,
    /// `max_iterations`/`min_iterations`.
    pub max_iterations: i32,
    /// See above.
    pub min_iterations: i32,
    /// `avg_iterations`.
    pub avg_iterations: f32,
    /// `max_error`/`min_error`/`avg_error`.
    pub max_error: f32,
    /// See above.
    pub min_error: f32,
    /// See above.
    pub avg_error: f32,
}

/// `ClothVertex`.
#[derive(Clone, Default)]
pub struct ClothVertex {
    /// `flags`.
    pub flags: i32,
    /// `v[3]`.
    pub v: [f32; 3],
    /// `xconst[3]`.
    pub xconst: [f32; 3],
    /// `x[3]`.
    pub x: [f32; 3],
    /// `xold[3]`.
    pub xold: [f32; 3],
    /// `tx[3]`.
    pub tx: [f32; 3],
    /// `txold[3]`.
    pub txold: [f32; 3],
    /// `tv[3]`.
    pub tv: [f32; 3],
    /// `mass`.
    pub mass: f32,
    /// `goal`.
    pub goal: f32,
    /// `impulse[3]`.
    pub impulse: [f32; 3],
    /// `xrest[3]`.
    pub xrest: [f32; 3],
    /// `dcvel[3]`.
    pub dcvel: [f32; 3],
    /// `impulse_count`.
    pub impulse_count: u32,
    /// `avg_spring_len`.
    pub avg_spring_len: f32,
    /// `struct_stiff`.
    pub struct_stiff: f32,
    /// `bend_stiff`.
    pub bend_stiff: f32,
    /// `shear_stiff`.
    pub shear_stiff: f32,
    /// `spring_count`.
    pub spring_count: i32,
    /// `shrink_factor`.
    pub shrink_factor: f32,
    /// `internal_stiff`.
    pub internal_stiff: f32,
    /// `pressure_factor`.
    pub pressure_factor: f32,
}

/// `ClothSpring` — `pa`/`pb` index lists instead of pointers.
#[derive(Clone, Default)]
pub struct ClothSpring {
    /// `ij`.
    pub ij: usize,
    /// `kl`.
    pub kl: usize,
    /// `mn`.
    pub mn: usize,
    /// `pa` poly-a vert indices.
    pub pa: Vec<i32>,
    /// `pb`.
    pub pb: Vec<i32>,
    /// `la`/`lb` (kept for parity = `pa`/`pb` lens).
    pub la: i32,
    /// See above.
    pub lb: i32,
    /// `restlen`.
    pub restlen: f32,
    /// `restang`.
    pub restang: f32,
    /// `type`.
    pub type_: i32,
    /// `flags`.
    pub flags: i32,
    /// `lin_stiffness`.
    pub lin_stiffness: f32,
    /// `ang_stiffness`.
    pub ang_stiffness: f32,
    /// `editrestlen`.
    pub editrestlen: f32,
    /// `target[3]`.
    pub target: [f32; 3],
}

/// `Cloth` — sim object.
#[derive(Default)]
pub struct Cloth {
    /// `verts`.
    pub verts: Vec<ClothVertex>,
    /// `springs` (LinkNode → Vec).
    pub springs: Vec<ClothSpring>,
    /// `numsprings`.
    pub numsprings: u32,
    /// `mvert_num`.
    pub mvert_num: u32,
    /// `primitive_num`.
    pub primitive_num: u32,
    /// `old_solver_type`.
    pub old_solver_type: u8,
    /// `bvhtree`.
    pub bvhtree: usize,
    /// `bvhselftree`.
    pub bvhselftree: usize,
    /// `vert_tris`.
    pub vert_tris: Vec<[i32; 3]>,
    /// `implicit` solver handle.
    pub implicit: usize,
    /// `edgeset` (OrderedEdge set → sorted Vec).
    pub edgeset: Vec<[i32; 2]>,
    /// `last_frame`.
    pub last_frame: i32,
    /// `initial_mesh_volume`.
    pub initial_mesh_volume: f32,
    /// `average_acceleration[3]`.
    pub average_acceleration: [f32; 3],
    /// `edges` (hair).
    pub edges: Vec<[i32; 2]>,
    /// `sew_edge_graph`.
    pub sew_edge_graph: Vec<[i32; 2]>,
}

/// `bvhtree_build_from_cloth` — over `verts[].xold`; `hairdata`
/// picks the edge variant.
pub fn bvhtree_build_from_cloth(
    cloth: &Cloth,
    hair: bool,
    epsilon: f32,
) -> Option<BVHTree> {
    if cloth.primitive_num == 0 {
        return None;
    }
    let mut bvhtree = kdopbvh::bvhtree_new(cloth.primitive_num as i32, epsilon, 4, 26);
    if !hair {
        for i in 0..cloth.primitive_num as usize {
            let t = cloth.vert_tris[i];
            let co: Vec<f32> = [
                cloth.verts[t[0] as usize].xold,
                cloth.verts[t[1] as usize].xold,
                cloth.verts[t[2] as usize].xold,
            ]
            .iter()
            .flat_map(|v| *v)
            .collect();
            kdopbvh::bvhtree_insert(&mut bvhtree, i as i32, &co, 3);
        }
    } else {
        for i in 0..cloth.primitive_num as usize {
            let e = cloth.edges[i];
            let co: Vec<f32> = [
                cloth.verts[e[0] as usize].xold,
                cloth.verts[e[1] as usize].xold,
            ]
            .iter()
            .flat_map(|v| *v)
            .collect();
            kdopbvh::bvhtree_insert(&mut bvhtree, i as i32, &co, 2);
        }
    }
    kdopbvh::bvhtree_balance(&mut bvhtree);
    Some(bvhtree)
}

/// `bvhtree_update_from_cloth` — `tx`/`txold` moving update.
pub fn bvhtree_update_from_cloth(
    cloth: &Cloth,
    bvhtree: &mut BVHTree,
    hair: bool,
    moving: bool,
) {
    if !hair {
        for i in 0..cloth.primitive_num as usize {
            let t = cloth.vert_tris[i];
            let co: Vec<f32> = if moving {
                [
                    cloth.verts[t[0] as usize].txold,
                    cloth.verts[t[1] as usize].txold,
                    cloth.verts[t[2] as usize].txold,
                ]
                .iter()
                .flat_map(|v| *v)
                .collect()
            } else {
                [
                    cloth.verts[t[0] as usize].tx,
                    cloth.verts[t[1] as usize].tx,
                    cloth.verts[t[2] as usize].tx,
                ]
                .iter()
                .flat_map(|v| *v)
                .collect()
            };
            let co_moving: Option<Vec<f32>> = if moving {
                Some(
                    [
                        cloth.verts[t[0] as usize].tx,
                        cloth.verts[t[1] as usize].tx,
                        cloth.verts[t[2] as usize].tx,
                    ]
                    .iter()
                    .flat_map(|v| *v)
                    .collect(),
                )
            } else {
                None
            };
            if !kdopbvh::bvhtree_update_node(bvhtree, i, &co, co_moving.as_deref(), 3) {
                break;
            }
        }
        kdopbvh::bvhtree_update_tree(bvhtree);
    } else {
        for i in 0..cloth.primitive_num as usize {
            let e = cloth.edges[i];
            let co: Vec<f32> = [
                cloth.verts[e[0] as usize].tx,
                cloth.verts[e[1] as usize].tx,
            ]
            .iter()
            .flat_map(|v| *v)
            .collect();
            if !kdopbvh::bvhtree_update_node(bvhtree, i, &co, None, 2) {
                break;
            }
        }
        kdopbvh::bvhtree_update_tree(bvhtree);
    }
}

/// `cloth_uses_vgroup`.
pub fn cloth_uses_vgroup(sim_parms: &ClothSimSettings, coll_parms: &ClothCollSettings) -> bool {
    ((coll_parms.flags & CLOTH_COLLSETTINGS_FLAG_SELF != 0)
        && (coll_parms.vgroup_selfcol > 0))
        || ((coll_parms.flags & CLOTH_COLLSETTINGS_FLAG_ENABLED != 0)
            && (coll_parms.vgroup_objcol > 0))
        || sim_parms.vgroup_pressure > 0
        || sim_parms.vgroup_struct > 0
        || sim_parms.vgroup_bend > 0
        || sim_parms.vgroup_shrink > 0
        || sim_parms.vgroup_intern > 0
        || sim_parms.vgroup_mass > 0
        || sim_parms.vgroup_shear > 0
}

/// `cloth_apply_vgroup` — deform-group weights → goal/stiff/flags.
pub fn cloth_apply_vgroup(
    sim_parms: &ClothSimSettings,
    coll_parms: &ClothCollSettings,
    verts: &mut [ClothVertex],
    dverts: &[MDeformVert],
) {
    if !cloth_uses_vgroup(sim_parms, coll_parms) {
        return;
    }
    for (i, vert) in verts.iter_mut().enumerate() {
        vert.goal = if sim_parms.vgroup_mass > 0 {
            sim_parms.defgoal
        } else {
            0.0
        };
        vert.shrink_factor = 0.0;
        vert.flags &= !(CLOTH_VERT_FLAG_PINNED
            | CLOTH_VERT_FLAG_NOSELFCOLL
            | CLOTH_VERT_FLAG_NOOBJCOLL);
        let Some(dvert) = dverts.get(i) else {
            continue;
        };
        for dw in &dvert.dw {
            let def = dw.def_nr as i16 + 1;
            if def == sim_parms.vgroup_mass {
                vert.goal = dw.weight.powi(4);
                if vert.goal >= SOFTGOALSNAP {
                    vert.flags |= CLOTH_VERT_FLAG_PINNED;
                }
            }
            if def == sim_parms.vgroup_struct {
                vert.struct_stiff = dw.weight;
            }
            if def == sim_parms.vgroup_shear {
                vert.shear_stiff = dw.weight;
            }
            if def == sim_parms.vgroup_bend {
                vert.bend_stiff = dw.weight;
            }
            if def == coll_parms.vgroup_selfcol && dw.weight > 0.0 {
                vert.flags |= CLOTH_VERT_FLAG_NOSELFCOLL;
            }
            if def == coll_parms.vgroup_objcol && dw.weight > 0.0 {
                vert.flags |= CLOTH_VERT_FLAG_NOOBJCOLL;
            }
            if def == sim_parms.vgroup_shrink {
                vert.shrink_factor = dw.weight;
            }
            if def == sim_parms.vgroup_intern {
                vert.internal_stiff = dw.weight;
            }
            if def == sim_parms.vgroup_pressure {
                vert.pressure_factor = dw.weight;
            }
        }
    }
}

/// `cloth_shrink_factor` — geometric mean of the two verts' lerped
/// shrink factors.
pub fn cloth_shrink_factor(
    sim_parms: &ClothSimSettings,
    verts: &[ClothVertex],
    i1: usize,
    i2: usize,
) -> f32 {
    let base = 1.0 - sim_parms.shrink_min;
    let delta = sim_parms.shrink_min - sim_parms.shrink_max;
    let k1 = base + delta * verts[i1].shrink_factor;
    let k2 = base + delta * verts[i2].shrink_factor;
    (k1 * k2).sqrt()
}

/// `cloth_bend_poly_dir` — normal of `(vert_i, vert_j, poly-cent)`.
pub fn cloth_bend_poly_dir(verts: &[ClothVertex], i: usize, j: usize, inds: &[i32]) -> Vec3 {
    let mut cent = Vec3::ZERO;
    let fact = 1.0 / inds.len() as f32;
    for &x in inds {
        let v = verts[x as usize].xrest;
        cent += Vec3::new(v[0], v[1], v[2]) * fact;
    }
    let a = verts[i].xrest;
    let b = verts[j].xrest;
    blenders_math::geom::normal_tri(
        Vec3::new(a[0], a[1], a[2]),
        Vec3::new(b[0], b[1], b[2]),
        cent,
    )
    .0
}

/// `cloth_spring_angle`.
pub fn cloth_spring_angle(
    verts: &[ClothVertex],
    i: usize,
    j: usize,
    i_a: &[i32],
    i_b: &[i32],
) -> f32 {
    let dir_a = cloth_bend_poly_dir(verts, j, i, i_a);
    let dir_b = cloth_bend_poly_dir(verts, i, j, i_b);
    let xr = |k: usize| {
        let v = verts[k].xrest;
        Vec3::new(v[0], v[1], v[2])
    };
    let vec_e = (xr(i) - xr(j)).normalized();
    let cos = dir_a.dot(dir_b);
    let sin = dir_a.cross(dir_b).dot(vec_e);
    sin.atan2(cos)
}

/// `cross_identity_v3` — skew-symmetric matrix of `v`.
pub fn cross_identity_v3(v: Vec3) -> [[f32; 3]; 3] {
    let mut r = [[0.0f32; 3]; 3];
    r[0][1] = v.z;
    r[0][2] = -v.y;
    r[1][0] = -v.z;
    r[1][2] = v.x;
    r[2][0] = v.y;
    r[2][1] = -v.x;
    r
}

/// `madd_m3_m3fl`.
pub fn madd_m3_m3fl(r: &mut [[f32; 3]; 3], m: &[[f32; 3]; 3], f: f32) {
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] += m[i][j] * f;
        }
    }
}

/// `cloth_parallel_transport_hair_frame` — rotate `mat` frame by
/// the `dir_old`→`dir_new` rotation.
pub fn cloth_parallel_transport_hair_frame(
    mat: &mut [[f32; 3]; 3],
    dir_old: Vec3,
    dir_new: Vec3,
) {
    let qm = blenders_math::quaternion::rotation_between_vecs_to_quat(dir_old, dir_new).to_mat3();
    let rot = [[qm.cols[0].x, qm.cols[1].x, qm.cols[2].x],
               [qm.cols[0].y, qm.cols[1].y, qm.cols[2].y],
               [qm.cols[0].z, qm.cols[1].z, qm.cols[2].z]];
    let mut out = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            out[i][j] = (0..3).map(|k| rot[i][k] * mat[k][j]).sum();
        }
    }
    *mat = out;
}

/// `cloth_hair_update_bending_targets` — per-spring target vec
/// (rest_target in current hair frame × restlen).
pub fn cloth_hair_update_bending_targets(cloth: &mut Cloth, hairdata: &[ClothHairData]) {
    if hairdata.is_empty() {
        return;
    }
    let mut hair_frame = [[0.0f32; 3]; 3];
    let mut dir_new = Vec3::ZERO;
    let mut prev_mn: i64 = -1;
    for s in 0..cloth.springs.len() {
        let (stype, sij, skl, smn, srestlen) = {
            let sp = &cloth.springs[s];
            (sp.type_, sp.ij, sp.kl, sp.mn, sp.restlen)
        };
        if stype != CLOTH_SPRING_TYPE_BENDING_HAIR {
            continue;
        }
        let is_root = skl as i64 != prev_mn;
        let hair_ij = &hairdata[sij];
        let hair_kl = &hairdata[skl];
        if is_root {
            hair_frame = hair_ij.rot;
            dir_new = Vec3::new(hair_frame[2][0], hair_frame[2][1], hair_frame[2][2]);
        }
        let dir_old = dir_new;
        let mn = cloth.verts[smn].x;
        let kl = cloth.verts[skl].x;
        dir_new = (Vec3::new(mn[0], mn[1], mn[2]) - Vec3::new(kl[0], kl[1], kl[2])).normalized();
        /* `mul_v3_m3v3(target, hair_frame, hair_kl.rest_target)`. */
        let rt = Vec3::new(hair_kl.rest_target[0], hair_kl.rest_target[1], hair_kl.rest_target[2]);
        let mut target = Vec3::ZERO;
        for i in 0..3 {
            target[i] = hair_frame[i][0] * rt.x + hair_frame[i][1] * rt.y + hair_frame[i][2] * rt.z;
        }
        let target = target * srestlen;
        cloth.springs[s].target = [target.x, target.y, target.z];
        cloth_parallel_transport_hair_frame(&mut hair_frame, dir_old, dir_new);
        prev_mn = smn as i64;
    }
}

/// `cloth_hair_update_bending_rest_targets` — rest-frame targets.
pub fn cloth_hair_update_bending_rest_targets(cloth: &mut Cloth, hairdata: &mut [ClothHairData]) {
    if hairdata.is_empty() {
        return;
    }
    let mut hair_frame = [[0.0f32; 3]; 3];
    let mut dir_new = Vec3::ZERO;
    let mut prev_mn: i64 = -1;
    for s in 0..cloth.springs.len() {
        let spring = &cloth.springs[s];
        if spring.type_ != CLOTH_SPRING_TYPE_BENDING_HAIR {
            continue;
        }
        let is_root = spring.kl as i64 != prev_mn;
        if is_root {
            hair_frame = hairdata[spring.ij].rot;
            dir_new = Vec3::new(hair_frame[2][0], hair_frame[2][1], hair_frame[2][2]);
        }
        let dir_old = dir_new;
        let mn = cloth.verts[spring.mn].xrest;
        let kl = cloth.verts[spring.kl].xrest;
        dir_new =
            (Vec3::new(mn[0], mn[1], mn[2]) - Vec3::new(kl[0], kl[1], kl[2])).normalized();
        /* `mul_transposed_m3_v3` — rest_target = frameᵀ·dir. */
        let mut rt = Vec3::ZERO;
        for i in 0..3 {
            rt[i] =
                hair_frame[0][i] * dir_new.x + hair_frame[1][i] * dir_new.y + hair_frame[2][i] * dir_new.z;
        }
        hairdata[spring.kl].rest_target = [rt.x, rt.y, rt.z];
        cloth_parallel_transport_hair_frame(&mut hair_frame, dir_old, dir_new);
        prev_mn = spring.mn as i64;
    }
}

/// `cloth_update_springs` — stiffness refresh + GOAL enable.
pub fn cloth_update_springs(
    cloth: &mut Cloth,
    hairdata: &mut [ClothHairData],
    sim_parms: &ClothSimSettings,
) {
    for s in 0..cloth.springs.len() {
        let spring = &mut cloth.springs[s];
        spring.lin_stiffness = 0.0;
        if sim_parms.bending_model as i32 == CLOTH_BENDING_ANGULAR
            && spring.type_ & CLOTH_SPRING_TYPE_BENDING != 0
        {
            spring.ang_stiffness = (cloth.verts[spring.kl].bend_stiff
                + cloth.verts[spring.ij].bend_stiff)
                / 2.0;
        }
        if spring.type_ & CLOTH_SPRING_TYPE_STRUCTURAL != 0 {
            spring.lin_stiffness = (cloth.verts[spring.kl].struct_stiff
                + cloth.verts[spring.ij].struct_stiff)
                / 2.0;
        } else if spring.type_ & CLOTH_SPRING_TYPE_SHEAR != 0 {
            spring.lin_stiffness = (cloth.verts[spring.kl].shear_stiff
                + cloth.verts[spring.ij].shear_stiff)
                / 2.0;
        } else if spring.type_ == CLOTH_SPRING_TYPE_BENDING {
            spring.lin_stiffness = (cloth.verts[spring.kl].bend_stiff
                + cloth.verts[spring.ij].bend_stiff)
                / 2.0;
        } else if spring.type_ & CLOTH_SPRING_TYPE_INTERNAL != 0 {
            spring.lin_stiffness = (cloth.verts[spring.kl].internal_stiff
                + cloth.verts[spring.ij].internal_stiff)
                / 2.0;
        } else if spring.type_ == CLOTH_SPRING_TYPE_BENDING_HAIR {
            if !hairdata.is_empty() {
                cloth.verts[spring.ij].bend_stiff = hairdata[spring.ij].bending_stiffness;
                cloth.verts[spring.kl].bend_stiff = hairdata[spring.kl].bending_stiffness;
            }
            spring.lin_stiffness = (cloth.verts[spring.ij].bend_stiff
                + cloth.verts[spring.kl].bend_stiff)
                / 2.0;
        } else if spring.type_ == CLOTH_SPRING_TYPE_GOAL {
            if cloth.verts[spring.ij].flags & CLOTH_VERT_FLAG_PINNED == 0
                && cloth.verts[spring.ij].goal > 1e-6
            {
                spring.flags &= !CLOTH_SPRING_FLAG_DEACTIVATE;
            } else {
                spring.flags |= CLOTH_SPRING_FLAG_DEACTIVATE;
            }
        }
    }
    cloth_hair_update_bending_targets(cloth, hairdata);
}

/// `cloth_update_verts` — `xrest` from world-space positions.
pub fn cloth_update_verts(cloth: &mut Cloth, positions_world: &[Vec3]) {
    for (i, v) in cloth.verts.iter_mut().enumerate() {
        let p = positions_world[i];
        v.xrest = [p.x, p.y, p.z];
    }
}

/// `cloth_update_spring_lengths` — restlen/restang refresh.
pub fn cloth_update_spring_lengths(cloth: &mut Cloth, sim_parms: &mut ClothSimSettings) {
    sim_parms.avg_spring_len = 0.0;
    for v in &mut cloth.verts {
        v.avg_spring_len = 0.0;
    }
    let mut struct_springs = 0;
    for s in 0..cloth.springs.len() {
        let (stype, sij, skl, spa, spb) = {
            let sp = &cloth.springs[s];
            (sp.type_, sp.ij, sp.kl, sp.pa.clone(), sp.pb.clone())
        };
        if stype != CLOTH_SPRING_TYPE_SEWING {
            let shrink_factor = if stype
                & (CLOTH_SPRING_TYPE_STRUCTURAL
                    | CLOTH_SPRING_TYPE_SHEAR
                    | CLOTH_SPRING_TYPE_BENDING
                    | CLOTH_SPRING_TYPE_INTERNAL)
                != 0
            {
                cloth_shrink_factor(sim_parms, &cloth.verts, sij, skl)
            } else {
                1.0
            };
            let a = cloth.verts[skl].xrest;
            let b = cloth.verts[sij].xrest;
            let len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2))
                .sqrt()
                * shrink_factor;
            cloth.springs[s].restlen = len;
            if stype & CLOTH_SPRING_TYPE_BENDING != 0 {
                cloth.springs[s].restang = cloth_spring_angle(&cloth.verts, sij, skl, &spa, &spb);
            }
        }
        if stype & CLOTH_SPRING_TYPE_STRUCTURAL != 0 {
            sim_parms.avg_spring_len += cloth.springs[s].restlen;
            cloth.verts[sij].avg_spring_len += cloth.springs[s].restlen;
            cloth.verts[skl].avg_spring_len += cloth.springs[s].restlen;
            struct_springs += 1;
        }
    }
    if struct_springs > 0 {
        sim_parms.avg_spring_len /= struct_springs as f32;
    }
    for v in &mut cloth.verts {
        if v.spring_count > 0 {
            v.avg_spring_len = v.avg_spring_len * 0.49 / v.spring_count as f32;
        }
    }
}

/// `cloth_clear_cache` — via injected `PointCache` ops.
pub fn cloth_clear_cache(edit_active: bool, clear_after: &mut dyn FnMut(f32), framenr: f32) {
    if edit_active {
        return;
    }
    clear_after(framenr);
}

/// `cloth_free_modifier` — drop sim state; `extern` variant keeps
/// `sim_parms`/`coll_parms` (both caller-owned here).
pub fn cloth_free_modifier(cloth: &mut Option<Cloth>) {
    *cloth = None;
}
/// `cloth_free_modifier_extern`.
pub fn cloth_free_modifier_extern(cloth: &mut Option<Cloth>) {
    cloth_free_modifier(cloth);
}
