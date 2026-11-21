//! Port of `DNA_modifier_types.h` — all modifier data structs.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::color_types::CurveMapping;
use crate::armature_types::ARM_DEF_VGROUP;
use crate::id::{ID, NIL};
use crate::modifier_enums::*;
use crate::small_types::ColorBand;
use crate::vec_types::{Rctf, Rcti};
use blenders_bli::listbase::ListBase;
/// `ModifierType`.
pub type Modifiertype = i32;
pub const eModifierType_None: Modifiertype = 0;
pub const eModifierType_Subsurf: Modifiertype = 1;
pub const eModifierType_Lattice: Modifiertype = 2;
pub const eModifierType_Curve: Modifiertype = 3;
pub const eModifierType_Build: Modifiertype = 4;
pub const eModifierType_Mirror: Modifiertype = 5;
pub const eModifierType_Decimate: Modifiertype = 6;
pub const eModifierType_Wave: Modifiertype = 7;
pub const eModifierType_Armature: Modifiertype = 8;
pub const eModifierType_Hook: Modifiertype = 9;
pub const eModifierType_Softbody: Modifiertype = 10;
pub const eModifierType_Boolean: Modifiertype = 11;
pub const eModifierType_Array: Modifiertype = 12;
pub const eModifierType_EdgeSplit: Modifiertype = 13;
pub const eModifierType_Displace: Modifiertype = 14;
pub const eModifierType_UVProject: Modifiertype = 15;
pub const eModifierType_Smooth: Modifiertype = 16;
pub const eModifierType_Cast: Modifiertype = 17;
pub const eModifierType_MeshDeform: Modifiertype = 18;
pub const eModifierType_ParticleSystem: Modifiertype = 19;
pub const eModifierType_ParticleInstance: Modifiertype = 20;
pub const eModifierType_Explode: Modifiertype = 21;
pub const eModifierType_Cloth: Modifiertype = 22;
pub const eModifierType_Collision: Modifiertype = 23;
pub const eModifierType_Bevel: Modifiertype = 24;
pub const eModifierType_Shrinkwrap: Modifiertype = 25;
pub const eModifierType_Fluidsim: Modifiertype = 26;
pub const eModifierType_Mask: Modifiertype = 27;
pub const eModifierType_SimpleDeform: Modifiertype = 28;
pub const eModifierType_Multires: Modifiertype = 29;
pub const eModifierType_Surface: Modifiertype = 30;
pub const eModifierType_Smoke: Modifiertype = 31;
pub const eModifierType_ShapeKey: Modifiertype = 32;
pub const eModifierType_Solidify: Modifiertype = 33;
pub const eModifierType_Screw: Modifiertype = 34;
pub const eModifierType_Warp: Modifiertype = 35;
pub const eModifierType_WeightVGEdit: Modifiertype = 36;
pub const eModifierType_WeightVGMix: Modifiertype = 37;
pub const eModifierType_WeightVGProximity: Modifiertype = 38;
pub const eModifierType_Ocean: Modifiertype = 39;
pub const eModifierType_DynamicPaint: Modifiertype = 40;
pub const eModifierType_Remesh: Modifiertype = 41;
pub const eModifierType_Skin: Modifiertype = 42;
pub const eModifierType_LaplacianSmooth: Modifiertype = 43;
pub const eModifierType_Triangulate: Modifiertype = 44;
pub const eModifierType_UVWarp: Modifiertype = 45;
pub const eModifierType_MeshCache: Modifiertype = 46;
pub const eModifierType_LaplacianDeform: Modifiertype = 47;
pub const eModifierType_Wireframe: Modifiertype = 48;
pub const eModifierType_DataTransfer: Modifiertype = 49;
pub const eModifierType_NormalEdit: Modifiertype = 50;
pub const eModifierType_CorrectiveSmooth: Modifiertype = 51;
pub const eModifierType_MeshSequenceCache: Modifiertype = 52;
pub const eModifierType_SurfaceDeform: Modifiertype = 53;
pub const eModifierType_WeightedNormal: Modifiertype = 54;
pub const eModifierType_Weld: Modifiertype = 55;
pub const eModifierType_Fluid: Modifiertype = 56;
pub const eModifierType_Nodes: Modifiertype = 57;
pub const eModifierType_MeshToVolume: Modifiertype = 58;
pub const eModifierType_VolumeDisplace: Modifiertype = 59;
pub const eModifierType_VolumeToMesh: Modifiertype = 60;
pub const eModifierType_GreasePencilOpacity: Modifiertype = 61;
pub const eModifierType_GreasePencilSubdiv: Modifiertype = 62;
pub const eModifierType_GreasePencilColor: Modifiertype = 63;
pub const eModifierType_GreasePencilTint: Modifiertype = 64;
pub const eModifierType_GreasePencilSmooth: Modifiertype = 65;
pub const eModifierType_GreasePencilOffset: Modifiertype = 66;
pub const eModifierType_GreasePencilNoise: Modifiertype = 67;
pub const eModifierType_GreasePencilMirror: Modifiertype = 68;
pub const eModifierType_GreasePencilThickness: Modifiertype = 69;
pub const eModifierType_GreasePencilLattice: Modifiertype = 70;
pub const eModifierType_GreasePencilDash: Modifiertype = 71;
pub const eModifierType_GreasePencilMultiply: Modifiertype = 72;
pub const eModifierType_GreasePencilLength: Modifiertype = 73;
pub const eModifierType_GreasePencilWeightAngle: Modifiertype = 74;
pub const eModifierType_GreasePencilArray: Modifiertype = 75;
pub const eModifierType_GreasePencilWeightProximity: Modifiertype = 76;
pub const eModifierType_GreasePencilHook: Modifiertype = 77;
pub const eModifierType_GreasePencilLineart: Modifiertype = 78;
pub const eModifierType_GreasePencilArmature: Modifiertype = 79;
pub const eModifierType_GreasePencilTime: Modifiertype = 80;
pub const eModifierType_GreasePencilEnvelope: Modifiertype = 81;
pub const eModifierType_GreasePencilOutline: Modifiertype = 82;
pub const eModifierType_GreasePencilShrinkwrap: Modifiertype = 83;
pub const eModifierType_GreasePencilBuild: Modifiertype = 84;
pub const eModifierType_GreasePencilSimplify: Modifiertype = 85;
pub const eModifierType_GreasePencilTexture: Modifiertype = 86;
pub const NUM_MODIFIER_TYPES: Modifiertype = eModifierType_GreasePencilTexture + 1;

/// `ModifierMode`.
pub type Modifiermode = u32;
pub const eModifierMode_Realtime: Modifiermode = 1 << 0;
pub const eModifierMode_Render: Modifiermode = 1 << 1;
pub const eModifierMode_Editmode: Modifiermode = 1 << 2;
pub const eModifierMode_OnCage: Modifiermode = 1 << 3;
pub const eModifierMode_Expanded_DEPRECATED: Modifiermode = 1 << 4;
pub const eModifierMode_Virtual: Modifiermode = 1 << 5;
pub const eModifierMode_ApplyOnSpline: Modifiermode = 1 << 6;
pub const eModifierMode_DisableTemporary: Modifiermode = 1 << 31;

/// `ModifierFlag`.
pub type Modifierflag = i16;
pub const eModifierFlag_OverrideLibrary_Local: Modifierflag = 1 << 0;
pub const eModifierFlag_SharedCaches: Modifierflag = 1 << 1;
pub const eModifierFlag_Active: Modifierflag = 1 << 2;
pub const eModifierFlag_UserModified: Modifierflag = 1 << 3;
pub const eModifierFlag_PinLast: Modifierflag = 1 << 4;

/// `SubsurfModifierFlag`.
pub type Subsurfmodifierflag = i16;
pub const eSubsurfModifierFlag_Incremental: Subsurfmodifierflag = 1 << 0;
pub const eSubsurfModifierFlag_DebugIncr: Subsurfmodifierflag = 1 << 1;
pub const eSubsurfModifierFlag_ControlEdges: Subsurfmodifierflag = 1 << 2;
pub const eSubsurfModifierFlag_SubsurfUv_DEPRECATED: Subsurfmodifierflag = 1 << 3;
pub const eSubsurfModifierFlag_UseCrease: Subsurfmodifierflag = 1 << 4;
pub const eSubsurfModifierFlag_UseCustomNormals: Subsurfmodifierflag = 1 << 5;
pub const eSubsurfModifierFlag_UseRecursiveSubdivision: Subsurfmodifierflag = 1 << 6;
pub const eSubsurfModifierFlag_UseAdaptiveSubdivision: Subsurfmodifierflag = 1 << 7;

/// `eSubsurfAdaptiveSpace`.
pub type Subsurfadaptivespace = i16;
pub const SUBSURF_ADAPTIVE_SPACE_PIXEL: Subsurfadaptivespace = 0;
pub const SUBSURF_ADAPTIVE_SPACE_OBJECT: Subsurfadaptivespace = 1;

/// `eSubsurfModifierType`.
pub type Subsurfmodifiertype = i16;
pub const SUBSURF_TYPE_CATMULL_CLARK: Subsurfmodifiertype = 0;
pub const SUBSURF_TYPE_SIMPLE: Subsurfmodifiertype = 1;

/// `eSubsurfUVSmooth`.
pub type Subsurfuvsmooth = i16;
pub const SUBSURF_UV_SMOOTH_NONE: Subsurfuvsmooth = 0;
pub const SUBSURF_UV_SMOOTH_PRESERVE_CORNERS: Subsurfuvsmooth = 1;
pub const SUBSURF_UV_SMOOTH_PRESERVE_CORNERS_AND_JUNCTIONS: Subsurfuvsmooth = 2;
pub const SUBSURF_UV_SMOOTH_PRESERVE_CORNERS_JUNCTIONS_AND_CONCAVE: Subsurfuvsmooth = 3;
pub const SUBSURF_UV_SMOOTH_PRESERVE_BOUNDARIES: Subsurfuvsmooth = 4;
pub const SUBSURF_UV_SMOOTH_ALL: Subsurfuvsmooth = 5;

/// `eSubsurfBoundarySmooth`.
pub type Subsurfboundarysmooth = i16;
pub const SUBSURF_BOUNDARY_SMOOTH_ALL: Subsurfboundarysmooth = 0;
pub const SUBSURF_BOUNDARY_SMOOTH_PRESERVE_CORNERS: Subsurfboundarysmooth = 1;

/// `LatticeModifierFlag`.
pub type Latticemodifierflag = i16;
pub const MOD_LATTICE_INVERT_VGROUP: Latticemodifierflag = 1 << 0;

/// `CurveModifierFlag`.
pub type Curvemodifierflag = i16;
pub const MOD_CURVE_INVERT_VGROUP: Curvemodifierflag = 1 << 0;

/// `CurveModifierDefaultAxis`.
pub type Curvemodifierdefaultaxis = i16;
pub const MOD_CURVE_POSX: Curvemodifierdefaultaxis = 1;
pub const MOD_CURVE_POSY: Curvemodifierdefaultaxis = 2;
pub const MOD_CURVE_POSZ: Curvemodifierdefaultaxis = 3;
pub const MOD_CURVE_NEGX: Curvemodifierdefaultaxis = 4;
pub const MOD_CURVE_NEGY: Curvemodifierdefaultaxis = 5;
pub const MOD_CURVE_NEGZ: Curvemodifierdefaultaxis = 6;

/// `BuildModifierFlag`.
pub type Buildmodifierflag = i16;
pub const MOD_BUILD_FLAG_RANDOMIZE: Buildmodifierflag = 1 << 0;
pub const MOD_BUILD_FLAG_REVERSE: Buildmodifierflag = 1 << 1;

/// `MaskModifierMode`.
pub type Maskmodifiermode = i16;
pub const MOD_MASK_MODE_VGROUP: Maskmodifiermode = 0;
pub const MOD_MASK_MODE_ARM: Maskmodifiermode = 1;

/// `MaskModifierFlag`.
pub type Maskmodifierflag = i16;
pub const MOD_MASK_INV: Maskmodifierflag = 1 << 0;
pub const MOD_MASK_SMOOTH: Maskmodifierflag = 1 << 1;

/// `ArrayModifierFitType`.
pub type Arraymodifierfittype = i32;
pub const MOD_ARR_FIXEDCOUNT: Arraymodifierfittype = 0;
pub const MOD_ARR_FITLENGTH: Arraymodifierfittype = 1;
pub const MOD_ARR_FITCURVE: Arraymodifierfittype = 2;

/// `ArrayModifierOffsetType`.
pub type Arraymodifieroffsettype = i32;
pub const MOD_ARR_OFF_CONST: Arraymodifieroffsettype = 1 << 0;
pub const MOD_ARR_OFF_RELATIVE: Arraymodifieroffsettype = 1 << 1;
pub const MOD_ARR_OFF_OBJ: Arraymodifieroffsettype = 1 << 2;

/// `ArrayModifierFlag`.
pub type Arraymodifierflag = i32;
pub const MOD_ARR_MERGE: Arraymodifierflag = 1 << 0;
pub const MOD_ARR_MERGEFINAL: Arraymodifierflag = 1 << 1;

/// `MirrorModifierFlag`.
pub type Mirrormodifierflag = i16;
pub const MOD_MIR_CLIPPING: Mirrormodifierflag = 1 << 0;
pub const MOD_MIR_MIRROR_U: Mirrormodifierflag = 1 << 1;
pub const MOD_MIR_MIRROR_V: Mirrormodifierflag = 1 << 2;
pub const MOD_MIR_AXIS_X: Mirrormodifierflag = 1 << 3;
pub const MOD_MIR_AXIS_Y: Mirrormodifierflag = 1 << 4;
pub const MOD_MIR_AXIS_Z: Mirrormodifierflag = 1 << 5;
pub const MOD_MIR_VGROUP: Mirrormodifierflag = 1 << 6;
pub const MOD_MIR_NO_MERGE: Mirrormodifierflag = 1 << 7;
pub const MOD_MIR_BISECT_AXIS_X: Mirrormodifierflag = 1 << 8;
pub const MOD_MIR_BISECT_AXIS_Y: Mirrormodifierflag = 1 << 9;
pub const MOD_MIR_BISECT_AXIS_Z: Mirrormodifierflag = 1 << 10;
pub const MOD_MIR_BISECT_FLIP_AXIS_X: Mirrormodifierflag = 1 << 11;
pub const MOD_MIR_BISECT_FLIP_AXIS_Y: Mirrormodifierflag = 1 << 12;
pub const MOD_MIR_BISECT_FLIP_AXIS_Z: Mirrormodifierflag = 1 << 13;
pub const MOD_MIR_MIRROR_UDIM: Mirrormodifierflag = 1 << 14;

/// `EdgeSplitModifierFlag`.
pub type Edgesplitmodifierflag = i32;
pub const MOD_EDGESPLIT_FROMANGLE: Edgesplitmodifierflag = 1 << 1;
pub const MOD_EDGESPLIT_FROMFLAG: Edgesplitmodifierflag = 1 << 2;

/// `BevelModifierFlag`.
pub type Bevelmodifierflag = i16;
pub const MOD_BEVEL_VERT_DEPRECATED: Bevelmodifierflag = 1 << 1;
pub const MOD_BEVEL_INVERT_VGROUP: Bevelmodifierflag = 1 << 2;
pub const MOD_BEVEL_ANGLE: Bevelmodifierflag = 1 << 3;
pub const MOD_BEVEL_WEIGHT: Bevelmodifierflag = 1 << 4;
pub const MOD_BEVEL_VGROUP: Bevelmodifierflag = 1 << 5;
pub const MOD_BEVEL_CUSTOM_PROFILE_DEPRECATED: Bevelmodifierflag = 1 << 7;
pub const MOD_BEVEL_OVERLAP_OK: Bevelmodifierflag = 1 << 13;
pub const MOD_BEVEL_EVEN_WIDTHS: Bevelmodifierflag = 1 << 14;
pub const MOD_BEVEL_HARDEN_NORMALS: Bevelmodifierflag = 1 << 15;

/// `BevelModifierValFlag`.
pub type Bevelmodifiervalflag = i16;
pub const MOD_BEVEL_AMT_OFFSET: Bevelmodifiervalflag = 0;
pub const MOD_BEVEL_AMT_WIDTH: Bevelmodifiervalflag = 1;
pub const MOD_BEVEL_AMT_DEPTH: Bevelmodifiervalflag = 2;
pub const MOD_BEVEL_AMT_PERCENT: Bevelmodifiervalflag = 3;
pub const MOD_BEVEL_AMT_ABSOLUTE: Bevelmodifiervalflag = 4;

/// `BevelModifierProfileType`.
pub type Bevelmodifierprofiletype = i16;
pub const MOD_BEVEL_PROFILE_SUPERELLIPSE: Bevelmodifierprofiletype = 0;
pub const MOD_BEVEL_PROFILE_CUSTOM: Bevelmodifierprofiletype = 1;

/// `BevelModifierEdgeFlag`.
pub type Bevelmodifieredgeflag = i16;
pub const MOD_BEVEL_MARK_SEAM: Bevelmodifieredgeflag = 1 << 0;
pub const MOD_BEVEL_MARK_SHARP: Bevelmodifieredgeflag = 1 << 1;

/// `BevelModifierFaceStrengthMode`.
pub type Bevelmodifierfacestrengthmode = i16;
pub const MOD_BEVEL_FACE_STRENGTH_NONE: Bevelmodifierfacestrengthmode = 0;
pub const MOD_BEVEL_FACE_STRENGTH_NEW: Bevelmodifierfacestrengthmode = 1;
pub const MOD_BEVEL_FACE_STRENGTH_AFFECTED: Bevelmodifierfacestrengthmode = 2;
pub const MOD_BEVEL_FACE_STRENGTH_ALL: Bevelmodifierfacestrengthmode = 3;

/// `BevelModifierMiter`.
pub type Bevelmodifiermiter = i16;
pub const MOD_BEVEL_MITER_SHARP: Bevelmodifiermiter = 0;
pub const MOD_BEVEL_MITER_PATCH: Bevelmodifiermiter = 1;
pub const MOD_BEVEL_MITER_ARC: Bevelmodifiermiter = 2;

/// `BevelModifierVMeshMethod`.
pub type Bevelmodifiervmeshmethod = i16;
pub const MOD_BEVEL_VMESH_ADJ: Bevelmodifiervmeshmethod = 0;
pub const MOD_BEVEL_VMESH_CUTOFF: Bevelmodifiervmeshmethod = 1;

/// `BevelModifierAffectType`.
pub type Bevelmodifieraffecttype = i8;
pub const MOD_BEVEL_AFFECT_VERTICES: Bevelmodifieraffecttype = 0;
pub const MOD_BEVEL_AFFECT_EDGES: Bevelmodifieraffecttype = 1;

/// `FluidModifierType`.
pub type Fluidmodifiertype = i32;
pub const MOD_FLUID_TYPE_DOMAIN: Fluidmodifiertype = 1 << 0;
pub const MOD_FLUID_TYPE_FLOW: Fluidmodifiertype = 1 << 1;
pub const MOD_FLUID_TYPE_EFFEC: Fluidmodifiertype = 1 << 2;

/// `DisplaceModifierFlag`.
pub type Displacemodifierflag = i16;
pub const MOD_DISP_INVERT_VGROUP: Displacemodifierflag = 1 << 0;

/// `DisplaceModifierDirection`.
pub type Displacemodifierdirection = i32;
pub const MOD_DISP_DIR_X: Displacemodifierdirection = 0;
pub const MOD_DISP_DIR_Y: Displacemodifierdirection = 1;
pub const MOD_DISP_DIR_Z: Displacemodifierdirection = 2;
pub const MOD_DISP_DIR_NOR: Displacemodifierdirection = 3;
pub const MOD_DISP_DIR_RGB_XYZ: Displacemodifierdirection = 4;
pub const MOD_DISP_DIR_CLNOR: Displacemodifierdirection = 5;

/// `DisplaceModifierTexMapping`.
pub type Displacemodifiertexmapping = i32;
pub const MOD_DISP_MAP_LOCAL: Displacemodifiertexmapping = 0;
pub const MOD_DISP_MAP_GLOBAL: Displacemodifiertexmapping = 1;
pub const MOD_DISP_MAP_OBJECT: Displacemodifiertexmapping = 2;
pub const MOD_DISP_MAP_UV: Displacemodifiertexmapping = 3;

/// `DisplaceModifierSpace`.
pub type Displacemodifierspace = i32;
pub const MOD_DISP_SPACE_LOCAL: Displacemodifierspace = 0;
pub const MOD_DISP_SPACE_GLOBAL: Displacemodifierspace = 1;

/// `DecimateModifierFlag`.
pub type Decimatemodifierflag = i16;
pub const MOD_DECIM_FLAG_INVERT_VGROUP: Decimatemodifierflag = 1 << 0;
pub const MOD_DECIM_FLAG_TRIANGULATE: Decimatemodifierflag = 1 << 1;
pub const MOD_DECIM_FLAG_ALL_BOUNDARY_VERTS: Decimatemodifierflag = 1 << 2;
pub const MOD_DECIM_FLAG_SYMMETRY: Decimatemodifierflag = 1 << 3;

/// `DecimateModifierMode`.
pub type Decimatemodifiermode = i16;
pub const MOD_DECIM_MODE_COLLAPSE: Decimatemodifiermode = 0;
pub const MOD_DECIM_MODE_UNSUBDIV: Decimatemodifiermode = 1;
pub const MOD_DECIM_MODE_DISSOLVE: Decimatemodifiermode = 2;

/// `SmoothModifierFlag`.
pub type Smoothmodifierflag = i16;
pub const MOD_SMOOTH_INVERT_VGROUP: Smoothmodifierflag = 1 << 0;
pub const MOD_SMOOTH_X: Smoothmodifierflag = 1 << 1;
pub const MOD_SMOOTH_Y: Smoothmodifierflag = 1 << 2;
pub const MOD_SMOOTH_Z: Smoothmodifierflag = 1 << 3;

/// `CastModifierFlag`.
pub type Castmodifierflag = i16;
pub const MOD_CAST_INVERT_VGROUP: Castmodifierflag = 1 << 0;
pub const MOD_CAST_X: Castmodifierflag = 1 << 1;
pub const MOD_CAST_Y: Castmodifierflag = 1 << 2;
pub const MOD_CAST_Z: Castmodifierflag = 1 << 3;
pub const MOD_CAST_USE_OB_TRANSFORM: Castmodifierflag = 1 << 4;
pub const MOD_CAST_SIZE_FROM_RADIUS: Castmodifierflag = 1 << 5;

/// `CastModifierType`.
pub type Castmodifiertype = i16;
pub const MOD_CAST_TYPE_SPHERE: Castmodifiertype = 0;
pub const MOD_CAST_TYPE_CYLINDER: Castmodifiertype = 1;
pub const MOD_CAST_TYPE_CUBOID: Castmodifiertype = 2;

/// `WaveModifierFlag`.
pub type Wavemodifierflag = i16;
pub const MOD_WAVE_INVERT_VGROUP: Wavemodifierflag = 1 << 0;
pub const MOD_WAVE_X: Wavemodifierflag = 1 << 1;
pub const MOD_WAVE_Y: Wavemodifierflag = 1 << 2;
pub const MOD_WAVE_CYCL: Wavemodifierflag = 1 << 3;
pub const MOD_WAVE_NORM: Wavemodifierflag = 1 << 4;
pub const MOD_WAVE_NORM_X: Wavemodifierflag = 1 << 5;
pub const MOD_WAVE_NORM_Y: Wavemodifierflag = 1 << 6;
pub const MOD_WAVE_NORM_Z: Wavemodifierflag = 1 << 7;

/// `HookModifierFlag`.
pub type Hookmodifierflag = i8;
pub const MOD_HOOK_UNIFORM_SPACE: Hookmodifierflag = 1 << 0;
pub const MOD_HOOK_INVERT_VGROUP: Hookmodifierflag = 1 << 1;

/// `HookModifierFalloff`.
pub type Hookmodifierfalloff = i8;
pub const eHook_Falloff_None: Hookmodifierfalloff = 0;
pub const eHook_Falloff_Curve: Hookmodifierfalloff = 1;
pub const eHook_Falloff_Sharp: Hookmodifierfalloff = 2;
pub const eHook_Falloff_Smooth: Hookmodifierfalloff = 3;
pub const eHook_Falloff_Root: Hookmodifierfalloff = 4;
pub const eHook_Falloff_Linear: Hookmodifierfalloff = 5;
pub const eHook_Falloff_Const: Hookmodifierfalloff = 6;
pub const eHook_Falloff_Sphere: Hookmodifierfalloff = 7;
pub const eHook_Falloff_InvSquare: Hookmodifierfalloff = 8;

/// `BooleanModifierMaterialMode`.
pub type Booleanmodifiermaterialmode = i8;
pub const eBooleanModifierMaterialMode_Index: Booleanmodifiermaterialmode = 0;
pub const eBooleanModifierMaterialMode_Transfer: Booleanmodifiermaterialmode = 1;

/// `BooleanModifierOp`.
pub type Booleanmodifierop = i8;
pub const eBooleanModifierOp_Intersect: Booleanmodifierop = 0;
pub const eBooleanModifierOp_Union: Booleanmodifierop = 1;
pub const eBooleanModifierOp_Difference: Booleanmodifierop = 2;

/// `BooleanModifierSolver`.
pub type Booleanmodifiersolver = i8;
pub const eBooleanModifierSolver_Float: Booleanmodifiersolver = 0;
pub const eBooleanModifierSolver_Mesh_Arr: Booleanmodifiersolver = 1;
pub const eBooleanModifierSolver_Manifold: Booleanmodifiersolver = 2;

/// `BooleanModifierFlag`.
pub type Booleanmodifierflag = i8;
pub const eBooleanModifierFlag_Self: Booleanmodifierflag = 1 << 0;
pub const eBooleanModifierFlag_Object: Booleanmodifierflag = 1 << 1;
pub const eBooleanModifierFlag_Collection: Booleanmodifierflag = 1 << 2;
pub const eBooleanModifierFlag_HoleTolerant: Booleanmodifierflag = 1 << 3;

/// `BooleanModifierBMeshFlag`.
pub type Booleanmodifierbmeshflag = i8;
pub const eBooleanModifierBMeshFlag_BMesh_Separate: Booleanmodifierbmeshflag = 1 << 0;
pub const eBooleanModifierBMeshFlag_BMesh_NoDissolve: Booleanmodifierbmeshflag = 1 << 1;
pub const eBooleanModifierBMeshFlag_BMesh_NoConnectRegions: Booleanmodifierbmeshflag = 1 << 2;

/// `MeshDeformModifierFlag`.
pub type Meshdeformmodifierflag = i16;
pub const MOD_MDEF_INVERT_VGROUP: Meshdeformmodifierflag = 1 << 0;
pub const MOD_MDEF_DYNAMIC_BIND: Meshdeformmodifierflag = 1 << 1;

/// `ParticleSystemModifierFlag`.
pub type Particlesystemmodifierflag = i16;
pub const eParticleSystemFlag_Pars: Particlesystemmodifierflag = 1 << 0;
pub const eParticleSystemFlag_psys_updated: Particlesystemmodifierflag = 1 << 1;
pub const eParticleSystemFlag_file_loaded: Particlesystemmodifierflag = 1 << 2;

/// `ParticleInstanceModifierFlag`.
pub type Particleinstancemodifierflag = i16;
pub const eParticleInstanceFlag_Parents: Particleinstancemodifierflag = 1 << 0;
pub const eParticleInstanceFlag_Children: Particleinstancemodifierflag = 1 << 1;
pub const eParticleInstanceFlag_Path: Particleinstancemodifierflag = 1 << 2;
pub const eParticleInstanceFlag_Unborn: Particleinstancemodifierflag = 1 << 3;
pub const eParticleInstanceFlag_Alive: Particleinstancemodifierflag = 1 << 4;
pub const eParticleInstanceFlag_Dead: Particleinstancemodifierflag = 1 << 5;
pub const eParticleInstanceFlag_KeepShape: Particleinstancemodifierflag = 1 << 6;
pub const eParticleInstanceFlag_UseSize: Particleinstancemodifierflag = 1 << 7;

/// `ParticleInstanceModifierSpace`.
pub type Particleinstancemodifierspace = i16;
pub const eParticleInstanceSpace_World: Particleinstancemodifierspace = 0;
pub const eParticleInstanceSpace_Local: Particleinstancemodifierspace = 1;

/// `ExplodeModifierFlag`.
pub type Explodemodifierflag = i16;
pub const eExplodeFlag_CalcFaces: Explodemodifierflag = 1 << 0;
pub const eExplodeFlag_PaSize: Explodemodifierflag = 1 << 1;
pub const eExplodeFlag_EdgeCut: Explodemodifierflag = 1 << 2;
pub const eExplodeFlag_Unborn: Explodemodifierflag = 1 << 3;
pub const eExplodeFlag_Alive: Explodemodifierflag = 1 << 4;
pub const eExplodeFlag_Dead: Explodemodifierflag = 1 << 5;
pub const eExplodeFlag_INVERT_VGROUP: Explodemodifierflag = 1 << 6;

/// `MultiresModifierFlag`.
pub type Multiresmodifierflag = i8;
pub const eMultiresModifierFlag_ControlEdges: Multiresmodifierflag = 1 << 0;
pub const eMultiresModifierFlag_PlainUv_DEPRECATED: Multiresmodifierflag = 1 << 1;
pub const eMultiresModifierFlag_UseCrease: Multiresmodifierflag = 1 << 2;
pub const eMultiresModifierFlag_UseCustomNormals: Multiresmodifierflag = 1 << 3;
pub const eMultiresModifierFlag_UseSculptBaseMesh: Multiresmodifierflag = 1 << 4;

/// `SimpleDeformModifierFlag`.
pub type Simpledeformmodifierflag = i8;
pub const MOD_SIMPLEDEFORM_FLAG_INVERT_VGROUP: Simpledeformmodifierflag = 1 << 0;

/// `SimpleDeformModifierMode`.
pub type Simpledeformmodifiermode = i8;
pub const MOD_SIMPLEDEFORM_MODE_TWIST: Simpledeformmodifiermode = 1;
pub const MOD_SIMPLEDEFORM_MODE_BEND: Simpledeformmodifiermode = 2;
pub const MOD_SIMPLEDEFORM_MODE_TAPER: Simpledeformmodifiermode = 3;
pub const MOD_SIMPLEDEFORM_MODE_STRETCH: Simpledeformmodifiermode = 4;

/// `SimpleDeformModifierLockAxis`.
pub type Simpledeformmodifierlockaxis = i8;
pub const MOD_SIMPLEDEFORM_LOCK_AXIS_X: Simpledeformmodifierlockaxis = 1 << 0;
pub const MOD_SIMPLEDEFORM_LOCK_AXIS_Y: Simpledeformmodifierlockaxis = 1 << 1;
pub const MOD_SIMPLEDEFORM_LOCK_AXIS_Z: Simpledeformmodifierlockaxis = 1 << 2;

/// `SolidifyModifierFlag`.
pub type Solidifymodifierflag = i32;
pub const MOD_SOLIDIFY_RIM: Solidifymodifierflag = 1 << 0;
pub const MOD_SOLIDIFY_EVEN: Solidifymodifierflag = 1 << 1;
pub const MOD_SOLIDIFY_NORMAL_CALC: Solidifymodifierflag = 1 << 2;
pub const MOD_SOLIDIFY_VGROUP_INV: Solidifymodifierflag = 1 << 3;
pub const MOD_SOLIDIFY_RIM_MATERIAL: Solidifymodifierflag = 1 << 4;
pub const MOD_SOLIDIFY_FLIP: Solidifymodifierflag = 1 << 5;
pub const MOD_SOLIDIFY_NOSHELL: Solidifymodifierflag = 1 << 6;
pub const MOD_SOLIDIFY_OFFSET_ANGLE_CLAMP: Solidifymodifierflag = 1 << 7;
pub const MOD_SOLIDIFY_NONMANIFOLD_FLAT_FACES: Solidifymodifierflag = 1 << 8;

/// `SolidifyModifierMode`.
pub type Solidifymodifiermode = i8;
pub const MOD_SOLIDIFY_MODE_EXTRUDE: Solidifymodifiermode = 0;
pub const MOD_SOLIDIFY_MODE_NONMANIFOLD: Solidifymodifiermode = 1;

/// `SolidifyModifierNonManifoldOffsetMode`.
pub type Solidifymodifiernonmanifoldoffsetmode = i8;
pub const MOD_SOLIDIFY_NONMANIFOLD_OFFSET_MODE_FIXED: Solidifymodifiernonmanifoldoffsetmode = 0;
pub const MOD_SOLIDIFY_NONMANIFOLD_OFFSET_MODE_EVEN: Solidifymodifiernonmanifoldoffsetmode = 1;
pub const MOD_SOLIDIFY_NONMANIFOLD_OFFSET_MODE_CONSTRAINTS: Solidifymodifiernonmanifoldoffsetmode = 2;

/// `SolidifyModifierNonManifoldBoundaryMode`.
pub type Solidifymodifiernonmanifoldboundarymode = i8;
pub const MOD_SOLIDIFY_NONMANIFOLD_BOUNDARY_MODE_NONE: Solidifymodifiernonmanifoldboundarymode = 0;
pub const MOD_SOLIDIFY_NONMANIFOLD_BOUNDARY_MODE_ROUND: Solidifymodifiernonmanifoldboundarymode = 1;
pub const MOD_SOLIDIFY_NONMANIFOLD_BOUNDARY_MODE_FLAT: Solidifymodifiernonmanifoldboundarymode = 2;

/// `ScrewModifierFlag`.
pub type Screwmodifierflag = i16;
pub const MOD_SCREW_NORMAL_FLIP: Screwmodifierflag = 1 << 0;
pub const MOD_SCREW_NORMAL_CALC: Screwmodifierflag = 1 << 1;
pub const MOD_SCREW_OBJECT_OFFSET: Screwmodifierflag = 1 << 2;
pub const MOD_SCREW_SMOOTH_SHADING: Screwmodifierflag = 1 << 5;
pub const MOD_SCREW_UV_STRETCH_U: Screwmodifierflag = 1 << 6;
pub const MOD_SCREW_UV_STRETCH_V: Screwmodifierflag = 1 << 7;
pub const MOD_SCREW_MERGE: Screwmodifierflag = 1 << 8;

/// `OceanModifierGeometryMode`.
pub type Oceanmodifiergeometrymode = i8;
pub const MOD_OCEAN_GEOM_GENERATE: Oceanmodifiergeometrymode = 0;
pub const MOD_OCEAN_GEOM_DISPLACE: Oceanmodifiergeometrymode = 1;
pub const MOD_OCEAN_GEOM_SIM_ONLY: Oceanmodifiergeometrymode = 2;

/// `OceanModifierSpectrum`.
pub type Oceanmodifierspectrum = i32;
pub const MOD_OCEAN_SPECTRUM_PHILLIPS: Oceanmodifierspectrum = 0;
pub const MOD_OCEAN_SPECTRUM_PIERSON_MOSKOWITZ: Oceanmodifierspectrum = 1;
pub const MOD_OCEAN_SPECTRUM_JONSWAP: Oceanmodifierspectrum = 2;
pub const MOD_OCEAN_SPECTRUM_TEXEL_MARSEN_ARSLOE: Oceanmodifierspectrum = 3;

/// `OceanModifierFlag`.
pub type Oceanmodifierflag = i8;
pub const MOD_OCEAN_GENERATE_FOAM: Oceanmodifierflag = 1 << 0;
pub const MOD_OCEAN_GENERATE_NORMALS: Oceanmodifierflag = 1 << 1;
pub const MOD_OCEAN_GENERATE_SPRAY: Oceanmodifierflag = 1 << 2;
pub const MOD_OCEAN_INVERT_SPRAY: Oceanmodifierflag = 1 << 3;

/// `WarpModifierFlag`.
pub type Warpmodifierflag = i8;
pub const MOD_WARP_VOLUME_PRESERVE: Warpmodifierflag = 1 << 0;
pub const MOD_WARP_INVERT_VGROUP: Warpmodifierflag = 1 << 1;

/// `WarpModifierFalloff`.
pub type Warpmodifierfalloff = i8;
pub const eWarp_Falloff_None: Warpmodifierfalloff = 0;
pub const eWarp_Falloff_Curve: Warpmodifierfalloff = 1;
pub const eWarp_Falloff_Sharp: Warpmodifierfalloff = 2;
pub const eWarp_Falloff_Smooth: Warpmodifierfalloff = 3;
pub const eWarp_Falloff_Root: Warpmodifierfalloff = 4;
pub const eWarp_Falloff_Linear: Warpmodifierfalloff = 5;
pub const eWarp_Falloff_Const: Warpmodifierfalloff = 6;
pub const eWarp_Falloff_Sphere: Warpmodifierfalloff = 7;
pub const eWarp_Falloff_InvSquare: Warpmodifierfalloff = 8;

/// `WeightVGEditModifierEditFlags`.
pub type Weightvgeditmodifiereditflags = i16;
pub const MOD_WVG_EDIT_WEIGHTS_NORMALIZE: Weightvgeditmodifiereditflags = 1 << 0;
pub const MOD_WVG_INVERT_FALLOFF: Weightvgeditmodifiereditflags = 1 << 1;
pub const MOD_WVG_EDIT_INVERT_VGROUP_MASK: Weightvgeditmodifiereditflags = 1 << 2;
pub const MOD_WVG_EDIT_ADD2VG: Weightvgeditmodifiereditflags = 1 << 3;
pub const MOD_WVG_EDIT_REMFVG: Weightvgeditmodifiereditflags = 1 << 4;

/// `WeightVGMixModifierMixMode`.
pub type Weightvgmixmodifiermixmode = i8;
pub const MOD_WVG_MIX_SET: Weightvgmixmodifiermixmode = 1;
pub const MOD_WVG_MIX_ADD: Weightvgmixmodifiermixmode = 2;
pub const MOD_WVG_MIX_SUB: Weightvgmixmodifiermixmode = 3;
pub const MOD_WVG_MIX_MUL: Weightvgmixmodifiermixmode = 4;
pub const MOD_WVG_MIX_DIV: Weightvgmixmodifiermixmode = 5;
pub const MOD_WVG_MIX_DIF: Weightvgmixmodifiermixmode = 6;
pub const MOD_WVG_MIX_AVG: Weightvgmixmodifiermixmode = 7;
pub const MOD_WVG_MIX_MIN: Weightvgmixmodifiermixmode = 8;
pub const MOD_WVG_MIX_MAX: Weightvgmixmodifiermixmode = 9;

/// `WeightVGMixModifierMixSet`.
pub type Weightvgmixmodifiermixset = i8;
pub const MOD_WVG_SET_ALL: Weightvgmixmodifiermixset = 1;
pub const MOD_WVG_SET_A: Weightvgmixmodifiermixset = 2;
pub const MOD_WVG_SET_B: Weightvgmixmodifiermixset = 3;
pub const MOD_WVG_SET_OR: Weightvgmixmodifiermixset = 4;
pub const MOD_WVG_SET_AND: Weightvgmixmodifiermixset = 5;

/// `WeightVGMixModifierFlag`.
pub type Weightvgmixmodifierflag = i8;
pub const MOD_WVG_MIX_INVERT_VGROUP_MASK: Weightvgmixmodifierflag = 1 << 0;
pub const MOD_WVG_MIX_WEIGHTS_NORMALIZE: Weightvgmixmodifierflag = 1 << 1;
pub const MOD_WVG_MIX_INVERT_VGROUP_A: Weightvgmixmodifierflag = 1 << 2;
pub const MOD_WVG_MIX_INVERT_VGROUP_B: Weightvgmixmodifierflag = 1 << 3;

/// `GreasePencilWeightProximityFlag`.
pub type Greasepencilweightproximityflag = i32;
pub const MOD_GREASE_PENCIL_WEIGHT_PROXIMITY_INVERT_OUTPUT: Greasepencilweightproximityflag = 1 << 0;
pub const MOD_GREASE_PENCIL_WEIGHT_PROXIMITY_MULTIPLY_DATA: Greasepencilweightproximityflag = 1 << 1;

/// `WeightVGProximityModifierProximityMode`.
pub type Weightvgproximitymodifierproximitymode = i32;
pub const MOD_WVG_PROXIMITY_OBJECT: Weightvgproximitymodifierproximitymode = 1;
pub const MOD_WVG_PROXIMITY_GEOMETRY: Weightvgproximitymodifierproximitymode = 2;

/// `WeightVGProximityModifierFlag`.
pub type Weightvgproximitymodifierflag = i32;
pub const MOD_WVG_PROXIMITY_GEOM_VERTS: Weightvgproximitymodifierflag = 1 << 0;
pub const MOD_WVG_PROXIMITY_GEOM_EDGES: Weightvgproximitymodifierflag = 1 << 1;
pub const MOD_WVG_PROXIMITY_GEOM_FACES: Weightvgproximitymodifierflag = 1 << 2;
pub const MOD_WVG_PROXIMITY_INVERT_VGROUP_MASK: Weightvgproximitymodifierflag = 1 << 3;
pub const MOD_WVG_PROXIMITY_INVERT_FALLOFF: Weightvgproximitymodifierflag = 1 << 4;
pub const MOD_WVG_PROXIMITY_WEIGHTS_NORMALIZE: Weightvgproximitymodifierflag = 1 << 5;

/// `WeightVGProximityFalloff`.
pub type Weightvgproximityfalloff = i16;
pub const MOD_WVG_MAPPING_NONE: Weightvgproximityfalloff = 0;
pub const MOD_WVG_MAPPING_CURVE: Weightvgproximityfalloff = 1;
pub const MOD_WVG_MAPPING_SHARP: Weightvgproximityfalloff = 2;
pub const MOD_WVG_MAPPING_SMOOTH: Weightvgproximityfalloff = 3;
pub const MOD_WVG_MAPPING_ROOT: Weightvgproximityfalloff = 4;
pub const MOD_WVG_MAPPING_SPHERE: Weightvgproximityfalloff = 7;
pub const MOD_WVG_MAPPING_RANDOM: Weightvgproximityfalloff = 8;
pub const MOD_WVG_MAPPING_STEP: Weightvgproximityfalloff = 9;

/// `WeightVGProximityModifierMaskTexChannel`.
pub type Weightvgproximitymodifiermasktexchannel = i32;
pub const MOD_WVG_MASK_TEX_USE_INT: Weightvgproximitymodifiermasktexchannel = 1;
pub const MOD_WVG_MASK_TEX_USE_RED: Weightvgproximitymodifiermasktexchannel = 2;
pub const MOD_WVG_MASK_TEX_USE_GREEN: Weightvgproximitymodifiermasktexchannel = 3;
pub const MOD_WVG_MASK_TEX_USE_BLUE: Weightvgproximitymodifiermasktexchannel = 4;
pub const MOD_WVG_MASK_TEX_USE_HUE: Weightvgproximitymodifiermasktexchannel = 5;
pub const MOD_WVG_MASK_TEX_USE_SAT: Weightvgproximitymodifiermasktexchannel = 6;
pub const MOD_WVG_MASK_TEX_USE_VAL: Weightvgproximitymodifiermasktexchannel = 7;
pub const MOD_WVG_MASK_TEX_USE_ALPHA: Weightvgproximitymodifiermasktexchannel = 8;

/// `DynamicPaintModifierType`.
pub type Dynamicpaintmodifiertype = i32;
pub const MOD_DYNAMICPAINT_TYPE_CANVAS: Dynamicpaintmodifiertype = 1 << 0;
pub const MOD_DYNAMICPAINT_TYPE_BRUSH: Dynamicpaintmodifiertype = 1 << 1;

/// `eRemeshModifierFlags`.
pub type Remeshmodifierflags = i8;
pub const MOD_REMESH_FLOOD_FILL: Remeshmodifierflags = 1 << 0;
pub const MOD_REMESH_SMOOTH_SHADING: Remeshmodifierflags = 1 << 1;

/// `eRemeshModifierMode`.
pub type Remeshmodifiermode = i8;
pub const MOD_REMESH_CENTROID: Remeshmodifiermode = 0;
pub const MOD_REMESH_MASS_POINT: Remeshmodifiermode = 1;
pub const MOD_REMESH_SHARP_FEATURES: Remeshmodifiermode = 2;
pub const MOD_REMESH_VOXEL: Remeshmodifiermode = 3;

/// `SkinModifierSymmetryAxis`.
pub type Skinmodifiersymmetryaxis = i8;
pub const MOD_SKIN_SYMM_X: Skinmodifiersymmetryaxis = 1 << 0;
pub const MOD_SKIN_SYMM_Y: Skinmodifiersymmetryaxis = 1 << 1;
pub const MOD_SKIN_SYMM_Z: Skinmodifiersymmetryaxis = 1 << 2;

/// `SkinModifierFlag`.
pub type Skinmodifierflag = i8;
pub const MOD_SKIN_SMOOTH_SHADING: Skinmodifierflag = 1;

/// `TriangulateModifierFlag`.
pub type Triangulatemodifierflag = i32;
pub const MOD_TRIANGULATE_BEAUTY: Triangulatemodifierflag = 1 << 0;
pub const MOD_TRIANGULATE_KEEP_CUSTOMLOOP_NORMALS: Triangulatemodifierflag = 1 << 1;

/// `TriangulateModifierNgonMethod`.
pub type Triangulatemodifierngonmethod = i32;
pub const MOD_TRIANGULATE_NGON_BEAUTY: Triangulatemodifierngonmethod = 0;
pub const MOD_TRIANGULATE_NGON_EARCLIP: Triangulatemodifierngonmethod = 1;

/// `TriangulateModifierQuadMethod`.
pub type Triangulatemodifierquadmethod = i32;
pub const MOD_TRIANGULATE_QUAD_BEAUTY: Triangulatemodifierquadmethod = 0;
pub const MOD_TRIANGULATE_QUAD_FIXED: Triangulatemodifierquadmethod = 1;
pub const MOD_TRIANGULATE_QUAD_ALTERNATE: Triangulatemodifierquadmethod = 2;
pub const MOD_TRIANGULATE_QUAD_SHORTEDGE: Triangulatemodifierquadmethod = 3;
pub const MOD_TRIANGULATE_QUAD_LONGEDGE: Triangulatemodifierquadmethod = 4;

/// `LaplacianSmoothModifierFlag`.
pub type Laplaciansmoothmodifierflag = i16;
pub const MOD_LAPLACIANSMOOTH_X: Laplaciansmoothmodifierflag = 1 << 1;
pub const MOD_LAPLACIANSMOOTH_Y: Laplaciansmoothmodifierflag = 1 << 2;
pub const MOD_LAPLACIANSMOOTH_Z: Laplaciansmoothmodifierflag = 1 << 3;
pub const MOD_LAPLACIANSMOOTH_PRESERVE_VOLUME: Laplaciansmoothmodifierflag = 1 << 4;
pub const MOD_LAPLACIANSMOOTH_NORMALIZED: Laplaciansmoothmodifierflag = 1 << 5;
pub const MOD_LAPLACIANSMOOTH_INVERT_VGROUP: Laplaciansmoothmodifierflag = 1 << 6;

/// `CorrectiveSmoothModifierType`.
pub type Correctivesmoothmodifiertype = i8;
pub const MOD_CORRECTIVESMOOTH_SMOOTH_SIMPLE: Correctivesmoothmodifiertype = 0;
pub const MOD_CORRECTIVESMOOTH_SMOOTH_LENGTH_WEIGHT: Correctivesmoothmodifiertype = 1;

/// `CorrectiveSmoothRestSource`.
pub type Correctivesmoothrestsource = i8;
pub const MOD_CORRECTIVESMOOTH_RESTSOURCE_ORCO: Correctivesmoothrestsource = 0;
pub const MOD_CORRECTIVESMOOTH_RESTSOURCE_BIND: Correctivesmoothrestsource = 1;

/// `CorrectiveSmoothModifierFlag`.
pub type Correctivesmoothmodifierflag = i16;
pub const MOD_CORRECTIVESMOOTH_INVERT_VGROUP: Correctivesmoothmodifierflag = 1 << 0;
pub const MOD_CORRECTIVESMOOTH_ONLY_SMOOTH: Correctivesmoothmodifierflag = 1 << 1;
pub const MOD_CORRECTIVESMOOTH_PIN_BOUNDARY: Correctivesmoothmodifierflag = 1 << 2;

/// `UVWarpModifierFlag`.
pub type Uvwarpmodifierflag = i16;
pub const MOD_UVWARP_INVERT_VGROUP: Uvwarpmodifierflag = 1 << 0;

/// `MeshCacheModifierFlag`.
pub type Meshcachemodifierflag = i8;
pub const MOD_MESHCACHE_INVERT_VERTEX_GROUP: Meshcachemodifierflag = 1 << 0;

/// `MeshCacheModifierType`.
pub type Meshcachemodifiertype = i8;
pub const MOD_MESHCACHE_TYPE_MDD: Meshcachemodifiertype = 1;
pub const MOD_MESHCACHE_TYPE_PC2: Meshcachemodifiertype = 2;

/// `MeshCacheModifierDeformMode`.
pub type Meshcachemodifierdeformmode = i8;
pub const MOD_MESHCACHE_DEFORM_OVERWRITE: Meshcachemodifierdeformmode = 0;
pub const MOD_MESHCACHE_DEFORM_INTEGRATE: Meshcachemodifierdeformmode = 1;

/// `MeshCacheModifierInterpolation`.
pub type Meshcachemodifierinterpolation = i8;
pub const MOD_MESHCACHE_INTERP_NONE: Meshcachemodifierinterpolation = 0;
pub const MOD_MESHCACHE_INTERP_LINEAR: Meshcachemodifierinterpolation = 1;

/// `MeshCacheModifierTimeMode`.
pub type Meshcachemodifiertimemode = i8;
pub const MOD_MESHCACHE_TIME_FRAME: Meshcachemodifiertimemode = 0;
pub const MOD_MESHCACHE_TIME_SECONDS: Meshcachemodifiertimemode = 1;
pub const MOD_MESHCACHE_TIME_FACTOR: Meshcachemodifiertimemode = 2;

/// `MeshCacheModifierPlayMode`.
pub type Meshcachemodifierplaymode = i8;
pub const MOD_MESHCACHE_PLAY_CFEA: Meshcachemodifierplaymode = 0;
pub const MOD_MESHCACHE_PLAY_EVAL: Meshcachemodifierplaymode = 1;

/// `MeshCacheModifierFlipAxis`.
pub type Meshcachemodifierflipaxis = i8;
pub const MOD_MESHCACHE_FLIP_AXIS_X: Meshcachemodifierflipaxis = 1 << 0;
pub const MOD_MESHCACHE_FLIP_AXIS_Y: Meshcachemodifierflipaxis = 1 << 1;
pub const MOD_MESHCACHE_FLIP_AXIS_Z: Meshcachemodifierflipaxis = 1 << 2;

/// `LaplacianDeformModifierFlag`.
pub type Laplaciandeformmodifierflag = i16;
pub const MOD_LAPLACIANDEFORM_BIND: Laplaciandeformmodifierflag = 1 << 0;
pub const MOD_LAPLACIANDEFORM_INVERT_VGROUP: Laplaciandeformmodifierflag = 1 << 1;

/// `WireframeModifierFlag`.
pub type Wireframemodifierflag = i16;
pub const MOD_WIREFRAME_INVERT_VGROUP: Wireframemodifierflag = 1 << 0;
pub const MOD_WIREFRAME_REPLACE: Wireframemodifierflag = 1 << 1;
pub const MOD_WIREFRAME_BOUNDARY: Wireframemodifierflag = 1 << 2;
pub const MOD_WIREFRAME_OFS_EVEN: Wireframemodifierflag = 1 << 3;
pub const MOD_WIREFRAME_OFS_RELATIVE: Wireframemodifierflag = 1 << 4;
pub const MOD_WIREFRAME_CREASE: Wireframemodifierflag = 1 << 5;

/// `WeldModifierFlag`.
pub type Weldmodifierflag = i8;
pub const MOD_WELD_INVERT_VGROUP: Weldmodifierflag = 1 << 0;
pub const MOD_WELD_LOOSE_EDGES: Weldmodifierflag = 1 << 1;

/// `WeldModifierMode`.
pub type Weldmodifiermode = i8;
pub const MOD_WELD_MODE_ALL: Weldmodifiermode = 0;
pub const MOD_WELD_MODE_CONNECTED: Weldmodifiermode = 1;

/// `DataTransferModifierFlag`.
pub type Datatransfermodifierflag = u32;
pub const MOD_DATATRANSFER_OBSRC_TRANSFORM: Datatransfermodifierflag = 1 << 0;
pub const MOD_DATATRANSFER_MAP_MAXDIST: Datatransfermodifierflag = 1 << 1;
pub const MOD_DATATRANSFER_INVERT_VGROUP: Datatransfermodifierflag = 1 << 2;
pub const MOD_DATATRANSFER_USE_VERT: Datatransfermodifierflag = 1 << 28;
pub const MOD_DATATRANSFER_USE_EDGE: Datatransfermodifierflag = 1 << 29;
pub const MOD_DATATRANSFER_USE_LOOP: Datatransfermodifierflag = 1 << 30;
pub const MOD_DATATRANSFER_USE_POLY: Datatransfermodifierflag = 1 << 31;

/// `NormalEditModifierMode`.
pub type Normaleditmodifiermode = i16;
pub const MOD_NORMALEDIT_MODE_RADIAL: Normaleditmodifiermode = 0;
pub const MOD_NORMALEDIT_MODE_DIRECTIONAL: Normaleditmodifiermode = 1;

/// `NormalEditModifierFlag`.
pub type Normaleditmodifierflag = i16;
pub const MOD_NORMALEDIT_INVERT_VGROUP: Normaleditmodifierflag = 1 << 0;
pub const MOD_NORMALEDIT_USE_DIRECTION_PARALLEL: Normaleditmodifierflag = 1 << 1;
pub const MOD_NORMALEDIT_NO_POLYNORS_FIX: Normaleditmodifierflag = 1 << 2;

/// `NormalEditModifierMixMode`.
pub type Normaleditmodifiermixmode = i16;
pub const MOD_NORMALEDIT_MIX_COPY: Normaleditmodifiermixmode = 0;
pub const MOD_NORMALEDIT_MIX_ADD: Normaleditmodifiermixmode = 1;
pub const MOD_NORMALEDIT_MIX_SUB: Normaleditmodifiermixmode = 2;
pub const MOD_NORMALEDIT_MIX_MUL: Normaleditmodifiermixmode = 3;

/// `MeshSeqCacheModifierReadFlag`.
pub type Meshseqcachemodifierreadflag = i8;
pub const MOD_MESHSEQ_READ_VERT: Meshseqcachemodifierreadflag = 1 << 0;
pub const MOD_MESHSEQ_READ_POLY: Meshseqcachemodifierreadflag = 1 << 1;
pub const MOD_MESHSEQ_READ_UV: Meshseqcachemodifierreadflag = 1 << 2;
pub const MOD_MESHSEQ_READ_COLOR: Meshseqcachemodifierreadflag = 1 << 3;
pub const MOD_MESHSEQ_INTERPOLATE_VERTICES: Meshseqcachemodifierreadflag = 1 << 4;
pub const MOD_MESHSEQ_READ_ATTRIBUTES: Meshseqcachemodifierreadflag = 1 << 5;

/// `SurfaceDeformModifierFlag`.
pub type Surfacedeformmodifierflag = i32;
pub const MOD_SDEF_BIND: Surfacedeformmodifierflag = 1 << 0;
pub const MOD_SDEF_INVERT_VGROUP: Surfacedeformmodifierflag = 1 << 1;
pub const MOD_SDEF_SPARSE_BIND: Surfacedeformmodifierflag = 1 << 2;

/// `SurfaceDeformModifierBindMode`.
pub type Surfacedeformmodifierbindmode = i32;
pub const MOD_SDEF_MODE_CORNER_TRIS: Surfacedeformmodifierbindmode = 0;
pub const MOD_SDEF_MODE_NGONS: Surfacedeformmodifierbindmode = 1;
pub const MOD_SDEF_MODE_CENTROID: Surfacedeformmodifierbindmode = 2;

/// `WeightedNormalModifierMode`.
pub type Weightednormalmodifiermode = i8;
pub const MOD_WEIGHTEDNORMAL_MODE_FACE: Weightednormalmodifiermode = 0;
pub const MOD_WEIGHTEDNORMAL_MODE_ANGLE: Weightednormalmodifiermode = 1;
pub const MOD_WEIGHTEDNORMAL_MODE_FACE_ANGLE: Weightednormalmodifiermode = 2;

/// `WeightedNormalModifierFlag`.
pub type Weightednormalmodifierflag = i8;
pub const MOD_WEIGHTEDNORMAL_KEEP_SHARP: Weightednormalmodifierflag = 1 << 0;
pub const MOD_WEIGHTEDNORMAL_INVERT_VGROUP: Weightednormalmodifierflag = 1 << 1;
pub const MOD_WEIGHTEDNORMAL_FACE_INFLUENCE: Weightednormalmodifierflag = 1 << 2;

/// `NodesModifierPanelFlag`.
pub type Nodesmodifierpanelflag = u32;
pub const NODES_MODIFIER_PANEL_OPEN: Nodesmodifierpanelflag = 1 << 0;

/// `NodesModifierBakeFlag`.
pub type Nodesmodifierbakeflag = u32;
pub const NODES_MODIFIER_BAKE_CUSTOM_SIMULATION_FRAME_RANGE: Nodesmodifierbakeflag = 1 << 0;
pub const NODES_MODIFIER_BAKE_CUSTOM_PATH: Nodesmodifierbakeflag = 1 << 1;

/// `NodesModifierBakeTarget`.
pub type Nodesmodifierbaketarget = i8;
pub const NODES_MODIFIER_BAKE_TARGET_INHERIT: Nodesmodifierbaketarget = 0;
pub const NODES_MODIFIER_BAKE_TARGET_PACKED: Nodesmodifierbaketarget = 1;
pub const NODES_MODIFIER_BAKE_TARGET_DISK: Nodesmodifierbaketarget = 2;

/// `NodesModifierBakeMode`.
pub type Nodesmodifierbakemode = u8;
pub const NODES_MODIFIER_BAKE_MODE_ANIMATION: Nodesmodifierbakemode = 0;
pub const NODES_MODIFIER_BAKE_MODE_STILL: Nodesmodifierbakemode = 1;

/// `GeometryNodesModifierPanel`.
pub type Geometrynodesmodifierpanel = i32;
pub const NODES_MODIFIER_PANEL_OUTPUT_ATTRIBUTES: Geometrynodesmodifierpanel = 0;
pub const NODES_MODIFIER_PANEL_MANAGE: Geometrynodesmodifierpanel = 1;
pub const NODES_MODIFIER_PANEL_BAKE: Geometrynodesmodifierpanel = 2;
pub const NODES_MODIFIER_PANEL_NAMED_ATTRIBUTES: Geometrynodesmodifierpanel = 3;
pub const NODES_MODIFIER_PANEL_BAKE_DATA_BLOCKS: Geometrynodesmodifierpanel = 4;
pub const NODES_MODIFIER_PANEL_WARNINGS: Geometrynodesmodifierpanel = 5;

/// `NodesModifierFlag`.
pub type Nodesmodifierflag = i8;
pub const NODES_MODIFIER_HIDE_DATABLOCK_SELECTOR: Nodesmodifierflag = 1 << 0;
pub const NODES_MODIFIER_HIDE_MANAGE_PANEL: Nodesmodifierflag = 1 << 1;

/// `MeshToVolumeModifierResolutionMode`.
pub type Meshtovolumemodifierresolutionmode = i32;
pub const MESH_TO_VOLUME_RESOLUTION_MODE_VOXEL_AMOUNT: Meshtovolumemodifierresolutionmode = 0;
pub const MESH_TO_VOLUME_RESOLUTION_MODE_VOXEL_SIZE: Meshtovolumemodifierresolutionmode = 1;

/// `VolumeDisplaceModifierTextureMapMode`.
pub type Volumedisplacemodifiertexturemapmode = i32;
pub const MOD_VOLUME_DISPLACE_MAP_LOCAL: Volumedisplacemodifiertexturemapmode = 0;
pub const MOD_VOLUME_DISPLACE_MAP_GLOBAL: Volumedisplacemodifiertexturemapmode = 1;
pub const MOD_VOLUME_DISPLACE_MAP_OBJECT: Volumedisplacemodifiertexturemapmode = 2;

/// `VolumeToMeshResolutionMode`.
pub type Volumetomeshresolutionmode = i32;
pub const VOLUME_TO_MESH_RESOLUTION_MODE_GRID: Volumetomeshresolutionmode = 0;
pub const VOLUME_TO_MESH_RESOLUTION_MODE_VOXEL_AMOUNT: Volumetomeshresolutionmode = 1;
pub const VOLUME_TO_MESH_RESOLUTION_MODE_VOXEL_SIZE: Volumetomeshresolutionmode = 2;

/// `VolumeToMeshFlag`.
pub type Volumetomeshflag = u32;
pub const VOLUME_TO_MESH_USE_SMOOTH_SHADE: Volumetomeshflag = 1 << 0;

/// `GreasePencilModifierInfluenceFlag`.
pub type Greasepencilmodifierinfluenceflag = i32;
pub const GREASE_PENCIL_INFLUENCE_INVERT_LAYER_FILTER: Greasepencilmodifierinfluenceflag = 1 << 0;
pub const GREASE_PENCIL_INFLUENCE_USE_LAYER_PASS_FILTER: Greasepencilmodifierinfluenceflag = 1 << 1;
pub const GREASE_PENCIL_INFLUENCE_INVERT_LAYER_PASS_FILTER: Greasepencilmodifierinfluenceflag = 1 << 2;
pub const GREASE_PENCIL_INFLUENCE_INVERT_MATERIAL_FILTER: Greasepencilmodifierinfluenceflag = 1 << 3;
pub const GREASE_PENCIL_INFLUENCE_USE_MATERIAL_PASS_FILTER: Greasepencilmodifierinfluenceflag = 1 << 4;
pub const GREASE_PENCIL_INFLUENCE_INVERT_MATERIAL_PASS_FILTER: Greasepencilmodifierinfluenceflag = 1 << 5;
pub const GREASE_PENCIL_INFLUENCE_INVERT_VERTEX_GROUP: Greasepencilmodifierinfluenceflag = 1 << 6;
pub const GREASE_PENCIL_INFLUENCE_USE_CUSTOM_CURVE: Greasepencilmodifierinfluenceflag = 1 << 7;
pub const GREASE_PENCIL_INFLUENCE_USE_LAYER_GROUP_FILTER: Greasepencilmodifierinfluenceflag = 1 << 8;

/// `GreasePencilModifierColorMode`.
pub type Greasepencilmodifiercolormode = i8;
pub const MOD_GREASE_PENCIL_COLOR_STROKE: Greasepencilmodifiercolormode = 0;
pub const MOD_GREASE_PENCIL_COLOR_FILL: Greasepencilmodifiercolormode = 1;
pub const MOD_GREASE_PENCIL_COLOR_BOTH: Greasepencilmodifiercolormode = 2;
pub const MOD_GREASE_PENCIL_COLOR_HARDNESS: Greasepencilmodifiercolormode = 3;

/// `GreasePencilOpacityModifierFlag`.
pub type Greasepencilopacitymodifierflag = i32;
pub const MOD_GREASE_PENCIL_OPACITY_USE_WEIGHT_AS_FACTOR: Greasepencilopacitymodifierflag = 1 << 0;
pub const MOD_GREASE_PENCIL_OPACITY_USE_UNIFORM_OPACITY: Greasepencilopacitymodifierflag = 1 << 1;

/// `GreasePencilSubdivideType`.
pub type Greasepencilsubdividetype = i32;
pub const MOD_GREASE_PENCIL_SUBDIV_CATMULL: Greasepencilsubdividetype = 0;
pub const MOD_GREASE_PENCIL_SUBDIV_SIMPLE: Greasepencilsubdividetype = 1;

/// `GreasePencilTintModifierMode`.
pub type Greasepenciltintmodifiermode = i8;
pub const MOD_GREASE_PENCIL_TINT_UNIFORM: Greasepenciltintmodifiermode = 0;
pub const MOD_GREASE_PENCIL_TINT_GRADIENT: Greasepenciltintmodifiermode = 1;

/// `GreasePencilTintModifierFlag`.
pub type Greasepenciltintmodifierflag = i16;
pub const MOD_GREASE_PENCIL_TINT_USE_WEIGHT_AS_FACTOR: Greasepenciltintmodifierflag = 1 << 0;

/// `eGreasePencilSmooth_Flag`.
pub type GreasepencilsmoothFlag = i32;
pub const MOD_GREASE_PENCIL_SMOOTH_MOD_LOCATION: GreasepencilsmoothFlag = 1 << 0;
pub const MOD_GREASE_PENCIL_SMOOTH_MOD_STRENGTH: GreasepencilsmoothFlag = 1 << 1;
pub const MOD_GREASE_PENCIL_SMOOTH_MOD_THICKNESS: GreasepencilsmoothFlag = 1 << 2;
pub const MOD_GREASE_PENCIL_SMOOTH_MOD_UV: GreasepencilsmoothFlag = 1 << 3;
pub const MOD_GREASE_PENCIL_SMOOTH_KEEP_SHAPE: GreasepencilsmoothFlag = 1 << 4;
pub const MOD_GREASE_PENCIL_SMOOTH_SMOOTH_ENDS: GreasepencilsmoothFlag = 1 << 5;

/// `GreasePencilOffsetModifierFlag`.
pub type Greasepenciloffsetmodifierflag = i32;
pub const MOD_GREASE_PENCIL_OFFSET_UNIFORM_RANDOM_SCALE: Greasepenciloffsetmodifierflag = 1 << 0;

/// `GreasePencilOffsetModifierMode`.
pub type Greasepenciloffsetmodifiermode = i32;
pub const MOD_GREASE_PENCIL_OFFSET_RANDOM: Greasepenciloffsetmodifiermode = 0;
pub const MOD_GREASE_PENCIL_OFFSET_LAYER: Greasepenciloffsetmodifiermode = 1;
pub const MOD_GREASE_PENCIL_OFFSET_MATERIAL: Greasepenciloffsetmodifiermode = 2;
pub const MOD_GREASE_PENCIL_OFFSET_STROKE: Greasepenciloffsetmodifiermode = 3;

/// `GreasePencilMirrorModifierFlag`.
pub type Greasepencilmirrormodifierflag = i32;
pub const MOD_GREASE_PENCIL_MIRROR_AXIS_X: Greasepencilmirrormodifierflag = 1 << 0;
pub const MOD_GREASE_PENCIL_MIRROR_AXIS_Y: Greasepencilmirrormodifierflag = 1 << 1;
pub const MOD_GREASE_PENCIL_MIRROR_AXIS_Z: Greasepencilmirrormodifierflag = 1 << 2;

/// `GreasePencilThicknessModifierFlag`.
pub type Greasepencilthicknessmodifierflag = i32;
pub const MOD_GREASE_PENCIL_THICK_NORMALIZE: Greasepencilthicknessmodifierflag = 1 << 0;
pub const MOD_GREASE_PENCIL_THICK_WEIGHT_FACTOR: Greasepencilthicknessmodifierflag = 1 << 1;

/// `GreasePencilDashModifierFlag`.
pub type Greasepencildashmodifierflag = i32;
pub const MOD_GREASE_PENCIL_DASH_USE_CYCLIC: Greasepencildashmodifierflag = 1 << 0;

/// `GreasePencilMultiplyModifierFlag`.
pub type Greasepencilmultiplymodifierflag = i32;
pub const MOD_GREASE_PENCIL_MULTIPLY_ENABLE_FADING: Greasepencilmultiplymodifierflag = 1 << 2;

/// `GreasePencilWeightAngleModifierFlag`.
pub type Greasepencilweightanglemodifierflag = i32;
pub const MOD_GREASE_PENCIL_WEIGHT_ANGLE_MULTIPLY_DATA: Greasepencilweightanglemodifierflag = 1 << 5;
pub const MOD_GREASE_PENCIL_WEIGHT_ANGLE_INVERT_OUTPUT: Greasepencilweightanglemodifierflag = 1 << 6;

/// `GreasePencilWeightAngleModifierSpace`.
pub type Greasepencilweightanglemodifierspace = i16;
pub const MOD_GREASE_PENCIL_WEIGHT_ANGLE_SPACE_LOCAL: Greasepencilweightanglemodifierspace = 0;
pub const MOD_GREASE_PENCIL_WEIGHT_ANGLE_SPACE_WORLD: Greasepencilweightanglemodifierspace = 1;

/// `GreasePencilArrayModifierFlag`.
pub type Greasepencilarraymodifierflag = i32;
pub const MOD_GREASE_PENCIL_ARRAY_USE_OFFSET: Greasepencilarraymodifierflag = 1 << 7;
pub const MOD_GREASE_PENCIL_ARRAY_USE_RELATIVE: Greasepencilarraymodifierflag = 1 << 8;
pub const MOD_GREASE_PENCIL_ARRAY_USE_OB_OFFSET: Greasepencilarraymodifierflag = 1 << 9;
pub const MOD_GREASE_PENCIL_ARRAY_UNIFORM_RANDOM_SCALE: Greasepencilarraymodifierflag = 1 << 10;

/// `GreasePencilHookFlag`.
pub type Greasepencilhookflag = i32;
pub const MOD_GREASE_PENCIL_HOOK_UNIFORM_SPACE: Greasepencilhookflag = 1 << 0;

/// `GreasePencilHookFalloff`.
pub type Greasepencilhookfalloff = i8;
pub const MOD_GREASE_PENCIL_HOOK_Falloff_None: Greasepencilhookfalloff = 0;
pub const MOD_GREASE_PENCIL_HOOK_Falloff_Curve: Greasepencilhookfalloff = 1;
pub const MOD_GREASE_PENCIL_HOOK_Falloff_Sharp: Greasepencilhookfalloff = 2;
pub const MOD_GREASE_PENCIL_HOOK_Falloff_Smooth: Greasepencilhookfalloff = 3;
pub const MOD_GREASE_PENCIL_HOOK_Falloff_Root: Greasepencilhookfalloff = 4;
pub const MOD_GREASE_PENCIL_HOOK_Falloff_Linear: Greasepencilhookfalloff = 5;
pub const MOD_GREASE_PENCIL_HOOK_Falloff_Const: Greasepencilhookfalloff = 6;
pub const MOD_GREASE_PENCIL_HOOK_Falloff_Sphere: Greasepencilhookfalloff = 7;
pub const MOD_GREASE_PENCIL_HOOK_Falloff_InvSquare: Greasepencilhookfalloff = 8;

/// `eGreasePencilLineartFlags`.
pub type Greasepencillineartflags = i32;
pub const LINEART_GPENCIL_BINARY_WEIGHTS: Greasepencillineartflags = 1
                                    << 2;
pub const LINEART_GPENCIL_IS_BAKED: Greasepencillineartflags = 1 << 3;
pub const LINEART_GPENCIL_USE_CACHE: Greasepencillineartflags = 1 << 4;
pub const LINEART_GPENCIL_OFFSET_TOWARDS_CUSTOM_CAMERA: Greasepencillineartflags = 1 << 5;
pub const LINEART_GPENCIL_INVERT_COLLECTION: Greasepencillineartflags = 1 << 6;
pub const LINEART_GPENCIL_INVERT_SILHOUETTE_FILTER: Greasepencillineartflags = 1 << 7;

/// `GreasePencilLineartModifierSource`.
pub type Greasepencillineartmodifiersource = i8;
pub const LINEART_SOURCE_COLLECTION: Greasepencillineartmodifiersource = 0;
pub const LINEART_SOURCE_OBJECT: Greasepencillineartmodifiersource = 1;
pub const LINEART_SOURCE_SCENE: Greasepencillineartmodifiersource = 2;

/// `GreasePencilLineartModifierShadowFilter`.
pub type Greasepencillineartmodifiershadowfilter = i32;
pub const LINEART_SHADOW_FILTER_NONE: Greasepencillineartmodifiershadowfilter = 0;
pub const LINEART_SHADOW_FILTER_ILLUMINATED: Greasepencillineartmodifiershadowfilter = 1;
pub const LINEART_SHADOW_FILTER_SHADED: Greasepencillineartmodifiershadowfilter = 2;
pub const LINEART_SHADOW_FILTER_ILLUMINATED_ENCLOSED_SHAPES: Greasepencillineartmodifiershadowfilter = 3;

/// `GreasePencilLineartMaskSwitches`.
pub type Greasepencillineartmaskswitches = u8;
pub const MOD_LINEART_MATERIAL_MASK_ENABLE: Greasepencillineartmaskswitches = 1 << 0;
pub const MOD_LINEART_MATERIAL_MASK_MATCH: Greasepencillineartmaskswitches = 1 << 1;
pub const MOD_LINEART_INTERSECTION_MATCH: Greasepencillineartmaskswitches = 1 << 2;

/// `eGreasePencilLineartMaskSwitches`.
pub type EGreasepencillineartmaskswitches = i8;
pub const LINEART_GPENCIL_MATERIAL_MASK_ENABLE: EGreasepencillineartmaskswitches = 1 << 0;
pub const LINEART_GPENCIL_MATERIAL_MASK_MATCH: EGreasepencillineartmaskswitches = 1 << 1;
pub const LINEART_GPENCIL_INTERSECTION_MATCH: EGreasepencillineartmaskswitches = 1 << 2;

/// `eGreasePencilLineartSilhouetteFilter`.
pub type Greasepencillineartsilhouettefilter = i32;
pub const LINEART_SILHOUETTE_FILTER_NONE: Greasepencillineartsilhouettefilter = 0;
pub const LINEART_SILHOUETTE_FILTER_GROUP: Greasepencillineartsilhouettefilter = 1 << 0;
pub const LINEART_SILHOUETTE_FILTER_INDIVIDUAL: Greasepencillineartsilhouettefilter = 1 << 1;

/// `GreasePencilTimeModifierFlag`.
pub type Greasepenciltimemodifierflag = i32;
pub const MOD_GREASE_PENCIL_TIME_KEEP_LOOP: Greasepenciltimemodifierflag = 1 << 0;
pub const MOD_GREASE_PENCIL_TIME_CUSTOM_RANGE: Greasepenciltimemodifierflag = 1 << 1;

/// `GreasePencilTimeModifierMode`.
pub type Greasepenciltimemodifiermode = i32;
pub const MOD_GREASE_PENCIL_TIME_MODE_NORMAL: Greasepenciltimemodifiermode = 0;
pub const MOD_GREASE_PENCIL_TIME_MODE_REVERSE: Greasepenciltimemodifiermode = 1;
pub const MOD_GREASE_PENCIL_TIME_MODE_FIX: Greasepenciltimemodifiermode = 2;
pub const MOD_GREASE_PENCIL_TIME_MODE_PINGPONG: Greasepenciltimemodifiermode = 3;
pub const MOD_GREASE_PENCIL_TIME_MODE_CHAIN: Greasepenciltimemodifiermode = 4;

/// `GreasePencilTimeModifierSegmentMode`.
pub type Greasepenciltimemodifiersegmentmode = i32;
pub const MOD_GREASE_PENCIL_TIME_SEG_MODE_NORMAL: Greasepenciltimemodifiersegmentmode = 0;
pub const MOD_GREASE_PENCIL_TIME_SEG_MODE_REVERSE: Greasepenciltimemodifiersegmentmode = 1;
pub const MOD_GREASE_PENCIL_TIME_SEG_MODE_PINGPONG: Greasepenciltimemodifiersegmentmode = 2;

/// `GreasePencilEnvelopeModifierMode`.
pub type Greasepencilenvelopemodifiermode = i32;
pub const MOD_GREASE_PENCIL_ENVELOPE_DEFORM: Greasepencilenvelopemodifiermode = 0;
pub const MOD_GREASE_PENCIL_ENVELOPE_SEGMENTS: Greasepencilenvelopemodifiermode = 1;
pub const MOD_GREASE_PENCIL_ENVELOPE_FILLS: Greasepencilenvelopemodifiermode = 2;

/// `GreasePencilOutlineModifierFlag`.
pub type Greasepenciloutlinemodifierflag = i32;
pub const MOD_GREASE_PENCIL_OUTLINE_KEEP_SHAPE: Greasepenciloutlinemodifierflag = 1 << 0;

/// `GreasePencilBuildMode`.
pub type Greasepencilbuildmode = i16;
pub const MOD_GREASE_PENCIL_BUILD_MODE_SEQUENTIAL: Greasepencilbuildmode = 0;
pub const MOD_GREASE_PENCIL_BUILD_MODE_CONCURRENT: Greasepencilbuildmode = 1;
pub const MOD_GREASE_PENCIL_BUILD_MODE_ADDITIVE: Greasepencilbuildmode = 2;

/// `GreasePencilBuildTransition`.
pub type Greasepencilbuildtransition = i16;
pub const MOD_GREASE_PENCIL_BUILD_TRANSITION_GROW: Greasepencilbuildtransition = 0;
pub const MOD_GREASE_PENCIL_BUILD_TRANSITION_SHRINK: Greasepencilbuildtransition = 1;
pub const MOD_GREASE_PENCIL_BUILD_TRANSITION_VANISH: Greasepencilbuildtransition = 2;

/// `GreasePencilBuildTimeAlignment`.
pub type Greasepencilbuildtimealignment = i16;
pub const MOD_GREASE_PENCIL_BUILD_TIMEALIGN_START: Greasepencilbuildtimealignment = 0;
pub const MOD_GREASE_PENCIL_BUILD_TIMEALIGN_END: Greasepencilbuildtimealignment = 1;

/// `GreasePencilBuildTimeMode`.
pub type Greasepencilbuildtimemode = i16;
pub const MOD_GREASE_PENCIL_BUILD_TIMEMODE_FRAMES: Greasepencilbuildtimemode = 0;
pub const MOD_GREASE_PENCIL_BUILD_TIMEMODE_PERCENTAGE: Greasepencilbuildtimemode = 1;
pub const MOD_GREASE_PENCIL_BUILD_TIMEMODE_DRAWSPEED: Greasepencilbuildtimemode = 2;

/// `GreasePencilBuildFlag`.
pub type Greasepencilbuildflag = i16;
pub const MOD_GREASE_PENCIL_BUILD_RESTRICT_TIME: Greasepencilbuildflag = 1 << 0;
pub const MOD_GREASE_PENCIL_BUILD_USE_FADING: Greasepencilbuildflag = 1 << 14;

/// `GreasePencilSimplifyModifierMode`.
pub type Greasepencilsimplifymodifiermode = i16;
pub const MOD_GREASE_PENCIL_SIMPLIFY_FIXED: Greasepencilsimplifymodifiermode = 0;
pub const MOD_GREASE_PENCIL_SIMPLIFY_ADAPTIVE: Greasepencilsimplifymodifiermode = 1;
pub const MOD_GREASE_PENCIL_SIMPLIFY_SAMPLE: Greasepencilsimplifymodifiermode = 2;
pub const MOD_GREASE_PENCIL_SIMPLIFY_MERGE: Greasepencilsimplifymodifiermode = 3;

/// `GreasePencilTextureModifierFit`.
pub type Greasepenciltexturemodifierfit = i16;
pub const MOD_GREASE_PENCIL_TEXTURE_FIT_STROKE: Greasepenciltexturemodifierfit = 0;
pub const MOD_GREASE_PENCIL_TEXTURE_CONSTANT_LENGTH: Greasepenciltexturemodifierfit = 1;

/// `GreasePencilTextureModifierMode`.
pub type Greasepenciltexturemodifiermode = i16;
pub const MOD_GREASE_PENCIL_TEXTURE_STROKE: Greasepenciltexturemodifiermode = 0;
pub const MOD_GREASE_PENCIL_TEXTURE_FILL: Greasepenciltexturemodifiermode = 1;
pub const MOD_GREASE_PENCIL_TEXTURE_STROKE_AND_FILL: Greasepenciltexturemodifiermode = 2;

/// `ModifierData`.
#[derive(Clone)]
pub struct ModifierData {
    /// `ModifierData *next`
    pub next: usize,
    /// `ModifierData *prev`
    pub prev: usize,
    /// `ModifierType type`
    pub type_: Modifiertype,
    /// `ModifierMode mode`
    pub mode: Modifiermode,
    /// `float execution_time`
    pub execution_time: f32,
    /// `ModifierFlag flag`
    pub flag: Modifierflag,
    /// `short ui_expand_flag`
    pub ui_expand_flag: i16,
    /// `uint16_t layout_panel_open_flag`
    pub layout_panel_open_flag: u16,
    /// `int persistent_uid`
    pub persistent_uid: i32,
    /// `char name[64]`
    pub name: [u8; 64],
    /// `char *error`
    pub error: usize,
    /// `IDProperty *system_properties`
    pub system_properties: usize,
    /// `void *runtime`
    pub runtime: usize,
}
impl Default for ModifierData {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            type_: eModifierType_None,
            mode: Default::default(),
            execution_time: 0.0,
            flag: Default::default(),
            ui_expand_flag: 0,
            layout_panel_open_flag: 0,
            persistent_uid: 0,
            name: [0; 64],
            error: NIL,
            system_properties: NIL,
            runtime: NIL,
        }
    }
}

/// `MappingInfoModifierData`.
#[derive(Clone)]
pub struct MappingInfoModifierData {
    pub modifier: ModifierData,
    /// `Tex *texture`
    pub texture: usize,
    /// `Object *map_object`
    pub map_object: usize,
    /// `char map_bone[64]`
    pub map_bone: [u8; 64],
    /// `char uvlayer_name[68]`
    pub uvlayer_name: [u8; 68],
    /// `int uvlayer_tmp`
    pub uvlayer_tmp: i32,
    /// `int texmapping`
    pub texmapping: i32,
}
impl Default for MappingInfoModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            texture: NIL,
            map_object: NIL,
            map_bone: [0; 64],
            uvlayer_name: [0; 68],
            uvlayer_tmp: 0,
            texmapping: 0,
        }
    }
}

/// `SubsurfModifierData`.
#[derive(Clone)]
pub struct SubsurfModifierData {
    pub modifier: ModifierData,
    /// `short subdivType`
    pub subdiv_type: i16,
    /// `short levels`
    pub levels: i16,
    /// `short renderLevels`
    pub render_levels: i16,
    /// `SubsurfModifierFlag flags`
    pub flags: Subsurfmodifierflag,
    /// `eSubsurfUVSmooth uv_smooth`
    pub uv_smooth: Subsurfuvsmooth,
    /// `short quality`
    pub quality: i16,
    /// `eSubsurfBoundarySmooth boundary_smooth`
    pub boundary_smooth: Subsurfboundarysmooth,
    /// `eSubsurfAdaptiveSpace adaptive_space`
    pub adaptive_space: Subsurfadaptivespace,
    /// `float adaptive_pixel_size`
    pub adaptive_pixel_size: f32,
    /// `float adaptive_object_edge_length`
    pub adaptive_object_edge_length: f32,
}
impl Default for SubsurfModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            subdiv_type: 0,
            levels: 1,
            render_levels: 2,
            flags: eSubsurfModifierFlag_UseCrease | eSubsurfModifierFlag_ControlEdges,
            uv_smooth: SUBSURF_UV_SMOOTH_PRESERVE_BOUNDARIES,
            quality: 3,
            boundary_smooth: SUBSURF_BOUNDARY_SMOOTH_ALL,
            adaptive_space: SUBSURF_ADAPTIVE_SPACE_PIXEL,
            adaptive_pixel_size: 1.0,
            adaptive_object_edge_length: 0.01,
        }
    }
}

/// `LatticeModifierData`.
#[derive(Clone)]
pub struct LatticeModifierData {
    pub modifier: ModifierData,
    /// `Object *object`
    pub object: usize,
    /// `char name[64]`
    pub name: [u8; 64],
    /// `float strength`
    pub strength: f32,
    /// `LatticeModifierFlag flag`
    pub flag: Latticemodifierflag,
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for LatticeModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            object: NIL,
            name: [0; 64],
            strength: 1.0,
            flag: Default::default(),
            pad1: NIL,
        }
    }
}

/// `CurveModifierData`.
#[derive(Clone)]
pub struct CurveModifierData {
    pub modifier: ModifierData,
    /// `Object *object`
    pub object: usize,
    /// `char name[64]`
    pub name: [u8; 64],
    /// `CurveModifierDefaultAxis defaxis`
    pub defaxis: Curvemodifierdefaultaxis,
    /// `CurveModifierFlag flag`
    pub flag: Curvemodifierflag,
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for CurveModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            object: NIL,
            name: [0; 64],
            defaxis: MOD_CURVE_POSX,
            flag: Default::default(),
            pad1: NIL,
        }
    }
}

/// `BuildModifierData`.
#[derive(Clone)]
pub struct BuildModifierData {
    pub modifier: ModifierData,
    /// `float start`
    pub start: f32,
    /// `float length`
    pub length: f32,
    /// `BuildModifierFlag flag`
    pub flag: Buildmodifierflag,
    /// `short randomize`
    pub randomize: i16,
    /// `int seed`
    pub seed: i32,
}
impl Default for BuildModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            start: 1.0,
            length: 100.0,
            flag: Default::default(),
            randomize: 0,
            seed: 0,
        }
    }
}

/// `MaskModifierData`.
#[derive(Clone)]
pub struct MaskModifierData {
    pub modifier: ModifierData,
    /// `Object *ob_arm`
    pub ob_arm: usize,
    /// `char vgroup[64]`
    pub vgroup: [u8; 64],
    /// `MaskModifierMode mode`
    pub mode: Maskmodifiermode,
    /// `MaskModifierFlag flag`
    pub flag: Maskmodifierflag,
    /// `float threshold`
    pub threshold: f32,
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for MaskModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            ob_arm: NIL,
            vgroup: [0; 64],
            mode: MOD_MASK_MODE_VGROUP,
            flag: Default::default(),
            threshold: 0.0,
            pad1: NIL,
        }
    }
}

/// `ArrayModifierData`.
#[derive(Clone)]
pub struct ArrayModifierData {
    pub modifier: ModifierData,
    /// `Object *start_cap`
    pub start_cap: usize,
    /// `Object *end_cap`
    pub end_cap: usize,
    /// `Object *curve_ob`
    pub curve_ob: usize,
    /// `Object *offset_ob`
    pub offset_ob: usize,
    /// `float offset[3]`
    pub offset: [f32; 3],
    /// `float scale[3]`
    pub scale: [f32; 3],
    /// `float length`
    pub length: f32,
    /// `float merge_dist`
    pub merge_dist: f32,
    /// `ArrayModifierFitType fit_type`
    pub fit_type: Arraymodifierfittype,
    /// `ArrayModifierOffsetType offset_type`
    pub offset_type: Arraymodifieroffsettype,
    /// `ArrayModifierFlag flags`
    pub flags: Arraymodifierflag,
    /// `int count`
    pub count: i32,
    /// `float uv_offset[2]`
    pub uv_offset: [f32; 2],
}
impl Default for ArrayModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            start_cap: NIL,
            end_cap: NIL,
            curve_ob: NIL,
            offset_ob: NIL,
            offset: [0.0; 3],
            scale: [0.0; 3],
            length: 0.0,
            merge_dist: 0.01,
            fit_type: MOD_ARR_FIXEDCOUNT,
            offset_type: MOD_ARR_OFF_RELATIVE,
            flags: Default::default(),
            count: 2,
            uv_offset: [0.0; 2],
        }
    }
}

/// `MirrorModifierData`.
#[derive(Clone)]
pub struct MirrorModifierData {
    pub modifier: ModifierData,
    /// `short axis`
    pub axis: i16,
    /// `MirrorModifierFlag flag`
    pub flag: Mirrormodifierflag,
    /// `float tolerance`
    pub tolerance: f32,
    /// `float bisect_threshold`
    pub bisect_threshold: f32,
    /// `uint8_t use_correct_order_on_merge`
    pub use_correct_order_on_merge: u8,
    /// `float uv_offset[2]`
    pub uv_offset: [f32; 2],
    /// `float uv_offset_copy[2]`
    pub uv_offset_copy: [f32; 2],
    /// `Object *mirror_ob`
    pub mirror_ob: usize,
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for MirrorModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            axis: 0,
            flag: MOD_MIR_AXIS_X | MOD_MIR_VGROUP,
            tolerance: 0.001,
            bisect_threshold: 0.001,
            use_correct_order_on_merge: 1,
            uv_offset: [0.0; 2],
            uv_offset_copy: [0.0; 2],
            mirror_ob: NIL,
            pad1: NIL,
        }
    }
}

/// `EdgeSplitModifierData`.
#[derive(Clone)]
pub struct EdgeSplitModifierData {
    pub modifier: ModifierData,
    /// `float split_angle`
    pub split_angle: f32,
    /// `EdgeSplitModifierFlag flags`
    pub flags: Edgesplitmodifierflag,
}
impl Default for EdgeSplitModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            split_angle: (30.0f32).to_radians(),
            flags: MOD_EDGESPLIT_FROMANGLE | MOD_EDGESPLIT_FROMFLAG,
        }
    }
}

/// `BevelModifierData`.
#[derive(Clone)]
pub struct BevelModifierData {
    pub modifier: ModifierData,
    /// `float value`
    pub value: f32,
    /// `int res`
    pub res: i32,
    /// `BevelModifierFlag flags`
    pub flags: Bevelmodifierflag,
    /// `BevelModifierValFlag val_flags`
    pub val_flags: Bevelmodifiervalflag,
    /// `BevelModifierProfileType profile_type`
    pub profile_type: Bevelmodifierprofiletype,
    /// `BevelModifierFlag lim_flags`
    pub lim_flags: Bevelmodifierflag,
    /// `short e_flags`
    pub e_flags: i16,
    /// `short mat`
    pub mat: i16,
    /// `BevelModifierEdgeFlag edge_flags`
    pub edge_flags: Bevelmodifieredgeflag,
    /// `BevelModifierFaceStrengthMode face_str_mode`
    pub face_str_mode: Bevelmodifierfacestrengthmode,
    /// `BevelModifierMiter miter_inner`
    pub miter_inner: Bevelmodifiermiter,
    /// `BevelModifierMiter miter_outer`
    pub miter_outer: Bevelmodifiermiter,
    /// `BevelModifierVMeshMethod vmesh_method`
    pub vmesh_method: Bevelmodifiervmeshmethod,
    /// `BevelModifierAffectType affect_type`
    pub affect_type: Bevelmodifieraffecttype,
    /// `float profile`
    pub profile: f32,
    /// `float bevel_angle`
    pub bevel_angle: f32,
    /// `float spread`
    pub spread: f32,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `CurveProfile *custom_profile`
    pub custom_profile: usize,
    /// `char edge_weight_name[64]`
    pub edge_weight_name: [u8; 64],
    /// `char vertex_weight_name[64]`
    pub vertex_weight_name: [u8; 64],
}
impl Default for BevelModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            value: 0.1,
            res: 1,
            flags: Default::default(),
            val_flags: MOD_BEVEL_AMT_OFFSET,
            profile_type: MOD_BEVEL_PROFILE_SUPERELLIPSE,
            lim_flags: MOD_BEVEL_ANGLE,
            e_flags: 0,
            mat: -1,
            edge_flags: Default::default(),
            face_str_mode: MOD_BEVEL_FACE_STRENGTH_NONE,
            miter_inner: MOD_BEVEL_MITER_SHARP,
            miter_outer: MOD_BEVEL_MITER_SHARP,
            vmesh_method: Default::default(),
            affect_type: MOD_BEVEL_AFFECT_EDGES,
            profile: 0.5,
            bevel_angle: (30.0f32).to_radians(),
            spread: 0.1,
            defgrp_name: [0; 64],
            custom_profile: NIL,
            edge_weight_name: [0; 64],
            vertex_weight_name: [0; 64],
        }
    }
}

/// `FluidModifierData`.
#[derive(Clone)]
pub struct FluidModifierData {
    pub modifier: ModifierData,
    /// `FluidDomainSettings *domain`
    pub domain: usize,
    /// `FluidFlowSettings *flow`
    pub flow: usize,
    /// `FluidEffectorSettings *effector`
    pub effector: usize,
    /// `float time`
    pub time: f32,
    /// `FluidModifierType type`
    pub type_: Fluidmodifiertype,
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for FluidModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            domain: NIL,
            flow: NIL,
            effector: NIL,
            time: 0.0,
            type_: Default::default(),
            pad1: NIL,
        }
    }
}

/// `DisplaceModifierData`.
#[derive(Clone)]
pub struct DisplaceModifierData {
    pub modifier: ModifierData,
    /// `Tex *texture`
    pub texture: usize,
    /// `Object *map_object`
    pub map_object: usize,
    /// `char map_bone[64]`
    pub map_bone: [u8; 64],
    /// `char uvlayer_name[68]`
    pub uvlayer_name: [u8; 68],
    /// `int uvlayer_tmp`
    pub uvlayer_tmp: i32,
    /// `DisplaceModifierTexMapping texmapping`
    pub texmapping: Displacemodifiertexmapping,
    /// `float strength`
    pub strength: f32,
    /// `DisplaceModifierDirection direction`
    pub direction: Displacemodifierdirection,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `float midlevel`
    pub midlevel: f32,
    /// `DisplaceModifierSpace space`
    pub space: Displacemodifierspace,
    /// `DisplaceModifierFlag flag`
    pub flag: Displacemodifierflag,
}
impl Default for DisplaceModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            texture: NIL,
            map_object: NIL,
            map_bone: [0; 64],
            uvlayer_name: [0; 68],
            uvlayer_tmp: 0,
            texmapping: MOD_DISP_MAP_LOCAL,
            strength: 1.0,
            direction: MOD_DISP_DIR_NOR,
            defgrp_name: [0; 64],
            midlevel: 0.5,
            space: MOD_DISP_SPACE_LOCAL,
            flag: Default::default(),
        }
    }
}

/// `UVProjectModifierData`.
#[derive(Clone)]
pub struct UVProjectModifierData {
    pub modifier: ModifierData,
    /// `Object *projectors[]`
    pub projectors: [usize; 10],
    /// `int projectors_num`
    pub projectors_num: i32,
    /// `float aspectx`
    pub aspectx: f32,
    /// `float aspecty`
    pub aspecty: f32,
    /// `float scalex`
    pub scalex: f32,
    /// `float scaley`
    pub scaley: f32,
    /// `char uvlayer_name[68]`
    pub uvlayer_name: [u8; 68],
    /// `int uvlayer_tmp`
    pub uvlayer_tmp: i32,
}
impl Default for UVProjectModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            projectors: [NIL; 10],
            projectors_num: 1,
            aspectx: 1.0,
            aspecty: 1.0,
            scalex: 1.0,
            scaley: 1.0,
            uvlayer_name: [0; 68],
            uvlayer_tmp: 0,
        }
    }
}

/// `DecimateModifierData`.
#[derive(Clone)]
pub struct DecimateModifierData {
    pub modifier: ModifierData,
    /// `float percent`
    pub percent: f32,
    /// `short iter`
    pub iter: i16,
    /// `char delimit`
    pub delimit: i8,
    /// `char symmetry_axis`
    pub symmetry_axis: i8,
    /// `float angle`
    pub angle: f32,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `float defgrp_factor`
    pub defgrp_factor: f32,
    /// `DecimateModifierFlag flag`
    pub flag: Decimatemodifierflag,
    /// `DecimateModifierMode mode`
    pub mode: Decimatemodifiermode,
    /// `int face_count`
    pub face_count: i32,
}
impl Default for DecimateModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            percent: 1.0,
            iter: 0,
            delimit: 0,
            symmetry_axis: 0,
            angle: (5.0f32).to_radians(),
            defgrp_name: [0; 64],
            defgrp_factor: 1.0,
            flag: Default::default(),
            mode: MOD_DECIM_MODE_COLLAPSE,
            face_count: 0,
        }
    }
}

/// `SmoothModifierData`.
#[derive(Clone)]
pub struct SmoothModifierData {
    pub modifier: ModifierData,
    /// `float fac`
    pub fac: f32,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `SmoothModifierFlag flag`
    pub flag: Smoothmodifierflag,
    /// `short repeat`
    pub repeat: i16,
}
impl Default for SmoothModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            fac: 0.5,
            defgrp_name: [0; 64],
            flag: MOD_SMOOTH_X | MOD_SMOOTH_Y | MOD_SMOOTH_Z,
            repeat: 1,
        }
    }
}

/// `CastModifierData`.
#[derive(Clone)]
pub struct CastModifierData {
    pub modifier: ModifierData,
    /// `Object *object`
    pub object: usize,
    /// `float fac`
    pub fac: f32,
    /// `float radius`
    pub radius: f32,
    /// `float size`
    pub size: f32,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `CastModifierFlag flag`
    pub flag: Castmodifierflag,
    /// `CastModifierType type`
    pub type_: Castmodifiertype,
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for CastModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            object: NIL,
            fac: 0.5,
            radius: 0.0,
            size: 0.0,
            defgrp_name: [0; 64],
            flag: MOD_CAST_X | MOD_CAST_Y | MOD_CAST_Z | MOD_CAST_SIZE_FROM_RADIUS,
            type_: MOD_CAST_TYPE_SPHERE,
            pad1: NIL,
        }
    }
}

/// `WaveModifierData`.
#[derive(Clone)]
pub struct WaveModifierData {
    pub modifier: ModifierData,
    /// `Tex *texture`
    pub texture: usize,
    /// `Object *map_object`
    pub map_object: usize,
    /// `char map_bone[64]`
    pub map_bone: [u8; 64],
    /// `char uvlayer_name[68]`
    pub uvlayer_name: [u8; 68],
    /// `int uvlayer_tmp`
    pub uvlayer_tmp: i32,
    /// `DisplaceModifierTexMapping texmapping`
    pub texmapping: Displacemodifiertexmapping,
    /// `Object *objectcenter`
    pub objectcenter: usize,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `float startx`
    pub startx: f32,
    /// `float starty`
    pub starty: f32,
    /// `float height`
    pub height: f32,
    /// `float width`
    pub width: f32,
    /// `float narrow`
    pub narrow: f32,
    /// `float speed`
    pub speed: f32,
    /// `float damp`
    pub damp: f32,
    /// `float falloff`
    pub falloff: f32,
    /// `float timeoffs`
    pub timeoffs: f32,
    /// `float lifetime`
    pub lifetime: f32,
    /// `void *_pad4`
    pub pad4: usize,
}
impl Default for WaveModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            texture: NIL,
            map_object: NIL,
            map_bone: [0; 64],
            uvlayer_name: [0; 68],
            uvlayer_tmp: 0,
            texmapping: MOD_DISP_MAP_LOCAL,
            objectcenter: NIL,
            defgrp_name: [0; 64],
            startx: 0.0,
            starty: 0.0,
            height: 0.5,
            width: 1.5,
            narrow: 1.5,
            speed: 0.25,
            damp: 10.0,
            falloff: 0.0,
            timeoffs: 0.0,
            lifetime: 0.0,
            pad4: NIL,
        }
    }
}

/// `ArmatureModifierData`.
#[derive(Clone)]
pub struct ArmatureModifierData {
    pub modifier: ModifierData,
    /// `short deformflag`
    pub deformflag: i16,
    /// `short multi`
    pub multi: i16,
    /// `Object *object`
    pub object: usize,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
}
impl Default for ArmatureModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            deformflag: ARM_DEF_VGROUP,
            multi: 0,
            object: NIL,
            defgrp_name: [0; 64],
        }
    }
}

/// `HookModifierData`.
#[derive(Clone)]
pub struct HookModifierData {
    pub modifier: ModifierData,
    /// `Object *object`
    pub object: usize,
    /// `char subtarget[64]`
    pub subtarget: [u8; 64],
    /// `HookModifierFlag flag`
    pub flag: Hookmodifierflag,
    /// `HookModifierFalloff falloff_type`
    pub falloff_type: Hookmodifierfalloff,
    /// `float parentinv[4][4]`
    pub parentinv: [[f32; 4]; 4],
    /// `float cent[3]`
    pub cent: [f32; 3],
    /// `float falloff`
    pub falloff: f32,
    /// `CurveMapping *curfalloff`
    pub curfalloff: usize,
    /// `int *indexar`
    pub indexar: usize,
    /// `int indexar_num`
    pub indexar_num: i32,
    /// `float force`
    pub force: f32,
    /// `char name[64]`
    pub name: [u8; 64],
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for HookModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            object: NIL,
            subtarget: [0; 64],
            flag: Default::default(),
            falloff_type: eHook_Falloff_Smooth,
            parentinv: [[0.0; 4]; 4],
            cent: [0.0; 3],
            falloff: 0.0,
            curfalloff: NIL,
            indexar: NIL,
            indexar_num: 0,
            force: 1.0,
            name: [0; 64],
            pad1: NIL,
        }
    }
}

/// `SoftbodyModifierData`.
#[derive(Clone)]
pub struct SoftbodyModifierData {
    pub modifier: ModifierData,
}
impl Default for SoftbodyModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
        }
    }
}

/// `ClothModifierData`.
#[derive(Clone)]
pub struct ClothModifierData {
    pub modifier: ModifierData,
    /// `Cloth *clothObject`
    pub cloth_object: usize,
    /// `ClothSimSettings *sim_parms`
    pub sim_parms: usize,
    /// `ClothCollSettings *coll_parms`
    pub coll_parms: usize,
    /// `PointCache *point_cache`
    pub point_cache: usize,
    /// `ClothHairData *hairdata`
    pub hairdata: usize,
    /// `float hair_grid_min[3]`
    pub hair_grid_min: [f32; 3],
    /// `float hair_grid_max[3]`
    pub hair_grid_max: [f32; 3],
    /// `int hair_grid_res[3]`
    pub hair_grid_res: [i32; 3],
    /// `float hair_grid_cellsize`
    pub hair_grid_cellsize: f32,
    /// `ClothSolverResult *solver_result`
    pub solver_result: usize,
}
impl Default for ClothModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            cloth_object: NIL,
            sim_parms: NIL,
            coll_parms: NIL,
            point_cache: NIL,
            hairdata: NIL,
            hair_grid_min: [0.0; 3],
            hair_grid_max: [0.0; 3],
            hair_grid_res: [0; 3],
            hair_grid_cellsize: 0.0,
            solver_result: NIL,
        }
    }
}

/// `CollisionModifierData`.
#[derive(Clone)]
pub struct CollisionModifierData {
    pub modifier: ModifierData,
    /// `unsigned_int mvert_num`
    pub mvert_num: u32,
    /// `unsigned_int tri_num`
    pub tri_num: u32,
    /// `float time_x`
    pub time_x: f32,
    /// `float time_xnew`
    pub time_xnew: f32,
    /// `char is_static`
    pub is_static: i8,
    /// `BVHTree *bvhtree`
    pub bvhtree: usize,
}
impl Default for CollisionModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            mvert_num: 0,
            tri_num: 0,
            time_x: -1000.0,
            time_xnew: -1000.0,
            is_static: 0,
            bvhtree: NIL,
        }
    }
}

/// `SurfaceModifierData_Runtime`.
#[derive(Clone)]
pub struct SurfaceModifierData_Runtime {
    /// `Mesh *mesh`
    pub mesh: usize,
    /// `int cfra_prev`
    pub cfra_prev: i32,
    /// `int verts_num`
    pub verts_num: i32,
}
impl Default for SurfaceModifierData_Runtime {
    fn default() -> Self {
        Self {
            mesh: NIL,
            cfra_prev: 0,
            verts_num: 0,
        }
    }
}

/// `SurfaceModifierData`.
#[derive(Clone)]
pub struct SurfaceModifierData {
    pub modifier: ModifierData,
}
impl Default for SurfaceModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
        }
    }
}

/// `BooleanModifierData`.
#[derive(Clone)]
pub struct BooleanModifierData {
    pub modifier: ModifierData,
    /// `Object *object`
    pub object: usize,
    /// `Collection *collection`
    pub collection: usize,
    /// `float double_threshold`
    pub double_threshold: f32,
    /// `BooleanModifierOp operation`
    pub operation: Booleanmodifierop,
    /// `BooleanModifierSolver solver`
    pub solver: Booleanmodifiersolver,
    /// `BooleanModifierMaterialMode material_mode`
    pub material_mode: Booleanmodifiermaterialmode,
    /// `BooleanModifierFlag flag`
    pub flag: Booleanmodifierflag,
    /// `BooleanModifierBMeshFlag bm_flag`
    pub bm_flag: Booleanmodifierbmeshflag,
}
impl Default for BooleanModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            object: NIL,
            collection: NIL,
            double_threshold: 1e-6,
            operation: eBooleanModifierOp_Difference,
            solver: eBooleanModifierSolver_Mesh_Arr,
            material_mode: eBooleanModifierMaterialMode_Index,
            flag: eBooleanModifierFlag_Object,
            bm_flag: Default::default(),
        }
    }
}

/// `MDefInfluence`.
#[derive(Clone)]
pub struct MDefInfluence {
    /// `int vertex`
    pub vertex: i32,
    /// `float weight`
    pub weight: f32,
}
impl Default for MDefInfluence {
    fn default() -> Self {
        Self {
            vertex: 0,
            weight: 0.0,
        }
    }
}

/// `MDefCell`.
#[derive(Clone)]
pub struct MDefCell {
    /// `int offset`
    pub offset: i32,
    /// `int influences_num`
    pub influences_num: i32,
}
impl Default for MDefCell {
    fn default() -> Self {
        Self {
            offset: 0,
            influences_num: 0,
        }
    }
}

/// `MeshDeformModifierData`.
#[derive(Clone)]
pub struct MeshDeformModifierData {
    pub modifier: ModifierData,
    /// `Object *object`
    pub object: usize,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `short gridsize`
    pub gridsize: i16,
    /// `MeshDeformModifierFlag flag`
    pub flag: Meshdeformmodifierflag,
    /// `MDefInfluence *bindinfluences`
    pub bindinfluences: usize,
    /// `int *bindoffsets`
    pub bindoffsets: usize,
    /// `float *bindcagecos`
    pub bindcagecos: usize,
    /// `int verts_num`
    pub verts_num: i32,
    /// `int cage_verts_num`
    pub cage_verts_num: i32,
    /// `MDefCell *dyngrid`
    pub dyngrid: usize,
    /// `MDefInfluence *dyninfluences`
    pub dyninfluences: usize,
    /// `int *dynverts`
    pub dynverts: usize,
    /// `int dyngridsize`
    pub dyngridsize: i32,
    /// `int influences_num`
    pub influences_num: i32,
    /// `float dyncellmin[3]`
    pub dyncellmin: [f32; 3],
    /// `float dyncellwidth`
    pub dyncellwidth: f32,
    /// `float bindmat[4][4]`
    pub bindmat: [[f32; 4]; 4],
    /// `float *bindweights`
    pub bindweights: usize,
    /// `float *bindcos`
    pub bindcos: usize,
}
impl Default for MeshDeformModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            object: NIL,
            defgrp_name: [0; 64],
            gridsize: 5,
            flag: Default::default(),
            bindinfluences: NIL,
            bindoffsets: NIL,
            bindcagecos: NIL,
            verts_num: 0,
            cage_verts_num: 0,
            dyngrid: NIL,
            dyninfluences: NIL,
            dynverts: NIL,
            dyngridsize: 0,
            influences_num: 0,
            dyncellmin: [0.0; 3],
            dyncellwidth: 0.0,
            bindmat: [[0.0; 4]; 4],
            bindweights: NIL,
            bindcos: NIL,
        }
    }
}

/// `ParticleSystemModifierData`.
#[derive(Clone)]
pub struct ParticleSystemModifierData {
    pub modifier: ModifierData,
    /// `ParticleSystem *psys`
    pub psys: usize,
    /// `Mesh *mesh_final`
    pub mesh_final: usize,
    /// `Mesh *mesh_original`
    pub mesh_original: usize,
    /// `int totdmvert`
    pub totdmvert: i32,
    /// `int totdmedge`
    pub totdmedge: i32,
    /// `int totdmface`
    pub totdmface: i32,
    /// `ParticleSystemModifierFlag flag`
    pub flag: Particlesystemmodifierflag,
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for ParticleSystemModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            psys: NIL,
            mesh_final: NIL,
            mesh_original: NIL,
            totdmvert: 0,
            totdmedge: 0,
            totdmface: 0,
            flag: Default::default(),
            pad1: NIL,
        }
    }
}

/// `ParticleInstanceModifierData`.
#[derive(Clone)]
pub struct ParticleInstanceModifierData {
    pub modifier: ModifierData,
    /// `Object *ob`
    pub ob: usize,
    /// `short psys`
    pub psys: i16,
    /// `short axis`
    pub axis: i16,
    /// `ParticleInstanceModifierSpace space`
    pub space: Particleinstancemodifierspace,
    /// `float position`
    pub position: f32,
    /// `float random_position`
    pub random_position: f32,
    /// `float rotation`
    pub rotation: f32,
    /// `float random_rotation`
    pub random_rotation: f32,
    /// `float particle_amount`
    pub particle_amount: f32,
    /// `float particle_offset`
    pub particle_offset: f32,
    /// `char index_layer_name[68]`
    pub index_layer_name: [u8; 68],
    /// `char value_layer_name[68]`
    pub value_layer_name: [u8; 68],
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for ParticleInstanceModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            ob: NIL,
            psys: 1,
            axis: 2,
            space: eParticleInstanceSpace_World,
            position: 1.0,
            random_position: 0.0,
            rotation: 0.0,
            random_rotation: 0.0,
            particle_amount: 1.0,
            particle_offset: 0.0,
            index_layer_name: [0; 68],
            value_layer_name: [0; 68],
            pad1: NIL,
        }
    }
}

/// `ExplodeModifierData`.
#[derive(Clone)]
pub struct ExplodeModifierData {
    pub modifier: ModifierData,
    /// `void *facepa`
    pub facepa: usize,
    /// `ExplodeModifierFlag flag`
    pub flag: Explodemodifierflag,
    /// `short vgroup`
    pub vgroup: i16,
    /// `float protect`
    pub protect: f32,
    /// `char uvname[68]`
    pub uvname: [u8; 68],
    /// `void *_pad2`
    pub pad2: usize,
}
impl Default for ExplodeModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            facepa: NIL,
            flag: eExplodeFlag_Unborn | eExplodeFlag_Alive | eExplodeFlag_Dead,
            vgroup: 0,
            protect: 0.0,
            uvname: [0; 68],
            pad2: NIL,
        }
    }
}

/// `MultiresModifierData`.
#[derive(Clone)]
pub struct MultiresModifierData {
    pub modifier: ModifierData,
    /// `char lvl`
    pub lvl: i8,
    /// `char sculptlvl`
    pub sculptlvl: i8,
    /// `char renderlvl`
    pub renderlvl: i8,
    /// `char totlvl`
    pub totlvl: i8,
    /// `char simple`
    pub simple: i8,
    /// `short quality`
    pub quality: i16,
    /// `short uv_smooth`
    pub uv_smooth: i16,
    /// `short boundary_smooth`
    pub boundary_smooth: i16,
}
impl Default for MultiresModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            lvl: 0,
            sculptlvl: 0,
            renderlvl: 0,
            totlvl: 0,
            simple: 0,
            quality: 4,
            uv_smooth: SUBSURF_UV_SMOOTH_PRESERVE_BOUNDARIES,
            boundary_smooth: SUBSURF_BOUNDARY_SMOOTH_ALL,
        }
    }
}

/// `FluidsimModifierData`.
#[derive(Clone)]
pub struct FluidsimModifierData {
    pub modifier: ModifierData,
    /// `FluidsimSettings *fss`
    pub fss: usize,
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for FluidsimModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            fss: NIL,
            pad1: NIL,
        }
    }
}

/// `SmokeModifierData`.
#[derive(Clone)]
pub struct SmokeModifierData {
    pub modifier: ModifierData,
    /// `int type`
    pub type_: i32,
}
impl Default for SmokeModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            type_: 0,
        }
    }
}

/// `ShrinkwrapModifierData`.
#[derive(Clone)]
pub struct ShrinkwrapModifierData {
    pub modifier: ModifierData,
    /// `Object *target`
    pub target: usize,
    /// `Object *auxTarget`
    pub aux_target: usize,
    /// `char vgroup_name[64]`
    pub vgroup_name: [u8; 64],
    /// `float keepDist`
    pub keep_dist: f32,
    /// `short shrinkType`
    pub shrink_type: i16,
    /// `char shrinkOpts`
    pub shrink_opts: i8,
    /// `char shrinkMode`
    pub shrink_mode: i8,
    /// `float projLimit`
    pub proj_limit: f32,
    /// `char projAxis`
    pub proj_axis: i8,
    /// `char subsurfLevels`
    pub subsurf_levels: i8,
}
impl Default for ShrinkwrapModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            target: NIL,
            aux_target: NIL,
            vgroup_name: [0; 64],
            keep_dist: 0.0,
            shrink_type: MOD_SHRINKWRAP_NEAREST_SURFACE,
            shrink_opts: MOD_SHRINKWRAP_PROJECT_ALLOW_POS_DIR as i8,
            shrink_mode: 0,
            proj_limit: 0.0,
            proj_axis: 0,
            subsurf_levels: 0,
        }
    }
}

/// `SimpleDeformModifierData`.
#[derive(Clone)]
pub struct SimpleDeformModifierData {
    pub modifier: ModifierData,
    /// `Object *origin`
    pub origin: usize,
    /// `char vgroup_name[64]`
    pub vgroup_name: [u8; 64],
    /// `float factor`
    pub factor: f32,
    /// `float limit[2]`
    pub limit: [f32; 2],
    /// `SimpleDeformModifierMode mode`
    pub mode: Simpledeformmodifiermode,
    /// `char axis`
    pub axis: i8,
    /// `SimpleDeformModifierLockAxis deform_axis`
    pub deform_axis: Simpledeformmodifierlockaxis,
    /// `SimpleDeformModifierFlag flag`
    pub flag: Simpledeformmodifierflag,
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for SimpleDeformModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            origin: NIL,
            vgroup_name: [0; 64],
            factor: (45.0f32).to_radians(),
            limit: [0.0; 2],
            mode: MOD_SIMPLEDEFORM_MODE_TWIST,
            axis: 0,
            deform_axis: Default::default(),
            flag: Default::default(),
            pad1: NIL,
        }
    }
}

/// `ShapeKeyModifierData`.
#[derive(Clone)]
pub struct ShapeKeyModifierData {
    pub modifier: ModifierData,
}
impl Default for ShapeKeyModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
        }
    }
}

/// `SolidifyModifierData`.
#[derive(Clone)]
pub struct SolidifyModifierData {
    pub modifier: ModifierData,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `char shell_defgrp_name[64]`
    pub shell_defgrp_name: [u8; 64],
    /// `char rim_defgrp_name[64]`
    pub rim_defgrp_name: [u8; 64],
    /// `float offset`
    pub offset: f32,
    /// `float offset_fac`
    pub offset_fac: f32,
    /// `float offset_fac_vg`
    pub offset_fac_vg: f32,
    /// `float offset_clamp`
    pub offset_clamp: f32,
    /// `SolidifyModifierMode mode`
    pub mode: Solidifymodifiermode,
    /// `float crease_inner`
    pub crease_inner: f32,
    /// `float crease_outer`
    pub crease_outer: f32,
    /// `float crease_rim`
    pub crease_rim: f32,
    /// `SolidifyModifierFlag flag`
    pub flag: Solidifymodifierflag,
    /// `short mat_ofs`
    pub mat_ofs: i16,
    /// `short mat_ofs_rim`
    pub mat_ofs_rim: i16,
    /// `float merge_tolerance`
    pub merge_tolerance: f32,
    /// `float bevel_convex`
    pub bevel_convex: f32,
}
impl Default for SolidifyModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            defgrp_name: [0; 64],
            shell_defgrp_name: [0; 64],
            rim_defgrp_name: [0; 64],
            offset: 0.01,
            offset_fac: -1.0,
            offset_fac_vg: 0.0,
            offset_clamp: 0.0,
            mode: MOD_SOLIDIFY_MODE_EXTRUDE,
            crease_inner: 0.0,
            crease_outer: 0.0,
            crease_rim: 0.0,
            flag: MOD_SOLIDIFY_RIM,
            mat_ofs: 0,
            mat_ofs_rim: 0,
            merge_tolerance: 0.0001,
            bevel_convex: 0.0,
        }
    }
}

/// `ScrewModifierData`.
#[derive(Clone)]
pub struct ScrewModifierData {
    pub modifier: ModifierData,
    /// `Object *ob_axis`
    pub ob_axis: usize,
    /// `unsigned_int steps`
    pub steps: u32,
    /// `unsigned_int render_steps`
    pub render_steps: u32,
    /// `unsigned_int iter`
    pub iter: u32,
    /// `float screw_ofs`
    pub screw_ofs: f32,
    /// `float angle`
    pub angle: f32,
    /// `float merge_dist`
    pub merge_dist: f32,
    /// `ScrewModifierFlag flag`
    pub flag: Screwmodifierflag,
    /// `char axis`
    pub axis: i8,
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for ScrewModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            ob_axis: NIL,
            steps: 16,
            render_steps: 16,
            iter: 1,
            screw_ofs: 0.0,
            angle: 2.0 * std::f32::consts::PI,
            merge_dist: 0.01,
            flag: MOD_SCREW_SMOOTH_SHADING,
            axis: 2,
            pad1: NIL,
        }
    }
}

/// `OceanModifierData`.
#[derive(Clone)]
pub struct OceanModifierData {
    pub modifier: ModifierData,
    /// `Ocean *ocean`
    pub ocean: usize,
    /// `OceanCache *oceancache`
    pub oceancache: usize,
    /// `int resolution`
    pub resolution: i32,
    /// `int viewport_resolution`
    pub viewport_resolution: i32,
    /// `int spatial_size`
    pub spatial_size: i32,
    /// `float wind_velocity`
    pub wind_velocity: f32,
    /// `float damp`
    pub damp: f32,
    /// `float smallest_wave`
    pub smallest_wave: f32,
    /// `float depth`
    pub depth: f32,
    /// `float wave_alignment`
    pub wave_alignment: f32,
    /// `float wave_direction`
    pub wave_direction: f32,
    /// `float wave_scale`
    pub wave_scale: f32,
    /// `float chop_amount`
    pub chop_amount: f32,
    /// `float foam_coverage`
    pub foam_coverage: f32,
    /// `float time`
    pub time: f32,
    /// `OceanModifierSpectrum spectrum`
    pub spectrum: Oceanmodifierspectrum,
    /// `float fetch_jonswap`
    pub fetch_jonswap: f32,
    /// `float sharpen_peak_jonswap`
    pub sharpen_peak_jonswap: f32,
    /// `int bakestart`
    pub bakestart: i32,
    /// `int bakeend`
    pub bakeend: i32,
    /// `char cachepath[1024]`
    pub cachepath: [u8; 1024],
    /// `char foamlayername[68]`
    pub foamlayername: [u8; 68],
    /// `char spraylayername[68]`
    pub spraylayername: [u8; 68],
    /// `char cached`
    pub cached: i8,
    /// `OceanModifierGeometryMode geometry_mode`
    pub geometry_mode: Oceanmodifiergeometrymode,
    /// `OceanModifierFlag flag`
    pub flag: Oceanmodifierflag,
    /// `short repeat_x`
    pub repeat_x: i16,
    /// `short repeat_y`
    pub repeat_y: i16,
    /// `int seed`
    pub seed: i32,
    /// `float size`
    pub size: f32,
    /// `float foam_fade`
    pub foam_fade: f32,
}
impl Default for OceanModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            ocean: NIL,
            oceancache: NIL,
            resolution: 7,
            viewport_resolution: 7,
            spatial_size: 50,
            wind_velocity: 30.0,
            damp: 0.5,
            smallest_wave: 0.01,
            depth: 200.0,
            wave_alignment: 0.0,
            wave_direction: 0.0,
            wave_scale: 1.0,
            chop_amount: 1.0,
            foam_coverage: 0.0,
            time: 1.0,
            spectrum: MOD_OCEAN_SPECTRUM_PHILLIPS,
            fetch_jonswap: 120.0,
            sharpen_peak_jonswap: 0.0,
            bakestart: 1,
            bakeend: 250,
            cachepath: [0; 1024],
            foamlayername: [0; 68],
            spraylayername: [0; 68],
            cached: 0,
            geometry_mode: MOD_OCEAN_GEOM_GENERATE,
            flag: Default::default(),
            repeat_x: 1,
            repeat_y: 1,
            seed: 0,
            size: 1.0,
            foam_fade: 0.98,
        }
    }
}

/// `WarpModifierData`.
#[derive(Clone)]
pub struct WarpModifierData {
    pub modifier: ModifierData,
    /// `Tex *texture`
    pub texture: usize,
    /// `Object *map_object`
    pub map_object: usize,
    /// `char map_bone[64]`
    pub map_bone: [u8; 64],
    /// `char uvlayer_name[68]`
    pub uvlayer_name: [u8; 68],
    /// `int uvlayer_tmp`
    pub uvlayer_tmp: i32,
    /// `DisplaceModifierTexMapping texmapping`
    pub texmapping: Displacemodifiertexmapping,
    /// `Object *object_from`
    pub object_from: usize,
    /// `Object *object_to`
    pub object_to: usize,
    /// `char bone_from[64]`
    pub bone_from: [u8; 64],
    /// `char bone_to[64]`
    pub bone_to: [u8; 64],
    /// `CurveMapping *curfalloff`
    pub curfalloff: usize,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `float strength`
    pub strength: f32,
    /// `float falloff_radius`
    pub falloff_radius: f32,
    /// `WarpModifierFlag flag`
    pub flag: Warpmodifierflag,
    /// `WarpModifierFalloff falloff_type`
    pub falloff_type: Warpmodifierfalloff,
    /// `void *_pad3`
    pub pad3: usize,
}
impl Default for WarpModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            texture: NIL,
            map_object: NIL,
            map_bone: [0; 64],
            uvlayer_name: [0; 68],
            uvlayer_tmp: 0,
            texmapping: MOD_DISP_MAP_LOCAL,
            object_from: NIL,
            object_to: NIL,
            bone_from: [0; 64],
            bone_to: [0; 64],
            curfalloff: NIL,
            defgrp_name: [0; 64],
            strength: 1.0,
            falloff_radius: 1.0,
            flag: Default::default(),
            falloff_type: eWarp_Falloff_Smooth,
            pad3: NIL,
        }
    }
}

/// `WeightVGEditModifierData`.
#[derive(Clone)]
pub struct WeightVGEditModifierData {
    pub modifier: ModifierData,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `WeightVGEditModifierEditFlags edit_flags`
    pub edit_flags: Weightvgeditmodifiereditflags,
    /// `WeightVGProximityFalloff falloff_type`
    pub falloff_type: Weightvgproximityfalloff,
    /// `float default_weight`
    pub default_weight: f32,
    /// `CurveMapping *cmap_curve`
    pub cmap_curve: usize,
    /// `float add_threshold`
    pub add_threshold: f32,
    /// `float rem_threshold`
    pub rem_threshold: f32,
    /// `float mask_constant`
    pub mask_constant: f32,
    /// `char mask_defgrp_name[64]`
    pub mask_defgrp_name: [u8; 64],
    /// `WeightVGProximityModifierMaskTexChannel mask_tex_use_channel`
    pub mask_tex_use_channel: Weightvgproximitymodifiermasktexchannel,
    /// `Tex *mask_texture`
    pub mask_texture: usize,
    /// `Object *mask_tex_map_obj`
    pub mask_tex_map_obj: usize,
    /// `char mask_tex_map_bone[64]`
    pub mask_tex_map_bone: [u8; 64],
    /// `DisplaceModifierTexMapping mask_tex_mapping`
    pub mask_tex_mapping: Displacemodifiertexmapping,
    /// `char mask_tex_uvlayer_name[68]`
    pub mask_tex_uvlayer_name: [u8; 68],
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for WeightVGEditModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            defgrp_name: [0; 64],
            edit_flags: Default::default(),
            falloff_type: MOD_WVG_MAPPING_NONE,
            default_weight: 0.0,
            cmap_curve: NIL,
            add_threshold: 0.01,
            rem_threshold: 0.01,
            mask_constant: 1.0,
            mask_defgrp_name: [0; 64],
            mask_tex_use_channel: MOD_WVG_MASK_TEX_USE_INT,
            mask_texture: NIL,
            mask_tex_map_obj: NIL,
            mask_tex_map_bone: [0; 64],
            mask_tex_mapping: MOD_DISP_MAP_LOCAL,
            mask_tex_uvlayer_name: [0; 68],
            pad1: NIL,
        }
    }
}

/// `WeightVGMixModifierData`.
#[derive(Clone)]
pub struct WeightVGMixModifierData {
    pub modifier: ModifierData,
    /// `char defgrp_name_a[64]`
    pub defgrp_name_a: [u8; 64],
    /// `char defgrp_name_b[64]`
    pub defgrp_name_b: [u8; 64],
    /// `float default_weight_a`
    pub default_weight_a: f32,
    /// `float default_weight_b`
    pub default_weight_b: f32,
    /// `WeightVGMixModifierMixMode mix_mode`
    pub mix_mode: Weightvgmixmodifiermixmode,
    /// `WeightVGMixModifierMixSet mix_set`
    pub mix_set: Weightvgmixmodifiermixset,
    /// `float mask_constant`
    pub mask_constant: f32,
    /// `char mask_defgrp_name[64]`
    pub mask_defgrp_name: [u8; 64],
    /// `WeightVGProximityModifierMaskTexChannel mask_tex_use_channel`
    pub mask_tex_use_channel: Weightvgproximitymodifiermasktexchannel,
    /// `Tex *mask_texture`
    pub mask_texture: usize,
    /// `Object *mask_tex_map_obj`
    pub mask_tex_map_obj: usize,
    /// `char mask_tex_map_bone[64]`
    pub mask_tex_map_bone: [u8; 64],
    /// `DisplaceModifierTexMapping mask_tex_mapping`
    pub mask_tex_mapping: Displacemodifiertexmapping,
    /// `char mask_tex_uvlayer_name[68]`
    pub mask_tex_uvlayer_name: [u8; 68],
    /// `WeightVGMixModifierFlag flag`
    pub flag: Weightvgmixmodifierflag,
}
impl Default for WeightVGMixModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            defgrp_name_a: [0; 64],
            defgrp_name_b: [0; 64],
            default_weight_a: 0.0,
            default_weight_b: 0.0,
            mix_mode: MOD_WVG_MIX_SET,
            mix_set: MOD_WVG_SET_AND,
            mask_constant: 1.0,
            mask_defgrp_name: [0; 64],
            mask_tex_use_channel: MOD_WVG_MASK_TEX_USE_INT,
            mask_texture: NIL,
            mask_tex_map_obj: NIL,
            mask_tex_map_bone: [0; 64],
            mask_tex_mapping: MOD_DISP_MAP_LOCAL,
            mask_tex_uvlayer_name: [0; 68],
            flag: Default::default(),
        }
    }
}

/// `WeightVGProximityModifierData`.
#[derive(Clone)]
pub struct WeightVGProximityModifierData {
    pub modifier: ModifierData,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `CurveMapping *cmap_curve`
    pub cmap_curve: usize,
    /// `WeightVGProximityModifierProximityMode proximity_mode`
    pub proximity_mode: Weightvgproximitymodifierproximitymode,
    /// `WeightVGProximityModifierFlag proximity_flags`
    pub proximity_flags: Weightvgproximitymodifierflag,
    /// `Object *proximity_ob_target`
    pub proximity_ob_target: usize,
    /// `float mask_constant`
    pub mask_constant: f32,
    /// `char mask_defgrp_name[64]`
    pub mask_defgrp_name: [u8; 64],
    /// `WeightVGProximityModifierMaskTexChannel mask_tex_use_channel`
    pub mask_tex_use_channel: Weightvgproximitymodifiermasktexchannel,
    /// `Tex *mask_texture`
    pub mask_texture: usize,
    /// `Object *mask_tex_map_obj`
    pub mask_tex_map_obj: usize,
    /// `char mask_tex_map_bone[64]`
    pub mask_tex_map_bone: [u8; 64],
    /// `DisplaceModifierTexMapping mask_tex_mapping`
    pub mask_tex_mapping: Displacemodifiertexmapping,
    /// `char mask_tex_uvlayer_name[68]`
    pub mask_tex_uvlayer_name: [u8; 68],
    /// `float min_dist`
    pub min_dist: f32,
    /// `float max_dist`
    pub max_dist: f32,
    /// `WeightVGProximityFalloff falloff_type`
    pub falloff_type: Weightvgproximityfalloff,
}
impl Default for WeightVGProximityModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            defgrp_name: [0; 64],
            cmap_curve: NIL,
            proximity_mode: MOD_WVG_PROXIMITY_OBJECT,
            proximity_flags: MOD_WVG_PROXIMITY_GEOM_VERTS,
            proximity_ob_target: NIL,
            mask_constant: 1.0,
            mask_defgrp_name: [0; 64],
            mask_tex_use_channel: MOD_WVG_MASK_TEX_USE_INT,
            mask_texture: NIL,
            mask_tex_map_obj: NIL,
            mask_tex_map_bone: [0; 64],
            mask_tex_mapping: MOD_DISP_MAP_LOCAL,
            mask_tex_uvlayer_name: [0; 68],
            min_dist: 0.0,
            max_dist: 1.0,
            falloff_type: MOD_WVG_MAPPING_NONE,
        }
    }
}

/// `DynamicPaintModifierData`.
#[derive(Clone)]
pub struct DynamicPaintModifierData {
    pub modifier: ModifierData,
    /// `DynamicPaintCanvasSettings *canvas`
    pub canvas: usize,
    /// `DynamicPaintBrushSettings *brush`
    pub brush: usize,
    /// `DynamicPaintModifierType type`
    pub type_: Dynamicpaintmodifiertype,
}
impl Default for DynamicPaintModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            canvas: NIL,
            brush: NIL,
            type_: MOD_DYNAMICPAINT_TYPE_CANVAS,
        }
    }
}

/// `RemeshModifierData`.
#[derive(Clone)]
pub struct RemeshModifierData {
    pub modifier: ModifierData,
    /// `float threshold`
    pub threshold: f32,
    /// `float scale`
    pub scale: f32,
    /// `float hermite_num`
    pub hermite_num: f32,
    /// `char depth`
    pub depth: i8,
    /// `eRemeshModifierFlags flag`
    pub flag: Remeshmodifierflags,
    /// `eRemeshModifierMode mode`
    pub mode: Remeshmodifiermode,
    /// `float voxel_size`
    pub voxel_size: f32,
    /// `float adaptivity`
    pub adaptivity: f32,
}
impl Default for RemeshModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            threshold: 1.0,
            scale: 0.9,
            hermite_num: 1.0,
            depth: 4,
            flag: MOD_REMESH_FLOOD_FILL,
            mode: MOD_REMESH_VOXEL,
            voxel_size: 0.1,
            adaptivity: 0.0,
        }
    }
}

/// `SkinModifierData`.
#[derive(Clone)]
pub struct SkinModifierData {
    pub modifier: ModifierData,
    /// `float branch_smoothing`
    pub branch_smoothing: f32,
    /// `SkinModifierFlag flag`
    pub flag: Skinmodifierflag,
    /// `SkinModifierSymmetryAxis symmetry_axes`
    pub symmetry_axes: Skinmodifiersymmetryaxis,
}
impl Default for SkinModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            branch_smoothing: 0.0,
            flag: Default::default(),
            symmetry_axes: MOD_SKIN_SYMM_X,
        }
    }
}

/// `TriangulateModifierData`.
#[derive(Clone)]
pub struct TriangulateModifierData {
    pub modifier: ModifierData,
    /// `TriangulateModifierFlag flag`
    pub flag: Triangulatemodifierflag,
    /// `TriangulateModifierQuadMethod quad_method`
    pub quad_method: Triangulatemodifierquadmethod,
    /// `TriangulateModifierNgonMethod ngon_method`
    pub ngon_method: Triangulatemodifierngonmethod,
    /// `int min_vertices`
    pub min_vertices: i32,
}
impl Default for TriangulateModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            flag: Default::default(),
            quad_method: MOD_TRIANGULATE_QUAD_SHORTEDGE,
            ngon_method: MOD_TRIANGULATE_NGON_BEAUTY,
            min_vertices: 4,
        }
    }
}

/// `LaplacianSmoothModifierData`.
#[derive(Clone)]
pub struct LaplacianSmoothModifierData {
    pub modifier: ModifierData,
    /// `float lambda`
    pub lambda: f32,
    /// `float lambda_border`
    pub lambda_border: f32,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `short repeat`
    pub repeat: i16,
}
impl Default for LaplacianSmoothModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            lambda: 0.01,
            lambda_border: 0.01,
            defgrp_name: [0; 64],
            repeat: 1,
        }
    }
}

/// `CorrectiveSmoothDeltaCache`.
#[derive(Clone)]
pub struct CorrectiveSmoothDeltaCache {
    /// `unsigned_int deltas_num`
    pub deltas_num: u32,
    /// `float lambda`
    pub lambda: f32,
    /// `float scale`
    pub scale: f32,
    /// `short repeat`
    pub repeat: i16,
    /// `short flag`
    pub flag: i16,
    /// `char smooth_type`
    pub smooth_type: i8,
    /// `char rest_source`
    pub rest_source: i8,
}
impl Default for CorrectiveSmoothDeltaCache {
    fn default() -> Self {
        Self {
            deltas_num: 0,
            lambda: 0.0,
            scale: 0.0,
            repeat: 0,
            flag: 0,
            smooth_type: 0,
            rest_source: 0,
        }
    }
}

/// `CorrectiveSmoothModifierData`.
#[derive(Clone)]
pub struct CorrectiveSmoothModifierData {
    pub modifier: ModifierData,
    /// `unsigned_int bind_coords_num`
    pub bind_coords_num: u32,
    /// `float lambda`
    pub lambda: f32,
    /// `float scale`
    pub scale: f32,
    /// `short repeat`
    pub repeat: i16,
    /// `CorrectiveSmoothModifierFlag flag`
    pub flag: Correctivesmoothmodifierflag,
    /// `CorrectiveSmoothModifierType smooth_type`
    pub smooth_type: Correctivesmoothmodifiertype,
    /// `CorrectiveSmoothRestSource rest_source`
    pub rest_source: Correctivesmoothrestsource,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    pub delta_cache: CorrectiveSmoothDeltaCache,
}
impl Default for CorrectiveSmoothModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            bind_coords_num: 0,
            lambda: 0.5,
            scale: 1.0,
            repeat: 5,
            flag: Default::default(),
            smooth_type: MOD_CORRECTIVESMOOTH_SMOOTH_SIMPLE,
            rest_source: MOD_CORRECTIVESMOOTH_RESTSOURCE_ORCO,
            defgrp_name: [0; 64],
            delta_cache: Default::default(),
        }
    }
}

/// `UVWarpModifierData`.
#[derive(Clone)]
pub struct UVWarpModifierData {
    pub modifier: ModifierData,
    /// `char axis_u`
    pub axis_u: i8,
    /// `char axis_v`
    pub axis_v: i8,
    /// `UVWarpModifierFlag flag`
    pub flag: Uvwarpmodifierflag,
    /// `float center[2]`
    pub center: [f32; 2],
    /// `float offset[2]`
    pub offset: [f32; 2],
    /// `float scale[2]`
    pub scale: [f32; 2],
    /// `float rotation`
    pub rotation: f32,
    /// `Object *object_src`
    pub object_src: usize,
    /// `char bone_src[64]`
    pub bone_src: [u8; 64],
    /// `Object *object_dst`
    pub object_dst: usize,
    /// `char bone_dst[64]`
    pub bone_dst: [u8; 64],
    /// `char vgroup_name[64]`
    pub vgroup_name: [u8; 64],
    /// `char uvlayer_name[68]`
    pub uvlayer_name: [u8; 68],
}
impl Default for UVWarpModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            axis_u: 0,
            axis_v: 1,
            flag: Default::default(),
            center: [0.0; 2],
            offset: [0.0; 2],
            scale: [0.0; 2],
            rotation: 0.0,
            object_src: NIL,
            bone_src: [0; 64],
            object_dst: NIL,
            bone_dst: [0; 64],
            vgroup_name: [0; 64],
            uvlayer_name: [0; 68],
        }
    }
}

/// `MeshCacheModifierData`.
#[derive(Clone)]
pub struct MeshCacheModifierData {
    pub modifier: ModifierData,
    /// `MeshCacheModifierFlag flag`
    pub flag: Meshcachemodifierflag,
    /// `MeshCacheModifierType type`
    pub type_: Meshcachemodifiertype,
    /// `MeshCacheModifierTimeMode time_mode`
    pub time_mode: Meshcachemodifiertimemode,
    /// `MeshCacheModifierPlayMode play_mode`
    pub play_mode: Meshcachemodifierplaymode,
    /// `char forward_axis`
    pub forward_axis: i8,
    /// `char up_axis`
    pub up_axis: i8,
    /// `MeshCacheModifierFlipAxis flip_axis`
    pub flip_axis: Meshcachemodifierflipaxis,
    /// `MeshCacheModifierInterpolation interp`
    pub interp: Meshcachemodifierinterpolation,
    /// `float factor`
    pub factor: f32,
    /// `MeshCacheModifierDeformMode deform_mode`
    pub deform_mode: Meshcachemodifierdeformmode,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `float frame_start`
    pub frame_start: f32,
    /// `float frame_scale`
    pub frame_scale: f32,
    /// `float eval_frame`
    pub eval_frame: f32,
    /// `float eval_time`
    pub eval_time: f32,
    /// `float eval_factor`
    pub eval_factor: f32,
    /// `char filepath[1024]`
    pub filepath: [u8; 1024],
}
impl Default for MeshCacheModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            flag: Default::default(),
            type_: MOD_MESHCACHE_TYPE_MDD,
            time_mode: MOD_MESHCACHE_TIME_FRAME,
            play_mode: MOD_MESHCACHE_PLAY_CFEA,
            forward_axis: 1,
            up_axis: 2,
            flip_axis: Default::default(),
            interp: MOD_MESHCACHE_INTERP_LINEAR,
            factor: 1.0,
            deform_mode: MOD_MESHCACHE_DEFORM_OVERWRITE,
            defgrp_name: [0; 64],
            frame_start: 0.0,
            frame_scale: 1.0,
            eval_frame: 0.0,
            eval_time: 0.0,
            eval_factor: 0.0,
            filepath: [0; 1024],
        }
    }
}

/// `LaplacianDeformModifierData`.
#[derive(Clone)]
pub struct LaplacianDeformModifierData {
    pub modifier: ModifierData,
    /// `char anchor_grp_name[64]`
    pub anchor_grp_name: [u8; 64],
    /// `int verts_num`
    pub verts_num: i32,
    /// `int repeat`
    pub repeat: i32,
    /// `float *vertexco`
    pub vertexco: usize,
    /// `void *cache_system`
    pub cache_system: usize,
    /// `LaplacianDeformModifierFlag flag`
    pub flag: Laplaciandeformmodifierflag,
}
impl Default for LaplacianDeformModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            anchor_grp_name: [0; 64],
            verts_num: 0,
            repeat: 1,
            vertexco: NIL,
            cache_system: NIL,
            flag: Default::default(),
        }
    }
}

/// `WireframeModifierData`.
#[derive(Clone)]
pub struct WireframeModifierData {
    pub modifier: ModifierData,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `float offset`
    pub offset: f32,
    /// `float offset_fac`
    pub offset_fac: f32,
    /// `float offset_fac_vg`
    pub offset_fac_vg: f32,
    /// `float crease_weight`
    pub crease_weight: f32,
    /// `WireframeModifierFlag flag`
    pub flag: Wireframemodifierflag,
    /// `short mat_ofs`
    pub mat_ofs: i16,
}
impl Default for WireframeModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            defgrp_name: [0; 64],
            offset: 0.02,
            offset_fac: 0.0,
            offset_fac_vg: 0.0,
            crease_weight: 1.0,
            flag: MOD_WIREFRAME_REPLACE | MOD_WIREFRAME_OFS_EVEN,
            mat_ofs: 0,
        }
    }
}

/// `WeldModifierData`.
#[derive(Clone)]
pub struct WeldModifierData {
    pub modifier: ModifierData,
    /// `float merge_dist`
    pub merge_dist: f32,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `WeldModifierMode mode`
    pub mode: Weldmodifiermode,
    /// `WeldModifierFlag flag`
    pub flag: Weldmodifierflag,
}
impl Default for WeldModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            merge_dist: 0.001,
            defgrp_name: [0; 64],
            mode: MOD_WELD_MODE_ALL,
            flag: Default::default(),
        }
    }
}

/// `DataTransferModifierData`.
#[derive(Clone)]
pub struct DataTransferModifierData {
    pub modifier: ModifierData,
    /// `Object *ob_source`
    pub ob_source: usize,
    /// `int data_types`
    pub data_types: i32,
    /// `int vmap_mode`
    pub vmap_mode: i32,
    /// `int emap_mode`
    pub emap_mode: i32,
    /// `int lmap_mode`
    pub lmap_mode: i32,
    /// `int pmap_mode`
    pub pmap_mode: i32,
    /// `float map_max_distance`
    pub map_max_distance: f32,
    /// `float map_ray_radius`
    pub map_ray_radius: f32,
    /// `float islands_precision`
    pub islands_precision: f32,
    /// `int mix_mode`
    pub mix_mode: i32,
    /// `float mix_factor`
    pub mix_factor: f32,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `DataTransferModifierFlag flags`
    pub flags: Datatransfermodifierflag,
    /// `void *_pad2`
    pub pad2: usize,
}
impl Default for DataTransferModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            ob_source: NIL,
            data_types: 0,
            vmap_mode: MREMAP_MODE_VERT_NEAREST,
            emap_mode: MREMAP_MODE_EDGE_NEAREST,
            lmap_mode: MREMAP_MODE_LOOP_NEAREST_POLYNOR,
            pmap_mode: MREMAP_MODE_POLY_NEAREST,
            map_max_distance: 1.0,
            map_ray_radius: 0.0,
            islands_precision: 0.0,
            mix_mode: CDT_MIX_TRANSFER,
            mix_factor: 0.0,
            defgrp_name: [0; 64],
            flags: MOD_DATATRANSFER_OBSRC_TRANSFORM,
            pad2: NIL,
        }
    }
}

/// `NormalEditModifierData`.
#[derive(Clone)]
pub struct NormalEditModifierData {
    pub modifier: ModifierData,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `Object *target`
    pub target: usize,
    /// `NormalEditModifierMode mode`
    pub mode: Normaleditmodifiermode,
    /// `NormalEditModifierFlag flag`
    pub flag: Normaleditmodifierflag,
    /// `NormalEditModifierMixMode mix_mode`
    pub mix_mode: Normaleditmodifiermixmode,
    /// `float mix_factor`
    pub mix_factor: f32,
    /// `float mix_limit`
    pub mix_limit: f32,
    /// `float offset[3]`
    pub offset: [f32; 3],
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for NormalEditModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            defgrp_name: [0; 64],
            target: NIL,
            mode: MOD_NORMALEDIT_MODE_RADIAL,
            flag: Default::default(),
            mix_mode: MOD_NORMALEDIT_MIX_COPY,
            mix_factor: 1.0,
            mix_limit: std::f32::consts::PI,
            offset: [0.0; 3],
            pad1: NIL,
        }
    }
}

/// `MeshSeqCacheModifierData`.
#[derive(Clone)]
pub struct MeshSeqCacheModifierData {
    pub modifier: ModifierData,
    /// `CacheFile *cache_file`
    pub cache_file: usize,
    /// `char object_path[1024]`
    pub object_path: [u8; 1024],
    /// `float velocity_scale`
    pub velocity_scale: f32,
    /// `CacheReader *reader`
    pub reader: usize,
    /// `char reader_object_path[1024]`
    pub reader_object_path: [u8; 1024],
}
impl Default for MeshSeqCacheModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            cache_file: NIL,
            object_path: [0; 1024],
            velocity_scale: 1.0,
            reader: NIL,
            reader_object_path: [0; 1024],
        }
    }
}

/// `SDefBind`.
#[derive(Clone)]
pub struct SDefBind {
    /// `unsigned_int *vert_inds`
    pub vert_inds: usize,
    /// `unsigned_int verts_num`
    pub verts_num: u32,
    /// `SurfaceDeformModifierBindMode mode`
    pub mode: Surfacedeformmodifierbindmode,
    /// `float *vert_weights`
    pub vert_weights: usize,
    /// `float normal_dist`
    pub normal_dist: f32,
    /// `float influence`
    pub influence: f32,
}
impl Default for SDefBind {
    fn default() -> Self {
        Self {
            vert_inds: NIL,
            verts_num: 0,
            mode: MOD_SDEF_MODE_CORNER_TRIS,
            vert_weights: NIL,
            normal_dist: 0.0,
            influence: 0.0,
        }
    }
}

/// `SDefVert`.
#[derive(Clone)]
pub struct SDefVert {
    /// `SDefBind *binds`
    pub binds: usize,
    /// `unsigned_int binds_num`
    pub binds_num: u32,
    /// `unsigned_int vertex_idx`
    pub vertex_idx: u32,
}
impl Default for SDefVert {
    fn default() -> Self {
        Self {
            binds: NIL,
            binds_num: 0,
            vertex_idx: 0,
        }
    }
}

/// `SurfaceDeformModifierData`.
#[derive(Clone)]
pub struct SurfaceDeformModifierData {
    pub modifier: ModifierData,
    /// `Depsgraph *depsgraph`
    pub depsgraph: usize,
    /// `Object *target`
    pub target: usize,
    /// `SDefVert *verts`
    pub verts: usize,
    /// `float falloff`
    pub falloff: f32,
    /// `unsigned_int mesh_verts_num`
    pub mesh_verts_num: u32,
    /// `unsigned_int bind_verts_num`
    pub bind_verts_num: u32,
    /// `unsigned_int target_verts_num`
    pub target_verts_num: u32,
    /// `unsigned_int target_polys_num`
    pub target_polys_num: u32,
    /// `SurfaceDeformModifierFlag flags`
    pub flags: Surfacedeformmodifierflag,
    /// `float mat[4][4]`
    pub mat: [[f32; 4]; 4],
    /// `float strength`
    pub strength: f32,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
}
impl Default for SurfaceDeformModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            depsgraph: NIL,
            target: NIL,
            verts: NIL,
            falloff: 4.0,
            mesh_verts_num: 0,
            bind_verts_num: 0,
            target_verts_num: 0,
            target_polys_num: 0,
            flags: Default::default(),
            mat: [[0.0; 4]; 4],
            strength: 1.0,
            defgrp_name: [0; 64],
        }
    }
}

/// `WeightedNormalModifierData`.
#[derive(Clone)]
pub struct WeightedNormalModifierData {
    pub modifier: ModifierData,
    /// `char defgrp_name[64]`
    pub defgrp_name: [u8; 64],
    /// `WeightedNormalModifierMode mode`
    pub mode: Weightednormalmodifiermode,
    /// `WeightedNormalModifierFlag flag`
    pub flag: Weightednormalmodifierflag,
    /// `short weight`
    pub weight: i16,
    /// `float thresh`
    pub thresh: f32,
}
impl Default for WeightedNormalModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            defgrp_name: [0; 64],
            mode: MOD_WEIGHTEDNORMAL_MODE_FACE,
            flag: Default::default(),
            weight: 50,
            thresh: 0.01,
        }
    }
}

/// `NodesModifierSettings`.
#[derive(Clone)]
pub struct NodesModifierSettings {
    /// `IDProperty *properties`
    pub properties: usize,
}
impl Default for NodesModifierSettings {
    fn default() -> Self {
        Self {
            properties: NIL,
        }
    }
}

/// `NodesModifierDataBlock`.
#[derive(Clone)]
pub struct NodesModifierDataBlock {
    /// `char *id_name`
    pub id_name: usize,
    /// `char *lib_name`
    pub lib_name: usize,
    /// `ID *id`
    pub id: usize,
    /// `int id_type`
    pub id_type: i32,
}
impl Default for NodesModifierDataBlock {
    fn default() -> Self {
        Self {
            id_name: NIL,
            lib_name: NIL,
            id: NIL,
            id_type: 0,
        }
    }
}

/// `NodesModifierBakeFile`.
#[derive(Clone)]
pub struct NodesModifierBakeFile {
    /// `const char *name`
    pub name: usize,
    /// `PackedFile *packed_file`
    pub packed_file: usize,
}
impl Default for NodesModifierBakeFile {
    fn default() -> Self {
        Self {
            name: NIL,
            packed_file: NIL,
        }
    }
}

/// `NodesModifierPackedBake`.
#[derive(Clone)]
pub struct NodesModifierPackedBake {
    /// `int meta_files_num`
    pub meta_files_num: i32,
    /// `int blob_files_num`
    pub blob_files_num: i32,
    /// `NodesModifierBakeFile *meta_files`
    pub meta_files: usize,
    /// `NodesModifierBakeFile *blob_files`
    pub blob_files: usize,
}
impl Default for NodesModifierPackedBake {
    fn default() -> Self {
        Self {
            meta_files_num: 0,
            blob_files_num: 0,
            meta_files: NIL,
            blob_files: NIL,
        }
    }
}

/// `NodesModifierBake`.
#[derive(Clone)]
pub struct NodesModifierBake {
    /// `int id`
    pub id: i32,
    /// `NodesModifierBakeFlag flag`
    pub flag: Nodesmodifierbakeflag,
    /// `NodesModifierBakeMode bake_mode`
    pub bake_mode: Nodesmodifierbakemode,
    /// `NodesModifierBakeTarget bake_target`
    pub bake_target: Nodesmodifierbaketarget,
    /// `char *directory`
    pub directory: usize,
    /// `int frame_start`
    pub frame_start: i32,
    /// `int frame_end`
    pub frame_end: i32,
    /// `int data_blocks_num`
    pub data_blocks_num: i32,
    /// `int active_data_block`
    pub active_data_block: i32,
    /// `NodesModifierDataBlock *data_blocks`
    pub data_blocks: usize,
    /// `NodesModifierPackedBake *packed`
    pub packed: usize,
    /// `void *_pad2`
    pub pad2: usize,
    /// `int64_t bake_size`
    pub bake_size: i64,
}
impl Default for NodesModifierBake {
    fn default() -> Self {
        Self {
            id: 0,
            flag: Default::default(),
            bake_mode: NODES_MODIFIER_BAKE_MODE_ANIMATION,
            bake_target: NODES_MODIFIER_BAKE_TARGET_INHERIT,
            directory: NIL,
            frame_start: 0,
            frame_end: 0,
            data_blocks_num: 0,
            active_data_block: 0,
            data_blocks: NIL,
            packed: NIL,
            pad2: NIL,
            bake_size: 0,
        }
    }
}

/// `NodesModifierData`.
#[derive(Clone)]
pub struct NodesModifierData {
    pub modifier: ModifierData,
    /// `bNodeTree *node_group`
    pub node_group: usize,
    pub settings_legacy: NodesModifierSettings,
    /// `char *bake_directory`
    pub bake_directory: usize,
    /// `NodesModifierFlag flag`
    pub flag: Nodesmodifierflag,
    /// `NodesModifierBakeTarget bake_target`
    pub bake_target: Nodesmodifierbaketarget,
    /// `int bakes_num`
    pub bakes_num: i32,
    /// `NodesModifierBake *bakes`
    pub bakes: usize,
    /// `NodesModifierRuntime *runtime`
    pub runtime: usize,
}
impl Default for NodesModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            node_group: NIL,
            settings_legacy: Default::default(),
            bake_directory: NIL,
            flag: Default::default(),
            bake_target: NODES_MODIFIER_BAKE_TARGET_PACKED,
            bakes_num: 0,
            bakes: NIL,
            runtime: NIL,
        }
    }
}

/// `MeshToVolumeModifierData`.
#[derive(Clone)]
pub struct MeshToVolumeModifierData {
    pub modifier: ModifierData,
    /// `Object *object`
    pub object: usize,
    /// `MeshToVolumeModifierResolutionMode resolution_mode`
    pub resolution_mode: Meshtovolumemodifierresolutionmode,
    /// `float voxel_size`
    pub voxel_size: f32,
    /// `int voxel_amount`
    pub voxel_amount: i32,
    /// `float interior_band_width`
    pub interior_band_width: f32,
    /// `float density`
    pub density: f32,
    /// `void *_pad3`
    pub pad3: usize,
}
impl Default for MeshToVolumeModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            object: NIL,
            resolution_mode: MESH_TO_VOLUME_RESOLUTION_MODE_VOXEL_AMOUNT,
            voxel_size: 0.0,
            voxel_amount: 0,
            interior_band_width: 0.0,
            density: 0.0,
            pad3: NIL,
        }
    }
}

/// `VolumeDisplaceModifierData`.
#[derive(Clone)]
pub struct VolumeDisplaceModifierData {
    pub modifier: ModifierData,
    /// `Tex *texture`
    pub texture: usize,
    /// `Object *texture_map_object`
    pub texture_map_object: usize,
    /// `VolumeDisplaceModifierTextureMapMode texture_map_mode`
    pub texture_map_mode: Volumedisplacemodifiertexturemapmode,
    /// `float strength`
    pub strength: f32,
    /// `float texture_mid_level[3]`
    pub texture_mid_level: [f32; 3],
    /// `float texture_sample_radius`
    pub texture_sample_radius: f32,
}
impl Default for VolumeDisplaceModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            texture: NIL,
            texture_map_object: NIL,
            texture_map_mode: MOD_VOLUME_DISPLACE_MAP_LOCAL,
            strength: 0.0,
            texture_mid_level: [0.0; 3],
            texture_sample_radius: 0.0,
        }
    }
}

/// `VolumeToMeshModifierData`.
#[derive(Clone)]
pub struct VolumeToMeshModifierData {
    pub modifier: ModifierData,
    /// `Object *object`
    pub object: usize,
    /// `float threshold`
    pub threshold: f32,
    /// `float adaptivity`
    pub adaptivity: f32,
    /// `VolumeToMeshFlag flag`
    pub flag: Volumetomeshflag,
    /// `VolumeToMeshResolutionMode resolution_mode`
    pub resolution_mode: Volumetomeshresolutionmode,
    /// `float voxel_size`
    pub voxel_size: f32,
    /// `int voxel_amount`
    pub voxel_amount: i32,
    /// `char grid_name[64]`
    pub grid_name: [u8; 64],
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for VolumeToMeshModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            object: NIL,
            threshold: 0.0,
            adaptivity: 0.0,
            flag: Default::default(),
            resolution_mode: VOLUME_TO_MESH_RESOLUTION_MODE_GRID,
            voxel_size: 0.0,
            voxel_amount: 0,
            grid_name: [0; 64],
            pad1: NIL,
        }
    }
}

/// `GreasePencilModifierInfluenceData`.
#[derive(Clone)]
pub struct GreasePencilModifierInfluenceData {
    /// `GreasePencilModifierInfluenceFlag flag`
    pub flag: Greasepencilmodifierinfluenceflag,
    /// `char layer_name[64]`
    pub layer_name: [u8; 64],
    /// `Material *material`
    pub material: usize,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `int material_pass`
    pub material_pass: i32,
    /// `char vertex_group_name[64]`
    pub vertex_group_name: [u8; 64],
    /// `CurveMapping *custom_curve`
    pub custom_curve: usize,
    /// `void *_pad2`
    pub pad2: usize,
}
impl Default for GreasePencilModifierInfluenceData {
    fn default() -> Self {
        Self {
            flag: Default::default(),
            layer_name: [0; 64],
            material: NIL,
            layer_pass: 0,
            material_pass: 0,
            vertex_group_name: [0; 64],
            custom_curve: NIL,
            pad2: NIL,
        }
    }
}

/// `GreasePencilOpacityModifierData`.
#[derive(Clone)]
pub struct GreasePencilOpacityModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `GreasePencilOpacityModifierFlag flag`
    pub flag: Greasepencilopacitymodifierflag,
    /// `GreasePencilModifierColorMode color_mode`
    pub color_mode: Greasepencilmodifiercolormode,
    /// `float color_factor`
    pub color_factor: f32,
    /// `float hardness_factor`
    pub hardness_factor: f32,
    /// `void *_pad2`
    pub pad2: usize,
}
impl Default for GreasePencilOpacityModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            flag: Default::default(),
            color_mode: MOD_GREASE_PENCIL_COLOR_BOTH,
            color_factor: 1.0,
            hardness_factor: 1.0,
            pad2: NIL,
        }
    }
}

/// `GreasePencilSubdivModifierData`.
#[derive(Clone)]
pub struct GreasePencilSubdivModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `GreasePencilSubdivideType type`
    pub type_: Greasepencilsubdividetype,
    /// `int level`
    pub level: i32,
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for GreasePencilSubdivModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            type_: MOD_GREASE_PENCIL_SUBDIV_CATMULL,
            level: 1,
            pad1: NIL,
        }
    }
}

/// `GreasePencilColorModifierData`.
#[derive(Clone)]
pub struct GreasePencilColorModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `GreasePencilModifierColorMode color_mode`
    pub color_mode: Greasepencilmodifiercolormode,
    /// `float hsv[3]`
    pub hsv: [f32; 3],
    /// `void *_pad2`
    pub pad2: usize,
}
impl Default for GreasePencilColorModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            color_mode: MOD_GREASE_PENCIL_COLOR_BOTH,
            hsv: [0.0; 3],
            pad2: NIL,
        }
    }
}

/// `GreasePencilTintModifierData`.
#[derive(Clone)]
pub struct GreasePencilTintModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `GreasePencilTintModifierFlag flag`
    pub flag: Greasepenciltintmodifierflag,
    /// `GreasePencilModifierColorMode color_mode`
    pub color_mode: Greasepencilmodifiercolormode,
    /// `GreasePencilTintModifierMode tint_mode`
    pub tint_mode: Greasepenciltintmodifiermode,
    /// `float factor`
    pub factor: f32,
    /// `float radius`
    pub radius: f32,
    /// `float color[3]`
    pub color: [f32; 3],
    /// `Object *object`
    pub object: usize,
    /// `ColorBand *color_ramp`
    pub color_ramp: usize,
    /// `void *_pad`
    pub pad: usize,
}
impl Default for GreasePencilTintModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            flag: Default::default(),
            color_mode: MOD_GREASE_PENCIL_COLOR_BOTH,
            tint_mode: MOD_GREASE_PENCIL_TINT_UNIFORM,
            factor: 0.5,
            radius: 1.0,
            color: [0.0; 3],
            object: NIL,
            color_ramp: NIL,
            pad: NIL,
        }
    }
}

/// `GreasePencilSmoothModifierData`.
#[derive(Clone)]
pub struct GreasePencilSmoothModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `eGreasePencilSmooth_Flag flag`
    pub flag: GreasepencilsmoothFlag,
    /// `float factor`
    pub factor: f32,
    /// `int step`
    pub step: i32,
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for GreasePencilSmoothModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            flag: MOD_GREASE_PENCIL_SMOOTH_MOD_LOCATION,
            factor: 1.0,
            step: 1,
            pad1: NIL,
        }
    }
}

/// `GreasePencilOffsetModifierData`.
#[derive(Clone)]
pub struct GreasePencilOffsetModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `GreasePencilOffsetModifierFlag flag`
    pub flag: Greasepenciloffsetmodifierflag,
    /// `GreasePencilOffsetModifierMode offset_mode`
    pub offset_mode: Greasepenciloffsetmodifiermode,
    /// `float loc[3]`
    pub loc: [f32; 3],
    /// `float rot[3]`
    pub rot: [f32; 3],
    /// `float scale[3]`
    pub scale: [f32; 3],
    /// `float stroke_loc[3]`
    pub stroke_loc: [f32; 3],
    /// `float stroke_rot[3]`
    pub stroke_rot: [f32; 3],
    /// `float stroke_scale[3]`
    pub stroke_scale: [f32; 3],
    /// `int seed`
    pub seed: i32,
    /// `int stroke_step`
    pub stroke_step: i32,
    /// `int stroke_start_offset`
    pub stroke_start_offset: i32,
    /// `void *_pad2`
    pub pad2: usize,
}
impl Default for GreasePencilOffsetModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            flag: Default::default(),
            offset_mode: MOD_GREASE_PENCIL_OFFSET_RANDOM,
            loc: [0.0; 3],
            rot: [0.0; 3],
            scale: [0.0; 3],
            stroke_loc: [0.0; 3],
            stroke_rot: [0.0; 3],
            stroke_scale: [0.0; 3],
            seed: 0,
            stroke_step: 1,
            stroke_start_offset: 0,
            pad2: NIL,
        }
    }
}

/// `GreasePencilNoiseModifierData`.
#[derive(Clone)]
pub struct GreasePencilNoiseModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `int flag`
    pub flag: i32,
    /// `float factor`
    pub factor: f32,
    /// `float factor_strength`
    pub factor_strength: f32,
    /// `float factor_thickness`
    pub factor_thickness: f32,
    /// `float factor_uvs`
    pub factor_uvs: f32,
    /// `float noise_scale`
    pub noise_scale: f32,
    /// `float noise_offset`
    pub noise_offset: f32,
    /// `short noise_mode`
    pub noise_mode: i16,
    /// `int step`
    pub step: i32,
    /// `int seed`
    pub seed: i32,
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for GreasePencilNoiseModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            flag: GP_NOISE_FULL_STROKE | GP_NOISE_USE_RANDOM,
            factor: 0.5,
            factor_strength: 0.0,
            factor_thickness: 0.0,
            factor_uvs: 0.0,
            noise_scale: 0.0,
            noise_offset: 0.0,
            noise_mode: 0,
            step: 4,
            seed: 1,
            pad1: NIL,
        }
    }
}

/// `GreasePencilMirrorModifierData`.
#[derive(Clone)]
pub struct GreasePencilMirrorModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `Object *object`
    pub object: usize,
    /// `GreasePencilMirrorModifierFlag flag`
    pub flag: Greasepencilmirrormodifierflag,
}
impl Default for GreasePencilMirrorModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            object: NIL,
            flag: MOD_GREASE_PENCIL_MIRROR_AXIS_X,
        }
    }
}

/// `GreasePencilThickModifierData`.
#[derive(Clone)]
pub struct GreasePencilThickModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `GreasePencilThicknessModifierFlag flag`
    pub flag: Greasepencilthicknessmodifierflag,
    /// `float thickness_fac`
    pub thickness_fac: f32,
    /// `float thickness`
    pub thickness: f32,
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for GreasePencilThickModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            flag: Default::default(),
            thickness_fac: 1.0,
            thickness: 0.02,
            pad1: NIL,
        }
    }
}

/// `GreasePencilLatticeModifierData`.
#[derive(Clone)]
pub struct GreasePencilLatticeModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `Object *object`
    pub object: usize,
    /// `float strength`
    pub strength: f32,
}
impl Default for GreasePencilLatticeModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            object: NIL,
            strength: 1.0,
        }
    }
}

/// `GreasePencilDashModifierSegment`.
#[derive(Clone)]
pub struct GreasePencilDashModifierSegment {
    /// `char name[64]`
    pub name: [u8; 64],
    /// `int dash`
    pub dash: i32,
    /// `int gap`
    pub gap: i32,
    /// `float radius`
    pub radius: f32,
    /// `float opacity`
    pub opacity: f32,
    /// `int mat_nr`
    pub mat_nr: i32,
    /// `GreasePencilDashModifierFlag flag`
    pub flag: Greasepencildashmodifierflag,
}
impl Default for GreasePencilDashModifierSegment {
    fn default() -> Self {
        Self {
            name: [0; 64],
            dash: 2,
            gap: 1,
            radius: 1.0,
            opacity: 1.0,
            mat_nr: -1,
            flag: Default::default(),
        }
    }
}

/// `GreasePencilDashModifierData`.
#[derive(Clone)]
pub struct GreasePencilDashModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `GreasePencilDashModifierSegment *segments_array`
    pub segments_array: usize,
    /// `int segments_num`
    pub segments_num: i32,
    /// `int segment_active_index`
    pub segment_active_index: i32,
    /// `int dash_offset`
    pub dash_offset: i32,
}
impl Default for GreasePencilDashModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            segments_array: NIL,
            segments_num: 0,
            segment_active_index: 0,
            dash_offset: 0,
        }
    }
}

/// `GreasePencilMultiModifierData`.
#[derive(Clone)]
pub struct GreasePencilMultiModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `GreasePencilMultiplyModifierFlag flag`
    pub flag: Greasepencilmultiplymodifierflag,
    /// `int duplications`
    pub duplications: i32,
    /// `float distance`
    pub distance: f32,
    /// `float offset`
    pub offset: f32,
    /// `float fading_center`
    pub fading_center: f32,
    /// `float fading_thickness`
    pub fading_thickness: f32,
    /// `float fading_opacity`
    pub fading_opacity: f32,
    /// `void *_pad`
    pub pad: usize,
}
impl Default for GreasePencilMultiModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            flag: Default::default(),
            duplications: 3,
            distance: 0.1,
            offset: 0.0,
            fading_center: 0.5,
            fading_thickness: 0.5,
            fading_opacity: 0.5,
            pad: NIL,
        }
    }
}

/// `GreasePencilLengthModifierData`.
#[derive(Clone)]
pub struct GreasePencilLengthModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `int flag`
    pub flag: i32,
    /// `float start_fac`
    pub start_fac: f32,
    /// `float end_fac`
    pub end_fac: f32,
    /// `float rand_start_fac`
    pub rand_start_fac: f32,
    /// `float rand_end_fac`
    pub rand_end_fac: f32,
    /// `float rand_offset`
    pub rand_offset: f32,
    /// `float overshoot_fac`
    pub overshoot_fac: f32,
    /// `int seed`
    pub seed: i32,
    /// `int step`
    pub step: i32,
    /// `int mode`
    pub mode: i32,
    /// `float point_density`
    pub point_density: f32,
    /// `float segment_influence`
    pub segment_influence: f32,
    /// `float max_angle`
    pub max_angle: f32,
    /// `void *_pad1`
    pub pad1: usize,
}
impl Default for GreasePencilLengthModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            flag: GP_LENGTH_USE_CURVATURE,
            start_fac: 0.1,
            end_fac: 0.1,
            rand_start_fac: 0.0,
            rand_end_fac: 0.0,
            rand_offset: 0.0,
            overshoot_fac: 0.1,
            seed: 0,
            step: 4,
            mode: 0,
            point_density: 30.0,
            segment_influence: 0.0,
            max_angle: 170.0f32.to_radians(),
            pad1: NIL,
        }
    }
}

/// `GreasePencilWeightAngleModifierData`.
#[derive(Clone)]
pub struct GreasePencilWeightAngleModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `GreasePencilWeightAngleModifierFlag flag`
    pub flag: Greasepencilweightanglemodifierflag,
    /// `float min_weight`
    pub min_weight: f32,
    /// `int16_t axis`
    pub axis: i16,
    /// `GreasePencilWeightAngleModifierSpace space`
    pub space: Greasepencilweightanglemodifierspace,
    /// `float angle`
    pub angle: f32,
    /// `char target_vgname[64]`
    pub target_vgname: [u8; 64],
    /// `void *_pad`
    pub pad: usize,
}
impl Default for GreasePencilWeightAngleModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            flag: Default::default(),
            min_weight: 0.0,
            axis: 1,
            space: MOD_GREASE_PENCIL_WEIGHT_ANGLE_SPACE_LOCAL,
            angle: 0.0,
            target_vgname: [0; 64],
            pad: NIL,
        }
    }
}

/// `GreasePencilArrayModifierData`.
#[derive(Clone)]
pub struct GreasePencilArrayModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `Object *object`
    pub object: usize,
    /// `int count`
    pub count: i32,
    /// `GreasePencilArrayModifierFlag flag`
    pub flag: Greasepencilarraymodifierflag,
    /// `float offset[3]`
    pub offset: [f32; 3],
    /// `float shift[3]`
    pub shift: [f32; 3],
    /// `float rnd_offset[3]`
    pub rnd_offset: [f32; 3],
    /// `float rnd_rot[3]`
    pub rnd_rot: [f32; 3],
    /// `float rnd_scale[3]`
    pub rnd_scale: [f32; 3],
    /// `int seed`
    pub seed: i32,
    /// `int mat_rpl`
    pub mat_rpl: i32,
}
impl Default for GreasePencilArrayModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            object: NIL,
            count: 2,
            flag: MOD_GREASE_PENCIL_ARRAY_USE_RELATIVE,
            offset: [0.0; 3],
            shift: [0.0; 3],
            rnd_offset: [0.0; 3],
            rnd_rot: [0.0; 3],
            rnd_scale: [0.0; 3],
            seed: 1,
            mat_rpl: 0,
        }
    }
}

/// `GreasePencilWeightProximityModifierData`.
#[derive(Clone)]
pub struct GreasePencilWeightProximityModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `GreasePencilWeightProximityFlag flag`
    pub flag: Greasepencilweightproximityflag,
    /// `char target_vgname[64]`
    pub target_vgname: [u8; 64],
    /// `float min_weight`
    pub min_weight: f32,
    /// `float dist_start`
    pub dist_start: f32,
    /// `float dist_end`
    pub dist_end: f32,
    /// `Object *object`
    pub object: usize,
}
impl Default for GreasePencilWeightProximityModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            flag: Default::default(),
            target_vgname: [0; 64],
            min_weight: 0.0,
            dist_start: 0.0,
            dist_end: 20.0,
            object: NIL,
        }
    }
}

/// `GreasePencilHookModifierData`.
#[derive(Clone)]
pub struct GreasePencilHookModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `Object *object`
    pub object: usize,
    /// `char subtarget[64]`
    pub subtarget: [u8; 64],
    /// `GreasePencilHookFlag flag`
    pub flag: Greasepencilhookflag,
    /// `GreasePencilHookFalloff falloff_type`
    pub falloff_type: Greasepencilhookfalloff,
    /// `float parentinv[4][4]`
    pub parentinv: [[f32; 4]; 4],
    /// `float cent[3]`
    pub cent: [f32; 3],
    /// `float falloff`
    pub falloff: f32,
    /// `float force`
    pub force: f32,
}
impl Default for GreasePencilHookModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            object: NIL,
            subtarget: [0; 64],
            flag: Default::default(),
            falloff_type: MOD_GREASE_PENCIL_HOOK_Falloff_Smooth,
            parentinv: [[0.0; 4]; 4],
            cent: [0.0; 3],
            falloff: 0.0,
            force: 0.5,
        }
    }
}

/// `GreasePencilLineartModifierData`.
#[derive(Clone)]
pub struct GreasePencilLineartModifierData {
    pub modifier: ModifierData,
    /// `GreasePencilLineartModifierSource source_type`
    pub source_type: Greasepencillineartmodifiersource,
    /// `char use_multiple_levels`
    pub use_multiple_levels: i8,
    /// `short level_start`
    pub level_start: i16,
    /// `short level_end`
    pub level_end: i16,
    /// `Object *source_camera`
    pub source_camera: usize,
    /// `Object *light_contour_object`
    pub light_contour_object: usize,
    /// `Object *source_object`
    pub source_object: usize,
    /// `Collection *source_collection`
    pub source_collection: usize,
    /// `Material *target_material`
    pub target_material: usize,
    /// `char target_layer[64]`
    pub target_layer: [u8; 64],
    /// `char source_vertex_group[64]`
    pub source_vertex_group: [u8; 64],
    /// `char vgname[64]`
    pub vgname: [u8; 64],
    /// `float overscan`
    pub overscan: f32,
    /// `float shadow_camera_fov`
    pub shadow_camera_fov: f32,
    /// `float shadow_camera_size`
    pub shadow_camera_size: f32,
    /// `float shadow_camera_near`
    pub shadow_camera_near: f32,
    /// `float shadow_camera_far`
    pub shadow_camera_far: f32,
    /// `float opacity`
    pub opacity: f32,
    /// `float radius`
    pub radius: f32,
    /// `short thickness_legacy`
    pub thickness_legacy: i16,
    /// `eGreasePencilLineartMaskSwitches mask_switches`
    pub mask_switches: EGreasepencillineartmaskswitches,
    /// `unsigned_char material_mask_bits`
    pub material_mask_bits: u8,
    /// `unsigned_char intersection_mask`
    pub intersection_mask: u8,
    /// `unsigned_char shadow_selection`
    pub shadow_selection: u8,
    /// `unsigned_char silhouette_selection`
    pub silhouette_selection: u8,
    /// `unsigned_char fill_strokes`
    pub fill_strokes: u8,
    /// `float crease_threshold`
    pub crease_threshold: f32,
    /// `float angle_splitting_threshold`
    pub angle_splitting_threshold: f32,
    /// `float chain_smooth_tolerance`
    pub chain_smooth_tolerance: f32,
    /// `float chaining_image_threshold`
    pub chaining_image_threshold: f32,
    /// `eGreasePencilLineartFlags flags`
    pub flags: Greasepencillineartflags,
    /// `float stroke_depth_offset`
    pub stroke_depth_offset: f32,
    /// `char level_start_override`
    pub level_start_override: i8,
    /// `char level_end_override`
    pub level_end_override: i8,
    /// `short edge_types_override`
    pub edge_types_override: i16,
    /// `char shadow_selection_override`
    pub shadow_selection_override: i8,
    /// `char shadow_use_silhouette_override`
    pub shadow_use_silhouette_override: i8,
    /// `LineartCache *shared_cache`
    pub shared_cache: usize,
    /// `LineartCache *cache`
    pub cache: usize,
    /// `LineartData *la_data_ptr`
    pub la_data_ptr: usize,
    /// `LineartModifierRuntime *runtime`
    pub runtime: usize,
}
impl Default for GreasePencilLineartModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            source_type: LINEART_SOURCE_COLLECTION,
            use_multiple_levels: 0,
            level_start: 0,
            level_end: 0,
            source_camera: NIL,
            light_contour_object: NIL,
            source_object: NIL,
            source_collection: NIL,
            target_material: NIL,
            target_layer: [0; 64],
            source_vertex_group: [0; 64],
            vgname: [0; 64],
            overscan: 0.1,
            shadow_camera_fov: 0.0,
            shadow_camera_size: 200.0,
            shadow_camera_near: 0.1,
            shadow_camera_far: 200.0,
            opacity: 1.0,
            radius: 0.0025,
            thickness_legacy: 25,
            mask_switches: Default::default(),
            material_mask_bits: 0,
            intersection_mask: 0,
            shadow_selection: 0,
            silhouette_selection: 0,
            fill_strokes: 0,
            crease_threshold: 140.0f32.to_radians(),
            angle_splitting_threshold: 0.0,
            chain_smooth_tolerance: 0.0,
            chaining_image_threshold: 0.001,
            flags: Default::default(),
            stroke_depth_offset: 0.05,
            level_start_override: 0,
            level_end_override: 0,
            edge_types_override: 0,
            shadow_selection_override: 0,
            shadow_use_silhouette_override: 0,
            shared_cache: NIL,
            cache: NIL,
            la_data_ptr: NIL,
            runtime: NIL,
        }
    }
}

/// `GreasePencilArmatureModifierData`.
#[derive(Clone)]
pub struct GreasePencilArmatureModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `Object *object`
    pub object: usize,
    /// `short deformflag`
    pub deformflag: i16,
}
impl Default for GreasePencilArmatureModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            object: NIL,
            deformflag: ARM_DEF_VGROUP,
        }
    }
}

/// `GreasePencilTimeModifierSegment`.
#[derive(Clone)]
pub struct GreasePencilTimeModifierSegment {
    /// `char name[64]`
    pub name: [u8; 64],
    /// `int segment_start`
    pub segment_start: i32,
    /// `int segment_end`
    pub segment_end: i32,
    /// `GreasePencilTimeModifierSegmentMode segment_mode`
    pub segment_mode: Greasepenciltimemodifiersegmentmode,
    /// `int segment_repeat`
    pub segment_repeat: i32,
}
impl Default for GreasePencilTimeModifierSegment {
    fn default() -> Self {
        Self {
            name: [0; 64],
            segment_start: 1,
            segment_end: 2,
            segment_mode: MOD_GREASE_PENCIL_TIME_SEG_MODE_NORMAL,
            segment_repeat: 1,
        }
    }
}

/// `GreasePencilTimeModifierData`.
#[derive(Clone)]
pub struct GreasePencilTimeModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `GreasePencilTimeModifierFlag flag`
    pub flag: Greasepenciltimemodifierflag,
    /// `int offset`
    pub offset: i32,
    /// `float frame_scale`
    pub frame_scale: f32,
    /// `GreasePencilTimeModifierMode mode`
    pub mode: Greasepenciltimemodifiermode,
    /// `int sfra`
    pub sfra: i32,
    /// `int efra`
    pub efra: i32,
    /// `GreasePencilTimeModifierSegment *segments_array`
    pub segments_array: usize,
    /// `int segments_num`
    pub segments_num: i32,
    /// `int segment_active_index`
    pub segment_active_index: i32,
}
impl Default for GreasePencilTimeModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            flag: MOD_GREASE_PENCIL_TIME_KEEP_LOOP,
            offset: 1,
            frame_scale: 1.0,
            mode: MOD_GREASE_PENCIL_TIME_MODE_NORMAL,
            sfra: 1,
            efra: 250,
            segments_array: NIL,
            segments_num: 1,
            segment_active_index: 0,
        }
    }
}

/// `GreasePencilEnvelopeModifierData`.
#[derive(Clone)]
pub struct GreasePencilEnvelopeModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `GreasePencilEnvelopeModifierMode mode`
    pub mode: Greasepencilenvelopemodifiermode,
    /// `int mat_nr`
    pub mat_nr: i32,
    /// `float thickness`
    pub thickness: f32,
    /// `float strength`
    pub strength: f32,
    /// `int skip`
    pub skip: i32,
    /// `int spread`
    pub spread: i32,
}
impl Default for GreasePencilEnvelopeModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            mode: MOD_GREASE_PENCIL_ENVELOPE_SEGMENTS,
            mat_nr: -1,
            thickness: 1.0,
            strength: 1.0,
            skip: 0,
            spread: 10,
        }
    }
}

/// `GreasePencilOutlineModifierData`.
#[derive(Clone)]
pub struct GreasePencilOutlineModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `Object *object`
    pub object: usize,
    /// `GreasePencilOutlineModifierFlag flag`
    pub flag: Greasepenciloutlinemodifierflag,
    /// `int thickness`
    pub thickness: i32,
    /// `float sample_length`
    pub sample_length: f32,
    /// `int subdiv`
    pub subdiv: i32,
    /// `Material *outline_material`
    pub outline_material: usize,
}
impl Default for GreasePencilOutlineModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            object: NIL,
            flag: MOD_GREASE_PENCIL_OUTLINE_KEEP_SHAPE,
            thickness: 1,
            sample_length: 0.0,
            subdiv: 3,
            outline_material: NIL,
        }
    }
}

/// `GreasePencilShrinkwrapModifierData`.
#[derive(Clone)]
pub struct GreasePencilShrinkwrapModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `Object *target`
    pub target: usize,
    /// `Object *aux_target`
    pub aux_target: usize,
    /// `float keep_dist`
    pub keep_dist: f32,
    /// `short shrink_type`
    pub shrink_type: i16,
    /// `char shrink_opts`
    pub shrink_opts: i8,
    /// `char shrink_mode`
    pub shrink_mode: i8,
    /// `float proj_limit`
    pub proj_limit: f32,
    /// `char proj_axis`
    pub proj_axis: i8,
    /// `char subsurf_levels`
    pub subsurf_levels: i8,
    /// `float smooth_factor`
    pub smooth_factor: f32,
    /// `int smooth_step`
    pub smooth_step: i32,
    /// `ShrinkwrapTreeData *cache_data`
    pub cache_data: usize,
}
impl Default for GreasePencilShrinkwrapModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            target: NIL,
            aux_target: NIL,
            keep_dist: 0.05,
            shrink_type: MOD_SHRINKWRAP_NEAREST_SURFACE,
            shrink_opts: MOD_SHRINKWRAP_PROJECT_ALLOW_POS_DIR as i8,
            shrink_mode: MOD_SHRINKWRAP_ON_SURFACE,
            proj_limit: 0.0,
            proj_axis: MOD_SHRINKWRAP_PROJECT_OVER_NORMAL,
            subsurf_levels: 0,
            smooth_factor: 0.05,
            smooth_step: 1,
            cache_data: NIL,
        }
    }
}

/// `GreasePencilBuildModifierData`.
#[derive(Clone)]
pub struct GreasePencilBuildModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `float start_frame`
    pub start_frame: f32,
    /// `float end_frame`
    pub end_frame: f32,
    /// `float start_delay`
    pub start_delay: f32,
    /// `float length`
    pub length: f32,
    /// `GreasePencilBuildFlag flag`
    pub flag: Greasepencilbuildflag,
    /// `GreasePencilBuildMode mode`
    pub mode: Greasepencilbuildmode,
    /// `GreasePencilBuildTransition transition`
    pub transition: Greasepencilbuildtransition,
    /// `GreasePencilBuildTimeAlignment time_alignment`
    pub time_alignment: Greasepencilbuildtimealignment,
    /// `float speed_fac`
    pub speed_fac: f32,
    /// `float speed_maxgap`
    pub speed_maxgap: f32,
    /// `GreasePencilBuildTimeMode time_mode`
    pub time_mode: Greasepencilbuildtimemode,
    /// `Object *object`
    pub object: usize,
    /// `float percentage_fac`
    pub percentage_fac: f32,
    /// `float fade_fac`
    pub fade_fac: f32,
    /// `char target_vgname[64]`
    pub target_vgname: [u8; 64],
    /// `float fade_opacity_strength`
    pub fade_opacity_strength: f32,
    /// `float fade_thickness_strength`
    pub fade_thickness_strength: f32,
}
impl Default for GreasePencilBuildModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            start_frame: 1.0,
            end_frame: 125.0,
            start_delay: 0.0,
            length: 100.0,
            flag: Default::default(),
            mode: MOD_GREASE_PENCIL_BUILD_MODE_SEQUENTIAL,
            transition: MOD_GREASE_PENCIL_BUILD_TRANSITION_GROW,
            time_alignment: MOD_GREASE_PENCIL_BUILD_TIMEALIGN_START,
            speed_fac: 1.2,
            speed_maxgap: 0.5,
            time_mode: MOD_GREASE_PENCIL_BUILD_TIMEMODE_FRAMES,
            object: NIL,
            percentage_fac: 0.0,
            fade_fac: 0.0,
            target_vgname: [0; 64],
            fade_opacity_strength: 0.0,
            fade_thickness_strength: 0.0,
        }
    }
}

/// `GreasePencilSimplifyModifierData`.
#[derive(Clone)]
pub struct GreasePencilSimplifyModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `GreasePencilSimplifyModifierMode mode`
    pub mode: Greasepencilsimplifymodifiermode,
    /// `short step`
    pub step: i16,
    /// `float factor`
    pub factor: f32,
    /// `float length`
    pub length: f32,
    /// `float sharp_threshold`
    pub sharp_threshold: f32,
    /// `float distance`
    pub distance: f32,
}
impl Default for GreasePencilSimplifyModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            mode: MOD_GREASE_PENCIL_SIMPLIFY_FIXED,
            step: 1,
            factor: 0.0,
            length: 0.1,
            sharp_threshold: 0.0,
            distance: 0.1,
        }
    }
}

/// `GreasePencilTextureModifierData`.
#[derive(Clone)]
pub struct GreasePencilTextureModifierData {
    pub modifier: ModifierData,
    pub influence: GreasePencilModifierInfluenceData,
    /// `float uv_offset`
    pub uv_offset: f32,
    /// `float uv_scale`
    pub uv_scale: f32,
    /// `float fill_rotation`
    pub fill_rotation: f32,
    /// `float fill_offset[2]`
    pub fill_offset: [f32; 2],
    /// `float fill_scale`
    pub fill_scale: f32,
    /// `int layer_pass`
    pub layer_pass: i32,
    /// `GreasePencilTextureModifierFit fit_method`
    pub fit_method: Greasepenciltexturemodifierfit,
    /// `GreasePencilTextureModifierMode mode`
    pub mode: Greasepenciltexturemodifiermode,
    /// `float alignment_rotation`
    pub alignment_rotation: f32,
}
impl Default for GreasePencilTextureModifierData {
    fn default() -> Self {
        Self {
            modifier: Default::default(),
            influence: Default::default(),
            uv_offset: 0.0,
            uv_scale: 1.0,
            fill_rotation: 0.0,
            fill_offset: [0.0; 2],
            fill_scale: 1.0,
            layer_pass: 0,
            fit_method: MOD_GREASE_PENCIL_TEXTURE_CONSTANT_LENGTH,
            mode: MOD_GREASE_PENCIL_TEXTURE_STROKE,
            alignment_rotation: 0.0,
        }
    }
}
