//! Port of `bmo_join_triangles.cc` — greedy triangle-pair joins driven by a heap.
//! elem_index → side-maps; CustomData delimiters (uvs/vcols) PORT_PENDING.

use crate::iterators::*;
use crate::operators::*;
use crate::{BMesh, BM_FACE, BM_NIL, BM_VERT};
use blenders_bli::heap::{Heap, HeapNode};
use blenders_math::geom::{angle_normalized_v3v3, area_tri, is_quad_flip_v3, normal_quad, normal_tri};
use blenders_math::quaternion::axis_angle_normalized_to_mat3;
use blenders_math::vector::{angle_signed_on_axis, Vec3};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

const FACE_OUT: u8 = 1;
const FACE_INPUT: u8 = 4;

const MAXIMUM_IMPROVEMENT: f32 = 0.99;

struct JoinEdgesState {
    topo_influnce: f32,
    use_topo_influence: bool,
    select_tris_only: bool,
    /// HeapNode per mesh-edge index (side-table; C uses elem_index).
    edge_queue_nodes: Vec<Option<HeapNode>>,
    e_index: Vec<i32>,
}

/// `quad_calc_error` — normal angle + edge angle + area balance, [0, 2π].
fn quad_calc_error(v1: Vec3, v2: Vec3, v3: Vec3, v4: Vec3) -> f32 {
    let mut error = 0.0;
    {
        let (n1, _) = normal_tri(v1, v2, v3);
        let (n2, _) = normal_tri(v1, v3, v4);
        let angle_a = if n1 == n2 {
            0.0
        } else {
            angle_normalized_v3v3(n1, n2)
        };
        let (n1b, _) = normal_tri(v2, v3, v4);
        let (n2b, _) = normal_tri(v4, v1, v2);
        let angle_b = if n1b == n2b {
            0.0
        } else {
            angle_normalized_v3v3(n1b, n2b)
        };
        error += (angle_a + angle_b) / (PI * 2.0);
    }
    {
        let e0 = (v1 - v2).normalized();
        let e1 = (v2 - v3).normalized();
        let e2 = (v3 - v4).normalized();
        let e3 = (v4 - v1).normalized();
        let diff = ((angle_normalized_v3v3(e0, e1) - FRAC_PI_2).abs()
            + (angle_normalized_v3v3(e1, e2) - FRAC_PI_2).abs()
            + (angle_normalized_v3v3(e2, e3) - FRAC_PI_2).abs()
            + (angle_normalized_v3v3(e3, e0) - FRAC_PI_2).abs())
            / (PI * 2.0);
        error += diff;
    }
    {
        let area_a = area_tri(v1, v2, v3) + area_tri(v1, v3, v4);
        let area_b = area_tri(v2, v3, v4) + area_tri(v4, v1, v2);
        let area_min = area_a.min(area_b);
        let area_max = area_a.max(area_b);
        let diff = if area_max != 0.0 {
            1.0 - area_min / area_max
        } else {
            1.0
        };
        error += diff;
    }
    error
}

/// `bm_edge_to_quad_verts` — 4 verts of the quad formed by e's two tris.
fn bm_edge_to_quad_verts(bm: &BMesh, e: usize) -> [usize; 4] {
    let l = bm.edge(e).l;
    let l_rad = bm.loo(l).radial_next;
    [
        bm.loo(l).v,
        bm.loo(bm.loo(l_rad).prev).v,
        bm.loo(bm.loo(l).next).v,
        bm.loo(bm.loo(l).prev).v,
    ]
}

struct DelimitData {
    do_seam: bool,
    do_sharp: bool,
    do_mat: bool,
    do_angle_face: bool,
    do_angle_shape: bool,
    angle_face_cos: f32,
    angle_shape: f32,
    /* PORT_PENDING: cdata[] loop-CustomData delimiters. */
}

fn bm_edge_delmimit_data_from_op(op: &BMOperator) -> DelimitData {
    let mut d = DelimitData {
        do_seam: bmo_slot_bool_get(op, false, "cmp_seam"),
        do_sharp: bmo_slot_bool_get(op, false, "cmp_sharp"),
        do_mat: bmo_slot_bool_get(op, false, "cmp_materials"),
        do_angle_face: false,
        do_angle_shape: false,
        angle_face_cos: 0.0,
        angle_shape: 0.0,
    };
    let angle_face = bmo_slot_float_get(op, false, "angle_face_threshold");
    if angle_face < (180.0f32).to_radians() {
        d.do_angle_face = true;
        d.angle_face_cos = angle_face.cos();
    }
    let angle_shape = bmo_slot_float_get(op, false, "angle_shape_threshold");
    if angle_shape < (180.0f32).to_radians() {
        d.do_angle_shape = true;
        d.angle_shape = angle_shape;
    }
    /* PORT_PENDING: cmp_uvs/cmp_vcols CD delimiters. */
    d
}

fn bm_edge_is_delimit(bm: &BMesh, e: usize, d: &DelimitData) -> bool {
    let l = bm.edge(e).l;
    let f_a = bm.loo(l).f;
    let f_b = bm.loo(bm.loo(l).radial_next).f;
    if d.do_seam && bm.edge(e).head.hflag & crate::BM_ELEM_SEAM != 0 {
        return true;
    }
    if d.do_sharp && bm.edge(e).head.hflag & crate::BM_ELEM_SMOOTH == 0 {
        return true;
    }
    if d.do_mat && bm.face(f_a).mat_nr != bm.face(f_b).mat_nr {
        return true;
    }
    if d.do_angle_face && bm.face(f_a).no.dot(bm.face(f_b).no) < d.angle_face_cos {
        return true;
    }
    if d.do_angle_shape {
        let verts = bm_edge_to_quad_verts(bm, e);
        if is_quad_flip_v3(
            bm.vert(verts[0]).co,
            bm.vert(verts[1]).co,
            bm.vert(verts[2]).co,
            bm.vert(verts[3]).co,
        ) != 0
        {
            return true;
        }
        /* quad shape angle check */
        if quad_calc_error(
            bm.vert(verts[0]).co,
            bm.vert(verts[1]).co,
            bm.vert(verts[2]).co,
            bm.vert(verts[3]).co,
        ) > d.angle_shape
        {
            return true;
        }
    }
    false
}

#[derive(Clone, Copy)]
struct JoinEdgesNeighborItem {
    e: usize,
    l: usize,
}

fn add_without_duplicates(items: &mut Vec<JoinEdgesNeighborItem>, e: usize, l: usize) {
    for it in items.iter() {
        if it.e == e {
            return;
        }
    }
    if items.len() < 8 {
        items.push(JoinEdgesNeighborItem { e, l });
    }
}

fn add_neighbors(bm: &BMesh, items: &mut Vec<JoinEdgesNeighborItem>, l_in_quad: usize) {
    let e = bm.loo(l_in_quad).e;
    if !crate::query::bm_edge_is_manifold(bm, e) {
        return;
    }
    let l_in_neighbor = bm.loo(l_in_quad).radial_next;
    if bm.face(bm.loo(l_in_neighbor).f).len != 3 {
        return;
    }
    for &l_other in &[bm.loo(l_in_neighbor).prev, bm.loo(l_in_neighbor).next] {
        let eo = bm.loo(l_other).e;
        if crate::query::bm_edge_is_manifold(bm, eo)
            && bm.face(bm.loo(bm.loo(l_other).radial_next).f).len == 3
        {
            add_without_duplicates(items, eo, l_in_neighbor);
        }
    }
}

/// `rotate_to_plane` — rotate quad verts (except the shared edge's) around the
/// shared edge axis into `plane_normal`'s plane.
fn rotate_to_plane(
    bm: &BMesh,
    quad_verts: [usize; 4],
    l_shared: usize,
    plane_normal: Vec3,
    r_quad_coordinates: &mut [Vec3; 4],
) {
    let v_a = bm.loo(l_shared).v;
    let v_b = bm.loo(bm.loo(l_shared).next).v;
    let rotation_axis = (bm.vert(v_a).co - bm.vert(v_b).co).normalized();
    let (quad_normal, _) = normal_quad(
        bm.vert(quad_verts[0]).co,
        bm.vert(quad_verts[1]).co,
        bm.vert(quad_verts[2]).co,
        bm.vert(quad_verts[3]).co,
    );
    let angle = angle_signed_on_axis(plane_normal, quad_normal, rotation_axis);
    let rot = axis_angle_normalized_to_mat3(rotation_axis, angle);
    for i in 0..4 {
        if quad_verts[i] == v_a || quad_verts[i] == v_b {
            r_quad_coordinates[i] = bm.vert(quad_verts[i]).co;
        } else {
            let local = bm.vert(quad_verts[i]).co - bm.vert(v_a).co;
            r_quad_coordinates[i] = rot * local + bm.vert(v_a).co;
        }
    }
}

/// `compute_alignment` — how well a candidate join edge aligns to a reference quad.
fn compute_alignment(
    bm: &BMesh,
    quad_a_vecs: &[Vec3; 4],
    quad_b_verts: [usize; 4],
    l_shared: usize,
    plane_normal: Vec3,
) -> f32 {
    let mut quad_b_coordinates = [Vec3::ZERO; 4];
    rotate_to_plane(bm, quad_b_verts, l_shared, plane_normal, &mut quad_b_coordinates);
    let mut quad_b_vecs = [Vec3::ZERO; 4];
    quad_b_vecs[0] = (quad_b_coordinates[0] - quad_b_coordinates[1]).normalized();
    quad_b_vecs[1] = (quad_b_coordinates[1] - quad_b_coordinates[2]).normalized();
    quad_b_vecs[2] = (quad_b_coordinates[2] - quad_b_coordinates[3]).normalized();
    quad_b_vecs[3] = (quad_b_coordinates[3] - quad_b_coordinates[0]).normalized();
    let mut error = [0.0f32; 4];
    for i in 0..4 {
        let angle_a = angle_normalized_v3v3(quad_a_vecs[i], quad_b_vecs[i]).abs();
        let angle_b = angle_normalized_v3v3(quad_a_vecs[i], quad_b_vecs[(i + 1) % 4]).abs();
        error[0] += angle_a;
        error[1] += angle_b;
        error[2] += PI - angle_a;
        error[3] += PI - angle_b;
    }
    let best_error = error.iter().cloned().fold(f32::MAX, f32::min) / 4.0;
    let alignment = 1.0 - best_error / FRAC_PI_4;
    alignment.max(0.0)
}

fn reprioritize_join(
    bm: &BMesh,
    s: &mut JoinEdgesState,
    edge_queue: &mut Heap<usize>,
    e_merge: usize,
    l_shared: usize,
    neighbor_quad_vecs: &[Vec3; 4],
    neighbor_quad_error: f32,
    neighbor_quad_normal: Vec3,
) {
    let ei = s.e_index[e_merge];
    if ei < 0 {
        return;
    }
    let node = match s.edge_queue_nodes[ei as usize] {
        Some(n) => n,
        None => return,
    };
    let join_error_curr = edge_queue.node_value(node);
    if neighbor_quad_error > join_error_curr {
        return;
    }
    let quad_verts_merge = bm_edge_to_quad_verts(bm, e_merge);
    let alignment = compute_alignment(
        bm, neighbor_quad_vecs, quad_verts_merge, l_shared, neighbor_quad_normal);
    let improvement = neighbor_quad_error - join_error_curr;
    let mut multiplier = s.topo_influnce * alignment;
    multiplier = multiplier.min(MAXIMUM_IMPROVEMENT);
    let join_error_next = join_error_curr + improvement * multiplier;
    edge_queue.node_value_update(node, join_error_next);
}

fn reprioritize_face_neighbors(
    bm: &BMesh,
    s: &mut JoinEdgesState,
    edge_queue: &mut Heap<usize>,
    f: usize,
    f_error: f32,
) {
    debug_assert_eq!(bm.face(f).len, 4);
    let mut items: Vec<JoinEdgesNeighborItem> = Vec::new();
    let l_quad = crate::polygon::bm_face_as_array_loop_quad(bm, f);
    for &l in &l_quad {
        add_neighbors(bm, &mut items, l);
    }
    if items.is_empty() {
        return;
    }
    let mut quad_vecs = [Vec3::ZERO; 4];
    let mut i = 3usize;
    for i_next in 0..4 {
        quad_vecs[i] =
            (bm.vert(bm.loo(l_quad[i]).v).co - bm.vert(bm.loo(l_quad[i_next]).v).co).normalized();
        i = i_next;
    }
    let f_no = bm.face(f).no;
    for item in &items {
        reprioritize_join(bm, s, edge_queue, item.e, item.l, &quad_vecs, f_error, f_no);
    }
}

/// `bm_faces_join_pair_by_edge`.
fn bm_faces_join_pair_by_edge(bm: &mut BMesh, e: usize) -> usize {
    let l_a = bm.edge(e).l;
    let l_b = bm.loo(l_a).radial_next;
    if bm.face(bm.loo(l_a).f).len != 3 || bm.face(bm.loo(l_b).f).len != 3 {
        return BM_NIL;
    }
    let v_quad = [
        bm.loo(l_a).v,
        bm.loo(bm.loo(l_b).prev).v,
        bm.loo(bm.loo(l_a).next).v,
        bm.loo(bm.loo(l_a).prev).v,
    ];
    if crate::query::bm_face_exists(bm, &v_quad) != BM_NIL {
        return BM_NIL;
    }
    crate::mods::bm_faces_join_pair(bm, l_a, l_b, true, None)
}

/// `bmo_join_triangles_exec`.
pub fn bmo_join_triangles_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let delimit_data = bm_edge_delmimit_data_from_op(op);
    let mut s = JoinEdgesState {
        topo_influnce: bmo_slot_float_get(op, false, "topology_influence"),
        use_topo_influence: false,
        select_tris_only: bmo_slot_bool_get(op, false, "deselect_joined"),
        edge_queue_nodes: Vec::new(),
        e_index: vec![-1; bm.epool.len()],
    };
    s.use_topo_influence = s.topo_influnce != 0.0;
    let mut edge_queue: Heap<usize> = Heap::new();
    if s.use_topo_influence {
        s.edge_queue_nodes = vec![None; bm.epool.len()];
    }

    let faces_in: Vec<usize> = op
        .slots_in
        .iter()
        .find(|sl| sl.name == "faces")
        .map(|sl| {
            sl.buf
                .iter()
                .filter(|(ht, _)| *ht == BM_FACE)
                .map(|(_, i)| *i)
                .collect()
        })
        .unwrap_or_default();
    for &f in &faces_in {
        if bm.face(f).len == 3 {
            bm.fpool[f].head.api_flag |= FACE_INPUT;
            if s.select_tris_only {
                bm.fpool[f].head.api_flag |= FACE_OUT;
            }
        }
    }

    let mut i = 0i32;
    for e in EdgesOfMesh::new(bm).collect::<Vec<_>>() {
        s.e_index[e] = i;
        if let Some((f_a, f_b)) = crate::query::bm_edge_face_pair(bm, e) {
            if bm.face(f_a).head.api_flag & FACE_INPUT != 0
                && bm.face(f_b).head.api_flag & FACE_INPUT != 0
                && !bm_edge_is_delimit(bm, e, &delimit_data)
            {
                let e_verts = bm_edge_to_quad_verts(bm, e);
                let merge_error = quad_calc_error(
                    bm.vert(e_verts[0]).co,
                    bm.vert(e_verts[1]).co,
                    bm.vert(e_verts[2]).co,
                    bm.vert(e_verts[3]).co,
                );
                let node = edge_queue.insert(merge_error, e);
                if s.use_topo_influence {
                    s.edge_queue_nodes[i as usize] = Some(node);
                }
            }
        }
        i += 1;
    }

    if s.use_topo_influence && !edge_queue.is_empty() {
        for &f in &faces_in {
            if bm.face(f).len == 4 {
                let f_verts = crate::polygon::bm_face_as_array_vert_quad(bm, f);
                let mut f_error = quad_calc_error(
                    bm.vert(f_verts[0]).co,
                    bm.vert(f_verts[1]).co,
                    bm.vert(f_verts[2]).co,
                    bm.vert(f_verts[3]).co,
                );
                f_error *= 2.0 - s.topo_influnce * MAXIMUM_IMPROVEMENT;
                reprioritize_face_neighbors(bm, &mut s, &mut edge_queue, f, f_error);
            }
        }
    }

    while !edge_queue.is_empty() {
        let f_error = edge_queue.top_value().unwrap_or(0.0);
        let e = match edge_queue.pop_min() {
            Some(e) => e,
            None => break,
        };
        let f_new = bm_faces_join_pair_by_edge(bm, e);
        if f_new != BM_NIL {
            if !s.select_tris_only {
                bm.fpool[f_new].head.api_flag |= FACE_OUT;
            }
            if s.use_topo_influence {
                reprioritize_face_neighbors(bm, &mut s, &mut edge_queue, f_new, f_error);
            }
        }
    }

    bmo_slot_buffer_from_enabled_flag(bm, op, true, "faces.out", BM_FACE, FACE_OUT);
}
