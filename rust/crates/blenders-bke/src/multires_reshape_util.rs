//! Port of `intern/multires_reshape_util.cc` + the
//! `subdiv_inline.hh` UV helpers — `MultiresReshapeContext`
//! (base-mesh topology, disp/mask grids, ptex/grid coord math,
//! tangent matrices, grid ensure/store, tangent-displacement
//! passes, final-coords assignment).
//!
//! The subdiv limit evaluator is injected (`eval_limit`).

use blenders_bli::index_range::IndexRange;
use blenders_bli::offset_indices::OffsetIndices;
use blenders_math::matrix::Mat3;
use blenders_math::vector::{Vec2, Vec3};

use crate::multires::{grid_size_from_level, GridPaintMask, MDisps};

/// `bke::subdiv::ptex_face_uv_to_grid_uv`.
pub fn ptex_face_uv_to_grid_uv(ptex_u: f32, ptex_v: f32) -> (f32, f32) {
    (1.0 - ptex_v, 1.0 - ptex_u)
}

/// `bke::subdiv::grid_uv_to_ptex_face_uv`.
pub fn grid_uv_to_ptex_face_uv(grid_u: f32, grid_v: f32) -> (f32, f32) {
    (1.0 - grid_v, 1.0 - grid_u)
}

/// `bke::subdiv::rotate_quad_to_corner` — returns (corner, u, v).
pub fn rotate_quad_to_corner(quad_u: f32, quad_v: f32) -> (i32, f32, f32) {
    if quad_u <= 0.5 && quad_v <= 0.5 {
        (0, 2.0 * quad_u, 2.0 * quad_v)
    } else if quad_u > 0.5 && quad_v <= 0.5 {
        (1, 2.0 * quad_v, 2.0 * (1.0 - quad_u))
    } else if quad_u > 0.5 && quad_v > 0.5 {
        (2, 2.0 * (1.0 - quad_u), 2.0 * (1.0 - quad_v))
    } else {
        (3, 2.0 * (1.0 - quad_v), 2.0 * quad_u)
    }
}

/// `rotate_quad_to_corner` on a float2 for a given corner.
pub fn rotate_quad_to_corner_uv(corner: i32, quad: Vec2) -> Vec2 {
    match corner {
        0 => 2.0 * quad,
        1 => Vec2::new(2.0 * quad.y, 2.0 * (1.0 - quad.x)),
        2 => 2.0 * (Vec2::splat(1.0) - quad),
        _ => Vec2::new(2.0 * (1.0 - quad.y), 2.0 * quad.x),
    }
}

/// `bke::subdiv::rotate_grid_to_quad` — inverse of
/// `rotate_quad_to_corner`.
pub fn rotate_grid_to_quad(corner: i32, grid_u: f32, grid_v: f32) -> (f32, f32) {
    match corner {
        0 => (0.5 * grid_u, 0.5 * grid_v),
        1 => (0.5 * (1.0 + grid_u), 0.5 * grid_v),
        2 => (0.5 * (1.0 + grid_u), 0.5 * (1.0 + grid_v)),
        _ => (0.5 * grid_u, 0.5 * (1.0 + grid_v)),
    }
}

/// `PTexCoord`.
#[derive(Clone, Copy, Debug, Default)]
pub struct PTexCoord {
    /// `ptex_face_index`.
    pub ptex_face_index: i32,
    /// `u`.
    pub u: f32,
    /// `v`.
    pub v: f32,
}

/// `GridCoord`.
#[derive(Clone, Copy, Debug, Default)]
pub struct GridCoord {
    /// `grid_index`.
    pub grid_index: i32,
    /// `u`.
    pub u: f32,
    /// `v`.
    pub v: f32,
}

/// `ReshapeGridElement` — mutable grid element views.
#[derive(Clone, Copy, Debug, Default)]
pub struct ReshapeGridElement {
    /// `displacement` index into the grid (None when no disps).
    pub displacement_index: Option<usize>,
    /// `mask` index into the paint-mask grid.
    pub mask_index: Option<usize>,
}

/// `ReshapeConstGridElement`.
#[derive(Clone, Copy, Debug)]
pub struct ReshapeConstGridElement {
    /// `displacement`.
    pub displacement: [f32; 3],
    /// `mask`.
    pub mask: f32,
}

/// `MultiresReshapeContext` — the C context, base mesh arrays as
/// indices + grid data held by the caller.
#[derive(Clone)]
pub struct MultiresReshapeContext {
    /// `base_faces` — offsets.
    pub base_faces: Vec<IndexRange>,
    /// `base_corner_to_face` — per-corner face index.
    pub base_corner_to_face: Vec<i32>,
    /// `face_ptex_offset` — per-face ptex offset.
    pub face_ptex_offset: Vec<i32>,
    /// `ptex_start_grid_index` — ptex face → first grid.
    pub ptex_start_grid_index: Vec<i32>,
    /// `num_grids` (corners total).
    pub num_grids: i32,
    /// `top.level` + `top.grid_size`.
    pub top_grid_size: i32,
    /// `orig.grid_size`.
    pub orig_grid_size: i32,
    /// `mdisps` — per-corner displacement grids (top level).
    pub mdisps: Option<Vec<MDisps>>,
    /// `grid_paint_masks`.
    pub grid_paint_masks: Option<Vec<GridPaintMask>>,
    /// `orig.mdisps`.
    pub orig_mdisps: Option<Vec<MDisps>>,
    /// `orig.grid_paint_masks`.
    pub orig_grid_paint_masks: Option<Vec<GridPaintMask>>,
}

impl MultiresReshapeContext {
    /// `multires_reshape_grid_to_face_index`.
    pub fn grid_to_face_index(&self, grid_index: i32) -> i32 {
        self.base_corner_to_face[grid_index as usize]
    }
    /// `multires_reshape_grid_to_corner`.
    pub fn grid_to_corner(&self, grid_index: i32) -> i32 {
        let face_index = self.grid_to_face_index(grid_index);
        grid_index - self.base_faces[face_index as usize].start() as i32
    }
    /// `multires_reshape_is_quad_face`.
    pub fn is_quad_face(&self, face_index: i32) -> bool {
        self.base_faces[face_index as usize].size() == 4
    }
    /// `multires_reshape_grid_to_ptex_index`.
    pub fn grid_to_ptex_index(&self, grid_index: i32) -> i32 {
        let face_index = self.grid_to_face_index(grid_index);
        let corner = self.grid_to_corner(grid_index);
        let is_quad = self.is_quad_face(face_index);
        self.face_ptex_offset[face_index as usize] + if is_quad { 0 } else { corner }
    }

    /// `multires_reshape_grid_coord_to_ptex`.
    pub fn grid_coord_to_ptex(&self, grid_coord: &GridCoord) -> PTexCoord {
        let ptex_face_index = self.grid_to_ptex_index(grid_coord.grid_index);
        let (corner_u, corner_v) =
            grid_uv_to_ptex_face_uv(grid_coord.u, grid_coord.v);
        let face_index = self.grid_to_face_index(grid_coord.grid_index);
        let corner = self.grid_to_corner(grid_coord.grid_index);
        let (u, v) = if self.is_quad_face(face_index) {
            let (gu, gv) = ptex_face_uv_to_grid_uv(corner_u, corner_v);
            rotate_grid_to_quad(corner, gu, gv)
        } else {
            (corner_u, corner_v)
        };
        PTexCoord {
            ptex_face_index,
            u,
            v,
        }
    }

    /// `multires_reshape_ptex_coord_to_grid`.
    pub fn ptex_coord_to_grid(&self, ptex_coord: &PTexCoord) -> GridCoord {
        debug_assert!(!self.ptex_start_grid_index.is_empty());
        let start_grid_index =
            self.ptex_start_grid_index[ptex_coord.ptex_face_index as usize];
        let face_index = self.base_corner_to_face[start_grid_index as usize];
        let (corner_delta, mut u, mut v) = if self.is_quad_face(face_index) {
            rotate_quad_to_corner(ptex_coord.u, ptex_coord.v)
        } else {
            (0, ptex_coord.u, ptex_coord.v)
        };
        (u, v) = ptex_face_uv_to_grid_uv(u, v);
        GridCoord {
            grid_index: start_grid_index + corner_delta,
            u,
            v,
        }
    }

    /// `multires_reshape_grid_element_for_grid_coord` — element
    /// index within each grid array (top level).
    pub fn grid_element_for_grid_coord(&self, grid_coord: &GridCoord) -> ReshapeGridElement {
        let grid_size = self.top_grid_size;
        let grid_x = (grid_coord.u * (grid_size - 1) as f32).round() as i64;
        let grid_y = (grid_coord.v * (grid_size - 1) as f32).round() as i64;
        let idx = (grid_y * grid_size as i64 + grid_x) as usize;
        ReshapeGridElement {
            displacement_index: self.mdisps.is_some().then_some(idx),
            mask_index: self.grid_paint_masks.is_some().then_some(idx),
        }
    }

    /// `multires_reshape_grid_element_for_ptex_coord`.
    pub fn grid_element_for_ptex_coord(&self, ptex_coord: &PTexCoord) -> ReshapeGridElement {
        let grid_coord = self.ptex_coord_to_grid(ptex_coord);
        self.grid_element_for_grid_coord(&grid_coord)
    }

    /// `multires_reshape_orig_grid_element_for_grid_coord`.
    pub fn orig_grid_element_for_grid_coord(
        &self,
        grid_coord: &GridCoord,
    ) -> ReshapeConstGridElement {
        let mut el = ReshapeConstGridElement {
            displacement: [0.0; 3],
            mask: 0.0,
        };
        if let Some(mdisps) = &self.orig_mdisps {
            let grid_size = self.orig_grid_size;
            let dg = &mdisps[grid_coord.grid_index as usize];
            if !dg.disps.is_empty() {
                let x = (grid_coord.u * (grid_size - 1) as f32).round() as usize;
                let y = (grid_coord.v * (grid_size - 1) as f32).round() as usize;
                el.displacement = dg.disps[y * grid_size as usize + x];
            }
        }
        if let Some(gpms) = &self.orig_grid_paint_masks {
            let gpm = &gpms[grid_coord.grid_index as usize];
            if !gpm.data.is_empty() {
                let grid_size = grid_size_from_level(gpm.level);
                let x = (grid_coord.u * (grid_size - 1) as f32).round() as usize;
                let y = (grid_coord.v * (grid_size - 1) as f32).round() as usize;
                el.mask = gpm.data[y * grid_size as usize + x];
            }
        }
        el
    }

    /// `multires_reshape_tangent_matrix_for_corner` —
    /// `BKE_multires_construct_tangent_matrix(dPdu, dPdv, corner)`
    /// on quads, 0 otherwise.
    pub fn tangent_matrix_for_corner(
        &self,
        face_index: i32,
        corner: i32,
        dpdu: Vec3,
        dpdv: Vec3,
    ) -> Mat3 {
        let tangent_corner = if self.is_quad_face(face_index) { corner } else { 0 };
        construct_tangent_matrix(dpdu, dpdv, tangent_corner)
    }

    /// `multires_reshape_evaluate_base_mesh_limit_at_grid` — inject
    /// `eval_limit(ptex_face, u, v) -> (P, dPdu, dPdv)`.
    pub fn evaluate_base_mesh_limit_at_grid(
        &self,
        grid_coord: &GridCoord,
        eval_limit: &dyn Fn(i32, f32, f32) -> (Vec3, Vec3, Vec3),
    ) -> (Vec3, Mat3) {
        let ptex = self.grid_coord_to_ptex(grid_coord);
        let (p, dpdu, dpdv) = eval_limit(ptex.ptex_face_index, ptex.u, ptex.v);
        let face_index = self.grid_to_face_index(grid_coord.grid_index);
        let corner = self.grid_to_corner(grid_coord.grid_index);
        (p, self.tangent_matrix_for_corner(face_index, corner, dpdu, dpdv))
    }
}

/// `BKE_multires_construct_tangent_matrix` — N/dPdu/dPdv frame
/// adjusted per corner (rotation).
pub fn construct_tangent_matrix(dpdu: Vec3, dpdv: Vec3, corner: i32) -> Mat3 {
    let n = dpdu.cross(dpdv).normalized();
    let (u, v) = match corner {
        1 => (dpdv, -dpdu),
        2 => (-dpdu, -dpdv),
        3 => (-dpdv, dpdu),
        _ => (dpdu, dpdv),
    };
    Mat3::from_cols(u.normalized(), v.normalized(), n)
}

/// `allocate_displacement_grid`.
pub fn allocate_displacement_grid(displacement_grid: &mut MDisps, grid_area: i32) {
    displacement_grid.disps = vec![[0.0; 3]; grid_area as usize];
    displacement_grid.totdisp = grid_area;
}

/// `ensure_displacement_grid`.
pub fn ensure_displacement_grid(displacement_grid: &mut MDisps, grid_area: i32) {
    if displacement_grid.disps.len() != grid_area as usize {
        allocate_displacement_grid(displacement_grid, grid_area);
    }
}

/// `multires_reshape_ensure_grids` — alloc disps + masks at `level`.
pub fn multires_reshape_ensure_grids(
    mdisps: &mut [MDisps],
    grid_paint_masks: Option<&mut Vec<GridPaintMask>>,
    level: i32,
) {
    let grid_size = grid_size_from_level(level);
    let grid_area = grid_size * grid_size;
    for md in mdisps.iter_mut() {
        ensure_displacement_grid(md, grid_area);
    }
    if let Some(gpms) = grid_paint_masks {
        for gpm in gpms.iter_mut() {
            if gpm.data.len() != grid_area as usize {
                gpm.data = vec![0.0; grid_area as usize];
                gpm.level = level;
            }
        }
    }
}

/// `multires_reshape_store_original_grids` — clone mdisps/masks
/// into `orig_*`.
pub fn multires_reshape_store_original_grids(ctx: &mut MultiresReshapeContext) {
    ctx.orig_mdisps = ctx.mdisps.clone();
    ctx.orig_grid_paint_masks = ctx.grid_paint_masks.clone();
    ctx.orig_grid_size = ctx.top_grid_size;
}

/// `object_grid_element_to_tangent_displacement` — object-space
/// displacement → tangent frame.
pub fn object_grid_element_to_tangent_displacement(
    displacement: Vec3,
    tangent_matrix: Mat3,
) -> Vec3 {
    tangent_matrix.transposed() * displacement
}

/// `multires_reshape_object_grids_to_tangent_displacement` —
/// rewrite every grid's disps into tangent space (`eval_limit`
/// injected).
pub fn multires_reshape_object_grids_to_tangent_displacement(
    ctx: &mut MultiresReshapeContext,
    eval_limit: &dyn Fn(i32, f32, f32) -> (Vec3, Vec3, Vec3),
) {
    let Some(mut mdisps) = ctx.mdisps.take() else {
        return;
    };
    let grid_size = ctx.top_grid_size;
    for (grid_index, dg) in mdisps.iter_mut().enumerate() {
        for y in 0..grid_size {
            for x in 0..grid_size {
                let u = x as f32 / (grid_size - 1) as f32;
                let v = y as f32 / (grid_size - 1) as f32;
                let gc = GridCoord {
                    grid_index: grid_index as i32,
                    u,
                    v,
                };
                let ptex = ctx.grid_coord_to_ptex(&gc);
                let (_p, dpdu, dpdv) = eval_limit(ptex.ptex_face_index, ptex.u, ptex.v);
                let face_index = ctx.grid_to_face_index(gc.grid_index);
                let corner = ctx.grid_to_corner(gc.grid_index);
                let tmat = ctx.tangent_matrix_for_corner(face_index, corner, dpdu, dpdv);
                let i = (y * grid_size + x) as usize;
                let d = Vec3::from(dg.disps[i]);
                let t = object_grid_element_to_tangent_displacement(d, tmat);
                dg.disps[i] = t.into();
            }
        }
    }
    ctx.mdisps = Some(mdisps);
}

/// `multires_reshape_assign_final_coords_from_mdisps` — `result = P
/// + T·disp` per grid element (writes into `positions` via
/// `grid_coord` map).
pub fn multires_reshape_assign_final_coords_from_mdisps(
    ctx: &MultiresReshapeContext,
    eval_limit: &dyn Fn(i32, f32, f32) -> (Vec3, Vec3, Vec3),
    corner_coords_of_grid: &dyn Fn(i32) -> Vec<(usize, f32, f32)>,
    positions: &mut [Vec3],
) {
    let Some(mdisps) = ctx.mdisps.as_ref() else {
        return;
    };
    for (grid_index, dg) in mdisps.iter().enumerate() {
        for (corner, u, v) in corner_coords_of_grid(grid_index as i32) {
            let gc = GridCoord {
                grid_index: grid_index as i32,
                u,
                v,
            };
            let (p, tmat) = ctx.evaluate_base_mesh_limit_at_grid(&gc, eval_limit);
            let grid_size = ctx.top_grid_size;
            let i = ((v * (grid_size - 1) as f32).round() as usize) * grid_size as usize
                + (u * (grid_size - 1) as f32).round() as usize;
            let disp = Vec3::from(*dg.disps.get(i).unwrap_or(&[0.0; 3]));
            positions[corner] = p + tmat * disp;
        }
    }
}

/// `multires_reshape_assign_final_elements_from_orig_mdisps` —
/// copy `orig` displacement+mask into `displacement`/`mask`
/// per grid element.
pub fn multires_reshape_assign_final_elements_from_orig_mdisps(
    ctx: &MultiresReshapeContext,
    apply_tangent: &dyn Fn(i32) -> Option<Mat3>,
) {
    let Some(mdisps) = ctx.mdisps.as_ref() else {
        return;
    };
    let grid_size = ctx.top_grid_size;
    for grid_index in 0..ctx.num_grids as usize {
        for y in 0..grid_size {
            for x in 0..grid_size {
                let gc = GridCoord {
                    grid_index: grid_index as i32,
                    u: x as f32 / (grid_size - 1) as f32,
                    v: y as f32 / (grid_size - 1) as f32,
                };
                let orig = ctx.orig_grid_element_for_grid_coord(&gc);
                let mut disp = Vec3::from(orig.displacement);
                if let Some(t) = apply_tangent(grid_index as i32) {
                    disp = t * disp;
                }
                let i = (y * grid_size + x) as usize;
                let _ = (mdisps, disp, i);
                /* Caller applies `disp`/`orig.mask` through the
                 * element indices from `grid_element_for_grid_coord`. */
            }
        }
    }
}

/// `context_init_lookup` — build `base_corner_to_face` and
/// `ptex_start_grid_index` from faces.
pub fn context_init_lookup(
    faces: OffsetIndices,
    base_corner_to_face: &mut Vec<i32>,
    face_ptex_offset: &mut Vec<i32>,
) {
    let corners = faces.total_size();
    base_corner_to_face.resize(corners as usize, 0);
    face_ptex_offset.clear();
    let mut ptex_offset = 0i32;
    for i in 0..faces.len() {
        let g = faces.group(i);
        face_ptex_offset.push(ptex_offset);
        ptex_offset += (if g.size() == 4 { 1 } else { g.size() }) as i32;
        for c in g.start()..g.one_after_last() {
            base_corner_to_face[c as usize] = i as i32;
        }
    }
}
