//! Port of `intern/multires_subdiv.cc` — multires→subdiv settings init.

use crate::subdiv::{Settings, ToMeshSettings};
use blenders_dna::modifier_types::{eMultiresModifierFlag_ControlEdges as CTRL_EDGES, eMultiresModifierFlag_UseCrease as USE_CREASES, MultiresModifierData};

/// `bke::subdiv::vtx_boundary_interpolation_from_subsurf` — subsurf boundary
/// smooth → vtx boundary interpolation (BKE_subdiv mapping).
pub fn vtx_boundary_interpolation_from_subsurf(_boundary_smooth: i32) -> i32 {
    // PORT_PENDING precise mapping; defaults match Blender's common case.
    crate::subdiv::SUBDIV_VTX_BOUNDARY_EDGE_AND_CORNER
}

/// `bke::subdiv::fvar_interpolation_from_uv_smooth`.
pub fn fvar_interpolation_from_uv_smooth(_uv_smooth: i32) -> i32 {
    crate::subdiv::SUBDIV_FVAR_LINEAR_INTERPOLATION_ALL
}

/// `BKE_multires_subdiv_settings_init`.
pub fn bke_multires_subdiv_settings_init(settings: &mut Settings, mmd: &MultiresModifierData) {
    settings.is_simple = false;
    settings.is_adaptive = true;
    settings.level = if settings.is_simple { 1 } else { mmd.quality as i32 };
    settings.use_creases = mmd.modifier.flag & USE_CREASES as i16 != 0;
    settings.vtx_boundary_interpolation =
        vtx_boundary_interpolation_from_subsurf(mmd.boundary_smooth as i32);
    settings.fvar_linear_interpolation =
        fvar_interpolation_from_uv_smooth(mmd.uv_smooth as i32);
}

/// `BKE_multires_subdiv_mesh_settings_init` — `level` comes precomputed
/// (`multires_get_level` needs scene/object render flags — PORT_PENDING).
pub fn bke_multires_subdiv_mesh_settings_init(
    mesh_settings: &mut ToMeshSettings,
    mmd: &MultiresModifierData,
    level: i32,
    ignore_control_edges: bool,
) {
    mesh_settings.resolution = (1 << level) + 1;
    mesh_settings.use_optimal_display =
        mmd.modifier.flag & CTRL_EDGES as i16 != 0 && !ignore_control_edges;
}
