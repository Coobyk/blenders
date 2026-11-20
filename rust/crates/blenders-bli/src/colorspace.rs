//! Port of `BLI_colorspace.cc` — `blender::colorspace` conversion matrices.
//! Values are set at runtime by OCIO in C; these are the identity defaults.

/// `colorspace::luma_coefficients` (Rec.709 luma).
pub const LUMA_COEFFICIENTS: [f32; 3] = [0.2126, 0.7152, 0.0722];

/// `colorspace::scene_linear_to_rec709` — identity default.
pub static SCENE_LINEAR_TO_REC709: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
/// `colorspace::rec709_to_scene_linear`.
pub static REC709_TO_SCENE_LINEAR: [[f32; 3]; 3] = SCENE_LINEAR_TO_REC709;
/// `colorspace::scene_linear_is_rec709`.
pub static SCENE_LINEAR_IS_REC709: bool = true;
/// `colorspace::scene_linear_to_xyz` — zero until OCIO sets it.
pub static SCENE_LINEAR_TO_XYZ: [[f32; 3]; 3] = [[0.0; 3]; 3];
/// `colorspace::xyz_to_scene_linear`.
pub static XYZ_TO_SCENE_LINEAR: [[f32; 3]; 3] = [[0.0; 3]; 3];
/// `colorspace::scene_linear_to_rec2020`.
pub static SCENE_LINEAR_TO_REC2020: [[f32; 3]; 3] = [[0.0; 3]; 3];
/// `colorspace::rec2020_to_scene_linear`.
pub static REC2020_TO_SCENE_LINEAR: [[f32; 3]; 3] = [[0.0; 3]; 3];
/// `colorspace::scene_linear_to_aces`.
pub static SCENE_LINEAR_TO_ACES: [[f32; 3]; 3] = [[0.0; 3]; 3];
/// `colorspace::aces_to_scene_linear`.
pub static ACES_TO_SCENE_LINEAR: [[f32; 3]; 3] = [[0.0; 3]; 3];
/// `colorspace::scene_linear_to_acescg`.
pub static SCENE_LINEAR_TO_ACESCG: [[f32; 3]; 3] = [[0.0; 3]; 3];
/// `colorspace::acescg_to_scene_linear`.
pub static ACESCG_TO_SCENE_LINEAR: [[f32; 3]; 3] = [[0.0; 3]; 3];
