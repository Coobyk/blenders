//! Port of `bmo_subdivide_edgering.cc` — subdivide an edge ring into
//! `cuts` connecting edge loops, with bezier/path/surface interpolation.
//!
//! C used `BMEdgeLoopStore*` pointers for pair identity; here loops live in a
//! `Vec<BMEdgeLoopStore>` and indices play that role.

use crate::edgeloop::*;
use crate::iterators::*;
use crate::operators::*;
use crate::BMesh;
use crate::{BM_EDGE, BM_ELEM_TAG, BM_FACE, BM_NIL, BM_VERT};
use blenders_math::vector::Vec3;
use std::collections::{HashMap, HashSet};
use std::collections::VecDeque;

const VERT_SHARED: u8 = 1 << 0;

const EDGE_RING: u8 = 1 << 0;
const EDGE_RIM: u8 = 1 << 1;
const EDGE_IN_STACK: u8 = 1 << 2;

const FACE_OUT: u8 = 1 << 0;
const FACE_SHARED: u8 = 1 << 1;
const FACE_IN_STACK: u8 = 1 << 2;

const SUBD_RING_INTERP_LINEAR: i32 = 0;
const SUBD_RING_INTERP_PATH: i32 = 1;
const SUBD_RING_INTERP_SURF: i32 = 2;

/// `bezier_handle_calc_length_v3`.
fn bezier_handle_calc_length_v3(co_a: Vec3, no_a: Vec3, co_b: Vec3, no_b: Vec3) -> f32 {
    let dot = no_a.dot(no_b);
    /* gives closest approx at a circle with 2 parallel handles */
    let mut fac = 1.333333_f32;
    if dot < 0.0 {
        /* Scale down to 0.666 if we point directly at each other. */
        let t = 1.0 + dot;
        fac = (fac * t) + (0.75 * (1.0 - t));
    }
    /* 2d length projected on plane of normals */
    let len;
    let mut co_a_ofs = no_a.cross(no_b);
    if co_a_ofs.length_squared() > f32::EPSILON {
        co_a_ofs += co_a;
        co_a_ofs = blenders_math::geom::closest_to_line(co_b, co_a, co_a_ofs).0;
    } else {
        co_a_ofs = co_a;
    }
    len = (co_a_ofs - co_b).length();
    (len * 0.5) * fac
}

/// `bm_edgeloop_vert_tag` (hflag TAG on all loop verts).
fn bm_edgeloop_vert_tag(bm: &mut BMesh, el_store: &BMEdgeLoopStore, tag: bool) {
    for &v in &el_store.verts {
        if tag {
            bm.vpool[v].head.hflag |= BM_ELEM_TAG;
        } else {
            bm.vpool[v].head.hflag &= !BM_ELEM_TAG;
        }
    }
}

/// `bmo_edgeloop_vert_tag` (oflag on all loop verts).
fn bmo_edgeloop_vert_tag(bm: &mut BMesh, el_store: &BMEdgeLoopStore, oflag: u8, tag: bool) {
    for &v in &el_store.verts {
        if tag {
            bm.vpool[v].head.api_flag |= oflag;
        } else {
            bm.vpool[v].head.api_flag &= !oflag;
        }
    }
}

/// `bmo_face_is_vert_tag_all`.
fn bmo_face_is_vert_tag_all(bm: &BMesh, f: usize, oflag: u8) -> bool {
    let l_first = bm.face(f).l_first;
    let mut l = l_first;
    loop {
        if bm.vert(bm.loo(l).v).head.api_flag & oflag == 0 {
            return false;
        }
        l = bm.loo(l).next;
        if l == l_first {
            return true;
        }
    }
}

/// `bm_vert_is_tag_edge_connect`.
fn bm_vert_is_tag_edge_connect(bm: &BMesh, v: usize) -> bool {
    for e in EdgesOfVert::new(bm, v) {
        if bm.edge(e).head.api_flag & EDGE_RING != 0 {
            let v_other = if bm.edge(e).v1 == v { bm.edge(e).v2 } else { bm.edge(e).v1 };
            if bm.vert(v_other).head.hflag & BM_ELEM_TAG != 0 {
                return true;
            }
        }
    }
    false
}

/// `bm_edgeloop_check_overlap_all`.
fn bm_edgeloop_check_overlap_all(
    bm: &mut BMesh,
    el_store_a: &BMEdgeLoopStore,
    el_store_b: &BMEdgeLoopStore,
) -> bool {
    let mut has_overlap = true;
    bm_edgeloop_vert_tag(bm, el_store_a, false);
    bm_edgeloop_vert_tag(bm, el_store_b, true);
    'outer: {
        for &v in &el_store_a.verts {
            if !bm_vert_is_tag_edge_connect(bm, v) {
                has_overlap = false;
                break 'outer;
            }
        }
        bm_edgeloop_vert_tag(bm, el_store_a, true);
        bm_edgeloop_vert_tag(bm, el_store_b, false);
        for &v in &el_store_b.verts {
            if !bm_vert_is_tag_edge_connect(bm, v) {
                has_overlap = false;
                break 'outer;
            }
        }
    }
    bm_edgeloop_vert_tag(bm, el_store_a, false);
    bm_edgeloop_vert_tag(bm, el_store_b, false);
    has_overlap
}

/// `bm_edgering_pair_calc` — deduped eloop index pairs sharing ring edges.
fn bm_edgering_pair_calc(
    bm: &BMesh,
    eloops_rim: &[BMEdgeLoopStore],
) -> Vec<(usize, usize)> {
    /* vert -> eloop index map */
    let mut vert_eloop_map: HashMap<usize, usize> = HashMap::new();
    for (i, el) in eloops_rim.iter().enumerate() {
        for &v in &el.verts {
            vert_eloop_map.insert(v, i);
        }
    }
    /* collect eloop pairs */
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    let mut pairs: Vec<(usize, usize)> = Vec::new();
    for (i, el) in eloops_rim.iter().enumerate() {
        let v = el.verts[0];
        for e in EdgesOfVert::new(bm, v) {
            if bm.edge(e).head.api_flag & EDGE_RING != 0 {
                let v_other =
                    if bm.edge(e).v1 == v { bm.edge(e).v2 } else { bm.edge(e).v1 };
                if let Some(&other) = vert_eloop_map.get(&v_other) {
                    let pair = if i < other { (i, other) } else { (other, i) };
                    if seen.insert(pair) {
                        pairs.push(pair);
                    }
                }
            }
        }
    }
    pairs
}

/// `bm_edge_subdiv_as_loop`.
fn bm_edge_subdiv_as_loop(
    bm: &mut BMesh,
    eloops: &mut Vec<BMEdgeLoopStore>,
    e: usize,
    v_a: usize,
    cuts: i32,
) {
    let mut v_arr = vec![BM_NIL; (cuts + 2) as usize];
    debug_assert!(bm.edge(e).v1 == v_a || bm.edge(e).v2 == v_a);
    let v_b = if bm.edge(e).v1 == v_a { bm.edge(e).v2 } else { bm.edge(e).v1 };
    crate::mods::bm_edge_split_n(bm, e, cuts, &mut v_arr[1..]);
    let e_v1 = bm.edge(e).v1;
    if v_a == e_v1 {
        v_arr[0] = v_a;
        v_arr[(cuts + 1) as usize] = v_b;
    } else {
        v_arr[0] = v_b;
        v_arr[(cuts + 1) as usize] = v_a;
    }
    let mut eloop = bm_edgeloop_from_verts(&v_arr, false);
    if v_a == e_v1 {
        bm_edgeloop_flip(&mut eloop);
    }
    eloops.push(eloop);
}

/// `bm_vert_calc_surface_tangent` — resulting normal always points towards
/// `FACE_SHARED`. Must be called after all loops have been created, but before
/// any mesh modifications.
fn bm_vert_calc_surface_tangent(bm: &BMesh, v: usize) -> Vec3 {
    /* get outer normal, fall back to inner (if this vertex is on a boundary) */
    let mut found_outer = false;
    let mut found_inner = false;
    let mut found_outer_tag = false;
    let mut no_outer = Vec3::ZERO;
    let mut no_inner = Vec3::ZERO;
    /* first find rim edges, typically we will only add 2 normals */
    for e in EdgesOfVert::new(bm, v) {
        if bm.edge(e).l == BM_NIL {
            /* wire edge — pass, this may confuse things */
        } else if bm.edge(e).head.api_flag & EDGE_RIM != 0 {
            let l_first = bm.edge(e).l;
            let mut l = l_first;
            loop {
                /* use unmarked (surrounding) faces to create surface tangent */
                let no = crate::query::bm_edge_calc_face_tangent(bm, e, l);
                let lf = bm.loo(l).f;
                if bm.face(lf).head.api_flag & FACE_SHARED != 0 {
                    no_inner += no;
                    found_inner = true;
                } else {
                    no_outer += no;
                    found_outer = true;
                    /* other side is used too, blend midway */
                    if bm.face(lf).head.api_flag & FACE_OUT != 0 {
                        found_outer_tag = true;
                    }
                }
                l = bm.loo(l).radial_next;
                if l == l_first {
                    break;
                }
            }
        }
    }
    /* detect if this vertex is in-between 2 loops (when blending multiple),
     * if so - take both inner and outer into account */
    if found_inner && found_outer_tag {
        /* blend between the 2 */
        (-no_outer.normalized() + no_inner.normalized()).normalized()
    } else if found_outer {
        -no_outer.normalized()
    } else {
        /* we always have inner geometry */
        debug_assert!(found_inner);
        no_inner.normalized()
    }
}

/// `bm_faces_share_tag_flush` — tag faces connected to an edge loop as
/// `FACE_SHARED` if all vertices are `VERT_SHARED`.
fn bm_faces_share_tag_flush(bm: &mut BMesh, e_arr: &[usize]) {
    for &e in e_arr {
        let l_first = bm.edge(e).l;
        let mut l = l_first;
        loop {
            let f = bm.loo(l).f;
            if bm.face(f).head.api_flag & FACE_SHARED == 0
                && bmo_face_is_vert_tag_all(bm, f, VERT_SHARED)
            {
                bm.fpool[f].head.api_flag |= FACE_SHARED;
            }
            l = bm.loo(l).radial_next;
            if l == l_first {
                break;
            }
        }
    }
}

/// `bm_faces_share_tag_clear`.
fn bm_faces_share_tag_clear(bm: &mut BMesh, e_arr: &[usize]) {
    for &e in e_arr {
        let l_first = bm.edge(e).l;
        let mut l = l_first;
        loop {
            let f = bm.loo(l).f;
            bm.fpool[f].head.api_flag &= !FACE_SHARED;
            l = bm.loo(l).radial_next;
            if l == l_first {
                break;
            }
        }
    }
}

/// `LoopPairStore` — vert spline-handle normals per loop, plus vert→index map.
struct LoopPairStore {
    nors_a: Vec<Vec3>,
    nors_b: Vec<Vec3>,
    nors_gh_a: HashMap<usize, usize>,
    nors_gh_b: HashMap<usize, usize>,
}

/// `bm_edgering_pair_store_create`.
fn bm_edgering_pair_store_create(
    bm: &mut BMesh,
    el_store_a: &BMEdgeLoopStore,
    el_store_b: &BMEdgeLoopStore,
    interp_mode: i32,
) -> LoopPairStore {
    let mut lpair = LoopPairStore {
        nors_a: Vec::new(),
        nors_b: Vec::new(),
        nors_gh_a: HashMap::new(),
        nors_gh_b: HashMap::new(),
    };
    if interp_mode == SUBD_RING_INTERP_SURF {
        let len_a = bm_edgeloop_length_get(el_store_a) as usize;
        let len_b = bm_edgeloop_length_get(el_store_b) as usize;
        let e_arr_a = bm_edgeloop_edges_get(bm, el_store_a);
        let e_arr_b = bm_edgeloop_edges_get(bm, el_store_b);
        lpair.nors_a = vec![Vec3::ZERO; len_a];
        lpair.nors_b = vec![Vec3::ZERO; len_b];

        /* all other verts must _not_ be tagged */
        bmo_edgeloop_vert_tag(bm, el_store_a, VERT_SHARED, true);
        bmo_edgeloop_vert_tag(bm, el_store_b, VERT_SHARED, true);
        /* tag all faces that are in-between both loops */
        bm_faces_share_tag_flush(bm, &e_arr_a);
        bm_faces_share_tag_flush(bm, &e_arr_b);

        /* now we have all data we need, calculate vertex spline nor! */
        for side in 0..2 {
            let (el_store, nors, nors_gh) = if side == 0 {
                (el_store_a, &mut lpair.nors_a, &mut lpair.nors_gh_a)
            } else {
                (el_store_b, &mut lpair.nors_b, &mut lpair.nors_gh_b)
            };
            for (i, &v) in el_store.verts.iter().enumerate() {
                nors[i] = bm_vert_calc_surface_tangent(bm, v);
                nors_gh.insert(v, i);
            }
        }
        /* cleanup verts share */
        bmo_edgeloop_vert_tag(bm, el_store_a, VERT_SHARED, false);
        bmo_edgeloop_vert_tag(bm, el_store_b, VERT_SHARED, false);
        /* cleanup faces share */
        bm_faces_share_tag_clear(bm, &e_arr_a);
        bm_faces_share_tag_clear(bm, &e_arr_b);
    }
    lpair
}

/// `BKE_curve_forward_diff_bezier` (3d convenience: eval per-dim into `resolu`
/// points, it = resolu - 1 segments).
fn forward_diff_bezier_v3(q: [Vec3; 4], resolu: usize) -> Vec<Vec3> {
    let it = resolu - 1;
    let mut out = vec![Vec3::ZERO; resolu];
    for d in 0..3 {
        let (q0, q1, q2, q3) = (q[0][d], q[1][d], q[2][d], q[3][d]);
        let mut f = it as f32;
        let rt0 = q0;
        let rt1 = 3.0 * (q1 - q0) / f;
        f *= f;
        let rt2 = 3.0 * (q0 - 2.0 * q1 + q2) / f;
        f *= it as f32;
        let rt3 = (q3 - q0 + 3.0 * (q1 - q2)) / f;
        let (mut q0, mut q1, q2_, q3_) = (rt0, rt1 + rt2 + rt3, 2.0 * rt2 + 6.0 * rt3, 6.0 * rt3);
        let mut q2 = q2_;
        let q3 = q3_;
        for a in 0..=it {
            out[a][d] = q0;
            q0 += q1;
            q1 += q2;
            q2 += q3;
        }
    }
    out
}

/// `bm_edgering_pair_interpolate`.
fn bm_edgering_pair_interpolate(
    bm: &mut BMesh,
    lpair: &LoopPairStore,
    el_store_a: &mut BMEdgeLoopStore,
    el_store_b: &mut BMEdgeLoopStore,
    eloops_ring: &mut Vec<BMEdgeLoopStore>,
    interp_mode: i32,
    cuts: i32,
    smooth: f32,
    falloff_cache: Option<&Vec<f32>>,
) {
    let resolu = (cuts + 2) as usize;
    let mut coord_array_main: Option<Vec<Vec3>> = None;

    bm_edgeloop_calc_center(bm, el_store_a);
    bm_edgeloop_calc_center(bm, el_store_b);
    let mut is_a_no_valid = bm_edgeloop_calc_normal(bm, el_store_a);
    let mut is_b_no_valid = bm_edgeloop_calc_normal(bm, el_store_b);
    let el_store_a_co = bm_edgeloop_center_get(el_store_a);
    let el_store_b_co = bm_edgeloop_center_get(el_store_b);
    /* correct normals need to be flipped to face each other
     * we know both normals point in the same direction so one will need flipping */
    let el_dir = (el_store_a_co - el_store_b_co).normalized();
    if !is_a_no_valid {
        is_a_no_valid = bm_edgeloop_calc_normal_aligned(bm, el_store_a, el_dir);
    }
    if !is_b_no_valid {
        is_b_no_valid = bm_edgeloop_calc_normal_aligned(bm, el_store_b, el_dir);
    }
    let _ = (is_a_no_valid, is_b_no_valid);
    let mut el_store_a_no = bm_edgeloop_normal_get(el_store_a);
    let mut el_store_b_no = bm_edgeloop_normal_get(el_store_b);
    if el_store_a_no.dot(el_dir) > 0.0 {
        el_store_a_no = -el_store_a_no;
    }
    if el_store_b_no.dot(el_dir) < 0.0 {
        el_store_b_no = -el_store_b_no;
    }
    /* Calculate the center spline, multiple. */
    if interp_mode == SUBD_RING_INTERP_PATH || falloff_cache.is_some() {
        let handle_len =
            bezier_handle_calc_length_v3(el_store_a_co, el_store_a_no, el_store_b_co, el_store_b_no)
                * smooth;
        let handle_a = el_store_a_co + el_store_a_no * handle_len;
        let handle_b = el_store_b_co + el_store_b_no * handle_len;
        coord_array_main = Some(forward_diff_bezier_v3(
            [el_store_a_co, handle_a, handle_b, el_store_b_co],
            resolu,
        ));
    }

    match interp_mode {
        SUBD_RING_INTERP_LINEAR => {
            if let Some(fc) = falloff_cache {
                let mut coord_array = vec![Vec3::ZERO; resolu];
                for (i, co) in coord_array.iter_mut().enumerate() {
                    *co = el_store_a_co.lerp(el_store_b_co, i as f32 / (resolu - 1) as f32);
                }
                for el_store_ring in eloops_ring.iter() {
                    for (i, &v) in el_store_ring.verts.iter().enumerate() {
                        if i > 0 && i < resolu - 1 {
                            /* shape */
                            bm.vpool[v].co =
                                coord_array[i].lerp(bm.vert(v).co, fc[i]);
                        }
                    }
                }
            }
        }
        SUBD_RING_INTERP_PATH => {
            let cam = coord_array_main.as_ref().unwrap();
            let mut direction_array = vec![Vec3::ZERO; resolu];
            let mut quat_array =
                vec![blenders_math::quaternion::Quaternion::IDENTITY; resolu];
            let mut tri_array = vec![[Vec3::ZERO; 3]; resolu];
            /* very similar to make_bevel_list_3D_minimum_twist */
            direction_array[0] = el_store_a_no;
            direction_array[resolu - 1] = -el_store_b_no;
            for i in 1..resolu - 1 {
                direction_array[i] =
                    blenders_math::vector::bisect_v3(cam[i - 1], cam[i], cam[i + 1]);
            }
            quat_array[0] =
                blenders_math::quaternion::vec_to_quat(direction_array[0], 5, 1).normalized();
            for i in 1..resolu {
                let angle = blenders_math::geom::angle_normalized_v3v3(
                    direction_array[i - 1],
                    direction_array[i],
                );
                if angle > 0.0 {
                    let cross_tmp = direction_array[i - 1].cross(direction_array[i]);
                    let q = blenders_math::quaternion::axis_angle_to_quat(cross_tmp, angle);
                    quat_array[i] = (q * quat_array[i - 1]).normalized();
                } else {
                    quat_array[i] = quat_array[i - 1];
                }
            }
            /* init base tri */
            for i in 0..resolu {
                let shape_size = falloff_cache.map(|fc| fc[i]).unwrap_or(1.0);
                for j in 0..3 {
                    let mut p = Vec3::ZERO;
                    if j == 1 {
                        p.x = shape_size;
                    } else if j == 2 {
                        p.y = shape_size;
                    }
                    tri_array[i][j] = quat_array[i] * p + cam[i];
                }
            }
            let tri_sta = tri_array[0];
            let tri_end = tri_array[resolu - 1];
            for el_store_ring in eloops_ring.iter() {
                let lb = &el_store_ring.verts;
                let v_a = lb[0];
                let v_b = lb[lb.len() - 1];
                /* skip first and last */
                for i in 1..lb.len() - 1 {
                    let co_a = blenders_math::geom::transform_point_by_tri(
                        bm.vert(v_a).co,
                        &tri_array[i],
                        &tri_sta,
                    );
                    let co_b = blenders_math::geom::transform_point_by_tri(
                        bm.vert(v_b).co,
                        &tri_array[i],
                        &tri_end,
                    );
                    let v = lb[i];
                    bm.vpool[v].co = co_a.lerp(co_b, i as f32 / (resolu - 1) as f32);
                }
            }
        }
        SUBD_RING_INTERP_SURF => {
            /* calculate a bezier handle per edge ring */
            for el_store_ring in eloops_ring.iter() {
                let lb = &el_store_ring.verts;
                let v_a = lb[0];
                let v_b = lb[lb.len() - 1];
                let co_a = bm.vert(v_a).co;
                let co_b = bm.vert(v_b).co;
                /* don't calculate normals here else we get into feedback loop
                 * when subdividing 2+ connected edge rings */
                let no_a = lpair.nors_a[lpair.nors_gh_a[&v_a]];
                let no_b = lpair.nors_b[lpair.nors_gh_b[&v_b]];
                let handle_len =
                    bezier_handle_calc_length_v3(co_a, no_a, co_b, no_b) * smooth;
                let handle_a = co_a + no_a * handle_len;
                let handle_b = co_b + no_b * handle_len;
                let coord_array =
                    forward_diff_bezier_v3([co_a, handle_a, handle_b, co_b], resolu);
                /* skip first and last */
                for i in 1..lb.len() - 1 {
                    if i < resolu - 1 {
                        let v = lb[i];
                        bm.vpool[v].co = coord_array[i];
                        /* shape */
                        if let (Some(fc), Some(cam)) = (falloff_cache, &coord_array_main) {
                            bm.vpool[v].co = cam[i].lerp(bm.vert(v).co, fc[i]);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

/// `bm_face_slice` — cut up an ngon into many slices.
fn bm_face_slice(bm: &mut BMesh, l: usize, cuts: i32) {
    /* TODO: interpolate edge data. */
    let mut l_new = l;
    for _ in 0..cuts {
        /* no chance of double */
        let f = bm.loo(l_new).f;
        let (l_prev, l_next2) = (bm.loo(l_new).prev, bm.loo(bm.loo(l_new).next).next);
        let mut l_out = BM_NIL;
        crate::mods::bm_face_split(bm, f, l_prev, l_next2, Some(&mut l_out), false);
        l_new = l_out;
        if l_new == BM_NIL {
            /* l_new->prev and l_new->next->next were adjacent. */
            break;
        }
        let (fa, fb) = (bm.loo(l_new).f, bm.loo(bm.loo(l_new).radial_next).f);
        if bm.face(fa).len < bm.face(fb).len {
            l_new = bm.loo(l_new).radial_next;
        }
        let (fa, fb) = (bm.loo(l_new).f, bm.loo(bm.loo(l_new).radial_next).f);
        bm.fpool[fa].head.api_flag |= FACE_OUT;
        bm.fpool[fb].head.api_flag |= FACE_OUT;
    }
}

/// `bm_edgering_pair_order_is_flipped`.
fn bm_edgering_pair_order_is_flipped(
    bm: &BMesh,
    el_store_a: &BMEdgeLoopStore,
    el_store_b: &BMEdgeLoopStore,
) -> bool {
    let lb_a: Vec<usize> = el_store_a.verts.iter().copied().collect();
    let lb_b: Vec<usize> = el_store_b.verts.iter().copied().collect();
    debug_assert_ne!(crate::query::bm_edge_exists(bm, lb_a[0], lb_b[0]), BM_NIL);
    /* step around any fan-faces on both sides */
    let mut i_a = 0usize;
    loop {
        i_a += 1;
        if i_a >= lb_a.len() {
            break;
        }
        let e1 = crate::query::bm_edge_exists(bm, lb_a[i_a], lb_b[0]);
        let e2 = crate::query::bm_edge_exists(bm, lb_a[i_a], lb_b[1]);
        if e1 == BM_NIL && e2 == BM_NIL {
            break;
        }
    }
    let mut i_b = 0usize;
    loop {
        i_b += 1;
        if i_b >= lb_b.len() {
            break;
        }
        let e1 = crate::query::bm_edge_exists(bm, lb_b[i_b], lb_a[0]);
        let e2 = crate::query::bm_edge_exists(bm, lb_b[i_b], lb_a[1]);
        if e1 == BM_NIL && e2 == BM_NIL {
            break;
        }
    }
    i_a -= 1;
    i_b -= 1;
    !(crate::query::bm_edge_exists(bm, lb_a[i_a], lb_b[i_b]) != BM_NIL
        || crate::query::bm_edge_exists(bm, lb_a[1], lb_b[i_b]) != BM_NIL
        || crate::query::bm_edge_exists(bm, lb_b[1], lb_a[i_a]) != BM_NIL)
}

/// `bm_edgering_pair_order` — align loops and ensure winding matches.
fn bm_edgering_pair_order(
    bm: &mut BMesh,
    el_store_a: &BMEdgeLoopStore,
    el_store_b: &mut BMEdgeLoopStore,
) {
    bm_edgeloop_vert_tag(bm, el_store_a, false);
    bm_edgeloop_vert_tag(bm, el_store_b, true);
    /* before going much further, get ourselves in order
     * - align loops (not strictly necessary but handy)
     * - ensure winding is set for both loops */
    if bm_edgeloop_is_closed(el_store_a) && bm_edgeloop_is_closed(el_store_b) {
        let v_first_a = el_store_a.verts[0];
        let mut v_other = BM_NIL;
        for e in EdgesOfVert::new(bm, v_first_a) {
            if bm.edge(e).head.api_flag & EDGE_RING != 0 {
                let vo =
                    if bm.edge(e).v1 == v_first_a { bm.edge(e).v2 } else { bm.edge(e).v1 };
                if bm.vert(vo).head.hflag & BM_ELEM_TAG != 0 {
                    v_other = vo;
                    break;
                }
            }
        }
        debug_assert_ne!(v_other, BM_NIL);
        let node_i = el_store_b.verts.iter().position(|&v| v == v_other).unwrap();
        el_store_b.verts.rotate_left(node_i);
        /* now check we are winding the same way */
        if bm_edgering_pair_order_is_flipped(bm, el_store_a, el_store_b) {
            bm_edgeloop_flip(el_store_b);
            /* re-ensure the first node */
            let node_i = el_store_b.verts.iter().position(|&v| v == v_other).unwrap();
            el_store_b.verts.rotate_left(node_i);
        }
        debug_assert!(!bm_edgering_pair_order_is_flipped(bm, el_store_a, el_store_b));
    } else {
        /* If we don't share and edge - flip. */
        let e = crate::query::bm_edge_exists(bm, el_store_a.verts[0], el_store_b.verts[0]);
        if e == BM_NIL || bm.edge(e).head.api_flag & EDGE_RING == 0 {
            bm_edgeloop_flip(el_store_b);
        }
    }
    /* for cases with multiple loops */
    bm_edgeloop_vert_tag(bm, el_store_b, false);
}

/// `bm_edgering_pair_subdiv` — subdivide connecting edges (loops not aligned).
fn bm_edgering_pair_subdiv(
    bm: &mut BMesh,
    el_store_a: &BMEdgeLoopStore,
    el_store_b: &BMEdgeLoopStore,
    eloops_ring: &mut Vec<BMEdgeLoopStore>,
    cuts: i32,
) {
    let mut edges_ring_arr: Vec<usize> = Vec::new();
    let mut faces_ring_arr: Vec<usize> = Vec::new();
    bm_edgeloop_vert_tag(bm, el_store_a, false);
    bm_edgeloop_vert_tag(bm, el_store_b, true);
    for &v in &el_store_a.verts {
        for e in EdgesOfVert::new(bm, v).collect::<Vec<_>>() {
            if bm.edge(e).head.api_flag & EDGE_IN_STACK == 0 {
                let v_other =
                    if bm.edge(e).v1 == v { bm.edge(e).v2 } else { bm.edge(e).v1 };
                if bm.vert(v_other).head.hflag & BM_ELEM_TAG != 0 {
                    bm.epool[e].head.api_flag |= EDGE_IN_STACK;
                    edges_ring_arr.push(e);
                    /* add faces to the stack */
                    for f in FacesOfEdge::new(bm, e).collect::<Vec<_>>() {
                        if bm.face(f).head.api_flag & FACE_OUT != 0
                            && bm.face(f).head.api_flag & FACE_IN_STACK == 0
                        {
                            bm.fpool[f].head.api_flag |= FACE_IN_STACK;
                            faces_ring_arr.push(f);
                        }
                    }
                }
            }
        }
    }
    while let Some(e) = edges_ring_arr.pop() {
        /* found opposite edge */
        bm.epool[e].head.api_flag &= !EDGE_IN_STACK;
        /* unrelated to subdiv, but if we _don't_ clear flag, multiple rings fail */
        bm.epool[e].head.api_flag &= !EDGE_RING;
        let v_other = if bm.vert(bm.edge(e).v1).head.hflag & BM_ELEM_TAG != 0 {
            bm.edge(e).v1
        } else {
            bm.edge(e).v2
        };
        bm_edge_subdiv_as_loop(bm, eloops_ring, e, v_other, cuts);
    }
    while let Some(f) = faces_ring_arr.pop() {
        bm.fpool[f].head.api_flag &= !FACE_IN_STACK;
        /* Check each edge of the face */
        let l_first = bm.face(f).l_first;
        let mut l = l_first;
        loop {
            if bm.edge(bm.loo(l).e).head.api_flag & EDGE_RIM != 0 {
                bm_face_slice(bm, l, cuts);
                break;
            }
            l = bm.loo(l).next;
            if l == l_first {
                break;
            }
        }
    }
    /* Clear tags so subdiv verts don't get tagged too. */
    for el_store_ring in eloops_ring.iter() {
        bm_edgeloop_vert_tag(bm, el_store_ring, false);
    }
    /* cleanup after */
    bm_edgeloop_vert_tag(bm, el_store_b, false);
}

/// `bm_edgering_pair_ringsubd`.
fn bm_edgering_pair_ringsubd(
    bm: &mut BMesh,
    lpair: &LoopPairStore,
    el_store_a: &mut BMEdgeLoopStore,
    el_store_b: &mut BMEdgeLoopStore,
    interp_mode: i32,
    cuts: i32,
    smooth: f32,
    falloff_cache: Option<&Vec<f32>>,
) {
    let mut eloops_ring: Vec<BMEdgeLoopStore> = Vec::new();
    bm_edgering_pair_order(bm, el_store_a, el_store_b);
    bm_edgering_pair_subdiv(bm, el_store_a, el_store_b, &mut eloops_ring, cuts);
    bm_edgering_pair_interpolate(
        bm,
        lpair,
        el_store_a,
        el_store_b,
        &mut eloops_ring,
        interp_mode,
        cuts,
        smooth,
        falloff_cache,
    );
}

/// `bmo_subdivide_edgering_exec`. NOTE: keep this operator fast, its used in a modifier.
pub fn bmo_subdivide_edgering_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let cuts = bmo_slot_int_get(op, false, "cuts");
    let interp_mode = bmo_slot_int_get(op, false, "interp_mode");
    let smooth = bmo_slot_float_get(op, false, "smooth");
    let resolu = (cuts + 2) as usize;
    /* optional 'shape' */
    let profile_shape = bmo_slot_int_get(op, false, "profile_shape");
    let profile_shape_factor = bmo_slot_float_get(op, false, "profile_shape_factor");
    let mut changed = false;

    bmo_slot_buffer_flag_enable(bm, op, false, "edges", EDGE_RING);
    crate::marking::bm_mesh_elem_hflag_disable_all(bm, BM_VERT, BM_ELEM_TAG);

    /* flag outer edges (loops defined as edges on the bounds of the edge ring) */
    let edges_in: Vec<usize> = op
        .slots_in
        .iter()
        .find(|s| s.name == "edges")
        .map(|s| {
            s.buf
                .iter()
                .filter(|(ht, _)| *ht == BM_EDGE)
                .map(|(_, i)| *i)
                .collect()
        })
        .unwrap_or_default();
    for &e in &edges_in {
        for f in FacesOfEdge::new(bm, e).collect::<Vec<_>>() {
            /* could support ngons, other areas would need updating too, see #48926. */
            if bm.face(f).len <= 4 && bm.face(f).head.api_flag & FACE_OUT == 0 {
                let mut ok = false;
                /* check at least 2 edges in the face are rings */
                let l_first = bm.face(f).l_first;
                let mut l = l_first;
                loop {
                    if bm.edge(bm.loo(l).e).head.api_flag & EDGE_RING != 0
                        && bm.loo(l).e != e
                    {
                        ok = true;
                        break;
                    }
                    l = bm.loo(l).next;
                    if l == l_first {
                        break;
                    }
                }
                if ok {
                    bm.fpool[f].head.api_flag |= FACE_OUT;
                    let l_first = bm.face(f).l_first;
                    let mut l = l_first;
                    loop {
                        let le = bm.loo(l).e;
                        if bm.edge(le).head.api_flag & EDGE_RING == 0 {
                            bm.epool[le].head.api_flag |= EDGE_RIM;
                        }
                        l = bm.loo(l).next;
                        if l == l_first {
                            break;
                        }
                    }
                }
            }
        }
    }

    /* Cache falloff for each step (symmetrical) */
    let falloff_cache: Option<Vec<f32>> = if profile_shape_factor != 0.0 {
        let mut fc = vec![0.0; resolu];
        for i in 0..resolu {
            let mut shape_size = 1.0f32;
            let mut fac = i as f32 / (resolu - 1) as f32;
            fac = (1.0 - 2.0 * (0.5 - fac).abs()).abs();
            fac = crate::query::bmesh_subd_falloff_calc(profile_shape, fac);
            shape_size += fac * profile_shape_factor;
            fc[i] = shape_size;
        }
        Some(fc)
    } else {
        None
    };

    /* Execute subdivision on all ring pairs */
    let mut eloops_rim = bm_mesh_edgeloops_find(
        bm,
        &|bm: &BMesh, e| bm.edge(e).head.api_flag & EDGE_RIM != 0,
        None,
    );
    let count = eloops_rim.len();
    if count < 2 {
        /* BMO_error_raise: "No edge rings found" (error system pending) */
    } else if count == 2 {
        /* avoid 'bm_edgering_pair_calc' since there's only one pair */
        let (a, b) = eloops_rim.split_at_mut(1);
        let el_store_a = &a[0];
        let el_store_b = &mut b[0];
        if bm_edgeloop_check_overlap_all(bm, el_store_a, el_store_b) {
            let lpair =
                bm_edgering_pair_store_create(bm, el_store_a, el_store_b, interp_mode);
            bm_edgering_pair_ringsubd(
                bm,
                &lpair,
                &mut a[0],
                el_store_b,
                interp_mode,
                cuts,
                smooth,
                falloff_cache.as_ref(),
            );
            changed = true;
        } else {
            /* BMO_error_raise: "Edge-ring pair isn't connected" */
        }
    } else {
        let eloop_pairs = bm_edgering_pair_calc(bm, &eloops_rim);
        if eloop_pairs.is_empty() {
            /* BMO_error_raise: "Edge-rings are not connected" */
        } else {
            /* first cache pairs */
            let mut lpair_arr: Vec<Option<LoopPairStore>> = Vec::with_capacity(eloop_pairs.len());
            for &(ia, ib) in &eloop_pairs {
                let (a, b) = if ia < ib {
                    let (lo, hi) = eloops_rim.split_at_mut(ib);
                    (&lo[ia], &hi[0])
                } else {
                    let (lo, hi) = eloops_rim.split_at_mut(ia);
                    (&hi[0], &lo[ib])
                };
                lpair_arr.push(if bm_edgeloop_check_overlap_all(bm, a, b) {
                    Some(bm_edgering_pair_store_create(bm, a, b, interp_mode))
                } else {
                    None
                });
            }
            for (pi, &(ia, ib)) in eloop_pairs.iter().enumerate() {
                if let Some(lpair) = &lpair_arr[pi] {
                    let (a, b) = if ia < ib {
                        let (lo, hi) = eloops_rim.split_at_mut(ib);
                        (&mut lo[ia], &mut hi[0])
                    } else {
                        let (lo, hi) = eloops_rim.split_at_mut(ia);
                        (&mut hi[0], &mut lo[ib])
                    };
                    bm_edgering_pair_ringsubd(
                        bm, lpair, a, b, interp_mode, cuts, smooth, falloff_cache.as_ref(),
                    );
                    changed = true;
                }
            }
        }
    }

    /* flag output */
    if changed {
        bmo_slot_buffer_from_enabled_flag(bm, op, true, "faces.out", BM_FACE, FACE_OUT);
    }
}
