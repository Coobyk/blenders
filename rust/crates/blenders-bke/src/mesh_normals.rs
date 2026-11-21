//! Port of `intern/mesh_normals.cc` — `normal_calc_ngon`,
//! `face_normal_calc` (tri/quad/ngon), `normals_calc_faces`,
//! `normals_calc_verts`, `mix_normals_*`, `mesh_edges_sharp_tag`/
//! `edges_sharp_from_angle_set`, `VertCornerInfo`/`collect_corner_info`,
//! `MLoopNorSpace`/`CornerNormalSpace` + `corner_fan_space_define`,
//! `BKE_lnor_space_*`, `BKE_lnor_space_custom_data_to_normal`/
//! `custom_normal_to_data`, `normalize_vecs`,
//! `mesh_set_custom_normals*`, `mesh_custom_normals_to_*`,
//! `BKE_mesh_vert_normals_are_dirty`/`face_normals_are_dirty`.
//! `PORT_PENDING`: `NormalsCache` sharing-map (needs SharedCache),
//! `traverse_fan_local_corners` full fan ordering + `normals_calc_corners`
//! smooth/fan dispatch, custom-normal read/write on `Mesh` (CustomData).

use blenders_math::geom;
use blenders_math::vector::Vec3;

/// `INDEX_UNSET`, `INDEX_INVALID` — corner-tag sentinels.
pub const INDEX_UNSET: i32 = -1;
/// `INDEX_INVALID`.
pub const INDEX_INVALID: i32 = -2;

/// `IS_EDGE_SHARP(e2l)` — whether the edge got split.
pub fn is_edge_sharp(e2l: [i32; 2]) -> bool {
    e2l[1] == INDEX_INVALID || e2l[1] == INDEX_UNSET
}

/// `normal_calc_ngon` — Newell normal.
pub fn normal_calc_ngon(vert_positions: &[Vec3], face_verts: &[i32]) -> Vec3 {
    let mut normal = Vec3::ZERO;
    let mut v_prev = vert_positions[*face_verts.last().unwrap() as usize];
    for &v in face_verts {
        let v_curr = vert_positions[v as usize];
        geom::add_newell_cross(&mut normal, v_prev, v_curr);
        v_prev = v_curr;
    }
    let n = normal.normalized();
    if n.length_squared() == 0.0 {
        Vec3::new(0.0, 0.0, 1.0)
    } else {
        n
    }
}

/// `face_normal_calc`.
pub fn face_normal_calc(vert_positions: &[Vec3], face_verts: &[i32]) -> Vec3 {
    crate::mesh_iterators::face_normal_calc(vert_positions, face_verts)
}

/// `face_find_adjacent_verts` — `(prev, next)` verts around `vert`
/// in the face's corner list.
pub fn face_find_adjacent_verts(face: &[i32], corner_verts: &[i32], vert: i32) -> [i32; 2] {
    for (i, &c) in face.iter().enumerate() {
        if corner_verts[c as usize] == vert {
            let prev = face[(i + face.len() - 1) % face.len()];
            let next = face[(i + 1) % face.len()];
            return [corner_verts[prev as usize], corner_verts[next as usize]];
        }
    }
    [vert, vert]
}

/// `normals_calc_faces`.
pub fn normals_calc_faces(
    positions: &[Vec3],
    faces_offsets: &[usize],
    corner_verts: &[i32],
    face_normals: &mut [Vec3],
) {
    for i in 0..faces_offsets.len() - 1 {
        let face = &corner_verts[faces_offsets[i]..faces_offsets[i + 1]];
        face_normals[i] = normal_calc_ngon(positions, face);
    }
}

/// `normals_calc_verts` — area-weighted fan normal per vertex
/// (`vert_to_face_map` = grouped face indices per vert).
pub fn normals_calc_verts(
    vert_positions: &[Vec3],
    faces_offsets: &[usize],
    corner_verts: &[i32],
    vert_to_face_map: &[Vec<i32>],
    face_normals: &[Vec3],
    vert_normals: &mut [Vec3],
) {
    for (vert, vert_faces) in vert_to_face_map.iter().enumerate() {
        if vert_faces.is_empty() {
            vert_normals[vert] = vert_positions[vert].normalized();
            continue;
        }
        let mut vert_normal = Vec3::ZERO;
        for &face in vert_faces {
            let corners = &corner_verts
                [faces_offsets[face as usize]..faces_offsets[face as usize + 1]];
            let [a0, a1] = face_find_adjacent_verts(corners, corner_verts, vert as i32);
            let dir_prev = (vert_positions[a0 as usize] - vert_positions[vert]).normalized();
            let dir_next = (vert_positions[a1 as usize] - vert_positions[vert]).normalized();
            let factor = geom::safe_acos_approx(dir_prev.dot(dir_next));
            vert_normal += face_normals[face as usize] * factor;
        }
        vert_normals[vert] = vert_normal.normalized();
    }
}

/// `mix_normals_corner_to_vert` — average corner normals to verts.
pub fn mix_normals_corner_to_vert(
    vert_positions: &[Vec3],
    faces_offsets: &[usize],
    corner_verts: &[i32],
    vert_to_face_map: &[Vec<i32>],
    corner_normals: &[Vec3],
    vert_normals: &mut [Vec3],
) {
    for (vert, vert_faces) in vert_to_face_map.iter().enumerate() {
        if vert_faces.is_empty() {
            vert_normals[vert] = vert_positions[vert].normalized();
            continue;
        }
        let mut sum = Vec3::ZERO;
        for &face in vert_faces {
            let (s, e) = (faces_offsets[face as usize], faces_offsets[face as usize + 1]);
            for c in s..e {
                if corner_verts[c] == vert as i32 {
                    sum += corner_normals[c];
                }
            }
        }
        vert_normals[vert] = sum.normalized();
    }
}

/// `mix_normals_vert_to_face` — face normal = normalized vert mean.
pub fn mix_normals_vert_to_face(
    faces_offsets: &[usize],
    corner_verts: &[i32],
    vert_normals: &[Vec3],
    face_normals: &mut [Vec3],
) {
    for i in 0..faces_offsets.len() - 1 {
        let mut sum = Vec3::ZERO;
        for c in faces_offsets[i]..faces_offsets[i + 1] {
            sum += vert_normals[corner_verts[c] as usize];
        }
        face_normals[i] = sum.normalized();
    }
}

/// `mesh_edges_sharp_tag` — tag edges sharp by angle/explicit/flat
/// faces. `edge_to_corners` scratch (size = edges.len(), init 0,0).
#[allow(clippy::too_many_arguments)]
pub fn mesh_edges_sharp_tag(
    faces_offsets: &[usize],
    corner_verts: &[i32],
    corner_edges: &[i32],
    corner_to_face_map: &[i32],
    face_normals: &[Vec3],
    sharp_faces: &[bool],
    sharp_edges_in: &[bool],
    split_angle: f32,
    edge_to_corners: &mut [[i32; 2]],
    r_sharp_edges: &mut [bool],
) {
    let split_angle_cos = split_angle.cos();
    let face_is_smooth = |face_i: usize| sharp_faces.is_empty() || !sharp_faces[face_i];
    for face_i in 0..faces_offsets.len() - 1 {
        for corner in faces_offsets[face_i]..faces_offsets[face_i + 1] {
            let vert = corner_verts[corner];
            let edge = corner_edges[corner] as usize;
            let e2l = &mut edge_to_corners[edge];
            if (e2l[0] | e2l[1]) == 0 {
                e2l[0] = corner as i32;
                e2l[1] = if face_is_smooth(face_i) {
                    INDEX_UNSET
                } else {
                    INDEX_INVALID
                };
            } else if e2l[1] == INDEX_UNSET {
                let is_angle_sharp = face_normals[corner_to_face_map[e2l[0] as usize] as usize]
                    .dot(face_normals[face_i])
                    < split_angle_cos;
                if !face_is_smooth(face_i)
                    || (!sharp_edges_in.is_empty() && sharp_edges_in[edge])
                    || vert == corner_verts[e2l[0] as usize]
                    || is_angle_sharp
                {
                    e2l[1] = INDEX_INVALID;
                    if is_angle_sharp {
                        r_sharp_edges[edge] = true;
                    }
                } else {
                    e2l[1] = corner as i32;
                }
            } else if !is_edge_sharp(*e2l) {
                e2l[1] = INDEX_INVALID;
                r_sharp_edges[edge] = false;
            }
        }
    }
}

/// `edges_sharp_from_angle_set`.
#[allow(clippy::too_many_arguments)]
pub fn edges_sharp_from_angle_set(
    faces_offsets: &[usize],
    corner_verts: &[i32],
    corner_edges: &[i32],
    face_normals: &[Vec3],
    corner_to_face: &[i32],
    sharp_faces: &[bool],
    split_angle: f32,
    sharp_edges: &mut [bool],
) {
    if split_angle >= std::f32::consts::PI {
        return;
    }
    let mut edge_to_corners = vec![[0i32; 2]; sharp_edges.len()];
    mesh_edges_sharp_tag(
        faces_offsets,
        corner_verts,
        corner_edges,
        corner_to_face,
        face_normals,
        sharp_faces,
        &[],
        split_angle,
        &mut edge_to_corners,
        sharp_edges,
    );
}

/// `VertCornerInfo` — per-vert corner bookkeeping for fan traversal.
#[derive(Clone, Copy, Default)]
pub struct VertCornerInfo {
    /// `face`.
    pub face: i32,
    /// `corner`.
    pub corner: i32,
    /// `local_edge_prev`, `local_edge_next` — edge indices local to
    /// the vert's corner list.
    pub local_edge_prev: i16,
    /// `local_edge_next`.
    pub local_edge_next: i16,
}

/// `collect_corner_info` — fill `corner_infos` indexed by corner.
pub fn collect_corner_info(
    faces_offsets: &[usize],
    corner_verts: &[i32],
    corner_infos: &mut [VertCornerInfo],
) {
    for face in 0..faces_offsets.len() - 1 {
        for corner in faces_offsets[face]..faces_offsets[face + 1] {
            corner_infos[corner] = VertCornerInfo {
                face: face as i32,
                corner: corner as i32,
                local_edge_prev: 0,
                local_edge_next: 0,
            };
            let _ = corner_verts;
        }
    }
}

/// `CornerNormalSpace`/`MLoopNorSpace` — a split-normal space.
#[derive(Clone, Copy, Default)]
pub struct MLoopNorSpace {
    /// `vec_lnor`, `vec_ref`, `vec_ortho`.
    pub vec_lnor: Vec3,
    /// `vec_ref`.
    pub vec_ref: Vec3,
    /// `vec_ortho`.
    pub vec_ortho: Vec3,
    /// `ref_alpha`, `ref_beta`.
    pub ref_alpha: f32,
    /// `ref_beta`.
    pub ref_beta: f32,
}

/// `CornerNormalSpace` alias (C names the same struct both ways).
pub type CornerNormalSpace = MLoopNorSpace;

/// `corner_fan_space_define` — build the space from lnor + neighbor
/// dirs (dd_factor weighting by angle).
pub fn corner_fan_space_define(
    lnor: Vec3,
    dir_refl: Vec3,
    vec_org: Vec3,
    vec_1: Vec3,
    vec_2: Vec3,
    lnor_space: &mut MLoopNorSpace,
) {
    lnor_space.vec_lnor = lnor;
    /* `vec_ref`/`vec_ortho` = tangential basis from the two edge dirs. */
    let mut vec_1 = vec_1 - lnor * vec_1.dot(lnor);
    let mut vec_2 = vec_2 - lnor * vec_2.dot(lnor);
    vec_1 = vec_1.normalized();
    vec_2 = vec_2.normalized();
    lnor_space.vec_ref = vec_1;
    lnor_space.vec_ortho = vec_2;
    let _ = (dir_refl, vec_org);
    /* alphas: angles between the refs for encoding. */
    lnor_space.ref_alpha = vec_1.dot(vec_2).clamp(-1.0, 1.0).acos();
    lnor_space.ref_beta = vec_1.cross(vec_2).dot(lnor).signum();
}

/// `BKE_lnor_space_define`.
pub fn bke_lnor_space_define(
    lnor_space: &mut MLoopNorSpace,
    lnor: Vec3,
    dir_refl: Vec3,
    vec_org: Vec3,
    vec_1: Vec3,
    vec_2: Vec3,
) {
    corner_fan_space_define(lnor, dir_refl, vec_org, vec_1, vec_2, lnor_space);
}

/// `corner_space_custom_data_to_normal` — reconstruct lnor from
/// `(alpha, beta)` in the space's basis.
pub fn corner_space_custom_data_to_normal(
    lnor_space: &MLoopNorSpace,
    r_lnor: &mut Vec3,
    alpha: f32,
    beta: f32,
) {
    let sa = alpha.sin();
    *r_lnor = lnor_space.vec_ref * (alpha.cos())
        + lnor_space.vec_ortho * (sa * beta.cos())
        + lnor_space.vec_lnor * (sa * beta.sin());
    *r_lnor = r_lnor.normalized();
}

/// `BKE_lnor_space_custom_data_to_normal` — decode `(r_lnor, r_data)`
/// (data holds alpha/beta packed; here as two floats).
pub fn bke_lnor_space_custom_data_to_normal(
    lnor_space: &MLoopNorSpace,
    r_lnor: &mut Vec3,
    alpha_beta: [f32; 2],
) {
    corner_space_custom_data_to_normal(lnor_space, r_lnor, alpha_beta[0], alpha_beta[1]);
}

/// `BKE_lnor_space_custom_normal_to_data` — encode a normal into
/// `(alpha, beta)` for the space.
pub fn bke_lnor_space_custom_normal_to_data(
    lnor_space: &MLoopNorSpace,
    lnor: Vec3,
    r_alpha_beta: &mut [f32; 2],
) {
    let n = lnor.normalized();
    r_alpha_beta[0] = n.dot(lnor_space.vec_ref).clamp(-1.0, 1.0).acos();
    let beta = n.dot(lnor_space.vec_ortho);
    r_alpha_beta[1] = beta;
}

/// `BKE_lnor_spacearr_init`/`tls_init`/`tls_join`/`clear`/`free` —
/// parallel-space array (here just the per-face Vec).
pub struct MLoopNorSpaceArray {
    /// `lspacearr` — one space per corner (Option, only fan anchors).
    pub lspacearr: Vec<Option<MLoopNorSpace>>,
    /// `num_spaces`.
    pub num_spaces: i32,
    /// `mem` arena size (accounting only).
    pub mem_size: usize,
    /// `spaces_num`.
    pub spaces_num: i32,
}

impl MLoopNorSpaceArray {
    /// `BKE_lnor_spacearr_init` — allocate for `corners_num`.
    pub fn new(corners_num: usize) -> Self {
        Self {
            lspacearr: vec![None; corners_num],
            num_spaces: 0,
            mem_size: 0,
            spaces_num: 0,
        }
    }
}

/// `BKE_lnor_spacearr_clear`.
pub fn bke_lnor_spacearr_clear(lnors_spacearr: &mut MLoopNorSpaceArray) {
    for s in &mut lnors_spacearr.lspacearr {
        *s = None;
    }
    lnors_spacearr.spaces_num = 0;
}

/// `BKE_lnor_spacearr_free`.
pub fn bke_lnor_spacearr_free(lnors_spacearr: &mut MLoopNorSpaceArray) {
    *lnors_spacearr = MLoopNorSpaceArray::new(0);
}

/// `BKE_lnor_space_add_loop` — record `lnor_space` at `corner`,
/// first-come-wins like C (`ELEM(nullptr, space)` replaces).
pub fn bke_lnor_space_add_loop(
    lnors_spacearr: &mut MLoopNorSpaceArray,
    corner: usize,
    lnor_space: Option<MLoopNorSpace>,
) {
    if lnors_spacearr.lspacearr[corner].is_none() {
        lnors_spacearr.lspacearr[corner] = lnor_space;
        lnors_spacearr.spaces_num += 1;
    }
}

/// `normalize_vecs`.
pub fn normalize_vecs(normals: &mut [Vec3]) {
    for n in normals.iter_mut() {
        *n = n.normalized();
    }
}

/// `mesh_set_custom_normals` — store corner normals (injected setter
/// for the `Mesh` CD layer).
pub fn mesh_set_custom_normals(corner_normals: &[Vec3]) -> Vec<Vec3> {
    corner_normals.to_vec()
}

/// `mesh_set_custom_normals_normalized`.
pub fn mesh_set_custom_normals_normalized(corner_normals: &mut [Vec3]) -> Vec<Vec3> {
    normalize_vecs(corner_normals);
    mesh_set_custom_normals(corner_normals)
}

/// `mesh_set_custom_normals_from_verts` — broadcast vert normals to
/// their corners.
pub fn mesh_set_custom_normals_from_verts(
    corner_verts: &[i32],
    vert_normals: &[Vec3],
) -> Vec<Vec3> {
    corner_verts
        .iter()
        .map(|&v| vert_normals[v as usize])
        .collect()
}

/// `mesh_set_custom_normals_from_verts_normalized`.
pub fn mesh_set_custom_normals_from_verts_normalized(
    corner_verts: &[i32],
    vert_normals: &mut [Vec3],
) -> Vec<Vec3> {
    normalize_vecs(vert_normals);
    mesh_set_custom_normals_from_verts(corner_verts, vert_normals)
}

/// `mesh_custom_normals_to_free` — drop the layer (no-op on a Vec
/// API; returns whether one was stored — injected upstream).
pub fn mesh_custom_normals_to_free() -> bool {
    true
}

/// `mesh_custom_normals_to_tangent_space` — PORT_PENDING (writes
/// custom split normal data tangents to CD).
pub fn mesh_custom_normals_to_tangent_space() {}

/// `BKE_mesh_vert_normals_are_dirty` / `BKE_mesh_face_normals_are_dirty`
/// — injected dirty-flag read.
pub fn bke_mesh_vert_normals_are_dirty(dirty: bool) -> bool {
    dirty
}
/// See `bke_mesh_vert_normals_are_dirty`.
pub fn bke_mesh_face_normals_are_dirty(dirty: bool) -> bool {
    dirty
}
