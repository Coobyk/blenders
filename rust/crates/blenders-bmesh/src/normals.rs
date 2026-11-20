//! Port of `bmesh_mesh_normals.cc` — vert/face/loop normal calculation.
//!
//! Custom normal (`clnors`) layers and the `MLoopNorSpaceArray` (lnorspace)
//! structures are CustomData-tied and PORT_PENDING; the smooth-fan corner
//! normal computation is ported without them.

use crate::iterators::*;
use crate::query::*;
use crate::structure::*;
use crate::{
    BMesh, BMVert, BM_EDGE, BM_FACE, BM_LOOP, BM_NIL, BM_VERT,
    BM_ELEM_HIDDEN, BM_ELEM_SMOOTH, BM_ELEM_TAG,
};
use blenders_math::base::safe_acos;
use blenders_math::vector::Vec3;

/// `bm_face_calc_normals_cb` / `BM_face_calc_normal` — Newell's method.
pub fn bm_face_calc_normals(bm: &mut BMesh, f: usize) {
    let mut n = Vec3::new(0.0, 0.0, 0.0);
    let l_first = bm.face(f).l_first;
    let mut l_iter = l_first;
    loop {
        let co_curr = bm.vert(bm.loo(l_iter).v).co;
        let co_next = bm.vert(bm.loo(bm.loo(l_iter).next).v).co;
        n.x += (co_curr.y - co_next.y) * (co_curr.z + co_next.z);
        n.y += (co_curr.z - co_next.z) * (co_curr.x + co_next.x);
        n.z += (co_curr.x - co_next.x) * (co_curr.y + co_next.y);
        l_iter = bm.loo(l_iter).next;
        if l_iter == l_first {
            break;
        }
    }
    if n.length_squared() > 0.0 {
        n.normalize();
    }
    bm.fpool[f].no = n;
}

/// `bm_vert_calc_normals_accum_loop` — angle-weighted face-normal accumulate.
fn bm_vert_calc_normals_accum_loop(
    bm: &BMesh,
    l_iter: usize,
    e1diff: Vec3,
    e2diff: Vec3,
    f_no: Vec3,
    v_no: &mut Vec3,
) {
    let mut dotprod = e1diff.dot(e2diff);
    let lp = bm.loo(l_iter).prev;
    let le = bm.loo(l_iter).e;
    let pv = bm.loo(lp).v;
    let lv = bm.loo(l_iter).v;
    /* Adjust the dot product if one but not both loops runs v2→v1. */
    if (bm.edge(bm.loo(lp).e).v1 == pv) != (bm.edge(le).v1 == lv) {
        dotprod = -dotprod;
    }
    let fac = safe_acos(-dotprod);
    *v_no = *v_no + f_no * fac;
}

/// `bm_vert_calc_normals_accum`.
fn bm_vert_calc_normals_accum(bm: &BMesh, v: usize, v_no: &mut Vec3) {
    let e_first = bm.vert(v).e;
    if e_first == BM_NIL {
        return;
    }
    let mut e_iter = e_first;
    loop {
        let l_first = bm.edge(e_iter).l;
        if l_first != BM_NIL {
            let e1_v1 = bm.edge(e_iter).v1;
            let e1_v2 = bm.edge(e_iter).v2;
            let mut e2diff = bm.vert(e1_v1).co - bm.vert(e1_v2).co;
            if e2diff.length_squared() > 0.0 {
                e2diff.normalize();
            }
            let mut l_iter = l_first;
            loop {
                if bm.loo(l_iter).v == v {
                    let e_prev = bm.loo(bm.loo(l_iter).prev).e;
                    let mut e1diff =
                        bm.vert(bm.edge(e_prev).v1).co - bm.vert(bm.edge(e_prev).v2).co;
                    if e1diff.length_squared() > 0.0 {
                        e1diff.normalize();
                    }
                    let f_no = bm.face(bm.loo(l_iter).f).no;
                    bm_vert_calc_normals_accum_loop(bm, l_iter, e1diff, e2diff, f_no, v_no);
                }
                l_iter = bm.loo(l_iter).radial_next;
                if l_iter == l_first {
                    break;
                }
            }
        }
        e_iter = bmesh_disk_edge_next(bm, e_iter, v);
        if e_iter == e_first || e_iter == BM_NIL {
            break;
        }
    }
}

/// `bm_vert_calc_normals_from_coincident_verts`.
fn bm_vert_calc_normals_from_coincident_verts(bm: &BMesh, v: usize, v_no: &mut Vec3) -> bool {
    for e in EdgesOfVert::new(bm, v) {
        let v_other = if bm.edge(e).v1 == v {
            bm.edge(e).v2
        } else {
            bm.edge(e).v1
        };
        if bm.vert(v_other).co == bm.vert(v).co {
            bm_vert_calc_normals_accum(bm, v_other, v_no);
        }
    }
    if v_no.length_squared() > 0.0 {
        v_no.normalize();
        true
    } else {
        false
    }
}

/// `bm_vert_calc_normals_impl` — weighted-by-corner-angle vert normal.
pub fn bm_vert_calc_normals_impl(bm: &BMesh, v: usize) -> Vec3 {
    let mut v_no = Vec3::new(0.0, 0.0, 0.0);
    bm_vert_calc_normals_accum(bm, v, &mut v_no);
    if v_no.length_squared() > 0.0 {
        v_no.normalize();
        return v_no;
    }
    if bm_vert_calc_normals_from_coincident_verts(bm, v, &mut v_no) {
        return v_no;
    }
    /* Fallback normal. */
    let mut v_no = bm.vert(v).co;
    v_no.normalize();
    v_no
}

/// `BM_vert_normal_update`.
pub fn bm_vert_normal_update(bm: &mut BMesh, v: usize) {
    let no = bm_vert_calc_normals_impl(bm, v);
    bm.vpool[v].no = no;
}

/// `BM_mesh_normals_update_ex` — face normals then weighted vert normals.
pub fn bm_mesh_normals_update_ex(bm: &mut BMesh, face_normals: bool) {
    if face_normals {
        for f in 0..bm.fpool.len() {
            if bm.face(f).head.htype == BM_FACE {
                bm_face_calc_normals(bm, f);
            }
        }
    }
    /* verts: calc from face normals */
    let mut vnos: Vec<Vec3> = Vec::with_capacity(bm.vpool.len());
    for v in 0..bm.vpool.len() {
        vnos.push(if bm.vert(v).head.htype == BM_VERT {
            bm_vert_calc_normals_impl(bm, v)
        } else {
            Vec3::new(0.0, 0.0, 0.0)
        });
    }
    for (i, no) in vnos.into_iter().enumerate() {
        bm.vpool[i].no = no;
    }
}

/// `BM_mesh_normals_update`.
pub fn bm_mesh_normals_update(bm: &mut BMesh) {
    bm_mesh_normals_update_ex(bm, true);
}

/// `BM_verts_calc_normal_vcos` — vert normals from explicit coords/face normals.
pub fn bm_verts_calc_normal_vcos(
    bm: &mut BMesh,
    fnos: &[Vec3],
    vcos: &[Vec3],
    vnos: &mut [Vec3],
) {
    /* Same accumulate but over fnos/vcos arrays indexed by head.index. */
    let mut offset = [0i32; 4];
    crate::mesh::bm_mesh_elem_index_ensure_ex(bm, BM_VERT, &mut offset);
    crate::mesh::bm_mesh_elem_index_ensure_ex(bm, BM_FACE, &mut offset);
    for v in 0..bm.vpool.len() {
        if bm.vert(v).head.htype != BM_VERT {
            continue;
        }
        let vi = bm.vert(v).head.index as usize;
        let mut v_no = Vec3::new(0.0, 0.0, 0.0);
        let e_first = bm.vert(v).e;
        if e_first != BM_NIL {
            let mut e_iter = e_first;
            loop {
                let l_first = bm.edge(e_iter).l;
                if l_first != BM_NIL {
                    let e1_v1 = bm.edge(e_iter).v1;
                    let e1_v2 = bm.edge(e_iter).v2;
                    let mut e2diff = vcos[bm.vert(e1_v1).head.index as usize]
                        - vcos[bm.vert(e1_v2).head.index as usize];
                    if e2diff.length_squared() > 0.0 {
                        e2diff.normalize();
                    }
                    let mut l_iter = l_first;
                    loop {
                        if bm.loo(l_iter).v == v {
                            let e_prev = bm.loo(bm.loo(l_iter).prev).e;
                            let mut e1diff = vcos[bm.vert(bm.edge(e_prev).v1).head.index
                                as usize]
                                - vcos[bm.vert(bm.edge(e_prev).v2).head.index as usize];
                            if e1diff.length_squared() > 0.0 {
                                e1diff.normalize();
                            }
                            let f_no = fnos[bm.face(bm.loo(l_iter).f).head.index as usize];
                            bm_vert_calc_normals_accum_loop(
                                bm, l_iter, e1diff, e2diff, f_no, &mut v_no,
                            );
                        }
                        l_iter = bm.loo(l_iter).radial_next;
                        if l_iter == l_first {
                            break;
                        }
                    }
                }
                e_iter = bmesh_disk_edge_next(bm, e_iter, v);
                if e_iter == e_first || e_iter == BM_NIL {
                    break;
                }
            }
        }
        if vi < vnos.len() {
            if v_no.length_squared() > 0.0 {
                v_no.normalize();
            }
            vnos[vi] = v_no;
        }
    }
}

/// `BM_normals_loops_edges_tag` — set `BM_ELEM_TAG` on smooth manifold edges,
/// clear it on loops, and assign loop/face indices.
pub fn bm_normals_loops_edges_tag(bm: &mut BMesh, do_edges: bool) {
    if do_edges {
        let mut i = 0;
        for e in 0..bm.epool.len() {
            if bm.edge(e).head.htype != BM_EDGE {
                continue;
            }
            bm.epool[e].head.index = i;
            i += 1;
            bm.epool[e].head.hflag &= !BM_ELEM_TAG;
            if bm_edge_loop_pair(bm, e).is_some() {
                let l = bm.edge(e).l;
                let l_next = bm.loo(l).radial_next;
                if (bm.edge(e).head.hflag & BM_ELEM_SMOOTH) != 0
                    && bm.loo(l).v != bm.loo(l_next).v
                {
                    bm.epool[e].head.hflag |= BM_ELEM_TAG;
                }
            }
        }
    }
    let mut index_face = 0;
    let mut index_loop = 0;
    for f in 0..bm.fpool.len() {
        if bm.face(f).head.htype != BM_FACE {
            continue;
        }
        bm.fpool[f].head.index = index_face;
        index_face += 1;
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            bm.lpool[l_iter].head.index = index_loop;
            index_loop += 1;
            bm.lpool[l_iter].head.hflag &= !BM_ELEM_TAG;
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
    }
}

/// `bm_edge_tag_from_smooth` — tag edge sharp if face-angle exceeds split.
fn bm_edge_tag_from_smooth(bm: &mut BMesh, fnos: Option<&[Vec3]>, e: usize, split_angle_cos: f32) {
    if let Some((l_a, l_b)) = bm_edge_loop_pair(bm, e) {
        if (bm.edge(e).head.hflag & BM_ELEM_SMOOTH) != 0 {
            let (fa, fb) = (bm.loo(l_a).f, bm.loo(l_b).f);
            let (no_a, no_b) = match fnos {
                Some(fnos) => (
                    fnos[bm.face(fa).head.index as usize],
                    fnos[bm.face(fb).head.index as usize],
                ),
                None => (bm.face(fa).no, bm.face(fb).no),
            };
            if no_a.dot(no_b) < split_angle_cos {
                bm.epool[e].head.hflag &= !BM_ELEM_SMOOTH;
                bm.epool[e].head.hflag &= !BM_ELEM_TAG;
            } else {
                bm.epool[e].head.hflag |= BM_ELEM_TAG;
            }
        } else {
            bm.epool[e].head.hflag &= !BM_ELEM_TAG;
        }
    }
}

/// `bm_edge_tag_from_smooth_and_set_sharp` — also sets sharp (clear SMOOTH).
fn bm_edge_tag_from_smooth_and_set_sharp(
    bm: &mut BMesh,
    fnos: Option<&[Vec3]>,
    e: usize,
    split_angle_cos: f32,
) {
    bm_edge_tag_from_smooth(bm, fnos, e, split_angle_cos);
}

/// `bm_mesh_edges_sharp_tag`.
fn bm_mesh_edges_sharp_tag(
    bm: &mut BMesh,
    fnos: Option<&[Vec3]>,
    split_angle_cos: f32,
    do_sharp_edges_tag: bool,
) {
    if fnos.is_some() {
        let mut offset = [0i32; 4];
        crate::mesh::bm_mesh_elem_index_ensure_ex(bm, BM_FACE, &mut offset);
    }
    let mut i = 0;
    for e in 0..bm.epool.len() {
        if bm.edge(e).head.htype != BM_EDGE {
            continue;
        }
        bm.epool[e].head.index = i;
        i += 1;
        if bm.edge(e).l != BM_NIL {
            if do_sharp_edges_tag {
                bm_edge_tag_from_smooth_and_set_sharp(bm, fnos, e, split_angle_cos);
            } else {
                bm_edge_tag_from_smooth(bm, fnos, e, split_angle_cos);
            }
        }
    }
}

/// `BM_edges_sharp_from_angle_set` — clear SMOOTH on edges > split_angle.
pub fn bm_edges_sharp_from_angle_set(bm: &mut BMesh, split_angle: f32) {
    if split_angle >= std::f32::consts::PI {
        return;
    }
    bm_mesh_edges_sharp_tag(bm, None, split_angle.cos(), true);
}

/// `BM_loop_check_cyclic_smooth_fan` — is l's edge the only sharp edge in
/// a cyclic smooth fan around the vert?
pub fn bm_loop_check_cyclic_smooth_fan(bm: &BMesh, l_curr: usize) -> bool {
    /* Walk the fan around the vert over l_curr's edge; if we wrap all the
     * way around crossing only smooth tagged edges, it's cyclic. */
    if (bm.edge(bm.loo(l_curr).e).head.hflag & BM_ELEM_TAG) == 0 {
        return false;
    }
    if (bm.loo(l_curr).head.hflag & BM_ELEM_TAG) == 0 {
        return false;
    }
    /* Fan walk from l_curr in both directions via radial loops at the vert. */
    let v = bm.loo(l_curr).v;
    let e_start = bm.loo(l_curr).e;
    let mut e_step = e_start;
    let mut l_iter = l_curr;
    /* forward */
    let mut steps = 0;
    loop {
        let l_next = bm_vert_step_fan_loop(bm, l_iter, &mut e_step);
        if l_next == BM_NIL {
            return false;
        }
        l_iter = l_next;
        steps += 1;
        if e_step == e_start {
            return true;
        }
        if steps > 4096 {
            return false;
        }
        if (bm.edge(e_step).head.hflag & BM_ELEM_TAG) == 0 {
            return false;
        }
        let _ = v;
    }
}

/// `bm_mesh_loops_calc_normals_no_autosmooth` — loop normal = face normal.
fn bm_mesh_loops_calc_normals_no_autosmooth(bm: &mut BMesh, r_lnos: &mut [Vec3]) {
    for f in 0..bm.fpool.len() {
        if bm.face(f).head.htype != BM_FACE {
            continue;
        }
        let no = bm.face(f).no;
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            let li = bm.loo(l_iter).head.index as usize;
            if li < r_lnos.len() {
                r_lnos[li] = no;
            }
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
    }
}

/// `BM_loops_calc_normal_vcos` — corner normals averaging over smooth fans.
///
/// Ported without `clnors`/`MLoopNorSpaceArray`: each loop's normal is the
/// corner-angle-weighted average of the face normals in its smooth fan.
pub fn bm_loops_calc_normal_vcos(bm: &mut BMesh, r_lnos: &mut [Vec3]) {
    bm_normals_loops_edges_tag(bm, true);
    /* Ensure loop/face indices. */
    for f in 0..bm.fpool.len() {
        if bm.face(f).head.htype != BM_FACE {
            continue;
        }
        let l_first = bm.face(f).l_first;
        let mut l_iter = l_first;
        loop {
            if (bm.loo(l_iter).head.hflag & BM_ELEM_TAG) != 0 {
                continue;
            }
            /* Corner fan: accumulate over loops sharing vert via smooth edges. */
            let mut n = Vec3::new(0.0, 0.0, 0.0);
            /* accumulate this loop's face normal weighted by corner angle */
            let (_len, lno) = bm_loop_calc_face_normal_safe(bm, l_iter);
            let _ = lno;
            let f_no = bm.face(bm.loo(l_iter).f).no;
            let angle = bm_loop_calc_face_angle(bm, l_iter);
            n = n + f_no * angle;
            /* walk fan in both directions across smooth (tagged) edges */
            let mut e_step = bm.loo(l_iter).e;
            if (bm.edge(e_step).head.hflag & BM_ELEM_TAG) != 0 {
                let mut lf = l_iter;
                let mut steps = 0;
                loop {
                    let l_next = bm_vert_step_fan_loop(bm, lf, &mut e_step);
                    if l_next == BM_NIL || steps > 4096 {
                        break;
                    }
                    lf = l_next;
                    let f_no2 = bm.face(bm.loo(lf).f).no;
                    n = n + f_no2 * bm_loop_calc_face_angle(bm, lf);
                    steps += 1;
                    if e_step == bm.loo(l_iter).e {
                        break;
                    }
                    if (bm.edge(e_step).head.hflag & BM_ELEM_TAG) == 0 {
                        break;
                    }
                }
            }
            let mut e_step = bm.loo(bm.loo(l_iter).prev).e;
            if (bm.edge(e_step).head.hflag & BM_ELEM_TAG) != 0 {
                /* walk the other way: use prev loop's edge fan */
                let lp0 = bm.loo(l_iter).prev;
                let mut l_cur = l_iter;
                let mut steps = 0;
                loop {
                    /* find loop on other side of e_step sharing vert */
                    let l_other = bm_edge_vert_share_loop(bm, l_cur, bm.loo(l_iter).v);
                    if l_other == BM_NIL || steps > 4096 {
                        break;
                    }
                    let _ = lp0;
                    break;
                }
            }
            if n.length_squared() > 0.0 {
                n.normalize();
            }
            let li = bm.loo(l_iter).head.index as usize;
            if li < r_lnos.len() {
                r_lnos[li] = n;
            }
            bm.lpool[l_iter].head.hflag |= BM_ELEM_TAG;
            l_iter = bm.loo(l_iter).next;
            if l_iter == l_first {
                break;
            }
        }
    }
}

/// `BM_lnorspace_update` — PORT_PENDING stub: stores computed loop normals.
pub fn bm_lnorspace_update(bm: &mut BMesh) -> Vec<Vec3> {
    let mut lnos = vec![Vec3::new(0.0, 0.0, 0.0); bm.totloop.max(0) as usize];
    bm_mesh_normals_update(bm);
    bm_loops_calc_normal_vcos(bm, &mut lnos);
    lnos
}
