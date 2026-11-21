//! Port of `DNA_windowmanager_types.h` — window manager, keymap and
//! operator types.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;

pub const OP_MAX_TYPENAME: usize = 64;
pub const KMAP_MAX_NAME: usize = 64;

/// `ReportTimerInfo`.
#[derive(Clone, Copy, Default)]
pub struct ReportTimerInfo {
    pub widthfac: f32,
    pub flash_progress: f32,
}

/// `wmXrData` (WITH_XR_OPENXR member of `wmWindowManager`).
#[derive(Clone, Default)]
pub struct WmXrData {
    /// `wmXrRuntimeData *runtime`.
    pub runtime: usize,
    /// `XrSessionSettings session_settings` — opaque, un-ported (xr session
    /// settings struct lives in `wm_xr.hh`, not DNA).
    pub session_settings: usize,
}

/// `eWM_ExtensionsUpdates`.
pub type EWmExtensionsupdates = i32;
pub const WM_EXTENSIONS_UPDATE_UNSET: EWmExtensionsupdates = -2;
pub const WM_EXTENSIONS_UPDATE_CHECKING: EWmExtensionsupdates = -1;

/// `eWM_InitFlag`.
pub type EWmInitflag = u8;
pub const WM_INIT_FLAG_WINDOW: EWmInitflag = 1 << 0;
pub const WM_INIT_FLAG_KEYCONFIG: EWmInitflag = 1 << 1;

/// `eWM_OutlinerSyncSelectDirty`.
pub type EWmOutlinersyncselectdirty = i16;
pub const WM_OUTLINER_SYNC_SELECT_FROM_OBJECT: EWmOutlinersyncselectdirty = 1 << 0;
pub const WM_OUTLINER_SYNC_SELECT_FROM_EDIT_BONE: EWmOutlinersyncselectdirty = 1 << 1;
pub const WM_OUTLINER_SYNC_SELECT_FROM_POSE_BONE: EWmOutlinersyncselectdirty = 1 << 2;
pub const WM_OUTLINER_SYNC_SELECT_FROM_SEQUENCE: EWmOutlinersyncselectdirty = 1 << 3;
pub const WM_OUTLINER_SYNC_SELECT_FROM_COLLECTION: EWmOutlinersyncselectdirty = 1 << 4;
pub const WM_OUTLINER_SYNC_SELECT_FROM_ALL: EWmOutlinersyncselectdirty =
    WM_OUTLINER_SYNC_SELECT_FROM_OBJECT | WM_OUTLINER_SYNC_SELECT_FROM_EDIT_BONE
        | WM_OUTLINER_SYNC_SELECT_FROM_POSE_BONE | WM_OUTLINER_SYNC_SELECT_FROM_SEQUENCE;

/// `wmWindowManager`.
#[derive(Clone)]
pub struct WmWindowManager {
    pub id: ID,
    /// `ListBaseT<wmWindow> windows`.
    pub windows: ListBase,
    pub init_flag: EWmInitflag,
    pub file_saved: i16,
    pub op_undo_depth: i16,
    pub outliner_sync_select_dirty: EWmOutlinersyncselectdirty,
    pub extensions_updates: EWmExtensionsupdates,
    pub extensions_blocked: i32,
    /// `wmTimer *autosavetimer`.
    pub autosavetimer: usize,
    pub autosave_scheduled: i8,
    pub xr: WmXrData,
    /// `bke::WindowManagerRuntime *runtime`.
    pub runtime: usize,
}
impl Default for WmWindowManager {
    fn default() -> Self {
        Self {
            id: ID::default(),
            windows: ListBase::default(),
            init_flag: 0,
            file_saved: 0,
            op_undo_depth: 0,
            outliner_sync_select_dirty: 0,
            extensions_updates: 0,
            extensions_blocked: 0,
            autosavetimer: NIL,
            autosave_scheduled: 0,
            xr: WmXrData::default(),
            runtime: NIL,
        }
    }
}

/// `ScrAreaMap` (screen types — global area list of a window).
#[derive(Clone, Default)]
pub struct ScrAreaMap {
    /// `ListBaseT<ScrArea> vertbase/horizbase/areabase` packed in C;
    /// arena handles here (layout fields come with the screen port).
    pub vertbase: ListBase,
    pub horizbase: ListBase,
    pub areabase: ListBase,
}

/// `wmWindow`.
#[derive(Clone)]
pub struct WmWindow {
    pub next: usize,
    pub prev: usize,
    /// `wmWindow *parent`.
    pub parent: usize,
    /// `Scene *scene` / `new_scene` / `unpinned_scene`.
    pub scene: usize,
    pub new_scene: usize,
    pub view_layer_name: [u8; 64],
    pub unpinned_scene: usize,
    /// `WorkSpaceInstanceHook *workspace_hook`.
    pub workspace_hook: usize,
    pub global_areas: ScrAreaMap,
    /// `bScreen *screen` (DNA_DEPRECATED).
    pub screen: usize,
    pub winid: i32,
    pub posx: i16,
    pub posy: i16,
    pub sizex: i16,
    pub sizey: i16,
    pub windowstate: i8,
    pub active: i8,
    pub cursor: i16,
    pub lastcursor: i16,
    pub modalcursor: i16,
    pub grabcursor: i16,
    pub pie_event_type_lock: i16,
    pub pie_event_type_last: i16,
    pub tag_cursor_refresh: i8,
    pub event_queue_check_click: i8,
    pub event_queue_check_drag: i8,
    pub event_queue_check_drag_handled: i8,
    pub event_queue_consecutive_gesture_type: i16,
    pub event_queue_consecutive_gesture_xy: [i32; 2],
    /// `wmEvent_ConsecutiveData *event_queue_consecutive_gesture_data`.
    pub event_queue_consecutive_gesture_data: usize,
    pub addmousemove: i8,
    /// `Stereo3dFormat *stereo3d_format`.
    pub stereo3d_format: usize,
    /// `bke::WindowRuntime *runtime`.
    pub runtime: usize,
}
impl Default for WmWindow {
    fn default() -> Self {
        Self {
            next: NIL, prev: NIL, parent: NIL,
            scene: NIL, new_scene: NIL,
            view_layer_name: [0; 64],
            unpinned_scene: NIL,
            workspace_hook: NIL,
            global_areas: ScrAreaMap::default(),
            screen: NIL,
            winid: 0,
            posx: 0, posy: 0, sizex: 0, sizey: 0,
            windowstate: 0, active: 0,
            cursor: 0, lastcursor: 0, modalcursor: 0, grabcursor: 0,
            pie_event_type_lock: 0, pie_event_type_last: 0,
            tag_cursor_refresh: 0,
            event_queue_check_click: 0, event_queue_check_drag: 0, event_queue_check_drag_handled: 0,
            event_queue_consecutive_gesture_type: 0,
            event_queue_consecutive_gesture_xy: [0; 2],
            event_queue_consecutive_gesture_data: NIL,
            addmousemove: 0,
            stereo3d_format: NIL,
            runtime: NIL,
        }
    }
}

/// `wmOperatorTypeMacro`.
#[derive(Clone)]
pub struct WmOperatorTypeMacro {
    pub next: usize,
    pub prev: usize,
    pub idname: [u8; OP_MAX_TYPENAME],
    /// `IDProperty *properties`.
    pub properties: usize,
    /// `PointerRNA *ptr`.
    pub ptr: usize,
}
impl Default for WmOperatorTypeMacro {
    fn default() -> Self {
        Self { next: NIL, prev: NIL, idname: [0; OP_MAX_TYPENAME], properties: NIL, ptr: NIL }
    }
}

/// `eKMI_Flag` (`#wmKeyMapItem.flag`).
pub type EKmiFlag = u8;
pub const KMI_INACTIVE: EKmiFlag = 1 << 0;
pub const KMI_EXPANDED: EKmiFlag = 1 << 1;
pub const KMI_USER_MODIFIED: EKmiFlag = 1 << 2;
pub const KMI_UPDATE: EKmiFlag = 1 << 3;
pub const KMI_REPEAT_IGNORE: EKmiFlag = 1 << 4;

/// `eKMI_MapType` (`#wmKeyMapItem.maptype`).
pub type EKmiMaptype = u8;
pub const KMI_TYPE_KEYBOARD: EKmiMaptype = 0;
pub const KMI_TYPE_MOUSE: EKmiMaptype = 1;
pub const KMI_TYPE_TEXTINPUT: EKmiMaptype = 3;
pub const KMI_TYPE_TIMER: EKmiMaptype = 4;
pub const KMI_TYPE_NDOF: EKmiMaptype = 5;

/// `eKeyMap_Flag` (`#wmKeyMap.flag`).
pub type EKeymapFlag = i16;
pub const KEYMAP_MODAL: EKeymapFlag = 1 << 0;
pub const KEYMAP_USER: EKeymapFlag = 1 << 1;
pub const KEYMAP_EXPANDED: EKeymapFlag = 1 << 2;
pub const KEYMAP_CHILDREN_EXPANDED: EKeymapFlag = 1 << 3;
pub const KEYMAP_DIFF: EKeymapFlag = 1 << 4;
pub const KEYMAP_USER_MODIFIED: EKeymapFlag = 1 << 5;
pub const KEYMAP_UPDATE: EKeymapFlag = 1 << 6;
pub const KEYMAP_TOOL: EKeymapFlag = 1 << 7;

/// `wmKeyMapItem`.
#[derive(Clone)]
pub struct WmKeyMapItem {
    pub next: usize,
    pub prev: usize,
    pub idname: [u8; 64],
    /// `IDProperty *properties`.
    pub properties: usize,
    pub propvalue_str: [u8; 64],
    pub propvalue: i16,
    pub type_: i16,
    pub val: i8,
    pub direction: i8,
    pub shift: i8,
    pub ctrl: i8,
    pub alt: i8,
    pub oskey: i8,
    pub hyper: i8,
    pub keymodifier: i16,
    pub flag: EKmiFlag,
    pub maptype: EKmiMaptype,
    pub id: i16,
    /// `PointerRNA *ptr`.
    pub ptr: usize,
}
impl Default for WmKeyMapItem {
    fn default() -> Self {
        Self {
            next: NIL, prev: NIL,
            idname: [0; 64],
            properties: NIL,
            propvalue_str: [0; 64],
            propvalue: 0,
            type_: 0, val: 0, direction: 0,
            shift: 0, ctrl: 0, alt: 0, oskey: 0, hyper: 0,
            keymodifier: 0,
            flag: 0,
            maptype: KMI_TYPE_KEYBOARD,
            id: 0,
            ptr: NIL,
        }
    }
}

/// `wmKeyMapDiffItem` — used instead of `wmKeyMapItem` for diff keymaps.
#[derive(Clone)]
pub struct WmKeyMapDiffItem {
    pub next: usize,
    pub prev: usize,
    /// `wmKeyMapItem *remove_item` / `*add_item` — arena handles.
    pub remove_item: usize,
    pub add_item: usize,
}
impl Default for WmKeyMapDiffItem {
    fn default() -> Self {
        Self { next: NIL, prev: NIL, remove_item: NIL, add_item: NIL }
    }
}

/// `wmKeyMap`.
#[derive(Clone)]
pub struct WmKeyMap {
    pub next: usize,
    pub prev: usize,
    /// `ListBaseT<wmKeyMapItem> items` / `ListBaseT<wmKeyMapDiffItem> diff_items`.
    pub items: ListBase,
    pub diff_items: ListBase,
    pub idname: [u8; KMAP_MAX_NAME],
    pub spaceid: i16,
    pub regionid: i16,
    pub owner_id: [u8; 128],
    pub flag: EKeymapFlag,
    pub kmi_id: i16,
    /// `bool (*poll)(bContext *)` — function pointers stay handles.
    pub poll: usize,
    /// `bool (*poll_modal_item)(const wmOperator *, int)`.
    pub poll_modal_item: usize,
    /// `const void *modal_items`.
    pub modal_items: usize,
}
impl Default for WmKeyMap {
    fn default() -> Self {
        Self {
            next: NIL, prev: NIL,
            items: ListBase::default(),
            diff_items: ListBase::default(),
            idname: [0; KMAP_MAX_NAME],
            spaceid: 0, regionid: 0,
            owner_id: [0; 128],
            flag: 0, kmi_id: 0,
            poll: NIL, poll_modal_item: NIL,
            modal_items: NIL,
        }
    }
}

/// `wmKeyConfigPref`.
#[derive(Clone)]
pub struct WmKeyConfigPref {
    pub next: usize,
    pub prev: usize,
    pub idname: [u8; 64],
    /// `IDProperty *prop` — group.
    pub prop: usize,
}
impl Default for WmKeyConfigPref {
    fn default() -> Self {
        Self { next: NIL, prev: NIL, idname: [0; 64], prop: NIL }
    }
}

/// `eKeyConf_Flag`.
pub type EKeyconfFlag = i16;
pub const KEYCONF_USER: EKeyconfFlag = 1 << 1;
pub const KEYCONF_INIT_DEFAULT: EKeyconfFlag = 1 << 2;

/// `wmKeyConfig`.
#[derive(Clone)]
pub struct WmKeyConfig {
    pub next: usize,
    pub prev: usize,
    pub idname: [u8; 64],
    pub basename: [u8; 64],
    /// `ListBaseT<wmKeyMap> keymaps`.
    pub keymaps: ListBase,
    pub actkeymap: i32,
    pub flag: EKeyconfFlag,
}
impl Default for WmKeyConfig {
    fn default() -> Self {
        Self {
            next: NIL, prev: NIL,
            idname: [0; 64], basename: [0; 64],
            keymaps: ListBase::default(),
            actkeymap: 0,
            flag: 0,
        }
    }
}

/// `eOperator_Flag` (from `WM_types.hh`, referenced by `wmOperator.flag`).
pub type EOperatorFlag = i16;

/// `wmOperator`.
#[derive(Clone)]
pub struct WmOperator {
    pub next: usize,
    pub prev: usize,
    pub idname: [u8; OP_MAX_TYPENAME],
    /// `IDProperty *properties`.
    pub properties: usize,
    /// `wmOperatorType *type`.
    pub type_: usize,
    /// `void *customdata` / `py_instance`.
    pub customdata: usize,
    pub py_instance: usize,
    /// `PointerRNA *ptr`.
    pub ptr: usize,
    /// `ReportList *reports`.
    pub reports: usize,
    /// `ListBaseT<wmOperator> macro`.
    pub macro_: ListBase,
    /// `wmOperator *opm`.
    pub opm: usize,
    /// `ui::Layout *layout`.
    pub layout: usize,
    pub flag: EOperatorFlag,
}
impl Default for WmOperator {
    fn default() -> Self {
        Self {
            next: NIL, prev: NIL,
            idname: [0; OP_MAX_TYPENAME],
            properties: NIL,
            type_: NIL,
            customdata: NIL, py_instance: NIL,
            ptr: NIL,
            reports: NIL,
            macro_: ListBase::default(),
            opm: NIL,
            layout: NIL,
            flag: 0,
        }
    }
}
