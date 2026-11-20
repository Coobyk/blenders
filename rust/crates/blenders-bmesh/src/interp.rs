//! Port of `bmesh_interp.cc` — loop/vert attribute interpolation.
//!
//! Most of this file drives `CustomData` layer interpolation, which is
//! PORT_PENDING. Ported here: the pure-geometry helpers (`quad_co`,
//! `mdisp_axis_from_quad`, `bm_loop_flip_equotion`, `bm_loop_flip_disp`)
//! used by multires displacement reprojection. Layer management functions
//! (`BM_data_layer_*`) become no-ops/stubs pending CustomData.

use crate::{BMesh, BM_NIL};
use blenders_math::geom;
use blenders_math::vector::{Vec2, Vec3};

/// `quad_co` — project quad + point into 2D along `n`, check containment,
/// resolve UV within the (possibly non-planar) quad.
pub fn quad_co(
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
    v4: Vec3,
    p: Vec3,
    n: Vec3,
) -> Option<Vec2> {
    let n2 = geom::normal_quad(v1, v2, v3, v4).0;
    if n.dot(n2) < -f32::EPSILON {
        return None;
    }
    /* rotate into plane */
    let mut projverts = [v1, v2, v3, v4, p];
    crate::polygon::poly_rotate_plane(n, &mut projverts);
    /* subtract origin (projected p) */
    let origin = Vec2::ZERO;
    let mut pv2 = [Vec2::ZERO; 4];
    for i in 0..4 {
        pv2[i] = Vec2::new(
            projverts[i].x - projverts[4].x,
            projverts[i].y - projverts[4].y,
        );
    }
    if !geom::isect_point_quad(origin, pv2[0], pv2[1], pv2[2], pv2[3]) == 1 {
        return None;
    }
    Some(geom::resolve_quad_uv_v2(
        origin, pv2[0], pv2[3], pv2[2], pv2[1],
    ))
}

/// `mdisp_axis_from_quad`.
pub fn mdisp_axis_from_quad(v1: Vec3, v2: Vec3, _v3: Vec3, v4: Vec3) -> (Vec3, Vec3) {
    let mut x = v4 - v1;
    let mut y = v2 - v1;
    x.normalize();
    y.normalize();
    (x, y)
}

/// `bm_loop_flip_equotion` — 2x2 system determinant for a coordinate pair.
fn bm_loop_flip_equotion(
    mat: &mut [[f32; 2]; 2],
    b: &mut [f32; 2],
    target_axis_x: Vec3,
    target_axis_y: Vec3,
    coord: Vec3,
    i: usize,
    j: usize,
) -> f32 {
    let get = |v: Vec3, k: usize| [v.x, v.y, v.z][k];
    mat[0][0] = get(target_axis_x, i);
    mat[0][1] = get(target_axis_y, i);
    mat[1][0] = get(target_axis_x, j);
    mat[1][1] = get(target_axis_y, j);
    b[0] = get(coord, i);
    b[1] = get(coord, j);
    mat[0][0] * mat[1][1] - mat[0][1] * mat[1][0]
}

/// `bm_loop_flip_disp` — reproject a (u, v, w) displacement from source
/// grid axes onto target grid axes.
pub fn bm_loop_flip_disp(
    source_axis_x: Vec3,
    source_axis_y: Vec3,
    target_axis_x: Vec3,
    target_axis_y: Vec3,
    disp: &mut Vec3,
) {
    let vx = source_axis_x * disp.x;
    let vy = source_axis_y * disp.y;
    let mut coord = vx + vy;
    /* project onto the target plane */
    let n = target_axis_x.cross(target_axis_y);
    let vec = blenders_math::vector::project_v3(coord, n);
    coord = coord - vec;

    let mut mat = [[0.0f32; 2]; 2];
    let mut b = [0.0f32; 2];
    let mut d = bm_loop_flip_equotion(&mut mat, &mut b, target_axis_x, target_axis_y, coord, 0, 1);
    if d.abs() < 1e-4 {
        d = bm_loop_flip_equotion(&mut mat, &mut b, target_axis_x, target_axis_y, coord, 0, 2);
        if d.abs() < 1e-4 {
            d = bm_loop_flip_equotion(&mut mat, &mut b, target_axis_x, target_axis_y, coord, 1, 2);
        }
    }
    if d.abs() < 1e-12 {
        disp.x = 0.0;
        disp.y = 0.0;
        return;
    }
    disp.x = (b[0] * mat[1][1] - mat[0][1] * b[1]) / d;
    disp.y = (mat[0][0] * b[1] - b[0] * mat[1][0]) / d;
}

/* ---- PORT_PENDING (CustomData-dependent): ----
 * `BM_data_interp_from_verts`, `BM_data_interp_from_edges`,
 * `BM_data_interp_face_vert_edge`, `BM_face_interp_from_face(_ex)`,
 * `BM_loop_interp_multires(_ex)`, `BM_face_interp_multires(_ex)`,
 * `BM_face_multires_bounds_smooth`, `BM_loop_interp_from_face`,
 * `BM_vert_interp_from_face`, `BM_data_layer_*`, `BM_elem_float_data_*`,
 * `BM_data_layer_lookup`, `BM_vert_loop_groups_data_layer_*`,
 * `BM_uv_map_attr_pin_*`.
 * These all read/write `CustomData` block contents and are re-ported once
 * the CustomData system exists in `blenders-bli`/`blenders-bke`.
 */

/// `BM_data_interp_from_verts` — PORT_PENDING stub.
pub fn bm_data_interp_from_verts(
    _bm: &mut BMesh,
    _v1: usize,
    _v2: usize,
    _v: usize,
    _fac: f32,
) {
    /* PORT_PENDING: CustomData */
}

/// `BM_face_interp_from_face` — PORT_PENDING stub.
pub fn bm_face_interp_from_face(
    _bm: &mut BMesh,
    _f_dst: usize,
    _f_src: usize,
    _do_vertex: bool,
) {
    /* PORT_PENDING: CustomData */
}

/// `BM_loop_interp_from_face` — PORT_PENDING stub.
pub fn bm_loop_interp_from_face(
    _bm: &mut BMesh,
    _l_dst: usize,
    _f_src: usize,
    _do_vertex: bool,
    _do_weights: bool,
) {
    /* PORT_PENDING: CustomData */
}

const _: usize = BM_NIL;
