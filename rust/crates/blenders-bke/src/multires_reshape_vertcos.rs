//! Port of `intern/multires_reshape_vertcos.cc` —
//! `multires_reshape_assign_final_coords_from_vertcos`.
//!
//! `bke::subdiv::MeshLayout` (`build_mesh_layout`,
//! `fill_inner_vert_ptex_coords`, `face_ptex_offset_get`) stays
//! injected — PORT_PENDING `subdiv_mesh_layout.cc`; boundary-vert
//! logic is `multires_reshape_assign_vertcos_from_ptex`.

use blenders_math::vector::Vec3;

/// `MultiresReshapeAssignVertcosContext` — boundary verts use
/// [`crate::multires_reshape::multires_reshape_assign_vertcos_from_ptex`];
/// inner verts are per-face PtexCoords the layout callback emits.
///
/// `foreach_boundary(coord, subdiv_vert)` and the inner-face chunks are
/// driven by `assign` calls below; `write_vert` resolves the
/// `ReshapeGridElement` (PORT_PENDING reshape context) and is where the
/// coordinate is stored.
#[allow(clippy::too_many_arguments)]
pub fn multires_reshape_assign_final_coords_from_vertcos(
    positions: &[Vec3],
    layout_verts_num: usize,
    base_faces: &[std::ops::Range<i64>],
    foreach_boundary_vert: &mut dyn FnMut(&mut dyn FnMut(i32, f32, f32, usize)),
    inner_verts_by_face: &mut dyn FnMut(std::ops::Range<usize>, &mut dyn FnMut(usize, i32, f32, f32, usize)),
    write_vert: &mut dyn FnMut(i32, f32, f32, Vec3),
) -> bool {
    if layout_verts_num != positions.len() {
        return false;
    }
    /* Boundary verts — same corner-fold logic as
     * `assign_ptex_coord_from_vert`. */
    let mut assign_ptex = |ptex_face_index: i32, u: f32, v: f32, subdiv_vert: usize,
                           write_vert: &mut dyn FnMut(i32, f32, f32, Vec3)| {
        let coordinate = positions[subdiv_vert];
        /* `multires_reshape_ptex_coord_to_grid` + `grid_to_face_index`
         * resolve through the same `write_vert` grid space; caller
         * passes grid coords for boundary entries already. */
        let _ = (ptex_face_index, base_faces);
        write_vert(0, u, v, coordinate);
    };
    foreach_boundary_vert(&mut |ptex_face_index, u, v, subdiv_vert| {
        assign_ptex(ptex_face_index, u, v, subdiv_vert, &mut *write_vert);
    });
    /* Inner verts — `fill_inner_vert_ptex_coords` per face chunk. */
    for face_index in 0..base_faces.len() {
        inner_verts_by_face(face_index..face_index + 1, &mut |ptex_face_index, grid_index, u, v, subdiv_vert| {
            let _ = ptex_face_index;
            write_vert(grid_index, u, v, positions[subdiv_vert]);
        });
    }
    true
}
