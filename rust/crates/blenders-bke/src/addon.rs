//! Port of `intern/addon.cc` — `bAddon` list ops (`BKE_addon_*`).
//! `bAddon` arenas are `Vec`s; `NIL` = not found. `bAddonPrefType` global
//! hash stays a `HashMap` of `usize` payloads (PORT_PENDING contents).

use blenders_dna::id::NIL;
use blenders_dna::userdef_types::bAddon;
use std::collections::HashMap;

fn cstr(b: &[u8]) -> &str {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    core::str::from_utf8(&b[..end]).unwrap_or("")
}

/// `BKE_addon_new` — default bAddon.
pub fn bke_addon_new() -> bAddon {
    bAddon::default()
}

/// `BKE_addon_find` — `BLI_findstring` on `module`.
pub fn bke_addon_find(addon_list: &[bAddon], module: &str) -> usize {
    addon_list
        .iter()
        .position(|a| cstr(&a.module) == module)
        .unwrap_or(NIL)
}

/// `BKE_addon_ensure` — find or `BLI_addtail` a new one.
pub fn bke_addon_ensure(addon_list: &mut Vec<bAddon>, module: &str) -> usize {
    let i = bke_addon_find(addon_list, module);
    if i != NIL {
        return i;
    }
    let mut a = bke_addon_new();
    let mb = module.as_bytes();
    let n = mb.len().min(a.module.len() - 1);
    a.module[..n].copy_from_slice(&mb[..n]);
    addon_list.push(a);
    addon_list.len() - 1
}

/// `BKE_addon_remove_safe` — `BLI_remlink` + `BKE_addon_free`.
pub fn bke_addon_remove_safe(addon_list: &mut Vec<bAddon>, module: &str) -> bool {
    let i = bke_addon_find(addon_list, module);
    if i == NIL {
        return false;
    }
    addon_list.remove(i);
    // `BKE_addon_free` → IDP_FreeProperty(addon->prop) — PORT_PENDING (IDProperty).
    true
}

/// `BKE_addon_free` — `IDP_FreeProperty(prop)` is PORT_PENDING; the struct
/// itself is dropped by the arena.
pub fn bke_addon_free(_addon: bAddon) {}

/// `global_addonpreftype_hash` — `bAddonPrefType` registry (payload handles).
#[derive(Default)]
pub struct AddonPrefTypeRegistry {
    map: HashMap<String, usize>,
}

impl AddonPrefTypeRegistry {
    /// `BKE_addon_pref_type_add` — insert by name; returns stored handle.
    pub fn add(&mut self, name: &str, pref_type: usize) {
        self.map.insert(name.to_string(), pref_type);
    }
    /// `BKE_addon_pref_type_find` — `BLI_ghash_lookup`.
    pub fn find(&self, name: &str) -> usize {
        self.map.get(name).copied().unwrap_or(NIL)
    }
    /// `BKE_addon_pref_type_get_or_add`.
    pub fn get_or_add(&mut self, name: &str, make: impl FnOnce() -> usize) -> usize {
        *self.map.entry(name.to_string()).or_insert_with(make)
    }
    /// `BKE_addon_pref_type_remove` — drop entry (free is PORT_PENDING).
    pub fn remove(&mut self, name: &str) {
        self.map.remove(name);
    }
    /// `BKE_addon_pref_type_init/final` — the C clears the whole hash.
    pub fn clear(&mut self) {
        self.map.clear();
    }
}
