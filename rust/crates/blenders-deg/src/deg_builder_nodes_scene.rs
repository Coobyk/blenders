//! Port of `intern/builder/deg_builder_nodes_scene.cc` —
/// scene-render/camera/parameters/compositor node building.

/// `R_DOCOMP`/`R_DOSEQ` (`DNA_scene_types.h` verbatim).
pub const R_DOCOMP: i32 = 1 << 1;
pub const R_DOSEQ: i32 = 1 << 2;

/// `DEG_ID_LINKED_*` (verbatim — mirror of
/// `deg_node_id.hh`).
pub const DEG_ID_LINKED_DIRECTLY: i32 = 0;
pub const DEG_ID_LINKED_INDIRECTLY: i32 = 1;

/// `BuilderMap::TAG_*` (verbatim — per-build dedup tags).
pub const TAG_PARAMETERS: u32 = 1 << 0;
pub const TAG_SCENE_COMPOSITOR: u32 = 1 << 1;

/// Injected builder calls — the concrete
/// `DepsgraphNodeBuilder` isn't unified yet.
pub struct SceneNodeBuildHooks<'a> {
    /// `add_id_node` → node handle.
    pub add_id_node: &'a mut dyn FnMut(usize) -> usize,
    /// `id_node->linked_state = ...`.
    pub set_linked_state: &'a mut dyn FnMut(usize, i32),
    /// `add_time_source`.
    pub add_time_source: &'a mut dyn FnMut(),
    /// `build_animdata`.
    pub build_animdata: &'a mut dyn FnMut(usize),
    /// `build_parameters`.
    pub build_parameters: &'a mut dyn FnMut(usize),
    /// `build_idproperties` (property-group handle).
    pub build_idproperties: &'a mut dyn FnMut(usize),
    /// `add_operation_node`.
    pub add_operation_node: &'a mut dyn FnMut(usize, usize, usize),
    /// `build_object`.
    pub build_object: &'a mut dyn FnMut(i32, usize, i32, bool),
    /// `build_nodetree`.
    pub build_nodetree: &'a mut dyn FnMut(usize),
    /// `build_scene_audio` / `_sequencer` / `_speakers`.
    pub build_scene_audio: &'a mut dyn FnMut(usize),
    /// `build_scene_sequencer`.
    pub build_scene_sequencer: &'a mut dyn FnMut(usize),
    /// `build_scene_speakers`.
    pub build_scene_speakers: &'a mut dyn FnMut(usize, usize),
    /// `built_map_.check_is_built_and_tag`.
    pub check_is_built_and_tag: &'a mut dyn FnMut(usize, u32) -> bool,
}

/// `build_scene_render` (verbatim).
pub fn build_scene_render(
    scene: usize,
    view_layer: usize,
    scemode: i32,
    scene_id: usize,
    scene_camera: usize,
    marker_cameras: &[usize],
    h: &mut SceneNodeBuildHooks,
) {
    let build_compositor = (scemode & R_DOCOMP) != 0;
    let build_sequencer = (scemode & R_DOSEQ) != 0;
    let id_node = (h.add_id_node)(scene_id);
    (h.set_linked_state)(id_node, DEG_ID_LINKED_DIRECTLY);
    (h.add_time_source)();
    (h.build_animdata)(scene_id);
    build_scene_parameters(scene, scene_id, scene_id, scene_id, &[], h);
    (h.build_scene_audio)(scene);
    if build_compositor {
        build_scene_compositor(scene, scene_id, &[], h);
    }
    if build_sequencer {
        (h.build_scene_sequencer)(scene);
        (h.build_scene_speakers)(scene, view_layer);
    }
    build_scene_camera(scene_camera, marker_cameras, h);
}

/// `build_scene_camera` (verbatim — camera + time-marker
/// cameras).
pub fn build_scene_camera(
    scene_camera: usize,
    marker_cameras: &[usize],
    h: &mut SceneNodeBuildHooks,
) {
    if scene_camera != 0 {
        (h.build_object)(-1, scene_camera, DEG_ID_LINKED_INDIRECTLY, true);
    }
    for camera in marker_cameras {
        if *camera != 0 && *camera != scene_camera {
            (h.build_object)(-1, *camera, DEG_ID_LINKED_INDIRECTLY, true);
        }
    }
}

/// `build_scene_parameters` (verbatim).
pub fn build_scene_parameters(
    scene: usize,
    scene_id: usize,
    id_properties: usize,
    id_system_properties: usize,
    marker_props: &[usize],
    h: &mut SceneNodeBuildHooks,
) {
    if (h.check_is_built_and_tag)(scene, TAG_PARAMETERS) {
        return;
    }
    (h.build_parameters)(scene_id);
    (h.build_idproperties)(id_properties);
    (h.build_idproperties)(id_system_properties);
    (h.add_operation_node)(scene_id, 6 /*NodeType::SCENE*/, 0 /*SCENE_EVAL*/);
    for prop in marker_props {
        (h.build_idproperties)(*prop);
    }
}

/// `build_scene_compositor` (verbatim — empty-eval op +
/// per-effect node groups).
pub fn build_scene_compositor(
    scene: usize,
    scene_id: usize,
    effect_node_groups_and_props: &[(usize, usize)],
    h: &mut SceneNodeBuildHooks,
) {
    if (h.check_is_built_and_tag)(scene, TAG_SCENE_COMPOSITOR) {
        return;
    }
    (h.add_operation_node)(
        scene_id,
        12, /* NodeType::COMPOSITOR */
        0,  /* OperationCode::COMPOSITOR_EVAL */
    );
    for (node_group, sys_props) in effect_node_groups_and_props {
        if *node_group == 0 {
            continue;
        }
        (h.build_nodetree)(*node_group);
        (h.build_idproperties)(*sys_props);
    }
}
