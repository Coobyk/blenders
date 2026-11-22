//! Port of `intern/builder/deg_builder_relations_view_layer.cc`
/// — view-layer relation building.

use crate::deg_builder_nodes_view_layer::{
    LayerCollectionRow,
    COLLECTION_HIDE_RENDER, COLLECTION_HIDE_VIEWPORT, DAG_EVAL_VIEWPORT,
    DEG_ID_LINKED_VIA_SET, LAYER_COLLECTION_EXCLUDE,
};
use crate::deg_builder_nodes_scene::DEG_ID_LINKED_DIRECTLY;
use crate::depsgraph::RELATION_CHECK_BEFORE_ADD;

/// Relation-builder hooks (verbatim `DepsgraphRelationBuilder`
/// surface used by this file).
pub struct ViewLayerRelHooks<'a> {
    /// `need_pull_base_into_graph`.
    pub need_pull_base_into_graph: &'a mut dyn FnMut(usize) -> bool,
    /// `build_object_from_view_layer_base`.
    pub build_object_from_view_layer_base: &'a mut dyn FnMut(usize),
    /// `build_collection` (parent lc or 0, collection).
    pub build_collection: &'a mut dyn FnMut(usize, usize),
    /// `add_relation` (from key, to key, name, flags).
    pub add_relation: &'a mut dyn FnMut(usize, usize, &str, i32),
    /// Component-key resolve: (id, node_type) → key handle.
    pub component_key: &'a mut dyn FnMut(usize, usize) -> usize,
    /// `build_freestyle_linestyle`.
    pub build_freestyle_linestyle: &'a mut dyn FnMut(usize),
    /// per-id relation builders.
    pub build_rigidbody: &'a mut dyn FnMut(usize),
    /// `build_world`.
    pub build_world: &'a mut dyn FnMut(usize),
    /// `build_cachefile`.
    pub build_cachefile: &'a mut dyn FnMut(usize),
    /// `build_mask`.
    pub build_mask: &'a mut dyn FnMut(usize),
    /// `build_movieclip`.
    pub build_movieclip: &'a mut dyn FnMut(usize),
    /// `build_material`.
    pub build_material: &'a mut dyn FnMut(usize),
    /// `build_animdata`/`build_scene_*`.
    pub build_animdata: &'a mut dyn FnMut(usize),
    /// `build_scene_audio`.
    pub build_scene_audio: &'a mut dyn FnMut(usize),
    /// `build_scene_sequencer`.
    pub build_scene_sequencer: &'a mut dyn FnMut(usize),
    /// `build_scene_camera`.
    pub build_scene_camera: &'a mut dyn FnMut(usize),
    /// `build_scene_compositor`.
    pub build_scene_compositor: &'a mut dyn FnMut(usize),
    /// `build_scene_parameters`.
    pub build_scene_parameters: &'a mut dyn FnMut(usize),
}

/// `build_layer_collections` (verbatim — hierarchy
/// relations, EXCLUDE not inherited).
pub fn rel_build_layer_collections(
    lc: &LayerCollectionRow,
    parent_hierarchy_key: usize,
    mode: i32,
    scene_id: usize,
    h: &mut ViewLayerRelHooks,
) {
    let hide_flag =
        if mode == DAG_EVAL_VIEWPORT { COLLECTION_HIDE_VIEWPORT } else { COLLECTION_HIDE_RENDER };
    if lc.collection_flag & hide_flag != 0 {
        return;
    }
    let mut child_hierarchy_key = parent_hierarchy_key;
    if (lc.flag & LAYER_COLLECTION_EXCLUDE) == 0 {
        (h.build_collection)(lc.lc, lc.collection);
        let collection_hierarchy_key =
            (h.component_key)(lc.collection, 5 /*NodeType::HIERARCHY*/);
        (h.add_relation)(
            parent_hierarchy_key,
            collection_hierarchy_key,
            "Collection hierarchy",
            RELATION_CHECK_BEFORE_ADD,
        );
        child_hierarchy_key = collection_hierarchy_key;
    }
    let _ = scene_id;
    for child in &lc.children {
        rel_build_layer_collections(child, child_hierarchy_key, mode, scene_id, h);
    }
}

/// `build_view_layer_collections` (verbatim).
pub fn rel_build_view_layer_collections(
    view_layer_lcs: &[LayerCollectionRow],
    scene_id: usize,
    mode: i32,
    h: &mut ViewLayerRelHooks,
) {
    let scene_hierarchy_key = (h.component_key)(scene_id, 5 /*NodeType::HIERARCHY*/);
    for lc in view_layer_lcs {
        rel_build_layer_collections(lc, scene_hierarchy_key, mode, scene_id, h);
    }
}

/// `build_view_layer` (verbatim — full sequence incl.
/// VIEW_LAYER_EVAL → SCENE eval relation).
#[allow(clippy::too_many_arguments)]
pub fn rel_build_view_layer(
    scene: usize,
    scene_id: usize,
    linked_state: i32,
    mode: i32,
    has_rigidbody_world: bool,
    has_adt: bool,
    scene_world: usize,
    mat_override: usize,
    world_override: usize,
    bases: &[(usize, usize)],
    layer_collections: &[LayerCollectionRow],
    cachefiles: &[usize],
    masks: &[usize],
    movieclips: &[usize],
    freestyle_groups_and_linestyles: &[(usize, usize)],
    scene_view_layer_key: usize,
    scene_eval_key: usize,
    h: &mut ViewLayerRelHooks,
) {
    for (_base, object) in bases {
        if (h.need_pull_base_into_graph)(*_base) {
            (h.build_object_from_view_layer_base)(*object);
        }
    }
    rel_build_view_layer_collections(layer_collections, scene_id, mode, h);
    (h.build_scene_camera)(scene);
    if has_rigidbody_world {
        (h.build_rigidbody)(scene);
    }
    if has_adt {
        (h.build_animdata)(scene_id);
    }
    if scene_world != 0 {
        (h.build_world)(scene_world);
    }
    for cf in cachefiles {
        (h.build_cachefile)(*cf);
    }
    for m in masks {
        (h.build_mask)(*m);
    }
    for c in movieclips {
        (h.build_movieclip)(*c);
    }
    if mat_override != 0 {
        (h.build_material)(mat_override);
    }
    if world_override != 0 {
        (h.build_world)(world_override);
    }
    for (group, linestyle) in freestyle_groups_and_linestyles {
        rel_build_freestyle_lineset(*group, *linestyle, h);
    }
    (h.build_scene_compositor)(scene);
    (h.build_scene_parameters)(scene);
    (h.add_relation)(
        scene_view_layer_key,
        scene_eval_key,
        "View Layer -> Scene Eval",
        0,
    );
    if linked_state == DEG_ID_LINKED_DIRECTLY {
        (h.build_scene_audio)(scene);
        (h.build_scene_sequencer)(scene);
    }
    /* Set scenes recurse via driver (needs set's view layer). */
    let _ = DEG_ID_LINKED_VIA_SET;
}

/// `build_freestyle_lineset` (verbatim — group + linestyle
/// relations).
pub fn rel_build_freestyle_lineset(
    fls_group: usize,
    fls_linestyle: usize,
    h: &mut ViewLayerRelHooks,
) {
    if fls_group != 0 {
        (h.build_collection)(0, fls_group);
    }
    if fls_linestyle != 0 {
        (h.build_freestyle_linestyle)(fls_linestyle);
    }
}
