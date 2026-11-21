//! Port of `intern/subdiv_deform.cc` — `deform_coarse_vertices` plus the
//! `PtexCoord` helpers from `intern/subdiv_mesh_layout.cc` it uses.
//!
//! `eval_begin_from_mesh`/`eval_limit_points`/`eval_displacement_averaged`
//! are OpenSubdiv evaluator calls — `usize` handles (PORT_PENDING).

use blenders_bli::index_range::IndexRange;
use blenders_math::vector::Vec2;
use blenders_math::vector::Vec3;

/// `PtexCoord` (BKE_subdiv_mesh_layout.hh).
#[derive(Clone, Copy, Default)]
pub struct PtexCoord {
    pub ptex_face_index: i32,
    pub uv: Vec2,
}

/// `face_corner_ptex_coord` (subdiv_mesh_layout.cc).
pub fn face_corner_ptex_coord(face: IndexRange, face_ptex_offset: i32, corner: i32) -> PtexCoord {
    const QUAD_UVS: [Vec2; 4] = [
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.0),
    ];
    if face.size() == 4 {
        return PtexCoord { ptex_face_index: face_ptex_offset, uv: QUAD_UVS[corner as usize] };
    }
    PtexCoord { ptex_face_index: face_ptex_offset + corner, uv: Vec2::ZERO }
}

/// `mesh::face_find_corner_from_vert` — index of the face corner using vert.
fn face_find_corner_from_vert(face: IndexRange, corner_verts: &[i32], vert: i32) -> i64 {
    for i in face.iter() {
        if corner_verts[i as usize] == vert {
            return i;
        }
    }
    debug_assert!(false, "vert not in face");
    0
}

/// `face_vert_ptex_coord`.
pub fn face_vert_ptex_coord(
    face: IndexRange,
    face_ptex_offset: i32,
    corner_verts: &[i32],
    vert: i32,
) -> PtexCoord {
    let corner = face_find_corner_from_vert(face, corner_verts, vert);
    face_corner_ptex_coord(face, face_ptex_offset, (corner - face.start()) as i32)
}

/// `deform_coarse_vertices` — `vert_to_face` grouped spans + stats/allocator
/// handles; OpenSubdiv eval calls are PORT_PENDING (the gather/scatter
/// structure is verbatim).
pub struct SubdivEval {
    /// `subdiv->stats` — `SubdivStats` handle.
    pub stats: crate::subdiv_stats::SubdivStats,
    /// `subdiv->displacement_evaluator` — 0 = none.
    pub displacement_evaluator: usize,
    /// `subdiv->evaluator` — OpenSubdiv evaluator.
    pub evaluator: usize,
}

/// Evaluator hooks — `eval_begin_from_mesh`, `eval_limit_points`,
/// `eval_displacement_averaged`. Return false = "OpenSubdiv unavailable".
/// PORT_PENDING bodies (OpenSubdiv).
pub trait SubdivEvaluator {
    fn eval_begin_from_mesh(&mut self, coarse_mesh: usize, vert_positions: &[Vec3]) -> bool;
    fn eval_limit_points(&self, coords: &[PtexCoord], positions: &mut [Vec3]);
    fn eval_displacement_averaged(
        &self,
        coords: &[PtexCoord],
        displacement_offsets: &[usize],
        displacement: &mut [Vec3],
    );
    fn face_ptex_offset(&self, face: usize) -> i32;
}

/// `deform_coarse_vertices`.
pub fn deform_coarse_vertices<E: SubdivEvaluator>(
    subdiv: &mut SubdivEval,
    eval: &mut E,
    coarse_mesh: usize,
    faces: &[IndexRange],
    corner_verts: &[i32],
    vert_to_face: &[Vec<usize>],
    vert_positions: &mut Vec3Buf,
) {
    crate::subdiv_stats::stats_begin(&mut subdiv.stats, crate::subdiv_stats::SUBDIV_STATS_SUBDIV_TO_MESH);
    /* `eval_begin_from_mesh` — OpenSubdiv evaluator refresh (PORT_PENDING). */
    if !eval.eval_begin_from_mesh(coarse_mesh, &vert_positions.0) {
        if !faces.is_empty() {
            crate::subdiv_stats::stats_end(&mut subdiv.stats, crate::subdiv_stats::SUBDIV_STATS_SUBDIV_TO_MESH);
        }
        return;
    }

    let have_displacement = subdiv.displacement_evaluator != 0;
    crate::subdiv_stats::stats_begin(&mut subdiv.stats, crate::subdiv_stats::SUBDIV_STATS_SUBDIV_TO_MESH_GEOMETRY);
    /* `threading::parallel_for` — serial here (PORT_PENDING task system). */
    for vert in 0..vert_to_face.len() {
        let vert_faces = &vert_to_face[vert];
        if vert_faces.is_empty() {
            continue;
        }
        let ptex_coord = |face: usize| {
            face_vert_ptex_coord(
                faces[face],
                eval.face_ptex_offset(face),
                corner_verts,
                vert as i32,
            )
        };
        let mut verts: Vec<usize> = Vec::new();
        let mut coords: Vec<PtexCoord> = Vec::new();
        let mut displacement_offsets: Vec<usize> = vec![0];
        let mut displacement_coords: Vec<PtexCoord> = Vec::new();
        verts.push(vert);
        coords.push(ptex_coord(vert_faces[0]));
        if have_displacement {
            for &face in vert_faces {
                displacement_coords.push(ptex_coord(face));
            }
            displacement_offsets.push(displacement_coords.len());
        }
        let mut positions = vec![Vec3::ZERO; coords.len()];
        eval.eval_limit_points(&coords, &mut positions);
        if have_displacement {
            let mut displacement = vec![Vec3::ZERO; coords.len()];
            eval.eval_displacement_averaged(
                &displacement_coords,
                &displacement_offsets,
                &mut displacement,
            );
            for i in 0..positions.len() {
                positions[i] += displacement[i];
            }
        }
        /* `array_utils::scatter` — positions[i] → vert_positions[verts[i]]. */
        for (i, &v) in verts.iter().enumerate() {
            vert_positions.0[v] = positions[i];
        }
    }
    crate::subdiv_stats::stats_end(&mut subdiv.stats, crate::subdiv_stats::SUBDIV_STATS_SUBDIV_TO_MESH_GEOMETRY);
    crate::subdiv_stats::stats_end(&mut subdiv.stats, crate::subdiv_stats::SUBDIV_STATS_SUBDIV_TO_MESH);
}

/// Mutable vert-position buffer (newtype so callers can name it).
pub struct Vec3Buf(pub Vec<Vec3>);
