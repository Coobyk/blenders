//! Port of `DNA_brush_enums.h` — mechanically converted enum constants.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
/// `eGPDbrush_Flag`.
pub type GpdbrushFlag = i32;
pub const GP_BRUSH_USE_PRESSURE: GpdbrushFlag = 1 << 0;
pub const GP_BRUSH_USE_STRENGTH_PRESSURE: GpdbrushFlag = 1 << 1;
pub const GP_BRUSH_USE_JITTER_PRESSURE: GpdbrushFlag = 1 << 2;
pub const GP_BRUSH_FILL_FIT_DISABLE: GpdbrushFlag = 1 << 3;
pub const GP_BRUSH_FILL_SHOW_EXTENDLINES: GpdbrushFlag = 1 << 4;
pub const GP_BRUSH_FILL_HIDE: GpdbrushFlag = 1 << 6;
pub const GP_BRUSH_FILL_SHOW_HELPLINES: GpdbrushFlag = 1 << 7;
pub const GP_BRUSH_STABILIZE_MOUSE: GpdbrushFlag = 1 << 8;
pub const GP_BRUSH_STABILIZE_MOUSE_TEMP: GpdbrushFlag = 1 << 9;
pub const GP_BRUSH_UNUSED_1: GpdbrushFlag = 1 << 10;
pub const GP_BRUSH_GROUP_SETTINGS: GpdbrushFlag = 1 << 11;
pub const GP_BRUSH_GROUP_RANDOM: GpdbrushFlag = 1 << 12;
pub const GP_BRUSH_MATERIAL_PINNED: GpdbrushFlag = 1 << 13;
pub const GP_BRUSH_DISSABLE_LASSO: GpdbrushFlag = 1 << 14;
pub const GP_BRUSH_OCCLUDE_ERASER: GpdbrushFlag = 1 << 15;
pub const GP_BRUSH_TRIM_STROKE: GpdbrushFlag = 1 << 16;
pub const GP_BRUSH_OUTLINE_STROKE: GpdbrushFlag = 1 << 17;
pub const GP_BRUSH_FILL_STROKE_COLLIDE: GpdbrushFlag = 1 << 18;
pub const GP_BRUSH_ERASER_KEEP_CAPS: GpdbrushFlag = 1 << 19;
pub const GP_BRUSH_ACTIVE_LAYER_ONLY: GpdbrushFlag = 1 << 20;
pub const GP_BRUSH_FILL_AUTO_REMOVE_FILL_GUIDES: GpdbrushFlag = 1 << 21;
pub const GP_BRUSH_FILL_INTERNAL_GAPS: GpdbrushFlag = 1 << 22;
pub const GP_BRUSH_USE_CYCLIC_STROKE: GpdbrushFlag = 1 << 23;

/// `eGPDbrush_Flag2`.
pub type GpdbrushFlag2 = i32;
pub const GP_BRUSH_UNUSED_2: GpdbrushFlag2 = 1 << 0;
pub const GP_BRUSH_UNUSED_3: GpdbrushFlag2 = 1 << 1;
pub const GP_BRUSH_UNUSED_4: GpdbrushFlag2 = 1 << 2;
pub const GP_BRUSH_USE_PRESS_AT_STROKE: GpdbrushFlag2 = 1 << 3;
pub const GP_BRUSH_USE_STRENGTH_AT_STROKE: GpdbrushFlag2 = 1 << 4;
pub const GP_BRUSH_USE_UV_AT_STROKE: GpdbrushFlag2 = 1 << 5;
pub const GP_BRUSH_UNUSED_5: GpdbrushFlag2 = 1 << 6;
pub const GP_BRUSH_UNUSED_6: GpdbrushFlag2 = 1 << 7;
pub const GP_BRUSH_UNUSED_7: GpdbrushFlag2 = 1 << 8;
pub const GP_BRUSH_USE_PRESSURE_RAND_PRESS: GpdbrushFlag2 = 1 << 9;
pub const GP_BRUSH_USE_STRENGTH_RAND_PRESS: GpdbrushFlag2 = 1 << 10;
pub const GP_BRUSH_USE_UV_RAND_PRESS: GpdbrushFlag2 = 1 << 11;
pub const GP_BRUSH_USE_STROKE: GpdbrushFlag2 = 1 << 12;
pub const GP_BRUSH_USE_FILL: GpdbrushFlag2 = 1 << 13;

/// `eGP_FillDrawModes`.
pub type GpFilldrawmodes = i32;
pub const GP_FILL_DMODE_BOTH: GpFilldrawmodes = 0;
pub const GP_FILL_DMODE_STROKE: GpFilldrawmodes = 1;
pub const GP_FILL_DMODE_CONTROL: GpFilldrawmodes = 2;

/// `eGP_FillExtendModes`.
pub type GpFillextendmodes = i32;
pub const GP_FILL_EMODE_EXTEND: GpFillextendmodes = 0;
pub const GP_FILL_EMODE_RADIUS: GpFillextendmodes = 1;

/// `eGP_FillLayerModes`.
pub type GpFilllayermodes = i16;
pub const GP_FILL_GPLMODE_VISIBLE: GpFilllayermodes = 0;
pub const GP_FILL_GPLMODE_ACTIVE: GpFilllayermodes = 1;
pub const GP_FILL_GPLMODE_ALL_ABOVE: GpFilllayermodes = 2;
pub const GP_FILL_GPLMODE_ALL_BELOW: GpFilllayermodes = 3;
pub const GP_FILL_GPLMODE_ABOVE: GpFilllayermodes = 4;
pub const GP_FILL_GPLMODE_BELOW: GpFilllayermodes = 5;

/// `eGP_FillSolver`.
pub type GpFillsolver = i32;
pub const GP_FILL_SOLVER_DELAUNAY: GpFillsolver = 0;
pub const GP_FILL_SOLVER_PIXEL: GpFillsolver = 1;

/// `eGP_BrushEraserMode`.
pub type GpBrusherasermode = i32;
pub const GP_BRUSH_ERASER_SOFT: GpBrusherasermode = 0;
pub const GP_BRUSH_ERASER_HARD: GpBrusherasermode = 1;
pub const GP_BRUSH_ERASER_STROKE: GpBrusherasermode = 2;

/// `eGP_BrushMode`.
pub type GpBrushmode = i16;
pub const GP_BRUSH_MODE_ACTIVE: GpBrushmode = 0;
pub const GP_BRUSH_MODE_MATERIAL: GpBrushmode = 1;
pub const GP_BRUSH_MODE_VERTEXCOLOR: GpBrushmode = 2;

/// `eBrushCurvePreset`.
pub type Brushcurvepreset = i32;
pub const BRUSH_CURVE_CUSTOM: Brushcurvepreset = 0;
pub const BRUSH_CURVE_SMOOTH: Brushcurvepreset = 1;
pub const BRUSH_CURVE_SPHERE: Brushcurvepreset = 2;
pub const BRUSH_CURVE_ROOT: Brushcurvepreset = 3;
pub const BRUSH_CURVE_SHARP: Brushcurvepreset = 4;
pub const BRUSH_CURVE_LIN: Brushcurvepreset = 5;
pub const BRUSH_CURVE_POW4: Brushcurvepreset = 6;
pub const BRUSH_CURVE_INVSQUARE: Brushcurvepreset = 7;
pub const BRUSH_CURVE_CONSTANT: Brushcurvepreset = 8;
pub const BRUSH_CURVE_SMOOTHER: Brushcurvepreset = 9;

/// `eBrushDeformTarget`.
pub type Brushdeformtarget = i32;
pub const BRUSH_DEFORM_TARGET_GEOMETRY: Brushdeformtarget = 0;
pub const BRUSH_DEFORM_TARGET_CLOTH_SIM: Brushdeformtarget = 1;

/// `eBrushElasticDeformType`.
pub type Brushelasticdeformtype = i32;
pub const BRUSH_ELASTIC_DEFORM_GRAB: Brushelasticdeformtype = 0;
pub const BRUSH_ELASTIC_DEFORM_GRAB_BISCALE: Brushelasticdeformtype = 1;
pub const BRUSH_ELASTIC_DEFORM_GRAB_TRISCALE: Brushelasticdeformtype = 2;
pub const BRUSH_ELASTIC_DEFORM_SCALE: Brushelasticdeformtype = 3;
pub const BRUSH_ELASTIC_DEFORM_TWIST: Brushelasticdeformtype = 4;

/// `eBrushClothDeformType`.
pub type Brushclothdeformtype = i32;
pub const BRUSH_CLOTH_DEFORM_DRAG: Brushclothdeformtype = 0;
pub const BRUSH_CLOTH_DEFORM_PUSH: Brushclothdeformtype = 1;
pub const BRUSH_CLOTH_DEFORM_GRAB: Brushclothdeformtype = 2;
pub const BRUSH_CLOTH_DEFORM_PINCH_POINT: Brushclothdeformtype = 3;
pub const BRUSH_CLOTH_DEFORM_PINCH_PERPENDICULAR: Brushclothdeformtype = 4;
pub const BRUSH_CLOTH_DEFORM_INFLATE: Brushclothdeformtype = 5;
pub const BRUSH_CLOTH_DEFORM_EXPAND: Brushclothdeformtype = 6;
pub const BRUSH_CLOTH_DEFORM_SNAKE_HOOK: Brushclothdeformtype = 7;

/// `eBrushSmoothDeformType`.
pub type Brushsmoothdeformtype = i32;
pub const BRUSH_SMOOTH_DEFORM_LAPLACIAN: Brushsmoothdeformtype = 0;
pub const BRUSH_SMOOTH_DEFORM_SURFACE: Brushsmoothdeformtype = 1;

/// `eBrushClothForceFalloffType`.
pub type Brushclothforcefallofftype = i32;
pub const BRUSH_CLOTH_FORCE_FALLOFF_RADIAL: Brushclothforcefallofftype = 0;
pub const BRUSH_CLOTH_FORCE_FALLOFF_PLANE: Brushclothforcefallofftype = 1;

/// `eBrushClothSimulationAreaType`.
pub type Brushclothsimulationareatype = i32;
pub const BRUSH_CLOTH_SIMULATION_AREA_LOCAL: Brushclothsimulationareatype = 0;
pub const BRUSH_CLOTH_SIMULATION_AREA_GLOBAL: Brushclothsimulationareatype = 1;
pub const BRUSH_CLOTH_SIMULATION_AREA_DYNAMIC: Brushclothsimulationareatype = 2;

/// `eBrushPoseDeformType`.
pub type Brushposedeformtype = i32;
pub const BRUSH_POSE_DEFORM_ROTATE_TWIST: Brushposedeformtype = 0;
pub const BRUSH_POSE_DEFORM_SCALE_TRANSLATE: Brushposedeformtype = 1;
pub const BRUSH_POSE_DEFORM_SQUASH_STRETCH: Brushposedeformtype = 2;

/// `eBrushPoseOriginType`.
pub type Brushposeorigintype = i32;
pub const BRUSH_POSE_ORIGIN_TOPOLOGY: Brushposeorigintype = 0;
pub const BRUSH_POSE_ORIGIN_FACE_SETS: Brushposeorigintype = 1;
pub const BRUSH_POSE_ORIGIN_FACE_SETS_FK: Brushposeorigintype = 2;

/// `eBrushSmearDeformType`.
pub type Brushsmeardeformtype = i32;
pub const BRUSH_SMEAR_DEFORM_DRAG: Brushsmeardeformtype = 0;
pub const BRUSH_SMEAR_DEFORM_PINCH: Brushsmeardeformtype = 1;
pub const BRUSH_SMEAR_DEFORM_EXPAND: Brushsmeardeformtype = 2;

/// `eBrushSlideDeformType`.
pub type Brushslidedeformtype = i32;
pub const BRUSH_SLIDE_DEFORM_DRAG: Brushslidedeformtype = 0;
pub const BRUSH_SLIDE_DEFORM_PINCH: Brushslidedeformtype = 1;
pub const BRUSH_SLIDE_DEFORM_EXPAND: Brushslidedeformtype = 2;

/// `eBrushBoundaryDeformType`.
pub type Brushboundarydeformtype = i32;
pub const BRUSH_BOUNDARY_DEFORM_BEND: Brushboundarydeformtype = 0;
pub const BRUSH_BOUNDARY_DEFORM_EXPAND: Brushboundarydeformtype = 1;
pub const BRUSH_BOUNDARY_DEFORM_INFLATE: Brushboundarydeformtype = 2;
pub const BRUSH_BOUNDARY_DEFORM_GRAB: Brushboundarydeformtype = 3;
pub const BRUSH_BOUNDARY_DEFORM_TWIST: Brushboundarydeformtype = 4;
pub const BRUSH_BOUNDARY_DEFORM_SMOOTH: Brushboundarydeformtype = 5;

/// `eBrushBoundaryFalloffType`.
pub type Brushboundaryfallofftype = i32;
pub const BRUSH_BOUNDARY_FALLOFF_CONSTANT: Brushboundaryfallofftype = 0;
pub const BRUSH_BOUNDARY_FALLOFF_RADIUS: Brushboundaryfallofftype = 1;
pub const BRUSH_BOUNDARY_FALLOFF_LOOP: Brushboundaryfallofftype = 2;
pub const BRUSH_BOUNDARY_FALLOFF_LOOP_INVERT: Brushboundaryfallofftype = 3;

/// `eBrushSnakeHookDeformType`.
pub type Brushsnakehookdeformtype = i32;
pub const BRUSH_SNAKE_HOOK_DEFORM_FALLOFF: Brushsnakehookdeformtype = 0;
pub const BRUSH_SNAKE_HOOK_DEFORM_ELASTIC: Brushsnakehookdeformtype = 1;

/// `eBrushPlaneInversionMode`.
pub type Brushplaneinversionmode = i32;
pub const BRUSH_PLANE_INVERT_DISPLACEMENT: Brushplaneinversionmode = 0;
pub const BRUSH_PLANE_SWAP_HEIGHT_AND_DEPTH: Brushplaneinversionmode = 1;

/// `eBrushProjectRayDirection`.
pub type Brushprojectraydirection = i8;
pub const BRUSH_PROJECT_RAY_DIRECTION_VIEW_NORMAL: Brushprojectraydirection = 0;
pub const BRUSH_PROJECT_RAY_DIRECTION_PLANE_NORMAL: Brushprojectraydirection = 1;

/// `eGp_Vertex_Mode`.
pub type GpVertexMode = i32;
pub const GPPAINT_MODE_STROKE: GpVertexMode = 0;
pub const GPPAINT_MODE_FILL: GpVertexMode = 1;
pub const GPPAINT_MODE_BOTH: GpVertexMode = 2;

/// `eGP_Sculpt_Flag`.
pub type GpSculptFlag = i32;
pub const GP_SCULPT_FLAG_INVERT: GpSculptFlag = 1 << 0;
pub const GP_SCULPT_FLAG_TMP_INVERT: GpSculptFlag = 1 << 3;

/// `eGP_Sculpt_Mode_Flag`.
pub type GpSculptModeFlag = i32;
pub const GP_SCULPT_FLAGMODE_APPLY_POSITION: GpSculptModeFlag = 1 << 0;
pub const GP_SCULPT_FLAGMODE_APPLY_STRENGTH: GpSculptModeFlag = 1 << 1;
pub const GP_SCULPT_FLAGMODE_APPLY_THICKNESS: GpSculptModeFlag = 1 << 2;
pub const GP_SCULPT_FLAGMODE_APPLY_UV: GpSculptModeFlag = 1 << 3;

/// `eAutomasking_flag`.
pub type AutomaskingFlag = i32;
pub const BRUSH_AUTOMASKING_TOPOLOGY: AutomaskingFlag = 1 << 0;
pub const BRUSH_AUTOMASKING_FACE_SETS: AutomaskingFlag = 1 << 1;
pub const BRUSH_AUTOMASKING_BOUNDARY_EDGES: AutomaskingFlag = 1 << 2;
pub const BRUSH_AUTOMASKING_BOUNDARY_FACE_SETS: AutomaskingFlag = 1 << 3;
pub const BRUSH_AUTOMASKING_CAVITY_NORMAL: AutomaskingFlag = 1 << 4;
pub const BRUSH_AUTOMASKING_CAVITY_INVERTED: AutomaskingFlag = 1 << 5;
pub const BRUSH_AUTOMASKING_CAVITY_ALL: AutomaskingFlag = (1 << 4) | (1 << 5);
pub const BRUSH_AUTOMASKING_CAVITY_USE_CURVE: AutomaskingFlag = 1 << 6;
pub const BRUSH_AUTOMASKING_BRUSH_NORMAL: AutomaskingFlag = 1 << 8;
pub const BRUSH_AUTOMASKING_VIEW_NORMAL: AutomaskingFlag = 1 << 9;
pub const BRUSH_AUTOMASKING_VIEW_OCCLUSION: AutomaskingFlag = 1 << 10;

/// `ePaintBrush_flag`.
pub type PaintbrushFlag = i32;
pub const BRUSH_PAINT_UNUSED_1: PaintbrushFlag = 1 << 0;
pub const BRUSH_PAINT_UNUSED_2: PaintbrushFlag = 1 << 1;
pub const BRUSH_PAINT_FLOW_PRESSURE: PaintbrushFlag = 1 << 2;
pub const BRUSH_PAINT_FLOW_PRESSURE_INVERT: PaintbrushFlag = 1 << 3;
pub const BRUSH_PAINT_WET_MIX_PRESSURE: PaintbrushFlag = 1 << 4;
pub const BRUSH_PAINT_WET_MIX_PRESSURE_INVERT: PaintbrushFlag = 1 << 5;
pub const BRUSH_PAINT_WET_PERSISTENCE_PRESSURE: PaintbrushFlag = 1 << 6;
pub const BRUSH_PAINT_WET_PERSISTENCE_PRESSURE_INVERT: PaintbrushFlag = 1 << 7;
pub const BRUSH_PAINT_DENSITY_PRESSURE: PaintbrushFlag = 1 << 8;
pub const BRUSH_PAINT_DENSITY_PRESSURE_INVERT: PaintbrushFlag = 1 << 9;

/// `eBrushGradientSourceStroke`.
pub type Brushgradientsourcestroke = i8;
pub const BRUSH_GRADIENT_PRESSURE: Brushgradientsourcestroke = 0;
pub const BRUSH_GRADIENT_SPACING_REPEAT: Brushgradientsourcestroke = 1;
pub const BRUSH_GRADIENT_SPACING_CLAMP: Brushgradientsourcestroke = 2;

/// `eBrushGradientSourceFill`.
pub type Brushgradientsourcefill = i8;
pub const BRUSH_GRADIENT_LINEAR: Brushgradientsourcefill = 0;
pub const BRUSH_GRADIENT_RADIAL: Brushgradientsourcefill = 1;

/// `eBrushStrokeType`.
pub type Brushstroketype = i8;
pub const BRUSH_STROKE_DOTS: Brushstroketype = 0;
pub const BRUSH_STROKE_AIRBRUSH: Brushstroketype = 1;
pub const BRUSH_STROKE_ANCHORED: Brushstroketype = 2;
pub const BRUSH_STROKE_SPACE: Brushstroketype = 3;
pub const BRUSH_STROKE_DRAG_DOT: Brushstroketype = 4;
pub const BRUSH_STROKE_LINE: Brushstroketype = 5;
pub const BRUSH_STROKE_CURVE: Brushstroketype = 6;

/// `eBrushFlags`.
pub type Brushflags = u32;
pub const BRUSH_HARDNESS_PRESSURE: Brushflags = 1 << 0;
pub const BRUSH_INVERT_TO_SCRAPE_FILL: Brushflags = 1 << 1;
pub const BRUSH_ALPHA_PRESSURE: Brushflags = 1 << 2;
pub const BRUSH_SIZE_PRESSURE: Brushflags = 1 << 3;
pub const BRUSH_JITTER_PRESSURE: Brushflags = 1 << 4;
pub const BRUSH_SPACING_PRESSURE: Brushflags = 1 << 5;
pub const BRUSH_ORIGINAL_PLANE: Brushflags = 1 << 6;
pub const BRUSH_GRAB_ACTIVE_VERTEX: Brushflags = 1 << 7;
pub const BRUSH_UNUSED_2: Brushflags = 1 << 8;
pub const BRUSH_DIR_IN: Brushflags = 1 << 9;
pub const BRUSH_UNUSED_3: Brushflags = 1 << 10;
pub const BRUSH_SMOOTH_STROKE: Brushflags = 1 << 11;
pub const BRUSH_PERSISTENT: Brushflags = 1 << 12;
pub const BRUSH_ACCUMULATE: Brushflags = 1 << 13;
pub const BRUSH_LOCK_ALPHA: Brushflags = 1 << 14;
pub const BRUSH_ORIGINAL_NORMAL: Brushflags = 1 << 15;
pub const BRUSH_OFFSET_PRESSURE: Brushflags = 1 << 16;
pub const BRUSH_SCENE_SPACING: Brushflags = 1 << 17;
pub const BRUSH_SPACE_ATTEN: Brushflags = 1 << 18;
pub const BRUSH_ADAPTIVE_SPACE: Brushflags = 1 << 19;
pub const BRUSH_LOCK_SIZE: Brushflags = 1 << 20;
pub const BRUSH_USE_GRADIENT: Brushflags = 1 << 21;
pub const BRUSH_EDGE_TO_EDGE: Brushflags = 1 << 22;
pub const BRUSH_UNUSED_4: Brushflags = 1 << 23;
pub const BRUSH_SMOOTH_PRESSURE: Brushflags = 1 << 24;
pub const BRUSH_UNUSED_7: Brushflags = 1 << 25;
pub const BRUSH_PLANE_TRIM: Brushflags = 1 << 26;
pub const BRUSH_FRONTFACE: Brushflags = 1 << 27;
pub const BRUSH_UNUSED_8: Brushflags = 1 << 28;
pub const BRUSH_UNUSED_5: Brushflags = 1 << 29;
pub const BRUSH_ABSOLUTE_JITTER: Brushflags = 1 << 30;
pub const BRUSH_UNUSED_6: Brushflags = 1 << 31;

/// `eBrushSamplingFlags`.
pub type Brushsamplingflags = i32;
pub const BRUSH_PAINT_ANTIALIASING: Brushsamplingflags = 1 << 0;

/// `eBrushFlags2`.
pub type Brushflags2 = i32;
pub const BRUSH_MULTIPLANE_SCRAPE_DYNAMIC: Brushflags2 = 1 << 0;
pub const BRUSH_MULTIPLANE_SCRAPE_PLANES_PREVIEW: Brushflags2 = 1 << 1;
pub const BRUSH_POSE_IK_ANCHORED: Brushflags2 = 1 << 2;
pub const BRUSH_USE_CONNECTED_ONLY: Brushflags2 = 1 << 3;
pub const BRUSH_CLOTH_PIN_SIMULATION_BOUNDARY: Brushflags2 = 1 << 4;
pub const BRUSH_POSE_USE_LOCK_ROTATION: Brushflags2 = 1 << 5;
pub const BRUSH_CLOTH_USE_COLLISION: Brushflags2 = 1 << 6;
pub const BRUSH_AREA_RADIUS_PRESSURE: Brushflags2 = 1 << 7;
pub const BRUSH_GRAB_SILHOUETTE: Brushflags2 = 1 << 8;
pub const BRUSH_USE_COLOR_AS_DISPLACEMENT: Brushflags2 = 1 << 9;
pub const BRUSH_JITTER_COLOR: Brushflags2 = 1 << 10;
pub const BRUSH_PROJECT_USE_BIDIRECTIONAL: Brushflags2 = 1 << 11;

/// `eBrushUnifiedPaintFlags`.
pub type Brushunifiedpaintflags = i32;
pub const BRUSH_USE_UNIFIED_PAINT_SIZE: Brushunifiedpaintflags = 1 << 0;
pub const BRUSH_USE_UNIFIED_PAINT_ALPHA: Brushunifiedpaintflags = 1 << 1;
pub const BRUSH_USE_UNIFIED_PAINT_WEIGHT: Brushunifiedpaintflags = 1 << 2;
pub const BRUSH_USE_UNIFIED_PAINT_COLOR: Brushunifiedpaintflags = 1 << 3;
pub const BRUSH_USE_UNIFIED_PAINT_INPUT_SAMPLES: Brushunifiedpaintflags = 1 << 4;

/// `BrushMaskPressureFlags`.
pub type Brushmaskpressureflags = i32;
pub const BRUSH_MASK_PRESSURE_RAMP: Brushmaskpressureflags = 1 << 1;
pub const BRUSH_MASK_PRESSURE_CUTOFF: Brushmaskpressureflags = 1 << 2;

/// `eOverlayFlags`.
pub type Overlayflags = i32;
pub const BRUSH_OVERLAY_CURSOR: Overlayflags = 1;
pub const BRUSH_OVERLAY_PRIMARY: Overlayflags = 1 << 1;
pub const BRUSH_OVERLAY_SECONDARY: Overlayflags = 1 << 2;
pub const BRUSH_OVERLAY_CURSOR_OVERRIDE_ON_STROKE: Overlayflags = 1 << 3;
pub const BRUSH_OVERLAY_PRIMARY_OVERRIDE_ON_STROKE: Overlayflags = 1 << 4;
pub const BRUSH_OVERLAY_SECONDARY_OVERRIDE_ON_STROKE: Overlayflags = 1 << 5;

/// `eBrushSculptType`.
pub type Brushsculpttype = i8;
pub const SCULPT_BRUSH_TYPE_DRAW: Brushsculpttype = 1;
pub const SCULPT_BRUSH_TYPE_SMOOTH: Brushsculpttype = 2;
pub const SCULPT_BRUSH_TYPE_PINCH: Brushsculpttype = 3;
pub const SCULPT_BRUSH_TYPE_INFLATE: Brushsculpttype = 4;
pub const SCULPT_BRUSH_TYPE_GRAB: Brushsculpttype = 5;
pub const SCULPT_BRUSH_TYPE_LAYER: Brushsculpttype = 6;
pub const SCULPT_BRUSH_TYPE_FLATTEN: Brushsculpttype = 7;
pub const SCULPT_BRUSH_TYPE_CLAY: Brushsculpttype = 8;
pub const SCULPT_BRUSH_TYPE_FILL: Brushsculpttype = 9;
pub const SCULPT_BRUSH_TYPE_SCRAPE: Brushsculpttype = 10;
pub const SCULPT_BRUSH_TYPE_NUDGE: Brushsculpttype = 11;
pub const SCULPT_BRUSH_TYPE_THUMB: Brushsculpttype = 12;
pub const SCULPT_BRUSH_TYPE_SNAKE_HOOK: Brushsculpttype = 13;
pub const SCULPT_BRUSH_TYPE_ROTATE: Brushsculpttype = 14;
pub const SCULPT_BRUSH_TYPE_SIMPLIFY: Brushsculpttype = 15;
pub const SCULPT_BRUSH_TYPE_CREASE: Brushsculpttype = 16;
pub const SCULPT_BRUSH_TYPE_BLOB: Brushsculpttype = 17;
pub const SCULPT_BRUSH_TYPE_CLAY_STRIPS: Brushsculpttype = 18;
pub const SCULPT_BRUSH_TYPE_MASK: Brushsculpttype = 19;
pub const SCULPT_BRUSH_TYPE_DRAW_SHARP: Brushsculpttype = 20;
pub const SCULPT_BRUSH_TYPE_ELASTIC_DEFORM: Brushsculpttype = 21;
pub const SCULPT_BRUSH_TYPE_POSE: Brushsculpttype = 22;
pub const SCULPT_BRUSH_TYPE_MULTIPLANE_SCRAPE: Brushsculpttype = 23;
pub const SCULPT_BRUSH_TYPE_SLIDE_RELAX: Brushsculpttype = 24;
pub const SCULPT_BRUSH_TYPE_CLAY_THUMB: Brushsculpttype = 25;
pub const SCULPT_BRUSH_TYPE_CLOTH: Brushsculpttype = 26;
pub const SCULPT_BRUSH_TYPE_DRAW_FACE_SETS: Brushsculpttype = 27;
pub const SCULPT_BRUSH_TYPE_PAINT: Brushsculpttype = 28;
pub const SCULPT_BRUSH_TYPE_SMEAR: Brushsculpttype = 29;
pub const SCULPT_BRUSH_TYPE_BOUNDARY: Brushsculpttype = 30;
pub const SCULPT_BRUSH_TYPE_DISPLACEMENT_ERASER: Brushsculpttype = 31;
pub const SCULPT_BRUSH_TYPE_DISPLACEMENT_SMEAR: Brushsculpttype = 32;
pub const SCULPT_BRUSH_TYPE_PLANE: Brushsculpttype = 33;
pub const SCULPT_BRUSH_TYPE_BLUR: Brushsculpttype = 34;
pub const SCULPT_BRUSH_TYPE_SCENE_PROJECT: Brushsculpttype = 35;

/// `eBrushCurvesSculptType`.
pub type Brushcurvessculpttype = i8;
pub const CURVES_SCULPT_BRUSH_TYPE_COMB: Brushcurvessculpttype = 0;
pub const CURVES_SCULPT_BRUSH_TYPE_DELETE: Brushcurvessculpttype = 1;
pub const CURVES_SCULPT_BRUSH_TYPE_SNAKE_HOOK: Brushcurvessculpttype = 2;
pub const CURVES_SCULPT_BRUSH_TYPE_ADD: Brushcurvessculpttype = 3;
pub const CURVES_SCULPT_BRUSH_TYPE_GROW_SHRINK: Brushcurvessculpttype = 4;
pub const CURVES_SCULPT_BRUSH_TYPE_SELECTION_PAINT: Brushcurvessculpttype = 5;
pub const CURVES_SCULPT_BRUSH_TYPE_PINCH: Brushcurvessculpttype = 6;
pub const CURVES_SCULPT_BRUSH_TYPE_SMOOTH: Brushcurvessculpttype = 7;
pub const CURVES_SCULPT_BRUSH_TYPE_PUFF: Brushcurvessculpttype = 8;
pub const CURVES_SCULPT_BRUSH_TYPE_DENSITY: Brushcurvessculpttype = 9;
pub const CURVES_SCULPT_BRUSH_TYPE_SLIDE: Brushcurvessculpttype = 10;
pub const CURVES_SCULPT_BRUSH_TYPE_CUT: Brushcurvessculpttype = 11;

/// `eBrushImagePaintType`.
pub type Brushimagepainttype = i8;
pub const IMAGE_PAINT_BRUSH_TYPE_DRAW: Brushimagepainttype = 0;
pub const IMAGE_PAINT_BRUSH_TYPE_SOFTEN: Brushimagepainttype = 1;
pub const IMAGE_PAINT_BRUSH_TYPE_SMEAR: Brushimagepainttype = 2;
pub const IMAGE_PAINT_BRUSH_TYPE_CLONE: Brushimagepainttype = 3;
pub const IMAGE_PAINT_BRUSH_TYPE_FILL: Brushimagepainttype = 4;
pub const IMAGE_PAINT_BRUSH_TYPE_MASK: Brushimagepainttype = 5;

/// `eBrushVertexPaintType`.
pub type Brushvertexpainttype = i8;
pub const VPAINT_BRUSH_TYPE_DRAW: Brushvertexpainttype = 0;
pub const VPAINT_BRUSH_TYPE_BLUR: Brushvertexpainttype = 1;
pub const VPAINT_BRUSH_TYPE_AVERAGE: Brushvertexpainttype = 2;
pub const VPAINT_BRUSH_TYPE_SMEAR: Brushvertexpainttype = 3;

/// `eBrushWeightPaintType`.
pub type Brushweightpainttype = i8;
pub const WPAINT_BRUSH_TYPE_DRAW: Brushweightpainttype = 0;
pub const WPAINT_BRUSH_TYPE_BLUR: Brushweightpainttype = 1;
pub const WPAINT_BRUSH_TYPE_AVERAGE: Brushweightpainttype = 2;
pub const WPAINT_BRUSH_TYPE_SMEAR: Brushweightpainttype = 3;

/// `eBrushGPaintType`.
pub type Brushgpainttype = i8;
pub const GPAINT_BRUSH_TYPE_DRAW: Brushgpainttype = 0;
pub const GPAINT_BRUSH_TYPE_FILL: Brushgpainttype = 1;
pub const GPAINT_BRUSH_TYPE_ERASE: Brushgpainttype = 2;
pub const GPAINT_BRUSH_TYPE_TINT: Brushgpainttype = 3;

/// `eBrushGPVertexType`.
pub type Brushgpvertextype = i8;
pub const GPVERTEX_BRUSH_TYPE_DRAW: Brushgpvertextype = 0;
pub const GPVERTEX_BRUSH_TYPE_BLUR: Brushgpvertextype = 1;
pub const GPVERTEX_BRUSH_TYPE_AVERAGE: Brushgpvertextype = 2;
pub const GPVERTEX_BRUSH_TYPE_TINT: Brushgpvertextype = 3;
pub const GPVERTEX_BRUSH_TYPE_SMEAR: Brushgpvertextype = 4;
pub const GPVERTEX_BRUSH_TYPE_REPLACE: Brushgpvertextype = 5;

/// `eBrushGPSculptType`.
pub type Brushgpsculpttype = i8;
pub const GPSCULPT_BRUSH_TYPE_SMOOTH: Brushgpsculpttype = 0;
pub const GPSCULPT_BRUSH_TYPE_THICKNESS: Brushgpsculpttype = 1;
pub const GPSCULPT_BRUSH_TYPE_STRENGTH: Brushgpsculpttype = 2;
pub const GPSCULPT_BRUSH_TYPE_GRAB: Brushgpsculpttype = 3;
pub const GPSCULPT_BRUSH_TYPE_PUSH: Brushgpsculpttype = 4;
pub const GPSCULPT_BRUSH_TYPE_TWIST: Brushgpsculpttype = 5;
pub const GPSCULPT_BRUSH_TYPE_PINCH: Brushgpsculpttype = 6;
pub const GPSCULPT_BRUSH_TYPE_RANDOMIZE: Brushgpsculpttype = 7;
pub const GPSCULPT_BRUSH_TYPE_CLONE: Brushgpsculpttype = 8;

/// `eBrushGPWeightType`.
pub type Brushgpweighttype = i8;
pub const GPWEIGHT_BRUSH_TYPE_DRAW: Brushgpweighttype = 0;
pub const GPWEIGHT_BRUSH_TYPE_BLUR: Brushgpweighttype = 1;
pub const GPWEIGHT_BRUSH_TYPE_AVERAGE: Brushgpweighttype = 2;
pub const GPWEIGHT_BRUSH_TYPE_SMEAR: Brushgpweighttype = 3;

/// `eBrushSculpt_DispDir`.
pub type BrushsculptDispdir = i32;
pub const SCULPT_DISP_DIR_AREA: BrushsculptDispdir = 0;
pub const SCULPT_DISP_DIR_VIEW: BrushsculptDispdir = 1;
pub const SCULPT_DISP_DIR_X: BrushsculptDispdir = 2;
pub const SCULPT_DISP_DIR_Y: BrushsculptDispdir = 3;
pub const SCULPT_DISP_DIR_Z: BrushsculptDispdir = 4;

/// `BrushMaskTool`.
pub type Brushmasktool = i8;
pub const BRUSH_MASK_DRAW: Brushmasktool = 0;
pub const BRUSH_MASK_SMOOTH: Brushmasktool = 1;

/// `eBlurKernelType`.
pub type Blurkerneltype = i32;
pub const KERNEL_GAUSSIAN: Blurkerneltype = 0;
pub const KERNEL_BOX: Blurkerneltype = 1;

/// `eBrushFalloffShape`.
pub type Brushfalloffshape = i8;
pub const PAINT_FALLOFF_SHAPE_SPHERE: Brushfalloffshape = 0;
pub const PAINT_FALLOFF_SHAPE_TUBE: Brushfalloffshape = 1;

/// `eBrushCurvesSculptFlag`.
pub type Brushcurvessculptflag = u32;
pub const BRUSH_CURVES_SCULPT_FLAG_SCALE_UNIFORM: Brushcurvessculptflag = 1 << 0;
pub const BRUSH_CURVES_SCULPT_FLAG_GROW_SHRINK_INVERT: Brushcurvessculptflag = 1 << 1;
pub const BRUSH_CURVES_SCULPT_FLAG_INTERPOLATE_LENGTH: Brushcurvessculptflag = 1 << 2;
pub const BRUSH_CURVES_SCULPT_FLAG_INTERPOLATE_SHAPE: Brushcurvessculptflag = 1 << 3;
pub const BRUSH_CURVES_SCULPT_FLAG_INTERPOLATE_POINT_COUNT: Brushcurvessculptflag = 1 << 4;
pub const BRUSH_CURVES_SCULPT_FLAG_INTERPOLATE_RADIUS: Brushcurvessculptflag = 1 << 5;

/// `eBrushCurvesSculptDensityMode`.
pub type Brushcurvessculptdensitymode = u8;
pub const BRUSH_CURVES_SCULPT_DENSITY_MODE_AUTO: Brushcurvessculptdensitymode = 0;
pub const BRUSH_CURVES_SCULPT_DENSITY_MODE_ADD: Brushcurvessculptdensitymode = 1;
pub const BRUSH_CURVES_SCULPT_DENSITY_MODE_REMOVE: Brushcurvessculptdensitymode = 2;

/// `eBrushColorJitterSettings_Flag`.
pub type BrushcolorjittersettingsFlag = i32;
pub const BRUSH_COLOR_JITTER_USE_HUE_AT_STROKE: BrushcolorjittersettingsFlag = 1 << 0;
pub const BRUSH_COLOR_JITTER_USE_SAT_AT_STROKE: BrushcolorjittersettingsFlag = 1 << 1;
pub const BRUSH_COLOR_JITTER_USE_VAL_AT_STROKE: BrushcolorjittersettingsFlag = 1 << 2;
pub const BRUSH_COLOR_JITTER_USE_HUE_RAND_PRESS: BrushcolorjittersettingsFlag = 1 << 3;
pub const BRUSH_COLOR_JITTER_USE_SAT_RAND_PRESS: BrushcolorjittersettingsFlag = 1 << 4;
pub const BRUSH_COLOR_JITTER_USE_VAL_RAND_PRESS: BrushcolorjittersettingsFlag = 1 << 5;

