//! Port of `intern/shrinkwrap.cc` — boundary data
//! (`merge_vert_dir`, `shrinkwrap_build_boundary_data`), the
//! target-project Newton solver (`target_project_tri_*`,
//! `target_project_solve_point_tri`, `update_hit`,
//! `target_project_edge`), `BKE_shrinkwrap_compute_smooth_normal`,
//! `shrinkwrap_snap_with_side`,
//! `BKE_shrinkwrap_snap_point_to_surface`,
//! `BKE_shrinkwrap_find_nearest_surface` (BVH queries injected),
//! `BKE_shrinkwrap_needs_normals`.
//! `PORT_PENDING`: `ShrinkwrapCalcData` driver loops, ImBuf/modifier
//! entry points, `BKE_shrinkwrap_init_tree` (BVH build).

use blenders_math::geom::interp_weights_tri;
use blenders_math::vector::{interp_v3_weighted3, Vec3};
use blenders_math::vector_inline::len_manhattan_v3;

/* `DNA_modifier_enums.h` shrinkwrap enums. */
/// `MOD_SHRINKWRAP_NEAREST_SURFACE`.
pub const MOD_SHRINKWRAP_NEAREST_SURFACE: i16 = 0;
/// `MOD_SHRINKWRAP_PROJECT`.
pub const MOD_SHRINKWRAP_PROJECT: i16 = 1;
/// `MOD_SHRINKWRAP_NEAREST_VERTEX`.
pub const MOD_SHRINKWRAP_NEAREST_VERTEX: i16 = 2;
/// `MOD_SHRINKWRAP_TARGET_PROJECT`.
pub const MOD_SHRINKWRAP_TARGET_PROJECT: i16 = 3;

/// `MOD_SHRINKWRAP_ON_SURFACE`.
pub const MOD_SHRINKWRAP_ON_SURFACE: i8 = 0;
/// `MOD_SHRINKWRAP_INSIDE`.
pub const MOD_SHRINKWRAP_INSIDE: i8 = 1;
/// `MOD_SHRINKWRAP_OUTSIDE`.
pub const MOD_SHRINKWRAP_OUTSIDE: i8 = 2;
/// `MOD_SHRINKWRAP_OUTSIDE_SURFACE`.
pub const MOD_SHRINKWRAP_OUTSIDE_SURFACE: i8 = 3;
/// `MOD_SHRINKWRAP_ABOVE_SURFACE`.
pub const MOD_SHRINKWRAP_ABOVE_SURFACE: i8 = 4;

/// `MOD_SHRINKWRAP_PROJECT_ALLOW_POS_DIR`.
pub const MOD_SHRINKWRAP_PROJECT_ALLOW_POS_DIR: i8 = 1 << 0;
/// `MOD_SHRINKWRAP_PROJECT_ALLOW_NEG_DIR`.
pub const MOD_SHRINKWRAP_PROJECT_ALLOW_NEG_DIR: i8 = 1 << 1;
/// `MOD_SHRINKWRAP_CULL_TARGET_FRONTFACE`.
pub const MOD_SHRINKWRAP_CULL_TARGET_FRONTFACE: i8 = 1 << 3;
/// `MOD_SHRINKWRAP_CULL_TARGET_BACKFACE`.
pub const MOD_SHRINKWRAP_CULL_TARGET_BACKFACE: i8 = 1 << 4;
/// `MOD_SHRINKWRAP_KEEP_ABOVE_SURFACE`.
pub const MOD_SHRINKWRAP_KEEP_ABOVE_SURFACE: i8 = 1 << 5;
/// `MOD_SHRINKWRAP_INVERT_VGROUP`.
pub const MOD_SHRINKWRAP_INVERT_VGROUP: i8 = 1 << 6;
/// `MOD_SHRINKWRAP_INVERT_CULL_TARGET`.
pub const MOD_SHRINKWRAP_INVERT_CULL_TARGET: i8 = 1 << 7;
/// `MOD_SHRINKWRAP_CULL_TARGET_MASK`.
pub const MOD_SHRINKWRAP_CULL_TARGET_MASK: i8 =
    MOD_SHRINKWRAP_CULL_TARGET_FRONTFACE | MOD_SHRINKWRAP_CULL_TARGET_BACKFACE;

/// `MOD_SHRINKWRAP_PROJECT_OVER_NORMAL`.
pub const MOD_SHRINKWRAP_PROJECT_OVER_NORMAL: i8 = 0;
/// `MOD_SHRINKWRAP_PROJECT_OVER_X_AXIS`.
pub const MOD_SHRINKWRAP_PROJECT_OVER_X_AXIS: i8 = 1 << 0;
/// `MOD_SHRINKWRAP_PROJECT_OVER_Y_AXIS`.
pub const MOD_SHRINKWRAP_PROJECT_OVER_Y_AXIS: i8 = 1 << 1;
/// `MOD_SHRINKWRAP_PROJECT_OVER_Z_AXIS`.
pub const MOD_SHRINKWRAP_PROJECT_OVER_Z_AXIS: i8 = 1 << 2;

/// `BKE_shrinkwrap_needs_normals`.
pub fn bke_shrinkwrap_needs_normals(shrink_type: i16, shrink_mode: i8) -> bool {
    shrink_type == MOD_SHRINKWRAP_TARGET_PROJECT || shrink_mode == MOD_SHRINKWRAP_ABOVE_SURFACE
}

/// `ShrinkwrapBoundaryVertData`.
#[derive(Clone, Copy, Default)]
pub struct ShrinkwrapBoundaryVertData {
    /// `direction` — avg direction of edges meeting here.
    pub direction: Vec3,
    /// `normal_plane` — dir orthogonal to vert normal.
    pub normal_plane: Vec3,
}

/// `ShrinkwrapBoundaryData`.
#[derive(Default)]
pub struct ShrinkwrapBoundaryData {
    /// `edge_is_boundary` bitmask (as bool vec).
    pub edge_is_boundary: Vec<bool>,
    /// `tri_has_boundary` bitmask.
    pub tri_has_boundary: Vec<bool>,
    /// `vert_boundary_id` → index in `boundary_verts` or -1.
    pub vert_boundary_id: Vec<i32>,
    /// `boundary_verts`.
    pub boundary_verts: Vec<ShrinkwrapBoundaryVertData>,
}

impl ShrinkwrapBoundaryData {
    /// `has_boundary`.
    pub fn has_boundary(&self) -> bool {
        !self.edge_is_boundary.is_empty()
    }
}

/// `merge_vert_dir`.
fn merge_vert_dir(
    vdata: &mut [ShrinkwrapBoundaryVertData],
    status: &mut [i8],
    index: usize,
    edge_dir: Vec3,
    side: i8,
) {
    let direction = &mut vdata[index].direction;
    if if status[index] >= 0 {
        status[index] == side
    } else {
        direction.dot(edge_dir) < 0.0
    } {
        *direction = *direction - edge_dir;
    } else {
        *direction = *direction + edge_dir;
    }
    status[index] = if status[index] == 0 { side } else { -1 };
}

/// `mesh::corner_tri_get_real_edges` — real edge per tri side or -1.
fn corner_tri_get_real_edges(
    edges: &[[i32; 2]],
    corner_verts: &[i32],
    corner_edges: &[i32],
    tri_corners: [i32; 3],
) -> [i32; 3] {
    let mut real_edges = [-1i32; 3];
    for i in 0..3 {
        let c = tri_corners[i] as usize;
        let e = corner_edges[c] as usize;
        let v0 = corner_verts[c];
        let en = edges[e];
        if en[0] == v0 || en[1] == v0 {
            real_edges[i] = e as i32;
        }
    }
    real_edges
}

/// `shrinkwrap_build_boundary_data` — boundary edges + per-vert
/// tangent plane data.
pub fn shrinkwrap_build_boundary_data(
    positions: &[Vec3],
    edges: &[[i32; 2]],
    corner_verts: &[i32],
    corner_edges: &[i32],
    corner_tris: &[[i32; 3]],
    vert_normals: &[Vec3],
) -> ShrinkwrapBoundaryData {
    let mut edge_mode = vec![0i8; edges.len()];
    for &e in corner_edges {
        let e = e as usize;
        if edge_mode[e] < 2 {
            edge_mode[e] += 1;
        }
    }
    let mut edge_is_boundary = vec![false; edges.len()];
    let mut num_boundary_edges = 0;
    for i in 0..edges.len() {
        if edge_mode[i] == 1 {
            edge_is_boundary[i] = true;
            num_boundary_edges += 1;
        }
    }
    if num_boundary_edges == 0 {
        return ShrinkwrapBoundaryData::default();
    }
    let mut data = ShrinkwrapBoundaryData::default();
    let mut tri_has_boundary = vec![false; corner_tris.len()];
    for (i, tri) in corner_tris.iter().enumerate() {
        let real_edges =
            corner_tri_get_real_edges(edges, corner_verts, corner_edges, *tri);
        for j in 0..3 {
            if real_edges[j] >= 0 && edge_is_boundary[real_edges[j] as usize] {
                tri_has_boundary[i] = true;
                break;
            }
        }
    }
    let mut vert_boundary_id = vec![0i32; positions.len()];
    for (i, &b) in edge_is_boundary.iter().enumerate() {
        if b {
            let edge = edges[i];
            vert_boundary_id[edge[0] as usize] = 1;
            vert_boundary_id[edge[1] as usize] = 1;
        }
    }
    let mut boundary_verts_num = 0i32;
    for id in vert_boundary_id.iter_mut() {
        *id = if *id != 0 {
            let r = boundary_verts_num;
            boundary_verts_num += 1;
            r
        } else {
            -1
        };
    }
    let mut boundary_verts =
        vec![ShrinkwrapBoundaryVertData::default(); boundary_verts_num as usize];
    let mut vert_status = vec![0i8; boundary_verts_num as usize];
    for (i, &b) in edge_is_boundary.iter().enumerate() {
        if b {
            let edge = edges[i];
            let dir = (positions[edge[1] as usize] - positions[edge[0] as usize])
                .normalized();
            merge_vert_dir(
                &mut boundary_verts,
                &mut vert_status,
                vert_boundary_id[edge[0] as usize] as usize,
                dir,
                1,
            );
            merge_vert_dir(
                &mut boundary_verts,
                &mut vert_status,
                vert_boundary_id[edge[1] as usize] as usize,
                dir,
                2,
            );
        }
    }
    for i in 0..positions.len() {
        let bidx = vert_boundary_id[i];
        if bidx >= 0 {
            let vdata = &mut boundary_verts[bidx as usize];
            vdata.direction = vdata.direction.normalized();
            let tmp = vert_normals[i].cross(vdata.direction);
            vdata.normal_plane = tmp.cross(vert_normals[i]).normalized();
        }
    }
    data.edge_is_boundary = edge_is_boundary;
    data.tri_has_boundary = tri_has_boundary;
    data.vert_boundary_id = vert_boundary_id;
    data.boundary_verts = boundary_verts;
    data
}

/* ---- target-project Newton solver ---- */

/// `TargetProjectTriData`.
struct TargetProjectTriData {
    vtri_co: [Vec3; 3],
    vtri_no: [Vec3; 3],
    point_co: Vec3,
    n0_minus_n2: Vec3,
    n1_minus_n2: Vec3,
    c0_minus_c2: Vec3,
    c1_minus_c2: Vec3,
}

/// `target_project_tri_deviation` — returns (delta, co_interp, no_interp).
fn target_project_tri_deviation(
    data: &TargetProjectTriData,
    x: &[f32; 3],
) -> ([f32; 3], Vec3, Vec3) {
    let w = Vec3::new(x[0], x[1], 1.0 - x[0] - x[1]);
    let co_interp = interp_v3_weighted3(data.vtri_co[0], data.vtri_co[1], data.vtri_co[2], w);
    let no_interp = interp_v3_weighted3(data.vtri_no[0], data.vtri_no[1], data.vtri_no[2], w);
    let d = co_interp + no_interp * x[2] - data.point_co;
    ([d.x, d.y, d.z], co_interp, no_interp)
}

/// `target_project_tri_jacobian`.
fn target_project_tri_jacobian(
    data: &TargetProjectTriData,
    x: &[f32; 3],
) -> [[f32; 3]; 3] {
    let j0 = data.c0_minus_c2 + data.n0_minus_n2 * x[2];
    let j1 = data.c1_minus_c2 + data.n1_minus_n2 * x[2];
    let j2 = data.vtri_no[2] + data.n0_minus_n2 * x[0] + data.n1_minus_n2 * x[1];
    [[j0.x, j0.y, j0.z], [j1.x, j1.y, j1.z], [j2.x, j2.y, j2.z]]
}

/// `target_project_tri_clamp`.
fn target_project_tri_clamp(x: &mut [f32; 3]) {
    x[0] = x[0].max(0.0);
    x[1] = x[1].max(0.0);
    if x[0] + x[1] > 1.0 {
        x[0] = x[0] / (x[0] + x[1]);
        x[1] = 1.0 - x[0];
    }
}

/// `target_project_tri_correct`.
fn target_project_tri_correct(x: &[f32; 3], step: &mut [f32; 3], x_next: &mut [f32; 3]) -> bool {
    let epsilon = 1e-5f32;
    let dir_epsilon = 0.5f32;
    let mut fixed = false;
    let mut locked = false;
    let sum = x[0] + x[1];
    let sstep = -(step[0] + step[1]);
    if sum + sstep > 1.0 {
        let ldist = 1.0 - sum;
        if ldist < epsilon * std::f32::consts::SQRT_2 {
            let step_len = (step[0] * step[0] + step[1] * step[1]).sqrt();
            if step_len > epsilon && sstep > step_len * dir_epsilon * std::f32::consts::SQRT_2 {
                return false;
            }
            step[0] += (sum + sstep - 1.0) * 0.5;
            step[1] += (sum + sstep - 1.0) * 0.5;
            fixed = true;
            locked = true;
        } else {
            let f = ldist / sstep;
            step[0] *= f;
            step[1] *= f;
            step[2] *= f;
            fixed = true;
        }
    }
    for i in 0..2 {
        if step[i] > x[i] {
            if x[i] < epsilon {
                let step_len = (step[0] * step[0] + step[1] * step[1]).sqrt();
                if step_len > epsilon && (locked || step[i] > step_len * dir_epsilon) {
                    return false;
                }
                step[i] = x[i];
                fixed = true;
            } else {
                let f = x[i] / step[i];
                step[0] *= f;
                step[1] *= f;
                step[2] *= f;
                fixed = true;
            }
        }
    }
    if fixed {
        x_next[0] = x[0] - step[0];
        x_next[1] = x[1] - step[1];
        x_next[2] = x[2] - step[2];
        target_project_tri_clamp(x_next);
    }
    true
}

/// `target_project_solve_point_tri` — Newton-solve for the nearest
/// point on the offset triangle surface. Returns
/// (hit_co, hit_no) on success.
#[allow(clippy::too_many_arguments)]
pub fn target_project_solve_point_tri(
    vtri_co: &[Vec3; 3],
    vtri_no: &[Vec3; 3],
    point_co: Vec3,
    hit_co: Vec3,
    hit_dist_sq: f32,
) -> Option<(Vec3, Vec3)> {
    let dist = hit_dist_sq.sqrt();
    let magnitude_estimate = dist
        + len_manhattan_v3([vtri_co[0].x, vtri_co[0].y, vtri_co[0].z])
        + len_manhattan_v3([vtri_co[1].x, vtri_co[1].y, vtri_co[1].z])
        + len_manhattan_v3([vtri_co[2].x, vtri_co[2].y, vtri_co[2].z]);
    let epsilon = magnitude_estimate * 1.0e-6;
    let w = interp_weights_tri(vtri_co[0], vtri_co[1], vtri_co[2], hit_co);
    let mut x = [w.x, w.y, 0.0];
    let r_hit_no = interp_v3_weighted3(vtri_no[0], vtri_no[1], vtri_no[2], w);
    let tmp = point_co - hit_co;
    x[2] = if tmp.dot(r_hit_no) < 0.0 { -dist } else { dist };
    let tri_data = TargetProjectTriData {
        vtri_co: *vtri_co,
        vtri_no: *vtri_no,
        point_co,
        n0_minus_n2: vtri_no[0] - vtri_no[2],
        n1_minus_n2: vtri_no[1] - vtri_no[2],
        c0_minus_c2: vtri_co[0] - vtri_co[2],
        c1_minus_c2: vtri_co[1] - vtri_co[2],
    };
    target_project_tri_clamp(&mut x);
    let mut result = [0.0; 3];
    let ok = blenders_bli::math_solvers::newton3d_solve(
        |xx| target_project_tri_deviation(&tri_data, xx).0,
        |xx| target_project_tri_jacobian(&tri_data, xx),
        |xx, step, x_next| target_project_tri_correct(xx, step, x_next),
        epsilon,
        20,
        false,
        x,
        &mut result,
    );
    if ok {
        let (_, co_interp, no_interp) = target_project_tri_deviation(&tri_data, &result);
        Some((co_interp, no_interp))
    } else {
        None
    }
}

/// `BVHTreeNearest` (local copy of the search result state).
#[derive(Clone, Copy)]
pub struct ShrinkwrapNearest {
    /// `index` (-1 = none).
    pub index: i32,
    /// `dist_sq`.
    pub dist_sq: f32,
    /// `co`.
    pub co: Vec3,
    /// `no`.
    pub no: Vec3,
}

impl Default for ShrinkwrapNearest {
    fn default() -> Self {
        Self {
            index: -1,
            dist_sq: f32::MAX,
            co: Vec3::ZERO,
            no: Vec3::ZERO,
        }
    }
}

/// `update_hit`.
pub fn update_hit(
    nearest: &mut ShrinkwrapNearest,
    index: i32,
    co: Vec3,
    hit_co: Vec3,
    hit_no: Vec3,
) -> bool {
    let dist_sq = hit_co.distance_squared(co);
    if dist_sq < nearest.dist_sq {
        nearest.index = index;
        nearest.dist_sq = dist_sq;
        nearest.co = hit_co;
        nearest.no = hit_no.normalized();
        return true;
    }
    false
}

/// `target_project_edge` — boundary-edge quadratic solve; `index`
/// identifies the candidate element for `nearest`.
#[allow(clippy::too_many_arguments)]
pub fn target_project_edge(
    boundary: &ShrinkwrapBoundaryData,
    edges: &[[i32; 2]],
    vert_positions: &[Vec3],
    vert_normals: &[Vec3],
    index: i32,
    co: Vec3,
    nearest: &mut ShrinkwrapNearest,
    eidx: usize,
) {
    let edge = edges[eidx];
    let vedge_co = [vert_positions[edge[0] as usize], vert_positions[edge[1] as usize]];
    let bid1 = boundary.vert_boundary_id[edge[0] as usize];
    let bid2 = boundary.vert_boundary_id[edge[1] as usize];
    if bid1 < 0 || bid2 < 0 {
        return;
    }
    let mut vedge_dir = [
        boundary.boundary_verts[bid1 as usize].normal_plane,
        boundary.boundary_verts[bid2 as usize].normal_plane,
    ];
    let dir = vedge_co[1] - vedge_co[0];
    if boundary.boundary_verts[bid1 as usize].direction.dot(dir) < 0.0 {
        vedge_dir[0] = -vedge_dir[0];
    }
    if boundary.boundary_verts[bid2 as usize].direction.dot(dir) < 0.0 {
        vedge_dir[1] = -vedge_dir[1];
    }
    let d0v0 = vedge_dir[0].dot(vedge_co[0]);
    let d0v1 = vedge_dir[0].dot(vedge_co[1]);
    let d1v0 = vedge_dir[1].dot(vedge_co[0]);
    let d1v1 = vedge_dir[1].dot(vedge_co[1]);
    let d0co = vedge_dir[0].dot(co);
    let a = d0v1 - d0v0 + d1v0 - d1v1;
    if a != 0.0 {
        let b = 2.0 * d0v0 - d0v1 - d0co - d1v0 + vedge_dir[1].dot(co);
        let c = d0co - d0v0;
        let det = b * b - 4.0 * a * c;
        if det >= 0.0 {
            let epsilon = 1e-6f32;
            let sdet = det.sqrt();
            let mut i = if det > 0.0 { 2 } else { 0 };
            while i >= 0 {
                let mut x = (-b + (i - 1) as f32 * sdet) / (2.0 * a);
                if x >= -epsilon && x <= 1.0 + epsilon {
                    x = x.clamp(0.0, 1.0);
                    let vedge_no = [
                        vert_normals[edge[0] as usize],
                        vert_normals[edge[1] as usize],
                    ];
                    let hit_co = vedge_co[0] + (vedge_co[1] - vedge_co[0]) * x;
                    let hit_no = vedge_no[0] + (vedge_no[1] - vedge_no[0]) * x;
                    update_hit(nearest, index, co, hit_co, hit_no);
                }
                i -= 2;
            }
        }
    }
}

/// `BKE_shrinkwrap_compute_smooth_normal` — interpolated normal at
/// hit point (corner normals or vert normals; sharp faces keep the
/// face normal).
#[allow(clippy::too_many_arguments)]
pub fn bke_shrinkwrap_compute_smooth_normal(
    corner_tris: &[[i32; 3]],
    corner_tri_faces: &[i32],
    sharp_faces: Option<&[bool]>,
    corner_verts: &[i32],
    corner_normals: &[Vec3],
    vert_normals: &[Vec3],
    face_normals: &[Vec3],
    vert_positions: &[Vec3],
    corner_tri_idx: usize,
    hit_co: Vec3,
    hit_no: Vec3,
    transform_apply: Option<&dyn Fn(Vec3) -> Vec3>,
    transform_invert_normal: Option<&dyn Fn(Vec3) -> Vec3>,
) -> Vec3 {
    let tri = corner_tris[corner_tri_idx];
    let face_i = corner_tri_faces[corner_tri_idx] as usize;
    if sharp_faces.map(|s| s[face_i]).unwrap_or(true) {
        let vert_indices = [
            corner_verts[tri[0] as usize] as usize,
            corner_verts[tri[1] as usize] as usize,
            corner_verts[tri[2] as usize] as usize,
        ];
        let no = if !corner_normals.is_empty() {
            [
                corner_normals[tri[0] as usize],
                corner_normals[tri[1] as usize],
                corner_normals[tri[2] as usize],
            ]
        } else {
            [
                vert_normals[vert_indices[0]],
                vert_normals[vert_indices[1]],
                vert_normals[vert_indices[2]],
            ]
        };
        let mut tmp_co = hit_co;
        if let Some(t) = transform_apply {
            tmp_co = t(tmp_co);
        }
        let w = interp_weights_tri(
            vert_positions[vert_indices[0]],
            vert_positions[vert_indices[1]],
            vert_positions[vert_indices[2]],
            tmp_co,
        );
        let mut r_no = interp_v3_weighted3(no[0], no[1], no[2], w);
        if let Some(t) = transform_invert_normal {
            r_no = t(r_no);
        } else {
            r_no = r_no.normalized();
        }
        r_no
    } else if !face_normals.is_empty() {
        let mut r_no = face_normals[face_i];
        if let Some(t) = transform_invert_normal {
            r_no = t(r_no);
        }
        r_no
    } else {
        hit_no
    }
}

/// `shrinkwrap_snap_with_side`.
fn shrinkwrap_snap_with_side(
    r_point_co: &mut Vec3,
    point_co: Vec3,
    hit_co: Vec3,
    hit_no: Vec3,
    goal_dist: f32,
    forcesign: f32,
    forcesnap: bool,
) {
    let delta = point_co - hit_co;
    let dist = delta.length();
    if dist < f32::EPSILON {
        if forcesnap || goal_dist > 0.0 {
            *r_point_co = hit_co + hit_no * (goal_dist * forcesign);
        } else {
            *r_point_co = hit_co;
        }
    } else {
        let dsign = if delta.dot(hit_no) >= 0.0 { 1.0 } else { -1.0 };
        let forcesign = if forcesign == 0.0 { dsign } else { forcesign };
        if forcesnap || dsign * dist * forcesign < goal_dist {
            let mut delta = delta * (dsign / dist);
            let dist_epsilon =
                (goal_dist.abs() + len_manhattan_v3([hit_co.x, hit_co.y, hit_co.z])) * 1e-4;
            if dist < dist_epsilon {
                delta = hit_no + (delta - hit_no) * (dist / dist_epsilon);
            }
            *r_point_co = hit_co + delta * (goal_dist * forcesign);
        } else {
            *r_point_co = point_co;
        }
    }
}

/// `BKE_shrinkwrap_snap_point_to_surface`.
#[allow(clippy::too_many_arguments)]
pub fn bke_shrinkwrap_snap_point_to_surface(
    mode: i8,
    smooth_normal: Option<Vec3>,
    hit_co: Vec3,
    hit_no: Vec3,
    goal_dist: f32,
    point_co: Vec3,
) -> Vec3 {
    let mut r_point_co = hit_co;
    match mode {
        m if m == MOD_SHRINKWRAP_ON_SURFACE => {
            if goal_dist != 0.0 {
                shrinkwrap_snap_with_side(
                    &mut r_point_co, point_co, hit_co, hit_no, goal_dist, 0.0, true,
                );
            }
        }
        m if m == MOD_SHRINKWRAP_INSIDE => {
            shrinkwrap_snap_with_side(
                &mut r_point_co, point_co, hit_co, hit_no, goal_dist, -1.0, false,
            );
        }
        m if m == MOD_SHRINKWRAP_OUTSIDE => {
            shrinkwrap_snap_with_side(
                &mut r_point_co, point_co, hit_co, hit_no, goal_dist, 1.0, false,
            );
        }
        m if m == MOD_SHRINKWRAP_OUTSIDE_SURFACE => {
            if goal_dist != 0.0 {
                shrinkwrap_snap_with_side(
                    &mut r_point_co, point_co, hit_co, hit_no, goal_dist, 1.0, true,
                );
            }
        }
        m if m == MOD_SHRINKWRAP_ABOVE_SURFACE => {
            if goal_dist != 0.0 {
                let n = smooth_normal.unwrap_or(hit_no);
                r_point_co = hit_co + n * goal_dist;
            }
        }
        _ => {}
    }
    r_point_co
}

/// `BKE_shrinkwrap_find_nearest_surface` — `find_nearest` callback
/// injected: `(co, nearest, type)` performing the BVH query
/// (nearest-vertex vs triangle-project dispatch kept here).
pub fn bke_shrinkwrap_find_nearest_surface(
    type_: i16,
    co: Vec3,
    nearest: &mut ShrinkwrapNearest,
    nearest_vertex_cb: &dyn Fn(Vec3, &mut ShrinkwrapNearest),
    tri_nearest_cb: &dyn Fn(Vec3, &mut ShrinkwrapNearest),
    tri_project_cb: &dyn Fn(Vec3, &mut ShrinkwrapNearest),
) {
    if type_ == MOD_SHRINKWRAP_NEAREST_VERTEX {
        nearest_vertex_cb(co, nearest);
        return;
    }
    if type_ == MOD_SHRINKWRAP_TARGET_PROJECT {
        tri_project_cb(co, nearest);
        if nearest.index < 0 {
            tri_nearest_cb(co, nearest);
        }
    } else {
        tri_nearest_cb(co, nearest);
    }
}
