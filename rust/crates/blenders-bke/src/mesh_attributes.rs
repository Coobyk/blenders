//! Port of `intern/mesh_attributes.cc` — mesh domain adaptation
//! (corner↔point↔edge↔face) for a generic `T: AttrMix` plus the
//! verbatim `bool` specializations. `GVArray` → `Vec<T>`; mesh
//! topology comes in as `MeshTopo` slices. `parallel_for` → serial.

use crate::attribute_math::{AttrMix, DefaultMixer};

/// Mesh topology inputs (slices of `Mesh` arrays).
pub struct MeshTopo<'a> {
    /// `verts_num`.
    pub verts_num: i32,
    /// `edges_num`.
    pub edges_num: i32,
    /// `corners_num`.
    pub corners_num: i32,
    /// `faces_num`.
    pub faces_num: i32,
    /// `edges`.
    pub edges: &'a [[i32; 2]],
    /// `corner_verts`.
    pub corner_verts: &'a [i32],
    /// `corner_edges`.
    pub corner_edges: &'a [i32],
    /// `face_offsets` (len faces_num+1).
    pub face_offsets: &'a [i32],
    /// `vert_to_face_map` — face indices per vertex.
    pub vert_to_face_map: &'a [Vec<i32>],
    /// `loose_verts` mask indices.
    pub loose_verts: &'a [i32],
    /// `loose_edges` mask indices.
    pub loose_edges: &'a [i32],
}

/// `mesh::face_find_corner_from_vert`.
fn face_find_corner_from_vert(face: (i32, i32), corner_verts: &[i32], vert: i32) -> i32 {
    for c in face.0..face.0 + face.1 {
        if corner_verts[c as usize] == vert {
            return c;
        }
    }
    face.0
}
/// `mesh::face_corner_next`.
fn face_corner_next(face: (i32, i32), corner: i32) -> i32 {
    if corner == face.0 + face.1 - 1 {
        face.0
    } else {
        corner + 1
    }
}
/// `mesh::face_corner_prev`.
fn face_corner_prev(face: (i32, i32), corner: i32) -> i32 {
    if corner == face.0 {
        face.0 + face.1 - 1
    } else {
        corner - 1
    }
}

/// `adapt_mesh_domain_corner_to_point_impl<T>`.
pub fn adapt_mesh_domain_corner_to_point<T: AttrMix + Default>(
    mesh: &MeshTopo,
    src: &[T],
) -> Vec<T> {
    let mut r_dst = vec![T::default(); mesh.verts_num as usize];
    for vert in 0..mesh.verts_num as usize {
        let vert_faces = &mesh.vert_to_face_map[vert];
        let mut mixer = DefaultMixer::new(vec![T::default()], T::default());
        for &face in vert_faces {
            let fr = (mesh.face_offsets[face as usize], mesh.face_offsets[face as usize + 1]
                - mesh.face_offsets[face as usize]);
            let corner = face_find_corner_from_vert(fr, mesh.corner_verts, vert as i32);
            mixer.mix_in(0, src[corner as usize], 1.0);
        }
        mixer.finalize();
        r_dst[vert] = mixer.buffer[0];
    }
    r_dst
}

/// `adapt_mesh_domain_corner_to_point_impl<bool>`.
pub fn adapt_mesh_domain_corner_to_point_bool(mesh: &MeshTopo, src: &[bool]) -> Vec<bool> {
    let mut r_dst = vec![true; mesh.verts_num as usize];
    for corner in 0..mesh.corners_num as usize {
        if !src[corner] {
            r_dst[mesh.corner_verts[corner] as usize] = false;
        }
    }
    for &v in mesh.loose_verts {
        r_dst[v as usize] = false;
    }
    r_dst
}

/// `adapt_mesh_domain_point_to_corner` — gather.
pub fn adapt_mesh_domain_point_to_corner<T: Copy>(mesh: &MeshTopo, src: &[T]) -> Vec<T> {
    (0..mesh.corners_num as usize)
        .map(|c| src[mesh.corner_verts[c] as usize])
        .collect()
}

/// `adapt_mesh_domain_corner_to_face`.
pub fn adapt_mesh_domain_corner_to_face<T: AttrMix + Default>(
    mesh: &MeshTopo,
    src: &[T],
) -> Vec<T> {
    (0..mesh.faces_num as usize)
        .map(|f| {
            let (s, e) = (
                mesh.face_offsets[f] as usize,
                mesh.face_offsets[f + 1] as usize,
            );
            let mut mixer = DefaultMixer::new(vec![T::default()], T::default());
            for corner in s..e {
                mixer.mix_in(0, src[corner], 1.0);
            }
            mixer.finalize();
            mixer.buffer[0]
        })
        .collect()
}

/// `adapt_mesh_domain_corner_to_face<bool>` — all-of.
pub fn adapt_mesh_domain_corner_to_face_bool(mesh: &MeshTopo, src: &[bool]) -> Vec<bool> {
    (0..mesh.faces_num as usize)
        .map(|f| {
            (mesh.face_offsets[f] as usize..mesh.face_offsets[f + 1] as usize).all(|c| src[c])
        })
        .collect()
}

/// `adapt_mesh_domain_corner_to_edge_impl<T>`.
pub fn adapt_mesh_domain_corner_to_edge<T: AttrMix + Default>(
    mesh: &MeshTopo,
    src: &[T],
) -> Vec<T> {
    let mut mixer = DefaultMixer::new(vec![T::default(); mesh.edges_num as usize], T::default());
    for face_index in 0..mesh.faces_num as usize {
        let (s, e) = (
            mesh.face_offsets[face_index],
            mesh.face_offsets[face_index + 1],
        );
        for corner in s..e {
            let next_corner = face_corner_next((s, e - s), corner);
            let edge_index = mesh.corner_edges[corner as usize] as usize;
            mixer.mix_in(edge_index, src[corner as usize], 1.0);
            mixer.mix_in(edge_index, src[next_corner as usize], 1.0);
        }
    }
    mixer.finalize();
    mixer.buffer
}

/// `adapt_mesh_domain_corner_to_edge_impl<bool>` — all-of pair;
/// loose edges false.
pub fn adapt_mesh_domain_corner_to_edge_bool(mesh: &MeshTopo, src: &[bool]) -> Vec<bool> {
    let mut r_values = vec![true; mesh.edges_num as usize];
    for face_index in 0..mesh.faces_num as usize {
        let (s, e) = (
            mesh.face_offsets[face_index],
            mesh.face_offsets[face_index + 1],
        );
        for corner in s..e {
            let next_corner = face_corner_next((s, e - s), corner);
            let edge_index = mesh.corner_edges[corner as usize] as usize;
            if !src[corner as usize] || !src[next_corner as usize] {
                r_values[edge_index] = false;
            }
        }
    }
    for &e in mesh.loose_edges {
        r_values[e as usize] = false;
    }
    r_values
}

/// `adapt_mesh_domain_face_to_point<T>`.
pub fn adapt_mesh_domain_face_to_point<T: AttrMix + Default>(
    mesh: &MeshTopo,
    src: &[T],
) -> Vec<T> {
    (0..mesh.verts_num as usize)
        .map(|point_i| {
            let mut mixer = DefaultMixer::new(vec![T::default()], T::default());
            for &face in &mesh.vert_to_face_map[point_i] {
                mixer.mix_in(0, src[face as usize], 1.0);
            }
            mixer.finalize();
            mixer.buffer[0]
        })
        .collect()
}

/// `adapt_mesh_domain_face_to_point<bool>` — any-of.
pub fn adapt_mesh_domain_face_to_point_bool(mesh: &MeshTopo, src: &[bool]) -> Vec<bool> {
    (0..mesh.verts_num as usize)
        .map(|p| mesh.vert_to_face_map[p].iter().any(|&f| src[f as usize]))
        .collect()
}

/// `adapt_mesh_domain_face_to_corner` — group gather.
pub fn adapt_mesh_domain_face_to_corner<T: Copy>(mesh: &MeshTopo, src: &[T]) -> Vec<T> {
    let mut out = Vec::with_capacity(mesh.corners_num as usize);
    for f in 0..mesh.faces_num as usize {
        let n = (mesh.face_offsets[f + 1] - mesh.face_offsets[f]) as usize;
        for _ in 0..n {
            out.push(src[f]);
        }
    }
    out
}

/// `adapt_mesh_domain_face_to_edge_impl<T>`.
pub fn adapt_mesh_domain_face_to_edge<T: AttrMix + Default>(
    mesh: &MeshTopo,
    src: &[T],
) -> Vec<T> {
    let mut mixer = DefaultMixer::new(vec![T::default(); mesh.edges_num as usize], T::default());
    for face_index in 0..mesh.faces_num as usize {
        let value = src[face_index];
        for e in mesh.face_offsets[face_index]..mesh.face_offsets[face_index + 1] {
            mixer.mix_in(mesh.corner_edges[e as usize] as usize, value, 1.0);
        }
    }
    mixer.finalize();
    mixer.buffer
}

/// `adapt_mesh_domain_face_to_edge_impl<bool>` — any-of.
pub fn adapt_mesh_domain_face_to_edge_bool(mesh: &MeshTopo, src: &[bool]) -> Vec<bool> {
    let mut r_values = vec![false; mesh.edges_num as usize];
    for face_index in 0..mesh.faces_num as usize {
        if src[face_index] {
            for e in mesh.face_offsets[face_index]..mesh.face_offsets[face_index + 1] {
                r_values[mesh.corner_edges[e as usize] as usize] = true;
            }
        }
    }
    r_values
}

/// `adapt_mesh_domain_point_to_face<T>`.
pub fn adapt_mesh_domain_point_to_face<T: AttrMix + Default>(
    mesh: &MeshTopo,
    src: &[T],
) -> Vec<T> {
    (0..mesh.faces_num as usize)
        .map(|f| {
            let mut mixer = DefaultMixer::new(vec![T::default()], T::default());
            for e in mesh.face_offsets[f]..mesh.face_offsets[f + 1] {
                mixer.mix_in(0, src[mesh.corner_verts[e as usize] as usize], 1.0);
            }
            mixer.finalize();
            mixer.buffer[0]
        })
        .collect()
}

/// `adapt_mesh_domain_point_to_face<bool>` — all-of.
pub fn adapt_mesh_domain_point_to_face_bool(mesh: &MeshTopo, src: &[bool]) -> Vec<bool> {
    (0..mesh.faces_num as usize)
        .map(|f| {
            (mesh.face_offsets[f]..mesh.face_offsets[f + 1])
                .all(|e| src[mesh.corner_verts[e as usize] as usize])
        })
        .collect()
}

/// `adapt_mesh_domain_point_to_edge<T>` — `mix2(0.5)`.
pub fn adapt_mesh_domain_point_to_edge<T: AttrMix>(mesh: &MeshTopo, src: &[T]) -> Vec<T> {
    mesh.edges
        .iter()
        .map(|e| T::mix2(0.5, src[e[0] as usize], src[e[1] as usize]))
        .collect()
}

/// `adapt_mesh_domain_point_to_edge<bool>` — and-of endpoints.
pub fn adapt_mesh_domain_point_to_edge_bool(mesh: &MeshTopo, src: &[bool]) -> Vec<bool> {
    mesh.edges
        .iter()
        .map(|e| src[e[0] as usize] && src[e[1] as usize])
        .collect()
}

/// `adapt_mesh_domain_edge_to_corner_impl<T>`.
pub fn adapt_mesh_domain_edge_to_corner<T: AttrMix + Default>(
    mesh: &MeshTopo,
    src: &[T],
) -> Vec<T> {
    let mut mixer = DefaultMixer::new(vec![T::default(); mesh.corners_num as usize], T::default());
    for face_index in 0..mesh.faces_num as usize {
        let (s, e) = (
            mesh.face_offsets[face_index],
            mesh.face_offsets[face_index + 1],
        );
        for corner in s..e {
            let corner_prev = face_corner_prev((s, e - s), corner);
            let edge = mesh.corner_edges[corner as usize] as usize;
            let edge_prev = mesh.corner_edges[corner_prev as usize] as usize;
            mixer.mix_in(corner as usize, src[edge], 1.0);
            mixer.mix_in(corner as usize, src[edge_prev], 1.0);
        }
    }
    mixer.finalize();
    mixer.buffer
}

/// `adapt_mesh_domain_edge_to_corner_impl<bool>` — and-of pair.
pub fn adapt_mesh_domain_edge_to_corner_bool(mesh: &MeshTopo, src: &[bool]) -> Vec<bool> {
    let mut r_values = vec![false; mesh.corners_num as usize];
    for face_index in 0..mesh.faces_num as usize {
        let (s, e) = (
            mesh.face_offsets[face_index],
            mesh.face_offsets[face_index + 1],
        );
        for corner in s..e {
            let corner_prev = face_corner_prev((s, e - s), corner);
            if src[mesh.corner_edges[corner as usize] as usize]
                && src[mesh.corner_edges[corner_prev as usize] as usize]
            {
                r_values[corner as usize] = true;
            }
        }
    }
    r_values
}

/// `adapt_mesh_domain_edge_to_point_impl<T>`.
pub fn adapt_mesh_domain_edge_to_point<T: AttrMix + Default>(
    mesh: &MeshTopo,
    src: &[T],
) -> Vec<T> {
    let mut mixer = DefaultMixer::new(vec![T::default(); mesh.verts_num as usize], T::default());
    for (i, edge) in mesh.edges.iter().enumerate() {
        let value = src[i];
        mixer.mix_in(edge[0] as usize, value, 1.0);
        mixer.mix_in(edge[1] as usize, value, 1.0);
    }
    mixer.finalize();
    mixer.buffer
}

/// `adapt_mesh_domain_edge_to_point_impl<bool>` — any-of.
pub fn adapt_mesh_domain_edge_to_point_bool(mesh: &MeshTopo, src: &[bool]) -> Vec<bool> {
    let mut r_values = vec![false; mesh.verts_num as usize];
    for (i, edge) in mesh.edges.iter().enumerate() {
        if src[i] {
            r_values[edge[0] as usize] = true;
            r_values[edge[1] as usize] = true;
        }
    }
    r_values
}

/// `adapt_mesh_domain_edge_to_face<T>`.
pub fn adapt_mesh_domain_edge_to_face<T: AttrMix + Default>(
    mesh: &MeshTopo,
    src: &[T],
) -> Vec<T> {
    (0..mesh.faces_num as usize)
        .map(|f| {
            let mut mixer = DefaultMixer::new(vec![T::default()], T::default());
            for e in mesh.face_offsets[f]..mesh.face_offsets[f + 1] {
                mixer.mix_in(0, src[mesh.corner_edges[e as usize] as usize], 1.0);
            }
            mixer.finalize();
            mixer.buffer[0]
        })
        .collect()
}

/// `adapt_mesh_domain_edge_to_face<bool>` — all-of.
pub fn adapt_mesh_domain_edge_to_face_bool(mesh: &MeshTopo, src: &[bool]) -> Vec<bool> {
    (0..mesh.faces_num as usize)
        .map(|f| {
            (mesh.face_offsets[f]..mesh.face_offsets[f + 1])
                .all(|e| src[mesh.corner_edges[e as usize] as usize])
        })
        .collect()
}
