//! Port of `polyfill_-from e->e_next`, omitting boundary edges.
//!
//! 2D, 3D, and half-edge pointers → `u32` indices.

use blenders_math::base::signum_i_ex;
use blenders_math::geom::{axis_dominant_to_m3, cross_tri_v2};
use blenders_math::vector::{Vec2, Vec3};

const UINT_MAXV: u32 = u32::MAX;

fn fv(a: [f32; 2]) -> Vec2 {
    Vec2::new(a[0], a[1])
}

#[derive(Clone, Copy)]
struct HalfEdge {
    v: u32,
    e_next: u32,
    e_radial: u32,
    base_index: u32,
}

fn is_boundary_edge(i_a: u32, i_b: u32, coord_last: u32) -> bool {
    i_a + 1 == i_b || (i_a == 0 && i_b == coord_last)
}

fn cross_tri_v3(v1: [f32; 3], v2: [f32; 3], v3: [f32; 3]) -> [f32; 3] {
    let a = [v2[0] - v1[0], v2[1] - v1[1], v2[2] - v1[2]];
    let b = [v3[0] - v1[0], v3[1] - v1[1], v3[2] - v1[2]];
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn len_v2v2(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// `BLI_polyfill_beautify_quad_rotate_calc_ex` — beauty score for
/// rotating edge v2-v4 of quad v1,v2,v3,v4; returns FLT_MAX when
/// rotation is disallowed, -FLT_MAX when always rotate.
pub fn polyfill_beautify_quad_rotate_calc_ex(
    v1: [f32; 2],
    v2: [f32; 2],
    v3: [f32; 2],
    v4: [f32; 2],
    lock_degenerate: bool,
    r_area: &mut Option<f32>,
) -> f32 {
    let eps_zero_area = 1e-12f32;
    let area_2x_234 = cross_tri_v2(fv(v2), fv(v3), fv(v4));
    let area_2x_241 = cross_tri_v2(fv(v2), fv(v4), fv(v1));
    let area_2x_123 = cross_tri_v2(fv(v1), fv(v2), fv(v3));
    let area_2x_134 = cross_tri_v2(fv(v1), fv(v3), fv(v4));

    if let Some(ra) = r_area.as_mut() {
        *ra = (area_2x_234.abs()
            + area_2x_241.abs()
            + area_2x_123.abs()
            + area_2x_134.abs())
            / 8.0;
    }

    if (area_2x_123 >= 0.0) != (area_2x_134 >= 0.0) {
        return f32::MAX;
    }
    if area_2x_123.abs() <= eps_zero_area || area_2x_134.abs() <= eps_zero_area {
        return f32::MAX;
    }
    if (area_2x_234 >= 0.0) != (area_2x_241 >= 0.0) {
        if lock_degenerate {
            return f32::MAX;
        }
        return -f32::MAX;
    }
    if area_2x_234.abs() <= eps_zero_area || area_2x_241.abs() <= eps_zero_area {
        return -f32::MAX;
    }

    let len_12 = len_v2v2(v1, v2);
    let len_23 = len_v2v2(v2, v3);
    let len_34 = len_v2v2(v3, v4);
    let len_41 = len_v2v2(v4, v1);
    let len_13 = len_v2v2(v1, v3);
    let len_24 = len_v2v2(v2, v4);

    let mut area_a = area_2x_234.abs();
    let mut area_b = area_2x_241.abs();
    let mut prim_a = len_23 + len_34 + len_24;
    let mut prim_b = len_41 + len_12 + len_24;
    let fac_24 = area_a / prim_a + area_b / prim_b;

    area_a = area_2x_123.abs();
    area_b = area_2x_134.abs();
    prim_a = len_12 + len_23 + len_13;
    prim_b = len_34 + len_41 + len_13;
    let fac_13 = area_a / prim_a + area_b / prim_b;

    fac_24 - fac_13
}

/// `BLI_polyfill_edge_calc_rotate_beauty__area` — 3D variant:
/// project to the dominant-axis plane first.
pub fn polyfill_edge_calc_rotate_beauty_area(
    v1: [f32; 3],
    v2: [f32; 3],
    v3: [f32; 3],
    v4: [f32; 3],
    lock_degenerate: bool,
) -> f32 {
    let eps = 1e-5f32;
    let no_a = cross_tri_v3(v2, v3, v4);
    let no_b = cross_tri_v3(v2, v4, v1);
    let no = Vec3::new(
        no_a[0] + no_b[0],
        no_a[1] + no_b[1],
        no_a[2] + no_b[2],
    );
    let no_scale = {
        let l = (no.x * no.x + no.y * no.y + no.z * no.z).sqrt();
        if l == 0.0 {
            return f32::MAX;
        }
        l
    };
    let n = no / no_scale;
    let m = axis_dominant_to_m3(n);
    let project = |v: [f32; 3]| -> [f32; 2] {
        let r = m * Vec3::from(v);
        [r.x, r.y]
    };
    let v1_xy = project(v1);
    let v2_xy = project(v2);
    let v3_xy = project(v3);
    let v4_xy = project(v4);
    if signum_i_ex(
        cross_tri_v2(fv(v2_xy), fv(v3_xy), fv(v4_xy)) / no_scale,
        eps,
    ) + signum_i_ex(
        cross_tri_v2(fv(v2_xy), fv(v4_xy), fv(v1_xy)) / no_scale,
        eps,
    ) == 0
    {
        return f32::MAX;
    }
    polyfill_beautify_quad_rotate_calc_ex(
        v1_xy,
        v2_xy,
        v3_xy,
        v4_xy,
        lock_degenerate,
        &mut None,
    )
}

/// `polyedge_rotate_beauty_calc`.
fn polyedge_rotate_beauty_calc(
    coords: &[[f32; 2]],
    edges: &[HalfEdge],
    e_a: usize,
    r_area: &mut Option<f32>,
) -> f32 {
    let e_b = edges[e_a].e_radial as usize;
    let e_a_other = edges[edges[e_a].e_next as usize].e_next as usize;
    let e_b_other = edges[edges[e_b].e_next as usize].e_next as usize;

    let v1 = coords[edges[e_a_other].v as usize];
    let v2 = coords[edges[e_a].v as usize];
    let v3 = coords[edges[e_b_other].v as usize];
    let v4 = coords[edges[e_b].v as usize];
    polyfill_beautify_quad_rotate_calc_ex(v1, v2, v3, v4, false, r_area)
}

/// `polyedge_beauty_cost_update_single`.
fn polyedge_beauty_cost_update_single(
    coords: &[[f32; 2]],
    edges: &[HalfEdge],
    e: usize,
    eheap: &mut crate::heap::Heap<usize>,
    eheap_table: &mut [Option<crate::heap::HeapNode>],
) {
    let i = edges[e].base_index as usize;
    let mut area = 0.0f32;
    let mut ra = Some(0.0f32);
    let cost = polyedge_rotate_beauty_calc(coords, edges, e, &mut ra);
    area = ra.unwrap_or(0.0);
    if cost < -1e-6 * area.max(1.0) {
        eheap.insert_or_update(&mut eheap_table[i], cost, e);
    } else if let Some(node) = eheap_table[i] {
        eheap.remove(node);
        eheap_table[i] = None;
    }
    let _ = area;
}

/// `polyedge_beauty_cost_update` — update the 4 half-edges around
/// the rotated edge.
fn polyedge_beauty_cost_update(
    coords: &[[f32; 2]],
    edges: &[HalfEdge],
    e: usize,
    eheap: &mut crate::heap::Heap<usize>,
    eheap_table: &mut [Option<crate::heap::HeapNode>],
) {
    let mut e_arr = [0usize; 4];
    e_arr[0] = edges[e].e_next as usize;
    e_arr[1] = edges[e_arr[0]].e_next as usize;
    let er = edges[e].e_radial as usize;
    e_arr[2] = edges[er].e_next as usize;
    e_arr[3] = edges[e_arr[2]].e_next as usize;
    for &ei in &e_arr {
        if edges[ei].base_index != UINT_MAXV {
            polyedge_beauty_cost_update_single(coords, edges, ei, eheap, eheap_table);
        }
    }
}

/// `polyedge_rotate` — flip the shared edge between two tris.
fn polyedge_rotate(edges: &mut [HalfEdge], e: usize) {
    let mut ed = [0usize; 6];
    ed[0] = e;
    ed[1] = edges[ed[0]].e_next as usize;
    ed[2] = edges[ed[1]].e_next as usize;
    ed[3] = edges[e].e_radial as usize;
    ed[4] = edges[ed[3]].e_next as usize;
    ed[5] = edges[ed[4]].e_next as usize;

    edges[ed[0]].e_next = ed[2] as u32;
    edges[ed[1]].e_next = ed[3] as u32;
    edges[ed[2]].e_next = ed[4] as u32;
    edges[ed[3]].e_next = ed[5] as u32;
    edges[ed[4]].e_next = ed[0] as u32;
    edges[ed[5]].e_next = ed[1] as u32;

    edges[ed[0]].v = edges[ed[5]].v;
    edges[ed[3]].v = edges[ed[2]].v;
}

/// `BLI_polyfill_beautify` — greedy edge-rotate beautification.
/// `tris` is modified in place.
pub fn polyfill_beautify(coords: &[[f32; 2]], tris: &mut [[u32; 3]]) {
    let coords_num = coords.len() as u32;
    let coord_last = coords_num - 1;
    let tris_len = tris.len() as u32;
    let edges_len = tris_len - 1;
    let half_edges_len = 3 * tris_len;

    let mut half_edges = vec![
        HalfEdge {
            v: 0,
            e_next: 0,
            e_radial: UINT_MAXV,
            base_index: UINT_MAXV,
        };
        half_edges_len as usize
    ];
    let mut order_edges: Vec<[u32; 3]> = Vec::with_capacity(2 * edges_len as usize); // [v0,v1,e_half]

    for i in 0..tris_len {
        let mut j_prev = 2u32;
        for j_curr in 0..3u32 {
            let e_index_prev = i * 3 + j_prev;
            let e_index_curr = i * 3 + j_curr;
            half_edges[e_index_prev as usize].v = tris[i as usize][j_prev as usize];
            half_edges[e_index_prev as usize].e_next = e_index_curr;
            let mut e_pair = [
                tris[i as usize][j_prev as usize],
                tris[i as usize][j_curr as usize],
            ];
            if e_pair[0] > e_pair[1] {
                e_pair.swap(0, 1);
            }
            if !is_boundary_edge(e_pair[0], e_pair[1], coord_last) {
                order_edges.push([e_pair[0], e_pair[1], e_index_prev]);
            }
            j_prev = j_curr;
        }
    }

    order_edges.sort_by(|a, b| a[0].cmp(&b[0]).then(a[1].cmp(&b[1])).then(a[2].cmp(&b[2])));

    let mut eheap: crate::heap::Heap<usize> = crate::heap::Heap::new();
    let mut eheap_table: Vec<Option<crate::heap::HeapNode>> =
        vec![None; edges_len as usize];

    let mut i = 0usize;
    let mut base_index = 0u32;
    while i < order_edges.len() {
        let oe_a = order_edges[i];
        let oe_b = order_edges[i + 1];
        i += 2;
        half_edges[oe_a[2] as usize].e_radial = oe_b[2];
        half_edges[oe_b[2] as usize].e_radial = oe_a[2];
        half_edges[oe_a[2] as usize].base_index = base_index;
        half_edges[oe_b[2] as usize].base_index = base_index;
        base_index += 1;
    }

    for i in 0..half_edges_len as usize {
        let e = half_edges[i];
        if e.e_radial < i as u32 {
            let cost = polyedge_rotate_beauty_calc(coords, &half_edges, i, &mut None);
            if cost < 0.0 {
                eheap_table[e.base_index as usize] = Some(eheap.insert(cost, i));
            }
        }
    }

    while !eheap.is_empty() {
        let Some(e) = eheap.pop_min() else {
            break;
        };
        eheap_table[half_edges[e].base_index as usize] = None;
        polyedge_rotate(&mut half_edges, e);
        polyedge_beauty_cost_update(coords, &half_edges, e, &mut eheap, &mut eheap_table);
    }

    let mut tri_index = 0usize;
    for i in 0..half_edges_len as usize {
        if half_edges[i].v != UINT_MAXV {
            let tri = &mut tris[tri_index];
            tri_index += 1;
            let mut e = i;
            for k in 0..3 {
                tri[k] = half_edges[e].v;
                half_edges[e].v = UINT_MAXV;
                e = half_edges[e].e_next as usize;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn beautify_runs() {
        let ar = [
            [0.0f32, 0.0],
            [2.0, 0.0],
            [2.0, 1.0],
            [1.0, 0.5],
            [0.0, 1.0],
        ];
        let mut tris = crate::polyfill_2d::polyfill_calc_pub(&ar, 0);
        polyfill_beautify(&ar, &mut tris);
        assert_eq!(tris.len(), 3);
    }
}
