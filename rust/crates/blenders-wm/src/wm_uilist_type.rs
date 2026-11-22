//! Port of `intern/wm_uilist_type.cc` — `uiListType`
//! registry + region unlink + full list-id naming.

use std::collections::HashMap;

/// `uiListType` fields used.
pub struct UiListType {
    /// `idname`.
    pub idname: String,
    /// `rna_ext.free`/`data`.
    pub rna_ext_free: usize,
    pub rna_ext_data: usize,
}

/// `UI_MAX_NAME_STR` verbatim.
pub const UI_MAX_NAME_STR: usize = 64;

/// `get_list_type_map` (verbatim).
#[derive(Default)]
pub struct UiListTypeMap {
    map: HashMap<String, usize>,
    types: Vec<UiListType>,
}

impl UiListTypeMap {
    /// `WM_uilisttype_find` (verbatim).
    pub fn wm_uilisttype_find(&self, idname: &str, quiet: bool, printf: &mut dyn FnMut(&str)) -> Option<usize> {
        if !idname.is_empty() {
            if let Some(&i) = self.map.get(idname) {
                return Some(i);
            }
        }
        if !quiet {
            (printf)(&format!("search for unknown uilisttype {}", idname));
        }
        None
    }

    /// `WM_uilisttype_add` (verbatim).
    pub fn wm_uilisttype_add(&mut self, ult: UiListType) -> bool {
        let i = self.types.len();
        self.map.insert(ult.idname.clone(), i);
        self.types.push(ult);
        true
    }

    /// `WM_uilisttype_remove_ptr` (verbatim — unlink +
    /// remove + delete).
    pub fn wm_uilisttype_remove_ptr(
        &mut self,
        idname: &str,
        unlink: &mut dyn FnMut(&str),
    ) {
        (unlink)(idname);
        let ok = self.map.remove(idname).is_some();
        assert!(ok);
    }

    /// `WM_uilisttype_init` (verbatim — reserve 16).
    pub fn wm_uilisttype_init(&mut self) {
        self.map.reserve(16);
    }

    /// `WM_uilisttype_free` (verbatim — rna_ext.free +
    /// clear).
    pub fn wm_uilisttype_free(&mut self, free_rna_ext: &mut dyn FnMut(usize, usize)) {
        for ult in &self.types {
            if ult.rna_ext_free != 0 {
                (free_rna_ext)(ult.rna_ext_free, ult.rna_ext_data);
            }
        }
        self.map.clear();
        self.types.clear();
    }
}

/// `WM_uilisttype_to_full_list_id` (verbatim —
/// `"{type}_{list}"`).
pub fn wm_uilisttype_to_full_list_id(type_idname: &str, list_id: Option<&str>) -> String {
    format!("{}_{}", type_idname, list_id.unwrap_or(""))
}

/// `WM_uilisttype_list_id_get` (verbatim — skip
/// `"{idname}_"` prefix).
pub fn wm_uilisttype_list_id_get<'a>(ult_idname: &str, list_list_id: &'a str) -> &'a str {
    assert!(list_list_id.as_bytes()[ult_idname.len()] == b'_');
    &list_list_id[ult_idname.len() + 1..]
}
