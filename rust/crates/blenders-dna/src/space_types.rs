//! Port of `DNA_space_types.h` — editor SpaceLink structs (outliner/node/image/view3d...).
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::color_types::ColorManagedViewSettings;
use crate::id::{ID, NIL};
use crate::screen_types::{ARegion, Panel};
use crate::vec_types::{Rctf, Rcti};
use crate::asset_types::{AssetAccess, AssetLibraryReference, ASSET_ACCESS_ONLINE_AND_OFFLINE};
use crate::image_types::ImageUser;
use crate::viewer_path_types::{ViewerPath};
use crate::view2d_types::{View2D};
use crate::color_types::{Histogram, Scopes};
use crate::camera_types::{Compositionguideflags, MovieClipUser};
use crate::space_enums::{Fileassetimportflags, Fileassetimportmethod, FilebrowseMode, Filedetails, Filedisplaytype, FileentryFlag, FileselAssetparamsFlag, FileselParamsAssetcatalogvisibility, FileselParamsFlag, FileselParamsRenameflag, Fileselecttype, Filesorttype, Filetags, GrapheditFlag, GrapheditMode, SpaceLinkFlag, SpacebuttonsFlag, SpacebuttonsOutlinersync, SpaceclipGizmoFlag, SpaceclipGpencilSource, SpaceclipMode, SpaceclipView, SpaceimageFlag, SpaceimageGizmoFlag, SpaceimageGridshapesource, SpaceimageMode, SpaceimagePixelroundmode, SpaceimageUvdt, SpaceimageUvdtStretch, SpaceimageoverlayFlag, SpacenlaFlag, SpacenodeFlag, SpacenodeGizmoFlag, SpacenodeInsertoffsetdir, SpacenodeShaderfrom, SpacenodeTexfrom, SpacenodeoverlayFlag, SpacenodeoverlayPreviewShape, SpaceoutlinerFilter, SpaceoutlinerFlag, SpaceoutlinerLiboverrideviewmode, SpaceoutlinerMode, SpaceoutlinerSearchFlags, SpaceoutlinerShowrestrictflag, SpaceoutlinerSortmethod, SpaceoutlinerStatefilter, SpaceoutlinerStoreflag, SpaceseqDisplays, SpaceseqDrawflag, SpaceseqFlag, SpaceseqGizmoFlag, SpaceseqOverlayframetype, SpaceseqProxyRendersize, SpaceseqRegiontype, SpaceseqSequencercacheoverlayFlag, SpaceseqSequencerpreviewoverlayFlag, SpaceseqSequencertimelineoverlayFlag, SpacetextFlags};
use crate::movieclip_types::{MovieClipScopes};
use crate::mask_types::{MaskDrawFlag, MaskDrawType, MaskOverlayMode};
use crate::mask_types::*;
use crate::small_types::BUuid;
use crate::space_enums::*;
use crate::view3d_types::{View3DAround, V3D_AROUND_CENTER_MEDIAN};
use blenders_bli::listbase::ListBase;
/// `SpreadsheetClosureInputOutput`.
pub type Spreadsheetclosureinputoutput = i8;
pub const SPREADSHEET_CLOSURE_NONE: Spreadsheetclosureinputoutput = 0;
pub const SPREADSHEET_CLOSURE_INPUT: Spreadsheetclosureinputoutput = 1;
pub const SPREADSHEET_CLOSURE_OUTPUT: Spreadsheetclosureinputoutput = 2;

/// `SpreadsheetGeometryItemType`.
pub type Spreadsheetgeometryitemtype = u8;
pub const SPREADSHEET_GEOMETRY_ITEM_TYPE_DOMAIN: Spreadsheetgeometryitemtype = 0;
pub const SPREADSHEET_GEOMETRY_ITEM_TYPE_BUNDLE: Spreadsheetgeometryitemtype = 1;

/// `SpaceLink`.
#[derive(Clone)]
pub struct SpaceLink {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `eSpace_Link_Flag link_flag`
    pub link_flag: SpaceLinkFlag,
}
impl Default for SpaceLink {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: Default::default(),
        }
    }
}

/// `SpaceInfo`.
#[derive(Clone)]
pub struct SpaceInfo {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `eSpace_Link_Flag link_flag`
    pub link_flag: SpaceLinkFlag,
    /// `char rpt_mask`
    pub rpt_mask: i8,
}
impl Default for SpaceInfo {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: Default::default(),
            rpt_mask: 0,
        }
    }
}

/// `SpaceProperties`.
#[derive(Clone)]
pub struct SpaceProperties {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `eSpace_Link_Flag link_flag`
    pub link_flag: SpaceLinkFlag,
    pub v2d: View2D,
    /// `short space_subtype`
    pub space_subtype: i16,
    /// `uint32_t visible_tabs`
    pub visible_tabs: u32,
    /// `short preview`
    pub preview: i16,
    /// `eSpaceButtons_Flag flag`
    pub flag: SpacebuttonsFlag,
    /// `eSpaceButtons_OutlinerSync outliner_sync`
    pub outliner_sync: SpacebuttonsOutlinersync,
    /// `void *path`
    pub path: usize,
    /// `int pathflag`
    pub pathflag: i32,
    /// `int dataicon`
    pub dataicon: i32,
    /// `ID *pinid`
    pub pinid: usize,
    /// `void *texuser`
    pub texuser: usize,
    /// `SpaceProperties_Runtime *runtime`
    pub runtime: usize,
}
impl Default for SpaceProperties {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: Default::default(),
            v2d: Default::default(),
            space_subtype: 0,
            visible_tabs: 0,
            preview: 0,
            flag: Default::default(),
            outliner_sync: PROPERTIES_SYNC_AUTO,
            path: NIL,
            pathflag: 0,
            dataicon: 0,
            pinid: NIL,
            texuser: NIL,
            runtime: NIL,
        }
    }
}

/// `SpaceOutliner`.
#[derive(Clone)]
pub struct SpaceOutliner {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `eSpace_Link_Flag link_flag`
    pub link_flag: SpaceLinkFlag,
    pub v2d: View2D,
    /// `BLI_mempool *treestore`
    pub treestore: usize,
    /// `char search_string[64]`
    pub search_string: [u8; 64],
    /// `eSpaceOutliner_Flag flag`
    pub flag: SpaceoutlinerFlag,
    /// `eSpaceOutliner_Mode outlinevis`
    pub outlinevis: SpaceoutlinerMode,
    /// `eSpaceOutliner_LibOverrideViewMode lib_override_view_mode`
    pub lib_override_view_mode: SpaceoutlinerLiboverrideviewmode,
    /// `eSpaceOutliner_StoreFlag storeflag`
    pub storeflag: SpaceoutlinerStoreflag,
    /// `eSpaceOutliner_Search_Flags search_flags`
    pub search_flags: SpaceoutlinerSearchFlags,
    /// `char sync_select_dirty`
    pub sync_select_dirty: i8,
    /// `eSpaceOutliner_Filter filter`
    pub filter: SpaceoutlinerFilter,
    /// `eSpaceOutliner_StateFilter filter_state`
    pub filter_state: SpaceoutlinerStatefilter,
    /// `eSpaceOutliner_ShowRestrictFlag show_restrict_flags`
    pub show_restrict_flags: SpaceoutlinerShowrestrictflag,
    /// `short filter_id_type`
    pub filter_id_type: i16,
    /// `eSpaceOutliner_SortMethod sort_method`
    pub sort_method: SpaceoutlinerSortmethod,
    /// `ed::outliner::SpaceOutliner_Runtime *runtime`
    pub runtime: usize,
}
impl Default for SpaceOutliner {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: Default::default(),
            v2d: Default::default(),
            treestore: NIL,
            search_string: [0; 64],
            flag: Default::default(),
            outlinevis: SO_SCENES,
            lib_override_view_mode: SO_LIB_OVERRIDE_VIEW_PROPERTIES,
            storeflag: Default::default(),
            search_flags: Default::default(),
            sync_select_dirty: 0,
            filter: Default::default(),
            filter_state: SO_FILTER_OB_ALL,
            show_restrict_flags: Default::default(),
            filter_id_type: 0,
            sort_method: SO_SORT_CUSTOM,
            runtime: NIL,
        }
    }
}

/// `SpaceGraph_Runtime`.
#[derive(Clone)]
pub struct SpaceGraph_Runtime {
    /// `char flag`
    pub flag: i8,
    /// `ListBaseT<FCurve>`
    pub ghost_curves: ListBase,
}
impl Default for SpaceGraph_Runtime {
    fn default() -> Self {
        Self {
            flag: 0,
            ghost_curves: ListBase::default(),
        }
    }
}

/// `SpaceGraph`.
#[derive(Clone)]
pub struct SpaceGraph {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `eSpace_Link_Flag link_flag`
    pub link_flag: SpaceLinkFlag,
    pub v2d: View2D,
    /// `bDopeSheet *ads`
    pub ads: usize,
    /// `eGraphEdit_Mode mode`
    pub mode: GrapheditMode,
    /// `short autosnap`
    pub autosnap: i16,
    /// `eGraphEdit_Flag flag`
    pub flag: GrapheditFlag,
    /// `float cursorTime`
    pub cursor_time: f32,
    /// `float cursorVal`
    pub cursor_val: f32,
    /// `int around`
    pub around: i32,
    /// `uint16_t local_view_bit`
    pub local_view_bit: u16,
    /// `rctf local_view_visible_region_before`
    pub local_view_visible_region_before: Rctf,
    pub runtime: SpaceGraph_Runtime,
}
impl Default for SpaceGraph {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: Default::default(),
            v2d: Default::default(),
            ads: NIL,
            mode: SIPO_MODE_ANIMATION,
            autosnap: 0,
            flag: Default::default(),
            cursor_time: 0.0,
            cursor_val: 0.0,
            around: 0,
            local_view_bit: 0,
            local_view_visible_region_before: Default::default(),
            runtime: Default::default(),
        }
    }
}

/// `SpaceNla`.
#[derive(Clone)]
pub struct SpaceNla {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `eSpace_Link_Flag link_flag`
    pub link_flag: SpaceLinkFlag,
    /// `short autosnap`
    pub autosnap: i16,
    /// `eSpaceNla_Flag flag`
    pub flag: SpacenlaFlag,
    /// `bDopeSheet *ads`
    pub ads: usize,
    pub v2d: View2D,
}
impl Default for SpaceNla {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: Default::default(),
            autosnap: 0,
            flag: Default::default(),
            ads: NIL,
            v2d: Default::default(),
        }
    }
}

/// `SequencerPreviewOverlay`.
#[derive(Clone)]
pub struct SequencerPreviewOverlay {
    /// `eSpaceSeq_SequencerPreviewOverlay_Flag flag`
    pub flag: SpaceseqSequencerpreviewoverlayFlag,
    /// `eCompositionGuideFlags composition_guide_flags`
    pub composition_guide_flags: Compositionguideflags,
    /// `float composition_guide_color[4]`
    pub composition_guide_color: [f32; 4],
}
impl Default for SequencerPreviewOverlay {
    fn default() -> Self {
        Self {
            flag: Default::default(),
            composition_guide_flags: Default::default(),
            composition_guide_color: [0.0; 4],
        }
    }
}

/// `SequencerTimelineOverlay`.
#[derive(Clone)]
pub struct SequencerTimelineOverlay {
    /// `eSpaceSeq_SequencerTimelineOverlay_Flag flag`
    pub flag: SpaceseqSequencertimelineoverlayFlag,
}
impl Default for SequencerTimelineOverlay {
    fn default() -> Self {
        Self {
            flag: Default::default(),
        }
    }
}

/// `SequencerCacheOverlay`.
#[derive(Clone)]
pub struct SequencerCacheOverlay {
    /// `eSpaceSeq_SequencerCacheOverlay_Flag flag`
    pub flag: SpaceseqSequencercacheoverlayFlag,
}
impl Default for SequencerCacheOverlay {
    fn default() -> Self {
        Self {
            flag: Default::default(),
        }
    }
}

/// `SpaceSeq`.
#[derive(Clone)]
pub struct SpaceSeq {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `eSpace_Link_Flag link_flag`
    pub link_flag: SpaceLinkFlag,
    pub v2d: View2D,
    /// `float xof`
    pub xof: f32,
    /// `float yof`
    pub yof: f32,
    /// `eSpaceSeq_RegionType mainb`
    pub mainb: SpaceseqRegiontype,
    /// `eSpaceSeq_Proxy_RenderSize render_size`
    pub render_size: SpaceseqProxyRendersize,
    /// `short chanshown`
    pub chanshown: i16,
    /// `short zebra`
    pub zebra: i16,
    /// `eSpaceSeq_Flag flag`
    pub flag: SpaceseqFlag,
    /// `float zoom`
    pub zoom: f32,
    /// `eSpaceSeq_Displays view`
    pub view: SpaceseqDisplays,
    /// `eSpaceSeq_OverlayFrameType overlay_frame_type`
    pub overlay_frame_type: SpaceseqOverlayframetype,
    /// `eSpaceSeq_DrawFlag draw_flag`
    pub draw_flag: SpaceseqDrawflag,
    /// `eSpaceSeq_Gizmo_Flag gizmo_flag`
    pub gizmo_flag: SpaceseqGizmoFlag,
    /// `float cursor[2]`
    pub cursor: [f32; 2],
    /// `bGPdata *gpd`
    pub gpd: usize,
    pub preview_overlay: SequencerPreviewOverlay,
    pub timeline_overlay: SequencerTimelineOverlay,
    pub cache_overlay: SequencerCacheOverlay,
    /// `char multiview_eye`
    pub multiview_eye: i8,
    /// `ed::vse::SpaceSeq_Runtime *runtime`
    pub runtime: usize,
}
impl Default for SpaceSeq {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: Default::default(),
            v2d: Default::default(),
            xof: 0.0,
            yof: 0.0,
            mainb: Default::default(),
            render_size: SEQ_RENDER_SIZE_SCENE,
            chanshown: 0,
            zebra: 0,
            flag: Default::default(),
            zoom: 0.0,
            view: SEQ_VIEW_SEQUENCE,
            overlay_frame_type: SEQ_OVERLAY_FRAME_TYPE_RECT,
            draw_flag: Default::default(),
            gizmo_flag: Default::default(),
            cursor: [0.0; 2],
            gpd: NIL,
            preview_overlay: Default::default(),
            timeline_overlay: Default::default(),
            cache_overlay: Default::default(),
            multiview_eye: 0,
            runtime: NIL,
        }
    }
}

/// `MaskSpaceInfo`.
#[derive(Clone)]
pub struct MaskSpaceInfo {
    /// `Mask *mask`
    pub mask: usize,
    pub draw_flag: MaskDrawFlag,
    pub draw_type: MaskDrawType,
    pub overlay_mode: MaskOverlayMode,
    /// `float blend_factor`
    pub blend_factor: f32,
}
impl Default for MaskSpaceInfo {
    fn default() -> Self {
        Self {
            mask: NIL,
            draw_flag: MASK_DRAWFLAG_SPLINE,
            draw_type: MASK_DT_OUTLINE,
            overlay_mode: MASK_OVERLAY_ALPHACHANNEL,
            blend_factor: 0.7,
        }
    }
}

/// `FileSelectParams`.
#[derive(Clone)]
pub struct FileSelectParams {
    /// `char title[96]`
    pub title: [u8; 96],
    /// `char dir[1282]`
    pub dir: [u8; 1282],
    /// `char file[256]`
    pub file: [u8; 256],
    /// `char renamefile[256]`
    pub renamefile: [u8; 256],
    /// `eFileSel_Params_RenameFlag rename_flag`
    pub rename_flag: FileselParamsRenameflag,
    /// `void *_pad3`
    pub pad3: usize,
    /// `char filter_glob[256]`
    pub filter_glob: [u8; 256],
    /// `char filter_search[64]`
    pub filter_search: [u8; 64],
    /// `uint64_t filter_id`
    pub filter_id: u64,
    /// `int active_file`
    pub active_file: i32,
    /// `int highlight_file`
    pub highlight_file: i32,
    /// `int sel_first`
    pub sel_first: i32,
    /// `int sel_last`
    pub sel_last: i32,
    /// `unsigned_short thumbnail_size`
    pub thumbnail_size: u16,
    /// `unsigned_short list_thumbnail_size`
    pub list_thumbnail_size: u16,
    /// `unsigned_short list_column_size`
    pub list_column_size: u16,
    /// `eFileSelectType type`
    pub type_: Fileselecttype,
    /// `eFileSel_Params_Flag flag`
    pub flag: FileselParamsFlag,
    /// `eFileSortType sort`
    pub sort: Filesorttype,
    /// `eFileDisplayType display`
    pub display: Filedisplaytype,
    /// `eFileDetails details_flags`
    pub details_flags: Filedetails,
    /// `int filter`
    pub filter: i32,
    /// `short recursion_level`
    pub recursion_level: i16,
}
impl Default for FileSelectParams {
    fn default() -> Self {
        Self {
            title: [0; 96],
            dir: [0; 1282],
            file: [0; 256],
            renamefile: [0; 256],
            rename_flag: Default::default(),
            pad3: NIL,
            filter_glob: [0; 256],
            filter_search: [0; 64],
            filter_id: 0,
            active_file: 0,
            highlight_file: 0,
            sel_first: 0,
            sel_last: 0,
            thumbnail_size: 0,
            list_thumbnail_size: 0,
            list_column_size: 0,
            type_: Default::default(),
            flag: Default::default(),
            sort: FILE_SORT_DEFAULT,
            display: FILE_DEFAULTDISPLAY,
            details_flags: Default::default(),
            filter: 0,
            recursion_level: 0,
        }
    }
}

/// `FileAssetSelectParams`.
#[derive(Clone)]
pub struct FileAssetSelectParams {
    pub base_params: FileSelectParams,
    pub asset_library_ref: AssetLibraryReference,
    pub asset_access: AssetAccess,
    /// `eFileSel_Params_AssetCatalogVisibility asset_catalog_visibility`
    pub asset_catalog_visibility: FileselParamsAssetcatalogvisibility,
    pub catalog_id: BUuid,
    /// `eFileAssetImportMethod import_method`
    pub import_method: Fileassetimportmethod,
    /// `eFileAssetImportFlags import_flags`
    pub import_flags: Fileassetimportflags,
    /// `eFileSel_AssetParams_Flag asset_flags`
    pub asset_flags: FileselAssetparamsFlag,
}
impl Default for FileAssetSelectParams {
    fn default() -> Self {
        Self {
            base_params: Default::default(),
            asset_library_ref: Default::default(),
            asset_access: ASSET_ACCESS_ONLINE_AND_OFFLINE,
            asset_catalog_visibility: FILE_SHOW_ASSETS_ALL_CATALOGS,
            catalog_id: Default::default(),
            import_method: FILE_ASSET_IMPORT_LINK,
            import_flags: Default::default(),
            asset_flags: Default::default(),
        }
    }
}

/// `FileFolderHistory`.
#[derive(Clone)]
pub struct FileFolderHistory {
    /// `FileFolderLists *next`
    pub next: usize,
    /// `FileFolderLists *prev`
    pub prev: usize,
    /// `eFileBrowse_Mode browse_mode`
    pub browse_mode: FilebrowseMode,
}
impl Default for FileFolderHistory {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            browse_mode: FILE_BROWSE_MODE_FILES,
        }
    }
}

/// `SpaceFile`.
#[derive(Clone)]
pub struct SpaceFile {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `eSpace_Link_Flag link_flag`
    pub link_flag: SpaceLinkFlag,
    /// `eFileBrowse_Mode browse_mode`
    pub browse_mode: FilebrowseMode,
    /// `eFileTags tags`
    pub tags: Filetags,
    /// `int scroll_offset`
    pub scroll_offset: i32,
    /// `FileSelectParams *params`
    pub params: usize,
    /// `FileAssetSelectParams *asset_params`
    pub asset_params: usize,
    /// `void *_pad2`
    pub pad2: usize,
    /// `FileList *files`
    pub files: usize,
    /// `ListBaseT<FileFolderHistory>`
    pub folder_histories: ListBase,
    /// `wmOperator *op`
    pub op: usize,
    /// `wmTimer *smoothscroll_timer`
    pub smoothscroll_timer: usize,
    /// `wmTimer *previews_timer`
    pub previews_timer: usize,
    /// `FileLayout *layout`
    pub layout: usize,
    /// `short recentnr`
    pub recentnr: i16,
    /// `short bookmarknr`
    pub bookmarknr: i16,
    /// `short systemnr`
    pub systemnr: i16,
    /// `short system_bookmarknr`
    pub system_bookmarknr: i16,
    /// `SpaceFile_Runtime *runtime`
    pub runtime: usize,
}
impl Default for SpaceFile {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: Default::default(),
            browse_mode: FILE_BROWSE_MODE_FILES,
            tags: Default::default(),
            scroll_offset: 0,
            params: NIL,
            asset_params: NIL,
            pad2: NIL,
            files: NIL,
            folder_histories: ListBase::default(),
            op: NIL,
            smoothscroll_timer: NIL,
            previews_timer: NIL,
            layout: NIL,
            recentnr: 0,
            bookmarknr: 0,
            systemnr: 0,
            system_bookmarknr: 0,
            runtime: NIL,
        }
    }
}

/// `FileDirEntry`.
#[derive(Clone)]
pub struct FileDirEntry {
    /// `FileDirEntry *next`
    pub next: usize,
    /// `FileDirEntry *prev`
    pub prev: usize,
    /// `uint32_t uid`
    pub uid: u32,
    /// `uint64_t size`
    pub size: u64,
    /// `int64_t time`
    pub time: i64,
    /// `char datetime_str[40]`
    pub datetime_str: [u8; 40],
    /// `int typeflag`
    pub typeflag: i32,
    /// `int blentype`
    pub blentype: i32,
    /// `char *relpath`
    pub relpath: usize,
    /// `char *redirection_path`
    pub redirection_path: usize,
    /// `ID *id`
    pub id: usize,
    /// `asset_system::AssetRepresentation *asset`
    pub asset: usize,
    /// `int preview_icon_id`
    pub preview_icon_id: i32,
    /// `eFileEntry_Flag flags`
    pub flags: FileentryFlag,
    /// `int attributes`
    pub attributes: i32,
}
impl Default for FileDirEntry {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            uid: 0,
            size: 0,
            time: 0,
            datetime_str: [0; 40],
            typeflag: 0,
            blentype: 0,
            relpath: NIL,
            redirection_path: NIL,
            id: NIL,
            asset: NIL,
            preview_icon_id: 0,
            flags: Default::default(),
            attributes: 0,
        }
    }
}

/// `FileDirEntryArr`.
#[derive(Clone)]
pub struct FileDirEntryArr {
    /// `int entries_num`
    pub entries_num: i32,
    /// `int entries_filtered_num`
    pub entries_filtered_num: i32,
    /// `char root[1024]`
    pub root: [u8; 1024],
}
impl Default for FileDirEntryArr {
    fn default() -> Self {
        Self {
            entries_num: 0,
            entries_filtered_num: 0,
            root: [0; 1024],
        }
    }
}

/// `SpaceImageOverlay`.
#[derive(Clone)]
pub struct SpaceImageOverlay {
    /// `eSpaceImageOverlay_Flag flag`
    pub flag: SpaceimageoverlayFlag,
    /// `float passepartout_alpha`
    pub passepartout_alpha: f32,
}
impl Default for SpaceImageOverlay {
    fn default() -> Self {
        Self {
            flag: Default::default(),
            passepartout_alpha: 0.0,
        }
    }
}

/// `SpaceImage`.
#[derive(Clone)]
pub struct SpaceImage {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `eSpace_Link_Flag link_flag`
    pub link_flag: SpaceLinkFlag,
    /// `Image *image`
    pub image: usize,
    pub iuser: ImageUser,
    pub scopes: Scopes,
    pub sample_line_hist: Histogram,
    /// `bGPdata *gpd`
    pub gpd: usize,
    /// `float cursor[2]`
    pub cursor: [f32; 2],
    /// `float xof`
    pub xof: f32,
    /// `float yof`
    pub yof: f32,
    /// `float zoom`
    pub zoom: f32,
    /// `float centx`
    pub centx: f32,
    /// `float centy`
    pub centy: f32,
    /// `eSpaceImage_Mode mode`
    pub mode: SpaceimageMode,
    /// `eSpaceImage_Mode mode_prev`
    pub mode_prev: SpaceimageMode,
    /// `char pin`
    pub pin: i8,
    /// `eSpaceImage_PixelRoundMode pixel_round_mode`
    pub pixel_round_mode: SpaceimagePixelroundmode,
    /// `char lock`
    pub lock: i8,
    /// `eSpaceImage_UVDT dt_uv`
    pub dt_uv: SpaceimageUvdt,
    /// `eSpaceImage_UVDT_Stretch dt_uvstretch`
    pub dt_uvstretch: SpaceimageUvdtStretch,
    /// `char around`
    pub around: i8,
    /// `eSpaceImage_Gizmo_Flag gizmo_flag`
    pub gizmo_flag: SpaceimageGizmoFlag,
    /// `eSpaceImage_GridShapeSource grid_shape_source`
    pub grid_shape_source: SpaceimageGridshapesource,
    /// `eSpaceImage_Flag flag`
    pub flag: SpaceimageFlag,
    /// `float uv_opacity`
    pub uv_opacity: f32,
    /// `float uv_face_opacity`
    pub uv_face_opacity: f32,
    /// `float uv_edge_opacity`
    pub uv_edge_opacity: f32,
    /// `float stretch_opacity`
    pub stretch_opacity: f32,
    /// `int tile_grid_shape[2]`
    pub tile_grid_shape: [i32; 2],
    /// `int custom_grid_subdiv[2]`
    pub custom_grid_subdiv: [i32; 2],
    pub mask_info: MaskSpaceInfo,
    pub overlay: SpaceImageOverlay,
}
impl Default for SpaceImage {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: Default::default(),
            image: NIL,
            iuser: Default::default(),
            scopes: Default::default(),
            sample_line_hist: Default::default(),
            gpd: NIL,
            cursor: [0.0; 2],
            xof: 0.0,
            yof: 0.0,
            zoom: 0.0,
            centx: 0.0,
            centy: 0.0,
            mode: SI_MODE_VIEW,
            mode_prev: SI_MODE_VIEW,
            pin: 0,
            pixel_round_mode: SI_PIXEL_ROUND_DISABLED,
            lock: 0,
            dt_uv: SI_UVDT_OUTLINE,
            dt_uvstretch: SI_UVDT_STRETCH_ANGLE,
            around: 0,
            gizmo_flag: Default::default(),
            grid_shape_source: SI_GRID_SHAPE_DYNAMIC,
            flag: Default::default(),
            uv_opacity: 0.0,
            uv_face_opacity: 0.0,
            uv_edge_opacity: 0.0,
            stretch_opacity: 0.0,
            tile_grid_shape: [0; 2],
            custom_grid_subdiv: [0; 2],
            mask_info: Default::default(),
            overlay: Default::default(),
        }
    }
}

/// `SpaceText`.
#[derive(Clone)]
pub struct SpaceText {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `eSpace_Link_Flag link_flag`
    pub link_flag: SpaceLinkFlag,
    /// `Text *text`
    pub text: usize,
    /// `int top`
    pub top: i32,
    /// `int left`
    pub left: i32,
    /// `eSpaceText_Flags flags`
    pub flags: SpacetextFlags,
    /// `short line_height`
    pub line_height: i16,
    /// `int tabnumber`
    pub tabnumber: i32,
    /// `char wordwrap`
    pub wordwrap: i8,
    /// `char doplugins`
    pub doplugins: i8,
    /// `char showlinenrs`
    pub showlinenrs: i8,
    /// `char showsyntax`
    pub showsyntax: i8,
    /// `char line_hlight`
    pub line_hlight: i8,
    /// `char overwrite`
    pub overwrite: i8,
    /// `char live_edit`
    pub live_edit: i8,
    /// `char findstr[256]`
    pub findstr: [u8; 256],
    /// `char replacestr[256]`
    pub replacestr: [u8; 256],
    /// `short margin_column`
    pub margin_column: i16,
    /// `ed::text::SpaceText_Runtime *runtime`
    pub runtime: usize,
}
impl Default for SpaceText {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: Default::default(),
            text: NIL,
            top: 0,
            left: 0,
            flags: Default::default(),
            line_height: 0,
            tabnumber: 0,
            wordwrap: 0,
            doplugins: 0,
            showlinenrs: 0,
            showsyntax: 0,
            line_hlight: 0,
            overwrite: 0,
            live_edit: 0,
            findstr: [0; 256],
            replacestr: [0; 256],
            margin_column: 0,
            runtime: NIL,
        }
    }
}

/// `Script`.
#[derive(Clone)]
pub struct Script {
    pub id: ID,
    /// `void *py_draw`
    pub py_draw: usize,
    /// `void *py_event`
    pub py_event: usize,
    /// `void *py_button`
    pub py_button: usize,
    /// `void *py_browsercallback`
    pub py_browsercallback: usize,
    /// `void *py_globaldict`
    pub py_globaldict: usize,
    /// `int flags`
    pub flags: i32,
    /// `int lastspace`
    pub lastspace: i32,
    /// `char scriptname[1024]`
    pub scriptname: [u8; 1024],
    /// `char scriptarg[256]`
    pub scriptarg: [u8; 256],
}
impl Default for Script {
    fn default() -> Self {
        Self {
            id: Default::default(),
            py_draw: NIL,
            py_event: NIL,
            py_button: NIL,
            py_browsercallback: NIL,
            py_globaldict: NIL,
            flags: 0,
            lastspace: 0,
            scriptname: [0; 1024],
            scriptarg: [0; 256],
        }
    }
}

/// `SpaceScript`.
#[derive(Clone)]
pub struct SpaceScript {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `char link_flag`
    pub link_flag: i8,
    /// `Script *script`
    pub script: usize,
    /// `short flags`
    pub flags: i16,
    /// `short menunr`
    pub menunr: i16,
    /// `void *but_refs`
    pub but_refs: usize,
}
impl Default for SpaceScript {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: 0,
            script: NIL,
            flags: 0,
            menunr: 0,
            but_refs: NIL,
        }
    }
}

/// `bNodeTreePath`.
#[derive(Clone)]
pub struct bNodeTreePath {
    /// `bNodeTreePath *next`
    pub next: usize,
    /// `bNodeTreePath *prev`
    pub prev: usize,
    /// `bNodeTree *nodetree`
    pub nodetree: usize,
    pub parent_key: bNodeInstanceKey,
    /// `float view_center[2]`
    pub view_center: [f32; 2],
    /// `float view_width`
    pub view_width: f32,
    /// `char node_name[64]`
    pub node_name: [u8; 64],
    /// `char display_name[64]`
    pub display_name: [u8; 64],
}
impl Default for bNodeTreePath {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            nodetree: NIL,
            parent_key: Default::default(),
            view_center: [0.0; 2],
            view_width: 0.0,
            node_name: [0; 64],
            display_name: [0; 64],
        }
    }
}

/// `SpaceNodeOverlay`.
#[derive(Clone)]
pub struct SpaceNodeOverlay {
    /// `eSpaceNodeOverlay_Flag flag`
    pub flag: SpacenodeoverlayFlag,
    /// `eSpaceNodeOverlay_preview_shape preview_shape`
    pub preview_shape: SpacenodeoverlayPreviewShape,
    /// `float passepartout_alpha`
    pub passepartout_alpha: f32,
}
impl Default for SpaceNodeOverlay {
    fn default() -> Self {
        Self {
            flag: Default::default(),
            preview_shape: SN_OVERLAY_PREVIEW_FLAT,
            passepartout_alpha: 0.0,
        }
    }
}

/// `SpaceNode`.
#[derive(Clone)]
pub struct SpaceNode {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `eSpace_Link_Flag link_flag`
    pub link_flag: SpaceLinkFlag,
    pub v2d: View2D,
    /// `ID *id`
    pub id: usize,
    /// `ID *from`
    pub from: usize,
    /// `eSpaceNode_Flag flag`
    pub flag: SpacenodeFlag,
    /// `eSpaceNode_InsertOffsetDir insert_ofs_dir`
    pub insert_ofs_dir: SpacenodeInsertoffsetdir,
    /// `float xof`
    pub xof: f32,
    /// `float yof`
    pub yof: f32,
    /// `float zoom`
    pub zoom: f32,
    /// `ListBaseT<bNodeTreePath>`
    pub treepath: ListBase,
    /// `bNodeTree *edittree`
    pub edittree: usize,
    /// `bNodeTree *nodetree`
    pub nodetree: usize,
    /// `char tree_idname[64]`
    pub tree_idname: [u8; 64],
    /// `int treetype`
    pub treetype: i32,
    /// `eSpaceNode_TexFrom texfrom`
    pub texfrom: SpacenodeTexfrom,
    /// `eSpaceNode_ShaderFrom shaderfrom`
    pub shaderfrom: SpacenodeShaderfrom,
    /// `char node_tree_sub_type`
    pub node_tree_sub_type: i8,
    /// `bNodeTree *selected_node_group`
    pub selected_node_group: usize,
    /// `bGPdata *gpd`
    pub gpd: usize,
    /// `eSpaceNode_Gizmo_Flag gizmo_flag`
    pub gizmo_flag: SpacenodeGizmoFlag,
    pub overlay: SpaceNodeOverlay,
    /// `ed::space_node::SpaceNode_Runtime *runtime`
    pub runtime: usize,
}
impl Default for SpaceNode {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: Default::default(),
            v2d: Default::default(),
            id: NIL,
            from: NIL,
            flag: Default::default(),
            insert_ofs_dir: SNODE_INSERTOFS_DIR_RIGHT,
            xof: 0.0,
            yof: 0.0,
            zoom: 0.0,
            treepath: ListBase::default(),
            edittree: NIL,
            nodetree: NIL,
            tree_idname: [0; 64],
            treetype: 0,
            texfrom: SNODE_TEX_WORLD,
            shaderfrom: SNODE_SHADER_OBJECT,
            node_tree_sub_type: 0,
            selected_node_group: NIL,
            gpd: NIL,
            gizmo_flag: Default::default(),
            overlay: Default::default(),
            runtime: NIL,
        }
    }
}

/// `ConsoleLine`.
#[derive(Clone)]
pub struct ConsoleLine {
    /// `ConsoleLine *next`
    pub next: usize,
    /// `ConsoleLine *prev`
    pub prev: usize,
    /// `int len_alloc`
    pub len_alloc: i32,
    /// `int len`
    pub len: i32,
    /// `char *line`
    pub line: usize,
    /// `int cursor`
    pub cursor: i32,
    /// `int type`
    pub type_: i32,
}
impl Default for ConsoleLine {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            len_alloc: 0,
            len: 0,
            line: NIL,
            cursor: 0,
            type_: 0,
        }
    }
}

/// `SpaceConsole`.
#[derive(Clone)]
pub struct SpaceConsole {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `char link_flag`
    pub link_flag: i8,
    /// `ListBaseT<ConsoleLine>`
    pub scrollback: ListBase,
    /// `ListBaseT<ConsoleLine>`
    pub history: ListBase,
    /// `char prompt[256]`
    pub prompt: [u8; 256],
    /// `char language[32]`
    pub language: [u8; 32],
    /// `int line_height`
    pub line_height: i32,
    /// `int history_index`
    pub history_index: i32,
    /// `int sel_start`
    pub sel_start: i32,
    /// `int sel_end`
    pub sel_end: i32,
    /// `SpaceConsole_Runtime *runtime`
    pub runtime: usize,
}
impl Default for SpaceConsole {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: 0,
            scrollback: ListBase::default(),
            history: ListBase::default(),
            prompt: [0; 256],
            language: [0; 32],
            line_height: 0,
            history_index: 0,
            sel_start: 0,
            sel_end: 0,
            runtime: NIL,
        }
    }
}

/// `SpaceUserPref`.
#[derive(Clone)]
pub struct SpaceUserPref {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `char link_flag`
    pub link_flag: i8,
    /// `char filter_type`
    pub filter_type: i8,
    /// `char filter[64]`
    pub filter: [u8; 64],
    /// `SpaceUserPref_Runtime *runtime`
    pub runtime: usize,
}
impl Default for SpaceUserPref {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: 0,
            filter_type: 0,
            filter: [0; 64],
            runtime: NIL,
        }
    }
}

/// `SpaceClipOverlay`.
#[derive(Clone)]
pub struct SpaceClipOverlay {
    /// `int flag`
    pub flag: i32,
}
impl Default for SpaceClipOverlay {
    fn default() -> Self {
        Self {
            flag: SC_SHOW_OVERLAYS | SC_SHOW_CURSOR,
        }
    }
}

/// `SpaceClip`.
#[derive(Clone)]
pub struct SpaceClip {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `eSpace_Link_Flag link_flag`
    pub link_flag: SpaceLinkFlag,
    /// `eSpaceClip_Gizmo_Flag gizmo_flag`
    pub gizmo_flag: SpaceclipGizmoFlag,
    /// `float xof`
    pub xof: f32,
    /// `float yof`
    pub yof: f32,
    /// `float xlockof`
    pub xlockof: f32,
    /// `float ylockof`
    pub ylockof: f32,
    /// `float zoom`
    pub zoom: f32,
    pub user: MovieClipUser,
    /// `MovieClip *clip`
    pub clip: usize,
    pub scopes: MovieClipScopes,
    /// `eSpaceClip_Mode mode`
    pub mode: SpaceclipMode,
    /// `eSpaceClip_View view`
    pub view: SpaceclipView,
    /// `int path_length`
    pub path_length: i32,
    /// `float loc[2]`
    pub loc: [f32; 2],
    /// `float scale`
    pub scale: f32,
    /// `float angle`
    pub angle: f32,
    /// `float stabmat[4][4]`
    pub stabmat: [[f32; 4]; 4],
    /// `float unistabmat[4][4]`
    pub unistabmat: [[f32; 4]; 4],
    /// `int postproc_flag`
    pub postproc_flag: i32,
    /// `eSpaceClip_GPencil_Source gpencil_src`
    pub gpencil_src: SpaceclipGpencilSource,
    /// `eView3D_Around around`
    pub around: View3DAround,
    /// `float cursor[2]`
    pub cursor: [f32; 2],
    pub mask_info: MaskSpaceInfo,
    pub overlay: SpaceClipOverlay,
}
impl Default for SpaceClip {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: SPACE_CLIP,
            link_flag: Default::default(),
            gizmo_flag: Default::default(),
            xof: 0.0,
            yof: 0.0,
            xlockof: 0.0,
            ylockof: 0.0,
            zoom: 1.0,
            user: Default::default(),
            clip: NIL,
            scopes: Default::default(),
            mode: SC_MODE_TRACKING,
            view: SC_VIEW_CLIP,
            path_length: 20,
            loc: [0.0; 2],
            scale: 0.0,
            angle: 0.0,
            stabmat: [[0.0; 4]; 4],
            unistabmat: [[0.0; 4]; 4],
            postproc_flag: 0,
            gpencil_src: SC_GPENCIL_SRC_CLIP,
            around: V3D_AROUND_CENTER_MEDIAN,
            cursor: [0.0; 2],
            mask_info: Default::default(),
            overlay: Default::default(),
        }
    }
}

/// `SpaceTopBar`.
#[derive(Clone)]
pub struct SpaceTopBar {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `char link_flag`
    pub link_flag: i8,
}
impl Default for SpaceTopBar {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: 0,
        }
    }
}

/// `SpaceStatusBar`.
#[derive(Clone)]
pub struct SpaceStatusBar {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `char link_flag`
    pub link_flag: i8,
}
impl Default for SpaceStatusBar {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: 0,
        }
    }
}

/// `SpreadsheetColumnID`.
#[derive(Clone)]
pub struct SpreadsheetColumnID {
    /// `char *name`
    pub name: usize,
}
impl Default for SpreadsheetColumnID {
    fn default() -> Self {
        Self {
            name: NIL,
        }
    }
}

/// `SpreadsheetColumn`.
#[derive(Clone)]
pub struct SpreadsheetColumn {
    /// `SpreadsheetColumnID *id`
    pub id: usize,
    /// `uint8_t data_type`
    pub data_type: u8,
    /// `uint32_t flag`
    pub flag: u32,
    /// `float width`
    pub width: f32,
    /// `uint32_t last_used`
    pub last_used: u32,
    /// `char *display_name`
    pub display_name: usize,
    /// `ed::spreadsheet::SpreadsheetColumnRuntime *runtime`
    pub runtime: usize,
}
impl Default for SpreadsheetColumn {
    fn default() -> Self {
        Self {
            id: NIL,
            data_type: 0,
            flag: 0,
            width: 0.0,
            last_used: 0,
            display_name: NIL,
            runtime: NIL,
        }
    }
}

/// `SpreadsheetInstanceID`.
#[derive(Clone)]
pub struct SpreadsheetInstanceID {
    /// `int reference_index`
    pub reference_index: i32,
}
impl Default for SpreadsheetInstanceID {
    fn default() -> Self {
        Self {
            reference_index: 0,
        }
    }
}

/// `SpreadsheetTableID`.
#[derive(Clone)]
pub struct SpreadsheetTableID {
    /// `int type`
    pub type_: i32,
}
impl Default for SpreadsheetTableID {
    fn default() -> Self {
        Self {
            type_: 0,
        }
    }
}

/// `SpreadsheetBundlePathElem`.
#[derive(Clone)]
pub struct SpreadsheetBundlePathElem {
    /// `char *identifier`
    pub identifier: usize,
}
impl Default for SpreadsheetBundlePathElem {
    fn default() -> Self {
        Self {
            identifier: NIL,
        }
    }
}

/// `SpreadsheetBundleTreeViewPath`.
#[derive(Clone)]
pub struct SpreadsheetBundleTreeViewPath {
    /// `SpreadsheetBundlePathElem *bundle_path`
    pub bundle_path: usize,
    /// `int bundle_path_num`
    pub bundle_path_num: i32,
    /// `SpreadsheetClosureInputOutput closure_input_output`
    pub closure_input_output: Spreadsheetclosureinputoutput,
}
impl Default for SpreadsheetBundleTreeViewPath {
    fn default() -> Self {
        Self {
            bundle_path: NIL,
            bundle_path_num: 0,
            closure_input_output: SPREADSHEET_CLOSURE_NONE,
        }
    }
}

/// `SpreadsheetTableIDGeometry`.
#[derive(Clone)]
pub struct SpreadsheetTableIDGeometry {
    pub base: SpreadsheetTableID,
    pub viewer_path: ViewerPath,
    /// `int viewer_item_identifier`
    pub viewer_item_identifier: i32,
    pub viewer_item_bundle_path: SpreadsheetBundleTreeViewPath,
    /// `SpreadsheetInstanceID *instance_ids`
    pub instance_ids: usize,
    /// `int instance_ids_num`
    pub instance_ids_num: i32,
    /// `uint8_t geometry_component_type`
    pub geometry_component_type: u8,
    /// `uint8_t attribute_domain`
    pub attribute_domain: u8,
    /// `uint8_t object_eval_state`
    pub object_eval_state: u8,
    /// `SpreadsheetGeometryItemType geometry_item_type`
    pub geometry_item_type: Spreadsheetgeometryitemtype,
    pub geometry_bundle_path: SpreadsheetBundleTreeViewPath,
    /// `int layer_index`
    pub layer_index: i32,
}
impl Default for SpreadsheetTableIDGeometry {
    fn default() -> Self {
        Self {
            base: Default::default(),
            viewer_path: Default::default(),
            viewer_item_identifier: 0,
            viewer_item_bundle_path: Default::default(),
            instance_ids: NIL,
            instance_ids_num: 0,
            geometry_component_type: 0,
            attribute_domain: 0,
            object_eval_state: 0,
            geometry_item_type: SPREADSHEET_GEOMETRY_ITEM_TYPE_DOMAIN,
            geometry_bundle_path: Default::default(),
            layer_index: 0,
        }
    }
}

/// `SpreadsheetTable`.
#[derive(Clone)]
pub struct SpreadsheetTable {
    /// `SpreadsheetTableID *id`
    pub id: usize,
    /// `SpreadsheetColumn *columns`
    pub columns: usize,
    /// `int num_columns`
    pub num_columns: i32,
    /// `uint32_t flag`
    pub flag: u32,
    /// `uint32_t last_used`
    pub last_used: u32,
    /// `uint32_t column_use_clock`
    pub column_use_clock: u32,
}
impl Default for SpreadsheetTable {
    fn default() -> Self {
        Self {
            id: NIL,
            columns: NIL,
            num_columns: 0,
            flag: 0,
            last_used: 0,
            column_use_clock: 0,
        }
    }
}

/// `SpaceSpreadsheet`.
#[derive(Clone)]
pub struct SpaceSpreadsheet {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `eSpace_Link_Flag link_flag`
    pub link_flag: SpaceLinkFlag,
    /// `SpreadsheetTable *tables`
    pub tables: usize,
    /// `int num_tables`
    pub num_tables: i32,
    /// `uint8_t filter_flag`
    pub filter_flag: u8,
    pub geometry_id: SpreadsheetTableIDGeometry,
    /// `uint32_t flag`
    pub flag: u32,
    /// `uint32_t table_use_clock`
    pub table_use_clock: u32,
    /// `int active_viewer_path_index`
    pub active_viewer_path_index: i32,
    /// `ed::spreadsheet::SpaceSpreadsheet_Runtime *runtime`
    pub runtime: usize,
}
impl Default for SpaceSpreadsheet {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: Default::default(),
            tables: NIL,
            num_tables: 0,
            filter_flag: 0,
            geometry_id: Default::default(),
            flag: 0,
            table_use_clock: 0,
            active_viewer_path_index: 0,
            runtime: NIL,
        }
    }
}

/// `SpreadsheetRowFilter`.
#[derive(Clone)]
pub struct SpreadsheetRowFilter {
    /// `SpreadsheetRowFilter *next`
    pub next: usize,
    /// `SpreadsheetRowFilter *prev`
    pub prev: usize,
    /// `char column_name[64]`
    pub column_name: [u8; 64],
    /// `uint8_t operation`
    pub operation: u8,
    /// `uint8_t flag`
    pub flag: u8,
    /// `int value_int`
    pub value_int: i32,
    /// `int value_int2[2]`
    pub value_int2: [i32; 2],
    /// `int value_int3[3]`
    pub value_int3: [i32; 3],
    /// `char *value_string`
    pub value_string: usize,
    /// `float value_float`
    pub value_float: f32,
    /// `float threshold`
    pub threshold: f32,
    /// `float value_float2[2]`
    pub value_float2: [f32; 2],
    /// `float value_float3[3]`
    pub value_float3: [f32; 3],
    /// `float value_float4[4]`
    pub value_float4: [f32; 4],
    /// `float value_color[4]`
    pub value_color: [f32; 4],
}
impl Default for SpreadsheetRowFilter {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            column_name: [0; 64],
            operation: 0,
            flag: 0,
            value_int: 0,
            value_int2: [0; 2],
            value_int3: [0; 3],
            value_string: NIL,
            value_float: 0.0,
            threshold: 0.0,
            value_float2: [0.0; 2],
            value_float3: [0.0; 3],
            value_float4: [0.0; 4],
            value_color: [0.0; 4],
        }
    }
}

/// `SpaceProject`.
#[derive(Clone)]
pub struct SpaceProject {
    /// `SpaceLink *next`
    pub next: usize,
    /// `SpaceLink *prev`
    pub prev: usize,
    /// `ListBaseT<ARegion>`
    pub regionbase: ListBase,
    /// `char spacetype`
    pub spacetype: i8,
    /// `char link_flag`
    pub link_flag: i8,
}
impl Default for SpaceProject {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::default(),
            spacetype: 0,
            link_flag: 0,
        }
    }
}

/// `bNodeInstanceKey` — PORT_PENDING full port in `node_types`.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct bNodeInstanceKey {
    pub value: i32,
}
