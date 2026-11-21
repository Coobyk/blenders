//! Port of `DNA_tracking_types.h` — movie tracking data structures.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::id::NIL;
use blenders_bli::listbase::ListBase;
/// `TrackingDistortionModel`.
pub type Trackingdistortionmodel = i16;
pub const TRACKING_DISTORTION_MODEL_POLYNOMIAL: Trackingdistortionmodel = 0;
pub const TRACKING_DISTORTION_MODEL_DIVISION: Trackingdistortionmodel = 1;
pub const TRACKING_DISTORTION_MODEL_NUKE: Trackingdistortionmodel = 2;
pub const TRACKING_DISTORTION_MODEL_BROWN: Trackingdistortionmodel = 3;

/// `TrackingCameraUnits`.
pub type Trackingcameraunits = i16;
pub const CAMERA_UNITS_PX: Trackingcameraunits = 0;
pub const CAMERA_UNITS_MM: Trackingcameraunits = 1;

/// `TrackingMarkerFlag`.
pub type Trackingmarkerflag = i32;
pub const MARKER_DISABLED: Trackingmarkerflag = 1 << 0;
pub const MARKER_TRACKED: Trackingmarkerflag = 1 << 1;
pub const MARKER_GRAPH_SEL_X: Trackingmarkerflag = 1 << 2;
pub const MARKER_GRAPH_SEL_Y: Trackingmarkerflag = 1 << 3;
pub const MARKER_GRAPH_SEL: Trackingmarkerflag = MARKER_GRAPH_SEL_X | MARKER_GRAPH_SEL_Y;

/// `TrackingTrackFlag`.
pub type Trackingtrackflag = i32;
pub const TRACK_SELECT: Trackingtrackflag = 1 << 0;
pub const TRACK_HAS_BUNDLE: Trackingtrackflag = 1 << 1;
pub const TRACK_DISABLE_RED: Trackingtrackflag = 1 << 2;
pub const TRACK_DISABLE_GREEN: Trackingtrackflag = 1 << 3;
pub const TRACK_DISABLE_BLUE: Trackingtrackflag = 1 << 4;
pub const TRACK_HIDDEN: Trackingtrackflag = 1 << 5;
pub const TRACK_LOCKED: Trackingtrackflag = 1 << 6;
pub const TRACK_CUSTOMCOLOR: Trackingtrackflag = 1 << 7;
pub const TRACK_USE_2D_STAB: Trackingtrackflag = 1 << 8;
pub const TRACK_PREVIEW_GRAYSCALE: Trackingtrackflag = 1 << 9;
pub const TRACK_DOPE_SEL: Trackingtrackflag = 1 << 10;
pub const TRACK_PREVIEW_ALPHA: Trackingtrackflag = 1 << 11;
pub const TRACK_USE_2D_STAB_ROT: Trackingtrackflag = 1 << 12;

/// `TrackingMotionModel`.
pub type Trackingmotionmodel = i16;
pub const TRACK_MOTION_MODEL_TRANSLATION: Trackingmotionmodel = 0;
pub const TRACK_MOTION_MODEL_TRANSLATION_ROTATION: Trackingmotionmodel = 1;
pub const TRACK_MOTION_MODEL_TRANSLATION_SCALE: Trackingmotionmodel = 2;
pub const TRACK_MOTION_MODEL_TRANSLATION_ROTATION_SCALE: Trackingmotionmodel = 3;
pub const TRACK_MOTION_MODEL_AFFINE: Trackingmotionmodel = 4;
pub const TRACK_MOTION_MODEL_HOMOGRAPHY: Trackingmotionmodel = 5;

/// `TrackingAlgorithmFlag`.
pub type Trackingalgorithmflag = i32;
pub const TRACK_ALGORITHM_FLAG_USE_BRUTE: Trackingalgorithmflag = 1 << 0;
pub const TRACK_ALGORITHM_FLAG_USE_NORMALIZATION: Trackingalgorithmflag = 1 << 2;
pub const TRACK_ALGORITHM_FLAG_USE_MASK: Trackingalgorithmflag = 1 << 3;

/// `eTrackFrameMatch`.
pub type Trackframematch = i16;
pub const TRACK_MATCH_KEYFRAME: Trackframematch = 0;
pub const TRACK_MATCH_PREVIOUS_FRAME: Trackframematch = 1;

/// `TrackingMotionFlag`.
pub type Trackingmotionflag = i16;
pub const TRACKING_MOTION_TRIPOD: Trackingmotionflag = 1 << 0;
pub const TRACKING_MOTION_MODAL: Trackingmotionflag = TRACKING_MOTION_TRIPOD;

/// `TrackingSpeed`.
pub type Trackingspeed = i16;
pub const TRACKING_SPEED_FASTEST: Trackingspeed = 0;
pub const TRACKING_SPEED_REALTIME: Trackingspeed = 1;
pub const TRACKING_SPEED_HALF: Trackingspeed = 2;
pub const TRACKING_SPEED_QUARTER: Trackingspeed = 4;
pub const TRACKING_SPEED_DOUBLE: Trackingspeed = 5;

/// `TrackingSettingsReconstructionFlag`.
pub type Trackingsettingsreconstructionflag = i32;
pub const TRACKING_USE_KEYFRAME_SELECTION: Trackingsettingsreconstructionflag = 1 << 1;

/// `TrackingRefineCameraFlag`.
pub type Trackingrefinecameraflag = i32;
pub const REFINE_NO_INTRINSICS: Trackingrefinecameraflag = 0;
pub const REFINE_FOCAL_LENGTH: Trackingrefinecameraflag = 1 << 0;
pub const REFINE_PRINCIPAL_POINT: Trackingrefinecameraflag = 1 << 1;
pub const REFINE_RADIAL_DISTORTION: Trackingrefinecameraflag = 1 << 2;
pub const REFINE_TANGENTIAL_DISTORTION: Trackingrefinecameraflag = 1 << 3;

/// `TrackingStabilizationFlag`.
pub type Trackingstabilizationflag = i32;
pub const TRACKING_2D_STABILIZATION: Trackingstabilizationflag = 1 << 0;
pub const TRACKING_AUTOSCALE: Trackingstabilizationflag = 1 << 1;
pub const TRACKING_STABILIZE_ROTATION: Trackingstabilizationflag = 1 << 2;
pub const TRACKING_STABILIZE_SCALE: Trackingstabilizationflag = 1 << 3;
pub const TRACKING_SHOW_STAB_TRACKS: Trackingstabilizationflag = 1 << 5;

/// `TrackingStabilizationFilter`.
pub type Trackingstabilizationfilter = i32;
pub const TRACKING_FILTER_NEAREST: Trackingstabilizationfilter = 0;
pub const TRACKING_FILTER_BILINEAR: Trackingstabilizationfilter = 1;
pub const TRACKING_FILTER_BICUBIC: Trackingstabilizationfilter = 2;

/// `TrackingReconstructionFlag`.
pub type Trackingreconstructionflag = i32;
pub const TRACKING_RECONSTRUCTED: Trackingreconstructionflag = 1 << 0;

/// `TrackingObjectFlag`.
pub type Trackingobjectflag = i32;
pub const TRACKING_OBJECT_CAMERA: Trackingobjectflag = 1 << 0;

/// `TrackingDopesheetSort`.
pub type Trackingdopesheetsort = i16;
pub const TRACKING_DOPE_SORT_NAME: Trackingdopesheetsort = 0;
pub const TRACKING_DOPE_SORT_LONGEST: Trackingdopesheetsort = 1;
pub const TRACKING_DOPE_SORT_TOTAL: Trackingdopesheetsort = 2;
pub const TRACKING_DOPE_SORT_AVERAGE_ERROR: Trackingdopesheetsort = 3;
pub const TRACKING_DOPE_SORT_START: Trackingdopesheetsort = 4;
pub const TRACKING_DOPE_SORT_END: Trackingdopesheetsort = 5;

/// `TrackingDopesheetFlag`.
pub type Trackingdopesheetflag = i16;
pub const TRACKING_DOPE_SORT_INVERSE: Trackingdopesheetflag = 1 << 0;
pub const TRACKING_DOPE_SELECTED_ONLY: Trackingdopesheetflag = 1 << 1;
pub const TRACKING_DOPE_SHOW_HIDDEN: Trackingdopesheetflag = 1 << 2;

/// `TrackingCoverage`.
pub type Trackingcoverage = i32;
pub const TRACKING_COVERAGE_BAD: Trackingcoverage = 0;
pub const TRACKING_COVERAGE_ACCEPTABLE: Trackingcoverage = 1;
pub const TRACKING_COVERAGE_OK: Trackingcoverage = 2;

/// `TrackingPlaneMarkerFlag`.
pub type Trackingplanemarkerflag = i32;
pub const PLANE_MARKER_DISABLED: Trackingplanemarkerflag = 1 << 0;
pub const PLANE_MARKER_TRACKED: Trackingplanemarkerflag = 1 << 1;

/// `TrackingPlaneTrackFlag`.
pub type Trackingplanetrackflag = i32;
pub const PLANE_TRACK_SELECT: Trackingplanetrackflag = 1 << 0;
pub const PLANE_TRACK_HIDDEN: Trackingplanetrackflag = 1 << 1;
pub const PLANE_TRACK_LOCKED: Trackingplanetrackflag = 1 << 2;
pub const PLANE_TRACK_AUTOKEY: Trackingplanetrackflag = 1 << 3;


/// `MovieReconstructedCamera`.
#[derive(Clone, Default)]
pub struct MovieReconstructedCamera {
    pub framenr: i32,
    pub error: f32,
    pub mat: [[f32; 4]; 4],
}

/// `MovieTrackingCamera`.
#[derive(Clone)]
pub struct MovieTrackingCamera {
    /// `void *intrinsics`.
    pub intrinsics: usize,
    pub distortion_model: i16,
    pub sensor_width: f32,
    pub pixel_aspect: f32,
    pub focal: f32,
    pub units: i16,
    pub principal_point: [f32; 2],
    pub principal_legacy: [f32; 2],
    pub k1: f32,
    pub k2: f32,
    pub k3: f32,
    pub division_k1: f32,
    pub division_k2: f32,
    pub nuke_k1: f32,
    pub nuke_k2: f32,
    pub nuke_p1: f32,
    pub nuke_p2: f32,
    pub brown_k1: f32,
    pub brown_k2: f32,
    pub brown_k3: f32,
    pub brown_k4: f32,
    pub brown_p1: f32,
    pub brown_p2: f32,
}
impl Default for MovieTrackingCamera {
    fn default() -> Self {
        Self {
            intrinsics: NIL,
            distortion_model: TRACKING_DISTORTION_MODEL_POLYNOMIAL,
            sensor_width: 0.0,
            pixel_aspect: 0.0,
            focal: 0.0,
            units: CAMERA_UNITS_PX,
            principal_point: [0.0; 2],
            principal_legacy: [0.0; 2],
            k1: 0.0, k2: 0.0, k3: 0.0,
            division_k1: 0.0, division_k2: 0.0,
            nuke_k1: 0.0, nuke_k2: 0.0,
            nuke_p1: 0.0, nuke_p2: 0.0,
            brown_k1: 0.0, brown_k2: 0.0, brown_k3: 0.0, brown_k4: 0.0,
            brown_p1: 0.0, brown_p2: 0.0,
        }
    }
}

/// `MovieTrackingMarker`.
#[derive(Clone, Default)]
pub struct MovieTrackingMarker {
    pub pos: [f32; 2],
    pub pattern_corners: [[f32; 2]; 4],
    pub search_min: [f32; 2],
    pub search_max: [f32; 2],
    pub framenr: i32,
    pub flag: i32,
}

/// `MovieTrackingTrack`.
#[derive(Clone)]
pub struct MovieTrackingTrack {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    pub pat_min_legacy: [f32; 2],
    pub pat_max_legacy: [f32; 2],
    pub search_min_legacy: [f32; 2],
    pub search_max_legacy: [f32; 2],
    pub offset: [f32; 2],
    pub markersnr: i32,
    /// `MovieTrackingMarker *markers` → Vec.
    pub markers: Vec<MovieTrackingMarker>,
    pub bundle_pos: [f32; 3],
    pub error: f32,
    pub flag: i32,
    pub pat_flag: i32,
    pub search_flag: i32,
    pub color: [f32; 3],
    pub frames_limit: i16,
    pub margin: i16,
    pub pattern_match: i16,
    pub motion_model: i16,
    pub algorithm_flag: i32,
    pub minimum_correlation: f32,
    /// `bGPdata *gpd`.
    pub gpd: usize,
    pub weight: f32,
    pub weight_stab: f32,
}
impl Default for MovieTrackingTrack {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            pat_min_legacy: [0.0; 2],
            pat_max_legacy: [0.0; 2],
            search_min_legacy: [0.0; 2],
            search_max_legacy: [0.0; 2],
            offset: [0.0; 2],
            markersnr: 0,
            markers: Vec::new(),
            bundle_pos: [0.0; 3],
            error: 0.0,
            flag: 0,
            pat_flag: 0,
            search_flag: 0,
            color: [0.0; 3],
            frames_limit: 0,
            margin: 0,
            pattern_match: TRACK_MATCH_KEYFRAME,
            motion_model: TRACK_MOTION_MODEL_TRANSLATION,
            algorithm_flag: 0,
            minimum_correlation: 0.0,
            gpd: NIL,
            weight: 0.0,
            weight_stab: 0.0,
        }
    }
}

/// `MovieTrackingPlaneMarker`.
#[derive(Clone, Default)]
pub struct MovieTrackingPlaneMarker {
    pub corners: [[f32; 2]; 4],
    pub framenr: i32,
    pub flag: i32,
}

/// `MovieTrackingPlaneTrack`.
#[derive(Clone)]
pub struct MovieTrackingPlaneTrack {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    /// `MovieTrackingTrack **point_tracks` → Vec of refs.
    pub point_tracks: Vec<usize>,
    pub point_tracksnr: i32,
    /// `MovieTrackingPlaneMarker *markers` → Vec.
    pub markers: Vec<MovieTrackingPlaneMarker>,
    pub markersnr: i32,
    pub flag: i32,
    /// `Image *image`.
    pub image: usize,
    pub image_opacity: f32,
    pub last_marker: i32,
}
impl Default for MovieTrackingPlaneTrack {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            point_tracks: Vec::new(),
            point_tracksnr: 0,
            markers: Vec::new(),
            markersnr: 0,
            flag: 0,
            image: NIL,
            image_opacity: 0.0,
            last_marker: 0,
        }
    }
}

/// `MovieTrackingSettings`.
#[derive(Clone)]
pub struct MovieTrackingSettings {
    pub default_motion_model: i16,
    pub default_algorithm_flag: i32,
    pub default_minimum_correlation: f32,
    pub default_pattern_size: i16,
    pub default_search_size: i16,
    pub default_frames_limit: i16,
    pub default_margin: i16,
    pub default_pattern_match: i16,
    pub default_flag: i32,
    pub default_weight: f32,
    pub motion_flag: i16,
    pub speed: i16,
    pub keyframe1_legacy: i32,
    pub keyframe2_legacy: i32,
    pub reconstruction_flag: i32,
    pub refine_camera_intrinsics: i32,
    pub dist: f32,
    pub clean_frames: i32,
    pub clean_action: i32,
    pub clean_error: f32,
    pub object_distance: f32,
}
impl Default for MovieTrackingSettings {
    fn default() -> Self {
        Self {
            default_motion_model: TRACK_MOTION_MODEL_TRANSLATION,
            default_algorithm_flag: 0,
            default_minimum_correlation: 0.0,
            default_pattern_size: 0,
            default_search_size: 0,
            default_frames_limit: 0,
            default_margin: 0,
            default_pattern_match: TRACK_MATCH_KEYFRAME,
            default_flag: 0,
            default_weight: 0.0,
            motion_flag: 0,
            speed: TRACKING_SPEED_FASTEST,
            keyframe1_legacy: 0,
            keyframe2_legacy: 0,
            reconstruction_flag: 0,
            refine_camera_intrinsics: REFINE_NO_INTRINSICS,
            dist: 0.0,
            clean_frames: 0,
            clean_action: 0,
            clean_error: 0.0,
            object_distance: 0.0,
        }
    }
}

/// `MovieTrackingStabilization`.
#[derive(Clone)]
pub struct MovieTrackingStabilization {
    pub flag: i32,
    pub tot_track: i32,
    pub act_track: i32,
    pub tot_rot_track: i32,
    pub act_rot_track: i32,
    pub maxscale: f32,
    /// `MovieTrackingTrack *rot_track_legacy`.
    pub rot_track_legacy: usize,
    pub anchor_frame: i32,
    pub target_pos: [f32; 2],
    pub target_rot: f32,
    pub scale: f32,
    pub locinf: f32,
    pub scaleinf: f32,
    pub rotinf: f32,
    pub filter: i32,
}
impl Default for MovieTrackingStabilization {
    fn default() -> Self {
        Self {
            flag: 0,
            tot_track: 0,
            act_track: 0,
            tot_rot_track: 0,
            act_rot_track: 0,
            maxscale: 0.0,
            rot_track_legacy: NIL,
            anchor_frame: 0,
            target_pos: [0.0; 2],
            target_rot: 0.0,
            scale: 0.0,
            locinf: 0.0,
            scaleinf: 0.0,
            rotinf: 0.0,
            filter: TRACKING_FILTER_NEAREST,
        }
    }
}

/// `MovieTrackingReconstruction`.
#[derive(Clone, Default)]
pub struct MovieTrackingReconstruction {
    pub flag: i32,
    pub error: f32,
    pub last_camera: i32,
    pub camnr: i32,
    /// `MovieReconstructedCamera *cameras` → Vec.
    pub cameras: Vec<MovieReconstructedCamera>,
}

/// `MovieTrackingObject`.
#[derive(Clone)]
pub struct MovieTrackingObject {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    pub flag: i32,
    pub scale: f32,
    /// `ListBaseT<MovieTrackingTrack>`, `ListBaseT<MovieTrackingPlaneTrack>`.
    pub tracks: ListBase,
    pub plane_tracks: ListBase,
    /// `MovieTrackingTrack *active_track`, `MovieTrackingPlaneTrack *active_plane_track`.
    pub active_track: usize,
    pub active_plane_track: usize,
    pub reconstruction: MovieTrackingReconstruction,
    pub keyframe1: i32,
    pub keyframe2: i32,
}
impl Default for MovieTrackingObject {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            flag: 0,
            scale: 0.0,
            tracks: ListBase::default(),
            plane_tracks: ListBase::default(),
            active_track: NIL,
            active_plane_track: NIL,
            reconstruction: MovieTrackingReconstruction::default(),
            keyframe1: 0,
            keyframe2: 0,
        }
    }
}

/// `MovieTrackingStats`.
#[derive(Clone)]
pub struct MovieTrackingStats {
    /// `char message[256]`.
    pub message: [u8; 256],
}
impl Default for MovieTrackingStats {
    fn default() -> Self {
        Self { message: [0; 256] }
    }
}

/// `MovieTrackingDopesheetChannel`.
#[derive(Clone)]
pub struct MovieTrackingDopesheetChannel {
    pub next: usize,
    pub prev: usize,
    /// `MovieTrackingTrack *track`.
    pub track: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    pub tot_segment: i32,
    /// `int *segments` → Vec.
    pub segments: Vec<i32>,
    pub max_segment: i32,
    pub total_frames: i32,
    pub first_not_disabled_marker_framenr: i32,
    pub last_not_disabled_marker_framenr: i32,
}
impl Default for MovieTrackingDopesheetChannel {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            track: NIL,
            name: [0; 64],
            tot_segment: 0,
            segments: Vec::new(),
            max_segment: 0,
            total_frames: 0,
            first_not_disabled_marker_framenr: 0,
            last_not_disabled_marker_framenr: 0,
        }
    }
}

/// `MovieTrackingDopesheetCoverageSegment`.
#[derive(Clone)]
pub struct MovieTrackingDopesheetCoverageSegment {
    pub next: usize,
    pub prev: usize,
    pub coverage: i32,
    pub start_frame: i32,
    pub end_frame: i32,
}
impl Default for MovieTrackingDopesheetCoverageSegment {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            coverage: TRACKING_COVERAGE_BAD,
            start_frame: 0,
            end_frame: 0,
        }
    }
}

/// `MovieTrackingDopesheet`.
#[derive(Clone)]
pub struct MovieTrackingDopesheet {
    pub ok: i32,
    pub sort_method: i16,
    pub flag: i16,
    /// `ListBaseT<MovieTrackingDopesheetCoverageSegment>`, `ListBaseT<MovieTrackingDopesheetChannel>`.
    pub coverage_segments: ListBase,
    pub channels: ListBase,
    pub tot_channel: i32,
}
impl Default for MovieTrackingDopesheet {
    fn default() -> Self {
        Self {
            ok: 0,
            sort_method: TRACKING_DOPE_SORT_NAME,
            flag: 0,
            coverage_segments: ListBase::default(),
            channels: ListBase::default(),
            tot_channel: 0,
        }
    }
}

/// `MovieTracking`.
#[derive(Clone)]
pub struct MovieTracking {
    pub settings: MovieTrackingSettings,
    pub camera: MovieTrackingCamera,
    /// `ListBaseT<MovieTrackingTrack>` (legacy), `ListBaseT<MovieTrackingPlaneTrack>` (legacy).
    pub tracks_legacy: ListBase,
    pub plane_tracks_legacy: ListBase,
    pub reconstruction_legacy: MovieTrackingReconstruction,
    pub stabilization: MovieTrackingStabilization,
    /// `MovieTrackingTrack *act_track_legacy`, `MovieTrackingPlaneTrack *act_plane_track_legacy`.
    pub act_track_legacy: usize,
    pub act_plane_track_legacy: usize,
    /// `ListBaseT<MovieTrackingObject>`.
    pub objects: ListBase,
    pub objectnr: i32,
    pub tot_object: i32,
    /// `MovieTrackingStats *stats`.
    pub stats: usize,
    pub dopesheet: MovieTrackingDopesheet,
}
impl Default for MovieTracking {
    fn default() -> Self {
        Self {
            settings: MovieTrackingSettings::default(),
            camera: MovieTrackingCamera::default(),
            tracks_legacy: ListBase::default(),
            plane_tracks_legacy: ListBase::default(),
            reconstruction_legacy: MovieTrackingReconstruction::default(),
            stabilization: MovieTrackingStabilization::default(),
            act_track_legacy: NIL,
            act_plane_track_legacy: NIL,
            objects: ListBase::default(),
            objectnr: 0,
            tot_object: 0,
            stats: NIL,
            dopesheet: MovieTrackingDopesheet::default(),
        }
    }
}
