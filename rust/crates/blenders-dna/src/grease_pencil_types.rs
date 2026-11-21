//! Port of `DNA_grease_pencil_types.h` — Grease Pencil v3 ID + layer tree.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::attribute_types::AttributeStorage;
use crate::customdata_types::CustomData;
use crate::curves_types::CurvesGeometry;
use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;
/// `GreasePencilStrokeCapType`.
pub type Greasepencilstrokecaptype = i8;
pub const GP_STROKE_CAP_TYPE_ROUND: Greasepencilstrokecaptype = 0;
pub const GP_STROKE_CAP_TYPE_FLAT: Greasepencilstrokecaptype = 1;
pub const GP_STROKE_CAP_TYPE_MAX: Greasepencilstrokecaptype = GP_STROKE_CAP_TYPE_FLAT + 1;

/// `GreasePencilDrawingType`.
pub type Greasepencildrawingtype = i8;
pub const GP_DRAWING: Greasepencildrawingtype = 0;
pub const GP_DRAWING_REFERENCE: Greasepencildrawingtype = 1;

/// `GreasePencilDrawingBaseFlag`.
pub type Greasepencildrawingbaseflag = u32;
pub const GreasePencilDrawingBaseFlag_TODO: Greasepencildrawingbaseflag = 0;

/// `GreasePencilFrameFlag`.
pub type Greasepencilframeflag = u32;
pub const GP_FRAME_SELECTED: Greasepencilframeflag = 1 << 0;
pub const GP_FRAME_IMPLICIT_HOLD: Greasepencilframeflag = 1 << 1;

/// `GreasePencilLayerFramesMapStorageFlag`.
pub type Greasepencillayerframesmapstorageflag = i32;
pub const GP_LAYER_FRAMES_STORAGE_DIRTY: Greasepencillayerframesmapstorageflag = 1 << 0;

/// `GreasePencilLayerMaskFlag`.
pub type Greasepencillayermaskflag = u16;
pub const GP_LAYER_MASK_HIDE: Greasepencillayermaskflag = 1 << 0;
pub const GP_LAYER_MASK_INVERT: Greasepencillayermaskflag = 1 << 1;

/// `GreasePencilLayerBlendMode`.
pub type Greasepencillayerblendmode = i8;
pub const GP_LAYER_BLEND_NONE: Greasepencillayerblendmode = 0;
pub const GP_LAYER_BLEND_HARDLIGHT: Greasepencillayerblendmode = 1;
pub const GP_LAYER_BLEND_ADD: Greasepencillayerblendmode = 2;
pub const GP_LAYER_BLEND_SUBTRACT: Greasepencillayerblendmode = 3;
pub const GP_LAYER_BLEND_MULTIPLY: Greasepencillayerblendmode = 4;
pub const GP_LAYER_BLEND_DIVIDE: Greasepencillayerblendmode = 5;

/// `GreasePencilLayerTreeNodeType`.
pub type Greasepencillayertreenodetype = i8;
pub const GP_LAYER_TREE_LEAF: Greasepencillayertreenodetype = 0;
pub const GP_LAYER_TREE_GROUP: Greasepencillayertreenodetype = 1;

/// `GreasePencilLayerTreeNodeFlag`.
pub type Greasepencillayertreenodeflag = u32;
pub const GP_LAYER_TREE_NODE_HIDE: Greasepencillayertreenodeflag = 1 << 0;
pub const GP_LAYER_TREE_NODE_LOCKED: Greasepencillayertreenodeflag = 1 << 1;
pub const GP_LAYER_TREE_NODE_SELECT: Greasepencillayertreenodeflag = 1 << 2;
pub const GP_LAYER_TREE_NODE_MUTE: Greasepencillayertreenodeflag = 1 << 3;
pub const GP_LAYER_TREE_NODE_USE_LIGHTS: Greasepencillayertreenodeflag = 1 << 4;
pub const GP_LAYER_TREE_NODE_HIDE_ONION_SKINNING: Greasepencillayertreenodeflag = 1 << 5;
pub const GP_LAYER_TREE_NODE_EXPANDED: Greasepencillayertreenodeflag = 1 << 6;
pub const GP_LAYER_TREE_NODE_HIDE_MASKS: Greasepencillayertreenodeflag = 1 << 7;
pub const GP_LAYER_TREE_NODE_DISABLE_MASKS_IN_VIEWLAYER: Greasepencillayertreenodeflag = 1 << 8;
pub const GP_LAYER_TREE_NODE_IGNORE_LOCKED_MATERIALS: Greasepencillayertreenodeflag = 1 << 9;

/// `GroupColorTag`.
pub type Groupcolortag = i8;
pub const LAYERGROUP_COLOR_NONE: Groupcolortag = -1;
pub const LAYERGROUP_COLOR_01: Groupcolortag = LAYERGROUP_COLOR_NONE + 1;
pub const LAYERGROUP_COLOR_02: Groupcolortag = LAYERGROUP_COLOR_01 + 1;
pub const LAYERGROUP_COLOR_03: Groupcolortag = LAYERGROUP_COLOR_02 + 1;
pub const LAYERGROUP_COLOR_04: Groupcolortag = LAYERGROUP_COLOR_03 + 1;
pub const LAYERGROUP_COLOR_05: Groupcolortag = LAYERGROUP_COLOR_04 + 1;
pub const LAYERGROUP_COLOR_06: Groupcolortag = LAYERGROUP_COLOR_05 + 1;
pub const LAYERGROUP_COLOR_07: Groupcolortag = LAYERGROUP_COLOR_06 + 1;
pub const LAYERGROUP_COLOR_08: Groupcolortag = LAYERGROUP_COLOR_07 + 1;

/// `GreasePencilFlag`.
pub type Greasepencilflag = u32;
pub const GREASE_PENCIL_ANIM_CHANNEL_EXPANDED: Greasepencilflag = 1 << 0;
pub const GREASE_PENCIL_AUTOLOCK_LAYERS: Greasepencilflag = 1 << 1;
pub const GREASE_PENCIL_STROKE_ORDER_3D: Greasepencilflag = 1 << 2;

/// `GreasePencilOnionSkinningMode`.
pub type Greasepencilonionskinningmode = i8;
pub const GP_ONION_SKINNING_MODE_ABSOLUTE: Greasepencilonionskinningmode = 0;
pub const GP_ONION_SKINNING_MODE_RELATIVE: Greasepencilonionskinningmode = 1;
pub const GP_ONION_SKINNING_MODE_SELECTED: Greasepencilonionskinningmode = 2;

/// `GreasePencilOnionSkinningFlag`.
pub type Greasepencilonionskinningflag = u8;
pub const GP_ONION_SKINNING_USE_CUSTOM_COLORS: Greasepencilonionskinningflag = 1 << 0;
pub const GP_ONION_SKINNING_USE_FADE: Greasepencilonionskinningflag = 1 << 1;
pub const GP_ONION_SKINNING_SHOW_LOOP: Greasepencilonionskinningflag = 1 << 2;

/// `GreasePencilOnionSkinningFilter`.
pub type Greasepencilonionskinningfilter = u8;
pub const GP_ONION_SKINNING_FILTER_KEYTYPE_KEYFRAME: Greasepencilonionskinningfilter = 1 << 0;
pub const GP_ONION_SKINNING_FILTER_KEYTYPE_EXTREME: Greasepencilonionskinningfilter = 1 << 1;
pub const GP_ONION_SKINNING_FILTER_KEYTYPE_BREAKDOWN: Greasepencilonionskinningfilter = 1 << 2;
pub const GP_ONION_SKINNING_FILTER_KEYTYPE_JITTER: Greasepencilonionskinningfilter = 1 << 3;
pub const GP_ONION_SKINNING_FILTER_KEYTYPE_MOVEHOLD: Greasepencilonionskinningfilter = 1 << 4;

/// `GREASE_PENCIL_ONION_SKINNING_FILTER_ALL` define.
pub const GREASE_PENCIL_ONION_SKINNING_FILTER_ALL: Greasepencilonionskinningfilter =
    GP_ONION_SKINNING_FILTER_KEYTYPE_KEYFRAME
    | GP_ONION_SKINNING_FILTER_KEYTYPE_EXTREME
    | GP_ONION_SKINNING_FILTER_KEYTYPE_BREAKDOWN
    | GP_ONION_SKINNING_FILTER_KEYTYPE_JITTER
    | GP_ONION_SKINNING_FILTER_KEYTYPE_MOVEHOLD;

/// `GP_STROKE_MITER_ANGLE_ROUND`/`BEVEL` defines.
pub const GP_STROKE_MITER_ANGLE_ROUND: f32 = 0.0;
pub const GP_STROKE_MITER_ANGLE_BEVEL: f32 = std::f32::consts::PI;

/// `GreasePencilDrawingBase`.
#[derive(Clone)]
pub struct GreasePencilDrawingBase {
    /// `GreasePencilDrawingType type`
    pub type_: Greasepencildrawingtype,
    /// `GreasePencilDrawingBaseFlag flag`
    pub flag: Greasepencildrawingbaseflag,
}
impl Default for GreasePencilDrawingBase {
    fn default() -> Self {
        Self {
            type_: GP_DRAWING,
            flag: Default::default(),
        }
    }
}

/// `GreasePencilDrawing`.
#[derive(Clone)]
pub struct GreasePencilDrawing {
    pub base: GreasePencilDrawingBase,
    pub geometry: CurvesGeometry,
    /// `bke::greasepencil::DrawingRuntime *runtime`
    pub runtime: usize,
}
impl Default for GreasePencilDrawing {
    fn default() -> Self {
        Self {
            base: Default::default(),
            geometry: Default::default(),
            runtime: NIL,
        }
    }
}

/// `GreasePencilDrawingReference`.
#[derive(Clone)]
pub struct GreasePencilDrawingReference {
    pub base: GreasePencilDrawingBase,
    /// `GreasePencil *id_reference`
    pub id_reference: usize,
}
impl Default for GreasePencilDrawingReference {
    fn default() -> Self {
        Self {
            base: Default::default(),
            id_reference: NIL,
        }
    }
}

/// `GreasePencilFrame`.
#[derive(Clone)]
pub struct GreasePencilFrame {
    /// `int drawing_index`
    pub drawing_index: i32,
    /// `GreasePencilFrameFlag flag`
    pub flag: Greasepencilframeflag,
    /// `int8_t type`
    pub type_: i8,
}
impl Default for GreasePencilFrame {
    fn default() -> Self {
        Self {
            drawing_index: 0,
            flag: Default::default(),
            type_: 0,
        }
    }
}

/// `GreasePencilLayerFramesMapStorage`.
#[derive(Clone)]
pub struct GreasePencilLayerFramesMapStorage {
    /// `int *keys`
    pub keys: usize,
    /// `GreasePencilFrame *values`
    pub values: usize,
    /// `int num`
    pub num: i32,
    /// `GreasePencilLayerFramesMapStorageFlag flag`
    pub flag: Greasepencillayerframesmapstorageflag,
}
impl Default for GreasePencilLayerFramesMapStorage {
    fn default() -> Self {
        Self {
            keys: NIL,
            values: NIL,
            num: 0,
            flag: Default::default(),
        }
    }
}

/// `GreasePencilLayerMask`.
#[derive(Clone)]
pub struct GreasePencilLayerMask {
    /// `GreasePencilLayerMask *next`
    pub next: usize,
    /// `GreasePencilLayerMask *prev`
    pub prev: usize,
    /// `char *layer_name`
    pub layer_name: usize,
    /// `GreasePencilLayerMaskFlag flag`
    pub flag: Greasepencillayermaskflag,
}
impl Default for GreasePencilLayerMask {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            layer_name: NIL,
            flag: Default::default(),
        }
    }
}

/// `GreasePencilLayerTreeNode`.
#[derive(Clone)]
pub struct GreasePencilLayerTreeNode {
    /// `GreasePencilLayerTreeNode *next`
    pub next: usize,
    /// `GreasePencilLayerTreeNode *prev`
    pub prev: usize,
    /// `GreasePencilLayerTreeGroup *parent`
    pub parent: usize,
    /// `char *name`
    pub name: usize,
    /// `GreasePencilLayerTreeNodeType type`
    pub type_: Greasepencillayertreenodetype,
    /// `float color[3]`
    pub color: [f32; 3],
    /// `GreasePencilLayerTreeNodeFlag flag`
    pub flag: Greasepencillayertreenodeflag,
}
impl Default for GreasePencilLayerTreeNode {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            parent: NIL,
            name: NIL,
            type_: GP_LAYER_TREE_LEAF,
            color: [0.0; 3],
            flag: Default::default(),
        }
    }
}

/// `GreasePencilLayer`.
#[derive(Clone)]
pub struct GreasePencilLayer {
    pub base: GreasePencilLayerTreeNode,
    pub frames_storage: GreasePencilLayerFramesMapStorage,
    /// `GreasePencilLayerBlendMode blend_mode`
    pub blend_mode: Greasepencillayerblendmode,
    /// `float opacity`
    pub opacity: f32,
    /// `ListBaseT<GreasePencilLayerMask>`
    pub masks: ListBase,
    /// `int active_mask_index`
    pub active_mask_index: i32,
    /// `Object *parent`
    pub parent: usize,
    /// `char *parsubstr`
    pub parsubstr: usize,
    /// `float parentinv[4][4]`
    pub parentinv: [[f32; 4]; 4],
    /// `float translation[3]`
    pub translation: [f32; 3],
    /// `float rotation[3]`
    pub rotation: [f32; 3],
    /// `float scale[3]`
    pub scale: [f32; 3],
    /// `char *viewlayername`
    pub viewlayername: usize,
    /// `bke::greasepencil::LayerRuntime *runtime`
    pub runtime: usize,
}
impl Default for GreasePencilLayer {
    fn default() -> Self {
        Self {
            base: Default::default(),
            frames_storage: Default::default(),
            blend_mode: GP_LAYER_BLEND_NONE,
            opacity: 0.0,
            masks: ListBase::default(),
            active_mask_index: 0,
            parent: NIL,
            parsubstr: NIL,
            parentinv: [[0.0; 4]; 4],
            translation: [0.0; 3],
            rotation: [0.0; 3],
            scale: [0.0; 3],
            viewlayername: NIL,
            runtime: NIL,
        }
    }
}

/// `GreasePencilLayerTreeGroup`.
#[derive(Clone)]
pub struct GreasePencilLayerTreeGroup {
    pub base: GreasePencilLayerTreeNode,
    /// `ListBaseT<GreasePencilLayerTreeNode>`
    pub children: ListBase,
    /// `GroupColorTag color_tag`
    pub color_tag: Groupcolortag,
    /// `bke::greasepencil::LayerGroupRuntime *runtime`
    pub runtime: usize,
}
impl Default for GreasePencilLayerTreeGroup {
    fn default() -> Self {
        Self {
            base: Default::default(),
            children: ListBase::default(),
            color_tag: LAYERGROUP_COLOR_01,
            runtime: NIL,
        }
    }
}

/// `GreasePencilOnionSkinningSettings`.
#[derive(Clone)]
pub struct GreasePencilOnionSkinningSettings {
    /// `float opacity`
    pub opacity: f32,
    /// `GreasePencilOnionSkinningMode mode`
    pub mode: Greasepencilonionskinningmode,
    /// `GreasePencilOnionSkinningFilter filter`
    pub filter: Greasepencilonionskinningfilter,
    /// `int16_t num_frames_before`
    pub num_frames_before: i16,
    /// `int16_t num_frames_after`
    pub num_frames_after: i16,
    /// `float color_before[3]`
    pub color_before: [f32; 3],
    /// `float color_after[3]`
    pub color_after: [f32; 3],
}
impl Default for GreasePencilOnionSkinningSettings {
    fn default() -> Self {
        Self {
            opacity: 0.5,
            mode: GP_ONION_SKINNING_MODE_RELATIVE,
            filter: GREASE_PENCIL_ONION_SKINNING_FILTER_ALL,
            num_frames_before: 1,
            num_frames_after: 1,
            color_before: [0.0; 3],
            color_after: [0.0; 3],
        }
    }
}

/// `GreasePencil`.
#[derive(Clone)]
pub struct GreasePencil {
    pub id: ID,
    /// `AnimData *adt`
    pub adt: usize,
    /// `GreasePencilDrawingBase *drawing_array`
    pub drawing_array: usize,
    /// `int drawing_array_num`
    pub drawing_array_num: i32,
    /// `GreasePencilLayerTreeGroup *root_group_ptr`
    pub root_group_ptr: usize,
    pub layers_data_legacy: CustomData,
    pub attribute_storage: AttributeStorage,
    /// `int attributes_active_index`
    pub attributes_active_index: i32,
    /// `GreasePencilLayerTreeNode *active_node`
    pub active_node: usize,
    /// `Material *material_array`
    pub material_array: usize,
    /// `short material_array_num`
    pub material_array_num: i16,
    /// `GreasePencilFlag flag`
    pub flag: Greasepencilflag,
    /// `ListBaseT<bDeformGroup>`
    pub vertex_group_names: ListBase,
    /// `int vertex_group_active_index`
    pub vertex_group_active_index: i32,
    pub onion_skinning_settings: GreasePencilOnionSkinningSettings,
    /// `bke::GreasePencilRuntime *runtime`
    pub runtime: usize,
}
impl Default for GreasePencil {
    fn default() -> Self {
        Self {
            id: Default::default(),
            adt: NIL,
            drawing_array: NIL,
            drawing_array_num: 0,
            root_group_ptr: NIL,
            layers_data_legacy: Default::default(),
            attribute_storage: Default::default(),
            attributes_active_index: -1,
            active_node: NIL,
            material_array: NIL,
            material_array_num: 0,
            flag: GREASE_PENCIL_ANIM_CHANNEL_EXPANDED,
            vertex_group_names: ListBase::default(),
            vertex_group_active_index: 0,
            onion_skinning_settings: Default::default(),
            runtime: NIL,
        }
    }
}
