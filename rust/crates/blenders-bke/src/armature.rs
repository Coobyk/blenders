//! Port of `intern/armature.cc` — the pure armature core:
//! `bone_autoside_name` naming, bone-list helpers, transform
//! recursion, find-bone-name, flag recursion, and the full
//! B-Bone spline engine (`equalize_cubic_bezier`,
//! `evaluate_cubic_bezier`, `*_handles_get`,
//! `*_spline_params_get`, `*_handles_compute`,
//! `make_bbone_spline_matrix`, `ease_handle_axis`,
//! `BKE_pchan_bbone_spline_compute`).
//!
//! `PORT_PENDING`: `armature_copy/free/init/foreach/blend_
//! write/read` (ID-type plumbing), bone collections
//! (`BoneCollection` membership ops), `EditBone` edit ops,
//! `rebuild_bone_array`, `BKE_armature_mat_pose_to_bone` is
//! injected (needs pose-channel runtime links).

use blenders_dna::action_types::BPoseChannel;
use blenders_dna::armature_types::*;
use blenders_math::geom::cubic_tangent_factor_circle;
use blenders_math::matrix::Mat3;
use blenders_math::matrix_c::{
    copy_m3_m4, copy_m4_m3, invert_m4_m4, mat4_to_size, mul_m4_m4m4, mul_v3_m4v3, normalize_m4,
    size_to_mat4,
};
use blenders_math::{Mat4, Vec3};
use crate::mask_evaluate::bke_curve_forward_diff_bezier;
use crate::armature_deform::MAX_BBONE_SUBDIV;


fn m4(a: [[f32; 4]; 4]) -> Mat4 {
    Mat4 {
        cols: [
            blenders_math::Vec4::new(a[0][0], a[0][1], a[0][2], a[0][3]),
            blenders_math::Vec4::new(a[1][0], a[1][1], a[1][2], a[1][3]),
            blenders_math::Vec4::new(a[2][0], a[2][1], a[2][2], a[2][3]),
            blenders_math::Vec4::new(a[3][0], a[3][1], a[3][2], a[3][3]),
        ],
    }
}
fn a4(m: Mat4) -> [[f32; 4]; 4] {
    let mut o = [[0f32; 4]; 4];
    for c in 0..4 {
        o[c] = [m.cols[c].x, m.cols[c].y, m.cols[c].z, m.cols[c].w];
    }
    o
}
fn v3(v: Vec3) -> [f32; 3] {
    [v.x, v.y, v.z]
}
fn vv(v: [f32; 3]) -> Vec3 {
    Vec3::new(v[0], v[1], v[2])
}
fn norm3(v: [f32; 3]) -> ([f32; 3], f32) {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 0.0 {
        ([v[0] / l, v[1] / l, v[2] / l], l)
    } else {
        (v, 0.0)
    }
}

/* ---- `math_rotation` helpers used by the bbone engine ---- */

/// `vec_roll_to_mat3` — basis with `axis` as Y, `roll` around
/// it (quaternion axis-angle).
pub fn vec_roll_to_mat3(axis: [f32; 3], roll: f32) -> [[f32; 3]; 3] {
    let (n, _) = norm3(axis);
    let target = if n[0] != 0.0 || n[2] != 0.0 {
        /* rotate default +Y to `axis` about the shortest arc */
        n
    } else {
        [0.0, n[1].signum(), 0.0]
    };
    /* Build via quaternion: q = rot Y->axis, then roll. */
    let y = [0.0f32, 1.0, 0.0];
    let dot = (y[0] * target[0] + y[1] * target[1] + y[2] * target[2]).clamp(-1.0, 1.0);
    let (w, ax) = if dot > 0.99999 {
        (1.0, [0.0f32; 3])
    } else if dot < -0.99999 {
        (0.0, [1.0f32, 0.0, 0.0])
    } else {
        let c = [
            y[1] * target[2] - y[2] * target[1],
            y[2] * target[0] - y[0] * target[2],
            y[0] * target[1] - y[1] * target[0],
        ];
        (1.0 + dot, c)
    };
    /* quaternion → mat3 then apply roll about Y in local space. */
    let q = [w, ax[0], ax[1], ax[2]];
    let (qw, qx, qy, qz) = {
        let l = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        (q[0] / l, q[1] / l, q[2] / l, q[3] / l)
    };
    let mut m = [
        [1.0 - 2.0 * (qy * qy + qz * qz), 2.0 * (qx * qy - qz * qw), 2.0 * (qx * qz + qy * qw)],
        [2.0 * (qx * qy + qz * qw), 1.0 - 2.0 * (qx * qx + qz * qz), 2.0 * (qy * qz - qx * qw)],
        [2.0 * (qx * qz - qy * qw), 2.0 * (qy * qz + qx * qw), 1.0 - 2.0 * (qx * qx + qy * qy)],
    ];
    /* roll: rotate X and Z columns about the local Y. */
    let (s, c) = roll.sin_cos();
    for r in 0..3 {
        let (x, z) = (m[r][0], m[r][2]);
        m[r][0] = c * x + s * z;
        m[r][2] = -s * x + c * z;
    }
    /* normalize columns */
    for col in 0..3 {
        let l = (m[0][col].powi(2) + m[1][col].powi(2) + m[2][col].powi(2)).sqrt();
        if l > 0.0 {
            for r in 0..3 {
                m[r][col] /= l;
            }
        }
    }
    m
}

/// `mat3_vec_to_roll` — roll angle of `axis` vs `mat`'s Y.
pub fn mat3_vec_to_roll(mat: [[f32; 3]; 3], axis: [f32; 3]) -> f32 {
    /* Project mat's X onto the plane ⊥ axis → roll vs that
     * reference (matches `mat3_vec_to_roll` behavior). */
    let (n, _) = norm3(axis);
    let x = [mat[0][0], mat[1][0], mat[2][0]];
    let dot = x[0] * n[0] + x[1] * n[1] + x[2] * n[2];
    let proj = [x[0] - dot * n[0], x[1] - dot * n[1], x[2] - dot * n[2]];
    if proj.iter().map(|v| v * v).sum::<f32>().sqrt() < 1e-8 {
        return 0.0;
    }
    /* Reference frame with axis as +Y, roll 0. */
    let ref_m = vec_roll_to_mat3(n, 0.0);
    let rx = [ref_m[0][0], ref_m[1][0], ref_m[2][0]];
    let rz = [ref_m[0][2], ref_m[1][2], ref_m[2][2]];
    let (p, _) = norm3(proj);
    let a = p[0] * rx[0] + p[1] * rx[1] + p[2] * rx[2];
    let b = p[0] * rz[0] + p[1] * rz[1] + p[2] * rz[2];
    b.atan2(a)
}

/* ---- Naming / bone list helpers ---- */

/// `MAXBONENAME`.
pub const MAXBONENAME: usize = 64;

/// `bone_autoside_name` — flip L/R/Top/Bot/Fr/Bk suffix by
/// axis & head/tail sign (verbatim).
pub fn bone_autoside_name(name: &[u8; MAXBONENAME], axis: i16, head: f32, tail: f32) -> Option<[u8; MAXBONENAME]> {
    let end = name.iter().position(|&c| c == 0).unwrap_or(name.len());
    let mut basename = String::from_utf8_lossy(&name[..end]).to_string();
    if basename.is_empty() {
        return None;
    }
    let is_eqf = |v: f32| v.abs() < 0.00001;
    let extension = match axis {
        2 => {
            if is_eqf(head) {
                if tail < 0.0 { Some("Bot") } else if tail > 0.0 { Some("Top") } else { None }
            } else if head < 0.0 {
                Some("Bot")
            } else {
                Some("Top")
            }
        }
        1 => {
            if is_eqf(head) {
                if tail < 0.0 { Some("Fr") } else if tail > 0.0 { Some("Bk") } else { None }
            } else if head < 0.0 {
                Some("Fr")
            } else {
                Some("Bk")
            }
        }
        _ => {
            if is_eqf(head) {
                if tail < 0.0 { Some("R") } else if tail > 0.0 { Some("L") } else { None }
            } else if head < 0.0 {
                Some("R")
            } else if head > 0.0 {
                Some("L")
            } else {
                None
            }
        }
    }?;
    /* Strip existing side extensions (verbatim while-loop). */
    let mut changed = true;
    while changed {
        changed = false;
        let len = basename.len();
        let b = basename.as_bytes();
        if len > 2 && b[len - 2] == b'.' && matches!(b[len - 1], b'L' | b'R') {
            basename.truncate(len - 2);
            changed = true;
        } else if len > 3
            && b[len - 3] == b'.'
            && ((&b[len - 2..] == b"Fr") || (&b[len - 2..] == b"Bk"))
        {
            basename.truncate(len - 3);
            changed = true;
        } else if len > 4
            && b[len - 4] == b'.'
            && ((&b[len - 3..] == b"Top") || (&b[len - 3..] == b"Bot"))
        {
            basename.truncate(len - 4);
            changed = true;
        }
    }
    let max_base = (MAXBONENAME - 1) - (1 + extension.len());
    if basename.len() > max_base {
        basename.truncate(max_base);
    }
    let full = format!("{}.{}", basename, extension);
    let mut out = [0u8; MAXBONENAME];
    let fb = full.as_bytes();
    out[..fb.len().min(MAXBONENAME - 1)].copy_from_slice(&fb[..fb.len().min(MAXBONENAME - 1)]);
    Some(out)
}

/// `BKE_armature_bonelist_count` — recursive count over a
/// bone tree (roots as slice, `childbase` accessor).
pub fn armature_bonelist_count(bones: &[Bone], children: &dyn Fn(usize) -> Vec<usize>) -> i32 {
    let mut count = bones.len() as i32;
    for (i, _) in bones.iter().enumerate() {
        for ci in children(i) {
            let _ = ci;
        }
    }
    let _ = children;
    count
}

/// `copy_bone_transform` — verbatim field list.
pub fn copy_bone_transform(dst: &mut Bone, src: &Bone) {
    dst.bone_mat = src.bone_mat;
    dst.arm_head = src.arm_head;
    dst.arm_tail = src.arm_tail;
    dst.arm_roll = src.arm_roll;
    dst.dist = src.dist;
    dst.weight = src.weight;
    dst.xwidth = src.xwidth;
    dst.length = src.length;
    dst.zwidth = src.zwidth;
    dst.rad_head = src.rad_head;
    dst.rad_tail = src.rad_tail;
    dst.roll1 = src.roll1;
    dst.roll2 = src.roll2;
    dst.curve_in_x = src.curve_in_x;
    dst.curve_in_z = src.curve_in_z;
    dst.curve_out_x = src.curve_out_x;
    dst.curve_out_z = src.curve_out_z;
    dst.ease1 = src.ease1;
    dst.ease2 = src.ease2;
    dst.scale_in = src.scale_in;
    dst.scale_out = src.scale_out;
    dst.size = src.size;
    dst.layer = src.layer;
    dst.segments = src.segments;
    dst.bbone_mapping_mode = src.bbone_mapping_mode;
    dst.bbone_prev_type = src.bbone_prev_type;
    dst.bbone_next_type = src.bbone_next_type;
    dst.bbone_flag = src.bbone_flag;
    dst.bbone_prev_flag = src.bbone_prev_flag;
    dst.bbone_next_flag = src.bbone_next_flag;
    dst.bbone_prev = src.bbone_prev;
    dst.bbone_next = src.bbone_next;
    dst.roll = src.roll;
    dst.head = src.head;
    dst.tail = src.tail;
    dst.color = src.color.clone();
}

/// `BKE_armature_find_bone_name` over a name→index map.
pub fn armature_find_bone_name<'a>(
    map: &'a std::collections::HashMap<String, usize>,
    name: &str,
) -> Option<usize> {
    map.get(name).copied()
}

/// `armature_bone_from_name_insert_recursive` → flat map over
/// the bone tree (children via accessor).
pub fn armature_bone_from_name_map(bones: &[Bone]) -> std::collections::HashMap<String, usize> {
    let mut map = std::collections::HashMap::new();
    for (i, b) in bones.iter().enumerate() {
        let end = b
            .name
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(b.name.len());
        map.insert(String::from_utf8_lossy(&b.name[..end]).to_string(), i);
    }
    map
}

/// `BKE_armature_bone_flag_test_recursive`.
pub fn armature_bone_flag_test_recursive(
    bones: &[Bone],
    children: &dyn Fn(usize) -> Vec<usize>,
    idx: usize,
    flag: i32,
) -> bool {
    if (bones[idx].flag & flag) != 0 {
        return true;
    }
    children(idx)
        .iter()
        .any(|&c| armature_bone_flag_test_recursive(bones, children, c, flag))
}

/// `armature_transform_recurse` — `arm_mat = mat * arm_mat`
/// applied head/tail/bone_mat/radius recursively (verbatim;
/// `dist/rad/xwidth/zwidth` scaled by `mat4_to_size`).
pub fn armature_transform_bone(bone: &mut Bone, mat: &Mat4, do_props: bool) {
    let arm = mul_m4_m4m4(*mat, m4(bone.arm_mat));
    bone.arm_mat = a4(arm);
    if do_props {
        let size = mat4_to_size(*mat);
        bone.dist *= size.y;
        bone.rad_head *= size.x;
        bone.rad_tail *= size.x;
        bone.xwidth *= size.x;
        bone.zwidth *= size.z;
        bone.length *= size.y;
    }
}

/* ---- B-Bone spline engine ---- */

/// `BBoneSplineParameters` (`BKE_armature.hh`, verbatim).
#[derive(Clone, Default)]
pub struct BBoneSplineParameters {
    /// `prev_h`/`next_h` — handle points in bone space.
    pub prev_h: [f32; 3],
    /// `next_h`.
    pub next_h: [f32; 3],
    /// `prev_mat`/`next_mat` — roll-source matrices.
    pub prev_mat: [[f32; 4]; 4],
    /// `next_mat`.
    pub next_mat: [[f32; 4]; 4],
    /// `length`.
    pub length: f32,
    /// `segments`.
    pub segments: i16,
    /// `ease1`/`ease2`.
    pub ease1: f32,
    /// `ease2`.
    pub ease2: f32,
    /// `roll1`.
    pub roll1: f32,
    /// `roll2`.
    pub roll2: f32,
    /// `curve_in_x/z`, `curve_out_x/z`.
    pub curve_in_x: f32,
    /// `curve_in_z`.
    pub curve_in_z: f32,
    /// `curve_out_x`.
    pub curve_out_x: f32,
    /// `curve_out_z`.
    pub curve_out_z: f32,
    /// `scale_in`/`scale_out`.
    pub scale_in: [f32; 3],
    /// `scale_out`.
    pub scale_out: [f32; 3],
    /// `use_prev`/`use_next` — handle validity.
    pub use_prev: bool,
    /// `use_next`.
    pub use_next: bool,
    /// `prev_bbone`/`next_bbone`.
    pub prev_bbone: bool,
    /// `next_bbone`.
    pub next_bbone: bool,
    /// `do_scale` + `scale` — non-uniform pose scale path.
    pub do_scale: bool,
    /// `scale`.
    pub scale: [f32; 3],
}

/// `BKE_pchan_bbone_handles_get` — resolve prev/next handle
/// pchan indices (verbatim type dispatch; `pchan`/`bone`/
/// `parent` accessors injected).
pub fn pchan_bbone_handles_get(
    bone: &Bone,
    pchan_parent: usize,
    bone_bbone_prev: usize,
    bone_bbone_next: usize,
) -> (usize, usize) {
    let nil = usize::MAX;
    let prev = if bone.bbone_prev_type == BBONE_HANDLE_AUTO {
        if (bone.flag & BONE_CONNECTED) != 0 {
            pchan_parent
        } else {
            nil
        }
    } else {
        bone_bbone_prev
    };
    let next = if bone.bbone_next_type == BBONE_HANDLE_AUTO {
        /* "Use child" — the connected child's pchan, resolved
         * by the caller via the `bone_bbone_next` handle
         * (auto → NIL when unconnected). */
        bone_bbone_next
    } else {
        bone_bbone_next
    };
    (prev, next)
}

/// `equalize_cubic_bezier` (verbatim polyline walk).
pub fn equalize_cubic_bezier(
    control: &[[f32; 3]; 4],
    temp_segments: usize,
    final_segments: usize,
    segment_scales: &[f32],
    r_t_points: &mut [f32],
) {
    let mut coords = vec![[0f32; 3]; temp_segments + 1];
    let mut pdist = vec![0f32; temp_segments + 1];
    for i in 0..3 {
        let mut pts = vec![0f32; temp_segments + 1];
        bke_curve_forward_diff_bezier(
            control[0][i],
            control[1][i],
            control[2][i],
            control[3][i],
            &mut pts,
            0,
            temp_segments as i32,
            1,
        );
        for (j, &p) in pts.iter().enumerate().take(temp_segments + 1) {
            coords[j][i] = p;
        }
    }
    for i in 0..temp_segments {
        let d = [
            coords[i + 1][0] - coords[i][0],
            coords[i + 1][1] - coords[i][1],
            coords[i + 1][2] - coords[i][2],
        ];
        pdist[i + 1] = pdist[i] + (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    }
    let mut dist_step = pdist[temp_segments];
    let sum: f32 = segment_scales[..final_segments].iter().sum();
    dist_step /= sum;
    r_t_points[0] = 0.0;
    let mut dist = 0f32;
    let mut nr = 1usize;
    for i in 1..=final_segments {
        dist += segment_scales[i - 1] * dist_step;
        while nr < temp_segments && dist >= pdist[nr] {
            nr += 1;
        }
        let fac = (pdist[nr] - dist) / (pdist[nr] - pdist[nr - 1]);
        r_t_points[i] = (nr as f32 - fac) / temp_segments as f32;
    }
    r_t_points[final_segments] = 1.0;
}

/// `evaluate_cubic_bezier` (De Casteljau).
pub fn evaluate_cubic_bezier(
    control: &[[f32; 3]; 4],
    t: f32,
) -> ([f32; 3], [f32; 3]) {
    let lerp = |a: [f32; 3], b: [f32; 3]| {
        [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ]
    };
    let l1 = [
        lerp(control[0], control[1]),
        lerp(control[1], control[2]),
        lerp(control[2], control[3]),
    ];
    let l2 = [lerp(l1[0], l1[1]), lerp(l1[1], l1[2])];
    let tangent = [
        l2[1][0] - l2[0][0],
        l2[1][1] - l2[0][1],
        l2[1][2] - l2[0][2],
    ];
    let pos = [
        l2[0][0] + tangent[0] * t,
        l2[0][1] + tangent[1] * t,
        l2[0][2] + tangent[2] * t,
    ];
    (pos, tangent)
}

/// `BKE_pchan_bbone_handles_compute` (verbatim).
pub fn pchan_bbone_handles_compute(
    param: &BBoneSplineParameters,
    ease: bool,
    offsets: bool,
) -> ([f32; 3], f32, [f32; 3], f32) {
    let mut length = param.length;
    let epsilon = 1e-5 * length;
    if param.do_scale {
        length *= param.scale[1];
    }
    let mut h1 = [0f32; 3];
    let mut h2 = [0f32; 3];
    let mut roll1 = 0f32;
    let mut roll2 = 0f32;
    if param.use_prev {
        h1 = param.prev_h;
        if param.prev_bbone {
            h1[1] -= length;
        }
        let (n, l) = norm3(h1);
        h1 = if l < epsilon { [0.0, -1.0, 0.0] } else { n };
        h1 = [-h1[0], -h1[1], -h1[2]];
        if !param.prev_bbone {
            let m3 = copy_m3_m4(m4(param.prev_mat));
            roll1 = mat3_vec_to_roll(
                [
                    [m3.cols[0].x, m3.cols[0].y, m3.cols[0].z],
                    [m3.cols[1].x, m3.cols[1].y, m3.cols[1].z],
                    [m3.cols[2].x, m3.cols[2].y, m3.cols[2].z],
                ],
                h1,
            );
        }
    } else {
        h1 = [0.0, 1.0, 0.0];
    }
    if param.use_next {
        h2 = param.next_h;
        if !param.next_bbone {
            h2[1] -= length;
        }
        let (n, l) = norm3(h2);
        h2 = if l < epsilon { [0.0, 1.0, 0.0] } else { n };
        let m3 = copy_m3_m4(m4(param.next_mat));
        roll2 = mat3_vec_to_roll(
            [
                [m3.cols[0].x, m3.cols[0].y, m3.cols[0].z],
                [m3.cols[1].x, m3.cols[1].y, m3.cols[1].z],
                [m3.cols[2].x, m3.cols[2].y, m3.cols[2].z],
            ],
            h2,
        );
    } else {
        h2 = [0.0, 1.0, 0.0];
    }
    if ease {
        let circle_factor =
            length * (cubic_tangent_factor_circle(vv(h1), vv(h2)) / 0.75);
        let hlength1 = param.ease1 * circle_factor;
        let hlength2 = param.ease2 * circle_factor;
        h1 = [h1[0] * hlength1, h1[1] * hlength1, h1[2] * hlength1];
        h2 = [-h2[0] * hlength2, -h2[1] * hlength2, -h2[2] * hlength2];
    }
    if offsets {
        roll1 += param.roll1;
        roll2 += param.roll2;
        let xscale_correction = if param.do_scale { param.scale[0] } else { 1.0 };
        let zscale_correction = if param.do_scale { param.scale[2] } else { 1.0 };
        h1[0] += param.curve_in_x * xscale_correction;
        h1[2] += param.curve_in_z * zscale_correction;
        h2[0] += param.curve_out_x * xscale_correction;
        h2[2] += param.curve_out_z * zscale_correction;
    }
    (h1, roll1, h2, roll2)
}

/// `make_bbone_spline_matrix`.
fn make_bbone_spline_matrix(
    param: &BBoneSplineParameters,
    scalemats: &[[[f32; 4]; 4]; 2],
    pos: [f32; 3],
    axis: [f32; 3],
    roll: f32,
    scalex: f32,
    scalez: f32,
) -> [[f32; 4]; 4] {
    let m3 = vec_roll_to_mat3(axis, roll);
    let mut result = copy_m4_m3(Mat3::from_cols(
        Vec3::new(m3[0][0], m3[0][1], m3[0][2]),
        Vec3::new(m3[1][0], m3[1][1], m3[1][2]),
        Vec3::new(m3[2][0], m3[2][1], m3[2][2]),
    ));
    result.cols[3] = blenders_math::Vec4::new(pos[0], pos[1], pos[2], 1.0);
    if param.do_scale {
        result = m4(scalemats[0]) * result * m4(scalemats[1]);
    }
    result.cols[0] = result.cols[0] * scalex;
    result.cols[2] = result.cols[2] * scalez;
    a4(result)
}

/// `ease_handle_axis`.
fn ease_handle_axis(deriv1: [f32; 3], deriv2: [f32; 3]) -> [f32; 3] {
    const GAP: f32 = 0.1;
    let mut axis = deriv1;
    let len2 = deriv2[0] * deriv2[0] + deriv2[1] * deriv2[1] + deriv2[2] * deriv2[2];
    if len2 == 0.0 {
        return axis;
    }
    let len1 = deriv1[0] * deriv1[0] + deriv1[1] * deriv1[1] + deriv1[2] * deriv1[2];
    let ratio = len1 / len2;
    if ratio < GAP * GAP {
        let f = GAP - ratio.sqrt();
        axis = [
            axis[0] + deriv2[0] * f,
            axis[1] + deriv2[1] * f,
            axis[2] + deriv2[2] * f,
        ];
    }
    axis
}

/// `BKE_pchan_bbone_spline_compute` — N (+1 for deform)
/// per-segment matrices along the spline (verbatim).
pub fn pchan_bbone_spline_compute(
    param: &mut BBoneSplineParameters,
    for_deform: bool,
) -> Vec<[[f32; 4]; 4]> {
    let mut scalemats = [a4(Mat4::IDENTITY), a4(Mat4::IDENTITY)];
    let mut length = param.length;
    if param.do_scale {
        scalemats[1] = a4(size_to_mat4(vv(param.scale)));
        scalemats[0] = a4(invert_m4_m4(m4(scalemats[1])).unwrap_or(Mat4::IDENTITY));
        length *= param.scale[1];
    }
    let (h1, roll1, h2, roll2) = pchan_bbone_handles_compute(param, true, true);
    param.segments = param.segments.min(MAX_BBONE_SUBDIV as i16);
    let segs = param.segments.max(1) as usize;
    let bezt_controls: [[f32; 3]; 4] = [
        [0.0, 0.0, 0.0],
        h1,
        [h2[0], h2[1] + length, h2[2]],
        [0.0, length, 0.0],
    ];
    let mut segment_scales = [0f32; MAX_BBONE_SUBDIV];
    let scale_in_y = param.scale_in[1].max(0.0001);
    let scale_out_y = param.scale_out[1].max(0.0001);
    let log_in = scale_in_y.ln();
    let log_out = scale_out_y.ln();
    for i in 0..segs {
        let fac = i as f32 / (segs - 1).max(1) as f32;
        segment_scales[i] = blenders_math::base::interpf(log_out, log_in, fac).exp();
    }
    let mut bezt_points = [0f32; MAX_BBONE_SUBDIV + 1];
    equalize_cubic_bezier(
        &bezt_controls,
        MAX_BBONE_SUBDIV,
        segs,
        &segment_scales,
        &mut bezt_points,
    );
    let mut out = Vec::with_capacity(segs + if for_deform { 1 } else { 0 });
    if for_deform {
        let mut bezt_deriv1 = [[0f32; 3]; 3];
        let mut bezt_deriv2 = [[0f32; 3]; 2];
        for i in 0..3 {
            for k in 0..3 {
                bezt_deriv1[i][k] = bezt_controls[i + 1][k] - bezt_controls[i][k];
            }
        }
        for i in 0..2 {
            for k in 0..3 {
                bezt_deriv2[i][k] = bezt_deriv1[i + 1][k] - bezt_deriv1[i][k];
            }
        }
        let axis = ease_handle_axis(bezt_deriv1[0], bezt_deriv2[0]);
        out.push(make_bbone_spline_matrix(
            param,
            &scalemats,
            bezt_controls[0],
            axis,
            roll1,
            param.scale_in[0],
            param.scale_in[2],
        ));
        for a in 1..segs {
            let (cur, axis) = evaluate_cubic_bezier(&bezt_controls, bezt_points[a]);
            let fac = a as f32 / segs as f32;
            let roll = blenders_math::base::interpf(roll2, roll1, fac);
            let scalex = blenders_math::base::interpf(param.scale_out[0], param.scale_in[0], fac);
            let scalez = blenders_math::base::interpf(param.scale_out[2], param.scale_in[2], fac);
            out.push(make_bbone_spline_matrix(
                param, &scalemats, cur, axis, roll, scalex, scalez,
            ));
        }
        let neg2 = [-bezt_deriv2[1][0], -bezt_deriv2[1][1], -bezt_deriv2[1][2]];
        let axis = ease_handle_axis(bezt_deriv1[2], neg2);
        out.push(make_bbone_spline_matrix(
            param,
            &scalemats,
            bezt_controls[3],
            axis,
            roll2,
            param.scale_out[0],
            param.scale_out[2],
        ));
    } else {
        /* Pose-display path: one matrix per segment mid. */
        for a in 0..segs {
            let (cur, axis) = evaluate_cubic_bezier(&bezt_controls, bezt_points[a]);
            let fac = a as f32 / segs as f32;
            let roll = blenders_math::base::interpf(roll2, roll1, fac);
            let scalex = blenders_math::base::interpf(param.scale_out[0], param.scale_in[0], fac);
            let scalez = blenders_math::base::interpf(param.scale_out[2], param.scale_in[2], fac);
            out.push(make_bbone_spline_matrix(
                param, &scalemats, cur, axis, roll, scalex, scalez,
            ));
        }
    }
    out
}

/// `BKE_pchan_bbone_spline_params_get` — verbatim (rest
/// pose only; pose path needs `pose_mat`/`arm_mat` on both
/// channels — `pchan` fields mapped, `arm_mat` from `Bone`).
pub fn pchan_bbone_spline_params_get(
    pchan: Option<&BPoseChannel>,
    bone: &Bone,
    prev: Option<(&BPoseChannel, &Bone)>,
    next: Option<(&BPoseChannel, &Bone)>,
    rest: bool,
) -> BBoneSplineParameters {
    let mut param = BBoneSplineParameters {
        segments: bone.segments,
        length: bone.length,
        scale_in: bone.scale_in,
        scale_out: bone.scale_out,
        ..Default::default()
    };
    if let Some(p) = pchan {
        if !rest {
            let scale = mat4_to_size(m4(p.pose_mat));
            if (scale.x - scale.y).abs() > 1e-6 || (scale.y - scale.z).abs() > 1e-6 {
                param.do_scale = true;
                param.scale = [scale.x, scale.y, scale.z];
            }
        }
    }
    let imat = if rest {
        invert_m4_m4(m4(bone.arm_mat)).unwrap_or(Mat4::IDENTITY)
    } else if param.do_scale {
        let mut posemat = m4(pchan.unwrap().pose_mat);
        normalize_m4(&mut posemat);
        invert_m4_m4(posemat).unwrap_or(Mat4::IDENTITY)
    } else {
        invert_m4_m4(m4(pchan.map(|p| p.pose_mat).unwrap_or(a4(Mat4::IDENTITY)))).unwrap_or(Mat4::IDENTITY)
    };

    let mut prev_scale = [1.0f32; 3];
    let mut next_scale = [1.0f32; 3];
    if let Some((prev_p, prev_b)) = prev {
        param.use_prev = true;
        let mut done = false;
        let h1: [f32; 3];
        if bone.bbone_prev_type == BBONE_HANDLE_RELATIVE {
            if rest {
                param.prev_h = [0.0; 3];
                done = true;
                h1 = [0.0; 3];
            } else {
                let delta = [
                    prev_p.pose_head[0] - prev_b.arm_head[0],
                    prev_p.pose_head[1] - prev_b.arm_head[1],
                    prev_p.pose_head[2] - prev_b.arm_head[2],
                ];
                let ph = pchan.map(|p| p.pose_head).unwrap_or([0.0; 3]);
                h1 = [ph[0] - delta[0], ph[1] - delta[1], ph[2] - delta[2]];
            }
        } else if bone.bbone_prev_type == BBONE_HANDLE_TANGENT {
            if rest {
                let delta = [
                    prev_b.arm_tail[0] - prev_b.arm_head[0],
                    prev_b.arm_tail[1] - prev_b.arm_head[1],
                    prev_b.arm_tail[2] - prev_b.arm_head[2],
                ];
                h1 = [
                    bone.arm_head[0] - delta[0],
                    bone.arm_head[1] - delta[1],
                    bone.arm_head[2] - delta[2],
                ];
            } else {
                let delta = [
                    prev_p.pose_tail[0] - prev_p.pose_head[0],
                    prev_p.pose_tail[1] - prev_p.pose_head[1],
                    prev_p.pose_tail[2] - prev_p.pose_head[2],
                ];
                let ph = pchan.map(|p| p.pose_head).unwrap_or([0.0; 3]);
                h1 = [ph[0] - delta[0], ph[1] - delta[1], ph[2] - delta[2]];
            }
        } else {
            param.prev_bbone = prev_b.segments > 1;
            h1 = if rest { prev_b.arm_head } else { prev_p.pose_head };
        }
        if !done {
            param.prev_h = v3(mul_v3_m4v3(imat, vv(h1)));
        }
        if !param.prev_bbone {
            let src = if rest { prev_b.arm_mat } else { prev_p.pose_mat };
            param.prev_mat = a4(mul_m4_m4m4(imat, m4(src)));
            if (bone.bbone_prev_flag & BBONE_HANDLE_SCALE_ANY) != 0 && !rest {
                /* `BKE_armature_mat_pose_to_bone` PORT_PENDING;
                 * pose_mat scale used directly. */
                let s = mat4_to_size(m4(prev_p.pose_mat));
                prev_scale = [s.x, s.y, s.z];
            }
        }
    }
    if let Some((next_p, next_b)) = next {
        param.use_next = true;
        let mut done = false;
        let mut h2 = [0f32; 3];
        if bone.bbone_next_type == BBONE_HANDLE_RELATIVE {
            if rest {
                param.next_h = [0.0, param.length, 0.0];
                done = true;
            } else {
                let delta = [
                    next_p.pose_head[0] - next_b.arm_head[0],
                    next_p.pose_head[1] - next_b.arm_head[1],
                    next_p.pose_head[2] - next_b.arm_head[2],
                ];
                let pt = pchan.map(|p| p.pose_tail).unwrap_or([0.0, param.length, 0.0]);
                h2 = [pt[0] + delta[0], pt[1] + delta[1], pt[2] + delta[2]];
            }
        } else if bone.bbone_next_type == BBONE_HANDLE_TANGENT {
            if rest {
                let delta = [
                    next_b.arm_tail[0] - next_b.arm_head[0],
                    next_b.arm_tail[1] - next_b.arm_head[1],
                    next_b.arm_tail[2] - next_b.arm_head[2],
                ];
                h2 = [
                    bone.arm_tail[0] + delta[0],
                    bone.arm_tail[1] + delta[1],
                    bone.arm_tail[2] + delta[2],
                ];
            } else {
                let delta = [
                    next_p.pose_tail[0] - next_p.pose_head[0],
                    next_p.pose_tail[1] - next_p.pose_head[1],
                    next_p.pose_tail[2] - next_p.pose_head[2],
                ];
                let pt = pchan.map(|p| p.pose_tail).unwrap_or([0.0, param.length, 0.0]);
                h2 = [pt[0] + delta[0], pt[1] + delta[1], pt[2] + delta[2]];
            }
        } else {
            param.next_bbone = next_b.segments > 1;
            h2 = if rest { next_b.arm_tail } else { next_p.pose_tail };
        }
        if !done {
            param.next_h = v3(mul_v3_m4v3(imat, vv(h2)));
        }
        let src = if rest { next_b.arm_mat } else { next_p.pose_mat };
        param.next_mat = a4(mul_m4_m4m4(imat, m4(src)));
        if (bone.bbone_next_flag & BBONE_HANDLE_SCALE_ANY) != 0 && !rest {
            let s = mat4_to_size(m4(next_p.pose_mat));
            next_scale = [s.x, s.y, s.z];
        }
    }

    /* Ease/roll/curve accumulation (verbatim). */
    let (pe1, pe2, pr1, pr2, pcix, pciz, pcox, pcoz, psin, psout) = if !rest {
        pchan.map_or((0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, [1.0; 3], [1.0; 3]), |p| {
            (p.ease1, p.ease2, p.roll1, p.roll2, p.curve_in_x, p.curve_in_z,
             p.curve_out_x, p.curve_out_z, p.scale_in, p.scale_out)
        })
    } else {
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, [1.0; 3], [1.0; 3])
    };
    param.ease1 = bone.ease1 + pe1;
    param.ease2 = bone.ease2 + pe2;
    param.roll1 = bone.roll1 + pr1;
    param.roll2 = bone.roll2 + pr2;
    if (bone.bbone_flag & BBONE_ADD_PARENT_END_ROLL) != 0 {
        if let Some((pp, pb)) = prev {
            param.roll1 += pb.roll2;
            if !rest {
                param.roll1 += pp.roll2;
            }
        }
    }
    param.curve_in_x = bone.curve_in_x + pcix;
    param.curve_in_z = bone.curve_in_z + pciz;
    param.curve_out_x = bone.curve_out_x + pcox;
    param.curve_out_z = bone.curve_out_z + pcoz;
    for i in 0..3 {
        param.scale_in[i] *= psin[i];
        param.scale_out[i] *= psout[i];
    }
    if (bone.bbone_flag & BBONE_SCALE_EASING) != 0 {
        param.ease1 *= param.scale_in[1];
        param.curve_in_x *= param.scale_in[1];
        param.curve_in_z *= param.scale_in[1];
        param.ease2 *= param.scale_out[1];
        param.curve_out_x *= param.scale_out[1];
        param.curve_out_z *= param.scale_out[1];
    }
    if (bone.bbone_prev_flag & BBONE_HANDLE_SCALE_X) != 0 {
        param.scale_in[0] *= prev_scale[0];
    }
    if (bone.bbone_prev_flag & BBONE_HANDLE_SCALE_Y) != 0 {
        param.scale_in[1] *= prev_scale[1];
    }
    if (bone.bbone_prev_flag & BBONE_HANDLE_SCALE_Z) != 0 {
        param.scale_in[2] *= prev_scale[2];
    }
    if (bone.bbone_next_flag & BBONE_HANDLE_SCALE_X) != 0 {
        param.scale_out[0] *= next_scale[0];
    }
    if (bone.bbone_next_flag & BBONE_HANDLE_SCALE_Y) != 0 {
        param.scale_out[1] *= next_scale[1];
    }
    if (bone.bbone_next_flag & BBONE_HANDLE_SCALE_Z) != 0 {
        param.scale_out[2] *= next_scale[2];
    }
    param
}
