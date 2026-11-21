//! Port of `intern/gtest_setup.cc` — `bke::gtest_setup()`
//! (the CTest global fixture init sequence).
//!
//! Every step is a foreign subsystem init — ported as the ordered
//! call list with each step injected (`&mut dyn FnMut`), preserving
//! the exact order upstream.

/// `gtest_setup` — runs `steps` in upstream order. Names mirror the
/// C calls: `CLG_init`, `BLI_threadapi_init`, `BKE_cpp_types_init`,
/// `register_common_functions`, `BKE_idtype_init`,
/// `IDP_struct_type_init`, `BKE_modifier_init`, `seq::modifiers_init`,
/// `BKE_shaderfx_init`, `BKE_volumes_init`, `DEG_register_node_types`,
/// `BKE_callback_global_init`, `BKE_appdir_init`,
/// `BLI_task_scheduler_init`, `fftw::initialize_float`, `IMB_init`,
/// `MOV_init`, `RNA_init`, `RE_texture_rng_init`, `RE_engines_init`,
/// `node_system_init`, `BKE_brush_system_init`,
/// `BKE_particle_init_rng`, `BKE_sound_init_once`,
/// `BKE_materials_init`, `IMB_cache_init`, `BKE_icons_init(4096)`.
pub fn gtest_setup(steps: &mut dyn FnMut(&'static str)) {
    for step in [
        "CLG_init", "BLI_threadapi_init", "BKE_cpp_types_init",
        "register_common_functions", "BKE_idtype_init", "IDP_struct_type_init",
        "BKE_modifier_init", "seq_modifiers_init", "BKE_shaderfx_init",
        "BKE_volumes_init", "DEG_register_node_types", "BKE_callback_global_init",
        "BKE_appdir_init", "BLI_task_scheduler_init", "fftw_initialize_float",
        "IMB_init", "MOV_init", "RNA_init", "RE_texture_rng_init",
        "RE_engines_init", "node_system_init", "BKE_brush_system_init",
        "BKE_particle_init_rng", "BKE_sound_init_once", "BKE_materials_init",
        "IMB_cache_init", "BKE_icons_init",
    ] {
        steps(step);
    }
}
