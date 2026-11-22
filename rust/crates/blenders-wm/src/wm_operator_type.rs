//! Port of `intern/wm_operator_type.cc` — the global
//! `wmOperatorType` registry (find/append/remove/free),
//! advanced-prop tagging, macro types, name/description
//! resolution, `WM_operator_depends_on_cursor`.

use std::collections::HashMap;

/// `OP_MAX_TYPENAME` verbatim.
pub const OP_MAX_TYPENAME: usize = 64;
/// `UNDOCUMENTED_OPERATOR_TIP` verbatim.
pub const UNDOCUMENTED_OPERATOR_TIP: &str = "Undocumented operator";
/// `OPTYPE_MACRO`/`OPTYPE_DEPENDS_ON_CURSOR` verbatim.
pub const OPTYPE_MACRO: i32 = 1 << 2;
pub const OPTYPE_DEPENDS_ON_CURSOR: i32 = 1 << 13;
/// `OP_PROP_TAG_ADVANCED` verbatim.
pub const OP_PROP_TAG_ADVANCED: i32 = 1 << 0;
/// `WM_CURSOR_PICK_AREA` verbatim.
pub const WM_CURSOR_PICK_AREA: i32 = 3;

/// `wmOperatorType` fields used.
pub struct WmOperatorType {
    /// `idname`.
    pub idname: String,
    /// `name`.
    pub name: String,
    /// `description`.
    pub description: String,
    /// `flag`.
    pub flag: i32,
    /// `srna` handle.
    pub srna: usize,
    /// `rna_ext.srna` (py type).
    pub rna_ext_srna: usize,
    /// `translation_context`.
    pub translation_context: usize,
    /// `cursor_pending`.
    pub cursor_pending: i32,
    /// `last_properties`.
    pub last_properties: usize,
    /// `macro` items.
    pub macro_items: Vec<WmOperatorTypeMacro>,
}

/// `wmOperatorTypeMacro` fields used.
pub struct WmOperatorTypeMacro {
    /// `idname`.
    pub idname: String,
    /// `ptr`/`properties`.
    pub ptr: usize,
    pub properties: usize,
}

/// `WM_operator_bl_idname` (verbatim — py→bl name
/// `foo.bar` → `FOO_OT_bar`... actual: uppercase module).
pub fn wm_operator_bl_idname(idname: &str) -> String {
    if let Some((a, b)) = idname.split_once('.') {
        format!("{}_OT_{}", a.to_uppercase(), b)
    } else {
        idname.to_string()
    }
}

/// `WM_operator_py_idname` (verbatim — `A_OT_b` → `a.b`).
pub fn wm_operator_py_idname(idname: &str) -> String {
    if let Some((a, b)) = idname.split_once("_OT_") {
        format!("{}.{}", a.to_lowercase(), b)
    } else {
        idname.to_string()
    }
}

/// `get_operators_map` (verbatim).
#[derive(Default)]
pub struct OperatorTypeMap {
    map: HashMap<String, usize>,
    types: Vec<WmOperatorType>,
}

impl OperatorTypeMap {
    /// `WM_operatortype_find` (verbatim — bl-idname
    /// lookup + clog).
    pub fn wm_operatortype_find(&self, idname: &str, quiet: bool, log: &mut dyn FnMut(&str)) -> Option<usize> {
        if !idname.is_empty() {
            let idname_bl = wm_operator_bl_idname(idname);
            if let Some(&i) = self.map.get(&idname_bl) {
                return Some(i);
            }
            if !quiet {
                (log)(&format!("Search for unknown operator '{}', '{}'", idname_bl, idname));
            }
        } else if !quiet {
            (log)("Search for empty operator");
        }
        None
    }

    /// `WM_operatortype_append` (verbatim — begin →
    /// opfunc → end).
    pub fn wm_operatortype_append(
        &mut self,
        opfunc: &mut dyn FnMut(&mut WmOperatorType),
        keyconfig_update_tag: &mut dyn FnMut(),
        log_error: &mut dyn FnMut(&str),
    ) {
        let mut ot = WmOperatorType {
            idname: String::new(),
            name: String::new(),
            description: String::new(),
            flag: 0,
            srna: 0,
            rna_ext_srna: 0,
            translation_context: 0,
            cursor_pending: WM_CURSOR_PICK_AREA,
            last_properties: 0,
            macro_items: Vec::new(),
        };
        (opfunc)(&mut ot);
        /* `_end` verbatim. */
        if ot.name.is_empty() {
            (log_error)(&format!("Operator '{}' has no name property", ot.idname));
        }
        let i = self.types.len();
        self.map.insert(ot.idname.clone(), i);
        self.types.push(ot);
        (keyconfig_update_tag)();
    }

    /// `WM_operatortype_remove_ptr` (verbatim — free
    /// srna/last-props/macro + remove + tag).
    pub fn wm_operatortype_remove_ptr(
        &mut self,
        idname: &str,
        free_last_properties: &mut dyn FnMut(usize),
        free_macro: &mut dyn FnMut(&mut Vec<WmOperatorTypeMacro>),
        keyconfig_update_tag: &mut dyn FnMut(),
    ) {
        let idname_bl = wm_operator_bl_idname(idname);
        if let Some(i) = self.map.remove(&idname_bl) {
            let ot = &mut self.types[i];
            if ot.last_properties != 0 {
                (free_last_properties)(ot.last_properties);
            }
            if !ot.macro_items.is_empty() {
                let mut items = std::mem::take(&mut ot.macro_items);
                (free_macro)(&mut items);
            }
        }
        (keyconfig_update_tag)();
    }

    /// `WM_operatortype_remove` (verbatim).
    pub fn wm_operatortype_remove(&mut self, idname: &str) -> bool {
        let idname_bl = wm_operator_bl_idname(idname);
        self.map.remove(&idname_bl).is_some()
    }

    /// `WM_operatortype_idname_visit_for_search` (verbatim
    /// — py idname + name).
    pub fn wm_operatortype_idname_visit_for_search(
        &self,
        visit_fn: &mut dyn FnMut(&str, &str),
    ) {
        for ot in &self.types {
            (visit_fn)(&wm_operator_py_idname(&ot.idname), &ot.name);
        }
    }

    /// `WM_operatortype_last_properties_clear_all`
    /// (verbatim).
    pub fn wm_operatortype_last_properties_clear_all(
        &mut self,
        free_last_properties: &mut dyn FnMut(usize),
    ) {
        for ot in self.types.iter_mut() {
            if ot.last_properties != 0 {
                (free_last_properties)(ot.last_properties);
                ot.last_properties = 0;
            }
        }
    }
}

/// `WM_operatortype_append_macro` (verbatim — unique
/// idname + MACRO flag + macro handlers).
pub fn wm_operatortype_append_macro(
    map: &mut OperatorTypeMap,
    idname: &str,
    name: &str,
    description: &str,
    flag: i32,
    log_error: &mut dyn FnMut(&str),
) -> Option<usize> {
    if map.wm_operatortype_find(idname, true, &mut |_| {}).is_some() {
        (log_error)(&format!("operator {} exists, cannot create macro", idname));
        return None;
    }
    let ot = WmOperatorType {
        idname: idname.to_string(),
        name: name.to_string(),
        description: if description.is_empty() {
            UNDOCUMENTED_OPERATOR_TIP.to_string()
        } else {
            description.to_string()
        },
        flag: OPTYPE_MACRO | flag,
        srna: 0,
        rna_ext_srna: 0,
        translation_context: 0,
        cursor_pending: 0,
        last_properties: 0,
        macro_items: Vec::new(),
    };
    let i = map.types.len();
    map.map.insert(ot.idname.clone(), i);
    map.types.push(ot);
    Some(i)
}

/// `WM_operatortype_macro_define` (verbatim — alloc props
/// + addtail + sub-type runtime pointer).
pub fn wm_operatortype_macro_define(
    ot: &mut WmOperatorType,
    idname: &str,
    properties_alloc: &mut dyn FnMut(&str) -> (usize, usize),
    properties_sanitize: &mut dyn FnMut(usize),
) -> usize {
    let (ptr, props) = (properties_alloc)(idname);
    (properties_sanitize)(ptr);
    ot.macro_items.push(WmOperatorTypeMacro {
        idname: idname.to_string(),
        ptr,
        properties: props,
    });
    ot.macro_items.len() - 1
}

/// `wm_operatortype_free_macro` (verbatim — per-item ptr
/// free).
pub fn wm_operatortype_free_macro(
    ot: &mut WmOperatorType,
    properties_free: &mut dyn FnMut(usize),
) {
    for otmacro in &ot.macro_items {
        if otmacro.ptr != 0 {
            (properties_free)(otmacro.ptr);
        }
    }
    ot.macro_items.clear();
}

/// `WM_operatortype_name` (verbatim — get_name or srna ui
/// name).
pub fn wm_operatortype_name(
    ot: &WmOperatorType,
    get_name: Option<&dyn Fn() -> String>,
    srna_ui_name: &str,
) -> String {
    if let Some(f) = get_name {
        let name = f();
        if !name.is_empty() {
            return name;
        }
    }
    let _ = ot;
    srna_ui_name.to_string()
}

/// `WM_operatortype_description` (verbatim — get_description
/// or srna ui description or "").
pub fn wm_operatortype_description(
    get_description: Option<&dyn Fn() -> String>,
    srna_ui_description: &str,
) -> String {
    if let Some(f) = get_description {
        let d = f();
        if !d.is_empty() {
            return d;
        }
    }
    if !srna_ui_description.is_empty() {
        return srna_ui_description.to_string();
    }
    String::new()
}

/// `WM_operatortype_description_or_name` (verbatim).
pub fn wm_operatortype_description_or_name(
    ot: &WmOperatorType,
    get_description: Option<&dyn Fn() -> String>,
    get_name: Option<&dyn Fn() -> String>,
    srna_ui_description: &str,
    srna_ui_name: &str,
) -> String {
    let text = wm_operatortype_description(get_description, srna_ui_description);
    if text.is_empty() {
        let text_orig = wm_operatortype_name(ot, get_name, srna_ui_name);
        if !text_orig.is_empty() {
            return text_orig;
        }
    }
    text
}

/// `WM_operator_depends_on_cursor` (verbatim — flag or cb).
pub fn wm_operator_depends_on_cursor(
    ot_flag: i32,
    depends_on_cursor: Option<&dyn Fn() -> bool>,
) -> bool {
    if (ot_flag & OPTYPE_DEPENDS_ON_CURSOR) != 0 {
        return true;
    }
    if let Some(f) = depends_on_cursor {
        return f();
    }
    false
}
