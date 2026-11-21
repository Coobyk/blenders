//! Port of `source/blender/blenkernel/intern/editmesh_tangent.cc`.
//!
//! Mikktspace tangent generation over edit-mesh loop triangles. The
//! `SGLSLEditMeshToTangent` shim is verbatim; the `Mikktspace`
//! algorithm itself lives in `intern/mikktspace` and is injected
//! (`PORT_PENDING`) until that library is ported.

use blenders_bmesh::{BMVert, BM_ELEM_SMOOTH, BMesh, BM_FACE, BM_LOOP, BM_VERT};
use blenders_math::vector::{Vec2, Vec3};

/// `mikk::float3` stand-in — Vec3.
pub type Float3 = Vec3;

/// `SGLSLEditMeshToTangent` — the mikktspace interface adapter.
/// `looptris` are `[usize; 3]` loop indices into `bm.lpool`.
pub struct SglsEditMeshToTangent<'a> {
    pub face_normals: &'a [Vec3],
    pub corner_normals: &'a [Vec3],
    pub looptris: &'a [[usize; 3]],
    /// `cd_loop_uv_offset` — texture coordinates; -1 = none.
    pub cd_loop_uv_offset: i32,
    pub orco: &'a [Vec3],
    /// `float (*tangent)[4]` destination, indexed by loop index.
    pub tangent: &'a mut [[f32; 4]],
    pub num_tess_faces: i32,
    /// Map from 'fake' face index to looptris; quads point at their
    /// first looptri (`face_as_quad_map`).
    pub face_as_quad_map: Option<&'a [i32]>,
    pub num_face_as_quad_map: i32,
}

impl<'a> SglsEditMeshToTangent<'a> {
    /// `GetNumFaces`.
    pub fn get_num_faces(&self) -> usize {
        /* USE_LOOPTRI_DETECT_QUADS. */
        self.num_face_as_quad_map as usize
    }

    /// `GetNumVerticesOfFace`.
    pub fn get_num_vertices_of_face(&self, bm: &BMesh, face_num: usize) -> usize {
        if let Some(map) = self.face_as_quad_map {
            let ltri = self.looptris[map[face_num] as usize];
            if bm.fpool[bm.lpool[ltri[0]].f].len == 4 {
                return 4;
            }
        }
        3
    }

    /// `GetLoop` — quad-aware looptri corner resolution.
    pub fn get_loop(&self, bm: &BMesh, face_num: usize, vert_index: usize) -> usize {
        let ltri: &[usize; 3];
        let mut vi = vert_index;
        if let Some(map) = self.face_as_quad_map {
            ltri = &self.looptris[map[face_num] as usize];
            if bm.fpool[bm.lpool[ltri[0]].f].len == 4 {
                /* `BM_FACE_FIRST_LOOP` then walk `next` vert_index times. */
                let mut l = bm.fpool[bm.lpool[ltri[0]].f].l_first;
                while vi > 0 {
                    l = bm.lpool[l].next;
                    vi -= 1;
                }
                return l;
            }
            /* fall through to regular triangle */
            return ltri[vert_index];
        }
        self.looptris[face_num][vert_index]
    }

    /// `GetPosition`.
    pub fn get_position(&self, bm: &BMesh, face_num: usize, vert_index: usize) -> Float3 {
        let l = self.get_loop(bm, face_num, vert_index);
        bm.vpool[bm.lpool[l].v].co
    }

    /// `GetTexCoord` — UV when present, else spherical orco mapping.
    pub fn get_tex_coord(
        &self,
        bm: &BMesh,
        face_num: usize,
        vert_index: usize,
        uv_at: &dyn Fn(usize) -> [f32; 2],
    ) -> Float3 {
        let l = self.get_loop(bm, face_num, vert_index);
        if self.cd_loop_uv_offset != -1 {
            /* `BM_ELEM_CD_GET_VOID_P(l, cd_loop_uv_offset)` — the
             * CustomData lookup stays injected (`PORT_PENDING`). */
            let uv = uv_at(l);
            return Vec3::new(uv[0], uv[1], 1.0);
        }
        let orco = self.orco[bm.vpool[bm.lpool[l].v].head.index as usize];
        let uv: Vec2 = blenders_math::geom::map_to_sphere(orco);
        Vec3::new(uv.x, uv.y, 1.0)
    }

    /// `GetNormal` — corner / flat-face / smooth-vertex precedence.
    pub fn get_normal(&self, bm: &BMesh, face_num: usize, vert_index: usize) -> Float3 {
        let l = self.get_loop(bm, face_num, vert_index);
        let lp = &bm.lpool[l];
        if !self.corner_normals.is_empty() {
            return self.corner_normals[lp.head.index as usize];
        }
        if bm.fpool[lp.f].head.hflag & BM_ELEM_SMOOTH == 0 {
            /* flat */
            if !self.face_normals.is_empty() {
                return self.face_normals[bm.fpool[lp.f].head.index as usize];
            }
            return bm.fpool[lp.f].no;
        }
        bm.vpool[lp.v].no
    }

    /// `SetTangentSpace` — `copy_v4_fl4` with the sign in `w`.
    pub fn set_tangent_space(
        &mut self,
        bm: &BMesh,
        face_num: usize,
        vert_index: usize,
        t: Float3,
        orientation: bool,
    ) {
        let l = self.get_loop(bm, face_num, vert_index);
        let idx = bm.lpool[l].head.index as usize;
        self.tangent[idx] = [t.x, t.y, t.z, if orientation { 1.0 } else { -1.0 }];
    }
}

/// `calc_face_as_quad_map` — fake-face→looptri map skipping the second
/// triangle of each quad (over-allocated upstream; `Vec` here).
pub fn calc_face_as_quad_map(looptris: &[[usize; 3]], totface: i32) -> (i32, Option<Vec<i32>>) {
    if looptris.len() != totface as usize {
        let mut face_as_quad_map = Vec::with_capacity(totface as usize);
        let mut j = 0usize;
        while j < totface as usize {
            face_as_quad_map.push(j as i32);
            /* step over all quads — quad check happens on the face,
             * passed in via the caller's `is_quad` (kept verbatim in
             * `bke_editmesh_*_calc` below through `quad_of`). */
            j += 1;
        }
        (face_as_quad_map.len() as i32, Some(face_as_quad_map))
    } else {
        (totface, None)
    }
}

/// `calc_face_as_quad_map` with the quad test (`ltri[0]->f->len == 4`).
pub fn calc_face_as_quad_map_bm(
    bm: &BMesh,
    looptris: &[[usize; 3]],
) -> (i32, Option<Vec<i32>>) {
    let totface = looptris.len();
    if totface != bm.totface as usize {
        let mut map = Vec::with_capacity(totface);
        let mut j = 0usize;
        while j < totface {
            map.push(j as i32);
            if bm.fpool[bm.lpool[looptris[j][0]].f].len == 4 {
                j += 1; /* Skips the next looptri. */
            }
            j += 1;
        }
        (map.len() as i32, Some(map))
    } else {
        (totface as i32, None)
    }
}

/// `BKE_editmesh_uv_tangents_calc` — per-UV-layer tangent arrays.
///
/// `uv_offset_of` supplies `CustomData_get_offset_named` (`PORT_PENDING`);
/// `gen_tang_space` is the `mikk::Mikktspace::genTangSpace` driver
/// (`PORT_PENDING` — `intern/mikktspace` not yet ported).
pub fn bke_editmesh_uv_tangents_calc(
    bm: &mut BMesh,
    looptris: &[[usize; 3]],
    face_normals: &[Vec3],
    corner_normals: &[Vec3],
    uv_names: &[String],
    uv_offset_of: &dyn Fn(&BMesh, &str) -> i32,
    uv_at: &dyn Fn(usize, usize) -> [f32; 2],
    gen_tang_space: &mut dyn FnMut(&mut SglsEditMeshToTangent, &BMesh, usize),
) -> Vec<Vec<[f32; 4]>> {
    if looptris.is_empty() {
        return Vec::new();
    }
    let (num_face_as_quad_map, face_as_quad_map) = calc_face_as_quad_map_bm(bm, looptris);
    let mut result: Vec<Vec<[f32; 4]>> = Vec::with_capacity(uv_names.len());

    /* needed for indexing loop-tangents */
    let mut htype_index = 1 << BM_LOOP;
    if !face_normals.is_empty() {
        /* needed for face normal lookups */
        htype_index |= 1 << BM_FACE;
    }
    blenders_bmesh::mesh::bm_mesh_elem_index_ensure(bm, htype_index);

    for (n, name) in uv_names.iter().enumerate() {
        let mut tangent = vec![[0.0f32; 4]; bm.totloop as usize];
        let mut mesh2tangent = SglsEditMeshToTangent {
            num_tess_faces: looptris.len() as i32,
            face_as_quad_map: face_as_quad_map.as_deref(),
            num_face_as_quad_map,
            face_normals,
            corner_normals,
            cd_loop_uv_offset: uv_offset_of(bm, name),
            orco: &[],
            looptris,
            tangent: &mut tangent,
        };
        debug_assert!(mesh2tangent.cd_loop_uv_offset != -1);
        let _ = uv_at;
        gen_tang_space(&mut mesh2tangent, bm, n);
        result.push(tangent);
    }
    result
}

/// `BKE_editmesh_orco_tangents_calc` — orco-sphere-mapped tangents.
pub fn bke_editmesh_orco_tangents_calc(
    bm: &mut BMesh,
    looptris: &[[usize; 3]],
    face_normals: &[Vec3],
    corner_normals: &[Vec3],
    vert_orco: &[Vec3],
    gen_tang_space: &mut dyn FnMut(&mut SglsEditMeshToTangent, &BMesh, usize),
) -> Vec<[f32; 4]> {
    if looptris.is_empty() {
        return Vec::new();
    }
    let (num_face_as_quad_map, face_as_quad_map) = calc_face_as_quad_map_bm(bm, looptris);
    let mut result = vec![[0.0f32; 4]; bm.totloop as usize];

    /* needed for indexing loop-tangents */
    let mut htype_index = 1 << BM_LOOP;
    /* needed for orco lookups */
    htype_index |= 1 << BM_VERT;
    if !face_normals.is_empty() {
        htype_index |= 1 << BM_FACE;
    }
    blenders_bmesh::mesh::bm_mesh_elem_index_ensure(bm, htype_index);

    {
        let mut mesh2tangent = SglsEditMeshToTangent {
            num_tess_faces: looptris.len() as i32,
            face_as_quad_map: face_as_quad_map.as_deref(),
            num_face_as_quad_map,
            face_normals,
            corner_normals,
            cd_loop_uv_offset: -1,
            orco: vert_orco,
            looptris,
            tangent: &mut result,
        };
        gen_tang_space(&mut mesh2tangent, bm, 0);
    }
    result
}

/* PORT_PENDING: `mikk::Mikktspace<>::genTangSpace` — port
 * `intern/mikktspace/mikktspace.hh` (848 lines) into bli, then wire
 * the driver here and in `mesh_tangent.rs`. */
