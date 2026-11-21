//! Port of `DNA_scene_enums.h` — mechanically converted enum constants.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
/// `eVGroupSelect`.
pub type Vgroupselect = i8;
pub const WT_VGROUP_ALL: Vgroupselect = 0;
pub const WT_VGROUP_ACTIVE: Vgroupselect = 1;
pub const WT_VGROUP_BONE_SELECT: Vgroupselect = 2;
pub const WT_VGROUP_BONE_DEFORM: Vgroupselect = 3;
pub const WT_VGROUP_BONE_DEFORM_OFF: Vgroupselect = 4;

/// `eSeqImageFitMethod`.
pub type Seqimagefitmethod = i32;
pub const SEQ_SCALE_TO_FIT: Seqimagefitmethod = 0;
pub const SEQ_SCALE_TO_FILL: Seqimagefitmethod = SEQ_SCALE_TO_FIT + 1;
pub const SEQ_STRETCH_TO_FILL: Seqimagefitmethod = SEQ_SCALE_TO_FILL + 1;
pub const SEQ_USE_ORIGINAL_SIZE: Seqimagefitmethod = SEQ_STRETCH_TO_FILL + 1;

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

/// `eUnifiedPaintSettingsFlags`.
pub type Unifiedpaintsettingsflags = i32;
pub const UNIFIED_PAINT_SIZE_DEPRECATED: Unifiedpaintsettingsflags = 1 << 0;
pub const UNIFIED_PAINT_ALPHA_DEPRECATED: Unifiedpaintsettingsflags = 1 << 1;
pub const UNIFIED_PAINT_BRUSH_LOCK_SIZE: Unifiedpaintsettingsflags = 1 << 2;
pub const UNIFIED_PAINT_FLAG_UNUSED_0: Unifiedpaintsettingsflags = 1 << 3;
pub const UNIFIED_PAINT_FLAG_UNUSED_1: Unifiedpaintsettingsflags = 1 << 4;
pub const UNIFIED_PAINT_WEIGHT_DEPRECATED: Unifiedpaintsettingsflags = 1 << 5;
pub const UNIFIED_PAINT_COLOR_DEPRECATED: Unifiedpaintsettingsflags = 1 << 6;
pub const UNIFIED_PAINT_INPUT_SAMPLES_DEPRECATED: Unifiedpaintsettingsflags = 1 << 7;
pub const UNIFIED_PAINT_COLOR_JITTER: Unifiedpaintsettingsflags = 1 << 8;

/// `PaintCurveVisibilityFlags`.
pub type Paintcurvevisibilityflags = i32;
pub const PAINT_CURVE_SHOW_STRENGTH: Paintcurvevisibilityflags = 1 << 0;
pub const PAINT_CURVE_SHOW_SIZE: Paintcurvevisibilityflags = 1 << 1;
pub const PAINT_CURVE_SHOW_JITTER: Paintcurvevisibilityflags = 1 << 2;
pub const PAINT_CURVE_SHOW_HARDNESS: Paintcurvevisibilityflags = 1 << 3;
pub const PAINT_CURVE_SHOW_AUTO_SMOOTH: Paintcurvevisibilityflags = 1 << 4;
pub const PAINT_CURVE_SHOW_SPACING: Paintcurvevisibilityflags = 1 << 5;

/// `eScenePassType`.
pub type Scenepasstype = u32;
pub const SCE_PASS_COMBINED: Scenepasstype = 1 << 0;
pub const SCE_PASS_DEPTH: Scenepasstype = 1 << 1;
pub const SCE_PASS_UNUSED_1: Scenepasstype = 1 << 2;
pub const SCE_PASS_UNUSED_2: Scenepasstype = 1 << 3;
pub const SCE_PASS_UNUSED_3: Scenepasstype = 1 << 4;
pub const SCE_PASS_SHADOW: Scenepasstype = 1 << 5;
pub const SCE_PASS_AO: Scenepasstype = 1 << 6;
pub const SCE_PASS_POSITION: Scenepasstype = 1 << 7;
pub const SCE_PASS_NORMAL: Scenepasstype = 1 << 8;
pub const SCE_PASS_VECTOR: Scenepasstype = 1 << 9;
pub const SCE_PASS_UNUSED_5: Scenepasstype = 1 << 10;
pub const SCE_PASS_INDEXOB: Scenepasstype = 1 << 11;
pub const SCE_PASS_UV: Scenepasstype = 1 << 12;
pub const SCE_PASS_UNUSED_6: Scenepasstype = 1 << 13;
pub const SCE_PASS_MIST: Scenepasstype = 1 << 14;
pub const SCE_PASS_UNUSED_7: Scenepasstype = 1 << 15;
pub const SCE_PASS_EMIT: Scenepasstype = 1 << 16;
pub const SCE_PASS_ENVIRONMENT: Scenepasstype = 1 << 17;
pub const SCE_PASS_INDEXMA: Scenepasstype = 1 << 18;
pub const SCE_PASS_DIFFUSE_DIRECT: Scenepasstype = 1 << 19;
pub const SCE_PASS_DIFFUSE_INDIRECT: Scenepasstype = 1 << 20;
pub const SCE_PASS_DIFFUSE_COLOR: Scenepasstype = 1 << 21;
pub const SCE_PASS_GLOSSY_DIRECT: Scenepasstype = 1 << 22;
pub const SCE_PASS_GLOSSY_INDIRECT: Scenepasstype = 1 << 23;
pub const SCE_PASS_GLOSSY_COLOR: Scenepasstype = 1 << 24;
pub const SCE_PASS_TRANSM_DIRECT: Scenepasstype = 1 << 25;
pub const SCE_PASS_TRANSM_INDIRECT: Scenepasstype = 1 << 26;
pub const SCE_PASS_TRANSM_COLOR: Scenepasstype = 1 << 27;
pub const SCE_PASS_SUBSURFACE_DIRECT: Scenepasstype = 1 << 28;
pub const SCE_PASS_SUBSURFACE_INDIRECT: Scenepasstype = 1 << 29;
pub const SCE_PASS_SUBSURFACE_COLOR: Scenepasstype = 1 << 30;
pub const SCE_PASS_ROUGHNESS: Scenepasstype = 1 << 31;

/// `eSceneLayer_Flag`.
pub type ScenelayerFlag = i32;
pub const SCE_LAY_SOLID: ScenelayerFlag = 1 << 0;
pub const SCE_LAY_UNUSED_1: ScenelayerFlag = 1 << 1;
pub const SCE_LAY_UNUSED_2: ScenelayerFlag = 1 << 2;
pub const SCE_LAY_UNUSED_3: ScenelayerFlag = 1 << 3;
pub const SCE_LAY_SKY: ScenelayerFlag = 1 << 4;
pub const SCE_LAY_STRAND: ScenelayerFlag = 1 << 5;
pub const SCE_LAY_FRS: ScenelayerFlag = 1 << 6;
pub const SCE_LAY_AO: ScenelayerFlag = 1 << 7;
pub const SCE_LAY_VOLUMES: ScenelayerFlag = 1 << 8;
pub const SCE_LAY_MOTION_BLUR: ScenelayerFlag = 1 << 9;
pub const SCE_LAY_GREASE_PENCIL: ScenelayerFlag = 1 << 10;
pub const SCE_LAY_FLAG_DEFAULT: ScenelayerFlag = (1 << 15) - 1;
pub const SCE_LAY_UNUSED_4: ScenelayerFlag = 1 << 15;
pub const SCE_LAY_UNUSED_5: ScenelayerFlag = 1 << 16;
pub const SCE_LAY_DISABLE: ScenelayerFlag = 1 << 17;
pub const SCE_LAY_UNUSED_6: ScenelayerFlag = 1 << 18;
pub const SCE_LAY_UNUSED_7: ScenelayerFlag = 1 << 19;

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

