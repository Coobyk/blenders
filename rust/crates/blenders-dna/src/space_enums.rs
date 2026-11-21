//! Port of `DNA_space_enums.h` — mechanically converted enum constants.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
/// `eGraphEdit_Runtime_Flag`.
pub type GrapheditRuntimeFlag = i8;
pub const SIPO_RUNTIME_FLAG_NEED_CHAN_SYNC: GrapheditRuntimeFlag = 1 << 0;
pub const SIPO_RUNTIME_FLAG_NEED_CHAN_SYNC_COLOR: GrapheditRuntimeFlag = 1 << 1;
pub const SIPO_RUNTIME_FLAG_TWEAK_HANDLES_LEFT: GrapheditRuntimeFlag = 1 << 2;
pub const SIPO_RUNTIME_FLAG_TWEAK_HANDLES_RIGHT: GrapheditRuntimeFlag = 1 << 3;

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

/// `eFileSelectType`.
pub type Fileselecttype = i16;
pub const FILE_LOADLIB: Fileselecttype = 1;
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
pub const FILE_TYPE_DIR: FileselFileTypes = 1 << 30;
pub const FILE_TYPE_BLENDERLIB: FileselFileTypes = 1 << 31;

/// `eSpaceImage_GridShapeSource`.
pub type SpaceimageGridshapesource = i8;
pub const SI_GRID_SHAPE_DYNAMIC: SpaceimageGridshapesource = 0;
pub const SI_GRID_SHAPE_FIXED: SpaceimageGridshapesource = 1;
pub const SI_GRID_SHAPE_PIXEL: SpaceimageGridshapesource = 2;

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
pub const SPREADSHEET_TABLE_FLAG_MANUALLY_EDITED: Spreadsheettableidtype = 1 << 0;

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

