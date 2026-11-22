//! Port of `intern/builder/deg_builder_nodes_view_layer.cc`
/// — view-layer node building.

use crate::deg_builder_nodes_scene::DEG_ID_LINKED_DIRECTLY;

/// `COLLECTION_HIDE_VIEWPORT`/`COLLECTION_HIDE_RENDER`
/// (`DNA_collection_types.h` verbatim).
pub const COLLECTION_HIDE_VIEWPORT: i32 = 1 << 0;
pub const COLLECTION_HIDE_RENDER: i32 = 1 << 5;
/// `LAYER_COLLECTION_EXCLUDE` (`DNA_layer_types.h`).
pub const LAYER_COLLECTION_EXCLUDE: i32 = 1 << 0;
/// `DEG_ID_LINKED_VIA_SET` (verbatim).
pub const DEG_ID_LINKED_VIA_SET: i32 = 2;
/// `DAG_EVAL_VIEWPORT` (verbatim).
pub const DAG_EVAL_VIEWPORT: i32 = 0;

/// One layer-collection row (verbatim `LayerCollection`
/// fields used here).
pub struct LayerCollectionRow {
    /// `lc.collection->flag`.
    pub collection_flag: i32,
    /// `lc.flag`.
    pub flag: i32,
    /// `lc.collection` handle.
    pub collection: usize,
    /// `lc` handle itself (for `build_collection`).
    pub lc: usize,
    /// `lc.layer_collections` children.
    pub children: Vec<LayerCollectionRow>,
}

/// Injected builder calls for view-layer building.
pub struct ViewLayerNodeBuildHooks<'a> {
    /// `add_id_node` → node handle.
    pub add_id_node: &'a mut dyn FnMut(usize) -> usize,
    /// `id_node->linked_state = ...`.
    pub set_linked_state: &'a mut dyn FnMut(usize, i32),
    /// `add_operation_node`.
    pub add_operation_node: &'a mut dyn FnMut(usize, usize, usize),
    /// `add_time_source`.
    pub add_time_source: &'a mut dyn FnMut(),
    /// `need_pull_base_into_graph`.
    pub need_pull_base_into_graph: &'a mut dyn FnMut(usize) -> bool,
    /// `is_object_visibility_animated`.
    pub is_object_visibility_animated: &'a mut dyn FnMut(usize) -> bool,
    /// `build_object` (base_index, object, linked_state, is_visible).
    pub build_object: &'a mut dyn FnMut(i32, usize, i32, bool),
    /// `build_collection` (parent lc or 0, collection).
    pub build_collection: &'a mut dyn FnMut(usize, usize),
    /// `build_freestyle_linestyle`.
    pub build_freestyle_linestyle: &'a mut dyn FnMut(usize),
    /// `build_rigidbody`/`build_world`/`build_cachefile`/
    /// `build_mask`/`build_movieclip`/`build_material`.
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
    /// `build_animdata`/`build_scene_audio`/`build_scene_sequencer`/
    /// `build_scene_camera`/`build_scene_compositor`/`build_scene_parameters`.
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

/// `build_layer_collections` (verbatim — skips
/// visibility-flagged collections, recurses children).
pub fn build_layer_collections(
    lb: &[LayerCollectionRow],
    mode: i32,
    h: &mut ViewLayerNodeBuildHooks,
) {
    let visibility_flag =
        if mode == DAG_EVAL_VIEWPORT { COLLECTION_HIDE_VIEWPORT } else { COLLECTION_HIDE_RENDER };
    for lc in lb {
        if lc.collection_flag & visibility_flag != 0 {
            continue;
        }
        if (lc.flag & LAYER_COLLECTION_EXCLUDE) == 0 {
            (h.build_collection)(lc.lc, lc.collection);
        }
        build_layer_collections(&lc.children, mode, h);
    }
}

/// `build_freestyle_lineset` (verbatim — group + linestyle).
pub fn build_freestyle_lineset(
    fls_group: usize,
    fls_linestyle: usize,
    h: &mut ViewLayerNodeBuildHooks,
) {
    if fls_group != 0 {
        (h.build_collection)(0, fls_group);
    }
    if fls_linestyle != 0 {
        (h.build_freestyle_linestyle)(fls_linestyle);
    }
}

/// `build_view_layer` (verbatim — full sequence; concrete
/// per-id builders injected).
#[allow(clippy::too_many_arguments)]
pub fn build_view_layer(
    scene: usize,
    scene_id: usize,
    view_layer: usize,
    linked_state: i32,
    has_rigidbody_world: bool,
    has_adt: bool,
    scene_world: usize,
    scene_set: usize,
    mat_override: usize,
    world_override: usize,
    bases: &[(usize, usize)],
    layer_collections: &[LayerCollectionRow],
    cachefiles: &[usize],
    masks: &[usize],
    movieclips: &[usize],
    freestyle_groups_and_linestyles: &[(usize, usize)],
    has_animated_visibility: &mut bool,
    h: &mut ViewLayerNodeBuildHooks,
) {
    let id_node = (h.add_id_node)(scene_id);
    (h.set_linked_state)(id_node, linked_state);
    (h.add_operation_node)(scene_id, 5 /*NodeType::HIERARCHY*/, 0 /*HIERARCHY*/);
    (h.add_time_source)();
    for (base_index, (_base, object)) in bases.iter().enumerate() {
        if !(h.need_pull_base_into_graph)(*_base) {
            continue;
        }
        (h.build_object)(base_index as i32, *object, linked_state, true);
        if !*has_animated_visibility && (h.is_object_visibility_animated)(*object) {
            *has_animated_visibility = true;
        }
    }
    build_layer_collections(layer_collections, 0, h);
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
        build_freestyle_lineset(*group, *linestyle, h);
    }
    if linked_state == DEG_ID_LINKED_DIRECTLY {
        (h.build_scene_audio)(scene);
        (h.build_scene_sequencer)(scene);
    }
    (h.add_operation_node)(
        scene_id,
        10, /* NodeType::LAYER_COLLECTIONS */
        0,  /* OperationCode::VIEW_LAYER_EVAL */
    );
    (h.build_scene_compositor)(scene);
    (h.build_scene_parameters)(scene);
    if scene_set != 0 {
        /* `BKE_view_layer_default_render(scene->set)` —
         * injected as the set's view layer by the driver. */
    }
}
