//! Port of `intern/builder/deg_builder_relations_scene.cc`
/// — scene relation building.

use crate::deg_builder_nodes_scene::{DEG_ID_LINKED_INDIRECTLY, R_DOCOMP, R_DOSEQ,
    TAG_PARAMETERS, TAG_SCENE_COMPOSITOR};
pub use crate::deg_builder_nodes_scene::{
    build_scene_camera, build_scene_compositor, build_scene_parameters, build_scene_render,
    SceneNodeBuildHooks as SceneRelationBuildHooks,
};

/// `DepsgraphRelationBuilder::build_scene_render`
/// (verbatim — same sequence as the node builder but via
/// relation-builder hooks; callers pass relation hooks).
pub struct RelSceneHooks<'a> {
    /// `build_scene_parameters` relation side.
    pub build_parameters: &'a mut dyn FnMut(usize),
    /// `build_animdata`.
    pub build_animdata: &'a mut dyn FnMut(usize),
    /// `build_scene_audio`.
    pub build_scene_audio: &'a mut dyn FnMut(usize),
    /// `build_scene_compositor`.
    pub build_scene_compositor: &'a mut dyn FnMut(usize),
    /// `build_scene_sequencer`.
    pub build_scene_sequencer: &'a mut dyn FnMut(usize),
    /// `build_scene_speakers`.
    pub build_scene_speakers: &'a mut dyn FnMut(usize, usize),
    /// `build_object` (relation-builder overload, no flags).
    pub build_object: &'a mut dyn FnMut(usize),
    /// `build_nodetree`.
    pub build_nodetree: &'a mut dyn FnMut(usize),
    /// `build_idproperties`.
    pub build_idproperties: &'a mut dyn FnMut(usize),
    /// `add_relation` (from key, to key, name).
    pub add_relation: &'a mut dyn FnMut(usize, usize, &str),
    /// `check_is_built_and_tag`.
    pub check_is_built_and_tag: &'a mut dyn FnMut(usize, u32) -> bool,
    /// `bke::compositor::add_depsgraph_relations` —
    /// PORT_PENDING compositor integration.
    pub add_compositor_depsgraph_relations: &'a mut dyn FnMut(usize),
}

/// `build_scene_render` (verbatim).
pub fn rel_build_scene_render(
    scene: usize,
    view_layer: usize,
    scemode: i32,
    scene_id: usize,
    scene_camera: usize,
    marker_cameras: &[usize],
    h: &mut RelSceneHooks,
) {
    let build_compositor = (scemode & R_DOCOMP) != 0;
    let build_sequencer = (scemode & R_DOSEQ) != 0;
    rel_build_scene_parameters(scene, scene_id, scene_id, scene_id, &[], h);
    (h.build_animdata)(scene_id);
    (h.build_scene_audio)(scene);
    if build_compositor {
        rel_build_scene_compositor(scene, scene_id, &[], h);
    }
    if build_sequencer {
        (h.build_scene_sequencer)(scene);
        (h.build_scene_speakers)(scene, view_layer);
    }
    rel_build_scene_camera(scene_camera, marker_cameras, h);
}

/// `build_scene_camera` (verbatim).
pub fn rel_build_scene_camera(
    scene_camera: usize,
    marker_cameras: &[usize],
    h: &mut RelSceneHooks,
) {
    if scene_camera != 0 {
        (h.build_object)(scene_camera);
    }
    for camera in marker_cameras {
        if *camera != 0 && *camera != scene_camera {
            (h.build_object)(*camera);
        }
    }
}

/// `build_scene_parameters` (verbatim — idprops +
/// PARAMETERS_EXIT → SCENE relation).
pub fn rel_build_scene_parameters(
    scene: usize,
    scene_id: usize,
    id_properties: usize,
    id_system_properties: usize,
    marker_props: &[usize],
    h: &mut RelSceneHooks,
) {
    if (h.check_is_built_and_tag)(scene, TAG_PARAMETERS) {
        return;
    }
    (h.build_idproperties)(id_properties);
    (h.build_idproperties)(id_system_properties);
    (h.build_parameters)(scene_id);
    /* `parameters_eval_key`/`scene_eval_key` resolved to
     * operation/component handles by the caller. */
    (h.add_relation)(scene_id, scene_id, "Parameters -> Scene Eval");
    for prop in marker_props {
        (h.build_idproperties)(*prop);
    }
}

/// `build_scene_compositor` (verbatim — NTREE_OUTPUT →
/// COMPOSITOR per effect + compositor's own relations).
pub fn rel_build_scene_compositor(
    scene: usize,
    scene_id: usize,
    effect_node_groups_and_output_ops: &[(usize, usize)],
    h: &mut RelSceneHooks,
) {
    if (h.check_is_built_and_tag)(scene, TAG_SCENE_COMPOSITOR) {
        return;
    }
    for (node_group, node_output_op) in effect_node_groups_and_output_ops {
        if *node_group == 0 {
            continue;
        }
        (h.add_relation)(*node_output_op, scene_id, "NTree Output -> Compositor");
        (h.build_nodetree)(*node_group);
        (h.add_compositor_depsgraph_relations)(*node_output_op);
    }
}

