//! Port of `DNA_userdef_types.h` — `UserDef` preferences.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::id::NIL;
use crate::anim_enums::*;
use crate::curve_enums::*;
use crate::asset_types::{ASSET_IMPORT_PACK, ASSET_LIBRARY_RELATIVE_PATH, AssetAccess};
use crate::id::FILTER_ID_ALL;
use crate::color_types::ColorManagedViewSettings;
use crate::small_types::{BUuid, ColorBand};
use crate::space_enums::*;
use crate::vec_types::Rctf;
use blenders_bli::listbase::ListBase;
/// `eUserPref_Flag`.
pub type UserprefFlag = i32;
pub const USER_AUTOSAVE: UserprefFlag = 1 << 0;
pub const USER_FLAG_NUMINPUT_ADVANCED: UserprefFlag = 1 << 1;
pub const USER_FLAG_RECENT_SEARCHES_DISABLE: UserprefFlag = 1 << 2;
pub const USER_MENU_CLOSE_LEAVE: UserprefFlag = 1 << 3;
pub const USER_FLAG_UNUSED_4: UserprefFlag = 1 << 4;
pub const USER_TRACKBALL: UserprefFlag = 1 << 5;
pub const USER_FLAG_UNUSED_6: UserprefFlag = 1 << 6;
pub const USER_FLAG_UNUSED_7: UserprefFlag = 1 << 7;
pub const USER_MAT_ON_OB: UserprefFlag = 1 << 8;
pub const USER_INTERNET_ALLOW: UserprefFlag = 1 << 9;
pub const USER_DEVELOPER_UI: UserprefFlag = 1 << 10;
pub const USER_TOOLTIPS: UserprefFlag = 1 << 11;
pub const USER_TWOBUTTONMOUSE: UserprefFlag = 1 << 12;
pub const USER_NONUMPAD: UserprefFlag = 1 << 13;
pub const USER_ADD_CURSORALIGNED: UserprefFlag = 1 << 14;
pub const USER_FILECOMPRESS: UserprefFlag = 1 << 15;
pub const USER_FLAG_UNUSED_5: UserprefFlag = 1 << 16;
pub const USER_CUSTOM_RANGE: UserprefFlag = 1 << 17;
pub const USER_ADD_EDITMODE: UserprefFlag = 1 << 18;
pub const USER_ADD_VIEWALIGNED: UserprefFlag = 1 << 19;
pub const USER_RELPATHS: UserprefFlag = 1 << 20;
pub const USER_RELEASECONFIRM: UserprefFlag = 1 << 21;
pub const USER_SCRIPT_AUTOEXEC_DISABLE: UserprefFlag = 1 << 22;
pub const USER_FILENOUI: UserprefFlag = 1 << 23;
pub const USER_NONEGFRAMES: UserprefFlag = 1 << 24;
pub const USER_TXT_TABSTOSPACES_DISABLE: UserprefFlag = 1 << 25;
pub const USER_TOOLTIPS_PYTHON: UserprefFlag = 1 << 26;
pub const USER_FLAG_UNUSED_27: UserprefFlag = 1 << 27;
pub const USER_HIDE_DOT_DATABLOCK: UserprefFlag = 1 << 28;

/// `eUserPref_AssetFlag`.
pub type UserprefAssetflag = i8;
pub const USER_ASSETS_USE_ONLINE_ESSENTIALS: UserprefAssetflag = 1 << 0;

/// `eUserPref_ExtensionFlag`.
pub type UserprefExtensionflag = i8;
pub const USER_EXTENSION_FLAG_ONLINE_ACCESS_HANDLED: UserprefExtensionflag = 1 << 0;

/// `eUserpref_File_Preview_Type`.
pub type UserprefFilePreviewType = i8;
pub const USER_FILE_PREVIEW_NONE: UserprefFilePreviewType = 0;
pub const USER_FILE_PREVIEW_AUTO: UserprefFilePreviewType = USER_FILE_PREVIEW_NONE + 1;
pub const USER_FILE_PREVIEW_SCREENSHOT: UserprefFilePreviewType = USER_FILE_PREVIEW_AUTO + 1;
pub const USER_FILE_PREVIEW_CAMERA: UserprefFilePreviewType = USER_FILE_PREVIEW_SCREENSHOT + 1;

/// `eUserpref_Save_Modified_Images`.
pub type UserprefSaveModifiedImages = i8;
pub const USER_SAVE_MODIFIED_IMAGES_ASK: UserprefSaveModifiedImages = 0;
pub const USER_SAVE_MODIFIED_IMAGES_ALWAYS: UserprefSaveModifiedImages = USER_SAVE_MODIFIED_IMAGES_ASK + 1;
pub const USER_SAVE_MODIFIED_IMAGES_NEVER: UserprefSaveModifiedImages = USER_SAVE_MODIFIED_IMAGES_ALWAYS + 1;

/// `eUserPref_PrefFlag`.
pub type UserprefPrefflag = i8;
pub const USER_PREF_FLAG_SAVE: UserprefPrefflag = 1 << 0;
pub const USER_PREF_FLAG_PROJECT_SAVE: UserprefPrefflag = 1 << 1;
pub const USER_PREF_FLAG_PROJECT_REMEMBER: UserprefPrefflag = 1 << 2;

/// `eViewZoom_Style`.
pub type ViewzoomStyle = i8;
pub const USER_ZOOM_CONTINUE: ViewzoomStyle = 0;
pub const USER_ZOOM_SCALE: ViewzoomStyle = 1;
pub const USER_ZOOM_DOLLY: ViewzoomStyle = 2;

/// `eViewNavigation_Method`.
pub type ViewnavigationMethod = i8;
pub const VIEW_NAVIGATION_WALK: ViewnavigationMethod = 0;
pub const VIEW_NAVIGATION_FLY: ViewnavigationMethod = 1;

/// `eUserpref_MiniAxisType`.
pub type UserprefMiniaxistype = i8;
pub const USER_MINI_AXIS_TYPE_GIZMO: UserprefMiniaxistype = 0;
pub const USER_MINI_AXIS_TYPE_MINIMAL: UserprefMiniaxistype = 1;
pub const USER_MINI_AXIS_TYPE_NONE: UserprefMiniaxistype = 2;

/// `eWalkNavigation_Flag`.
pub type WalknavigationFlag = i16;
pub const USER_WALK_GRAVITY: WalknavigationFlag = 1 << 0;
pub const USER_WALK_MOUSE_REVERSE: WalknavigationFlag = 1 << 1;

/// `eUserpref_UI_Flag`.
pub type UserprefUiFlag = u32;
pub const USER_NO_MULTITOUCH_GESTURES: UserprefUiFlag = 1 << 0;
pub const USER_REDUCE_MOTION: UserprefUiFlag = 1 << 1;
pub const USER_WHEELZOOMDIR: UserprefUiFlag = 1 << 2;
pub const USER_FILTERFILEEXTS: UserprefUiFlag = 1 << 3;
pub const USER_DRAWVIEWINFO: UserprefUiFlag = 1 << 4;
pub const USER_PLAINMENUS: UserprefUiFlag = 1 << 5;
pub const USER_LOCK_CURSOR_ADJUST: UserprefUiFlag = 1 << 6;
pub const USER_HEADER_BOTTOM: UserprefUiFlag = 1 << 7;
pub const USER_HEADER_FROM_PREF: UserprefUiFlag = 1 << 8;
pub const USER_MENUOPENAUTO: UserprefUiFlag = 1 << 9;
pub const USER_DEPTH_CURSOR: UserprefUiFlag = 1 << 10;
pub const USER_AUTOPERSP: UserprefUiFlag = 1 << 11;
pub const USER_NODE_AUTO_OFFSET: UserprefUiFlag = 1 << 12;
pub const USER_GLOBALUNDO: UserprefUiFlag = 1 << 13;
pub const USER_ORBIT_SELECTION: UserprefUiFlag = 1 << 14;
pub const USER_DEPTH_NAVIGATE: UserprefUiFlag = 1 << 15;
pub const USER_HIDE_DOT: UserprefUiFlag = 1 << 16;
pub const USER_SHOW_GIZMO_NAVIGATE: UserprefUiFlag = 1 << 17;
pub const USER_SHOW_VIEWPORTNAME: UserprefUiFlag = 1 << 18;
pub const USER_AREA_CORNER_HANDLE: UserprefUiFlag = 1 << 19;
pub const USER_ZOOM_TO_MOUSEPOS: UserprefUiFlag = 1 << 20;
pub const USER_SHOW_FPS: UserprefUiFlag = 1 << 21;
pub const USER_REGISTER_ALL_USERS: UserprefUiFlag = 1 << 22;
pub const USER_FILTER_BRUSHES_BY_TOOL: UserprefUiFlag = 1 << 23;
pub const USER_CONTINUOUS_MOUSE: UserprefUiFlag = 1 << 24;
pub const USER_ZOOM_INVERT: UserprefUiFlag = 1 << 25;
pub const USER_ZOOM_HORIZ: UserprefUiFlag = 1 << 26;
pub const USER_SPLASH_DISABLE: UserprefUiFlag = 1 << 27;
pub const USER_HIDE_RECENT: UserprefUiFlag = 1 << 28;
pub const USER_SHOW_THUMBNAILS: UserprefUiFlag = 1 << 29;
pub const USER_SAVE_PROMPT: UserprefUiFlag = 1 << 30;
pub const USER_HIDE_SYSTEM_BOOKMARKS: UserprefUiFlag = 1 << 31;

/// `eUserpref_UI_Flag2`.
pub type UserprefUiFlag2 = i8;
pub const USER_ALWAYS_SHOW_NUMBER_ARROWS: UserprefUiFlag2 = 1 << 0;
pub const USER_REGION_OVERLAP: UserprefUiFlag2 = 1 << 1;
pub const USER_UIFLAG2_UNUSED_2: UserprefUiFlag2 = 1 << 2;
pub const USER_UIFLAG2_UNUSED_3: UserprefUiFlag2 = 1 << 3;
pub const USER_UIFLAG2_UNUSED_4: UserprefUiFlag2 = 1 << 4;
pub const USER_UIFLAG2_PANEL_TABS_COMPACT: UserprefUiFlag2 = 1 << 5;

/// `eUserpref_GPU_Flag`.
pub type UserprefGpuFlag = i8;
pub const USER_GPU_FLAG_WORKBENCH_RT_SHADOWS: UserprefGpuFlag = 1 << 0;
pub const USER_GPU_FLAG_NO_EDIT_MODE_SMOOTH_WIRE: UserprefGpuFlag = 1 << 1;
pub const USER_GPU_FLAG_OVERLAY_SMOOTH_WIRE: UserprefGpuFlag = 1 << 2;
pub const USER_GPU_FLAG_SUBDIVISION_EVALUATION: UserprefGpuFlag = 1 << 3;
pub const USER_GPU_FLAG_FRESNEL_EDIT: UserprefGpuFlag = 1 << 4;
pub const USER_GPU_FLAG_VIDEO_DECODING: UserprefGpuFlag = 1 << 5;

/// `eUserPref_GPUBackendType`.
pub type UserprefGpubackendtype = i16;
pub const USER_GPU_BACKEND_OPENGL: UserprefGpubackendtype = 1 << 0;
pub const USER_GPU_BACKEND_METAL: UserprefGpubackendtype = 1 << 1;
pub const USER_GPU_BACKEND_VULKAN: UserprefGpubackendtype = 1 << 3;
pub const USER_GPU_BACKEND_DEFAULT: UserprefGpubackendtype = USER_GPU_BACKEND_VULKAN;

/// `eUserpref_TabletAPI`.
pub type UserprefTabletapi = i16;
pub const USER_TABLET_AUTOMATIC: UserprefTabletapi = 0;
pub const USER_TABLET_NATIVE: UserprefTabletapi = 1;
pub const USER_TABLET_WINTAB: UserprefTabletapi = 2;

/// `eUserPref_Tablet_Flags`.
pub type UserprefTabletFlags = i32;
pub const USER_TABLET_SHOW_DEBUG_VALUES: UserprefTabletFlags = 1 << 0;

/// `eUserpref_APP_Flag`.
pub type UserprefAppFlag = i8;
pub const USER_APP_LOCK_CORNER_SPLIT: UserprefAppFlag = 1 << 0;
pub const USER_APP_HIDE_REGION_TOGGLE: UserprefAppFlag = 1 << 1;
pub const USER_APP_LOCK_EDGE_RESIZE: UserprefAppFlag = 1 << 2;

/// `eUserpref_StatusBar_Flag`.
pub type UserprefStatusbarFlag = i8;
pub const STATUSBAR_SHOW_MEMORY: UserprefStatusbarFlag = 1 << 0;
pub const STATUSBAR_SHOW_VRAM: UserprefStatusbarFlag = 1 << 1;
pub const STATUSBAR_SHOW_STATS: UserprefStatusbarFlag = 1 << 2;
pub const STATUSBAR_SHOW_VERSION: UserprefStatusbarFlag = 1 << 3;
pub const STATUSBAR_SHOW_SCENE_DURATION: UserprefStatusbarFlag = 1 << 4;
pub const STATUSBAR_SHOW_EXTENSIONS_UPDATES: UserprefStatusbarFlag = 1 << 5;

/// `eAutokey_Mode`.
pub type AutokeyMode = i16;
pub const AUTOKEY_ON: AutokeyMode = 1;
pub const AUTOKEY_MODE_NORMAL: AutokeyMode = 3;
pub const AUTOKEY_MODE_EDITKEYS: AutokeyMode = 5;

/// `eZoomFrame_Mode`.
pub type ZoomframeMode = i8;
pub const ZOOM_FRAME_MODE_KEEP_RANGE: ZoomframeMode = 0;
pub const ZOOM_FRAME_MODE_SECONDS: ZoomframeMode = 1;
pub const ZOOM_FRAME_MODE_KEYFRAMES: ZoomframeMode = 2;

/// `eKeying_Flag`.
pub type KeyingFlag = i16;
pub const KEYING_FLAG_VISUALKEY: KeyingFlag = 1 << 2;
pub const KEYING_FLAG_XYZ2RGB: KeyingFlag = 1 << 3;
pub const KEYING_FLAG_CYCLEAWARE: KeyingFlag = 1 << 8;
pub const AUTOKEY_FLAG_INSERTAVAILABLE: KeyingFlag = 1 << 0;
pub const AUTOKEY_FLAG_INSERTNEEDED: KeyingFlag = 1 << 1;
pub const AUTOKEY_FLAG_ONLYKEYINGSET: KeyingFlag = 1 << 6;
pub const AUTOKEY_FLAG_NOWARNING: KeyingFlag = 1 << 7;
pub const AUTOKEY_FLAG_LAYERED_RECORD: KeyingFlag = 1 << 10;
pub const MANUALKEY_FLAG_INSERTNEEDED: KeyingFlag = 1 << 11;

/// `eKeyInsertChannels`.
pub type Keyinsertchannels = i16;
pub const USER_ANIM_KEY_CHANNEL_LOCATION: Keyinsertchannels = 1 << 0;
pub const USER_ANIM_KEY_CHANNEL_ROTATION: Keyinsertchannels = 1 << 1;
pub const USER_ANIM_KEY_CHANNEL_SCALE: Keyinsertchannels = 1 << 2;
pub const USER_ANIM_KEY_CHANNEL_ROTATION_MODE: Keyinsertchannels = 1 << 3;
pub const USER_ANIM_KEY_CHANNEL_CUSTOM_PROPERTIES: Keyinsertchannels = 1 << 4;

/// `eUserpref_Anim_Flags`.
pub type UserprefAnimFlags = i16;
pub const USER_ANIM_SHOW_CHANNEL_GROUP_COLORS: UserprefAnimFlags = 1 << 0;
pub const USER_ANIM_ONLY_SHOW_SELECTED_CURVE_KEYS: UserprefAnimFlags = 1 << 1;
pub const USER_ANIM_HIGH_QUALITY_DRAWING: UserprefAnimFlags = 1 << 2;

/// `eFixToCam_Flags`.
pub type FixtocamFlags = u8;
pub const FIX_TO_CAM_FLAG_USE_LOC: FixtocamFlags = 1 << 0;
pub const FIX_TO_CAM_FLAG_USE_ROT: FixtocamFlags = 1 << 1;
pub const FIX_TO_CAM_FLAG_USE_SCALE: FixtocamFlags = 1 << 2;

/// `eUserpref_Translation_Flags`.
pub type UserprefTranslationFlags = i16;
pub const USER_TR_TOOLTIPS: UserprefTranslationFlags = 1 << 0;
pub const USER_TR_IFACE: UserprefTranslationFlags = 1 << 1;
pub const USER_TR_REPORTS: UserprefTranslationFlags = 1 << 2;
pub const USER_TR_UNUSED_3: UserprefTranslationFlags = 1 << 3;
pub const USER_TR_UNUSED_4: UserprefTranslationFlags = 1 << 4;
pub const USER_DOTRANSLATE_DEPRECATED: UserprefTranslationFlags = 1 << 5;
pub const USER_TR_UNUSED_6: UserprefTranslationFlags = 1 << 6;
pub const USER_TR_UNUSED_7: UserprefTranslationFlags = 1 << 7;
pub const USER_TR_NEWDATANAME: UserprefTranslationFlags = 1 << 8;

/// `eTextEdit_Flags`.
pub type TexteditFlags = i8;
pub const USER_TEXT_EDIT_AUTO_CLOSE: TexteditFlags = 1 << 0;

/// `eText_Draw_Options`.
pub type TextDrawOptions = i8;
pub const USER_TEXT_DISABLE_AA: TextDrawOptions = 1 << 0;
pub const USER_TEXT_HINTING_NONE: TextDrawOptions = 1 << 1;
pub const USER_TEXT_HINTING_SLIGHT: TextDrawOptions = 1 << 2;
pub const USER_TEXT_HINTING_FULL: TextDrawOptions = 1 << 3;
pub const USER_TEXT_RENDER_SUBPIXELAA: TextDrawOptions = 1 << 4;

/// `eGP_UserdefSettings`.
pub type GpUserdefsettings = i16;
pub const GP_PAINT_UNUSED_0: GpUserdefsettings = 1 << 0;

/// `eUserpref_Gizmo_Flag`.
pub type UserprefGizmoFlag = i8;
pub const USER_GIZMO_DRAW: UserprefGizmoFlag = 1 << 0;

/// `eColorPicker_Types`.
pub type ColorpickerTypes = i16;
pub const USER_CP_CIRCLE_HSV: ColorpickerTypes = 0;
pub const USER_CP_SQUARE_SV: ColorpickerTypes = 1;
pub const USER_CP_SQUARE_HS: ColorpickerTypes = 2;
pub const USER_CP_SQUARE_HV: ColorpickerTypes = 3;
pub const USER_CP_CIRCLE_HSL: ColorpickerTypes = 4;

/// `eTimecodeStyles`.
pub type Timecodestyles = i16;
pub const USER_TIMECODE_MINIMAL: Timecodestyles = 0;
pub const USER_TIMECODE_SMPTE_MSF: Timecodestyles = 1;
pub const USER_TIMECODE_SMPTE_FULL: Timecodestyles = 2;
pub const USER_TIMECODE_MILLISECONDS: Timecodestyles = 3;
pub const USER_TIMECODE_SECONDS_ONLY: Timecodestyles = 4;
pub const USER_TIMECODE_SUBRIP: Timecodestyles = 100;

/// `eNdof_Flag`.
pub type NdofFlag = i32;
pub const NDOF_SHOW_GUIDE_ORBIT_AXIS: NdofFlag = 1 << 0;
pub const NDOF_FLY_HELICOPTER: NdofFlag = 1 << 1;
pub const NDOF_LOCK_HORIZON: NdofFlag = 1 << 2;
pub const NDOF_SHOULD_PAN: NdofFlag = 1 << 3;
pub const NDOF_SHOULD_ZOOM: NdofFlag = 1 << 4;
pub const NDOF_SHOULD_ROTATE: NdofFlag = 1 << 5;
pub const NDOF_SWAP_YZ_AXIS: NdofFlag = 1 << 7;
pub const NDOF_ROTX_INVERT_AXIS: NdofFlag = 1 << 9;
pub const NDOF_ROTY_INVERT_AXIS: NdofFlag = 1 << 10;
pub const NDOF_ROTZ_INVERT_AXIS: NdofFlag = 1 << 11;
pub const NDOF_PANX_INVERT_AXIS: NdofFlag = 1 << 12;
pub const NDOF_PANY_INVERT_AXIS: NdofFlag = 1 << 13;
pub const NDOF_PANZ_INVERT_AXIS: NdofFlag = 1 << 14;
pub const NDOF_TURNTABLE: NdofFlag = 1 << 15;
pub const NDOF_CAMERA_PAN_ZOOM: NdofFlag = 1 << 16;
pub const NDOF_ORBIT_CENTER_AUTO: NdofFlag = 1 << 17;
pub const NDOF_ORBIT_CENTER_SELECTED: NdofFlag = 1 << 18;
pub const NDOF_SHOW_GUIDE_ORBIT_CENTER: NdofFlag = 1 << 19;
pub const NDOF_FLY_SPEED_AUTO: NdofFlag = 1 << 20;

/// `eNdof_Navigation_Mode`.
pub type NdofNavigationMode = u8;
pub const NDOF_NAVIGATION_MODE_OBJECT: NdofNavigationMode = 0;
pub const NDOF_NAVIGATION_MODE_FLY: NdofNavigationMode = 1;
pub const NDOF_NAVIGATION_MODE_DRONE: NdofNavigationMode = 2;

/// `eMultiSample_Type`.
pub type MultisampleType = i16;
pub const USER_MULTISAMPLE_NONE: MultisampleType = 0;
pub const USER_MULTISAMPLE_2: MultisampleType = 2;
pub const USER_MULTISAMPLE_4: MultisampleType = 4;
pub const USER_MULTISAMPLE_8: MultisampleType = 8;
pub const USER_MULTISAMPLE_16: MultisampleType = 16;

/// `eUserpref_VirtualPixel`.
pub type UserprefVirtualpixel = i32;
pub const VIRTUAL_PIXEL_NATIVE: UserprefVirtualpixel = 0;
pub const VIRTUAL_PIXEL_DOUBLE: UserprefVirtualpixel = 1;

/// `eUserpref_FactorDisplay`.
pub type UserprefFactordisplay = i8;
pub const USER_FACTOR_AS_FACTOR: UserprefFactordisplay = 0;
pub const USER_FACTOR_AS_PERCENTAGE: UserprefFactordisplay = 1;

/// `eUserpref_XrNavigationFlags`.
pub type UserprefXrnavigationflags = i16;
pub const USER_XR_NAV_SNAP_TURN: UserprefXrnavigationflags = 1 << 0;
pub const USER_XR_NAV_INVERT_ROTATION: UserprefXrnavigationflags = 1 << 1;

/// `eUserpref_RenderDisplayType`.
pub type UserprefRenderdisplaytype = i8;
pub const USER_RENDER_DISPLAY_NONE: UserprefRenderdisplaytype = 0;
pub const USER_RENDER_DISPLAY_SCREEN: UserprefRenderdisplaytype = 1;
pub const USER_RENDER_DISPLAY_AREA: UserprefRenderdisplaytype = 2;
pub const USER_RENDER_DISPLAY_WINDOW: UserprefRenderdisplaytype = 3;

/// `eUserpref_TempSpaceDisplayType`.
pub type UserprefTempspacedisplaytype = i8;
pub const USER_TEMP_SPACE_DISPLAY_FULLSCREEN: UserprefTempspacedisplaytype = 0;
pub const USER_TEMP_SPACE_DISPLAY_WINDOW: UserprefTempspacedisplaytype = 1;

/// `eUserpref_EmulateMMBMod`.
pub type UserprefEmulatemmbmod = i8;
pub const USER_EMU_MMB_MOD_ALT: UserprefEmulatemmbmod = 0;
pub const USER_EMU_MMB_MOD_OSKEY: UserprefEmulatemmbmod = 1;

/// `eUserpref_TrackpadScrollDir`.
pub type UserprefTrackpadscrolldir = i8;
pub const USER_TRACKPAD_SCROLL_DIR_TRADITIONAL: UserprefTrackpadscrolldir = 0;
pub const USER_TRACKPAD_SCROLL_DIR_NATURAL: UserprefTrackpadscrolldir = 1;

/// `eUserpref_DiskCacheCompression`.
pub type UserprefDiskcachecompression = i8;
pub const USER_SEQ_DISK_CACHE_COMPRESSION_NONE: UserprefDiskcachecompression = 0;
pub const USER_SEQ_DISK_CACHE_COMPRESSION_LOW: UserprefDiskcachecompression = 1;
pub const USER_SEQ_DISK_CACHE_COMPRESSION_HIGH: UserprefDiskcachecompression = 2;

/// `eUserpref_SeqProxySetup`.
pub type UserprefSeqproxysetup = i16;
pub const USER_SEQ_PROXY_SETUP_MANUAL: UserprefSeqproxysetup = 0;
pub const USER_SEQ_PROXY_SETUP_AUTOMATIC: UserprefSeqproxysetup = 1;

/// `eUserpref_VideoDecodingDevice`.
pub type UserprefVideodecodingdevice = i16;
pub const USER_VIDEO_DECODING_DEVICE_AUTOMATIC: UserprefVideodecodingdevice = 0;
pub const USER_VIDEO_DECODING_DEVICE_VIDEOTOOLBOX: UserprefVideodecodingdevice = 1;
pub const USER_VIDEO_DECODING_DEVICE_D3D11VA: UserprefVideodecodingdevice = 2;
pub const USER_VIDEO_DECODING_DEVICE_CUDA: UserprefVideodecodingdevice = 3;
pub const USER_VIDEO_DECODING_DEVICE_VULKAN: UserprefVideodecodingdevice = 4;

/// `eUserpref_SeqEditorFlags`.
pub type UserprefSeqeditorflags = i32;
pub const USER_SEQ_ED_UNUSED_0: UserprefSeqeditorflags = 1 << 0;
pub const USER_SEQ_ED_CONNECT_STRIPS_BY_DEFAULT: UserprefSeqeditorflags = 1 << 1;
pub const USER_SEQ_ED_CLAMP_STRIPS_BY_DEFAULT: UserprefSeqeditorflags = 1 << 2;

/// `eUserpref_ShaderCompileMethod`.
pub type UserprefShadercompilemethod = i16;
pub const USER_SHADER_COMPILE_THREAD: UserprefShadercompilemethod = 0;
pub const USER_SHADER_COMPILE_SUBPROCESS: UserprefShadercompilemethod = 1;

/// `eUserpref_Language`.
pub type UserprefLanguage = i16;
pub const ULANGUAGE_AUTO: UserprefLanguage = 0;
pub const ULANGUAGE_ENGLISH: UserprefLanguage = 1;

/// `ePathCompare_Flag`.
pub type PathcompareFlag = i8;
pub const USER_PATHCMP_GLOB: PathcompareFlag = 1 << 0;

/// `eUserMenu_Type`.
pub type UsermenuType = i8;
pub const USER_MENU_TYPE_SEP: UsermenuType = 1;
pub const USER_MENU_TYPE_OPERATOR: UsermenuType = 2;
pub const USER_MENU_TYPE_MENU: UsermenuType = 3;
pub const USER_MENU_TYPE_PROP: UsermenuType = 4;

/// `eUserExtensionRepo_Flag`.
pub type UserextensionrepoFlag = u8;
pub const USER_EXTENSION_REPO_FLAG_NO_CACHE: UserextensionrepoFlag = 1 << 0;
pub const USER_EXTENSION_REPO_FLAG_DISABLED: UserextensionrepoFlag = 1 << 1;
pub const USER_EXTENSION_REPO_FLAG_USE_CUSTOM_DIRECTORY: UserextensionrepoFlag = 1 << 2;
pub const USER_EXTENSION_REPO_FLAG_USE_REMOTE_URL: UserextensionrepoFlag = 1 << 3;
pub const USER_EXTENSION_REPO_FLAG_SYNC_ON_STARTUP: UserextensionrepoFlag = 1 << 4;
pub const USER_EXTENSION_REPO_FLAG_USE_ACCESS_TOKEN: UserextensionrepoFlag = 1 << 5;
pub const USER_EXTENSION_REPO_FLAG_ASSET_LIBRARIES_COLLAPSED: UserextensionrepoFlag = 1 << 6;
pub const USER_EXTENSION_REPO_FLAG_USE_ACCESS_TOKEN_ASSET_LIBRARIES: UserextensionrepoFlag = 1 << 7;

/// `eUserExtensionRepo_Source`.
pub type UserextensionrepoSource = u8;
pub const USER_EXTENSION_REPO_SOURCE_USER: UserextensionrepoSource = 0;
pub const USER_EXTENSION_REPO_SOURCE_SYSTEM: UserextensionrepoSource = 1;

/// `eUserPref_Section`.
pub type UserprefSection = i8;
pub const USER_SECTION_INTERFACE: UserprefSection = 0;
pub const USER_SECTION_EDITING: UserprefSection = 1;
pub const USER_SECTION_SAVE_LOAD: UserprefSection = 2;
pub const USER_SECTION_SYSTEM: UserprefSection = 3;
pub const USER_SECTION_THEME: UserprefSection = 4;
pub const USER_SECTION_INPUT: UserprefSection = 5;
pub const USER_SECTION_ADDONS: UserprefSection = 6;
pub const USER_SECTION_LIGHT: UserprefSection = 7;
pub const USER_SECTION_KEYMAP: UserprefSection = 8;
pub const USER_SECTION_WORKSPACE_CONFIG: UserprefSection = 9;
pub const USER_SECTION_WORKSPACE_ADDONS: UserprefSection = 10;
pub const USER_SECTION_WORKSPACE_KEYMAPS: UserprefSection = 11;
pub const USER_SECTION_VIEWPORT: UserprefSection = 12;
pub const USER_SECTION_ANIMATION: UserprefSection = 13;
pub const USER_SECTION_NAVIGATION: UserprefSection = 14;
pub const USER_SECTION_FILE_PATHS: UserprefSection = 15;
pub const USER_SECTION_EXPERIMENTAL: UserprefSection = 16;
pub const USER_SECTION_EXTENSIONS: UserprefSection = 17;
pub const USER_SECTION_DEVELOPER_TOOLS: UserprefSection = 18;
pub const USER_SECTION_ASSETS: UserprefSection = 19;

/// `eUserPref_SpaceData_Flag`.
pub type UserprefSpacedataFlag = i8;
pub const USER_SPACEDATA_INPUT_HIDE_UI_KEYCONFIG: UserprefSpacedataFlag = 1 << 0;
pub const USER_SPACEDATA_ADDONS_SHOW_ONLY_ENABLED: UserprefSpacedataFlag = 1 << 1;

/// `bAddon`.
#[derive(Clone)]
pub struct bAddon {
    /// `bAddon *next`
    pub next: usize,
    /// `bAddon *prev`
    pub prev: usize,
    /// `char module[128]`
    pub module: [u8; 128],
    /// `IDProperty *prop`
    pub prop: usize,
}
impl Default for bAddon {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            module: [0; 128],
            prop: NIL,
        }
    }
}

/// `bPathCompare`.
#[derive(Clone)]
pub struct bPathCompare {
    /// `bPathCompare *next`
    pub next: usize,
    /// `bPathCompare *prev`
    pub prev: usize,
    /// `char path[768]`
    pub path: [u8; 768],
    /// `ePathCompare_Flag flag`
    pub flag: PathcompareFlag,
}
impl Default for bPathCompare {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            path: [0; 768],
            flag: Default::default(),
        }
    }
}

/// `bUserMenu`.
#[derive(Clone)]
pub struct bUserMenu {
    /// `bUserMenu *next`
    pub next: usize,
    /// `bUserMenu *prev`
    pub prev: usize,
    /// `char space_type`
    pub space_type: i8,
    /// `char context[64]`
    pub context: [u8; 64],
    /// `ListBaseT<bUserMenuItem>`
    pub items: ListBase,
}
impl Default for bUserMenu {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            space_type: 0,
            context: [0; 64],
            items: ListBase::default(),
        }
    }
}

/// `bUserMenuItem`.
#[derive(Clone)]
pub struct bUserMenuItem {
    /// `bUserMenuItem *next`
    pub next: usize,
    /// `bUserMenuItem *prev`
    pub prev: usize,
    /// `char ui_name[64]`
    pub ui_name: [u8; 64],
    /// `eUserMenu_Type type`
    pub type_: UsermenuType,
}
impl Default for bUserMenuItem {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            ui_name: [0; 64],
            type_: Default::default(),
        }
    }
}

/// `bUserMenuItem_Op`.
#[derive(Clone)]
pub struct bUserMenuItem_Op {
    pub item: bUserMenuItem,
    /// `char op_idname[64]`
    pub op_idname: [u8; 64],
    /// `IDProperty *prop`
    pub prop: usize,
    /// `char op_prop_enum[64]`
    pub op_prop_enum: [u8; 64],
    /// `char opcontext`
    pub opcontext: i8,
}
impl Default for bUserMenuItem_Op {
    fn default() -> Self {
        Self {
            item: Default::default(),
            op_idname: [0; 64],
            prop: NIL,
            op_prop_enum: [0; 64],
            opcontext: 0,
        }
    }
}

/// `bUserMenuItem_Menu`.
#[derive(Clone)]
pub struct bUserMenuItem_Menu {
    pub item: bUserMenuItem,
    /// `char mt_idname[64]`
    pub mt_idname: [u8; 64],
}
impl Default for bUserMenuItem_Menu {
    fn default() -> Self {
        Self {
            item: Default::default(),
            mt_idname: [0; 64],
        }
    }
}

/// `bUserMenuItem_Prop`.
#[derive(Clone)]
pub struct bUserMenuItem_Prop {
    pub item: bUserMenuItem,
    /// `char context_data_path[256]`
    pub context_data_path: [u8; 256],
    /// `char prop_id[64]`
    pub prop_id: [u8; 64],
    /// `int prop_index`
    pub prop_index: i32,
}
impl Default for bUserMenuItem_Prop {
    fn default() -> Self {
        Self {
            item: Default::default(),
            context_data_path: [0; 256],
            prop_id: [0; 64],
            prop_index: 0,
        }
    }
}

/// `bUserAssetLibrary`.
#[derive(Clone)]
pub struct bUserAssetLibrary {
    /// `bUserAssetLibrary *next`
    pub next: usize,
    /// `bUserAssetLibrary *prev`
    pub prev: usize,
    /// `char name[64]`
    pub name: [u8; 64],
    /// `char dirpath[1024]`
    pub dirpath: [u8; 1024],
    /// `char resolved_dirpath[1024]`
    pub resolved_dirpath: [u8; 1024],
    /// `char remote_url[1024]`
    pub remote_url: [u8; 1024],
    /// `char *auth_token`
    pub auth_token: usize,
    pub uuid: BUuid,
    /// `char *invalid_uuid`
    pub invalid_uuid: usize,
    /// `char extension_id[128]`
    pub extension_id: [u8; 128],
    /// `short import_method`
    pub import_method: i16,
    /// `short flag`
    pub flag: i16,
}
impl Default for bUserAssetLibrary {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            dirpath: [0; 1024],
            resolved_dirpath: [0; 1024],
            remote_url: [0; 1024],
            auth_token: NIL,
            uuid: Default::default(),
            invalid_uuid: NIL,
            extension_id: [0; 128],
            import_method: ASSET_IMPORT_PACK as i16,
            flag: ASSET_LIBRARY_RELATIVE_PATH as i16,
        }
    }
}

/// `bUserExtensionRepo`.
#[derive(Clone)]
pub struct bUserExtensionRepo {
    /// `bUserExtensionRepo *next`
    pub next: usize,
    /// `bUserExtensionRepo *prev`
    pub prev: usize,
    /// `char name[64]`
    pub name: [u8; 64],
    /// `char module[48]`
    pub module: [u8; 48],
    /// `char *access_token`
    pub access_token: usize,
    /// `char custom_dirpath[1024]`
    pub custom_dirpath: [u8; 1024],
    /// `char remote_url[1024]`
    pub remote_url: [u8; 1024],
    /// `eUserExtensionRepo_Flag flag`
    pub flag: UserextensionrepoFlag,
    /// `eUserExtensionRepo_Source source`
    pub source: UserextensionrepoSource,
}
impl Default for bUserExtensionRepo {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            module: [0; 48],
            access_token: NIL,
            custom_dirpath: [0; 1024],
            remote_url: [0; 1024],
            flag: Default::default(),
            source: Default::default(),
        }
    }
}

/// `SolidLight`.
#[derive(Clone)]
pub struct SolidLight {
    /// `int flag`
    pub flag: i32,
    /// `float smooth`
    pub smooth: f32,
    /// `float col[4]`
    pub col: [f32; 4],
    /// `float spec[4]`
    pub spec: [f32; 4],
    /// `float vec[4]`
    pub vec: [f32; 4],
}
impl Default for SolidLight {
    fn default() -> Self {
        Self {
            flag: 0,
            smooth: 0.0,
            col: [0.0; 4],
            spec: [0.0; 4],
            vec: [0.0; 4],
        }
    }
}

/// `WalkNavigation`.
#[derive(Clone)]
pub struct WalkNavigation {
    /// `float mouse_speed`
    pub mouse_speed: f32,
    /// `float walk_speed`
    pub walk_speed: f32,
    /// `float walk_speed_factor`
    pub walk_speed_factor: f32,
    /// `float view_height`
    pub view_height: f32,
    /// `float jump_height`
    pub jump_height: f32,
    /// `float teleport_time`
    pub teleport_time: f32,
    /// `eWalkNavigation_Flag flag`
    pub flag: WalknavigationFlag,
}
impl Default for WalkNavigation {
    fn default() -> Self {
        Self {
            mouse_speed: 1.0,
            walk_speed: 2.5,
            walk_speed_factor: 5.0,
            view_height: 1.6,
            jump_height: 0.4,
            teleport_time: 0.2,
            flag: Default::default(),
        }
    }
}

/// `XrNavigation`.
#[derive(Clone)]
pub struct XrNavigation {
    /// `float vignette_intensity`
    pub vignette_intensity: f32,
    /// `float turn_speed`
    pub turn_speed: f32,
    /// `float turn_amount`
    pub turn_amount: f32,
    /// `eUserpref_XrNavigationFlags flag`
    pub flag: UserprefXrnavigationflags,
}
impl Default for XrNavigation {
    fn default() -> Self {
        Self {
            vignette_intensity: 70.0,
            turn_speed: 60.0f32.to_radians(),
            turn_amount: 30.0f32.to_radians(),
            flag: USER_XR_NAV_SNAP_TURN,
        }
    }
}

/// `UserDef_Runtime`.
#[derive(Clone)]
pub struct UserDef_Runtime {
    /// `char is_dirty`
    pub is_dirty: i8,
}
impl Default for UserDef_Runtime {
    fn default() -> Self {
        Self {
            is_dirty: 0,
        }
    }
}

/// `UserDef_SpaceData`.
#[derive(Clone)]
pub struct UserDef_SpaceData {
    /// `eUserPref_Section section_active`
    pub section_active: UserprefSection,
    /// `eUserPref_SpaceData_Flag flag`
    pub flag: UserprefSpacedataFlag,
}
impl Default for UserDef_SpaceData {
    fn default() -> Self {
        Self {
            section_active: USER_SECTION_INTERFACE,
            flag: Default::default(),
        }
    }
}

/// `UserDef_FileSpaceData`.
#[derive(Clone)]
pub struct UserDef_FileSpaceData {
    /// `eFileDisplayType display_type`
    pub display_type: Filedisplaytype,
    /// `eFileSortType sort_type`
    pub sort_type: Filesorttype,
    /// `int thumbnail_size`
    pub thumbnail_size: i32,
    /// `eFileSel_Params_Flag flag`
    pub flag: FileselParamsFlag,
    /// `uint64_t filter_id`
    pub filter_id: u64,
}
impl Default for UserDef_FileSpaceData {
    fn default() -> Self {
        Self {
            display_type: FILE_VERTICALDISPLAY,
            sort_type: FILE_SORT_ALPHA,
            thumbnail_size: 96,
            flag: FILE_HIDE_DOT,
            filter_id: FILTER_ID_ALL,
        }
    }
}

/// `UserDef_TempWinBounds`.
#[derive(Clone)]
pub struct UserDef_TempWinBounds {
    /// `rctf file`
    pub file: Rctf,
    /// `rctf userpref`
    pub userpref: Rctf,
    /// `rctf image`
    pub image: Rctf,
    /// `rctf graph`
    pub graph: Rctf,
    /// `rctf info`
    pub info: Rctf,
    /// `rctf project`
    pub project: Rctf,
    /// `rctf outliner`
    pub outliner: Rctf,
}
impl Default for UserDef_TempWinBounds {
    fn default() -> Self {
        Self {
            file: Default::default(),
            userpref: Default::default(),
            image: Default::default(),
            graph: Default::default(),
            info: Default::default(),
            project: Default::default(),
            outliner: Default::default(),
        }
    }
}

/// `UserDef_Experimental`.
#[derive(Clone)]
pub struct UserDef_Experimental {
    /// `char use_undo_legacy`
    pub use_undo_legacy: i8,
    /// `char no_override_auto_resync`
    pub no_override_auto_resync: i8,
    /// `char use_cycles_debug`
    pub use_cycles_debug: i8,
    /// `char use_eevee_debug`
    pub use_eevee_debug: i8,
    /// `char show_asset_debug_info`
    pub show_asset_debug_info: i8,
    /// `char no_asset_indexing`
    pub no_asset_indexing: i8,
    /// `char use_viewport_debug`
    pub use_viewport_debug: i8,
    /// `char use_all_linked_data_direct`
    pub use_all_linked_data_direct: i8,
    /// `char use_extensions_debug`
    pub use_extensions_debug: i8,
    /// `char use_recompute_usercount_on_save_debug`
    pub use_recompute_usercount_on_save_debug: i8,
    /// `char write_legacy_blend_file_format`
    pub write_legacy_blend_file_format: i8,
    /// `char no_data_block_packing`
    pub no_data_block_packing: i8,
    /// `char use_paint_debug`
    pub use_paint_debug: i8,
    /// `char use_video_decoding_debug`
    pub use_video_decoding_debug: i8,
    /// `char SANITIZE_AFTER_HERE`
    pub sanitize_after_here: i8,
    /// `char use_new_curves_tools`
    pub use_new_curves_tools: i8,
    /// `char use_extended_asset_browser`
    pub use_extended_asset_browser: i8,
    /// `char use_3d_texture_paint`
    pub use_3d_texture_paint: i8,
    /// `char use_shader_node_previews`
    pub use_shader_node_previews: i8,
    /// `char use_remote_asset_libraries`
    pub use_remote_asset_libraries: i8,
}
impl Default for UserDef_Experimental {
    fn default() -> Self {
        Self {
            use_undo_legacy: 0,
            no_override_auto_resync: 0,
            use_cycles_debug: 0,
            use_eevee_debug: 0,
            show_asset_debug_info: 0,
            no_asset_indexing: 0,
            use_viewport_debug: 0,
            use_all_linked_data_direct: 0,
            use_extensions_debug: 0,
            use_recompute_usercount_on_save_debug: 0,
            write_legacy_blend_file_format: 0,
            no_data_block_packing: 0,
            use_paint_debug: 0,
            use_video_decoding_debug: 0,
            sanitize_after_here: Default::default(),
            use_new_curves_tools: 0,
            use_extended_asset_browser: 0,
            use_3d_texture_paint: 0,
            use_shader_node_previews: 0,
            use_remote_asset_libraries: 1,
        }
    }
}

/// `bUserScriptDirectory`.
#[derive(Clone)]
pub struct bUserScriptDirectory {
    /// `bUserScriptDirectory *next`
    pub next: usize,
    /// `bUserScriptDirectory *prev`
    pub prev: usize,
    /// `char name[64]`
    pub name: [u8; 64],
    /// `char dir_path[768]`
    pub dir_path: [u8; 768],
}
impl Default for bUserScriptDirectory {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            dir_path: [0; 768],
        }
    }
}

/// `bUserAssetShelfSettings`.
#[derive(Clone)]
pub struct bUserAssetShelfSettings {
    /// `bUserAssetShelfSettings *next`
    pub next: usize,
    /// `bUserAssetShelfSettings *prev`
    pub prev: usize,
    /// `char shelf_idname[64]`
    pub shelf_idname: [u8; 64],
    /// `ListBaseT<AssetCatalogPathLink>`
    pub enabled_catalog_paths: ListBase,
}
impl Default for bUserAssetShelfSettings {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            shelf_idname: [0; 64],
            enabled_catalog_paths: ListBase::default(),
        }
    }
}

/// `UserDef`.
#[derive(Clone)]
pub struct UserDef {
    /// `int versionfile`
    pub versionfile: i32,
    /// `int subversionfile`
    pub subversionfile: i32,
    /// `eUserPref_PrefFlag pref_flag`
    pub pref_flag: UserprefPrefflag,
    /// `char savetime`
    pub savetime: i8,
    /// `eUserpref_EmulateMMBMod mouse_emulate_3_button_modifier`
    pub mouse_emulate_3_button_modifier: UserprefEmulatemmbmod,
    /// `eUserpref_TrackpadScrollDir trackpad_scroll_direction`
    pub trackpad_scroll_direction: UserprefTrackpadscrolldir,
    /// `char tempdir[768]`
    pub tempdir: [u8; 768],
    /// `char pythondir_legacy[768]`
    pub pythondir_legacy: [u8; 768],
    /// `char image_editor[1024]`
    pub image_editor: [u8; 1024],
    /// `char text_editor[1024]`
    pub text_editor: [u8; 1024],
    /// `char text_editor_args[256]`
    pub text_editor_args: [u8; 256],
    /// `char anim_player[1024]`
    pub anim_player: [u8; 1024],
    /// `int anim_player_preset`
    pub anim_player_preset: i32,
    /// `short v2d_min_gridsize`
    pub v2d_min_gridsize: i16,
    /// `eTimecodeStyles timecode_style`
    pub timecode_style: Timecodestyles,
    /// `short versions`
    pub versions: i16,
    /// `short dbl_click_time`
    pub dbl_click_time: i16,
    pub asset_access: AssetAccess,
    /// `char border_width`
    pub border_width: i8,
    /// `eUserpref_MiniAxisType mini_axis_type`
    pub mini_axis_type: UserprefMiniaxistype,
    /// `eUserpref_UI_Flag2 uiflag2`
    pub uiflag2: UserprefUiFlag2,
    /// `char date_format`
    pub date_format: i8,
    /// `char time_format`
    pub time_format: i8,
    /// `eUserpref_APP_Flag app_flag`
    pub app_flag: UserprefAppFlag,
    /// `eViewZoom_Style viewzoom`
    pub viewzoom: ViewzoomStyle,
    /// `eUserpref_Language language`
    pub language: UserprefLanguage,
    /// `int mixbufsize`
    pub mixbufsize: i32,
    /// `int audiodevice`
    pub audiodevice: i32,
    /// `int audiorate`
    pub audiorate: i32,
    /// `int audioformat`
    pub audioformat: i32,
    /// `int audiochannels`
    pub audiochannels: i32,
    /// `float ui_scale`
    pub ui_scale: f32,
    /// `int ui_line_width`
    pub ui_line_width: i32,
    /// `int dpi`
    pub dpi: i32,
    /// `float scale_factor`
    pub scale_factor: f32,
    /// `float inv_scale_factor`
    pub inv_scale_factor: f32,
    /// `float pixelsize`
    pub pixelsize: f32,
    /// `eUserpref_VirtualPixel virtual_pixel`
    pub virtual_pixel: UserprefVirtualpixel,
    /// `int scrollback`
    pub scrollback: i32,
    /// `char node_margin`
    pub node_margin: i8,
    /// `char node_preview_res`
    pub node_preview_res: i8,
    /// `short menuthreshold1`
    pub menuthreshold1: i16,
    /// `short menuthreshold2`
    pub menuthreshold2: i16,
    /// `char app_template[64]`
    pub app_template: [u8; 64],
    /// `ListBaseT<bTheme>`
    pub themes: ListBase,
    /// `ListBaseT<uiFont>`
    pub uifonts: ListBase,
    /// `ListBaseT<uiStyle>`
    pub uistyles: ListBase,
    /// `ListBaseT<bAddon>`
    pub addons: ListBase,
    /// `ListBaseT<bPathCompare>`
    pub autoexec_paths: ListBase,
    /// `ListBaseT<bUserScriptDirectory>`
    pub script_directories: ListBase,
    /// `ListBaseT<bUserMenu>`
    pub user_menus: ListBase,
    /// `ListBaseT<bUserAssetLibrary>`
    pub asset_libraries: ListBase,
    /// `ListBaseT<bUserExtensionRepo>`
    pub extension_repos: ListBase,
    /// `ListBaseT<bUserAssetShelfSettings>`
    pub asset_shelves_settings: ListBase,
    /// `char keyconfigstr[64]`
    pub keyconfigstr: [u8; 64],
    /// `short active_extension_repo`
    pub active_extension_repo: i16,
    /// `eUserPref_ExtensionFlag extension_flag`
    pub extension_flag: UserprefExtensionflag,
    /// `uint8_t network_timeout`
    pub network_timeout: u8,
    /// `uint8_t network_connection_limit`
    pub network_connection_limit: u8,
    /// `short active_asset_library`
    pub active_asset_library: i16,
    /// `eUserPref_AssetFlag asset_flag`
    pub asset_flag: UserprefAssetflag,
    /// `short undosteps`
    pub undosteps: i16,
    /// `int undomemory`
    pub undomemory: i32,
    /// `float gpu_viewport_quality`
    pub gpu_viewport_quality: f32,
    /// `short gp_manhattandist`
    pub gp_manhattandist: i16,
    /// `short gp_euclideandist`
    pub gp_euclideandist: i16,
    /// `short gp_eraser`
    pub gp_eraser: i16,
    /// `eGP_UserdefSettings gp_settings`
    pub gp_settings: GpUserdefsettings,
    /// `SolidLight light_param[4]`
    pub light_param: [SolidLight; 4],
    /// `float light_ambient[3]`
    pub light_ambient: [f32; 3],
    /// `eUserpref_Gizmo_Flag gizmo_flag`
    pub gizmo_flag: UserprefGizmoFlag,
    /// `char gizmo_size`
    pub gizmo_size: i8,
    /// `char gizmo_size_navigate_v3d`
    pub gizmo_size_navigate_v3d: i8,
    /// `char noodle_curving`
    pub noodle_curving: i8,
    /// `short edit_studio_light`
    pub edit_studio_light: i16,
    /// `short lookdev_sphere_size`
    pub lookdev_sphere_size: i16,
    /// `short vbotimeout`
    pub vbotimeout: i16,
    /// `short vbocollectrate`
    pub vbocollectrate: i16,
    /// `short textimeout`
    pub textimeout: i16,
    /// `short texcollectrate`
    pub texcollectrate: i16,
    /// `int memcachelimit`
    pub memcachelimit: i32,
    /// `int nodes_stack_limit`
    pub nodes_stack_limit: i32,
    /// `int prefetchframes`
    pub prefetchframes: i32,
    /// `short rvisize`
    pub rvisize: i16,
    /// `short rvibright`
    pub rvibright: i16,
    /// `short recent_files`
    pub recent_files: i16,
    /// `short smooth_viewtx`
    pub smooth_viewtx: i16,
    /// `short glreslimit`
    pub glreslimit: i16,
    /// `eColorPicker_Types color_picker_type`
    pub color_picker_type: ColorpickerTypes,
    /// `eFCurve_Smoothing auto_smoothing_new`
    pub auto_smoothing_new: FCurveSmoothing,
    /// `eBezTriple_Interpolation ipo_new`
    pub ipo_new: BezTripleInterpolation,
    /// `eBezTriple_Handle keyhandles_new`
    pub keyhandles_new: BezTripleHandle,
    /// `eZoomFrame_Mode view_frame_type`
    pub view_frame_type: ZoomframeMode,
    /// `int view_frame_keyframes`
    pub view_frame_keyframes: i32,
    /// `float view_frame_seconds`
    pub view_frame_seconds: f32,
    /// `int gpu_preferred_index`
    pub gpu_preferred_index: i32,
    /// `uint32_t gpu_preferred_vendor_id`
    pub gpu_preferred_vendor_id: u32,
    /// `uint32_t gpu_preferred_device_id`
    pub gpu_preferred_device_id: u32,
    /// `short gpu_shader_workers`
    pub gpu_shader_workers: i16,
    /// `eUserpref_ShaderCompileMethod shader_compilation_method`
    pub shader_compilation_method: UserprefShadercompilemethod,
    /// `eUserPref_GPUBackendType gpu_backend`
    pub gpu_backend: UserprefGpubackendtype,
    /// `short playback_fps_samples`
    pub playback_fps_samples: i16,
    /// `short widget_unit`
    pub widget_unit: i16,
    /// `short anisotropic_filter`
    pub anisotropic_filter: i16,
    /// `eUserpref_TabletAPI tablet_api`
    pub tablet_api: UserprefTabletapi,
    /// `float pressure_threshold_max`
    pub pressure_threshold_max: f32,
    /// `float pressure_softness`
    pub pressure_softness: f32,
    /// `eUserPref_Tablet_Flags tablet_flag`
    pub tablet_flag: UserprefTabletFlags,
    /// `float ndof_translation_sensitivity`
    pub ndof_translation_sensitivity: f32,
    /// `float ndof_rotation_sensitivity`
    pub ndof_rotation_sensitivity: f32,
    /// `float ndof_deadzone`
    pub ndof_deadzone: f32,
    /// `eNdof_Navigation_Mode ndof_navigation_mode`
    pub ndof_navigation_mode: NdofNavigationMode,
    /// `float glalphaclip`
    pub glalphaclip: f32,
    /// `eAutokey_Mode autokey_mode`
    pub autokey_mode: AutokeyMode,
    /// `eUserpref_Anim_Flags animation_flag`
    pub animation_flag: UserprefAnimFlags,
    /// `eText_Draw_Options text_render`
    pub text_render: TextDrawOptions,
    /// `eViewNavigation_Method navigation_mode`
    pub navigation_mode: ViewnavigationMethod,
    /// `float view_rotate_sensitivity_turntable`
    pub view_rotate_sensitivity_turntable: f32,
    /// `float view_rotate_sensitivity_trackball`
    pub view_rotate_sensitivity_trackball: f32,
    pub coba_weight: ColorBand,
    /// `float sculpt_paint_overlay_col[3]`
    pub sculpt_paint_overlay_col: [f32; 3],
    /// `float gpencil_new_layer_col[4]`
    pub gpencil_new_layer_col: [f32; 4],
    /// `char drag_threshold_mouse`
    pub drag_threshold_mouse: i8,
    /// `char drag_threshold_tablet`
    pub drag_threshold_tablet: i8,
    /// `char drag_threshold`
    pub drag_threshold: i8,
    /// `char move_threshold`
    pub move_threshold: i8,
    /// `char font_path_ui[1024]`
    pub font_path_ui: [u8; 1024],
    /// `char font_path_ui_mono[1024]`
    pub font_path_ui_mono: [u8; 1024],
    /// `int compute_device_type`
    pub compute_device_type: i32,
    /// `float fcu_inactive_alpha`
    pub fcu_inactive_alpha: f32,
    /// `short pie_tap_timeout`
    pub pie_tap_timeout: i16,
    /// `short pie_initial_timeout`
    pub pie_initial_timeout: i16,
    /// `short pie_animation_timeout`
    pub pie_animation_timeout: i16,
    /// `short pie_menu_confirm`
    pub pie_menu_confirm: i16,
    /// `short pie_menu_radius`
    pub pie_menu_radius: i16,
    /// `short pie_menu_threshold`
    pub pie_menu_threshold: i16,
    /// `eUserpref_SeqEditorFlags sequencer_editor_flag`
    pub sequencer_editor_flag: UserprefSeqeditorflags,
    /// `eUserpref_FactorDisplay factor_display_type`
    pub factor_display_type: UserprefFactordisplay,
    /// `char viewport_aa`
    pub viewport_aa: i8,
    /// `eUserpref_RenderDisplayType render_display_type`
    pub render_display_type: UserprefRenderdisplaytype,
    /// `eUserpref_TempSpaceDisplayType filebrowser_display_type`
    pub filebrowser_display_type: UserprefTempspacedisplaytype,
    /// `eUserpref_TempSpaceDisplayType preferences_display_type`
    pub preferences_display_type: UserprefTempspacedisplaytype,
    /// `float sequencer_default_strip_length`
    pub sequencer_default_strip_length: f32,
    /// `eUserpref_SeqProxySetup sequencer_proxy_setup`
    pub sequencer_proxy_setup: UserprefSeqproxysetup,
    /// `eUserpref_VideoDecodingDevice video_decoding_device`
    pub video_decoding_device: UserprefVideodecodingdevice,
    /// `float collection_instance_empty_size`
    pub collection_instance_empty_size: f32,
    /// `eTextEdit_Flags text_flag`
    pub text_flag: TexteditFlags,
    /// `eUserpref_Save_Modified_Images save_modified_images`
    pub save_modified_images: UserprefSaveModifiedImages,
    /// `eUserpref_File_Preview_Type file_preview_type`
    pub file_preview_type: UserprefFilePreviewType,
    pub walk_navigation: WalkNavigation,
    pub xr_navigation: XrNavigation,
    pub space_data: UserDef_SpaceData,
    pub file_space_data: UserDef_FileSpaceData,
    pub stored_bounds: UserDef_TempWinBounds,
    pub experimental: UserDef_Experimental,
    pub runtime: UserDef_Runtime,
}
impl Default for UserDef {
    fn default() -> Self {
        Self {
            versionfile: 0,
            subversionfile: 0,
            pref_flag: USER_PREF_FLAG_SAVE | USER_PREF_FLAG_PROJECT_SAVE,
            savetime: 2,
            mouse_emulate_3_button_modifier: Default::default(),
            trackpad_scroll_direction: Default::default(),
            tempdir: [0; 768],
            pythondir_legacy: [0; 768],
            image_editor: [0; 1024],
            text_editor: [0; 1024],
            text_editor_args: [0; 256],
            anim_player: [0; 1024],
            anim_player_preset: 0,
            v2d_min_gridsize: 45,
            timecode_style: USER_TIMECODE_MINIMAL,
            versions: 1,
            dbl_click_time: 350,
            asset_access: 0,
            border_width: 2,
            mini_axis_type: USER_MINI_AXIS_TYPE_GIZMO,
            uiflag2: USER_REGION_OVERLAP,
            date_format: 0,
            time_format: 0,
            app_flag: Default::default(),
            viewzoom: USER_ZOOM_DOLLY,
            language: ULANGUAGE_ENGLISH,
            mixbufsize: 2048,
            audiodevice: 0,
            audiorate: 48000,
            audioformat: 0x24,
            audiochannels: 2,
            ui_scale: 1.0,
            ui_line_width: 0,
            dpi: 0,
            scale_factor: 0.0,
            inv_scale_factor: 0.0,
            pixelsize: 1.0,
            virtual_pixel: Default::default(),
            scrollback: 256,
            node_margin: 40,
            node_preview_res: 120,
            menuthreshold1: 5,
            menuthreshold2: 2,
            app_template: [0; 64],
            themes: ListBase::default(),
            uifonts: ListBase::default(),
            uistyles: ListBase::default(),
            addons: ListBase::default(),
            autoexec_paths: ListBase::default(),
            script_directories: ListBase::default(),
            user_menus: ListBase::default(),
            asset_libraries: ListBase::default(),
            extension_repos: ListBase::default(),
            asset_shelves_settings: ListBase::default(),
            keyconfigstr: [0; 64],
            active_extension_repo: 0,
            extension_flag: Default::default(),
            network_timeout: 10,
            network_connection_limit: 5,
            active_asset_library: 0,
            asset_flag: USER_ASSETS_USE_ONLINE_ESSENTIALS,
            undosteps: 32,
            undomemory: 0,
            gpu_viewport_quality: 0.0,
            gp_manhattandist: 1,
            gp_euclideandist: 2,
            gp_eraser: 25,
            gp_settings: Default::default(),
            light_param: core::array::from_fn(|_| SolidLight::default()),
            light_ambient: [0.0; 3],
            gizmo_flag: USER_GIZMO_DRAW,
            gizmo_size: 75,
            gizmo_size_navigate_v3d: 80,
            noodle_curving: 4,
            edit_studio_light: 0,
            lookdev_sphere_size: 150,
            vbotimeout: 120,
            vbocollectrate: 60,
            textimeout: 120,
            texcollectrate: 60,
            memcachelimit: 4096,
            nodes_stack_limit: 100,
            prefetchframes: 0,
            rvisize: 25,
            rvibright: 8,
            recent_files: 200,
            smooth_viewtx: 200,
            glreslimit: 0,
            color_picker_type: USER_CP_CIRCLE_HSV,
            auto_smoothing_new: FCURVE_SMOOTH_CONT_ACCEL,
            ipo_new: BEZT_IPO_BEZ,
            keyhandles_new: HD_AUTO_ANIM,
            view_frame_type: ZOOM_FRAME_MODE_KEEP_RANGE,
            view_frame_keyframes: 1,
            view_frame_seconds: 0.0,
            gpu_preferred_index: 0,
            gpu_preferred_vendor_id: 0,
            gpu_preferred_device_id: 0,
            gpu_shader_workers: 0,
            shader_compilation_method: USER_SHADER_COMPILE_THREAD,
            gpu_backend: USER_GPU_BACKEND_DEFAULT,
            playback_fps_samples: 8,
            widget_unit: 0,
            anisotropic_filter: 2,
            tablet_api: USER_TABLET_AUTOMATIC,
            pressure_threshold_max: 1.0,
            pressure_softness: 0.0,
            tablet_flag: Default::default(),
            ndof_translation_sensitivity: 4.0,
            ndof_rotation_sensitivity: 4.0,
            ndof_deadzone: 0.0,
            ndof_navigation_mode: Default::default(),
            glalphaclip: 0.004,
            autokey_mode: (AUTOKEY_MODE_NORMAL & !AUTOKEY_ON),
            animation_flag: USER_ANIM_HIGH_QUALITY_DRAWING,
            text_render: Default::default(),
            navigation_mode: VIEW_NAVIGATION_WALK,
            view_rotate_sensitivity_turntable: 0.4f32.to_radians(),
            view_rotate_sensitivity_trackball: 1.0,
            coba_weight: Default::default(),
            sculpt_paint_overlay_col: [0.0; 3],
            gpencil_new_layer_col: [0.0; 4],
            drag_threshold_mouse: 3,
            drag_threshold_tablet: 10,
            drag_threshold: 30,
            move_threshold: 2,
            font_path_ui: [0; 1024],
            font_path_ui_mono: [0; 1024],
            compute_device_type: 0,
            fcu_inactive_alpha: 0.25,
            pie_tap_timeout: 20,
            pie_initial_timeout: 0,
            pie_animation_timeout: 6,
            pie_menu_confirm: 0,
            pie_menu_radius: 100,
            pie_menu_threshold: 12,
            sequencer_editor_flag: USER_SEQ_ED_CONNECT_STRIPS_BY_DEFAULT,
            factor_display_type: USER_FACTOR_AS_FACTOR,
            viewport_aa: 8,
            render_display_type: USER_RENDER_DISPLAY_WINDOW,
            filebrowser_display_type: USER_TEMP_SPACE_DISPLAY_WINDOW,
            preferences_display_type: USER_TEMP_SPACE_DISPLAY_WINDOW,
            sequencer_default_strip_length: 1.0,
            sequencer_proxy_setup: USER_SEQ_PROXY_SETUP_AUTOMATIC,
            video_decoding_device: USER_VIDEO_DECODING_DEVICE_AUTOMATIC,
            collection_instance_empty_size: 1.0,
            text_flag: Default::default(),
            save_modified_images: USER_SAVE_MODIFIED_IMAGES_ASK,
            file_preview_type: USER_FILE_PREVIEW_AUTO,
            walk_navigation: Default::default(),
            xr_navigation: Default::default(),
            space_data: Default::default(),
            file_space_data: Default::default(),
            stored_bounds: Default::default(),
            experimental: Default::default(),
            runtime: Default::default(),
        }
    }
}
