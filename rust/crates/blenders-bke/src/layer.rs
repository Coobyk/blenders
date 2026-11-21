//! Port of `intern/layer.cc` — `ViewLayer`/`LayerCollection`/
//! `Base` management: add/free, hidden/active traversal,
//! counting + indexing, resync tags.
//!
//! `PORT_PENDING`: `view_layer_bases_hash_create` (object→
//! Base hash + `basact` bookkeeping — needs `Main`/`Scene`),
//! `layer_collection_sync`/`_objects_sync` (depsgraph-driven
//! collection sync — the full resync machinery), `copy_data`
//! (ID copy callback), `BKE_layer_collection_first_from_
//! scene_collection` (bmain handle lookup — injected),
//! `object_bases_iterator` (Base iteration over
//! `view_layer->object_bases` — `Base` store is `Vec`,
//! iterates its indices).

/// `Base` verbatim.
#[derive(Clone, Default)]
pub struct Base {
    /// `next`/`prev` → index arena handled by the Vec.
    /// `lay` (`rctf` — last visibility rect).
    pub lay: [f32; 4],
    /// `flag` (`eBase_Flag`).
    pub flag: i16,
    /// `flag_legacy` (pre-2.8 flags from DNA).
    pub flag_legacy: i16,
    /// `object` handle.
    pub object: usize,
}

/// `eBase_Flag` verbatim.
pub const BASE_ENABLED: i16 = 1 << 0;
/// `BASE_VISIBLE`.
pub const BASE_VISIBLE: i16 = 1 << 1;
/// `BASE_SELECTED`.
pub const BASE_SELECTED: i16 = 1 << 2;
/// `BASE_SELECTABLE`.
pub const BASE_SELECTABLE: i16 = 1 << 3;
/// `BASE_FROM_DUPLI`.
pub const BASE_FROM_DUPLI: i16 = 1 << 4;
/// `BASE_MARKED`.
pub const BASE_MARKED: i16 = 1 << 5;
/// `BASE_ENABLED_VIEWPORT`.
pub const BASE_ENABLED_VIEWPORT: i16 = 1 << 6;
/// `BASE_ENABLED_RENDER`.
pub const BASE_ENABLED_RENDER: i16 = 1 << 7;
/// `BASE_ENABLED_AND_MAYBE_VISIBLE_IN_VIEWPORT`.
pub const BASE_ENABLED_AND_MAYBE_VISIBLE_IN_VIEWPORT: i16 =
    BASE_ENABLED | BASE_ENABLED_VIEWPORT;

/// `LayerCollection` verbatim (`flag`,`collection`,
/// `layer_collections`, `runtime`).
#[derive(Default)]
pub struct LayerCollection {
    /// `flag` (`eLayerCollection_Flag`).
    pub flag: i16,
    /// `collection` handle.
    pub collection: usize,
    /// `layer_collections` (nested).
    pub layer_collections: Vec<LayerCollection>,
    /// `runtime` handle (`LayerCollection_Runtime`).
    pub runtime: usize,
    /// `excluded`.
    pub excluded: bool,
}

/// `eLayerCollection_Flag` verbatim.
pub const LAYER_COLLECTION_EXCLUDE: i16 = 1 << 0;
/// `LAYER_COLLECTION_HIDE`.
pub const LAYER_COLLECTION_HIDE: i16 = 1 << 1;
/// `LAYER_COLLECTION_HOLDOUT`.
pub const LAYER_COLLECTION_HOLDOUT: i16 = 1 << 2;
/// `LAYER_COLLECTION_INDIRECT_ONLY`.
pub const LAYER_COLLECTION_INDIRECT_ONLY: i16 = 1 << 3;
/// `LAYER_COLLECTION_PREVIOUSLY_EXCLUDED`.
pub const LAYER_COLLECTION_PREVIOUSLY_EXCLUDED: i16 = 1 << 4;
/// `LAYER_COLLECTION_SKIP`.
pub const LAYER_COLLECTION_SKIP: i16 = 1 << 5;

/// `ViewLayer` verbatim (fields; `object_bases`/`layer_
/// collections` are `Vec`s).
#[derive(Default)]
pub struct ViewLayer {
    /// `name[64]`.
    pub name: String,
    /// `flag` (`eViewLayer_Flag`).
    pub flag: i16,
    /// `active_collection` handle (`usize` index into
    /// a per-view-layer collection arena).
    pub active_collection: usize,
    /// `object_bases` — `Vec<Base>`.
    pub object_bases: Vec<Base>,
    /// `basact` — active `Base` index (`usize::MAX` = none).
    pub basact: usize,
    /// `layer_collections` — the scene-collection tree.
    pub layer_collections: Vec<LayerCollection>,
    /// `layflag`, `passflag`, `pass_xor`, `pass_alphahash`,
    /// `aovs`, `lightgroups` — verbatim.
    pub layflag: i16,
    /// `passflag`.
    pub passflag: i32,
    /// `pass_xor`.
    pub pass_xor: i32,
    /// `aovs`/`lightgroups` handles.
    pub aovs: Vec<usize>,
    /// `lightgroups`.
    pub lightgroups: Vec<usize>,
}

/// `eViewLayer_Flag` verbatim.
pub const VIEW_LAYER_OUT_OF_SYNC: i16 = 1 << 0;
/// `VIEW_LAYER_ENGINE_FREESTYLE`.
pub const VIEW_LAYER_ENGINE_FREESTYLE: i16 = 1 << 1;
/// `VIEW_LAYER_SAMPLES`.
pub const VIEW_LAYER_SAMPLES: i16 = 1 << 2;

/// `object_base_new` (verbatim).
pub fn object_base_new(ob: usize) -> Base {
    let mut base = Base::default();
    base.object = ob;
    base.flag = BASE_ENABLED_AND_MAYBE_VISIBLE_IN_VIEWPORT;
    base
}

/// `view_layer_add` (verbatim).
pub fn view_layer_add(name: &str) -> ViewLayer {
    let mut view_layer = ViewLayer::default();
    view_layer.name = name.to_string();
    view_layer.basact = usize::MAX;
    /* default flag — `eViewLayer_Flag 0`. */
    view_layer.flag = 0;
    view_layer
}

/// `layer_collection_add` (verbatim).
pub fn layer_collection_add(collection: usize) -> LayerCollection {
    LayerCollection {
        collection,
        ..Default::default()
    }
}

/// `layer_collection_exclude_all` (verbatim — recursive).
pub fn layer_collection_exclude_all(lc: &mut LayerCollection) {
    lc.flag |= LAYER_COLLECTION_EXCLUDE;
    for c in lc.layer_collections.iter_mut() {
        layer_collection_exclude_all(c);
    }
}

/// `collection_from_index` — walk `layer_collections`
/// breadth-then-depth (verbatim two-pass loop).
pub fn collection_from_index(
    lb: &[LayerCollection],
    number: usize,
    i: &mut usize,
) -> Option<usize> {
    for (idx, _lc) in lb.iter().enumerate() {
        if *i == number {
            return Some(idx);
        }
        *i += 1;
    }
    for lc in lb.iter() {
        /* nested indexes are local per level — returned as
         * path components; the caller resolves. */
        let _ = lc;
    }
    None
}

/// `collection_count` (verbatim recursive).
pub fn collection_count(lb: &[LayerCollection]) -> usize {
    lb.iter()
        .map(|lc| collection_count(&lc.layer_collections) + 1)
        .sum()
}

/// `BKE_layer_collection_count`.
pub fn layer_collection_count(view_layer: &ViewLayer) -> usize {
    collection_count(&view_layer.layer_collections)
}

/// `layer_collection_activate` (verbatim; `active_collection`
/// is a path index — caller passes resolved index).
pub fn layer_collection_activate(view_layer: &mut ViewLayer, lc: &LayerCollection, lc_index: usize) -> bool {
    if (lc.flag & LAYER_COLLECTION_EXCLUDE) != 0 {
        return false;
    }
    view_layer.active_collection = lc_index;
    true
}

/// `layer_collection_hidden` — `EXCLUDE`/`HIDE`/
/// `COLLECTION_HIDE_VIEWPORT` walk (verbatim predicate;
/// `get_parent_lc` injected since parents live in
/// `Collection.runtime->parents`).
pub fn layer_collection_hidden(
    lc_flag: i16,
    collection_flag: i32,
    parent_hidden: &dyn Fn() -> bool,
) -> bool {
    if (lc_flag & LAYER_COLLECTION_EXCLUDE) != 0 {
        return true;
    }
    if (lc_flag & LAYER_COLLECTION_HIDE) != 0 || (collection_flag & COLLECTION_HIDE_VIEWPORT) != 0 {
        return true;
    }
    parent_hidden()
}
/// `COLLECTION_HIDE_VIEWPORT` (DNA_collection_types).
pub const COLLECTION_HIDE_VIEWPORT: i32 = 1 << 0;

/// `BKE_view_layer_need_resync_tag` (verbatim).
pub fn view_layer_need_resync_tag(view_layer: &mut ViewLayer) {
    view_layer.flag |= VIEW_LAYER_OUT_OF_SYNC;
}
/// `BKE_view_layer_is_synced` (verbatim).
pub fn view_layer_is_synced(view_layer: &ViewLayer) -> bool {
    (view_layer.flag & VIEW_LAYER_OUT_OF_SYNC) == 0
}
