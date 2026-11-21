//! Port of `intern/icons.cc` — the global icon registry
//! (`Map<int, Icon*>` → `HashMap<i32, Icon>`), deferred delete queue,
//! and the `BKE_icon_*` ensure/get/set/delete API.
//!
//! Icon payloads (ID, ImBuf, PreviewImage, bGPDlayer, Icon_Geom,
//! StudioLight) are `usize` handles since those types live outside
//! this crate; freeing them is injected.

use std::collections::HashMap;
use std::sync::Mutex;

/// `ICON_DATA_*` (BKE_icon.hh).
pub const ICON_DATA_ID: i32 = 0;
pub const ICON_DATA_IMBUF: i32 = 1;
pub const ICON_DATA_PREVIEW: i32 = 2;
pub const ICON_DATA_GPLAYER: i32 = 3;
pub const ICON_DATA_GEOM: i32 = 4;
pub const ICON_DATA_STUDIOLIGHT: i32 = 5;

/// `ICON_FLAG_*`.
pub const ICON_FLAG_MANAGED: i32 = 1 << 0;

/// `Icon`.
pub struct Icon {
    /// `id` (the icon_id it is registered under).
    pub id: i32,
    /// `drawinfo` — opaque draw-info payload.
    pub drawinfo: usize,
    /// `obj` — payload handle (ID / ImBuf / PreviewImage / ...).
    pub obj: usize,
    /// `id_type`.
    pub id_type: i16,
    /// `obj_type` — `ICON_DATA_*`.
    pub obj_type: i16,
    /// `flag`.
    pub flag: i32,
}

/// `Icon_Geom` — vertex geometry icon data.
pub struct IconGeom {
    /// `mem` — packed buffer (when set, `coords`/`colors` are views
    /// into it; freeing `mem` frees all).
    pub mem: Option<Vec<u8>>,
    /// `coords` — only when `mem` is None.
    pub coords: Vec<[f32; 3]>,
    /// `colors`.
    pub colors: Vec<[u8; 4]>,
    /// `icon_id`.
    pub icon_id: i32,
}

/// Hooks for payload freeing / ID icon_id write-back.
pub trait IconHooks {
    /// `icon_free_data` — clear the payload's `icon_id` field
    /// (ID/PreviewImage/GPlayer/Icon_Geom/StudioLight cases) and
    /// free ImBuf for `ICON_DATA_IMBUF`.
    fn free_payload(&mut self, _obj_type: i16, _obj: usize, _icon_id: i32) {}
    /// `icon_free`'s deeper free (geom mem, drawinfo_free callback).
    fn free_geom(&mut self, _geom: usize) {}
    /// `G.background` gate.
    fn is_background(&self) -> bool {
        false
    }
    /// `BKE_previewimg_id_get_p` — returns the ID's preview handle
    /// (or None when the ID has no preview slot).
    fn preview_of_id(&mut self, _id: usize) -> Option<usize> {
        None
    }
    /// `BKE_studiolight_unset_icon_id` — called on StudioLight free.
    fn studiolight_unset_icon_id(&mut self, _sl: usize, _icon_id: i32) {}
}

struct IconRegistry {
    icons: HashMap<i32, Icon>,
    next_icon_id: i32,
    first_icon_id: i32,
    delete_queue: Vec<i32>,
}

static REGISTRY: Mutex<Option<IconRegistry>> = Mutex::new(None);

/// Lazily create the registry (`HashMap::new` isn't const).
fn reg_get<'a>(guard: &'a mut Option<IconRegistry>) -> &'a mut IconRegistry {
    guard.get_or_insert_with(|| IconRegistry {
        icons: HashMap::new(),
        next_icon_id: 1,
        first_icon_id: 1,
        delete_queue: Vec::new(),
    })
}

/// `get_next_free_id`.
fn get_next_free_id(reg: &mut IconRegistry) -> i32 {
    let mut start_id = reg.first_icon_id;
    if reg.next_icon_id >= reg.first_icon_id {
        let next_id = reg.next_icon_id;
        reg.next_icon_id += 1;
        return next_id;
    }
    while reg.icons.contains_key(&start_id) && start_id >= reg.first_icon_id {
        start_id += 1;
    }
    if start_id >= reg.first_icon_id {
        return start_id;
    }
    0
}

/// `icon_create` — register `icon` under `icon_id` (asserts unique).
fn icon_create(reg: &mut IconRegistry, icon_id: i32, obj_type: i16, obj: usize) -> i32 {
    let icon = Icon {
        id: icon_id,
        drawinfo: 0,
        obj,
        id_type: 0,
        obj_type,
        flag: 0,
    };
    let existing = reg.icons.insert(icon_id, icon);
    debug_assert!(existing.is_none());
    icon_id
}

/// `BKE_icons_init`.
pub fn bke_icons_init(first_dyn_id: i32) {
    let mut guard = REGISTRY.lock().unwrap();
    let reg = reg_get(&mut guard);
    reg.next_icon_id = first_dyn_id;
    reg.first_icon_id = first_dyn_id;
    reg.delete_queue.clear();
}

/// `BKE_icons_free` — free every icon (payload free via hooks).
pub fn bke_icons_free(hooks: &mut dyn IconHooks) {
    let mut guard = REGISTRY.lock().unwrap();
    let reg = reg_get(&mut guard);
    for (_, icon) in reg.icons.drain() {
        hooks.free_payload(icon.obj_type, icon.obj, icon.id);
    }
    reg.delete_queue.clear();
}

/// `BKE_icons_deferred_free` — free queued icons.
pub fn bke_icons_deferred_free(hooks: &mut dyn IconHooks) {
    let mut guard = REGISTRY.lock().unwrap();
    let reg = reg_get(&mut guard);
    for icon_id in reg.delete_queue.drain(..) {
        if let Some(icon) = reg.icons.remove(&icon_id) {
            hooks.free_payload(icon.obj_type, icon.obj, icon.id);
        }
    }
}

/// `BKE_icon_changed` — bumps the preview's `gputex` indicator in C;
/// here it only re-touches the icon (UI notification injected).
pub fn bke_icon_changed(icon_id: i32, hooks: &mut dyn IconHooks) -> bool {
    if icon_id == 0 || hooks.is_background() {
        return false;
    }
    REGISTRY
        .lock()
        .unwrap()
        .as_ref()
        .map(|reg| reg.icons.contains_key(&icon_id))
        .unwrap_or(false)
}

/// `BKE_icon_id_ensure` — allocate an icon for an ID when it lacks one.
/// `id` is the ID handle; `id_icon_id` gets/sets `id->icon_id`;
/// `set_id_icon_id` writes the new id back.
pub fn bke_icon_id_ensure(
    id: usize,
    id_icon_id: i32,
    hooks: &mut dyn IconHooks,
    set_id_icon_id: &mut dyn FnMut(usize, i32),
) -> i32 {
    if hooks.is_background() {
        return 0;
    }
    if id_icon_id != 0 {
        return id_icon_id;
    }
    let mut guard = REGISTRY.lock().unwrap();
    let reg = reg_get(&mut guard);
    let icon_id = get_next_free_id(reg);
    if icon_id == 0 {
        /* "not enough IDs" */
        return 0;
    }
    set_id_icon_id(id, icon_id);
    icon_create(reg, icon_id, ICON_DATA_ID as i16, id);
    icon_id
}

/// `BKE_icon_gplayer_color_ensure` — same for a GP layer handle.
pub fn bke_icon_gplayer_color_ensure(
    gpl: usize,
    gpl_icon_id: i32,
    hooks: &mut dyn IconHooks,
    set_gpl_icon_id: &mut dyn FnMut(usize, i32),
) -> i32 {
    if hooks.is_background() {
        return 0;
    }
    if gpl_icon_id != 0 {
        return gpl_icon_id;
    }
    let mut guard = REGISTRY.lock().unwrap();
    let reg = reg_get(&mut guard);
    let icon_id = get_next_free_id(reg);
    if icon_id == 0 {
        return 0;
    }
    set_gpl_icon_id(gpl, icon_id);
    icon_create(reg, icon_id, ICON_DATA_GPLAYER as i16, gpl);
    icon_id
}

/// `BKE_icon_preview_ensure` — icon for a PreviewImage handle.
pub fn bke_icon_preview_ensure(
    preview: usize,
    preview_icon_id: i32,
    hooks: &mut dyn IconHooks,
    set_preview_icon_id: &mut dyn FnMut(usize, i32),
) -> i32 {
    if hooks.is_background() {
        return 0;
    }
    if preview_icon_id != 0 {
        return preview_icon_id;
    }
    let mut guard = REGISTRY.lock().unwrap();
    let reg = reg_get(&mut guard);
    let icon_id = get_next_free_id(reg);
    if icon_id == 0 {
        return 0;
    }
    set_preview_icon_id(preview, icon_id);
    icon_create(reg, icon_id, ICON_DATA_PREVIEW as i16, preview);
    icon_id
}

/// `BKE_icon_imbuf_create` — create a managed icon wrapping an ImBuf.
pub fn bke_icon_imbuf_create(ibuf: usize) -> i32 {
    let mut guard = REGISTRY.lock().unwrap();
    let reg = reg_get(&mut guard);
    let icon_id = get_next_free_id(reg);
    icon_create(reg, icon_id, ICON_DATA_IMBUF as i16, ibuf);
    if let Some(icon) = reg.icons.get_mut(&icon_id) {
        icon.flag = ICON_FLAG_MANAGED;
    }
    icon_id
}

/// `BKE_icon_is_imbuf`.
pub fn bke_icon_is_imbuf(icon_id: i32) -> bool {
    REGISTRY
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|reg| reg.icons.get(&icon_id))
        .map(|icon| icon.obj_type == ICON_DATA_IMBUF as i16)
        .unwrap_or(false)
}

/// `BKE_icon_get` — payload handle of `icon_id`.
pub fn bke_icon_get(icon_id: i32) -> Option<(i16, usize)> {
    REGISTRY
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|reg| reg.icons.get(&icon_id))
        .map(|icon| (icon.obj_type, icon.obj))
}

/// `BKE_icon_set` — set `drawinfo`/`id_type` on an existing icon.
pub fn bke_icon_set(icon_id: i32, drawinfo: usize, id_type: i16) {
    if let Some(reg) = REGISTRY.lock().unwrap().as_mut() {
        if let Some(icon) = reg.icons.get_mut(&icon_id) {
            icon.drawinfo = drawinfo;
            icon.id_type = id_type;
        }
    }
}

/// `BKE_icon_id_delete` — queue the ID's icon for deferred delete.
pub fn bke_icon_id_delete(icon_id: i32) {
    if let Some(reg) = REGISTRY.lock().unwrap().as_mut() {
        if icon_id != 0 && reg.icons.contains_key(&icon_id) {
            reg.delete_queue.push(icon_id);
        }
    }
}

/// `BKE_icon_delete` — immediate delete; returns whether found.
pub fn bke_icon_delete(icon_id: i32, hooks: &mut dyn IconHooks) -> bool {
    if let Some(reg) = REGISTRY.lock().unwrap().as_mut() {
        if let Some(icon) = reg.icons.remove(&icon_id) {
            hooks.free_payload(icon.obj_type, icon.obj, icon.id);
            return true;
        }
    }
    false
}

/// `BKE_icon_delete_unmanaged` — delete only when not
/// `ICON_FLAG_MANAGED`.
pub fn bke_icon_delete_unmanaged(icon_id: i32, hooks: &mut dyn IconHooks) -> bool {
    if let Some(reg) = REGISTRY.lock().unwrap().as_mut() {
        match reg.icons.get(&icon_id) {
            Some(icon) if icon.flag & ICON_FLAG_MANAGED == 0 => {}
            _ => return false,
        }
        if let Some(icon) = reg.icons.remove(&icon_id) {
            hooks.free_payload(icon.obj_type, icon.obj, icon.id);
            return true;
        }
    }
    false
}

/// `BKE_icon_geom_ensure` — icon for `Icon_Geom` data.
pub fn bke_icon_geom_ensure(
    geom: usize,
    geom_icon_id: i32,
    hooks: &mut dyn IconHooks,
    set_geom_icon_id: &mut dyn FnMut(usize, i32),
) -> i32 {
    if hooks.is_background() {
        return 0;
    }
    if geom_icon_id != 0 {
        return geom_icon_id;
    }
    let mut guard = REGISTRY.lock().unwrap();
    let reg = reg_get(&mut guard);
    let icon_id = get_next_free_id(reg);
    if icon_id == 0 {
        return 0;
    }
    set_geom_icon_id(geom, icon_id);
    icon_create(reg, icon_id, ICON_DATA_GEOM as i16, geom);
    icon_id
}

/// `BKE_icon_ensure_studio_light` — icon for a StudioLight.
pub fn bke_icon_ensure_studio_light(
    sl: usize,
    id_type: i16,
    hooks: &mut dyn IconHooks,
) -> i32 {
    if hooks.is_background() {
        return 0;
    }
    let mut guard = REGISTRY.lock().unwrap();
    let reg = reg_get(&mut guard);
    let icon_id = get_next_free_id(reg);
    if icon_id == 0 {
        return 0;
    }
    icon_create(reg, icon_id, ICON_DATA_STUDIOLIGHT as i16, sl);
    if let Some(icon) = reg.icons.get_mut(&icon_id) {
        icon.id_type = id_type;
        icon.flag = ICON_FLAG_MANAGED;
    }
    icon_id
}
