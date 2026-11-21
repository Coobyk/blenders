//! Port of `intern/subdiv_mesh.cc` (portable parts) +
//! `subdiv_inline.hh` — `quad_weights_from_uv`, `FaceElements`,
//! `AttributeMixer`/`NormalMixer`, `interpolate_in_faces`,
//! loose-edge interpolation (`find_edge_neighbors`,
//! `find_loose_edge_interpolation_positions`,
//! `mesh_interpolate_position_on_edge`,
//! `interpolate_loose_edge_vert_data`, `subdiv_mesh_loose_edges`).
//! `PORT_PENDING`: custom-data attrs (GSpan), MDeformVert mixing,
//! orco, `subdiv_to_mesh` driver (needs Subdiv evaluator).

use blenders_math::vector::Vec3;

/// `quad_weights_from_uv` — bilinear weights (w0..w3 corner order).
pub fn quad_weights_from_uv(u: f32, v: f32) -> [f32; 4] {
    [
        (1.0 - u) * (1.0 - v),
        u * (1.0 - v),
        u * v,
        (1.0 - u) * v,
    ]
}

/// `quad_weights_from_coords`.
pub fn quad_weights_from_coords(coords: &[[f32; 2]]) -> Vec<[f32; 4]> {
    coords
        .iter()
        .map(|c| quad_weights_from_uv(c[0], c[1]))
        .collect()
}

/// `PtexCoord` (subdiv ptex coordinates).
#[derive(Clone, Copy, Default)]
pub struct PtexCoord {
    /// `ptex_face_index`.
    pub ptex_face_index: i32,
    /// `uv`.
    pub uv: [f32; 2],
}

/// `ElementRange` — a range of (coord → element) pairs.
#[derive(Clone, Copy, Default)]
pub struct ElementRange {
    /// `coord` start (into ptex coords).
    pub coord_start: usize,
    /// `element` start (into subdiv output).
    pub element_start: usize,
    /// count.
    pub count: usize,
}

/// `FaceElements` — groups of element ranges by face.
#[derive(Default)]
pub struct FaceElements {
    /// `faces` (face index per group).
    pub faces: Vec<i32>,
    /// `ranges` (element ranges, grouped).
    pub ranges: Vec<ElementRange>,
    /// `ranges_by_face` offsets (len groups+1).
    pub ranges_offsets: Vec<usize>,
}

impl FaceElements {
    /// `ranges_by_face` accessor — group's range slice.
    pub fn ranges_of(&self, group: usize) -> &[ElementRange] {
        &self.ranges[self.ranges_offsets[group]..self.ranges_offsets[group + 1]]
    }
}

/// `Mixable` — the AttributeMixer<T> contract.
pub trait Mixable: Copy {
    /// `mix2(0.5, a, b)` equivalent — midpoint.
    fn mid(a: Self, b: Self) -> Self;
    /// `mix4(w, a, b, c, d)`.
    fn quad(w: &[f32; 4], a: Self, b: Self, c: Self, d: Self) -> Self;
    /// weighted mean over (src, indices, weights).
    fn center(src: &[Self], indices: &[usize], weights: &[f32]) -> Self;
}

impl Mixable for f32 {
    fn mid(a: f32, b: f32) -> f32 {
        (a + b) * 0.5
    }
    fn quad(w: &[f32; 4], a: f32, b: f32, c: f32, d: f32) -> f32 {
        a * w[0] + b * w[1] + c * w[2] + d * w[3]
    }
    fn center(src: &[f32], indices: &[usize], weights: &[f32]) -> f32 {
        let mut r = 0.0;
        for i in 0..indices.len() {
            r += src[indices[i]] * weights[i];
        }
        r
    }
}

impl Mixable for Vec3 {
    fn mid(a: Vec3, b: Vec3) -> Vec3 {
        (a + b) * 0.5
    }
    fn quad(w: &[f32; 4], a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> Vec3 {
        a * w[0] + b * w[1] + c * w[2] + d * w[3]
    }
    fn center(src: &[Vec3], indices: &[usize], weights: &[f32]) -> Vec3 {
        let mut r = Vec3::ZERO;
        for i in 0..indices.len() {
            r = r + src[indices[i]] * weights[i];
        }
        r
    }
}

/// `NormalMixer` — `Mixable` for normals (normalize each result).
pub struct NormalMixable;

impl NormalMixable {
    /// `NormalMixer::center`.
    pub fn center(src: &[Vec3], indices: &[usize], weights: &[f32]) -> Vec3 {
        let mut r = Vec3::ZERO;
        for i in 0..indices.len() {
            r = r + src[indices[i]] * weights[i];
        }
        r.normalized()
    }
    /// `NormalMixer::mid`.
    pub fn mid(a: Vec3, b: Vec3) -> Vec3 {
        ((a + b) * 0.5).normalized()
    }
    /// `NormalMixer::quad`.
    pub fn quad(w: &[f32; 4], a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> Vec3 {
        (a * w[0] + b * w[1] + c * w[2] + d * w[3]).normalized()
    }
}

/// `interpolate_in_faces<T, AttributeMixer<T>>` — quad faces use
/// direct bilinear; other faces use corner/edge-mid/center quad.
#[allow(clippy::too_many_arguments)]
pub fn interpolate_in_faces<T: Mixable>(
    faces_offsets: &[usize], /* face -> corner range */
    corner_indices: &[usize], /* src index per corner (e.g. corner_verts) */
    face_ptex_offset: &[i32],
    groups: &FaceElements,
    coords: &[PtexCoord],
    weights: &[[f32; 4]],
    src: &[T],
    dst: &mut [T],
) {
    let mut face_src_indices: Vec<usize> = Vec::new();
    let mut center_weights: Vec<f32> = Vec::new();
    let mut edge_mids: Vec<T> = Vec::new();
    for group in 0..groups.faces.len() {
        let face_index = groups.faces[group] as usize;
        let face_start = faces_offsets[face_index];
        let face_end = faces_offsets[face_index + 1];
        let corners_num = face_end - face_start;
        let ranges = groups.ranges_of(group);
        if corners_num == 4 {
            let v0 = src[corner_indices[face_start]];
            let v1 = src[corner_indices[face_start + 1]];
            let v2 = src[corner_indices[face_start + 2]];
            let v3 = src[corner_indices[face_start + 3]];
            for range in ranges {
                for i in 0..range.count {
                    dst[range.element_start + i] =
                        T::quad(&weights[range.coord_start + i], v0, v1, v2, v3);
                }
            }
            continue;
        }
        face_src_indices.clear();
        for i in face_start..face_end {
            face_src_indices.push(corner_indices[i]);
        }
        center_weights.clear();
        center_weights.resize(corners_num, 1.0 / corners_num as f32);
        let center = T::center(src, &face_src_indices, &center_weights);
        edge_mids.clear();
        for i in 0..corners_num {
            edge_mids.push(T::mid(
                src[face_src_indices[i]],
                src[face_src_indices[(i + 1) % corners_num]],
            ));
        }
        for range in ranges {
            for i in 0..range.count {
                let coord = range.coord_start + i;
                let corner = (coords[coord].ptex_face_index - face_ptex_offset[face_index])
                    as usize;
                let prev_corner = (corner + corners_num - 1) % corners_num;
                dst[range.element_start + i] = T::quad(
                    &weights[coord],
                    src[face_src_indices[corner]],
                    edge_mids[corner],
                    center,
                    edge_mids[prev_corner],
                );
            }
        }
    }
}

/// `interpolate_in_faces<float3, NormalMixer>` variant.
#[allow(clippy::too_many_arguments)]
pub fn interpolate_normals_in_faces(
    faces_offsets: &[usize],
    face_ptex_offset: &[i32],
    groups: &FaceElements,
    coords: &[PtexCoord],
    weights: &[[f32; 4]],
    src: &[Vec3],
    dst: &mut [Vec3],
) {
    let mut center_weights: Vec<f32> = Vec::new();
    let mut edge_mids: Vec<Vec3> = Vec::new();
    for group in 0..groups.faces.len() {
        let face_index = groups.faces[group] as usize;
        let face_start = faces_offsets[face_index];
        let face_end = faces_offsets[face_index + 1];
        let corners_num = face_end - face_start;
        let ranges = groups.ranges_of(group);
        /* src_index = corner itself for corner-domain attrs. */
        let src_index = |c: usize| c;
        if corners_num == 4 {
            let v0 = src[src_index(face_start)];
            let v1 = src[src_index(face_start + 1)];
            let v2 = src[src_index(face_start + 2)];
            let v3 = src[src_index(face_start + 3)];
            for range in ranges {
                for i in 0..range.count {
                    dst[range.element_start + i] =
                        NormalMixable::quad(&weights[range.coord_start + i], v0, v1, v2, v3);
                }
            }
            continue;
        }
        let face_src_indices: Vec<usize> = (face_start..face_end).collect();
        center_weights.clear();
        center_weights.resize(corners_num, 1.0 / corners_num as f32);
        let center = NormalMixable::center(src, &face_src_indices, &center_weights);
        edge_mids.clear();
        for i in 0..corners_num {
            edge_mids.push(NormalMixable::mid(
                src[face_src_indices[i]],
                src[face_src_indices[(i + 1) % corners_num]],
            ));
        }
        for range in ranges {
            for i in 0..range.count {
                let coord = range.coord_start + i;
                let corner = (coords[coord].ptex_face_index - face_ptex_offset[face_index])
                    as usize;
                let prev_corner = (corner + corners_num - 1) % corners_num;
                dst[range.element_start + i] = NormalMixable::quad(
                    &weights[coord],
                    src[face_src_indices[corner]],
                    edge_mids[corner],
                    center,
                    edge_mids[prev_corner],
                );
            }
        }
    }
}

/// `mesh::edge_other_vert`.
pub fn edge_other_vert(edge: [i32; 2], vert: i32) -> i32 {
    if edge[0] == vert {
        edge[1]
    } else {
        edge[0]
    }
}

/// `find_edge_neighbors`.
pub fn find_edge_neighbors(
    coarse_edges: &[[i32; 2]],
    vert_to_edge_map: &(Vec<usize>, Vec<usize>),
    edge_index: usize,
) -> [Option<[i32; 2]>; 2] {
    let neighbor_edge_if_single = |vert: i32| -> Option<[i32; 2]> {
        let (offsets, indices) = vert_to_edge_map;
        let neighbors = &indices[offsets[vert as usize]..offsets[vert as usize + 1]];
        if neighbors.len() != 2 {
            return None;
        }
        if neighbors[0] == edge_index {
            Some(coarse_edges[neighbors[1]])
        } else {
            Some(coarse_edges[neighbors[0]])
        }
    };
    let edge = coarse_edges[edge_index];
    [neighbor_edge_if_single(edge[0]), neighbor_edge_if_single(edge[1])]
}

/// `find_loose_edge_interpolation_positions`.
pub fn find_loose_edge_interpolation_positions(
    coarse_positions: &[Vec3],
    coarse_edge: [i32; 2],
    neighbors: &[Option<[i32; 2]>; 2],
) -> [Vec3; 4] {
    let mut result = [Vec3::ZERO; 4];
    result[1] = coarse_positions[coarse_edge[0] as usize];
    result[2] = coarse_positions[coarse_edge[1] as usize];
    result[0] = match neighbors[0] {
        Some(other) => coarse_positions[edge_other_vert(other, coarse_edge[0]) as usize],
        None => result[1] * 2.0 - result[2],
    };
    result[3] = match neighbors[1] {
        Some(other) => coarse_positions[edge_other_vert(other, coarse_edge[1]) as usize],
        None => result[2] * 2.0 - result[1],
    };
    result
}

/// `mesh_interpolate_position_on_edge` — linear for simple subdiv,
/// B-spline curve position otherwise.
pub fn mesh_interpolate_position_on_edge(
    coarse_positions: &[Vec3],
    coarse_edges: &[[i32; 2]],
    vert_to_edge_map: &(Vec<usize>, Vec<usize>),
    coarse_edge_index: usize,
    is_simple: bool,
    u: f32,
) -> Vec3 {
    let edge = coarse_edges[coarse_edge_index];
    if is_simple {
        return coarse_positions[edge[0] as usize]
            + (coarse_positions[edge[1] as usize] - coarse_positions[edge[0] as usize]) * u;
    }
    let neighbors = find_edge_neighbors(coarse_edges, vert_to_edge_map, coarse_edge_index);
    let points = find_loose_edge_interpolation_positions(coarse_positions, edge, &neighbors);
    let weights = crate::anim_path::key_curve_position_weights(u, crate::anim_path::KEY_BSPLINE);
    points[0] * weights[0] + points[1] * weights[1] + points[2] * weights[2]
        + points[3] * weights[3]
}

/// `interpolate_loose_edge_vert_data` — f32 attribute version:
/// `dst[base_edge_verts[i]] = mix2(u, a, b)` for interior edge verts.
#[allow(clippy::too_many_arguments)]
pub fn interpolate_loose_edge_vert_data(
    edges: &[[i32; 2]],
    resolution: usize,
    loose_edges: &[usize],
    base_edge_verts: &dyn Fn(usize) -> std::ops::Range<usize>,
    src: &[f32],
    dst: &mut [f32],
) {
    let inv_resolution_1 = 1.0 / (resolution - 1) as f32;
    for &edge in loose_edges {
        let a = src[edges[edge][0] as usize];
        let b = src[edges[edge][1] as usize];
        let edge_verts = base_edge_verts(edge);
        for (i, ev) in edge_verts.enumerate() {
            let u = (i + 1) as f32 * inv_resolution_1;
            dst[ev] = a * (1.0 - u) + b * u;
        }
    }
}

/// `subdiv_mesh_loose_edges` — write interpolated positions along
/// loose edges (serial port).
#[allow(clippy::too_many_arguments)]
pub fn subdiv_mesh_loose_edges(
    coarse_positions: &[Vec3],
    coarse_edges: &[[i32; 2]],
    vert_to_edge_map: &(Vec<usize>, Vec<usize>),
    loose_edges: &[usize],
    resolution: usize,
    is_simple: bool,
    base_edge_verts: &dyn Fn(usize) -> std::ops::Range<usize>,
    subdiv_positions: &mut [Vec3],
) {
    let inv_resolution_1 = 1.0 / (resolution - 1) as f32;
    for &coarse_edge_index in loose_edges {
        let coarse_edge = coarse_edges[coarse_edge_index];
        let edge_verts = base_edge_verts(coarse_edge_index);
        for i in 0..resolution {
            let u = i as f32 * inv_resolution_1;
            let subdiv_vert_index = if i == 0 {
                coarse_edge[0] as usize
            } else if i == resolution - 1 {
                coarse_edge[1] as usize
            } else {
                edge_verts.start + i - 1
            };
            subdiv_positions[subdiv_vert_index] = mesh_interpolate_position_on_edge(
                coarse_positions,
                coarse_edges,
                vert_to_edge_map,
                coarse_edge_index,
                is_simple,
                u,
            );
        }
    }
}

/// `subdiv_mesh_is_center_vert` — is (u,v) the face center.
pub fn subdiv_mesh_is_center_vert(coarse_face_size: usize, u: f32, v: f32) -> bool {
    u == 0.5 && v == 0.5 && coarse_face_size > 4
}
