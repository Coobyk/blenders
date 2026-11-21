//! Port of `DNA_view3d_types.h` + `DNA_view3d_enums.h` — `View3D`,
//! `RegionView3D`, `View3DCursor`, `View3DShading`, `View3DOverlay` + enums.
#![allow(non_camel_case_types)]
use crate::action_types::ROT_MODE_XYZ;
use crate::id::NIL;
use crate::layer_types::SCE_PASS_COMBINED;
use crate::object_types::{DrawType, OB_PLAINAXES, OB_SOLID};
use crate::vec_types::Rctf;
use crate::viewer_path_types::ViewerPath;
use blenders_bli::listbase::ListBase;

/// `eHandleDisplay`.
pub type HandleDisplay = i32;
pub const CURVE_HANDLE_SELECTED: HandleDisplay = 0;
pub const CURVE_HANDLE_ALL: HandleDisplay = 1;
pub const CURVE_HANDLE_NONE: HandleDisplay = 2;

/// `eView3D_StereoFlag`.
pub const V3D_S3D_DISPCAMERAS: i16 = 1 << 0;
pub const V3D_S3D_DISPPLANE: i16 = 1 << 1;
pub const V3D_S3D_DISPVOLUME: i16 = 1 << 2;

/// `eView3D_Flag`.
pub type View3DFlag = i16;
pub const V3D_LOCAL_COLLECTIONS: View3DFlag = 1 << 0;
pub const V3D_FLAG_UNUSED_1: View3DFlag = 1 << 1;
pub const V3D_HIDE_HELPLINES: View3DFlag = 1 << 2;
pub const V3D_FLAG_UNUSED_2: View3DFlag = 1 << 3;
pub const V3D_XR_SESSION_MIRROR: View3DFlag = 1 << 4;
pub const V3D_XR_SESSION_SURFACE: View3DFlag = 1 << 5;
pub const V3D_FLAG_UNUSED_10: View3DFlag = 1 << 10;
pub const V3D_SELECT_OUTLINE: View3DFlag = 1 << 11;
pub const V3D_FLAG_UNUSED_12: View3DFlag = 1 << 12;
pub const V3D_GLOBAL_STATS: View3DFlag = 1 << 13;
pub const V3D_DRAW_CENTERS: View3DFlag = 1 << 15;

/// `eView3D_Runtime_Flag`.
pub const V3D_RUNTIME_XR_SESSION_ROOT: i32 = 1 << 0;
pub const V3D_RUNTIME_DEPTHBUF_OVERRIDDEN: i32 = 1 << 1;
pub const V3D_RUNTIME_LOCAL_MAYBE_EMPTY: i32 = 1 << 2;
pub const V3D_RUNTIME_OFS_LAST_CENTER_IS_VALID: i32 = 1 << 3;

/// `eRegionView3D_Persp`.
pub type RegionView3DPersp = i8;
pub const RV3D_ORTHO: RegionView3DPersp = 0;
pub const RV3D_PERSP: RegionView3DPersp = 1;
pub const RV3D_CAMOB: RegionView3DPersp = 2;

/// `eRegionView3D_Flag`.
pub const RV3D_CLIPPING: i16 = 1 << 2;
pub const RV3D_ZOFFSET_DISABLED: i16 = 1 << 6;
pub const RV3D_WAS_CAMOB: i16 = 1 << 7;
pub const RV3D_FLIP_X: i16 = 1 << 8;

/// `eRegionView3D_ViewLock`.
pub const RV3D_LOCK_ROTATION: i8 = 1 << 0;
pub const RV3D_BOXVIEW: i8 = 1 << 1;
pub const RV3D_BOXCLIP: i8 = 1 << 2;
pub const RV3D_LOCK_LOCATION: i8 = 1 << 3;
pub const RV3D_LOCK_ZOOM_AND_DOLLY: i8 = 1 << 4;
pub const RV3D_LOCK_ANY_TRANSFORM: i8 =
    RV3D_LOCK_LOCATION | RV3D_LOCK_ROTATION | RV3D_LOCK_ZOOM_AND_DOLLY;

/// `eRegionView3D_ViewLockQuad`.
pub const RV3D_VIEWLOCK_INIT: i8 = 1 << 7;

/// `eRegionView3D_View`.
pub type RegionView3DView = i8;
pub const RV3D_VIEW_USER: RegionView3DView = 0;
pub const RV3D_VIEW_FRONT: RegionView3DView = 1;
pub const RV3D_VIEW_BACK: RegionView3DView = 2;
pub const RV3D_VIEW_LEFT: RegionView3DView = 3;
pub const RV3D_VIEW_RIGHT: RegionView3DView = 4;
pub const RV3D_VIEW_TOP: RegionView3DView = 5;
pub const RV3D_VIEW_BOTTOM: RegionView3DView = 6;
pub const RV3D_VIEW_CAMERA: RegionView3DView = 8;
/// `RV3D_VIEW_IS_AXIS`.
pub fn rv3d_view_is_axis(view: i8) -> bool {
    (RV3D_VIEW_FRONT..=RV3D_VIEW_BOTTOM).contains(&view)
}

/// `eRegionView3D_ViewAxisRoll`.
pub type RegionView3DViewAxisRoll = i8;
pub const RV3D_VIEW_AXIS_ROLL_0: RegionView3DViewAxisRoll = 0;
pub const RV3D_VIEW_AXIS_ROLL_90: RegionView3DViewAxisRoll = 1;
pub const RV3D_VIEW_AXIS_ROLL_180: RegionView3DViewAxisRoll = 2;
pub const RV3D_VIEW_AXIS_ROLL_270: RegionView3DViewAxisRoll = 3;

/// `eRegionView3D_NDOFFlag`.
pub const RV3D_NDOF_OFS_IS_VALID: i8 = 1 << 0;

/// `eView3D_Flag2`.
pub type View3DFlag2 = i32;
pub const V3D_HIDE_OVERLAYS: View3DFlag2 = 1 << 2;
pub const V3D_SHOW_VIEWER: View3DFlag2 = 1 << 3;
pub const V3D_SHOW_ANNOTATION: View3DFlag2 = 1 << 4;
pub const V3D_LOCK_CAMERA: View3DFlag2 = 1 << 5;
pub const V3D_FLAG2_UNUSED_6: View3DFlag2 = 1 << 6;
pub const V3D_SHOW_RECONSTRUCTION: View3DFlag2 = 1 << 7;
pub const V3D_SHOW_CAMERAPATH: View3DFlag2 = 1 << 8;
pub const V3D_SHOW_BUNDLENAME: View3DFlag2 = 1 << 9;
pub const V3D_FLAG2_UNUSED_10: View3DFlag2 = 1 << 10;
pub const V3D_RENDER_BORDER: View3DFlag2 = 1 << 11;
pub const V3D_FLAG2_UNUSED_12: View3DFlag2 = 1 << 12;
pub const V3D_FLAG2_UNUSED_13: View3DFlag2 = 1 << 13;
pub const V3D_FLAG2_UNUSED_14: View3DFlag2 = 1 << 14;
pub const V3D_FLAG2_UNUSED_15: View3DFlag2 = 1 << 15;
pub const V3D_XR_SHOW_CONTROLLERS: View3DFlag2 = 1 << 16;
pub const V3D_XR_SHOW_CUSTOM_OVERLAYS: View3DFlag2 = 1 << 17;
pub const V3D_SHOW_CAMERA_GUIDES: View3DFlag2 = 1 << 18;
pub const V3D_SHOW_CAMERA_PASSEPARTOUT: View3DFlag2 = 1 << 19;
pub const V3D_XR_SHOW_PASSTHROUGH: View3DFlag2 = 1 << 20;

/// `eView3D_GPFlag`.
pub type View3DGpFlag = i16;
pub const V3D_GP_FADE_OBJECTS: View3DGpFlag = 1 << 0;
pub const V3D_GP_SHOW_GRID: View3DGpFlag = 1 << 1;
pub const V3D_GP_SHOW_EDIT_LINES: View3DGpFlag = 1 << 2;
pub const V3D_GP_SHOW_MULTIEDIT_LINES: View3DGpFlag = 1 << 3;
pub const V3D_GP_SHOW_ONION_SKIN: View3DGpFlag = 1 << 4;
pub const V3D_GP_FADE_NOACTIVE_LAYERS: View3DGpFlag = 1 << 5;
pub const V3D_GP_FADE_NOACTIVE_GPENCIL: View3DGpFlag = 1 << 6;
pub const V3D_GP_SHOW_STROKE_DIRECTION: View3DGpFlag = 1 << 7;
pub const V3D_GP_SHOW_MATERIAL_NAME: View3DGpFlag = 1 << 8;
pub const V3D_GP_SHOW_GRID_XRAY: View3DGpFlag = 1 << 9;
pub const V3D_GP_FORCE_STROKE_ORDER_3D: View3DGpFlag = 1 << 10;
pub const V3D_GP_ONION_SKIN_ACTIVE_OBJECT: View3DGpFlag = 1 << 11;

/// `eView3DShading_Flag`.
pub type View3DShadingFlag = i16;
pub const V3D_SHADING_OBJECT_OUTLINE: View3DShadingFlag = 1 << 0;
pub const V3D_SHADING_XRAY: View3DShadingFlag = 1 << 1;
pub const V3D_SHADING_SHADOW: View3DShadingFlag = 1 << 2;
pub const V3D_SHADING_SCENE_LIGHTS: View3DShadingFlag = 1 << 3;
pub const V3D_SHADING_SPECULAR_HIGHLIGHT: View3DShadingFlag = 1 << 4;
pub const V3D_SHADING_CAVITY: View3DShadingFlag = 1 << 5;
pub const V3D_SHADING_MATCAP_FLIP_X: View3DShadingFlag = 1 << 6;
pub const V3D_SHADING_SCENE_WORLD: View3DShadingFlag = 1 << 7;
pub const V3D_SHADING_XRAY_WIREFRAME: View3DShadingFlag = 1 << 8;
pub const V3D_SHADING_WORLD_ORIENTATION: View3DShadingFlag = 1 << 9;
pub const V3D_SHADING_BACKFACE_CULLING: View3DShadingFlag = 1 << 10;
pub const V3D_SHADING_DEPTH_OF_FIELD: View3DShadingFlag = 1 << 11;
pub const V3D_SHADING_SCENE_LIGHTS_RENDER: View3DShadingFlag = 1 << 12;
pub const V3D_SHADING_SCENE_WORLD_RENDER: View3DShadingFlag = 1 << 13;
pub const V3D_SHADING_STUDIOLIGHT_VIEW_ROTATION: View3DShadingFlag = 1 << 14;

/// `eView3D_DebugFlag`.
pub const V3D_DEBUG_FREEZE_CULLING: i16 = 1 << 0;

/// `eView3DShading_CavityType`.
pub type View3DShadingCavityType = i8;
pub const V3D_SHADING_CAVITY_SSAO: View3DShadingCavityType = 0;
pub const V3D_SHADING_CAVITY_CURVATURE: View3DShadingCavityType = 1;
pub const V3D_SHADING_CAVITY_BOTH: View3DShadingCavityType = 2;

/// `View3DShadingUseCompositor`.
pub type View3DShadingUseCompositor = i8;
pub const V3D_SHADING_USE_COMPOSITOR_DISABLED: View3DShadingUseCompositor = 0;
pub const V3D_SHADING_USE_COMPOSITOR_CAMERA: View3DShadingUseCompositor = 1;
pub const V3D_SHADING_USE_COMPOSITOR_ALWAYS: View3DShadingUseCompositor = 2;

/// `eView3DOverlay_Flag`.
pub type View3DOverlayFlag = i32;
pub const V3D_OVERLAY_FACE_ORIENTATION: View3DOverlayFlag = 1 << 0;
pub const V3D_OVERLAY_HIDE_CURSOR: View3DOverlayFlag = 1 << 1;
pub const V3D_OVERLAY_BONE_SELECT: View3DOverlayFlag = 1 << 2;
pub const V3D_OVERLAY_LOOK_DEV: View3DOverlayFlag = 1 << 3;
pub const V3D_OVERLAY_WIREFRAMES: View3DOverlayFlag = 1 << 4;
pub const V3D_OVERLAY_HIDE_TEXT: View3DOverlayFlag = 1 << 5;
pub const V3D_OVERLAY_HIDE_MOTION_PATHS: View3DOverlayFlag = 1 << 6;
pub const V3D_OVERLAY_ONION_SKINS: View3DOverlayFlag = 1 << 7;
pub const V3D_OVERLAY_HIDE_BONES: View3DOverlayFlag = 1 << 8;
pub const V3D_OVERLAY_HIDE_OBJECT_XTRAS: View3DOverlayFlag = 1 << 9;
pub const V3D_OVERLAY_HIDE_OBJECT_ORIGINS: View3DOverlayFlag = 1 << 10;
pub const V3D_OVERLAY_STATS: View3DOverlayFlag = 1 << 11;
pub const V3D_OVERLAY_FADE_INACTIVE: View3DOverlayFlag = 1 << 12;
pub const V3D_OVERLAY_VIEWER_ATTRIBUTE: View3DOverlayFlag = 1 << 13;
pub const V3D_OVERLAY_SCULPT_SHOW_MASK: View3DOverlayFlag = 1 << 14;
pub const V3D_OVERLAY_SCULPT_SHOW_FACE_SETS: View3DOverlayFlag = 1 << 15;
pub const V3D_OVERLAY_SCULPT_CURVES_CAGE: View3DOverlayFlag = 1 << 16;
pub const V3D_OVERLAY_SHOW_LIGHT_COLORS: View3DOverlayFlag = 1 << 17;
pub const V3D_OVERLAY_VIEWER_ATTRIBUTE_TEXT: View3DOverlayFlag = 1 << 18;
pub const V3D_OVERLAY_PERFORMANCE: View3DOverlayFlag = 1 << 19;

/// `eView3DOverlay_EditFlag`.
pub type View3DOverlayEditFlag = i32;
pub const V3D_OVERLAY_EDIT_VERT_NORMALS: View3DOverlayEditFlag = 1 << 0;
pub const V3D_OVERLAY_EDIT_LOOP_NORMALS: View3DOverlayEditFlag = 1 << 1;
pub const V3D_OVERLAY_EDIT_FACE_NORMALS: View3DOverlayEditFlag = 1 << 2;
pub const V3D_OVERLAY_EDIT_RETOPOLOGY: View3DOverlayEditFlag = 1 << 3;
pub const V3D_OVERLAY_EDIT_WEIGHT: View3DOverlayEditFlag = 1 << 4;
pub const V3D_OVERLAY_EDIT_EDGES_DEPRECATED: View3DOverlayEditFlag = 1 << 5;
pub const V3D_OVERLAY_EDIT_FACES: View3DOverlayEditFlag = 1 << 6;
pub const V3D_OVERLAY_EDIT_FACE_DOT: View3DOverlayEditFlag = 1 << 7;
pub const V3D_OVERLAY_EDIT_SEAMS: View3DOverlayEditFlag = 1 << 8;
pub const V3D_OVERLAY_EDIT_SHARP: View3DOverlayEditFlag = 1 << 9;
pub const V3D_OVERLAY_EDIT_CREASES: View3DOverlayEditFlag = 1 << 10;
pub const V3D_OVERLAY_EDIT_BWEIGHTS: View3DOverlayEditFlag = 1 << 11;
pub const V3D_OVERLAY_EDIT_FREESTYLE_EDGE: View3DOverlayEditFlag = 1 << 12;
pub const V3D_OVERLAY_EDIT_FREESTYLE_FACE: View3DOverlayEditFlag = 1 << 13;
pub const V3D_OVERLAY_EDIT_STATVIS: View3DOverlayEditFlag = 1 << 14;
pub const V3D_OVERLAY_EDIT_EDGE_LEN: View3DOverlayEditFlag = 1 << 15;
pub const V3D_OVERLAY_EDIT_EDGE_ANG: View3DOverlayEditFlag = 1 << 16;
pub const V3D_OVERLAY_EDIT_FACE_ANG: View3DOverlayEditFlag = 1 << 17;
pub const V3D_OVERLAY_EDIT_FACE_AREA: View3DOverlayEditFlag = 1 << 18;
pub const V3D_OVERLAY_EDIT_INDICES: View3DOverlayEditFlag = 1 << 19;
pub const V3D_OVERLAY_EDIT_CU_NORMALS: View3DOverlayEditFlag = 1 << 21;
pub const V3D_OVERLAY_EDIT_CONSTANT_SCREEN_SIZE_NORMALS: View3DOverlayEditFlag = 1 << 22;

/// `eView3DOverlay_PaintFlag`.
pub const V3D_OVERLAY_PAINT_WIRE: i32 = 1 << 0;

/// `eView3DOverlay_WPaintFlag`.
pub const V3D_OVERLAY_WPAINT_CONTOURS: i32 = 1 << 0;

/// `eView3D_Around`.
pub type View3DAround = i32;
pub const V3D_AROUND_CENTER_BOUNDS: View3DAround = 0;
pub const V3D_AROUND_CURSOR: View3DAround = 1;
pub const V3D_AROUND_LOCAL_ORIGINS: View3DAround = 2;
pub const V3D_AROUND_CENTER_MEDIAN: View3DAround = 3;
pub const V3D_AROUND_ACTIVE: View3DAround = 4;

/// `eView3D_GridFlag`.
pub const V3D_SHOW_FLOOR: i8 = 1 << 0;
pub const V3D_SHOW_X: i8 = 1 << 1;
pub const V3D_SHOW_Y: i8 = 1 << 2;
pub const V3D_SHOW_Z: i8 = 1 << 3;
pub const V3D_SHOW_ORTHO_GRID: i8 = 1 << 4;

/// `eView3D_Orientation`.
pub type View3DOrientation = i32;
pub const V3D_ORIENT_GLOBAL: View3DOrientation = 0;
pub const V3D_ORIENT_LOCAL: View3DOrientation = 1;
pub const V3D_ORIENT_NORMAL: View3DOrientation = 2;
pub const V3D_ORIENT_VIEW: View3DOrientation = 3;
pub const V3D_ORIENT_GIMBAL: View3DOrientation = 4;
pub const V3D_ORIENT_CURSOR: View3DOrientation = 5;
pub const V3D_ORIENT_PARENT: View3DOrientation = 6;
pub const V3D_ORIENT_CUSTOM: View3DOrientation = 1024;
pub const V3D_ORIENT_CUSTOM_MATRIX: View3DOrientation = V3D_ORIENT_CUSTOM - 1;

/// `eView3D_GizmoFlag`.
pub const V3D_GIZMO_HIDE: i8 = 1 << 0;
pub const V3D_GIZMO_HIDE_NAVIGATE: i8 = 1 << 1;
pub const V3D_GIZMO_HIDE_CONTEXT: i8 = 1 << 2;
pub const V3D_GIZMO_HIDE_TOOL: i8 = 1 << 3;
pub const V3D_GIZMO_HIDE_MODIFIER: i8 = 1 << 4;

/// `eView3D_GizmoShowObject`.
pub const V3D_GIZMO_SHOW_OBJECT_TRANSLATE: i8 = 1 << 0;
pub const V3D_GIZMO_SHOW_OBJECT_ROTATE: i8 = 1 << 1;
pub const V3D_GIZMO_SHOW_OBJECT_SCALE: i8 = 1 << 2;

/// `eView3D_GizmoShowArmature`.
pub const V3D_GIZMO_SHOW_ARMATURE_BBONE: i8 = 1 << 0;
pub const V3D_GIZMO_SHOW_ARMATURE_ROLL: i8 = 1 << 1;

/// `eView3D_GizmoShowEmpty`.
pub const V3D_GIZMO_SHOW_EMPTY_IMAGE: i8 = 1 << 0;
pub const V3D_GIZMO_SHOW_EMPTY_FORCE_FIELD: i8 = 1 << 1;

/// `eView3D_GizmoShowLight`.
pub const V3D_GIZMO_SHOW_LIGHT_SIZE: i8 = 1 << 0;
pub const V3D_GIZMO_SHOW_LIGHT_LOOK_AT: i8 = 1 << 1;

/// `eView3D_GizmoShowCamera`.
pub const V3D_GIZMO_SHOW_CAMERA_LENS: i8 = 1 << 0;
pub const V3D_GIZMO_SHOW_CAMERA_DOF_DIST: i8 = 1 << 2;

/// `eV3DPlaceDepth`.
pub const V3D_PLACE_DEPTH_SURFACE: i8 = 0;
pub const V3D_PLACE_DEPTH_CURSOR_PLANE: i8 = 1;
pub const V3D_PLACE_DEPTH_CURSOR_VIEW: i8 = 2;

/// `eV3DPlaceOrient`.
pub const V3D_PLACE_ORIENT_SURFACE: i8 = 0;
pub const V3D_PLACE_ORIENT_DEFAULT: i8 = 1;

/// `eV3DOffscreenDrawFlag` (DNA_view3d_enums.h).
pub const V3D_OFSDRAW_NONE: i32 = 0;
pub const V3D_OFSDRAW_SHOW_ANNOTATION: i32 = 1 << 0;
pub const V3D_OFSDRAW_OVERRIDE_SCENE_SETTINGS: i32 = 1 << 1;
pub const V3D_OFSDRAW_SHOW_GRIDFLOOR: i32 = 1 << 2;
pub const V3D_OFSDRAW_SHOW_SELECTION: i32 = 1 << 3;
pub const V3D_OFSDRAW_XR_SHOW_CONTROLLERS: i32 = 1 << 4;
pub const V3D_OFSDRAW_XR_SHOW_CUSTOM_OVERLAYS: i32 = 1 << 5;
pub const V3D_OFSDRAW_SHOW_OBJECT_EXTRAS: i32 = 1 << 6;
pub const V3D_OFSDRAW_XR_SHOW_PASSTHROUGH: i32 = 1 << 7;
pub const V3D_OFSDRAW_NO_WORLD_BACKGROUND_OVERRIDE: i32 = 1 << 8;

/// `eV3DShadingLightingMode` (DNA_view3d_enums.h).
pub type V3DShadingLightingMode = i8;
pub const V3D_LIGHTING_FLAT: V3DShadingLightingMode = 0;
pub const V3D_LIGHTING_STUDIO: V3DShadingLightingMode = 1;
pub const V3D_LIGHTING_MATCAP: V3DShadingLightingMode = 2;

/// `eV3DShadingColorType` (DNA_view3d_enums.h).
pub type V3DShadingColorType = i8;
pub const V3D_SHADING_MATERIAL_COLOR: V3DShadingColorType = 0;
pub const V3D_SHADING_RANDOM_COLOR: V3DShadingColorType = 1;
pub const V3D_SHADING_SINGLE_COLOR: V3DShadingColorType = 2;
pub const V3D_SHADING_TEXTURE_COLOR: V3DShadingColorType = 3;
pub const V3D_SHADING_OBJECT_COLOR: V3DShadingColorType = 4;
pub const V3D_SHADING_VERTEX_COLOR: V3DShadingColorType = 5;

/// `eV3DShadingBackgroundType` (DNA_view3d_enums.h).
pub type V3DShadingBackgroundType = i8;
pub const V3D_SHADING_BACKGROUND_THEME: V3DShadingBackgroundType = 0;
pub const V3D_SHADING_BACKGROUND_WORLD: V3DShadingBackgroundType = 1;
pub const V3D_SHADING_BACKGROUND_VIEWPORT: V3DShadingBackgroundType = 2;

/// `STEREO_3D_ID` (DNA_scene_enums.h) placeholder.
pub const STEREO_3D_ID: i8 = 2;
/// `SPACE_VIEW3D` (DNA_space_enums.h) placeholder.
pub const SPACE_VIEW3D: i8 = 1;

/// `RegionView3D`.
#[derive(Clone)]
pub struct RegionView3D {
    pub winmat: [[f32; 4]; 4],
    pub viewmat: [[f32; 4]; 4],
    pub viewinv: [[f32; 4]; 4],
    pub persmat: [[f32; 4]; 4],
    pub persinv: [[f32; 4]; 4],
    pub clip: [[f32; 4]; 6],
    /// `BoundBox *clipbb`.
    pub clipbb: usize,
    /// `RegionView3D *localvd`.
    pub localvd: usize,
    /// DNA_DEPRECATED `gridview`.
    pub gridview: f32,
    pub viewquat: [f32; 4],
    pub dist: f32,
    pub camdx: f32,
    pub camdy: f32,
    pub pixsize: f32,
    pub ofs: [f32; 3],
    pub camzoom: f32,
    pub camroll: f32,
    pub is_persp: i8,
    pub persp: RegionView3DPersp,
    pub view: RegionView3DView,
    pub view_axis_roll: RegionView3DViewAxisRoll,
    pub viewlock: i8,
    pub viewlock_quad: i8,
    pub rflag: i16,
    pub ofs_lock: [f32; 2],
    pub lviewquat: [f32; 4],
    pub lpersp: RegionView3DPersp,
    pub lview: RegionView3DView,
    pub lview_axis_roll: RegionView3DViewAxisRoll,
    pub ndof_flag: i8,
    pub ndof_ofs: [f32; 3],
    /// `bke::RegionView3DRuntime *runtime`.
    pub runtime: usize,
}
impl Default for RegionView3D {
    fn default() -> Self {
        Self {
            winmat: [[0.0; 4]; 4],
            viewmat: [[0.0; 4]; 4],
            viewinv: [[0.0; 4]; 4],
            persmat: [[0.0; 4]; 4],
            persinv: [[0.0; 4]; 4],
            clip: [[0.0; 4]; 6],
            clipbb: NIL,
            localvd: NIL,
            gridview: 0.0,
            viewquat: [0.0; 4],
            dist: 0.0,
            camdx: 0.0,
            camdy: 0.0,
            pixsize: 0.0,
            ofs: [0.0; 3],
            camzoom: 0.0,
            camroll: 0.0,
            is_persp: 0,
            persp: RV3D_ORTHO,
            view: RV3D_VIEW_USER,
            view_axis_roll: RV3D_VIEW_AXIS_ROLL_0,
            viewlock: 0,
            viewlock_quad: 0,
            rflag: 0,
            ofs_lock: [0.0; 2],
            lviewquat: [0.0; 4],
            lpersp: RV3D_ORTHO,
            lview: RV3D_VIEW_USER,
            lview_axis_roll: RV3D_VIEW_AXIS_ROLL_0,
            ndof_flag: 0,
            ndof_ofs: [0.0; 3],
            runtime: NIL,
        }
    }
}

/// `View3DCursor`.
#[derive(Clone)]
pub struct View3DCursor {
    pub location: [f32; 3],
    pub rotation_quaternion: [f32; 4],
    pub rotation_euler: [f32; 3],
    pub rotation_axis: [f32; 3],
    pub rotation_angle: f32,
    pub rotation_mode: i16,
}
impl Default for View3DCursor {
    fn default() -> Self {
        Self {
            location: [0.0; 3],
            rotation_quaternion: [1.0, 0.0, 0.0, 0.0],
            rotation_euler: [0.0; 3],
            rotation_axis: [0.0, 1.0, 0.0],
            rotation_angle: 0.0,
            rotation_mode: ROT_MODE_XYZ,
        }
    }
}

/// `View3DShading`.
#[derive(Clone)]
pub struct View3DShading {
    pub type_: DrawType,
    pub prev_type: DrawType,
    pub prev_type_wire: DrawType,
    pub color_type: V3DShadingColorType,
    pub flag: View3DShadingFlag,
    pub light: V3DShadingLightingMode,
    pub background_type: V3DShadingBackgroundType,
    pub cavity_type: View3DShadingCavityType,
    pub wire_color_type: V3DShadingColorType,
    pub use_compositor: View3DShadingUseCompositor,
    /// `char studio_light[256]`, `lookdev_light[256]`, `matcap[256]`.
    pub studio_light: [u8; 256],
    pub lookdev_light: [u8; 256],
    pub matcap: [u8; 256],
    pub shadow_intensity: f32,
    pub single_color: [f32; 3],
    pub studiolight_rot_z: f32,
    pub studiolight_background: f32,
    pub studiolight_intensity: f32,
    pub studiolight_blur: f32,
    pub object_outline_color: [f32; 3],
    pub xray_alpha: f32,
    pub xray_alpha_wire: f32,
    pub cavity_valley_factor: f32,
    pub cavity_ridge_factor: f32,
    pub background_color: [f32; 3],
    pub curvature_ridge_factor: f32,
    pub curvature_valley_factor: f32,
    pub render_pass: i32,
    /// `char aov_name[64]`.
    pub aov_name: [u8; 64],
    /// `IDProperty *prop`.
    pub prop: usize,
}
impl Default for View3DShading {
    fn default() -> Self {
        Self {
            type_: OB_SOLID,
            prev_type: OB_SOLID,
            prev_type_wire: 0,
            color_type: V3D_SHADING_MATERIAL_COLOR,
            flag: V3D_SHADING_SPECULAR_HIGHLIGHT
                | V3D_SHADING_XRAY_WIREFRAME
                | V3D_SHADING_SCENE_LIGHTS_RENDER
                | V3D_SHADING_SCENE_WORLD_RENDER,
            light: V3D_LIGHTING_STUDIO,
            background_type: V3D_SHADING_BACKGROUND_THEME,
            cavity_type: V3D_SHADING_CAVITY_CURVATURE,
            wire_color_type: V3D_SHADING_SINGLE_COLOR,
            use_compositor: V3D_SHADING_USE_COMPOSITOR_DISABLED,
            studio_light: [0; 256],
            lookdev_light: [0; 256],
            matcap: [0; 256],
            shadow_intensity: 0.5,
            single_color: [0.8, 0.8, 0.8],
            studiolight_rot_z: 0.0,
            studiolight_background: 0.0,
            studiolight_intensity: 1.0,
            studiolight_blur: 0.0,
            object_outline_color: [0.0; 3],
            xray_alpha: 0.5,
            xray_alpha_wire: 0.5,
            cavity_valley_factor: 1.0,
            cavity_ridge_factor: 1.0,
            background_color: [0.05, 0.05, 0.05],
            curvature_ridge_factor: 1.0,
            curvature_valley_factor: 1.0,
            render_pass: SCE_PASS_COMBINED,
            aov_name: [0; 64],
            prop: NIL,
        }
    }
}
impl View3DShading {
    /// `V3D_USES_SCENE_LIGHTS`.
    pub fn uses_scene_lights(&self) -> bool {
        (self.type_ == crate::object_types::OB_MATERIAL
            && self.flag & V3D_SHADING_SCENE_LIGHTS != 0)
            || (self.type_ == crate::object_types::OB_RENDER
                && self.flag & V3D_SHADING_SCENE_LIGHTS_RENDER != 0)
    }
    /// `V3D_USES_SCENE_WORLD`.
    pub fn uses_scene_world(&self) -> bool {
        (self.type_ == crate::object_types::OB_MATERIAL
            && self.flag & V3D_SHADING_SCENE_WORLD != 0)
            || (self.type_ == crate::object_types::OB_RENDER
                && self.flag & V3D_SHADING_SCENE_WORLD_RENDER != 0)
    }
}

/// `View3DOverlay`.
#[derive(Clone)]
pub struct View3DOverlay {
    pub flag: View3DOverlayFlag,
    pub edit_flag: View3DOverlayEditFlag,
    pub normals_length: f32,
    pub normals_constant_screen_size: f32,
    pub paint_flag: i32,
    pub wpaint_flag: i32,
    pub texture_paint_mode_opacity: f32,
    pub vertex_paint_mode_opacity: f32,
    pub weight_paint_mode_opacity: f32,
    pub sculpt_mode_mask_opacity: f32,
    pub sculpt_mode_face_sets_opacity: f32,
    pub viewer_attribute_opacity: f32,
    pub xray_alpha_bone: f32,
    pub bone_wire_alpha: f32,
    pub fade_alpha: f32,
    pub wireframe_threshold: f32,
    pub wireframe_opacity: f32,
    pub retopology_offset: f32,
    pub gpencil_paper_opacity: f32,
    pub gpencil_grid_opacity: f32,
    pub gpencil_fade_layer: f32,
    pub gpencil_grid_color: [f32; 3],
    pub gpencil_grid_scale: [f32; 2],
    pub gpencil_grid_offset: [f32; 2],
    pub gpencil_grid_subdivisions: i32,
    pub gpencil_vertex_paint_opacity: f32,
    pub handle_display: HandleDisplay,
    pub sculpt_curves_cage_opacity: f32,
}
impl Default for View3DOverlay {
    fn default() -> Self {
        Self {
            flag: V3D_OVERLAY_VIEWER_ATTRIBUTE
                | V3D_OVERLAY_SCULPT_SHOW_MASK
                | V3D_OVERLAY_SCULPT_SHOW_FACE_SETS,
            edit_flag: V3D_OVERLAY_EDIT_FACES
                | V3D_OVERLAY_EDIT_SEAMS
                | V3D_OVERLAY_EDIT_SHARP
                | V3D_OVERLAY_EDIT_FREESTYLE_EDGE
                | V3D_OVERLAY_EDIT_FREESTYLE_FACE
                | V3D_OVERLAY_EDIT_CREASES
                | V3D_OVERLAY_EDIT_BWEIGHTS,
            normals_length: 0.1,
            normals_constant_screen_size: 7.0,
            paint_flag: 0,
            wpaint_flag: 0,
            texture_paint_mode_opacity: 1.0,
            vertex_paint_mode_opacity: 1.0,
            weight_paint_mode_opacity: 1.0,
            sculpt_mode_mask_opacity: 0.75,
            sculpt_mode_face_sets_opacity: 1.0,
            viewer_attribute_opacity: 1.0,
            xray_alpha_bone: 0.5,
            bone_wire_alpha: 1.0,
            fade_alpha: 0.40,
            wireframe_threshold: 1.0,
            wireframe_opacity: 1.0,
            retopology_offset: 0.01,
            gpencil_paper_opacity: 0.5,
            gpencil_grid_opacity: 0.9,
            gpencil_fade_layer: 0.0,
            gpencil_grid_color: [0.5; 3],
            gpencil_grid_scale: [1.0; 2],
            gpencil_grid_offset: [0.0; 2],
            gpencil_grid_subdivisions: 4,
            gpencil_vertex_paint_opacity: 1.0,
            handle_display: CURVE_HANDLE_SELECTED,
            sculpt_curves_cage_opacity: 0.0,
        }
    }
}

/// `SceneStats` placeholder.
#[derive(Clone, Default)]
pub struct SceneStats(pub u64);

/// `View3D_Runtime`.
#[derive(Clone)]
pub struct View3DRuntime {
    /// `void *properties_storage`.
    pub properties_storage: usize,
    /// `void (*properties_storage_free)(void *)`.
    pub properties_storage_free: usize,
    pub flag: i32,
    pub ofs_last_center: [f32; 3],
    /// `SceneStats *local_stats`.
    pub local_stats: usize,
    pub last_sync_time: f32,
    pub last_submission_time: f32,
}
impl Default for View3DRuntime {
    fn default() -> Self {
        Self {
            properties_storage: NIL,
            properties_storage_free: NIL,
            flag: 0,
            ofs_last_center: [0.0; 3],
            local_stats: NIL,
            last_sync_time: 0.0,
            last_submission_time: 0.0,
        }
    }
}

/// `View3D`.
#[derive(Clone)]
pub struct View3D {
    /// `SpaceLink *next`, `*prev`.
    pub next: usize,
    pub prev: usize,
    /// `ListBaseT<ARegion>`.
    pub regionbase: ListBase,
    pub spacetype: i8,
    pub link_flag: i8,
    /// DNA_DEPRECATED `viewquat`, `dist`.
    pub viewquat: [f32; 4],
    pub dist: f32,
    pub bundle_size: f32,
    pub bundle_drawtype: i8,
    /// DNA_DEPRECATED `drawtype`.
    pub drawtype: i8,
    pub multiview_eye: i8,
    pub object_type_exclude_viewport: i32,
    pub object_type_exclude_select: i32,
    /// DNA_DEPRECATED `persp`, `view`.
    pub persp: RegionView3DPersp,
    pub view: RegionView3DView,
    /// `Object *camera`, `*ob_center`.
    pub camera: usize,
    pub ob_center: usize,
    pub render_border: Rctf,
    /// `View3D *localvd`.
    pub localvd: usize,
    /// `char ob_center_bone[64]`.
    pub ob_center_bone: [u8; 64],
    pub local_view_uid: u16,
    /// DNA_DEPRECATED `layact`.
    pub layact: i32,
    pub local_collections_uid: u16,
    pub debug_flag: i16,
    pub ob_center_cursor: i16,
    pub scenelock: i16,
    pub gp_flag: View3DGpFlag,
    pub flag: View3DFlag,
    pub flag2: View3DFlag2,
    pub lens: f32,
    pub grid: f32,
    pub clip_start: f32,
    pub clip_end: f32,
    pub xr_vignette_aperture: f32,
    /// DNA_DEPRECATED `ofs[2]`.
    pub ofs: [f32; 2],
    pub gizmo_flag: i8,
    pub gizmo_show_object: i8,
    pub gizmo_show_armature: i8,
    pub gizmo_show_empty: i8,
    pub gizmo_show_light: i8,
    pub gizmo_show_camera: i8,
    pub gridflag: i8,
    pub gridlines: i16,
    pub gridsubdiv: i16,
    pub vertex_opacity: f32,
    /// DNA_DEPRECATED `bGPdata *gpd`.
    pub gpd: usize,
    pub stereo3d_flag: i16,
    pub stereo3d_camera: i8,
    pub stereo3d_convergence_factor: f32,
    pub stereo3d_volume_alpha: f32,
    pub stereo3d_convergence_alpha: f32,
    pub shading: View3DShading,
    pub overlay: View3DOverlay,
    pub viewer_path: ViewerPath,
    pub runtime: View3DRuntime,
}
impl Default for View3D {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::new(),
            spacetype: SPACE_VIEW3D,
            link_flag: 0,
            viewquat: [0.0; 4],
            dist: 0.0,
            bundle_size: 0.2,
            bundle_drawtype: OB_PLAINAXES,
            drawtype: 0,
            multiview_eye: 0,
            object_type_exclude_viewport: 0,
            object_type_exclude_select: 0,
            persp: 0,
            view: 0,
            camera: NIL,
            ob_center: NIL,
            render_border: Rctf::default(),
            localvd: NIL,
            ob_center_bone: [0; 64],
            local_view_uid: 0,
            layact: 0,
            local_collections_uid: 0,
            debug_flag: 0,
            ob_center_cursor: 0,
            scenelock: 1,
            gp_flag: V3D_GP_SHOW_EDIT_LINES,
            flag: V3D_SELECT_OUTLINE,
            flag2: V3D_SHOW_RECONSTRUCTION
                | V3D_SHOW_ANNOTATION
                | V3D_SHOW_VIEWER
                | V3D_SHOW_CAMERA_GUIDES
                | V3D_SHOW_CAMERA_PASSEPARTOUT,
            lens: 50.0,
            grid: 1.0,
            clip_start: 0.01,
            clip_end: 1000.0,
            xr_vignette_aperture: 0.0,
            ofs: [0.0; 2],
            gizmo_flag: 0,
            gizmo_show_object: 0,
            gizmo_show_armature: 0,
            gizmo_show_empty: 0,
            gizmo_show_light: 0,
            gizmo_show_camera: 0,
            gridflag: V3D_SHOW_X | V3D_SHOW_Y | V3D_SHOW_FLOOR | V3D_SHOW_ORTHO_GRID,
            gridlines: 16,
            gridsubdiv: 10,
            vertex_opacity: 1.0,
            gpd: NIL,
            stereo3d_flag: V3D_S3D_DISPPLANE,
            stereo3d_camera: STEREO_3D_ID,
            stereo3d_convergence_factor: 0.0,
            stereo3d_volume_alpha: 0.05,
            stereo3d_convergence_alpha: 0.15,
            shading: View3DShading::default(),
            overlay: View3DOverlay::default(),
            viewer_path: ViewerPath::default(),
            runtime: View3DRuntime::default(),
        }
    }
}

pub const RV3D_CAMZOOM_MIN: i32 = -30;
pub const RV3D_CAMZOOM_MAX: i32 = 600;
pub const RV3D_CAMZOOM_MIN_FACTOR: f32 = 0.1657359312880714853;
pub const RV3D_CAMZOOM_MAX_FACTOR: f32 = 44.9852813742385702928;
