//! Port of `intern/subdiv_displacement_multires.cc` — `Displacement`
//! evaluator backed by `MDisps` grids (multires modifier).
//!
//! `eval_limit_point_and_derivatives`,
//! `multiresModifier_ensure_external_read` and `face_ptex_offset_get`
//! are injected; `MDisps.disps` → `Vec<Vec3>` grids.

use blenders_bli::offset_indices::OffsetIndices;
use blenders_math::matrix::Mat3;
use blenders_math::vector::{Vec2, Vec3};

use crate::subdiv_ccg_mask::{
    grid_size_from_level, ptex_face_uv_to_grid_uv, rotate_quad_to_corner, PolyCornerIndex,
};

/// `MDisps`.
pub struct MDisps {
    /// `float (*disps)[3]` or empty when `disps == nullptr`.
    pub disps: Vec<Vec3>,
}

/// `AverageWith`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum AverageWith {
    None,
    All,
    Prev,
    Next,
}

/// `MultiresDisplacementData`.
pub struct MultiresDisplacementData {
    /// `Subdiv *subdiv` — handle for the injected limit eval.
    pub subdiv: usize,
    pub grid_size: i32,
    /// `const Mesh *mesh` — handle.
    pub mesh: usize,
    /// `const MultiresModifierData *mmd` — handle.
    pub mmd: usize,
    /// `OffsetIndices<int> faces` (offsets slice).
    pub faces: Vec<i32>,
    /// `const MDisps *mdisps` (corner-indexed grids).
    pub mdisps: Vec<MDisps>,
    /// Indexed by PTEX face index → face/corner. For a quad face this
    /// is the index of the first corner only (one PTEX per quad).
    pub ptex_face_corner: Vec<PolyCornerIndex>,
    /// Indexed by coarse face index → first PTEX face index.
    pub face_ptex_offset: Vec<i32>,
    /// Sanity check — `initialize()` ran before `eval_displacement()`.
    pub is_initialized: bool,
}

impl MultiresDisplacementData {
    fn faces_oi(&self) -> OffsetIndices<'_> {
        OffsetIndices { data: &self.faces }
    }
}

/// `grid_uv_to_ptex_face_uv` (subdiv_inline.hh).
fn grid_uv_to_ptex_face_uv(grid_u: f32, grid_v: f32) -> Vec2 {
    Vec2::new(1.0 - grid_v, 1.0 - grid_u)
}

/// `rotate_grid_to_quad` (subdiv_inline.hh).
fn rotate_grid_to_quad(corner: i32, grid_u: f32, grid_v: f32) -> Vec2 {
    match corner {
        0 => Vec2::new(0.5 - grid_v * 0.5, 0.5 - grid_u * 0.5),
        1 => Vec2::new(0.5 + grid_u * 0.5, 0.5 - grid_v * 0.5),
        2 => Vec2::new(0.5 + grid_v * 0.5, 0.5 + grid_u * 0.5),
        _ => {
            debug_assert_eq!(corner, 3);
            Vec2::new(0.5 - grid_u * 0.5, 0.5 + grid_v * 0.5)
        }
    }
}

/// `BKE_multires_construct_tangent_matrix` (multires_inline.hh).
fn bke_multires_construct_tangent_matrix(d_pdu: Vec3, d_pdv: Vec3, corner: i32) -> Mat3 {
    let (x, y) = match corner {
        0 => (-d_pdv, -d_pdu),
        1 => (d_pdu, -d_pdv),
        2 => (d_pdv, d_pdu),
        _ => {
            debug_assert_eq!(corner, 3);
            (-d_pdu, d_pdv)
        }
    };
    let z = d_pdu.cross(d_pdv);
    Mat3::from_cols(x.normalized(), y.normalized(), z.normalized())
}

/// `displacement_get_grid_and_coord` — `(corner, &grid, grid_uv)`.
fn displacement_get_grid_and_coord<'a>(
    data: &'a MultiresDisplacementData,
    ptex_face_index: usize,
    u: f32,
    v: f32,
) -> (i32, &'a MDisps, Vec2) {
    let face_corner = data.ptex_face_corner[ptex_face_index];
    let face = data.faces_oi().group(face_corner.face_index as usize);
    let start_grid_index = face.start() + face_corner.corner as i64;
    if face.size() == 4 {
        let (corner, corner_uv) = rotate_quad_to_corner(u, v);
        let g = ptex_face_uv_to_grid_uv(corner_uv);
        (
            corner,
            &data.mdisps[(start_grid_index + corner as i64) as usize],
            g,
        )
    } else {
        let g = ptex_face_uv_to_grid_uv(Vec2::new(u, v));
        (0, &data.mdisps[start_grid_index as usize], g)
    }
}

/// `displacement_get_other_grid` — the grid `corner_delta` away.
fn displacement_get_other_grid<'a>(
    data: &'a MultiresDisplacementData,
    ptex_face_index: usize,
    corner: i32,
    corner_delta: i32,
) -> &'a MDisps {
    let face_corner = data.ptex_face_corner[ptex_face_index];
    let face = data.faces_oi().group(face_corner.face_index as usize);
    let effective_corner = if face.size() == 4 {
        corner
    } else {
        face_corner.corner
    };
    let next_corner = ((effective_corner + corner_delta) as i64 + face.size()) % face.size();
    &data.mdisps[(face.start() + next_corner) as usize]
}

/// `read_displacement_grid` — grid sample + `AverageWith` flag.
fn read_displacement_grid(
    displacement_grid: &MDisps,
    grid_size: i32,
    grid_u: f32,
    grid_v: f32,
) -> (AverageWith, Vec3) {
    if displacement_grid.disps.is_empty() {
        return (AverageWith::None, Vec3::ZERO);
    }
    let x = (grid_u * (grid_size - 1) as f32).round() as i32;
    let y = (grid_v * (grid_size - 1) as f32).round() as i32;
    let tangent_d = displacement_grid.disps[(y * grid_size + x) as usize];
    if x == 0 && y == 0 {
        return (AverageWith::All, tangent_d);
    }
    if x == 0 {
        return (AverageWith::Prev, tangent_d);
    }
    if y == 0 {
        return (AverageWith::Next, tangent_d);
    }
    (AverageWith::None, tangent_d)
}

/// `average_convert_grid_coord_to_ptex`.
fn average_convert_grid_coord_to_ptex(
    num_corners: i64,
    corner: i32,
    grid_u: f32,
    grid_v: f32,
) -> Vec2 {
    if num_corners == 4 {
        rotate_grid_to_quad(corner, grid_u, grid_v)
    } else {
        grid_uv_to_ptex_face_uv(grid_u, grid_v)
    }
}

/// `average_construct_tangent_matrix` — partials at `(u, v)` of the
/// other PTEX face via the injected `eval_limit`.
fn average_construct_tangent_matrix(
    num_corners: i64,
    ptex_face_index: usize,
    corner: i32,
    u: f32,
    v: f32,
    eval_limit: &mut dyn FnMut(usize, f32, f32) -> (Vec3, Vec3, Vec3),
) -> Mat3 {
    let is_quad = num_corners == 4;
    let quad_corner = if is_quad { corner } else { 0 };
    let (_dummy_p, d_pdu, d_pdv) = eval_limit(ptex_face_index, u, v);
    bke_multires_construct_tangent_matrix(d_pdu, d_pdv, quad_corner)
}

/// `average_read_displacement_object` — other-grid sample → object
/// space through the other face's tangent matrix.
#[allow(clippy::too_many_arguments)]
fn average_read_displacement_object(
    data: &MultiresDisplacementData,
    displacement_grid: &MDisps,
    grid_u: f32,
    grid_v: f32,
    ptex_face_index: usize,
    corner_index: i32,
    eval_limit: &mut dyn FnMut(usize, f32, f32) -> (Vec3, Vec3, Vec3),
) -> Vec3 {
    let face_corner = data.ptex_face_corner[ptex_face_index];
    let num_corners = data.faces_oi().group(face_corner.face_index as usize).size();
    /* (u, v) within the other PTEX face matching the grid coords. */
    let uv = average_convert_grid_coord_to_ptex(num_corners, corner_index, grid_u, grid_v);
    /* Tangent matrix for the other PTEX face's partials. */
    let tangent_matrix = average_construct_tangent_matrix(
        num_corners,
        ptex_face_index,
        corner_index,
        uv.x,
        uv.y,
        eval_limit,
    );
    /* Displacement in the other grid's tangent space. */
    let (_, tangent_d) =
        read_displacement_grid(displacement_grid, data.grid_size, grid_u, grid_v);
    /* Convert to object space. */
    tangent_matrix.transform_direction(tangent_d)
}

/// `average_get_other_ptex_and_corner`.
fn average_get_other_ptex_and_corner(
    data: &MultiresDisplacementData,
    ptex_face_index: usize,
    corner: i32,
    corner_delta: i32,
) -> (usize, i32) {
    let face_corner = data.ptex_face_corner[ptex_face_index];
    let face_index = face_corner.face_index as usize;
    let num_corners = data.faces_oi().group(face_index).size();
    let is_quad = num_corners == 4;
    let start_ptex_face_index = data.face_ptex_offset[face_index];
    let other_corner_index = ((corner + corner_delta) as i64 + num_corners) % num_corners;
    let other_ptex_face_index = if is_quad {
        start_ptex_face_index
    } else {
        start_ptex_face_index + other_corner_index as i32
    };
    (other_ptex_face_index as usize, other_corner_index as i32)
}

/// `average_with_other` — fold the neighbor grid's object-space
/// displacement into `r_d` (grid coords already relative to other).
#[allow(clippy::too_many_arguments)]
fn average_with_other(
    data: &MultiresDisplacementData,
    ptex_face_index: usize,
    corner: i32,
    grid_u: f32,
    grid_v: f32,
    corner_delta: i32,
    eval_limit: &mut dyn FnMut(usize, f32, f32) -> (Vec3, Vec3, Vec3),
    r_d: &mut Vec3,
) {
    let other_displacement_grid =
        displacement_get_other_grid(data, ptex_face_index, corner, corner_delta);
    let (other_ptex_face_index, other_corner_index) =
        average_get_other_ptex_and_corner(data, ptex_face_index, corner, corner_delta);
    let other_d = average_read_displacement_object(
        data,
        other_displacement_grid,
        grid_u,
        grid_v,
        other_ptex_face_index,
        other_corner_index,
        eval_limit,
    );
    *r_d += other_d;
    *r_d = *r_d * 0.5;
}

/// `average_with_all`.
fn average_with_all(
    data: &MultiresDisplacementData,
    ptex_face_index: usize,
    corner: i32,
    eval_limit: &mut dyn FnMut(usize, f32, f32) -> (Vec3, Vec3, Vec3),
    r_d: &mut Vec3,
) {
    let face_corner = data.ptex_face_corner[ptex_face_index];
    let num_corners = data.faces_oi().group(face_corner.face_index as usize).size();
    for corner_delta in 1..num_corners {
        average_with_other(
            data,
            ptex_face_index,
            corner,
            0.0,
            0.0,
            corner_delta as i32,
            eval_limit,
            r_d,
        );
    }
}

/// `average_displacement`.
fn average_displacement(
    data: &MultiresDisplacementData,
    average_with: AverageWith,
    ptex_face_index: usize,
    corner: i32,
    grid_u: f32,
    grid_v: f32,
    eval_limit: &mut dyn FnMut(usize, f32, f32) -> (Vec3, Vec3, Vec3),
    r_d: &mut Vec3,
) {
    match average_with {
        AverageWith::All => {
            average_with_all(data, ptex_face_index, corner, eval_limit, r_d)
        }
        AverageWith::Prev => average_with_other(
            data,
            ptex_face_index,
            corner,
            grid_v,
            0.0,
            -1,
            eval_limit,
            r_d,
        ),
        AverageWith::Next => average_with_other(
            data,
            ptex_face_index,
            corner,
            0.0,
            grid_u,
            1,
            eval_limit,
            r_d,
        ),
        AverageWith::None => {}
    }
}

/// `displacement_get_face_corner`.
fn displacement_get_face_corner(
    data: &MultiresDisplacementData,
    ptex_face_index: usize,
    u: f32,
    v: f32,
) -> i32 {
    let face_corner = data.ptex_face_corner[ptex_face_index];
    let num_corners = data.faces_oi().group(face_corner.face_index as usize).size();
    let is_quad = num_corners == 4;
    if is_quad {
        let (corner, _corner_uv) = rotate_quad_to_corner(u, v);
        return corner;
    }
    face_corner.corner
}

/// `Displacement` — evaluator vtable over `MultiresDisplacementData`.
pub struct Displacement {
    pub user_data: MultiresDisplacementData,
}

impl Displacement {
    /// `initialize` — `multiresModifier_ensure_external_read` injected.
    pub fn initialize(&mut self, ensure_external_read: &mut dyn FnMut(usize, usize)) {
        ensure_external_read(self.user_data.mesh, self.user_data.mmd);
        self.user_data.is_initialized = true;
    }

    /// `eval_displacement`.
    pub fn eval_displacement(
        &mut self,
        ptex_face_index: usize,
        u: f32,
        v: f32,
        d_pdu: Vec3,
        d_pdv: Vec3,
        eval_limit: &mut dyn FnMut(usize, f32, f32) -> (Vec3, Vec3, Vec3),
    ) -> Vec3 {
        debug_assert!(self.user_data.is_initialized);
        let grid_size = self.user_data.grid_size;
        /* Get displacement in tangent space. */
        let (corner_of_quad, displacement_grid, grid_uv) =
            displacement_get_grid_and_coord(&self.user_data, ptex_face_index, u, v);
        /* Read current grid displacement and see if averaging is
         * needed. */
        let (average_with, tangent_d) =
            read_displacement_grid(displacement_grid, grid_size, grid_uv.x, grid_uv.y);
        /* Convert it to the object space. */
        let tangent_matrix = bke_multires_construct_tangent_matrix(d_pdu, d_pdv, corner_of_quad);
        let mut r_d = tangent_matrix.transform_direction(tangent_d);
        /* For the boundary points of grid average two (or all)
         * neighbor grids. */
        let corner = displacement_get_face_corner(&self.user_data, ptex_face_index, u, v);
        average_displacement(
            &self.user_data,
            average_with,
            ptex_face_index,
            corner,
            grid_uv.x,
            grid_uv.y,
            eval_limit,
            &mut r_d,
        );
        r_d
    }
}

/// `count_num_ptex_faces` — 1 per quad, else corner count.
pub fn count_num_ptex_faces(faces: OffsetIndices<'_>, faces_num: usize) -> usize {
    let mut num_ptex_faces = 0;
    for face_index in 0..faces_num {
        let f = faces.group(face_index);
        num_ptex_faces += if f.size() == 4 { 1 } else { f.size() as usize };
    }
    num_ptex_faces
}

/// `displacement_data_init_mapping`.
fn displacement_data_init_mapping(
    data: &mut MultiresDisplacementData,
    faces: OffsetIndices<'_>,
    faces_num: usize,
) {
    let num_ptex_faces = count_num_ptex_faces(faces, faces_num);
    data.ptex_face_corner.clear();
    data.ptex_face_corner.reserve(num_ptex_faces);
    /* Fill in offsets. */
    for face_index in 0..faces_num {
        let face = faces.group(face_index);
        if face.size() == 4 {
            data.ptex_face_corner.push(PolyCornerIndex {
                face_index: face_index as i32,
                corner: 0,
            });
        } else {
            for corner in 0..face.size() {
                data.ptex_face_corner.push(PolyCornerIndex {
                    face_index: face_index as i32,
                    corner: corner as i32,
                });
            }
        }
    }
}

/// `displacement_init_data` + `displacement_init_functions` →
/// `displacement_attach_from_multires` result.
///
/// `faces`/`mdisps`/`face_ptex_offset` come from the caller (Mesh +
/// Subdiv accessors).
pub fn displacement_attach_from_multires(
    subdiv: usize,
    mesh: usize,
    mmd: usize,
    faces: Vec<i32>,
    mdisps: Option<Vec<MDisps>>,
    face_ptex_offset: Vec<i32>,
    totlvl: i32,
    faces_num: usize,
) -> Option<Displacement> {
    /* It is possible to have mesh without CD_MDISPS layer. Happens
     * when using dynamic topology. */
    let mdisps = mdisps?;

    /* Build the ptex-corner mapping before moving `faces` in. */
    let ptex_face_corner = {
        let oi = OffsetIndices { data: &faces };
        let mut v = Vec::with_capacity(count_num_ptex_faces(oi, faces_num));
        for face_index in 0..faces_num {
            let face = oi.group(face_index);
            if face.size() == 4 {
                v.push(PolyCornerIndex {
                    face_index: face_index as i32,
                    corner: 0,
                });
            } else {
                for corner in 0..face.size() {
                    v.push(PolyCornerIndex {
                        face_index: face_index as i32,
                        corner: corner as i32,
                    });
                }
            }
        }
        v
    };
    let data = MultiresDisplacementData {
        subdiv,
        grid_size: grid_size_from_level(totlvl),
        mesh,
        mmd,
        faces,
        mdisps,
        ptex_face_corner,
        face_ptex_offset,
        is_initialized: false,
    };
    Some(Displacement { user_data: data })
}

/* `IndexRange` import — kept for API parity. */
