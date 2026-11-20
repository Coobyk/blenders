//! Port of `bmo_circularize.cc` — circle-fitting of boundary/wire loops.
//! BVH over looptris → linear scan (port convention).

use std::collections::HashMap;
use crate::edgeloop::{BMEdgeLoopFindParams, bm_edgeloop_is_closed, bm_edgeloop_verts_get,
                      bm_mesh_edgeloops_find};
use crate::operators::*;
use crate::{BMesh, BM_EDGE, BM_FACE, BM_VERT};
use blenders_math::matrix::Mat3;
use blenders_math::vector::{Vec2, Vec3};

const NON_LINEAR_LS_MAX_ITER: i32 = 500;
const MIRROR_LIMIT: f32 = 0.001;
const EPS: f32 = 1e-6;
const FIT_METHOD_CONTRACT: i32 = 1;

struct CircleVert {
    v: usize,
    co_2d: Vec2,
    target_2d: Vec2,
}

struct VertChain {
    verts: Vec<usize>,
    is_closed: bool,
    is_wire: bool,
}

/// `is_valid_boundary_edge`.
fn is_valid_boundary_edge(
    bm: &BMesh, e: usize, hflag: u8, check_axis: [bool; 3], wire: bool) -> bool
{
    if bm.edge(e).head.hflag & hflag == 0
        || bm.edge(e).head.hflag & crate::BM_ELEM_HIDDEN != 0
    { return false; }
    if (bm.edge(e).l != crate::BM_NIL) != wire { return false; }
    let mut sel_faces = 0;
    for f in crate::iterators::FacesOfEdge::new(bm, e) {
        if bm.face(f).head.hflag & crate::BM_ELEM_HIDDEN == 0
            && bm.face(f).head.hflag & hflag != 0
        {
            sel_faces += 1;
            if sel_faces > 1 { break; }
        }
    }
    if sel_faces > 1 { return false; }
    let (c1, c2) = (bm.vert(bm.edge(e).v1).co, bm.vert(bm.edge(e).v2).co);
    for i in 0..3 {
        if check_axis[i] && c1[i].abs() < MIRROR_LIMIT && c2[i].abs() < MIRROR_LIMIT {
            return false;
        }
    }
    true
}

/// `bm_vert_chain_extract_from_boundary_edges`.
fn bm_vert_chain_extract_from_boundary_edges(
    bm: &mut BMesh, hflag: u8, check_axis: [bool; 3], wire: bool) -> Vec<VertChain>
{
    let params = BMEdgeLoopFindParams { use_vert_junction: true };
    let eloops = bm_mesh_edgeloops_find(
        bm, &|bm: &BMesh, e| is_valid_boundary_edge(bm, e, hflag, check_axis, wire),
        Some(&params));
    let mut chains = Vec::new();
    for el in &eloops {
        let verts: Vec<usize> = bm_edgeloop_verts_get(el).iter().copied().collect();
        if wire {
            let mut touches_face_region = false;
            'outer: for &v in &verts {
                for e in crate::iterators::EdgesOfVert::new(bm, v) {
                    if bm.edge(e).head.hflag & hflag != 0 && bm.edge(e).l != crate::BM_NIL {
                        touches_face_region = true;
                        break 'outer;
                    }
                }
            }
            if touches_face_region { continue; }
        }
        if verts.len() >= 3 {
            chains.push(VertChain {
                verts,
                is_closed: bm_edgeloop_is_closed(el),
                is_wire: wire,
            });
        }
    }
    chains
}

/// `bm_vert_chain_orientation_matrix_calc` → (center, mat[z=up]).
fn chain_orientation_matrix(bm: &BMesh, chain: &[usize]) -> Option<(Vec3, Mat3)> {
    let mut center = Vec3::ZERO;
    for &v in chain { center = center + bm.vert(v).co; }
    center = center / chain.len() as f32;
    let mut normal = Vec3::ZERO;
    let mut prev = *chain.last().unwrap();
    for &curr in chain {
        blenders_math::geom::add_newell_cross(
            &mut normal, bm.vert(prev).co, bm.vert(curr).co);
        prev = curr;
    }
    if normal.length() < EPS { return None; }
    let normal = normal.normalized();
    let (n1, n2) = blenders_math::geom::ortho_basis(normal);
    Some((center, Mat3 { cols: [n1, n2, normal] }))
}

/// `project_chain_to_2d`.
fn project_chain_to_2d(bm: &BMesh, chain: &[usize], center: Vec3, mat: Mat3) -> Vec<CircleVert> {
    chain.iter().map(|&v| {
        let vec = bm.vert(v).co - center;
        CircleVert {
            v,
            co_2d: Vec2::new(vec.dot(mat.cols[0]), vec.dot(mat.cols[1])),
            target_2d: Vec2::ZERO,
        }
    }).collect()
}

/// `calculate_circle_best_fit` — Gauss–Newton least squares.
fn calculate_circle_best_fit(
    verts: &[CircleVert], fixed_center: Option<Vec2>) -> (Vec2, f32)
{
    if let Some(c) = fixed_center {
        let mut radius = 0.0;
        for cv in verts { radius += cv.co_2d.length(); }
        return (c, radius / verts.len() as f32);
    }
    let mut center = Vec2::ZERO;
    let mut radius = 1.0f32;
    let mut converged = false;
    for _ in 0..NON_LINEAR_LS_MAX_ITER {
        let mut normal_matrix = [[0.0f32; 3]; 3]; /* row-major accum */
        let mut jtr = Vec3::ZERO;
        for cv in verts {
            let d_vec = center - cv.co_2d;
            let dist = d_vec.length();
            if dist < EPS { continue; }
            let j_row = [d_vec.x / dist, d_vec.y / dist, -1.0];
            let residual = radius - dist;
            for row in 0..3 {
                for col in 0..3 {
                    normal_matrix[col][row] += j_row[row] * j_row[col];
                }
                jtr[row] += j_row[row] * residual;
            }
        }
        let m3 = Mat3 { cols: [
            Vec3::new(normal_matrix[0][0], normal_matrix[0][1], normal_matrix[0][2]),
            Vec3::new(normal_matrix[1][0], normal_matrix[1][1], normal_matrix[1][2]),
            Vec3::new(normal_matrix[2][0], normal_matrix[2][1], normal_matrix[2][2]),
        ]};
        let Some(inv) = m3.inverted() else { break };
        let delta = inv * jtr;
        center.x += delta.x;
        center.y += delta.y;
        radius += delta.z;
        if delta.x.abs() < EPS && delta.y.abs() < EPS && delta.z.abs() < EPS {
            converged = true;
            break;
        }
    }
    if !converged {
        center = Vec2::ZERO;
        radius = 0.0;
        for cv in verts { radius += cv.co_2d.length(); }
        radius /= verts.len() as f32;
    }
    (center, radius)
}

/// `calculate_circle_inside_fit` — edge-length-weighted centroid + min dist.
fn calculate_circle_inside_fit(
    verts: &[CircleVert], fixed_center: Option<Vec2>) -> (Vec2, f32)
{
    let center = if let Some(c) = fixed_center {
        c
    } else {
        let mut center = Vec2::ZERO;
        let mut total_len = 0.0;
        let mut prev = verts.last().unwrap().co_2d;
        for cv in verts {
            let len = (prev - cv.co_2d).length();
            center = center + (prev + cv.co_2d) * len;
            total_len += len;
            prev = cv.co_2d;
        }
        if total_len != 0.0 { center = center * (0.5 / total_len); }
        center
    };
    let mut radius_sq = f32::MAX;
    for cv in verts {
        radius_sq = radius_sq.min((center - cv.co_2d).length_squared());
    }
    (center, radius_sq.sqrt())
}

/// `calculate_target_locations`.
fn calculate_target_locations(
    verts: &mut [CircleVert], center: Vec2, radius: f32,
    is_regular: bool, is_closed: bool, rotation_angle: f32)
{
    let mut step = 0.0;
    let mut start_angle = 0.0;
    if is_regular {
        let mut total_angle = 2.0 * std::f32::consts::PI;
        let mut divisions = verts.len();
        if !is_closed && divisions > 1 {
            total_angle = 0.0;
            divisions = verts.len() - 1;
            let mut vec_prev = (verts[0].co_2d - center).normalized();
            for i in 1..verts.len() {
                let vec_curr = (verts[i].co_2d - center).normalized();
                total_angle -= blenders_math::vector::angle_signed_v2v2(vec_prev, vec_curr);
                vec_prev = vec_curr;
            }
            let max_angle = 2.0 * std::f32::consts::PI;
            total_angle = total_angle.clamp(-max_angle, max_angle);
        }
        step = total_angle / divisions as f32;
        let mut sum_sin = 0.0;
        let mut sum_cos = 0.0;
        for (i, cv) in verts.iter().enumerate() {
            let vec = cv.co_2d - center;
            let angle_diff = vec.y.atan2(vec.x) - step * i as f32;
            sum_sin += angle_diff.sin();
            sum_cos += angle_diff.cos();
        }
        start_angle = sum_sin.atan2(sum_cos);
    }
    for (i, cv) in verts.iter_mut().enumerate() {
        let angle = if is_regular {
            start_angle + step * i as f32 - rotation_angle
        } else {
            let vec = cv.co_2d - center;
            vec.y.atan2(vec.x) - rotation_angle
        };
        cv.target_2d = Vec2::new(
            center.x + angle.cos() * radius, center.y + angle.sin() * radius);
    }
}

type TessCache = HashMap<usize, Vec<[usize; 3]>>;

/// `project_on_mesh` — project `center_pos` along `normal` onto the
/// faces/edges around `v`, else nearest loop-tri (linear scan, was BVH).
fn project_on_mesh(
    bm: &mut BMesh, looptris: &[[usize; 3]], v: usize, center_pos: Vec3, normal: Vec3,
    tess_cache: &mut TessCache) -> Vec3
{
    let vec0 = center_pos - bm.vert(v).co;
    let length = vec0.length();
    if length == 0.0 { return center_pos; }
    let vec = vec0 / length;
    let angle = blenders_math::geom::angle_normalized_v3v3(vec, normal);
    if angle.abs() < EPS || (std::f32::consts::PI - angle).abs() < EPS {
        return bm.vert(v).co;
    }
    let p2 = center_pos + normal;
    let mut best = f32::MAX;
    let mut r_pos = center_pos;
    let mut found = false;
    let mut test_tri = |bm: &BMesh, a: usize, b: usize, c: usize,
                        best: &mut f32, r_pos: &mut Vec3, found: &mut bool| {
        if let Some((lambda, _u, _v)) = blenders_math::geom::isect_line_tri(
            center_pos, p2, bm.vert(a).co, bm.vert(b).co, bm.vert(c).co)
        {
            let hit = center_pos + normal * lambda;
            let d = (center_pos - hit).length_squared();
            if d < *best {
                *best = d;
                *r_pos = hit;
                *found = true;
            }
        }
    };
    for f in crate::iterators::FacesOfVert::new(bm, v).collect::<Vec<_>>() {
        if bm.face(f).len < 3 || bm.face(f).head.hflag & crate::BM_ELEM_HIDDEN != 0 {
            continue;
        }
        let flen = bm.face(f).len;
        if flen == 3 || flen == 4 {
            let l_start = bm.face(f).l_first;
            let v1 = bm.loo(bm.loo(l_start).prev).v;
            let v2 = bm.loo(l_start).v;
            let v3 = bm.loo(bm.loo(l_start).next).v;
            test_tri(bm, v1, v2, v3, &mut best, &mut r_pos, &mut found);
            if flen == 4 {
                let v4 = bm.loo(bm.loo(bm.loo(l_start).next).next).v;
                test_tri(bm, v1, v3, v4, &mut best, &mut r_pos, &mut found);
            }
        } else {
            if !tess_cache.contains_key(&f) {
                let mut looptris_f: Vec<[usize; 3]> = Vec::new();
                crate::tessellate::bmesh_calc_tessellation_for_face_impl(
                    bm, &mut looptris_f, f, true);
                let tris: Vec<[usize; 3]> = looptris_f.iter().map(|&[a, b, c]|
                    [bm.loo(a).v, bm.loo(b).v, bm.loo(c).v]).collect();
                tess_cache.insert(f, tris);
            }
            for &[a, b, c] in &tess_cache[&f] {
                test_tri(bm, a, b, c, &mut best, &mut r_pos, &mut found);
            }
        }
    }
    if found { return r_pos; }
    for e in crate::iterators::EdgesOfVert::new(bm, v) {
        let (c1, c2) = (bm.vert(bm.edge(e).v1).co, bm.vert(bm.edge(e).v2).co);
        let (closest, _l) = blenders_math::geom::closest_to_line(center_pos, c1, c2);
        let fac = blenders_math::geom::line_point_factor(closest, c1, c2);
        if fac > EPS && fac < 1.0 - EPS {
            let d = (center_pos - closest).length_squared();
            if d < best {
                best = d;
                r_pos = closest;
                found = true;
            }
        }
    }
    if found { return r_pos; }
    /* BVH nearest → linear scan over all tris */
    let mut best_d = f32::MAX;
    let mut best_hit = center_pos;
    let mut any = false;
    for lt in looptris {
        let (co, _) = blenders_math::geom::closest_on_tri_to_point(
            center_pos, bm.vert(bm.loo(lt[0]).v).co,
            bm.vert(bm.loo(lt[1]).v).co, bm.vert(bm.loo(lt[2]).v).co);
        let d = (center_pos - co).length_squared();
        if d < best_d {
            best_d = d;
            best_hit = co;
            any = true;
        }
    }
    if any { best_hit } else { center_pos }
}

/// `bmo_circularize_exec`.
pub fn bmo_circularize_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let factor = bmo_slot_float_get(op, false, "factor");
    let custom_radius = bmo_slot_float_get(op, false, "custom_radius");
    let angle = bmo_slot_float_get(op, false, "angle");
    let fit_method = bmo_slot_int_get(op, false, "fit_method");
    let flatten = bmo_slot_float_get(op, false, "flatten");
    let regular = bmo_slot_bool_get(op, false, "regular");
    let check_axis = [
        bmo_slot_bool_get(op, false, "mirror_x"),
        bmo_slot_bool_get(op, false, "mirror_y"),
        bmo_slot_bool_get(op, false, "mirror_z"),
    ];
    let lock_x = bmo_slot_bool_get(op, false, "lock_x");
    let lock_y = bmo_slot_bool_get(op, false, "lock_y");
    let lock_z = bmo_slot_bool_get(op, false, "lock_z");
    crate::marking::bm_mesh_elem_hflag_disable_all(
        bm, (1 << BM_VERT) | (1 << BM_EDGE) | (1 << BM_FACE), crate::BM_ELEM_TAG);
    bmo_slot_buffer_hflag_enable(
        bm, op, false, "geom",
        (1 << BM_VERT) | (1 << BM_EDGE) | (1 << BM_FACE), crate::BM_ELEM_TAG, false);
    let mut chains: Vec<VertChain> = Vec::new();
    for extract_wire in [false, true] {
        if !extract_wire && bm.totface == 0 { continue; }
        chains.extend(bm_vert_chain_extract_from_boundary_edges(
            bm, crate::BM_ELEM_TAG, check_axis, extract_wire));
    }
    let mut looptris: Vec<[usize; 3]> = Vec::new();
    if flatten < 1.0 && bm.totface > 0 {
        crate::tessellate::bm_mesh_calc_tessellation(bm, &mut looptris);
    }
    let mut tess_cache: TessCache = HashMap::new();
    for chain_data in &mut chains {
        let mut normal_accum = Vec3::ZERO;
        for &v in &chain_data.verts { normal_accum = normal_accum + bm.vert(v).no; }
        let Some((mut center_3d, mut mat)) = chain_orientation_matrix(bm, &chain_data.verts)
        else {
            /* BMO_error_raise: cancel silently (error system pending) */
            return;
        };
        if mat.cols[2].dot(normal_accum) < 0.0 {
            chain_data.verts.reverse();
            let Some((c, m)) = chain_orientation_matrix(bm, &chain_data.verts)
            else { return };
            center_3d = c;
            mat = m;
        }
        let mut is_mirrored = false;
        let mut mirror_axis = usize::MAX;
        if !chain_data.is_closed {
            let cs = bm.vert(chain_data.verts[0]).co;
            let ce = bm.vert(*chain_data.verts.last().unwrap()).co;
            for i in 0..3 {
                if check_axis[i] && cs[i].abs() < MIRROR_LIMIT && ce[i].abs() < MIRROR_LIMIT {
                    is_mirrored = true;
                    mirror_axis = i;
                }
            }
        }
        if is_mirrored {
            let cs = bm.vert(chain_data.verts[0]).co;
            let ce = bm.vert(*chain_data.verts.last().unwrap()).co;
            center_3d = (cs + ce) / 2.0;
            let p = (cs - center_3d).normalized();
            let q = mat.cols[2].cross(p).normalized();
            mat.cols[0] = p;
            mat.cols[1] = q;
        }
        let mut circle_verts = project_chain_to_2d(bm, &chain_data.verts, center_3d, mat);
        let fixed_center = if is_mirrored { Some(Vec2::ZERO) } else { None };
        let (center_2d, mut radius) = if fit_method == FIT_METHOD_CONTRACT {
            calculate_circle_inside_fit(&circle_verts, fixed_center)
        } else {
            calculate_circle_best_fit(&circle_verts, fixed_center)
        };
        if custom_radius > 0.0 { radius = custom_radius; }
        calculate_target_locations(
            &mut circle_verts, center_2d, radius, regular, chain_data.is_closed, angle);
        for cv in &circle_verts {
            let target_local = Vec3::new(cv.target_2d.x, cv.target_2d.y, 0.0);
            let mut final_pos = center_3d + mat * target_local;
            if flatten < 1.0 && !chain_data.is_wire {
                let projected = project_on_mesh(
                    bm, &looptris, cv.v, final_pos, mat.cols[2], &mut tess_cache);
                final_pos = projected + (final_pos - projected) * flatten;
            }
            if is_mirrored && mirror_axis != usize::MAX
                && (cv.v == chain_data.verts[0] || cv.v == *chain_data.verts.last().unwrap())
            {
                final_pos[mirror_axis] = 0.0;
            }
            if lock_x || lock_y || lock_z {
                let orig = bm.vert(cv.v).co;
                if lock_x { final_pos.x = orig.x; }
                if lock_y { final_pos.y = orig.y; }
                if lock_z { final_pos.z = orig.z; }
            }
            let cur = bm.vert(cv.v).co;
            bm.vpool[cv.v].co = cur + (final_pos - cur) * factor;
        }
    }
}
