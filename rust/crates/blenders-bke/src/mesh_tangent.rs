//! Port of `intern/mesh_tangent.cc` — tangent-space calculation for
//! meshes (mikktspace driver for tris/quads meshes and for
//! `corner_tris` with quad detection).
//!
//! `mikk::Mikktspace::genTangSpace` is injected (`PORT_PENDING`:
//! `intern/mikktspace` unported).

use blenders_bli::offset_indices::OffsetIndices;
use blenders_math::geom::{map_to_sphere, normal_quad, normal_tri};
use blenders_math::vector::{Vec2, Vec3};

/// `MeshToTangentQuadsTris` — direct mesh→tangent adapter for
/// tris/quads meshes (no tessellation).
pub struct MeshToTangentQuadsTris<'a> {
    pub positions: &'a [Vec3],
    pub faces: OffsetIndices<'a>,
    pub corner_verts: &'a [i32],
    pub corner_normals: &'a [Vec3],
    pub uv_map: &'a [Vec2],
    pub tangents: &'a mut [[f32; 4]],
}

impl MeshToTangentQuadsTris<'_> {
    /// `GetNumFaces`.
    pub fn get_num_faces(&self) -> usize {
        self.faces.len()
    }

    /// `GetNumVerticesOfFace`.
    pub fn get_num_vertices_of_face(&self, face_num: usize) -> usize {
        self.faces.group(face_num).size() as usize
    }

    /// `GetPosition`.
    pub fn get_position(&self, face_num: usize, vert_num: usize) -> Vec3 {
        let loop_idx = (self.faces.group(face_num).start() + vert_num as i64) as usize;
        self.positions[self.corner_verts[loop_idx] as usize]
    }

    /// `GetTexCoord`.
    pub fn get_tex_coord(&self, face_num: usize, vert_num: usize) -> Vec3 {
        let uv = self.uv_map[(self.faces.group(face_num).start() + vert_num as i64) as usize];
        Vec3::from([uv[0], uv[1], 1.0])
    }

    /// `GetNormal`.
    pub fn get_normal(&self, face_num: usize, vert_num: usize) -> Vec3 {
        self.corner_normals[(self.faces.group(face_num).start() + vert_num as i64) as usize]
    }

    /// `SetTangentSpace`.
    pub fn set_tangent_space(&mut self, face_num: usize, vert_num: usize, t: Vec3, orientation: bool) {
        let loop_idx = (self.faces.group(face_num).start() + vert_num as i64) as usize;
        self.tangents[loop_idx] = [t[0], t[1], t[2], if orientation { 1.0 } else { -1.0 }];
    }
}

/// `calc_uv_tangent_tris_quads` — report error on faces > 4 verts.
/// `gen_tang_space` = `mikk::Mikktspace<MeshToTangentQuadsTris>::genTangSpace`.
#[allow(clippy::too_many_arguments)]
pub fn calc_uv_tangent_tris_quads(
    vert_positions: &[Vec3],
    faces: OffsetIndices,
    corner_verts: &[i32],
    corner_normals: &[Vec3],
    uv_map: &[Vec2],
    results: &mut [[f32; 4]],
    gen_tang_space: &mut dyn FnMut(&mut MeshToTangentQuadsTris),
    report_error: &mut dyn FnMut(&str),
) {
    let mut mesh_to_tangent = MeshToTangentQuadsTris {
        positions: vert_positions,
        faces,
        corner_verts,
        corner_normals,
        uv_map,
        tangents: results,
    };

    /* First check we do have a tris/quads only mesh. */
    for i in 0..faces.len() {
        if faces.group(i).size() > 4 {
            report_error("Tangent space can only be computed for tris/quads, aborting");
            return;
        }
    }

    gen_tang_space(&mut mesh_to_tangent);
}

/* Necessary complexity to handle corner_tris as quads for correct
 * tangents. `USE_TRI_DETECT_QUADS` always on upstream. */

/// `SGLSLMeshToTangent` — tessellated-mesh adapter with quad
/// detection through `face_as_quad_map`.
pub struct SglslMeshToTangent<'a> {
    pub positions: &'a [Vec3],
    pub faces: OffsetIndices<'a>,
    pub corner_verts: &'a [i32],
    pub corner_tris: &'a [[i32; 3]],
    pub tri_faces: &'a [i32],
    pub vert_normals: &'a [Vec3],
    pub face_normals: &'a [Vec3],
    pub corner_normals: &'a [Vec3],
    pub uv_map: &'a [Vec2],
    pub orco: &'a [Vec3],
    pub sharp_faces: &'a [bool],
    pub tangents: &'a mut [[f32; 4]],
    /* `USE_TRI_DETECT_QUADS`: map from 'fake' face index to
     * corner_tris, quads point to the first corner_tris of the quad. */
    pub face_as_quad_map: Option<&'a [i32]>,
    pub num_face_as_quad_map: usize,
}

impl SglslMeshToTangent<'_> {
    /// `GetNumFaces`.
    pub fn get_num_faces(&self) -> usize {
        self.num_face_as_quad_map
    }

    /// `GetNumVerticesOfFace`.
    pub fn get_num_vertices_of_face(&self, face_num: usize) -> usize {
        if let Some(map) = self.face_as_quad_map {
            let face_index = self.tri_faces[map[face_num] as usize] as usize;
            if self.faces.group(face_index).size() == 4 {
                return 4;
            }
        }
        3
    }

    /// `GetLoop` — resolves quad-aware loop index; returns
    /// (loop_index, tri, face_index).
    fn get_loop(&self, face_num: usize, vert_num: usize) -> (usize, [i32; 3], usize) {
        if let Some(map) = self.face_as_quad_map {
            let tri = self.corner_tris[map[face_num] as usize];
            let face_index = self.tri_faces[map[face_num] as usize] as usize;
            if self.faces.group(face_index).size() == 4 {
                let range = self.faces.group(face_index);
                let loop_idx = range.start() + vert_num as i64;
                return (loop_idx as usize, tri, face_index);
            }
            return (tri[vert_num] as usize, tri, face_index);
        }
        let tri = self.corner_tris[face_num];
        let face_index = self.tri_faces[face_num] as usize;
        (tri[vert_num] as usize, tri, face_index)
    }

    /// `GetPosition`.
    pub fn get_position(&self, face_num: usize, vert_num: usize) -> Vec3 {
        let (loop_index, _, _) = self.get_loop(face_num, vert_num);
        self.positions[self.corner_verts[loop_index] as usize]
    }

    /// `GetTexCoord` — uv map or orco→sphere.
    pub fn get_tex_coord(&self, face_num: usize, vert_num: usize) -> Vec3 {
        let (loop_index, _, _) = self.get_loop(face_num, vert_num);
        if !self.uv_map.is_empty() {
            let uv = self.uv_map[loop_index];
            return Vec3::from([uv[0], uv[1], 1.0]);
        }
        let l_orco = self.orco[self.corner_verts[loop_index] as usize];
        let uv = map_to_sphere(l_orco);
        Vec3::from([uv[0], uv[1], 1.0])
    }

    /// `GetNormal` — corner, face (sharp), or vert normal.
    pub fn get_normal(&self, face_num: usize, vert_num: usize) -> Vec3 {
        let (loop_index, tri, face_index) = self.get_loop(face_num, vert_num);
        if !self.corner_normals.is_empty() {
            return self.corner_normals[loop_index];
        }
        if !self.sharp_faces.is_empty() && self.sharp_faces[face_index] {
            /* flat */
            if !self.face_normals.is_empty() {
                return self.face_normals[face_index];
            }
            let face = self.faces.group(face_index);
            if face.size() == 4 {
                let p = |i: i64| self.positions
                    [self.corner_verts[(face.start() + i) as usize] as usize];
                return normal_quad(p(0), p(1), p(2), p(3)).0;
            }
            let p = |i: usize| self.positions[self.corner_verts[tri[i] as usize] as usize];
            return normal_tri(p(0), p(1), p(2)).0;
        }
        self.vert_normals[self.corner_verts[loop_index] as usize]
    }

    /// `SetTangentSpace`.
    pub fn set_tangent_space(&mut self, face_num: usize, vert_num: usize, t: Vec3, orientation: bool) {
        let (loop_index, _, _) = self.get_loop(face_num, vert_num);
        self.tangents[loop_index] = [t[0], t[1], t[2], if orientation { 1.0 } else { -1.0 }];
    }
}

/// `calc_face_as_quad_map` — Some(map) when ngons/quads exist.
pub fn calc_face_as_quad_map(
    faces: OffsetIndices,
    corner_tris: &[[i32; 3]],
    corner_tri_faces: &[i32],
) -> (usize, Option<Vec<i32>>) {
    if corner_tris.len() != faces.len() {
        /* Over allocate, since we don't know how many ngon or quads
         * we have. */
        let mut face_as_quad_map = vec![0i32; corner_tris.len()];
        /* Map fake face index to corner_tris. */
        let mut k = 0usize;
        let mut j = 0usize;
        while j < corner_tris.len() {
            face_as_quad_map[k] = j as i32;
            k += 1;
            /* step over all quads */
            if faces.group(corner_tri_faces[j] as usize).size() == 4 {
                j += 1; /* Skips the next corner_tri. */
            }
            j += 1;
        }
        return (k, Some(face_as_quad_map));
    }
    (corner_tris.len(), None)
}

/// `calc_uv_tangents` — per-UV-map tangent arrays.
/// `gen_tang_space` = `mikk::Mikktspace<SGLSLMeshToTangent>::genTangSpace`.
#[allow(clippy::too_many_arguments)]
pub fn calc_uv_tangents(
    vert_positions: &[Vec3],
    faces: OffsetIndices,
    corner_verts: &[i32],
    corner_tris: &[[i32; 3]],
    corner_tri_faces: &[i32],
    sharp_faces: &[bool],
    vert_normals: &[Vec3],
    face_normals: &[Vec3],
    corner_normals: &[Vec3],
    uv_maps: &[&[Vec2]],
    gen_tang_space: &mut dyn FnMut(&mut SglslMeshToTangent),
) -> Vec<Vec<[f32; 4]>> {
    if corner_tris.is_empty() {
        return Vec::new();
    }

    let (num_face_as_quad_map, face_as_quad_map) =
        calc_face_as_quad_map(faces, corner_tris, corner_tri_faces);

    let mut results = Vec::with_capacity(uv_maps.len());
    for uv_map in uv_maps {
        let mut tangents = vec![[0.0f32; 4]; corner_verts.len()];
        {
            let mut mesh2tangent = SglslMeshToTangent {
                positions: vert_positions,
                faces,
                corner_verts,
                corner_tris,
                tri_faces: corner_tri_faces,
                sharp_faces,
                vert_normals,
                face_normals,
                corner_normals,
                uv_map,
                /* `orco = vert_positions` when uv used. */
                orco: vert_positions,
                tangents: &mut tangents,
                face_as_quad_map: face_as_quad_map.as_deref(),
                num_face_as_quad_map,
            };
            gen_tang_space(&mut mesh2tangent);
        }
        results.push(tangents);
    }

    results
}

/// `calc_orco_tangents` — sphere-mapped orco tangents.
#[allow(clippy::too_many_arguments)]
pub fn calc_orco_tangents(
    vert_positions: &[Vec3],
    faces: OffsetIndices,
    corner_verts: &[i32],
    corner_tris: &[[i32; 3]],
    corner_tri_faces: &[i32],
    sharp_faces: &[bool],
    vert_normals: &[Vec3],
    face_normals: &[Vec3],
    corner_normals: &[Vec3],
    vert_orco: &[Vec3],
    gen_tang_space: &mut dyn FnMut(&mut SglslMeshToTangent),
) -> Vec<[f32; 4]> {
    if corner_tris.is_empty() {
        return Vec::new();
    }

    let (num_face_as_quad_map, face_as_quad_map) =
        calc_face_as_quad_map(faces, corner_tris, corner_tri_faces);

    let mut results = vec![[0.0f32; 4]; corner_verts.len()];
    {
        let mut mesh2tangent = SglslMeshToTangent {
            positions: vert_positions,
            faces,
            corner_verts,
            corner_tris,
            tri_faces: corner_tri_faces,
            vert_normals,
            face_normals,
            corner_normals,
            sharp_faces,
            uv_map: &[],
            orco: vert_orco,
            tangents: &mut results,
            face_as_quad_map: face_as_quad_map.as_deref(),
            num_face_as_quad_map,
        };
        gen_tang_space(&mut mesh2tangent);
    }

    results
}
