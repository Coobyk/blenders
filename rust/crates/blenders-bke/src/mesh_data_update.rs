//! Port of `intern/mesh_data_update.cc` (portable parts) —
//! `mesh_get_mapped_verts_coords` and `mesh_init_origspace`.
//! `PORT_PENDING`: `mesh_calc_modifiers`/`editbmesh_calc_modifiers`
//! (depsgraph + modifier stack), `mesh_build_data`,
//! `object_get_datamask`, `mesh_data_update` (depsgraph drivers),
//! orco mesh creation, `set_rest_position`, edit-hints.

use blenders_math::vector::{Vec2, Vec3};

/// `MappedUserData` gather — `foreach_mapped_vert` supplies
/// `(index, co)` pairs (deformed path only; otherwise copy coords).
pub fn mesh_get_mapped_verts_coords(
    deformed_only: bool,
    wrapper_vert_coords: &[Vec3],
    foreach_mapped_vert: &dyn Fn(&mut dyn FnMut(usize, Vec3)),
    r_cos: &mut [Vec3],
) {
    if !deformed_only {
        for c in r_cos.iter_mut() {
            *c = Vec3::ZERO;
        }
        let mut visited = vec![false; r_cos.len()];
        foreach_mapped_vert(&mut |index, co| {
            if !visited[index] {
                r_cos[index] = co;
                visited[index] = true;
            }
        });
    } else {
        r_cos.copy_from_slice(wrapper_vert_coords);
    }
}

/// `OrigSpaceLoop` — `uv` per corner.
#[derive(Clone, Copy, Default)]
pub struct OrigSpaceLoop {
    /// `uv`.
    pub uv: Vec2,
}

/// `mesh_init_origspace` — default 0..1 UVs for tris/quads, otherwise
/// project the face onto its dominant axis and normalize to 0..1.
pub fn mesh_init_origspace(
    positions: &[Vec3],
    face_offsets: &[usize],
    corner_verts: &[i32],
    lof_array: &mut [OrigSpaceLoop],
) {
    let default_osf: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let mut vcos_2d: Vec<Vec2> = Vec::with_capacity(64);
    for i in 0..face_offsets.len() - 1 {
        let face_start = face_offsets[i];
        let face_end = face_offsets[i + 1];
        let face_len = face_end - face_start;
        if face_len == 3 || face_len == 4 {
            for j in 0..face_len {
                lof_array[face_start + j].uv = Vec2::new(default_osf[j][0], default_osf[j][1]);
            }
        } else {
            let mut min = [f32::MAX; 2];
            let mut max = [f32::MIN; 2];
            let face_verts = &corner_verts[face_start..face_end];
            let p_nor = crate::mesh_iterators::face_normal_calc(positions, face_verts);
            let mat = blenders_math::geom::axis_dominant_to_m3(p_nor);
            vcos_2d.clear();
            for &cv in face_verts {
                let co = mat * positions[cv as usize];
                vcos_2d.push(Vec2::new(co.x, co.y));
                if co.x > max[0] {
                    max[0] = co.x;
                } else if co.x < min[0] {
                    min[0] = co.x;
                }
                if co.y > max[1] {
                    max[1] = co.y;
                } else if co.y < min[1] {
                    min[1] = co.y;
                }
            }
            let translate = [-min[0], -min[1]];
            let mut scale = [max[0] - min[0], max[1] - min[1]];
            if scale[0] == 0.0 {
                scale[0] = 1e-9;
            }
            if scale[1] == 0.0 {
                scale[1] = 1e-9;
            }
            scale = [1.0 / scale[0], 1.0 / scale[1]];
            for j in 0..face_len {
                let v = vcos_2d[j];
                lof_array[face_start + j].uv = Vec2::new(
                    (v.x + translate[0]) * scale[0],
                    (v.y + translate[1]) * scale[1],
                );
            }
        }
    }
}
