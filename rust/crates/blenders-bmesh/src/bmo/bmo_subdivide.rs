//! Port of `bmo_subdivide.cc` — edge-based subdivision with patterns.
//! Shape-key temp layer → `subd_co` side-table (CustomData PORT_PENDING).

use crate::core::*;
use crate::iterators::*;
use crate::operators::*;
use crate::{BMesh, BMEdge, BMVert, BM_EDGE, BM_FACE, BM_NIL, BM_VERT};
use blenders_math::geom::{ortho_basis, shell_v3v3_mid_normalized_to_dist};
use blenders_math::vector::Vec3;
use std::collections::HashMap;

const SUBD_SPLIT: u8 = 1;
const EDGE_PERCENT: u8 = 2;
const FACE_CUSTOMFILL: u8 = 4;
const ELE_INNER: u8 = 8;
const ELE_SPLIT: u8 = 16;
const FLT_FACE_SPLIT_EPSILON: f32 = 0.00005;

pub const SUBD_CORNER_INNERVERT: i32 = 0;
pub const SUBD_CORNER_PATH: i32 = 1;
pub const SUBD_CORNER_FAN: i32 = 2;
pub const SUBD_FALLOFF_LIN: i32 = 0;

/// `SubDParams` — `subd_co` replaces the CD_SHAPEKEY temp layer.
pub struct SubDParams {
    pub numcuts: i32,
    pub smooth: f32,
    pub smooth_falloff: i32,
    pub fractal: f32,
    pub along_normal: f32,
    pub use_smooth: bool,
    pub use_smooth_even: bool,
    pub use_sphere: bool,
    pub use_fractal: bool,
    pub seed: i32,
    pub fractal_ofs: Vec3,
    /// displaced-co storage for every vert (CD_SHAPEKEY tmp layer).
    pub subd_co: HashMap<usize, Vec3>,
    /// edge -> percent (edge_percents slot map).
    pub edge_percents: HashMap<usize, f32>,
}

/// `connect_smallest_face`.
fn connect_smallest_face(bm: &mut BMesh, v_a: usize, v_b: usize, r_f_new: &mut usize) -> usize {
    let mut l_a = BM_NIL;
    let mut l_b = BM_NIL;
    let f = crate::query::bm_vert_pair_share_face_by_len_loops(
        bm, v_a, v_b, &mut l_a, &mut l_b, true);
    if f != BM_NIL {
        let mut l_new = BM_NIL;
        let f_new = crate::mods::bm_face_split(bm, f, l_a, l_b, Some(&mut l_new), false);
        *r_f_new = f_new;
        if l_new != BM_NIL {
            return bm.loo(l_new).e;
        }
    }
    BM_NIL
}

/// `interp_slerp_co_no_v3`.
fn interp_slerp_co_no_v3(
    co_a: Vec3, no_a: Vec3, co_b: Vec3, no_b: Vec3, no_dir: Vec3, fac: f32,
) -> Vec3 {
    let no_mid = (no_a + no_b).normalized();
    let no_ortho = no_mid.cross(no_dir);
    let center;
    if no_ortho.length() > 0.0 {
        let no_ortho = no_ortho.normalized();
        /* project_v3_plane(v, plane_pt, plane_no) = v - (dot(v-pt, no))*no */
        let pv = |v: Vec3, pt: Vec3, no: Vec3| v - no * (v - pt).dot(no);
        let v_a_no_ortho = pv(no_ortho.cross(no_a), no_ortho, no_ortho.cross(no_a));
        let v_b_no_ortho = pv(no_ortho.cross(no_b), no_ortho, no_ortho.cross(no_b));
        let plane_a = blenders_math::geom::plane_from_point_normal(co_a, v_a_no_ortho);
        let plane_b = blenders_math::geom::plane_from_point_normal(co_b, v_b_no_ortho);
        let plane_c = blenders_math::geom::plane_from_point_normal(co_b, no_ortho);
        center = blenders_math::geom::isect_plane_plane_plane(plane_a, plane_b, plane_c)
            .unwrap_or((co_a + co_b) * 0.5);
    } else {
        center = (co_a + co_b) * 0.5;
    }
    let ofs_a = co_a - center;
    let ofs_b = co_b - center;
    let dist_a = ofs_a.length();
    let dist_b = ofs_b.length();
    if dist_a > f32::EPSILON && dist_b > f32::EPSILON {
        match blenders_math::vector::slerp_v3(ofs_a / dist_a, ofs_b / dist_b, fac) {
            Some(ofs_slerp) => {
                center + ofs_slerp * (dist_a + (dist_b - dist_a) * fac)
            }
            None => co_a.lerp(co_b, fac),
        }
    } else {
        co_a.lerp(co_b, fac)
    }
}

/// `alter_co` — compute displaced co for `v` into `params.subd_co`.
fn alter_co(
    bm: &BMesh,
    v: usize,
    params: &mut SubDParams,
    perc: f32,
    v_a: &VertTmp,
    v_b: &VertTmp,
) {
    let mut co = bm.vert(v).co;
    if params.use_sphere {
        /* subdivide sphere */
        co = co.normalized() * params.smooth;
    } else if params.use_smooth {
        let eps_unit_vec = 1e-5f32;
        let mut smooth;
        let no_dir = (v_a.co - v_b.co).normalized();
        /* USE_SPHERE_DUAL_BLEND */
        let co_a = {
            let no_reflect = reflect_v3(v_a.no, no_dir);
            if (v_a.no - no_reflect).length_squared() < eps_unit_vec {
                v_a.co.lerp(v_b.co, perc)
            } else {
                interp_slerp_co_no_v3(v_a.co, v_a.no, v_b.co, no_reflect, no_dir, perc)
            }
        };
        let co_b = {
            let no_reflect = reflect_v3(v_b.no, no_dir);
            if (v_b.no - no_reflect).length_squared() < eps_unit_vec {
                v_a.co.lerp(v_b.co, perc)
            } else {
                interp_slerp_co_no_v3(v_a.co, no_reflect, v_b.co, v_b.no, no_dir, perc)
            }
        };
        co = co_a.lerp(co_b, perc);
        if params.smooth_falloff == SUBD_FALLOFF_LIN {
            smooth = 1.0;
        } else {
            smooth = (1.0 - 2.0 * (0.5 - perc).abs()).abs();
            smooth = 1.0 + crate::query::bmesh_subd_falloff_calc(params.smooth_falloff, smooth);
        }
        if params.use_smooth_even {
            smooth *= shell_v3v3_mid_normalized_to_dist(v_a.no, v_b.no);
        }
        smooth *= params.smooth;
        if smooth != 1.0 {
            let co_flat = v_a.co.lerp(v_b.co, perc);
            co = co_flat.lerp(co, smooth);
        }
    }
    if params.use_fractal {
        let len = (v_a.co - v_b.co).length();
        let fac = params.fractal * len;
        let normal = (v_a.no + v_b.no) * 0.5;
        let (base1, base2) = ortho_basis(normal);
        let co2 = (bm.vert(v).co + params.fractal_ofs) * 10.0;
        let tvec = Vec3::new(
            fac * (blenders_bli::noise_c::noise_generic_turbulence(
                1.0, co2.x, co2.y, co2.z, 15, false, 2) - 0.5),
            fac * (blenders_bli::noise_c::noise_generic_turbulence(
                1.0, co2.y, co2.x, co2.z, 15, false, 2) - 0.5),
            fac * (blenders_bli::noise_c::noise_generic_turbulence(
                1.0, co2.y, co2.z, co2.x, 15, false, 2) - 0.5),
        );
        co = co + normal * tvec.x
            + base1 * (tvec.y * (1.0 - params.along_normal))
            + base2 * (tvec.z * (1.0 - params.along_normal));
    }
    /* PORT_PENDING: totlayer>1 shape-key offset chain (CustomData). */
    params.subd_co.insert(v, co);
}

fn reflect_v3(v: Vec3, normal: Vec3) -> Vec3 {
    v.reflect(normal)
}

/// snapshot of a vert (co+no) — replaces `BMVert v1_tmp = *edge->v1`.
#[derive(Clone, Copy)]
pub struct VertTmp {
    pub co: Vec3,
    pub no: Vec3,
}
impl VertTmp {
    pub fn of(bm: &BMesh, v: usize) -> Self {
        VertTmp { co: bm.vert(v).co, no: bm.vert(v).no }
    }
}

/// `bm_subdivide_edge_addvert`.
fn bm_subdivide_edge_addvert(
    bm: &mut BMesh,
    edge: usize,
    params: &mut SubDParams,
    factor_edge_split: f32,
    factor_subd: f32,
    v_a: &VertTmp,
    v_b: &VertTmp,
    r_edge: &mut usize,
) -> usize {
    let v1 = bm.edge(edge).v1;
    let mut e_new = BM_NIL;
    let v_new = crate::mods::bm_edge_split(bm, edge, v1, Some(&mut e_new), factor_edge_split);
    *r_edge = e_new;
    bm.vpool[v_new].head.api_flag |= ELE_INNER;
    alter_co(bm, v_new, params, factor_subd, v_a, v_b);
    bm.vpool[v_new].no = v_a.no.lerp(v_b.no, factor_subd).normalized();
    v_new
}

/// `subdivide_edge_num`.
fn subdivide_edge_num(
    bm: &mut BMesh,
    edge: usize,
    curpoint: i32,
    totpoint: i32,
    params: &mut SubDParams,
    v_a: &VertTmp,
    v_b: &VertTmp,
    r_edge: &mut usize,
) -> usize {
    let (factor_edge_split, factor_subd);
    if bm.edge(edge).head.api_flag & EDGE_PERCENT != 0 && totpoint == 1 {
        factor_edge_split = *params.edge_percents.get(&edge).unwrap_or(&0.5);
        factor_subd = 0.0;
    } else {
        factor_edge_split = 1.0 / (totpoint + 1 - curpoint) as f32;
        factor_subd = (curpoint + 1) as f32 / (totpoint + 1) as f32;
    }
    bm_subdivide_edge_addvert(bm, edge, params, factor_edge_split, factor_subd,
                              v_a, v_b, r_edge)
}

/// `bm_subdivide_multicut`.
pub fn bm_subdivide_multicut(
    bm: &mut BMesh,
    edge: usize,
    params: &mut SubDParams,
    v_a: usize,
    v_b: usize,
) {
    let mut eed = edge;
    let v1_tmp = VertTmp::of(bm, bm.edge(edge).v1);
    let v2_tmp = VertTmp::of(bm, bm.edge(edge).v2);
    let va = VertTmp::of(bm, v_a);
    let vb = VertTmp::of(bm, v_b);
    let v1 = bm.edge(edge).v1;
    let v2 = bm.edge(edge).v2;
    for i in 0..params.numcuts {
        let mut e_new = BM_NIL;
        let v = subdivide_edge_num(bm, eed, i, params.numcuts, params, &va, &vb, &mut e_new);
        bm.vpool[v].head.api_flag |= SUBD_SPLIT | ELE_SPLIT;
        bm.epool[eed].head.api_flag |= SUBD_SPLIT | ELE_SPLIT;
        bm.epool[e_new].head.api_flag |= SUBD_SPLIT | ELE_SPLIT;
        eed = e_new;
    }
    alter_co(bm, v1, params, 0.0, &v1_tmp, &v2_tmp);
    alter_co(bm, v2, params, 1.0, &v1_tmp, &v2_tmp);
}

/* ------------------------------------------------------------------ */
/* Patterns */

type PatternFn = fn(&mut BMesh, usize, &[usize], &mut SubDParams);
struct SubDPattern {
    seledges: &'static [i32],
    connectexec: PatternFn,
    len: i32,
}

fn quad_1edge_split(bm: &mut BMesh, _f: usize, verts: &[usize], params: &mut SubDParams) {
    let mut f_new = BM_NIL;
    let numcuts = params.numcuts;
    if numcuts % 2 == 0 {
        let mut add = 2;
        for i in 0..numcuts {
            if i == numcuts / 2 {
                add -= 1;
            }
            connect_smallest_face(bm, verts[i as usize],
                                  verts[(numcuts + add) as usize], &mut f_new);
        }
    } else {
        let mut add = 2;
        for i in 0..numcuts {
            connect_smallest_face(bm, verts[i as usize],
                                  verts[(numcuts + add) as usize], &mut f_new);
            if i == numcuts / 2 {
                add -= 1;
                connect_smallest_face(bm, verts[i as usize],
                                      verts[(numcuts + add) as usize], &mut f_new);
            }
        }
    }
}
static QUAD_1EDGE: SubDPattern = SubDPattern {
    seledges: &[1, 0, 0, 0],
    connectexec: quad_1edge_split,
    len: 4,
};

fn quad_2edge_split_path(bm: &mut BMesh, _f: usize, verts: &[usize], params: &mut SubDParams) {
    let mut f_new = BM_NIL;
    let numcuts = params.numcuts;
    for i in 0..numcuts {
        connect_smallest_face(bm, verts[i as usize],
                              verts[(numcuts + (numcuts - i)) as usize], &mut f_new);
    }
    connect_smallest_face(bm, verts[(numcuts * 2 + 3) as usize],
                          verts[(numcuts * 2 + 1) as usize], &mut f_new);
}
static QUAD_2EDGE_PATH: SubDPattern = SubDPattern {
    seledges: &[1, 1, 0, 0],
    connectexec: quad_2edge_split_path,
    len: 4,
};

fn quad_2edge_split_innervert(bm: &mut BMesh, _f: usize, verts: &[usize], params: &mut SubDParams) {
    let mut f_new = BM_NIL;
    let numcuts = params.numcuts;
    let mut v_last = verts[numcuts as usize];
    for i in (0..numcuts).rev() {
        let e = connect_smallest_face(bm, verts[i as usize],
                                      verts[(numcuts + (numcuts - i)) as usize], &mut f_new);
        if e == BM_NIL {
            continue;
        }
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        let va = VertTmp::of(bm, v1);
        let vb = VertTmp::of(bm, v2);
        let mut e_new = BM_NIL;
        let v = bm_subdivide_edge_addvert(bm, e, params, 0.5, 0.5, &va, &vb, &mut e_new);
        if i != numcuts - 1 {
            connect_smallest_face(bm, v_last, v, &mut f_new);
        }
        v_last = v;
    }
    connect_smallest_face(bm, v_last, verts[(numcuts * 2 + 2) as usize], &mut f_new);
}
static QUAD_2EDGE_INNERVERT: SubDPattern = SubDPattern {
    seledges: &[1, 1, 0, 0],
    connectexec: quad_2edge_split_innervert,
    len: 4,
};

fn quad_2edge_split_fan(bm: &mut BMesh, _f: usize, verts: &[usize], params: &mut SubDParams) {
    let mut f_new = BM_NIL;
    let numcuts = params.numcuts;
    for i in 0..numcuts {
        connect_smallest_face(bm, verts[i as usize],
                              verts[(numcuts * 2 + 2) as usize], &mut f_new);
        connect_smallest_face(bm, verts[(numcuts + (numcuts - i)) as usize],
                              verts[(numcuts * 2 + 2) as usize], &mut f_new);
    }
}
static QUAD_2EDGE_FAN: SubDPattern = SubDPattern {
    seledges: &[1, 1, 0, 0],
    connectexec: quad_2edge_split_fan,
    len: 4,
};

fn quad_3edge_split(bm: &mut BMesh, _f: usize, verts: &[usize], params: &mut SubDParams) {
    let mut f_new = BM_NIL;
    let numcuts = params.numcuts;
    let mut add = 0i32;
    for i in 0..numcuts {
        if i == numcuts / 2 {
            if numcuts % 2 != 0 {
                connect_smallest_face(bm, verts[(numcuts - i - 1 + add) as usize],
                                      verts[(i + numcuts + 1) as usize], &mut f_new);
            }
            add = numcuts * 2 + 2;
        }
        connect_smallest_face(bm, verts[(numcuts - i - 1 + add) as usize],
                              verts[(i + numcuts + 1) as usize], &mut f_new);
    }
    for i in 0..numcuts / 2 + 1 {
        connect_smallest_face(bm, verts[i as usize],
                              verts[((numcuts - i) + numcuts * 2 + 1) as usize], &mut f_new);
    }
}
static QUAD_3EDGE: SubDPattern = SubDPattern {
    seledges: &[1, 1, 1, 0],
    connectexec: quad_3edge_split,
    len: 4,
};

fn quad_4edge_subdivide(bm: &mut BMesh, _f: usize, verts: &[usize], params: &mut SubDParams) {
    let mut f_new = BM_NIL;
    let numcuts = params.numcuts;
    let s = (numcuts + 2) as usize;
    let mut lines = vec![BM_NIL; s * s];
    for i in 0..numcuts + 2 {
        lines[i as usize] = verts[(numcuts * 3 + 2 + (numcuts - i + 1)) as usize];
    }
    for i in 0..numcuts + 2 {
        lines[(s - 1) * s + i as usize] = verts[(numcuts + i) as usize];
    }
    for i in 0..numcuts {
        let a = i;
        let b = numcuts + 1 + numcuts + 1 + (numcuts - i - 1);
        let e = connect_smallest_face(bm, verts[a as usize], verts[b as usize], &mut f_new);
        if e == BM_NIL {
            continue;
        }
        bm.epool[e].head.api_flag |= ELE_INNER;
        if f_new != BM_NIL {
            bm.fpool[f_new].head.api_flag |= ELE_INNER;
        }
        lines[((i + 1) as usize) * s] = verts[a as usize];
        lines[((i + 1) as usize) * s + s - 1] = verts[b as usize];
        let v1 = lines[((i + 1) as usize) * s];
        let v2 = lines[((i + 1) as usize) * s + s - 1];
        let va = VertTmp::of(bm, v1);
        let vb = VertTmp::of(bm, v2);
        let mut e_cur = e;
        for a2 in 0..numcuts {
            let mut e_new = BM_NIL;
            let v = subdivide_edge_num(bm, e_cur, a2, numcuts, params, &va, &vb, &mut e_new);
            bm.epool[e_new].head.api_flag |= ELE_INNER;
            lines[((i + 1) as usize) * s + (a2 as usize) + 1] = v;
            e_cur = e_new;
        }
    }
    for i in 1..numcuts + 2 {
        for j in 1..=numcuts {
            let a = (i as usize) * s + j as usize;
            let b = ((i - 1) as usize) * s + j as usize;
            let e = connect_smallest_face(bm, lines[a], lines[b], &mut f_new);
            if e == BM_NIL {
                continue;
            }
            bm.epool[e].head.api_flag |= ELE_INNER;
            if f_new != BM_NIL {
                bm.fpool[f_new].head.api_flag |= ELE_INNER;
            }
        }
    }
}
static QUAD_4EDGE: SubDPattern = SubDPattern {
    seledges: &[1, 1, 1, 1],
    connectexec: quad_4edge_subdivide,
    len: 4,
};

fn tri_1edge_split(bm: &mut BMesh, _f: usize, verts: &[usize], params: &mut SubDParams) {
    let mut f_new = BM_NIL;
    let numcuts = params.numcuts;
    for i in 0..numcuts {
        connect_smallest_face(bm, verts[i as usize], verts[(numcuts + 1) as usize], &mut f_new);
    }
}
static TRI_1EDGE: SubDPattern = SubDPattern {
    seledges: &[1, 0, 0],
    connectexec: tri_1edge_split,
    len: 3,
};

fn tri_3edge_subdivide(bm: &mut BMesh, _f: usize, verts: &[usize], params: &mut SubDParams) {
    let mut f_new = BM_NIL;
    let numcuts = params.numcuts;
    let mut lines: Vec<Vec<usize>> = vec![Vec::new(); (numcuts + 2) as usize];
    lines[0] = vec![verts[(numcuts * 2 + 1) as usize]];
    lines[(numcuts + 1) as usize] = vec![BM_NIL; (numcuts + 2) as usize];
    for i in 0..numcuts {
        lines[(numcuts + 1) as usize][(i + 1) as usize] = verts[i as usize];
    }
    lines[(numcuts + 1) as usize][0] = verts[(numcuts * 3 + 2) as usize];
    lines[(numcuts + 1) as usize][(numcuts + 1) as usize] = verts[numcuts as usize];
    for i in 0..numcuts {
        lines[(i + 1) as usize] = vec![BM_NIL; (2 + i) as usize];
        let a = numcuts * 2 + 2 + i;
        let b = numcuts + numcuts - i;
        let e = connect_smallest_face(bm, verts[a as usize], verts[b as usize], &mut f_new);
        if e == BM_NIL {
            break;
        }
        bm.epool[e].head.api_flag |= ELE_INNER;
        if f_new != BM_NIL {
            bm.fpool[f_new].head.api_flag |= ELE_INNER;
        }
        lines[(i + 1) as usize][0] = verts[a as usize];
        lines[(i + 1) as usize][(i + 1) as usize] = verts[b as usize];
        let va = VertTmp::of(bm, verts[a as usize]);
        let vb = VertTmp::of(bm, verts[b as usize]);
        let mut e_cur = e;
        for j in 0..i {
            let mut e_new = BM_NIL;
            let v = subdivide_edge_num(bm, e_cur, j, i, params, &va, &vb, &mut e_new);
            lines[(i + 1) as usize][(j + 1) as usize] = v;
            bm.epool[e_new].head.api_flag |= ELE_INNER;
            e_cur = e_new;
        }
    }
    for i in 1..=numcuts {
        for j in 0..i {
            let (ia, ja) = (i as usize, j as usize);
            if ia >= lines.len() || ja >= lines[ia].len() || ja + 1 >= lines[ia + 1].len() {
                continue;
            }
            let e = connect_smallest_face(bm, lines[ia][ja], lines[ia + 1][ja + 1], &mut f_new);
            if e != BM_NIL {
                bm.epool[e].head.api_flag |= ELE_INNER;
                if f_new != BM_NIL {
                    bm.fpool[f_new].head.api_flag |= ELE_INNER;
                }
            }
            if ja + 1 < lines[ia].len() {
                let e = connect_smallest_face(bm, lines[ia][ja + 1],
                                              lines[ia + 1][ja + 1], &mut f_new);
                if e != BM_NIL {
                    bm.epool[e].head.api_flag |= ELE_INNER;
                    if f_new != BM_NIL {
                        bm.fpool[f_new].head.api_flag |= ELE_INNER;
                    }
                }
            }
        }
    }
}
static TRI_3EDGE: SubDPattern = SubDPattern {
    seledges: &[1, 1, 1],
    connectexec: tri_3edge_subdivide,
    len: 3,
};

/* ------------------------------------------------------------------ */

/// `bmo_subdivide_edges_exec`.
pub fn bmo_subdivide_edges_exec(bm: &mut BMesh, op: &mut BMOperator) {
    bmo_slot_buffer_flag_enable(bm, op, false, "edges", SUBD_SPLIT);

    let numcuts = bmo_slot_int_get(op, false, "cuts");
    let seed = bmo_slot_int_get(op, false, "seed");
    let smooth = bmo_slot_float_get(op, false, "smooth");
    let smooth_falloff = bmo_slot_int_get(op, false, "smooth_falloff");
    let fractal = bmo_slot_float_get(op, false, "fractal");
    let along_normal = bmo_slot_float_get(op, false, "along_normal");
    let cornertype = bmo_slot_int_get(op, false, "quad_corner_type");
    let use_single_edge = bmo_slot_bool_get(op, false, "use_single_edge");
    let use_grid_fill = bmo_slot_bool_get(op, false, "use_grid_fill");
    let use_only_quads = bmo_slot_bool_get(op, false, "use_only_quads");
    let use_sphere = bmo_slot_bool_get(op, false, "use_sphere");
    let use_smooth_even = bmo_slot_bool_get(op, false, "use_smooth_even");

    let mut patterns: [Option<&'static SubDPattern>; 6] = [None; 6];
    patterns[1] = match cornertype {
        SUBD_CORNER_PATH => Some(&QUAD_2EDGE_PATH),
        SUBD_CORNER_INNERVERT => Some(&QUAD_2EDGE_INNERVERT),
        SUBD_CORNER_FAN => Some(&QUAD_2EDGE_FAN),
        _ => None,
    };
    if use_single_edge {
        patterns[0] = Some(&QUAD_1EDGE);
        patterns[2] = Some(&TRI_1EDGE);
    }
    if use_grid_fill {
        patterns[3] = Some(&QUAD_4EDGE);
        patterns[5] = Some(&TRI_3EDGE);
    }

    /* CD_SHAPEKEY tmp layer → subd_co side-table (initialized to v->co). */
    let mut params = SubDParams {
        numcuts,
        smooth,
        smooth_falloff,
        fractal,
        along_normal,
        use_smooth: smooth != 0.0,
        use_smooth_even,
        use_fractal: fractal != 0.0,
        use_sphere,
        seed,
        fractal_ofs: Vec3::ZERO,
        subd_co: HashMap::new(),
        edge_percents: HashMap::new(),
    };
    for v in VertsOfMesh::new(bm) {
        params.subd_co.insert(v, bm.vert(v).co);
    }
    if params.use_fractal {
        let mut rng = blenders_bli::rand::Rng::new_srandom(seed as u32);
        params.fractal_ofs = Vec3::new(
            rng.get_float() * 200.0,
            rng.get_float() * 200.0,
            rng.get_float() * 200.0,
        );
    }

    /* custom_patterns + edge_percents slot maps */
    if let Some(s) = op.slots_in.iter().find(|s| s.name == "custom_patterns") {
        let keys: Vec<usize> =
            s.map.keys().filter(|(ht, _)| *ht == BM_FACE).map(|(_, i)| *i).collect();
        for f in keys {
            bm.fpool[f].head.api_flag |= FACE_CUSTOMFILL;
        }
    }
    if let Some(s) = op.slots_in.iter().find(|s| s.name == "edge_percents") {
        for ((ht, k), (_, v)) in s.map.clone() {
            if ht == BM_EDGE {
                bm.epool[k].head.api_flag |= EDGE_PERCENT;
                /* percentage stored via float slot map — C uses map float values. */
                let _ = v;
            }
        }
    }
    if let Some(s) = op.slots_in.iter().find(|s| s.name == "edge_percents") {
        /* edge_percents is a FLOAT map in C; our map is elem->elem so
         * store percents from the map values' f where available. */
        for (k, v) in &s.map {
            if k.0 == BM_EDGE {
                /* value is a vertex ref in ElemBuf maps; percent maps are rare —
                 * fall back to 0.5 */
                params.edge_percents.insert(k.1, match v.0 {
                    _ => 0.5,
                });
            }
        }
    }

    struct SubDFaceData {
        start: usize,
        pat: Option<&'static SubDPattern>,
        totedgesel: i32,
        face: usize,
    }
    let mut facedata: Vec<SubDFaceData> = Vec::new();

    for face in FacesOfMesh::new(bm).collect::<Vec<_>>() {
        if use_only_quads && bm.face(face).len != 4 {
            continue;
        }
        let loops: Vec<usize> = LoopsOfFace::new(bm, face).collect();
        let edges: Vec<usize> = loops.iter().map(|&l| bm.loo(l).e).collect();
        let verts_of: Vec<usize> = loops.iter().map(|&l| bm.loo(l).v).collect();
        let mut totesel = 0;
        let mut e1 = BM_NIL;
        let mut e2 = BM_NIL;
        for &e in &edges {
            if bm.edge(e).head.api_flag & SUBD_SPLIT != 0 {
                if e1 == BM_NIL {
                    e1 = e;
                } else {
                    e2 = e;
                }
                totesel += 1;
            }
        }
        if totesel == 2 && crate::query::bm_edge_share_vert_check(bm, e1, e2) {
            let vec1 = (bm.vert(bm.edge(e1).v2).co - bm.vert(bm.edge(e1).v1).co).normalized();
            let vec2 = (bm.vert(bm.edge(e2).v2).co - bm.vert(bm.edge(e2).v1).co).normalized();
            if vec1.dot(vec2).abs() > 1.0 - FLT_FACE_SPLIT_EPSILON {
                totesel = 0;
            }
        }
        let mut matched = false;
        /* pattern matching over rotations */
        for pat in patterns.iter().flatten() {
            if pat.len != bm.face(face).len {
                continue;
            }
            let plen = pat.len as usize;
            for a in 0..plen {
                let mut m = true;
                for b in 0..plen {
                    let j = (b + a) % plen;
                    if (bm.edge(edges[j]).head.api_flag & SUBD_SPLIT != 0) != (pat.seledges[b] != 0) {
                        m = false;
                        break;
                    }
                }
                if m {
                    matched = true;
                    bm.fpool[face].head.api_flag |= SUBD_SPLIT;
                    facedata.push(SubDFaceData {
                        start: verts_of[a],
                        pat: Some(pat),
                        totedgesel: totesel,
                        face,
                    });
                    break;
                }
            }
            if matched {
                break;
            }
        }
        if !matched && totesel != 0 {
            bm.fpool[face].head.api_flag |= SUBD_SPLIT;
            facedata.push(SubDFaceData {
                start: BM_NIL,
                pat: None,
                totedgesel: totesel,
                face,
            });
        }
    }

    /* subdivide input edges */
    let einput: Vec<usize> = op
        .slots_in
        .iter()
        .find(|s| s.name == "edges")
        .map(|s| s.buf.iter().filter(|(ht, _)| *ht == BM_EDGE).map(|(_, i)| *i).collect())
        .unwrap_or_default();
    for edge in einput {
        let (v1, v2) = (bm.edge(edge).v1, bm.edge(edge).v2);
        bm_subdivide_multicut(bm, edge, &mut params, v1, v2);
    }

    /* copy displaced coords back (CD_SHAPEKEY tmp read) */
    for v in VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if let Some(&co) = params.subd_co.get(&v) {
            bm.vpool[v].co = co;
        }
    }

    /* execute per-face pattern fills */
    for fd in &facedata {
        let face = fd.face;
        if fd.pat.is_none() && fd.totedgesel == 2 {
            /* generic path: split between successive ELE_INNER verts */
            let loops: Vec<usize> = LoopsOfFace::new(bm, face).collect();
            let vlen = loops.len() as i64;
            let nc = numcuts as i64;
            let mut a: i64 = 0;
            for i in 0..vlen {
                let prev = if i != 0 { i - 1 } else { vlen - 1 };
                let lp = loops[prev as usize];
                let lc = loops[i as usize];
                if bm.vert(bm.loo(lp).v).head.api_flag & ELE_INNER == 0
                    && bm.vert(bm.loo(lc).v).head.api_flag & ELE_INNER != 0
                {
                    a = i;
                    break;
                }
            }
            let mut b;
            if bm.vert(bm.loo(loops[((a + nc + 1) % vlen) as usize]).v).head.api_flag & ELE_INNER
                != 0
            {
                b = (a + nc + 1) % vlen;
            } else {
                b = a;
                for j in 0..vlen {
                    let bb = (j + a + nc + 1) % vlen;
                    let prev = if bb == 0 { vlen - 1 } else { bb - 1 };
                    if bm.vert(bm.loo(loops[prev as usize]).v).head.api_flag & ELE_INNER == 0
                        && bm.vert(bm.loo(loops[bb as usize]).v).head.api_flag & ELE_INNER != 0
                    {
                        b = bb;
                        break;
                    }
                }
            }
            b = (b + nc - 1) % vlen;
            if b < 0 {
                b += vlen;
            }
            let mut loops_split: Vec<(usize, usize)> = Vec::new();
            let (mut ai, mut bi) = (a, b);
            for _ in 0..numcuts {
                let mut ok = true;
                let va = bm.loo(loops[ai as usize]).v;
                let vb = bm.loo(loops[bi as usize]).v;
                for other_loop in LoopsOfVert::new(bm, va) {
                    let of = bm.loo(other_loop).f;
                    if of != face && crate::query::bm_vert_in_face(bm, vb, of) {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    loops_split.push((loops[ai as usize], loops[bi as usize]));
                } else {
                    loops_split.push((BM_NIL, BM_NIL));
                }
                bi = (bi - 1).rem_euclid(vlen);
                ai = (ai + 1) % vlen;
            }
            for (l0, l1) in loops_split {
                if l0 != BM_NIL {
                    let mut l_new = BM_NIL;
                    let f_new = crate::mods::bm_face_split(
                        bm, face, l0, l1, Some(&mut l_new), false);
                    if f_new != BM_NIL && l_new != BM_NIL {
                        let e = bm.loo(l_new).e;
                        bm.epool[e].head.api_flag |= ELE_INNER;
                    }
                }
            }
            continue;
        }
        let pat = match fd.pat {
            Some(p) => p,
            None => continue,
        };
        /* reorder verts so pattern index 0 follows fd->start */
        let loops: Vec<usize> = LoopsOfFace::new(bm, face).collect();
        let flen = loops.len();
        let mut a = 0usize;
        for (j, &l) in loops.iter().enumerate() {
            if bm.loo(l).v == fd.start {
                a = j + 1;
                break;
            }
        }
        /* verts[] = the face's CURRENT verts (loops have grown from the edge
         * splits), rotated so verts[0] is the loop after fd->start — this is
         * exactly what the C does. */
        let mut verts: Vec<usize> = vec![BM_NIL; flen];
        for (j, &l) in loops.iter().enumerate() {
            let bi = (j + flen - (a % flen)) % flen;
            verts[bi] = bm.loo(l).v;
        }
        (pat.connectexec)(bm, face, &verts, &mut params);
    }

    /* PORT_PENDING: crease_vert CD layer zeroing on ELE_INNER verts. */

    bmo_slot_buffer_from_enabled_flag(bm, op, true, "geom_inner.out", BM_ALL_NOLOOP, ELE_INNER);
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "geom_split.out", BM_ALL_NOLOOP, ELE_SPLIT);
    bmo_slot_buffer_from_enabled_flag(
        bm, op, true, "geom.out", BM_ALL_NOLOOP, ELE_INNER | ELE_SPLIT | SUBD_SPLIT);
}

const BM_ALL_NOLOOP: u8 = (1 << BM_VERT) | (1 << BM_EDGE) | (1 << BM_FACE);

/// `bmo_bisect_edges_exec`.
pub fn bmo_bisect_edges_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let numcuts = bmo_slot_int_get(op, false, "cuts");
    let mut params = SubDParams {
        numcuts,
        smooth: 0.0,
        smooth_falloff: 0,
        fractal: 0.0,
        along_normal: 0.0,
        use_smooth: false,
        use_smooth_even: false,
        use_sphere: false,
        use_fractal: false,
        seed: 0,
        fractal_ofs: Vec3::ZERO,
        subd_co: HashMap::new(),
        edge_percents: HashMap::new(),
    };
    /* edge_percents slot → EDGE_PERCENT flag + percent map (elem-map values are
     * vertex refs in our model; percent defaults to 0.5 — PORT_PENDING float map). */
    if let Some(s) = op.slots_in.iter().find(|s| s.name == "edge_percents") {
        let keys: Vec<usize> =
            s.map.keys().filter(|(ht, _)| *ht == BM_EDGE).map(|(_, i)| *i).collect();
        for e in keys {
            bm.epool[e].head.api_flag |= EDGE_PERCENT;
            params.edge_percents.insert(e, 0.5);
        }
    }
    for v in VertsOfMesh::new(bm) {
        params.subd_co.insert(v, bm.vert(v).co);
    }
    let edges: Vec<usize> = op
        .slots_in
        .iter()
        .find(|s| s.name == "edges")
        .map(|s| s.buf.iter().filter(|(ht, _)| *ht == BM_EDGE).map(|(_, i)| *i).collect())
        .unwrap_or_default();
    for e in edges {
        let (v1, v2) = (bm.edge(e).v1, bm.edge(e).v2);
        bm_subdivide_multicut(bm, e, &mut params, v1, v2);
    }
    for v in VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if let Some(&co) = params.subd_co.get(&v) {
            bm.vpool[v].co = co;
        }
    }
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "geom_split.out", BM_ALL_NOLOOP, ELE_SPLIT);
}

/// `BM_mesh_esubdivide` — convenience wrapper for `subdivide_edges`.
#[allow(clippy::too_many_arguments)]
pub fn bm_mesh_esubdivide(
    bm: &mut BMesh,
    edge_hflag: u8,
    smooth: f32,
    smooth_falloff: i32,
    use_smooth_even: bool,
    fractal: f32,
    along_normal: f32,
    numcuts: i32,
    seltype: i32,
    cornertype: i32,
    use_single_edge: bool,
    use_grid_fill: bool,
    use_only_quads: bool,
    seed: i32,
) {
    let mut op = crate::operators::bmo_op_init(
        bm, &crate::opdefines::OPDEF_SUBDIVIDE_EDGES, 0);
    {
        let s = op.slots_in.iter_mut().find(|s| s.name == "edges").unwrap();
        for e in EdgesOfMesh::new(bm) {
            if bm.edge(e).head.hflag & edge_hflag != 0 {
                s.buf.push((BM_EDGE, e));
            }
        }
    }
    for (k, v) in [
        ("smooth", format!("{smooth}")),
        ("smooth_falloff", format!("{smooth_falloff}")),
        ("use_smooth_even", format!("{}", use_smooth_even as i32)),
        ("fractal", format!("{fractal}")),
        ("along_normal", format!("{along_normal}")),
        ("cuts", format!("{numcuts}")),
        ("quad_corner_type", format!("{cornertype}")),
        ("use_single_edge", format!("{}", use_single_edge as i32)),
        ("use_grid_fill", format!("{}", use_grid_fill as i32)),
        ("use_only_quads", format!("{}", use_only_quads as i32)),
        ("seed", format!("{seed}")),
    ] {
        if let Some(s) = op.slots_in.iter_mut().find(|s| s.name == k) {
            match s.slot_type {
                crate::operators::BMOSlotType::Int => s.i = v.parse().unwrap_or(0),
                crate::operators::BMOSlotType::Flt => s.f = v.parse().unwrap_or(0.0),
                crate::operators::BMOSlotType::Bool => s.b = v == "1",
                _ => {}
            }
        }
    }
    bmo_op_exec(bm, &mut op, bmo_subdivide_edges_exec);
    /* selection flush per seltype */
    const SUBDIV_SELECT_NONE: i32 = 0;
    const SUBDIV_SELECT_ORIG: i32 = 1;
    const SUBDIV_SELECT_INNER: i32 = 2;
    const SUBDIV_SELECT_LOOPCUT: i32 = 3;
    match seltype {
        SUBDIV_SELECT_NONE => {}
        SUBDIV_SELECT_ORIG => {
            if edge_hflag & crate::BM_ELEM_SELECT != 0 {
                crate::operators::bmo_slot_buffer_hflag_enable(
                    bm, &mut op, true, "geom_inner.out", BM_ALL_NOLOOP,
                    crate::BM_ELEM_SELECT, true);
                crate::marking::bm_mesh_select_flush_from_verts(bm, true);
            }
        }
        SUBDIV_SELECT_INNER => {
            if edge_hflag & crate::BM_ELEM_SELECT != 0 {
                crate::operators::bmo_slot_buffer_hflag_enable(
                    bm, &mut op, true, "geom_inner.out", (1 << BM_VERT) | (1 << BM_EDGE),
                    crate::BM_ELEM_SELECT, true);
            }
        }
        SUBDIV_SELECT_LOOPCUT => {
            if edge_hflag & crate::BM_ELEM_SELECT != 0 {
                crate::marking::bm_mesh_elem_hflag_disable_all(
                    bm, BM_ALL_NOLOOP, crate::BM_ELEM_SELECT);
                crate::operators::bmo_slot_buffer_hflag_enable(
                    bm, &mut op, true, "geom_inner.out", 1 << BM_EDGE,
                    crate::BM_ELEM_SELECT, true);
            }
        }
        _ => {}
    }
    /* PORT_PENDING: BM_mesh_uvselect_flush_post_subdivide (CustomData UVs). */
    crate::operators::bmo_op_finish(bm, op);
}
