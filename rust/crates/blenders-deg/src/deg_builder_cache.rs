//! Port of `intern/builder/deg_builder_cache.cc` —
/// animated-property cache used while building.

use std::collections::{HashMap, HashSet};

/// `AnimatedPropertyID` (verbatim — data pointer +
/// property handle).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimatedPropertyID {
    /// `PointerRNA.data`.
    pub data: usize,
    /// `PropertyRNA *`.
    pub property_rna: usize,
}

impl AnimatedPropertyID {
    /// `AnimatedPropertyID::hash` (verbatim).
    pub fn hash(&self) -> u64 {
        let ptr1 = self.data as u64;
        let ptr2 = self.property_rna as u64;
        ((ptr1 >> 4).wrapping_mul(33)) ^ (ptr2 >> 4)
    }
}

/// `AnimatedPropertyStorage` (verbatim — per-id animated
/// property sets; `is_fully_initialized` lazy flag).
#[derive(Default)]
pub struct AnimatedPropertyStorage {
    /// `animated_objects_set`.
    pub animated_objects_set: HashSet<usize>,
    /// `animated_properties_set`.
    pub animated_properties_set: HashSet<(usize, usize)>,
    /// `is_fully_initialized`.
    pub is_fully_initialized: bool,
}

impl AnimatedPropertyStorage {
    /// `tagPropertyAsAnimated` (verbatim).
    pub fn tag_property_as_animated(&mut self, property_id: AnimatedPropertyID) {
        self.animated_objects_set.insert(property_id.data);
        self.animated_properties_set
            .insert((property_id.data, property_id.property_rna));
    }

    /// `isPropertyAnimated` (verbatim).
    pub fn is_property_animated(&self, property_id: AnimatedPropertyID) -> bool {
        self.animated_properties_set
            .contains(&(property_id.data, property_id.property_rna))
    }

    /// `isAnyPropertyAnimated` (verbatim).
    pub fn is_any_property_animated(&self, data: usize) -> bool {
        self.animated_objects_set.contains(&data)
    }
}

/// `DepsgraphBuilderCache` (verbatim — id → storage map).
#[derive(Default)]
pub struct DepsgraphBuilderCache {
    /// `animated_property_storage_map_`.
    pub animated_property_storage_map: HashMap<usize, AnimatedPropertyStorage>,
}

impl DepsgraphBuilderCache {
    /// `ensureAnimatedPropertyStorage` (verbatim).
    pub fn ensure_animated_property_storage(
        &mut self,
        id: usize,
    ) -> &mut AnimatedPropertyStorage {
        self.animated_property_storage_map.entry(id).or_default()
    }

    /// `ensureInitializedAnimatedPropertyStorage` (verbatim
    /// — runs `initializeFromID` once via the injected
    /// initializer which walks the id's fcurves).
    pub fn ensure_initialized_animated_property_storage(
        &mut self,
        id: usize,
        initialize: &mut dyn FnMut(&mut AnimatedPropertyStorage),
    ) {
        let storage = self.animated_property_storage_map.entry(id).or_default();
        if !storage.is_fully_initialized {
            initialize(storage);
            storage.is_fully_initialized = true;
        }
    }
}
