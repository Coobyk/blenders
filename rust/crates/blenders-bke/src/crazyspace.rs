//! Port of `intern/crazyspace.cc` — crazyspace (deformation-space)
//! transforms: `set_crazy_vertex_quat`,
//! `BKE_crazyspace_set_quats_mesh`/`_editmesh`,
//! `get_first_deform_matrices` (editbmesh + sculpt variants),
//! `build_sculpt`, the `*_api_*` matrix ops, and curves/greasepencil
//! deformation lookup.
//!
//! Modifier stack walking, Depsgraph and mesh/eval copies are
//! injected as callbacks; quat math and loops are ported verbatim.

use blenders_bli::offset_indices::OffsetIndices;
use blenders_math::matrix::Mat3;
use blenders_math::quaternion::Quaternion;
use blenders_math::vector::Vec3;

/// `set_crazy_vertex_quat` — `q2 * q1⁻¹` between orig and deformed
/// corner tri frames. `tan_calc_quat_v3` = normalized `tri_to_quat`.
pub fn set_crazy_vertex_quat(
    co_1: Vec3,
    co_2: Vec3,
    co_3: Vec3,
    vd_1: Vec3,
    vd_2: Vec3,
    vd_3: Vec3,
) -> Quaternion {
    let q1 = blenders_math::quaternion::tri_to_quat(co_1, co_2, co_3).0;
    let q2 = blenders_math::quaternion::tri_to_quat(vd_1, vd_2, vd_3).0;
    /* `sub_qt_qtqt(r, q2, q1)` = q2 * conjugate(q1). */
    q2 * q1.conjugated()
}

/// `BKE_crazyspace_set_quats_mesh` — per-vert quats over faces.
pub fn crazyspace_set_quats_mesh(
    positions: &[Vec3],
    faces: OffsetIndices,
    corner_verts: &[i32],
    origcos: Option<&[Vec3]>,
    mappedcos: &[Vec3],
    quats: &mut [Quaternion],
) {
    let mut vert_tag = vec![false; positions.len()];
    let positions = origcos.unwrap_or(positions);

    for i in 0..faces.len() {
        let g = faces.group(i);
        let face: Vec<usize> = (g.start()..g.one_after_last()).map(|c| c as usize).collect();
        let n = face.len();
        for (j, &corner) in face.iter().enumerate() {
            let vert = corner_verts[corner] as usize;
            if vert_tag[vert] {
                continue;
            }
            let vert_prev = corner_verts[face[(n + j - 1) % n]] as usize;
            let vert_next = corner_verts[face[(j + 1) % n]] as usize;

            let vd_prev = mappedcos[vert_prev];
            let vd_curr = mappedcos[vert];
            let vd_next = mappedcos[vert_next];
            let co_prev = positions[vert_prev];
            let co_curr = positions[vert];
            let co_next = positions[vert_next];

            quats[vert] = set_crazy_vertex_quat(
                co_curr, co_next, co_prev, vd_curr, vd_next, vd_prev,
            );
            vert_tag[vert] = true;
        }
    }
}

/// `BKE_crazyspace_set_quats_editmesh` — same loop over a BMesh;
/// loop triples injected by the caller as `(prev, curr, next)` vert
/// indices + flags.
pub fn crazyspace_set_quats_editmesh(
    loop_triples: &dyn Fn(&mut dyn FnMut(usize, usize, usize, bool /* hidden */, bool /* select */)),
    bm_vert_cos: &[Vec3],
    origcos: Option<&[Vec3]>,
    mappedcos: &[Vec3],
    use_select: bool,
    quats: &mut [Quaternion],
) {
    let mut vert_tag = vec![false; bm_vert_cos.len()];
    let positions = origcos.unwrap_or(bm_vert_cos);

    loop_triples(&mut |vert_prev, vert, vert_next, hidden, select| {
        if hidden || vert_tag[vert] || (use_select && !select) {
            return;
        }
        let vd_prev = mappedcos[vert_prev];
        let vd_curr = mappedcos[vert];
        let vd_next = mappedcos[vert_next];
        let co_prev = positions[vert_prev];
        let co_curr = positions[vert];
        let co_next = positions[vert_next];
        quats[vert] =
            set_crazy_vertex_quat(co_curr, co_next, co_prev, vd_curr, vd_next, vd_prev);
        vert_tag[vert] = true;
    });
}

/// `modifiers_disable_subsurf_temporary` — toggle
/// `eModifierMode_DisableTemporary` on Subsurf modifiers up to
/// `cage_index`. Modifier iteration is injected.
pub fn modifiers_disable_subsurf_temporary(
    cage_index: usize,
    modifiers: &mut dyn FnMut(&mut dyn FnMut(&mut dyn ModifierHandle)),
) -> bool {
    let mut changed = false;
    let mut i = 0;
    modifiers(&mut |md| {
        if i > cage_index {
            return;
        }
        i += 1;
        if md.is_subsurf() {
            md.toggle_disable_temporary();
            changed = true;
        }
    });
    changed
}

/// `ModifierHandle` — injected modifier record.
pub trait ModifierHandle {
    /// `md->type == eModifierType_Subsurf`.
    fn is_subsurf(&self) -> bool;
    /// `md->mode ^= eModifierMode_DisableTemporary`.
    fn toggle_disable_temporary(&mut self);
    /// `BKE_modifier_is_enabled` for realtime.
    fn is_enabled_realtime(&self) -> bool;
    /// `mti->type == OnlyDeform` and has `deform_matrices`.
    fn supports_deform_matrices(&self) -> bool;
    /// `mti->type == OnlyDeform` (supports deform verts).
    fn supports_deform(&self) -> bool;
    /// `mti->deform_matrices`.
    fn deform_matrices(&mut self, deformcos: &mut [Vec3], deformmats: &mut [Mat3]);
    /// `mti->deform_verts`.
    fn deform_verts(&mut self, deformcos: &mut [Vec3]);
    /// `BKE_modifier_is_correctable_deformed`.
    fn is_correctable_deformed(&self) -> bool;
    /// `is_sculpt_base_mesh && type == Multires` skip test.
    fn is_multires(&self) -> bool;
}

/// `BKE_sculpt_get_first_deform_matrices` — first OnlyDeform pass
/// then the correctable count. `modifiers()` yields each md until
/// exhaustion; `mesh_positions`/`verts_num` init the arrays.
#[allow(clippy::too_many_arguments)]
pub fn sculpt_get_first_deform_matrices<I: Iterator<Item = std::cell::RefCell<Box<dyn ModifierHandle>>>>(
    modifiers: &mut I,
    is_sculpt_base_mesh: bool,
    mesh_positions: &[Vec3],
    deformmats: &mut Vec<Mat3>,
    deformcos: &mut Vec<Vec3>,
) -> usize {
    let mut modifiers_left_num = 0;
    let mut first_pass_done = false;

    for md_cell in modifiers.by_ref() {
        let mut md = md_cell.borrow_mut();
        if !md.is_enabled_realtime() {
            continue;
        }
        if is_sculpt_base_mesh && md.is_multires() {
            continue;
        }
        if md.supports_deform_matrices() {
            if deformmats.is_empty() {
                *deformcos = mesh_positions.to_vec();
                deformmats.resize(mesh_positions.len(), Mat3::IDENTITY);
            }
            if md.supports_deform_matrices() {
                md.deform_matrices(deformcos, deformmats);
            } else {
                debug_assert!(md.supports_deform());
                break;
            }
            first_pass_done = true;
        }
    }

    /* Remaining modifiers: count correctable-deformed. */
    for md_cell in modifiers.by_ref() {
        let md = md_cell.borrow();
        if !md.is_enabled_realtime() {
            continue;
        }
        if md.supports_deform() {
            modifiers_left_num += 1;
        }
    }
    let _ = first_pass_done;
    modifiers_left_num
}

/// `BKE_crazyspace_build_sculpt` — combine deform_matrices with
/// vertex quats. Modifier loop injected like
/// `sculpt_get_first_deform_matrices`.
pub fn crazyspace_build_sculpt<I: Iterator<Item = std::cell::RefCell<Box<dyn ModifierHandle>>>>(
    mesh_positions: &[Vec3],
    faces: OffsetIndices,
    corner_verts: &[i32],
    deformmats: &mut Vec<Mat3>,
    deformcos: &mut Vec<Vec3>,
    modifiers: &mut I,
) {
    let totleft = sculpt_get_first_deform_matrices(
        modifiers,
        false,
        mesh_positions,
        deformmats,
        deformcos,
    );
    if totleft != 0 {
        if deformcos.is_empty() {
            *deformcos = mesh_positions.to_vec();
            deformmats.resize(mesh_positions.len(), Mat3::IDENTITY);
        }
        let orig_verts = deformcos.clone();
        let mut quats = vec![Quaternion::IDENTITY; mesh_positions.len()];
        crazyspace_set_quats_mesh(
            mesh_positions,
            faces,
            corner_verts,
            Some(&orig_verts),
            deformcos,
            &mut quats,
        );
        for i in 0..mesh_positions.len() {
            let qmat = quats[i].to_mat3();
            deformmats[i] = qmat * deformmats[i];
        }
    }
    if deformmats.is_empty() {
        *deformcos = mesh_positions.to_vec();
        deformmats.resize(mesh_positions.len(), Mat3::IDENTITY);
    }
}

/// `BKE_crazyspace_api_eval` guard — object must be OB_MESH and
/// caches empty.
pub fn crazyspace_api_eval_needed(imats_empty: bool, cos_empty: bool, is_mesh: bool) -> bool {
    if !imats_empty || !cos_empty {
        return false;
    }
    is_mesh
}

/// `BKE_crazyspace_api_displacement_to_deformed`.
pub fn crazyspace_api_displacement_to_deformed(
    imats: &[Mat3],
    vert: usize,
    displacement: Vec3,
) -> Vec3 {
    imats[vert] * displacement
}

/// `BKE_crazyspace_api_displacement_to_original` — inverse mul.
pub fn crazyspace_api_displacement_to_original(
    imats: &[Mat3],
    vert: usize,
    displacement: Vec3,
) -> Vec3 {
    imats[vert].inverted().unwrap_or(Mat3::IDENTITY) * displacement
}

/// `GeometryDeformation` — `bke::crazyspace` result record.
#[derive(Clone, Default)]
pub struct GeometryDeformation {
    /// `positions`.
    pub positions: Vec<Vec3>,
    /// `handle_positions_left`.
    pub handle_positions_left: Vec<Vec3>,
    /// `handle_positions_right`.
    pub handle_positions_right: Vec<Vec3>,
    /// `deform_mats`.
    pub deform_mats: Vec<Mat3>,
    /// `tangents` (grease pencil).
    pub tangents: Vec<Vec3>,
}
