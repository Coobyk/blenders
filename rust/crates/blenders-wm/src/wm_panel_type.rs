//! Port of `intern/wm_panel_type.cc` — the global
//! `PanelType` registry keyed by `idname`.

use std::collections::HashMap;

/// `PanelType` fields used (`idname`, `label`).
pub struct PanelType {
    /// `idname`.
    pub idname: String,
    /// `label`.
    pub label: String,
    /// `ext` — full type (PORT_PENDING until panels land).
    pub ext: usize,
}

/// `get_panel_type_map` (verbatim — static id→type map).
#[derive(Default)]
pub struct PanelTypeMap {
    map: HashMap<String, usize>,
    types: Vec<PanelType>,
}

impl PanelTypeMap {
    /// `WM_paneltype_find` (verbatim — empty→null, quiet
    /// flag prints).
    pub fn wm_paneltype_find(&self, idname: &str, quiet: bool, printf: &mut dyn FnMut(&str)) -> Option<usize> {
        if !idname.is_empty() {
            if let Some(&i) = self.map.get(idname) {
                return Some(i);
            }
        }
        if !quiet {
            (printf)(&format!("search for unknown paneltype {}", idname));
        }
        None
    }

    /// `WM_paneltype_add` (verbatim).
    pub fn wm_paneltype_add(&mut self, pt: PanelType) -> bool {
        let i = self.types.len();
        self.map.insert(pt.idname.clone(), i);
        self.types.push(pt);
        true
    }

    /// `WM_paneltype_remove` (verbatim — assert found).
    pub fn wm_paneltype_remove(&mut self, idname: &str) {
        let ok = self.map.remove(idname).is_some();
        assert!(ok);
    }

    /// `WM_paneltype_init` (verbatim — reserve 512).
    pub fn wm_paneltype_init(&mut self) {
        self.map.reserve(512);
    }

    /// `WM_paneltype_clear` (verbatim).
    pub fn wm_paneltype_clear(&mut self) {
        self.map.clear();
        self.types.clear();
    }

    /// `WM_paneltype_idname_visit_for_search` (verbatim —
    /// visit idname/label pairs).
    pub fn wm_paneltype_idname_visit_for_search(
        &self,
        visit_fn: &mut dyn FnMut(&str, &str),
    ) {
        for pt in &self.types {
            (visit_fn)(&pt.idname, &pt.label);
        }
    }
}
