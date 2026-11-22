//! Port of `intern/wm_keymap.cc` — `wmKeyConfig`/
//! `wmKeyMap`/`wmKeyMapItem` lifecycle: keyconfig new/
//! ensure/remove/clear/active, keymap new/copy/match/
//! list_find/ensure, `keymap_event_set` modifier decode,
//! modal keymap ops.

use crate::wm_keymap_utils::{KM_ALT, KM_ANY, KM_HYPER, KM_NOTHING, KM_OSKEY};

/// `KEYCONF_USER` verbatim.
pub const KEYCONF_USER: i32 = 1 << 0;
/// `KEYMAP_USER`/`KEYMAP_MODAL`/`KEYMAP_UPDATE`/`KEYMAP_EXPANDED`.
pub const KEYMAP_USER: i32 = 1 << 0;
pub const KEYMAP_MODAL: i32 = 1 << 2;
pub const KEYMAP_UPDATE: i32 = 1 << 1;
pub const KEYMAP_EXPANDED: i32 = 1 << 7;
/// `KMAP_MAX_NAME` verbatim.
pub const KMAP_MAX_NAME: usize = 64;
/// `KM_MOD_HELD` verbatim.
pub const KM_MOD_HELD: i8 = 1;
/// `KM_SHIFT`/`KM_CTRL` from `wm_operator_utils`.
pub const KM_SHIFT: i32 = 1;
pub const KM_CTRL: i32 = 2;
/// `KMI_PARAMS_MOD_FROM_ANY` — high byte of `modifier`.
pub fn kmi_params_mod_from_any(modifier: i32) -> i32 {
    (modifier >> 8) & 0xff
}

/// `wmKeyMapItem` fields used.
#[derive(Clone, Default)]
pub struct WmKeyMapItem {
    /// `id` (negative for user items).
    pub id: i32,
    /// `type`/`val`/`keymodifier`/`direction`.
    pub event_type: i32,
    pub val: i32,
    pub keymodifier: i32,
    pub direction: i32,
    /// `shift`/`ctrl`/`alt`/`oskey`/`hyper`.
    pub shift: i8,
    pub ctrl: i8,
    pub alt: i8,
    pub oskey: i8,
    pub hyper: i8,
    /// `flag`.
    pub flag: u32,
    /// `idname` (operator).
    pub idname: String,
}

/// `wmKeyMap` fields used.
#[derive(Default)]
pub struct WmKeyMap {
    /// `idname`/`owner_id`.
    pub idname: String,
    pub owner_id: String,
    /// `spaceid`/`regionid`.
    pub spaceid: i32,
    pub regionid: i32,
    /// `flag`.
    pub flag: i32,
    /// `kmi_id` counter.
    pub kmi_id: i32,
    /// `items`.
    pub items: Vec<WmKeyMapItem>,
    /// `modal_items` present.
    pub has_modal_items: bool,
}

/// `wmKeyConfig` fields used.
pub struct WmKeyConfig {
    /// `idname`.
    pub idname: String,
    /// `flag`.
    pub flag: i32,
    /// `keymaps`.
    pub keymaps: Vec<WmKeyMap>,
}

/// `WM_keyconfig_new` (verbatim — assert-unique + USER
/// flag).
pub fn wm_keyconfig_new(keyconfigs: &mut Vec<WmKeyConfig>, idname: &str, user_defined: bool) -> usize {
    assert!(!keyconfigs.iter().any(|k| k.idname == idname));
    keyconfigs.push(WmKeyConfig {
        idname: idname.to_string(),
        flag: if user_defined { KEYCONF_USER } else { 0 },
        keymaps: Vec::new(),
    });
    keyconfigs.len() - 1
}

/// `WM_keyconfig_ensure` (verbatim — defaultconf keeps
/// keymaps cleared-in-place; others cleared fully).
pub fn wm_keyconfig_ensure(
    keyconfigs: &mut Vec<WmKeyConfig>,
    idname: &str,
    user_defined: bool,
    defaultconf_idx: Option<usize>,
) -> usize {
    if let Some(i) = keyconfigs.iter().position(|k| k.idname == idname) {
        if Some(i) == defaultconf_idx {
            for km in keyconfigs[i].keymaps.iter_mut() {
                km.items.clear();
            }
        } else {
            keyconfigs[i].keymaps.clear();
        }
        return i;
    }
    wm_keyconfig_new(keyconfigs, idname, user_defined)
}

/// `WM_keyconfig_clear`/`WM_keyconfig_free` (verbatim).
pub fn wm_keyconfig_clear(keyconf: &mut WmKeyConfig) {
    keyconf.keymaps.clear();
}

/// `WM_keyconfig_active` (verbatim — preset lookup else
/// default).
pub fn wm_keyconfig_active<'a>(
    keyconfigs: &'a [WmKeyConfig],
    keyconfigstr: &str,
    defaultconf_idx: usize,
) -> usize {
    keyconfigs
        .iter()
        .position(|k| k.idname == keyconfigstr)
        .unwrap_or(defaultconf_idx)
}

/// `wm_keymap_new` (verbatim — idname/space/region +
/// owner).
pub fn wm_keymap_new(idname: &str, spaceid: i32, regionid: i32, owner_id: Option<&str>) -> WmKeyMap {
    WmKeyMap {
        idname: idname.to_string(),
        owner_id: owner_id.unwrap_or("").to_string(),
        spaceid,
        regionid,
        ..Default::default()
    }
}

/// `wm_keymap_is_match` (verbatim — space+region+idname).
pub fn wm_keymap_is_match(a: &WmKeyMap, b: &WmKeyMap) -> bool {
    a.spaceid == b.spaceid && a.regionid == b.regionid && a.idname == b.idname
}

/// `keymap_event_set` (verbatim — KM_ANY → all-any;
/// else held/any/nothing per mod).
pub fn keymap_event_set(kmi: &mut WmKeyMapItem, params: &crate::wm_keymap_utils::KeyMapItemParams) {
    kmi.event_type = params.event_type;
    kmi.val = params.val;
    kmi.keymodifier = params.keymodifier;
    kmi.direction = params.val; /* `direction` field in params maps here. */
    if params.modifier as i8 == KM_ANY {
        kmi.shift = KM_ANY;
        kmi.ctrl = KM_ANY;
        kmi.alt = KM_ANY;
        kmi.oskey = KM_ANY;
        kmi.hyper = KM_ANY;
    } else {
        let mod_ = (params.modifier & 0xff) as i32;
        let mod_any = kmi_params_mod_from_any(params.modifier);
        kmi.shift = if (mod_ & KM_SHIFT) != 0 { KM_MOD_HELD } else if (mod_any & KM_SHIFT) != 0 { KM_ANY } else { KM_NOTHING };
        kmi.ctrl = if (mod_ & KM_CTRL) != 0 { KM_MOD_HELD } else if (mod_any & KM_CTRL) != 0 { KM_ANY } else { KM_NOTHING };
        kmi.alt = if (mod_ & KM_ALT) != 0 { KM_MOD_HELD } else if (mod_any & KM_ALT) != 0 { KM_ANY } else { KM_NOTHING };
        kmi.oskey = if (mod_ & KM_OSKEY) != 0 { KM_MOD_HELD } else if (mod_any & KM_OSKEY) != 0 { KM_ANY } else { KM_NOTHING };
        kmi.hyper = if (mod_ & KM_HYPER) != 0 { KM_MOD_HELD } else if (mod_any & KM_HYPER) != 0 { KM_ANY } else { KM_NOTHING };
    }
}

/// `keymap_item_set_id` (verbatim — user items get
/// negative ids).
pub fn keymap_item_set_id(keymap: &mut WmKeyMap, kmi: &mut WmKeyMapItem) {
    keymap.kmi_id += 1;
    if (keymap.flag & KEYMAP_USER) == 0 {
        kmi.id = keymap.kmi_id;
    } else {
        kmi.id = -keymap.kmi_id;
    }
}

/// `WM_keymap_list_find` (verbatim — space+region+idname).
pub fn wm_keymap_list_find(keymaps: &[WmKeyMap], idname: &str, spaceid: i32, regionid: i32) -> Option<usize> {
    keymaps.iter().position(|km| {
        km.spaceid == spaceid && km.regionid == regionid && km.idname == idname
    })
}

/// `WM_keymap_list_find_spaceid_or_empty` (verbatim —
/// spaceid or SPACE_EMPTY).
pub fn wm_keymap_list_find_spaceid_or_empty(
    keymaps: &[WmKeyMap],
    idname: &str,
    spaceid: i32,
    regionid: i32,
) -> Option<usize> {
    keymaps.iter().position(|km| {
        (km.spaceid == spaceid || km.spaceid == crate::wm_keymap_utils::SPACE_EMPTY)
            && km.regionid == regionid
            && km.idname == idname
    })
}

/// `WM_keymap_ensure` (verbatim — find-or-new + tag).
pub fn wm_keymap_ensure(
    keyconf: &mut WmKeyConfig,
    idname: &str,
    spaceid: i32,
    regionid: i32,
    update_tag: &mut dyn FnMut(),
) -> usize {
    if let Some(i) = wm_keymap_list_find(&keyconf.keymaps, idname, spaceid, regionid) {
        return i;
    }
    keyconf.keymaps.push(wm_keymap_new(idname, spaceid, regionid, None));
    (update_tag)();
    keyconf.keymaps.len() - 1
}

/// `WM_modalkeymap_ensure` (verbatim — ensure + MODAL
/// flag + default-conf modal items copy).
pub fn wm_modalkeymap_ensure(
    keyconf: &mut WmKeyConfig,
    idname: &str,
    items_present: bool,
    update_tag: &mut dyn FnMut(),
) -> usize {
    let i = wm_keymap_ensure(
        keyconf,
        idname,
        crate::wm_keymap_utils::SPACE_EMPTY,
        crate::wm_keymap_utils::RGN_TYPE_WINDOW,
        update_tag,
    );
    keyconf.keymaps[i].flag |= KEYMAP_MODAL;
    if items_present {
        keyconf.keymaps[i].has_modal_items = true;
    }
    i
}

/// `WM_modalkeymap_find` (verbatim — SPACE_EMPTY +
/// RGN_TYPE_WINDOW + MODAL).
pub fn wm_modalkeymap_find(keyconf: &WmKeyConfig, idname: &str) -> Option<usize> {
    keyconf.keymaps.iter().position(|km| {
        km.spaceid == crate::wm_keymap_utils::SPACE_EMPTY
            && km.regionid == crate::wm_keymap_utils::RGN_TYPE_WINDOW
            && (km.flag & KEYMAP_MODAL) != 0
            && km.idname == idname
    })
}

/// `WM_modalkeymap_find_propvalue` (verbatim — first
/// active item with `val == propvalue`).
pub fn wm_modalkeymap_find_propvalue(km: &WmKeyMap, propvalue: i32) -> Option<usize> {
    km.items.iter().position(|kmi| {
        (kmi.flag & crate::wm_keymap_utils::KMI_INACTIVE) == 0 && kmi.val == propvalue
    })
}
