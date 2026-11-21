//! Port of `DNA_action_types.h` — `bAction`, `ActionLayer/Slot/Strip`,
//! `bPose`, `bPoseChannel`, `bMotionPath`, `bDopeSheet`, `SpaceAction` + enums.
#![allow(non_camel_case_types)]

use crate::id::{ID, NIL};
use crate::small_types::SessionUid;
use blenders_bli::listbase::ListBase;

pub const DNA_DEFAULT_ACTION_LAST_SLOT_HANDLE: i32 = 0x37627bf5;

/// `eMotionPathVert_Flag`.
pub const MOTIONPATH_VERT_SEL: i32 = 1 << 0;
pub const MOTIONPATH_VERT_KEY: i32 = 1 << 1;
pub const MOTIONPATH_VERT_EVALUATED: i32 = 1 << 2;

/// `eMotionPath_Flag`.
pub const MOTIONPATH_FLAG_BHEAD: i32 = 1 << 0;
pub const MOTIONPATH_FLAG_EDIT: i32 = 1 << 1;
pub const MOTIONPATH_FLAG_CUSTOM: i32 = 1 << 2;
pub const MOTIONPATH_FLAG_LINES: i32 = 1 << 3;
pub const MOTIONPATH_FLAG_BAKE_CAMERA: i32 = 1 << 4;

/// `eAnimViz_RecalcFlags`.
pub const ANIMVIZ_RECALC_PATHS: i16 = 1 << 0;

/// `eMotionPaths_Types`.
pub const MOTIONPATH_TYPE_RANGE: i16 = 0;
pub const MOTIONPATH_TYPE_ACFRA: i16 = 1;

/// `eMotionPath_Ranges`.
pub const MOTIONPATH_RANGE_SCENE: i16 = 0;
pub const MOTIONPATH_RANGE_KEYS_SELECTED: i16 = 1;
pub const MOTIONPATH_RANGE_KEYS_ALL: i16 = 2;
pub const MOTIONPATH_RANGE_MANUAL: i16 = 3;

/// `eMotionPaths_ViewFlag`.
pub const MOTIONPATH_VIEW_FNUMS: i16 = 1 << 0;
pub const MOTIONPATH_VIEW_KFRAS: i16 = 1 << 1;
pub const MOTIONPATH_VIEW_KFNOS: i16 = 1 << 2;
pub const MOTIONPATH_VIEW_KFACT: i16 = 1 << 3;

/// `eMotionPath_BakeFlag`.
pub const MOTIONPATH_BAKE_HEADS: i16 = 1 << 1;
pub const MOTIONPATH_BAKE_HAS_PATHS: i16 = 1 << 2;
pub const MOTIONPATH_BAKE_CAMERA_SPACE: i16 = 1 << 3;

/// `bPoseChannelRuntimeFlag`.
pub const POSE_RUNTIME_TRANSFORM: u8 = 1 << 0;
pub const POSE_RUNTIME_HINGE_CHILD_TRANSFORM: u8 = 1 << 1;
pub const POSE_RUNTIME_TRANSFORM_CHILD: u8 = 1 << 2;
pub const POSE_RUNTIME_IN_SELECTION_AREA: u8 = 1 << 3;

/// `ePchan_Flag`.
pub const POSE_TRANSFORM_AT_CUSTOM_TX: i16 = 1 << 4;
pub const POSE_TRANSFORM_AROUND_CUSTOM_TX: i16 = 1 << 5;
pub const POSE_SELECTED: i16 = 1 << 6;
pub const POSE_SELECTED_ROOT: i16 = 1 << 7;
pub const POSE_SELECTED_TIP: i16 = 1 << 8;
pub const POSE_SELECTED_ALL: i16 = POSE_SELECTED | POSE_SELECTED_ROOT | POSE_SELECTED_TIP;
pub const POSE_CHAIN: i16 = 1 << 9;
pub const POSE_DONE: i16 = 1 << 10;
pub const POSE_IKTREE: i16 = 1 << 13;
pub const POSE_IKSPLINE: i16 = 1 << 15;

/// `ePchan_ConstFlag`.
pub const PCHAN_HAS_IK: i8 = 1 << 0;
pub const PCHAN_HAS_CONST: i8 = 1 << 1;
pub const PCHAN_HAS_NO_TARGET: i8 = 1 << 3;
pub const PCHAN_HAS_SPLINEIK: i8 = 1 << 5;
pub const PCHAN_INFLUENCED_BY_IK: i8 = 1 << 6;

/// `ePchan_IkFlag`.
pub const BONE_IK_NO_XDOF: i16 = 1 << 0;
pub const BONE_IK_NO_YDOF: i16 = 1 << 1;
pub const BONE_IK_NO_ZDOF: i16 = 1 << 2;
pub const BONE_IK_XLIMIT: i16 = 1 << 3;
pub const BONE_IK_YLIMIT: i16 = 1 << 4;
pub const BONE_IK_ZLIMIT: i16 = 1 << 5;
pub const BONE_IK_ROTCTL: i16 = 1 << 6;
pub const BONE_IK_LINCTL: i16 = 1 << 7;
pub const BONE_IK_NO_XDOF_TEMP: i16 = 1 << 10;
pub const BONE_IK_NO_YDOF_TEMP: i16 = 1 << 11;
pub const BONE_IK_NO_ZDOF_TEMP: i16 = 1 << 12;

/// `ePchan_DrawFlag`.
pub const PCHAN_DRAW_NO_CUSTOM_BONE_SIZE: i8 = 1 << 0;
pub const PCHAN_DRAW_HIDDEN: i8 = 1 << 1;

/// `ePchan_BBoneFlag` (DNA_DEPRECATED_ALLOW).
pub const PCHAN_BBONE_CUSTOM_HANDLES: i8 = 1 << 1;
pub const PCHAN_BBONE_CUSTOM_START_REL: i8 = 1 << 2;
pub const PCHAN_BBONE_CUSTOM_END_REL: i8 = 1 << 3;

/// `eRotationModes`.
pub const ROT_MODE_QUAT: i16 = 0;
pub const ROT_MODE_XYZ: i16 = 1;
pub const ROT_MODE_XZY: i16 = 2;
pub const ROT_MODE_YXZ: i16 = 3;
pub const ROT_MODE_YZX: i16 = 4;
pub const ROT_MODE_ZXY: i16 = 5;
pub const ROT_MODE_ZYX: i16 = 6;
pub const ROT_MODE_AXISANGLE: i16 = -1;
pub const ROT_MODE_MIN: i16 = ROT_MODE_AXISANGLE;
pub const ROT_MODE_MAX: i16 = ROT_MODE_ZYX;

/// `ePose_Flags`.
pub const POSE_RECALC: i16 = 1 << 0;
pub const POSE_CONSTRAINTS_TIMEDEPEND: i16 = 1 << 3;
pub const POSE_WAS_REBUILT: i16 = 1 << 5;
pub const POSE_FLAG_DEPRECATED: i16 = 1 << 6;
pub const POSE_CONSTRAINTS_NEED_UPDATE_FLAGS: i16 = 1 << 7;
pub const POSE_AUTO_IK: i16 = 1 << 8;
pub const POSE_MIRROR_EDIT: i16 = 1 << 9;
pub const POSE_MIRROR_RELATIVE: i16 = 1 << 10;

/// `ePose_IKSolverType`.
pub const IKSOLVER_STANDARD: i32 = 0;
pub const IKSOLVER_ITASC: i32 = 1;

/// `eItasc_Flags`.
pub const ITASC_AUTO_STEP: i16 = 1 << 0;
pub const ITASC_INITIAL_REITERATION: i16 = 1 << 1;
pub const ITASC_REITERATION: i16 = 1 << 2;
pub const ITASC_SIMULATION: i16 = 1 << 3;
pub const ITASC_TRANSLATE_ROOT_BONES: i16 = 1 << 4;

/// `eItasc_Solver`.
pub const ITASC_SOLVER_SDLS: i16 = 0;
pub const ITASC_SOLVER_DLS: i16 = 1;

/// `eActionGroup_Flag`.
pub const AGRP_SELECTED: u32 = 1 << 0;
pub const AGRP_ACTIVE: u32 = 1 << 1;
pub const AGRP_PROTECTED: u32 = 1 << 2;
pub const AGRP_EXPANDED: u32 = 1 << 3;
pub const AGRP_MUTED: u32 = 1 << 4;
pub const AGRP_NOTVISIBLE: u32 = 1 << 5;
pub const AGRP_EXPANDED_G: u32 = 1 << 6;
pub const AGRP_MODIFIERS_OFF: u32 = 1 << 7;
pub const AGRP_CURVES_ALWAYS_VISIBLE: u32 = 1 << 17;
pub const AGRP_TEMP: u32 = 1 << 30;
pub const AGRP_MOVED: u32 = 1 << 31;

/// `eAction_Flags`.
pub const ACT_COLLAPSED: i32 = 1 << 0;
pub const ACT_SELECTED: i32 = 1 << 1;
pub const ACT_MUTED: i32 = 1 << 9;
pub const ACT_FRAME_RANGE: i32 = 1 << 12;
pub const ACT_CYCLIC: i32 = 1 << 13;

/// `eDopeSheet_FilterFlag`.
pub const ADS_FILTER_ONLYSEL: i32 = 1 << 0;
pub const ADS_FILTER_ONLYDRIVERS: i32 = 1 << 1;
pub const ADS_FILTER_ONLYNLA: i32 = 1 << 2;
pub const ADS_FILTER_SELEDIT: i32 = 1 << 3;
pub const ADS_FILTER_SUMMARY: i32 = 1 << 4;
pub const ADS_FILTER_ONLY_SLOTS_OF_ACTIVE: i32 = 1 << 5;
pub const ADS_FILTER_NOSHAPEKEYS: i32 = 1 << 6;
pub const ADS_FILTER_NOMESH: i32 = 1 << 7;
pub const ADS_FILTER_NOOBJ: i32 = 1 << 8;
pub const ADS_FILTER_NOLAT: i32 = 1 << 9;
pub const ADS_FILTER_NOCAM: i32 = 1 << 10;
pub const ADS_FILTER_NOMAT: i32 = 1 << 11;
pub const ADS_FILTER_NOLAM: i32 = 1 << 12;
pub const ADS_FILTER_NOCUR: i32 = 1 << 13;
pub const ADS_FILTER_NOWOR: i32 = 1 << 14;
pub const ADS_FILTER_NOSCE: i32 = 1 << 15;
pub const ADS_FILTER_NOPART: i32 = 1 << 16;
pub const ADS_FILTER_NOMBA: i32 = 1 << 17;
pub const ADS_FILTER_NOARM: i32 = 1 << 18;
pub const ADS_FILTER_NONTREE: i32 = 1 << 19;
pub const ADS_FILTER_NOTEX: i32 = 1 << 20;
pub const ADS_FILTER_NOSPK: i32 = 1 << 21;
pub const ADS_FILTER_NOLINESTYLE: i32 = 1 << 22;
pub const ADS_FILTER_NOMODIFIERS: i32 = 1 << 23;
pub const ADS_FILTER_NOGPENCIL: i32 = 1 << 24;
pub const ADS_FILTER_NLA_NOACT: i32 = 1 << 25;
pub const ADS_FILTER_INCL_HIDDEN: i32 = 1 << 26;
pub const ADS_FILTER_ONLY_ERRORS: i32 = 1 << 28;

/// `eDopeSheet_FilterFlag2`.
pub const ADS_FILTER_NOCACHEFILES: i32 = 1 << 1;
pub const ADS_FILTER_NOMOVIECLIPS: i32 = 1 << 2;
pub const ADS_FILTER_NOHAIR: i32 = 1 << 3;
pub const ADS_FILTER_NOPOINTCLOUD: i32 = 1 << 4;
pub const ADS_FILTER_NOVOLUME: i32 = 1 << 5;
pub const ADS_FILTER_DRIVER_FALLBACK_AS_ERROR: i32 = 1 << 6;
pub const ADS_FILTER_NOLIGHTPROBE: i32 = 1 << 7;

/// `eDopeSheet_Flag`.
pub const ADS_FLAG_SUMMARY_COLLAPSED: i32 = 1 << 0;
pub const ADS_FLAG_SHOW_DBFILTERS: i32 = 1 << 1;
pub const ADS_FLAG_FUZZY_NAMES: i32 = 1 << 2;
pub const ADS_FLAG_NO_DB_SORT: i32 = 1 << 3;
pub const ADS_FLAG_INVERT_FILTER: i32 = 1 << 4;

/// `SpaceActionOverlays_Flag`.
pub const ADS_OVERLAY_SHOW_OVERLAYS: i32 = 1 << 0;
pub const ADS_SHOW_SCENE_STRIP_FRAME_RANGE: i32 = 1 << 1;

/// `eSAction_Flag`.
pub const SACTION_MOVING: i16 = 1 << 0;
pub const SACTION_SLIDERS: i16 = 1 << 1;
pub const SACTION_DRAWTIME: i16 = 1 << 2;
pub const SACTION_NOTRANSKEYCULL: i16 = 1 << 4;
pub const SACTION_POSEMARKERS_SHOW: i16 = 1 << 6;
pub const SACTION_NOREALTIMEUPDATES: i16 = 1 << 10;
pub const SACTION_MARKERS_MOVE: i16 = 1 << 11;
pub const SACTION_SHOW_INTERPOLATION: i16 = 1 << 12;
pub const SACTION_SHOW_EXTREMES: i16 = 1 << 13;
pub const SACTION_SHOW_MARKERS: i16 = 1 << 14;

/// `eSAction_Runtime_Flag`.
pub const SACTION_RUNTIME_FLAG_NEED_CHAN_SYNC: i8 = 1 << 0;

/// `eAnimEdit_Context`.
pub const SACTCONT_ACTION: i8 = 0;
pub const SACTCONT_SHAPEKEY: i8 = 1;
pub const SACTCONT_GPENCIL: i8 = 2;
pub const SACTCONT_DOPESHEET: i8 = 3;
pub const SACTCONT_MASK: i8 = 4;
pub const SACTCONT_CACHEFILE: i8 = 5;
pub const SACTCONT_TIMELINE: i8 = 6;

/// `eAnimEdit_AutoSnap` (DNA_DEPRECATED).
pub const SACTSNAP_STEP: i32 = 1;
pub const SACTSNAP_FRAME: i32 = 2;
pub const SACTSNAP_MARKER: i32 = 3;
pub const SACTSNAP_SECOND: i32 = 4;
pub const SACTSNAP_TSTEP: i32 = 5;

/// `eTimeline_Cache_Flag`.
pub const TIME_CACHE_DISPLAY: u16 = 1 << 0;
pub const TIME_CACHE_SOFTBODY: u16 = 1 << 1;
pub const TIME_CACHE_PARTICLES: u16 = 1 << 2;
pub const TIME_CACHE_CLOTH: u16 = 1 << 3;
pub const TIME_CACHE_SMOKE: u16 = 1 << 4;
pub const TIME_CACHE_DYNAMICPAINT: u16 = 1 << 5;
pub const TIME_CACHE_RIGIDBODY: u16 = 1 << 6;
pub const TIME_CACHE_SIMULATION_NODES: u16 = 1 << 7;
pub const TIME_CACHE_COMPOSITOR: u16 = 1 << 8;

/// `bMotionPathVert`.
#[derive(Clone, Copy, Default)]
pub struct BMotionPathVert {
    pub co: [f32; 3],
    pub flag: i32,
}

/// `bMotionPath`.
#[derive(Clone)]
pub struct BMotionPath {
    /// `bMotionPathVert *points` → Vec.
    pub points: Vec<BMotionPathVert>,
    pub length: i32,
    pub start_frame: i32,
    pub end_frame: i32,
    pub color: [f32; 3],
    pub color_post: [f32; 3],
    pub line_thickness: i32,
    pub flag: i32,
    /// `gpu::VertBuf *points_vbo`, `gpu::Batch *` — opaque.
    pub points_vbo: usize,
    pub batch_line: usize,
    pub batch_points: usize,
    /// `bke::motionpath::Runtime *runtime`.
    pub runtime: usize,
}
impl Default for BMotionPath {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            length: 0,
            start_frame: 0,
            end_frame: 0,
            color: [0.0; 3],
            color_post: [0.0; 3],
            line_thickness: 0,
            flag: 0,
            points_vbo: NIL,
            batch_line: NIL,
            batch_points: NIL,
            runtime: NIL,
        }
    }
}

/// `bAnimVizSettings`.
#[derive(Clone)]
pub struct BAnimVizSettings {
    pub recalc: i16,
    pub path_type: i16,
    pub path_step: i16,
    pub path_range: i16,
    pub path_viewflag: i16,
    pub path_bakeflag: i16,
    pub path_sf: i32,
    pub path_ef: i32,
    pub path_bc: i32,
    pub path_ac: i32,
}
impl Default for BAnimVizSettings {
    fn default() -> Self {
        Self {
            recalc: 0,
            path_type: 0,
            path_step: 0,
            path_range: 0,
            path_viewflag: MOTIONPATH_VIEW_KFRAS | MOTIONPATH_VIEW_KFNOS,
            path_bakeflag: 0,
            path_sf: 1,
            path_ef: 250,
            path_bc: 10,
            path_ac: 10,
        }
    }
}

/// `bPoseChannelDrawData`.
#[derive(Clone, Default)]
pub struct BPoseChannelDrawData {
    pub solid_color: [f32; 4],
    pub wire_color: [f32; 4],
    pub bbone_matrix_len: i32,
    /// `float bbone_matrix[][4][4]` → Vec of 4x4.
    pub bbone_matrix: Vec<[[f32; 4]; 4]>,
}

/// `bPoseChannel_BBoneSegmentBoundary`.
#[derive(Clone, Copy, Default)]
pub struct BPoseChannelBboneSegmentBoundary {
    pub point: [f32; 3],
    pub plane_normal: [f32; 3],
    pub plane_offset: f32,
    pub depth_scale: f32,
}

pub const BONE_INDEX_UNKNOWN: i64 = -1;

/// `DualQuat`/`Mat4` → use vec_types DualQuat, `Mat4` = [[f32;4];4].
pub type Mat4 = [[f32; 4]; 4];

/// `bPoseChannel_Runtime`.
#[derive(Clone)]
pub struct BPoseChannelRuntime {
    pub session_uid: SessionUid,
    pub bone_index: i64,
    pub deform_dual_quat: crate::vec_types::DualQuat,
    pub bbone_segments: i32,
    pub bbone_arc_length_reciprocal: f32,
    pub flag: u8,
    /// `Mat4 *bbone_*` → Vec.
    pub bbone_rest_mats: Vec<Mat4>,
    pub bbone_pose_mats: Vec<Mat4>,
    pub bbone_deform_mats: Vec<Mat4>,
    pub bbone_dual_quats: Vec<crate::vec_types::DualQuat>,
    pub bbone_segment_boundaries: Vec<BPoseChannelBboneSegmentBoundary>,
}
impl Default for BPoseChannelRuntime {
    fn default() -> Self {
        Self {
            session_uid: SessionUid::default(),
            bone_index: BONE_INDEX_UNKNOWN,
            deform_dual_quat: crate::vec_types::DualQuat::default(),
            bbone_segments: 0,
            bbone_arc_length_reciprocal: 0.0,
            flag: 0,
            bbone_rest_mats: Vec::new(),
            bbone_pose_mats: Vec::new(),
            bbone_deform_mats: Vec::new(),
            bbone_dual_quats: Vec::new(),
            bbone_segment_boundaries: Vec::new(),
        }
    }
}

pub use crate::armature_types::BoneColor;

/// `bPoseChannel`.
#[derive(Clone)]
pub struct BPoseChannel {
    pub next: usize,
    pub prev: usize,
    /// `IDProperty *prop`, `*system_properties`.
    pub prop: usize,
    pub system_properties: usize,
    /// `ListBaseT<bConstraint>`.
    pub constraints: ListBase,
    /// `char name[64]`.
    pub name: [u8; 64],
    pub flag: i16,
    pub ikflag: i16,
    pub protectflag: i16,
    pub agrp_index: i16,
    pub constflag: i8,
    /// DNA_DEPRECATED `selectflag`.
    pub selectflag: i8,
    pub drawflag: i8,
    /// DNA_DEPRECATED `bboneflag`.
    pub bboneflag: i8,
    /// `bPoseChannel *parent`, `*child`.
    pub parent: usize,
    pub child: usize,
    /// `ListBaseT<PoseTree>` iktree, `ListBaseT<tSplineIK_Tree>` siktree.
    pub iktree: ListBase,
    pub siktree: ListBase,
    /// `bMotionPath *mpath`.
    pub mpath: usize,
    /// `Object *custom`.
    pub custom: usize,
    /// `bPoseChannel *custom_tx`.
    pub custom_tx: usize,
    pub custom_scale: f32,
    pub custom_scale_xyz: [f32; 3],
    pub custom_translation: [f32; 3],
    pub custom_rotation_euler: [f32; 3],
    pub custom_shape_wire_width: f32,
    pub loc: [f32; 3],
    pub scale: [f32; 3],
    pub eul: [f32; 3],
    pub quat: [f32; 4],
    pub rot_axis: [f32; 3],
    pub rot_angle: f32,
    pub rotmode: i16,
    pub chan_mat: [[f32; 4]; 4],
    pub pose_mat: [[f32; 4]; 4],
    pub disp_mat: [[f32; 4]; 4],
    pub disp_tail_mat: [[f32; 4]; 4],
    pub constinv: [[f32; 4]; 4],
    pub pose_head: [f32; 3],
    pub pose_tail: [f32; 3],
    pub limitmin: [f32; 3],
    pub limitmax: [f32; 3],
    pub stiffness: [f32; 3],
    pub ikstretch: f32,
    pub ikrotweight: f32,
    pub iklinweight: f32,
    pub roll1: f32,
    pub roll2: f32,
    pub curve_in_x: f32,
    pub curve_in_z: f32,
    pub curve_out_x: f32,
    pub curve_out_z: f32,
    pub ease1: f32,
    pub ease2: f32,
    /// DNA_DEPRECATED `scale_in_x` etc.
    pub scale_in_x: f32,
    pub scale_in_z: f32,
    pub scale_out_x: f32,
    pub scale_out_z: f32,
    pub scale_in: [f32; 3],
    pub scale_out: [f32; 3],
    /// `bPoseChannel *bbone_prev`, `*bbone_next`.
    pub bbone_prev: usize,
    pub bbone_next: usize,
    /// `void *temp`.
    pub temp: usize,
    /// `bPoseChannelDrawData *draw_data`.
    pub draw_data: usize,
    /// `bPoseChannel *orig_pchan`.
    pub orig_pchan: usize,
    pub color: BoneColor,
    pub runtime: BPoseChannelRuntime,
}
impl Default for BPoseChannel {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            prop: NIL,
            system_properties: NIL,
            constraints: ListBase::new(),
            name: [0; 64],
            flag: 0,
            ikflag: 0,
            protectflag: 0,
            agrp_index: 0,
            constflag: 0,
            selectflag: 0,
            drawflag: 0,
            bboneflag: 0,
            parent: NIL,
            child: NIL,
            iktree: ListBase::new(),
            siktree: ListBase::new(),
            mpath: NIL,
            custom: NIL,
            custom_tx: NIL,
            custom_scale: 0.0,
            custom_scale_xyz: [1.0; 3],
            custom_translation: [0.0; 3],
            custom_rotation_euler: [0.0; 3],
            custom_shape_wire_width: 0.0,
            loc: [0.0; 3],
            scale: [1.0; 3],
            eul: [0.0; 3],
            quat: [1.0, 0.0, 0.0, 0.0],
            rot_axis: [0.0; 3],
            rot_angle: 0.0,
            rotmode: 0,
            chan_mat: [[0.0; 4]; 4],
            pose_mat: [[0.0; 4]; 4],
            disp_mat: [[0.0; 4]; 4],
            disp_tail_mat: [[0.0; 4]; 4],
            constinv: [[0.0; 4]; 4],
            pose_head: [0.0; 3],
            pose_tail: [0.0; 3],
            limitmin: [0.0; 3],
            limitmax: [0.0; 3],
            stiffness: [0.0; 3],
            ikstretch: 0.0,
            ikrotweight: 0.0,
            iklinweight: 0.0,
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
            bbone_prev: NIL,
            bbone_next: NIL,
            temp: NIL,
            draw_data: NIL,
            orig_pchan: NIL,
            color: BoneColor::default(),
            runtime: BPoseChannelRuntime::default(),
        }
    }
}

/// `bPose`.
#[derive(Clone)]
pub struct BPose {
    /// `ListBaseT<bPoseChannel>`.
    pub chanbase: ListBase,
    pub flag: i16,
    pub ctime: f32,
    pub stride_offset: [f32; 3],
    pub cyclic_offset: [f32; 3],
    /// `ListBaseT<bActionGroup>`.
    pub agroups: ListBase,
    pub active_group: i32,
    pub iksolver: i32,
    /// `bItasc *ikparam`.
    pub ikparam: usize,
    pub avs: BAnimVizSettings,
    /// `bke::bPoseRuntime *runtime`.
    pub runtime: usize,
}
impl Default for BPose {
    fn default() -> Self {
        Self {
            chanbase: ListBase::new(),
            flag: 0,
            ctime: 0.0,
            stride_offset: [0.0; 3],
            cyclic_offset: [0.0; 3],
            agroups: ListBase::new(),
            active_group: 0,
            iksolver: 0,
            ikparam: NIL,
            avs: BAnimVizSettings::default(),
            runtime: NIL,
        }
    }
}

/// `bIKParam`.
#[derive(Clone, Default)]
pub struct BIkParam {
    pub iksolver: i32,
}

/// `bItasc`.
#[derive(Clone, Default)]
pub struct BItasc {
    pub iksolver: i32,
    pub precision: f32,
    pub numiter: i16,
    pub numstep: i16,
    pub minstep: f32,
    pub maxstep: f32,
    pub solver: i16,
    pub flag: i16,
    pub feedback: f32,
    pub maxvel: f32,
    pub dampmax: f32,
    pub dampeps: f32,
}

/// `ThemeWireColor` (DNA_color_types.h) placeholder.
#[derive(Clone, Copy, Default)]
pub struct ThemeWireColor(pub u64);

/// `bActionGroup`.
#[derive(Clone)]
pub struct BActionGroup {
    pub next: usize,
    pub prev: usize,
    /// `ListBaseT<FCurve>`.
    pub channels: ListBase,
    pub fcurve_range_start: i32,
    pub fcurve_range_length: i32,
    /// `ActionChannelbag *channelbag`.
    pub channelbag: usize,
    pub flag: u32,
    pub custom_col: i32,
    /// `char name[64]`.
    pub name: [u8; 64],
    pub cs: ThemeWireColor,
}
impl Default for BActionGroup {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            channels: ListBase::new(),
            fcurve_range_start: 0,
            fcurve_range_length: 0,
            channelbag: NIL,
            flag: 0,
            custom_col: 0,
            name: [0; 64],
            cs: ThemeWireColor::default(),
        }
    }
}

/// `bAction` — layered action data-block.
#[derive(Clone)]
pub struct BAction {
    pub id: ID,
    /// `ActionLayer **layer_array` → Vec of refs.
    pub layer_array: Vec<usize>,
    pub layer_active_index: i32,
    /// `ActionSlot **slot_array` → Vec.
    pub slot_array: Vec<usize>,
    pub last_slot_handle: i32,
    /// `ActionStripKeyframeData **strip_keyframe_data_array` → Vec.
    pub strip_keyframe_data_array: Vec<usize>,
    /// `ListBaseT<FCurve>` (legacy curves).
    pub curves: ListBase,
    /// `ListBaseT<bActionGroup>`.
    pub groups: ListBase,
    /// `ListBaseT<TimeMarker>`.
    pub markers: ListBase,
    pub flag: i32,
    pub active_marker: i32,
    pub idroot: i32,
    pub frame_start: f32,
    pub frame_end: f32,
    /// `PreviewImage *preview`.
    pub preview: usize,
}
impl Default for BAction {
    fn default() -> Self {
        Self {
            id: ID::default(),
            layer_array: Vec::new(),
            layer_active_index: 0,
            slot_array: Vec::new(),
            last_slot_handle: DNA_DEFAULT_ACTION_LAST_SLOT_HANDLE,
            strip_keyframe_data_array: Vec::new(),
            curves: ListBase::new(),
            groups: ListBase::new(),
            markers: ListBase::new(),
            flag: 0,
            active_marker: 0,
            idroot: 0,
            frame_start: 0.0,
            frame_end: 0.0,
            preview: NIL,
        }
    }
}

/// `bDopeSheet`.
#[derive(Clone)]
pub struct BDopeSheet {
    /// `ID *source`.
    pub source: usize,
    /// `ListBase chanbase`.
    pub chanbase: ListBase,
    /// `Collection *filter_grp`.
    pub filter_grp: usize,
    /// `char searchstr[64]`.
    pub searchstr: [u8; 64],
    pub filterflag: i32,
    pub filterflag2: i32,
    pub flag: i32,
    pub rename_index: i32,
}
impl Default for BDopeSheet {
    fn default() -> Self {
        Self {
            source: NIL,
            chanbase: ListBase::new(),
            filter_grp: NIL,
            searchstr: [0; 64],
            filterflag: 0,
            filterflag2: 0,
            flag: 0,
            rename_index: 0,
        }
    }
}

/// `SpaceAction_Runtime`.
#[derive(Clone, Default)]
pub struct SpaceActionRuntime {
    pub flag: i8,
}

/// `SpaceActionOverlays`.
#[derive(Clone, Default)]
pub struct SpaceActionOverlays {
    pub flag: i32,
}

pub use crate::view2d_types::View2D;

/// `SpaceAction`.
#[derive(Clone)]
pub struct SpaceAction {
    /// `SpaceLink *next`, `*prev`.
    pub next: usize,
    pub prev: usize,
    /// `ListBaseT<ARegion>`.
    pub regionbase: ListBase,
    pub spacetype: i8,
    pub link_flag: i8,
    /// DNA_DEPRECATED `View2D v2d`.
    pub v2d: View2D,
    /// DNA_DEPRECATED `bAction *action`.
    pub action: usize,
    pub ads: BDopeSheet,
    pub timeslide: f32,
    pub flag: i16,
    pub mode: i8,
    pub mode_prev: i8,
    /// DNA_DEPRECATED `autosnap`.
    pub autosnap: i8,
    pub cache_display: u16,
    pub overlays: SpaceActionOverlays,
    pub runtime: SpaceActionRuntime,
}
impl Default for SpaceAction {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            regionbase: ListBase::new(),
            spacetype: 0,
            link_flag: 0,
            v2d: View2D::default(),
            action: NIL,
            ads: BDopeSheet::default(),
            timeslide: 0.0,
            flag: 0,
            mode: 0,
            mode_prev: 0,
            autosnap: 0,
            cache_display: 0,
            overlays: SpaceActionOverlays::default(),
            runtime: SpaceActionRuntime::default(),
        }
    }
}

/// `ActionLayer`.
#[derive(Clone)]
pub struct ActionLayer {
    /// `char name[64]`.
    pub name: [u8; 64],
    pub influence: f32,
    pub layer_flags: u8,
    pub layer_mix_mode: i8,
    /// `ActionStrip **strip_array` → Vec.
    pub strip_array: Vec<usize>,
}
impl Default for ActionLayer {
    fn default() -> Self {
        Self {
            name: [0; 64],
            influence: 1.0,
            layer_flags: 0,
            layer_mix_mode: 0,
            strip_array: Vec::new(),
        }
    }
}

/// `ActionSlot`.
#[derive(Clone)]
pub struct ActionSlot {
    /// `char identifier[MAX_ID_NAME]`.
    pub identifier: [u8; 258],
    pub idtype: i16,
    pub handle: i32,
    pub slot_flags: i8,
    /// `animrig::SlotRuntime *runtime`.
    pub runtime: usize,
}
impl Default for ActionSlot {
    fn default() -> Self {
        Self {
            identifier: [0; 258],
            idtype: 0,
            handle: 0,
            slot_flags: 0,
            runtime: NIL,
        }
    }
}

/// `ActionStrip`.
#[derive(Clone)]
pub struct ActionStrip {
    pub strip_type: i8,
    pub data_index: i32,
    pub frame_start: f32,
    pub frame_end: f32,
    pub frame_offset: f32,
}
impl Default for ActionStrip {
    fn default() -> Self {
        Self {
            strip_type: 0,
            data_index: -1,
            frame_start: f32::NEG_INFINITY,
            frame_end: f32::INFINITY,
            frame_offset: 0.0,
        }
    }
}

/// `ActionStripKeyframeData`.
#[derive(Clone, Default)]
pub struct ActionStripKeyframeData {
    /// `ActionChannelbag **channelbag_array` → Vec.
    pub channelbag_array: Vec<usize>,
}

/// `ActionChannelbag`.
#[derive(Clone, Default)]
pub struct ActionChannelbag {
    pub slot_handle: i32,
    /// `bActionGroup **group_array` → Vec.
    pub group_array: Vec<usize>,
    /// `FCurve **fcurve_array` → Vec.
    pub fcurve_array: Vec<usize>,
}

/* Cross-check: `AnimData::slot_handle`, `NlaStrip::action_slot_handle`,
   `ActionSlot::handle`, `bAction::last_slot_handle`, `ActionChannelbag::slot_handle`
   are all `i32` — mirroring the C++ static_asserts. */
const _: fn() = || {
    let a = crate::anim_types::AnimData::default();
    let n = crate::anim_types::NlaStrip::default();
    let s = ActionSlot::default();
    let b = BAction::default();
    let c = ActionChannelbag::default();
    let _all_i32: (i32, i32, i32, i32, i32) =
        (a.slot_handle, n.action_slot_handle, s.handle, b.last_slot_handle, c.slot_handle);
};
