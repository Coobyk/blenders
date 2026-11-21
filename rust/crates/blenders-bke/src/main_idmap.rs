//! Port of `intern/main_idmap.cc` — `BKE_main_idmap_*` (Main ID
//! lookup indexes: by (type,name,lib) and by session_uid).
//!
//! GHash/mempool → `HashMap`; lazy type-map build preserved
//! (`lookup_name` populates the type map on first use). `FOREACH_MAIN_ID`
//! and `which_libbase` are injected (`all_ids`/`list_ids_of_type`).

use std::collections::{HashMap, HashSet};

use blenders_dna::id::NIL;

/// `MAIN_IDMAP_TYPE_NAME` / `MAIN_IDMAP_TYPE_UID`.
pub const MAIN_IDMAP_TYPE_NAME: i32 = 1 << 0;
pub const MAIN_IDMAP_TYPE_UID: i32 = 1 << 1;

/// `IDNameLib_Key`.
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct IdNameLibKey {
    /// `id->name + 2` (name sans ID code prefix).
    pub name: String,
    /// `id->lib` handle (`NIL` = local).
    pub lib: usize,
}

/// `IDNameLib_Map`.
pub struct IdNameLibMap {
    /// `bmain` — opaque Main handle.
    pub bmain: usize,
    /// `idmap_types` bitmask.
    pub idmap_types: i32,
    /// `type_maps` — lazily built per idcode: id_type → map.
    /// `None` = not yet built (first `lookup_name` builds it).
    type_maps: HashMap<i16, Option<HashMap<IdNameLibKey, usize>>>,
    /// `uid_map` (Some when TYPE_UID).
    uid_map: Option<HashMap<u32, usize>>,
    /// `valid_id_pointers` (Some when `create_valid_ids_set`).
    valid_id_pointers: Option<HashSet<usize>>,
}

impl IdNameLibMap {
    /// `BKE_main_idmap_create` — `idcodes` = the INDEX_ID_MAX idcode
    /// sequence (`BKE_idtype_idcode_iter_step` output); `all_ids()`
     /// yields `(id_handle, session_uid)` for the UID map;
    /// `valid_ids` pre-collected when `create_valid_ids_set`
    /// (covers `BKE_main_set_create(bmain, old_bmain)`).
    pub fn create(
        bmain: usize,
        create_valid_ids_set: bool,
        idmap_types: i32,
        idcodes: &[i16],
        all_ids: &mut dyn FnMut(&mut dyn FnMut(usize, u32)),
        valid_ids: Option<HashSet<usize>>,
    ) -> Self {
        let mut type_maps = HashMap::new();
        for &t in idcodes {
            type_maps.insert(t, None);
        }
        let uid_map = if idmap_types & MAIN_IDMAP_TYPE_UID != 0 {
            let mut m = HashMap::new();
            all_ids(&mut |id, uid| {
                debug_assert!(uid != u32::MAX); /* MAIN_ID_SESSION_UID_UNSET */
                m.insert(uid, id);
            });
            Some(m)
        } else {
            None
        };
        Self {
            bmain,
            idmap_types,
            type_maps,
            uid_map,
            valid_id_pointers: if create_valid_ids_set { valid_ids } else { None },
        }
    }

    /// `main_idmap_from_idcode`.
    fn type_map_present(&self, id_type: i16) -> bool {
        self.idmap_types & MAIN_IDMAP_TYPE_NAME != 0 && self.type_maps.contains_key(&id_type)
    }

    /// `BKE_main_idmap_insert_id` — `name`/`lib`/`uid`/`id_type` of
    /// the inserted ID (name already stripped of the 2-char prefix).
    pub fn insert_id(&mut self, id: usize, id_type: i16, name: &str, lib: usize, session_uid: u32) {
        if self.idmap_types & MAIN_IDMAP_TYPE_NAME != 0 {
            if let Some(Some(map)) = self.type_maps.get_mut(&id_type) {
                map.insert(IdNameLibKey { name: name.to_string(), lib }, id);
            }
        }
        if let Some(uid_map) = &mut self.uid_map {
            debug_assert!(session_uid != u32::MAX);
            uid_map.insert(session_uid, id);
        }
    }

    /// `BKE_main_idmap_remove_id`.
    pub fn remove_id(&mut self, id_type: i16, name: &str, lib: usize, session_uid: u32) {
        if self.idmap_types & MAIN_IDMAP_TYPE_NAME != 0 {
            if let Some(Some(map)) = self.type_maps.get_mut(&id_type) {
                map.remove(&IdNameLibKey { name: name.to_string(), lib });
            }
        }
        if let Some(uid_map) = &mut self.uid_map {
            debug_assert!(session_uid != u32::MAX);
            uid_map.remove(&session_uid);
        }
    }

    /// `BKE_main_idmap_main_get`.
    pub fn main_get(&self) -> usize { self.bmain }

    /// `BKE_main_idmap_lookup_name` — lazy-builds the type map from
    /// `list_ids_of_type(id_type, cb(id_handle, name, lib))` on first
    /// lookup (`which_libbase` + `ListBase` walk).
    pub fn lookup_name(
        &mut self,
        id_type: i16,
        name: &str,
        lib: usize,
        list_ids_of_type: &mut dyn FnMut(i16, &mut dyn FnMut(usize, &str, usize)),
    ) -> usize {
        if !self.type_map_present(id_type) {
            return NIL;
        }
        let slot = self.type_maps.get_mut(&id_type).unwrap();
        if slot.is_none() {
            let mut map = HashMap::new();
            list_ids_of_type(id_type, &mut |id, id_name, id_lib| {
                map.insert(IdNameLibKey { name: id_name.to_string(), lib: id_lib }, id);
            });
            *slot = Some(map);
        }
        self.type_maps[&id_type]
            .as_ref()
            .unwrap()
            .get(&IdNameLibKey { name: name.to_string(), lib })
            .copied()
            .unwrap_or(NIL)
    }

    /// `BKE_main_idmap_lookup_id` — gated by `valid_id_pointers`.
    pub fn lookup_id(
        &mut self,
        id: usize,
        id_type: i16,
        name: &str,
        lib: usize,
        list_ids_of_type: &mut dyn FnMut(i16, &mut dyn FnMut(usize, &str, usize)),
    ) -> usize {
        match &self.valid_id_pointers {
            Some(set) if !set.contains(&id) => NIL,
            _ => self.lookup_name(id_type, name, lib, list_ids_of_type),
        }
    }

    /// `BKE_main_idmap_lookup_uid`.
    pub fn lookup_uid(&self, session_uid: u32) -> usize {
        match &self.uid_map {
            Some(m) => m.get(&session_uid).copied().unwrap_or(NIL),
            None => NIL,
        }
    }

    /// `BKE_main_idmap_clear` — clear contents, keep allocated maps
    /// (upstream `BLI_ghash_clear` keeps the table).
    pub fn clear(&mut self) {
        for slot in self.type_maps.values_mut() {
            if let Some(m) = slot { m.clear(); }
        }
        if let Some(m) = &mut self.uid_map { m.clear(); }
        if let Some(s) = &mut self.valid_id_pointers { s.clear(); }
    }

    /// `BKE_main_idmap_destroy` — drop all maps (upstream frees the
    /// ghash + mempool + sets). Rust drop handles it; explicit for
    /// parity.
    pub fn destroy(self) {}
}
