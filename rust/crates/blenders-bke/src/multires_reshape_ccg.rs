//! Port of `intern/multires_reshape_ccg.cc` —
//! `multires_reshape_assign_final_coords_from_ccg`.

use blenders_math::vector::Vec3;

use crate::multires_reshape::grid_xy_to_vert;

/// `ReshapeGridElement` write targets — pointers in C, indices into the
/// caller's displacement/mask grids here (`usize::MAX` = null).
#[derive(Clone, Copy)]
pub struct ReshapeGridElementMut {
    pub displacement: usize,
    pub mask: usize,
}

/// `multires_reshape_assign_final_coords_from_ccg` — CCG grids →
/// reshape grids. `grid_element_for` resolves
/// `multires_reshape_grid_element_for_grid_coord` (reshape context,
/// PORT_PENDING); writes go into `displacements`/`masks_out`.
#[allow(clippy::too_many_arguments)]
pub fn multires_reshape_assign_final_coords_from_ccg(
    reshape_level: i32,
    reshape_grid_size: i32,
    positions: &[Vec3],
    masks: &[f32],
    grids_num: i32,
    grid_element_for: &mut dyn FnMut(i32, f32, f32) -> ReshapeGridElementMut,
    displacements: &mut [Vec3],
    masks_out: &mut [f32],
) -> bool {
    let reshape_grid_size_1_inv = 1.0 / (reshape_grid_size - 1) as f32;
    /* `CCGKey reshape_level_key` — level → grid stride via
     * `grid_size_from_level` (bke::ccg::grid_xy_to_vert). */
    let level_grid_size = crate::subdiv_ccg_mask::grid_size_from_level(reshape_level) as i32;
    /* threading::parallel_for(grain 2048/(gs²)) — serial here. */
    for grid_index in 0..grids_num {
        for y in 0..reshape_grid_size {
            let v = y as f32 * reshape_grid_size_1_inv;
            for x in 0..reshape_grid_size {
                let u = x as f32 * reshape_grid_size_1_inv;
                let vert = grid_xy_to_vert(grid_index, level_grid_size, x, y);
                let grid_element = grid_element_for(grid_index, u, v);
                debug_assert!(grid_element.displacement != usize::MAX);
                if grid_element.displacement != usize::MAX {
                    displacements[grid_element.displacement] = positions[vert];
                }
                if !masks.is_empty() && grid_element.mask != usize::MAX {
                    masks_out[grid_element.mask] = masks[vert];
                }
            }
        }
    }
    true
}
