//! Port of `DNA_scene_enums.h` — mechanically converted enum constants.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
/// `ePaintSymmetryFlags`.
pub type Paintsymmetryflags = i32;
pub const PAINT_SYMM_NONE: Paintsymmetryflags = 0;
pub const PAINT_SYMM_X: Paintsymmetryflags = 1 << 0;
pub const PAINT_SYMM_Y: Paintsymmetryflags = 1 << 1;
pub const PAINT_SYMM_Z: Paintsymmetryflags = 1 << 2;
pub const PAINT_SYMMETRY_FEATHER: Paintsymmetryflags = 1 << 3;
pub const PAINT_TILE_X: Paintsymmetryflags = 1 << 4;
pub const PAINT_TILE_Y: Paintsymmetryflags = 1 << 5;
pub const PAINT_TILE_Z: Paintsymmetryflags = 1 << 6;

/// `PaintCurveVisibilityFlags`.
pub type Paintcurvevisibilityflags = i32;
pub const PAINT_CURVE_SHOW_STRENGTH: Paintcurvevisibilityflags = 1 << 0;
pub const PAINT_CURVE_SHOW_SIZE: Paintcurvevisibilityflags = 1 << 1;
pub const PAINT_CURVE_SHOW_JITTER: Paintcurvevisibilityflags = 1 << 2;
pub const PAINT_CURVE_SHOW_HARDNESS: Paintcurvevisibilityflags = 1 << 3;
pub const PAINT_CURVE_SHOW_AUTO_SMOOTH: Paintcurvevisibilityflags = 1 << 4;
pub const PAINT_CURVE_SHOW_SPACING: Paintcurvevisibilityflags = 1 << 5;

/// `eSceneView_Flag`.
pub type SceneviewFlag = i32;
pub const SCE_VIEW_DISABLE: SceneviewFlag = 1 << 0;

/// `eSceneViews_Format`.
pub type SceneviewsFormat = i8;
pub const SCE_VIEWS_FORMAT_STEREO_3D: SceneviewsFormat = 0;
pub const SCE_VIEWS_FORMAT_MULTIVIEW: SceneviewsFormat = 1;

/// `eImageFormat_ViewsFormat`.
pub type ImageformatViewsformat = i8;
pub const R_IMF_VIEWS_INDIVIDUAL: ImageformatViewsformat = 0;
pub const R_IMF_VIEWS_STEREO_3D: ImageformatViewsformat = 1;
pub const R_IMF_VIEWS_MULTIVIEW: ImageformatViewsformat = 2;

/// `eStereoDisplayMode`.
pub type Stereodisplaymode = i8;
pub const S3D_DISPLAY_ANAGLYPH: Stereodisplaymode = 0;
pub const S3D_DISPLAY_INTERLACE: Stereodisplaymode = 1;
pub const S3D_DISPLAY_PAGEFLIP: Stereodisplaymode = 2;
pub const S3D_DISPLAY_SIDEBYSIDE: Stereodisplaymode = 3;
pub const S3D_DISPLAY_TOPBOTTOM: Stereodisplaymode = 4;

/// `eStereo3dFlag`.
pub type Stereo3dflag = i16;
pub const S3D_INTERLACE_SWAP: Stereo3dflag = 1 << 0;
pub const S3D_SIDEBYSIDE_CROSSEYED: Stereo3dflag = 1 << 1;
pub const S3D_SQUEEZED_FRAME: Stereo3dflag = 1 << 2;

/// `eStereo3dAnaglyphType`.
pub type Stereo3danaglyphtype = i8;
pub const S3D_ANAGLYPH_REDCYAN: Stereo3danaglyphtype = 0;
pub const S3D_ANAGLYPH_GREENMAGENTA: Stereo3danaglyphtype = 1;
pub const S3D_ANAGLYPH_YELLOWBLUE: Stereo3danaglyphtype = 2;

/// `eStereo3dInterlaceType`.
pub type Stereo3dinterlacetype = i8;
pub const S3D_INTERLACE_ROW: Stereo3dinterlacetype = 0;
pub const S3D_INTERLACE_COLUMN: Stereo3dinterlacetype = 1;
pub const S3D_INTERLACE_CHECKERBOARD: Stereo3dinterlacetype = 2;

/// `eStereoViews`.
pub type Stereoviews = i8;
pub const STEREO_LEFT_ID: Stereoviews = 0;
pub const STEREO_RIGHT_ID: Stereoviews = 1;
pub const STEREO_3D_ID: Stereoviews = 2;
pub const STEREO_MONO_ID: Stereoviews = 3;

