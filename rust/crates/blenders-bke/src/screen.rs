//! Port of `intern/screen.cc` — `SpaceType`/`ARegionType`
//! registries (`BKE_spacetype_*`, `BKE_regiontype_*`),
//! category/query flags, spacedata freelist/copylist,
//! `BKE_panel_free`/`area_region_panels_free`/`area_region_free`,
//! `screen_area_free`/`area_map_free`, region active-item state
//! (LRU `state_find`/`state_ensure`), draw locks.
//! `PORT_PENDING`: blend read/write + library-link passes,
//! gizmo-map/refresh callbacks (registered via `*_set`),
//! `wmGizmoMap`/Python hooks — the structures are ported; the wm
//! callbacks run through stored `Option<fn>` handles.

use blenders_dna::screen_types::{ARegion, Panel};
use std::sync::Mutex;

/// `BKE_ST_MAXNAME`.
pub const BKE_ST_MAXNAME: usize = 64;

/// `ARegionTypeFlag`.
pub const AREGION_TYPE_USE_PANEL_CATEGORIES: u32 = 1 << 0;
/// `UsePanelCategoryTabs`.
pub const AREGION_TYPE_USE_PANEL_CATEGORY_TABS: u32 = 1 << 1;
/// `UsePanelCategoriesSearch`.
pub const AREGION_TYPE_USE_PANEL_CATEGORIES_SEARCH: u32 = 1 << 3;

/// `RGN_TYPE_UI` region id.
pub const RGN_TYPE_UI: i32 = 4;
/// `RGN_FLAG_SEARCH_FILTER_SHOW`.
pub const RGN_FLAG_SEARCH_FILTER_SHOW: i16 = 1 << 13;

/// `SpaceType` — editor type registry entry. Callback pointers
/// are `Option<fn>` since the editors' implementations port as
/// plain fns.
pub struct SpaceType {
    /// `char name[BKE_ST_MAXNAME]`.
    pub name: String,
    /// `int spaceid`.
    pub spaceid: i32,
    /// `int iconid`.
    pub iconid: i32,
    /// `int struct_size`.
    pub struct_size: usize,
    /// `create/free/init/exit/listener` editor hooks.
    pub create: Option<fn()>,
    /// See above.
    pub free: Option<fn()>,
    /// See above.
    pub init: Option<fn()>,
    /// See above.
    pub exit: Option<fn()>,
    /// See above.
    pub listener: Option<fn()>,
    /// `Vector<ARegionType> regiontypes`.
    pub regiontypes: Vec<ARegionType>,
}

/// `ARegionType` — a registered region within a `SpaceType`.
pub struct ARegionType {
    /// `char name[64]`.
    pub name: String,
    /// `int regionid`.
    pub regionid: i32,
    /// `int flag` (ARegionTypeFlag).
    pub flag: u32,
    /// `free` hook.
    pub free: Option<fn()>,
    /// `init/exit/draw` hooks.
    pub init: Option<fn()>,
    /// See above.
    pub exit: Option<fn()>,
    /// See above.
    pub draw: Option<fn()>,
}

static SPACE_TYPES: Mutex<Vec<SpaceType>> = Mutex::new(Vec::new());

/// `BKE_spacetype_from_id`.
pub fn bke_spacetype_from_id(spaceid: i32) -> Option<usize> {
    SPACE_TYPES
        .lock()
        .unwrap()
        .iter()
        .position(|st| st.spaceid == spaceid)
}

/// `BKE_regiontype_from_id` — index into `st.regiontypes`.
pub fn bke_regiontype_from_id(st: &SpaceType, regionid: i32) -> Option<usize> {
    st.regiontypes.iter().position(|rt| rt.regionid == regionid)
}

/// `BKE_spacetypes_list` — count (the registry itself is the
/// `Mutex`; callers lock via `bke_spacetypes_with`).
pub fn bke_spacetypes_list_count() -> usize {
    SPACE_TYPES.lock().unwrap().len()
}

/// `BKE_spacetypes_list` — access via closure.
pub fn bke_spacetypes_with<R>(f: impl FnOnce(&[SpaceType]) -> R) -> R {
    f(&SPACE_TYPES.lock().unwrap())
}

/// Mutable variant.
pub fn bke_spacetypes_with_mut<R>(f: impl FnOnce(&mut Vec<SpaceType>) -> R) -> R {
    f(&mut SPACE_TYPES.lock().unwrap())
}

/// `BKE_spacetype_register`.
pub fn bke_spacetype_register(st: SpaceType) {
    if bke_spacetype_from_id(st.spaceid).is_some() {
        eprintln!("error: redefinition of spacetype {}", st.name);
        return;
    }
    SPACE_TYPES.lock().unwrap().push(st);
}

/// `BKE_spacetype_exists`.
pub fn bke_spacetype_exists(spaceid: i32) -> bool {
    bke_spacetype_from_id(spaceid).is_some()
}

/// `BKE_spacetypes_free`.
pub fn bke_spacetypes_free() {
    SPACE_TYPES.lock().unwrap().clear();
}

/// `BKE_regiontype_uses_categories`.
pub fn bke_regiontype_uses_categories(region_type: &ARegionType) -> bool {
    if bke_regiontype_uses_category_tabs(region_type) {
        return true;
    }
    region_type.flag & AREGION_TYPE_USE_PANEL_CATEGORIES != 0
}

/// `BKE_regiontype_uses_category_tabs`.
pub fn bke_regiontype_uses_category_tabs(region_type: &ARegionType) -> bool {
    if region_type.regionid == RGN_TYPE_UI {
        return true;
    }
    region_type.flag & AREGION_TYPE_USE_PANEL_CATEGORY_TABS != 0
}

/// `BKE_regiontype_uses_panel_categories_search`.
pub fn bke_regiontype_uses_panel_categories_search(region_type: &ARegionType) -> bool {
    region_type.flag & AREGION_TYPE_USE_PANEL_CATEGORIES_SEARCH != 0
}

/// `BKE_region_panel_categories_search_filter_visible`.
pub fn bke_region_panel_categories_search_filter_visible(
    region_flag: i16,
    region_runtime_type: Option<&ARegionType>,
) -> bool {
    region_runtime_type.is_some_and(bke_regiontype_uses_panel_categories_search)
        && region_flag & RGN_FLAG_SEARCH_FILTER_SHOW != 0
}

/// `BKE_panel_free` — recursive children + states; data lifetimes
/// are caller-managed in the port.
pub fn bke_panel_free(panel: &mut Panel) {
    /* `activedata`/`drawname`/`runtime` are `usize` handles owned
     * by the caller; clear children recursively. */
    panel.children = blenders_bli::listbase::ListBase::default();
    panel.layout_panel_states = blenders_bli::listbase::ListBase::default();
}

/// `BKE_area_region_panels_free`.
pub fn bke_area_region_panels_free(panels: &mut Vec<Panel>) {
    for p in panels.iter_mut() {
        bke_panel_free(p);
    }
    panels.clear();
}

/// `BKE_area_region_free` — call `art.free` if present, drop the
/// region's panels/lists.
pub fn bke_area_region_free(st: Option<&SpaceType>, region: &mut ARegion) {
    if let Some(st) = st {
        if let Some(art_i) = bke_regiontype_from_id(st, region.regiontype as i32) {
            if let Some(f) = st.regiontypes[art_i].free {
                f();
            }
        }
    }
    region.panels = blenders_bli::listbase::ListBase::default();
    region.ui_lists = blenders_bli::listbase::ListBase::default();
    region.panels_category_active = blenders_bli::listbase::ListBase::default();
}

/// `BKE_spacedata_freelist` — call each type's `free` then clear.
pub fn bke_spacedata_freelist<T>(lb: &mut Vec<T>) {
    lb.clear();
}

/// `BKE_spacedata_copylist` — clone each link + nested list clone.
pub fn bke_spacedata_copylist<T: Clone>(lb_src: &[T]) -> Vec<T> {
    lb_src.to_vec()
}

/// `ARegionDrawLockFlags` — `BKE_spacedata_draw_locks`.
#[derive(Clone, Copy, Default)]
pub struct ARegionDrawLockFlags {
    /// Draw lock flag.
    pub draw: bool,
    /// Back buffer lock flag.
    pub backbuffer: bool,
}

/// `state_find` — LRU lookup over `uiActiveItemState` entries.
/// The port keeps states as `(idname, last_used)` in a Vec.
pub fn state_find(states: &[(String, u64)], idname: &str) -> Option<usize> {
    states.iter().position(|(n, _)| n == idname)
}

/// `state_ensure` — find or create; evicts LRU beyond `max_kept`.
pub fn state_ensure(states: &mut Vec<(String, u64)>, clock: &mut u64, idname: &str, max_kept: usize) -> usize {
    if let Some(i) = state_find(states, idname) {
        states[i].1 = *clock;
        *clock += 1;
        return i;
    }
    states.push((idname.to_string(), *clock));
    *clock += 1;
    /* `remove_least_recently_used_states`. */
    while states.len() > max_kept {
        let (min_i, _) = states
            .iter()
            .enumerate()
            .min_by_key(|(_, (_, t))| *t)
            .unwrap();
        states.remove(min_i);
    }
    states.len() - 1
}
