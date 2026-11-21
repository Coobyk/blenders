//! Port of `DNA_node_types.h` — bNodeTree/bNode/bNodeSocket + per-node data.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::color_types::{ColorManagedColorspaceSettings, ColorManagedDisplaySettings, ColorManagedViewSettings, CurveMapping};
use crate::small_types::{BUuid, ColorBand};
use crate::texture_types::{ColorMapping, TexMapping};
use crate::id::{ID, NIL};
use crate::image_types::ImageUser;
use crate::scene_types::ImageFormatData;
use crate::screen_types::TextboxState;
use crate::node_tree_interface_types::*;

use crate::vec_types::{Rctf, Rcti};
use blenders_bli::listbase::ListBase;
/// `eNodeStack_Datatype`.
pub type NodestackDatatype = i16;
pub const NS_OSA_VECTORS: NodestackDatatype = 1;
pub const NS_OSA_VALUES: NodestackDatatype = 2;

/// `eNodeSocket_ConversionRule`.
pub type NodesocketConversionrule = i16;
pub const NS_CR_CENTER: NodesocketConversionrule = 0;
pub const NS_CR_NONE: NodesocketConversionrule = 1;
pub const NS_CR_FIT_WIDTH: NodesocketConversionrule = 2;
pub const NS_CR_FIT_HEIGHT: NodesocketConversionrule = 3;
pub const NS_CR_FIT: NodesocketConversionrule = 4;
pub const NS_CR_STRETCH: NodesocketConversionrule = 5;

/// `eNodeSocketDatatype`.
pub type Nodesocketdatatype = i16;
pub const SOCK_CUSTOM: Nodesocketdatatype = -1;
pub const SOCK_FLOAT: Nodesocketdatatype = 0;
pub const SOCK_VECTOR: Nodesocketdatatype = 1;
pub const SOCK_RGBA: Nodesocketdatatype = 2;
pub const SOCK_SHADER: Nodesocketdatatype = 3;
pub const SOCK_BOOLEAN: Nodesocketdatatype = 4;
pub const SOCK_INT: Nodesocketdatatype = 6;
pub const SOCK_STRING: Nodesocketdatatype = 7;
pub const SOCK_OBJECT: Nodesocketdatatype = 8;
pub const SOCK_IMAGE: Nodesocketdatatype = 9;
pub const SOCK_GEOMETRY: Nodesocketdatatype = 10;
pub const SOCK_COLLECTION: Nodesocketdatatype = 11;
pub const SOCK_TEXTURE: Nodesocketdatatype = 12;
pub const SOCK_MATERIAL: Nodesocketdatatype = 13;
pub const SOCK_ROTATION: Nodesocketdatatype = 14;
pub const SOCK_MENU: Nodesocketdatatype = 15;
pub const SOCK_MATRIX: Nodesocketdatatype = 16;
pub const SOCK_BUNDLE: Nodesocketdatatype = 17;
pub const SOCK_CLOSURE: Nodesocketdatatype = 18;
pub const SOCK_FONT: Nodesocketdatatype = 19;
pub const SOCK_SCENE: Nodesocketdatatype = 20;
pub const SOCK_TEXT_ID: Nodesocketdatatype = 21;
pub const SOCK_MASK: Nodesocketdatatype = 22;
pub const SOCK_SOUND: Nodesocketdatatype = 23;
pub const SOCK_INT_VECTOR: Nodesocketdatatype = 24;
pub const SOCK_DYNAMIC: Nodesocketdatatype = 25;

/// `eNodeSocketDisplayShape`.
pub type Nodesocketdisplayshape = i8;
pub const SOCK_DISPLAY_SHAPE_CIRCLE: Nodesocketdisplayshape = 0;
pub const SOCK_DISPLAY_SHAPE_SQUARE: Nodesocketdisplayshape = 1;
pub const SOCK_DISPLAY_SHAPE_DIAMOND: Nodesocketdisplayshape = 2;
pub const SOCK_DISPLAY_SHAPE_CIRCLE_DOT: Nodesocketdisplayshape = 3;
pub const SOCK_DISPLAY_SHAPE_SQUARE_DOT: Nodesocketdisplayshape = 4;
pub const SOCK_DISPLAY_SHAPE_DIAMOND_DOT: Nodesocketdisplayshape = 5;
pub const SOCK_DISPLAY_SHAPE_LINE: Nodesocketdisplayshape = 6;
pub const SOCK_DISPLAY_SHAPE_VOLUME_GRID: Nodesocketdisplayshape = 7;
pub const SOCK_DISPLAY_SHAPE_LIST: Nodesocketdisplayshape = 8;

/// `eNodeSocketInOut`.
pub type Nodesocketinout = i16;
pub const SOCK_IN: Nodesocketinout = 1 << 0;
pub const SOCK_OUT: Nodesocketinout = 1 << 1;

/// `eNodeSocketFlag`.
pub type Nodesocketflag = i16;
pub const SOCK_SELECT: Nodesocketflag = 1 << 0;
pub const SOCK_HIDDEN: Nodesocketflag = 1 << 1;
pub const SOCK_IS_LINKED: Nodesocketflag = 1 << 2;
pub const SOCK_UNAVAIL: Nodesocketflag = 1 << 3;
pub const SOCK_GIZMO_PIN: Nodesocketflag = 1 << 4;
pub const SOCK_COLLAPSED: Nodesocketflag = 1 << 6;
pub const SOCK_HIDE_VALUE: Nodesocketflag = 1 << 7;
pub const SOCK_AUTO_HIDDEN__DEPRECATED: Nodesocketflag = 1 << 8;
pub const SOCK_NO_INTERNAL_LINK_LEGACY: Nodesocketflag = 1 << 9;
pub const SOCK_COMPACT_LEGACY: Nodesocketflag = 1 << 10;
pub const SOCK_MULTI_INPUT: Nodesocketflag = 1 << 11;
pub const SOCK_HIDE_LABEL_LEGACY: Nodesocketflag = 1 << 12;
pub const SOCK_HIDE_IN_MODIFIER: Nodesocketflag = 1 << 13;
pub const SOCK_PANEL_COLLAPSED: Nodesocketflag = 1 << 14;

/// `eNodePanelFlag`.
pub type Nodepanelflag = i8;
pub const NODE_PANEL_COLLAPSED: Nodepanelflag = 1 << 0;
pub const NODE_PANEL_PARENT_COLLAPSED: Nodepanelflag = 1 << 1;
pub const NODE_PANEL_CONTENT_VISIBLE: Nodepanelflag = 1 << 2;

/// `eViewerNodeShortcut`.
pub type Viewernodeshortcut = i32;
pub const NODE_VIEWER_SHORTCUT_NONE: Viewernodeshortcut = 0;
pub const NODE_VIEWER_SHORTCUT_SLOT_1: Viewernodeshortcut = 1;
pub const NODE_VIEWER_SHORTCUT_SLOT_2: Viewernodeshortcut = 2;
pub const NODE_VIEWER_SHORTCUT_SLOT_3: Viewernodeshortcut = 3;
pub const NODE_VIEWER_SHORTCUT_SLOT_4: Viewernodeshortcut = 4;
pub const NODE_VIEWER_SHORTCUT_SLOT_5: Viewernodeshortcut = 5;
pub const NODE_VIEWER_SHORTCUT_SLOT_6: Viewernodeshortcut = 6;
pub const NODE_VIEWER_SHORTCUT_SLOT_7: Viewernodeshortcut = 7;
pub const NODE_VIEWER_SHORTCUT_SLOT_8: Viewernodeshortcut = 8;
pub const NODE_VIEWER_SHORTCUT_SLOT_9: Viewernodeshortcut = 9;

/// `NodeWarningPropagation`.
pub type Nodewarningpropagation = i8;
pub const NODE_WARNING_PROPAGATION_ALL: Nodewarningpropagation = 0;
pub const NODE_WARNING_PROPAGATION_NONE: Nodewarningpropagation = 1;
pub const NODE_WARNING_PROPAGATION_ONLY_ERRORS: Nodewarningpropagation = 2;
pub const NODE_WARNING_PROPAGATION_ONLY_ERRORS_AND_WARNINGS: Nodewarningpropagation = 3;

/// `eNode_Flag`.
pub type NodeFlag = i32;
pub const NODE_SELECT: NodeFlag = 1 << 0;
pub const NODE_OPTIONS: NodeFlag = 1 << 1;
pub const NODE_PREVIEW: NodeFlag = 1 << 2;
pub const NODE_COLLAPSED: NodeFlag = 1 << 3;
pub const NODE_ACTIVE: NodeFlag = 1 << 4;
pub const NODE_DO_OUTPUT: NodeFlag = 1 << 6;
pub const NODE_TEST: NodeFlag = 1 << 8;
pub const NODE_MUTED: NodeFlag = 1 << 9;
pub const NODE_ACTIVE_TEXTURE: NodeFlag = 1 << 14;
pub const NODE_CUSTOM_COLOR: NodeFlag = 1 << 15;
pub const NODE_INIT: NodeFlag = 1 << 16;
pub const NODE_ACTIVE_PAINT_CANVAS: NodeFlag = 1 << 19;

/// `eNode_Update`.
pub type NodeUpdate = i16;
pub const NODE_UPDATE_ID: NodeUpdate = 1;

/// `eNodeLink_Flag`.
pub type NodelinkFlag = i32;
pub const NODE_LINK_INSERT_TARGET: NodelinkFlag = 1 << 0;
pub const NODE_LINK_VALID: NodelinkFlag = 1 << 1;
pub const NODE_LINK_TEST: NodelinkFlag = 1 << 2;
pub const NODE_LINK_TEMP_HIGHLIGHT: NodelinkFlag = 1 << 3;
pub const NODE_LINK_MUTED: NodelinkFlag = 1 << 4;
pub const NODE_LINK_INSERT_TARGET_INVALID: NodelinkFlag = 1 << 5;

/// `eNodeTree_Type`.
pub type NodetreeType = i32;
pub const NTREE_UNDEFINED: NodetreeType = -2;
pub const NTREE_CUSTOM: NodetreeType = -1;
pub const NTREE_SHADER: NodetreeType = 0;
pub const NTREE_COMPOSIT: NodetreeType = 1;
pub const NTREE_TEXTURE: NodetreeType = 2;
pub const NTREE_GEOMETRY: NodetreeType = 3;

/// `eNodeTree_Flag`.
pub type NodetreeFlag = i32;
pub const NTREE_DS_EXPAND: NodetreeFlag = 1 << 0;
pub const NTREE_UNUSED_2: NodetreeFlag = 1 << 2;
pub const NTREE_VIEWER_BORDER: NodetreeFlag = 1 << 4;
pub const NTREE_IS_GPU_SHADER_INTERNAL: NodetreeFlag = 1 << 6;

/// `eNodeTreeRuntimeFlag`.
pub type Nodetreeruntimeflag = i32;
pub const NTREE_RUNTIME_FLAG_HAS_IMAGE_ANIMATION: Nodetreeruntimeflag = 1 << 0;
pub const NTREE_RUNTIME_FLAG_HAS_MATERIAL_OUTPUT: Nodetreeruntimeflag = 1 << 1;
pub const NTREE_RUNTIME_FLAG_HAS_SIMULATION_ZONE: Nodetreeruntimeflag = 1 << 2;

/// `GeometryNodeAssetTraitFlag`.
pub type Geometrynodeassettraitflag = i32;
pub const GEO_NODE_ASSET_TOOL: Geometrynodeassettraitflag = 1 << 0;
pub const GEO_NODE_ASSET_EDIT: Geometrynodeassettraitflag = 1 << 1;
pub const GEO_NODE_ASSET_SCULPT: Geometrynodeassettraitflag = 1 << 2;
pub const GEO_NODE_ASSET_MESH: Geometrynodeassettraitflag = 1 << 3;
pub const GEO_NODE_ASSET_CURVE: Geometrynodeassettraitflag = 1 << 4;
pub const GEO_NODE_ASSET_POINTCLOUD: Geometrynodeassettraitflag = 1 << 5;
pub const GEO_NODE_ASSET_MODIFIER: Geometrynodeassettraitflag = 1 << 6;
pub const GEO_NODE_ASSET_OBJECT: Geometrynodeassettraitflag = 1 << 7;
pub const GEO_NODE_ASSET_WAIT_FOR_CURSOR: Geometrynodeassettraitflag = 1 << 8;
pub const GEO_NODE_ASSET_GREASE_PENCIL: Geometrynodeassettraitflag = 1 << 9;
pub const GEO_NODE_ASSET_PAINT: Geometrynodeassettraitflag = 1 << 10;
pub const GEO_NODE_ASSET_HIDE_MODIFIER_MANAGE_PANEL: Geometrynodeassettraitflag = 1 << 11;

/// `CompositorNodeAssetTraitFlag`.
pub type Compositornodeassettraitflag = i32;
pub const COMPOSIT_NODE_ASSET_STRIP_MODIFIER: Compositornodeassettraitflag = 1 << 0;
pub const COMPOSIT_NODE_ASSET_SCENE_EFFECT: Compositornodeassettraitflag = 1 << 1;

/// `CMPNodeMaskType`.
pub type Cmpnodemasktype = i32;
pub const CMP_NODE_MASKTYPE_ADD: Cmpnodemasktype = 0;
pub const CMP_NODE_MASKTYPE_SUBTRACT: Cmpnodemasktype = 1;
pub const CMP_NODE_MASKTYPE_MULTIPLY: Cmpnodemasktype = 2;
pub const CMP_NODE_MASKTYPE_NOT: Cmpnodemasktype = 3;

/// `CMPNodeDilateErodeMethod`.
pub type Cmpnodedilateerodemethod = i32;
pub const CMP_NODE_DILATE_ERODE_STEP: Cmpnodedilateerodemethod = 0;
pub const CMP_NODE_DILATE_ERODE_DISTANCE_THRESHOLD: Cmpnodedilateerodemethod = 1;
pub const CMP_NODE_DILATE_ERODE_DISTANCE: Cmpnodedilateerodemethod = 2;
pub const CMP_NODE_DILATE_ERODE_DISTANCE_FEATHER: Cmpnodedilateerodemethod = 3;

/// `eNodeInpaint_Type`.
pub type NodeinpaintType = i16;
pub const CMP_NODE_INPAINT_SIMPLE: NodeinpaintType = 0;

/// `CMPNodeMaskFlags`.
pub type Cmpnodemaskflags = i32;
pub const CMP_NODE_MASK_FLAG_NO_FEATHER: Cmpnodemaskflags = 1 << 1;
pub const CMP_NODE_MASK_FLAG_MOTION_BLUR: Cmpnodemaskflags = 1 << 2;
pub const CMP_NODE_MASK_FLAG_SIZE_FIXED: Cmpnodemaskflags = 1 << 8;
pub const CMP_NODE_MASK_FLAG_SIZE_FIXED_SCENE: Cmpnodemaskflags = 1 << 9;

/// `CMPNodeGlareQuality`.
pub type Cmpnodeglarequality = i32;
pub const CMP_NODE_GLARE_QUALITY_HIGH: Cmpnodeglarequality = 0;
pub const CMP_NODE_GLARE_QUALITY_MEDIUM: Cmpnodeglarequality = 1;
pub const CMP_NODE_GLARE_QUALITY_LOW: Cmpnodeglarequality = 2;

/// `NodeGeometryViewerItemFlag`.
pub type Nodegeometryvieweritemflag = i32;
pub const NODE_GEO_VIEWER_ITEM_FLAG_AUTO_REMOVE: Nodegeometryvieweritemflag = 1 << 0;

/// `NodeClosureFlag`.
pub type Nodeclosureflag = u8;
pub const NODE_CLOSURE_FLAG_DEFINE_SIGNATURE: Nodeclosureflag = 1 << 0;

/// `NodeEvaluateClosureFlag`.
pub type Nodeevaluateclosureflag = u8;
pub const NODE_EVALUATE_CLOSURE_FLAG_DEFINE_SIGNATURE: Nodeevaluateclosureflag = 1 << 0;

/// `NodeGeometryTransformGizmoFlag`.
pub type Nodegeometrytransformgizmoflag = u32;
pub const GEO_NODE_TRANSFORM_GIZMO_USE_TRANSLATION_X: Nodegeometrytransformgizmoflag = 1 << 0;
pub const GEO_NODE_TRANSFORM_GIZMO_USE_TRANSLATION_Y: Nodegeometrytransformgizmoflag = 1 << 1;
pub const GEO_NODE_TRANSFORM_GIZMO_USE_TRANSLATION_Z: Nodegeometrytransformgizmoflag = 1 << 2;
pub const GEO_NODE_TRANSFORM_GIZMO_USE_ROTATION_X: Nodegeometrytransformgizmoflag = 1 << 3;
pub const GEO_NODE_TRANSFORM_GIZMO_USE_ROTATION_Y: Nodegeometrytransformgizmoflag = 1 << 4;
pub const GEO_NODE_TRANSFORM_GIZMO_USE_ROTATION_Z: Nodegeometrytransformgizmoflag = 1 << 5;
pub const GEO_NODE_TRANSFORM_GIZMO_USE_SCALE_X: Nodegeometrytransformgizmoflag = 1 << 6;
pub const GEO_NODE_TRANSFORM_GIZMO_USE_SCALE_Y: Nodegeometrytransformgizmoflag = 1 << 7;
pub const GEO_NODE_TRANSFORM_GIZMO_USE_SCALE_Z: Nodegeometrytransformgizmoflag = 1 << 8;

/// `NodeGeometryBakeItemFlag`.
pub type Nodegeometrybakeitemflag = i32;
pub const GEO_NODE_BAKE_ITEM_IS_ATTRIBUTE: Nodegeometrybakeitemflag = 1 << 0;

/// `NodeCombineBundleFlag`.
pub type Nodecombinebundleflag = u8;
pub const NODE_COMBINE_BUNDLE_FLAG_DEFINE_SIGNATURE: Nodecombinebundleflag = 1 << 0;

/// `NodeSeparateBundleFlag`.
pub type Nodeseparatebundleflag = u8;
pub const NODE_SEPARATE_BUNDLE_FLAG_DEFINE_SIGNATURE: Nodeseparatebundleflag = 1 << 0;

/// `eNodeScript_Mode`.
pub type NodescriptMode = i8;
pub const NODE_SCRIPT_INTERNAL: NodescriptMode = 0;
pub const NODE_SCRIPT_EXTERNAL: NodescriptMode = 1;

/// `eNodeScript_Flag`.
pub type NodescriptFlag = i8;
pub const NODE_SCRIPT_AUTO_UPDATE: NodescriptFlag = 1;

/// `eNodeIES_Mode`.
pub type NodeiesMode = i8;
pub const NODE_IES_INTERNAL: NodeiesMode = 0;
pub const NODE_IES_EXTERNAL: NodeiesMode = 1;

/// `eNodeFrame_Flag`.
pub type NodeframeFlag = i8;
pub const NODE_FRAME_SHRINK: NodeframeFlag = 1;
pub const NODE_FRAME_RESIZEABLE: NodeframeFlag = 2;

/// `eNodeProxy_Flag`.
pub type NodeproxyFlag = i8;
pub const NODE_PROXY_AUTOTYPE: NodeproxyFlag = 1;

/// `eNodeConductiveFresnel_Type`.
pub type NodeconductivefresnelType = i16;
pub const SHD_PHYSICAL_CONDUCTOR: NodeconductivefresnelType = 0;
pub const SHD_CONDUCTOR_F82: NodeconductivefresnelType = 1;

/// `eNodeGlossy_Dist`.
pub type NodeglossyDist = i16;
pub const SHD_GLOSSY_BECKMANN: NodeglossyDist = 0;
pub const SHD_GLOSSY_SHARP_DEPRECATED: NodeglossyDist = 1;
pub const SHD_GLOSSY_GGX: NodeglossyDist = 2;
pub const SHD_GLOSSY_ASHIKHMIN_SHIRLEY: NodeglossyDist = 3;
pub const SHD_GLOSSY_MULTI_GGX: NodeglossyDist = 4;

/// `eNodeLightEval_Mode`.
pub type NodelightevalMode = i16;
pub const SHD_LIGHT_EVAL_DIFFUSE: NodelightevalMode = 0;
pub const SHD_LIGHT_EVAL_GLOSSY: NodelightevalMode = 1;

/// `eNodeVectorTransform_Type`.
pub type NodevectortransformType = i16;
pub const SHD_VECT_TRANSFORM_TYPE_VECTOR: NodevectortransformType = 0;
pub const SHD_VECT_TRANSFORM_TYPE_POINT: NodevectortransformType = 1;
pub const SHD_VECT_TRANSFORM_TYPE_NORMAL: NodevectortransformType = 2;

/// `eNodeVectorTransform_Space`.
pub type NodevectortransformSpace = i16;
pub const SHD_VECT_TRANSFORM_SPACE_WORLD: NodevectortransformSpace = 0;
pub const SHD_VECT_TRANSFORM_SPACE_OBJECT: NodevectortransformSpace = 1;
pub const SHD_VECT_TRANSFORM_SPACE_CAMERA: NodevectortransformSpace = 2;
pub const SHD_VECT_TRANSFORM_SPACE_LIGHT: NodevectortransformSpace = 3;

/// `eNodeShader_AttributeType`.
pub type NodeshaderAttributetype = i16;
pub const SHD_ATTRIBUTE_GEOMETRY: NodeshaderAttributetype = 0;
pub const SHD_ATTRIBUTE_OBJECT: NodeshaderAttributetype = 1;
pub const SHD_ATTRIBUTE_INSTANCER: NodeshaderAttributetype = 2;
pub const SHD_ATTRIBUTE_VIEW_LAYER: NodeshaderAttributetype = 3;
pub const SHD_ATTRIBUTE_LIGHT: NodeshaderAttributetype = 4;

/// `eNodeToon_Mode`.
pub type NodetoonMode = i16;
pub const SHD_TOON_DIFFUSE: NodetoonMode = 0;
pub const SHD_TOON_GLOSSY: NodetoonMode = 1;

/// `eNodeHair_Component`.
pub type NodehairComponent = i16;
pub const SHD_HAIR_REFLECTION: NodehairComponent = 0;
pub const SHD_HAIR_TRANSMISSION: NodehairComponent = 1;

/// `eNodePrincipledHair_Model`.
pub type NodeprincipledhairModel = i16;
pub const SHD_PRINCIPLED_HAIR_CHIANG: NodeprincipledhairModel = 0;
pub const SHD_PRINCIPLED_HAIR_HUANG: NodeprincipledhairModel = 1;

/// `eNodePrincipledHair_Param`.
pub type NodeprincipledhairParam = i16;
pub const SHD_PRINCIPLED_HAIR_REFLECTANCE: NodeprincipledhairParam = 0;
pub const SHD_PRINCIPLED_HAIR_PIGMENT_CONCENTRATION: NodeprincipledhairParam = 1;
pub const SHD_PRINCIPLED_HAIR_DIRECT_ABSORPTION: NodeprincipledhairParam = 2;

/// `eNodeBlend_Type`.
pub type NodeblendType = i16;
pub const SHD_BLEND_LINEAR: NodeblendType = 0;
pub const SHD_BLEND_QUADRATIC: NodeblendType = 1;
pub const SHD_BLEND_EASING: NodeblendType = 2;
pub const SHD_BLEND_DIAGONAL: NodeblendType = 3;
pub const SHD_BLEND_RADIAL: NodeblendType = 4;
pub const SHD_BLEND_QUADRATIC_SPHERE: NodeblendType = 5;
pub const SHD_BLEND_SPHERICAL: NodeblendType = 6;

/// `eNodeNoise_Basis`.
pub type NodenoiseBasis = i16;
pub const SHD_NOISE_PERLIN: NodenoiseBasis = 0;
pub const SHD_NOISE_VORONOI_F1: NodenoiseBasis = 1;
pub const SHD_NOISE_VORONOI_F2: NodenoiseBasis = 2;
pub const SHD_NOISE_VORONOI_F3: NodenoiseBasis = 3;
pub const SHD_NOISE_VORONOI_F4: NodenoiseBasis = 4;
pub const SHD_NOISE_VORONOI_F2_F1: NodenoiseBasis = 5;
pub const SHD_NOISE_VORONOI_CRACKLE: NodenoiseBasis = 6;
pub const SHD_NOISE_CELL_NOISE: NodenoiseBasis = 7;

/// `eNodeNoise_Type`.
pub type NodenoiseType = i16;
pub const SHD_NOISE_SOFT: NodenoiseType = 0;
pub const SHD_NOISE_HARD: NodenoiseType = 1;

/// `eNodeVoronoi_Dist`.
pub type NodevoronoiDist = i16;
pub const SHD_VORONOI_EUCLIDEAN: NodevoronoiDist = 0;
pub const SHD_VORONOI_MANHATTAN: NodevoronoiDist = 1;
pub const SHD_VORONOI_CHEBYCHEV: NodevoronoiDist = 2;
pub const SHD_VORONOI_MINKOWSKI: NodevoronoiDist = 3;

/// `eNodeVoronoi_Type`.
pub type NodevoronoiType = i16;
pub const SHD_VORONOI_F1: NodevoronoiType = 0;
pub const SHD_VORONOI_F2: NodevoronoiType = 1;
pub const SHD_VORONOI_SMOOTH_F1: NodevoronoiType = 2;
pub const SHD_VORONOI_DISTANCE_TO_EDGE: NodevoronoiType = 3;
pub const SHD_VORONOI_N_SPHERE_RADIUS: NodevoronoiType = 4;

/// `eNodeMusgrave_Type`.
pub type NodemusgraveType = i16;
pub const SHD_MUSGRAVE_MULTIFRACTAL: NodemusgraveType = 0;
pub const SHD_MUSGRAVE_FBM: NodemusgraveType = 1;
pub const SHD_MUSGRAVE_HYBRID_MULTIFRACTAL: NodemusgraveType = 2;
pub const SHD_MUSGRAVE_RIDGED_MULTIFRACTAL: NodemusgraveType = 3;
pub const SHD_MUSGRAVE_HETERO_TERRAIN: NodemusgraveType = 4;

/// `eNodeNoiseTexture_Type`.
pub type NodenoisetextureType = i16;
pub const SHD_NOISE_MULTIFRACTAL: NodenoisetextureType = 0;
pub const SHD_NOISE_FBM: NodenoisetextureType = 1;
pub const SHD_NOISE_HYBRID_MULTIFRACTAL: NodenoisetextureType = 2;
pub const SHD_NOISE_RIDGED_MULTIFRACTAL: NodenoisetextureType = 3;
pub const SHD_NOISE_HETERO_TERRAIN: NodenoisetextureType = 4;

/// `eNodeWave_Type`.
pub type NodewaveType = i16;
pub const SHD_WAVE_BANDS: NodewaveType = 0;
pub const SHD_WAVE_RINGS: NodewaveType = 1;

/// `eNodeWave_BandsDir`.
pub type NodewaveBandsdir = i16;
pub const SHD_WAVE_BANDS_DIRECTION_X: NodewaveBandsdir = 0;
pub const SHD_WAVE_BANDS_DIRECTION_Y: NodewaveBandsdir = 1;
pub const SHD_WAVE_BANDS_DIRECTION_Z: NodewaveBandsdir = 2;
pub const SHD_WAVE_BANDS_DIRECTION_DIAGONAL: NodewaveBandsdir = 3;

/// `eNodeWave_RingsDir`.
pub type NodewaveRingsdir = i16;
pub const SHD_WAVE_RINGS_DIRECTION_X: NodewaveRingsdir = 0;
pub const SHD_WAVE_RINGS_DIRECTION_Y: NodewaveRingsdir = 1;
pub const SHD_WAVE_RINGS_DIRECTION_Z: NodewaveRingsdir = 2;
pub const SHD_WAVE_RINGS_DIRECTION_SPHERICAL: NodewaveRingsdir = 3;

/// `eNodeWave_Profile`.
pub type NodewaveProfile = i16;
pub const SHD_WAVE_PROFILE_SIN: NodewaveProfile = 0;
pub const SHD_WAVE_PROFILE_SAW: NodewaveProfile = 1;
pub const SHD_WAVE_PROFILE_TRI: NodewaveProfile = 2;

/// `eNodeSky_Type`.
pub type NodeskyType = i16;
pub const SHD_SKY_PREETHAM: NodeskyType = 0;
pub const SHD_SKY_HOSEK: NodeskyType = 1;
pub const SHD_SKY_SINGLE_SCATTERING: NodeskyType = 2;
pub const SHD_SKY_MULTIPLE_SCATTERING: NodeskyType = 3;

/// `eNodeProj_Env`.
pub type NodeprojEnv = i16;
pub const SHD_PROJ_EQUIRECTANGULAR: NodeprojEnv = 0;
pub const SHD_PROJ_MIRROR_BALL: NodeprojEnv = 1;

/// `NodeGaborType`.
pub type Nodegabortype = i8;
pub const SHD_GABOR_TYPE_2D: Nodegabortype = 0;
pub const SHD_GABOR_TYPE_3D: Nodegabortype = 1;

/// `eNodeImage_Extension`.
pub type NodeimageExtension = i16;
pub const SHD_IMAGE_EXTENSION_REPEAT: NodeimageExtension = 0;
pub const SHD_IMAGE_EXTENSION_EXTEND: NodeimageExtension = 1;
pub const SHD_IMAGE_EXTENSION_CLIP: NodeimageExtension = 2;
pub const SHD_IMAGE_EXTENSION_MIRROR: NodeimageExtension = 3;

/// `eNodeProj_Image`.
pub type NodeprojImage = i16;
pub const SHD_PROJ_FLAT: NodeprojImage = 0;
pub const SHD_PROJ_BOX: NodeprojImage = 1;
pub const SHD_PROJ_SPHERE: NodeprojImage = 2;
pub const SHD_PROJ_TUBE: NodeprojImage = 3;

/// `eNodeInterp_Image`.
pub type NodeinterpImage = i16;
pub const SHD_INTERP_LINEAR: NodeinterpImage = 0;
pub const SHD_INTERP_CLOSEST: NodeinterpImage = 1;
pub const SHD_INTERP_CUBIC: NodeinterpImage = 2;
pub const SHD_INTERP_SMART: NodeinterpImage = 3;

/// `eNodeTangent_Type`.
pub type NodetangentType = i16;
pub const SHD_TANGENT_RADIAL: NodetangentType = 0;
pub const SHD_TANGENT_UVMAP: NodetangentType = 1;

/// `eNodeTangent_Axis`.
pub type NodetangentAxis = i16;
pub const SHD_TANGENT_AXIS_X: NodetangentAxis = 0;
pub const SHD_TANGENT_AXIS_Y: NodetangentAxis = 1;
pub const SHD_TANGENT_AXIS_Z: NodetangentAxis = 2;

/// `eNodeSpace_Normal`.
pub type NodespaceNormal = i16;
pub const SHD_SPACE_TANGENT: NodespaceNormal = 0;
pub const SHD_SPACE_OBJECT: NodespaceNormal = 1;
pub const SHD_SPACE_WORLD: NodespaceNormal = 2;
pub const SHD_SPACE_BLENDER_OBJECT: NodespaceNormal = 3;
pub const SHD_SPACE_BLENDER_WORLD: NodespaceNormal = 4;

/// `eNodeNormalMap_Convention`.
pub type NodenormalmapConvention = i16;
pub const SHD_NORMAL_MAP_CONVENTION_OPENGL: NodenormalmapConvention = 0;
pub const SHD_NORMAL_MAP_CONVENTION_DIRECTX: NodenormalmapConvention = 1;

/// `eNodeNormalMap_Base`.
pub type NodenormalmapBase = i16;
pub const SHD_NORMAL_MAP_BASE_ORIGINAL: NodenormalmapBase = 0;
pub const SHD_NORMAL_MAP_BASE_DISPLACED: NodenormalmapBase = 1;

/// `eNodeAO_Flag`.
pub type NodeaoFlag = i16;
pub const SHD_AO_INSIDE: NodeaoFlag = 1;
pub const SHD_AO_LOCAL: NodeaoFlag = 2;

/// `eNodeMapping_VectorType`.
pub type NodemappingVectortype = i16;
pub const NODE_MAPPING_TYPE_POINT: NodemappingVectortype = 0;
pub const NODE_MAPPING_TYPE_TEXTURE: NodemappingVectortype = 1;
pub const NODE_MAPPING_TYPE_VECTOR: NodemappingVectortype = 2;
pub const NODE_MAPPING_TYPE_NORMAL: NodemappingVectortype = 3;

/// `eNodeVectorRotate_Type`.
pub type NodevectorrotateType = i16;
pub const NODE_VECTOR_ROTATE_TYPE_AXIS: NodevectorrotateType = 0;
pub const NODE_VECTOR_ROTATE_TYPE_AXIS_X: NodevectorrotateType = 1;
pub const NODE_VECTOR_ROTATE_TYPE_AXIS_Y: NodevectorrotateType = 2;
pub const NODE_VECTOR_ROTATE_TYPE_AXIS_Z: NodevectorrotateType = 3;
pub const NODE_VECTOR_ROTATE_TYPE_EULER_XYZ: NodevectorrotateType = 4;

/// `eNodeShader_MathClamp`.
pub type NodeshaderMathclamp = i8;
pub const SHD_MATH_CLAMP: NodeshaderMathclamp = 1;

/// `NodeMathOperation`.
pub type Nodemathoperation = i32;
pub const NODE_MATH_ADD: Nodemathoperation = 0;
pub const NODE_MATH_SUBTRACT: Nodemathoperation = 1;
pub const NODE_MATH_MULTIPLY: Nodemathoperation = 2;
pub const NODE_MATH_DIVIDE: Nodemathoperation = 3;
pub const NODE_MATH_SINE: Nodemathoperation = 4;
pub const NODE_MATH_COSINE: Nodemathoperation = 5;
pub const NODE_MATH_TANGENT: Nodemathoperation = 6;
pub const NODE_MATH_ARCSINE: Nodemathoperation = 7;
pub const NODE_MATH_ARCCOSINE: Nodemathoperation = 8;
pub const NODE_MATH_ARCTANGENT: Nodemathoperation = 9;
pub const NODE_MATH_POWER: Nodemathoperation = 10;
pub const NODE_MATH_LOGARITHM: Nodemathoperation = 11;
pub const NODE_MATH_MINIMUM: Nodemathoperation = 12;
pub const NODE_MATH_MAXIMUM: Nodemathoperation = 13;
pub const NODE_MATH_ROUND: Nodemathoperation = 14;
pub const NODE_MATH_LESS_THAN: Nodemathoperation = 15;
pub const NODE_MATH_GREATER_THAN: Nodemathoperation = 16;
pub const NODE_MATH_MODULO: Nodemathoperation = 17;
pub const NODE_MATH_ABSOLUTE: Nodemathoperation = 18;
pub const NODE_MATH_ARCTAN2: Nodemathoperation = 19;
pub const NODE_MATH_FLOOR: Nodemathoperation = 20;
pub const NODE_MATH_CEIL: Nodemathoperation = 21;
pub const NODE_MATH_FRACTION: Nodemathoperation = 22;
pub const NODE_MATH_SQRT: Nodemathoperation = 23;
pub const NODE_MATH_INV_SQRT: Nodemathoperation = 24;
pub const NODE_MATH_SIGN: Nodemathoperation = 25;
pub const NODE_MATH_EXPONENT: Nodemathoperation = 26;
pub const NODE_MATH_RADIANS: Nodemathoperation = 27;
pub const NODE_MATH_DEGREES: Nodemathoperation = 28;
pub const NODE_MATH_SINH: Nodemathoperation = 29;
pub const NODE_MATH_COSH: Nodemathoperation = 30;
pub const NODE_MATH_TANH: Nodemathoperation = 31;
pub const NODE_MATH_TRUNC: Nodemathoperation = 32;
pub const NODE_MATH_SNAP: Nodemathoperation = 33;
pub const NODE_MATH_WRAP: Nodemathoperation = 34;
pub const NODE_MATH_COMPARE: Nodemathoperation = 35;
pub const NODE_MATH_MULTIPLY_ADD: Nodemathoperation = 36;
pub const NODE_MATH_PINGPONG: Nodemathoperation = 37;
pub const NODE_MATH_SMOOTH_MIN: Nodemathoperation = 38;
pub const NODE_MATH_SMOOTH_MAX: Nodemathoperation = 39;
pub const NODE_MATH_FLOORED_MODULO: Nodemathoperation = 40;

/// `NodeVectorMathOperation`.
pub type Nodevectormathoperation = i32;
pub const NODE_VECTOR_MATH_ADD: Nodevectormathoperation = 0;
pub const NODE_VECTOR_MATH_SUBTRACT: Nodevectormathoperation = 1;
pub const NODE_VECTOR_MATH_MULTIPLY: Nodevectormathoperation = 2;
pub const NODE_VECTOR_MATH_DIVIDE: Nodevectormathoperation = 3;
pub const NODE_VECTOR_MATH_CROSS_PRODUCT: Nodevectormathoperation = 4;
pub const NODE_VECTOR_MATH_PROJECT: Nodevectormathoperation = 5;
pub const NODE_VECTOR_MATH_REFLECT: Nodevectormathoperation = 6;
pub const NODE_VECTOR_MATH_DOT_PRODUCT: Nodevectormathoperation = 7;
pub const NODE_VECTOR_MATH_DISTANCE: Nodevectormathoperation = 8;
pub const NODE_VECTOR_MATH_LENGTH: Nodevectormathoperation = 9;
pub const NODE_VECTOR_MATH_SCALE: Nodevectormathoperation = 10;
pub const NODE_VECTOR_MATH_NORMALIZE: Nodevectormathoperation = 11;
pub const NODE_VECTOR_MATH_SNAP: Nodevectormathoperation = 12;
pub const NODE_VECTOR_MATH_FLOOR: Nodevectormathoperation = 13;
pub const NODE_VECTOR_MATH_CEIL: Nodevectormathoperation = 14;
pub const NODE_VECTOR_MATH_MODULO: Nodevectormathoperation = 15;
pub const NODE_VECTOR_MATH_FRACTION: Nodevectormathoperation = 16;
pub const NODE_VECTOR_MATH_ABSOLUTE: Nodevectormathoperation = 17;
pub const NODE_VECTOR_MATH_MINIMUM: Nodevectormathoperation = 18;
pub const NODE_VECTOR_MATH_MAXIMUM: Nodevectormathoperation = 19;
pub const NODE_VECTOR_MATH_WRAP: Nodevectormathoperation = 20;
pub const NODE_VECTOR_MATH_SINE: Nodevectormathoperation = 21;
pub const NODE_VECTOR_MATH_COSINE: Nodevectormathoperation = 22;
pub const NODE_VECTOR_MATH_TANGENT: Nodevectormathoperation = 23;
pub const NODE_VECTOR_MATH_REFRACT: Nodevectormathoperation = 24;
pub const NODE_VECTOR_MATH_FACEFORWARD: Nodevectormathoperation = 25;
pub const NODE_VECTOR_MATH_MULTIPLY_ADD: Nodevectormathoperation = 26;
pub const NODE_VECTOR_MATH_POWER: Nodevectormathoperation = 27;
pub const NODE_VECTOR_MATH_SIGN: Nodevectormathoperation = 28;
pub const NODE_VECTOR_MATH_ROUND: Nodevectormathoperation = 29;

/// `NodeBooleanMathOperation`.
pub type Nodebooleanmathoperation = i32;
pub const NODE_BOOLEAN_MATH_AND: Nodebooleanmathoperation = 0;
pub const NODE_BOOLEAN_MATH_OR: Nodebooleanmathoperation = 1;
pub const NODE_BOOLEAN_MATH_NOT: Nodebooleanmathoperation = 2;
pub const NODE_BOOLEAN_MATH_NAND: Nodebooleanmathoperation = 3;
pub const NODE_BOOLEAN_MATH_NOR: Nodebooleanmathoperation = 4;
pub const NODE_BOOLEAN_MATH_XNOR: Nodebooleanmathoperation = 5;
pub const NODE_BOOLEAN_MATH_XOR: Nodebooleanmathoperation = 6;
pub const NODE_BOOLEAN_MATH_IMPLY: Nodebooleanmathoperation = 7;
pub const NODE_BOOLEAN_MATH_NIMPLY: Nodebooleanmathoperation = 8;

/// `NodeShaderMixMode`.
pub type Nodeshadermixmode = i8;
pub const NODE_MIX_MODE_UNIFORM: Nodeshadermixmode = 0;
pub const NODE_MIX_MODE_NON_UNIFORM: Nodeshadermixmode = 1;

/// `NodeCompareMode`.
pub type Nodecomparemode = i8;
pub const NODE_COMPARE_MODE_ELEMENT: Nodecomparemode = 0;
pub const NODE_COMPARE_MODE_LENGTH: Nodecomparemode = 1;
pub const NODE_COMPARE_MODE_AVERAGE: Nodecomparemode = 2;
pub const NODE_COMPARE_MODE_DOT_PRODUCT: Nodecomparemode = 3;
pub const NODE_COMPARE_MODE_DIRECTION: Nodecomparemode = 4;

/// `NodeCompareOperation`.
pub type Nodecompareoperation = i8;
pub const NODE_COMPARE_LESS_THAN: Nodecompareoperation = 0;
pub const NODE_COMPARE_LESS_EQUAL: Nodecompareoperation = 1;
pub const NODE_COMPARE_GREATER_THAN: Nodecompareoperation = 2;
pub const NODE_COMPARE_GREATER_EQUAL: Nodecompareoperation = 3;
pub const NODE_COMPARE_EQUAL: Nodecompareoperation = 4;
pub const NODE_COMPARE_NOT_EQUAL: Nodecompareoperation = 5;
pub const NODE_COMPARE_COLOR_BRIGHTER: Nodecompareoperation = 6;
pub const NODE_COMPARE_COLOR_DARKER: Nodecompareoperation = 7;

/// `NodeIntegerMathOperation`.
pub type Nodeintegermathoperation = i32;
pub const NODE_INTEGER_MATH_ADD: Nodeintegermathoperation = 0;
pub const NODE_INTEGER_MATH_SUBTRACT: Nodeintegermathoperation = 1;
pub const NODE_INTEGER_MATH_MULTIPLY: Nodeintegermathoperation = 2;
pub const NODE_INTEGER_MATH_DIVIDE: Nodeintegermathoperation = 3;
pub const NODE_INTEGER_MATH_MULTIPLY_ADD: Nodeintegermathoperation = 4;
pub const NODE_INTEGER_MATH_POWER: Nodeintegermathoperation = 5;
pub const NODE_INTEGER_MATH_FLOORED_MODULO: Nodeintegermathoperation = 6;
pub const NODE_INTEGER_MATH_ABSOLUTE: Nodeintegermathoperation = 7;
pub const NODE_INTEGER_MATH_MINIMUM: Nodeintegermathoperation = 8;
pub const NODE_INTEGER_MATH_MAXIMUM: Nodeintegermathoperation = 9;
pub const NODE_INTEGER_MATH_GCD: Nodeintegermathoperation = 10;
pub const NODE_INTEGER_MATH_LCM: Nodeintegermathoperation = 11;
pub const NODE_INTEGER_MATH_NEGATE: Nodeintegermathoperation = 12;
pub const NODE_INTEGER_MATH_SIGN: Nodeintegermathoperation = 13;
pub const NODE_INTEGER_MATH_DIVIDE_FLOOR: Nodeintegermathoperation = 14;
pub const NODE_INTEGER_MATH_DIVIDE_CEIL: Nodeintegermathoperation = 15;
pub const NODE_INTEGER_MATH_DIVIDE_ROUND: Nodeintegermathoperation = 16;
pub const NODE_INTEGER_MATH_MODULO: Nodeintegermathoperation = 17;

/// `FloatToIntRoundingMode`.
pub type Floattointroundingmode = i32;
pub const FN_NODE_FLOAT_TO_INT_ROUND: Floattointroundingmode = 0;
pub const FN_NODE_FLOAT_TO_INT_FLOOR: Floattointroundingmode = 1;
pub const FN_NODE_FLOAT_TO_INT_CEIL: Floattointroundingmode = 2;
pub const FN_NODE_FLOAT_TO_INT_TRUNCATE: Floattointroundingmode = 3;

/// `eNodeClamp_Type`.
pub type NodeclampType = i8;
pub const NODE_CLAMP_MINMAX: NodeclampType = 0;
pub const NODE_CLAMP_RANGE: NodeclampType = 1;

/// `eNodeMapRange_Type`.
pub type NodemaprangeType = i8;
pub const NODE_MAP_RANGE_LINEAR: NodemaprangeType = 0;
pub const NODE_MAP_RANGE_STEPPED: NodemaprangeType = 1;
pub const NODE_MAP_RANGE_SMOOTHSTEP: NodemaprangeType = 2;
pub const NODE_MAP_RANGE_SMOOTHERSTEP: NodemaprangeType = 3;

/// `eNodeShader_MixRgbFlag`.
pub type NodeshaderMixrgbflag = i8;
pub const SHD_MIXRGB_USE_ALPHA: NodeshaderMixrgbflag = 1;
pub const SHD_MIXRGB_CLAMP: NodeshaderMixrgbflag = 2;

/// `eNodeSubsurface_Type`.
pub type NodesubsurfaceType = i16;
pub const SHD_SUBSURFACE_COMPATIBLE: NodesubsurfaceType = 0;
pub const SHD_SUBSURFACE_CUBIC: NodesubsurfaceType = 1;
pub const SHD_SUBSURFACE_GAUSSIAN: NodesubsurfaceType = 2;
pub const SHD_SUBSURFACE_BURLEY: NodesubsurfaceType = 3;
pub const SHD_SUBSURFACE_RANDOM_WALK_LEGACY: NodesubsurfaceType = 4;
pub const SHD_SUBSURFACE_RANDOM_WALK_SKIN: NodesubsurfaceType = 5;
pub const SHD_SUBSURFACE_RANDOM_WALK: NodesubsurfaceType = 6;

/// `eNodeBlur_Aspect`.
pub type NodeblurAspect = i16;
pub const CMP_NODE_BLUR_ASPECT_NONE: NodeblurAspect = 0;
pub const CMP_NODE_BLUR_ASPECT_Y: NodeblurAspect = 1;
pub const CMP_NODE_BLUR_ASPECT_X: NodeblurAspect = 2;

/// `CMPNodeTranslateRepeatAxis`.
pub type Cmpnodetranslaterepeataxis = i32;
pub const CMP_NODE_TRANSLATE_REPEAT_AXIS_NONE: Cmpnodetranslaterepeataxis = 0;
pub const CMP_NODE_TRANSLATE_REPEAT_AXIS_X: Cmpnodetranslaterepeataxis = 1;
pub const CMP_NODE_TRANSLATE_REPEAT_AXIS_Y: Cmpnodetranslaterepeataxis = 2;
pub const CMP_NODE_TRANSLATE_REPEAT_AXIS_XY: Cmpnodetranslaterepeataxis = 3;

/// `CMPExtensionMode`.
pub type Cmpextensionmode = i32;
pub const CMP_NODE_EXTENSION_MODE_CLIP: Cmpextensionmode = 0;
pub const CMP_NODE_EXTENSION_MODE_EXTEND: Cmpextensionmode = 1;
pub const CMP_NODE_EXTENSION_MODE_REPEAT: Cmpextensionmode = 2;

/// `eNodeCompositor_OutputFlag`.
pub type NodecompositorOutputflag = i8;
pub const CMP_NODE_OUTPUT_IGNORE_ALPHA: NodecompositorOutputflag = 1;

/// `CMPNodeColorBalanceMethod`.
pub type Cmpnodecolorbalancemethod = i32;
pub const CMP_NODE_COLOR_BALANCE_LGG: Cmpnodecolorbalancemethod = 0;
pub const CMP_NODE_COLOR_BALANCE_ASC_CDL: Cmpnodecolorbalancemethod = 1;
pub const CMP_NODE_COLOR_BALANCE_WHITEPOINT: Cmpnodecolorbalancemethod = 2;

/// `CMPNodeAlphaConvertMode`.
pub type Cmpnodealphaconvertmode = i32;
pub const CMP_NODE_ALPHA_CONVERT_PREMULTIPLY: Cmpnodealphaconvertmode = 0;
pub const CMP_NODE_ALPHA_CONVERT_UNPREMULTIPLY: Cmpnodealphaconvertmode = 1;

/// `CMPNodeDistanceMatteColorSpace`.
pub type Cmpnodedistancemattecolorspace = i32;
pub const CMP_NODE_DISTANCE_MATTE_COLOR_SPACE_RGBA: Cmpnodedistancemattecolorspace = 0;
pub const CMP_NODE_DISTANCE_MATTE_COLOR_SPACE_YCCA: Cmpnodedistancemattecolorspace = 1;

/// `CMPNodeColorSpillLimitAlgorithm`.
pub type Cmpnodecolorspilllimitalgorithm = i32;
pub const CMP_NODE_COLOR_SPILL_LIMIT_ALGORITHM_SINGLE: Cmpnodecolorspilllimitalgorithm = 0;
pub const CMP_NODE_COLOR_SPILL_LIMIT_ALGORITHM_AVERAGE: Cmpnodecolorspilllimitalgorithm = 1;

/// `CMPNodeChannelMatteLimitAlgorithm`.
pub type Cmpnodechannelmattelimitalgorithm = i32;
pub const CMP_NODE_CHANNEL_MATTE_LIMIT_ALGORITHM_SINGLE: Cmpnodechannelmattelimitalgorithm = 0;
pub const CMP_NODE_CHANNEL_MATTE_LIMIT_ALGORITHM_MAX: Cmpnodechannelmattelimitalgorithm = 1;

/// `CMPNodeFlipMode`.
pub type Cmpnodeflipmode = i32;
pub const CMP_NODE_FLIP_X: Cmpnodeflipmode = 0;
pub const CMP_NODE_FLIP_Y: Cmpnodeflipmode = 1;
pub const CMP_NODE_FLIP_X_Y: Cmpnodeflipmode = 2;

/// `CMPNodeScaleMethod`.
pub type Cmpnodescalemethod = i32;
pub const CMP_NODE_SCALE_RELATIVE: Cmpnodescalemethod = 0;
pub const CMP_NODE_SCALE_ABSOLUTE: Cmpnodescalemethod = 1;
pub const CMP_NODE_SCALE_RENDER_PERCENT: Cmpnodescalemethod = 2;
pub const CMP_NODE_SCALE_RENDER_SIZE: Cmpnodescalemethod = 3;

/// `CMPNodeScaleRenderSizeMethod`.
pub type Cmpnodescalerendersizemethod = i32;
pub const CMP_NODE_SCALE_RENDER_SIZE_STRETCH: Cmpnodescalerendersizemethod = 0;
pub const CMP_NODE_SCALE_RENDER_SIZE_FIT: Cmpnodescalerendersizemethod = 1;
pub const CMP_NODE_SCALE_RENDER_SIZE_CROP: Cmpnodescalerendersizemethod = 2;

/// `CMPNodeFilterMethod`.
pub type Cmpnodefiltermethod = i32;
pub const CMP_NODE_FILTER_SOFT: Cmpnodefiltermethod = 0;
pub const CMP_NODE_FILTER_SHARP_BOX: Cmpnodefiltermethod = 1;
pub const CMP_NODE_FILTER_LAPLACE: Cmpnodefiltermethod = 2;
pub const CMP_NODE_FILTER_SOBEL: Cmpnodefiltermethod = 3;
pub const CMP_NODE_FILTER_PREWITT: Cmpnodefiltermethod = 4;
pub const CMP_NODE_FILTER_KIRSCH: Cmpnodefiltermethod = 5;
pub const CMP_NODE_FILTER_SHADOW: Cmpnodefiltermethod = 6;
pub const CMP_NODE_FILTER_SHARP_DIAMOND: Cmpnodefiltermethod = 7;

/// `CMPNodeLevelsChannel`.
pub type Cmpnodelevelschannel = i32;
pub const CMP_NODE_LEVLES_LUMINANCE: Cmpnodelevelschannel = 1;
pub const CMP_NODE_LEVLES_RED: Cmpnodelevelschannel = 2;
pub const CMP_NODE_LEVLES_GREEN: Cmpnodelevelschannel = 3;
pub const CMP_NODE_LEVLES_BLUE: Cmpnodelevelschannel = 4;
pub const CMP_NODE_LEVLES_LUMINANCE_BT709: Cmpnodelevelschannel = 5;

/// `CMPNodeToneMapType`.
pub type Cmpnodetonemaptype = i32;
pub const CMP_NODE_TONE_MAP_SIMPLE: Cmpnodetonemaptype = 0;
pub const CMP_NODE_TONE_MAP_PHOTORECEPTOR: Cmpnodetonemaptype = 1;

/// `CMPNodeTrackPositionMode`.
pub type Cmpnodetrackpositionmode = i32;
pub const CMP_NODE_TRACK_POSITION_ABSOLUTE: Cmpnodetrackpositionmode = 0;
pub const CMP_NODE_TRACK_POSITION_RELATIVE_START: Cmpnodetrackpositionmode = 1;
pub const CMP_NODE_TRACK_POSITION_RELATIVE_FRAME: Cmpnodetrackpositionmode = 2;
pub const CMP_NODE_TRACK_POSITION_ABSOLUTE_FRAME: Cmpnodetrackpositionmode = 3;

/// `CMPNodeGlareType`.
pub type Cmpnodeglaretype = i32;
pub const CMP_NODE_GLARE_SIMPLE_STAR: Cmpnodeglaretype = 0;
pub const CMP_NODE_GLARE_FOG_GLOW: Cmpnodeglaretype = 1;
pub const CMP_NODE_GLARE_STREAKS: Cmpnodeglaretype = 2;
pub const CMP_NODE_GLARE_GHOST: Cmpnodeglaretype = 3;
pub const CMP_NODE_GLARE_BLOOM: Cmpnodeglaretype = 4;
pub const CMP_NODE_GLARE_SUN_BEAMS: Cmpnodeglaretype = 5;
pub const CMP_NODE_GLARE_KERNEL: Cmpnodeglaretype = 6;

/// `CMPNodeKuwahara`.
pub type Cmpnodekuwahara = i32;
pub const CMP_NODE_KUWAHARA_CLASSIC: Cmpnodekuwahara = 0;
pub const CMP_NODE_KUWAHARA_ANISOTROPIC: Cmpnodekuwahara = 1;

/// `CMPNodeInterpolation`.
pub type Cmpnodeinterpolation = i32;
pub const CMP_NODE_INTERPOLATION_NEAREST: Cmpnodeinterpolation = 0;
pub const CMP_NODE_INTERPOLATION_BILINEAR: Cmpnodeinterpolation = 1;
pub const CMP_NODE_INTERPOLATION_BICUBIC: Cmpnodeinterpolation = 2;
pub const CMP_NODE_INTERPOLATION_ANISOTROPIC: Cmpnodeinterpolation = 3;

/// `CMPNodeSetAlphaMode`.
pub type Cmpnodesetalphamode = i32;
pub const CMP_NODE_SETALPHA_MODE_APPLY: Cmpnodesetalphamode = 0;
pub const CMP_NODE_SETALPHA_MODE_REPLACE_ALPHA: Cmpnodesetalphamode = 1;

/// `CMPNodeBlurType`.
pub type Cmpnodeblurtype = i32;
pub const CMP_NODE_BLUR_TYPE_BOX: Cmpnodeblurtype = 0;
pub const CMP_NODE_BLUR_TYPE_TENT: Cmpnodeblurtype = 1;
pub const CMP_NODE_BLUR_TYPE_QUAD: Cmpnodeblurtype = 2;
pub const CMP_NODE_BLUR_TYPE_CUBIC: Cmpnodeblurtype = 3;
pub const CMP_NODE_BLUR_TYPE_CATROM: Cmpnodeblurtype = 4;
pub const CMP_NODE_BLUR_TYPE_GAUSS: Cmpnodeblurtype = 5;
pub const CMP_NODE_BLUR_TYPE_MITCH: Cmpnodeblurtype = 6;
pub const CMP_NODE_BLUR_TYPE_FAST_GAUSS: Cmpnodeblurtype = 7;

/// `CMPNodeDenoisePrefilter`.
pub type Cmpnodedenoiseprefilter = i32;
pub const CMP_NODE_DENOISE_PREFILTER_FAST: Cmpnodedenoiseprefilter = 0;
pub const CMP_NODE_DENOISE_PREFILTER_NONE: Cmpnodedenoiseprefilter = 1;
pub const CMP_NODE_DENOISE_PREFILTER_ACCURATE: Cmpnodedenoiseprefilter = 2;

/// `CMPNodeDenoiseQuality`.
pub type Cmpnodedenoisequality = i32;
pub const CMP_NODE_DENOISE_QUALITY_SCENE: Cmpnodedenoisequality = 0;
pub const CMP_NODE_DENOISE_QUALITY_HIGH: Cmpnodedenoisequality = 1;
pub const CMP_NODE_DENOISE_QUALITY_BALANCED: Cmpnodedenoisequality = 2;
pub const CMP_NODE_DENOISE_QUALITY_FAST: Cmpnodedenoisequality = 3;

/// `CMPNodeCombSepColorMode`.
pub type Cmpnodecombsepcolormode = u8;
pub const CMP_NODE_COMBSEP_COLOR_RGB: Cmpnodecombsepcolormode = 0;
pub const CMP_NODE_COMBSEP_COLOR_HSV: Cmpnodecombsepcolormode = 1;
pub const CMP_NODE_COMBSEP_COLOR_HSL: Cmpnodecombsepcolormode = 2;
pub const CMP_NODE_COMBSEP_COLOR_YCC: Cmpnodecombsepcolormode = 3;
pub const CMP_NODE_COMBSEP_COLOR_YUV: Cmpnodecombsepcolormode = 4;

/// `CMPNodeCryptomatteSource`.
pub type Cmpnodecryptomattesource = i32;
pub const CMP_NODE_CRYPTOMATTE_SOURCE_RENDER: Cmpnodecryptomattesource = 0;
pub const CMP_NODE_CRYPTOMATTE_SOURCE_IMAGE: Cmpnodecryptomattesource = 1;

/// `CMPNodeChannelMatteColorSpace`.
pub type Cmpnodechannelmattecolorspace = i32;
pub const CMP_NODE_CHANNEL_MATTE_CS_RGB: Cmpnodechannelmattecolorspace = 0;
pub const CMP_NODE_CHANNEL_MATTE_CS_HSV: Cmpnodechannelmattecolorspace = 1;
pub const CMP_NODE_CHANNEL_MATTE_CS_YUV: Cmpnodechannelmattecolorspace = 2;
pub const CMP_NODE_CHANNEL_MATTE_CS_YCC: Cmpnodechannelmattecolorspace = 3;

/// `CMPNodeLensDistortionType`.
pub type Cmpnodelensdistortiontype = i32;
pub const CMP_NODE_LENS_DISTORTION_RADIAL: Cmpnodelensdistortiontype = 0;
pub const CMP_NODE_LENS_DISTORTION_HORIZONTAL: Cmpnodelensdistortiontype = 1;

/// `CMPNodeAlphaOverOperationType`.
pub type Cmpnodealphaoveroperationtype = i32;
pub const CMP_NODE_ALPHA_OVER_OPERATION_TYPE_OVER: Cmpnodealphaoveroperationtype = 0;
pub const CMP_NODE_ALPHA_OVER_OPERATION_TYPE_DISJOINT_OVER: Cmpnodealphaoveroperationtype = 1;
pub const CMP_NODE_ALPHA_OVER_OPERATION_TYPE_CONJOINT_OVER: Cmpnodealphaoveroperationtype = 2;

/// `CMPNodeRelativeToPixelDataType`.
pub type Cmpnoderelativetopixeldatatype = i32;
pub const CMP_NODE_RELATIVE_TO_PIXEL_DATA_TYPE_FLOAT: Cmpnoderelativetopixeldatatype = 0;
pub const CMP_NODE_RELATIVE_TO_PIXEL_DATA_TYPE_VECTOR: Cmpnoderelativetopixeldatatype = 1;

/// `CMPNodeRelativeToPixelReferenceDimension`.
pub type Cmpnoderelativetopixelreferencedimension = i32;
pub const CMP_NODE_RELATIVE_TO_PIXEL_REFERENCE_DIMENSION_PER_DIMENSION: Cmpnoderelativetopixelreferencedimension = 0;
pub const CMP_NODE_RELATIVE_TO_PIXEL_REFERENCE_DIMENSION_X: Cmpnoderelativetopixelreferencedimension = 1;
pub const CMP_NODE_RELATIVE_TO_PIXEL_REFERENCE_DIMENSION_Y: Cmpnoderelativetopixelreferencedimension = 2;
pub const CMP_NODE_RELATIVE_TO_PIXEL_REFERENCE_DIMENSION_GREATER: Cmpnoderelativetopixelreferencedimension = 3;
pub const CMP_NODE_RELATIVE_TO_PIXEL_REFERENCE_DIMENSION_SMALLER: Cmpnoderelativetopixelreferencedimension = 4;
pub const CMP_NODE_RELATIVE_TO_PIXEL_REFERENCE_DIMENSION_DIAGONAL: Cmpnoderelativetopixelreferencedimension = 5;

/// `CMPNodeStringToImageHorizontalAlignment`.
pub type Cmpnodestringtoimagehorizontalalignment = i32;
pub const CMP_NODE_STRING_TO_IMAGE_HORIZONTAL_ALIGNMENT_LEFT: Cmpnodestringtoimagehorizontalalignment = 0;
pub const CMP_NODE_STRING_TO_IMAGE_HORIZONTAL_ALIGNMENT_CENTER: Cmpnodestringtoimagehorizontalalignment = 1;
pub const CMP_NODE_STRING_TO_IMAGE_HORIZONTAL_ALIGNMENT_RIGHT: Cmpnodestringtoimagehorizontalalignment = 2;

/// `CMPNodeStringToImageVerticalAlignment`.
pub type Cmpnodestringtoimageverticalalignment = i32;
pub const CMP_NODE_STRING_TO_IMAGE_VERTICAL_ALIGNMENT_TOP: Cmpnodestringtoimageverticalalignment = 0;
pub const CMP_NODE_STRING_TO_IMAGE_VERTICAL_ALIGNMENT_TOP_BASELINE: Cmpnodestringtoimageverticalalignment = 1;
pub const CMP_NODE_STRING_TO_IMAGE_VERTICAL_ALIGNMENT_MIDDLE: Cmpnodestringtoimageverticalalignment = 2;
pub const CMP_NODE_STRING_TO_IMAGE_VERTICAL_ALIGNMENT_BOTTOM_BASELINE: Cmpnodestringtoimageverticalalignment = 3;
pub const CMP_NODE_STRING_TO_IMAGE_VERTICAL_ALIGNMENT_BOTTOM: Cmpnodestringtoimageverticalalignment = 4;

/// `eNodeScattering_PhaseFunc`.
pub type NodescatteringPhasefunc = i16;
pub const SHD_PHASE_HENYEY_GREENSTEIN: NodescatteringPhasefunc = 0;
pub const SHD_PHASE_FOURNIER_FORAND: NodescatteringPhasefunc = 1;
pub const SHD_PHASE_DRAINE: NodescatteringPhasefunc = 2;
pub const SHD_PHASE_RAYLEIGH: NodescatteringPhasefunc = 3;
pub const SHD_PHASE_MIE: NodescatteringPhasefunc = 4;

/// `NodeShaderOutputTarget`.
pub type Nodeshaderoutputtarget = i32;
pub const SHD_OUTPUT_ALL: Nodeshaderoutputtarget = 0;
pub const SHD_OUTPUT_EEVEE: Nodeshaderoutputtarget = 1;
pub const SHD_OUTPUT_CYCLES: Nodeshaderoutputtarget = 2;

/// `GeometryNodeProximityTargetType`.
pub type Geometrynodeproximitytargettype = u8;
pub const GEO_NODE_PROX_TARGET_POINTS: Geometrynodeproximitytargettype = 0;
pub const GEO_NODE_PROX_TARGET_EDGES: Geometrynodeproximitytargettype = 1;
pub const GEO_NODE_PROX_TARGET_FACES: Geometrynodeproximitytargettype = 2;

/// `GeometryNodeCurvePrimitiveCircleMode`.
pub type Geometrynodecurveprimitivecirclemode = i32;
pub const GEO_NODE_CURVE_PRIMITIVE_CIRCLE_TYPE_POINTS: Geometrynodecurveprimitivecirclemode = 0;
pub const GEO_NODE_CURVE_PRIMITIVE_CIRCLE_TYPE_RADIUS: Geometrynodecurveprimitivecirclemode = 1;

/// `GeometryNodeCurveHandleType`.
pub type Geometrynodecurvehandletype = u8;
pub const GEO_NODE_CURVE_HANDLE_FREE: Geometrynodecurvehandletype = 0;
pub const GEO_NODE_CURVE_HANDLE_AUTO: Geometrynodecurvehandletype = 1;
pub const GEO_NODE_CURVE_HANDLE_VECTOR: Geometrynodecurvehandletype = 2;
pub const GEO_NODE_CURVE_HANDLE_ALIGN: Geometrynodecurvehandletype = 3;

/// `GeometryNodeCurveHandleMode`.
pub type Geometrynodecurvehandlemode = u8;
pub const GEO_NODE_CURVE_HANDLE_LEFT: Geometrynodecurvehandlemode = 1 << 0;
pub const GEO_NODE_CURVE_HANDLE_RIGHT: Geometrynodecurvehandlemode = 1 << 1;

/// `GeometryNodeDistributePointsInVolumeMode`.
pub type Geometrynodedistributepointsinvolumemode = i32;
pub const GEO_NODE_DISTRIBUTE_POINTS_IN_VOLUME_DENSITY_RANDOM: Geometrynodedistributepointsinvolumemode = 0;
pub const GEO_NODE_DISTRIBUTE_POINTS_IN_VOLUME_DENSITY_GRID: Geometrynodedistributepointsinvolumemode = 1;

/// `GeometryNodeDistributePointsOnFacesMode`.
pub type Geometrynodedistributepointsonfacesmode = i32;
pub const GEO_NODE_POINT_DISTRIBUTE_POINTS_ON_FACES_RANDOM: Geometrynodedistributepointsonfacesmode = 0;
pub const GEO_NODE_POINT_DISTRIBUTE_POINTS_ON_FACES_POISSON: Geometrynodedistributepointsonfacesmode = 1;

/// `GeometryNodeExtrudeMeshMode`.
pub type Geometrynodeextrudemeshmode = u8;
pub const GEO_NODE_EXTRUDE_MESH_VERTICES: Geometrynodeextrudemeshmode = 0;
pub const GEO_NODE_EXTRUDE_MESH_EDGES: Geometrynodeextrudemeshmode = 1;
pub const GEO_NODE_EXTRUDE_MESH_FACES: Geometrynodeextrudemeshmode = 2;

/// `FunctionNodeRotateEulerType`.
pub type Functionnoderotateeulertype = i32;
pub const FN_NODE_ROTATE_EULER_TYPE_EULER: Functionnoderotateeulertype = 0;
pub const FN_NODE_ROTATE_EULER_TYPE_AXIS_ANGLE: Functionnoderotateeulertype = 1;

/// `FunctionNodeRotateEulerSpace`.
pub type Functionnoderotateeulerspace = i32;
pub const FN_NODE_ROTATE_EULER_SPACE_OBJECT: Functionnoderotateeulerspace = 0;
pub const FN_NODE_ROTATE_EULER_SPACE_LOCAL: Functionnoderotateeulerspace = 1;

/// `NodeAlignEulerToVectorAxis`.
pub type Nodealigneulertovectoraxis = i32;
pub const FN_NODE_ALIGN_EULER_TO_VECTOR_AXIS_X: Nodealigneulertovectoraxis = 0;
pub const FN_NODE_ALIGN_EULER_TO_VECTOR_AXIS_Y: Nodealigneulertovectoraxis = 1;
pub const FN_NODE_ALIGN_EULER_TO_VECTOR_AXIS_Z: Nodealigneulertovectoraxis = 2;

/// `NodeAlignEulerToVectorPivotAxis`.
pub type Nodealigneulertovectorpivotaxis = i32;
pub const FN_NODE_ALIGN_EULER_TO_VECTOR_PIVOT_AXIS_AUTO: Nodealigneulertovectorpivotaxis = 0;
pub const FN_NODE_ALIGN_EULER_TO_VECTOR_PIVOT_AXIS_X: Nodealigneulertovectorpivotaxis = 1;
pub const FN_NODE_ALIGN_EULER_TO_VECTOR_PIVOT_AXIS_Y: Nodealigneulertovectorpivotaxis = 2;
pub const FN_NODE_ALIGN_EULER_TO_VECTOR_PIVOT_AXIS_Z: Nodealigneulertovectorpivotaxis = 3;

/// `GeometryNodeTransformSpace`.
pub type Geometrynodetransformspace = u8;
pub const GEO_NODE_TRANSFORM_SPACE_ORIGINAL: Geometrynodetransformspace = 0;
pub const GEO_NODE_TRANSFORM_SPACE_RELATIVE: Geometrynodetransformspace = 1;

/// `GeometryNodePointsToVolumeResolutionMode`.
pub type Geometrynodepointstovolumeresolutionmode = u8;
pub const GEO_NODE_POINTS_TO_VOLUME_RESOLUTION_MODE_AMOUNT: Geometrynodepointstovolumeresolutionmode = 0;
pub const GEO_NODE_POINTS_TO_VOLUME_RESOLUTION_MODE_SIZE: Geometrynodepointstovolumeresolutionmode = 1;

/// `GeometryNodeMeshCircleFillType`.
pub type Geometrynodemeshcirclefilltype = u8;
pub const GEO_NODE_MESH_CIRCLE_FILL_NONE: Geometrynodemeshcirclefilltype = 0;
pub const GEO_NODE_MESH_CIRCLE_FILL_NGON: Geometrynodemeshcirclefilltype = 1;
pub const GEO_NODE_MESH_CIRCLE_FILL_TRIANGLE_FAN: Geometrynodemeshcirclefilltype = 2;

/// `GeometryNodeMergeByDistanceMode`.
pub type Geometrynodemergebydistancemode = u8;
pub const GEO_NODE_MERGE_BY_DISTANCE_MODE_ALL: Geometrynodemergebydistancemode = 0;
pub const GEO_NODE_MERGE_BY_DISTANCE_MODE_CONNECTED: Geometrynodemergebydistancemode = 1;

/// `GeometryNodeUVUnwrapMethod`.
pub type Geometrynodeuvunwrapmethod = u8;
pub const GEO_NODE_UV_UNWRAP_METHOD_ANGLE_BASED: Geometrynodeuvunwrapmethod = 0;
pub const GEO_NODE_UV_UNWRAP_METHOD_CONFORMAL: Geometrynodeuvunwrapmethod = 1;
pub const GEO_NODE_UV_UNWRAP_METHOD_MINIMUM_STRETCH: Geometrynodeuvunwrapmethod = 2;

/// `GeometryNodeRealizeInstanceFlag`.
pub type Geometrynoderealizeinstanceflag = i32;
pub const GEO_NODE_REALIZE_TO_POINT_DOMAIN: Geometrynoderealizeinstanceflag = 1 << 0;

/// `GeometryNodeMeshLineMode`.
pub type Geometrynodemeshlinemode = u8;
pub const GEO_NODE_MESH_LINE_MODE_END_POINTS: Geometrynodemeshlinemode = 0;
pub const GEO_NODE_MESH_LINE_MODE_OFFSET: Geometrynodemeshlinemode = 1;

/// `GeometryNodeMeshLineCountMode`.
pub type Geometrynodemeshlinecountmode = u8;
pub const GEO_NODE_MESH_LINE_COUNT_TOTAL: Geometrynodemeshlinecountmode = 0;
pub const GEO_NODE_MESH_LINE_COUNT_RESOLUTION: Geometrynodemeshlinecountmode = 1;

/// `GeometryNodeCurvePrimitiveArcMode`.
pub type Geometrynodecurveprimitivearcmode = u8;
pub const GEO_NODE_CURVE_PRIMITIVE_ARC_TYPE_POINTS: Geometrynodecurveprimitivearcmode = 0;
pub const GEO_NODE_CURVE_PRIMITIVE_ARC_TYPE_RADIUS: Geometrynodecurveprimitivearcmode = 1;

/// `GeometryNodeCurvePrimitiveLineMode`.
pub type Geometrynodecurveprimitivelinemode = u8;
pub const GEO_NODE_CURVE_PRIMITIVE_LINE_MODE_POINTS: Geometrynodecurveprimitivelinemode = 0;
pub const GEO_NODE_CURVE_PRIMITIVE_LINE_MODE_DIRECTION: Geometrynodecurveprimitivelinemode = 1;

/// `GeometryNodeCurvePrimitiveQuadMode`.
pub type Geometrynodecurveprimitivequadmode = u8;
pub const GEO_NODE_CURVE_PRIMITIVE_QUAD_MODE_RECTANGLE: Geometrynodecurveprimitivequadmode = 0;
pub const GEO_NODE_CURVE_PRIMITIVE_QUAD_MODE_PARALLELOGRAM: Geometrynodecurveprimitivequadmode = 1;
pub const GEO_NODE_CURVE_PRIMITIVE_QUAD_MODE_TRAPEZOID: Geometrynodecurveprimitivequadmode = 2;
pub const GEO_NODE_CURVE_PRIMITIVE_QUAD_MODE_KITE: Geometrynodecurveprimitivequadmode = 3;
pub const GEO_NODE_CURVE_PRIMITIVE_QUAD_MODE_POINTS: Geometrynodecurveprimitivequadmode = 4;

/// `GeometryNodeCurvePrimitiveBezierSegmentMode`.
pub type Geometrynodecurveprimitivebeziersegmentmode = u8;
pub const GEO_NODE_CURVE_PRIMITIVE_BEZIER_SEGMENT_POSITION: Geometrynodecurveprimitivebeziersegmentmode = 0;
pub const GEO_NODE_CURVE_PRIMITIVE_BEZIER_SEGMENT_OFFSET: Geometrynodecurveprimitivebeziersegmentmode = 1;

/// `GeometryNodeCurveResampleMode`.
pub type Geometrynodecurveresamplemode = u8;
pub const GEO_NODE_CURVE_RESAMPLE_COUNT: Geometrynodecurveresamplemode = 0;
pub const GEO_NODE_CURVE_RESAMPLE_LENGTH: Geometrynodecurveresamplemode = 1;
pub const GEO_NODE_CURVE_RESAMPLE_EVALUATED: Geometrynodecurveresamplemode = 2;

/// `GeometryNodeCurveSampleMode`.
pub type Geometrynodecurvesamplemode = u8;
pub const GEO_NODE_CURVE_SAMPLE_FACTOR: Geometrynodecurvesamplemode = 0;
pub const GEO_NODE_CURVE_SAMPLE_LENGTH: Geometrynodecurvesamplemode = 1;

/// `GeometryNodeCurveFilletMode`.
pub type Geometrynodecurvefilletmode = u8;
pub const GEO_NODE_CURVE_FILLET_BEZIER: Geometrynodecurvefilletmode = 0;
pub const GEO_NODE_CURVE_FILLET_POLY: Geometrynodecurvefilletmode = 1;

/// `GeometryNodeAttributeTransferMode`.
pub type Geometrynodeattributetransfermode = u8;
pub const GEO_NODE_ATTRIBUTE_TRANSFER_NEAREST_FACE_INTERPOLATED: Geometrynodeattributetransfermode = 0;
pub const GEO_NODE_ATTRIBUTE_TRANSFER_NEAREST: Geometrynodeattributetransfermode = 1;
pub const GEO_NODE_ATTRIBUTE_TRANSFER_INDEX: Geometrynodeattributetransfermode = 2;

/// `GeometryNodeRaycastMapMode`.
pub type Geometrynoderaycastmapmode = u8;
pub const GEO_NODE_RAYCAST_INTERPOLATED: Geometrynoderaycastmapmode = 0;
pub const GEO_NODE_RAYCAST_NEAREST: Geometrynoderaycastmapmode = 1;

/// `GeometryNodeCurveFillMode`.
pub type Geometrynodecurvefillmode = u8;
pub const GEO_NODE_CURVE_FILL_MODE_TRIANGULATED: Geometrynodecurvefillmode = 0;
pub const GEO_NODE_CURVE_FILL_MODE_NGONS: Geometrynodecurvefillmode = 1;

/// `GeometryNodeCurveFillRule`.
pub type Geometrynodecurvefillrule = u8;
pub const GEO_NODE_CURVE_FILL_RULE_EVEN_ODD: Geometrynodecurvefillrule = 0;
pub const GEO_NODE_CURVE_FILL_RULE_NON_ZERO: Geometrynodecurvefillrule = 1;

/// `GeometryNodeMeshToPointsMode`.
pub type Geometrynodemeshtopointsmode = u8;
pub const GEO_NODE_MESH_TO_POINTS_VERTICES: Geometrynodemeshtopointsmode = 0;
pub const GEO_NODE_MESH_TO_POINTS_EDGES: Geometrynodemeshtopointsmode = 1;
pub const GEO_NODE_MESH_TO_POINTS_FACES: Geometrynodemeshtopointsmode = 2;
pub const GEO_NODE_MESH_TO_POINTS_CORNERS: Geometrynodemeshtopointsmode = 3;

/// `GeometryNodeStringToCurvesOverflowMode`.
pub type Geometrynodestringtocurvesoverflowmode = u8;
pub const GEO_NODE_STRING_TO_CURVES_MODE_OVERFLOW: Geometrynodestringtocurvesoverflowmode = 0;
pub const GEO_NODE_STRING_TO_CURVES_MODE_SCALE_TO_FIT: Geometrynodestringtocurvesoverflowmode = 1;
pub const GEO_NODE_STRING_TO_CURVES_MODE_TRUNCATE: Geometrynodestringtocurvesoverflowmode = 2;

/// `GeometryNodeStringToCurvesAlignXMode`.
pub type Geometrynodestringtocurvesalignxmode = u8;
pub const GEO_NODE_STRING_TO_CURVES_ALIGN_X_LEFT: Geometrynodestringtocurvesalignxmode = 0;
pub const GEO_NODE_STRING_TO_CURVES_ALIGN_X_CENTER: Geometrynodestringtocurvesalignxmode = 1;
pub const GEO_NODE_STRING_TO_CURVES_ALIGN_X_RIGHT: Geometrynodestringtocurvesalignxmode = 2;
pub const GEO_NODE_STRING_TO_CURVES_ALIGN_X_JUSTIFY: Geometrynodestringtocurvesalignxmode = 3;
pub const GEO_NODE_STRING_TO_CURVES_ALIGN_X_FLUSH: Geometrynodestringtocurvesalignxmode = 4;

/// `GeometryNodeStringToCurvesAlignYMode`.
pub type Geometrynodestringtocurvesalignymode = u8;
pub const GEO_NODE_STRING_TO_CURVES_ALIGN_Y_TOP_BASELINE: Geometrynodestringtocurvesalignymode = 0;
pub const GEO_NODE_STRING_TO_CURVES_ALIGN_Y_TOP: Geometrynodestringtocurvesalignymode = 1;
pub const GEO_NODE_STRING_TO_CURVES_ALIGN_Y_MIDDLE: Geometrynodestringtocurvesalignymode = 2;
pub const GEO_NODE_STRING_TO_CURVES_ALIGN_Y_BOTTOM_BASELINE: Geometrynodestringtocurvesalignymode = 3;
pub const GEO_NODE_STRING_TO_CURVES_ALIGN_Y_BOTTOM: Geometrynodestringtocurvesalignymode = 4;

/// `GeometryNodeStringToCurvesPivotMode`.
pub type Geometrynodestringtocurvespivotmode = u8;
pub const GEO_NODE_STRING_TO_CURVES_PIVOT_MODE_MIDPOINT: Geometrynodestringtocurvespivotmode = 0;
pub const GEO_NODE_STRING_TO_CURVES_PIVOT_MODE_TOP_LEFT: Geometrynodestringtocurvespivotmode = 1;
pub const GEO_NODE_STRING_TO_CURVES_PIVOT_MODE_TOP_CENTER: Geometrynodestringtocurvespivotmode = 2;
pub const GEO_NODE_STRING_TO_CURVES_PIVOT_MODE_TOP_RIGHT: Geometrynodestringtocurvespivotmode = 3;
pub const GEO_NODE_STRING_TO_CURVES_PIVOT_MODE_BOTTOM_LEFT: Geometrynodestringtocurvespivotmode = 4;
pub const GEO_NODE_STRING_TO_CURVES_PIVOT_MODE_BOTTOM_CENTER: Geometrynodestringtocurvespivotmode = 5;
pub const GEO_NODE_STRING_TO_CURVES_PIVOT_MODE_BOTTOM_RIGHT: Geometrynodestringtocurvespivotmode = 6;

/// `GeometryNodeDeleteGeometryMode`.
pub type Geometrynodedeletegeometrymode = i8;
pub const GEO_NODE_DELETE_GEOMETRY_MODE_ALL: Geometrynodedeletegeometrymode = 0;
pub const GEO_NODE_DELETE_GEOMETRY_MODE_EDGE_FACE: Geometrynodedeletegeometrymode = 1;
pub const GEO_NODE_DELETE_GEOMETRY_MODE_ONLY_FACE: Geometrynodedeletegeometrymode = 2;

/// `GeometryNodeScaleElementsMode`.
pub type Geometrynodescaleelementsmode = i32;
pub const GEO_NODE_SCALE_ELEMENTS_UNIFORM: Geometrynodescaleelementsmode = 0;
pub const GEO_NODE_SCALE_ELEMENTS_SINGLE_AXIS: Geometrynodescaleelementsmode = 1;

/// `NodeCombSepColorMode`.
pub type Nodecombsepcolormode = i8;
pub const NODE_COMBSEP_COLOR_RGB: Nodecombsepcolormode = 0;
pub const NODE_COMBSEP_COLOR_HSV: Nodecombsepcolormode = 1;
pub const NODE_COMBSEP_COLOR_HSL: Nodecombsepcolormode = 2;

/// `GeometryNodeGizmoColor`.
pub type Geometrynodegizmocolor = i32;
pub const GEO_NODE_GIZMO_COLOR_PRIMARY: Geometrynodegizmocolor = 0;
pub const GEO_NODE_GIZMO_COLOR_SECONDARY: Geometrynodegizmocolor = 1;
pub const GEO_NODE_GIZMO_COLOR_X: Geometrynodegizmocolor = 2;
pub const GEO_NODE_GIZMO_COLOR_Y: Geometrynodegizmocolor = 3;
pub const GEO_NODE_GIZMO_COLOR_Z: Geometrynodegizmocolor = 4;

/// `GeometryNodeLinearGizmoDrawStyle`.
pub type Geometrynodelineargizmodrawstyle = i32;
pub const GEO_NODE_LINEAR_GIZMO_DRAW_STYLE_ARROW: Geometrynodelineargizmodrawstyle = 0;
pub const GEO_NODE_LINEAR_GIZMO_DRAW_STYLE_CROSS: Geometrynodelineargizmodrawstyle = 1;
pub const GEO_NODE_LINEAR_GIZMO_DRAW_STYLE_BOX: Geometrynodelineargizmodrawstyle = 2;

/// `NodeGeometryTransformMode`.
pub type Nodegeometrytransformmode = i32;
pub const GEO_NODE_TRANSFORM_MODE_COMPONENTS: Nodegeometrytransformmode = 0;
pub const GEO_NODE_TRANSFORM_MODE_MATRIX: Nodegeometrytransformmode = 1;

/// `NodeGeometryMergeLayersMode`.
pub type Nodegeometrymergelayersmode = i32;
pub const GEO_NODE_MERGE_LAYERS_BY_NAME: Nodegeometrymergelayersmode = 0;
pub const GEO_NODE_MERGE_LAYERS_BY_ID: Nodegeometrymergelayersmode = 1;

/// `NodeGeometryGreasePencilStrokeType`.
pub type Nodegeometrygreasepencilstroketype = i8;
pub const GEO_NODE_GREASE_PENCIL_STROKE: Nodegeometrygreasepencilstroketype = 0;
pub const GEO_NODE_GREASE_PENCIL_FILL: Nodegeometrygreasepencilstroketype = 1;
pub const GEO_NODE_GREASE_PENCIL_BOTH: Nodegeometrygreasepencilstroketype = 2;

/// `NodeGeometryRasterizePointsItemType`.
pub type Nodegeometryrasterizepointsitemtype = i32;
pub const GEO_NODE_RASTERIZE_POINTS_ITEM_TYPE_SCALAR: Nodegeometryrasterizepointsitemtype = 0;
pub const GEO_NODE_RASTERIZE_POINTS_ITEM_TYPE_VECTOR: Nodegeometryrasterizepointsitemtype = 1;

/// `bNodeStack`.
#[derive(Clone)]
pub struct bNodeStack {
    /// `float vec[4]`
    pub vec: [f32; 4],
    /// `float min`
    pub min: f32,
    /// `float max`
    pub max: f32,
    /// `void *data`
    pub data: usize,
    /// `short hasinput`
    pub hasinput: i16,
    /// `short hasoutput`
    pub hasoutput: i16,
    /// `eNodeStack_Datatype datatype`
    pub datatype: NodestackDatatype,
    /// `short sockettype`
    pub sockettype: i16,
    /// `short is_copy`
    pub is_copy: i16,
    /// `short external`
    pub external: i16,
}
impl Default for bNodeStack {
    fn default() -> Self {
        Self {
            vec: [0.0; 4],
            min: 0.0,
            max: 0.0,
            data: NIL,
            hasinput: 0,
            hasoutput: 0,
            datatype: Default::default(),
            sockettype: 0,
            is_copy: 0,
            external: 0,
        }
    }
}

/// `bNodeSocket`.
#[derive(Clone)]
pub struct bNodeSocket {
    /// `bNodeSocket *next`
    pub next: usize,
    /// `bNodeSocket *prev`
    pub prev: usize,
    /// `IDProperty *prop`
    pub prop: usize,
    /// `char identifier[64]`
    pub identifier: [u8; 64],
    /// `char name[64]`
    pub name: [u8; 64],
    /// `void *storage`
    pub storage: usize,
    /// `eNodeSocketDatatype type`
    pub type_: Nodesocketdatatype,
    /// `eNodeSocketFlag flag`
    pub flag: Nodesocketflag,
    /// `short limit`
    pub limit: i16,
    /// `eNodeSocketInOut in_out`
    pub in_out: Nodesocketinout,
    /// `bke::bNodeSocketType *typeinfo`
    pub typeinfo: usize,
    /// `char idname[64]`
    pub idname: [u8; 64],
    /// `void *default_value`
    pub default_value: usize,
    /// `int stack_index`
    pub stack_index: i32,
    /// `eNodeSocketDisplayShape display_shape`
    pub display_shape: Nodesocketdisplayshape,
    /// `char attribute_domain`
    pub attribute_domain: i8,
    /// `char label[64]`
    pub label: [u8; 64],
    /// `char description[64]`
    pub description: [u8; 64],
    /// `char *default_attribute_name`
    pub default_attribute_name: usize,
    /// `int own_index`
    pub own_index: i32,
    /// `int to_index`
    pub to_index: i32,
    /// `bNodeLink *link`
    pub link: usize,
    pub ns: bNodeStack,
    /// `bke::bNodeSocketRuntime *runtime`
    pub runtime: usize,
}
impl Default for bNodeSocket {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            prop: NIL,
            identifier: [0; 64],
            name: [0; 64],
            storage: NIL,
            type_: Default::default(),
            flag: Default::default(),
            limit: 0,
            in_out: Default::default(),
            typeinfo: NIL,
            idname: [0; 64],
            default_value: NIL,
            stack_index: 0,
            display_shape: SOCK_DISPLAY_SHAPE_CIRCLE,
            attribute_domain: 0,
            label: [0; 64],
            description: [0; 64],
            default_attribute_name: NIL,
            own_index: 0,
            to_index: 0,
            link: NIL,
            ns: Default::default(),
            runtime: NIL,
        }
    }
}

/// `bNodePanelState`.
#[derive(Clone)]
pub struct bNodePanelState {
    /// `int identifier`
    pub identifier: i32,
    /// `eNodePanelFlag flag`
    pub flag: Nodepanelflag,
}
impl Default for bNodePanelState {
    fn default() -> Self {
        Self {
            identifier: 0,
            flag: Default::default(),
        }
    }
}

/// `bNode`.
#[derive(Clone)]
pub struct bNode {
    /// `bNode *next`
    pub next: usize,
    /// `bNode *prev`
    pub prev: usize,
    /// `ListBaseT<bNodeSocket>`
    pub inputs: ListBase,
    /// `char name[64]`
    pub name: [u8; 64],
    /// `int32_t identifier`
    pub identifier: i32,
    /// `eNode_Flag flag`
    pub flag: NodeFlag,
    /// `char idname[64]`
    pub idname: [u8; 64],
    /// `bke::bNodeType *typeinfo`
    pub typeinfo: usize,
    /// `int16_t type_legacy`
    pub type_legacy: i16,
    /// `int16_t ui_order`
    pub ui_order: i16,
    /// `int16_t custom1`
    pub custom1: i16,
    /// `int16_t custom2`
    pub custom2: i16,
    /// `float custom3`
    pub custom3: f32,
    /// `float custom4`
    pub custom4: f32,
    /// `NodeWarningPropagation warning_propagation`
    pub warning_propagation: Nodewarningpropagation,
    /// `ID *id`
    pub id: usize,
    /// `void *storage`
    pub storage: usize,
    /// `IDProperty *prop`
    pub prop: usize,
    /// `IDProperty *system_properties`
    pub system_properties: usize,
    /// `bNode *parent`
    pub parent: usize,
    /// `float location[2]`
    pub location: [f32; 2],
    /// `float width`
    pub width: f32,
    /// `float height`
    pub height: f32,
    /// `float locx_legacy`
    pub locx_legacy: f32,
    /// `float locy_legacy`
    pub locy_legacy: f32,
    /// `float offsetx_legacy`
    pub offsetx_legacy: f32,
    /// `float offsety_legacy`
    pub offsety_legacy: f32,
    /// `char label[64]`
    pub label: [u8; 64],
    /// `float color[3]`
    pub color: [f32; 3],
    /// `int num_panel_states`
    pub num_panel_states: i32,
    /// `bNodePanelState *panel_states_array`
    pub panel_states_array: usize,
    /// `bke::bNodeRuntime *runtime`
    pub runtime: usize,
}
impl Default for bNode {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            inputs: ListBase::default(),
            name: [0; 64],
            identifier: 0,
            flag: Default::default(),
            idname: [0; 64],
            typeinfo: NIL,
            type_legacy: 0,
            ui_order: 0,
            custom1: 0,
            custom2: 0,
            custom3: 0.0,
            custom4: 0.0,
            warning_propagation: NODE_WARNING_PROPAGATION_ALL,
            id: NIL,
            storage: NIL,
            prop: NIL,
            system_properties: NIL,
            parent: NIL,
            location: [0.0; 2],
            width: 0.0,
            height: 0.0,
            locx_legacy: 0.0,
            locy_legacy: 0.0,
            offsetx_legacy: 0.0,
            offsety_legacy: 0.0,
            label: [0; 64],
            color: [0.0; 3],
            num_panel_states: 0,
            panel_states_array: NIL,
            runtime: NIL,
        }
    }
}

/// `bNodeInstanceKey`.
#[derive(Clone)]
pub struct bNodeInstanceKey {
    /// `unsigned_int value`
    pub value: u32,
}
impl Default for bNodeInstanceKey {
    fn default() -> Self {
        Self {
            value: 0,
        }
    }
}

/// `bNodeLink`.
#[derive(Clone)]
pub struct bNodeLink {
    /// `bNodeLink *next`
    pub next: usize,
    /// `bNodeLink *prev`
    pub prev: usize,
    /// `bNode *fromnode`
    pub fromnode: usize,
    /// `bNode *tonode`
    pub tonode: usize,
    /// `bNodeSocket *fromsock`
    pub fromsock: usize,
    /// `bNodeSocket *tosock`
    pub tosock: usize,
    /// `eNodeLink_Flag flag`
    pub flag: NodelinkFlag,
    /// `int multi_input_sort_id`
    pub multi_input_sort_id: i32,
}
impl Default for bNodeLink {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            fromnode: NIL,
            tonode: NIL,
            fromsock: NIL,
            tosock: NIL,
            flag: Default::default(),
            multi_input_sort_id: 0,
        }
    }
}

/// `bNestedNodePath`.
#[derive(Clone)]
pub struct bNestedNodePath {
    /// `int32_t node_id`
    pub node_id: i32,
    /// `int32_t id_in_node`
    pub id_in_node: i32,
}
impl Default for bNestedNodePath {
    fn default() -> Self {
        Self {
            node_id: 0,
            id_in_node: 0,
        }
    }
}

/// `bNestedNodeRef`.
#[derive(Clone)]
pub struct bNestedNodeRef {
    /// `int32_t id`
    pub id: i32,
    pub path: bNestedNodePath,
}
impl Default for bNestedNodeRef {
    fn default() -> Self {
        Self {
            id: 0,
            path: Default::default(),
        }
    }
}

/// `bNodeTree`.
#[derive(Clone)]
pub struct bNodeTree {
    pub id: ID,
    /// `AnimData *adt`
    pub adt: usize,
    /// `ID *owner_id`
    pub owner_id: usize,
    /// `bke::bNodeTreeType *typeinfo`
    pub typeinfo: usize,
    /// `char idname[64]`
    pub idname: [u8; 64],
    /// `char *description`
    pub description: usize,
    /// `bGPdata *gpd`
    pub gpd: usize,
    /// `float view_center[2]`
    pub view_center: [f32; 2],
    /// `float view_width`
    pub view_width: f32,
    /// `uint32_t next_node_identifier_seed`
    pub next_node_identifier_seed: u32,
    /// `ListBaseT<bNode>`
    pub nodes: ListBase,
    /// `ListBaseT<bNodeLink>`
    pub links: ListBase,
    /// `eNodeTree_Type type`
    pub type_: NodetreeType,
    /// `int cur_index`
    pub cur_index: i32,
    /// `eNodeTree_Flag flag`
    pub flag: NodetreeFlag,
    /// `int chunksize`
    pub chunksize: i32,
    /// `int execution_mode`
    pub execution_mode: i32,
    /// `int precision`
    pub precision: i32,
    /// `int color_tag`
    pub color_tag: i32,
    /// `int default_group_node_width`
    pub default_group_node_width: i32,
    /// `rctf viewer_border`
    pub viewer_border: Rctf,
    /// `ListBaseT<bNodeSocket>`
    pub inputs_legacy: ListBase,
    /// `ListBaseT<bNodeSocket>`
    pub outputs_legacy: ListBase,
    pub tree_interface: BNodeTreeInterface,
    pub active_viewer_key: bNodeInstanceKey,
    /// `int nested_node_refs_num`
    pub nested_node_refs_num: i32,
    /// `bNestedNodeRef *nested_node_refs`
    pub nested_node_refs: usize,
    /// `GeometryNodeAssetTraits *geometry_node_asset_traits`
    pub geometry_node_asset_traits: usize,
    /// `CompositorNodeAssetTraits *compositor_node_asset_traits`
    pub compositor_node_asset_traits: usize,
    /// `PreviewImage *preview`
    pub preview: usize,
    /// `bke::bNodeTreeRuntime *runtime`
    pub runtime: usize,
}
impl Default for bNodeTree {
    fn default() -> Self {
        Self {
            id: Default::default(),
            adt: NIL,
            owner_id: NIL,
            typeinfo: NIL,
            idname: [0; 64],
            description: NIL,
            gpd: NIL,
            view_center: [0.0; 2],
            view_width: 0.0,
            next_node_identifier_seed: 0,
            nodes: ListBase::default(),
            links: ListBase::default(),
            type_: Default::default(),
            cur_index: 0,
            flag: Default::default(),
            chunksize: 0,
            execution_mode: 0,
            precision: 0,
            color_tag: 0,
            default_group_node_width: 0,
            viewer_border: Default::default(),
            inputs_legacy: ListBase::default(),
            outputs_legacy: ListBase::default(),
            tree_interface: Default::default(),
            active_viewer_key: Default::default(),
            nested_node_refs_num: 0,
            nested_node_refs: NIL,
            geometry_node_asset_traits: NIL,
            compositor_node_asset_traits: NIL,
            preview: NIL,
            runtime: NIL,
        }
    }
}

/// `bNodeSocketValueInt`.
#[derive(Clone)]
pub struct bNodeSocketValueInt {
    /// `int subtype`
    pub subtype: i32,
    /// `int value`
    pub value: i32,
    /// `int min`
    pub min: i32,
    /// `int max`
    pub max: i32,
}
impl Default for bNodeSocketValueInt {
    fn default() -> Self {
        Self {
            subtype: 0,
            value: 0,
            min: 0,
            max: 0,
        }
    }
}

/// `bNodeSocketValueFloat`.
#[derive(Clone)]
pub struct bNodeSocketValueFloat {
    /// `int subtype`
    pub subtype: i32,
    /// `float value`
    pub value: f32,
    /// `float min`
    pub min: f32,
    /// `float max`
    pub max: f32,
    /// `float step`
    pub step: f32,
    /// `int precision`
    pub precision: i32,
}
impl Default for bNodeSocketValueFloat {
    fn default() -> Self {
        Self {
            subtype: 0,
            value: 0.0,
            min: 0.0,
            max: 0.0,
            step: 0.1,
            precision: 3,
        }
    }
}

/// `bNodeSocketValueBoolean`.
#[derive(Clone)]
pub struct bNodeSocketValueBoolean {
    /// `char value`
    pub value: i8,
}
impl Default for bNodeSocketValueBoolean {
    fn default() -> Self {
        Self {
            value: 0,
        }
    }
}

/// `bNodeSocketValueVector`.
#[derive(Clone)]
pub struct bNodeSocketValueVector {
    /// `int subtype`
    pub subtype: i32,
    /// `float value[4]`
    pub value: [f32; 4],
    /// `float min`
    pub min: f32,
    /// `float max`
    pub max: f32,
    /// `int dimensions`
    pub dimensions: i32,
}
impl Default for bNodeSocketValueVector {
    fn default() -> Self {
        Self {
            subtype: 0,
            value: [0.0; 4],
            min: 0.0,
            max: 0.0,
            dimensions: 0,
        }
    }
}

/// `bNodeSocketValueIntVector`.
#[derive(Clone)]
pub struct bNodeSocketValueIntVector {
    /// `int subtype`
    pub subtype: i32,
    /// `int value[3]`
    pub value: [i32; 3],
    /// `int min`
    pub min: i32,
    /// `int max`
    pub max: i32,
    /// `int dimensions`
    pub dimensions: i32,
}
impl Default for bNodeSocketValueIntVector {
    fn default() -> Self {
        Self {
            subtype: 0,
            value: [0; 3],
            min: 0,
            max: 0,
            dimensions: 0,
        }
    }
}

/// `bNodeSocketValueRotation`.
#[derive(Clone)]
pub struct bNodeSocketValueRotation {
    /// `float value_euler[3]`
    pub value_euler: [f32; 3],
}
impl Default for bNodeSocketValueRotation {
    fn default() -> Self {
        Self {
            value_euler: [0.0; 3],
        }
    }
}

/// `bNodeSocketValueRGBA`.
#[derive(Clone)]
pub struct bNodeSocketValueRGBA {
    /// `float value[4]`
    pub value: [f32; 4],
}
impl Default for bNodeSocketValueRGBA {
    fn default() -> Self {
        Self {
            value: [0.0; 4],
        }
    }
}

/// `bNodeSocketValueString`.
#[derive(Clone)]
pub struct bNodeSocketValueString {
    /// `int subtype`
    pub subtype: i32,
    /// `char value[1024]`
    pub value: [u8; 1024],
}
impl Default for bNodeSocketValueString {
    fn default() -> Self {
        Self {
            subtype: 0,
            value: [0; 1024],
        }
    }
}

/// `bNodeSocketValueObject`.
#[derive(Clone)]
pub struct bNodeSocketValueObject {
    /// `Object *value`
    pub value: usize,
}
impl Default for bNodeSocketValueObject {
    fn default() -> Self {
        Self {
            value: NIL,
        }
    }
}

/// `bNodeSocketValueImage`.
#[derive(Clone)]
pub struct bNodeSocketValueImage {
    /// `Image *value`
    pub value: usize,
}
impl Default for bNodeSocketValueImage {
    fn default() -> Self {
        Self {
            value: NIL,
        }
    }
}

/// `bNodeSocketValueCollection`.
#[derive(Clone)]
pub struct bNodeSocketValueCollection {
    /// `Collection *value`
    pub value: usize,
}
impl Default for bNodeSocketValueCollection {
    fn default() -> Self {
        Self {
            value: NIL,
        }
    }
}

/// `bNodeSocketValueTexture`.
#[derive(Clone)]
pub struct bNodeSocketValueTexture {
    /// `Tex *value`
    pub value: usize,
}
impl Default for bNodeSocketValueTexture {
    fn default() -> Self {
        Self {
            value: NIL,
        }
    }
}

/// `bNodeSocketValueMaterial`.
#[derive(Clone)]
pub struct bNodeSocketValueMaterial {
    /// `Material *value`
    pub value: usize,
}
impl Default for bNodeSocketValueMaterial {
    fn default() -> Self {
        Self {
            value: NIL,
        }
    }
}

/// `bNodeSocketValueFont`.
#[derive(Clone)]
pub struct bNodeSocketValueFont {
    /// `VFont *value`
    pub value: usize,
}
impl Default for bNodeSocketValueFont {
    fn default() -> Self {
        Self {
            value: NIL,
        }
    }
}

/// `bNodeSocketValueScene`.
#[derive(Clone)]
pub struct bNodeSocketValueScene {
    /// `Scene *value`
    pub value: usize,
}
impl Default for bNodeSocketValueScene {
    fn default() -> Self {
        Self {
            value: NIL,
        }
    }
}

/// `bNodeSocketValueText`.
#[derive(Clone)]
pub struct bNodeSocketValueText {
    /// `Text *value`
    pub value: usize,
}
impl Default for bNodeSocketValueText {
    fn default() -> Self {
        Self {
            value: NIL,
        }
    }
}

/// `bNodeSocketValueMask`.
#[derive(Clone)]
pub struct bNodeSocketValueMask {
    /// `Mask *value`
    pub value: usize,
}
impl Default for bNodeSocketValueMask {
    fn default() -> Self {
        Self {
            value: NIL,
        }
    }
}

/// `bNodeSocketValueSound`.
#[derive(Clone)]
pub struct bNodeSocketValueSound {
    /// `bSound *value`
    pub value: usize,
}
impl Default for bNodeSocketValueSound {
    fn default() -> Self {
        Self {
            value: NIL,
        }
    }
}

/// `bNodeSocketValueMenu`.
#[derive(Clone)]
pub struct bNodeSocketValueMenu {
    /// `int value`
    pub value: i32,
    /// `int runtime_flag`
    pub runtime_flag: i32,
}
impl Default for bNodeSocketValueMenu {
    fn default() -> Self {
        Self {
            value: 0,
            runtime_flag: 0,
        }
    }
}

/// `GeometryNodeAssetTraits`.
#[derive(Clone)]
pub struct GeometryNodeAssetTraits {
    /// `int flag`
    pub flag: i32,
    /// `char *node_tool_idname`
    pub node_tool_idname: usize,
}
impl Default for GeometryNodeAssetTraits {
    fn default() -> Self {
        Self {
            flag: 0,
            node_tool_idname: NIL,
        }
    }
}

/// `CompositorNodeAssetTraits`.
#[derive(Clone)]
pub struct CompositorNodeAssetTraits {
    /// `int flag`
    pub flag: i32,
}
impl Default for CompositorNodeAssetTraits {
    fn default() -> Self {
        Self {
            flag: 0,
        }
    }
}

/// `NodeFrame`.
#[derive(Clone)]
pub struct NodeFrame {
    /// `short flag`
    pub flag: i16,
    /// `short label_size`
    pub label_size: i16,
}
impl Default for NodeFrame {
    fn default() -> Self {
        Self {
            flag: 0,
            label_size: 0,
        }
    }
}

/// `NodeReroute`.
#[derive(Clone)]
pub struct NodeReroute {
    /// `char type_idname[64]`
    pub type_idname: [u8; 64],
}
impl Default for NodeReroute {
    fn default() -> Self {
        Self {
            type_idname: [0; 64],
        }
    }
}

/// `NodeImplicitConversion`.
#[derive(Clone)]
pub struct NodeImplicitConversion {
    /// `char type_idname[64]`
    pub type_idname: [u8; 64],
}
impl Default for NodeImplicitConversion {
    fn default() -> Self {
        Self {
            type_idname: [0; 64],
        }
    }
}

/// `NodeImageAnim`.
#[derive(Clone)]
pub struct NodeImageAnim {
    /// `int frames`
    pub frames: i32,
    /// `int sfra`
    pub sfra: i32,
    /// `int nr`
    pub nr: i32,
    /// `char cyclic`
    pub cyclic: i8,
    /// `char movie`
    pub movie: i8,
}
impl Default for NodeImageAnim {
    fn default() -> Self {
        Self {
            frames: 0,
            sfra: 0,
            nr: 0,
            cyclic: 0,
            movie: 0,
        }
    }
}

/// `ColorCorrectionData`.
#[derive(Clone)]
pub struct ColorCorrectionData {
    /// `float saturation`
    pub saturation: f32,
    /// `float contrast`
    pub contrast: f32,
    /// `float gamma`
    pub gamma: f32,
    /// `float gain`
    pub gain: f32,
    /// `float lift`
    pub lift: f32,
}
impl Default for ColorCorrectionData {
    fn default() -> Self {
        Self {
            saturation: 0.0,
            contrast: 0.0,
            gamma: 0.0,
            gain: 0.0,
            lift: 0.0,
        }
    }
}

/// `NodeColorCorrection`.
#[derive(Clone)]
pub struct NodeColorCorrection {
    pub master: ColorCorrectionData,
    pub shadows: ColorCorrectionData,
    pub midtones: ColorCorrectionData,
    pub highlights: ColorCorrectionData,
    /// `float startmidtones`
    pub startmidtones: f32,
    /// `float endmidtones`
    pub endmidtones: f32,
}
impl Default for NodeColorCorrection {
    fn default() -> Self {
        Self {
            master: Default::default(),
            shadows: Default::default(),
            midtones: Default::default(),
            highlights: Default::default(),
            startmidtones: 0.0,
            endmidtones: 0.0,
        }
    }
}

/// `NodeBokehImage`.
#[derive(Clone)]
pub struct NodeBokehImage {
    /// `float angle`
    pub angle: f32,
    /// `int flaps`
    pub flaps: i32,
    /// `float rounding`
    pub rounding: f32,
    /// `float catadioptric`
    pub catadioptric: f32,
    /// `float lensshift`
    pub lensshift: f32,
}
impl Default for NodeBokehImage {
    fn default() -> Self {
        Self {
            angle: 0.0,
            flaps: 0,
            rounding: 0.0,
            catadioptric: 0.0,
            lensshift: 0.0,
        }
    }
}

/// `NodeBoxMask`.
#[derive(Clone)]
pub struct NodeBoxMask {
    /// `float x`
    pub x: f32,
    /// `float y`
    pub y: f32,
    /// `float rotation`
    pub rotation: f32,
    /// `float height`
    pub height: f32,
    /// `float width`
    pub width: f32,
}
impl Default for NodeBoxMask {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            height: 0.0,
            width: 0.0,
        }
    }
}

/// `NodeEllipseMask`.
#[derive(Clone)]
pub struct NodeEllipseMask {
    /// `float x`
    pub x: f32,
    /// `float y`
    pub y: f32,
    /// `float rotation`
    pub rotation: f32,
    /// `float height`
    pub height: f32,
    /// `float width`
    pub width: f32,
}
impl Default for NodeEllipseMask {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            height: 0.0,
            width: 0.0,
        }
    }
}

/// `NodeImageLayer`.
#[derive(Clone)]
pub struct NodeImageLayer {
    /// `int pass_index`
    pub pass_index: i32,
    /// `char pass_name[64]`
    pub pass_name: [u8; 64],
}
impl Default for NodeImageLayer {
    fn default() -> Self {
        Self {
            pass_index: 0,
            pass_name: [0; 64],
        }
    }
}

/// `NodeBlurData`.
#[derive(Clone)]
pub struct NodeBlurData {
    /// `short sizex`
    pub sizex: i16,
    /// `short sizey`
    pub sizey: i16,
    /// `short samples`
    pub samples: i16,
    /// `short maxspeed`
    pub maxspeed: i16,
    /// `short minspeed`
    pub minspeed: i16,
    /// `short relative`
    pub relative: i16,
    /// `short aspect`
    pub aspect: i16,
    /// `short curved`
    pub curved: i16,
    /// `float fac`
    pub fac: f32,
    /// `float percentx`
    pub percentx: f32,
    /// `float percenty`
    pub percenty: f32,
    /// `short filtertype`
    pub filtertype: i16,
    /// `char bokeh`
    pub bokeh: i8,
    /// `char gamma`
    pub gamma: i8,
}
impl Default for NodeBlurData {
    fn default() -> Self {
        Self {
            sizex: 0,
            sizey: 0,
            samples: 0,
            maxspeed: 0,
            minspeed: 0,
            relative: 0,
            aspect: 0,
            curved: 0,
            fac: 0.0,
            percentx: 0.0,
            percenty: 0.0,
            filtertype: 0,
            bokeh: 0,
            gamma: 0,
        }
    }
}

/// `NodeDBlurData`.
#[derive(Clone)]
pub struct NodeDBlurData {
    /// `float center_x`
    pub center_x: f32,
    /// `float center_y`
    pub center_y: f32,
    /// `float distance`
    pub distance: f32,
    /// `float angle`
    pub angle: f32,
    /// `float spin`
    pub spin: f32,
    /// `float zoom`
    pub zoom: f32,
    /// `short iter`
    pub iter: i16,
}
impl Default for NodeDBlurData {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_y: 0.0,
            distance: 0.0,
            angle: 0.0,
            spin: 0.0,
            zoom: 0.0,
            iter: 0,
        }
    }
}

/// `NodeBilateralBlurData`.
#[derive(Clone)]
pub struct NodeBilateralBlurData {
    /// `float sigma_color`
    pub sigma_color: f32,
    /// `float sigma_space`
    pub sigma_space: f32,
    /// `short iter`
    pub iter: i16,
}
impl Default for NodeBilateralBlurData {
    fn default() -> Self {
        Self {
            sigma_color: 0.0,
            sigma_space: 0.0,
            iter: 0,
        }
    }
}

/// `NodeKuwaharaData`.
#[derive(Clone)]
pub struct NodeKuwaharaData {
    /// `short size`
    pub size: i16,
    /// `short variation`
    pub variation: i16,
    /// `int uniformity`
    pub uniformity: i32,
    /// `float sharpness`
    pub sharpness: f32,
    /// `float eccentricity`
    pub eccentricity: f32,
    /// `char high_precision`
    pub high_precision: i8,
}
impl Default for NodeKuwaharaData {
    fn default() -> Self {
        Self {
            size: 0,
            variation: 0,
            uniformity: 0,
            sharpness: 0.0,
            eccentricity: 0.0,
            high_precision: 0,
        }
    }
}

/// `NodeAntiAliasingData`.
#[derive(Clone)]
pub struct NodeAntiAliasingData {
    /// `float threshold`
    pub threshold: f32,
    /// `float contrast_limit`
    pub contrast_limit: f32,
    /// `float corner_rounding`
    pub corner_rounding: f32,
}
impl Default for NodeAntiAliasingData {
    fn default() -> Self {
        Self {
            threshold: 0.0,
            contrast_limit: 0.0,
            corner_rounding: 0.0,
        }
    }
}

/// `NodeHueSat`.
#[derive(Clone)]
pub struct NodeHueSat {
    /// `float hue`
    pub hue: f32,
    /// `float sat`
    pub sat: f32,
    /// `float val`
    pub val: f32,
}
impl Default for NodeHueSat {
    fn default() -> Self {
        Self {
            hue: 0.0,
            sat: 0.0,
            val: 0.0,
        }
    }
}

/// `NodeImageFile`.
#[derive(Clone)]
pub struct NodeImageFile {
    /// `char name[1024]`
    pub name: [u8; 1024],
    pub im_format: ImageFormatData,
    /// `int sfra`
    pub sfra: i32,
    /// `int efra`
    pub efra: i32,
}
impl Default for NodeImageFile {
    fn default() -> Self {
        Self {
            name: [0; 1024],
            im_format: Default::default(),
            sfra: 0,
            efra: 0,
        }
    }
}

/// `NodeCompositorFileOutputItem`.
#[derive(Clone)]
pub struct NodeCompositorFileOutputItem {
    /// `int identifier`
    pub identifier: i32,
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    /// `char vector_socket_dimensions`
    pub vector_socket_dimensions: i8,
    /// `char override_node_format`
    pub override_node_format: i8,
    /// `char save_as_render`
    pub save_as_render: i8,
    /// `char *name`
    pub name: usize,
    pub format: ImageFormatData,
}
impl Default for NodeCompositorFileOutputItem {
    fn default() -> Self {
        Self {
            identifier: 0,
            socket_type: Default::default(),
            vector_socket_dimensions: 0,
            override_node_format: 0,
            save_as_render: 0,
            name: NIL,
            format: Default::default(),
        }
    }
}

/// `NodeCompositorFileOutput`.
#[derive(Clone)]
pub struct NodeCompositorFileOutput {
    /// `char directory[1024]`
    pub directory: [u8; 1024],
    /// `char *file_name`
    pub file_name: usize,
    pub format: ImageFormatData,
    /// `NodeCompositorFileOutputItem *items`
    pub items: usize,
    /// `int items_count`
    pub items_count: i32,
    /// `int active_item_index`
    pub active_item_index: i32,
    /// `char save_as_render`
    pub save_as_render: i8,
    /// `char use_file_extension`
    pub use_file_extension: i8,
}
impl Default for NodeCompositorFileOutput {
    fn default() -> Self {
        Self {
            directory: [0; 1024],
            file_name: NIL,
            format: Default::default(),
            items: NIL,
            items_count: 0,
            active_item_index: 0,
            save_as_render: 0,
            use_file_extension: 0,
        }
    }
}

/// `NodeImageMultiFileSocket`.
#[derive(Clone)]
pub struct NodeImageMultiFileSocket {
    /// `short use_render_format`
    pub use_render_format: i16,
    /// `short use_node_format`
    pub use_node_format: i16,
    /// `char save_as_render`
    pub save_as_render: i8,
    /// `char path[1024]`
    pub path: [u8; 1024],
    pub format: ImageFormatData,
    /// `char layer[62]`
    pub layer: [u8; 62],
}
impl Default for NodeImageMultiFileSocket {
    fn default() -> Self {
        Self {
            use_render_format: 0,
            use_node_format: 0,
            save_as_render: 0,
            path: [0; 1024],
            format: Default::default(),
            layer: [0; 62],
        }
    }
}

/// `NodeChroma`.
#[derive(Clone)]
pub struct NodeChroma {
    /// `float t1`
    pub t1: f32,
    /// `float t2`
    pub t2: f32,
    /// `float t3`
    pub t3: f32,
    /// `float fsize`
    pub fsize: f32,
    /// `float fstrength`
    pub fstrength: f32,
    /// `float falpha`
    pub falpha: f32,
    /// `float key[4]`
    pub key: [f32; 4],
    /// `short algorithm`
    pub algorithm: i16,
    /// `short channel`
    pub channel: i16,
}
impl Default for NodeChroma {
    fn default() -> Self {
        Self {
            t1: 0.0,
            t2: 0.0,
            t3: 0.0,
            fsize: 0.0,
            fstrength: 0.0,
            falpha: 0.0,
            key: [0.0; 4],
            algorithm: 0,
            channel: 0,
        }
    }
}

/// `NodeTwoXYs`.
#[derive(Clone)]
pub struct NodeTwoXYs {
    /// `short x1`
    pub x1: i16,
    /// `short x2`
    pub x2: i16,
    /// `short y1`
    pub y1: i16,
    /// `short y2`
    pub y2: i16,
    /// `float fac_x1`
    pub fac_x1: f32,
    /// `float fac_x2`
    pub fac_x2: f32,
    /// `float fac_y1`
    pub fac_y1: f32,
    /// `float fac_y2`
    pub fac_y2: f32,
}
impl Default for NodeTwoXYs {
    fn default() -> Self {
        Self {
            x1: 0,
            x2: 0,
            y1: 0,
            y2: 0,
            fac_x1: 0.0,
            fac_x2: 0.0,
            fac_y1: 0.0,
            fac_y2: 0.0,
        }
    }
}

/// `NodeTwoFloats`.
#[derive(Clone)]
pub struct NodeTwoFloats {
    /// `float x`
    pub x: f32,
    /// `float y`
    pub y: f32,
}
impl Default for NodeTwoFloats {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
        }
    }
}

/// `NodeVertexCol`.
#[derive(Clone)]
pub struct NodeVertexCol {
    /// `char name[64]`
    pub name: [u8; 64],
}
impl Default for NodeVertexCol {
    fn default() -> Self {
        Self {
            name: [0; 64],
        }
    }
}

/// `NodeCMPCombSepColor`.
#[derive(Clone)]
pub struct NodeCMPCombSepColor {
    /// `CMPNodeCombSepColorMode mode`
    pub mode: Cmpnodecombsepcolormode,
    /// `uint8_t ycc_mode`
    pub ycc_mode: u8,
}
impl Default for NodeCMPCombSepColor {
    fn default() -> Self {
        Self {
            mode: CMP_NODE_COMBSEP_COLOR_RGB,
            ycc_mode: 0,
        }
    }
}

/// `NodeDefocus`.
#[derive(Clone)]
pub struct NodeDefocus {
    /// `char bktype`
    pub bktype: i8,
    /// `char gamco`
    pub gamco: i8,
    /// `char no_zbuf`
    pub no_zbuf: i8,
    /// `float fstop`
    pub fstop: f32,
    /// `float maxblur`
    pub maxblur: f32,
    /// `float scale`
    pub scale: f32,
    /// `float rotation`
    pub rotation: f32,
}
impl Default for NodeDefocus {
    fn default() -> Self {
        Self {
            bktype: 0,
            gamco: 0,
            no_zbuf: 1,
            fstop: 0.0,
            maxblur: 0.0,
            scale: 0.0,
            rotation: 0.0,
        }
    }
}

/// `NodeScriptDict`.
#[derive(Clone)]
pub struct NodeScriptDict {
    /// `void *dict`
    pub dict: usize,
    /// `void *node`
    pub node: usize,
}
impl Default for NodeScriptDict {
    fn default() -> Self {
        Self {
            dict: NIL,
            node: NIL,
        }
    }
}

/// `NodeGlare`.
#[derive(Clone)]
pub struct NodeGlare {
    /// `char type`
    pub type_: i8,
    /// `char quality`
    pub quality: i8,
    /// `char iter`
    pub iter: i8,
    /// `char angle`
    pub angle: i8,
    /// `char size`
    pub size: i8,
    /// `char star_45`
    pub star_45: i8,
    /// `char streaks`
    pub streaks: i8,
    /// `float colmod`
    pub colmod: f32,
    /// `float mix`
    pub mix: f32,
    /// `float threshold`
    pub threshold: f32,
    /// `float fade`
    pub fade: f32,
    /// `float angle_ofs`
    pub angle_ofs: f32,
}
impl Default for NodeGlare {
    fn default() -> Self {
        Self {
            type_: 0,
            quality: 0,
            iter: 0,
            angle: 0,
            size: 0,
            star_45: 0,
            streaks: 0,
            colmod: 0.0,
            mix: 0.0,
            threshold: 0.0,
            fade: 0.0,
            angle_ofs: 0.0,
        }
    }
}

/// `NodeTonemap`.
#[derive(Clone)]
pub struct NodeTonemap {
    /// `float key`
    pub key: f32,
    /// `float offset`
    pub offset: f32,
    /// `float gamma`
    pub gamma: f32,
    /// `float f`
    pub f: f32,
    /// `float m`
    pub m: f32,
    /// `float a`
    pub a: f32,
    /// `float c`
    pub c: f32,
    /// `int type`
    pub type_: i32,
}
impl Default for NodeTonemap {
    fn default() -> Self {
        Self {
            key: 0.0,
            offset: 0.0,
            gamma: 0.0,
            f: 0.0,
            m: 0.0,
            a: 0.0,
            c: 0.0,
            type_: 0,
        }
    }
}

/// `NodeLensDist`.
#[derive(Clone)]
pub struct NodeLensDist {
    /// `short jit`
    pub jit: i16,
    /// `short proj`
    pub proj: i16,
    /// `short fit`
    pub fit: i16,
    /// `int distortion_type`
    pub distortion_type: i32,
}
impl Default for NodeLensDist {
    fn default() -> Self {
        Self {
            jit: 0,
            proj: 0,
            fit: 0,
            distortion_type: 0,
        }
    }
}

/// `NodeColorBalance`.
#[derive(Clone)]
pub struct NodeColorBalance {
    /// `float slope[3]`
    pub slope: [f32; 3],
    /// `float offset[3]`
    pub offset: [f32; 3],
    /// `float power[3]`
    pub power: [f32; 3],
    /// `float offset_basis`
    pub offset_basis: f32,
    /// `float lift[3]`
    pub lift: [f32; 3],
    /// `float gamma[3]`
    pub gamma: [f32; 3],
    /// `float gain[3]`
    pub gain: [f32; 3],
    /// `float input_temperature`
    pub input_temperature: f32,
    /// `float input_tint`
    pub input_tint: f32,
    /// `float output_temperature`
    pub output_temperature: f32,
    /// `float output_tint`
    pub output_tint: f32,
}
impl Default for NodeColorBalance {
    fn default() -> Self {
        Self {
            slope: [0.0; 3],
            offset: [0.0; 3],
            power: [0.0; 3],
            offset_basis: 0.0,
            lift: [0.0; 3],
            gamma: [0.0; 3],
            gain: [0.0; 3],
            input_temperature: 0.0,
            input_tint: 0.0,
            output_temperature: 0.0,
            output_tint: 0.0,
        }
    }
}

/// `NodeColorspill`.
#[derive(Clone)]
pub struct NodeColorspill {
    /// `short limchan`
    pub limchan: i16,
    /// `short unspill`
    pub unspill: i16,
    /// `float limscale`
    pub limscale: f32,
    /// `float uspillr`
    pub uspillr: f32,
    /// `float uspillg`
    pub uspillg: f32,
    /// `float uspillb`
    pub uspillb: f32,
}
impl Default for NodeColorspill {
    fn default() -> Self {
        Self {
            limchan: 0,
            unspill: 0,
            limscale: 0.0,
            uspillr: 0.0,
            uspillg: 0.0,
            uspillb: 0.0,
        }
    }
}

/// `NodeConvertColorSpace`.
#[derive(Clone)]
pub struct NodeConvertColorSpace {
    /// `char from_color_space[64]`
    pub from_color_space: [u8; 64],
    /// `char from_interop_id[64]`
    pub from_interop_id: [u8; 64],
    /// `char to_color_space[64]`
    pub to_color_space: [u8; 64],
    /// `char to_interop_id[64]`
    pub to_interop_id: [u8; 64],
}
impl Default for NodeConvertColorSpace {
    fn default() -> Self {
        Self {
            from_color_space: [0; 64],
            from_interop_id: [0; 64],
            to_color_space: [0; 64],
            to_interop_id: [0; 64],
        }
    }
}

/// `NodeConvertToDisplay`.
#[derive(Clone)]
pub struct NodeConvertToDisplay {
    pub display_settings: ColorManagedDisplaySettings,
    pub view_settings: ColorManagedViewSettings,
}
impl Default for NodeConvertToDisplay {
    fn default() -> Self {
        Self {
            display_settings: Default::default(),
            view_settings: Default::default(),
        }
    }
}

/// `NodeDilateErode`.
#[derive(Clone)]
pub struct NodeDilateErode {
    /// `char falloff`
    pub falloff: i8,
}
impl Default for NodeDilateErode {
    fn default() -> Self {
        Self {
            falloff: 0,
        }
    }
}

/// `NodeMask`.
#[derive(Clone)]
pub struct NodeMask {
    /// `int size_x`
    pub size_x: i32,
    /// `int size_y`
    pub size_y: i32,
}
impl Default for NodeMask {
    fn default() -> Self {
        Self {
            size_x: 0,
            size_y: 0,
        }
    }
}

/// `NodeSetAlpha`.
#[derive(Clone)]
pub struct NodeSetAlpha {
    /// `char mode`
    pub mode: i8,
}
impl Default for NodeSetAlpha {
    fn default() -> Self {
        Self {
            mode: 0,
        }
    }
}

/// `NodeTexBase`.
#[derive(Clone)]
pub struct NodeTexBase {
    pub tex_mapping: TexMapping,
    pub color_mapping: ColorMapping,
}
impl Default for NodeTexBase {
    fn default() -> Self {
        Self {
            tex_mapping: Default::default(),
            color_mapping: Default::default(),
        }
    }
}

/// `NodeTexSky`.
#[derive(Clone)]
pub struct NodeTexSky {
    pub base: NodeTexBase,
    /// `int sky_model`
    pub sky_model: i32,
    /// `float sun_direction[3]`
    pub sun_direction: [f32; 3],
    /// `float turbidity`
    pub turbidity: f32,
    /// `float ground_albedo`
    pub ground_albedo: f32,
    /// `float sun_size`
    pub sun_size: f32,
    /// `float sun_intensity`
    pub sun_intensity: f32,
    /// `float sun_elevation`
    pub sun_elevation: f32,
    /// `float sun_rotation`
    pub sun_rotation: f32,
    /// `float altitude`
    pub altitude: f32,
    /// `float air_density`
    pub air_density: f32,
    /// `float aerosol_density`
    pub aerosol_density: f32,
    /// `float ozone_density`
    pub ozone_density: f32,
    /// `char sun_disc`
    pub sun_disc: i8,
}
impl Default for NodeTexSky {
    fn default() -> Self {
        Self {
            base: Default::default(),
            sky_model: 0,
            sun_direction: [0.0; 3],
            turbidity: 0.0,
            ground_albedo: 0.0,
            sun_size: 0.0,
            sun_intensity: 0.0,
            sun_elevation: 0.0,
            sun_rotation: 0.0,
            altitude: 0.0,
            air_density: 0.0,
            aerosol_density: 0.0,
            ozone_density: 0.0,
            sun_disc: 0,
        }
    }
}

/// `NodeTexImage`.
#[derive(Clone)]
pub struct NodeTexImage {
    pub base: NodeTexBase,
    pub iuser: ImageUser,
    /// `int color_space`
    pub color_space: i32,
    /// `int projection`
    pub projection: i32,
    /// `float projection_blend`
    pub projection_blend: f32,
    /// `int interpolation`
    pub interpolation: i32,
    /// `int extension`
    pub extension: i32,
}
impl Default for NodeTexImage {
    fn default() -> Self {
        Self {
            base: Default::default(),
            iuser: Default::default(),
            color_space: 0,
            projection: 0,
            projection_blend: 0.0,
            interpolation: 0,
            extension: 0,
        }
    }
}

/// `NodeTexChecker`.
#[derive(Clone)]
pub struct NodeTexChecker {
    pub base: NodeTexBase,
}
impl Default for NodeTexChecker {
    fn default() -> Self {
        Self {
            base: Default::default(),
        }
    }
}

/// `NodeTexBrick`.
#[derive(Clone)]
pub struct NodeTexBrick {
    pub base: NodeTexBase,
    /// `int offset_freq`
    pub offset_freq: i32,
    /// `int squash_freq`
    pub squash_freq: i32,
    /// `float offset`
    pub offset: f32,
    /// `float squash`
    pub squash: f32,
}
impl Default for NodeTexBrick {
    fn default() -> Self {
        Self {
            base: Default::default(),
            offset_freq: 0,
            squash_freq: 0,
            offset: 0.0,
            squash: 0.0,
        }
    }
}

/// `NodeTexEnvironment`.
#[derive(Clone)]
pub struct NodeTexEnvironment {
    pub base: NodeTexBase,
    pub iuser: ImageUser,
    /// `int color_space`
    pub color_space: i32,
    /// `int projection`
    pub projection: i32,
    /// `int interpolation`
    pub interpolation: i32,
}
impl Default for NodeTexEnvironment {
    fn default() -> Self {
        Self {
            base: Default::default(),
            iuser: Default::default(),
            color_space: 0,
            projection: 0,
            interpolation: 0,
        }
    }
}

/// `NodeTexGabor`.
#[derive(Clone)]
pub struct NodeTexGabor {
    pub base: NodeTexBase,
    /// `NodeGaborType type`
    pub type_: Nodegabortype,
}
impl Default for NodeTexGabor {
    fn default() -> Self {
        Self {
            base: Default::default(),
            type_: SHD_GABOR_TYPE_2D,
        }
    }
}

/// `NodeTexGradient`.
#[derive(Clone)]
pub struct NodeTexGradient {
    pub base: NodeTexBase,
    /// `int gradient_type`
    pub gradient_type: i32,
}
impl Default for NodeTexGradient {
    fn default() -> Self {
        Self {
            base: Default::default(),
            gradient_type: 0,
        }
    }
}

/// `NodeTexNoise`.
#[derive(Clone)]
pub struct NodeTexNoise {
    pub base: NodeTexBase,
    /// `int dimensions`
    pub dimensions: i32,
    /// `uint8_t type`
    pub type_: u8,
    /// `uint8_t normalize`
    pub normalize: u8,
}
impl Default for NodeTexNoise {
    fn default() -> Self {
        Self {
            base: Default::default(),
            dimensions: 0,
            type_: 0,
            normalize: 0,
        }
    }
}

/// `NodeTexVoronoi`.
#[derive(Clone)]
pub struct NodeTexVoronoi {
    pub base: NodeTexBase,
    /// `int dimensions`
    pub dimensions: i32,
    /// `int feature`
    pub feature: i32,
    /// `int distance`
    pub distance: i32,
    /// `int normalize`
    pub normalize: i32,
    /// `int coloring`
    pub coloring: i32,
}
impl Default for NodeTexVoronoi {
    fn default() -> Self {
        Self {
            base: Default::default(),
            dimensions: 0,
            feature: 0,
            distance: 0,
            normalize: 0,
            coloring: 0,
        }
    }
}

/// `NodeTexMusgrave`.
#[derive(Clone)]
pub struct NodeTexMusgrave {
    pub base: NodeTexBase,
    /// `int musgrave_type`
    pub musgrave_type: i32,
    /// `int dimensions`
    pub dimensions: i32,
}
impl Default for NodeTexMusgrave {
    fn default() -> Self {
        Self {
            base: Default::default(),
            musgrave_type: 0,
            dimensions: 0,
        }
    }
}

/// `NodeTexWave`.
#[derive(Clone)]
pub struct NodeTexWave {
    pub base: NodeTexBase,
    /// `int wave_type`
    pub wave_type: i32,
    /// `int bands_direction`
    pub bands_direction: i32,
    /// `int rings_direction`
    pub rings_direction: i32,
    /// `int wave_profile`
    pub wave_profile: i32,
}
impl Default for NodeTexWave {
    fn default() -> Self {
        Self {
            base: Default::default(),
            wave_type: 0,
            bands_direction: 0,
            rings_direction: 0,
            wave_profile: 0,
        }
    }
}

/// `NodeTexMagic`.
#[derive(Clone)]
pub struct NodeTexMagic {
    pub base: NodeTexBase,
    /// `int depth`
    pub depth: i32,
}
impl Default for NodeTexMagic {
    fn default() -> Self {
        Self {
            base: Default::default(),
            depth: 0,
        }
    }
}

/// `NodeShaderAttribute`.
#[derive(Clone)]
pub struct NodeShaderAttribute {
    /// `char name[256]`
    pub name: [u8; 256],
    /// `int type`
    pub type_: i32,
}
impl Default for NodeShaderAttribute {
    fn default() -> Self {
        Self {
            name: [0; 256],
            type_: 0,
        }
    }
}

/// `NodeShaderVectTransform`.
#[derive(Clone)]
pub struct NodeShaderVectTransform {
    /// `int type`
    pub type_: i32,
    /// `int convert_from`
    pub convert_from: i32,
    /// `int convert_to`
    pub convert_to: i32,
}
impl Default for NodeShaderVectTransform {
    fn default() -> Self {
        Self {
            type_: 0,
            convert_from: 0,
            convert_to: 0,
        }
    }
}

/// `NodeShaderPrincipled`.
#[derive(Clone)]
pub struct NodeShaderPrincipled {
    /// `char use_subsurface_auto_radius`
    pub use_subsurface_auto_radius: i8,
}
impl Default for NodeShaderPrincipled {
    fn default() -> Self {
        Self {
            use_subsurface_auto_radius: 0,
        }
    }
}

/// `NodeShaderHairPrincipled`.
#[derive(Clone)]
pub struct NodeShaderHairPrincipled {
    /// `short model`
    pub model: i16,
    /// `short parametrization`
    pub parametrization: i16,
}
impl Default for NodeShaderHairPrincipled {
    fn default() -> Self {
        Self {
            model: 0,
            parametrization: 0,
        }
    }
}

/// `TexNodeOutput`.
#[derive(Clone)]
pub struct TexNodeOutput {
    /// `char name[64]`
    pub name: [u8; 64],
}
impl Default for TexNodeOutput {
    fn default() -> Self {
        Self {
            name: [0; 64],
        }
    }
}

/// `NodeKeyingScreenData`.
#[derive(Clone)]
pub struct NodeKeyingScreenData {
    /// `char tracking_object[64]`
    pub tracking_object: [u8; 64],
    /// `float smoothness`
    pub smoothness: f32,
}
impl Default for NodeKeyingScreenData {
    fn default() -> Self {
        Self {
            tracking_object: [0; 64],
            smoothness: 0.0,
        }
    }
}

/// `NodeKeyingData`.
#[derive(Clone)]
pub struct NodeKeyingData {
    /// `float screen_balance`
    pub screen_balance: f32,
    /// `float despill_factor`
    pub despill_factor: f32,
    /// `float despill_balance`
    pub despill_balance: f32,
    /// `int edge_kernel_radius`
    pub edge_kernel_radius: i32,
    /// `float edge_kernel_tolerance`
    pub edge_kernel_tolerance: f32,
    /// `float clip_black`
    pub clip_black: f32,
    /// `float clip_white`
    pub clip_white: f32,
    /// `int dilate_distance`
    pub dilate_distance: i32,
    /// `int feather_distance`
    pub feather_distance: i32,
    /// `int feather_falloff`
    pub feather_falloff: i32,
    /// `int blur_pre`
    pub blur_pre: i32,
    /// `int blur_post`
    pub blur_post: i32,
}
impl Default for NodeKeyingData {
    fn default() -> Self {
        Self {
            screen_balance: 0.0,
            despill_factor: 0.0,
            despill_balance: 0.0,
            edge_kernel_radius: 0,
            edge_kernel_tolerance: 0.0,
            clip_black: 0.0,
            clip_white: 0.0,
            dilate_distance: 0,
            feather_distance: 0,
            feather_falloff: 0,
            blur_pre: 0,
            blur_post: 0,
        }
    }
}

/// `NodeTrackPosData`.
#[derive(Clone)]
pub struct NodeTrackPosData {
    /// `char tracking_object[64]`
    pub tracking_object: [u8; 64],
    /// `char track_name[64]`
    pub track_name: [u8; 64],
}
impl Default for NodeTrackPosData {
    fn default() -> Self {
        Self {
            tracking_object: [0; 64],
            track_name: [0; 64],
        }
    }
}

/// `NodeTransformData`.
#[derive(Clone)]
pub struct NodeTransformData {
    /// `short interpolation`
    pub interpolation: i16,
    /// `char extension_x`
    pub extension_x: i8,
    /// `char extension_y`
    pub extension_y: i8,
}
impl Default for NodeTransformData {
    fn default() -> Self {
        Self {
            interpolation: 0,
            extension_x: 0,
            extension_y: 0,
        }
    }
}

/// `NodeTranslateData`.
#[derive(Clone)]
pub struct NodeTranslateData {
    /// `char wrap_axis`
    pub wrap_axis: i8,
    /// `char relative`
    pub relative: i8,
    /// `short extension_x`
    pub extension_x: i16,
    /// `short extension_y`
    pub extension_y: i16,
    /// `short interpolation`
    pub interpolation: i16,
}
impl Default for NodeTranslateData {
    fn default() -> Self {
        Self {
            wrap_axis: 0,
            relative: 0,
            extension_x: 0,
            extension_y: 0,
            interpolation: 0,
        }
    }
}

/// `NodeRotateData`.
#[derive(Clone)]
pub struct NodeRotateData {
    /// `short interpolation`
    pub interpolation: i16,
    /// `char extension_x`
    pub extension_x: i8,
    /// `char extension_y`
    pub extension_y: i8,
}
impl Default for NodeRotateData {
    fn default() -> Self {
        Self {
            interpolation: 0,
            extension_x: 0,
            extension_y: 0,
        }
    }
}

/// `NodeScaleData`.
#[derive(Clone)]
pub struct NodeScaleData {
    /// `short interpolation`
    pub interpolation: i16,
    /// `char extension_x`
    pub extension_x: i8,
    /// `char extension_y`
    pub extension_y: i8,
}
impl Default for NodeScaleData {
    fn default() -> Self {
        Self {
            interpolation: 0,
            extension_x: 0,
            extension_y: 0,
        }
    }
}

/// `NodeCornerPinData`.
#[derive(Clone)]
pub struct NodeCornerPinData {
    /// `short interpolation`
    pub interpolation: i16,
    /// `char extension_x`
    pub extension_x: i8,
    /// `char extension_y`
    pub extension_y: i8,
}
impl Default for NodeCornerPinData {
    fn default() -> Self {
        Self {
            interpolation: 0,
            extension_x: 0,
            extension_y: 0,
        }
    }
}

/// `NodeDisplaceData`.
#[derive(Clone)]
pub struct NodeDisplaceData {
    /// `short interpolation`
    pub interpolation: i16,
    /// `char extension_x`
    pub extension_x: i8,
    /// `char extension_y`
    pub extension_y: i8,
}
impl Default for NodeDisplaceData {
    fn default() -> Self {
        Self {
            interpolation: 0,
            extension_x: 0,
            extension_y: 0,
        }
    }
}

/// `NodeMapUVData`.
#[derive(Clone)]
pub struct NodeMapUVData {
    /// `short interpolation`
    pub interpolation: i16,
    /// `char extension_x`
    pub extension_x: i8,
    /// `char extension_y`
    pub extension_y: i8,
}
impl Default for NodeMapUVData {
    fn default() -> Self {
        Self {
            interpolation: 0,
            extension_x: 0,
            extension_y: 0,
        }
    }
}

/// `NodePlaneTrackDeformData`.
#[derive(Clone)]
pub struct NodePlaneTrackDeformData {
    /// `char tracking_object[64]`
    pub tracking_object: [u8; 64],
    /// `char plane_track_name[64]`
    pub plane_track_name: [u8; 64],
    /// `char flag`
    pub flag: i8,
    /// `char motion_blur_samples`
    pub motion_blur_samples: i8,
    /// `float motion_blur_shutter`
    pub motion_blur_shutter: f32,
}
impl Default for NodePlaneTrackDeformData {
    fn default() -> Self {
        Self {
            tracking_object: [0; 64],
            plane_track_name: [0; 64],
            flag: 0,
            motion_blur_samples: 0,
            motion_blur_shutter: 0.0,
        }
    }
}

/// `NodeShaderScript`.
#[derive(Clone)]
pub struct NodeShaderScript {
    /// `int mode`
    pub mode: i32,
    /// `int flag`
    pub flag: i32,
    /// `char filepath[1024]`
    pub filepath: [u8; 1024],
    /// `char bytecode_hash[64]`
    pub bytecode_hash: [u8; 64],
    /// `char *bytecode`
    pub bytecode: usize,
}
impl Default for NodeShaderScript {
    fn default() -> Self {
        Self {
            mode: 0,
            flag: 0,
            filepath: [0; 1024],
            bytecode_hash: [0; 64],
            bytecode: NIL,
        }
    }
}

/// `NodeShaderTangent`.
#[derive(Clone)]
pub struct NodeShaderTangent {
    /// `int direction_type`
    pub direction_type: i32,
    /// `int axis`
    pub axis: i32,
    /// `char uv_map[64]`
    pub uv_map: [u8; 64],
}
impl Default for NodeShaderTangent {
    fn default() -> Self {
        Self {
            direction_type: 0,
            axis: 0,
            uv_map: [0; 64],
        }
    }
}

/// `NodeShaderNormalMap`.
#[derive(Clone)]
pub struct NodeShaderNormalMap {
    /// `int space`
    pub space: i32,
    /// `char uv_map[64]`
    pub uv_map: [u8; 64],
    /// `char convention`
    pub convention: i8,
    /// `char base`
    pub base: i8,
}
impl Default for NodeShaderNormalMap {
    fn default() -> Self {
        Self {
            space: 0,
            uv_map: [0; 64],
            convention: SHD_NORMAL_MAP_CONVENTION_OPENGL as i8,
            base: SHD_NORMAL_MAP_BASE_DISPLACED as i8,
        }
    }
}

/// `NodeRadialTiling`.
#[derive(Clone)]
pub struct NodeRadialTiling {
    /// `uint8_t normalize`
    pub normalize: u8,
}
impl Default for NodeRadialTiling {
    fn default() -> Self {
        Self {
            normalize: 0,
        }
    }
}

/// `NodeShaderUVMap`.
#[derive(Clone)]
pub struct NodeShaderUVMap {
    /// `char uv_map[64]`
    pub uv_map: [u8; 64],
}
impl Default for NodeShaderUVMap {
    fn default() -> Self {
        Self {
            uv_map: [0; 64],
        }
    }
}

/// `NodeShaderVertexColor`.
#[derive(Clone)]
pub struct NodeShaderVertexColor {
    /// `char layer_name[64]`
    pub layer_name: [u8; 64],
}
impl Default for NodeShaderVertexColor {
    fn default() -> Self {
        Self {
            layer_name: [0; 64],
        }
    }
}

/// `NodeShaderTexIES`.
#[derive(Clone)]
pub struct NodeShaderTexIES {
    /// `int mode`
    pub mode: i32,
    /// `char filepath[1024]`
    pub filepath: [u8; 1024],
}
impl Default for NodeShaderTexIES {
    fn default() -> Self {
        Self {
            mode: 0,
            filepath: [0; 1024],
        }
    }
}

/// `NodeShaderOutputAOV`.
#[derive(Clone)]
pub struct NodeShaderOutputAOV {
    /// `char name[64]`
    pub name: [u8; 64],
}
impl Default for NodeShaderOutputAOV {
    fn default() -> Self {
        Self {
            name: [0; 64],
        }
    }
}

/// `NodeSunBeams`.
#[derive(Clone)]
pub struct NodeSunBeams {
    /// `float source[2]`
    pub source: [f32; 2],
    /// `float ray_length`
    pub ray_length: f32,
}
impl Default for NodeSunBeams {
    fn default() -> Self {
        Self {
            source: [0.0; 2],
            ray_length: 0.0,
        }
    }
}

/// `CryptomatteEntry`.
#[derive(Clone)]
pub struct CryptomatteEntry {
    /// `CryptomatteEntry *next`
    pub next: usize,
    /// `CryptomatteEntry *prev`
    pub prev: usize,
    /// `float encoded_hash`
    pub encoded_hash: f32,
    /// `char name[64]`
    pub name: [u8; 64],
}
impl Default for CryptomatteEntry {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            encoded_hash: 0.0,
            name: [0; 64],
        }
    }
}

/// `CryptomatteLayer`.
#[derive(Clone)]
pub struct CryptomatteLayer {
    /// `CryptomatteEntry *next`
    pub next: usize,
    /// `CryptomatteEntry *prev`
    pub prev: usize,
    /// `char name[64]`
    pub name: [u8; 64],
}
impl Default for CryptomatteLayer {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
        }
    }
}

/// `NodeCryptomatte_Runtime`.
#[derive(Clone)]
pub struct NodeCryptomatte_Runtime {
    /// `ListBaseT<CryptomatteLayer>`
    pub layers: ListBase,
    /// `float add[3]`
    pub add: [f32; 3],
    /// `float remove[3]`
    pub remove: [f32; 3],
}
impl Default for NodeCryptomatte_Runtime {
    fn default() -> Self {
        Self {
            layers: ListBase::default(),
            add: [0.0; 3],
            remove: [0.0; 3],
        }
    }
}

/// `NodeCryptomatte`.
#[derive(Clone)]
pub struct NodeCryptomatte {
    pub iuser: ImageUser,
    /// `ListBaseT<CryptomatteEntry>`
    pub entries: ListBase,
    /// `char layer_name[64]`
    pub layer_name: [u8; 64],
    /// `char *matte_id`
    pub matte_id: usize,
    /// `int inputs_num`
    pub inputs_num: i32,
    pub runtime: NodeCryptomatte_Runtime,
}
impl Default for NodeCryptomatte {
    fn default() -> Self {
        Self {
            iuser: Default::default(),
            entries: ListBase::default(),
            layer_name: [0; 64],
            matte_id: NIL,
            inputs_num: 0,
            runtime: Default::default(),
        }
    }
}

/// `NodeDenoise`.
#[derive(Clone)]
pub struct NodeDenoise {
    /// `char hdr`
    pub hdr: i8,
    /// `char prefilter`
    pub prefilter: i8,
    /// `char quality`
    pub quality: i8,
}
impl Default for NodeDenoise {
    fn default() -> Self {
        Self {
            hdr: 0,
            prefilter: 0,
            quality: 0,
        }
    }
}

/// `NodeMapRange`.
#[derive(Clone)]
pub struct NodeMapRange {
    /// `uint8_t data_type`
    pub data_type: u8,
    /// `uint8_t interpolation_type`
    pub interpolation_type: u8,
    /// `uint8_t clamp`
    pub clamp: u8,
}
impl Default for NodeMapRange {
    fn default() -> Self {
        Self {
            data_type: 0,
            interpolation_type: 0,
            clamp: 0,
        }
    }
}

/// `NodeRandomValue`.
#[derive(Clone)]
pub struct NodeRandomValue {
    /// `uint8_t data_type`
    pub data_type: u8,
}
impl Default for NodeRandomValue {
    fn default() -> Self {
        Self {
            data_type: 0,
        }
    }
}

/// `NodeAccumulateField`.
#[derive(Clone)]
pub struct NodeAccumulateField {
    /// `uint8_t data_type`
    pub data_type: u8,
    /// `uint8_t domain`
    pub domain: u8,
}
impl Default for NodeAccumulateField {
    fn default() -> Self {
        Self {
            data_type: 0,
            domain: 0,
        }
    }
}

/// `NodeInputBool`.
#[derive(Clone)]
pub struct NodeInputBool {
    /// `uint8_t boolean`
    pub boolean: u8,
}
impl Default for NodeInputBool {
    fn default() -> Self {
        Self {
            boolean: 0,
        }
    }
}

/// `NodeInputInt`.
#[derive(Clone)]
pub struct NodeInputInt {
    /// `int integer`
    pub integer: i32,
}
impl Default for NodeInputInt {
    fn default() -> Self {
        Self {
            integer: 0,
        }
    }
}

/// `NodeInputMenu`.
#[derive(Clone)]
pub struct NodeInputMenu {
    /// `int value`
    pub value: i32,
}
impl Default for NodeInputMenu {
    fn default() -> Self {
        Self {
            value: 0,
        }
    }
}

/// `NodeInputRotation`.
#[derive(Clone)]
pub struct NodeInputRotation {
    /// `float rotation_euler[3]`
    pub rotation_euler: [f32; 3],
}
impl Default for NodeInputRotation {
    fn default() -> Self {
        Self {
            rotation_euler: [0.0; 3],
        }
    }
}

/// `NodeInputVector`.
#[derive(Clone)]
pub struct NodeInputVector {
    /// `float vector[4]`
    pub vector: [f32; 4],
    /// `int dimensions`
    pub dimensions: i32,
}
impl Default for NodeInputVector {
    fn default() -> Self {
        Self {
            vector: [0.0; 4],
            dimensions: 3,
        }
    }
}

/// `NodeInputIntVector`.
#[derive(Clone)]
pub struct NodeInputIntVector {
    /// `int vector[3]`
    pub vector: [i32; 3],
    /// `int dimensions`
    pub dimensions: i32,
}
impl Default for NodeInputIntVector {
    fn default() -> Self {
        Self {
            vector: [0; 3],
            dimensions: 3,
        }
    }
}

/// `NodeInputColor`.
#[derive(Clone)]
pub struct NodeInputColor {
    /// `float color[4]`
    pub color: [f32; 4],
}
impl Default for NodeInputColor {
    fn default() -> Self {
        Self {
            color: [0.0; 4],
        }
    }
}

/// `NodeInputString`.
#[derive(Clone)]
pub struct NodeInputString {
    /// `char *string`
    pub string: usize,
    pub textbox_state: TextboxState,
}
impl Default for NodeInputString {
    fn default() -> Self {
        Self {
            string: NIL,
            textbox_state: Default::default(),
        }
    }
}

/// `NodeGeometryExtrudeMesh`.
#[derive(Clone)]
pub struct NodeGeometryExtrudeMesh {
    /// `GeometryNodeExtrudeMeshMode mode`
    pub mode: Geometrynodeextrudemeshmode,
}
impl Default for NodeGeometryExtrudeMesh {
    fn default() -> Self {
        Self {
            mode: GEO_NODE_EXTRUDE_MESH_VERTICES,
        }
    }
}

/// `NodeGeometryObjectInfo`.
#[derive(Clone)]
pub struct NodeGeometryObjectInfo {
    /// `GeometryNodeTransformSpace transform_space`
    pub transform_space: Geometrynodetransformspace,
}
impl Default for NodeGeometryObjectInfo {
    fn default() -> Self {
        Self {
            transform_space: GEO_NODE_TRANSFORM_SPACE_ORIGINAL,
        }
    }
}

/// `NodeGeometryCollectionInfo`.
#[derive(Clone)]
pub struct NodeGeometryCollectionInfo {
    /// `GeometryNodeTransformSpace transform_space`
    pub transform_space: Geometrynodetransformspace,
}
impl Default for NodeGeometryCollectionInfo {
    fn default() -> Self {
        Self {
            transform_space: GEO_NODE_TRANSFORM_SPACE_ORIGINAL,
        }
    }
}

/// `NodeGeometryProximity`.
#[derive(Clone)]
pub struct NodeGeometryProximity {
    /// `GeometryNodeProximityTargetType target_element`
    pub target_element: Geometrynodeproximitytargettype,
}
impl Default for NodeGeometryProximity {
    fn default() -> Self {
        Self {
            target_element: GEO_NODE_PROX_TARGET_POINTS,
        }
    }
}

/// `NodeGeometryVolumeToMesh`.
#[derive(Clone)]
pub struct NodeGeometryVolumeToMesh {
    /// `uint8_t resolution_mode`
    pub resolution_mode: u8,
}
impl Default for NodeGeometryVolumeToMesh {
    fn default() -> Self {
        Self {
            resolution_mode: 0,
        }
    }
}

/// `NodeGeometryMeshToVolume`.
#[derive(Clone)]
pub struct NodeGeometryMeshToVolume {
    /// `uint8_t resolution_mode`
    pub resolution_mode: u8,
}
impl Default for NodeGeometryMeshToVolume {
    fn default() -> Self {
        Self {
            resolution_mode: 0,
        }
    }
}

/// `NodeGeometrySubdivisionSurface`.
#[derive(Clone)]
pub struct NodeGeometrySubdivisionSurface {
    /// `uint8_t uv_smooth`
    pub uv_smooth: u8,
    /// `uint8_t boundary_smooth`
    pub boundary_smooth: u8,
}
impl Default for NodeGeometrySubdivisionSurface {
    fn default() -> Self {
        Self {
            uv_smooth: 0,
            boundary_smooth: 0,
        }
    }
}

/// `NodeGeometryMeshCircle`.
#[derive(Clone)]
pub struct NodeGeometryMeshCircle {
    /// `GeometryNodeMeshCircleFillType fill_type`
    pub fill_type: Geometrynodemeshcirclefilltype,
}
impl Default for NodeGeometryMeshCircle {
    fn default() -> Self {
        Self {
            fill_type: GEO_NODE_MESH_CIRCLE_FILL_NONE,
        }
    }
}

/// `NodeGeometryMeshCylinder`.
#[derive(Clone)]
pub struct NodeGeometryMeshCylinder {
    /// `GeometryNodeMeshCircleFillType fill_type`
    pub fill_type: Geometrynodemeshcirclefilltype,
}
impl Default for NodeGeometryMeshCylinder {
    fn default() -> Self {
        Self {
            fill_type: GEO_NODE_MESH_CIRCLE_FILL_NONE,
        }
    }
}

/// `NodeGeometryMeshCone`.
#[derive(Clone)]
pub struct NodeGeometryMeshCone {
    /// `GeometryNodeMeshCircleFillType fill_type`
    pub fill_type: Geometrynodemeshcirclefilltype,
}
impl Default for NodeGeometryMeshCone {
    fn default() -> Self {
        Self {
            fill_type: GEO_NODE_MESH_CIRCLE_FILL_NONE,
        }
    }
}

/// `NodeGeometryMergeByDistance`.
#[derive(Clone)]
pub struct NodeGeometryMergeByDistance {
    /// `GeometryNodeMergeByDistanceMode mode`
    pub mode: Geometrynodemergebydistancemode,
}
impl Default for NodeGeometryMergeByDistance {
    fn default() -> Self {
        Self {
            mode: GEO_NODE_MERGE_BY_DISTANCE_MODE_ALL,
        }
    }
}

/// `NodeGeometryMeshLine`.
#[derive(Clone)]
pub struct NodeGeometryMeshLine {
    /// `GeometryNodeMeshLineMode mode`
    pub mode: Geometrynodemeshlinemode,
    /// `GeometryNodeMeshLineCountMode count_mode`
    pub count_mode: Geometrynodemeshlinecountmode,
}
impl Default for NodeGeometryMeshLine {
    fn default() -> Self {
        Self {
            mode: GEO_NODE_MESH_LINE_MODE_END_POINTS,
            count_mode: GEO_NODE_MESH_LINE_COUNT_TOTAL,
        }
    }
}

/// `NodeSwitch`.
#[derive(Clone)]
pub struct NodeSwitch {
    /// `eNodeSocketDatatype input_type`
    pub input_type: Nodesocketdatatype,
}
impl Default for NodeSwitch {
    fn default() -> Self {
        Self {
            input_type: Default::default(),
        }
    }
}

/// `NodeEnumItem`.
#[derive(Clone)]
pub struct NodeEnumItem {
    /// `char *name`
    pub name: usize,
    /// `char *description`
    pub description: usize,
    /// `int32_t identifier`
    pub identifier: i32,
}
impl Default for NodeEnumItem {
    fn default() -> Self {
        Self {
            name: NIL,
            description: NIL,
            identifier: 0,
        }
    }
}

/// `NodeEnumDefinition`.
#[derive(Clone)]
pub struct NodeEnumDefinition {
    /// `NodeEnumItem *items_array`
    pub items_array: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int active_index`
    pub active_index: i32,
    /// `uint32_t next_identifier`
    pub next_identifier: u32,
}
impl Default for NodeEnumDefinition {
    fn default() -> Self {
        Self {
            items_array: NIL,
            items_num: 0,
            active_index: 0,
            next_identifier: 0,
        }
    }
}

/// `NodeMenuSwitch`.
#[derive(Clone)]
pub struct NodeMenuSwitch {
    pub enum_definition: NodeEnumDefinition,
    /// `eNodeSocketDatatype data_type`
    pub data_type: Nodesocketdatatype,
}
impl Default for NodeMenuSwitch {
    fn default() -> Self {
        Self {
            enum_definition: Default::default(),
            data_type: Default::default(),
        }
    }
}

/// `NodeGeometryCurveSplineType`.
#[derive(Clone)]
pub struct NodeGeometryCurveSplineType {
    /// `uint8_t spline_type`
    pub spline_type: u8,
}
impl Default for NodeGeometryCurveSplineType {
    fn default() -> Self {
        Self {
            spline_type: 0,
        }
    }
}

/// `NodeGeometrySetCurveHandlePositions`.
#[derive(Clone)]
pub struct NodeGeometrySetCurveHandlePositions {
    /// `GeometryNodeCurveHandleMode mode`
    pub mode: Geometrynodecurvehandlemode,
}
impl Default for NodeGeometrySetCurveHandlePositions {
    fn default() -> Self {
        Self {
            mode: Default::default(),
        }
    }
}

/// `NodeGeometryCurveSetHandles`.
#[derive(Clone)]
pub struct NodeGeometryCurveSetHandles {
    /// `GeometryNodeCurveHandleType handle_type`
    pub handle_type: Geometrynodecurvehandletype,
    /// `GeometryNodeCurveHandleMode mode`
    pub mode: Geometrynodecurvehandlemode,
}
impl Default for NodeGeometryCurveSetHandles {
    fn default() -> Self {
        Self {
            handle_type: GEO_NODE_CURVE_HANDLE_FREE,
            mode: Default::default(),
        }
    }
}

/// `NodeGeometryCurveSelectHandles`.
#[derive(Clone)]
pub struct NodeGeometryCurveSelectHandles {
    /// `GeometryNodeCurveHandleType handle_type`
    pub handle_type: Geometrynodecurvehandletype,
    /// `GeometryNodeCurveHandleMode mode`
    pub mode: Geometrynodecurvehandlemode,
}
impl Default for NodeGeometryCurveSelectHandles {
    fn default() -> Self {
        Self {
            handle_type: GEO_NODE_CURVE_HANDLE_FREE,
            mode: Default::default(),
        }
    }
}

/// `NodeGeometryCurvePrimitiveArc`.
#[derive(Clone)]
pub struct NodeGeometryCurvePrimitiveArc {
    /// `GeometryNodeCurvePrimitiveArcMode mode`
    pub mode: Geometrynodecurveprimitivearcmode,
}
impl Default for NodeGeometryCurvePrimitiveArc {
    fn default() -> Self {
        Self {
            mode: GEO_NODE_CURVE_PRIMITIVE_ARC_TYPE_POINTS,
        }
    }
}

/// `NodeGeometryCurvePrimitiveLine`.
#[derive(Clone)]
pub struct NodeGeometryCurvePrimitiveLine {
    /// `GeometryNodeCurvePrimitiveLineMode mode`
    pub mode: Geometrynodecurveprimitivelinemode,
}
impl Default for NodeGeometryCurvePrimitiveLine {
    fn default() -> Self {
        Self {
            mode: GEO_NODE_CURVE_PRIMITIVE_LINE_MODE_POINTS,
        }
    }
}

/// `NodeGeometryCurvePrimitiveCircle`.
#[derive(Clone)]
pub struct NodeGeometryCurvePrimitiveCircle {
    /// `uint8_t mode`
    pub mode: u8,
}
impl Default for NodeGeometryCurvePrimitiveCircle {
    fn default() -> Self {
        Self {
            mode: 0,
        }
    }
}

/// `NodeGeometryCurvePrimitiveQuad`.
#[derive(Clone)]
pub struct NodeGeometryCurvePrimitiveQuad {
    /// `GeometryNodeCurvePrimitiveQuadMode mode`
    pub mode: Geometrynodecurveprimitivequadmode,
}
impl Default for NodeGeometryCurvePrimitiveQuad {
    fn default() -> Self {
        Self {
            mode: GEO_NODE_CURVE_PRIMITIVE_QUAD_MODE_RECTANGLE,
        }
    }
}

/// `NodeGeometryCurveResample`.
#[derive(Clone)]
pub struct NodeGeometryCurveResample {
    /// `GeometryNodeCurveResampleMode mode`
    pub mode: Geometrynodecurveresamplemode,
    /// `uint8_t keep_last_segment`
    pub keep_last_segment: u8,
}
impl Default for NodeGeometryCurveResample {
    fn default() -> Self {
        Self {
            mode: GEO_NODE_CURVE_RESAMPLE_COUNT,
            keep_last_segment: 0,
        }
    }
}

/// `NodeGeometryCurveFillet`.
#[derive(Clone)]
pub struct NodeGeometryCurveFillet {
    /// `GeometryNodeCurveFilletMode mode`
    pub mode: Geometrynodecurvefilletmode,
}
impl Default for NodeGeometryCurveFillet {
    fn default() -> Self {
        Self {
            mode: GEO_NODE_CURVE_FILLET_BEZIER,
        }
    }
}

/// `NodeGeometryCurveTrim`.
#[derive(Clone)]
pub struct NodeGeometryCurveTrim {
    /// `GeometryNodeCurveSampleMode mode`
    pub mode: Geometrynodecurvesamplemode,
}
impl Default for NodeGeometryCurveTrim {
    fn default() -> Self {
        Self {
            mode: GEO_NODE_CURVE_SAMPLE_FACTOR,
        }
    }
}

/// `NodeGeometryCurveToPoints`.
#[derive(Clone)]
pub struct NodeGeometryCurveToPoints {
    /// `GeometryNodeCurveResampleMode mode`
    pub mode: Geometrynodecurveresamplemode,
}
impl Default for NodeGeometryCurveToPoints {
    fn default() -> Self {
        Self {
            mode: GEO_NODE_CURVE_RESAMPLE_COUNT,
        }
    }
}

/// `NodeGeometryCurveSample`.
#[derive(Clone)]
pub struct NodeGeometryCurveSample {
    /// `GeometryNodeCurveSampleMode mode`
    pub mode: Geometrynodecurvesamplemode,
    /// `int8_t use_all_curves`
    pub use_all_curves: i8,
    /// `int8_t data_type`
    pub data_type: i8,
}
impl Default for NodeGeometryCurveSample {
    fn default() -> Self {
        Self {
            mode: GEO_NODE_CURVE_SAMPLE_FACTOR,
            use_all_curves: 0,
            data_type: 0,
        }
    }
}

/// `NodeGeometryTransferAttribute`.
#[derive(Clone)]
pub struct NodeGeometryTransferAttribute {
    /// `int8_t data_type`
    pub data_type: i8,
    /// `int8_t domain`
    pub domain: i8,
    /// `GeometryNodeAttributeTransferMode mode`
    pub mode: Geometrynodeattributetransfermode,
}
impl Default for NodeGeometryTransferAttribute {
    fn default() -> Self {
        Self {
            data_type: 0,
            domain: 0,
            mode: GEO_NODE_ATTRIBUTE_TRANSFER_NEAREST_FACE_INTERPOLATED,
        }
    }
}

/// `NodeGeometrySampleIndex`.
#[derive(Clone)]
pub struct NodeGeometrySampleIndex {
    /// `int8_t data_type`
    pub data_type: i8,
    /// `int8_t domain`
    pub domain: i8,
    /// `int8_t clamp`
    pub clamp: i8,
}
impl Default for NodeGeometrySampleIndex {
    fn default() -> Self {
        Self {
            data_type: 0,
            domain: 0,
            clamp: 0,
        }
    }
}

/// `NodeRaycastSampleAttributeItem`.
#[derive(Clone)]
pub struct NodeRaycastSampleAttributeItem {
    /// `int8_t data_type`
    pub data_type: i8,
    /// `int identifier`
    pub identifier: i32,
    /// `char *name`
    pub name: usize,
}
impl Default for NodeRaycastSampleAttributeItem {
    fn default() -> Self {
        Self {
            data_type: 0,
            identifier: Default::default(),
            name: NIL,
        }
    }
}

/// `NodeGeometryRaycast`.
#[derive(Clone)]
pub struct NodeGeometryRaycast {
    /// `GeometryNodeRaycastMapMode mapping`
    pub mapping: Geometrynoderaycastmapmode,
    /// `int8_t data_type`
    pub data_type: i8,
}
impl Default for NodeGeometryRaycast {
    fn default() -> Self {
        Self {
            mapping: GEO_NODE_RAYCAST_INTERPOLATED,
            data_type: 0,
        }
    }
}

/// `NodeShaderRaycast`.
#[derive(Clone)]
pub struct NodeShaderRaycast {
    /// `int next_identifier`
    pub next_identifier: i32,
    /// `NodeRaycastSampleAttributeItem *sample_attribute_items`
    pub sample_attribute_items: usize,
    /// `int sample_attribute_items_num`
    pub sample_attribute_items_num: i32,
    /// `int active_index`
    pub active_index: i32,
}
impl Default for NodeShaderRaycast {
    fn default() -> Self {
        Self {
            next_identifier: 0,
            sample_attribute_items: NIL,
            sample_attribute_items_num: 0,
            active_index: 0,
        }
    }
}

/// `NodeGeometryCurveFill`.
#[derive(Clone)]
pub struct NodeGeometryCurveFill {
    /// `GeometryNodeCurveFillMode mode`
    pub mode: Geometrynodecurvefillmode,
    /// `GeometryNodeCurveFillRule fill_rule`
    pub fill_rule: Geometrynodecurvefillrule,
}
impl Default for NodeGeometryCurveFill {
    fn default() -> Self {
        Self {
            mode: GEO_NODE_CURVE_FILL_MODE_TRIANGULATED,
            fill_rule: GEO_NODE_CURVE_FILL_RULE_EVEN_ODD,
        }
    }
}

/// `NodeGeometryMeshToPoints`.
#[derive(Clone)]
pub struct NodeGeometryMeshToPoints {
    /// `GeometryNodeMeshToPointsMode mode`
    pub mode: Geometrynodemeshtopointsmode,
}
impl Default for NodeGeometryMeshToPoints {
    fn default() -> Self {
        Self {
            mode: GEO_NODE_MESH_TO_POINTS_VERTICES,
        }
    }
}

/// `NodeGeometryAttributeCaptureItem`.
#[derive(Clone)]
pub struct NodeGeometryAttributeCaptureItem {
    /// `int8_t data_type`
    pub data_type: i8,
    /// `int identifier`
    pub identifier: i32,
    /// `char *name`
    pub name: usize,
}
impl Default for NodeGeometryAttributeCaptureItem {
    fn default() -> Self {
        Self {
            data_type: 0,
            identifier: 0,
            name: NIL,
        }
    }
}

/// `NodeGeometryAttributeCapture`.
#[derive(Clone)]
pub struct NodeGeometryAttributeCapture {
    /// `int8_t data_type_legacy`
    pub data_type_legacy: i8,
    /// `int8_t domain`
    pub domain: i8,
    /// `int next_identifier`
    pub next_identifier: i32,
    /// `NodeGeometryAttributeCaptureItem *capture_items`
    pub capture_items: usize,
    /// `int capture_items_num`
    pub capture_items_num: i32,
    /// `int active_index`
    pub active_index: i32,
}
impl Default for NodeGeometryAttributeCapture {
    fn default() -> Self {
        Self {
            data_type_legacy: 0,
            domain: 0,
            next_identifier: 0,
            capture_items: NIL,
            capture_items_num: 0,
            active_index: 0,
        }
    }
}

/// `NodeGeometryStoreNamedAttribute`.
#[derive(Clone)]
pub struct NodeGeometryStoreNamedAttribute {
    /// `int8_t data_type`
    pub data_type: i8,
    /// `int8_t domain`
    pub domain: i8,
}
impl Default for NodeGeometryStoreNamedAttribute {
    fn default() -> Self {
        Self {
            data_type: 0,
            domain: 0,
        }
    }
}

/// `NodeGeometryInputNamedAttribute`.
#[derive(Clone)]
pub struct NodeGeometryInputNamedAttribute {
    /// `int8_t data_type`
    pub data_type: i8,
}
impl Default for NodeGeometryInputNamedAttribute {
    fn default() -> Self {
        Self {
            data_type: 0,
        }
    }
}

/// `NodeGeometryStringToCurves`.
#[derive(Clone)]
pub struct NodeGeometryStringToCurves {
    /// `GeometryNodeStringToCurvesOverflowMode overflow`
    pub overflow: Geometrynodestringtocurvesoverflowmode,
    /// `GeometryNodeStringToCurvesAlignXMode align_x`
    pub align_x: Geometrynodestringtocurvesalignxmode,
    /// `GeometryNodeStringToCurvesAlignYMode align_y`
    pub align_y: Geometrynodestringtocurvesalignymode,
    /// `GeometryNodeStringToCurvesPivotMode pivot_mode`
    pub pivot_mode: Geometrynodestringtocurvespivotmode,
}
impl Default for NodeGeometryStringToCurves {
    fn default() -> Self {
        Self {
            overflow: GEO_NODE_STRING_TO_CURVES_MODE_OVERFLOW,
            align_x: GEO_NODE_STRING_TO_CURVES_ALIGN_X_LEFT,
            align_y: GEO_NODE_STRING_TO_CURVES_ALIGN_Y_TOP_BASELINE,
            pivot_mode: GEO_NODE_STRING_TO_CURVES_PIVOT_MODE_MIDPOINT,
        }
    }
}

/// `NodeGeometryDeleteGeometry`.
#[derive(Clone)]
pub struct NodeGeometryDeleteGeometry {
    /// `int8_t domain`
    pub domain: i8,
    /// `GeometryNodeDeleteGeometryMode mode`
    pub mode: Geometrynodedeletegeometrymode,
}
impl Default for NodeGeometryDeleteGeometry {
    fn default() -> Self {
        Self {
            domain: 0,
            mode: GEO_NODE_DELETE_GEOMETRY_MODE_ALL,
        }
    }
}

/// `NodeGeometryDuplicateElements`.
#[derive(Clone)]
pub struct NodeGeometryDuplicateElements {
    /// `int8_t domain`
    pub domain: i8,
}
impl Default for NodeGeometryDuplicateElements {
    fn default() -> Self {
        Self {
            domain: 0,
        }
    }
}

/// `NodeGeometryMergeLayers`.
#[derive(Clone)]
pub struct NodeGeometryMergeLayers {
    /// `int8_t mode`
    pub mode: i8,
}
impl Default for NodeGeometryMergeLayers {
    fn default() -> Self {
        Self {
            mode: 0,
        }
    }
}

/// `NodeGeometrySeparateGeometry`.
#[derive(Clone)]
pub struct NodeGeometrySeparateGeometry {
    /// `int8_t domain`
    pub domain: i8,
}
impl Default for NodeGeometrySeparateGeometry {
    fn default() -> Self {
        Self {
            domain: 0,
        }
    }
}

/// `NodeGeometryImageTexture`.
#[derive(Clone)]
pub struct NodeGeometryImageTexture {
    /// `int8_t interpolation`
    pub interpolation: i8,
    /// `int8_t extension`
    pub extension: i8,
}
impl Default for NodeGeometryImageTexture {
    fn default() -> Self {
        Self {
            interpolation: 0,
            extension: 0,
        }
    }
}

/// `NodeGeometryViewerItem`.
#[derive(Clone)]
pub struct NodeGeometryViewerItem {
    /// `char *name`
    pub name: usize,
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    /// `uint8_t flag`
    pub flag: u8,
    /// `int identifier`
    pub identifier: i32,
}
impl Default for NodeGeometryViewerItem {
    fn default() -> Self {
        Self {
            name: NIL,
            socket_type: Default::default(),
            flag: 0,
            identifier: 0,
        }
    }
}

/// `NodeGeometryViewer`.
#[derive(Clone)]
pub struct NodeGeometryViewer {
    /// `NodeGeometryViewerItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int active_index`
    pub active_index: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
    /// `int8_t data_type_legacy`
    pub data_type_legacy: i8,
    /// `int8_t domain`
    pub domain: i8,
}
impl Default for NodeGeometryViewer {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            active_index: 0,
            next_identifier: 0,
            data_type_legacy: 0,
            domain: 0,
        }
    }
}

/// `NodeGeometryUVUnwrap`.
#[derive(Clone)]
pub struct NodeGeometryUVUnwrap {
    /// `GeometryNodeUVUnwrapMethod method`
    pub method: Geometrynodeuvunwrapmethod,
}
impl Default for NodeGeometryUVUnwrap {
    fn default() -> Self {
        Self {
            method: GEO_NODE_UV_UNWRAP_METHOD_ANGLE_BASED,
        }
    }
}

/// `NodeSimulationItem`.
#[derive(Clone)]
pub struct NodeSimulationItem {
    /// `char *name`
    pub name: usize,
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    /// `short attribute_domain`
    pub attribute_domain: i16,
    /// `int identifier`
    pub identifier: i32,
}
impl Default for NodeSimulationItem {
    fn default() -> Self {
        Self {
            name: NIL,
            socket_type: Default::default(),
            attribute_domain: 0,
            identifier: 0,
        }
    }
}

/// `NodeGeometrySimulationInput`.
#[derive(Clone)]
pub struct NodeGeometrySimulationInput {
    /// `int32_t output_node_id`
    pub output_node_id: i32,
}
impl Default for NodeGeometrySimulationInput {
    fn default() -> Self {
        Self {
            output_node_id: 0,
        }
    }
}

/// `NodeGeometrySimulationOutput`.
#[derive(Clone)]
pub struct NodeGeometrySimulationOutput {
    /// `NodeSimulationItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int active_index`
    pub active_index: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
}
impl Default for NodeGeometrySimulationOutput {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            active_index: 0,
            next_identifier: 0,
        }
    }
}

/// `NodeRepeatItem`.
#[derive(Clone)]
pub struct NodeRepeatItem {
    /// `char *name`
    pub name: usize,
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    /// `int identifier`
    pub identifier: i32,
}
impl Default for NodeRepeatItem {
    fn default() -> Self {
        Self {
            name: NIL,
            socket_type: Default::default(),
            identifier: 0,
        }
    }
}

/// `NodeGeometryRepeatInput`.
#[derive(Clone)]
pub struct NodeGeometryRepeatInput {
    /// `int32_t output_node_id`
    pub output_node_id: i32,
}
impl Default for NodeGeometryRepeatInput {
    fn default() -> Self {
        Self {
            output_node_id: 0,
        }
    }
}

/// `NodeGeometryRepeatOutput`.
#[derive(Clone)]
pub struct NodeGeometryRepeatOutput {
    /// `NodeRepeatItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int active_index`
    pub active_index: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
    /// `int inspection_index`
    pub inspection_index: i32,
}
impl Default for NodeGeometryRepeatOutput {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            active_index: 0,
            next_identifier: 0,
            inspection_index: 0,
        }
    }
}

/// `NodeGeometryForeachGeometryElementInput`.
#[derive(Clone)]
pub struct NodeGeometryForeachGeometryElementInput {
    /// `int32_t output_node_id`
    pub output_node_id: i32,
}
impl Default for NodeGeometryForeachGeometryElementInput {
    fn default() -> Self {
        Self {
            output_node_id: 0,
        }
    }
}

/// `NodeForeachGeometryElementInputItem`.
#[derive(Clone)]
pub struct NodeForeachGeometryElementInputItem {
    /// `char *name`
    pub name: usize,
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    /// `int identifier`
    pub identifier: i32,
}
impl Default for NodeForeachGeometryElementInputItem {
    fn default() -> Self {
        Self {
            name: NIL,
            socket_type: Default::default(),
            identifier: 0,
        }
    }
}

/// `NodeForeachGeometryElementMainItem`.
#[derive(Clone)]
pub struct NodeForeachGeometryElementMainItem {
    /// `char *name`
    pub name: usize,
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    /// `int identifier`
    pub identifier: i32,
}
impl Default for NodeForeachGeometryElementMainItem {
    fn default() -> Self {
        Self {
            name: NIL,
            socket_type: Default::default(),
            identifier: 0,
        }
    }
}

/// `NodeForeachGeometryElementGenerationItem`.
#[derive(Clone)]
pub struct NodeForeachGeometryElementGenerationItem {
    /// `char *name`
    pub name: usize,
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    /// `uint8_t domain`
    pub domain: u8,
    /// `int identifier`
    pub identifier: i32,
}
impl Default for NodeForeachGeometryElementGenerationItem {
    fn default() -> Self {
        Self {
            name: NIL,
            socket_type: Default::default(),
            domain: 0,
            identifier: 0,
        }
    }
}

/// `NodeForeachGeometryElementInputItems`.
#[derive(Clone)]
pub struct NodeForeachGeometryElementInputItems {
    /// `NodeForeachGeometryElementInputItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int active_index`
    pub active_index: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
}
impl Default for NodeForeachGeometryElementInputItems {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            active_index: 0,
            next_identifier: 0,
        }
    }
}

/// `NodeForeachGeometryElementMainItems`.
#[derive(Clone)]
pub struct NodeForeachGeometryElementMainItems {
    /// `NodeForeachGeometryElementMainItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int active_index`
    pub active_index: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
}
impl Default for NodeForeachGeometryElementMainItems {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            active_index: 0,
            next_identifier: 0,
        }
    }
}

/// `NodeForeachGeometryElementGenerationItems`.
#[derive(Clone)]
pub struct NodeForeachGeometryElementGenerationItems {
    /// `NodeForeachGeometryElementGenerationItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int active_index`
    pub active_index: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
}
impl Default for NodeForeachGeometryElementGenerationItems {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            active_index: 0,
            next_identifier: 0,
        }
    }
}

/// `NodeGeometryForeachGeometryElementOutput`.
#[derive(Clone)]
pub struct NodeGeometryForeachGeometryElementOutput {
    pub input_items: NodeForeachGeometryElementInputItems,
    pub main_items: NodeForeachGeometryElementMainItems,
    pub generation_items: NodeForeachGeometryElementGenerationItems,
    /// `int inspection_index`
    pub inspection_index: i32,
    /// `uint8_t domain`
    pub domain: u8,
}
impl Default for NodeGeometryForeachGeometryElementOutput {
    fn default() -> Self {
        Self {
            input_items: Default::default(),
            main_items: Default::default(),
            generation_items: Default::default(),
            inspection_index: 0,
            domain: 0,
        }
    }
}

/// `NodeClosureInput`.
#[derive(Clone)]
pub struct NodeClosureInput {
    /// `int32_t output_node_id`
    pub output_node_id: i32,
}
impl Default for NodeClosureInput {
    fn default() -> Self {
        Self {
            output_node_id: 0,
        }
    }
}

/// `NodeClosureInputItem`.
#[derive(Clone)]
pub struct NodeClosureInputItem {
    /// `char *name`
    pub name: usize,
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    pub structure_type: NodeSocketInterfaceStructureType,
    /// `int identifier`
    pub identifier: i32,
}
impl Default for NodeClosureInputItem {
    fn default() -> Self {
        Self {
            name: NIL,
            socket_type: Default::default(),
            structure_type: NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_AUTO,
            identifier: 0,
        }
    }
}

/// `NodeClosureOutputItem`.
#[derive(Clone)]
pub struct NodeClosureOutputItem {
    /// `char *name`
    pub name: usize,
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    pub structure_type: NodeSocketInterfaceStructureType,
    /// `int identifier`
    pub identifier: i32,
}
impl Default for NodeClosureOutputItem {
    fn default() -> Self {
        Self {
            name: NIL,
            socket_type: Default::default(),
            structure_type: NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_AUTO,
            identifier: 0,
        }
    }
}

/// `NodeClosureInputItems`.
#[derive(Clone)]
pub struct NodeClosureInputItems {
    /// `NodeClosureInputItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int active_index`
    pub active_index: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
}
impl Default for NodeClosureInputItems {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            active_index: 0,
            next_identifier: 0,
        }
    }
}

/// `NodeClosureOutputItems`.
#[derive(Clone)]
pub struct NodeClosureOutputItems {
    /// `NodeClosureOutputItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int active_index`
    pub active_index: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
}
impl Default for NodeClosureOutputItems {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            active_index: 0,
            next_identifier: 0,
        }
    }
}

/// `NodeClosureOutput`.
#[derive(Clone)]
pub struct NodeClosureOutput {
    pub input_items: NodeClosureInputItems,
    pub output_items: NodeClosureOutputItems,
    /// `NodeClosureFlag flag`
    pub flag: Nodeclosureflag,
}
impl Default for NodeClosureOutput {
    fn default() -> Self {
        Self {
            input_items: Default::default(),
            output_items: Default::default(),
            flag: Default::default(),
        }
    }
}

/// `NodeEvaluateClosureInputItem`.
#[derive(Clone)]
pub struct NodeEvaluateClosureInputItem {
    /// `char *name`
    pub name: usize,
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    pub structure_type: NodeSocketInterfaceStructureType,
    /// `int identifier`
    pub identifier: i32,
}
impl Default for NodeEvaluateClosureInputItem {
    fn default() -> Self {
        Self {
            name: NIL,
            socket_type: Default::default(),
            structure_type: NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_AUTO,
            identifier: 0,
        }
    }
}

/// `NodeEvaluateClosureOutputItem`.
#[derive(Clone)]
pub struct NodeEvaluateClosureOutputItem {
    /// `char *name`
    pub name: usize,
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    pub structure_type: NodeSocketInterfaceStructureType,
    /// `int identifier`
    pub identifier: i32,
}
impl Default for NodeEvaluateClosureOutputItem {
    fn default() -> Self {
        Self {
            name: NIL,
            socket_type: Default::default(),
            structure_type: NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_AUTO,
            identifier: 0,
        }
    }
}

/// `NodeEvaluateClosureInputItems`.
#[derive(Clone)]
pub struct NodeEvaluateClosureInputItems {
    /// `NodeEvaluateClosureInputItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int active_index`
    pub active_index: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
}
impl Default for NodeEvaluateClosureInputItems {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            active_index: 0,
            next_identifier: 0,
        }
    }
}

/// `NodeEvaluateClosureOutputItems`.
#[derive(Clone)]
pub struct NodeEvaluateClosureOutputItems {
    /// `NodeEvaluateClosureOutputItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int active_index`
    pub active_index: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
}
impl Default for NodeEvaluateClosureOutputItems {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            active_index: 0,
            next_identifier: 0,
        }
    }
}

/// `NodeEvaluateClosure`.
#[derive(Clone)]
pub struct NodeEvaluateClosure {
    pub input_items: NodeEvaluateClosureInputItems,
    pub output_items: NodeEvaluateClosureOutputItems,
    /// `NodeEvaluateClosureFlag flag`
    pub flag: Nodeevaluateclosureflag,
}
impl Default for NodeEvaluateClosure {
    fn default() -> Self {
        Self {
            input_items: Default::default(),
            output_items: Default::default(),
            flag: Default::default(),
        }
    }
}

/// `IndexSwitchItem`.
#[derive(Clone)]
pub struct IndexSwitchItem {
    /// `int identifier`
    pub identifier: i32,
}
impl Default for IndexSwitchItem {
    fn default() -> Self {
        Self {
            identifier: 0,
        }
    }
}

/// `NodeIndexSwitch`.
#[derive(Clone)]
pub struct NodeIndexSwitch {
    /// `IndexSwitchItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
    /// `eNodeSocketDatatype data_type`
    pub data_type: Nodesocketdatatype,
}
impl Default for NodeIndexSwitch {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            next_identifier: 0,
            data_type: Default::default(),
        }
    }
}

/// `CombineListItem`.
#[derive(Clone)]
pub struct CombineListItem {
    /// `int identifier`
    pub identifier: i32,
}
impl Default for CombineListItem {
    fn default() -> Self {
        Self {
            identifier: 0,
        }
    }
}

/// `NodeCombineList`.
#[derive(Clone)]
pub struct NodeCombineList {
    /// `CombineListItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
    /// `eNodeSocketDatatype data_type`
    pub data_type: Nodesocketdatatype,
}
impl Default for NodeCombineList {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            next_identifier: 0,
            data_type: Default::default(),
        }
    }
}

/// `GeometryNodeFieldToGridItem`.
#[derive(Clone)]
pub struct GeometryNodeFieldToGridItem {
    /// `eNodeSocketDatatype data_type`
    pub data_type: Nodesocketdatatype,
    /// `int identifier`
    pub identifier: i32,
    /// `char *name`
    pub name: usize,
}
impl Default for GeometryNodeFieldToGridItem {
    fn default() -> Self {
        Self {
            data_type: Default::default(),
            identifier: 0,
            name: NIL,
        }
    }
}

/// `GeometryNodeFieldToGrid`.
#[derive(Clone)]
pub struct GeometryNodeFieldToGrid {
    /// `eNodeSocketDatatype data_type`
    pub data_type: Nodesocketdatatype,
    /// `int next_identifier`
    pub next_identifier: i32,
    /// `GeometryNodeFieldToGridItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int active_index`
    pub active_index: i32,
}
impl Default for GeometryNodeFieldToGrid {
    fn default() -> Self {
        Self {
            data_type: Default::default(),
            next_identifier: 0,
            items: NIL,
            items_num: 0,
            active_index: 0,
        }
    }
}

/// `GeometryNodeFieldToListItem`.
#[derive(Clone)]
pub struct GeometryNodeFieldToListItem {
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    /// `int identifier`
    pub identifier: i32,
    /// `char *name`
    pub name: usize,
}
impl Default for GeometryNodeFieldToListItem {
    fn default() -> Self {
        Self {
            socket_type: SOCK_FLOAT,
            identifier: 0,
            name: NIL,
        }
    }
}

/// `GeometryNodeFieldToList`.
#[derive(Clone)]
pub struct GeometryNodeFieldToList {
    /// `int next_identifier`
    pub next_identifier: i32,
    /// `GeometryNodeFieldToListItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int active_index`
    pub active_index: i32,
}
impl Default for GeometryNodeFieldToList {
    fn default() -> Self {
        Self {
            next_identifier: 0,
            items: NIL,
            items_num: 0,
            active_index: 0,
        }
    }
}

/// `GeometryNodeClosureToListItem`.
#[derive(Clone)]
pub struct GeometryNodeClosureToListItem {
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    pub structure_type: NodeSocketInterfaceStructureType,
    /// `int identifier`
    pub identifier: i32,
    /// `char *name`
    pub name: usize,
}
impl Default for GeometryNodeClosureToListItem {
    fn default() -> Self {
        Self {
            socket_type: SOCK_FLOAT,
            structure_type: NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_AUTO,
            identifier: 0,
            name: NIL,
        }
    }
}

/// `GeometryNodeClosureToList`.
#[derive(Clone)]
pub struct GeometryNodeClosureToList {
    /// `int next_identifier`
    pub next_identifier: i32,
    /// `GeometryNodeClosureToListItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int active_index`
    pub active_index: i32,
}
impl Default for GeometryNodeClosureToList {
    fn default() -> Self {
        Self {
            next_identifier: 0,
            items: NIL,
            items_num: 0,
            active_index: 0,
        }
    }
}

/// `NodeGeometryDistributePointsInVolume`.
#[derive(Clone)]
pub struct NodeGeometryDistributePointsInVolume {
    /// `uint8_t mode`
    pub mode: u8,
}
impl Default for NodeGeometryDistributePointsInVolume {
    fn default() -> Self {
        Self {
            mode: 0,
        }
    }
}

/// `NodeGeometryRasterizePointsItem`.
#[derive(Clone)]
pub struct NodeGeometryRasterizePointsItem {
    /// `char *name`
    pub name: usize,
    /// `short type`
    pub type_: i16,
    /// `int identifier`
    pub identifier: i32,
}
impl Default for NodeGeometryRasterizePointsItem {
    fn default() -> Self {
        Self {
            name: NIL,
            type_: Default::default(),
            identifier: Default::default(),
        }
    }
}

/// `NodeGeometryRasterizePoints`.
#[derive(Clone)]
pub struct NodeGeometryRasterizePoints {
    /// `NodeGeometryRasterizePointsItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int active_index`
    pub active_index: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
}
impl Default for NodeGeometryRasterizePoints {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: Default::default(),
            active_index: Default::default(),
            next_identifier: Default::default(),
        }
    }
}

/// `NodeFunctionCompare`.
#[derive(Clone)]
pub struct NodeFunctionCompare {
    /// `NodeCompareOperation operation`
    pub operation: Nodecompareoperation,
    /// `NodeCompareMode mode`
    pub mode: Nodecomparemode,
    /// `eNodeSocketDatatype data_type`
    pub data_type: Nodesocketdatatype,
}
impl Default for NodeFunctionCompare {
    fn default() -> Self {
        Self {
            operation: NODE_COMPARE_LESS_THAN,
            mode: NODE_COMPARE_MODE_ELEMENT,
            data_type: Default::default(),
        }
    }
}

/// `NodeCombSepColor`.
#[derive(Clone)]
pub struct NodeCombSepColor {
    /// `NodeCombSepColorMode mode`
    pub mode: Nodecombsepcolormode,
}
impl Default for NodeCombSepColor {
    fn default() -> Self {
        Self {
            mode: NODE_COMBSEP_COLOR_RGB,
        }
    }
}

/// `NodeShaderMix`.
#[derive(Clone)]
pub struct NodeShaderMix {
    /// `eNodeSocketDatatype data_type`
    pub data_type: Nodesocketdatatype,
    /// `NodeShaderMixMode factor_mode`
    pub factor_mode: Nodeshadermixmode,
    /// `int8_t clamp_factor`
    pub clamp_factor: i8,
    /// `int8_t clamp_result`
    pub clamp_result: i8,
    /// `int8_t blend_type`
    pub blend_type: i8,
}
impl Default for NodeShaderMix {
    fn default() -> Self {
        Self {
            data_type: Default::default(),
            factor_mode: NODE_MIX_MODE_UNIFORM,
            clamp_factor: 0,
            clamp_result: 0,
            blend_type: 0,
        }
    }
}

/// `NodeGeometryLinearGizmo`.
#[derive(Clone)]
pub struct NodeGeometryLinearGizmo {
    /// `GeometryNodeGizmoColor color_id`
    pub color_id: Geometrynodegizmocolor,
    /// `GeometryNodeLinearGizmoDrawStyle draw_style`
    pub draw_style: Geometrynodelineargizmodrawstyle,
}
impl Default for NodeGeometryLinearGizmo {
    fn default() -> Self {
        Self {
            color_id: GEO_NODE_GIZMO_COLOR_PRIMARY,
            draw_style: GEO_NODE_LINEAR_GIZMO_DRAW_STYLE_ARROW,
        }
    }
}

/// `NodeGeometryDialGizmo`.
#[derive(Clone)]
pub struct NodeGeometryDialGizmo {
    /// `GeometryNodeGizmoColor color_id`
    pub color_id: Geometrynodegizmocolor,
}
impl Default for NodeGeometryDialGizmo {
    fn default() -> Self {
        Self {
            color_id: GEO_NODE_GIZMO_COLOR_PRIMARY,
        }
    }
}

/// `NodeGeometryTransformGizmo`.
#[derive(Clone)]
pub struct NodeGeometryTransformGizmo {
    /// `NodeGeometryTransformGizmoFlag flag`
    pub flag: Nodegeometrytransformgizmoflag,
}
impl Default for NodeGeometryTransformGizmo {
    fn default() -> Self {
        Self {
            flag: Default::default(),
        }
    }
}

/// `NodeGeometryBakeItem`.
#[derive(Clone)]
pub struct NodeGeometryBakeItem {
    /// `char *name`
    pub name: usize,
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    /// `int16_t attribute_domain`
    pub attribute_domain: i16,
    /// `int identifier`
    pub identifier: i32,
    /// `int32_t flag`
    pub flag: i32,
}
impl Default for NodeGeometryBakeItem {
    fn default() -> Self {
        Self {
            name: NIL,
            socket_type: Default::default(),
            attribute_domain: 0,
            identifier: 0,
            flag: 0,
        }
    }
}

/// `NodeGeometryBake`.
#[derive(Clone)]
pub struct NodeGeometryBake {
    /// `NodeGeometryBakeItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
    /// `int active_index`
    pub active_index: i32,
}
impl Default for NodeGeometryBake {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            next_identifier: 0,
            active_index: 0,
        }
    }
}

/// `NodeCombineBundleItem`.
#[derive(Clone)]
pub struct NodeCombineBundleItem {
    /// `char *name`
    pub name: usize,
    /// `int identifier`
    pub identifier: i32,
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    pub structure_type: NodeSocketInterfaceStructureType,
}
impl Default for NodeCombineBundleItem {
    fn default() -> Self {
        Self {
            name: NIL,
            identifier: 0,
            socket_type: Default::default(),
            structure_type: NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_AUTO,
        }
    }
}

/// `NodeCombineBundle`.
#[derive(Clone)]
pub struct NodeCombineBundle {
    /// `NodeCombineBundleItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
    /// `int active_index`
    pub active_index: i32,
    /// `NodeCombineBundleFlag flag`
    pub flag: Nodecombinebundleflag,
}
impl Default for NodeCombineBundle {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            next_identifier: 0,
            active_index: 0,
            flag: Default::default(),
        }
    }
}

/// `NodeSeparateBundleItem`.
#[derive(Clone)]
pub struct NodeSeparateBundleItem {
    /// `char *name`
    pub name: usize,
    /// `int identifier`
    pub identifier: i32,
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    pub structure_type: NodeSocketInterfaceStructureType,
}
impl Default for NodeSeparateBundleItem {
    fn default() -> Self {
        Self {
            name: NIL,
            identifier: 0,
            socket_type: Default::default(),
            structure_type: NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_AUTO,
        }
    }
}

/// `NodeSeparateBundle`.
#[derive(Clone)]
pub struct NodeSeparateBundle {
    /// `NodeSeparateBundleItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
    /// `int active_index`
    pub active_index: i32,
    /// `NodeSeparateBundleFlag flag`
    pub flag: Nodeseparatebundleflag,
}
impl Default for NodeSeparateBundle {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            next_identifier: 0,
            active_index: 0,
            flag: Default::default(),
        }
    }
}

/// `NodeFunctionFormatStringItem`.
#[derive(Clone)]
pub struct NodeFunctionFormatStringItem {
    /// `char *name`
    pub name: usize,
    /// `int identifier`
    pub identifier: i32,
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
}
impl Default for NodeFunctionFormatStringItem {
    fn default() -> Self {
        Self {
            name: NIL,
            identifier: 0,
            socket_type: Default::default(),
        }
    }
}

/// `NodeFunctionFormatString`.
#[derive(Clone)]
pub struct NodeFunctionFormatString {
    /// `NodeFunctionFormatStringItem *items`
    pub items: usize,
    /// `int items_num`
    pub items_num: i32,
    /// `int next_identifier`
    pub next_identifier: i32,
    /// `int active_index`
    pub active_index: i32,
}
impl Default for NodeFunctionFormatString {
    fn default() -> Self {
        Self {
            items: NIL,
            items_num: 0,
            next_identifier: 0,
            active_index: 0,
        }
    }
}

/// `NodeGeometryListGetItem`.
#[derive(Clone)]
pub struct NodeGeometryListGetItem {
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    pub structure_type: NodeSocketInterfaceStructureType,
}
impl Default for NodeGeometryListGetItem {
    fn default() -> Self {
        Self {
            socket_type: SOCK_FLOAT,
            structure_type: NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_AUTO,
        }
    }
}

/// `NodeGetBundleItem`.
#[derive(Clone)]
pub struct NodeGetBundleItem {
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    pub structure_type: NodeSocketInterfaceStructureType,
}
impl Default for NodeGetBundleItem {
    fn default() -> Self {
        Self {
            socket_type: Default::default(),
            structure_type: NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_AUTO,
        }
    }
}

/// `NodeStoreBundleItem`.
#[derive(Clone)]
pub struct NodeStoreBundleItem {
    /// `eNodeSocketDatatype socket_type`
    pub socket_type: Nodesocketdatatype,
    pub structure_type: NodeSocketInterfaceStructureType,
}
impl Default for NodeStoreBundleItem {
    fn default() -> Self {
        Self {
            socket_type: Default::default(),
            structure_type: NODE_SOCKET_INTERFACE_STRUCTURE_TYPE_AUTO,
        }
    }
}

/// `NodeComment`.
#[derive(Clone)]
pub struct NodeComment {
    /// `char *text`
    pub text: usize,
    pub textbox_state_node: TextboxState,
    pub textbox_state_panel: TextboxState,
}
impl Default for NodeComment {
    fn default() -> Self {
        Self {
            text: NIL,
            textbox_state_node: Default::default(),
            textbox_state_panel: Default::default(),
        }
    }
}
