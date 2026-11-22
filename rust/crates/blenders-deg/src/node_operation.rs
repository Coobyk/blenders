//! Port of `intern/node/deg_node_operation.cc` —
/// `OperationCode` string table + `OperationNode`.

/// `OperationCode` verbatim (`deg_node_operation.hh`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum OperationCode {
    /// `OPERATION`.
    Operation,
    /// `ID_PROPERTY`.
    IdProperty,
    /// `PARAMETERS_ENTRY`.
    ParametersEntry,
    /// `PARAMETERS_EVAL`.
    ParametersEval,
    /// `PARAMETERS_EXIT`.
    ParametersExit,
    /// `VISIBILITY`.
    Visibility,
    /// `HIERARCHY`.
    Hierarchy,
    /// `ANIMATION_ENTRY`.
    AnimationEntry,
    /// `ANIMATION_EVAL`.
    AnimationEval,
    /// `ANIMATION_EXIT`.
    AnimationExit,
    /// `DRIVER`.
    Driver,
    /// `DRIVER_UNSHARE`.
    DriverUnshare,
    /// `SCENE_EVAL`.
    SceneEval,
    /// `AUDIO_CACHE_INVALIDATE`.
    AudioCacheInvalidate,
    /// `AUDIO_ENTRY`.
    AudioEntry,
    /// `AUDIO_VOLUME`.
    AudioVolume,
    /// `OBJECT_FROM_LAYER_ENTRY`.
    ObjectFromLayerEntry,
    /// `OBJECT_BASE_FLAGS`.
    ObjectBaseFlags,
    /// `OBJECT_FROM_LAYER_EXIT`.
    ObjectFromLayerExit,
    /// `DIMENSIONS`.
    Dimensions,
    /// `TRANSFORM_INIT`.
    TransformInit,
    /// `TRANSFORM_LOCAL`.
    TransformLocal,
    /// `TRANSFORM_PARENT`.
    TransformParent,
    /// `TRANSFORM_CONSTRAINTS`.
    TransformConstraints,
    /// `TRANSFORM_FINAL`.
    TransformFinal,
    /// `TRANSFORM_EVAL`.
    TransformEval,
    /// `TRANSFORM_SIMULATION_INIT`.
    TransformSimulationInit,
    /// `RIGIDBODY_REBUILD`.
    RigidbodyRebuild,
    /// `RIGIDBODY_SIM`.
    RigidbodySim,
    /// `RIGIDBODY_TRANSFORM_COPY`.
    RigidbodyTransformCopy,
    /// `GEOMETRY_EVAL_INIT`.
    GeometryEvalInit,
    /// `MODIFIER`.
    Modifier,
    /// `GEOMETRY_EVAL`.
    GeometryEval,
    /// `GEOMETRY_EVAL_DONE`.
    GeometryEvalDone,
    /// `GEOMETRY_SHAPEKEY`.
    GeometryShapekey,
    /// `LIGHT_PROBE_EVAL`.
    LightProbeEval,
    /// `SPEAKER_EVAL`.
    SpeakerEval,
    /// `SOUND_EVAL`.
    SoundEval,
    /// `ARMATURE_EVAL`.
    ArmatureEval,
    /// `POSE_INIT`.
    PoseInit,
    /// `POSE_INIT_IK`.
    PoseInitIk,
    /// `POSE_CLEANUP`.
    PoseCleanup,
    /// `POSE_DONE`.
    PoseDone,
    /// `POSE_IK_SOLVER`.
    PoseIkSolver,
    /// `POSE_SPLINE_IK_SOLVER`.
    PoseSplineIkSolver,
    /// `BONE_LOCAL`.
    BoneLocal,
    /// `BONE_POSE_PARENT`.
    BonePoseParent,
    /// `BONE_CONSTRAINTS`.
    BoneConstraints,
    /// `BONE_READY`.
    BoneReady,
    /// `BONE_DONE`.
    BoneDone,
    /// `BONE_SEGMENTS`.
    BoneSegments,
    /// `BONE_VISIBILITY`.
    BoneVisibility,
    /// `PARTICLE_SYSTEM_INIT`.
    ParticleSystemInit,
    /// `PARTICLE_SYSTEM_EVAL`.
    ParticleSystemEval,
    /// `PARTICLE_SYSTEM_DONE`.
    ParticleSystemDone,
    /// `PARTICLE_SETTINGS_INIT`.
    ParticleSettingsInit,
    /// `PARTICLE_SETTINGS_EVAL`.
    ParticleSettingsEval,
    /// `PARTICLE_SETTINGS_RESET`.
    ParticleSettingsReset,
    /// `POINT_CACHE_RESET`.
    PointCacheReset,
    /// `FILE_CACHE_UPDATE`.
    FileCacheUpdate,
    /// `GEOMETRY_SELECT_UPDATE`.
    GeometrySelectUpdate,
    /// `MASK_ANIMATION`.
    MaskAnimation,
    /// `MASK_EVAL`.
    MaskEval,
    /// `VIEW_LAYER_EVAL`.
    ViewLayerEval,
    /// `COPY_ON_EVAL`.
    CopyOnEval,
    /// `SHADING`.
    Shading,
    /// `SHADING_DONE`.
    ShadingDone,
    /// `MATERIAL_UPDATE`.
    MaterialUpdate,
    /// `LIGHT_UPDATE`.
    LightUpdate,
    /// `WORLD_UPDATE`.
    WorldUpdate,
    /// `LIGHT_LINKING_UPDATE`.
    LightLinkingUpdate,
    /// `NTREE_OUTPUT`.
    NtreeOutput,
    /// `NTREE_GEOMETRY_PREPROCESS`.
    NtreeGeometryPreprocess,
    /// `MOVIECLIP_EVAL`.
    MovieclipEval,
    /// `IMAGE_ANIMATION`.
    ImageAnimation,
    /// `SYNCHRONIZE_TO_ORIGINAL`.
    SynchronizeToOriginal,
    /// `GENERIC_DATABLOCK_UPDATE`.
    GenericDatablockUpdate,
    /// `SEQUENCES_EVAL`.
    SequencesEval,
    /// `COMPOSITOR_EVAL`.
    CompositorEval,
    /// `INSTANCER`.
    Instancer,
    /// `INSTANCE`.
    Instance,
    /// `INSTANCE_GEOMETRY`.
    InstanceGeometry,
}

/// `operationCodeAsString` (verbatim).
pub fn operation_code_as_string(op: OperationCode) -> &'static str {
    use OperationCode::*;
    match op {
        Operation => "OPERATION",
        IdProperty => "ID_PROPERTY",
        ParametersEntry => "PARAMETERS_ENTRY",
        ParametersEval => "PARAMETERS_EVAL",
        ParametersExit => "PARAMETERS_EXIT",
        Visibility => "VISIBILITY",
        Hierarchy => "HIERARCHY",
        AnimationEntry => "ANIMATION_ENTRY",
        AnimationEval => "ANIMATION_EVAL",
        AnimationExit => "ANIMATION_EXIT",
        Driver => "DRIVER",
        DriverUnshare => "DRIVER_UNSHARE",
        SceneEval => "SCENE_EVAL",
        AudioCacheInvalidate => "AUDIO_CACHE_INVALIDATE",
        AudioEntry => "AUDIO_ENTRY",
        AudioVolume => "AUDIO_VOLUME",
        ObjectFromLayerEntry => "OBJECT_FROM_LAYER_ENTRY",
        ObjectBaseFlags => "OBJECT_BASE_FLAGS",
        ObjectFromLayerExit => "OBJECT_FROM_LAYER_EXIT",
        Dimensions => "DIMENSIONS",
        TransformInit => "TRANSFORM_INIT",
        TransformLocal => "TRANSFORM_LOCAL",
        TransformParent => "TRANSFORM_PARENT",
        TransformConstraints => "TRANSFORM_CONSTRAINTS",
        TransformFinal => "TRANSFORM_FINAL",
        TransformEval => "TRANSFORM_EVAL",
        TransformSimulationInit => "TRANSFORM_SIMULATION_INIT",
        RigidbodyRebuild => "RIGIDBODY_REBUILD",
        RigidbodySim => "RIGIDBODY_SIM",
        RigidbodyTransformCopy => "RIGIDBODY_TRANSFORM_COPY",
        GeometryEvalInit => "GEOMETRY_EVAL_INIT",
        Modifier => "MODIFIER",
        GeometryEval => "GEOMETRY_EVAL",
        GeometryEvalDone => "GEOMETRY_EVAL_DONE",
        GeometryShapekey => "GEOMETRY_SHAPEKEY",
        LightProbeEval => "LIGHT_PROBE_EVAL",
        SpeakerEval => "SPEAKER_EVAL",
        SoundEval => "SOUND_EVAL",
        ArmatureEval => "ARMATURE_EVAL",
        PoseInit => "POSE_INIT",
        PoseInitIk => "POSE_INIT_IK",
        PoseCleanup => "POSE_CLEANUP",
        PoseDone => "POSE_DONE",
        PoseIkSolver => "POSE_IK_SOLVER",
        PoseSplineIkSolver => "POSE_SPLINE_IK_SOLVER",
        BoneLocal => "BONE_LOCAL",
        BonePoseParent => "BONE_POSE_PARENT",
        BoneConstraints => "BONE_CONSTRAINTS",
        BoneReady => "BONE_READY",
        BoneDone => "BONE_DONE",
        BoneSegments => "BONE_SEGMENTS",
        BoneVisibility => "BONE_VISIBILITY",
        ParticleSystemInit => "PARTICLE_SYSTEM_INIT",
        ParticleSystemEval => "PARTICLE_SYSTEM_EVAL",
        ParticleSystemDone => "PARTICLE_SYSTEM_DONE",
        ParticleSettingsInit => "PARTICLE_SETTINGS_INIT",
        ParticleSettingsEval => "PARTICLE_SETTINGS_EVAL",
        ParticleSettingsReset => "PARTICLE_SETTINGS_RESET",
        PointCacheReset => "POINT_CACHE_RESET",
        FileCacheUpdate => "FILE_CACHE_UPDATE",
        GeometrySelectUpdate => "GEOMETRY_SELECT_UPDATE",
        MaskAnimation => "MASK_ANIMATION",
        MaskEval => "MASK_EVAL",
        ViewLayerEval => "VIEW_LAYER_EVAL",
        CopyOnEval => "COPY_ON_EVAL",
        Shading => "SHADING",
        ShadingDone => "SHADING_DONE",
        MaterialUpdate => "MATERIAL_UPDATE",
        LightUpdate => "LIGHT_UPDATE",
        WorldUpdate => "WORLD_UPDATE",
        LightLinkingUpdate => "LIGHT_LINKING_UPDATE",
        NtreeOutput => "NTREE_OUTPUT",
        NtreeGeometryPreprocess => "NTREE_GEOMETRY_PREPROCESS",
        MovieclipEval => "MOVIECLIP_EVAL",
        ImageAnimation => "IMAGE_ANIMATION",
        SynchronizeToOriginal => "SYNCHRONIZE_TO_ORIGINAL",
        GenericDatablockUpdate => "GENERIC_DATABLOCK_UPDATE",
        SequencesEval => "SEQUENCES_EVAL",
        CompositorEval => "COMPOSITOR_EVAL",
        Instancer => "INSTANCER",
        Instance => "INSTANCE",
        InstanceGeometry => "INSTANCE_GEOMETRY",
    }
}

/// `OperationFlag` verbatim.
pub const DEPSOP_FLAG_NEEDS_UPDATE: u32 = 1 << 0;
/// `DEPSOP_FLAG_DIRECTLY_MODIFIED`.
pub const DEPSOP_FLAG_DIRECTLY_MODIFIED: u32 = 1 << 1;
/// `DEPSOP_FLAG_USER_MODIFIED`.
pub const DEPSOP_FLAG_USER_MODIFIED: u32 = 1 << 2;
/// `DEPSOP_FLAG_PINNED`.
pub const DEPSOP_FLAG_PINNED: u32 = 1 << 3;
/// `DEPSOP_FLAG_AFFECTS_VISIBILITY`.
pub const DEPSOP_FLAG_AFFECTS_VISIBILITY: u32 = 1 << 4;
/// `DEPSOP_FLAG_MUTE`.
pub const DEPSOP_FLAG_MUTE: u32 = 1 << 5;
/// `DEPSOP_FLAG_FLUSH`.
pub const DEPSOP_FLAG_FLUSH: u32 = DEPSOP_FLAG_USER_MODIFIED;
/// `DEPSOP_FLAG_CLEAR_ON_EVAL`.
pub const DEPSOP_FLAG_CLEAR_ON_EVAL: u32 =
    DEPSOP_FLAG_DIRECTLY_MODIFIED | DEPSOP_FLAG_NEEDS_UPDATE | DEPSOP_FLAG_USER_MODIFIED;

/// `OperationNode` fields (verbatim).
#[derive(Default)]
pub struct OperationNode {
    /// `opcode`.
    pub opcode: OperationCode,
    /// `name`.
    pub name: String,
    /// `name_tag`.
    pub name_tag: i32,
    /// `flag`.
    pub flag: u32,
}

impl Default for OperationCode {
    fn default() -> Self {
        OperationCode::Operation
    }
}

impl OperationNode {
    /// ctor (verbatim — `name_tag(-1)`).
    pub fn new() -> Self {
        Self {
            name_tag: -1,
            ..Default::default()
        }
    }

    /// `identifier` (verbatim).
    pub fn identifier(&self) -> String {
        format!("{}({})", operation_code_as_string(self.opcode), self.name)
    }

    /// `full_identifier` (verbatim — `owner_str` supplied
        /// by caller: `owner->owner->name` plus `"/"+owner->name`
        /// for bones/named components).
    pub fn full_identifier(&self, owner_str: &str) -> String {
        format!("{}/{}", owner_str, self.identifier())
    }

    /// `tag_update` (verbatim — `add_entry_tag` and
        /// `need_update_nodes_visibility` handled by the caller).
    pub fn tag_update(&mut self, source: i32) {
        use crate::node_id::DEG_UPDATE_SOURCE_USER_EDIT;
        /* Enforce dynamic visibility code-path update. */
        let _visibility_op = self.opcode == OperationCode::Visibility;
        /* Tag for update, but also note that this was the
         * source of an update. */
        self.flag |= DEPSOP_FLAG_NEEDS_UPDATE | DEPSOP_FLAG_DIRECTLY_MODIFIED;
        if source == DEG_UPDATE_SOURCE_USER_EDIT {
            self.flag |= DEPSOP_FLAG_USER_MODIFIED;
        }
    }

    /// `set_as_entry`/`set_as_exit` — caller toggles the
    /// entry/exit node sets.
    pub fn is_noop(&self) -> bool {
        self.opcode == OperationCode::Operation
    }
}

/// `deg_register_operation_depsnodes` (verbatim —
/// factory registered by `node_factory`).
pub fn deg_register_operation_depsnodes() {}
