//! Port of `DNA_screen_types.h` — screen/area/region/UI panel types.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::asset_types::AssetLibraryReference;
use crate::id::{ID, NIL};
use crate::vec_types::Rcti;
use crate::view2d_types::View2D;
use blenders_bli::listbase::ListBase;
/// `eScreen_Flag`.
pub type ScreenFlag = i16;
pub const SCREEN_DEPRECATED: ScreenFlag = 1;
pub const SCREEN_COLLAPSE_STATUSBAR: ScreenFlag = 2;

/// `eScreen_State`.
pub type ScreenState = i8;
pub const SCREENNORMAL: ScreenState = 0;
pub const SCREENMAXIMIZED: ScreenState = 1;
pub const SCREENFULL: ScreenState = 2;

/// `eScreen_Fullscreen_Flag`.
pub type ScreenFullscreenFlag = i16;
pub const FULLSCREEN_RESTORE_GIZMO_NAVIGATE: ScreenFullscreenFlag = 1 << 0;
pub const FULLSCREEN_RESTORE_TEXT: ScreenFullscreenFlag = 1 << 1;
pub const FULLSCREEN_RESTORE_STATS: ScreenFullscreenFlag = 1 << 2;

/// `eScreen_Redraws_Flag`.
pub type ScreenRedrawsFlag = i16;
pub const TIME_REGION: ScreenRedrawsFlag = 1 << 0;
pub const TIME_ALL_3D_WIN: ScreenRedrawsFlag = 1 << 1;
pub const TIME_ALL_ANIM_WIN: ScreenRedrawsFlag = 1 << 2;
pub const TIME_ALL_BUTS_WIN: ScreenRedrawsFlag = 1 << 3;
pub const TIME_SEQ: ScreenRedrawsFlag = 1 << 5;
pub const TIME_ALL_IMAGE_WIN: ScreenRedrawsFlag = 1 << 6;
pub const TIME_NODES: ScreenRedrawsFlag = 1 << 8;
pub const TIME_CLIPS: ScreenRedrawsFlag = 1 << 9;
pub const TIME_SPREADSHEETS: ScreenRedrawsFlag = 1 << 10;
pub const TIME_FOLLOW: ScreenRedrawsFlag = 1 << 15;

/// `LayoutPanelStateFlag`.
pub type Layoutpanelstateflag = i32;
pub const LAYOUT_PANEL_STATE_FLAG_OPEN: Layoutpanelstateflag = 1 << 0;

/// `uiPanelDataExpansion`.
pub type Uipaneldataexpansion = i32;
pub const UI_PANEL_DATA_EXPAND_ROOT: Uipaneldataexpansion = 1 << 0;
pub const UI_SUBPANEL_DATA_EXPAND_1: Uipaneldataexpansion = 1 << 1;
pub const UI_SUBPANEL_DATA_EXPAND_2: Uipaneldataexpansion = 1 << 2;
pub const UI_SUBPANEL_DATA_EXPAND_3: Uipaneldataexpansion = 1 << 3;
pub const UI_SUBPANEL_DATA_EXPAND_4: Uipaneldataexpansion = 1 << 4;
pub const UI_SUBPANEL_DATA_EXPAND_5: Uipaneldataexpansion = 1 << 5;
pub const UI_SUBPANEL_DATA_EXPAND_6: Uipaneldataexpansion = 1 << 6;
pub const UI_SUBPANEL_DATA_EXPAND_7: Uipaneldataexpansion = 1 << 7;
pub const UI_SUBPANEL_DATA_EXPAND_8: Uipaneldataexpansion = 1 << 8;
pub const UI_SUBPANEL_DATA_EXPAND_9: Uipaneldataexpansion = 1 << 9;
pub const UI_SUBPANEL_DATA_EXPAND_10: Uipaneldataexpansion = 1 << 10;
pub const UI_SUBPANEL_DATA_EXPAND_11: Uipaneldataexpansion = 1 << 11;
pub const UI_SUBPANEL_DATA_EXPAND_12: Uipaneldataexpansion = 1 << 12;
pub const UI_SUBPANEL_DATA_EXPAND_13: Uipaneldataexpansion = 1 << 13;
pub const UI_SUBPANEL_DATA_EXPAND_14: Uipaneldataexpansion = 1 << 14;
pub const UI_SUBPANEL_DATA_EXPAND_15: Uipaneldataexpansion = 1 << 15;

/// `ePanel_Flag`.
pub type PanelFlag = i16;
pub const PNL_SELECT: PanelFlag = 1 << 0;
pub const PNL_UNUSED_1: PanelFlag = 1 << 1;
pub const PNL_CLOSED: PanelFlag = 1 << 2;
pub const PNL_PIN: PanelFlag = 1 << 5;
pub const PNL_POPOVER: PanelFlag = 1 << 6;
pub const PNL_INSTANCED_LIST_ORDER_CHANGED: PanelFlag = 1 << 7;

/// `euiList_LayoutType`.
pub type UilistLayouttype = i32;
pub const UILST_LAYOUT_DEFAULT: UilistLayouttype = 0;
pub const UILST_LAYOUT_COMPACT: UilistLayouttype = 1;

/// `euiList_Flag`.
pub type UilistFlag = i32;
pub const UILST_SCROLL_TO_ACTIVE_ITEM: UilistFlag = 1 << 0;

/// `euiList_Filter`.
pub type UilistFilter = i32;
pub const UILST_FLT_ITEM_NEVER_SHOW: UilistFilter = 1 << 16;
pub const UILST_FLT_ITEM: UilistFilter = 1 << 30;

/// `euiList_FilterFlag`.
pub type UilistFilterflag = i32;
pub const UILST_FLT_SHOW: UilistFilterflag = 1 << 0;
pub const UILST_FLT_EXCLUDE: UilistFilterflag = UILST_FLT_ITEM;

/// `euiList_FilterSortFlag`.
pub type UilistFiltersortflag = i32;
pub const UILST_FLT_SORT_ALPHA: UilistFiltersortflag = 1;
pub const UILST_FLT_SORT_LOCK: UilistFiltersortflag = 1 << 30;
pub const UILST_FLT_SORT_REVERSE: UilistFiltersortflag = 1 << 31;

/// `uiViewStateFlag`.
pub type Uiviewstateflag = u16;
pub const UI_VIEW_SHOW_FILTER_OPTIONS: Uiviewstateflag = 1 << 0;
pub const UI_VIEW_SORT_ALPHA: Uiviewstateflag = 1 << 1;
pub const UI_VIEW_FILTER_INVERT: Uiviewstateflag = 1 << 2;

/// `uiActiveItemStateFlag`.
pub type Uiactiveitemstateflag = i32;
pub const UI_ACTIVE_ITEM_STATE_FLAG_PENDING: Uiactiveitemstateflag = 1 << 0;

/// `uiPreviewTag`.
pub type Uipreviewtag = i16;
pub const UI_PREVIEW_TAG_DIRTY: Uipreviewtag = 1 << 0;

/// `GlobalAreaFlag`.
pub type Globalareaflag = i16;
pub const GLOBAL_AREA_IS_HIDDEN: Globalareaflag = 1 << 0;

/// `GlobalAreaAlign`.
pub type Globalareaalign = i16;
pub const GLOBAL_AREA_ALIGN_TOP: Globalareaalign = 0;
pub const GLOBAL_AREA_ALIGN_BOTTOM: Globalareaalign = 1;

/// `eScrArea_Flag`.
pub type ScrareaFlag = i16;
pub const HEADER_NO_PULLDOWN: ScrareaFlag = 1 << 0;
pub const AREA_TEMP_INFO: ScrareaFlag = 1 << 3;
pub const AREA_FLAG_REGION_SIZE_UPDATE: ScrareaFlag = 1 << 3;
pub const AREA_FLAG_ACTIVE_TOOL_UPDATE: ScrareaFlag = 1 << 4;
pub const AREA_FLAG_UNUSED_6: ScrareaFlag = 1 << 6;
pub const AREA_FLAG_STACKED_FULLSCREEN: ScrareaFlag = 1 << 7;
pub const AREA_FLAG_ACTIONZONES_UPDATE: ScrareaFlag = 1 << 8;
pub const AREA_FLAG_OFFSCREEN: ScrareaFlag = 1 << 9;

/// `eRegion_Type`.
pub type RegionType = i16;
pub const RGN_TYPE_WINDOW: RegionType = 0;
pub const RGN_TYPE_HEADER: RegionType = 1;
pub const RGN_TYPE_CHANNELS: RegionType = 2;
pub const RGN_TYPE_TEMPORARY: RegionType = 3;
pub const RGN_TYPE_UI: RegionType = 4;
pub const RGN_TYPE_TOOLS: RegionType = 5;
pub const RGN_TYPE_TOOL_PROPS: RegionType = 6;
pub const RGN_TYPE_PREVIEW: RegionType = 7;
pub const RGN_TYPE_HUD: RegionType = 8;
pub const RGN_TYPE_NAV_BAR: RegionType = 9;
pub const RGN_TYPE_EXECUTE: RegionType = 10;
pub const RGN_TYPE_FOOTER: RegionType = 11;
pub const RGN_TYPE_TOOL_HEADER: RegionType = 12;
pub const RGN_TYPE_XR: RegionType = 13;
pub const RGN_TYPE_ASSET_SHELF: RegionType = 14;
pub const RGN_TYPE_ASSET_SHELF_HEADER: RegionType = 15;
pub const RGN_TYPE_SCRUBBING: RegionType = 16;

/// `eRegion_Alignment`.
pub type RegionAlignment = i16;
pub const RGN_ALIGN_NONE: RegionAlignment = 0;
pub const RGN_ALIGN_TOP: RegionAlignment = 1;
pub const RGN_ALIGN_BOTTOM: RegionAlignment = 2;
pub const RGN_ALIGN_LEFT: RegionAlignment = 3;
pub const RGN_ALIGN_RIGHT: RegionAlignment = 4;
pub const RGN_ALIGN_HSPLIT: RegionAlignment = 5;
pub const RGN_ALIGN_VSPLIT: RegionAlignment = 6;
pub const RGN_ALIGN_FLOAT: RegionAlignment = 7;
pub const RGN_ALIGN_QSPLIT: RegionAlignment = 8;
pub const RGN_SPLIT_PREV: RegionAlignment = 1 << 5;
pub const RGN_SPLIT_SCALE_PREV: RegionAlignment = 1 << 6;
pub const RGN_ALIGN_HIDE_WITH_PREV: RegionAlignment = 1 << 7;
pub const RGN_STACK_ON_PREV: RegionAlignment = 1 << 8;

/// `eRegion_Flag`.
pub type RegionFlag = i16;
pub const RGN_FLAG_HIDDEN: RegionFlag = 1 << 0;
pub const RGN_FLAG_TOO_SMALL: RegionFlag = 1 << 1;
pub const RGN_FLAG_DYNAMIC_SIZE: RegionFlag = 1 << 2;
pub const RGN_FLAG_TEMP_REGIONDATA: RegionFlag = 1 << 3;
pub const RGN_FLAG_NO_USER_RESIZE: RegionFlag = 1 << 4;
pub const RGN_FLAG_SIZE_CLAMP_X: RegionFlag = 1 << 5;
pub const RGN_FLAG_SIZE_CLAMP_Y: RegionFlag = 1 << 6;
pub const RGN_FLAG_HIDDEN_BY_USER: RegionFlag = 1 << 7;
pub const RGN_FLAG_SEARCH_FILTER_ACTIVE: RegionFlag = 1 << 8;
pub const RGN_FLAG_SEARCH_FILTER_UPDATE: RegionFlag = 1 << 9;
pub const RGN_FLAG_POLL_FAILED: RegionFlag = 1 << 10;
pub const RGN_FLAG_RESIZE_RESPECT_BUTTON_SECTIONS: RegionFlag = 1 << 11;
pub const RGN_FLAG_INDICATE_OVERFLOW: RegionFlag = 1 << 12;
pub const RGN_FLAG_SEARCH_FILTER_SHOW: RegionFlag = 1 << 13;

/// `eRegion_DrawFlag`.
pub type RegionDrawflag = i32;
pub const RGN_DRAW: RegionDrawflag = 1;
pub const RGN_DRAW_PARTIAL: RegionDrawflag = 2;
pub const RGN_DRAW_NO_REBUILD: RegionDrawflag = 4;
pub const RGN_DRAWING: RegionDrawflag = 8;
pub const RGN_REFRESH_UI: RegionDrawflag = 16;
pub const RGN_DRAW_EDITOR_OVERLAYS: RegionDrawflag = 32;

/// `AssetShelfSettings_DisplayFlag`.
pub type AssetshelfsettingsDisplayflag = i16;
pub const ASSETSHELF_SHOW_NAMES: AssetshelfsettingsDisplayflag = 1 << 0;

/// `AssetShelf_InstanceFlag`.
pub type AssetshelfInstanceflag = i16;
pub const ASSETSHELF_REGION_IS_HIDDEN: AssetshelfInstanceflag = 1 << 0;

/// `bScreen`.
#[derive(Clone)]
pub struct bScreen {
    pub id: ID,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `Scene *scene`
    pub scene: usize,
    /// `eScreen_Flag flag`
    pub flag: ScreenFlag,
    /// `short winid`
    pub winid: i16,
    /// `eScreen_Redraws_Flag redraws_flag`
    pub redraws_flag: ScreenRedrawsFlag,
    /// `char temp`
    pub temp: i8,
    /// `eScreen_State state`
    pub state: ScreenState,
    /// `char do_draw`
    pub do_draw: i8,
    /// `char do_refresh`
    pub do_refresh: i8,
    /// `char do_draw_gesture`
    pub do_draw_gesture: i8,
    /// `char do_draw_cursor_overlay`
    pub do_draw_cursor_overlay: i8,
    /// `char do_draw_drag`
    pub do_draw_drag: i8,
    /// `char skip_handling`
    pub skip_handling: i8,
    /// `char scrubbing`
    pub scrubbing: i8,
    /// `ARegion *active_region`
    pub active_region: usize,
    /// `wmTimer *animtimer`
    pub animtimer: usize,
    /// `void *context`
    pub context: usize,
    /// `eScreen_Fullscreen_Flag fullscreen_flag`
    pub fullscreen_flag: ScreenFullscreenFlag,
    /// `wmTooltipState *tool_tip`
    pub tool_tip: usize,
    /// `PreviewImage *preview`
    pub preview: usize,
}
impl Default for bScreen {
    fn default() -> Self {
        Self {
            id: Default::default(),
            regionbase: ListBase::default(),
            scene: NIL,
            flag: Default::default(),
            winid: 0,
            redraws_flag: Default::default(),
            temp: 0,
            state: SCREENNORMAL,
            do_draw: 0,
            do_refresh: 0,
            do_draw_gesture: 0,
            do_draw_cursor_overlay: 0,
            do_draw_drag: 0,
            skip_handling: 0,
            scrubbing: 0,
            active_region: NIL,
            animtimer: NIL,
            context: NIL,
            fullscreen_flag: Default::default(),
            tool_tip: NIL,
            preview: NIL,
        }
    }
}

/// `ScrVert`.
#[derive(Clone)]
pub struct ScrVert {
    /// `ScrVert *next`
    pub next: usize,
    /// `ScrVert *prev`
    pub prev: usize,
    /// `ScrVert *newv`
    pub newv: usize,
    /// `vec2s vec`
    pub vec: [f32; 2],
    /// `short flag`
    pub flag: i16,
    /// `short editflag`
    pub editflag: i16,
}
impl Default for ScrVert {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            newv: NIL,
            vec: [0.0; 2],
            flag: 0,
            editflag: 0,
        }
    }
}

/// `ScrEdge`.
#[derive(Clone)]
pub struct ScrEdge {
    /// `ScrEdge *next`
    pub next: usize,
    /// `ScrEdge *prev`
    pub prev: usize,
    /// `ScrVert *v1`
    pub v1: usize,
    /// `ScrVert *v2`
    pub v2: usize,
    /// `short border`
    pub border: i16,
    /// `short flag`
    pub flag: i16,
}
impl Default for ScrEdge {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            v1: NIL,
            v2: NIL,
            border: 0,
            flag: 0,
        }
    }
}

/// `ScrAreaMap`.
#[derive(Clone)]
pub struct ScrAreaMap {
    /// `ListBaseT<ScrVert>`
    pub vertbase: ListBase,
    /// `ListBaseT<ScrEdge>`
    pub edgebase: ListBase,
    /// `ListBaseT<ScrArea>`
    pub areabase: ListBase,
}
impl Default for ScrAreaMap {
    fn default() -> Self {
        Self {
            vertbase: ListBase::default(),
            edgebase: ListBase::default(),
            areabase: ListBase::default(),
        }
    }
}

/// `LayoutPanelState`.
#[derive(Clone)]
pub struct LayoutPanelState {
    /// `LayoutPanelState *next`
    pub next: usize,
    /// `LayoutPanelState *prev`
    pub prev: usize,
    /// `char *idname`
    pub idname: usize,
    /// `LayoutPanelStateFlag flag`
    pub flag: Layoutpanelstateflag,
    /// `uint32_t last_used`
    pub last_used: u32,
}
impl Default for LayoutPanelState {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            idname: NIL,
            flag: Default::default(),
            last_used: 0,
        }
    }
}

/// `Panel`.
#[derive(Clone)]
pub struct Panel {
    /// `Panel *next`
    pub next: usize,
    /// `Panel *prev`
    pub prev: usize,
    /// `PanelType *type`
    pub type_: usize,
    /// `ui::Layout *layout`
    pub layout: usize,
    /// `char panelname[64]`
    pub panelname: [u8; 64],
    /// `char *drawname`
    pub drawname: usize,
    /// `int ofsx`
    pub ofsx: i32,
    /// `int ofsy`
    pub ofsy: i32,
    /// `int sizex`
    pub sizex: i32,
    /// `int sizey`
    pub sizey: i32,
    /// `int blocksizex`
    pub blocksizex: i32,
    /// `int blocksizey`
    pub blocksizey: i32,
    /// `short labelofs`
    pub labelofs: i16,
    /// `ePanel_Flag flag`
    pub flag: PanelFlag,
    /// `short runtime_flag`
    pub runtime_flag: i16,
    /// `int sortorder`
    pub sortorder: i32,
    /// `void *activedata`
    pub activedata: usize,
    /// `ListBaseT<Panel>`
    pub children: ListBase,
    /// `ListBaseT<LayoutPanelState>`
    pub layout_panel_states: ListBase,
    /// `uint32_t layout_panel_states_clock`
    pub layout_panel_states_clock: u32,
    /// `Panel_Runtime *runtime`
    pub runtime: usize,
}
impl Default for Panel {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            type_: NIL,
            layout: NIL,
            panelname: [0; 64],
            drawname: NIL,
            ofsx: 0,
            ofsy: 0,
            sizex: 0,
            sizey: 0,
            blocksizex: 0,
            blocksizey: 0,
            labelofs: 0,
            flag: Default::default(),
            runtime_flag: 0,
            sortorder: 0,
            activedata: NIL,
            children: ListBase::default(),
            layout_panel_states: ListBase::default(),
            layout_panel_states_clock: 0,
            runtime: NIL,
        }
    }
}

/// `PanelCategoryDyn`.
#[derive(Clone)]
pub struct PanelCategoryDyn {
    /// `PanelCategoryDyn *next`
    pub next: usize,
    /// `PanelCategoryDyn *prev`
    pub prev: usize,
    /// `char idname[64]`
    pub idname: [u8; 64],
    /// `int icon`
    pub icon: i32,
}
impl Default for PanelCategoryDyn {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            idname: [0; 64],
            icon: 0,
        }
    }
}

/// `PanelCategoryStack`.
#[derive(Clone)]
pub struct PanelCategoryStack {
    /// `PanelCategoryStack *next`
    pub next: usize,
    /// `PanelCategoryStack *prev`
    pub prev: usize,
    /// `char idname[64]`
    pub idname: [u8; 64],
}
impl Default for PanelCategoryStack {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            idname: [0; 64],
        }
    }
}

/// `uiListDyn`.
#[derive(Clone)]
pub struct uiListDyn {
    /// `uiListFreeRuntimeDataFunc free_runtime_data_fn`
    pub free_runtime_data_fn: usize,
    /// `int height`
    pub height: i32,
    /// `int visual_height`
    pub visual_height: i32,
    /// `int visual_height_min`
    pub visual_height_min: i32,
    /// `int items_len`
    pub items_len: i32,
    /// `int items_shown`
    pub items_shown: i32,
    /// `int resize`
    pub resize: i32,
    /// `int resize_prev`
    pub resize_prev: i32,
    /// `void *customdata`
    pub customdata: usize,
    /// `int *items_filter_flags`
    pub items_filter_flags: usize,
    /// `int *items_filter_neworder`
    pub items_filter_neworder: usize,
    /// `wmOperatorType *custom_drag_optype`
    pub custom_drag_optype: usize,
}
impl Default for uiListDyn {
    fn default() -> Self {
        Self {
            free_runtime_data_fn: Default::default(),
            height: 0,
            visual_height: 0,
            visual_height_min: 0,
            items_len: 0,
            items_shown: 0,
            resize: 0,
            resize_prev: 0,
            customdata: NIL,
            items_filter_flags: NIL,
            items_filter_neworder: NIL,
            custom_drag_optype: NIL,
        }
    }
}

/// `uiList`.
#[derive(Clone)]
pub struct uiList {
    /// `uiList *next`
    pub next: usize,
    /// `uiList *prev`
    pub prev: usize,
    /// `uiListType *type`
    pub type_: usize,
    /// `char list_id[256]`
    pub list_id: [u8; 256],
    /// `euiList_LayoutType layout_type`
    pub layout_type: UilistLayouttype,
    /// `euiList_Flag flag`
    pub flag: UilistFlag,
    /// `int list_scroll`
    pub list_scroll: i32,
    /// `int list_grip`
    pub list_grip: i32,
    /// `int list_last_len`
    pub list_last_len: i32,
    /// `int list_last_activei`
    pub list_last_activei: i32,
    /// `char filter_byname[256]`
    pub filter_byname: [u8; 256],
    /// `euiList_FilterFlag filter_flag`
    pub filter_flag: UilistFilterflag,
    /// `euiList_FilterSortFlag filter_sort_flag`
    pub filter_sort_flag: UilistFiltersortflag,
    /// `IDProperty *properties`
    pub properties: usize,
    /// `uiListDyn *dyn_data`
    pub dyn_data: usize,
}
impl Default for uiList {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            type_: NIL,
            list_id: [0; 256],
            layout_type: UILST_LAYOUT_DEFAULT,
            flag: Default::default(),
            list_scroll: 0,
            list_grip: 0,
            list_last_len: 0,
            list_last_activei: 0,
            filter_byname: [0; 256],
            filter_flag: Default::default(),
            filter_sort_flag: Default::default(),
            properties: NIL,
            dyn_data: NIL,
        }
    }
}

/// `uiViewState`.
#[derive(Clone)]
pub struct uiViewState {
    /// `int custom_height`
    pub custom_height: i32,
    /// `int scroll_offset`
    pub scroll_offset: i32,
    /// `uiViewStateFlag flag`
    pub flag: Uiviewstateflag,
    /// `uint8_t invert_sort_type`
    pub invert_sort_type: u8,
    /// `char search_string[256]`
    pub search_string: [u8; 256],
}
impl Default for uiViewState {
    fn default() -> Self {
        Self {
            custom_height: 0,
            scroll_offset: 0,
            flag: Default::default(),
            invert_sort_type: 0,
            search_string: [0; 256],
        }
    }
}

/// `uiViewStateLink`.
#[derive(Clone)]
pub struct uiViewStateLink {
    /// `uiViewStateLink *next`
    pub next: usize,
    /// `uiViewStateLink *prev`
    pub prev: usize,
    /// `char idname[64]`
    pub idname: [u8; 64],
    pub state: uiViewState,
}
impl Default for uiViewStateLink {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            idname: [0; 64],
            state: Default::default(),
        }
    }
}

/// `uiActiveItemState`.
#[derive(Clone)]
pub struct uiActiveItemState {
    /// `uiActiveItemState *next`
    pub next: usize,
    /// `uiActiveItemState *prev`
    pub prev: usize,
    /// `char *idname`
    pub idname: usize,
    /// `int active_index`
    pub active_index: i32,
    /// `uint32_t last_used`
    pub last_used: u32,
    /// `uiActiveItemStateFlag flag`
    pub flag: Uiactiveitemstateflag,
}
impl Default for uiActiveItemState {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            idname: NIL,
            active_index: 0,
            last_used: 0,
            flag: Default::default(),
        }
    }
}

/// `TransformOrientation`.
#[derive(Clone)]
pub struct TransformOrientation {
    /// `TransformOrientation *next`
    pub next: usize,
    /// `TransformOrientation *prev`
    pub prev: usize,
    /// `char name[64]`
    pub name: [u8; 64],
    /// `float mat[3][3]`
    pub mat: [[f32; 3]; 3],
}
impl Default for TransformOrientation {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            mat: [[0.0; 3]; 3],
        }
    }
}

/// `uiPreview`.
#[derive(Clone)]
pub struct uiPreview {
    /// `uiPreview *next`
    pub next: usize,
    /// `uiPreview *prev`
    pub prev: usize,
    /// `char preview_id[64]`
    pub preview_id: [u8; 64],
    /// `short height`
    pub height: i16,
    /// `uiPreviewTag tag`
    pub tag: Uipreviewtag,
    /// `unsigned_int id_session_uid`
    pub id_session_uid: u32,
}
impl Default for uiPreview {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            preview_id: [0; 64],
            height: 0,
            tag: Default::default(),
            id_session_uid: 0,
        }
    }
}

/// `TextboxState`.
#[derive(Clone)]
pub struct TextboxState {
    /// `int visible_lines`
    pub visible_lines: i32,
    /// `int scroll`
    pub scroll: i32,
}
impl Default for TextboxState {
    fn default() -> Self {
        Self {
            visible_lines: 3,
            scroll: 0,
        }
    }
}

/// `uiTextboxStateLink`.
#[derive(Clone)]
pub struct uiTextboxStateLink {
    /// `uiTextboxStateLink *next`
    pub next: usize,
    /// `uiTextboxStateLink *prev`
    pub prev: usize,
    /// `char *idname`
    pub idname: usize,
    pub state: TextboxState,
}
impl Default for uiTextboxStateLink {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            idname: NIL,
            state: Default::default(),
        }
    }
}

/// `ScrGlobalAreaData`.
#[derive(Clone)]
pub struct ScrGlobalAreaData {
    /// `short cur_fixed_height`
    pub cur_fixed_height: i16,
    /// `short size_min`
    pub size_min: i16,
    /// `short size_max`
    pub size_max: i16,
    /// `GlobalAreaAlign align`
    pub align: Globalareaalign,
    /// `GlobalAreaFlag flag`
    pub flag: Globalareaflag,
}
impl Default for ScrGlobalAreaData {
    fn default() -> Self {
        Self {
            cur_fixed_height: 0,
            size_min: 0,
            size_max: 0,
            align: GLOBAL_AREA_ALIGN_TOP,
            flag: Default::default(),
        }
    }
}

/// `ScrArea_Runtime`.
#[derive(Clone)]
pub struct ScrArea_Runtime {
    /// `bToolRef *tool`
    pub tool: usize,
    /// `char is_tool_set`
    pub is_tool_set: i8,
}
impl Default for ScrArea_Runtime {
    fn default() -> Self {
        Self {
            tool: NIL,
            is_tool_set: 0,
        }
    }
}

/// `ScrArea`.
#[derive(Clone)]
pub struct ScrArea {
    /// `ScrArea *next`
    pub next: usize,
    /// `ScrArea *prev`
    pub prev: usize,
    /// `ScrVert *v1`
    pub v1: usize,
    /// `ScrVert *v2`
    pub v2: usize,
    /// `ScrVert *v3`
    pub v3: usize,
    /// `ScrVert *v4`
    pub v4: usize,
    /// `bScreen *full`
    pub full: usize,
    pub totrct: Rcti,
    /// `char spacetype`
    pub spacetype: i8,
    /// `char butspacetype`
    pub butspacetype: i8,
    /// `short butspacetype_subtype`
    pub butspacetype_subtype: i16,
    /// `short winx`
    pub winx: i16,
    /// `short winy`
    pub winy: i16,
    /// `char headertype`
    pub headertype: i8,
    /// `char do_refresh`
    pub do_refresh: i8,
    /// `eScrArea_Flag flag`
    pub flag: ScrareaFlag,
    /// `short region_active_win`
    pub region_active_win: i16,
    /// `SpaceType *type`
    pub type_: usize,
    /// `ScrGlobalAreaData *global`
    pub global: usize,
    /// `float quadview_ratio[2]`
    pub quadview_ratio: [f32; 2],
    /// `ListBaseT<SpaceLink>`
    pub spacedata: ListBase,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    pub runtime: ScrArea_Runtime,
}
impl Default for ScrArea {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            v1: NIL,
            v2: NIL,
            v3: NIL,
            v4: NIL,
            full: NIL,
            totrct: Default::default(),
            spacetype: 0,
            butspacetype: 0,
            butspacetype_subtype: 0,
            winx: 0,
            winy: 0,
            headertype: 0,
            do_refresh: 0,
            flag: Default::default(),
            region_active_win: 0,
            type_: NIL,
            global: NIL,
            quadview_ratio: [0.0; 2],
            spacedata: ListBase::default(),
            regionbase: ListBase::default(),
            runtime: Default::default(),
        }
    }
}

/// `ARegion`.
#[derive(Clone)]
pub struct ARegion {
    /// `ARegion *next`
    pub next: usize,
    /// `ARegion *prev`
    pub prev: usize,
    pub v2d: View2D,
    pub winrct: Rcti,
    /// `short winx`
    pub winx: i16,
    /// `short winy`
    pub winy: i16,
    /// `int category_scroll`
    pub category_scroll: i32,
    /// `eRegion_Type regiontype`
    pub regiontype: RegionType,
    /// `eRegion_Alignment alignment`
    pub alignment: RegionAlignment,
    /// `eRegion_Flag flag`
    pub flag: RegionFlag,
    /// `short sizex`
    pub sizex: i16,
    /// `short sizey`
    pub sizey: i16,
    /// `short overlap`
    pub overlap: i16,
    /// `short flagfullscreen`
    pub flagfullscreen: i16,
    /// `ListBaseT<Panel>`
    pub panels: ListBase,
    /// `ListBaseT<PanelCategoryStack>`
    pub panels_category_active: ListBase,
    /// `ListBaseT<uiList>`
    pub ui_lists: ListBase,
    /// `ListBaseT<uiPreview>`
    pub ui_previews: ListBase,
    /// `ListBaseT<uiViewStateLink>`
    pub view_states: ListBase,
    /// `ListBaseT<uiTextboxStateLink>`
    pub textbox_states: ListBase,
    /// `ListBaseT<uiActiveItemState>`
    pub active_item_states: ListBase,
    /// `uint32_t active_item_states_clock`
    pub active_item_states_clock: u32,
    /// `void *regiondata`
    pub regiondata: usize,
    /// `bke::ARegionRuntime *runtime`
    pub runtime: usize,
}
impl Default for ARegion {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            v2d: Default::default(),
            winrct: Default::default(),
            winx: 0,
            winy: 0,
            category_scroll: 0,
            regiontype: RGN_TYPE_WINDOW,
            alignment: RGN_ALIGN_NONE,
            flag: Default::default(),
            sizex: 0,
            sizey: 0,
            overlap: 0,
            flagfullscreen: 0,
            panels: ListBase::default(),
            panels_category_active: ListBase::default(),
            ui_lists: ListBase::default(),
            ui_previews: ListBase::default(),
            view_states: ListBase::default(),
            textbox_states: ListBase::default(),
            active_item_states: ListBase::default(),
            active_item_states_clock: 0,
            regiondata: NIL,
            runtime: NIL,
        }
    }
}

/// `AssetShelfSettings`.
#[derive(Clone)]
pub struct AssetShelfSettings {
    pub asset_library_reference: AssetLibraryReference,
    /// `ListBaseT<AssetCatalogPathLink>`
    pub enabled_catalog_paths: ListBase,
    /// `char search_string[64]`
    pub search_string: [u8; 64],
    /// `short preview_size`
    pub preview_size: i16,
    /// `AssetShelfSettings_DisplayFlag display_flag`
    pub display_flag: AssetshelfsettingsDisplayflag,
}
impl Default for AssetShelfSettings {
    fn default() -> Self {
        Self {
            asset_library_reference: Default::default(),
            enabled_catalog_paths: ListBase::default(),
            search_string: [0; 64],
            preview_size: 0,
            display_flag: Default::default(),
        }
    }
}

/// `AssetShelf`.
#[derive(Clone)]
pub struct AssetShelf {
    /// `AssetShelf *next`
    pub next: usize,
    /// `AssetShelf *prev`
    pub prev: usize,
    /// `char idname[64]`
    pub idname: [u8; 64],
    /// `AssetShelfType *type`
    pub type_: usize,
    pub settings: AssetShelfSettings,
    /// `short preferred_row_count`
    pub preferred_row_count: i16,
    /// `AssetShelf_InstanceFlag instance_flag`
    pub instance_flag: AssetshelfInstanceflag,
}
impl Default for AssetShelf {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            idname: [0; 64],
            type_: NIL,
            settings: Default::default(),
            preferred_row_count: 0,
            instance_flag: Default::default(),
        }
    }
}

/// `RegionAssetShelf`.
#[derive(Clone)]
pub struct RegionAssetShelf {
    /// `ListBaseT<AssetShelf>`
    pub shelves: ListBase,
    /// `AssetShelf *active_shelf`
    pub active_shelf: usize,
}
impl Default for RegionAssetShelf {
    fn default() -> Self {
        Self {
            shelves: ListBase::default(),
            active_shelf: NIL,
        }
    }
}

/// `FileHandler`.
#[derive(Clone)]
pub struct FileHandler {
    /// `bke::FileHandlerType *type`
    pub type_: usize,
}
impl Default for FileHandler {
    fn default() -> Self {
        Self {
            type_: NIL,
        }
    }
}
