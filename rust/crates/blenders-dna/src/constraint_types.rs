//! Port of `DNA_constraint_types.h` — `bConstraint` + all type-specific data structs.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use crate::id::NIL;
use blenders_bli::listbase::ListBase;
/// `eConstraintTargetFlag`.
pub type Constrainttargetflag = i16;
pub const CONSTRAINT_TAR_TEMP: Constrainttargetflag = 1 << 0;
pub const CONSTRAINT_TAR_CUSTOM_SPACE: Constrainttargetflag = 1 << 1;

/// `eConstraintObType`.
pub type Constraintobtype = i16;
pub const CONSTRAINT_OBTYPE_OBJECT: Constraintobtype = 1;
pub const CONSTRAINT_OBTYPE_BONE: Constraintobtype = 2;
pub const CONSTRAINT_OBTYPE_VERT: Constraintobtype = 3;

/// `eConstraint_IK_Type`.
pub type ConstraintIkType = i16;
pub const CONSTRAINT_IK_COPYPOSE: ConstraintIkType = 0;
pub const CONSTRAINT_IK_DISTANCE: ConstraintIkType = 1;

/// `eGeometryAttributeConstraint_Flags`.
pub type GeometryattributeconstraintFlags = u8;
pub const APPLY_TARGET_TRANSFORM: GeometryattributeconstraintFlags = 1 << 0;
pub const MIX_LOC: GeometryattributeconstraintFlags = 1 << 1;
pub const MIX_ROT: GeometryattributeconstraintFlags = 1 << 2;
pub const MIX_SCALE: GeometryattributeconstraintFlags = 1 << 3;

/// `Attribute_Domain`.
pub type AttributeDomain = u8;
pub const CON_ATTRIBUTE_DOMAIN_POINT: AttributeDomain = 0;
pub const CON_ATTRIBUTE_DOMAIN_EDGE: AttributeDomain = 1;
pub const CON_ATTRIBUTE_DOMAIN_FACE: AttributeDomain = 2;
pub const CON_ATTRIBUTE_DOMAIN_FACE_CORNER: AttributeDomain = 3;
pub const CON_ATTRIBUTE_DOMAIN_CURVE: AttributeDomain = 4;
pub const CON_ATTRIBUTE_DOMAIN_INSTANCE: AttributeDomain = 5;

/// `Attribute_Data_Type`.
pub type AttributeDataType = u8;
pub const CON_ATTRIBUTE_VECTOR: AttributeDataType = 0;
pub const CON_ATTRIBUTE_QUATERNION: AttributeDataType = 1;
pub const CON_ATTRIBUTE_4X4MATRIX: AttributeDataType = 2;

/// `Attribute_MixMode`.
pub type AttributeMixmode = u8;
pub const CON_ATTRIBUTE_MIX_REPLACE: AttributeMixmode = 0;
pub const CON_ATTRIBUTE_MIX_BEFORE_SPLIT: AttributeMixmode = 1;
pub const CON_ATTRIBUTE_MIX_AFTER_SPLIT: AttributeMixmode = 2;
pub const CON_ATTRIBUTE_MIX_BEFORE_FULL: AttributeMixmode = 3;
pub const CON_ATTRIBUTE_MIX_AFTER_FULL: AttributeMixmode = 4;

/// `eBConstraint_Types`.
pub type BconstraintTypes = i16;
pub const CONSTRAINT_TYPE_NULL: BconstraintTypes = 0;
pub const CONSTRAINT_TYPE_CHILDOF: BconstraintTypes = 1;
pub const CONSTRAINT_TYPE_TRACKTO: BconstraintTypes = 2;
pub const CONSTRAINT_TYPE_KINEMATIC: BconstraintTypes = 3;
pub const CONSTRAINT_TYPE_FOLLOWPATH: BconstraintTypes = 4;
pub const CONSTRAINT_TYPE_ROTLIMIT: BconstraintTypes = 5;
pub const CONSTRAINT_TYPE_LOCLIMIT: BconstraintTypes = 6;
pub const CONSTRAINT_TYPE_SIZELIMIT: BconstraintTypes = 7;
pub const CONSTRAINT_TYPE_ROTLIKE: BconstraintTypes = 8;
pub const CONSTRAINT_TYPE_LOCLIKE: BconstraintTypes = 9;
pub const CONSTRAINT_TYPE_SIZELIKE: BconstraintTypes = 10;
pub const CONSTRAINT_TYPE_ACTION: BconstraintTypes = 12;
pub const CONSTRAINT_TYPE_LOCKTRACK: BconstraintTypes = 13;
pub const CONSTRAINT_TYPE_DISTLIMIT: BconstraintTypes = 14;
pub const CONSTRAINT_TYPE_STRETCHTO: BconstraintTypes = 15;
pub const CONSTRAINT_TYPE_MINMAX: BconstraintTypes = 16;
pub const CONSTRAINT_TYPE_CLAMPTO: BconstraintTypes = 18;
pub const CONSTRAINT_TYPE_TRANSFORM: BconstraintTypes = 19;
pub const CONSTRAINT_TYPE_SHRINKWRAP: BconstraintTypes = 20;
pub const CONSTRAINT_TYPE_DAMPTRACK: BconstraintTypes = 21;
pub const CONSTRAINT_TYPE_SPLINEIK: BconstraintTypes = 22;
pub const CONSTRAINT_TYPE_TRANSLIKE: BconstraintTypes = 23;
pub const CONSTRAINT_TYPE_SAMEVOL: BconstraintTypes = 24;
pub const CONSTRAINT_TYPE_PIVOT: BconstraintTypes = 25;
pub const CONSTRAINT_TYPE_FOLLOWTRACK: BconstraintTypes = 26;
pub const CONSTRAINT_TYPE_CAMERASOLVER: BconstraintTypes = 27;
pub const CONSTRAINT_TYPE_OBJECTSOLVER: BconstraintTypes = 28;
pub const CONSTRAINT_TYPE_TRANSFORM_CACHE: BconstraintTypes = 29;
pub const CONSTRAINT_TYPE_ARMATURE: BconstraintTypes = 30;
pub const CONSTRAINT_TYPE_GEOMETRY_ATTRIBUTE: BconstraintTypes = 31;
pub const NUM_CONSTRAINT_TYPES: BconstraintTypes = CONSTRAINT_TYPE_GEOMETRY_ATTRIBUTE + 1;

/// `eBConstraint_Flags`.
pub type BconstraintFlags = i16;
pub const CONSTRAINT_EXPAND_DEPRECATED: BconstraintFlags = 1 << 0;
pub const CONSTRAINT_DISABLE: BconstraintFlags = 1 << 2;
pub const CONSTRAINT_ACTIVE: BconstraintFlags = 1 << 4;
pub const CONSTRAINT_SPACEONCE: BconstraintFlags = 1 << 6;
pub const CONSTRAINT_OWN_IPO: BconstraintFlags = 1 << 7;
pub const CONSTRAINT_OFF: BconstraintFlags = 1 << 9;
pub const CONSTRAINT_BBONE_SHAPE: BconstraintFlags = 1 << 10;
pub const CONSTRAINT_OVERRIDE_LIBRARY_LOCAL: BconstraintFlags = 1 << 11;
pub const CONSTRAINT_BBONE_SHAPE_FULL: BconstraintFlags = 1 << 12;

/// `eBConstraint_SpaceTypes`.
pub type BconstraintSpacetypes = i8;
pub const CONSTRAINT_SPACE_WORLD: BconstraintSpacetypes = 0;
pub const CONSTRAINT_SPACE_CUSTOM: BconstraintSpacetypes = 5;
pub const CONSTRAINT_SPACE_LOCAL: BconstraintSpacetypes = 1;
pub const CONSTRAINT_SPACE_POSE: BconstraintSpacetypes = 2;
pub const CONSTRAINT_SPACE_PARLOCAL: BconstraintSpacetypes = 3;
pub const CONSTRAINT_SPACE_OWNLOCAL: BconstraintSpacetypes = 6;
pub const CONSTRAINT_SPACE_INVALID: BconstraintSpacetypes = 4;

/// `eConstraint_EulerOrder`.
pub type ConstraintEulerorder = i8;
pub const CONSTRAINT_EULER_AUTO: ConstraintEulerorder = 0;
pub const CONSTRAINT_EULER_XYZ: ConstraintEulerorder = 1;
pub const CONSTRAINT_EULER_XZY: ConstraintEulerorder = 2;
pub const CONSTRAINT_EULER_YXZ: ConstraintEulerorder = 3;
pub const CONSTRAINT_EULER_YZX: ConstraintEulerorder = 4;
pub const CONSTRAINT_EULER_ZXY: ConstraintEulerorder = 5;
pub const CONSTRAINT_EULER_ZYX: ConstraintEulerorder = 6;

/// `eCopyRotation_Flags`.
pub type CopyrotationFlags = i32;
pub const ROTLIKE_X: CopyrotationFlags = 1 << 0;
pub const ROTLIKE_Y: CopyrotationFlags = 1 << 1;
pub const ROTLIKE_Z: CopyrotationFlags = 1 << 2;
pub const ROTLIKE_X_INVERT: CopyrotationFlags = 1 << 4;
pub const ROTLIKE_Y_INVERT: CopyrotationFlags = 1 << 5;
pub const ROTLIKE_Z_INVERT: CopyrotationFlags = 1 << 6;
pub const ROTLIKE_OFFSET: CopyrotationFlags = 1 << 7;

/// `eCopyRotation_MixMode`.
pub type CopyrotationMixmode = i8;
pub const ROTLIKE_MIX_REPLACE: CopyrotationMixmode = 0;
pub const ROTLIKE_MIX_OFFSET: CopyrotationMixmode = 1;
pub const ROTLIKE_MIX_ADD: CopyrotationMixmode = 2;
pub const ROTLIKE_MIX_BEFORE: CopyrotationMixmode = 3;
pub const ROTLIKE_MIX_AFTER: CopyrotationMixmode = 4;

/// `eCopyLocation_Flags`.
pub type CopylocationFlags = i32;
pub const LOCLIKE_X: CopylocationFlags = 1 << 0;
pub const LOCLIKE_Y: CopylocationFlags = 1 << 1;
pub const LOCLIKE_Z: CopylocationFlags = 1 << 2;
pub const LOCLIKE_TIP: CopylocationFlags = 1 << 3;
pub const LOCLIKE_X_INVERT: CopylocationFlags = 1 << 4;
pub const LOCLIKE_Y_INVERT: CopylocationFlags = 1 << 5;
pub const LOCLIKE_Z_INVERT: CopylocationFlags = 1 << 6;
pub const LOCLIKE_OFFSET: CopylocationFlags = 1 << 7;

/// `eCopyScale_Flags`.
pub type CopyscaleFlags = i32;
pub const SIZELIKE_X: CopyscaleFlags = 1 << 0;
pub const SIZELIKE_Y: CopyscaleFlags = 1 << 1;
pub const SIZELIKE_Z: CopyscaleFlags = 1 << 2;
pub const SIZELIKE_OFFSET: CopyscaleFlags = 1 << 3;
pub const SIZELIKE_MULTIPLY: CopyscaleFlags = 1 << 4;
pub const SIZELIKE_UNIFORM: CopyscaleFlags = 1 << 5;

/// `eCopyTransforms_Flags`.
pub type CopytransformsFlags = i32;
pub const TRANSLIKE_REMOVE_TARGET_SHEAR: CopytransformsFlags = 1 << 0;

/// `eCopyTransforms_MixMode`.
pub type CopytransformsMixmode = i8;
pub const TRANSLIKE_MIX_REPLACE: CopytransformsMixmode = 0;
pub const TRANSLIKE_MIX_BEFORE: CopytransformsMixmode = 1;
pub const TRANSLIKE_MIX_AFTER: CopytransformsMixmode = 2;
pub const TRANSLIKE_MIX_BEFORE_SPLIT: CopytransformsMixmode = 3;
pub const TRANSLIKE_MIX_AFTER_SPLIT: CopytransformsMixmode = 4;
pub const TRANSLIKE_MIX_BEFORE_FULL: CopytransformsMixmode = 5;
pub const TRANSLIKE_MIX_AFTER_FULL: CopytransformsMixmode = 6;

/// `eTransform_ToFrom`.
pub type TransformTofrom = i16;
pub const TRANS_LOCATION: TransformTofrom = 0;
pub const TRANS_ROTATION: TransformTofrom = 1;
pub const TRANS_SCALE: TransformTofrom = 2;

/// `eTransform_MixModeLoc`.
pub type TransformMixmodeloc = i8;
pub const TRANS_MIXLOC_ADD: TransformMixmodeloc = 0;
pub const TRANS_MIXLOC_REPLACE: TransformMixmodeloc = 1;

/// `eTransform_MixModeRot`.
pub type TransformMixmoderot = i8;
pub const TRANS_MIXROT_ADD: TransformMixmoderot = 0;
pub const TRANS_MIXROT_REPLACE: TransformMixmoderot = 1;
pub const TRANS_MIXROT_BEFORE: TransformMixmoderot = 2;
pub const TRANS_MIXROT_AFTER: TransformMixmoderot = 3;

/// `eTransform_MixModeScale`.
pub type TransformMixmodescale = i8;
pub const TRANS_MIXSCALE_REPLACE: TransformMixmodescale = 0;
pub const TRANS_MIXSCALE_MULTIPLY: TransformMixmodescale = 1;

/// `eSameVolume_Axis`.
pub type SamevolumeAxis = i8;
pub const SAMEVOL_X: SamevolumeAxis = 0;
pub const SAMEVOL_Y: SamevolumeAxis = 1;
pub const SAMEVOL_Z: SamevolumeAxis = 2;

/// `eSameVolume_Mode`.
pub type SamevolumeMode = i8;
pub const SAMEVOL_STRICT: SamevolumeMode = 0;
pub const SAMEVOL_UNIFORM: SamevolumeMode = 1;
pub const SAMEVOL_SINGLE_AXIS: SamevolumeMode = 2;

/// `eActionConstraint_Flags`.
pub type ActionconstraintFlags = i32;
pub const ACTCON_BONE_USE_OBJECT_ACTION: ActionconstraintFlags = 1 << 0;
pub const ACTCON_USE_EVAL_TIME: ActionconstraintFlags = 1 << 1;

/// `eActionConstraint_MixMode`.
pub type ActionconstraintMixmode = i8;
pub const ACTCON_MIX_REPLACE: ActionconstraintMixmode = 6;
pub const ACTCON_MIX_AFTER_FULL: ActionconstraintMixmode = 0;
pub const ACTCON_MIX_BEFORE_FULL: ActionconstraintMixmode = 3;
pub const ACTCON_MIX_AFTER: ActionconstraintMixmode = 1;
pub const ACTCON_MIX_BEFORE: ActionconstraintMixmode = 2;
pub const ACTCON_MIX_AFTER_SPLIT: ActionconstraintMixmode = 4;
pub const ACTCON_MIX_BEFORE_SPLIT: ActionconstraintMixmode = 5;

/// `eLockAxis_Modes`.
pub type LockaxisModes = i32;
pub const LOCK_X: LockaxisModes = 0;
pub const LOCK_Y: LockaxisModes = 1;
pub const LOCK_Z: LockaxisModes = 2;

/// `eUpAxis_Modes`.
pub type UpaxisModes = i32;
pub const UP_X: UpaxisModes = 0;
pub const UP_Y: UpaxisModes = 1;
pub const UP_Z: UpaxisModes = 2;

/// `eTrackToAxis_Modes`.
pub type TracktoaxisModes = i32;
pub const TRACK_X: TracktoaxisModes = 0;
pub const TRACK_Y: TracktoaxisModes = 1;
pub const TRACK_Z: TracktoaxisModes = 2;
pub const TRACK_nX: TracktoaxisModes = 3;
pub const TRACK_nY: TracktoaxisModes = 4;
pub const TRACK_nZ: TracktoaxisModes = 5;

/// `eShrinkwrap_Flags`.
pub type ShrinkwrapFlags = i8;
pub const CON_SHRINKWRAP_PROJECT_OPPOSITE: ShrinkwrapFlags = 1 << 0;
pub const CON_SHRINKWRAP_PROJECT_INVERT_CULL: ShrinkwrapFlags = 1 << 1;
pub const CON_SHRINKWRAP_TRACK_NORMAL: ShrinkwrapFlags = 1 << 2;
pub const CON_SHRINKWRAP_PROJECT_CULL_FRONTFACE: ShrinkwrapFlags = 1 << 3;
pub const CON_SHRINKWRAP_PROJECT_CULL_BACKFACE: ShrinkwrapFlags = 1 << 4;

/// `eFollowPath_Flags`.
pub type FollowpathFlags = i32;
pub const FOLLOWPATH_FOLLOW: FollowpathFlags = 1 << 0;
pub const FOLLOWPATH_STATIC: FollowpathFlags = 1 << 1;
pub const FOLLOWPATH_RADIUS: FollowpathFlags = 1 << 2;

/// `eTrackTo_Flags`.
pub type TracktoFlags = i32;
pub const TARGET_Z_UP: TracktoFlags = 1 << 0;

/// `eStretchTo_VolMode`.
pub type StretchtoVolmode = i32;
pub const VOLUME_XZ: StretchtoVolmode = 0;
pub const VOLUME_X: StretchtoVolmode = 1;
pub const VOLUME_Z: StretchtoVolmode = 2;
pub const NO_VOLUME: StretchtoVolmode = 3;

/// `eStretchTo_PlaneMode`.
pub type StretchtoPlanemode = i32;
pub const PLANE_X: StretchtoPlanemode = 0;
pub const SWING_Y: StretchtoPlanemode = 1;
pub const PLANE_Z: StretchtoPlanemode = 2;

/// `eClampTo_Modes`.
pub type ClamptoModes = i32;
pub const CLAMPTO_AUTO: ClamptoModes = 0;
pub const CLAMPTO_X: ClamptoModes = 1;
pub const CLAMPTO_Y: ClamptoModes = 2;
pub const CLAMPTO_Z: ClamptoModes = 3;

/// `eClampTo_Flags`.
pub type ClamptoFlags = i32;
pub const CLAMPTO_CYCLIC: ClamptoFlags = 1 << 0;

/// `eKinematic_Flags`.
pub type KinematicFlags = i16;
pub const CONSTRAINT_IK_TIP: KinematicFlags = 1 << 0;
pub const CONSTRAINT_IK_ROT: KinematicFlags = 1 << 1;
pub const CONSTRAINT_IK_AUTO: KinematicFlags = 1 << 2;
pub const CONSTRAINT_IK_TEMP: KinematicFlags = 1 << 3;
pub const CONSTRAINT_IK_STRETCH: KinematicFlags = 1 << 4;
pub const CONSTRAINT_IK_POS: KinematicFlags = 1 << 5;
pub const CONSTRAINT_IK_SETANGLE: KinematicFlags = 1 << 6;
pub const CONSTRAINT_IK_GETANGLE: KinematicFlags = 1 << 7;
pub const CONSTRAINT_IK_NO_POS_X: KinematicFlags = 1 << 8;
pub const CONSTRAINT_IK_NO_POS_Y: KinematicFlags = 1 << 9;
pub const CONSTRAINT_IK_NO_POS_Z: KinematicFlags = 1 << 10;
pub const CONSTRAINT_IK_NO_ROT_X: KinematicFlags = 1 << 11;
pub const CONSTRAINT_IK_NO_ROT_Y: KinematicFlags = 1 << 12;
pub const CONSTRAINT_IK_NO_ROT_Z: KinematicFlags = 1 << 13;
pub const CONSTRAINT_IK_TARGETAXIS: KinematicFlags = 1 << 14;

/// `eSplineIK_Flags`.
pub type SplineikFlags = i16;
pub const CONSTRAINT_SPLINEIK_BOUND: SplineikFlags = 1 << 0;
pub const CONSTRAINT_SPLINEIK_NO_ROOT: SplineikFlags = 1 << 1;
pub const CONSTRAINT_SPLINEIK_SCALE_LIMITED: SplineikFlags = 1 << 2;
pub const CONSTRAINT_SPLINEIK_EVENSPLITS: SplineikFlags = 1 << 3;
pub const CONSTRAINT_SPLINEIK_NO_CURVERAD: SplineikFlags = 1 << 4;
pub const CONSTRAINT_SPLINEIK_USE_BULGE_MIN: SplineikFlags = 1 << 5;
pub const CONSTRAINT_SPLINEIK_USE_BULGE_MAX: SplineikFlags = 1 << 6;
pub const CONSTRAINT_SPLINEIK_USE_ORIGINAL_SCALE: SplineikFlags = 1 << 7;

/// `eSplineIK_XZScaleModes`.
pub type SplineikXzscalemodes = i16;
pub const CONSTRAINT_SPLINEIK_XZS_NONE: SplineikXzscalemodes = 0;
pub const CONSTRAINT_SPLINEIK_XZS_ORIGINAL: SplineikXzscalemodes = 1;
pub const CONSTRAINT_SPLINEIK_XZS_INVERSE: SplineikXzscalemodes = 2;
pub const CONSTRAINT_SPLINEIK_XZS_VOLUMETRIC: SplineikXzscalemodes = 3;

/// `eSplineIK_YScaleModes`.
pub type SplineikYscalemodes = i16;
pub const CONSTRAINT_SPLINEIK_YS_NONE: SplineikYscalemodes = 0;
pub const CONSTRAINT_SPLINEIK_YS_FIT_CURVE: SplineikYscalemodes = 1;
pub const CONSTRAINT_SPLINEIK_YS_ORIGINAL: SplineikYscalemodes = 2;

/// `eArmature_Flags`.
pub type ArmatureFlags = i32;
pub const CONSTRAINT_ARMATURE_QUATERNION: ArmatureFlags = 1 << 0;
pub const CONSTRAINT_ARMATURE_ENVELOPE: ArmatureFlags = 1 << 1;
pub const CONSTRAINT_ARMATURE_CUR_LOCATION: ArmatureFlags = 1 << 2;

/// `eFloor_Flags`.
pub type FloorFlags = i32;
pub const MINMAX_USEROT: FloorFlags = 1 << 2;

/// `eTransformLimits_Flags2`.
pub type TransformlimitsFlags2 = i16;
pub const LIMIT_TRANSFORM: TransformlimitsFlags2 = 1 << 1;

/// `eTransformLimits_Flags`.
pub type TransformlimitsFlags = i16;
pub const LIMIT_XMIN: TransformlimitsFlags = 1 << 0;
pub const LIMIT_XMAX: TransformlimitsFlags = 1 << 1;
pub const LIMIT_YMIN: TransformlimitsFlags = 1 << 2;
pub const LIMIT_YMAX: TransformlimitsFlags = 1 << 3;
pub const LIMIT_ZMIN: TransformlimitsFlags = 1 << 4;
pub const LIMIT_ZMAX: TransformlimitsFlags = 1 << 5;

/// `eRotLimit_Flags`.
pub type RotlimitFlags = i16;
pub const LIMIT_XROT: RotlimitFlags = 1 << 0;
pub const LIMIT_YROT: RotlimitFlags = 1 << 1;
pub const LIMIT_ZROT: RotlimitFlags = 1 << 2;
pub const LIMIT_ROT_LEGACY_BEHAVIOR: RotlimitFlags = 1 << 3;

/// `eDistLimit_Flag`.
pub type DistlimitFlag = i16;
pub const LIMITDIST_USESOFT: DistlimitFlag = 1 << 0;
pub const LIMITDIST_TRANSFORM: DistlimitFlag = 1 << 1;

/// `eDistLimit_Modes`.
pub type DistlimitModes = i16;
pub const LIMITDIST_INSIDE: DistlimitModes = 0;
pub const LIMITDIST_OUTSIDE: DistlimitModes = 1;
pub const LIMITDIST_ONSURFACE: DistlimitModes = 2;

/// `eChildOf_Flags`.
pub type ChildofFlags = i32;
pub const CHILDOF_LOCX: ChildofFlags = 1 << 0;
pub const CHILDOF_LOCY: ChildofFlags = 1 << 1;
pub const CHILDOF_LOCZ: ChildofFlags = 1 << 2;
pub const CHILDOF_ROTX: ChildofFlags = 1 << 3;
pub const CHILDOF_ROTY: ChildofFlags = 1 << 4;
pub const CHILDOF_ROTZ: ChildofFlags = 1 << 5;
pub const CHILDOF_SIZEX: ChildofFlags = 1 << 6;
pub const CHILDOF_SIZEY: ChildofFlags = 1 << 7;
pub const CHILDOF_SIZEZ: ChildofFlags = 1 << 8;
pub const CHILDOF_ALL: ChildofFlags = 511;
pub const CHILDOF_SET_INVERSE: ChildofFlags = 1 << 9;

/// `ePivotConstraint_Axis`.
pub type PivotconstraintAxis = i16;
pub const PIVOTCON_AXIS_NONE: PivotconstraintAxis = -1;
pub const PIVOTCON_AXIS_X_NEG: PivotconstraintAxis = 0;
pub const PIVOTCON_AXIS_Y_NEG: PivotconstraintAxis = 1;
pub const PIVOTCON_AXIS_Z_NEG: PivotconstraintAxis = 2;
pub const PIVOTCON_AXIS_X: PivotconstraintAxis = 3;
pub const PIVOTCON_AXIS_Y: PivotconstraintAxis = 4;
pub const PIVOTCON_AXIS_Z: PivotconstraintAxis = 5;

/// `ePivotConstraint_Flag`.
pub type PivotconstraintFlag = i16;
pub const PIVOTCON_FLAG_OFFSET_ABS: PivotconstraintFlag = 1 << 0;
pub const PIVOTCON_FLAG_ROTACT_NEG: PivotconstraintFlag = 1 << 1;

/// `eFollowTrack_Flags`.
pub type FollowtrackFlags = i32;
pub const FOLLOWTRACK_ACTIVECLIP: FollowtrackFlags = 1 << 0;
pub const FOLLOWTRACK_USE_3D_POSITION: FollowtrackFlags = 1 << 1;
pub const FOLLOWTRACK_USE_UNDISTORTION: FollowtrackFlags = 1 << 2;

/// `eFollowTrack_FrameMethod`.
pub type FollowtrackFramemethod = i32;
pub const FOLLOWTRACK_FRAME_STRETCH: FollowtrackFramemethod = 0;
pub const FOLLOWTRACK_FRAME_FIT: FollowtrackFramemethod = 1;
pub const FOLLOWTRACK_FRAME_CROP: FollowtrackFramemethod = 2;

/// `eCameraSolver_Flags`.
pub type CamerasolverFlags = i32;
pub const CAMERASOLVER_ACTIVECLIP: CamerasolverFlags = 1 << 0;

/// `eObjectSolver_Flags`.
pub type ObjectsolverFlags = i32;
pub const OBJECTSOLVER_ACTIVECLIP: ObjectsolverFlags = 1 << 0;
pub const OBJECTSOLVER_SET_INVERSE: ObjectsolverFlags = 1 << 1;

/// `eStretchTo_Flags`.
pub type StretchtoFlags = i32;
pub const STRETCHTOCON_USE_BULGE_MIN: StretchtoFlags = 1 << 0;
pub const STRETCHTOCON_USE_BULGE_MAX: StretchtoFlags = 1 << 1;


/// `bConstraint`.
#[derive(Clone)]
pub struct BConstraint {
    pub next: usize,
    pub prev: usize,
    /// `void *data` → type-specific constraint ref.
    pub data: usize,
    pub type_: i16,
    pub flag: i16,
    pub ownspace: i8,
    pub tarspace: i8,
    pub ui_expand_flag: i16,
    /// `Object *space_object`.
    pub space_object: usize,
    /// `char space_subtarget[64]`, `name[64]`.
    pub space_subtarget: [u8; 64],
    pub name: [u8; 64],
    pub enforce: f32,
    pub headtail: f32,
    pub lin_error: f32,
    pub rot_error: f32,
}
impl Default for BConstraint {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            data: NIL,
            type_: CONSTRAINT_TYPE_NULL,
            flag: 0,
            ownspace: CONSTRAINT_SPACE_WORLD as i8,
            tarspace: CONSTRAINT_SPACE_WORLD as i8,
            ui_expand_flag: 0,
            space_object: NIL,
            space_subtarget: [0; 64],
            name: [0; 64],
            enforce: 1.0,
            headtail: 0.0,
            lin_error: 0.0,
            rot_error: 0.0,
        }
    }
}

/// `bConstraintTarget`.
#[derive(Clone)]
pub struct BConstraintTarget {
    pub next: usize,
    pub prev: usize,
    /// `Object *tar`.
    pub tar: usize,
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
    pub matrix: [[f32; 4]; 4],
    pub space: i8,
    pub flag: i16,
    pub type_: i16,
    pub rot_order: i16,
    pub weight: f32,
}
impl Default for BConstraintTarget {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            tar: NIL,
            subtarget: [0; 64],
            matrix: [[0.0; 4]; 4],
            space: CONSTRAINT_SPACE_WORLD as i8,
            flag: 0,
            type_: CONSTRAINT_OBTYPE_OBJECT,
            rot_order: 0,
            weight: 0.0,
        }
    }
}

/// `bKinematicConstraint` (IK).
#[derive(Clone)]
pub struct BKinematicConstraint {
    /// `Object *tar`.
    pub tar: usize,
    pub iterations: i16,
    pub flag: i16,
    pub rootbone: i16,
    pub max_rootbone: i16,
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
    /// `Object *poletar`.
    pub poletar: usize,
    /// `char polesubtarget[64]`.
    pub polesubtarget: [u8; 64],
    pub poleangle: f32,
    pub weight: f32,
    pub orientweight: f32,
    pub grabtarget: [f32; 3],
    pub type_: ConstraintIkType,
    pub mode: i16,
    pub dist: f32,
}
impl Default for BKinematicConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            iterations: 0,
            flag: 0,
            rootbone: 0,
            max_rootbone: 0,
            subtarget: [0; 64],
            poletar: NIL,
            polesubtarget: [0; 64],
            poleangle: 0.0,
            weight: 0.0,
            orientweight: 0.0,
            grabtarget: [0.0; 3],
            type_: CONSTRAINT_IK_COPYPOSE,
            mode: LIMITDIST_INSIDE,
            dist: 0.0,
        }
    }
}

/// `bSplineIKConstraint`.
#[derive(Clone)]
pub struct BSplineIkConstraint {
    /// `Object *tar`.
    pub tar: usize,
    /// `float *points` → Vec.
    pub points: Vec<f32>,
    pub numpoints: i16,
    pub chainlen: i16,
    pub flag: i16,
    pub xz_scale_mode: i16,
    pub y_scale_mode: i16,
    pub bulge: f32,
    pub bulge_min: f32,
    pub bulge_max: f32,
    pub bulge_smooth: f32,
}
impl Default for BSplineIkConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            points: Vec::new(),
            numpoints: 0,
            chainlen: 0,
            flag: 0,
            xz_scale_mode: CONSTRAINT_SPLINEIK_XZS_NONE,
            y_scale_mode: CONSTRAINT_SPLINEIK_YS_NONE,
            bulge: 0.0,
            bulge_min: 0.0,
            bulge_max: 0.0,
            bulge_smooth: 0.0,
        }
    }
}

/// `bArmatureConstraint`.
#[derive(Clone, Default)]
pub struct BArmatureConstraint {
    pub flag: i32,
    /// `ListBaseT<bConstraintTarget>`.
    pub targets: ListBase,
}

/// `bTrackToConstraint`.
#[derive(Clone)]
pub struct BTrackToConstraint {
    /// `Object *tar`.
    pub tar: usize,
    pub reserved1: i32,
    pub reserved2: i32,
    pub flags: i32,
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
}
impl Default for BTrackToConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            reserved1: TRACK_X,
            reserved2: UP_Y,
            flags: 0,
            subtarget: [0; 64],
        }
    }
}

/// `bRotateLikeConstraint`.
#[derive(Clone)]
pub struct BRotateLikeConstraint {
    /// `Object *tar`.
    pub tar: usize,
    pub flag: i32,
    pub euler_order: i8,
    pub mix_mode: i8,
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
}
impl Default for BRotateLikeConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            flag: 0,
            euler_order: CONSTRAINT_EULER_AUTO as i8,
            mix_mode: ROTLIKE_MIX_REPLACE as i8,
            subtarget: [0; 64],
        }
    }
}

/// `bLocateLikeConstraint`.
#[derive(Clone)]
pub struct BLocateLikeConstraint {
    /// `Object *tar`.
    pub tar: usize,
    pub flag: i32,
    pub reserved1: i32,
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
}
impl Default for BLocateLikeConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            flag: 0,
            reserved1: 0,
            subtarget: [0; 64],
        }
    }
}

/// `bSizeLikeConstraint`.
#[derive(Clone)]
pub struct BSizeLikeConstraint {
    /// `Object *tar`.
    pub tar: usize,
    pub flag: i32,
    pub power: f32,
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
}
impl Default for BSizeLikeConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            flag: SIZELIKE_MULTIPLY,
            power: 0.0,
            subtarget: [0; 64],
        }
    }
}

/// `bSameVolumeConstraint`.
#[derive(Clone)]
pub struct BSameVolumeConstraint {
    pub free_axis: i8,
    pub mode: i8,
    pub volume: f32,
}
impl Default for BSameVolumeConstraint {
    fn default() -> Self {
        Self {
            free_axis: SAMEVOL_X as i8,
            mode: SAMEVOL_STRICT as i8,
            volume: 0.0,
        }
    }
}

/// `bTransLikeConstraint`.
#[derive(Clone)]
pub struct BTransLikeConstraint {
    /// `Object *tar`.
    pub tar: usize,
    pub flag: i32,
    pub mix_mode: i8,
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
}
impl Default for BTransLikeConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            flag: 0,
            mix_mode: TRANSLIKE_MIX_REPLACE as i8,
            subtarget: [0; 64],
        }
    }
}

/// `bMinMaxConstraint` (Floor).
#[derive(Clone)]
pub struct BMinMaxConstraint {
    /// `Object *tar`.
    pub tar: usize,
    pub minmaxflag: i32,
    pub offset: f32,
    pub flag: i32,
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
}
impl Default for BMinMaxConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            minmaxflag: 0,
            offset: 0.0,
            flag: 0,
            subtarget: [0; 64],
        }
    }
}

/// `bActionConstraint`.
#[derive(Clone)]
pub struct BActionConstraint {
    /// `Object *tar`.
    pub tar: usize,
    pub type_: i16,
    pub local: i16,
    pub start: i32,
    pub end: i32,
    pub min: f32,
    pub max: f32,
    pub flag: i32,
    pub mix_mode: i8,
    pub eval_time: f32,
    /// `bAction *act`.
    pub act: usize,
    pub action_slot_handle: i32,
    /// `char last_slot_identifier[258]`.
    pub last_slot_identifier: [u8; 258],
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
}
impl Default for BActionConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            type_: 0,
            local: 0,
            start: 0,
            end: 0,
            min: 0.0,
            max: 0.0,
            flag: 0,
            mix_mode: ACTCON_MIX_AFTER_FULL as i8,
            eval_time: 0.0,
            act: NIL,
            action_slot_handle: 0,
            last_slot_identifier: [0; 258],
            subtarget: [0; 64],
        }
    }
}

/// `bLockTrackConstraint`.
#[derive(Clone)]
pub struct BLockTrackConstraint {
    /// `Object *tar`.
    pub tar: usize,
    pub trackflag: i32,
    pub lockflag: i32,
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
}
impl Default for BLockTrackConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            trackflag: TRACK_X,
            lockflag: LOCK_X,
            subtarget: [0; 64],
        }
    }
}

/// `bDampTrackConstraint`.
#[derive(Clone)]
pub struct BDampTrackConstraint {
    /// `Object *tar`.
    pub tar: usize,
    pub trackflag: i32,
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
}
impl Default for BDampTrackConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            trackflag: TRACK_X,
            subtarget: [0; 64],
        }
    }
}

/// `bFollowPathConstraint`.
#[derive(Clone, Default)]
pub struct BFollowPathConstraint {
    /// `Object *tar`.
    pub tar: usize,
    pub offset: f32,
    pub offset_fac: f32,
    pub followflag: i32,
    pub trackflag: i16,
    pub upflag: i16,
}
impl BFollowPathConstraint {
    pub fn _init() -> Self {
        Self { tar: NIL, ..Self::default() }
    }
}

/// `bStretchToConstraint`.
#[derive(Clone)]
pub struct BStretchToConstraint {
    /// `Object *tar`.
    pub tar: usize,
    pub flag: i32,
    pub volmode: i32,
    pub plane: i32,
    pub orglength: f32,
    pub bulge: f32,
    pub bulge_min: f32,
    pub bulge_max: f32,
    pub bulge_smooth: f32,
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
}
impl Default for BStretchToConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            flag: 0,
            volmode: VOLUME_XZ,
            plane: PLANE_X,
            orglength: 0.0,
            bulge: 0.0,
            bulge_min: 0.0,
            bulge_max: 0.0,
            bulge_smooth: 0.0,
            subtarget: [0; 64],
        }
    }
}

/// `bRigidBodyJointConstraint`.
#[derive(Clone)]
pub struct BRigidBodyJointConstraint {
    /// `Object *tar`, `*child`.
    pub tar: usize,
    pub child: usize,
    pub type_: i32,
    pub piv_x: f32,
    pub piv_y: f32,
    pub piv_z: f32,
    pub ax_x: f32,
    pub ax_y: f32,
    pub ax_z: f32,
    pub min_limit: [f32; 6],
    pub max_limit: [f32; 6],
    pub extra_fz: f32,
    pub flag: i16,
}
impl Default for BRigidBodyJointConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            child: NIL,
            type_: 0,
            piv_x: 0.0,
            piv_y: 0.0,
            piv_z: 0.0,
            ax_x: 0.0,
            ax_y: 0.0,
            ax_z: 0.0,
            min_limit: [0.0; 6],
            max_limit: [0.0; 6],
            extra_fz: 0.0,
            flag: 0,
        }
    }
}

/// `bClampToConstraint`.
#[derive(Clone)]
pub struct BClampToConstraint {
    /// `Object *tar`.
    pub tar: usize,
    pub flag: i32,
    pub flag2: i32,
}
impl Default for BClampToConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            flag: CLAMPTO_AUTO,
            flag2: 0,
        }
    }
}

/// `bChildOfConstraint`.
#[derive(Clone)]
pub struct BChildOfConstraint {
    /// `Object *tar`.
    pub tar: usize,
    pub flag: i32,
    pub invmat: [[f32; 4]; 4],
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
}
impl Default for BChildOfConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            flag: CHILDOF_ALL,
            invmat: [[0.0; 4]; 4],
            subtarget: [0; 64],
        }
    }
}

/// `bTransformConstraint`.
#[derive(Clone)]
pub struct BTransformConstraint {
    /// `Object *tar`.
    pub tar: usize,
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
    pub from: i16,
    pub to: i16,
    /// `char map[3]` — axis mapping chars.
    pub map: [u8; 3],
    pub expo: i8,
    pub from_rotation_mode: i8,
    pub to_euler_order: i8,
    pub mix_mode_loc: i8,
    pub mix_mode_rot: i8,
    pub mix_mode_scale: i8,
    pub from_min: [f32; 3],
    pub from_max: [f32; 3],
    pub to_min: [f32; 3],
    pub to_max: [f32; 3],
    pub from_min_rot: [f32; 3],
    pub from_max_rot: [f32; 3],
    pub to_min_rot: [f32; 3],
    pub to_max_rot: [f32; 3],
    pub from_min_scale: [f32; 3],
    pub from_max_scale: [f32; 3],
    pub to_min_scale: [f32; 3],
    pub to_max_scale: [f32; 3],
}
impl Default for BTransformConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            subtarget: [0; 64],
            from: TRANS_LOCATION,
            to: TRANS_LOCATION,
            map: [0; 3],
            expo: 0,
            from_rotation_mode: 0,
            to_euler_order: CONSTRAINT_EULER_AUTO as i8,
            mix_mode_loc: TRANS_MIXLOC_ADD as i8,
            mix_mode_rot: TRANS_MIXROT_ADD as i8,
            mix_mode_scale: TRANS_MIXSCALE_REPLACE as i8,
            from_min: [0.0; 3],
            from_max: [0.0; 3],
            to_min: [0.0; 3],
            to_max: [0.0; 3],
            from_min_rot: [0.0; 3],
            from_max_rot: [0.0; 3],
            to_min_rot: [0.0; 3],
            to_max_rot: [0.0; 3],
            from_min_scale: [0.0; 3],
            from_max_scale: [0.0; 3],
            to_min_scale: [0.0; 3],
            to_max_scale: [0.0; 3],
        }
    }
}

/// `bPivotConstraint`.
#[derive(Clone)]
pub struct BPivotConstraint {
    /// `Object *tar`.
    pub tar: usize,
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
    pub offset: [f32; 3],
    pub rot_axis: i16,
    pub flag: i16,
}
impl Default for BPivotConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            subtarget: [0; 64],
            offset: [0.0; 3],
            rot_axis: PIVOTCON_AXIS_X_NEG,
            flag: 0,
        }
    }
}

/// `bLocLimitConstraint`.
#[derive(Clone, Default)]
pub struct BLocLimitConstraint {
    pub xmin: f32,
    pub xmax: f32,
    pub ymin: f32,
    pub ymax: f32,
    pub zmin: f32,
    pub zmax: f32,
    pub flag: i16,
    pub flag2: i16,
}

/// `bRotLimitConstraint`.
#[derive(Clone)]
pub struct BRotLimitConstraint {
    pub xmin: f32,
    pub xmax: f32,
    pub ymin: f32,
    pub ymax: f32,
    pub zmin: f32,
    pub zmax: f32,
    pub flag: i16,
    pub flag2: i16,
    pub euler_order: i8,
}
impl Default for BRotLimitConstraint {
    fn default() -> Self {
        Self {
            xmin: 0.0,
            xmax: 0.0,
            ymin: 0.0,
            ymax: 0.0,
            zmin: 0.0,
            zmax: 0.0,
            flag: 0,
            flag2: 0,
            euler_order: CONSTRAINT_EULER_AUTO as i8,
        }
    }
}

/// `bSizeLimitConstraint`.
#[derive(Clone, Default)]
pub struct BSizeLimitConstraint {
    pub xmin: f32,
    pub xmax: f32,
    pub ymin: f32,
    pub ymax: f32,
    pub zmin: f32,
    pub zmax: f32,
    pub flag: i16,
    pub flag2: i16,
}

/// `bDistLimitConstraint`.
#[derive(Clone)]
pub struct BDistLimitConstraint {
    /// `Object *tar`.
    pub tar: usize,
    /// `char subtarget[64]`.
    pub subtarget: [u8; 64],
    pub dist: f32,
    pub soft: f32,
    pub flag: i16,
    pub mode: i16,
}
impl Default for BDistLimitConstraint {
    fn default() -> Self {
        Self {
            tar: NIL,
            subtarget: [0; 64],
            dist: 0.0,
            soft: 0.0,
            flag: 0,
            mode: LIMITDIST_INSIDE,
        }
    }
}

/// `bShrinkwrapConstraint`.
#[derive(Clone)]
pub struct BShrinkwrapConstraint {
    /// `Object *target`.
    pub target: usize,
    pub dist: f32,
    pub shrink_type: i16,
    pub proj_axis: i8,
    pub proj_axis_space: i8,
    pub proj_limit: f32,
    pub shrink_mode: i8,
    pub flag: i8,
    pub track_axis: i8,
}
impl Default for BShrinkwrapConstraint {
    fn default() -> Self {
        Self {
            target: NIL,
            dist: 0.0,
            shrink_type: 0,
            proj_axis: 0,
            proj_axis_space: 0,
            proj_limit: 0.0,
            shrink_mode: 0,
            flag: 0,
            track_axis: 0,
        }
    }
}

/// `bFollowTrackConstraint`.
#[derive(Clone)]
pub struct BFollowTrackConstraint {
    /// `MovieClip *clip`.
    pub clip: usize,
    /// `char track[64]`.
    pub track: [u8; 64],
    pub flag: i32,
    pub frame_method: i32,
    /// `char object[64]`.
    pub object: [u8; 64],
    /// `Object *camera`, `*depth_ob`.
    pub camera: usize,
    pub depth_ob: usize,
}
impl Default for BFollowTrackConstraint {
    fn default() -> Self {
        Self {
            clip: NIL,
            track: [0; 64],
            flag: 0,
            frame_method: FOLLOWTRACK_FRAME_STRETCH,
            object: [0; 64],
            camera: NIL,
            depth_ob: NIL,
        }
    }
}

/// `bCameraSolverConstraint`.
#[derive(Clone)]
pub struct BCameraSolverConstraint {
    /// `MovieClip *clip`.
    pub clip: usize,
    pub flag: i32,
}
impl Default for BCameraSolverConstraint {
    fn default() -> Self {
        Self {
            clip: NIL,
            flag: 0,
        }
    }
}

/// `bObjectSolverConstraint`.
#[derive(Clone)]
pub struct BObjectSolverConstraint {
    /// `MovieClip *clip`.
    pub clip: usize,
    pub flag: i32,
    /// `char object[64]`.
    pub object: [u8; 64],
    pub invmat: [[f32; 4]; 4],
    /// `Object *camera`.
    pub camera: usize,
}
impl Default for BObjectSolverConstraint {
    fn default() -> Self {
        Self {
            clip: NIL,
            flag: 0,
            object: [0; 64],
            invmat: [[0.0; 4]; 4],
            camera: NIL,
        }
    }
}

/// `bTransformCacheConstraint`.
#[derive(Clone)]
pub struct BTransformCacheConstraint {
    /// `CacheFile *cache_file`.
    pub cache_file: usize,
    /// `char object_path[1024]`.
    pub object_path: [u8; 1024],
    /// `CacheReader *reader`.
    pub reader: usize,
    /// `char reader_object_path[1024]`.
    pub reader_object_path: [u8; 1024],
}
impl Default for BTransformCacheConstraint {
    fn default() -> Self {
        Self {
            cache_file: NIL,
            object_path: [0; 1024],
            reader: NIL,
            reader_object_path: [0; 1024],
        }
    }
}

/// `bGeometryAttributeConstraint`.
#[derive(Clone)]
pub struct BGeometryAttributeConstraint {
    /// `Object *target`.
    pub target: usize,
    /// `char *attribute_name`.
    pub attribute_name: Option<String>,
    pub sample_index: i32,
    pub apply_target_transform: u8,
    pub mix_mode: u8,
    pub domain: u8,
    pub data_type: u8,
    pub flags: u8,
}
impl Default for BGeometryAttributeConstraint {
    fn default() -> Self {
        Self {
            target: NIL,
            attribute_name: None,
            sample_index: 0,
            apply_target_transform: 0,
            mix_mode: CON_ATTRIBUTE_MIX_REPLACE,
            domain: CON_ATTRIBUTE_DOMAIN_POINT,
            data_type: CON_ATTRIBUTE_VECTOR,
            flags: 0,
        }
    }
}
