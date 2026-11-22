//! Port of `intern/builder/deg_builder_map.cc` — `BuilderMap`.

use std::collections::{HashMap, HashSet};

/// `BuilderMap` — per-ID tag bit sets.
#[derive(Default)]
pub struct BuilderMap {
    /// `id_tags_` (`id` → arena index).
    pub id_tags: HashMap<usize, i32>,
}

impl BuilderMap {
    /// `check_is_built` (verbatim).
    pub fn check_is_built(&self, id: usize, tag: i32) -> bool {
        (self.get_id_tag(id) & tag) == tag
    }

    /// `tag_built` (verbatim).
    pub fn tag_built(&mut self, id: usize, tag: i32) {
        *self.id_tags.entry(id).or_insert(0) |= tag;
    }

    /// `check_is_built_and_tag` (verbatim).
    pub fn check_is_built_and_tag(&mut self, id: usize, tag: i32) -> bool {
        let id_tag = self.id_tags.entry(id).or_insert(0);
        let result = (*id_tag & tag) == tag;
        *id_tag |= tag;
        result
    }

    /// `get_ID_tag` (verbatim).
    pub fn get_id_tag(&self, id: usize) -> i32 {
        *self.id_tags.get(&id).unwrap_or(&0)
    }

    /// `get_ids` (verbatim).
    pub fn get_ids(&self) -> HashSet<usize> {
        self.id_tags.keys().copied().collect()
    }
}
