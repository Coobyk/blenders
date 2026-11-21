//! Port of `intern/main_namemap.cc` — the `UniqueName_Map`
//! machinery: per-(library, id-type) name pools tracking used `.NNN`
//! suffixes (`UniqueName_Value` bitmask + multi-usage counts),
//! `populate`/`add_name`/`remove_full_name`, the `id_name_final_build`
//! overflow fallback, and `BKE_main_namemap_get_unique_name`.
//!
//! `Main` iteration + `BKE_id_name`/`GS`/`id->lib` queries are
//! injected; per-library maps are kept in `LibraryNameMaps`.

use std::collections::HashMap;

use blenders_bli::string_utils::string_split_name_number;

/// `MAX_NUMBER`.
pub const MAX_NUMBER: i32 = 999_999_999;
/// `NO_AVAILABLE_NUMBER`.
pub const NO_AVAILABLE_NUMBER: i32 = -1;
/// `MAX_ID_NAME`.
pub const MAX_ID_NAME: usize = 66;

/// `UniqueName_Value` — used-number tracking for one base name.
#[derive(Default)]
pub struct UniqueNameValue {
    /// `mask` — bitmap of used numbers (`mask[n]` true when used);
    /// bit 0 unused.
    pub mask: Vec<bool>,
    /// `numbers_multi_usages` — numbers used more than once
    /// (global map only).
    pub numbers_multi_usages: Option<HashMap<i32, i32>>,
    /// `max_value_in_use`.
    pub max_value_in_use: Option<i32>,
}

/// `max_exact_tracking` — beyond this the mask stops growing.
const MAX_EXACT_TRACKING: i32 = 1023;

impl UniqueNameValue {
    /// `mark_used`.
    pub fn mark_used(&mut self, number: i32) {
        debug_assert!(number >= 0);
        if (0..=MAX_EXACT_TRACKING).contains(&number) {
            if self.mask.len() <= number as usize {
                self.mask.resize(number as usize + 1, false);
            }
            if self.mask[number as usize] {
                let multi = self.numbers_multi_usages.get_or_insert_with(HashMap::new);
                let count = multi.entry(number).or_insert(1);
                debug_assert!(*count >= 1);
                *count += 1;
            } else {
                self.mask[number as usize] = true;
            }
        }
        if number <= MAX_NUMBER {
            match &mut self.max_value_in_use {
                Some(max) => *max = (*max).max(number),
                None => self.max_value_in_use = Some(number),
            }
        }
    }

    /// `mark_unused`.
    pub fn mark_unused(&mut self, number: i32) {
        debug_assert!(number >= 0);
        if (0..=MAX_EXACT_TRACKING).contains(&number) {
            debug_assert!((number as usize) < self.mask.len());
            if let Some(multi) = self.numbers_multi_usages.as_mut() {
                if let Some(count) = multi.get_mut(&number) {
                    debug_assert!(*count > 1);
                    *count -= 1;
                    if *count == 1 {
                        multi.remove(&number);
                    }
                    return;
                }
            }
            self.mask[number as usize] = false;
        }
        if self.max_value_in_use == Some(number) {
            if number > 0 {
                *self.max_value_in_use.as_mut().unwrap() -= 1;
            } else {
                self.max_value_in_use = None;
            }
        }
    }

    /// `get_smallest_unused` — exact only up to
    /// `max_exact_tracking`; beyond that returns
    /// `max_value_in_use + 1` or `NO_AVAILABLE_NUMBER`.
    pub fn get_smallest_unused(&self) -> i32 {
        if self.mask.len() < 2 {
            return 1;
        }
        if let Some(i) = self.mask[1..].iter().position(|&b| !b) {
            return i as i32 + 1;
        }
        if self.mask.len() as i32 <= MAX_EXACT_TRACKING {
            return self.mask.len() as i32;
        }
        if let Some(max) = self.max_value_in_use {
            if max + 1 <= MAX_NUMBER {
                return max + 1;
            }
            return NO_AVAILABLE_NUMBER;
        }
        1
    }
}

/// `UniqueName_TypeMap`.
#[derive(Default)]
pub struct UniqueNameTypeMap {
    /// `full_names` — name → use count (count matters only for
    /// `is_global` maps).
    pub full_names: HashMap<String, i32>,
    /// `base_name_to_num_suffix`.
    pub base_name_to_num_suffix: HashMap<String, UniqueNameValue>,
}

/// `UniqueName_Map` — per-ID-type name maps. `type_maps` is keyed
/// by `BKE_idtype_idcode_to_index(id_type)` (index range stays under
/// 64).
pub struct UniqueNameMap {
    /// `type_maps`.
    pub type_maps: Vec<UniqueNameTypeMap>,
    /// `is_global`.
    pub is_global: bool,
}

impl UniqueNameMap {
    /// `UniqueName_Map(is_global)`.
    pub fn new(is_global: bool) -> Self {
        Self {
            type_maps: (0..64).map(|_| UniqueNameTypeMap::default()).collect(),
            is_global,
        }
    }

    /// `find_by_type`.
    pub fn find_by_type(&mut self, id_type_index: usize) -> &mut UniqueNameTypeMap {
        if id_type_index >= self.type_maps.len() {
            self.type_maps
                .resize_with(id_type_index + 1, UniqueNameTypeMap::default);
        }
        &mut self.type_maps[id_type_index]
    }

    /// `find_by_type` (read).
    pub fn find_by_type_ref(&self, id_type_index: usize) -> Option<&UniqueNameTypeMap> {
        self.type_maps.get(id_type_index)
    }

    /// `add_name`.
    pub fn add_name(
        &mut self,
        type_map_index: usize,
        name_full: &str,
        name_base: &str,
        number: i32,
    ) {
        debug_assert!(name_full.len() < MAX_ID_NAME - 2);
        let is_global = self.is_global;
        let type_map = self.find_by_type(type_map_index);
        if is_global {
            let count = type_map.full_names.entry(name_full.to_string()).or_insert(0);
            if *count == 0 {
                type_map
                    .base_name_to_num_suffix
                    .entry(name_base.to_string())
                    .or_insert_with(UniqueNameValue::default)
                    .mark_used(number);
            }
            *count += 1;
            return;
        }
        if type_map.full_names.insert(name_full.to_string(), 1).is_some() {
            debug_assert!(false, "name already in map");
            return;
        }
        type_map
            .base_name_to_num_suffix
            .entry(name_base.to_string())
            .or_insert_with(UniqueNameValue::default)
            .mark_used(number);
    }

    /// `remove_full_name`.
    pub fn remove_full_name(&mut self, type_map_index: usize, name_full: &str) {
        debug_assert!(name_full.len() < MAX_ID_NAME - 2);
        let (name_base, number) = string_split_name_number(name_full, '.');
        let is_global = self.is_global;
        let type_map = self.find_by_type(type_map_index);

        if is_global {
            match type_map.full_names.get_mut(name_full) {
                None => {
                    debug_assert!(false, "name not in map");
                    return;
                }
                Some(count) if *count > 1 => {
                    *count -= 1;
                    return;
                }
                Some(_) => {
                    type_map.full_names.remove(name_full);
                }
            }
        } else if type_map.full_names.remove(name_full).is_none() {
            debug_assert!(false, "name not in map");
            return;
        }

        let empty = match type_map.base_name_to_num_suffix.get_mut(&name_base) {
            Some(val) => {
                val.mark_unused(number);
                val.max_value_in_use.is_none()
            }
            None => return,
        };
        if empty {
            type_map.base_name_to_num_suffix.remove(&name_base);
        }
    }
}

/// `id_name_final_build` — produce `base.NNN`; on overflow (>66
/// chars) shorten the base or fall back to `_NNN`/hash suffixes.
/// Returns false when `r_name_final` needs another loop round
/// (i.e. the built name may still collide — caller re-checks).
pub fn id_name_final_build(
    type_map: &mut UniqueNameTypeMap,
    base_name: &str,
    number: i32,
    r_name_final: &mut String,
) -> bool {
    if number != NO_AVAILABLE_NUMBER {
        debug_assert!(number >= 0 && number <= MAX_NUMBER);
        *r_name_final = format!("{base_name}.{number:03}");
        if r_name_final.len() < MAX_ID_NAME - 2 {
            return true;
        }
    }

    *r_name_final = base_name.to_string();

    while r_name_final.len() > 8 {
        /* Truncate by one char (C strips invalid UTF-8 edges). */
        let mut truncated = r_name_final.clone();
        truncated.pop();
        while !truncated.is_char_boundary(truncated.len()) {
            truncated.pop();
        }
        *r_name_final = truncated;
        match type_map.base_name_to_num_suffix.get(r_name_final) {
            None => return false,
            Some(val) if val.max_value_in_use.unwrap_or(0) < MAX_NUMBER => return false,
            _ => {}
        }
    }

    let mut suffix: u64 = 1;
    let new_base_name = r_name_final.clone();
    *r_name_final = format!("{r_name_final}_{suffix:03}");
    while r_name_final.len() < MAX_ID_NAME - 2 - 12 {
        match type_map.base_name_to_num_suffix.get(r_name_final) {
            None => return false,
            Some(val) if val.max_value_in_use.unwrap_or(0) < MAX_NUMBER => return false,
            _ => {}
        }
        suffix += 1;
        *r_name_final = format!("{new_base_name}_{suffix:03}");
    }

    /* Brute-force hash suffixes (never happens in practice). */
    loop {
        let h = blenders_bli::hash_mm3::hash_mm3(r_name_final.as_bytes(), 0);
        *r_name_final = format!("{new_base_name}_{h}");
        match type_map.base_name_to_num_suffix.get(r_name_final) {
            None => return false,
            Some(val) if val.max_value_in_use.unwrap_or(0) < MAX_NUMBER => return false,
            _ => {}
        }
    }
}

/// `namemap_get_name` — mutate `r_name_full` to a unique name and
/// register it in both `name_map` and `name_map_other` when given.
/// `id_type_index` selects the type map. Returns `is_name_changed`.
pub fn namemap_get_name(
    name_map: &mut UniqueNameMap,
    mut name_map_other: Option<&mut UniqueNameMap>,
    id_type_index: usize,
    r_name_full: &mut String,
) -> bool {
    debug_assert!(r_name_full.len() < MAX_ID_NAME - 2);
    let mut is_name_changed = false;

    loop {
        let (name_base, number) = string_split_name_number(r_name_full, '.');
        {
            let type_map = name_map.find_by_type(id_type_index);
            type_map
                .base_name_to_num_suffix
                .entry(name_base.clone())
                .or_insert_with(UniqueNameValue::default);
            if !type_map.full_names.contains_key(r_name_full) {
                name_map.add_name(id_type_index, r_name_full, &name_base, number);
                if let Some(other) = name_map_other.as_deref_mut() {
                    other.add_name(id_type_index, r_name_full, &name_base, number);
                }
                return is_name_changed;
            }

            is_name_changed = true;

            let number_to_use = type_map
                .base_name_to_num_suffix
                .get(&name_base)
                .map(|v| v.get_smallest_unused())
                .unwrap_or(1);

            if !id_name_final_build(type_map, &name_base, number_to_use, r_name_full) {
                continue;
            }
        }

        name_map.add_name(id_type_index, r_name_full, &name_base, number_to_use_hack(r_name_full));
        if let Some(other) = name_map_other.as_deref_mut() {
            other.add_name(id_type_index, r_name_full, &name_base, number_to_use_hack(r_name_full));
        }
        return is_name_changed;
    }
}

/// Recover the numeric suffix of a final name (`number_to_use` —
/// re-split since `id_name_final_build` can rewrite the base).
fn number_to_use_hack(name_full: &str) -> i32 {
    string_split_name_number(name_full, '.').1
}

/// `LibraryNameMaps` — `bmain.namespaces` equivalent: one
/// `UniqueNameMap` per library (None = local) plus the global map.
pub struct MainNameMaps {
    /// `get_namemap_for(bmain, lib)` per library handle.
    pub per_library: HashMap<usize, UniqueNameMap>,
    /// `get_global_namemap_for(bmain)`.
    pub global: UniqueNameMap,
}

impl MainNameMaps {
    /// Create with a global map (`is_global=true`).
    pub fn new() -> Self {
        Self {
            per_library: HashMap::new(),
            global: UniqueNameMap::new(true),
        }
    }

    /// `get_namemap_for` — the map for `lib` (usize handle, 0 =
    /// local), lazily created.
    pub fn get_namemap_for(&mut self, lib: usize) -> &mut UniqueNameMap {
        self.per_library
            .entry(lib)
            .or_insert_with(|| UniqueNameMap::new(false))
    }

    /// `BKE_main_namemap_contain_name` — local map lookup.
    pub fn contain_name(&mut self, lib: usize, id_type_index: usize, name: &str) -> bool {
        self.get_namemap_for(lib)
            .find_by_type_ref(id_type_index)
            .map(|tm| tm.full_names.contains_key(name))
            .unwrap_or(false)
    }

    /// `BKE_main_global_namemap_contain_name`.
    pub fn global_contain_name(&mut self, id_type_index: usize, name: &str) -> bool {
        self.global
            .find_by_type(id_type_index)
            .full_names
            .contains_key(name)
    }

    /// `UniqueName_Map::populate` — rebuild from a full ID list.
    /// `ids` yields `(id_type_index, lib, name)` per ID.
    pub fn populate(
        &mut self,
        lib: usize,
        ignore_name: Option<&str>,
        ids: &mut dyn Iterator<Item = (usize, usize, String)>,
    ) {
        let map = self.get_namemap_for(lib);
        for tm in map.type_maps.iter_mut() {
            tm.full_names.clear();
            tm.base_name_to_num_suffix.clear();
        }
        for (type_index, id_lib, name) in ids {
            if !map.is_global && id_lib != lib {
                continue;
            }
            if ignore_name == Some(name.as_str()) {
                continue;
            }
            let (name_base, number) = string_split_name_number(&name, '.');
            /* Skip the `id` prefix chars (name includes GS prefix
             * in the C — here callers pass the name part). */
            map.type_maps[type_index]
                .base_name_to_num_suffix
                .entry(name_base)
                .or_insert_with(UniqueNameValue::default)
                .mark_used(number);
        }
    }

    /// `BKE_main_namemap_get_unique_name` — per-library unique name
    /// (plus the global mirror map).
    pub fn get_unique_name(&mut self, lib: usize, id_type_index: usize, name: &str) -> String {
        let mut r_name_full = name.to_string();
        let Self { per_library, global } = self;
        let lib_map = per_library
            .entry(lib)
            .or_insert_with(|| UniqueNameMap::new(false));
        namemap_get_name(lib_map, Some(global), id_type_index, &mut r_name_full);
        r_name_full
    }

    /// `BKE_main_global_namemap_get_unique_name`.
    pub fn get_unique_name_global(&mut self, lib: usize, id_type_index: usize, name: &str) -> String {
        let mut r_name_full = name.to_string();
        let Self { per_library, global } = self;
        let lib_map = per_library
            .entry(lib)
            .or_insert_with(|| UniqueNameMap::new(false));
        namemap_get_name(global, Some(lib_map), id_type_index, &mut r_name_full);
        r_name_full
    }

    /// `BKE_main_namemap_remove_id` — remove a name from both maps.
    pub fn remove_id(&mut self, lib: usize, id_type_index: usize, name: &str) {
        self.get_namemap_for(lib).remove_full_name(id_type_index, name);
        self.global.remove_full_name(id_type_index, name);
    }
}
