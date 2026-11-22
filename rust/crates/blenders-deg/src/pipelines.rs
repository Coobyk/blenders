//! Port of `intern/builder/pipeline_{view_layer,render,all_objects,compositor,from_ids,from_collection}.cc`
/// — builder pipeline types (build steps injected).

/// `DEG_ID_LINKED_DIRECTLY` verbatim (`DEG_depsgraph.hh`).
pub const DEG_ID_LINKED_DIRECTLY: i32 = 0;
/// `DEG_ID_LINKED_INDIRECTLY`.
pub const DEG_ID_LINKED_INDIRECTLY: i32 = 1;

/// `ViewLayerBuilderPipeline::build_nodes` /
/// `build_relations` (verbatim).
pub fn view_layer_build_nodes(build_view_layer: &mut dyn FnMut(i32)) {
    build_view_layer(DEG_ID_LINKED_DIRECTLY);
}

/// `ViewLayerBuilderPipeline::build_relations`.
pub fn view_layer_build_relations(build_view_layer: &mut dyn FnMut(i32)) {
    build_view_layer(DEG_ID_LINKED_DIRECTLY);
}

/// `RenderBuilderPipeline` — sets
/// `is_render_pipeline_depsgraph` and overrides
/// `build_idproperties` to no-op on both builders.
pub struct RenderBuilderPipeline {
    /// `deg_graph_->is_render_pipeline_depsgraph`.
    pub is_render_pipeline_depsgraph: bool,
}

impl RenderBuilderPipeline {
    /// ctor (verbatim).
    pub fn new() -> Self {
        Self {
            is_render_pipeline_depsgraph: true,
        }
    }
    /// `build_idproperties` override (verbatim — no-op).
    pub fn build_idproperties(&self) {}
    /// `build_nodes` → `build_scene_render`.
    pub fn build_nodes(&self, build_scene_render: &mut dyn FnMut()) {
        build_scene_render();
    }
    /// `build_relations` → `build_scene_render`.
    pub fn build_relations(&self, build_scene_render: &mut dyn FnMut()) {
        build_scene_render();
    }
}

/// `AllObjectsBuilderPipeline` —
/// `need_pull_base_into_graph` always true (verbatim).
pub fn all_objects_need_pull_base_into_graph() -> bool {
    true
}

/* ---- `pipeline.cc` — AbstractBuilderPipeline ---- */

/// `AbstractBuilderPipeline::build` (verbatim — sanity
/// check → nodes → relations → finalize).
pub fn pipeline_build(
    debug_build_or_time: bool,
    now_seconds: &dyn Fn() -> f64,
    build_step_sanity_check: &mut dyn FnMut(),
    build_step_nodes: &mut dyn FnMut(),
    build_step_relations: &mut dyn FnMut(),
    build_step_finalize: &mut dyn FnMut(),
    need_sanity_checks: bool,
    do_sanity_checks: &mut dyn FnMut(),
    printf: &mut dyn FnMut(&str),
) {
    let mut start_time = 0.0;
    if debug_build_or_time {
        start_time = (now_seconds)();
    }
    (build_step_sanity_check)();
    (build_step_nodes)();
    (build_step_relations)();
    (build_step_finalize)();
    if need_sanity_checks {
        (do_sanity_checks)();
    }
    if debug_build_or_time {
        (printf)(&format!(
            "Depsgraph built in {} seconds.",
            (now_seconds)() - start_time
        ));
    }
}

/// `AbstractBuilderPipeline::build_step_nodes` (verbatim).
pub fn pipeline_build_step_nodes(
    node_builder_begin: &mut dyn FnMut(),
    build_nodes: &mut dyn FnMut(),
    node_builder_end: &mut dyn FnMut(),
    need_sanity_checks: bool,
    get_built_ids: &mut dyn FnMut(),
) {
    (node_builder_begin)();
    (build_nodes)();
    (node_builder_end)();
    if need_sanity_checks {
        (get_built_ids)();
    }
}

/// `AbstractBuilderPipeline::build_step_relations`
/// (verbatim — begin → build → cow + driver + audio-cache
/// → built ids).
pub fn pipeline_build_step_relations(
    relation_builder_begin: &mut dyn FnMut(),
    build_relations: &mut dyn FnMut(),
    build_copy_on_write_relations: &mut dyn FnMut(),
    build_driver_relations: &mut dyn FnMut(),
    build_scene_audio_cache_invalidation_relations: &mut dyn FnMut(),
    need_sanity_checks: bool,
    get_built_ids: &mut dyn FnMut(),
) {
    (relation_builder_begin)();
    (build_relations)();
    (build_copy_on_write_relations)();
    (build_driver_relations)();
    (build_scene_audio_cache_invalidation_relations)();
    if need_sanity_checks {
        (get_built_ids)();
    }
}

/// `AbstractBuilderPipeline::build_step_finalize`
/// (verbatim — cycles → transitive reduction → scene_cow →
/// finalize → tag → need_update_relations=false).
pub fn pipeline_build_step_finalize(
    deg_graph_detect_cycles: &mut dyn FnMut(),
    debug_value_799: bool,
    deg_graph_transitive_reduction: &mut dyn FnMut(),
    store_scene_cow: &mut dyn FnMut(),
    deg_graph_build_finalize: &mut dyn FnMut(),
    deg_graph_tag_on_visible_update: &mut dyn FnMut(bool),
    need_update_relations: &mut bool,
) {
    (deg_graph_detect_cycles)();
    if debug_value_799 {
        (deg_graph_transitive_reduction)();
    }
    (store_scene_cow)();
    (deg_graph_build_finalize)();
    (deg_graph_tag_on_visible_update)(false);
    *need_update_relations = false;
}
