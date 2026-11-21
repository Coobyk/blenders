//! Port of `intern/subdiv_modifier.cc` — `BKE_subsurf_modifier_*`
//! (settings init, runtime init, GPU-eval gates).
//!
//! `update_from_mesh` (the OpenSubdiv descriptor) and GPU capability checks
//! stay PORT_PENDING.

use blenders_dna::modifier_types::{
    SubsurfModifierData, eSubsurfModifierFlag_UseCrease, eSubsurfModifierFlag_UseCustomNormals,
    eSubsurfModifierFlag_UseRecursiveSubdivision, SUBSURF_TYPE_SIMPLE,
};

use crate::multires_subdiv::{fvar_interpolation_from_uv_smooth, vtx_boundary_interpolation_from_subsurf};
use crate::subdiv::Settings;

/// `SubsurfRuntimeData` (BKE_subdiv_modifier.hh).
#[derive(Clone, Default)]
pub struct SubsurfRuntimeData {
    pub settings: Settings,
    /// `subdiv::Subdiv *subdiv_cpu` / `*subdiv_gpu`.
    pub subdiv_cpu: usize,
    pub subdiv_gpu: usize,
    /// `used_cpu`/`used_gpu` — frame countdowns.
    pub used_cpu: i32,
    pub used_gpu: i32,
    /// `Object *object` / `Mesh *mesh` cached inputs.
    pub object: usize,
    pub mesh: usize,
}

/// `BKE_subsurf_modifier_settings_init`.
pub fn bke_subsurf_modifier_settings_init(smd: &SubsurfModifierData, use_render_params: bool) -> Settings {
    let requested_levels = if use_render_params { smd.render_levels } else { smd.levels };
    let mut settings = Settings::default();
    settings.is_simple = smd.subdiv_type == SUBSURF_TYPE_SIMPLE;
    settings.is_adaptive = smd.flags & eSubsurfModifierFlag_UseRecursiveSubdivision == 0;
    settings.level = if settings.is_simple {
        1
    } else if settings.is_adaptive {
        smd.quality as i32
    } else {
        requested_levels as i32
    };
    settings.use_creases = smd.flags & eSubsurfModifierFlag_UseCrease != 0;
    settings.vtx_boundary_interpolation = vtx_boundary_interpolation_from_subsurf(smd.boundary_smooth as i32);
    settings.fvar_linear_interpolation = fvar_interpolation_from_uv_smooth(smd.uv_smooth as i32);
    settings
}

/// `BKE_subsurf_modifier_runtime_init` — returns false at level 0.
pub fn bke_subsurf_modifier_runtime_init(smd: &SubsurfModifierData, use_render_params: bool, runtime_data: &mut Option<SubsurfRuntimeData>) -> bool {
    let settings = bke_subsurf_modifier_settings_init(smd, use_render_params);
    if settings.level == 0 {
        if let Some(rt) = runtime_data {
            rt.settings = settings;
            rt.used_cpu = 0;
            rt.used_gpu = 0;
        }
        return false;
    }
    let rt = runtime_data.get_or_insert_with(SubsurfRuntimeData::default);
    rt.settings = settings;
    true
}

/// `BKE_subsurf_modifier_use_custom_loop_normals` — checks `custom_normal`
/// corner Int16_2D attr; the attribute lookup is injected until
/// `mesh->attributes()` lands (returns `(domain, type)`).
pub fn bke_subsurf_modifier_use_custom_loop_normals(
    smd: &SubsurfModifierData,
    custom_normal_attr: Option<(i32, i32)>,
    attr_domain_corner: i32,
    attr_type_int16_2d: i32,
) -> bool {
    if smd.flags & eSubsurfModifierFlag_UseCustomNormals == 0 {
        return false;
    }
    match custom_normal_attr {
        Some((domain, data_type)) => domain == attr_domain_corner && data_type == attr_type_int16_2d,
        None => false,
    }
}

/// `BKE_subsurf_modifier_has_split_normals` — `normals_domain` is the
/// mesh's `MeshNormalDomain` (Corner = caller passes true).
pub fn bke_subsurf_modifier_has_split_normals(
    smd: &SubsurfModifierData,
    custom_normal_attr: Option<(i32, i32)>,
    attr_domain_corner: i32,
    attr_type_int16_2d: i32,
    mesh_normals_domain_is_corner: bool,
) -> bool {
    bke_subsurf_modifier_use_custom_loop_normals(smd, custom_normal_attr, attr_domain_corner, attr_type_int16_2d)
        || mesh_normals_domain_is_corner
}

/// `is_subdivision_evaluation_possible_on_gpu` — GPU caps (PORT_PENDING;
/// `MAX_GPU_SUBDIV_SSBOS` gate behind a bool param).
fn is_subdivision_evaluation_possible_on_gpu(backend_none: bool, max_ssbos: i32, max_gpu_subdiv_ssbos: i32) -> bool {
    if backend_none {
        return false;
    }
    if max_ssbos < max_gpu_subdiv_ssbos {
        return false;
    }
    true
}

/// `USER_GPU_FLAG_SUBDIVISION_EVALUATION` (DNA_userdef_types.h flag —
/// injected as a bool until the userdef flag enum lands here).
/// `BKE_subsurf_modifier_force_disable_gpu_evaluation_for_mesh`.
pub fn bke_subsurf_modifier_force_disable_gpu_evaluation_for_mesh(
    smd: &SubsurfModifierData,
    user_gpu_subdivision_evaluation: bool,
    backend_none: bool,
    max_ssbos: i32,
    max_gpu_subdiv_ssbos: i32,
    custom_normal_attr: Option<(i32, i32)>,
    attr_domain_corner: i32,
    attr_type_int16_2d: i32,
    mesh_normals_domain_is_corner: bool,
) -> bool {
    if !user_gpu_subdivision_evaluation {
        return false;
    }
    if !is_subdivision_evaluation_possible_on_gpu(backend_none, max_ssbos, max_gpu_subdiv_ssbos) {
        return false;
    }
    bke_subsurf_modifier_has_split_normals(
        smd, custom_normal_attr, attr_domain_corner, attr_type_int16_2d, mesh_normals_domain_is_corner,
    )
}

/// `BKE_subsurf_modifier_can_do_gpu_subdiv`.
pub fn bke_subsurf_modifier_can_do_gpu_subdiv(
    smd: &SubsurfModifierData,
    user_gpu_subdivision_evaluation: bool,
    backend_none: bool,
    max_ssbos: i32,
    max_gpu_subdiv_ssbos: i32,
    custom_normal_attr: Option<(i32, i32)>,
    attr_domain_corner: i32,
    attr_type_int16_2d: i32,
    mesh_normals_domain_is_corner: bool,
) -> bool {
    user_gpu_subdivision_evaluation
        && is_subdivision_evaluation_possible_on_gpu(backend_none, max_ssbos, max_gpu_subdiv_ssbos)
        && !bke_subsurf_modifier_has_split_normals(
            smd, custom_normal_attr, attr_domain_corner, attr_type_int16_2d, mesh_normals_domain_is_corner,
        )
}

/// `BKE_subsurf_modifier_eval_required_mode`.
pub fn bke_subsurf_modifier_eval_required_mode(is_final_render: bool, is_edit_mode: bool) -> u32 {
    use blenders_dna::modifier_types::{eModifierMode_Editmode, eModifierMode_Realtime, eModifierMode_Render};
    if is_final_render {
        return eModifierMode_Render;
    }
    eModifierMode_Realtime | if is_edit_mode { eModifierMode_Editmode } else { 0 }
}
