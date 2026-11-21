//! Ports of `intern/multires_reshape_{ccg,subdivide,vertcos}.cc` +
//! `multires_versioning.cc` — multires reshape helpers.
//!
//! CCG/ReshapeContext internals are `usize` handles (PORT_PENDING the
//! sculpt-subdiv bridge); the object-space grid fill + vertcos sync loops
//! are ported verbatim.

use blenders_bli::offset_indices::OffsetIndices;
use blenders_math::vector::Vec3;

/// `MDisps` (DNA_meshdata_types.h) — one corner's displacement grid.
pub struct MDisps {
    /// `disps` — `totdisp` vec3 values.
    pub disps: Vec<Vec3>,
    /// `totdisp` — element count (corner_num*grid_size²).
    pub totdisp: i32,
    pub hidden: bool,
    /// `lvl`.
    pub lvl: i16,
}

/// `mid_v3_v3v3` inline.
fn mid_v3_v3v3(a: Vec3, b: Vec3) -> Vec3 {
    (a + b) * 0.5
}

/// `multires_subdivide_create_object_space_linear_grids` —
/// fill each face's 4-element linear MDISPS grid from face center + edge
/// midpoints.
pub fn multires_subdivide_create_object_space_linear_grids(
    faces: &OffsetIndices,
    corner_verts: &[i32],
    positions: &[Vec3],
    mdisps: &mut [MDisps],
) {
    for p in 0..faces.len() {
        let face = faces.group(p);
        let face_center = {
            let mut sum = Vec3::ZERO;
            for i in face.iter() {
                sum += positions[corner_verts[i as usize] as usize];
            }
            sum / face.size() as f32
        };
        for l in 0..face.size() {
            let loop_index = (face.start() + l) as usize;
            let prev_loop_index = if l - 1 >= 0 {
                loop_index - 1
            } else {
                (loop_index as i64 + face.size() - 1) as usize
            };
            let next_loop_index = if l + 1 < face.size() {
                loop_index + 1
            } else {
                face.start() as usize
            };
            let vert = corner_verts[loop_index] as usize;
            let vert_next = corner_verts[next_loop_index] as usize;
            let vert_prev = corner_verts[prev_loop_index] as usize;
            let m = &mut mdisps[loop_index];
            m.totdisp = 4;
            if m.disps.len() < 4 {
                m.disps.resize(4, Vec3::ZERO);
            }
            m.disps[0] = face_center;
            m.disps[1] = mid_v3_v3v3(positions[vert], positions[vert_next]);
            m.disps[2] = mid_v3_v3v3(positions[vert], positions[vert_prev]);
            m.disps[3] = positions[vert];
        }
    }
}

/// `multires_reshape_ensure_grids` — creates/resizes MDISPS layers
/// (PORT_PENDING `CustomData_add_layer`; the caller passes the slice).

/// `multires_subdivide_create_tangent_displacement_linear_grids` — adds one
/// level: ensure grids → fill linear → convert to tangent displacement
/// (`multires_reshape_object_grids_to_tangent_displacement` is the
/// reshape-context conversion — PORT_PENDING ccg context).
pub fn multires_subdivide_create_tangent_displacement_linear_grids(
    faces: &OffsetIndices,
    corner_verts: &[i32],
    positions: &[Vec3],
    mdisps: &mut [MDisps],
    new_top_level: i32,
) {
    if new_top_level == 1 {
        /* No MDISPS yet: level-1 grids = face centers + edge midpoints. */
        multires_subdivide_create_object_space_linear_grids(
            faces, corner_verts, positions, mdisps,
        );
    }
    /* `multires_set_tot_level` + `multires_reshape_object_grids_to_tangent_displacement`
     * — PORT_PENDING (reshape context / ccg). */
}

// --- intern/multires_reshape_ccg.cc --------------------------------------

/// `ReshapeGridElement` — displacement+mask write targets for one grid
/// corner (raw pointers in C → indices into caller buffers).
pub struct ReshapeGridElement {
    /// Index into the target `positions`/`displacement` buffer (usize::MAX = none).
    pub displacement: usize,
    /// Index into the mask buffer (usize::MAX = none).
    pub mask: usize,
}

/// `GridCoord` (multires_reshape.hh).
#[derive(Clone, Copy)]
pub struct GridCoord {
    pub grid_index: i32,
    pub u: f32,
    pub v: f32,
}

/// `bke::ccg::grid_xy_to_vert` — CCG vertex index within subdiv-level grid.
/// `reshape_level_key` = (level, grid_size); `(x,y)` grid coords.
pub fn grid_xy_to_vert(grid_index: i32, grid_size: i32, x: i32, y: i32) -> usize {
    (grid_index * grid_size * grid_size + y * grid_size + x) as usize
}

/// `multires_reshape_copy_vertcoords_to_ccg`-style copy loop from
/// `multires_reshape_ccg.cc` — each grid element writes
/// `positions[vert]`/`masks[vert]` into its targets.
/// `element_for` resolves `(grid_index, u, v)` → `ReshapeGridElement`
/// (PORT_PENDING the ccg mapping).
pub fn multires_reshape_write_ccg(
    grids_num: usize,
    reshape_grid_size: i32,
    positions: &[Vec3],
    masks: &[f32],
    displacements: &mut [Vec3],
    masks_out: &mut [f32],
    element_for: &mut dyn FnMut(i32, f32, f32) -> ReshapeGridElement,
) -> bool {
    let reshape_grid_size_1_inv = 1.0 / (reshape_grid_size - 1) as f32;
    for grid_index in 0..grids_num as i32 {
        for y in 0..reshape_grid_size {
            let v = y as f32 * reshape_grid_size_1_inv;
            for x in 0..reshape_grid_size {
                let u = x as f32 * reshape_grid_size_1_inv;
                let vert = grid_xy_to_vert(grid_index, reshape_grid_size, x, y);
                let e = element_for(grid_index, u, v);
                if e.displacement != usize::MAX {
                    displacements[e.displacement] = positions[vert];
                }
                if !masks.is_empty() && e.mask != usize::MAX {
                    masks_out[e.mask] = masks[vert];
                }
            }
        }
    }
    true
}

// --- intern/multires_versioning.cc ---------------------------------------

/// `OPENSUBDIV_SHARPNESS_INFINITE`.
pub const OPENSUBDIV_SHARPNESS_INFINITE: f32 = 10.0;

/// `subdiv_for_simple_to_catmull_clark` — builds an infinitely-sharp
/// evaluator emulating old simple subdivision. PORT_PENDING: needs the
/// OpenSubdiv converter + evaluator (foreign dep); the sharpness arrays and
/// sequence are documented.
///
/// Steps: `converter_init_for_mesh` → fill `edge_sharpness`/`vert_sharpness`
/// with `OPENSUBDIV_SHARPNESS_INFINITE` → `subdiv::new_from_converter` →
/// `eval_begin_from_mesh(CPU)`; on failure `subdiv::free` + NIL.
pub fn subdiv_for_simple_to_catmull_clark() -> usize {
    blenders_dna::id::NIL
}

/// `multires_do_versions_simple_to_catmull_clark` — 2.90→3.0 data
/// conversion: store object-space displacement of simple limit, then rebuild
/// tangent displacement vs Catmull-Clark (both reshape-context calls are
/// PORT_PENDING; the sequence is verbatim).
pub fn multires_do_versions_simple_to_catmull_clark(_object: usize, _mmd: usize) {
    // WITH_OPENSUBDIV == false branch: UNUSED_VARS(object, mmd).
    // PORT_PENDING the OpenSubdiv path.
}

// --- intern/multires_reshape_vertcos.cc -----------------------------------

/// `multires_reshape_ptex_coord_to_grid` dependency — injected since
/// `multires_reshape.cc` ptex mapping is PORT_PENDING.
/// Signature: `(ptex_face_index, u, v) -> GridCoord`.
pub type PtexToGridFn<'a> = &'a mut dyn FnMut(i32, f32, f32) -> GridCoord;

/// `assign_ptex_coord_from_vert` — assigns `positions[subdiv_vert_index]`
/// to all grid elements the ptex corner maps onto (corner mirroring when
/// `u`/`v` == 0). `element_for(grid_coord)` resolves the write targets.
pub fn multires_reshape_assign_vertcos_from_ptex(
    positions: &[Vec3],
    ptex: &crate::subdiv_deform::PtexCoord,
    subdiv_vert_index: usize,
    num_corners: i32,
    start_grid_index: i32,
    ptex_to_grid: PtexToGridFn,
    element_for: &mut dyn FnMut(&GridCoord) -> ReshapeGridElement,
    displacements: &mut [Vec3],
) {
    let grid_coord = ptex_to_grid(ptex.ptex_face_index, ptex.uv.x, ptex.uv.y);
    let corner = grid_coord.grid_index - start_grid_index;
    if grid_coord.u == 0.0 && grid_coord.v == 0.0 {
        for current_corner in 0..num_corners {
            let cc = GridCoord {
                grid_index: start_grid_index + current_corner,
                u: grid_coord.u,
                v: grid_coord.v,
            };
            let e = element_for(&cc);
            if e.displacement != usize::MAX {
                displacements[e.displacement] = positions[subdiv_vert_index];
            }
        }
        return;
    }
    let e = element_for(&grid_coord);
    if e.displacement != usize::MAX {
        displacements[e.displacement] = positions[subdiv_vert_index];
    }
    if grid_coord.u == 0.0 {
        let prev = GridCoord {
            grid_index: start_grid_index + ((corner + num_corners - 1) % num_corners),
            u: grid_coord.v,
            v: 0.0,
        };
        let e = element_for(&prev);
        if e.displacement != usize::MAX {
            displacements[e.displacement] = positions[subdiv_vert_index];
        }
    }
    if grid_coord.v == 0.0 {
        let next = GridCoord {
            grid_index: start_grid_index + ((corner + 1) % num_corners),
            u: 0.0,
            v: grid_coord.u,
        };
        let e = element_for(&next);
        if e.displacement != usize::MAX {
            displacements[e.displacement] = positions[subdiv_vert_index];
        }
    }
}
