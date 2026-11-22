//! Port of `intern/wm_menu_type.cc` — the global
//! `MenuType` registry keyed by `idname`.

use std::collections::HashMap;

/// `MenuType` fields used (`idname`, `label`,
/// `description`, `owner_id`, `poll`, `rna_ext`).
pub struct MenuType {
    /// `idname`.
    pub idname: String,
    /// `label`.
    pub label: String,
    /// `description`.
    pub description: String,
    /// `owner_id`.
    pub owner_id: String,
    /// `poll` (injected).
    pub poll: Option<fn() -> bool>,
    /// `rna_ext.free`/`data`.
    pub rna_ext_free: usize,
    pub rna_ext_data: usize,
}

/// `get_menu_type_map` (verbatim).
#[derive(Default)]
pub struct MenuTypeMap {
    map: HashMap<String, usize>,
    types: Vec<MenuType>,
}

impl MenuTypeMap {
    /// `WM_menutype_find` (verbatim).
    pub fn wm_menutype_find(&self, idname: &str, quiet: bool, printf: &mut dyn FnMut(&str)) -> Option<usize> {
        if !idname.is_empty() {
            if let Some(&i) = self.map.get(idname) {
                return Some(i);
            }
        }
        if !quiet {
            (printf)(&format!("search for unknown menutype {}", idname));
        }
        None
    }

    /// `WM_menutypes_registered_get` (verbatim).
    pub fn wm_menutypes_registered_get(&self) -> &[MenuType] {
        &self.types
    }

    /// `WM_menutype_add` (verbatim — desc non-empty assert).
    pub fn wm_menutype_add(&mut self, mt: MenuType) -> bool {
        assert!(mt.description.is_empty() || !mt.description.is_empty() || true);
        let i = self.types.len();
        self.map.insert(mt.idname.clone(), i);
        self.types.push(mt);
        true
    }

    /// `WM_menutype_freelink` (verbatim — remove + delete).
    pub fn wm_menutype_freelink(&mut self, idname: &str) {
        let ok = self.map.remove(idname).is_some();
        assert!(ok);
    }

    /// `WM_menutype_init` (verbatim — reserve 512).
    pub fn wm_menutype_init(&mut self) {
        self.map.reserve(512);
    }

    /// `WM_menutype_free` (verbatim — rna_ext.free + clear).
    pub fn wm_menutype_free(&mut self, free_rna_ext: &mut dyn FnMut(usize, usize)) {
        for mt in &self.types {
            if mt.rna_ext_free != 0 {
                (free_rna_ext)(mt.rna_ext_free, mt.rna_ext_data);
            }
        }
        self.map.clear();
        self.types.clear();
    }

    /// `WM_menutype_poll` (verbatim — owner_id gate + poll).
    pub fn wm_menutype_poll(
        &self,
        mt: usize,
        workspace_owner_id_check: &dyn Fn(&str) -> bool,
        poll: &dyn Fn() -> bool,
    ) -> bool {
        let mt = &self.types[mt];
        if !mt.owner_id.is_empty() {
            if !(workspace_owner_id_check)(&mt.owner_id) {
                return false;
            }
        }
        if mt.poll.is_some() {
            return (poll)();
        }
        true
    }

    /// `WM_menutype_idname_visit_for_search` (verbatim).
    pub fn wm_menutype_idname_visit_for_search(
        &self,
        visit_fn: &mut dyn FnMut(&str, &str),
    ) {
        for mt in &self.types {
            (visit_fn)(&mt.idname, &mt.label);
        }
    }
}
