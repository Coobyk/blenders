//! Port of `intern/depsgraph_query_iter.cc` +
/// `DEG_depsgraph_query.hh` iter structs — object/id
/// iterators over the depsgraph.

use std::collections::HashSet;

/// `DEG_ITER_OBJECT_FLAG_*` verbatim.
pub const DEG_ITER_OBJECT_FLAG_LINKED_DIRECTLY: i32 = 1 << 0;
/// `DEG_ITER_OBJECT_FLAG_LINKED_INDIRECTLY`.
pub const DEG_ITER_OBJECT_FLAG_LINKED_INDIRECTLY: i32 = 1 << 1;
/// `DEG_ITER_OBJECT_FLAG_LINKED_VIA_SET`.
pub const DEG_ITER_OBJECT_FLAG_LINKED_VIA_SET: i32 = 1 << 2;
/// `DEG_ITER_OBJECT_FLAG_VISIBLE`.
pub const DEG_ITER_OBJECT_FLAG_VISIBLE: i32 = 1 << 3;
/// `DEG_ITER_OBJECT_FLAG_DUPLI`.
pub const DEG_ITER_OBJECT_FLAG_DUPLI: i32 = 1 << 4;

/// `OB_DUPLIVERTS|OB_DUPLIFACES` (`DNA_object_types.h`).
pub const OB_DUPLIVERTS: i32 = 1;
/// `OB_DUPLIFACES`.
pub const OB_DUPLIFACES: i32 = 2;
/// `OB_DUPLI` mask.
pub const OB_DUPLI: i32 = OB_DUPLIVERTS | OB_DUPLIFACES | 4 | 8 | 16;
/// `OB_NEG_SCALE`.
pub const OB_NEG_SCALE: i32 = 1 << 13;
/// `BASE_FROM_DUPLI` (`DNA_object_enums.h`).
pub const BASE_FROM_DUPLI: i32 = 1 << 6;
/// `BASE_ENABLED_AND_MAYBE_VISIBLE_IN_VIEWPORT`.
pub const BASE_ENABLED_AND_MAYBE_VISIBLE_IN_VIEWPORT: i32 = 3;
/// `OB_VISIBLE_*` mask bits used here (verbatim).
pub const OB_VISIBLE_SELF: i32 = 1 << 0;
/// `OB_VISIBLE_PARTICLES`.
pub const OB_VISIBLE_PARTICLES: i32 = 1 << 1;
/// `OB_VISIBLE_INSTANCES`.
pub const OB_VISIBLE_INSTANCES: i32 = 1 << 2;
/// `OB_VISIBLE_ALL`.
pub const OB_VISIBLE_ALL: i32 = OB_VISIBLE_SELF | OB_VISIBLE_PARTICLES | OB_VISIBLE_INSTANCES;
/// `DAG_EVAL_RENDER`.
pub const DAG_EVAL_RENDER: i32 = 2;

/// `DEG_ID_LINKED_*` mirror (verbatim).
pub const DEG_ID_LINKED_DIRECTLY: i32 = 0;
/// `DEG_ID_LINKED_VIA_SET`.
pub const DEG_ID_LINKED_VIA_SET: i32 = 1;
/// `DEG_ID_LINKED_INDIRECTLY`.
pub const DEG_ID_LINKED_INDIRECTLY: i32 = 2;

/// `DupliObject` verbatim fields used by the iterator.
#[derive(Clone, Default)]
pub struct DupliObject {
    /// `ob`.
    pub ob: usize,
    /// `ob_data`.
    pub ob_data: usize,
    /// `type` (`OB_DUPLI*`).
    pub r#type: i32,
    /// `no_draw`.
    pub no_draw: bool,
    /// `mat` (4x4 world matrix, column-major).
    pub mat: [[f32; 4]; 4],
}

/// `DEGObjectIterSettings` (verbatim — `included_objects`
/// is an orig-id set; `viewer_path` a foreign handle).
#[derive(Default)]
pub struct DegObjectIterSettings {
    /// `included_objects`.
    pub included_objects: Option<HashSet<usize>>,
    /// `viewer_path` (foreign `ViewerPath` handle).
    pub viewer_path: usize,
}

/// `DEGObjectIterData` (verbatim fields).
#[derive(Default)]
pub struct DegObjectIterData {
    /// `settings`.
    pub settings: DegObjectIterSettings,
    /// `flag`.
    pub flag: i32,
    /// `eval_mode`.
    pub eval_mode: i32,
    /// `object_orig_with_preview`.
    pub object_orig_with_preview: usize,
    /// `next_object` (yield value).
    pub next_object: usize,
    /// `dupli_parent`.
    pub dupli_parent: usize,
    /// `dupli_list`.
    pub dupli_list: Vec<DupliObject>,
    /// `dupli_object_next` (index, -1 sentinel via
    /// `dupli_object_next_index`).
    pub dupli_object_next_index: i32,
    /// `dupli_object_current` (index into `dupli_list`,
    /// -1 none).
    pub dupli_object_current_index: i32,
    /// `id_node_index`.
    pub id_node_index: usize,
    /// `num_id_nodes`.
    pub num_id_nodes: usize,
    /// `temp_dupli_object` (handle for the temp object the
    /// caller materializes).
    pub temp_dupli_object: usize,
}

/// `DEGIDIterData` (verbatim).
pub struct DegIdIterData {
    /// `only_updated`.
    pub only_updated: bool,
    /// `id_node_index`.
    pub id_node_index: usize,
    /// `num_id_nodes`.
    pub num_id_nodes: usize,
}

/// Hooks for object/dupli/collection data the iterator
/// reads out of DNA types.
pub struct IterHooks<'a> {
    /// `id_node->is_visible_on_build`.
    pub is_visible_on_build: &'a dyn Fn(usize) -> bool,
    /// `id_node->linked_state`.
    pub linked_state: &'a dyn Fn(usize) -> i32,
    /// `id_node->id_cow` (the object).
    pub id_cow: &'a dyn Fn(usize) -> usize,
    /// `id_node->id_orig`.
    pub id_orig: &'a dyn Fn(usize) -> usize,
    /// `id_node->id_cow->recalc`.
    pub cow_recalc: &'a dyn Fn(usize) -> u32,
    /// `id_node->id_cow` GS type.
    pub id_gs: &'a dyn Fn(usize) -> u16,
    /// `object->type`.
    pub ob_type: &'a dyn Fn(usize) -> i16,
    /// `object->transflag`.
    pub ob_transflag: &'a dyn Fn(usize) -> i32,
    /// `object->parent` + `parent->transflag`.
    pub parent_transflag: &'a dyn Fn(usize) -> i32,
    /// `object->runtime->geometry_set_eval != nullptr`.
    pub has_geometry_set_eval: &'a dyn Fn(usize) -> bool,
    /// `BKE_object_visibility(ob, eval_mode)`.
    pub object_visibility: &'a dyn Fn(usize, i32) -> i32,
    /// `object->runtime->select_id = orig->runtime->select_id`.
    pub copy_select_id: &'a mut dyn FnMut(usize, usize),
    /// `object_duplilist(graph, object, included, list)` —
    /// fills `dupli_list`.
    pub object_duplilist: &'a mut dyn FnMut(usize, &mut Vec<DupliObject>),
    /// `object_duplilist_preview(graph, object, viewer_path, list)`.
    pub object_duplilist_preview: &'a mut dyn FnMut(usize, &mut Vec<DupliObject>),
    /// `DEG_iterator_temp_object_from_dupli` → yields a temp
    /// object handle (or None).
    pub temp_object_from_dupli: &'a mut dyn FnMut(usize, &DupliObject, i32) -> Option<usize>,
    /// `DEG_iterator_temp_object_free_properties`.
    pub temp_object_free_properties: &'a mut dyn FnMut(&DupliObject, usize),
    /// `find_object_with_preview_geometry(viewer_path)`.
    pub object_with_preview: &'a dyn Fn(usize) -> usize,
}

/// `deg_object_hide_original` (verbatim — `parent_transflag`
/// resolves to 0 when `ob->parent` is None).
pub fn deg_object_hide_original(eval_mode: i32, dob_type: Option<i32>, parent_transflag: i32) -> bool {
    const HIDE_ORIGINAL_TYPES: i32 = OB_DUPLIVERTS | OB_DUPLIFACES;
    if eval_mode == DAG_EVAL_RENDER || dob_type.is_some() {
        if dob_type.map_or(true, |t| t & HIDE_ORIGINAL_TYPES == 0)
            && parent_transflag & HIDE_ORIGINAL_TYPES != 0
        {
            return true;
        }
    }
    false
}

/// `DEG_iterator_object_is_visible` (verbatim).
pub fn deg_iterator_object_is_visible(
    eval_mode: i32,
    ob: usize,
    h: &IterHooks,
) -> bool {
    if (h.ob_type)(ob) == crate::depsgraph_tag::OB_MBALL {
        return true;
    }
    !deg_object_hide_original(eval_mode, None, (h.parent_transflag)(ob))
}

/// `DEG_iterator_dupli_is_visible` (verbatim).
pub fn deg_iterator_dupli_is_visible(dob: &DupliObject, eval_mode: i32, h: &IterHooks) -> bool {
    if dob.no_draw {
        return false;
    }
    if dob.ob_data != 0 && (h.id_gs)(dob.ob_data) == blenders_dna::id::gs(b"MB") {
        return false;
    }
    if (h.ob_type)(dob.ob) != crate::depsgraph_tag::OB_MBALL
        && deg_object_hide_original(eval_mode, Some(dob.r#type), (h.parent_transflag)(dob.ob))
    {
        return false;
    }
    true
}

/// `deg_iterator_duplis_init` (verbatim).
pub fn deg_iterator_duplis_init(data: &mut DegObjectIterData, object: usize) {
    if data.dupli_list.is_empty() {
        return;
    }
    data.dupli_parent = object;
    data.dupli_object_next_index = 0;
    data.dupli_object_current_index = -1;
}

/// `deg_iterator_duplis_step` (verbatim — returns false on
/// exhaustion; yields the temp-object handle through
/// `next_object`).
pub fn deg_iterator_duplis_step(
    data: &mut DegObjectIterData,
    h: &mut IterHooks,
) -> bool {
    if data.dupli_list.is_empty() {
        return false;
    }
    let mut index = data.dupli_object_next_index;
    while index >= 0 && (index as usize) < data.dupli_list.len() {
        let dob = data.dupli_list[index as usize].clone();
        index += 1;
        if (index as usize) >= data.dupli_list.len() {
            index = -1;
        }
        if !deg_iterator_dupli_is_visible(&dob, data.eval_mode, h) {
            continue;
        }
        if let Some(included) = &data.settings.included_objects {
            if !included.contains(&dob.ob) {
                continue;
            }
        }
        if data.dupli_object_current_index >= 0 {
            let cur = data.dupli_list[data.dupli_object_current_index as usize].clone();
            (h.temp_object_free_properties)(&cur, data.temp_dupli_object);
        }
        data.dupli_object_current_index = index.max(0) as i32;
        if let Some(temp) = (h.temp_object_from_dupli)(data.dupli_parent, &dob, data.eval_mode)
        {
            data.dupli_object_next_index = index;
            data.next_object = temp;
            return true;
        }
    }
    if data.dupli_object_current_index >= 0 {
        let cur = data.dupli_list[data.dupli_object_current_index as usize].clone();
        (h.temp_object_free_properties)(&cur, data.temp_dupli_object);
    }
    data.dupli_list.clear();
    data.dupli_parent = 0;
    data.dupli_object_next_index = -1;
    data.dupli_object_current_index = -1;
    false
}

/// `deg_iterator_objects_step` (verbatim — iterates
/// `id_nodes` for `ID_OB` entries, fills the dupli list or
/// yields the object itself; `id_nodes` is the flat
/// `id_node` index range).
pub fn deg_iterator_objects_step(
    data: &mut DegObjectIterData,
    num_id_nodes: usize,
    h: &mut IterHooks,
) -> bool {
    while data.id_node_index < num_id_nodes {
        let id_node = data.id_node_index;
        if !(h.is_visible_on_build)(id_node) {
            data.id_node_index += 1;
            continue;
        }
        let id_cow = (h.id_cow)(id_node);
        if (h.id_gs)(id_cow) != blenders_dna::id::gs(b"OB") {
            data.id_node_index += 1;
            continue;
        }
        match (h.linked_state)(id_node) {
            DEG_ID_LINKED_DIRECTLY
                if (data.flag & DEG_ITER_OBJECT_FLAG_LINKED_DIRECTLY) == 0 =>
            {
                data.id_node_index += 1;
                continue;
            }
            DEG_ID_LINKED_VIA_SET
                if (data.flag & DEG_ITER_OBJECT_FLAG_LINKED_VIA_SET) == 0 =>
            {
                data.id_node_index += 1;
                continue;
            }
            DEG_ID_LINKED_INDIRECTLY
                if (data.flag & DEG_ITER_OBJECT_FLAG_LINKED_INDIRECTLY) == 0 =>
            {
                data.id_node_index += 1;
                continue;
            }
            _ => {}
        }
        let object = id_cow;
        let object_orig = (h.id_orig)(id_node);
        if let Some(included) = &data.settings.included_objects {
            if !included.contains(&object_orig) {
                data.id_node_index += 1;
                continue;
            }
        }
        (h.copy_select_id)(object, object_orig);
        if object_orig == data.object_orig_with_preview {
            (h.object_duplilist_preview)(object, &mut data.dupli_list);
            deg_iterator_duplis_init(data, object);
            data.id_node_index += 1;
            return true;
        }
        let mut ob_visibility = OB_VISIBLE_ALL;
        if data.flag & DEG_ITER_OBJECT_FLAG_VISIBLE != 0 {
            ob_visibility = (h.object_visibility)(object, data.eval_mode);
            if !deg_iterator_object_is_visible(data.eval_mode, object, h) {
                data.id_node_index += 1;
                continue;
            }
        }
        if ob_visibility & OB_VISIBLE_INSTANCES != 0
            && (data.flag & DEG_ITER_OBJECT_FLAG_DUPLI) != 0
            && (((h.ob_transflag)(object)) & OB_DUPLI != 0 || (h.has_geometry_set_eval)(object))
        {
            (h.object_duplilist)(object, &mut data.dupli_list);
            deg_iterator_duplis_init(data, object);
        }
        if ob_visibility & (OB_VISIBLE_SELF | OB_VISIBLE_PARTICLES) != 0 {
            data.next_object = object;
        }
        data.id_node_index += 1;
        return true;
    }
    false
}

/// `DEG_iterator_objects_next` driver (verbatim — pulls
/// `next_object`, then dupli step, then object step).
pub fn deg_iterator_objects_next(
    data: &mut DegObjectIterData,
    num_id_nodes: usize,
    h: &mut IterHooks,
) -> Option<usize> {
    loop {
        if data.next_object != 0 {
            let obj = data.next_object;
            data.next_object = 0;
            return Some(obj);
        }
        if deg_iterator_duplis_step(data, h) {
            continue;
        }
        if deg_iterator_objects_step(data, num_id_nodes, h) {
            continue;
        }
        return None;
    }
}

/// `DEG_iterator_ids_step` — returns `Some(id_cow)` or
/// `None` (skip) (verbatim).
pub fn deg_iterator_ids_step(
    id_node: usize,
    only_updated: bool,
    h: &IterHooks,
) -> Option<usize> {
    let id_cow = (h.id_cow)(id_node);
    if !only_updated && !(h.is_visible_on_build)(id_node) {
        return None;
    }
    if only_updated && (h.cow_recalc)(id_node) & blenders_dna::id::ID_RECALC_ALL == 0 {
        return None;
    }
    Some(id_cow)
}

/// `DEG_iterator_ids_begin`/`next` (verbatim — the skip
/// loop folded into a `next` that advances to the next
/// non-skipped id).
pub fn deg_iterator_ids_next(
    data: &mut DegIdIterData,
    num_id_nodes: usize,
    h: &IterHooks,
) -> Option<usize> {
    loop {
        data.id_node_index += 1;
        if data.id_node_index >= num_id_nodes {
            return None;
        }
        if let Some(id) = deg_iterator_ids_step(data.id_node_index, data.only_updated, h) {
            return Some(id);
        }
    }
}
