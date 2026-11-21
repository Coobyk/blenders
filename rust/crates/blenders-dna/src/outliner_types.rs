//! Port of `DNA_outliner_types.h` — outliner tree-store elements.

use crate::id::NIL;

/// `eTreeStoreElem_Flag` (short).
pub type TreeStoreElemFlag = i16;
pub const TSE_CLOSED: TreeStoreElemFlag = 1 << 0;
pub const TSE_SELECTED: TreeStoreElemFlag = 1 << 1;
pub const TSE_TEXTBUT: TreeStoreElemFlag = 1 << 2;
pub const TSE_CHILDSEARCH: TreeStoreElemFlag = 1 << 3;
pub const TSE_SEARCHMATCH: TreeStoreElemFlag = 1 << 4;
pub const TSE_HIGHLIGHTED: TreeStoreElemFlag = 1 << 5;
pub const TSE_DRAG_INTO: TreeStoreElemFlag = 1 << 6;
pub const TSE_DRAG_BEFORE: TreeStoreElemFlag = 1 << 7;
pub const TSE_DRAG_AFTER: TreeStoreElemFlag = 1 << 8;
pub const TSE_ACTIVE: TreeStoreElemFlag = 1 << 9;
pub const TSE_HIGHLIGHTED_ICON: TreeStoreElemFlag = 1 << 11;
pub const TSE_DRAG_ANY: TreeStoreElemFlag = TSE_DRAG_INTO | TSE_DRAG_BEFORE | TSE_DRAG_AFTER;
pub const TSE_HIGHLIGHTED_ANY: TreeStoreElemFlag = TSE_HIGHLIGHTED | TSE_HIGHLIGHTED_ICON;

/// `eTreeStoreElemType` (short).
pub type TreeStoreElemType = i16;
pub const TSE_SOME_ID: TreeStoreElemType = 0;
pub const TSE_NLA: TreeStoreElemType = 1;
pub const TSE_NLA_ACTION: TreeStoreElemType = 2;
pub const TSE_DEFGROUP_BASE: TreeStoreElemType = 3;
pub const TSE_DEFGROUP: TreeStoreElemType = 4;
pub const TSE_BONE: TreeStoreElemType = 5;
pub const TSE_EBONE: TreeStoreElemType = 6;
pub const TSE_CONSTRAINT_BASE: TreeStoreElemType = 7;
pub const TSE_CONSTRAINT: TreeStoreElemType = 8;
pub const TSE_MODIFIER_BASE: TreeStoreElemType = 9;
pub const TSE_MODIFIER: TreeStoreElemType = 10;
pub const TSE_LINKED_OB: TreeStoreElemType = 11;
pub const TSE_POSE_BASE: TreeStoreElemType = 13;
pub const TSE_POSE_CHANNEL: TreeStoreElemType = 14;
pub const TSE_ANIM_DATA: TreeStoreElemType = 15;
pub const TSE_DRIVER_BASE: TreeStoreElemType = 16;
pub const TSE_R_LAYER_BASE: TreeStoreElemType = 19;
pub const TSE_R_LAYER: TreeStoreElemType = 20;
pub const TSE_BONE_COLLECTION_BASE: TreeStoreElemType = 24;
pub const TSE_BONE_COLLECTION: TreeStoreElemType = 25;
pub const TSE_STRIP: TreeStoreElemType = 26;
pub const TSE_STRIP_DATA: TreeStoreElemType = 27;
pub const TSE_STRIP_DUP: TreeStoreElemType = 28;
pub const TSE_LINKED_PSYS: TreeStoreElemType = 29;
pub const TSE_RNA_STRUCT: TreeStoreElemType = 30;
pub const TSE_RNA_PROPERTY: TreeStoreElemType = 31;
pub const TSE_RNA_ARRAY_ELEM: TreeStoreElemType = 32;
pub const TSE_NLA_TRACK: TreeStoreElemType = 33;
pub const TSE_ID_BASE: TreeStoreElemType = 36;
pub const TSE_GP_LAYER: TreeStoreElemType = 37;
pub const TSE_LAYER_COLLECTION: TreeStoreElemType = 38;
pub const TSE_SCENE_COLLECTION_BASE: TreeStoreElemType = 39;
pub const TSE_VIEW_COLLECTION_BASE: TreeStoreElemType = 40;
pub const TSE_SCENE_OBJECTS_BASE: TreeStoreElemType = 41;
pub const TSE_GPENCIL_EFFECT_BASE: TreeStoreElemType = 42;
pub const TSE_GPENCIL_EFFECT: TreeStoreElemType = 43;
pub const TSE_LIBRARY_OVERRIDE_BASE: TreeStoreElemType = 44;
pub const TSE_LIBRARY_OVERRIDE: TreeStoreElemType = 45;
pub const TSE_LIBRARY_OVERRIDE_OPERATION: TreeStoreElemType = 46;
pub const TSE_GENERIC_LABEL: TreeStoreElemType = 47;
pub const TSE_GREASE_PENCIL_NODE: TreeStoreElemType = 48;
pub const TSE_LINKED_NODE_TREE: TreeStoreElemType = 49;
pub const TSE_ACTION_SLOT: TreeStoreElemType = 50;
pub const TSE_SHAPE_KEY_BLOCK: TreeStoreElemType = 51;
pub const TSE_SHAPE_KEY_BASE: TreeStoreElemType = 52;

/// `TSE_IS_REAL_ID(tse)` — element types that wrap a real ID.
pub fn tse_is_real_id(t: TreeStoreElemType) -> bool {
    !matches!(
        t,
        TSE_NLA
            | TSE_NLA_TRACK
            | TSE_DRIVER_BASE
            | TSE_STRIP
            | TSE_STRIP_DATA
            | TSE_STRIP_DUP
            | TSE_RNA_STRUCT
            | TSE_RNA_PROPERTY
            | TSE_RNA_ARRAY_ELEM
            | TSE_ID_BASE
            | TSE_GP_LAYER
            | TSE_GENERIC_LABEL
    )
}

/// `TreeStoreElem`.
#[derive(Clone)]
pub struct TreeStoreElem {
    pub type_: TreeStoreElemType,
    pub nr: i16,
    pub flag: TreeStoreElemFlag,
    pub used: i16,
    /// `ID *id`.
    pub id: usize,
}
impl Default for TreeStoreElem {
    fn default() -> Self {
        Self {
            type_: TSE_SOME_ID,
            nr: 0,
            flag: 0,
            used: 0,
            id: NIL,
        }
    }
}

/// `TreeStore`.
#[derive(Clone, Default)]
pub struct TreeStore {
    /// `DNA_DEPRECATED int totelem`.
    pub totelem: i32,
    pub usedelem: i32,
    /// `TreeStoreElem *data` → Vec.
    pub data: Vec<TreeStoreElem>,
}
