//! Port of `xr/intern/wm_xr_actionmap.cc` — XR
//! actionmap/item/binding containers + name-unique
//! helpers.

/// `WM_XR_ACTIONMAP*_STR_DEFAULT` verbatim.
pub const WM_XR_ACTIONMAP_STR_DEFAULT: &str = "Actionmap";
pub const WM_XR_ACTIONMAP_ITEM_STR_DEFAULT: &str = "Item";
pub const WM_XR_ACTIONMAP_BINDING_STR_DEFAULT: &str = "Binding";
/// `MAX_NAME` verbatim.
pub const MAX_NAME: usize = 64;

/// Shared name-dedup from `WM_xr_actionmap*_ensure_unique`
/// (verbatim — numeric suffix, fall back to default base
/// when too long).
fn ensure_unique_name(name: &mut String, default_base: &str, exists: &dyn Fn(&str) -> bool) {
    let mut baselen = name.len().min(MAX_NAME - 1);
    let mut idx = 0usize;
    while (exists)(&name[..baselen.min(name.len())].to_string() + &name[baselen..]) {
        idx += 1;
        if (baselen + 1) + ((idx as f64).log10() as usize + 1) > MAX_NAME {
            baselen = default_base.len().min(MAX_NAME - 1);
            *name = default_base[..baselen].to_string();
            idx = 0;
        } else {
            let mut base = name[..baselen].to_string();
            base.push_str(&idx.to_string());
            *name = base;
        }
    }
}

/// `WM_xr_actionmap_binding_ensure_unique`.
pub fn wm_xr_actionmap_binding_ensure_unique(
    name: &mut String,
    exists: &dyn Fn(&str) -> bool,
) {
    ensure_unique_name(name, WM_XR_ACTIONMAP_BINDING_STR_DEFAULT, exists);
}
/// `WM_xr_actionmap_item_ensure_unique`.
pub fn wm_xr_actionmap_item_ensure_unique(
    name: &mut String,
    exists: &dyn Fn(&str) -> bool,
) {
    ensure_unique_name(name, WM_XR_ACTIONMAP_ITEM_STR_DEFAULT, exists);
}
/// `WM_xr_actionmap_ensure_unique`.
pub fn wm_xr_actionmap_ensure_unique(
    name: &mut String,
    exists: &dyn Fn(&str) -> bool,
) {
    ensure_unique_name(name, WM_XR_ACTIONMAP_STR_DEFAULT, exists);
}

/// `WM_xr_actionmap_remove` index fix-up (verbatim —
/// actactionmap/selactionmap clamp).
pub fn wm_xr_actionmap_remove_fix_indices(
    idx: usize,
    actactionmap: &mut i32,
    selactionmap: &mut i32,
) {
    if (idx as i32) <= *actactionmap {
        *actactionmap -= 1;
        if *actactionmap < 0 {
            *actactionmap = 0;
        }
    }
    if (idx as i32) <= *selactionmap {
        *selactionmap -= 1;
        if *selactionmap < 0 {
            *selactionmap = 0;
        }
    }
}
