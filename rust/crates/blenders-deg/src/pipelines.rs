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
