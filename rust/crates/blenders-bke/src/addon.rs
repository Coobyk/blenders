//! Port of `intern/addon.cc` — `bAddon` list ops +
//! `global_addonpreftype_hash` registry.
//!
//! ListBase → `Vec` arena + `usize` handles; `GHash` → `HashMap`
//! keyed by `idname`, values index an arena.

use std::collections::HashMap;
use std::sync::Mutex;

use blenders_dna::id::NIL;
use blenders_dna::userdef_types::bAddon;

/// `bAddonPrefType` (BKE_addon.h — not DNA).
#[derive(Clone)]
pub struct bAddonPrefType {
    /// `char idname[128]` (MAX_NAME).
    pub idname: [u8; 128],
    /// `ExtensionRNA rna_ext` — RNA integration (PORT_PENDING).
    pub rna_ext: usize,
}

impl Default for bAddonPrefType {
    fn default() -> Self {
        Self { idname: [0; 128], rna_ext: 0 }
    }
}

fn cstr(b: &[u8]) -> &str {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    core::str::from_utf8(&b[..end]).unwrap_or("")
}

/// `BKE_addon_new`.
pub fn bke_addon_new() -> bAddon {
    bAddon::default()
}

/// `BKE_addon_find` (`BLI_findstring` on `module`).
pub fn bke_addon_find(addon_list: &[bAddon], module: &str) -> usize {
    for (i, addon) in addon_list.iter().enumerate() {
        if cstr(&addon.module) == module {
            return i;
        }
    }
    NIL
}

/// `BKE_addon_ensure` — find or `BLI_addtail` a new entry.
pub fn bke_addon_ensure(addon_list: &mut Vec<bAddon>, module: &str) -> usize {
    let found = bke_addon_find(addon_list, module);
    if found != NIL {
        return found;
    }
    let mut addon = bke_addon_new();
    let mb = module.as_bytes();
    let n = mb.len().min(addon.module.len() - 1);
    addon.module[..n].copy_from_slice(&mb[..n]);
    addon_list.push(addon);
    addon_list.len() - 1
}

/// `BKE_addon_remove_safe` (`BLI_remlink` + `BKE_addon_free`).
/// `prop` is an IDProperty arena index; freeing it is the caller's
/// arena's job (dropped entry returned? — C frees in place; the
/// `prop` field rides along with the removed `bAddon`).
pub fn bke_addon_remove_safe(addon_list: &mut Vec<bAddon>, module: &str) -> bool {
    let found = bke_addon_find(addon_list, module);
    if found != NIL {
        /* `IDP_FreeProperty(addon->prop)` — prop dies with the addon. */
        addon_list.remove(found);
        return true;
    }
    false
}

/* --- global_addonpreftype_hash --- */

static ADDON_PREFTYPE_HASH: Mutex<Option<HashMap<String, usize>>> = Mutex::new(None);
static ADDON_PREFTYPES: Mutex<Vec<bAddonPrefType>> = Mutex::new(Vec::new());

/// `BKE_addon_pref_type_find` — `quiet` gates the CLOG_WARN.
pub fn bke_addon_pref_type_find(idname: &str, _quiet: bool) -> usize {
    if !idname.is_empty() {
        if let Some(map) = ADDON_PREFTYPE_HASH.lock().unwrap().as_ref() {
            if let Some(&i) = map.get(idname) {
                return i;
            }
        }
        /* `CLOG_WARN("search for unknown addon-pref '%s'")` */
    } else {
        /* `CLOG_WARN("search for empty addon-pref")` */
    }
    NIL
}

/// `BKE_addon_pref_type_get` — the find result's data.
pub fn bke_addon_pref_type_get(i: usize) -> Option<bAddonPrefType> {
    if i == NIL {
        return None;
    }
    Some(ADDON_PREFTYPES.lock().unwrap()[i].clone())
}

/// `BKE_addon_pref_type_add` (`BLI_ghash_insert` by `idname`).
pub fn bke_addon_pref_type_add(apt: bAddonPrefType) {
    let key = cstr(&apt.idname).to_string();
    let mut arena = ADDON_PREFTYPES.lock().unwrap();
    arena.push(apt);
    let idx = arena.len() - 1;
    drop(arena);
    if let Some(map) = ADDON_PREFTYPE_HASH.lock().unwrap().as_mut() {
        map.insert(key, idx);
    }
}

/// `BKE_addon_pref_type_remove` (`BLI_ghash_remove` + free).
pub fn bke_addon_pref_type_remove(apt: usize) {
    if apt == NIL {
        return;
    }
    let key = cstr(&ADDON_PREFTYPES.lock().unwrap()[apt].idname).to_string();
    if let Some(map) = ADDON_PREFTYPE_HASH.lock().unwrap().as_mut() {
        map.remove(&key);
    }
}

/// `BKE_addon_pref_type_init`.
pub fn bke_addon_pref_type_init() {
    let mut hash = ADDON_PREFTYPE_HASH.lock().unwrap();
    debug_assert!(hash.is_none());
    *hash = Some(HashMap::new());
}

/// `BKE_addon_pref_type_free` (`BLI_ghash_free` — arena cleared).
pub fn bke_addon_pref_type_free() {
    *ADDON_PREFTYPE_HASH.lock().unwrap() = None;
    ADDON_PREFTYPES.lock().unwrap().clear();
}
