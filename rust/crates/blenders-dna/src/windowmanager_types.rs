//! Port of `DNA_windowmanager_types.h` — wmWindowManager/wmWindow/keymap/operator.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::id::{ID, NIL};
use crate::screen_types::ScrAreaMap;
use crate::windowmanager_enums::OperatorFlag;
use crate::xr_types::*;
use blenders_bli::listbase::ListBase;
/// `eWM_ExtensionsUpdates`.
pub type WmExtensionsupdates = i32;
pub const WM_EXTENSIONS_UPDATE_UNSET: WmExtensionsupdates = -2;
pub const WM_EXTENSIONS_UPDATE_CHECKING: WmExtensionsupdates = -1;

/// `eWM_InitFlag`.
pub type WmInitflag = u8;
pub const WM_INIT_FLAG_WINDOW: WmInitflag = 1 << 0;
pub const WM_INIT_FLAG_KEYCONFIG: WmInitflag = 1 << 1;

/// `eWM_OutlinerSyncSelectDirty`.
pub type WmOutlinersyncselectdirty = i16;
pub const WM_OUTLINER_SYNC_SELECT_FROM_OBJECT: WmOutlinersyncselectdirty = 1 << 0;
pub const WM_OUTLINER_SYNC_SELECT_FROM_EDIT_BONE: WmOutlinersyncselectdirty = 1 << 1;
pub const WM_OUTLINER_SYNC_SELECT_FROM_POSE_BONE: WmOutlinersyncselectdirty = 1 << 2;
pub const WM_OUTLINER_SYNC_SELECT_FROM_SEQUENCE: WmOutlinersyncselectdirty = 1 << 3;
pub const WM_OUTLINER_SYNC_SELECT_FROM_COLLECTION: WmOutlinersyncselectdirty = 1 << 4;

/// `eKMI_Flag`.
pub type KmiFlag = u8;
pub const KMI_INACTIVE: KmiFlag = 1 << 0;
pub const KMI_EXPANDED: KmiFlag = 1 << 1;
pub const KMI_USER_MODIFIED: KmiFlag = 1 << 2;
pub const KMI_UPDATE: KmiFlag = 1 << 3;
pub const KMI_REPEAT_IGNORE: KmiFlag = 1 << 4;

/// `eKMI_MapType`.
pub type KmiMaptype = u8;
pub const KMI_TYPE_KEYBOARD: KmiMaptype = 0;
pub const KMI_TYPE_MOUSE: KmiMaptype = 1;
pub const KMI_TYPE_TEXTINPUT: KmiMaptype = 3;
pub const KMI_TYPE_TIMER: KmiMaptype = 4;
pub const KMI_TYPE_NDOF: KmiMaptype = 5;

/// `eKeyMap_Flag`.
pub type KeymapFlag = i16;
pub const KEYMAP_MODAL: KeymapFlag = 1 << 0;
pub const KEYMAP_USER: KeymapFlag = 1 << 1;
pub const KEYMAP_EXPANDED: KeymapFlag = 1 << 2;
pub const KEYMAP_CHILDREN_EXPANDED: KeymapFlag = 1 << 3;
pub const KEYMAP_DIFF: KeymapFlag = 1 << 4;
pub const KEYMAP_USER_MODIFIED: KeymapFlag = 1 << 5;
pub const KEYMAP_UPDATE: KeymapFlag = 1 << 6;
pub const KEYMAP_TOOL: KeymapFlag = 1 << 7;

/// `eKeyConf_Flag`.
pub type KeyconfFlag = i16;
pub const KEYCONF_USER: KeyconfFlag = 1 << 1;
pub const KEYCONF_INIT_DEFAULT: KeyconfFlag = 1 << 2;


/// `ReportTimerInfo`.
#[derive(Clone)]
pub struct ReportTimerInfo {
    /// `float widthfac`
    pub widthfac: f32,
    /// `float flash_progress`
    pub flash_progress: f32,
}
impl Default for ReportTimerInfo {
    fn default() -> Self {
        Self {
            widthfac: 0.0,
            flash_progress: 0.0,
        }
    }
}

/// `wmXrData`.
#[derive(Clone)]
pub struct wmXrData {
    /// `wmXrRuntimeData *runtime`
    pub runtime: usize,
    pub session_settings: XrSessionSettings,
}
impl Default for wmXrData {
    fn default() -> Self {
        Self {
            runtime: NIL,
            session_settings: Default::default(),
        }
    }
}

/// `wmWindowManager`.
#[derive(Clone)]
pub struct wmWindowManager {
    pub id: ID,
    /// `ListBaseT<wmWindow>`
    pub windows: ListBase,
    /// `eWM_InitFlag init_flag`
    pub init_flag: WmInitflag,
    /// `short file_saved`
    pub file_saved: i16,
    /// `short op_undo_depth`
    pub op_undo_depth: i16,
    /// `eWM_OutlinerSyncSelectDirty outliner_sync_select_dirty`
    pub outliner_sync_select_dirty: WmOutlinersyncselectdirty,
    /// `eWM_ExtensionsUpdates extensions_updates`
    pub extensions_updates: WmExtensionsupdates,
    /// `int extensions_blocked`
    pub extensions_blocked: i32,
    /// `wmTimer *autosavetimer`
    pub autosavetimer: usize,
    /// `char autosave_scheduled`
    pub autosave_scheduled: i8,
    pub xr: wmXrData,
    /// `bke::WindowManagerRuntime *runtime`
    pub runtime: usize,
}
impl Default for wmWindowManager {
    fn default() -> Self {
        Self {
            id: Default::default(),
            windows: ListBase::default(),
            init_flag: Default::default(),
            file_saved: 0,
            op_undo_depth: 0,
            outliner_sync_select_dirty: Default::default(),
            extensions_updates: Default::default(),
            extensions_blocked: 0,
            autosavetimer: NIL,
            autosave_scheduled: 0,
            xr: Default::default(),
            runtime: NIL,
        }
    }
}

/// `wmWindow`.
#[derive(Clone)]
pub struct wmWindow {
    /// `wmWindow *next`
    pub next: usize,
    /// `wmWindow *prev`
    pub prev: usize,
    /// `wmWindow *parent`
    pub parent: usize,
    /// `Scene *scene`
    pub scene: usize,
    /// `Scene *new_scene`
    pub new_scene: usize,
    /// `char view_layer_name[64]`
    pub view_layer_name: [u8; 64],
    /// `Scene *unpinned_scene`
    pub unpinned_scene: usize,
    /// `WorkSpaceInstanceHook *workspace_hook`
    pub workspace_hook: usize,
    pub global_areas: ScrAreaMap,
    /// `bScreen *screen`
    pub screen: usize,
    /// `int winid`
    pub winid: i32,
    /// `short posx`
    pub posx: i16,
    /// `short posy`
    pub posy: i16,
    /// `short sizex`
    pub sizex: i16,
    /// `short sizey`
    pub sizey: i16,
    /// `char windowstate`
    pub windowstate: i8,
    /// `char active`
    pub active: i8,
    /// `short cursor`
    pub cursor: i16,
    /// `short lastcursor`
    pub lastcursor: i16,
    /// `short modalcursor`
    pub modalcursor: i16,
    /// `short grabcursor`
    pub grabcursor: i16,
    /// `short pie_event_type_lock`
    pub pie_event_type_lock: i16,
    /// `short pie_event_type_last`
    pub pie_event_type_last: i16,
    /// `char tag_cursor_refresh`
    pub tag_cursor_refresh: i8,
    /// `char event_queue_check_click`
    pub event_queue_check_click: i8,
    /// `char event_queue_check_drag`
    pub event_queue_check_drag: i8,
    /// `char event_queue_check_drag_handled`
    pub event_queue_check_drag_handled: i8,
    /// `short event_queue_consecutive_gesture_type`
    pub event_queue_consecutive_gesture_type: i16,
    /// `int event_queue_consecutive_gesture_xy[2]`
    pub event_queue_consecutive_gesture_xy: [i32; 2],
    /// `wmEvent_ConsecutiveData *event_queue_consecutive_gesture_data`
    pub event_queue_consecutive_gesture_data: usize,
    /// `char addmousemove`
    pub addmousemove: i8,
    /// `Stereo3dFormat *stereo3d_format`
    pub stereo3d_format: usize,
    /// `bke::WindowRuntime *runtime`
    pub runtime: usize,
}
impl Default for wmWindow {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            parent: NIL,
            scene: NIL,
            new_scene: NIL,
            view_layer_name: [0; 64],
            unpinned_scene: NIL,
            workspace_hook: NIL,
            global_areas: Default::default(),
            screen: NIL,
            winid: 0,
            posx: 0,
            posy: 0,
            sizex: 0,
            sizey: 0,
            windowstate: 0,
            active: 0,
            cursor: 0,
            lastcursor: 0,
            modalcursor: 0,
            grabcursor: 0,
            pie_event_type_lock: 0,
            pie_event_type_last: 0,
            tag_cursor_refresh: 0,
            event_queue_check_click: 0,
            event_queue_check_drag: 0,
            event_queue_check_drag_handled: 0,
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
pub struct wmOperatorTypeMacro {
    /// `wmOperatorTypeMacro *next`
    pub next: usize,
    /// `wmOperatorTypeMacro *prev`
    pub prev: usize,
    /// `char idname[64]`
    pub idname: [u8; 64],
    /// `IDProperty *properties`
    pub properties: usize,
    /// `PointerRNA *ptr`
    pub ptr: usize,
}
impl Default for wmOperatorTypeMacro {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            idname: [0; 64],
            properties: NIL,
            ptr: NIL,
        }
    }
}

/// `wmKeyMapItem`.
#[derive(Clone)]
pub struct wmKeyMapItem {
    /// `wmKeyMapItem *next`
    pub next: usize,
    /// `wmKeyMapItem *prev`
    pub prev: usize,
    /// `char idname[64]`
    pub idname: [u8; 64],
    /// `IDProperty *properties`
    pub properties: usize,
    /// `char propvalue_str[64]`
    pub propvalue_str: [u8; 64],
    /// `short propvalue`
    pub propvalue: i16,
    /// `short type`
    pub type_: i16,
    /// `int8_t val`
    pub val: i8,
    /// `int8_t direction`
    pub direction: i8,
    /// `int8_t shift`
    pub shift: i8,
    /// `int8_t ctrl`
    pub ctrl: i8,
    /// `int8_t alt`
    pub alt: i8,
    /// `int8_t oskey`
    pub oskey: i8,
    /// `int8_t hyper`
    pub hyper: i8,
    /// `short keymodifier`
    pub keymodifier: i16,
    /// `eKMI_Flag flag`
    pub flag: KmiFlag,
    /// `eKMI_MapType maptype`
    pub maptype: KmiMaptype,
    /// `short id`
    pub id: i16,
    /// `PointerRNA *ptr`
    pub ptr: usize,
}
impl Default for wmKeyMapItem {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            idname: [0; 64],
            properties: NIL,
            propvalue_str: [0; 64],
            propvalue: 0,
            type_: 0,
            val: 0,
            direction: 0,
            shift: 0,
            ctrl: 0,
            alt: 0,
            oskey: 0,
            hyper: 0,
            keymodifier: 0,
            flag: Default::default(),
            maptype: KMI_TYPE_KEYBOARD,
            id: 0,
            ptr: NIL,
        }
    }
}

/// `wmKeyMapDiffItem`.
#[derive(Clone)]
pub struct wmKeyMapDiffItem {
    /// `wmKeyMapDiffItem *next`
    pub next: usize,
    /// `wmKeyMapDiffItem *prev`
    pub prev: usize,
    /// `wmKeyMapItem *remove_item`
    pub remove_item: usize,
    /// `wmKeyMapItem *add_item`
    pub add_item: usize,
}
impl Default for wmKeyMapDiffItem {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            remove_item: NIL,
            add_item: NIL,
        }
    }
}

/// `wmKeyMap`.
#[derive(Clone)]
pub struct wmKeyMap {
    /// `wmKeyMap *next`
    pub next: usize,
    /// `wmKeyMap *prev`
    pub prev: usize,
    /// `ListBaseT<wmKeyMapItem>`
    pub items: ListBase,
    /// `ListBaseT<wmKeyMapDiffItem>`
    pub diff_items: ListBase,
    /// `char idname[64]`
    pub idname: [u8; 64],
    /// `short spaceid`
    pub spaceid: i16,
    /// `short regionid`
    pub regionid: i16,
    /// `char owner_id[128]`
    pub owner_id: [u8; 128],
    /// `eKeyMap_Flag flag`
    pub flag: KeymapFlag,
    /// `short kmi_id`
    pub kmi_id: i16,
}
impl Default for wmKeyMap {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            items: ListBase::default(),
            diff_items: ListBase::default(),
            idname: [0; 64],
            spaceid: 0,
            regionid: 0,
            owner_id: [0; 128],
            flag: Default::default(),
            kmi_id: 0,
        }
    }
}

/// `wmKeyConfigPref`.
#[derive(Clone)]
pub struct wmKeyConfigPref {
    /// `wmKeyConfigPref *next`
    pub next: usize,
    /// `wmKeyConfigPref *prev`
    pub prev: usize,
    /// `char idname[64]`
    pub idname: [u8; 64],
    /// `IDProperty *prop`
    pub prop: usize,
}
impl Default for wmKeyConfigPref {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            idname: [0; 64],
            prop: NIL,
        }
    }
}

/// `wmKeyConfig`.
#[derive(Clone)]
pub struct wmKeyConfig {
    /// `wmKeyConfig *next`
    pub next: usize,
    /// `wmKeyConfig *prev`
    pub prev: usize,
    /// `char idname[64]`
    pub idname: [u8; 64],
    /// `char basename[64]`
    pub basename: [u8; 64],
    /// `ListBaseT<wmKeyMap>`
    pub keymaps: ListBase,
    /// `int actkeymap`
    pub actkeymap: i32,
    /// `eKeyConf_Flag flag`
    pub flag: KeyconfFlag,
}
impl Default for wmKeyConfig {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            idname: [0; 64],
            basename: [0; 64],
            keymaps: ListBase::default(),
            actkeymap: 0,
            flag: Default::default(),
        }
    }
}

/// `wmOperator`.
#[derive(Clone)]
pub struct wmOperator {
    /// `wmOperator *next`
    pub next: usize,
    /// `wmOperator *prev`
    pub prev: usize,
    /// `char idname[64]`
    pub idname: [u8; 64],
    /// `IDProperty *properties`
    pub properties: usize,
    /// `wmOperatorType *type`
    pub type_: usize,
    /// `void *customdata`
    pub customdata: usize,
    /// `void *py_instance`
    pub py_instance: usize,
    /// `PointerRNA *ptr`
    pub ptr: usize,
    /// `ReportList *reports`
    pub reports: usize,
    /// `ListBaseT<wmOperator>`
    pub macro_: ListBase,
    /// `wmOperator *opm`
    pub opm: usize,
    /// `ui::Layout *layout`
    pub layout: usize,
    /// `eOperator_Flag flag`
    pub flag: OperatorFlag,
}
impl Default for wmOperator {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            idname: [0; 64],
            properties: NIL,
            type_: NIL,
            customdata: NIL,
            py_instance: NIL,
            ptr: NIL,
            reports: NIL,
            macro_: ListBase::default(),
            opm: NIL,
            layout: NIL,
            flag: Default::default(),
        }
    }
}
