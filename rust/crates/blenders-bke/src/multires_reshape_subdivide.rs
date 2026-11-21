//! Port of `intern/multires_reshape_subdivide.cc` —
//! multires subdivide → object-space linear displacement grids.
//!
//! `CustomData`/`MultiresReshapeContext`/`multires_set_tot_level`
//! aren't ported — the driver fn is a verbatim skeleton with those
//! steps injected. The linear-grid fill math is ported verbatim.

use blenders_math::vector::Vec3;

/// `MDisps` — per-corner displacement grid.
#[derive(Clone, Default)]
pub struct MDisp {
    /// `totdisp` — count of used `disps` slots.
    pub totdisp: i32,
    /// `disps[4]` — [face center, next mid, prev mid, vert].
    pub disps: [Vec3; 4],
}

/// `face_center_calc` — mean of face corner positions.
fn face_center(positions: &[Vec3], corner_verts: &[i32], face: std::ops::Range<usize>) -> Vec3 {
    let mut c = Vec3::ZERO;
    let mut n = 0f32;
    for l in face {
        let v = positions[corner_verts[l] as usize];
        c = Vec3::new(c.x + v.x, c.y + v.y, c.z + v.z);
        n += 1.0;
    }
    if n > 0.0 { Vec3::new(c.x / n, c.y / n, c.z / n) } else { c }
}

/// `multires_subdivide_create_object_space_linear_grids` —
/// verbatim loop: `disps` = [face center, mid(vert,vert_next),
/// mid(vert,vert_prev), vert]; `totdisp=4`.
pub fn multires_subdivide_create_object_space_linear_grids(
    positions: &[Vec3],
    faces: &[std::ops::Range<i64>],
    corner_verts: &[i32],
) -> Vec<MDisp> {
    let mut mdisps = vec![MDisp::default(); corner_verts.len()];
    for (p, face) in faces.iter().enumerate() {
        let _ = p;
        let start = face.start as usize;
        let end = face.end as usize;
        let size = end - start;
        let center = face_center(positions, corner_verts, start..end);
        for l in 0..size {
            let loop_index = start + l;
            /* C index arithmetic on the flattened corner arrays:
             * prev wraps to face end, next wraps to face start. */
            let prev_loop_index = if l > 0 { loop_index - 1 } else { loop_index + size - 1 };
            let next_loop_index = if l + 1 < size { loop_index + 1 } else { start };
            let vert = corner_verts[loop_index] as usize;
            let vert_next = corner_verts[next_loop_index] as usize;
            let vert_prev = corner_verts[prev_loop_index] as usize;
            let m = &mut mdisps[loop_index];
            m.totdisp = 4;
            m.disps[0] = center;
            m.disps[1] = (positions[vert] + positions[vert_next]) * 0.5; /* mid_v3_v3v3 */
            m.disps[2] = (positions[vert] + positions[vert_prev]) * 0.5;
            m.disps[3] = positions[vert];
        }
    }
    mdisps
}

/// `multires_subdivide_create_tangent_displacement_linear_grids` —
/// the driver. All non-mesh steps injected (`sculpt_rebuild`,
/// `has_mdisps`/`add_mdisps`, `set_tot_level`, `reshape_to_tangent`).
/// Returns the linear grids when `new_top_level == 1`.
pub fn multires_subdivide_create_tangent_displacement_linear_grids(
    totlvl: i32,
    sculpt_rebuild: &mut dyn FnMut(),
    has_mdisps: &dyn Fn() -> bool,
    add_mdisps: &mut dyn FnMut(),
    ensure_grids: &mut dyn FnMut(i32),
    linear_grids: &mut dyn FnMut() -> Vec<MDisp>,
    set_tot_level: &mut dyn FnMut(i32),
    reshape_to_tangent: &mut dyn FnMut(i32) -> bool,
) -> Option<Vec<MDisp>> {
    sculpt_rebuild();
    let new_top_level = totlvl + 1;
    if !has_mdisps() {
        add_mdisps();
    }
    let mut grids = None;
    if new_top_level == 1 {
        ensure_grids(1);
        grids = Some(linear_grids());
    }
    set_tot_level(new_top_level);
    if !reshape_to_tangent(new_top_level) {
        return grids;
    }
    grids
}
