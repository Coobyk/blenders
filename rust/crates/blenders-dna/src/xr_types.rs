//! Port of `DNA_xr_types.h` — XR session settings + action maps.

use crate::id::NIL;
use crate::view3d_types::View3DShading;
use blenders_bli::listbase::ListBase;

/// `eXrSessionFlag`.
pub type XrSessionFlag = i32;
pub const XR_SESSION_USE_POSITION_TRACKING: XrSessionFlag = 1 << 0;
pub const XR_SESSION_USE_ABSOLUTE_TRACKING: XrSessionFlag = 1 << 1;

/// `eXRSessionBasePoseType` (char).
pub type XrSessionBasePoseType = i8;
pub const XR_BASE_POSE_SCENE_CAMERA: XrSessionBasePoseType = 0;
pub const XR_BASE_POSE_OBJECT: XrSessionBasePoseType = 1;
pub const XR_BASE_POSE_CUSTOM: XrSessionBasePoseType = 2;

/// `eXrSessionControllerDrawStyle` (char).
pub type XrSessionControllerDrawStyle = i8;
pub const XR_CONTROLLER_DRAW_DARK: XrSessionControllerDrawStyle = 0;
pub const XR_CONTROLLER_DRAW_LIGHT: XrSessionControllerDrawStyle = 1;
pub const XR_CONTROLLER_DRAW_DARK_RAY: XrSessionControllerDrawStyle = 2;
pub const XR_CONTROLLER_DRAW_LIGHT_RAY: XrSessionControllerDrawStyle = 3;

/// `eXrActionType` (char).
pub type XrActionType = i8;
pub const XR_BOOLEAN_INPUT: XrActionType = 1;
pub const XR_FLOAT_INPUT: XrActionType = 2;
pub const XR_VECTOR2F_INPUT: XrActionType = 3;
pub const XR_POSE_INPUT: XrActionType = 4;
pub const XR_VIBRATION_OUTPUT: XrActionType = 100;

/// `eXrOpFlag` (short).
pub type XrOpFlag = i16;
pub const XR_OP_PRESS: XrOpFlag = 0;
pub const XR_OP_RELEASE: XrOpFlag = 1;
pub const XR_OP_MODAL: XrOpFlag = 2;

/// `eXrActionFlag` (short).
pub type XrActionFlag = i16;
pub const XR_ACTION_BIMANUAL: XrActionFlag = 1 << 0;

/// `eXrHapticFlag` (short).
pub type XrHapticFlag = i16;
pub const XR_HAPTIC_MATCHUSERPATHS: XrHapticFlag = 1 << 0;
pub const XR_HAPTIC_PRESS: XrHapticFlag = 1 << 1;
pub const XR_HAPTIC_RELEASE: XrHapticFlag = 1 << 2;
pub const XR_HAPTIC_REPEAT: XrHapticFlag = 1 << 3;

/// `eXrAxisFlag` (short).
pub type XrAxisFlag = i16;
pub const XR_AXIS0_POS: XrAxisFlag = 1 << 0;
pub const XR_AXIS0_NEG: XrAxisFlag = 1 << 1;
pub const XR_AXIS1_POS: XrAxisFlag = 1 << 2;
pub const XR_AXIS1_NEG: XrAxisFlag = 1 << 3;

/// `eXrPoseFlag` (short).
pub type XrPoseFlag = i16;
pub const XR_POSE_GRIP: XrPoseFlag = 1 << 0;
pub const XR_POSE_AIM: XrPoseFlag = 1 << 1;

/// `eXrViewfinderHand`.
pub type XrViewfinderHand = i32;
pub const XR_VIEWFINDER_HAND_LEFT: XrViewfinderHand = 0;
pub const XR_VIEWFINDER_HAND_RIGHT: XrViewfinderHand = 1;

/// `eXrViewfinderMode`.
pub type XrViewfinderMode = i32;
pub const XR_VIEWFINDER_MODE_LIVE: XrViewfinderMode = 0;
pub const XR_VIEWFINDER_MODE_PLAYBACK: XrViewfinderMode = 1;
pub const XR_VIEWFINDER_MODE_CONFIRM: XrViewfinderMode = 2;

/// `eXrViewfinderLiveAction`.
pub type XrViewfinderLiveAction = i32;
pub const XR_VIEWFINDER_ACTION_LIVE_LENS: XrViewfinderLiveAction = 0;
pub const XR_VIEWFINDER_ACTION_LIVE_DOF: XrViewfinderLiveAction = 1;
pub const XR_VIEWFINDER_ACTION_LIVE_FOCUS: XrViewfinderLiveAction = 2;
pub const XR_VIEWFINDER_ACTION_LIVE_APERTURE: XrViewfinderLiveAction = 3;

/// `eXrViewfinderPlaybackAction`.
pub type XrViewfinderPlaybackAction = i32;
pub const XR_VIEWFINDER_ACTION_PB_BROWSE: XrViewfinderPlaybackAction = 0;
pub const XR_VIEWFINDER_ACTION_PB_PREVIEW: XrViewfinderPlaybackAction = 1;
pub const XR_VIEWFINDER_ACTION_PB_DELETE: XrViewfinderPlaybackAction = 2;

/// `eXrViewfinderConfirmAction`.
pub type XrViewfinderConfirmAction = i32;
pub const XR_VIEWFINDER_ACTION_CF_CONFIRM: XrViewfinderConfirmAction = 0;
pub const XR_VIEWFINDER_ACTION_CF_CANCEL: XrViewfinderConfirmAction = 1;

/// `XR_MAX_USER_PATH_LENGTH`, `XR_MAX_COMPONENT_PATH_LENGTH`.
pub const XR_MAX_USER_PATH_LENGTH: usize = 64;
pub const XR_MAX_COMPONENT_PATH_LENGTH: usize = 192;

/// `XrSessionSettings`.
#[derive(Clone)]
pub struct XrSessionSettings {
    pub shading: View3DShading,
    pub base_scale: f32,
    pub base_pose_type: XrSessionBasePoseType,
    /// `Object *base_pose_object`.
    pub base_pose_object: usize,
    pub base_pose_location: [f32; 3],
    pub base_pose_angle: f32,
    pub draw_flags: i8,
    pub controller_draw_style: i8,
    pub viewfinder_enabled: i8,
    pub viewfinder_crosshair_enabled: i8,
    pub viewfinder_hand: i8,
    pub viewfinder_scale: f32,
    pub viewfinder_passepartout_overscan: f32,
    pub viewfinder_passepartout_opacity: f32,
    pub clip_start: f32,
    pub clip_end: f32,
    pub flag: i32,
    pub object_type_exclude_viewport: i32,
    pub object_type_exclude_select: i32,
    pub fly_speed: f32,
    pub view_scale: f32,
}
impl Default for XrSessionSettings {
    fn default() -> Self {
        Self {
            shading: View3DShading::default(),
            base_scale: 0.0,
            base_pose_type: XR_BASE_POSE_SCENE_CAMERA,
            base_pose_object: NIL,
            base_pose_location: [0.0; 3],
            base_pose_angle: 0.0,
            draw_flags: 0,
            controller_draw_style: 0,
            viewfinder_enabled: 0,
            viewfinder_crosshair_enabled: 0,
            viewfinder_hand: 0,
            viewfinder_scale: 1.0,
            viewfinder_passepartout_overscan: 0.5,
            viewfinder_passepartout_opacity: 0.5,
            clip_start: 0.0,
            clip_end: 0.0,
            flag: 0,
            object_type_exclude_viewport: 0,
            object_type_exclude_select: 0,
            fly_speed: 0.0,
            view_scale: 1.0,
        }
    }
}

/// `XrComponentPath`.
#[derive(Clone)]
pub struct XrComponentPath {
    pub next: usize,
    pub prev: usize,
    /// `char path[192]`.
    pub path: [u8; 192],
}
impl Default for XrComponentPath {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            path: [0; 192],
        }
    }
}

/// `XrActionMapBinding`.
#[derive(Clone)]
pub struct XrActionMapBinding {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`, `profile[256]`.
    pub name: [u8; 64],
    pub profile: [u8; 256],
    /// `ListBaseT<XrComponentPath>`.
    pub component_paths: ListBase,
    pub float_threshold: f32,
    pub axis_flag: XrAxisFlag,
    pub pose_location: [f32; 3],
    pub pose_rotation: [f32; 3],
}
impl Default for XrActionMapBinding {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            profile: [0; 256],
            component_paths: ListBase::default(),
            float_threshold: 0.0,
            axis_flag: 0,
            pose_location: [0.0; 3],
            pose_rotation: [0.0; 3],
        }
    }
}

/// `XrUserPath`.
#[derive(Clone)]
pub struct XrUserPath {
    pub next: usize,
    pub prev: usize,
    /// `char path[64]`.
    pub path: [u8; 64],
}
impl Default for XrUserPath {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            path: [0; 64],
        }
    }
}

/// `XrActionMapItem`.
#[derive(Clone)]
pub struct XrActionMapItem {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    pub type_: XrActionType,
    /// `ListBaseT<XrUserPath>`.
    pub user_paths: ListBase,
    /// `char op[64]` — operator idname.
    pub op: [u8; 64],
    /// `IDProperty *op_properties`.
    pub op_properties: usize,
    /// `PointerRNA *op_properties_ptr`.
    pub op_properties_ptr: usize,
    pub op_flag: XrOpFlag,
    pub action_flag: XrActionFlag,
    pub haptic_flag: XrHapticFlag,
    pub pose_flag: XrPoseFlag,
    /// `char haptic_name[64]`.
    pub haptic_name: [u8; 64],
    pub haptic_duration: f32,
    pub haptic_frequency: f32,
    pub haptic_amplitude: f32,
    pub selbinding: i16,
    /// `ListBaseT<XrActionMapBinding>`.
    pub bindings: ListBase,
}
impl Default for XrActionMapItem {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            type_: 0,
            user_paths: ListBase::default(),
            op: [0; 64],
            op_properties: NIL,
            op_properties_ptr: NIL,
            op_flag: XR_OP_PRESS,
            action_flag: 0,
            haptic_flag: 0,
            pose_flag: 0,
            haptic_name: [0; 64],
            haptic_duration: 0.0,
            haptic_frequency: 0.0,
            haptic_amplitude: 0.0,
            selbinding: 0,
            bindings: ListBase::default(),
        }
    }
}

/// `XrActionMap`.
#[derive(Clone)]
pub struct XrActionMap {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    /// `ListBaseT<XrActionMapItem>`.
    pub items: ListBase,
    pub selitem: i16,
}
impl Default for XrActionMap {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            items: ListBase::default(),
            selitem: 0,
        }
    }
}
