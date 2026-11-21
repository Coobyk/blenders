//! Port of `intern/context.cc` — `bContext`: the context
//! store (`BContextStore` name→value entries), the member
//! lookup chain (store → region → area → screen → window
//! callbacks), `bContextDataResult` typed results, and the
//! full `CTX_wm_*`/`CTX_data_*` getter surface.
//!
//! `PointerRNA` is modeled as `RnaPtr { owner_id, type_name,
//! data }` — type checks compare `StructRNA` identifier
//! strings. `PORT_PENDING`: python dict members
//! (`BPY_context_member_get`), `ctx_data_base_collection_get`
//! (needs `BKE_view_layer_synced_ensure`/`base_find`), the
//! `operator_poll_msg` tooltip params, RNA write-lock flag.

/// `eContextResult`.
pub const CTX_RESULT_MEMBER_NOT_FOUND: i32 = 0;
/// `CTX_RESULT_OK`.
pub const CTX_RESULT_OK: i32 = 1;
/// `CTX_RESULT_NO_DATA`.
pub const CTX_RESULT_NO_DATA: i32 = -1;

/// `PointerRNA` stand-in: owner ID + struct name + data
/// handle (usize).
#[derive(Clone, Default)]
pub struct RnaPtr {
    /// `owner_id`.
    pub owner_id: usize,
    /// `type` — `StructRNA` identifier.
    pub type_name: &'static str,
    /// `data`.
    pub data: usize,
}
impl RnaPtr {
    /// `ptr.data != nullptr`.
    pub fn is_valid(&self) -> bool {
        self.data != 0 && self.data != usize::MAX
    }
}

/// `bContextStoreEntry::value` variant.
#[derive(Clone)]
pub enum BContextStoreValue {
    /// `PointerRNA`.
    Ptr(RnaPtr),
    /// `std::string`.
    Str(String),
    /// `int64_t`.
    Int(i64),
}

/// `bContextStoreEntry`.
#[derive(Clone)]
pub struct BContextStoreEntry {
    /// `name`.
    pub name: String,
    /// `value`.
    pub value: BContextStoreValue,
}

/// `bContextStore`.
#[derive(Default)]
pub struct BContextStore {
    /// `entries`.
    pub entries: Vec<BContextStoreEntry>,
    /// `used`.
    pub used: bool,
}

/// `ContextDataType`.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ContextDataType {
    /// `Pointer`.
    #[default]
    Pointer,
    /// `Collection`.
    Collection,
    /// `Property`.
    Property,
    /// `String`.
    String,
    /// `Int64`.
    Int64,
}

/// `bContextDataResult`.
#[derive(Default)]
pub struct BContextDataResult {
    /// `ptr`.
    pub ptr: RnaPtr,
    /// `list`.
    pub list: Vec<RnaPtr>,
    /// `prop`/`index` — property pointer + element index.
    pub prop: usize,
    /// `index`.
    pub index: i32,
    /// `str`.
    pub str_: String,
    /// `int_value`.
    pub int_value: i64,
    /// `dir` — member-name list for `CTX_data_dir_get`.
    pub dir: Vec<&'static str>,
    /// `type`.
    pub type_: ContextDataType,
}

/// `bContextDataCallback`.
pub type BContextDataCallback =
    fn(&BContext, &str, &mut BContextDataResult) -> i32;

/// `bContext::wm` block.
pub struct BContextWm {
    /// `manager`.
    pub manager: usize,
    /// `window`.
    pub window: usize,
    /// `workspace`.
    pub workspace: usize,
    /// `screen`.
    pub screen: usize,
    /// `area`.
    pub area: usize,
    /// `region`.
    pub region: usize,
    /// `region_popup`.
    pub gizmo_group: usize,
    /// `region_popup`.
    pub region_popup: usize,
    /// `store` — `Option` (C nullable).
    pub store: Option<BContextStore>,
    /// `operator_poll_msg`.
    pub operator_poll_msg: Option<String>,
    /// Injected region-context callback (`region->type->
    /// context`), resolves through a caller-supplied lookup
    /// since `ARegionType` is unported.
    pub region_context_cb: Option<BContextDataCallback>,
    /// `area->type->context`.
    pub area_context_cb: Option<BContextDataCallback>,
    /// `screen->context`.
    pub screen_context_cb: Option<BContextDataCallback>,
    /// `window->type->context`.
    pub window_context_cb: Option<BContextDataCallback>,
}
impl Default for BContextWm {
    fn default() -> Self {
        Self {
            manager: usize::MAX,
            window: usize::MAX,
            workspace: usize::MAX,
            screen: usize::MAX,
            area: usize::MAX,
            region: usize::MAX,
            region_popup: usize::MAX,
            gizmo_group: usize::MAX,
            store: None,
            operator_poll_msg: None,
            region_context_cb: None,
            area_context_cb: None,
            screen_context_cb: None,
            window_context_cb: None,
        }
    }
}

/// `bContext::data` block.
pub struct BContextData {
    /// `main`.
    pub main: usize,
    /// `scene`.
    pub scene: usize,
    /// `recursion`.
    pub recursion: i32,
    /// `ui_data_access_deny`.
    pub ui_data_access_deny: bool,
    /// `py_init`.
    pub py_init: bool,
    /// `py_context` / `py_context_orig` — python dict handles.
    pub py_context: usize,
    /// `py_context_orig`.
    pub py_context_orig: usize,
    /// `rna_disallow_writes`.
    pub rna_disallow_writes: Option<bool>,
}
impl Default for BContextData {
    fn default() -> Self {
        Self {
            main: usize::MAX,
            scene: usize::MAX,
            recursion: 0,
            ui_data_access_deny: false,
            py_init: false,
            py_context: usize::MAX,
            py_context_orig: usize::MAX,
            rna_disallow_writes: None,
        }
    }
}

/// `bContext`.
#[derive(Default)]
pub struct BContext {
    /// `thread`.
    pub thread: i32,
    /// `wm`.
    pub wm: BContextWm,
    /// `data`.
    pub data: BContextData,
    /// Whether lookup happens off the main thread
    /// (`BLI_thread_is_main` stand-in).
    pub is_main_thread: bool,
}

/// `CTX_create` / `CTX_copy` / `CTX_free` — value semantics.
pub fn ctx_create() -> BContext {
    BContext {
        is_main_thread: true,
        ..Default::default()
    }
}

/// `CTX_store_add` — append entry by name.
pub fn ctx_store_add<'a>(store: &'a mut BContextStore, name: &str, value: BContextStoreValue) -> &'a mut BContextStoreEntry {
    store.used = true;
    store.entries.push(BContextStoreEntry {
        name: name.to_string(),
        value,
    });
    store.entries.last_mut().unwrap()
}

/// `CTX_store_ptr_lookup`.
pub fn ctx_store_ptr_lookup<'a>(
    store: &'a BContextStore,
    name: &str,
    type_: Option<&str>,
) -> Option<&'a RnaPtr> {
    store.entries.iter().rev().find_map(|e| {
        if e.name != name {
            return None;
        }
        if let BContextStoreValue::Ptr(p) = &e.value {
            if type_.is_none() || type_ == Some(p.type_name) {
                return Some(p);
            }
        }
        None
    })
}

/// `CTX_store_string_lookup`.
pub fn ctx_store_string_lookup<'a>(store: &'a BContextStore, name: &str) -> Option<&'a str> {
    store.entries.iter().rev().find_map(|e| {
        if e.name != name {
            return None;
        }
        if let BContextStoreValue::Str(s) = &e.value {
            return Some(s.as_str());
        }
        None
    })
}

/// `CTX_store_int_lookup`.
pub fn ctx_store_int_lookup(store: &BContextStore, name: &str) -> Option<i64> {
    store.entries.iter().rev().find_map(|e| {
        if e.name != name {
            return None;
        }
        if let BContextStoreValue::Int(v) = &e.value {
            return Some(*v);
        }
        None
    })
}

/// `CTX_py_init_get`/`set`, `CTX_py_dict_get`/`orig`.
pub fn ctx_py_init_get(c: &BContext) -> bool {
    c.data.py_init
}
/// `CTX_py_init_set`.
pub fn ctx_py_init_set(c: &mut BContext, value: bool) {
    c.data.py_init = value;
}
/// `CTX_py_dict_get`.
pub fn ctx_py_dict_get(c: &BContext) -> usize {
    c.data.py_context
}
/// `CTX_py_dict_get_orig`.
pub fn ctx_py_dict_get_orig(c: &BContext) -> usize {
    c.data.py_context_orig
}
/// `CTX_rna_disallow_write_set_p`.
pub fn ctx_rna_disallow_write_set_p(c: &mut BContext, v: Option<bool>) {
    c.data.rna_disallow_writes = v;
}

/// `ctx_data_get` — the verbatim priority chain:
/// store → region → area → screen → window (done
/// propagation via `-(-ret | -done)`).
pub fn ctx_data_get(c: &mut BContext, member: &str, result: &mut BContextDataResult) -> i32 {
    *result = BContextDataResult::default();
    let recursion = c.data.recursion;
    let mut done = 0;

    /* Python dict member lookup PORT_PENDING
     * (`BPY_context_member_get`). */

    if !c.is_main_thread || c.data.ui_data_access_deny {
        return CTX_RESULT_MEMBER_NOT_FOUND;
    }

    /* 1: `C->wm.store` */
    if done != 1 && recursion < 1 && c.wm.store.is_some() {
        c.data.recursion = 1;
        let store = c.wm.store.as_ref().unwrap();
        if let Some(ptr) = ctx_store_ptr_lookup(store, member, None) {
            result.ptr = ptr.clone();
            done = 1;
        } else if let Some(s) = ctx_store_string_lookup(store, member) {
            result.str_ = s.to_string();
            result.type_ = ContextDataType::String;
            done = 1;
        } else if let Some(v) = ctx_store_int_lookup(store, member) {
            result.int_value = v;
            result.type_ = ContextDataType::Int64;
            done = 1;
        }
    }
    /* 2: region callback. */
    if done != 1 && recursion < 2 && c.wm.region != usize::MAX {
        c.data.recursion = 2;
        if let Some(cb) = c.wm.region_context_cb {
            let ret = cb(c, member, result);
            if ret != 0 {
                done = -(-ret | -done);
            }
        }
    }
    /* 3: area. */
    if done != 1 && recursion < 3 && c.wm.area != usize::MAX {
        c.data.recursion = 3;
        if let Some(cb) = c.wm.area_context_cb {
            let ret = cb(c, member, result);
            if ret != 0 {
                done = -(-ret | -done);
            }
        }
    }
    /* 4: screen. */
    if done != 1 && recursion < 4 && c.wm.screen != usize::MAX {
        c.data.recursion = 4;
        if let Some(cb) = c.wm.screen_context_cb {
            let ret = cb(c, member, result);
            if ret != 0 {
                done = -(-ret | -done);
            }
        }
    }
    /* 5: window. */
    if done != 1 && recursion < 5 && c.wm.window != usize::MAX {
        c.data.recursion = 5;
        if let Some(cb) = c.wm.window_context_cb {
            let ret = cb(c, member, result);
            if ret != 0 {
                done = -(-ret | -done);
            }
        }
    }

    c.data.recursion = recursion;
    match done {
        1 => CTX_RESULT_OK,
        -1 => CTX_RESULT_NO_DATA,
        _ => CTX_RESULT_MEMBER_NOT_FOUND,
    }
}

/// `ctx_data_pointer_get`.
pub fn ctx_data_pointer_get(c: &mut BContext, member: &str) -> usize {
    let mut result = BContextDataResult::default();
    if ctx_data_get(c, member, &mut result) == CTX_RESULT_OK {
        return result.ptr.data;
    }
    0
}

/// `ctx_data_pointer_verify`.
pub fn ctx_data_pointer_verify(c: &mut BContext, member: &str) -> Option<usize> {
    let mut result = BContextDataResult::default();
    if ctx_data_get(c, member, &mut result) == CTX_RESULT_OK {
        Some(result.ptr.data)
    } else {
        None
    }
}

/// `ctx_data_collection_get`.
pub fn ctx_data_collection_get(c: &mut BContext, member: &str) -> Option<Vec<RnaPtr>> {
    let mut result = BContextDataResult::default();
    if ctx_data_get(c, member, &mut result) == CTX_RESULT_OK {
        Some(std::mem::take(&mut result.list))
    } else {
        None
    }
}

/// `CTX_data_pointer_get` — full `RnaPtr` (not just data).
pub fn data_pointer_get(c: &mut BContext, member: &str) -> Option<RnaPtr> {
    let mut result = BContextDataResult::default();
    if ctx_data_get(c, member, &mut result) == CTX_RESULT_OK
        && result.type_ == ContextDataType::Pointer
    {
        return Some(result.ptr);
    }
    None
}

/// `CTX_data_pointer_get_type(_silent)` — name-match against
/// `type_name` (warn path dropped: silent variant verbatim).
pub fn data_pointer_get_type_silent(
    c: &mut BContext,
    member: &str,
    type_name: &'static str,
) -> Option<RnaPtr> {
    let ptr = data_pointer_get(c, member)?;
    if ptr.type_name == type_name {
        Some(ptr)
    } else {
        None
    }
}

/// `CTX_data_string_get` / `CTX_data_int_get`.
pub fn data_string_get(c: &mut BContext, member: &str) -> Option<String> {
    let mut result = BContextDataResult::default();
    if ctx_data_get(c, member, &mut result) == CTX_RESULT_OK {
        return Some(result.str_);
    }
    None
}
/// `CTX_data_int_get`.
pub fn data_int_get(c: &mut BContext, member: &str) -> Option<i64> {
    let mut result = BContextDataResult::default();
    if ctx_data_get(c, member, &mut result) == CTX_RESULT_OK {
        return Some(result.int_value);
    }
    None
}

/* Result setters (verbatim). */

/// `CTX_data_id_pointer_set`.
pub fn data_id_pointer_set(result: &mut BContextDataResult, id: usize) {
    result.ptr.data = id;
    result.type_ = ContextDataType::Pointer;
}
/// `CTX_data_pointer_set`.
pub fn data_pointer_set(result: &mut BContextDataResult, id: usize, type_name: &'static str, data: usize) {
    result.ptr = RnaPtr {
        owner_id: id,
        type_name,
        data,
    };
    result.type_ = ContextDataType::Pointer;
}
/// `CTX_data_id_list_add`.
pub fn data_id_list_add(result: &mut BContextDataResult, id: usize) {
    result.list.push(RnaPtr {
        owner_id: id,
        data: id,
        ..Default::default()
    });
    result.type_ = ContextDataType::Collection;
}
/// `CTX_data_list_add`.
pub fn data_list_add(result: &mut BContextDataResult, id: usize, type_name: &'static str, data: usize) {
    result.list.push(RnaPtr {
        owner_id: id,
        type_name,
        data,
    });
    result.type_ = ContextDataType::Collection;
}
/// `CTX_data_prop_set`.
pub fn data_prop_set(result: &mut BContextDataResult, prop: usize, index: i32) {
    result.prop = prop;
    result.index = index;
    result.type_ = ContextDataType::Property;
}
/// `CTX_data_dir_set`.
pub fn data_dir_set(result: &mut BContextDataResult, dir: Vec<&'static str>) {
    result.dir = dir;
}
/// `CTX_data_type_set`/`get`.
pub fn data_type_set(result: &mut BContextDataResult, t: ContextDataType) {
    result.type_ = t;
}
/// `CTX_data_type_get`.
pub fn data_type_get(result: &BContextDataResult) -> ContextDataType {
    result.type_
}

/* ---- `CTX_wm_*` getters (verbatim member routing) ---- */

macro_rules! ctx_wm {
    ($name:ident, $member:literal) => {
        /// `CTX_wm_*`/`CTX_data_*` getter.
        pub fn $name(c: &mut BContext) -> usize {
            ctx_data_pointer_get(c, $member)
        }
    };
}

/// `CTX_wm_manager`.
pub fn wm_manager(c: &BContext) -> usize {
    c.wm.manager
}
/// `CTX_wm_interface_locked`.
pub fn wm_interface_locked(_c: &BContext) -> bool {
    false /* PORT_PENDING: wm->message_bus lock state. */
}
/// `CTX_wm_window`.
pub fn wm_window(c: &BContext) -> usize {
    /* `ctx_wm_python_context_get` fallback to `C->wm.window`
     * verbatim (py override PORT_PENDING). */
    c.wm.window
}
/// `CTX_wm_workspace`.
pub fn wm_workspace(c: &BContext) -> usize {
    c.wm.workspace
}
/// `CTX_wm_screen`.
pub fn wm_screen(c: &BContext) -> usize {
    c.wm.screen
}
/// `CTX_wm_area`.
pub fn wm_area(c: &BContext) -> usize {
    c.wm.area
}
/// `CTX_wm_space_data`.
pub fn wm_space_data(c: &mut BContext) -> usize {
    ctx_data_pointer_get(c, "space_data")
}
/// `CTX_wm_region`.
pub fn wm_region(c: &BContext) -> usize {
    c.wm.region
}
/// `CTX_wm_region_data`.
pub fn wm_region_data(c: &mut BContext) -> usize {
    ctx_data_pointer_get(c, "region_data")
}
/// `CTX_wm_region_popup`.
pub fn wm_region_popup(c: &BContext) -> usize {
    c.wm.region_popup
}
/// `CTX_wm_gizmo_group`.
pub fn wm_gizmo_group(c: &BContext) -> usize {
    c.wm.gizmo_group
}
ctx_wm!(wm_message_bus, "message_bus");
ctx_wm!(wm_reports, "reports");
ctx_wm!(wm_view3d, "view3d");
ctx_wm!(wm_region_view3d, "region_view3d");
ctx_wm!(wm_space_text, "space_text");
ctx_wm!(wm_space_console, "space_console");
ctx_wm!(wm_space_image, "space_image");
ctx_wm!(wm_space_properties, "space_properties");
ctx_wm!(wm_space_file, "space_file");
ctx_wm!(wm_space_graph, "space_graph");
ctx_wm!(wm_space_info, "space_info");
ctx_wm!(wm_space_nla, "space_nla");
ctx_wm!(wm_space_outliner, "space_outliner");
ctx_wm!(wm_space_node, "space_node");
ctx_wm!(wm_space_userpref, "space_userpref");
ctx_wm!(wm_space_clip, "space_clip");
ctx_wm!(wm_space_seq, "space_sequencer");
ctx_wm!(wm_space_dopesheet, "space_dopesheet");

/* ---- `CTX_data_*` getters ---- */

ctx_wm!(data_main, "main");
/// `CTX_data_main` fallback to `C->data.main`.
pub fn data_main(c: &mut BContext) -> usize {
    let v = ctx_data_pointer_get(c, "main");
    if v == 0 {
        c.data.main
    } else {
        v
    }
}
ctx_wm!(data_scene, "scene");
ctx_wm!(data_view_layer, "view_layer");
ctx_wm!(data_engine, "engine");
ctx_wm!(data_tool_settings, "tool_settings");
ctx_wm!(data_mode_string, "mode");
ctx_wm!(data_pref_path_prefices, "preferences_path_prefices");

ctx_wm!(data_selected_ids, "selected_ids");
ctx_wm!(data_selected_nodes, "selected_nodes");
ctx_wm!(data_selected_editable_objects, "selected_editable_objects");
ctx_wm!(data_editable_objects, "editable_objects");
ctx_wm!(data_selected_objects, "selected_objects");
ctx_wm!(data_visible_objects, "visible_objects");
ctx_wm!(data_selectable_objects, "selectable_objects");
ctx_wm!(data_active_object, "active_object");
ctx_wm!(data_edit_object, "edit_object");
ctx_wm!(data_edit_image, "edit_image");
ctx_wm!(data_edit_text, "edit_text");
ctx_wm!(data_edit_movieclip, "edit_movieclip");
ctx_wm!(data_edit_mask, "edit_mask");
ctx_wm!(data_active_bone, "active_bone");
ctx_wm!(data_edit_cachefile, "edit_cachefile");
ctx_wm!(data_selected_bones, "selected_bones");
ctx_wm!(data_selected_editable_bones, "selected_editable_bones");
ctx_wm!(data_visible_bones, "visible_bones");
ctx_wm!(data_editable_bones, "editable_bones");
ctx_wm!(data_active_pose_bone, "active_pose_bone");
ctx_wm!(data_selected_pose_bones, "selected_pose_bones");
ctx_wm!(
    data_selected_pose_bones_from_active_object,
    "selected_pose_bones_from_active_object"
);
ctx_wm!(data_visible_pose_bones, "visible_pose_bones");
ctx_wm!(data_asset_library_reference, "asset_library_reference");
ctx_wm!(data_asset, "asset");
ctx_wm!(data_selectable_gpencil_objects, "selectable_gpencil_objects");
ctx_wm!(data_selected_gpencil_objects, "selected_gpencil_objects");
ctx_wm!(data_editable_gpencil_objects, "editable_gpencil_objects");
ctx_wm!(data_visible_gpencil_objects, "visible_gpencil_objects");
ctx_wm!(data_editable_gpencil_strokes, "editable_gpencil_strokes");
ctx_wm!(data_visible_gpencil_strokes, "visible_gpencil_strokes");
ctx_wm!(data_selected_curves, "selected_curves");
ctx_wm!(data_editable_curves, "editable_curves");
ctx_wm!(data_visible_curves, "visible_curves");
ctx_wm!(data_selectable_curves, "selectable_curves");
ctx_wm!(data_active_gpencil_frame, "active_gpencil_frame");
ctx_wm!(data_active_gpencil_layer, "active_gpencil_layer");
ctx_wm!(data_visible_gpencil_layers, "visible_gpencil_layers");
ctx_wm!(data_editable_gpencil_layers, "editable_gpencil_layers");
ctx_wm!(data_selected_gpencil_layers, "selected_gpencil_layers");
ctx_wm!(data_active_annotation_layer, "active_annotation_layer");
ctx_wm!(data_visible_annotation_layers, "visible_annotation_layers");
ctx_wm!(data_editable_annotation_layers, "editable_annotation_layers");
ctx_wm!(data_selected_annotation_layers, "selected_annotation_layers");

/// `CTX_data_collection_remap_property` — replace each ptr
/// with its `propname` property pointer (RNA resolve
/// `PORT_PENDING` → passthrough marker).
pub fn data_collection_remap_property(collection: &mut [RnaPtr], propname: &str) {
    for ptr in collection {
        let _ = ptr;
        let _ = propname;
        /* `RNA_pointer_get(ptr, propname)` → `PORT_PENDING`. */
    }
}

/// `CTX_data_equals`.
pub fn data_equals(member: &str, s: &str) -> bool {
    member == s
}
/// `CTX_data_dir` — member is a directory entry.
pub fn data_dir(member: &str) -> bool {
    member.starts_with("__") == false && member.ends_with("/") == false
}

/// `CTX_data_mode_enum_ex` — `Object` (type, mode) pair →
/// `CTX_MODE_*` (verbatim switch; object fields injected).
pub const CTX_MODE_OBJECT: i32 = 0;
/// `CTX_MODE_EDIT_MESH`… `EDIT_CURVES`.
pub const CTX_MODE_EDIT_MESH: i32 = 1;
/// `CTX_MODE_EDIT_CURVE`.
pub const CTX_MODE_EDIT_CURVE: i32 = 2;
/// `CTX_MODE_EDIT_SURFACE`.
pub const CTX_MODE_EDIT_SURFACE: i32 = 3;
/// `CTX_MODE_EDIT_TEXT`.
pub const CTX_MODE_EDIT_TEXT: i32 = 4;
/// `CTX_MODE_EDIT_ARMATURE`.
pub const CTX_MODE_EDIT_ARMATURE: i32 = 5;
/// `CTX_MODE_EDIT_METABALL`.
pub const CTX_MODE_EDIT_METABALL: i32 = 6;
/// `CTX_MODE_EDIT_LATTICE`.
pub const CTX_MODE_EDIT_LATTICE: i32 = 7;
/// `CTX_MODE_EDIT_CURVES`.
pub const CTX_MODE_EDIT_CURVES: i32 = 8;
/// `CTX_MODE_EDIT_GREASE_PENCIL`.
pub const CTX_MODE_EDIT_GREASE_PENCIL: i32 = 9;
/// `CTX_MODE_POSE`.
pub const CTX_MODE_POSE: i32 = 10;
/// `CTX_MODE_SCULPT`.
pub const CTX_MODE_SCULPT: i32 = 11;
/// `CTX_MODE_PAINT_WEIGHT`.
pub const CTX_MODE_PAINT_WEIGHT: i32 = 12;
/// `CTX_MODE_PAINT_VERTEX`.
pub const CTX_MODE_PAINT_VERTEX: i32 = 13;
/// `CTX_MODE_PAINT_TEXTURE`.
pub const CTX_MODE_PAINT_TEXTURE: i32 = 14;
/// `CTX_MODE_PARTICLE`.
pub const CTX_MODE_PARTICLE: i32 = 15;
/// `CTX_MODE_PAINT_GPENCIL`/`SCULPT`/`WEIGHT`/`VERTEX`/`EDIT`.
pub const CTX_MODE_PAINT_GREASE_PENCIL: i32 = 16;
/// `CTX_MODE_SCULPT_GREASE_PENCIL`.
pub const CTX_MODE_SCULPT_GREASE_PENCIL: i32 = 17;
/// `CTX_MODE_WEIGHT_GREASE_PENCIL`.
pub const CTX_MODE_WEIGHT_GREASE_PENCIL: i32 = 18;
/// `CTX_MODE_VERTEX_GREASE_PENCIL`.
pub const CTX_MODE_VERTEX_GREASE_PENCIL: i32 = 19;
/// `CTX_MODE_EDIT_GPENCIL_LEGACY`.
pub const CTX_MODE_EDIT_GPENCIL_LEGACY: i32 = 20;
/// `CTX_MODE_SCULPT_CURVES`.
pub const CTX_MODE_SCULPT_CURVES: i32 = 21;
/// `CTX_MODE_NUM`.
pub const CTX_MODE_NUM: i32 = 22;

/* `OB_MODE_*`/`OB_TYPE_*` pairs consumed by
 * `data_mode_enum_ex` (verbatim values from
 * `DNA_object_enums.h`). */
const OB_MODE_OBJECT: i32 = 0;
const OB_MODE_EDIT: i32 = 1;
const OB_MODE_SCULPT: i32 = 2;
const OB_MODE_VERTEX_PAINT: i32 = 3;
const OB_MODE_WEIGHT_PAINT: i32 = 4;
const OB_MODE_TEXTURE_PAINT: i32 = 5;
const OB_MODE_PARTICLE_EDIT: i32 = 6;
const OB_MODE_POSE: i32 = 7;
const OB_MODE_EDIT_GPENCIL_LEGACY: i32 = 8;
const OB_MODE_PAINT_GPENCIL_LEGACY: i32 = 9;
const OB_MODE_SCULPT_GPENCIL_LEGACY: i32 = 10;
const OB_MODE_WEIGHT_GPENCIL_LEGACY: i32 = 11;
const OB_MODE_VERTEX_GPENCIL_LEGACY: i32 = 12;
const OB_MODE_SCULPT_CURVES: i32 = 13;

const OB_MESH: i16 = 1;
const OB_CURVE_LEGACY: i16 = 2;
const OB_SURF: i16 = 3;
const OB_FONT: i16 = 4;
const OB_MBALL: i16 = 5;
const OB_LATTICE: i16 = 7;
const OB_ARMATURE: i16 = 10;
const OB_CURVES: i16 = 16;
const OB_GREASE_PENCIL: i16 = 18;
const OB_POINTCLOUD: i16 = 19;
const OB_GPENCIL_LEGACY: i16 = 20;

/// `CTX_data_mode_enum_ex` (verbatim switch over
/// `obedit`/`obact` (type, mode) pairs).
pub fn data_mode_enum_ex(obedit: Option<(i16, i32)>, obact: Option<(i16, i32)>) -> i32 {
    if let Some((ty, mode)) = obedit {
        if ty == OB_MESH {
            return CTX_MODE_EDIT_MESH;
        }
        if ty == OB_CURVE_LEGACY {
            return CTX_MODE_EDIT_CURVE;
        }
        if ty == OB_SURF {
            return CTX_MODE_EDIT_SURFACE;
        }
        if ty == OB_FONT {
            return CTX_MODE_EDIT_TEXT;
        }
        if ty == OB_ARMATURE {
            return CTX_MODE_EDIT_ARMATURE;
        }
        if ty == OB_MBALL {
            return CTX_MODE_EDIT_METABALL;
        }
        if ty == OB_LATTICE {
            return CTX_MODE_EDIT_LATTICE;
        }
        if ty == OB_CURVES {
            return CTX_MODE_EDIT_CURVES;
        }
        if ty == OB_GREASE_PENCIL {
            return CTX_MODE_EDIT_GREASE_PENCIL;
        }
        if ty == OB_POINTCLOUD {
            return CTX_MODE_EDIT_MESH;
        }
        let _ = mode;
        return CTX_MODE_EDIT_MESH;
    }
    if let Some((ty, mode)) = obact {
        match mode {
            OB_MODE_POSE if ty == OB_ARMATURE => return CTX_MODE_POSE,
            OB_MODE_SCULPT => return CTX_MODE_SCULPT,
            OB_MODE_WEIGHT_PAINT => return CTX_MODE_PAINT_WEIGHT,
            OB_MODE_VERTEX_PAINT => return CTX_MODE_PAINT_VERTEX,
            OB_MODE_TEXTURE_PAINT => return CTX_MODE_PAINT_TEXTURE,
            OB_MODE_PARTICLE_EDIT => return CTX_MODE_PARTICLE,
            OB_MODE_PAINT_GPENCIL_LEGACY => return CTX_MODE_PAINT_GREASE_PENCIL,
            OB_MODE_EDIT_GPENCIL_LEGACY => return CTX_MODE_EDIT_GPENCIL_LEGACY,
            OB_MODE_SCULPT_GPENCIL_LEGACY => return CTX_MODE_SCULPT_GREASE_PENCIL,
            OB_MODE_WEIGHT_GPENCIL_LEGACY => return CTX_MODE_WEIGHT_GREASE_PENCIL,
            OB_MODE_VERTEX_GPENCIL_LEGACY => return CTX_MODE_VERTEX_GREASE_PENCIL,
            OB_MODE_SCULPT_CURVES => return CTX_MODE_SCULPT_CURVES,
            _ => {}
        }
        if ty == OB_GPENCIL_LEGACY && mode == OB_MODE_EDIT {
            return CTX_MODE_EDIT_GPENCIL_LEGACY;
        }
    }
    CTX_MODE_OBJECT
}
