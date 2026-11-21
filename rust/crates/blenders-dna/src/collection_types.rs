//! Port of `DNA_collection_types.h`.
use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;

/// `IOHandlerPanelFlag`.
pub const IO_HANDLER_PANEL_OPEN: i32 = 1 << 0;

/// `eCollectionLightLinkingState`.
pub const COLLECTION_LIGHT_LINKING_STATE_INCLUDE: u8 = 0;
pub const COLLECTION_LIGHT_LINKING_STATE_EXCLUDE: u8 = 1;

/// `eCollectionLineArt_Usage`.
pub const COLLECTION_LRT_INCLUDE: u8 = 0;
pub const COLLECTION_LRT_OCCLUSION_ONLY: u8 = 1 << 0;
pub const COLLECTION_LRT_EXCLUDE: u8 = 1 << 1;
pub const COLLECTION_LRT_INTERSECTION_ONLY: u8 = 1 << 2;
pub const COLLECTION_LRT_NO_INTERSECTION: u8 = 1 << 3;
pub const COLLECTION_LRT_FORCE_INTERSECTION: u8 = 1 << 4;

/// `eCollectionLineArt_Flags`.
pub const COLLECTION_LRT_USE_INTERSECTION_MASK: u8 = 1 << 0;
pub const COLLECTION_LRT_USE_INTERSECTION_PRIORITY: u8 = 1 << 1;

/// `eCollection_Flag`.
pub const COLLECTION_HIDE_VIEWPORT: u8 = 1 << 0;
pub const COLLECTION_HIDE_SELECT: u8 = 1 << 1;
pub const COLLECTION_HIDE_RENDER: u8 = 1 << 3;
pub const COLLECTION_HAS_OBJECT_CACHE: u8 = 1 << 4;
pub const COLLECTION_IS_MASTER: u8 = 1 << 5;
pub const COLLECTION_HAS_OBJECT_CACHE_INSTANCED: u8 = 1 << 6;
pub const COLLECTION_FLAG_ALL_RUNTIME: u8 =
    COLLECTION_HAS_OBJECT_CACHE | COLLECTION_HAS_OBJECT_CACHE_INSTANCED;

/// `CollectionColorTag`.
pub const COLLECTION_COLOR_NONE: i8 = -1;
pub const COLLECTION_COLOR_TOT: i8 = 9;

/// `CollectionLightLinking`.
#[derive(Clone, Copy, Default)]
pub struct CollectionLightLinking {
    pub link_state: u8,
}

/// `CollectionObject`.
#[derive(Clone, Default)]
pub struct CollectionObject {
    pub next: usize,
    pub prev: usize,
    /// `Object *ob`.
    pub ob: usize,
    pub light_linking: CollectionLightLinking,
    pub sort_index: i32,
    pub parented_sort_index: i32,
}
impl CollectionObject {
    pub fn new() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            ob: NIL,
            light_linking: CollectionLightLinking::default(),
            sort_index: -1,
            parented_sort_index: -1,
        }
    }
}

/// `CollectionChild`.
#[derive(Clone, Default)]
pub struct CollectionChild {
    pub next: usize,
    pub prev: usize,
    /// `Collection *collection`.
    pub collection: usize,
    pub light_linking: CollectionLightLinking,
    pub sort_index: i32,
}
impl CollectionChild {
    pub fn new() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            collection: NIL,
            light_linking: CollectionLightLinking::default(),
            sort_index: -1,
        }
    }
}

/// `CollectionImport`.
#[derive(Clone)]
pub struct CollectionImport {
    /// `char fh_idname[64]`.
    pub fh_idname: [u8; 64],
    /// `IDProperty *import_properties`.
    pub import_properties: usize,
    pub flag: u32,
    /// `bke::CollectionImportRuntime *runtime`.
    pub runtime: usize,
}
impl Default for CollectionImport {
    fn default() -> Self {
        Self {
            fh_idname: [0; 64],
            import_properties: NIL,
            flag: 0,
            runtime: NIL,
        }
    }
}

/// `CollectionExport`.
#[derive(Clone)]
pub struct CollectionExport {
    pub next: usize,
    pub prev: usize,
    /// `char fh_idname[64]`.
    pub fh_idname: [u8; 64],
    /// `char name[64]`.
    pub name: [u8; 64],
    /// `IDProperty *export_properties`.
    pub export_properties: usize,
    pub flag: u32,
    /// `ListBaseT<LayoutPanelState>`.
    pub layout_panel_states: ListBase,
}
impl Default for CollectionExport {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            fh_idname: [0; 64],
            name: [0; 64],
            export_properties: NIL,
            flag: 0,
            layout_panel_states: ListBase::new(),
        }
    }
}

/// `Collection`.
#[derive(Clone)]
pub struct Collection {
    pub id: ID,
    /// `ID *owner_id`.
    pub owner_id: usize,
    /// `ListBaseT<CollectionObject>`.
    pub gobject: Vec<CollectionObject>,
    /// `ListBaseT<CollectionChild>`.
    pub children: Vec<CollectionChild>,
    pub active_exporter_index: i32,
    /// `ListBaseT<CollectionExport>`.
    pub exporters: Vec<CollectionExport>,
    /// `CollectionImport *importer`.
    pub importer: usize,
    /// `PreviewImage *preview`.
    pub preview: usize,
    /// DNA_DEPRECATED `layer`.
    pub layer: u32,
    pub instance_offset: [f32; 3],
    pub flag: u8,
    pub color_tag: i8,
    pub lineart_usage: u8,
    pub lineart_flags: u8,
    pub lineart_intersection_mask: u8,
    pub lineart_intersection_priority: u8,
    /// DNA_DEPRECATED `ViewLayer *view_layer`.
    pub view_layer: usize,
    /// `bke::CollectionRuntime *runtime`.
    pub runtime: usize,
}
impl Default for Collection {
    fn default() -> Self {
        Self {
            id: ID::default(),
            owner_id: NIL,
            gobject: Vec::new(),
            children: Vec::new(),
            active_exporter_index: 0,
            exporters: Vec::new(),
            importer: NIL,
            preview: NIL,
            layer: 0,
            instance_offset: [0.0; 3],
            flag: 0,
            color_tag: COLLECTION_COLOR_NONE,
            lineart_usage: COLLECTION_LRT_INCLUDE,
            lineart_flags: 0,
            lineart_intersection_mask: 0,
            lineart_intersection_priority: 0,
            view_layer: NIL,
            runtime: NIL,
        }
    }
}
