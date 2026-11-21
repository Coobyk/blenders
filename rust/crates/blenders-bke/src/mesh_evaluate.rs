//! Port of `intern/mesh_evaluate.cc` — mesh evaluation helpers:
//! face center/area/angles, surface/volume centroids, `calc_volume`,
//! `mdisp_flip`, hide/select flush (`mesh_*_hide_from_vert`,
//! `mesh_*_flush` — attribute storage injected) and
//! `BKE_mesh_calc_relative_deform`.

use blenders_bli::offset_indices::OffsetIndices;
use blenders_math::{geom, Vec3};

use crate::mesh_iterators::face_normal_calc;

/// `face_center_calc_ngon`.
pub fn face_center_calc_ngon(vert_positions: &[Vec3], face_verts: &[i32]) -> Vec3 {
    let w = 1.0 / face_verts.len() as f32;
    let mut center = Vec3::ZERO;
    for &v in face_verts {
        center += vert_positions[v as usize] * w;
    }
    center
}

/// `face_center_calc`.
pub fn face_center_calc(vert_positions: &[Vec3], face_verts: &[i32]) -> Vec3 {
    match face_verts.len() {
        3 => {
            (vert_positions[face_verts[0] as usize]
                + vert_positions[face_verts[1] as usize]
                + vert_positions[face_verts[2] as usize])
                / 3.0
        }
        4 => {
            (vert_positions[face_verts[0] as usize]
                + vert_positions[face_verts[1] as usize]
                + vert_positions[face_verts[2] as usize]
                + vert_positions[face_verts[3] as usize])
                / 4.0
        }
        _ => face_center_calc_ngon(vert_positions, face_verts),
    }
}

/// `face_area_calc`.
pub fn face_area_calc(vert_positions: &[Vec3], face_verts: &[i32]) -> f32 {
    if face_verts.len() == 3 {
        return geom::area_tri(
            vert_positions[face_verts[0] as usize],
            vert_positions[face_verts[1] as usize],
            vert_positions[face_verts[2] as usize],
        );
    }
    let coords: Vec<Vec3> = face_verts
        .iter()
        .map(|&v| vert_positions[v as usize])
        .collect();
    geom::area_poly(&coords)
}

/// `BKE_mesh_calc_area` — total face area.
pub fn bke_mesh_calc_area(
    positions: &[Vec3],
    faces: OffsetIndices,
    corner_verts: &[i32],
) -> f32 {
    let mut total_area = 0.0;
    for i in 0..faces.len() {
        let g = faces.group(i);
        total_area += face_area_calc(
            positions,
            &corner_verts[g.start() as usize..g.one_after_last() as usize],
        );
    }
    total_area
}

/// `mesh_calc_face_volume_centroid_with_reference_center` —
/// signed-volume-weighted centroid accumulator (coords relative to
/// `reference_center`).
pub fn mesh_calc_face_volume_centroid_with_reference_center(
    positions: &[Vec3],
    face_verts: &[i32],
    reference_center: Vec3,
) -> (Vec3, f32) {
    let mut r_cent = Vec3::ZERO;
    let mut total_volume = 0.0f32;
    let v_pivot = positions[face_verts[0] as usize] - reference_center;
    let mut v_step1 = positions[face_verts[1] as usize] - reference_center;
    for &fv in &face_verts[2..] {
        let v_step2 = positions[fv as usize] - reference_center;
        let tetra_volume = geom::volume_tri_tetrahedron_signed_6x(v_pivot, v_step1, v_step2);
        total_volume += tetra_volume;
        r_cent += tetra_volume * (v_pivot + v_step1 + v_step2);
        v_step1 = v_step2;
    }
    (r_cent, total_volume)
}

/// `face_area_centroid_calc` — `(centroid, signed_total_area)`.
pub fn face_area_centroid_calc(positions: &[Vec3], face_verts: &[i32]) -> (Vec3, f32) {
    let normal = face_normal_calc(positions, face_verts);

    let v1 = positions[face_verts[0] as usize];
    let mut v2 = positions[face_verts[1] as usize];
    let mut r_cent = Vec3::ZERO;
    let mut total_area = 0.0f32;

    for &fv in &face_verts[2..] {
        let v3 = positions[fv as usize];
        let tri_area = geom::area_tri_signed(v1, v2, v3, normal);
        total_area += tri_area;
        r_cent += tri_area * (v1 + v2 + v3) / 3.0;
        v2 = v3;
    }

    r_cent *= 1.0 / total_area;
    (r_cent, total_area)
}

/// `face_angles_calc` — per-corner interior angles.
pub fn face_angles_calc(vert_positions: &[Vec3], face_verts: &[i32], angles: &mut [f32]) {
    let n = face_verts.len();
    let mut i_this = n - 1;
    let mut i_next = 0;

    let mut nor_prev = (vert_positions[face_verts[i_this - 1] as usize]
        - vert_positions[face_verts[i_this] as usize])
        .normalized();

    while i_next < n {
        let nor_next = (vert_positions[face_verts[i_this] as usize]
            - vert_positions[face_verts[i_next] as usize])
            .normalized();
        angles[i_this] = geom::angle_normalized_v3v3(nor_prev, nor_next);

        nor_prev = nor_next;
        i_this = i_next;
        i_next += 1;
    }
}

/// `BKE_mesh_center_median` — mean of all vert positions.
/// Returns false when the mesh has no verts.
pub fn bke_mesh_center_median(positions: &[Vec3]) -> Option<Vec3> {
    if positions.is_empty() {
        return None;
    }
    let mut center = Vec3::ZERO;
    for &p in positions {
        center += p;
    }
    Some(center / positions.len() as f32)
}

/// `BKE_mesh_center_median_from_faces` — mean of face corner
/// positions.
pub fn bke_mesh_center_median_from_faces(
    positions: &[Vec3],
    faces: OffsetIndices,
    corner_verts: &[i32],
) -> Option<Vec3> {
    let mut cent = Vec3::ZERO;
    let mut tot = 0usize;
    for i in 0..faces.len() {
        let g = faces.group(i);
        for &v in &corner_verts[g.start() as usize..g.one_after_last() as usize] {
            cent += positions[v as usize];
        }
        tot += g.size() as usize;
    }
    if faces.len() == 0 {
        return None;
    }
    Some(cent / tot as f32)
}

/// `BKE_mesh_center_of_surface` — area-weighted face centroid.
pub fn bke_mesh_center_of_surface(
    positions: &[Vec3],
    faces: OffsetIndices,
    corner_verts: &[i32],
) -> Option<Vec3> {
    let mut cent = Vec3::ZERO;
    let mut total_area = 0.0f32;
    for i in 0..faces.len() {
        let g = faces.group(i);
        let (face_cent, face_area) = face_area_centroid_calc(
            positions,
            &corner_verts[g.start() as usize..g.one_after_last() as usize],
        );
        cent += face_cent * face_area;
        total_area += face_area;
    }
    if faces.len() != 0 {
        cent *= 1.0 / total_area;
    }
    if !cent.is_finite() {
        return bke_mesh_center_median(positions);
    }
    if faces.len() == 0 {
        None
    } else {
        Some(cent)
    }
}

/// `BKE_mesh_center_of_volume` — volume-weighted centroid.
pub fn bke_mesh_center_of_volume(
    positions: &[Vec3],
    faces: OffsetIndices,
    corner_verts: &[i32],
) -> Option<Vec3> {
    let init_cent = bke_mesh_center_median_from_faces(positions, faces, corner_verts);
    let mut cent = Vec3::ZERO;
    let mut total_volume = 0.0f32;

    for i in 0..faces.len() {
        let g = faces.group(i);
        let (face_cent, face_volume) = mesh_calc_face_volume_centroid_with_reference_center(
            positions,
            &corner_verts[g.start() as usize..g.one_after_last() as usize],
            init_cent.unwrap_or(Vec3::ZERO),
        );
        cent += face_cent;
        total_volume += face_volume;
    }
    if total_volume != 0.0 {
        cent *= 0.25 / total_volume;
    }
    if !cent.is_finite() {
        return init_cent;
    }
    let cent = cent + init_cent.unwrap_or(Vec3::ZERO);
    if faces.len() == 0 {
        None
    } else {
        Some(cent)
    }
}

/// `mesh_calc_center_centroid_ex` — tri-area-weighted vert centroid.
fn mesh_calc_center_centroid_ex(
    positions: &[Vec3],
    corner_tris: &[[i32; 3]],
    corner_verts: &[i32],
) -> Option<Vec3> {
    if corner_tris.is_empty() {
        return None;
    }
    let mut r_center = Vec3::ZERO;
    let mut totweight = 0.0f32;
    for tri in corner_tris {
        let v1 = positions[corner_verts[tri[0] as usize] as usize];
        let v2 = positions[corner_verts[tri[1] as usize] as usize];
        let v3 = positions[corner_verts[tri[2] as usize] as usize];
        let area = geom::area_tri(v1, v2, v3);
        r_center += v1 * area + v2 * area + v3 * area;
        totweight += area;
    }
    if totweight == 0.0 {
        return None;
    }
    Some(r_center / (3.0 * totweight))
}

/// `BKE_mesh_calc_volume` — signed tetra volume about the tri
/// centroid (+ `r_center`). Returns `(volume, center)`.
pub fn bke_mesh_calc_volume(
    vert_positions: &[Vec3],
    corner_tris: &[[i32; 3]],
    corner_verts: &[i32],
) -> (Option<f32>, Option<Vec3>) {
    if corner_tris.is_empty() {
        return (None, None);
    }
    let Some(center) = mesh_calc_center_centroid_ex(vert_positions, corner_tris, corner_verts)
    else {
        return (None, None);
    };

    let mut totvol = 0.0f32;
    let mut r_center = Vec3::ZERO;
    for tri in corner_tris {
        let v1 = vert_positions[corner_verts[tri[0] as usize] as usize];
        let v2 = vert_positions[corner_verts[tri[1] as usize] as usize];
        let v3 = vert_positions[corner_verts[tri[2] as usize] as usize];
        let vol = geom::volume_tetrahedron_signed(center, v1, v2, v3);
        totvol += vol;
        r_center += v1 * vol + v2 * vol + v3 * vol;
    }
    if totvol != 0.0 {
        r_center *= (1.0 / 3.0) / totvol;
    }
    (Some(totvol), Some(r_center))
}

/// `BKE_mesh_mdisp_flip` — transpose the displacement grid and swap
/// x/y (+ negate z with `use_loop_mdisp_flip`).
pub fn bke_mesh_mdisp_flip(disps: &mut [[f32; 3]], use_loop_mdisp_flip: bool) {
    if disps.is_empty() {
        return;
    }
    let sides = (disps.len() as f32).sqrt() as usize;
    for x in 0..sides {
        for y in 0..x {
            let a = y * sides + x;
            let b = x * sides + y;
            disps.swap(a, b);
            disps[a].swap(0, 1);
            disps[b].swap(0, 1);
            if use_loop_mdisp_flip {
                disps[a][2] *= -1.0;
                disps[b][2] *= -1.0;
            }
        }
        let a = x * sides + x;
        disps[a].swap(0, 1);
        if use_loop_mdisp_flip {
            disps[a][2] *= -1.0;
        }
    }
}

/// `mesh_edge_hide_from_vert` — `.hide_edge` from `.hide_vert`.
pub fn mesh_edge_hide_from_vert(
    edges: &[[i32; 2]],
    hide_vert: &[bool],
    hide_edge: &mut [bool],
) {
    for (i, e) in edges.iter().enumerate() {
        hide_edge[i] = hide_vert[e[0] as usize] || hide_vert[e[1] as usize];
    }
}

/// `mesh_face_hide_from_vert` — `.hide_poly` from `.hide_vert`.
pub fn mesh_face_hide_from_vert(
    faces: OffsetIndices,
    corner_verts: &[i32],
    hide_vert: &[bool],
    hide_poly: &mut [bool],
) {
    for i in 0..faces.len() {
        let g = faces.group(i);
        hide_poly[i] = corner_verts[g.start() as usize..g.one_after_last() as usize]
            .iter()
            .any(|&v| hide_vert[v as usize]);
    }
}

/// `mesh_hide_face_flush`'s core loop — propagate `.hide_poly` to
/// `.hide_vert`/`.hide_edge` (attribute storage itself is caller's).
#[allow(clippy::too_many_arguments)]
pub fn mesh_hide_face_flush(
    faces: OffsetIndices,
    corner_verts: &[i32],
    corner_edges: &[i32],
    hide_poly: &[bool],
    hide_vert: &mut [bool],
    hide_edge: &mut [bool],
) {
    for i in 0..faces.len() {
        let g = faces.group(i);
        if hide_poly[i] {
            for &v in &corner_verts[g.start() as usize..g.one_after_last() as usize] {
                hide_vert[v as usize] = true;
            }
            for &e in &corner_edges[g.start() as usize..g.one_after_last() as usize] {
                hide_edge[e as usize] = true;
            }
        }
    }
    for i in 0..faces.len() {
        let g = faces.group(i);
        if !hide_poly[i] {
            for &v in &corner_verts[g.start() as usize..g.one_after_last() as usize] {
                hide_vert[v as usize] = false;
            }
            for &e in &corner_edges[g.start() as usize..g.one_after_last() as usize] {
                hide_edge[e as usize] = false;
            }
        }
    }
}

/// `BKE_mesh_calc_relative_deform` — reconstruct `vert_cos_new` by
/// per-corner `transform_point_by_tri` mapping and averaging.
#[allow(clippy::too_many_arguments)]
pub fn bke_mesh_calc_relative_deform(
    faces: OffsetIndices,
    corner_verts: &[i32],
    totvert: usize,
    vert_cos_src: &[Vec3],
    vert_cos_dst: &[Vec3],
    vert_cos_org: &[Vec3],
    vert_cos_new: &mut [Vec3],
) {
    let mut vert_accum = vec![0i32; totvert];
    vert_cos_new[..totvert].fill(Vec3::ZERO);

    for i in 0..faces.len() {
        let g = faces.group(i);
        let face_verts = &corner_verts[g.start() as usize..g.one_after_last() as usize];
        let n = face_verts.len();
        for j in 0..n {
            let v_prev = face_verts[(n + j - 1) % n] as usize;
            let v_curr = face_verts[j] as usize;
            let v_next = face_verts[(j + 1) % n] as usize;

            let tvec = geom::transform_point_by_tri(
                vert_cos_dst[v_curr],
                &[
                    vert_cos_org[v_prev],
                    vert_cos_org[v_curr],
                    vert_cos_org[v_next],
                ],
                &[
                    vert_cos_src[v_prev],
                    vert_cos_src[v_curr],
                    vert_cos_src[v_next],
                ],
            );

            vert_cos_new[v_curr] += tvec;
            vert_accum[v_curr] += 1;
        }
    }

    for i in 0..totvert {
        if vert_accum[i] != 0 {
            vert_cos_new[i] *= 1.0 / vert_accum[i] as f32;
        } else {
            vert_cos_new[i] = vert_cos_org[i];
        }
    }
}

/// `mesh_select_vert_flush` / `mesh_select_edge_flush`'s array
/// parts — fill `select_*` at the complement of a hide mask.
/// `hide`/`select` are same-domain bool arrays; the mask plumbing
/// (`IndexMask`) maps to a plain filter in Rust.
pub fn select_flush_fill_unhidden(hide: &[bool], select: &mut [bool]) {
    for (i, &h) in hide.iter().enumerate() {
        if !h {
            select[i] = true;
        }
    }
}

/// `mesh_select_face_flush` — copy `.select_poly` into point/edge
/// domain arrays (caller resolves domain arrays).
pub fn select_flush_copy_domain(src: &[bool], dst: &mut [bool]) {
    dst[..src.len()].copy_from_slice(src);
}
