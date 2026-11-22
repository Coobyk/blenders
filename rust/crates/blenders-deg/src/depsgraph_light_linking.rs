//! Port of `intern/depsgraph_light_linking.cc` +
/// `intern/depsgraph_light_linking.hh` — light/shadow linking
/// set evaluation.

use std::collections::HashMap;

/// `LightLinkingType` verbatim.
pub const LIGHT_LINKING_RECEIVER: i32 = 0;
/// `LIGHT_LINKING_BLOCKER`.
pub const LIGHT_LINKING_BLOCKER: i32 = 1;

/// `eCollectionLightLinkingState` verbatim.
pub const COLLECTION_LIGHT_LINKING_STATE_INCLUDE: i8 = 1;
/// `COLLECTION_LIGHT_LINKING_STATE_EXCLUDE`.
pub const COLLECTION_LIGHT_LINKING_STATE_EXCLUDE: i8 = -1;

/// `LightSet` verbatim.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LightSet {
    /// `include_collection_mask`.
    pub include_collection_mask: u64,
    /// `exclude_collection_mask`.
    pub exclude_collection_mask: u64,
}

/// `LightSet::MAX_ID`.
pub const LIGHT_SET_MAX_ID: u64 = 63;
/// `LightSet::DEFAULT_ID`.
pub const LIGHT_SET_DEFAULT_ID: u64 = 0;

/// `EmitterSetMembership` verbatim.
#[derive(Clone, Copy, Default)]
pub struct EmitterSetMembership {
    /// `included_sets_mask`.
    pub included_sets_mask: u64,
    /// `excluded_sets_mask`.
    pub excluded_sets_mask: u64,
}

/// `EmitterSetMembership::SET_MEMBERSHIP_ALL`.
pub const SET_MEMBERSHIP_ALL: u64 = u64::MAX;

impl EmitterSetMembership {
    /// `get_mask` (verbatim).
    pub fn get_mask(&self) -> u64 {
        let effective_included_mask = if self.included_sets_mask != 0 {
            self.included_sets_mask
        } else {
            SET_MEMBERSHIP_ALL
        };
        effective_included_mask & !self.excluded_sets_mask
    }
}

/// `EmitterData` verbatim.
#[derive(Clone, Copy, Default)]
pub struct EmitterData {
    /// `collection_mask`.
    pub collection_mask: u64,
    /// `light_membership`.
    pub light_membership: EmitterSetMembership,
    /// `shadow_membership`.
    pub shadow_membership: EmitterSetMembership,
}

/// `EmitterData::MAX_COLLECTION_ID`.
pub const EMITTER_DATA_MAX_COLLECTION_ID: u64 = 63;

/// `LightLinkingRuntime` verbatim (assigned onto evaluated
/// `Object.light_linking->runtime`).
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct LightLinkingRuntime {
    /// `light_set_membership`.
    pub light_set_membership: u64,
    /// `shadow_set_membership`.
    pub shadow_set_membership: u64,
    /// `receiver_light_set`.
    pub receiver_light_set: u64,
    /// `blocker_shadow_set`.
    pub blocker_shadow_set: u64,
}

/// `runtime_no_links` static (verbatim).
pub const RUNTIME_NO_LINKS: LightLinkingRuntime = LightLinkingRuntime {
    light_set_membership: SET_MEMBERSHIP_ALL,
    shadow_set_membership: SET_MEMBERSHIP_ALL,
    receiver_light_set: 0,
    blocker_shadow_set: 0,
};

/// `LightSetIDManager` (verbatim).
pub struct LightSetIdManager {
    /// `next_light_set_id_`.
    pub next_light_set_id: u64,
    /// `light_set_id_map_`.
    pub light_set_id_map: HashMap<LightSet, u64>,
}

impl LightSetIdManager {
    /// `LightSetIDManager(scene)` (verbatim).
    pub fn new() -> Self {
        Self {
            next_light_set_id: LIGHT_SET_DEFAULT_ID + 1,
            light_set_id_map: HashMap::new(),
        }
    }

    /// `get` (verbatim — returns false when past `MAX_ID`).
    pub fn get(&mut self, light_set: LightSet) -> (u64, bool) {
        let id = *self.light_set_id_map.entry(light_set).or_insert_with(|| {
            let new_id = self.next_light_set_id;
            self.next_light_set_id += 1;
            new_id
        });
        (id, id <= LIGHT_SET_MAX_ID)
    }
}

/// `EmitterDataMap` (verbatim — emitter→data keyed by
/// emitter's linking collection, `usize` handles).
pub struct EmitterDataMap {
    /// `link_type_`.
    pub link_type: i32,
    /// `emitter_data_map_` keyed by the collection id that
    /// `BKE_light_linking_collection_get` returns.
    pub emitter_data_map: HashMap<usize, EmitterData>,
    /// `next_collection_id_`.
    pub next_collection_id: u64,
}

impl EmitterDataMap {
    /// `EmitterDataMap(link_type)` (verbatim).
    pub fn new(link_type: i32) -> Self {
        Self {
            link_type,
            emitter_data_map: HashMap::new(),
            next_collection_id: 0,
        }
    }

    /// `is_empty` (verbatim).
    pub fn is_empty(&self) -> bool {
        self.emitter_data_map.is_empty()
    }

    /// `clear` (verbatim).
    pub fn clear(&mut self) {
        self.emitter_data_map.clear();
        self.next_collection_id = 0;
    }

    /// `ensure_data_if_possible` (verbatim — `get_collection`
    /// resolves the emitter's linking collection).
    pub fn ensure_data_if_possible(
        &mut self,
        collection: usize,
    ) -> Option<&mut EmitterData> {
        let next = &mut self.next_collection_id;
        let data = self.emitter_data_map.entry(collection).or_insert_with(|| {
            let collection_id = *next;
            *next += 1;
            let mut d = EmitterData::default();
            if collection_id > EMITTER_DATA_MAX_COLLECTION_ID {
                d.collection_mask = 0;
            } else {
                d.collection_mask = 1u64 << collection_id;
            }
            d
        });
        if data.collection_mask == 0 {
            return None;
        }
        Some(data)
    }

    /// `get_data` (verbatim — `collection_orig` resolved by
    /// caller through `DEG_get_original`).
    pub fn get_data(&self, collection_orig: Option<usize>) -> Option<&EmitterData> {
        self.emitter_data_map.get(&collection_orig?)
    }

    /// `can_skip_emitter` (verbatim — `collection` already
    /// resolved to the orig collection id or None).
    pub fn can_skip_emitter(&self, collection: Option<usize>) -> bool {
        match collection {
            None => true,
            Some(c) => self.emitter_data_map.contains_key(&c),
        }
    }
}

/// `LinkingData` (verbatim — receiver/blocker-centric sets,
/// `usize` object handles).
pub struct LinkingData {
    /// `link_type_`.
    pub link_type: i32,
    /// `light_linked_sets_` (receiver → set being built).
    pub light_linked_sets: HashMap<usize, LightSet>,
    /// `object_light_sets_` (receiver → final set id).
    pub object_light_sets: HashMap<usize, u64>,
}

impl LinkingData {
    /// `LinkingData(link_type)` (verbatim).
    pub fn new(link_type: i32) -> Self {
        Self {
            link_type,
            light_linked_sets: HashMap::new(),
            object_light_sets: HashMap::new(),
        }
    }

    /// `clear` (verbatim).
    pub fn clear(&mut self) {
        self.light_linked_sets.clear();
        self.object_light_sets.clear();
    }

    /// `link_object` (verbatim).
    pub fn link_object(
        &mut self,
        emitter_data: &EmitterData,
        link_state: i8,
        object: usize,
    ) {
        let light_set = self.light_linked_sets.entry(object).or_default();
        match link_state {
            COLLECTION_LIGHT_LINKING_STATE_INCLUDE => {
                light_set.include_collection_mask |= emitter_data.collection_mask;
                light_set.exclude_collection_mask &= !emitter_data.collection_mask;
            }
            COLLECTION_LIGHT_LINKING_STATE_EXCLUDE => {
                light_set.exclude_collection_mask |= emitter_data.collection_mask;
                light_set.include_collection_mask &= !emitter_data.collection_mask;
            }
            _ => {}
        }
    }

    /// `ensure_light_set_for` (verbatim).
    pub fn ensure_light_set_for(&mut self, object: usize) -> &mut LightSet {
        self.light_linked_sets.entry(object).or_default()
    }

    /// `clear_after_build` (verbatim).
    pub fn clear_after_build(&mut self) {
        self.light_linked_sets.clear();
    }

    /// `end_build` (verbatim).
    pub fn end_build(&mut self, emitter_data_map: &mut EmitterDataMap) {
        let mut id_manager = LightSetIdManager::new();
        let items: Vec<(usize, LightSet)> = self
            .light_linked_sets
            .iter()
            .map(|(&k, &v)| (k, v))
            .collect();
        for (receiver, light_set) in items {
            let (light_set_id, ok) = id_manager.get(light_set);
            if !ok {
                continue;
            }
            let light_set_mask = 1u64 << light_set_id;
            self.object_light_sets.insert(receiver, light_set_id);
            /* `update_emitters_membership` (verbatim). */
            let link_type = self.link_type;
            for emitter_data in emitter_data_map.emitter_data_map.values_mut() {
                let set_membership = if link_type == LIGHT_LINKING_BLOCKER {
                    &mut emitter_data.shadow_membership
                } else {
                    &mut emitter_data.light_membership
                };
                if emitter_data.collection_mask & light_set.include_collection_mask != 0 {
                    set_membership.included_sets_mask |= light_set_mask;
                }
                if emitter_data.collection_mask & light_set.exclude_collection_mask != 0 {
                    set_membership.excluded_sets_mask |= light_set_mask;
                }
            }
        }
        self.clear_after_build();
    }

    /// `get_light_set_for` (verbatim — `object_orig`
    /// resolved by caller).
    pub fn get_light_set_for(&self, object_orig: usize) -> u64 {
        self.object_light_sets
            .get(&object_orig)
            .copied()
            .unwrap_or(LIGHT_SET_DEFAULT_ID)
    }
}

/// `Cache` (verbatim — light + shadow maps and linking).
pub struct LightLinkingCache {
    /// `light_emitter_data_map_`.
    pub light_emitter_data_map: EmitterDataMap,
    /// `shadow_emitter_data_map_`.
    pub shadow_emitter_data_map: EmitterDataMap,
    /// `light_linking_`.
    pub light_linking: LinkingData,
    /// `shadow_linking_`.
    pub shadow_linking: LinkingData,
}

impl Default for LightLinkingCache {
    fn default() -> Self {
        Self::new()
    }
}

impl LightLinkingCache {
    /// `Cache()` (verbatim).
    pub fn new() -> Self {
        Self {
            light_emitter_data_map: EmitterDataMap::new(LIGHT_LINKING_RECEIVER),
            shadow_emitter_data_map: EmitterDataMap::new(LIGHT_LINKING_BLOCKER),
            light_linking: LinkingData::new(LIGHT_LINKING_RECEIVER),
            shadow_linking: LinkingData::new(LIGHT_LINKING_BLOCKER),
        }
    }

    /// `clear` (verbatim).
    pub fn clear(&mut self) {
        self.light_emitter_data_map.clear();
        self.shadow_emitter_data_map.clear();
        self.light_linking.clear();
        self.shadow_linking.clear();
    }

    /// `add_light_linking_emitter` (verbatim —
    /// `get_collection(emitter)` and the
    /// `foreach_light_collection_object` traversal injected).
    pub fn add_light_linking_emitter(
        &mut self,
        emitter: usize,
        emitter_collection: Option<usize>,
        foreach_collection_object: &mut dyn FnMut(&mut dyn FnMut(i8, usize)),
    ) {
        if self.light_emitter_data_map.can_skip_emitter(emitter_collection) {
            return;
        }
        let Some(collection) = emitter_collection else {
            return;
        };
        if self
            .light_emitter_data_map
            .ensure_data_if_possible(collection)
            .is_none()
        {
            return;
        }
        let emitter_data = *self
            .light_emitter_data_map
            .get_data(Some(collection))
            .unwrap();
        let linking = &mut self.light_linking;
        foreach_collection_object(&mut |link_state, receiver| {
            linking.link_object(&emitter_data, link_state, receiver);
        });
        let _ = emitter;
    }

    /// `add_shadow_linking_emitter` (verbatim).
    pub fn add_shadow_linking_emitter(
        &mut self,
        emitter: usize,
        emitter_collection: Option<usize>,
        foreach_collection_object: &mut dyn FnMut(&mut dyn FnMut(i8, usize)),
    ) {
        if self.shadow_emitter_data_map.can_skip_emitter(emitter_collection) {
            return;
        }
        let Some(collection) = emitter_collection else {
            return;
        };
        if self
            .shadow_emitter_data_map
            .ensure_data_if_possible(collection)
            .is_none()
        {
            return;
        }
        let emitter_data = *self
            .shadow_emitter_data_map
            .get_data(Some(collection))
            .unwrap();
        let linking = &mut self.shadow_linking;
        foreach_collection_object(&mut |link_state, blocker| {
            linking.link_object(&emitter_data, link_state, blocker);
        });
        let _ = emitter;
    }

    /// `end_build` (verbatim).
    pub fn end_build(&mut self) {
        if !self.has_light_linking() {
            return;
        }
        self.light_linking.end_build(&mut self.light_emitter_data_map);
        self.shadow_linking.end_build(&mut self.shadow_emitter_data_map);
    }

    /// `has_light_linking` (verbatim).
    pub fn has_light_linking(&self) -> bool {
        !self.light_emitter_data_map.is_empty() || !self.shadow_emitter_data_map.is_empty()
    }

    /// `eval_runtime_data` (verbatim — returns the runtime
    /// to assign; `has_light_linking`/`need_runtime`/
    /// `light_linking` presence decisions made by the
    /// caller which owns the object).
    pub fn eval_runtime_data(
        &self,
        object_orig: usize,
        light_emitter_collection_orig: Option<usize>,
        shadow_emitter_collection_orig: Option<usize>,
        has_light_linking_ptr: bool,
    ) -> Option<LightLinkingRuntime> {
        if !self.has_light_linking() {
            if has_light_linking_ptr {
                return Some(RUNTIME_NO_LINKS);
            }
            return None;
        }
        let mut runtime = LightLinkingRuntime::default();
        runtime.receiver_light_set = self.light_linking.get_light_set_for(object_orig);
        runtime.blocker_shadow_set = self.shadow_linking.get_light_set_for(object_orig);
        runtime.light_set_membership = self
            .light_emitter_data_map
            .get_data(light_emitter_collection_orig)
            .map(|d| d.light_membership.get_mask())
            .unwrap_or(SET_MEMBERSHIP_ALL);
        runtime.shadow_set_membership = self
            .shadow_emitter_data_map
            .get_data(shadow_emitter_collection_orig)
            .map(|d| d.shadow_membership.get_mask())
            .unwrap_or(SET_MEMBERSHIP_ALL);
        if runtime == RUNTIME_NO_LINKS {
            return None;
        }
        Some(runtime)
    }
}
