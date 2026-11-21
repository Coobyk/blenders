//! Port of `DNA_armature_types.h` — `bArmature`, `Bone`, `BoneColor`,
//! `BoneCollection` + enums.
#![allow(non_camel_case_types)]
use crate::id::{ID, NIL};
use crate::action_types::ThemeWireColor;
use blenders_bli::listbase::ListBase;

/// `eArmature_Flag`.
pub type ArmatureFlag = i32;
pub const ARM_RESTPOS: ArmatureFlag = 1 << 0;
pub const ARM_FLAG_UNUSED_1: ArmatureFlag = 1 << 1;
pub const ARM_DRAWAXES: ArmatureFlag = 1 << 2;
pub const ARM_DRAWNAMES: ArmatureFlag = 1 << 3;
pub const ARM_DRAW_RELATION_FROM_HEAD: ArmatureFlag = 1 << 5;
pub const ARM_BCOLL_SOLO_ACTIVE: ArmatureFlag = 1 << 6;
pub const ARM_FLAG_UNUSED_7: ArmatureFlag = 1 << 7;
pub const ARM_MIRROR_EDIT: ArmatureFlag = 1 << 8;
pub const ARM_FLAG_UNUSED_9: ArmatureFlag = 1 << 9;
pub const ARM_NO_CUSTOM: ArmatureFlag = 1 << 10;
pub const ARM_COL_CUSTOM: ArmatureFlag = 1 << 11;
pub const ARM_FLAG_UNUSED_12: ArmatureFlag = 1 << 12;
pub const ARM_DS_EXPAND: ArmatureFlag = 1 << 13;
pub const ARM_HAS_VIZ_DEPS: ArmatureFlag = 1 << 14;

/// `eArmature_Drawtype`.
pub type ArmatureDrawtype = i32;
pub const ARM_DRAW_TYPE_ARMATURE_DEFINED: ArmatureDrawtype = -1;
pub const ARM_DRAW_TYPE_OCTA: ArmatureDrawtype = 0;
pub const ARM_DRAW_TYPE_STICK: ArmatureDrawtype = 1;
pub const ARM_DRAW_TYPE_B_BONE: ArmatureDrawtype = 2;
pub const ARM_DRAW_TYPE_ENVELOPE: ArmatureDrawtype = 3;
pub const ARM_DRAW_TYPE_WIRE: ArmatureDrawtype = 4;

/// `eArmature_DeformFlag`.
pub type ArmatureDeformFlag = i16;
pub const ARM_DEF_VGROUP: ArmatureDeformFlag = 1 << 0;
pub const ARM_DEF_ENVELOPE: ArmatureDeformFlag = 1 << 1;
pub const ARM_DEF_QUATERNION: ArmatureDeformFlag = 1 << 2;
pub const ARM_DEF_B_BONE_REST: ArmatureDeformFlag = 1 << 3;
pub const ARM_DEF_INVERT_VGROUP: ArmatureDeformFlag = 1 << 4;

/// `eArmature_PathFlag` (DNA_DEPRECATED).
pub const ARM_PATH_FNUMS: i16 = 1 << 0;
pub const ARM_PATH_KFRAS: i16 = 1 << 1;
pub const ARM_PATH_HEADS: i16 = 1 << 2;
pub const ARM_PATH_ACFRA: i16 = 1 << 3;
pub const ARM_PATH_KFNOS: i16 = 1 << 4;

/// `eBone_Flag`.
pub type BoneFlag = i32;
pub const BONE_SELECTED: BoneFlag = 1 << 0;
pub const BONE_ROOTSEL: BoneFlag = 1 << 1;
pub const BONE_TIPSEL: BoneFlag = 1 << 2;
pub const BONE_TRANSFORM: BoneFlag = 1 << 3;
pub const BONE_CONNECTED: BoneFlag = 1 << 4;
pub const BONE_HIDDEN_P: BoneFlag = 1 << 6;
pub const BONE_DONE: BoneFlag = 1 << 7;
pub const BONE_DRAW_ACTIVE: BoneFlag = 1 << 8;
pub const BONE_HINGE: BoneFlag = 1 << 9;
pub const BONE_HIDDEN_A: BoneFlag = 1 << 10;
pub const BONE_MULT_VG_ENV: BoneFlag = 1 << 11;
pub const BONE_NO_DEFORM: BoneFlag = 1 << 12;
pub const BONE_UNKEYED: BoneFlag = 1 << 13;
pub const BONE_HINGE_CHILD_TRANSFORM: BoneFlag = 1 << 14;
pub const BONE_NO_SCALE: BoneFlag = 1 << 15;
pub const BONE_DRAWWIRE: BoneFlag = 1 << 17;
pub const BONE_NO_CYCLICOFFSET: BoneFlag = 1 << 18;
pub const BONE_EDITMODE_LOCKED: BoneFlag = 1 << 19;
pub const BONE_TRANSFORM_CHILD: BoneFlag = 1 << 20;
pub const BONE_UNSELECTABLE: BoneFlag = 1 << 21;
pub const BONE_NO_LOCAL_LOCATION: BoneFlag = 1 << 22;
pub const BONE_RELATIVE_PARENTING: BoneFlag = 1 << 23;
pub const BONE_ADD_PARENT_END_ROLL: BoneFlag = 1 << 24;
pub const BONE_TRANSFORM_MIRROR: BoneFlag = 1 << 25;
pub const BONE_DRAW_LOCKED_WEIGHT: BoneFlag = 1 << 26;

/// `eBone_InheritScaleMode`.
pub type BoneInheritScaleMode = i8;
pub const BONE_INHERIT_SCALE_FULL: BoneInheritScaleMode = 0;
pub const BONE_INHERIT_SCALE_FIX_SHEAR: BoneInheritScaleMode = 1;
pub const BONE_INHERIT_SCALE_AVERAGE: BoneInheritScaleMode = 2;
pub const BONE_INHERIT_SCALE_NONE: BoneInheritScaleMode = 3;
pub const BONE_INHERIT_SCALE_NONE_LEGACY: BoneInheritScaleMode = 4;
pub const BONE_INHERIT_SCALE_ALIGNED: BoneInheritScaleMode = 5;

/// `eBone_BBoneHandleType`.
pub type BoneBboneHandleType = i8;
pub const BBONE_HANDLE_AUTO: BoneBboneHandleType = 0;
pub const BBONE_HANDLE_ABSOLUTE: BoneBboneHandleType = 1;
pub const BBONE_HANDLE_RELATIVE: BoneBboneHandleType = 2;
pub const BBONE_HANDLE_TANGENT: BoneBboneHandleType = 3;

/// `eBone_BBoneMappingMode`.
pub type BoneBboneMappingMode = i8;
pub const BBONE_MAPPING_STRAIGHT: BoneBboneMappingMode = 0;
pub const BBONE_MAPPING_CURVED: BoneBboneMappingMode = 1;

/// `eBone_BBoneFlag`.
pub const BBONE_ADD_PARENT_END_ROLL: i32 = 1 << 0;
pub const BBONE_SCALE_EASING: i32 = 1 << 1;

/// `eBone_BBoneHandleFlag`.
pub const BBONE_HANDLE_SCALE_X: i16 = 1 << 0;
pub const BBONE_HANDLE_SCALE_Y: i16 = 1 << 1;
pub const BBONE_HANDLE_SCALE_Z: i16 = 1 << 2;
pub const BBONE_HANDLE_SCALE_EASE: i16 = 1 << 3;
pub const BBONE_HANDLE_SCALE_ANY: i16 =
    BBONE_HANDLE_SCALE_X | BBONE_HANDLE_SCALE_Y | BBONE_HANDLE_SCALE_Z | BBONE_HANDLE_SCALE_EASE;

pub const MAXBONENAME: usize = 64;

/// `eBoneCollection_Flag`.
pub type BoneCollectionFlag = u8;
pub const BONE_COLLECTION_VISIBLE: BoneCollectionFlag = 1 << 0;
pub const BONE_COLLECTION_SELECTABLE: BoneCollectionFlag = 1 << 1;
pub const BONE_COLLECTION_OVERRIDE_LIBRARY_LOCAL: BoneCollectionFlag = 1 << 2;
pub const BONE_COLLECTION_ANCESTORS_VISIBLE: BoneCollectionFlag = 1 << 3;
pub const BONE_COLLECTION_SOLO: BoneCollectionFlag = 1 << 4;
pub const BONE_COLLECTION_EXPANDED: BoneCollectionFlag = 1 << 5;

/// `BoneColor`.
#[derive(Clone, Default)]
pub struct BoneColor {
    pub palette_index: i8,
    pub custom: ThemeWireColor,
}

/// `Bone_Runtime`.
#[derive(Clone, Default)]
pub struct BoneRuntime {
    /// `ListBaseT<BoneCollectionReference>`.
    pub collections: ListBase,
}

/// `Bone`.
#[derive(Clone)]
pub struct Bone {
    pub next: usize,
    pub prev: usize,
    /// `IDProperty *prop`, `*system_properties`.
    pub prop: usize,
    pub system_properties: usize,
    /// `Bone *parent`.
    pub parent: usize,
    /// `ListBaseT<Bone>` childbase.
    pub childbase: ListBase,
    /// `char name[64]`.
    pub name: [u8; 64],
    pub roll: f32,
    pub head: [f32; 3],
    pub tail: [f32; 3],
    pub bone_mat: [[f32; 3]; 3],
    pub flag: i32,
    pub drawtype: i8,
    pub color: BoneColor,
    pub inherit_scale_mode: BoneInheritScaleMode,
    pub arm_head: [f32; 3],
    pub arm_tail: [f32; 3],
    pub arm_mat: [[f32; 4]; 4],
    pub arm_roll: f32,
    pub dist: f32,
    pub weight: f32,
    pub xwidth: f32,
    pub length: f32,
    pub zwidth: f32,
    pub rad_head: f32,
    pub rad_tail: f32,
    pub roll1: f32,
    pub roll2: f32,
    pub curve_in_x: f32,
    pub curve_in_z: f32,
    pub curve_out_x: f32,
    pub curve_out_z: f32,
    pub ease1: f32,
    pub ease2: f32,
    /// DNA_DEPRECATED.
    pub scale_in_x: f32,
    pub scale_in_z: f32,
    pub scale_out_x: f32,
    pub scale_out_z: f32,
    pub scale_in: [f32; 3],
    pub scale_out: [f32; 3],
    pub size: [f32; 3],
    pub layer: i32,
    pub segments: i16,
    pub bbone_mapping_mode: BoneBboneMappingMode,
    pub bbone_prev_type: BoneBboneHandleType,
    pub bbone_next_type: BoneBboneHandleType,
    pub bbone_flag: i32,
    pub bbone_prev_flag: i16,
    pub bbone_next_flag: i16,
    /// `Bone *bbone_prev`, `*bbone_next`.
    pub bbone_prev: usize,
    pub bbone_next: usize,
    pub runtime: BoneRuntime,
}
impl Default for Bone {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            prop: NIL,
            system_properties: NIL,
            parent: NIL,
            childbase: ListBase::new(),
            name: [0; 64],
            roll: 0.0,
            head: [0.0; 3],
            tail: [0.0; 3],
            bone_mat: [[0.0; 3]; 3],
            flag: 0,
            drawtype: ARM_DRAW_TYPE_ARMATURE_DEFINED as i8,
            color: BoneColor::default(),
            inherit_scale_mode: 0,
            arm_head: [0.0; 3],
            arm_tail: [0.0; 3],
            arm_mat: [[0.0; 4]; 4],
            arm_roll: 0.0,
            dist: 0.0,
            weight: 0.0,
            xwidth: 0.0,
            length: 0.0,
            zwidth: 0.0,
            rad_head: 0.0,
            rad_tail: 0.0,
            roll1: 0.0,
            roll2: 0.0,
            curve_in_x: 0.0,
            curve_in_z: 0.0,
            curve_out_x: 0.0,
            curve_out_z: 0.0,
            ease1: 0.0,
            ease2: 0.0,
            scale_in_x: 0.0,
            scale_in_z: 0.0,
            scale_out_x: 0.0,
            scale_out_z: 0.0,
            scale_in: [1.0; 3],
            scale_out: [1.0; 3],
            size: [0.0; 3],
            layer: 0,
            segments: 0,
            bbone_mapping_mode: 0,
            bbone_prev_type: 0,
            bbone_next_type: 0,
            bbone_flag: 0,
            bbone_prev_flag: 0,
            bbone_next_flag: 0,
            bbone_prev: NIL,
            bbone_next: NIL,
            runtime: BoneRuntime::default(),
        }
    }
}

/// `bArmature`.
#[derive(Clone)]
pub struct BArmature {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    /// `ListBaseT<Bone>`.
    pub bonebase: ListBase,
    /// `GHash *bonehash`.
    pub bonehash: usize,
    /// `ListBaseT<EditBone> *edbo`.
    pub edbo: usize,
    /// `Bone *act_bone`.
    pub act_bone: usize,
    /// `EditBone *act_edbone`.
    pub act_edbone: usize,
    pub needs_flush_to_id: i8,
    pub flag: ArmatureFlag,
    pub drawtype: ArmatureDrawtype,
    pub deformflag: ArmatureDeformFlag,
    pub pathflag: i16,
    /// `ListBaseT<BoneCollection>` (legacy).
    pub collections_legacy: ListBase,
    /// `BoneCollection **collection_array` → Vec.
    pub collection_array: Vec<usize>,
    pub collection_root_count: i32,
    /// `char active_collection_name[64]`.
    pub active_collection_name: [u8; 64],
    /// DNA_DEPRECATED `layer_used`, `layer`, `layer_protected`.
    pub layer_used: u32,
    pub layer: u32,
    pub layer_protected: u32,
    pub axes_position: f32,
    /// `bke::bArmature_Runtime *runtime`.
    pub runtime: usize,
}
impl Default for BArmature {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            bonebase: ListBase::new(),
            bonehash: NIL,
            edbo: NIL,
            act_bone: NIL,
            act_edbone: NIL,
            needs_flush_to_id: 0,
            flag: ARM_COL_CUSTOM,
            drawtype: ARM_DRAW_TYPE_OCTA,
            deformflag: ARM_DEF_VGROUP | ARM_DEF_ENVELOPE,
            pathflag: 0,
            collections_legacy: ListBase::new(),
            collection_array: Vec::new(),
            collection_root_count: 0,
            active_collection_name: [0; 64],
            layer_used: 0,
            layer: 1,
            layer_protected: 0,
            axes_position: 0.0,
            runtime: NIL,
        }
    }
}
impl BArmature {
    /// `collections_span()` → indices of all collections.
    pub fn collections_span(&self) -> &[usize] {
        &self.collection_array
    }
    /// `collections_roots()` → first `collection_root_count` entries.
    pub fn collections_roots(&self) -> &[usize] {
        &self.collection_array[..(self.collection_root_count.max(0) as usize)
            .min(self.collection_array.len())]
    }
}

/// `BoneCollection`.
#[derive(Clone)]
pub struct BoneCollection {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    /// `ListBaseT<BoneCollectionMember>`.
    pub bones: ListBase,
    pub flags: BoneCollectionFlag,
    pub child_index: i32,
    pub child_count: i32,
    /// `IDProperty *prop`, `*system_properties`.
    pub prop: usize,
    pub system_properties: usize,
}
impl Default for BoneCollection {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            bones: ListBase::new(),
            flags: 0,
            child_index: 0,
            child_count: 0,
            prop: NIL,
            system_properties: NIL,
        }
    }
}
impl BoneCollection {
    pub fn is_visible(&self) -> bool {
        self.flags & BONE_COLLECTION_VISIBLE != 0
    }
    pub fn is_visible_ancestors(&self) -> bool {
        self.flags & BONE_COLLECTION_ANCESTORS_VISIBLE != 0
    }
    pub fn is_visible_with_ancestors(&self) -> bool {
        self.is_visible() && self.is_visible_ancestors()
    }
    pub fn is_solo(&self) -> bool {
        self.flags & BONE_COLLECTION_SOLO != 0
    }
    pub fn is_expanded(&self) -> bool {
        self.flags & BONE_COLLECTION_EXPANDED != 0
    }
}

/// `BoneCollectionMember`.
#[derive(Clone)]
pub struct BoneCollectionMember {
    pub next: usize,
    pub prev: usize,
    /// `Bone *bone`.
    pub bone: usize,
}
impl Default for BoneCollectionMember {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            bone: NIL,
        }
    }
}

/// `BoneCollectionReference`.
#[derive(Clone)]
pub struct BoneCollectionReference {
    pub next: usize,
    pub prev: usize,
    /// `BoneCollection *bcoll`.
    pub bcoll: usize,
}
impl Default for BoneCollectionReference {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            bcoll: NIL,
        }
    }
}
