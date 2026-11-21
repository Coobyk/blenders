//! Port of `DNA_space_enums.h` — mechanically converted enum constants.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
/// `eSpace_Link_Flag`.
pub type SpaceLinkFlag = i8;
pub const SPACE_FLAG_TYPE_TEMPORARY: SpaceLinkFlag = 1 << 0;
pub const SPACE_FLAG_TYPE_WAS_ACTIVE: SpaceLinkFlag = 1 << 1;

/// `eSpaceInfo_RptMask`.
pub type SpaceinfoRptmask = i8;
pub const INFO_RPT_DEBUG: SpaceinfoRptmask = 1 << 0;
pub const INFO_RPT_INFO: SpaceinfoRptmask = 1 << 1;
pub const INFO_RPT_OP: SpaceinfoRptmask = 1 << 2;
pub const INFO_RPT_WARN: SpaceinfoRptmask = 1 << 3;
pub const INFO_RPT_ERR: SpaceinfoRptmask = 1 << 4;

/// `eSpaceButtons_Context_Legacy`.
pub type SpacebuttonsContextLegacy = i16;
pub const CONTEXT_SCENE: SpacebuttonsContextLegacy = 0;
pub const CONTEXT_OBJECT: SpacebuttonsContextLegacy = 1;
pub const CONTEXT_SHADING: SpacebuttonsContextLegacy = 3;
pub const CONTEXT_EDITING: SpacebuttonsContextLegacy = 4;

/// `eSpaceButtons_Legacy`.
pub type SpacebuttonsLegacy = i16;
pub const BUTS_LAMP: SpacebuttonsLegacy = 1;
pub const BUTS_MAT: SpacebuttonsLegacy = 2;
pub const BUTS_TEX: SpacebuttonsLegacy = 3;
pub const BUTS_ANIM: SpacebuttonsLegacy = 4;
pub const BUTS_WORLD: SpacebuttonsLegacy = 5;
pub const BUTS_RENDER: SpacebuttonsLegacy = 6;
pub const BUTS_EDIT: SpacebuttonsLegacy = 7;
pub const BUTS_FPAINT: SpacebuttonsLegacy = 9;
pub const BUTS_RADIO: SpacebuttonsLegacy = 10;
pub const BUTS_SCRIPT: SpacebuttonsLegacy = 11;
pub const BUTS_CONSTRAINT: SpacebuttonsLegacy = 13;

/// `eSpaceButtons_Context`.
pub type SpacebuttonsContext = i16;
pub const BCONTEXT_SEPARATOR: SpacebuttonsContext = -1;
pub const BCONTEXT_RENDER: SpacebuttonsContext = 0;
pub const BCONTEXT_SCENE: SpacebuttonsContext = 1;
pub const BCONTEXT_WORLD: SpacebuttonsContext = 2;
pub const BCONTEXT_OBJECT: SpacebuttonsContext = 3;
pub const BCONTEXT_DATA: SpacebuttonsContext = 4;
pub const BCONTEXT_MATERIAL: SpacebuttonsContext = 5;
pub const BCONTEXT_TEXTURE: SpacebuttonsContext = 6;
pub const BCONTEXT_PARTICLE: SpacebuttonsContext = 7;
pub const BCONTEXT_PHYSICS: SpacebuttonsContext = 8;
pub const BCONTEXT_BONE: SpacebuttonsContext = 9;
pub const BCONTEXT_MODIFIER: SpacebuttonsContext = 10;
pub const BCONTEXT_CONSTRAINT: SpacebuttonsContext = 11;
pub const BCONTEXT_BONE_CONSTRAINT: SpacebuttonsContext = 12;
pub const BCONTEXT_VIEW_LAYER: SpacebuttonsContext = 13;
pub const BCONTEXT_TOOL: SpacebuttonsContext = 14;
pub const BCONTEXT_SHADERFX: SpacebuttonsContext = 15;
pub const BCONTEXT_OUTPUT: SpacebuttonsContext = 16;
pub const BCONTEXT_COLLECTION: SpacebuttonsContext = 17;
pub const BCONTEXT_STRIP: SpacebuttonsContext = 18;
pub const BCONTEXT_STRIP_MODIFIER: SpacebuttonsContext = 19;
pub const BCONTEXT_COMPOSITOR: SpacebuttonsContext = 20;
pub const BCONTEXT_TOT: SpacebuttonsContext = BCONTEXT_COMPOSITOR + 1;

/// `eSpaceButtons_Flag`.
pub type SpacebuttonsFlag = i8;
pub const SB_PIN_CONTEXT: SpacebuttonsFlag = 1 << 1;
pub const SB_FLAG_UNUSED_2: SpacebuttonsFlag = 1 << 2;
pub const SB_FLAG_UNUSED_3: SpacebuttonsFlag = 1 << 3;
pub const SB_TEX_USER_LIMITED: SpacebuttonsFlag = 1 << 3;
pub const SB_SHADING_CONTEXT: SpacebuttonsFlag = 1 << 4;

/// `eSpaceButtons_OutlinerSync`.
pub type SpacebuttonsOutlinersync = i8;
pub const PROPERTIES_SYNC_AUTO: SpacebuttonsOutlinersync = 0;
pub const PROPERTIES_SYNC_NEVER: SpacebuttonsOutlinersync = 1;
pub const PROPERTIES_SYNC_ALWAYS: SpacebuttonsOutlinersync = 2;

/// `eSpaceOutliner_Flag`.
pub type SpaceoutlinerFlag = i16;
pub const SO_FLAG_UNUSED_1: SpaceoutlinerFlag = 1 << 2;
pub const SO_FLAG_UNUSED_4: SpaceoutlinerFlag = 1 << 4;
pub const SO_SYNC_SELECT: SpaceoutlinerFlag = 1 << 5;
pub const SO_MODE_COLUMN: SpaceoutlinerFlag = 1 << 6;
pub const SO_SCROLL_TO_ACTIVE: SpaceoutlinerFlag = 1 << 7;
pub const SO_EXPAND_ON_FOCUS: SpaceoutlinerFlag = 1 << 8;
pub const SO_USERS_COLUMN: SpaceoutlinerFlag = 1 << 9;

/// `eSpaceOutliner_SortMethod`.
pub type SpaceoutlinerSortmethod = i16;
pub const SO_SORT_NONE: SpaceoutlinerSortmethod = 0;
pub const SO_SORT_ALPHA: SpaceoutlinerSortmethod = 1;
pub const SO_SORT_CUSTOM: SpaceoutlinerSortmethod = 2;

/// `eSpaceOutliner_Filter`.
pub type SpaceoutlinerFilter = i32;
pub const SO_FILTER_SEARCH: SpaceoutlinerFilter = 1 << 0;
pub const SO_FILTER_CLEARED_1: SpaceoutlinerFilter = 1 << 1;
pub const SO_FILTER_NO_OBJECT: SpaceoutlinerFilter = 1 << 2;
pub const SO_FILTER_NO_OB_CONTENT: SpaceoutlinerFilter = 1 << 3;
pub const SO_FILTER_NO_CHILDREN: SpaceoutlinerFilter = 1 << 4;
pub const SO_FILTER_UNUSED_5: SpaceoutlinerFilter = 1 << 5;
pub const SO_FILTER_SHOW_SYSTEM_OVERRIDES: SpaceoutlinerFilter = SO_FILTER_UNUSED_5;
pub const SO_FILTER_NO_OB_MESH: SpaceoutlinerFilter = 1 << 6;
pub const SO_FILTER_NO_OB_ARMATURE: SpaceoutlinerFilter = 1 << 7;
pub const SO_FILTER_NO_OB_EMPTY: SpaceoutlinerFilter = 1 << 8;
pub const SO_FILTER_NO_OB_LAMP: SpaceoutlinerFilter = 1 << 9;
pub const SO_FILTER_NO_OB_CAMERA: SpaceoutlinerFilter = 1 << 10;
pub const SO_FILTER_NO_OB_OTHERS: SpaceoutlinerFilter = 1 << 11;
pub const SO_FILTER_OB_STATE_SELECTABLE: SpaceoutlinerFilter = 1 << 12;
pub const SO_FILTER_OB_STATE_VISIBLE: SpaceoutlinerFilter = 1 << 13;
pub const SO_FILTER_OB_STATE_INVERSE: SpaceoutlinerFilter = 1 << 14;
pub const SO_FILTER_OB_STATE_SELECTED: SpaceoutlinerFilter = 1 << 15;
pub const SO_FILTER_OB_STATE_ACTIVE: SpaceoutlinerFilter = 1 << 16;
pub const SO_FILTER_NO_COLLECTION: SpaceoutlinerFilter = 1 << 17;
pub const SO_FILTER_NO_VIEW_LAYERS: SpaceoutlinerFilter = 1 << 18;
pub const SO_FILTER_ID_TYPE: SpaceoutlinerFilter = 1 << 19;
pub const SO_FILTER_NO_OB_GREASE_PENCIL: SpaceoutlinerFilter = 1 << 20;
pub const SO_FILTER_NO_OB_DATA: SpaceoutlinerFilter = 1 << 21;
pub const SO_FILTER_NO_OB_ANIMATION: SpaceoutlinerFilter = 1 << 22;
pub const SO_FILTER_NO_OB_CONSTRAINTS: SpaceoutlinerFilter = 1 << 23;
pub const SO_FILTER_NO_OB_SHAPE_KEYS: SpaceoutlinerFilter = 1 << 24;
pub const SO_FILTER_NO_OB_MATERIAL: SpaceoutlinerFilter = 1 << 25;
pub const SO_FILTER_NO_OB_DEFGROUP: SpaceoutlinerFilter = 1 << 26;
pub const SO_FILTER_NO_OB_MODIFIERS: SpaceoutlinerFilter = 1 << 27;
pub const SO_FILTER_NO_ARMATURE_BONE_COLLECTION: SpaceoutlinerFilter = 1 << 28;
pub const SO_FILTER_NO_GREASE_PENCIL_EFFECTS: SpaceoutlinerFilter = 1 << 29;
pub const SO_FILTER_NO_POSE_BONES: SpaceoutlinerFilter = 1 << 30;

/// `eSpaceOutliner_StateFilter`.
pub type SpaceoutlinerStatefilter = i8;
pub const SO_FILTER_OB_ALL: SpaceoutlinerStatefilter = 0;
pub const SO_FILTER_OB_VISIBLE: SpaceoutlinerStatefilter = 1;
pub const SO_FILTER_OB_HIDDEN: SpaceoutlinerStatefilter = 2;
pub const SO_FILTER_OB_SELECTED: SpaceoutlinerStatefilter = 3;
pub const SO_FILTER_OB_ACTIVE: SpaceoutlinerStatefilter = 4;
pub const SO_FILTER_OB_SELECTABLE: SpaceoutlinerStatefilter = 5;

/// `eSpaceOutliner_ShowRestrictFlag`.
pub type SpaceoutlinerShowrestrictflag = i8;
pub const SO_RESTRICT_ENABLE: SpaceoutlinerShowrestrictflag = 1 << 0;
pub const SO_RESTRICT_SELECT: SpaceoutlinerShowrestrictflag = 1 << 1;
pub const SO_RESTRICT_HIDE: SpaceoutlinerShowrestrictflag = 1 << 2;
pub const SO_RESTRICT_VIEWPORT: SpaceoutlinerShowrestrictflag = 1 << 3;
pub const SO_RESTRICT_RENDER: SpaceoutlinerShowrestrictflag = 1 << 4;
pub const SO_RESTRICT_HOLDOUT: SpaceoutlinerShowrestrictflag = 1 << 5;
pub const SO_RESTRICT_INDIRECT_ONLY: SpaceoutlinerShowrestrictflag = 1 << 6;

/// `eSpaceOutliner_Mode`.
pub type SpaceoutlinerMode = i16;
pub const SO_SCENES: SpaceoutlinerMode = 0;
pub const SO_LIBRARIES: SpaceoutlinerMode = 7;
pub const SO_SEQUENCE: SpaceoutlinerMode = 10;
pub const SO_DATA_API: SpaceoutlinerMode = 11;
pub const SO_ID_ORPHANS: SpaceoutlinerMode = 14;
pub const SO_VIEW_LAYER: SpaceoutlinerMode = 15;
pub const SO_OVERRIDES_LIBRARY: SpaceoutlinerMode = 16;

/// `eSpaceOutliner_LibOverrideViewMode`.
pub type SpaceoutlinerLiboverrideviewmode = i16;
pub const SO_LIB_OVERRIDE_VIEW_PROPERTIES: SpaceoutlinerLiboverrideviewmode = 0;
pub const SO_LIB_OVERRIDE_VIEW_HIERARCHIES: SpaceoutlinerLiboverrideviewmode = 1;

/// `eSpaceOutliner_StoreFlag`.
pub type SpaceoutlinerStoreflag = i16;
pub const SO_TREESTORE_CLEANUP: SpaceoutlinerStoreflag = 1 << 0;
pub const SO_TREESTORE_UNUSED_1: SpaceoutlinerStoreflag = 1 << 1;
pub const SO_TREESTORE_REBUILD: SpaceoutlinerStoreflag = 1 << 2;

/// `eSpaceOutliner_Search_Flags`.
pub type SpaceoutlinerSearchFlags = i8;
pub const SO_FIND_CASE_SENSITIVE: SpaceoutlinerSearchFlags = 1 << 0;
pub const SO_FIND_COMPLETE: SpaceoutlinerSearchFlags = 1 << 1;
pub const SO_SEARCH_RECURSIVE: SpaceoutlinerSearchFlags = 1 << 2;

/// `eGraphEdit_Flag`.
pub type GrapheditFlag = i32;
pub const SIPO_NOTRANSKEYCULL: GrapheditFlag = 1 << 1;
pub const SIPO_NOHANDLES: GrapheditFlag = 1 << 2;
pub const SIPO_AUTOLOCK_AXIS: GrapheditFlag = 1 << 3;
pub const SIPO_DRAWTIME: GrapheditFlag = 1 << 4;
pub const SIPO_SLIDERS: GrapheditFlag = 1 << 7;
pub const SIPO_NODRAWCURSOR: GrapheditFlag = 1 << 8;
pub const SIPO_SELVHANDLESONLY: GrapheditFlag = 1 << 9;
pub const SIPO_NOREALTIMEUPDATES: GrapheditFlag = 1 << 11;
pub const SIPO_NORMALIZE: GrapheditFlag = 1 << 14;
pub const SIPO_NORMALIZE_FREEZE: GrapheditFlag = 1 << 15;
pub const SIPO_SHOW_MARKERS: GrapheditFlag = 1 << 16;
pub const SIPO_NO_DRAW_EXTRAPOLATION: GrapheditFlag = 1 << 17;

/// `eGraphEdit_Mode`.
pub type GrapheditMode = i16;
pub const SIPO_MODE_ANIMATION: GrapheditMode = 0;
pub const SIPO_MODE_DRIVERS: GrapheditMode = 1;

/// `eGraphEdit_Runtime_Flag`.
pub type GrapheditRuntimeFlag = i8;
pub const SIPO_RUNTIME_FLAG_NEED_CHAN_SYNC: GrapheditRuntimeFlag = 1 << 0;
pub const SIPO_RUNTIME_FLAG_NEED_CHAN_SYNC_COLOR: GrapheditRuntimeFlag = 1 << 1;
pub const SIPO_RUNTIME_FLAG_TWEAK_HANDLES_LEFT: GrapheditRuntimeFlag = 1 << 2;
pub const SIPO_RUNTIME_FLAG_TWEAK_HANDLES_RIGHT: GrapheditRuntimeFlag = 1 << 3;

/// `eSpaceNla_Flag`.
pub type SpacenlaFlag = i16;
pub const SNLA_FLAG_UNUSED_0: SpacenlaFlag = 1 << 0;
pub const SNLA_FLAG_UNUSED_1: SpacenlaFlag = 1 << 1;
pub const SNLA_DRAWTIME: SpacenlaFlag = 1 << 2;
pub const SNLA_FLAG_UNUSED_3: SpacenlaFlag = 1 << 3;
pub const SNLA_NOSTRIPCURVES: SpacenlaFlag = 1 << 5;
pub const SNLA_NOREALTIMEUPDATES: SpacenlaFlag = 1 << 6;
pub const SNLA_NOLOCALMARKERS: SpacenlaFlag = 1 << 7;
pub const SNLA_SHOW_MARKERS: SpacenlaFlag = 1 << 8;

/// `eSpaceSeq_SequencerPreviewOverlay_Flag`.
pub type SpaceseqSequencerpreviewoverlayFlag = i32;
pub const SEQ_PREVIEW_SHOW_2D_CURSOR: SpaceseqSequencerpreviewoverlayFlag = 1 << 1;
pub const SEQ_PREVIEW_SHOW_OUTLINE_SELECTED: SpaceseqSequencerpreviewoverlayFlag = 1 << 2;
pub const SEQ_PREVIEW_SHOW_SAFE_MARGINS: SpaceseqSequencerpreviewoverlayFlag = 1 << 3;
pub const SEQ_PREVIEW_SHOW_GPENCIL: SpaceseqSequencerpreviewoverlayFlag = 1 << 4;
pub const SEQ_PREVIEW_SHOW_SAFE_CENTER: SpaceseqSequencerpreviewoverlayFlag = 1 << 9;
pub const SEQ_PREVIEW_SHOW_METADATA: SpaceseqSequencerpreviewoverlayFlag = 1 << 10;
pub const SEQ_PREVIEW_SHOW_COMPOSITION_GUIDES: SpaceseqSequencerpreviewoverlayFlag = 1 << 11;

/// `eSpaceSeq_SequencerTimelineOverlay_Flag`.
pub type SpaceseqSequencertimelineoverlayFlag = i32;
pub const SEQ_TIMELINE_SHOW_STRIP_OFFSETS: SpaceseqSequencertimelineoverlayFlag = 1 << 1;
pub const SEQ_TIMELINE_STRIP_END_THUMBNAILS: SpaceseqSequencertimelineoverlayFlag = 1 << 2;
pub const SEQ_TIMELINE_SHOW_STRIP_COLOR_TAG: SpaceseqSequencertimelineoverlayFlag = 1 << 3;
pub const SEQ_TIMELINE_SHOW_STRIP_RETIMING: SpaceseqSequencertimelineoverlayFlag = 1 << 4;
pub const SEQ_TIMELINE_SHOW_FCURVES: SpaceseqSequencertimelineoverlayFlag = 1 << 5;
pub const SEQ_TIMELINE_ALL_WAVEFORMS: SpaceseqSequencertimelineoverlayFlag = 1 << 7;
pub const SEQ_TIMELINE_NO_WAVEFORMS: SpaceseqSequencertimelineoverlayFlag = 1 << 8;
pub const SEQ_TIMELINE_WAVEFORMS_HALF: SpaceseqSequencertimelineoverlayFlag = 1 << 9;
pub const SEQ_TIMELINE_SHOW_STRIP_NAME: SpaceseqSequencertimelineoverlayFlag = 1 << 14;
pub const SEQ_TIMELINE_SHOW_STRIP_SOURCE: SpaceseqSequencertimelineoverlayFlag = 1 << 15;
pub const SEQ_TIMELINE_SHOW_STRIP_DURATION: SpaceseqSequencertimelineoverlayFlag = 1 << 16;
pub const SEQ_TIMELINE_SHOW_GRID: SpaceseqSequencertimelineoverlayFlag = 1 << 18;
pub const SEQ_TIMELINE_CONTINUOUS_THUMBNAILS: SpaceseqSequencertimelineoverlayFlag = 1 << 19;
pub const SEQ_TIMELINE_MIDDLE_THUMBNAILS: SpaceseqSequencertimelineoverlayFlag = 1 << 20;
pub const SEQ_TIMELINE_SHOW_THUMBNAILS: SpaceseqSequencertimelineoverlayFlag = 1 << 21;

/// `eSpaceSeq_SequencerCacheOverlay_Flag`.
pub type SpaceseqSequencercacheoverlayFlag = i32;
pub const SEQ_CACHE_SHOW: SpaceseqSequencercacheoverlayFlag = 1 << 1;
pub const SEQ_CACHE_SHOW_RAW: SpaceseqSequencercacheoverlayFlag = 1 << 2;
pub const SEQ_CACHE_SHOW_FINAL_OUT: SpaceseqSequencercacheoverlayFlag = 1 << 5;

/// `eSpaceSeq_RegionType`.
pub type SpaceseqRegiontype = i16;
pub const SEQ_DRAW_IMG_IMBUF: SpaceseqRegiontype = 1;
pub const SEQ_DRAW_IMG_WAVEFORM: SpaceseqRegiontype = 2;
pub const SEQ_DRAW_IMG_VECTORSCOPE: SpaceseqRegiontype = 3;
pub const SEQ_DRAW_IMG_HISTOGRAM: SpaceseqRegiontype = 4;
pub const SEQ_DRAW_IMG_RGBPARADE: SpaceseqRegiontype = 5;

/// `eSpaceSeq_DrawFlag`.
pub type SpaceseqDrawflag = i8;
pub const SEQ_DRAW_UNUSED_0: SpaceseqDrawflag = 1 << 0;
pub const SEQ_DRAW_UNUSED_1: SpaceseqDrawflag = 1 << 1;
pub const SEQ_DRAW_EDIT_POINT_PREVIEW: SpaceseqDrawflag = 1 << 2;

/// `eSpaceSeq_Flag`.
pub type SpaceseqFlag = i32;
pub const SEQ_DRAWFRAMES: SpaceseqFlag = 1 << 0;
pub const SEQ_MARKER_TRANS: SpaceseqFlag = 1 << 1;
pub const SEQ_DRAW_COLOR_SEPARATED_UNUSED_2: SpaceseqFlag = 1 << 2;
pub const SEQ_CLAMP_VIEW: SpaceseqFlag = 1 << 3;
pub const SPACE_SEQ_DESELECT_STRIP_HANDLE: SpaceseqFlag = 1 << 4;
pub const SPACE_SEQ_FLAG_UNUSED_5: SpaceseqFlag = 1 << 5;
pub const SEQ_USE_ALPHA: SpaceseqFlag = 1 << 6;
pub const SPACE_SEQ_FLAG_UNUSED_10: SpaceseqFlag = 1 << 10;
pub const SEQ_SHOW_MARKERS: SpaceseqFlag = 1 << 11;
pub const SEQ_ZOOM_TO_FIT: SpaceseqFlag = 1 << 12;
pub const SEQ_SHOW_OVERLAY: SpaceseqFlag = 1 << 13;
pub const SPACE_SEQ_FLAG_UNUSED_14: SpaceseqFlag = 1 << 14;
pub const SPACE_SEQ_FLAG_UNUSED_15: SpaceseqFlag = 1 << 15;
pub const SPACE_SEQ_FLAG_UNUSED_16: SpaceseqFlag = 1 << 16;
pub const SEQ_USE_PROXIES: SpaceseqFlag = 1 << 17;
pub const SEQ_SHOW_GRID: SpaceseqFlag = 1 << 18;
pub const SEQ_SHOW_SCRUBBING_REGION: SpaceseqFlag = 1 << 19;

/// `eSpaceSeq_Displays`.
pub type SpaceseqDisplays = i8;
pub const SEQ_VIEW_SEQUENCE: SpaceseqDisplays = 1;
pub const SEQ_VIEW_PREVIEW: SpaceseqDisplays = 2;
pub const SEQ_VIEW_SEQUENCE_PREVIEW: SpaceseqDisplays = 3;

/// `eSpaceSeq_Proxy_RenderSize`.
pub type SpaceseqProxyRendersize = i16;
pub const SEQ_RENDER_SIZE_NONE: SpaceseqProxyRendersize = -1;
pub const SEQ_RENDER_SIZE_SCENE: SpaceseqProxyRendersize = 0;
pub const SEQ_RENDER_SIZE_PROXY_25: SpaceseqProxyRendersize = 25;
pub const SEQ_RENDER_SIZE_PROXY_50: SpaceseqProxyRendersize = 50;
pub const SEQ_RENDER_SIZE_PROXY_75: SpaceseqProxyRendersize = 75;
pub const SEQ_RENDER_SIZE_PROXY_100: SpaceseqProxyRendersize = 99;
pub const SEQ_RENDER_SIZE_FULL_DEPRECATED: SpaceseqProxyRendersize = 100;

/// `eSpaceSeq_Gizmo_Flag`.
pub type SpaceseqGizmoFlag = i8;
pub const SEQ_GIZMO_HIDE: SpaceseqGizmoFlag = 1 << 0;
pub const SEQ_GIZMO_HIDE_NAVIGATE: SpaceseqGizmoFlag = 1 << 1;
pub const SEQ_GIZMO_HIDE_CONTEXT: SpaceseqGizmoFlag = 1 << 2;
pub const SEQ_GIZMO_HIDE_TOOL: SpaceseqGizmoFlag = 1 << 3;

/// `eSpaceSeq_OverlayFrameType`.
pub type SpaceseqOverlayframetype = i8;
pub const SEQ_OVERLAY_FRAME_TYPE_RECT: SpaceseqOverlayframetype = 0;
pub const SEQ_OVERLAY_FRAME_TYPE_REFERENCE: SpaceseqOverlayframetype = 1;
pub const SEQ_OVERLAY_FRAME_TYPE_CURRENT: SpaceseqOverlayframetype = 2;

/// `eFileAssetImportMethod`.
pub type Fileassetimportmethod = i16;
pub const FILE_ASSET_IMPORT_LINK: Fileassetimportmethod = 0;
pub const FILE_ASSET_IMPORT_APPEND: Fileassetimportmethod = 1;
pub const FILE_ASSET_IMPORT_APPEND_REUSE: Fileassetimportmethod = 2;
pub const FILE_ASSET_IMPORT_FOLLOW_PREFS: Fileassetimportmethod = 3;
pub const FILE_ASSET_IMPORT_PACK: Fileassetimportmethod = 4;

/// `eFileAssetImportFlags`.
pub type Fileassetimportflags = i16;
pub const FILE_ASSET_IMPORT_INSTANCE_COLLECTIONS_ON_LINK: Fileassetimportflags = 1 << 0;
pub const FILE_ASSET_IMPORT_INSTANCE_COLLECTIONS_ON_APPEND: Fileassetimportflags = 1 << 1;

/// `eFileSel_AssetParams_Flag`.
pub type FileselAssetparamsFlag = i32;

/// `eFileBrowse_Mode`.
pub type FilebrowseMode = i8;
pub const FILE_BROWSE_MODE_FILES: FilebrowseMode = 0;
pub const FILE_BROWSE_MODE_ASSETS: FilebrowseMode = 1;

/// `eFileDisplayType`.
pub type Filedisplaytype = i16;
pub const FILE_DEFAULTDISPLAY: Filedisplaytype = 0;
pub const FILE_VERTICALDISPLAY: Filedisplaytype = 1;
pub const FILE_HORIZONTALDISPLAY: Filedisplaytype = 2;
pub const FILE_IMGDISPLAY: Filedisplaytype = 3;

/// `eFileSortType`.
pub type Filesorttype = i16;
pub const FILE_SORT_DEFAULT: Filesorttype = 0;
pub const FILE_SORT_ALPHA: Filesorttype = 1;
pub const FILE_SORT_EXTENSION: Filesorttype = 2;
pub const FILE_SORT_TIME: Filesorttype = 3;
pub const FILE_SORT_SIZE: Filesorttype = 4;
pub const FILE_SORT_ASSET_CATALOG: Filesorttype = 5;

/// `eFileTags`.
pub type Filetags = i16;
pub const FILE_TAG_REBUILD_MAIN_FILES: Filetags = 1 << 0;

/// `eFileDetails`.
pub type Filedetails = i8;
pub const FILE_DETAILS_SIZE: Filedetails = 1 << 0;
pub const FILE_DETAILS_DATETIME: Filedetails = 1 << 1;

/// `eFileSelectType`.
pub type Fileselecttype = i16;
pub const FILE_LOADLIB: Fileselecttype = 1;
pub const FILE_MAIN_ASSET: Fileselecttype = 3;
pub const FILE_ASSET_LIBRARY: Fileselecttype = 4;
pub const FILE_ASSET_LIBRARY_ALL: Fileselecttype = 5;
pub const FILE_ASSET_LIBRARY_REMOTE: Fileselecttype = 6;
pub const FILE_ASSET_LIBRARY_ESSENTIALS: Fileselecttype = 7;
pub const FILE_UNIX: Fileselecttype = 8;
pub const FILE_BLENDER: Fileselecttype = 8;
pub const FILE_SPECIAL: Fileselecttype = 9;

/// `eFileSel_Params_Flag`.
pub type FileselParamsFlag = i16;
pub const FILE_PARAMS_FLAG_UNUSED_1: FileselParamsFlag = 1 << 0;
pub const FILE_RELPATH: FileselParamsFlag = 1 << 1;
pub const FILE_LINK: FileselParamsFlag = 1 << 2;
pub const FILE_HIDE_DOT: FileselParamsFlag = 1 << 3;
pub const FILE_AUTOSELECT: FileselParamsFlag = 1 << 4;
pub const FILE_ACTIVE_COLLECTION: FileselParamsFlag = 1 << 5;
pub const FILE_PARAMS_FLAG_UNUSED_2: FileselParamsFlag = 1 << 6;
pub const FILE_DIRSEL_ONLY: FileselParamsFlag = 1 << 7;
pub const FILE_FILTER: FileselParamsFlag = 1 << 8;
pub const FILE_PARAMS_FLAG_UNUSED_3: FileselParamsFlag = 1 << 9;
pub const FILE_PATH_TOKENS_ALLOW: FileselParamsFlag = 1 << 10;
pub const FILE_SORT_INVERT: FileselParamsFlag = 1 << 11;
pub const FILE_HIDE_TOOL_PROPS: FileselParamsFlag = 1 << 12;
pub const FILE_CHECK_EXISTING: FileselParamsFlag = 1 << 13;
pub const FILE_ASSETS_ONLY: FileselParamsFlag = 1 << 14;
pub const FILE_FILTER_ASSET_CATALOG: FileselParamsFlag = 1 << 15;

/// `eFileSel_Params_AssetCatalogVisibility`.
pub type FileselParamsAssetcatalogvisibility = i16;
pub const FILE_SHOW_ASSETS_ALL_CATALOGS: FileselParamsAssetcatalogvisibility = 0;
pub const FILE_SHOW_ASSETS_FROM_CATALOG: FileselParamsAssetcatalogvisibility = FILE_SHOW_ASSETS_ALL_CATALOGS + 1;
pub const FILE_SHOW_ASSETS_WITHOUT_CATALOG: FileselParamsAssetcatalogvisibility = FILE_SHOW_ASSETS_FROM_CATALOG + 1;

/// `eFileSel_Params_RenameFlag`.
pub type FileselParamsRenameflag = i16;
pub const FILE_PARAMS_RENAME_PENDING: FileselParamsRenameflag = 1 << 0;
pub const FILE_PARAMS_RENAME_ACTIVE: FileselParamsRenameflag = 1 << 1;
pub const FILE_PARAMS_RENAME_POSTSCROLL_PENDING: FileselParamsRenameflag = 1 << 2;
pub const FILE_PARAMS_RENAME_POSTSCROLL_ACTIVE: FileselParamsRenameflag = 1 << 3;

/// `eFileSel_File_Types`.
pub type FileselFileTypes = i32;
pub const FILE_TYPE_BLENDER: FileselFileTypes = 1 << 2;
pub const FILE_TYPE_BLENDER_BACKUP: FileselFileTypes = 1 << 3;
pub const FILE_TYPE_IMAGE: FileselFileTypes = 1 << 4;
pub const FILE_TYPE_MOVIE: FileselFileTypes = 1 << 5;
pub const FILE_TYPE_PYSCRIPT: FileselFileTypes = 1 << 6;
pub const FILE_TYPE_FTFONT: FileselFileTypes = 1 << 7;
pub const FILE_TYPE_SOUND: FileselFileTypes = 1 << 8;
pub const FILE_TYPE_TEXT: FileselFileTypes = 1 << 9;
pub const FILE_TYPE_ARCHIVE: FileselFileTypes = 1 << 10;
pub const FILE_TYPE_FOLDER: FileselFileTypes = 1 << 11;
pub const FILE_TYPE_BTX: FileselFileTypes = 1 << 12;
pub const FILE_TYPE_UNUSED_13: FileselFileTypes = 1 << 13;
pub const FILE_TYPE_OPERATOR: FileselFileTypes = 1 << 14;
pub const FILE_TYPE_BUNDLE: FileselFileTypes = 1 << 15;
pub const FILE_TYPE_ALEMBIC: FileselFileTypes = 1 << 16;
pub const FILE_TYPE_OBJECT_IO: FileselFileTypes = 1 << 17;
pub const FILE_TYPE_USD: FileselFileTypes = 1 << 18;
pub const FILE_TYPE_VOLUME: FileselFileTypes = 1 << 19;
pub const FILE_TYPE_ASSET: FileselFileTypes = 1 << 28;
pub const FILE_TYPE_ASSET_ONLINE: FileselFileTypes = 1 << 29;
pub const FILE_TYPE_DIR: FileselFileTypes = 1 << 30;
pub const FILE_TYPE_BLENDERLIB: FileselFileTypes = 1 << 31;

/// `eDirEntry_SelectFlag`.
pub type DirentrySelectflag = i8;
pub const FILE_SEL_HIGHLIGHTED: DirentrySelectflag = 1 << 2;
pub const FILE_SEL_SELECTED: DirentrySelectflag = 1 << 3;
pub const FILE_SEL_EDITING: DirentrySelectflag = 1 << 4;

/// `eFileEntry_Flag`.
pub type FileentryFlag = i16;
pub const FILE_ENTRY_INVALID_PREVIEW: FileentryFlag = 1 << 0;
pub const FILE_ENTRY_NAME_FREE: FileentryFlag = 1 << 1;
pub const FILE_ENTRY_PREVIEW_LOADING: FileentryFlag = 1 << 2;
pub const FILE_ENTRY_BLENDERLIB_NO_PREVIEW: FileentryFlag = 1 << 3;

/// `eSpaceImage_GridShapeSource`.
pub type SpaceimageGridshapesource = i8;
pub const SI_GRID_SHAPE_DYNAMIC: SpaceimageGridshapesource = 0;
pub const SI_GRID_SHAPE_FIXED: SpaceimageGridshapesource = 1;
pub const SI_GRID_SHAPE_PIXEL: SpaceimageGridshapesource = 2;

/// `eSpaceImage_UVDT`.
pub type SpaceimageUvdt = i8;
pub const SI_UVDT_OUTLINE: SpaceimageUvdt = 0;
pub const SI_UVDT_DASH: SpaceimageUvdt = 1;
pub const SI_UVDT_BLACK: SpaceimageUvdt = 2;
pub const SI_UVDT_WHITE: SpaceimageUvdt = 3;

/// `eSpaceImage_UVDT_Stretch`.
pub type SpaceimageUvdtStretch = i8;
pub const SI_UVDT_STRETCH_ANGLE: SpaceimageUvdtStretch = 0;
pub const SI_UVDT_STRETCH_AREA: SpaceimageUvdtStretch = 1;

/// `eSpaceImage_PixelRoundMode`.
pub type SpaceimagePixelroundmode = i8;
pub const SI_PIXEL_ROUND_DISABLED: SpaceimagePixelroundmode = 0;
pub const SI_PIXEL_ROUND_CENTER: SpaceimagePixelroundmode = 1;
pub const SI_PIXEL_ROUND_CORNER: SpaceimagePixelroundmode = 2;

/// `eSpaceImage_Mode`.
pub type SpaceimageMode = i8;
pub const SI_MODE_VIEW: SpaceimageMode = 0;
pub const SI_MODE_PAINT: SpaceimageMode = 1;
pub const SI_MODE_MASK: SpaceimageMode = 2;
pub const SI_MODE_UV: SpaceimageMode = 3;

/// `eSpaceImage_Flag`.
pub type SpaceimageFlag = i32;
pub const SI_FLAG_UNUSED_0: SpaceimageFlag = 1 << 0;
pub const SI_FLAG_UNUSED_1: SpaceimageFlag = 1 << 1;
pub const SI_CLIP_UV: SpaceimageFlag = 1 << 2;
pub const SI_FLAG_UNUSED_3: SpaceimageFlag = 1 << 3;
pub const SI_NO_DRAWFACES: SpaceimageFlag = 1 << 4;
pub const SI_DRAWSHADOW: SpaceimageFlag = 1 << 5;
pub const SI_FLAG_UNUSED_6: SpaceimageFlag = 1 << 6;
pub const SI_FLAG_UNUSED_7: SpaceimageFlag = 1 << 7;
pub const SI_FLAG_UNUSED_8: SpaceimageFlag = 1 << 8;
pub const SI_COORDFLOATS: SpaceimageFlag = 1 << 9;
pub const SI_FLAG_UNUSED_10: SpaceimageFlag = 1 << 10;
pub const SI_LIVE_UNWRAP: SpaceimageFlag = 1 << 11;
pub const SI_USE_ALPHA: SpaceimageFlag = 1 << 12;
pub const SI_SHOW_ALPHA: SpaceimageFlag = 1 << 13;
pub const SI_SHOW_ZBUF: SpaceimageFlag = 1 << 14;
pub const SI_PREVSPACE: SpaceimageFlag = 1 << 15;
pub const SI_FULLWINDOW: SpaceimageFlag = 1 << 16;
pub const SI_FLAG_UNUSED_17: SpaceimageFlag = 1 << 17;
pub const SI_FLAG_UNUSED_18: SpaceimageFlag = 1 << 18;
pub const SI_DRAW_TILE: SpaceimageFlag = 1 << 19;
pub const SI_FLAG_UNUSED_20: SpaceimageFlag = 1 << 20;
pub const SI_DRAW_STRETCH: SpaceimageFlag = 1 << 21;
pub const SI_SHOW_GPENCIL: SpaceimageFlag = 1 << 22;
pub const SI_FLAG_UNUSED_23: SpaceimageFlag = 1 << 23;
pub const SI_FLAG_UNUSED_24: SpaceimageFlag = 1 << 24;
pub const SI_NO_DRAW_TEXPAINT: SpaceimageFlag = 1 << 25;
pub const SI_DRAW_METADATA: SpaceimageFlag = 1 << 26;
pub const SI_SHOW_R: SpaceimageFlag = 1 << 27;
pub const SI_SHOW_G: SpaceimageFlag = 1 << 28;
pub const SI_SHOW_B: SpaceimageFlag = 1 << 29;
pub const SI_GRID_OVER_IMAGE: SpaceimageFlag = 1 << 30;
pub const SI_NO_DRAW_UV_GUIDE: SpaceimageFlag = 1 << 31;

/// `eSpaceImageOverlay_Flag`.
pub type SpaceimageoverlayFlag = i32;
pub const SI_OVERLAY_SHOW_OVERLAYS: SpaceimageoverlayFlag = 1 << 0;
pub const SI_OVERLAY_SHOW_GRID_BACKGROUND: SpaceimageoverlayFlag = 1 << 1;
pub const SI_OVERLAY_DRAW_RENDER_REGION: SpaceimageoverlayFlag = 1 << 2;
pub const SI_OVERLAY_DRAW_TEXT_INFO: SpaceimageoverlayFlag = 1 << 3;

/// `eSpaceImage_Gizmo_Flag`.
pub type SpaceimageGizmoFlag = i8;
pub const SI_GIZMO_HIDE: SpaceimageGizmoFlag = 1 << 0;
pub const SI_GIZMO_HIDE_NAVIGATE: SpaceimageGizmoFlag = 1 << 1;
pub const SI_GIZMO_HIDE_ACTIVE_NODE: SpaceimageGizmoFlag = 1 << 2;

/// `eSpaceText_Flags`.
pub type SpacetextFlags = i16;
pub const ST_SCROLL_SELECT: SpacetextFlags = 1 << 0;
pub const ST_FLAG_UNUSED_4: SpacetextFlags = 1 << 4;
pub const ST_FIND_WRAP: SpacetextFlags = 1 << 5;
pub const ST_FIND_ALL: SpacetextFlags = 1 << 6;
pub const ST_SHOW_MARGIN: SpacetextFlags = 1 << 7;
pub const ST_MATCH_CASE: SpacetextFlags = 1 << 8;
pub const ST_FLAG_UNUSED_9: SpacetextFlags = 1 << 9;

/// `eSpaceNodeOverlay_Flag`.
pub type SpacenodeoverlayFlag = i32;
pub const SN_OVERLAY_SHOW_OVERLAYS: SpacenodeoverlayFlag = 1 << 1;
pub const SN_OVERLAY_SHOW_WIRE_COLORS: SpacenodeoverlayFlag = 1 << 2;
pub const SN_OVERLAY_SHOW_TIMINGS: SpacenodeoverlayFlag = 1 << 3;
pub const SN_OVERLAY_SHOW_PATH: SpacenodeoverlayFlag = 1 << 4;
pub const SN_OVERLAY_SHOW_NAMED_ATTRIBUTES: SpacenodeoverlayFlag = 1 << 5;
pub const SN_OVERLAY_SHOW_PREVIEWS: SpacenodeoverlayFlag = 1 << 6;
pub const SN_OVERLAY_SHOW_REROUTE_AUTO_LABELS: SpacenodeoverlayFlag = 1 << 7;
pub const SN_OVERLAY_SHOW_RENDER_REGION: SpacenodeoverlayFlag = 1 << 8;
pub const SN_OVERLAY_SHOW_TEXT_INFO: SpacenodeoverlayFlag = 1 << 9;

/// `eSpaceNodeOverlay_preview_shape`.
pub type SpacenodeoverlayPreviewShape = i32;
pub const SN_OVERLAY_PREVIEW_FLAT: SpacenodeoverlayPreviewShape = 0;
pub const SN_OVERLAY_PREVIEW_3D: SpacenodeoverlayPreviewShape = 1;

/// `eSpaceNode_Flag`.
pub type SpacenodeFlag = i16;
pub const SNODE_BACKDRAW: SpacenodeFlag = 1 << 1;
pub const SNODE_SHOW_GPENCIL: SpacenodeFlag = 1 << 2;
pub const SNODE_USE_ALPHA: SpacenodeFlag = 1 << 3;
pub const SNODE_SHOW_ALPHA: SpacenodeFlag = 1 << 4;
pub const SNODE_SHOW_R: SpacenodeFlag = 1 << 7;
pub const SNODE_SHOW_G: SpacenodeFlag = 1 << 8;
pub const SNODE_SHOW_B: SpacenodeFlag = 1 << 9;
pub const SNODE_FLAG_UNUSED_5: SpacenodeFlag = 1 << 5;
pub const SNODE_FLAG_UNUSED_6: SpacenodeFlag = 1 << 6;
pub const SNODE_FLAG_UNUSED_10: SpacenodeFlag = 1 << 10;
pub const SNODE_FLAG_UNUSED_11: SpacenodeFlag = 1 << 11;
pub const SNODE_PIN: SpacenodeFlag = 1 << 12;
pub const SNODE_FLAG_UNUSED_12: SpacenodeFlag = 1 << 13;

/// `eSpaceNode_Gizmo_Flag`.
pub type SpacenodeGizmoFlag = i8;
pub const SNODE_GIZMO_HIDE: SpacenodeGizmoFlag = 1 << 0;
pub const SNODE_GIZMO_HIDE_ACTIVE_NODE: SpacenodeGizmoFlag = 1 << 1;

/// `eSpaceNode_TexFrom`.
pub type SpacenodeTexfrom = i16;
pub const SNODE_TEX_WORLD: SpacenodeTexfrom = 1;
pub const SNODE_TEX_BRUSH: SpacenodeTexfrom = 2;
pub const SNODE_TEX_LINESTYLE: SpacenodeTexfrom = 3;

/// `eSpaceNode_ShaderFrom`.
pub type SpacenodeShaderfrom = i8;
pub const SNODE_SHADER_OBJECT: SpacenodeShaderfrom = 0;
pub const SNODE_SHADER_WORLD: SpacenodeShaderfrom = 1;
pub const SNODE_SHADER_LINESTYLE: SpacenodeShaderfrom = 2;

/// `SpaceNodeGeometryNodesType`.
pub type Spacenodegeometrynodestype = i8;
pub const SNODE_GEOMETRY_MODIFIER: Spacenodegeometrynodestype = 0;
pub const SNODE_GEOMETRY_TOOL: Spacenodegeometrynodestype = 1;

/// `SpaceNodeCompositorNodesType`.
pub type Spacenodecompositornodestype = i8;
pub const SNODE_COMPOSITOR_SCENE: Spacenodecompositornodestype = 0;
pub const SNODE_COMPOSITOR_SEQUENCER: Spacenodecompositornodestype = 1;

/// `eSpaceNode_InsertOffsetDir`.
pub type SpacenodeInsertoffsetdir = i8;
pub const SNODE_INSERTOFS_DIR_RIGHT: SpacenodeInsertoffsetdir = 0;
pub const SNODE_INSERTOFS_DIR_LEFT: SpacenodeInsertoffsetdir = 1;

/// `eConsoleLine_Type`.
pub type ConsolelineType = i32;
pub const CONSOLE_LINE_OUTPUT: ConsolelineType = 0;
pub const CONSOLE_LINE_INPUT: ConsolelineType = 1;
pub const CONSOLE_LINE_INFO: ConsolelineType = 2;
pub const CONSOLE_LINE_ERROR: ConsolelineType = 3;

/// `eSpaceClipOverlay_Flag`.
pub type SpaceclipoverlayFlag = i32;
pub const SC_SHOW_OVERLAYS: SpaceclipoverlayFlag = 1 << 0;
pub const SC_SHOW_CURSOR: SpaceclipoverlayFlag = 1 << 1;

/// `eSpaceClip_Flag`.
pub type SpaceclipFlag = i32;
pub const SC_SHOW_MARKER_PATTERN: SpaceclipFlag = 1 << 0;
pub const SC_SHOW_MARKER_SEARCH: SpaceclipFlag = 1 << 1;
pub const SC_LOCK_SELECTION: SpaceclipFlag = 1 << 2;
pub const SC_SHOW_TINY_MARKER: SpaceclipFlag = 1 << 3;
pub const SC_SHOW_TRACK_PATH: SpaceclipFlag = 1 << 4;
pub const SC_SHOW_BUNDLES: SpaceclipFlag = 1 << 5;
pub const SC_MUTE_FOOTAGE: SpaceclipFlag = 1 << 6;
pub const SC_HIDE_DISABLED: SpaceclipFlag = 1 << 7;
pub const SC_SHOW_NAMES: SpaceclipFlag = 1 << 8;
pub const SC_SHOW_GRID: SpaceclipFlag = 1 << 9;
pub const SC_SHOW_STABLE: SpaceclipFlag = 1 << 10;
pub const SC_MANUAL_CALIBRATION: SpaceclipFlag = 1 << 11;
pub const SC_SHOW_ANNOTATION: SpaceclipFlag = 1 << 12;
pub const SC_SHOW_FILTERS: SpaceclipFlag = 1 << 13;
pub const SC_SHOW_GRAPH_FRAMES: SpaceclipFlag = 1 << 14;
pub const SC_SHOW_GRAPH_TRACKS_MOTION: SpaceclipFlag = 1 << 15;
pub const SC_LOCK_TIMECURSOR: SpaceclipFlag = 1 << 17;
pub const SC_SHOW_SECONDS: SpaceclipFlag = 1 << 18;
pub const SC_SHOW_GRAPH_SEL_ONLY: SpaceclipFlag = 1 << 19;
pub const SC_SHOW_GRAPH_HIDDEN: SpaceclipFlag = 1 << 20;
pub const SC_SHOW_GRAPH_TRACKS_ERROR: SpaceclipFlag = 1 << 21;
pub const SC_SHOW_METADATA: SpaceclipFlag = 1 << 22;

/// `eSpaceClip_Mode`.
pub type SpaceclipMode = i16;
pub const SC_MODE_TRACKING: SpaceclipMode = 0;
pub const SC_MODE_MASKEDIT: SpaceclipMode = 3;

/// `eSpaceClip_View`.
pub type SpaceclipView = i16;
pub const SC_VIEW_CLIP: SpaceclipView = 0;
pub const SC_VIEW_GRAPH: SpaceclipView = 1;
pub const SC_VIEW_DOPESHEET: SpaceclipView = 2;

/// `eSpaceClip_GPencil_Source`.
pub type SpaceclipGpencilSource = i16;
pub const SC_GPENCIL_SRC_CLIP: SpaceclipGpencilSource = 0;
pub const SC_GPENCIL_SRC_TRACK: SpaceclipGpencilSource = 1;

/// `eSpaceClip_Gizmo_Flag`.
pub type SpaceclipGizmoFlag = i8;
pub const SCLIP_GIZMO_HIDE: SpaceclipGizmoFlag = 1 << 0;
pub const SCLIP_GIZMO_HIDE_NAVIGATE: SpaceclipGizmoFlag = 1 << 1;

/// `eSpaceSpreadsheet_Flag`.
pub type SpacespreadsheetFlag = i32;
pub const SPREADSHEET_FLAG_PINNED: SpacespreadsheetFlag = 1 << 0;
pub const SPREADSHEET_FLAG_CONTEXT_PATH_COLLAPSED_LEGACY: SpacespreadsheetFlag = 1 << 1;
pub const SPREADSHEET_FLAG_SHOW_INTERNAL_ATTRIBUTES: SpacespreadsheetFlag = 1 << 2;

/// `eSpaceSpreadsheet_FilterFlag`.
pub type SpacespreadsheetFilterflag = u8;
pub const SPREADSHEET_FILTER_SELECTED_ONLY: SpacespreadsheetFilterflag = 1 << 0;
pub const SPREADSHEET_FILTER_ENABLE: SpacespreadsheetFilterflag = 1 << 1;

/// `eSpaceSpreadsheet_RowFilterFlag`.
pub type SpacespreadsheetRowfilterflag = u8;
pub const SPREADSHEET_ROW_FILTER_UI_EXPAND: SpacespreadsheetRowfilterflag = 1 << 0;
pub const SPREADSHEET_ROW_FILTER_BOOL_VALUE: SpacespreadsheetRowfilterflag = 1 << 1;
pub const SPREADSHEET_ROW_FILTER_ENABLED: SpacespreadsheetRowfilterflag = 1 << 2;

/// `eSpreadsheetFilterOperation`.
pub type Spreadsheetfilteroperation = u8;
pub const SPREADSHEET_ROW_FILTER_EQUAL: Spreadsheetfilteroperation = 0;
pub const SPREADSHEET_ROW_FILTER_GREATER: Spreadsheetfilteroperation = 1;
pub const SPREADSHEET_ROW_FILTER_LESS: Spreadsheetfilteroperation = 2;

/// `eSpaceSpreadsheet_ObjectEvalState`.
pub type SpacespreadsheetObjectevalstate = u8;
pub const SPREADSHEET_OBJECT_EVAL_STATE_EVALUATED: SpacespreadsheetObjectevalstate = 0;
pub const SPREADSHEET_OBJECT_EVAL_STATE_ORIGINAL: SpacespreadsheetObjectevalstate = 1;
pub const SPREADSHEET_OBJECT_EVAL_STATE_VIEWER_NODE: SpacespreadsheetObjectevalstate = 2;

/// `eSpaceSpreadsheet_ContextType`.
pub type SpacespreadsheetContexttype = i32;
pub const SPREADSHEET_CONTEXT_OBJECT: SpacespreadsheetContexttype = 0;
pub const SPREADSHEET_CONTEXT_MODIFIER: SpacespreadsheetContexttype = 1;
pub const SPREADSHEET_CONTEXT_NODE: SpacespreadsheetContexttype = 2;

/// `eSpreadsheetColumnValueType`.
pub type Spreadsheetcolumnvaluetype = i32;
pub const SPREADSHEET_VALUE_TYPE_UNKNOWN: Spreadsheetcolumnvaluetype = -1;
pub const SPREADSHEET_VALUE_TYPE_BOOL: Spreadsheetcolumnvaluetype = 0;
pub const SPREADSHEET_VALUE_TYPE_INT32: Spreadsheetcolumnvaluetype = 1;
pub const SPREADSHEET_VALUE_TYPE_FLOAT: Spreadsheetcolumnvaluetype = 2;
pub const SPREADSHEET_VALUE_TYPE_FLOAT2: Spreadsheetcolumnvaluetype = 3;
pub const SPREADSHEET_VALUE_TYPE_FLOAT3: Spreadsheetcolumnvaluetype = 4;
pub const SPREADSHEET_VALUE_TYPE_COLOR: Spreadsheetcolumnvaluetype = 5;
pub const SPREADSHEET_VALUE_TYPE_INSTANCES: Spreadsheetcolumnvaluetype = 6;
pub const SPREADSHEET_VALUE_TYPE_STRING: Spreadsheetcolumnvaluetype = 7;
pub const SPREADSHEET_VALUE_TYPE_BYTE_COLOR: Spreadsheetcolumnvaluetype = 8;
pub const SPREADSHEET_VALUE_TYPE_INT8: Spreadsheetcolumnvaluetype = 9;
pub const SPREADSHEET_VALUE_TYPE_INT32_2D: Spreadsheetcolumnvaluetype = 10;
pub const SPREADSHEET_VALUE_TYPE_QUATERNION: Spreadsheetcolumnvaluetype = 11;
pub const SPREADSHEET_VALUE_TYPE_FLOAT4X4: Spreadsheetcolumnvaluetype = 12;
pub const SPREADSHEET_VALUE_TYPE_BUNDLE_ITEM: Spreadsheetcolumnvaluetype = 13;
pub const SPREADSHEET_VALUE_TYPE_INT64: Spreadsheetcolumnvaluetype = 14;
pub const SPREADSHEET_VALUE_TYPE_INT32_3D: Spreadsheetcolumnvaluetype = 15;
pub const SPREADSHEET_VALUE_TYPE_FLOAT4: Spreadsheetcolumnvaluetype = 16;

/// `eSpreadsheetColumnFlag`.
pub type Spreadsheetcolumnflag = i32;
pub const SPREADSHEET_COLUMN_FLAG_UNAVAILABLE: Spreadsheetcolumnflag = 1 << 0;

/// `eSpreadsheetTableIDType`.
pub type Spreadsheettableidtype = i32;
pub const SPREADSHEET_TABLE_ID_TYPE_GEOMETRY: Spreadsheettableidtype = 0;

/// `eSpreadsheetTableFlag`.
pub type Spreadsheettableflag = i32;
pub const SPREADSHEET_TABLE_FLAG_MANUALLY_EDITED: Spreadsheettableflag = 1 << 0;

/// `eSpace_Type`.
pub type SpaceType = i8;
pub const SPACE_EMPTY: SpaceType = 0;
pub const SPACE_VIEW3D: SpaceType = 1;
pub const SPACE_GRAPH: SpaceType = 2;
pub const SPACE_OUTLINER: SpaceType = 3;
pub const SPACE_PROPERTIES: SpaceType = 4;
pub const SPACE_FILE: SpaceType = 5;
pub const SPACE_IMAGE: SpaceType = 6;
pub const SPACE_INFO: SpaceType = 7;
pub const SPACE_SEQ: SpaceType = 8;
pub const SPACE_TEXT: SpaceType = 9;
pub const SPACE_IMASEL: SpaceType = 10;
pub const SPACE_SOUND: SpaceType = 11;
pub const SPACE_ACTION: SpaceType = 12;
pub const SPACE_NLA: SpaceType = 13;
pub const SPACE_SCRIPT: SpaceType = 14;
pub const SPACE_TIME: SpaceType = 15;
pub const SPACE_NODE: SpaceType = 16;
pub const SPACE_LOGIC: SpaceType = 17;
pub const SPACE_CONSOLE: SpaceType = 18;
pub const SPACE_USERPREF: SpaceType = 19;
pub const SPACE_CLIP: SpaceType = 20;
pub const SPACE_TOPBAR: SpaceType = 21;
pub const SPACE_STATUSBAR: SpaceType = 22;
pub const SPACE_SPREADSHEET: SpaceType = 23;
pub const SPACE_PROJECT: SpaceType = 24;

