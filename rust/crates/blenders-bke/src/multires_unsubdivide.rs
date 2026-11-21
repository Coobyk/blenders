//! Port of `intern/multires_unsubdivide.cc` — BMesh-based
//! "unsubdivide" (rebuild a lower multires level from a high-res
//! quad mesh): pole detection, face-center tagging, elem ids,
//! `unsubdivide_build_base_mesh_from_tags` (via `dissolve_verts`),
//! `edge_step`/`face_step`/`get_initial_edge_y` grid walk,
//! `write_loop_in_face_grid` / `write_face_grid_in_unsubdivide_grid`.
//! `PORT_PENDING`: `MultiresUnsubdivideContext` grid extraction
//! driver (`multires_unsubdivide_extract_grids`,
//! `multires_unsubdivide_to_basemesh`,
//! `multiresModifier_rebuild_subdiv`) — needs MDisps + CustomData.

use std::collections::VecDeque;

use blenders_bmesh::iterators::{EdgesOfFace, EdgesOfVert, FacesOfVert, VertsOfFace};
use blenders_bmesh::operators::{BMOpSlot, BMOperator, BMOSlotType};
use blenders_bmesh::query::{bm_edge_loop_pair, bm_edge_share_quad_check, bm_vert_is_boundary, bm_vert_is_wire};
use blenders_bmesh::structure::bm_edge_other_vert;
use blenders_bmesh::{BMesh, BM_EDGE, BM_ELEM_SELECT, BM_ELEM_TAG, BM_FACE, BM_NIL, BM_VERT};
use blenders_math::vector::Vec3;

fn vert_index(bm: &BMesh, v: usize) -> i32 {
    bm.vpool[v].head.index
}

fn vert_tagged(bm: &BMesh, v: usize) -> bool {
    bm.vpool[v].head.hflag & BM_ELEM_TAG != 0
}

fn set_vert_tag(bm: &mut BMesh, v: usize, val: bool) {
    if val {
        bm.vpool[v].head.hflag |= BM_ELEM_TAG;
    } else {
        bm.vpool[v].head.hflag &= !BM_ELEM_TAG;
    }
}

/// `is_vertex_in_id`.
fn is_vertex_in_id(bm: &BMesh, v: usize, elem_id: &[i32], elem: i32) -> bool {
    elem_id[vert_index(bm, v) as usize] == elem
}

/// `is_vertex_pole_three` — non-boundary 3-valence vert.
fn is_vertex_pole_three(bm: &BMesh, v: usize) -> bool {
    !bm_vert_is_boundary(bm, v) && EdgesOfVert::new(bm, v).count() == 3
}

/// `is_vertex_pole` — non-boundary 3 or >4 valence vert.
fn is_vertex_pole(bm: &BMesh, v: usize) -> bool {
    let n = EdgesOfVert::new(bm, v).count();
    !bm_vert_is_boundary(bm, v) && (n == 3 || n > 4)
}

/// `unsubdivide_find_any_pole` — prefer a valence-3 pole.
fn unsubdivide_find_any_pole(bm: &BMesh, elem_id: &[i32], elem: i32) -> Option<usize> {
    let mut pole = None;
    for v in 0..bm.totvert as usize {
        if is_vertex_in_id(bm, v, elem_id, elem) && is_vertex_pole_three(bm, v) {
            return Some(v);
        }
        if is_vertex_in_id(bm, v, elem_id, elem) && is_vertex_pole(bm, v) {
            pole = Some(v);
        }
    }
    pole
}

/// `unsubdivide_is_all_quads`.
fn unsubdivide_is_all_quads(bm: &BMesh) -> bool {
    if bm.totface < 3 {
        return false;
    }
    for f in 0..bm.totface as usize {
        if VertsOfFace::new(bm, f).count() != 4 {
            return false;
        }
    }
    for v in 0..bm.totvert as usize {
        if bm_vert_is_wire(bm, v) || bm.vert(v).e == BM_NIL {
            return false;
        }
    }
    true
}

/// `is_vertex_diagonal` — not edge-connected.
fn is_vertex_diagonal(bm: &BMesh, from_v: usize, to_v: usize) -> bool {
    blenders_bmesh::structure::bmesh_disk_edge_exists(bm, from_v, to_v) == BM_NIL
}

/// `unsubdivide_face_center_vertex_tag` — BFS over diagonal
/// (non-adjacent-in-face) verts, tagging them.
fn unsubdivide_face_center_vertex_tag(bm: &mut BMesh, initial_vertex: usize) {
    let mut visited = vec![false; bm.totvert as usize];
    let mut queue: VecDeque<usize> = VecDeque::new();
    for f in FacesOfVert::new(bm, initial_vertex).collect::<Vec<_>>() {
        for neighbor_v in VertsOfFace::new(bm, f).collect::<Vec<_>>() {
            let ni = vert_index(bm, neighbor_v) as usize;
            if neighbor_v != initial_vertex
                && is_vertex_diagonal(bm, neighbor_v, initial_vertex)
            {
                queue.push_back(neighbor_v);
                visited[ni] = true;
                set_vert_tag(bm, neighbor_v, true);
            }
        }
    }
    while let Some(from_v) = queue.pop_front() {
        let mut diagonals: Vec<usize> = Vec::new();
        for f in FacesOfVert::new(bm, from_v).collect::<Vec<_>>() {
            for neighbor_v in VertsOfFace::new(bm, f).collect::<Vec<_>>() {
                if neighbor_v != from_v && is_vertex_diagonal(bm, neighbor_v, from_v) {
                    diagonals.push(neighbor_v);
                }
            }
        }
        for diagonal_v in diagonals {
            for f in FacesOfVert::new(bm, diagonal_v).collect::<Vec<_>>() {
                for neighbor_v in VertsOfFace::new(bm, f).collect::<Vec<_>>() {
                    let ni = vert_index(bm, neighbor_v) as usize;
                    if !visited[ni]
                        && neighbor_v != diagonal_v
                        && is_vertex_diagonal(bm, neighbor_v, diagonal_v)
                    {
                        queue.push_back(neighbor_v);
                        visited[ni] = true;
                        set_vert_tag(bm, neighbor_v, true);
                    }
                }
            }
        }
    }
}

/// `unsubdivide_is_center_vertex_tag_valid`.
fn unsubdivide_is_center_vertex_tag_valid(bm: &BMesh, elem_id: &[i32], elem: i32) -> bool {
    for v in 0..bm.totvert as usize {
        if is_vertex_in_id(bm, v, elem_id, elem) {
            if vert_tagged(bm, v) {
                if bm_vert_is_boundary(bm, v) {
                    return false;
                }
                for f in FacesOfVert::new(bm, v) {
                    for neighbor_v in VertsOfFace::new(bm, f) {
                        if neighbor_v != v && vert_tagged(bm, neighbor_v) {
                            return false;
                        }
                    }
                }
            }
            if bm_vert_is_boundary(bm, v) {
                let mut any_tagged = false;
                for f in FacesOfVert::new(bm, v) {
                    for neighbor_v in VertsOfFace::new(bm, f) {
                        if neighbor_v != v && vert_tagged(bm, neighbor_v) {
                            any_tagged = true;
                        }
                    }
                }
                if !any_tagged {
                    return false;
                }
            }
        }
    }
    true
}

/// `unsubdivide_tag_disconnected_mesh_element` — try poles first,
/// then each vert of the component's first face, until a valid
/// center-tagging is found.
fn unsubdivide_tag_disconnected_mesh_element(
    bm: &mut BMesh,
    elem_id: &[i32],
    elem: i32,
) -> bool {
    let mut initial_vertex: VecDeque<usize> = VecDeque::new();
    if let Some(pole) = unsubdivide_find_any_pole(bm, elem_id, elem) {
        initial_vertex.push_back(pole);
    }
    let mut init_face = None;
    'outer: for f in 0..bm.totface as usize {
        for v in VertsOfFace::new(bm, f) {
            if is_vertex_in_id(bm, v, elem_id, elem) {
                init_face = Some(f);
                break 'outer;
            }
        }
    }
    let init_face = match init_face {
        Some(f) => f,
        None => return false,
    };
    for v in VertsOfFace::new(bm, init_face).collect::<Vec<_>>() {
        initial_vertex.push_back(v);
    }
    while let Some(iv) = initial_vertex.pop_front() {
        unsubdivide_face_center_vertex_tag(bm, iv);
        if unsubdivide_is_center_vertex_tag_valid(bm, elem_id, elem) {
            return true;
        }
        for v in 0..bm.totvert as usize {
            if is_vertex_in_id(bm, v, elem_id, elem) {
                set_vert_tag(bm, v, false);
            }
        }
    }
    false
}

/// `unsubdivide_init_elem_ids` — flood-fill connected components;
/// returns count.
fn unsubdivide_init_elem_ids(bm: &mut BMesh, elem_id: &mut [i32]) -> i32 {
    blenders_bmesh::mesh::bm_mesh_elem_index_ensure(bm, BM_VERT);
    let mut visited = vec![false; bm.totvert as usize];
    let mut current_id = 0;
    for i in 0..bm.totvert as usize {
        if !visited[i] {
            let mut queue: VecDeque<usize> = VecDeque::new();
            visited[i] = true;
            elem_id[i] = current_id;
            let iv = blenders_bmesh::mesh::bm_vert_at_index_find(bm, i as i32);
            queue.push_back(iv);
            while let Some(current_v) = queue.pop_front() {
                for ed in EdgesOfVert::new(bm, current_v).collect::<Vec<_>>() {
                    let neighbor_v = bm_edge_other_vert(bm, ed, current_v);
                    let ni = vert_index(bm, neighbor_v) as usize;
                    if !visited[ni] {
                        visited[ni] = true;
                        elem_id[ni] = current_id;
                        queue.push_back(neighbor_v);
                    }
                }
            }
            current_id += 1;
        }
    }
    current_id
}

fn dissolve_verts_by_flag(bm: &mut BMesh, hflag: u8) {
    /* `dissolve_verts verts=%hv use_face_split=false use_boundary_tear=true`. */
    let mut verts_slot = BMOpSlot::default();
    verts_slot.slot_type = BMOSlotType::ElemBuf;
    verts_slot.name = "verts".to_string();
    let mut fs_slot = BMOpSlot::default();
    fs_slot.slot_type = BMOSlotType::Bool;
    fs_slot.name = "use_face_split".to_string();
    fs_slot.b = false;
    let mut bt_slot = BMOpSlot::default();
    bt_slot.slot_type = BMOSlotType::Bool;
    bt_slot.name = "use_boundary_tear".to_string();
    bt_slot.b = true;
    let mut op = BMOperator {
        opcode: 0,
        type_flag: 0,
        flag: 0,
        slots_in: vec![verts_slot, fs_slot, bt_slot],
        slots_out: Vec::new(),
    };
    blenders_bmesh::operators::bmo_slot_buffer_from_hflag(
        bm, &mut op, false, "verts", BM_VERT, hflag, true,
    );
    blenders_bmesh::bmo::bmo_dissolve::bmo_dissolve_verts_exec(bm, &mut op);
}

/// `unsubdivide_build_base_mesh_from_tags` — dissolve all verts
/// adjacent to tagged verts, then dissolve the tagged ones.
fn unsubdivide_build_base_mesh_from_tags(bm: &mut BMesh) {
    for v in 0..bm.totvert as usize {
        bm.vpool[v].head.hflag |= BM_ELEM_SELECT;
    }
    for e in 0..bm.totedge as usize {
        bm.epool[e].head.hflag |= BM_ELEM_SELECT;
    }
    for f in 0..bm.totface as usize {
        bm.fpool[f].head.hflag |= BM_ELEM_SELECT;
    }
    for v in 0..bm.totvert as usize {
        for ed in EdgesOfVert::new(bm, v).collect::<Vec<_>>() {
            let v_neighbor = bm_edge_other_vert(bm, ed, v);
            if vert_tagged(bm, v_neighbor) {
                bm.vpool[v].head.hflag &= !BM_ELEM_SELECT;
            }
        }
    }
    dissolve_verts_by_flag(bm, BM_ELEM_TAG);
    for v in 0..bm.totvert as usize {
        bm.vpool[v].head.hflag &= !BM_ELEM_TAG;
    }
    for v in 0..bm.totvert as usize {
        if bm.vpool[v].head.hflag & BM_ELEM_SELECT == 0 {
            bm.vpool[v].head.hflag |= BM_ELEM_TAG;
        }
    }
    dissolve_verts_by_flag(bm, BM_ELEM_TAG);
}

/// `multires_unsubdivide_single_level` — one unsubdivide step.
/// Returns false if the mesh isn't all-quads or no valid tagging.
pub fn multires_unsubdivide_single_level(bm: &mut BMesh) -> bool {
    if !unsubdivide_is_all_quads(bm) {
        return false;
    }
    blenders_bmesh::mesh::bm_mesh_elem_index_ensure(bm, BM_VERT);
    let mut elem_id = vec![0i32; bm.totvert as usize];
    let tot_ids = unsubdivide_init_elem_ids(bm, &mut elem_id);
    for v in 0..bm.totvert as usize {
        bm.vpool[v].head.hflag &= !(BM_ELEM_SELECT | blenders_bmesh::BM_ELEM_HIDDEN | BM_ELEM_TAG);
    }
    let mut valid = true;
    for id in 0..tot_ids {
        if !unsubdivide_tag_disconnected_mesh_element(bm, &elem_id, id) {
            valid = false;
            break;
        }
    }
    if valid {
        unsubdivide_build_base_mesh_from_tags(bm);
    }
    valid
}

/// `edge_step` — step across the quad to the next edge; fills
/// `r_next_vertex`.
fn edge_step(bm: &BMesh, v: usize, edge: usize, r_next_vertex: &mut usize) -> usize {
    if edge == BM_NIL {
        *r_next_vertex = v;
        return edge;
    }
    *r_next_vertex = bm_edge_other_vert(bm, edge, v);
    for test_edge in EdgesOfVert::new(bm, *r_next_vertex) {
        if !bm_edge_share_quad_check(bm, test_edge, edge) {
            return test_edge;
        }
    }
    BM_NIL
}

/// `face_step` — the quad on the other side of `edge` (or `f`).
fn face_step(bm: &BMesh, edge: usize, f: usize) -> usize {
    if let Some((l_a, l_b)) = bm_edge_loop_pair(bm, edge) {
        let fa = bm.loo(l_a).f;
        let fb = bm.loo(l_b).f;
        if f == fa {
            return fb;
        }
        if f == fb {
            return fa;
        }
    }
    f
}

/// `get_initial_edge_y` — the face edge touching `initial_vertex`
/// that isn't `edge_x`.
fn get_initial_edge_y(bm: &BMesh, f: usize, edge_x: usize, initial_vertex: usize) -> usize {
    for test_edge in EdgesOfFace::new(bm, f) {
        if edge_x != test_edge {
            let (v1, v2) = (bm.edge(test_edge).v1, bm.edge(test_edge).v2);
            if (v1 != initial_vertex && v2 == initial_vertex)
                || (v2 != initial_vertex && v1 == initial_vertex)
            {
                return test_edge;
            }
        }
    }
    BM_NIL
}

/// `write_loop_in_face_grid` — rotate/flip an `orig_grid_size`
/// displacement patch into a `face_grid_size` slot, corner `loop`.
pub fn write_loop_in_face_grid(
    face_grid: &mut [Vec3],
    mdisp_disps: &[Vec3],
    face_grid_size: usize,
    orig_grid_size: usize,
    lp: usize,
) {
    let grid_offset = orig_grid_size - 1;
    let origin = [grid_offset as i32; 2];
    let (step_x, step_y): ([i32; 2], [i32; 2]) = match lp {
        0 => ([-1, 0], [0, -1]),
        1 => ([0, 1], [-1, 0]),
        2 => ([1, 0], [0, 1]),
        3 => ([0, -1], [1, 0]),
        _ => ([0, 0], [0, 0]),
    };
    for y in 0..orig_grid_size {
        for x in 0..orig_grid_size {
            let remap_x =
                (origin[1] + step_x[1] * x as i32 + step_y[1] * y as i32) as usize;
            let remap_y =
                (origin[0] + step_x[0] * x as i32 + step_y[0] * y as i32) as usize;
            let final_index = remap_x + remap_y * face_grid_size;
            face_grid[final_index] = mdisp_disps[x + y * orig_grid_size];
        }
    }
}

/// `MultiresUnsubdivideGrid` — extracted high-res grid for one
/// base-mesh face.
#[derive(Default)]
pub struct MultiresUnsubdivideGrid {
    /// `grid_co` — vertex positions in grid order.
    pub grid_co: Vec<Vec3>,
    /// `grid_size`.
    pub grid_size: usize,
    /// `face_index`.
    pub face_index: i32,
    /// `loop_index`.
    pub loop_index: i32,
    /// `ptex_face_index`.
    pub ptex_face_index: i32,
}

/// `MultiresUnsubdivideContext` — `num_grids`, grids, base mesh.
#[derive(Default)]
pub struct MultiresUnsubdivideContext {
    /// `grids` per base face.
    pub grids: Vec<MultiresUnsubdivideGrid>,
    /// `num_grids`.
    pub num_grids: i32,
    /// `subdivision_level` produced.
    pub subdivision_level: i32,
    /// `num_new_levels`.
    pub num_new_levels: i32,
}

/// `MultiresUnsubdivideInfo`.
#[derive(Default)]
pub struct MultiresUnsubdivideInfo {
    /// `totals` — verts/edges/faces/loops created.
    pub totals: [i32; 4],
}

/// `write_face_grid_in_unsubdivide_grid` — splice one face's
/// `face_grid_size` patch into the big grid at `(gunsub_x, gunsub_y)`
/// with the C-side flip.
pub fn write_face_grid_in_unsubdivide_grid(
    grid: &mut MultiresUnsubdivideGrid,
    face_grid: &[Vec3],
    face_grid_size: usize,
    gunsub_x: usize,
    gunsub_y: usize,
) {
    let grid_it = face_grid_size - 1;
    for y in 0..face_grid_size {
        for x in 0..face_grid_size {
            let remap_x = grid_it * gunsub_x + x;
            let remap_y = grid_it * gunsub_y + y;
            let remap_index_y = grid.grid_size - remap_x - 1;
            let remap_index_x = grid.grid_size - remap_y - 1;
            let grid_index = remap_index_x + remap_index_y * grid.grid_size;
            grid.grid_co[grid_index] = face_grid[x + y * face_grid_size];
        }
    }
}
