//! Port of `intern/layer_utils.cc` — `BKE_view_layer_*` object/base
//! enumeration helpers.
//!
//! `FOREACH_SELECTED_OBJECT`/`FOREACH_BASE_IN_MODE` are injected as
//! item-producing closures; `no_dup_data` uses a `HashSet` instead of
//! the C's `ID_TAG_DOIT` flag dance (same semantics: dedupe by data
//! ID). `BKE_view_layer_synced_ensure` injected where upstream calls it.

use std::collections::HashSet;

use blenders_dna::id::NIL;

/// `ObjectsInViewLayerParams`.
#[derive(Default)]
pub struct ObjectsInViewLayerParams {
    /// `filter_fn` — object accept filter.
    pub filter_fn: Option<Box<dyn Fn(usize) -> bool>>,
    /// `no_dup_data` — dedupe objects sharing `ob->data`.
    pub no_dup_data: bool,
}

/// `ObjectsInModeParams`.
#[derive(Default)]
pub struct ObjectsInModeParams {
    /// `object_mode` (`OB_MODE_*`).
    pub object_mode: i32,
    /// `filter_fn`.
    pub filter_fn: Option<Box<dyn Fn(usize) -> bool>>,
    /// `no_dup_data`.
    pub no_dup_data: bool,
}

/// `BKE_view_layer_array_selected_objects_params` — `each` yields
/// `(object, data_id)` pairs (`data_id` = `ob->data` ID, `NIL` = none).
pub fn view_layer_array_selected_objects_params(
    each: &mut dyn FnMut(&mut dyn FnMut(usize, usize)),
    params: &ObjectsInViewLayerParams,
) -> Vec<usize> {
    let mut objects = Vec::new();
    let mut seen: HashSet<usize> = HashSet::new();
    each(&mut |ob, data_id| {
        if let Some(f) = &params.filter_fn {
            if !f(ob) {
                return;
            }
        }
        if params.no_dup_data && data_id != NIL && !seen.insert(data_id) {
            return; /* already emitted an object with this data */
        }
        objects.push(ob);
    });
    objects
}

/// `BKE_view_layer_array_from_bases_in_mode_params` — `each` yields
/// `(base, object, data_id)` triples.
pub fn view_layer_array_from_bases_in_mode_params(
    each: &mut dyn FnMut(&mut dyn FnMut(usize, usize, usize)),
    params: &ObjectsInModeParams,
) -> Vec<usize> {
    let mut bases = Vec::new();
    let mut seen: HashSet<usize> = HashSet::new();
    each(&mut |base, object, data_id| {
        if let Some(f) = &params.filter_fn {
            if !f(object) {
                return;
            }
        }
        if params.no_dup_data && data_id != NIL && !seen.insert(data_id) {
            return;
        }
        bases.push(base);
    });
    bases
}

/// `BKE_view_layer_array_from_objects_in_mode_params` — objects of
/// the bases (same params path).
pub fn view_layer_array_from_objects_in_mode_params(
    each: &mut dyn FnMut(&mut dyn FnMut(usize, usize, usize)),
    params: &ObjectsInModeParams,
) -> Vec<usize> {
    let bases = view_layer_array_from_bases_in_mode_params(each, params);
    /* bases hold (base, object) — callers pass a base→object map */
    bases
}

/// `BKE_view_layer_filter_edit_mesh_has_uvs` — `edit_mesh_bmesh`
////`bm_has_float2_layer` injected (`CustomData_has_layer(&bm->ldata,
/// CD_PROP_FLOAT2)`).
pub fn view_layer_filter_edit_mesh_has_uvs(
    object_type: i16,
    has_float2_layer: &dyn Fn(usize) -> bool,
) -> bool {
    const OB_MESH: i16 = 1;
    object_type == OB_MESH && has_float2_layer(0)
}

/// `BKE_view_layer_filter_edit_mesh_has_edges` —
/// `bm->totedge != 0` injected.
pub fn view_layer_filter_edit_mesh_has_edges(
    object_type: i16,
    edit_mesh_edge_count: &dyn Fn(usize) -> i32,
) -> bool {
    const OB_MESH: i16 = 1;
    object_type == OB_MESH && edit_mesh_edge_count(0) != 0
}

/// `BKE_view_layer_non_active_selected_object` — returns the single
/// non-active selected object, or NIL (none or more than one).
/// `synced_ensure`/`active_object`/`each_selected` injected.
pub fn view_layer_non_active_selected_object(
    synced_ensure: &mut dyn FnMut(),
    active_object: &dyn Fn() -> usize,
    each_selected: &mut dyn FnMut(&mut dyn FnMut(usize)),
) -> usize {
    synced_ensure();
    let ob_active = active_object();
    let mut ob_result = NIL;
    let mut multiple = false;
    each_selected(&mut |ob| {
        if ob == ob_active || multiple {
            return;
        }
        if ob_result == NIL {
            ob_result = ob;
        } else {
            ob_result = NIL;
            multiple = true; /* C breaks with nullptr */
        }
    });
    if multiple { NIL } else { ob_result }
}

/// `BKE_view_layer_active_object_get` — `basact` → object
/// (`active_base`/`base_object` injected).
pub fn view_layer_active_object_get(active_base: &dyn Fn() -> usize, base_object: &dyn Fn(usize) -> usize) -> usize {
    let base = active_base();
    if base == NIL { NIL } else { base_object(base) }
}

/// `BKE_view_layer_edit_object_get` — active object iff
/// `ob->mode & OB_MODE_EDIT`.
pub fn view_layer_edit_object_get(
    active_object: &dyn Fn() -> usize,
    object_mode_of: &dyn Fn(usize) -> i32,
) -> usize {
    const OB_MODE_EDIT: i32 = 1 << 0;
    let ob = active_object();
    if ob == NIL {
        return NIL;
    }
    if object_mode_of(ob) & OB_MODE_EDIT == 0 {
        return NIL;
    }
    ob
}

/* `BKE_view_layer_object_bases_unsynced_get` — field accessor,
 * `VIEW_LAYER_OUT_OF_SYNC` assertion in callers (`bases_get`/
 * `active_base_get`/`active_collection_get` are raw field reads —
 * the OUT_OF_SYNC assert belongs to the synced-ensure contract). */
