//! Port of `DNA_camera_types.h` — `Camera`, `CameraDOFSettings`,
//! `CameraStereoSettings`, `CameraBGImage`, `Camera_Runtime`.
#![allow(non_camel_case_types)]
use crate::id::{ID, NIL};
use crate::image_types::ImageUser;
use crate::small_types::GpuDofSettings;
use blenders_bli::listbase::ListBase;

const DEG2RADF: fn(f32) -> f32 = |deg| deg * (std::f32::consts::PI / 180.0);

/// `eCamera_Type`.
pub const CAM_PERSP: i8 = 0;
pub const CAM_ORTHO: i8 = 1;
pub const CAM_PANO: i8 = 2;
pub const CAM_CUSTOM: i8 = 3;

/// `eCamera_PanoType`.
pub const CAM_PANORAMA_EQUIRECTANGULAR: i8 = 0;
pub const CAM_PANORAMA_FISHEYE_EQUIDISTANT: i8 = 1;
pub const CAM_PANORAMA_FISHEYE_EQUISOLID: i8 = 2;
pub const CAM_PANORAMA_MIRRORBALL: i8 = 3;
pub const CAM_PANORAMA_FISHEYE_LENS_POLYNOMIAL: i8 = 4;
pub const CAM_PANORAMA_EQUIANGULAR_CUBEMAP_FACE: i8 = 5;
pub const CAM_PANORAMA_CENTRAL_CYLINDRICAL: i8 = 6;

/// `eCamera_CustomMode`.
pub const CAM_CUSTOM_SHADER_INTERNAL: i32 = 0;
pub const CAM_CUSTOM_SHADER_EXTERNAL: i32 = 1;

/// `eCompositionGuideFlags`.
/// `eCompositionGuideFlags`.
pub type Compositionguideflags = i16;
pub const COMPOSITION_GUIDES_CENTER: i16 = 1 << 0;
pub const COMPOSITION_GUIDES_CENTER_DIAG: i16 = 1 << 1;
pub const COMPOSITION_GUIDES_THIRDS: i16 = 1 << 2;
pub const COMPOSITION_GUIDES_GOLDEN: i16 = 1 << 3;
pub const COMPOSITION_GUIDES_GOLDEN_TRI_A: i16 = 1 << 4;
pub const COMPOSITION_GUIDES_GOLDEN_TRI_B: i16 = 1 << 5;
pub const COMPOSITION_GUIDES_HARMONY_TRI_A: i16 = 1 << 6;
pub const COMPOSITION_GUIDES_HARMONY_TRI_B: i16 = 1 << 7;

/// `eCamera_Flag`.
pub const CAM_SHOWLIMITS: i16 = 1 << 0;
pub const CAM_SHOWMIST: i16 = 1 << 1;
pub const CAM_SHOWPASSEPARTOUT: i16 = 1 << 2;
pub const CAM_SHOW_SAFE_MARGINS: i16 = 1 << 3;
pub const CAM_SHOWNAME: i16 = 1 << 4;
pub const CAM_ANGLETOGGLE: i16 = 1 << 5;
pub const CAM_DS_EXPAND: i16 = 1 << 6;
pub const CAM_PANORAMA: i16 = 1 << 7;
pub const CAM_SHOWSENSOR: i16 = 1 << 8;
pub const CAM_SHOW_SAFE_CENTER: i16 = 1 << 9;
pub const CAM_SHOW_BG_IMAGE: i16 = 1 << 10;

/// `eCamera_SensorFit`.
pub const CAMERA_SENSOR_FIT_AUTO: i8 = 0;
pub const CAMERA_SENSOR_FIT_HOR: i8 = 1;
pub const CAMERA_SENSOR_FIT_VERT: i8 = 2;

pub const DEFAULT_SENSOR_WIDTH: f32 = 36.0;
pub const DEFAULT_SENSOR_HEIGHT: f32 = 24.0;

/// `eCamera_Stereo_ConvergenceMode`.
pub const CAM_S3D_OFFAXIS: i16 = 0;
pub const CAM_S3D_PARALLEL: i16 = 1;
pub const CAM_S3D_TOE: i16 = 2;

/// `eCamera_Stereo_Pivot`.
pub const CAM_S3D_PIVOT_LEFT: i16 = 0;
pub const CAM_S3D_PIVOT_RIGHT: i16 = 1;
pub const CAM_S3D_PIVOT_CENTER: i16 = 2;

/// `eCamera_Stereo_Flag`.
pub const CAM_S3D_SPHERICAL: i16 = 1 << 0;
pub const CAM_S3D_POLE_MERGE: i16 = 1 << 1;

/// `eCamera_BGImage_Flag`.
pub const CAM_BGIMG_FLAG_EXPANDED: i16 = 1 << 1;
pub const CAM_BGIMG_FLAG_CAMERACLIP: i16 = 1 << 2;
pub const CAM_BGIMG_FLAG_DISABLED: i16 = 1 << 3;
pub const CAM_BGIMG_FLAG_FOREGROUND: i16 = 1 << 4;
pub const CAM_BGIMG_FLAG_CAMERA_ASPECT: i16 = 1 << 5;
pub const CAM_BGIMG_FLAG_CAMERA_CROP: i16 = 1 << 6;
pub const CAM_BGIMG_FLAG_FLIP_X: i16 = 1 << 7;
pub const CAM_BGIMG_FLAG_FLIP_Y: i16 = 1 << 8;
pub const CAM_BGIMG_FLAG_OVERRIDE_LIBRARY_LOCAL: i16 = 1 << 9;

/// `eCamera_BGImage_Source`.
pub const CAM_BGIMG_SOURCE_IMAGE: i16 = 0;
pub const CAM_BGIMG_SOURCE_MOVIE: i16 = 1;

/// `eCamera_DOF_Flag`.
pub const CAM_DOF_ENABLED: i16 = 1 << 0;

/// `MovieClipUser` (DNA_movieclip_types.h) placeholder.
#[derive(Clone, Default)]
pub struct MovieClipUser(pub u64);

/// `CameraStereoSettings`.
#[derive(Clone)]
pub struct CameraStereoSettings {
    pub interocular_distance: f32,
    pub convergence_distance: f32,
    pub convergence_mode: i16,
    pub pivot: i16,
    pub flag: i16,
    pub pole_merge_angle_from: f32,
    pub pole_merge_angle_to: f32,
}
impl Default for CameraStereoSettings {
    fn default() -> Self {
        Self {
            interocular_distance: 0.065,
            convergence_distance: 30.0 * 0.065,
            convergence_mode: 0,
            pivot: 0,
            flag: 0,
            pole_merge_angle_from: DEG2RADF(60.0),
            pole_merge_angle_to: DEG2RADF(75.0),
        }
    }
}

/// `CameraBGImage`.
#[derive(Clone)]
pub struct CameraBgImage {
    pub next: usize,
    pub prev: usize,
    /// `Image *ima`.
    pub ima: usize,
    pub iuser: ImageUser,
    /// `MovieClip *clip`.
    pub clip: usize,
    pub cuser: MovieClipUser,
    pub offset: [f32; 2],
    pub scale: f32,
    pub rotation: f32,
    pub alpha: f32,
    pub flag: i16,
    pub source: i16,
}
impl Default for CameraBgImage {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            ima: NIL,
            iuser: ImageUser::default(),
            clip: NIL,
            cuser: MovieClipUser::default(),
            offset: [0.0; 2],
            scale: 0.0,
            rotation: 0.0,
            alpha: 0.0,
            flag: 0,
            source: 0,
        }
    }
}

/// `CameraDOFSettings`.
#[derive(Clone)]
pub struct CameraDofSettings {
    /// `Object *focus_object`.
    pub focus_object: usize,
    /// `char focus_subtarget[64]`.
    pub focus_subtarget: [u8; 64],
    pub focus_distance: f32,
    pub aperture_fstop: f32,
    pub aperture_rotation: f32,
    pub aperture_ratio: f32,
    pub aperture_blades: i32,
    pub flag: i16,
}
impl Default for CameraDofSettings {
    fn default() -> Self {
        Self {
            focus_object: NIL,
            focus_subtarget: [0; 64],
            focus_distance: 10.0,
            aperture_fstop: 2.8,
            aperture_rotation: 0.0,
            aperture_ratio: 1.0,
            aperture_blades: 0,
            flag: 0,
        }
    }
}

/// `Camera_Runtime`.
#[derive(Clone, Default)]
pub struct CameraRuntime {
    /// `float drw_corners[2][4][2]`.
    pub drw_corners: [[[f32; 2]; 4]; 2],
    /// `float drw_tria[2][2]`.
    pub drw_tria: [[f32; 2]; 2],
    pub drw_depth: [f32; 2],
    pub drw_focusmat: [[f32; 4]; 4],
    pub drw_normalmat: [[f32; 4]; 4],
}

/// `Camera`.
#[derive(Clone)]
pub struct Camera {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    pub type_: i8,
    pub composition_guide_flags: i16,
    pub flag: i16,
    pub passepartalpha: f32,
    pub clip_start: f32,
    pub clip_end: f32,
    pub lens: f32,
    pub ortho_scale: f32,
    pub drawsize: f32,
    pub sensor_x: f32,
    pub sensor_y: f32,
    pub shiftx: f32,
    pub shifty: f32,
    /// DNA_DEPRECATED `dof_distance`.
    pub dof_distance: f32,
    pub sensor_fit: i8,
    pub panorama_type: i8,
    pub fisheye_fov: f32,
    pub fisheye_lens: f32,
    pub latitude_min: f32,
    pub latitude_max: f32,
    pub longitude_min: f32,
    pub longitude_max: f32,
    pub fisheye_polynomial_k0: f32,
    pub fisheye_polynomial_k1: f32,
    pub fisheye_polynomial_k2: f32,
    pub fisheye_polynomial_k3: f32,
    pub fisheye_polynomial_k4: f32,
    pub central_cylindrical_range_u_min: f32,
    pub central_cylindrical_range_u_max: f32,
    pub central_cylindrical_range_v_min: f32,
    pub central_cylindrical_range_v_max: f32,
    pub central_cylindrical_radius: f32,
    /// `Text *custom_shader`.
    pub custom_shader: usize,
    /// `char custom_filepath[1024]`.
    pub custom_filepath: [u8; 1024],
    /// `char custom_bytecode_hash[64]`.
    pub custom_bytecode_hash: [u8; 64],
    /// `char *custom_bytecode` → Vec.
    pub custom_bytecode: Vec<u8>,
    pub custom_mode: i32,
    /// DNA_DEPRECATED `Object *dof_ob`.
    pub dof_ob: usize,
    /// DNA_DEPRECATED `GPUDOFSettings gpu_dof`.
    pub gpu_dof: GpuDofSettings,
    pub dof: CameraDofSettings,
    /// `ListBaseT<CameraBGImage>`.
    pub bg_images: ListBase,
    pub stereo: CameraStereoSettings,
    pub composition_guide_color: [f32; 4],
    pub runtime: CameraRuntime,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            type_: 0,
            composition_guide_flags: 0,
            flag: CAM_SHOWPASSEPARTOUT,
            passepartalpha: 0.5,
            clip_start: 0.1,
            clip_end: 1000.0,
            lens: 50.0,
            ortho_scale: 6.0,
            drawsize: 1.0,
            sensor_x: DEFAULT_SENSOR_WIDTH,
            sensor_y: DEFAULT_SENSOR_HEIGHT,
            shiftx: 0.0,
            shifty: 0.0,
            dof_distance: 0.0,
            sensor_fit: 0,
            panorama_type: CAM_PANORAMA_FISHEYE_EQUISOLID,
            fisheye_fov: std::f32::consts::PI,
            fisheye_lens: 10.5,
            latitude_min: -0.5 * std::f32::consts::PI,
            latitude_max: 0.5 * std::f32::consts::PI,
            longitude_min: -std::f32::consts::PI,
            longitude_max: std::f32::consts::PI,
            fisheye_polynomial_k0: -1.1735143712967577e-05,
            fisheye_polynomial_k1: -0.019988736953434998,
            fisheye_polynomial_k2: -3.3525322965709175e-06,
            fisheye_polynomial_k3: 3.099275275886036e-06,
            fisheye_polynomial_k4: -2.6064646454854524e-08,
            central_cylindrical_range_u_min: DEG2RADF(-180.0),
            central_cylindrical_range_u_max: DEG2RADF(180.0),
            central_cylindrical_range_v_min: -1.0,
            central_cylindrical_range_v_max: 1.0,
            central_cylindrical_radius: 1.0,
            custom_shader: NIL,
            custom_filepath: [0; 1024],
            custom_bytecode_hash: [0; 64],
            custom_bytecode: Vec::new(),
            custom_mode: 0,
            dof_ob: NIL,
            gpu_dof: GpuDofSettings::default(),
            dof: CameraDofSettings::default(),
            bg_images: ListBase::new(),
            stereo: CameraStereoSettings::default(),
            composition_guide_color: [0.5, 0.5, 0.5, 1.0],
            runtime: CameraRuntime::default(),
        }
    }
}
