//! Port of `intern/light_linking.cc` — per-object light linking:
//! `LightLinking` (receiver/blocker collections), collection
//! assign/get/new, receiver add + ordering (before/after), unlink,
//! and `link_receiver_to_emitter` / `select_receivers_of_emitter`.
//!
//! `Collection.gobject`/`Collection.children` are `Vec` arenas of
//! `CollectionObject`/`CollectionChild` (matching `collection.rs`);
//! ID handles are `usize`. DEG tags become recorded updates.

/// `LightLinkingType`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LightLinkingType {
    /// `LIGHT_LINKING_RECEIVER`.
    Receiver,
    /// `LIGHT_LINKING_BLOCKER`.
    Blocker,
}

/// `eCollectionLightLinkingState`.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum CollectionLightLinkingState {
    /// `COLLECTION_LIGHT_LINKING_STATE_INCLUDE`.
    #[default]
    Include,
    /// `COLLECTION_LIGHT_LINKING_STATE_EXCLUDE`.
    Exclude,
}

/// `ID_OB` / `ID_GR` — the two ID types that can be receivers.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReceiverIdType {
    Object,
    Collection,
    Other,
}

/// `LightLinking` (DNA_light_linking_defaults? — object side data).
#[derive(Default)]
pub struct LightLinking {
    /// `Collection *receiver_collection`.
    pub receiver_collection: usize,
    /// `Collection *blocker_collection`.
    pub blocker_collection: usize,
}

/// `CollectionObject` — `.gobject` entry carrying link state.
pub struct CollectionObject {
    pub ob: usize,
    /// `CollectionLightLinking light_linking`.
    pub link_state: CollectionLightLinkingState,
}

/// `CollectionChild` — `.children` entry carrying link state.
pub struct CollectionChild {
    pub collection: usize,
    pub link_state: CollectionLightLinkingState,
}

/// `Collection` light-linking surface.
#[derive(Default)]
pub struct LinkingCollection {
    pub gobject: Vec<CollectionObject>,
    pub children: Vec<CollectionChild>,
}

/// `BKE_light_linking_ensure` — `object.light_linking` is
/// `Option<LightLinking>` in Rust; allocate when missing.
pub fn bke_light_linking_ensure(light_linking: &mut Option<LightLinking>) {
    if light_linking.is_none() {
        *light_linking = Some(LightLinking::default());
    }
}

/// `BKE_light_linking_collection_get`.
pub fn bke_light_linking_collection_get(
    light_linking: Option<&LightLinking>,
    link_type: LightLinkingType,
) -> usize {
    let Some(ll) = light_linking else { return 0 };
    match link_type {
        LightLinkingType::Receiver => ll.receiver_collection,
        LightLinkingType::Blocker => ll.blocker_collection,
    }
}

/// `BKE_light_linking_copy` — clone both collections; `id_us_plus`
/// unless `no_user_refcount`.
pub fn bke_light_linking_copy(
    src: Option<&LightLinking>,
    no_user_refcount: bool,
    id_us_plus: &mut dyn FnMut(usize),
) -> Option<LightLinking> {
    let src = src?;
    let dst = LightLinking {
        receiver_collection: src.receiver_collection,
        blocker_collection: src.blocker_collection,
    };
    if !no_user_refcount {
        if src.receiver_collection != 0 {
            id_us_plus(src.receiver_collection);
        }
        if src.blocker_collection != 0 {
            id_us_plus(src.blocker_collection);
        }
    }
    Some(dst)
}

/// `BKE_light_linking_delete` — `id_us_min` both, drop the struct.
pub fn bke_light_linking_delete(
    light_linking: &mut Option<LightLinking>,
    no_user_refcount: bool,
    id_us_min: &mut dyn FnMut(usize),
) {
    if let Some(ll) = light_linking.take() {
        if !no_user_refcount {
            if ll.receiver_collection != 0 {
                id_us_min(ll.receiver_collection);
            }
            if ll.blocker_collection != 0 {
                id_us_min(ll.blocker_collection);
            }
        }
    }
}

/// `BKE_light_linking_free_if_empty`.
pub fn bke_light_linking_free_if_empty(
    light_linking: &mut Option<LightLinking>,
    id_us_min: &mut dyn FnMut(usize),
) {
    if let Some(ll) = light_linking {
        if ll.receiver_collection == 0 && ll.blocker_collection == 0 {
            bke_light_linking_delete(light_linking, true, id_us_min);
        }
    }
}

/// `get_default_collection_name` — `"Light Linking for {obname}"` /
/// `"Shadow Linking for {obname}"`.
pub fn get_default_collection_name(object_name: &str, link_type: LightLinkingType) -> String {
    match link_type {
        LightLinkingType::Receiver => format!("Light Linking for {object_name}"),
        LightLinkingType::Blocker => format!("Shadow Linking for {object_name}"),
    }
}

/// `BKE_light_linking_collection_assign_only`.
pub fn bke_light_linking_collection_assign_only(
    light_linking: &mut Option<LightLinking>,
    new_collection: usize,
    link_type: LightLinkingType,
    id_us_min: &mut dyn FnMut(usize),
    id_us_plus: &mut dyn FnMut(usize),
) {
    let old_collection = bke_light_linking_collection_get(light_linking.as_ref(), link_type);
    if old_collection != 0 {
        id_us_min(old_collection);
    }

    if new_collection != 0 {
        bke_light_linking_ensure(light_linking);
    }

    if let Some(ll) = light_linking {
        match link_type {
            LightLinkingType::Receiver => ll.receiver_collection = new_collection,
            LightLinkingType::Blocker => ll.blocker_collection = new_collection,
        }
    }

    if new_collection != 0 {
        id_us_plus(new_collection);
    }

    bke_light_linking_free_if_empty(light_linking, id_us_min);
}

/// `find_collection_object`.
pub fn find_collection_object(collection: &LinkingCollection, object: usize) -> Option<usize> {
    collection
        .gobject
        .iter()
        .position(|co| co.ob == object)
}

/// `find_collection_child`.
pub fn find_collection_child(collection: &LinkingCollection, child: usize) -> Option<usize> {
    collection
        .children
        .iter()
        .position(|cc| cc.collection == child)
}

/// `light_linking_collection_add_object` — push onto `gobject`,
/// return index of the entry (`BKE_collection_object_add` folded in).
pub fn light_linking_collection_add_object(
    collection: &mut LinkingCollection,
    object: usize,
) -> Option<usize> {
    collection.gobject.push(CollectionObject {
        ob: object,
        link_state: CollectionLightLinkingState::Include,
    });
    find_collection_object(collection, object)
}

/// `light_linking_collection_add_collection` — push onto `children`.
pub fn light_linking_collection_add_collection(
    collection: &mut LinkingCollection,
    child: usize,
) -> Option<usize> {
    collection.children.push(CollectionChild {
        collection: child,
        link_state: CollectionLightLinkingState::Include,
    });
    find_collection_child(collection, child)
}

/// `light_linking_add_receiver_to_collection` — add an object or
/// child collection and set its `link_state`.
///
/// `receiver_id_type` + `allow_all_objects` gate empties/non-geometry
/// objects exactly like the C (`has_geometry` is injected).
#[allow(clippy::too_many_arguments)]
pub fn light_linking_add_receiver_to_collection(
    collection: &mut LinkingCollection,
    receiver: usize,
    receiver_id_type: ReceiverIdType,
    link_state: CollectionLightLinkingState,
    allow_all_objects: bool,
    has_geometry: &dyn Fn(usize) -> bool,
) -> Option<usize> {
    let idx = match receiver_id_type {
        ReceiverIdType::Object => {
            if !allow_all_objects && !has_geometry(receiver) {
                return None;
            }
            light_linking_collection_add_object(collection, receiver)
        }
        ReceiverIdType::Collection => {
            light_linking_collection_add_collection(collection, receiver)
        }
        ReceiverIdType::Other => return None,
    };

    let idx = idx?;
    match receiver_id_type {
        ReceiverIdType::Object => collection.gobject[idx].link_state = link_state,
        ReceiverIdType::Collection => collection.children[idx].link_state = link_state,
        ReceiverIdType::Other => {}
    }
    Some(idx)
}

/// `order_collection_receiver_before`.
pub fn order_collection_receiver_before(
    collection: &mut LinkingCollection,
    receiver: usize,
    before: usize,
    before_id_type: ReceiverIdType,
) {
    let Some(child_i) = find_collection_child(collection, receiver) else {
        return;
    };

    if before_id_type != ReceiverIdType::Collection {
        /* Adding before object: move the collection to the very
         * bottom. */
        let entry = collection.children.remove(child_i);
        collection.children.push(entry);
        return;
    }

    let Some(before_i) = find_collection_child(collection, before) else {
        return;
    };

    let entry = collection.children.remove(child_i);
    let insert_i = if child_i < before_i { before_i - 1 } else { before_i };
    collection.children.insert(insert_i, entry);
}

/// `order_collection_receiver_after`.
pub fn order_collection_receiver_after(
    collection: &mut LinkingCollection,
    receiver: usize,
    after: usize,
    after_id_type: ReceiverIdType,
) {
    let Some(child_i) = find_collection_child(collection, receiver) else {
        return;
    };

    if after_id_type != ReceiverIdType::Collection {
        /* move to the very bottom. */
        let entry = collection.children.remove(child_i);
        collection.children.push(entry);
        return;
    }

    let Some(after_i) = find_collection_child(collection, after) else {
        return;
    };

    let entry = collection.children.remove(child_i);
    let insert_i = if child_i < after_i { after_i } else { after_i + 1 };
    collection.children.insert(insert_i, entry);
}

/// `order_object_receiver_before`.
pub fn order_object_receiver_before(
    collection: &mut LinkingCollection,
    receiver: usize,
    before: usize,
    before_id_type: ReceiverIdType,
) {
    let Some(obj_i) = find_collection_object(collection, receiver) else {
        return;
    };

    if before_id_type != ReceiverIdType::Object {
        /* Adding before collection: move the receiver to the very
         * beginning of the child objects list. */
        let entry = collection.gobject.remove(obj_i);
        collection.gobject.insert(0, entry);
        return;
    }

    let Some(before_i) = find_collection_object(collection, before) else {
        return;
    };

    let entry = collection.gobject.remove(obj_i);
    let insert_i = if obj_i < before_i { before_i - 1 } else { before_i };
    collection.gobject.insert(insert_i, entry);
}

/// `order_object_receiver_after`.
pub fn order_object_receiver_after(
    collection: &mut LinkingCollection,
    receiver: usize,
    after: usize,
    after_id_type: ReceiverIdType,
) {
    let Some(obj_i) = find_collection_object(collection, receiver) else {
        return;
    };

    if after_id_type != ReceiverIdType::Object {
        /* move to head of objects. */
        let entry = collection.gobject.remove(obj_i);
        collection.gobject.insert(0, entry);
        return;
    }

    let Some(after_i) = find_collection_object(collection, after) else {
        return;
    };

    let entry = collection.gobject.remove(obj_i);
    let insert_i = if obj_i < after_i { after_i } else { after_i + 1 };
    collection.gobject.insert(insert_i, entry);
}

/// `BKE_light_linking_add_receiver_to_collection_before`.
#[allow(clippy::too_many_arguments)]
pub fn bke_light_linking_add_receiver_to_collection_before(
    collection: &mut LinkingCollection,
    receiver: usize,
    receiver_id_type: ReceiverIdType,
    before: usize,
    before_id_type: ReceiverIdType,
    link_state: CollectionLightLinkingState,
    has_geometry: &dyn Fn(usize) -> bool,
) {
    light_linking_add_receiver_to_collection(
        collection,
        receiver,
        receiver_id_type,
        link_state,
        true, /* allow_all_objects */
        has_geometry,
    );

    match receiver_id_type {
        ReceiverIdType::Object => {
            order_object_receiver_before(collection, receiver, before, before_id_type);
        }
        ReceiverIdType::Collection => {
            order_collection_receiver_before(collection, receiver, before, before_id_type);
        }
        ReceiverIdType::Other => {}
    }
}

/// `BKE_light_linking_add_receiver_to_collection_after`.
#[allow(clippy::too_many_arguments)]
pub fn bke_light_linking_add_receiver_to_collection_after(
    collection: &mut LinkingCollection,
    receiver: usize,
    receiver_id_type: ReceiverIdType,
    after: usize,
    after_id_type: ReceiverIdType,
    link_state: CollectionLightLinkingState,
    has_geometry: &dyn Fn(usize) -> bool,
) {
    light_linking_add_receiver_to_collection(
        collection,
        receiver,
        receiver_id_type,
        link_state,
        true, /* allow_all_objects */
        has_geometry,
    );

    match receiver_id_type {
        ReceiverIdType::Object => {
            order_object_receiver_after(collection, receiver, after, after_id_type);
        }
        ReceiverIdType::Collection => {
            order_collection_receiver_after(collection, receiver, after, after_id_type);
        }
        ReceiverIdType::Other => {}
    }
}

/// `BKE_light_linking_unlink_id_from_collection` — returns `Err` for
/// unsupported ID types (C reports to `ReportList`).
pub fn bke_light_linking_unlink_id_from_collection(
    collection: &mut LinkingCollection,
    id: usize,
    id_type: ReceiverIdType,
) -> Result<(), String> {
    match id_type {
        ReceiverIdType::Object => {
            if let Some(i) = find_collection_object(collection, id) {
                collection.gobject.remove(i);
            }
        }
        ReceiverIdType::Collection => {
            if let Some(i) = find_collection_child(collection, id) {
                collection.children.remove(i);
            }
        }
        ReceiverIdType::Other => {
            return Err(format!(
                "Cannot unlink unsupported id from light linking collection"
            ));
        }
    }
    Ok(())
}

/// `BKE_light_linking_link_receiver_to_emitter` — the gate plus
/// ensure-collection + add-receiver.
///
/// `emitter_ll` is the emitter's `Option<LightLinking>` slot,
/// `new_collection` creates+registers a collection when needed.
#[allow(clippy::too_many_arguments)]
pub fn bke_light_linking_link_receiver_to_emitter(
    emitter_ll: &mut Option<LightLinking>,
    emitter_name: &str,
    receiver: usize,
    receiver_has_geometry: bool,
    link_type: LightLinkingType,
    link_state: CollectionLightLinkingState,
    collection_access: &mut dyn FnMut(usize) -> LinkingCollection,
    new_collection: &mut dyn FnMut(&str) -> (usize, LinkingCollection),
    id_us_min: &mut dyn FnMut(usize),
    id_us_plus: &mut dyn FnMut(usize),
    collection_store: &mut dyn FnMut(usize, LinkingCollection),
) {
    if !receiver_has_geometry {
        return;
    }

    let mut collection = bke_light_linking_collection_get(emitter_ll.as_ref(), link_type);
    let mut ll_coll = if collection == 0 {
        /* `BKE_light_linking_collection_new`. */
        let (handle, coll) = new_collection(&get_default_collection_name(emitter_name, link_type));
        bke_light_linking_collection_assign_only(
            emitter_ll,
            handle,
            link_type,
            id_us_min,
            id_us_plus,
        );
        collection = handle;
        coll
    } else {
        collection_access(collection)
    };

    light_linking_add_receiver_to_collection(
        &mut ll_coll,
        receiver,
        ReceiverIdType::Object,
        link_state,
        false,
        &|_| receiver_has_geometry,
    );
    collection_store(collection, ll_coll);
}

/// `BKE_light_linking_select_receivers_of_emitter` — returns the set
/// of receiver object handles in the emitter's linking collection
/// (caller deselects others, selects these).
pub fn bke_light_linking_select_receivers_of_emitter(
    emitter_ll: Option<&LightLinking>,
    link_type: LightLinkingType,
    collection: Option<&LinkingCollection>,
) -> Vec<usize> {
    if bke_light_linking_collection_get(emitter_ll, link_type) == 0 {
        return Vec::new();
    }
    let Some(collection) = collection else {
        return Vec::new();
    };
    collection.gobject.iter().map(|co| co.ob).collect()
}
