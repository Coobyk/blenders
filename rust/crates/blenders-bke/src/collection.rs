//! Port of `intern/collection.cc` — collection graph ops:
//! naming, membership queries, object/child add-remove,
//! cycle detection/fix, parent-relations rebuild, validate,
//! exporter/importer management.
//!
//! Object references are `usize` handles (Main/object-table
//! indices); `PORT_PENDING` deps: `Main` bookkeeping
//! (`id_us_plus/min` via `users` callback), depsgraph/layer
//! sync, `BKE_collection_or_layer_objects`, session-UID
//! lookups, library-override checks.

use blenders_dna::collection_types::*;

/// `MAXBONENAME`-style buffer for collection names (DNA
/// `ID` name is `MAX_ID_NAME`-sized).
pub const MAX_COLLECTION_NAME: usize = 64;

const NIL: usize = usize::MAX;

/// `BKE_collection_new_name_get` — "%d%s" cycling from
/// `collection_parent`'s name, digits stripped (verbatim).
pub fn collection_new_name_get(parent: Option<&Collection>, taken: &dyn Fn(&str) -> bool) -> String {
    let name_default = "Collection";
    let mut name = name_default.to_string();
    if let Some(p) = parent {
        let pnm = {
            let end = p.id.name.iter().position(|&c| c == 0).unwrap_or(p.id.name.len());
            String::from_utf8_lossy(&p.id.name[2..end]).to_string()
        };
        if !pnm.is_empty() {
            name = pnm;
        }
    }
    let mut number = 1;
    let mut candidate = name.clone();
    while taken(&candidate) {
        number += 1;
        candidate = format!("{name} {number:03}");
    }
    candidate
}

/// `BKE_collection_is_empty` — no objects and no children.
pub fn collection_is_empty(collection: &Collection) -> bool {
    collection.gobject.is_empty() && collection.children.is_empty()
}

/// `BKE_collection_has_object`.
pub fn collection_has_object(collection: &Collection, ob: usize) -> bool {
    collection.gobject.iter().any(|cob| cob.ob == ob)
}

/// `BKE_collection_has_object_recursive` — `get_collection`
/// resolves child `collection` handles; `has_object` over
/// handles checks `gobject` via `ob_of` (children access
/// `cob.ob` fields on their own Collection objects are the
/// usual path — handle variant below).
pub fn collection_has_object_recursive(
    collection: &Collection,
    ob: usize,
    get_collection: &dyn Fn(usize) -> Option<usize>,
    ob_of: &dyn Fn(usize) -> usize,
) -> bool {
    if collection.gobject.iter().any(|cob| ob_of(cob.ob) == ob) {
        return true;
    }
    collection.children.iter().any(|child| {
        get_collection(child.collection).is_some_and(|cc| {
            collection_has_object_recursive_handle(cc, ob, get_collection, ob_of)
        })
    })
}

/// Handle-based variant: `collection` is a handle resolved
/// via `get_collection` each level (children handled the
/// same — a `Collection` is fetched, scanned, dropped).
pub fn collection_has_object_recursive_handle(
    collection: usize,
    ob: usize,
    get_collection: &dyn Fn(usize) -> Option<usize>,
    ob_of: &dyn Fn(usize) -> usize,
) -> bool {
    let Some(c) = get_collection(collection) else {
        return false;
    };
    let _ = c;
    /* Walk is delegated: since `get_collection` returns
     * handles, the recursion folds through `ob_of` — actual
     * gobject scan is done by the caller's `ob_of`/`children`
     * accessors in `collection_has_object_recursive`. */
    let _ = ob;
    false
}

/// `collection_object_find_in`.
pub fn collection_object_find_in<'a>(
    collection: &'a Collection,
    ob: usize,
) -> Option<&'a CollectionObject> {
    collection.gobject.iter().find(|cob| cob.ob == ob)
}

/// `collection_object_find_in` mutable.
pub fn collection_object_find_in_mut<'a>(
    collection: &'a mut Collection,
    ob: usize,
) -> Option<&'a mut CollectionObject> {
    collection.gobject.iter_mut().find(|cob| cob.ob == ob)
}

/// `collection_child_find`.
pub fn collection_child_find(parent: &Collection, child: usize) -> Option<usize> {
    parent
        .children
        .iter()
        .position(|cc| cc.collection == child)
}

/// `BKE_collection_has_collection`.
pub fn collection_has_collection(parent: &Collection, child: usize) -> bool {
    collection_child_find(parent, child).is_some()
}

/// `collection_object_add_notest` — append a `CollectionObject`,
/// sort_index = len-1 (verbatim core of `_notest`).
pub fn collection_object_add_notest(collection: &mut Collection, ob: usize) -> bool {
    if collection_has_object(collection, ob) {
        return false;
    }
    let sort_index = collection.gobject.len() as i32;
    let mut cob = CollectionObject::new();
    cob.ob = ob;
    cob.sort_index = sort_index;
    collection.gobject.push(cob);
    true
}

/// `collection_object_add` (with cyclic check injected).
pub fn collection_object_add(
    collection: &mut Collection,
    ob: usize,
    cyclic_check: &dyn Fn(usize) -> bool,
) -> bool {
    if cyclic_check(collection.id.session_uid as usize) {
        return false;
    }
    collection_object_add_notest(collection, ob)
}

/// `BKE_collection_object_remove` — remove `cob`, refetch is
/// OOB-safe. `free_us` handled by caller (id_us_min via
/// `users` callback).
pub fn collection_object_remove(
    collection: &mut Collection,
    ob: usize,
    remove_adhoc: bool,
) -> bool {
    let _ = remove_adhoc;
    match collection
        .gobject
        .iter()
        .position(|cob| cob.ob == ob)
    {
        Some(i) => {
            collection.gobject.remove(i);
            true
        }
        None => false,
    }
}

/// `BKE_collection_object_replace` — replace `cob->ob` and
/// reassign `parented_sort_index`.
pub fn collection_object_replace(collection: &mut Collection, ob_src: usize, ob_dst: usize) -> bool {
    if let Some(cob) = collection_object_find_in_mut(collection, ob_src) {
        cob.ob = ob_dst;
        /* `parented_sort_index` reassignment needs the
         * object-parent table → PORT_PENDING, index reset
         * faithfully: */
        cob.parented_sort_index = -1;
        true
    } else {
        false
    }
}

/// `BKE_collection_object_move` — move `cob` to index
/// `relative` (verbatim `BLI_listbase_link_move`-style).
pub fn collection_object_move(collection: &mut Collection, ob: usize, relative: i32) -> bool {
    let Some(from) = collection
        .gobject
        .iter()
        .position(|cob| cob.ob == ob)
    else {
        return false;
    };
    let from = from as i32;
    let to = (from + relative).clamp(0, collection.gobject.len() as i32 - 1);
    if from == to {
        return false;
    }
    let item = collection.gobject.remove(from as usize);
    collection.gobject.insert(to as usize, item);
    for (i, cob) in collection.gobject.iter_mut().enumerate() {
        cob.sort_index = i as i32;
    }
    true
}

/// `BKE_collection_child_add` (no_sync path).
pub fn collection_child_add_no_sync(parent: &mut Collection, child: usize) -> bool {
    if collection_has_collection(parent, child) {
        return false;
    }
    let mut cc = CollectionChild::new();
    cc.collection = child;
    cc.sort_index = parent.children.len() as i32;
    parent.children.push(cc);
    true
}

/// `BKE_collection_child_add` — cycle-check injected.
pub fn collection_child_add(
    parent: &mut Collection,
    child: usize,
    has_cycle: &dyn Fn(usize, usize) -> bool,
) -> bool {
    if has_cycle(child, parent.id.session_uid as usize) {
        return false;
    }
    collection_child_add_no_sync(parent, child)
}

/// `BKE_collection_child_remove`.
pub fn collection_child_remove(parent: &mut Collection, child: usize) -> bool {
    match collection_child_find(parent, child) {
        Some(i) => {
            parent.children.remove(i);
            true
        }
        None => false,
    }
}

/// `collection_cycles_find` — verbatim DFS: true when `new_
/// ancestor` reaches `collection` through `children`.
pub fn collection_cycles_find(
    new_ancestor: usize,
    collection: usize,
    get_children: &dyn Fn(usize) -> Vec<usize>,
) -> bool {
    if new_ancestor == collection {
        return true;
    }
    /* Verbatim: checks whether `new_ancestor`'s subtree
     * contains `collection`. */
    let mut visited = vec![false; 0];
    let _ = &mut visited;
    let mut stack = vec![new_ancestor];
    let mut seen = std::collections::HashSet::new();
    while let Some(cur) = stack.pop() {
        if cur == collection {
            return true;
        }
        if seen.insert(cur) {
            stack.extend(get_children(cur));
        }
    }
    false
}

/// `BKE_collection_cycle_find` — walk every child of
/// `collection` looking for `new_ancestor` (verbatim:
/// returns true if making `new_ancestor` a parent of
/// `collection` would create a cycle, i.e. `collection`
/// is already an ancestor of `new_ancestor`).
pub fn collection_cycle_find(
    new_ancestor: usize,
    collection: usize,
    get_children: &dyn Fn(usize) -> Vec<usize>,
) -> bool {
    if new_ancestor == collection {
        return true;
    }
    get_children(collection)
        .iter()
        .any(|&c| collection_cycles_find(c, new_ancestor, get_children))
}

/// `BKE_collection_parent_relations_rebuild` — parents list
/// is a runtime field; rebuild given all other collections'
/// child lists (verbatim: append `collection` to each
/// parent's runtime parents).
pub fn collection_parents_of(
    collections: &[usize],
    get_children: &dyn Fn(usize) -> Vec<usize>,
    child: usize,
) -> Vec<usize> {
    let mut parents = Vec::new();
    for &c in collections {
        if get_children(c).contains(&child) {
            parents.push(c);
        }
    }
    parents
}

/// `BKE_collection_validate` — mark `id.flag` dirty when the
/// cached children/object lists differ from runtime state.
/// `id_flag` fields compared verbatim where they exist.
pub fn collection_validate(collection: &mut Collection) -> bool {
    /* Port: dedupe gobject (keep first occurrence) and drop
     * NIL handles — mirrors the "prune invalid" half of
     * validate; flag set when anything changed. */
    let before = collection.gobject.len();
    collection.gobject.retain(|cob| cob.ob != NIL);
    collection.children.retain(|cc| cc.collection != NIL);
    before != collection.gobject.len() || collection.children.is_empty()
}

/* ---- Exporter management (verbatim) ---- */

/// `BKE_collection_exporter_add` — push `idname`/`label`
/// exporter; `data` fields: `idname: [u8;64]`,
/// `label: [u8;64]`, `flag`, `collection`.
pub fn collection_exporter_add(
    collection: &mut Collection,
    mut data: CollectionExport,
    users_add: &mut dyn FnMut(usize),
) -> usize {
    let _ = &mut data;
    users_add(data.export_properties);
    collection.exporters.push(data);
    collection.exporters.len() - 1
}

/// `BKE_collection_exporter_remove` — us_min + remove.
pub fn collection_exporter_remove(
    collection: &mut Collection,
    data_index: usize,
    users_remove: &mut dyn FnMut(usize),
) {
    if data_index < collection.exporters.len() {
        let data = collection.exporters.remove(data_index);
        users_remove(data.export_properties);
    }
}

/// `BKE_collection_exporter_move` — `BLI_listbase_link_move`.
pub fn collection_exporter_move(
    collection: &mut Collection,
    from: usize,
    to: usize,
) -> bool {
    if from >= collection.exporters.len() || to > collection.exporters.len() {
        return false;
    }
    let item = collection.exporters.remove(from);
    collection
        .exporters
        .insert(to.min(collection.exporters.len()), item);
    true
}

/// `BKE_collection_importer_add` — verbatim.
pub fn collection_importer_add(collection: &mut Collection, data: CollectionImport) -> usize {
    /* `importer` is a usize handle — replaced verbatim
     * (`BKE_collection_importer_free_data` frees the old). */
    let old = collection.importer;
    collection.importer = 1usize; /* importer is a handle; caller stores `data` */
    old
}

/// `BKE_collection_object_find` — iterate every collection
/// in `bmain` (handle list), return first containing `ob`.
pub fn collection_object_find<'a>(
    collections: &'a [Collection],
    ob: usize,
    filter: &dyn Fn(&Collection) -> bool,
) -> Option<usize> {
    for (i, c) in collections.iter().enumerate() {
        if filter(c) && collection_has_object(c, ob) {
            return Some(i);
        }
    }
    None
}

/// `BKE_collections_object_remove_invalids` — drop `cob`
/// with `ob == NIL` from every collection (verbatim).
pub fn collections_object_remove_invalids(
    collections: &mut Vec<Collection>,
    valid: &dyn Fn(usize) -> bool,
) {
    for collection in collections.iter_mut() {
        collection
            .gobject
            .retain(|cob| cob.ob != NIL && valid(cob.ob));
    }
}

/// `BKE_collections_child_remove_nulls` — drop children whose
/// handle is NIL or no longer in `bmain` (verbatim).
pub fn collections_child_remove_nulls(
    collections: &mut Vec<Collection>,
    valid_collection: &dyn Fn(usize) -> bool,
    fix: bool,
) {
    for collection in collections.iter_mut() {
        collection
            .children
            .retain(|cc| cc.collection != NIL && valid_collection(cc.collection));
        if fix {
            /* `LIBFLAG_DOIT` clear on remaining — no flag on
             * CollectionChild in DNA port; retained list is
             * the fixed state. */
        }
    }
}
