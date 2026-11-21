//! Port of `intern/keyconfig.cc` — `BKE_keyconfig_pref_*` (keyconfig
//! preference lookup + the pref-type registry + keymap item filtering).

use std::collections::HashMap;
use std::sync::Mutex;

use blenders_dna::id::NIL;
use blenders_dna::windowmanager_types::{WmKeyConfigPref, WmKeyMap, WmKeyMapItem};
use crate::idprop::{IDProperty, IDPData, IDP_GROUP};

/// `WM_KEYCONFIG_STR_DEFAULT`.
pub const WM_KEYCONFIG_STR_DEFAULT: &str = "Blender";

/// `wmKeyConfigPrefType_Runtime` (BKE_keyconfig.h) — the `ExtensionRNA`
/// member stays an opaque handle.
pub struct WmKeyConfigPrefTypeRuntime {
    /// `char idname[64]` — string in Rust.
    pub idname: String,
    /// `ExtensionRNA rna_ext` — PORT_PENDING RNA field.
    pub rna_ext: usize,
}

/// `BKE_keyconfig_pref_ensure` — find or add a `wmKeyConfigPref` by idname,
/// ensuring its `prop` ID group exists.
///
/// `user_keyconfig_prefs` is a Vec (the UserDef ListBase in our model);
/// `idp_arena` is the shared IDProperty arena.
pub fn bke_keyconfig_pref_ensure(
    user_keyconfig_prefs: &mut Vec<WmKeyConfigPref>,
    kc_idname: &str,
    idp_arena: &mut Vec<IDProperty>,
) -> usize {
    let idx = match user_keyconfig_prefs.iter().position(|k| {
        k.idname.iter().take_while(|&&c| c != 0).map(|&c| c as char).collect::<String>() == kc_idname
    }) {
        Some(i) => i,
        None => {
            let mut kpt = WmKeyConfigPref::default();
            let bytes = kc_idname.as_bytes();
            let n = bytes.len().min(63);
            kpt.idname[..n].copy_from_slice(&bytes[..n]);
            user_keyconfig_prefs.push(kpt);
            user_keyconfig_prefs.len() - 1
        }
    };
    if user_keyconfig_prefs[idx].prop == NIL {
        let mut group = IDProperty::default();
        group.name = kc_idname.to_string();
        group.type_ = IDP_GROUP;
        group.subtype = IDP_GROUP;
        group.data = IDPData::Group(Vec::new());
        idp_arena.push(group);
        user_keyconfig_prefs[idx].prop = idp_arena.len() - 1;
    }
    idx
}

/// `global_keyconfigpreftype_hash` — `None` until
/// `bke_keyconfig_pref_type_init`, matching the C assert.
static GLOBAL_KEYCONFIGPREFTYPE_HASH: Mutex<Option<HashMap<String, usize>>> = Mutex::new(None);

/// Pref-type arena — `usize` values in the hash index this.
static PREF_TYPES: Mutex<Vec<WmKeyConfigPrefTypeRuntime>> = Mutex::new(Vec::new());

/// `BKE_keyconfig_pref_type_find` — `None` on miss (the C prints unless
/// `quiet`; keep the prints verbatim to stderr).
pub fn bke_keyconfig_pref_type_find(idname: &str, quiet: bool) -> Option<usize> {
    let hash = GLOBAL_KEYCONFIGPREFTYPE_HASH.lock().unwrap();
    let hash = hash.as_ref()?;
    if !idname.is_empty() {
        if let Some(&idx) = hash.get(idname) {
            return Some(idx);
        }
        if !quiet {
            eprintln!("search for unknown keyconfig-pref '{}'", idname);
        }
    } else if !quiet {
        eprintln!("search for empty keyconfig-pref");
    }
    None
}

/// `BKE_keyconfig_pref_type_add`.
pub fn bke_keyconfig_pref_type_add(kpt_rt: WmKeyConfigPrefTypeRuntime) {
    let mut types = PREF_TYPES.lock().unwrap();
    types.push(kpt_rt);
    let idx = types.len() - 1;
    let idname = types[idx].idname.clone();
    if let Some(hash) = GLOBAL_KEYCONFIGPREFTYPE_HASH.lock().unwrap().as_mut() {
        hash.insert(idname, idx);
    }
}

/// `BKE_keyconfig_pref_type_remove` — removes hash entry and the arena slot
/// (the C frees the runtime struct too).
pub fn bke_keyconfig_pref_type_remove(idx: usize) {
    let mut types = PREF_TYPES.lock().unwrap();
    if idx >= types.len() {
        return;
    }
    let idname = types[idx].idname.clone();
    types.remove(idx);
    drop(types);
    if let Some(hash) = GLOBAL_KEYCONFIGPREFTYPE_HASH.lock().unwrap().as_mut() {
        hash.remove(&idname);
        /* Re-point indices above the removed slot. */
        for v in hash.values_mut() {
            if *v > idx {
                *v -= 1;
            }
        }
    }
}

/// `BKE_keyconfig_pref_type_init`.
pub fn bke_keyconfig_pref_type_init() {
    let mut hash = GLOBAL_KEYCONFIGPREFTYPE_HASH.lock().unwrap();
    debug_assert!(hash.is_none());
    *hash = Some(HashMap::new());
}

/// `BKE_keyconfig_pref_type_free`.
pub fn bke_keyconfig_pref_type_free() {
    *GLOBAL_KEYCONFIGPREFTYPE_HASH.lock().unwrap() = None;
    PREF_TYPES.lock().unwrap().clear();
}

/// `BKE_keyconfig_pref_set_select_mouse` — set `select_mouse` on the default
/// keyconfig pref (create when missing, overwrite only with `override`).
pub fn bke_keyconfig_pref_set_select_mouse(
    user_keyconfig_prefs: &mut Vec<WmKeyConfigPref>,
    value: i32,
    override_: bool,
    idp_arena: &mut Vec<IDProperty>,
) {
    let kpt = bke_keyconfig_pref_ensure(user_keyconfig_prefs, WM_KEYCONFIG_STR_DEFAULT, idp_arena);
    let prop = user_keyconfig_prefs[kpt].prop;
    let group_children = match &idp_arena[prop].data {
        IDPData::Group(c) => c.clone(),
        _ => Vec::new(),
    };
    let existing = group_children.iter().copied().find(|&ci| idp_arena[ci].name == "select_mouse");
    match existing {
        None => {
            /* `IDP_AddToGroup(kpt->prop, create("select_mouse", value))`. */
            let mut p = IDProperty::default();
            p.name = "select_mouse".to_string();
            p.type_ = crate::idprop::IDP_INT;
            p.data = IDPData::Val(value as i64);
            idp_arena.push(p);
            let pi = idp_arena.len() - 1;
            if let IDPData::Group(children) = &mut idp_arena[prop].data {
                children.push(pi);
            }
        }
        Some(i) if override_ => {
            idp_arena[i].data = IDPData::Val(value as i64); /* `IDP_int_set` */
        }
        _ => {}
    }
}

/// `wmKeyConfigFilterItemParams` (BKE_keyconfig.h).
#[derive(Clone, Copy, Default)]
pub struct WmKeyConfigFilterItemParams {
    pub check_item: bool,
    pub check_diff_item_add: bool,
    pub check_diff_item_remove: bool,
}

/// `WM_KEY_CONFIG_FILTER_ITEM_ALL`.
pub const WM_KEY_CONFIG_FILTER_ITEM_ALL: WmKeyConfigFilterItemParams =
    WmKeyConfigFilterItemParams { check_item: true, check_diff_item_add: true, check_diff_item_remove: true };

/// `BKE_keyconfig_keymap_filter_item` — remove items matching `filter_fn`
/// from `diff_items`/`items`. The lists are Vecs of handles into the
/// caller's arenas; `diff_items` yields `(remove_item, add_item)` handle
/// pairs.
pub fn bke_keyconfig_keymap_filter_item<'a>(
    diff_items: &mut Vec<(usize, usize)>,
    items: &mut Vec<WmKeyMapItem>,
    params: &WmKeyConfigFilterItemParams,
    get_item: &mut dyn FnMut(usize) -> &'a mut WmKeyMapItem,
    filter_fn: &mut dyn FnMut(&mut WmKeyMapItem) -> bool,
) {
    if params.check_diff_item_add || params.check_diff_item_remove {
        let mut i = 0;
        while i < diff_items.len() {
            let (rm, add) = diff_items[i];
            let mut remove = false;
            if params.check_diff_item_add && add != NIL && filter_fn(get_item(add)) {
                remove = true;
            }
            if !remove && params.check_diff_item_remove && rm != NIL && filter_fn(get_item(rm)) {
                remove = true;
            }
            if remove {
                diff_items.remove(i);
            } else {
                i += 1;
            }
        }
    }
    if params.check_item {
        let mut i = 0;
        while i < items.len() {
            if filter_fn(&mut items[i]) {
                items.remove(i);
            } else {
                i += 1;
            }
        }
    }
}

/// `BKE_keyconfig_pref_filter_items` — across `userdef->user_keymaps`.
/// `keymaps` yields each keymap's `(diff_items, items)`; resolved the same
/// way as `bke_keyconfig_keymap_filter_item`.
pub fn bke_keyconfig_pref_filter_items<'a>(
    user_keymaps: &mut [WmKeyMap],
    diff_items_of: &mut dyn FnMut(&mut WmKeyMap) -> &mut Vec<(usize, usize)>,
    items_of: &mut dyn FnMut(&mut WmKeyMap) -> &mut Vec<WmKeyMapItem>,
    params: &WmKeyConfigFilterItemParams,
    get_item: &mut dyn FnMut(usize) -> &'a mut WmKeyMapItem,
    filter_fn: &mut dyn FnMut(&mut WmKeyMapItem) -> bool,
) {
    for keymap in user_keymaps.iter_mut() {
        bke_keyconfig_keymap_filter_item(
            diff_items_of(keymap), items_of(keymap), params, get_item, filter_fn,
        );
    }
}
