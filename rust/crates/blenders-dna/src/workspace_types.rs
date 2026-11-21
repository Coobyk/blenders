//! Port of `DNA_workspace_types.h` — `WorkSpace`, tools, layouts, hooks.

use crate::asset_types::AssetLibraryReference;
use crate::id::{ID, NIL};
use crate::viewer_path_types::ViewerPath;
use blenders_bli::listbase::ListBase;

/// `ebToolRef_Runtime_Flag`.
pub type BToolRefRuntimeFlag = i32;
pub const TOOLREF_FLAG_FALLBACK_KEYMAP: BToolRefRuntimeFlag = 1 << 0;
pub const TOOLREF_FLAG_USE_BRUSHES: BToolRefRuntimeFlag = 1 << 1;

/// `bToolRef_Runtime`.
#[derive(Clone)]
pub struct BToolRefRuntime {
    pub cursor: i32,
    /// `char keymap[64]`, `gizmo_group[64]`, `data_block[64]`, `keymap_fallback[64]`, `op[64]`.
    pub keymap: [u8; 64],
    pub gizmo_group: [u8; 64],
    pub data_block: [u8; 64],
    pub brush_type: i32,
    pub keymap_fallback: [u8; 64],
    pub op: [u8; 64],
    pub index: i32,
    pub flag: BToolRefRuntimeFlag,
}
impl Default for BToolRefRuntime {
    fn default() -> Self {
        Self {
            cursor: 0,
            keymap: [0; 64],
            gizmo_group: [0; 64],
            data_block: [0; 64],
            brush_type: 0,
            keymap_fallback: [0; 64],
            op: [0; 64],
            index: 0,
            flag: 0,
        }
    }
}

/// `bToolRef`.
#[derive(Clone)]
pub struct BToolRef {
    pub next: usize,
    pub prev: usize,
    /// `char idname[64]`, `idname_fallback[64]`, `idname_pending[64]`.
    pub idname: [u8; 64],
    pub idname_fallback: [u8; 64],
    pub idname_pending: [u8; 64],
    pub tag: i16,
    pub space_type: i16,
    pub mode: i32,
    /// `IDProperty *properties`.
    pub properties: usize,
    /// `bToolRef_Runtime *runtime`.
    pub runtime: usize,
}
impl Default for BToolRef {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            idname: [0; 64],
            idname_fallback: [0; 64],
            idname_pending: [0; 64],
            tag: 0,
            space_type: 0,
            mode: 0,
            properties: NIL,
            runtime: NIL,
        }
    }
}

/// `WorkSpaceLayout`.
#[derive(Clone)]
pub struct WorkSpaceLayout {
    pub next: usize,
    pub prev: usize,
    /// `bScreen *screen`.
    pub screen: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
}
impl Default for WorkSpaceLayout {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            screen: NIL,
            name: [0; 64],
        }
    }
}

/// `wmOwnerID`.
#[derive(Clone)]
pub struct WmOwnerId {
    pub next: usize,
    pub prev: usize,
    /// `char name[128]`.
    pub name: [u8; 128],
}
impl Default for WmOwnerId {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 128],
        }
    }
}

/// `eWorkSpaceFlags`.
pub type WorkSpaceFlags = i32;
pub const WORKSPACE_USE_FILTER_BY_ORIGIN: WorkSpaceFlags = 1 << 1;
pub const WORKSPACE_USE_PIN_SCENE: WorkSpaceFlags = 1 << 2;
pub const WORKSPACE_SYNC_SCENE_TIME: WorkSpaceFlags = 1 << 3;
pub const WORKSPACE_USE_STRIP_SCENE_RANGE_FOR_PLAYBACK: WorkSpaceFlags = 1 << 4;

/// `WorkSpace` — workspace ID block.
#[derive(Clone)]
pub struct WorkSpace {
    pub id: ID,
    /// `ListBaseT<WorkSpaceLayout>`.
    pub layouts: ListBase,
    /// `ListBaseT<WorkSpaceDataRelation>`.
    pub hook_layout_relations: ListBase,
    /// `ListBaseT<wmOwnerID>`.
    pub owner_ids: ListBase,
    /// `ListBaseT<bToolRef>`.
    pub tools: ListBase,
    /// `Scene *pin_scene`, `*sequencer_scene`.
    pub pin_scene: usize,
    pub sequencer_scene: usize,
    pub object_mode: i32,
    pub flags: WorkSpaceFlags,
    pub order: i32,
    /// `bke::WorkSpaceRuntime *runtime`.
    pub runtime: usize,
    pub asset_library_ref: AssetLibraryReference,
    pub viewer_path: ViewerPath,
}
impl Default for WorkSpace {
    fn default() -> Self {
        Self {
            id: ID::default(),
            layouts: ListBase::default(),
            hook_layout_relations: ListBase::default(),
            owner_ids: ListBase::default(),
            tools: ListBase::default(),
            pin_scene: NIL,
            sequencer_scene: NIL,
            object_mode: 0,
            flags: 0,
            order: 0,
            runtime: NIL,
            asset_library_ref: AssetLibraryReference::default(),
            viewer_path: ViewerPath::default(),
        }
    }
}

/// `WorkSpaceDataRelation`.
#[derive(Clone)]
pub struct WorkSpaceDataRelation {
    pub next: usize,
    pub prev: usize,
    /// `void *parent`, `*value`.
    pub parent: usize,
    pub value: usize,
    pub parentid: i32,
}
impl Default for WorkSpaceDataRelation {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            parent: NIL,
            value: NIL,
            parentid: 0,
        }
    }
}

/// `WorkSpaceInstanceHook`.
#[derive(Clone, Default)]
pub struct WorkSpaceInstanceHook {
    /// `WorkSpace *active`.
    pub active: usize,
    /// `WorkSpaceLayout *act_layout`.
    pub act_layout: usize,
    /// `WorkSpace *temp_workspace_store`.
    pub temp_workspace_store: usize,
    /// `WorkSpaceLayout *temp_layout_store`.
    pub temp_layout_store: usize,
}
