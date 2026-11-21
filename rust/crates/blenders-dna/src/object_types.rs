//! Port of `DNA_object_types.h` — `Object`, `ObHook`, `BoundBox`,
//! `ObjectLineArt`, `LightLinking`, `bDeformGroup` + object enums.
use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;

pub const MAX_VGROUP_NAME: usize = 64;
pub const MAX_NAME: usize = 64;
/// `_DNA_DEFAULT_UNIT_QT` / `_DNA_DEFAULT_UNIT_M4` from DNA_defaults.h.
pub const DNA_DEFAULT_UNIT_QT: [f32; 4] = [1.0, 0.0, 0.0, 0.0];
pub const DNA_DEFAULT_UNIT_M4: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// `ebDeformGroup_Flag`.
pub const DG_LOCK_WEIGHT: i8 = 1;

/// `eObject_Flag`.
pub const OB_SELECT: i16 = 1 << 0;
pub const BA_WAS_SEL: i16 = 1 << 1;
pub const BA_SNAP_FIX_DEPS_FIASCO: i16 = 1 << 2;
pub const BA_TEMP_TAG: i16 = 1 << 5;
pub const BA_TRANSFORM_LOCKED_IN_PLACE: i16 = 1 << 7;
pub const BA_TRANSFORM_CHILD: i16 = 1 << 8;
pub const BA_TRANSFORM_PARENT: i16 = 1 << 13;
pub const OB_FROMDUPLI: i16 = 1 << 9;
pub const OB_DONE: i16 = 1 << 10;
pub const OB_FLAG_USE_SIMULATION_CACHE: i16 = 1 << 11;
pub const OB_FLAG_ACTIVE_CLIPBOARD: i16 = 1 << 12;

/// `ObjectType`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
#[repr(i16)]
pub enum ObjectType {
    #[default]
    ObEmpty = 0,
    ObMesh = 1,
    ObCurvesLegacy = 2,
    ObSurf = 3,
    ObFont = 4,
    ObMball = 5,
    ObLamp = 10,
    ObCamera = 11,
    ObSpeaker = 12,
    ObLightProbe = 13,
    ObLattice = 22,
    ObArmature = 25,
    ObGpencilLegacy = 26,
    ObCurves = 27,
    ObPointCloud = 28,
    ObVolume = 29,
    ObGreasePencil = 30,
}
pub const OB_TYPE_MAX: i16 = 31;

/// `eObject_Partype`.
pub const PAROBJECT: i16 = 0;
pub const PARSKEL: i16 = 4;
pub const PARVERT1: i16 = 5;
pub const PARVERT3: i16 = 6;
pub const PARBONE: i16 = 7;
pub const PARTYPE: i16 = (1 << 4) - 1;

/// `eObject_TransFlag`.
pub const OB_TRANSFORM_ADJUST_ROOT_PARENT_FOR_VIEW_LOCK: i16 = 1 << 0;
pub const OB_NEG_SCALE: i16 = 1 << 2;
pub const OB_DUPLIVERTS: i16 = 1 << 4;
pub const OB_DUPLIROT: i16 = 1 << 5;
pub const OB_DUPLICOLLECTION: i16 = 1 << 8;
pub const OB_DUPLIFACES: i16 = 1 << 9;
pub const OB_DUPLIFACES_SCALE: i16 = 1 << 10;
pub const OB_DUPLIPARTS: i16 = 1 << 11;
pub const OB_NO_CONSTRAINTS: i16 = 1 << 13;
pub const OB_PARENT_USE_FINAL_INDICES: i16 = 1 << 14;
pub const OB_DUPLI: i16 = OB_DUPLIVERTS | OB_DUPLICOLLECTION | OB_DUPLIFACES | OB_DUPLIPARTS;

/// `eObject_Axis`.
pub const OB_POSX: i16 = 0;
pub const OB_POSY: i16 = 1;
pub const OB_POSZ: i16 = 2;
pub const OB_NEGX: i16 = 3;
pub const OB_NEGY: i16 = 4;
pub const OB_NEGZ: i16 = 5;

/// `eObject_DrawExtraFlag`.
pub const OB_DRAWBOUNDOX: i16 = 1 << 0;
pub const OB_AXIS: i16 = 1 << 1;
pub const OB_TEXSPACE: i16 = 1 << 2;
pub const OB_DRAWNAME: i16 = 1 << 3;
pub const OB_DRAWWIRE: i16 = 1 << 5;
pub const OB_DRAW_IN_FRONT: i16 = 1 << 6;
pub const OB_DRAWTRANSP: i16 = 1 << 7;
pub const OB_DRAW_ALL_EDGES: i16 = 1 << 8;
pub const OB_DRAW_NO_SHADOW_CAST: i16 = 1 << 9;
pub const OB_USE_GPENCIL_LIGHTS: i16 = 1 << 10;

/// `eObject_EmptyDrawType`.
pub const OB_ARROWS: i8 = 1;
pub const OB_PLAINAXES: i8 = 2;
pub const OB_CIRCLE: i8 = 3;
pub const OB_SINGLE_ARROW: i8 = 4;
pub const OB_CUBE: i8 = 5;
pub const OB_EMPTY_SPHERE: i8 = 6;
pub const OB_EMPTY_CONE: i8 = 7;
pub const OB_EMPTY_IMAGE: i8 = 8;

/// `eGpencil_AddType`.
pub const GP_EMPTY: i32 = 0;
pub const GP_STROKE: i32 = 1;
pub const GP_MONKEY: i32 = 2;
pub const GREASE_PENCIL_LINEART_SCENE: i32 = 3;
pub const GREASE_PENCIL_LINEART_OBJECT: i32 = 4;
pub const GREASE_PENCIL_LINEART_COLLECTION: i32 = 5;

/// `eObject_BoundType`.
pub const OB_BOUND_BOX: i8 = 0;
pub const OB_BOUND_SPHERE: i8 = 1;
pub const OB_BOUND_CYLINDER: i8 = 2;
pub const OB_BOUND_CONE: i8 = 3;
pub const OB_BOUND_CAPSULE: i8 = 7;

/// `eObject_VisibilityFlag`.
pub const OB_HIDE_VIEWPORT: i32 = 1 << 0;
pub const OB_HIDE_SELECT: i32 = 1 << 1;
pub const OB_HIDE_RENDER: i32 = 1 << 2;
pub const OB_HIDE_CAMERA: i32 = 1 << 3;
pub const OB_HIDE_DIFFUSE: i32 = 1 << 4;
pub const OB_HIDE_GLOSSY: i32 = 1 << 5;
pub const OB_HIDE_TRANSMISSION: i32 = 1 << 6;
pub const OB_HIDE_VOLUME_SCATTER: i32 = 1 << 7;
pub const OB_HIDE_SHADOW: i32 = 1 << 8;
pub const OB_HOLDOUT: i32 = 1 << 9;
pub const OB_SHADOW_CATCHER: i32 = 1 << 10;
pub const OB_HIDE_PROBE_VOLUME: i32 = 1 << 11;
pub const OB_HIDE_PROBE_CUBEMAP: i32 = 1 << 12;
pub const OB_HIDE_PROBE_PLANAR: i32 = 1 << 13;
pub const OB_HIDE_SURFACE_PICK: i32 = 1 << 14;
pub const OB_HIDE_RAYCAST: i32 = 1 << 15;

/// `eObject_ShapeFlag`.
pub const OB_SHAPE_LOCK: i8 = 1 << 0;
pub const OB_SHAPE_EDIT_MODE: i8 = 1 << 2;

/// `eObject_NlaFlag`.
pub const OB_ADS_COLLAPSED: i16 = 1 << 10;

/// `eObject_ProtectFlag`.
pub const OB_LOCK_LOCX: i16 = 1 << 0;
pub const OB_LOCK_LOCY: i16 = 1 << 1;
pub const OB_LOCK_LOCZ: i16 = 1 << 2;
pub const OB_LOCK_LOC: i16 = OB_LOCK_LOCX | OB_LOCK_LOCY | OB_LOCK_LOCZ;
pub const OB_LOCK_ROTX: i16 = 1 << 3;
pub const OB_LOCK_ROTY: i16 = 1 << 4;
pub const OB_LOCK_ROTZ: i16 = 1 << 5;
pub const OB_LOCK_ROT: i16 = OB_LOCK_ROTX | OB_LOCK_ROTY | OB_LOCK_ROTZ;
pub const OB_LOCK_SCALEX: i16 = 1 << 6;
pub const OB_LOCK_SCALEY: i16 = 1 << 7;
pub const OB_LOCK_SCALEZ: i16 = 1 << 8;
pub const OB_LOCK_SCALE: i16 = OB_LOCK_SCALEX | OB_LOCK_SCALEY | OB_LOCK_SCALEZ;
pub const OB_LOCK_ROTW: i16 = 1 << 9;
pub const OB_LOCK_ROT4D: i16 = 1 << 10;

/// `eObject_DuplicatorVisibilityFlag`.
pub const OB_DUPLI_FLAG_VIEWPORT: i8 = 1 << 0;
pub const OB_DUPLI_FLAG_RENDER: i8 = 1 << 1;

/// `eObject_EmptyImageDepth`.
pub const OB_EMPTY_IMAGE_DEPTH_DEFAULT: i8 = 0;
pub const OB_EMPTY_IMAGE_DEPTH_FRONT: i8 = 1;
pub const OB_EMPTY_IMAGE_DEPTH_BACK: i8 = 2;

/// `eObject_EmptyImageVisibilityFlag`.
pub const OB_EMPTY_IMAGE_HIDE_PERSPECTIVE: i8 = 1 << 0;
pub const OB_EMPTY_IMAGE_HIDE_ORTHOGRAPHIC: i8 = 1 << 1;
pub const OB_EMPTY_IMAGE_HIDE_BACK: i8 = 1 << 2;
pub const OB_EMPTY_IMAGE_HIDE_FRONT: i8 = 1 << 3;
pub const OB_EMPTY_IMAGE_HIDE_NON_AXIS_ALIGNED: i8 = 1 << 4;

/// `eObject_EmptyImageFlag`.
pub const OB_EMPTY_IMAGE_USE_ALPHA_BLEND: i8 = 1 << 0;

/// `bDeformGroup`.
#[derive(Clone)]
pub struct BDeformGroup {
    pub next: usize,
    pub prev: usize,
    /// `char name[MAX_VGROUP_NAME]`.
    pub name: [u8; MAX_VGROUP_NAME],
    pub flag: i8,
}
impl Default for BDeformGroup {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; MAX_VGROUP_NAME],
            flag: 0,
        }
    }
}

/// `bFaceMap` (DNA_DEPRECATED_ALLOW).
#[derive(Clone)]
pub struct BFaceMap {
    pub next: usize,
    pub prev: usize,
    /// `char name[MAX_VGROUP_NAME]`.
    pub name: [u8; MAX_VGROUP_NAME],
    pub flag: i8,
}
impl Default for BFaceMap {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; MAX_VGROUP_NAME],
            flag: 0,
        }
    }
}

/// `BoundBox`.
#[derive(Clone, Copy)]
pub struct BoundBox {
    pub vec: [[f32; 3]; 8],
}
impl Default for BoundBox {
    fn default() -> Self {
        Self { vec: [[0.0; 3]; 8] }
    }
}

/// `eObjectLineArt_Usage`.
pub const OBJECT_LRT_INHERIT: i16 = 0;
pub const OBJECT_LRT_INCLUDE: i16 = 1 << 0;
pub const OBJECT_LRT_OCCLUSION_ONLY: i16 = 1 << 1;
pub const OBJECT_LRT_EXCLUDE: i16 = 1 << 2;
pub const OBJECT_LRT_INTERSECTION_ONLY: i16 = 1 << 3;
pub const OBJECT_LRT_NO_INTERSECTION: i16 = 1 << 4;
pub const OBJECT_LRT_FORCE_INTERSECTION: i16 = 1 << 5;

/// `eObjectLineArt_Flags`.
pub const OBJECT_LRT_OWN_CREASE: i16 = 1 << 0;
pub const OBJECT_LRT_OWN_INTERSECTION_PRIORITY: i16 = 1 << 1;

/// `DEG2RAD(140)` for crease_threshold default.
pub const LINEART_CREASE_THRESHOLD_DEG140: f32 = 140.0 * (std::f32::consts::PI / 180.0);

/// `ObjectLineArt`.
#[derive(Clone, Default)]
pub struct ObjectLineArt {
    pub usage: i16,
    pub flags: i16,
    pub crease_threshold: f32,
    pub intersection_priority: u8,
}

/// `LightLinkingRuntime`.
#[derive(Clone, Copy, Default)]
pub struct LightLinkingRuntime {
    pub light_set_membership: u64,
    pub shadow_set_membership: u64,
    pub receiver_light_set: u8,
    pub blocker_shadow_set: u8,
}

/// `LightLinking`.
#[derive(Clone, Default)]
pub struct LightLinking {
    /// `Collection *receiver_collection`.
    pub receiver_collection: usize,
    /// `Collection *blocker_collection`.
    pub blocker_collection: usize,
    pub runtime: LightLinkingRuntime,
}

/* Foreign enum placeholders (from other DNA headers):
   `eObjectMode` (DNA_object_enums.h), `eDrawType` (DNA_object_enums.h),
   `ROT_MODE_EUL` (DNA_action_types.h), `bAnimVizSettings` (DNA_action_types.h),
   `ObjectModifierFlag` (DNA_modifier_enums.h). */
pub type ObjectMode = i16;
pub type DrawType = i8;
pub type AnimVizSettings = u64;
pub type ObjectModifierFlag = i8;
pub const OB_MODE_OBJECT: i16 = 0;
pub const OB_TEXTURE: i8 = 5;
pub const ROT_MODE_EUL: i16 = 0;

/// `Object` — the object data-block.
#[derive(Clone)]
pub struct Object {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    pub type_: ObjectType,
    pub partype: i16,
    pub par1: i32,
    pub par2: i32,
    pub par3: i32,
    /// `char parsubstr[MAX_NAME]`.
    pub parsubstr: [u8; MAX_NAME],
    /// `Object *parent`.
    pub parent: usize,
    /// `Object *track`.
    pub track: usize,
    pub parent_bone_head_tail_factor: f32,
    /// DNA_DEPRECATED `Object *proxy`.
    pub proxy: usize,
    /// DNA_DEPRECATED `Object *proxy_group`.
    pub proxy_group: usize,
    /// DNA_DEPRECATED `Object *proxy_from`.
    pub proxy_from: usize,
    /// DNA_DEPRECATED `bAction *poselib`.
    pub poselib: usize,
    /// `bPose *pose`.
    pub pose: usize,
    /// `ID *data`.
    pub data: usize,
    /// DNA_DEPRECATED `bGPdata *gpd`.
    pub gpd: usize,
    /// `bAnimVizSettings avs`.
    pub avs: AnimVizSettings,
    /// `bMotionPath *mpath`.
    pub mpath: usize,
    /// `ListBaseT<Effect>` (deprecated).
    pub effect: ListBase,
    /// `ListBaseT<bDeformGroup>` (versioning only).
    pub defbase: ListBase,
    /// `ListBaseT<bFaceMap>` (versioning only).
    pub fmaps: ListBase,
    /// `ListBaseT<ModifierData>`.
    pub modifiers: ListBase,
    /// `ListBaseT<GpencilModifierData>`.
    pub greasepencil_modifiers: ListBase,
    /// `ListBaseT<ShaderFxData>`.
    pub shader_fx: ListBase,
    pub mode: ObjectMode,
    pub restore_mode: ObjectMode,
    /// `Material **mat` → arena indices.
    pub mat: Vec<usize>,
    /// `char *matbits` → per-material link flag bytes.
    pub matbits: Vec<i8>,
    pub totcol: i32,
    pub actcol: i32,
    pub loc: [f32; 3],
    pub dloc: [f32; 3],
    pub scale: [f32; 3],
    /// DNA_DEPRECATED `dsize`.
    pub dsize: [f32; 3],
    pub dscale: [f32; 3],
    pub rot: [f32; 3],
    pub drot: [f32; 3],
    pub quat: [f32; 4],
    pub dquat: [f32; 4],
    pub rot_axis: [f32; 3],
    pub drot_axis: [f32; 3],
    pub rot_angle: f32,
    pub drot_angle: f32,
    pub parentinv: [[f32; 4]; 4],
    pub constinv: [[f32; 4]; 4],
    /// DNA_DEPRECATED `lay`.
    pub lay: u32,
    pub flag: i16,
    /// DNA_DEPRECATED `colbits`.
    pub colbits: i16,
    pub transflag: i16,
    pub protectflag: i16,
    pub trackflag: i16,
    pub upflag: i16,
    pub nlaflag: i16,
    pub duplicator_visibility_flag: i8,
    pub base_flag: i16,
    pub base_local_view_bits: u16,
    pub col_group: u16,
    pub col_mask: u16,
    pub rotmode: i16,
    pub boundtype: i8,
    pub collision_boundtype: i8,
    pub dtx: i16,
    pub dt: DrawType,
    pub empty_drawtype: i8,
    pub empty_drawsize: f32,
    pub instance_faces_scale: f32,
    pub index: i16,
    /// DNA_DEPRECATED `actdef`.
    pub actdef: u16,
    pub color: [f32; 4],
    pub visibility_flag: i32,
    pub softflag: i16,
    pub shapenr: i16,
    pub shapeflag: i8,
    /// `ListBaseT<bConstraint>`.
    pub constraints: ListBase,
    /// `ListBaseT<ObHook>`.
    pub hooks: ListBase,
    /// `ListBaseT<ParticleSystem>`.
    pub particlesystem: ListBase,
    /// `PartDeflect *pd`.
    pub pd: usize,
    /// `SoftBody *soft`.
    pub soft: usize,
    /// `Collection *instance_collection`.
    pub instance_collection: usize,
    /// DNA_DEPRECATED `FluidsimSettings *fluidsimSettings`.
    pub fluidsim_settings: usize,
    /// `ListBaseT<LinkData>`.
    pub pc_ids: ListBase,
    /// `RigidBodyOb *rigidbody_object`.
    pub rigidbody_object: usize,
    /// `RigidBodyCon *rigidbody_constraint`.
    pub rigidbody_constraint: usize,
    pub ima_ofs: [f32; 2],
    /// `ImageUser *iuser`.
    pub iuser: usize,
    pub empty_image_visibility_flag: i8,
    pub empty_image_depth: i8,
    pub empty_image_flag: i8,
    pub modifier_flag: ObjectModifierFlag,
    pub shadow_terminator_normal_offset: f32,
    pub shadow_terminator_geometry_offset: f32,
    pub shadow_terminator_shading_offset: f32,
    /// `PreviewImage *preview`.
    pub preview: usize,
    pub lineart: ObjectLineArt,
    /// `LightgroupMembership *lightgroup`.
    pub lightgroup: usize,
    /// `LightLinking *light_linking`.
    pub light_linking: usize,
    /// `LightProbeObjectCache *lightprobe_cache`.
    pub lightprobe_cache: usize,
    /// `bke::ObjectRuntime *runtime`.
    pub runtime: usize,
    /* `object_to_world`/`world_to_object` live on the runtime. */
}

impl Default for Object {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            type_: ObjectType::ObEmpty,
            partype: PAROBJECT,
            par1: 0,
            par2: 0,
            par3: 0,
            parsubstr: [0; MAX_NAME],
            parent: NIL,
            track: NIL,
            parent_bone_head_tail_factor: 1.0,
            proxy: NIL,
            proxy_group: NIL,
            proxy_from: NIL,
            poselib: NIL,
            pose: NIL,
            data: NIL,
            gpd: NIL,
            avs: 0,
            mpath: NIL,
            effect: ListBase::new(),
            defbase: ListBase::new(),
            fmaps: ListBase::new(),
            modifiers: ListBase::new(),
            greasepencil_modifiers: ListBase::new(),
            shader_fx: ListBase::new(),
            mode: OB_MODE_OBJECT,
            restore_mode: OB_MODE_OBJECT,
            mat: Vec::new(),
            matbits: Vec::new(),
            totcol: 0,
            actcol: 0,
            loc: [0.0; 3],
            dloc: [0.0; 3],
            scale: [1.0; 3],
            dsize: [0.0; 3],
            dscale: [1.0; 3],
            rot: [0.0; 3],
            drot: [0.0; 3],
            quat: DNA_DEFAULT_UNIT_QT,
            dquat: DNA_DEFAULT_UNIT_QT,
            rot_axis: [0.0, 1.0, 0.0],
            drot_axis: [0.0, 1.0, 0.0],
            rot_angle: 0.0,
            drot_angle: 0.0,
            parentinv: DNA_DEFAULT_UNIT_M4,
            constinv: DNA_DEFAULT_UNIT_M4,
            lay: 0,
            flag: OB_FLAG_USE_SIMULATION_CACHE,
            colbits: 0,
            transflag: 0,
            protectflag: OB_LOCK_ROT4D,
            trackflag: OB_POSY,
            upflag: OB_POSZ,
            nlaflag: 0,
            duplicator_visibility_flag: OB_DUPLI_FLAG_VIEWPORT | OB_DUPLI_FLAG_RENDER,
            base_flag: 0,
            base_local_view_bits: 0,
            col_group: 0x01,
            col_mask: 0xffff,
            rotmode: ROT_MODE_EUL,
            boundtype: OB_BOUND_BOX,
            collision_boundtype: OB_BOUND_BOX,
            dtx: 0,
            dt: OB_TEXTURE,
            empty_drawtype: OB_PLAINAXES,
            empty_drawsize: 1.0,
            instance_faces_scale: 1.0,
            index: 0,
            actdef: 0,
            color: [1.0; 4],
            visibility_flag: 0,
            softflag: 0,
            shapenr: 0,
            shapeflag: 0,
            constraints: ListBase::new(),
            hooks: ListBase::new(),
            particlesystem: ListBase::new(),
            pd: NIL,
            soft: NIL,
            instance_collection: NIL,
            fluidsim_settings: NIL,
            pc_ids: ListBase::new(),
            rigidbody_object: NIL,
            rigidbody_constraint: NIL,
            ima_ofs: [-0.5, -0.5],
            iuser: NIL,
            empty_image_visibility_flag: 0,
            empty_image_depth: OB_EMPTY_IMAGE_DEPTH_DEFAULT,
            empty_image_flag: 0,
            modifier_flag: 0,
            shadow_terminator_normal_offset: 0.0,
            shadow_terminator_geometry_offset: 0.1,
            shadow_terminator_shading_offset: 0.0,
            preview: NIL,
            lineart: ObjectLineArt::default(),
            lightgroup: NIL,
            light_linking: NIL,
            lightprobe_cache: NIL,
            runtime: NIL,
        }
    }
}

/// `ObHook`.
#[derive(Clone)]
pub struct ObHook {
    pub next: usize,
    pub prev: usize,
    /// `Object *parent`.
    pub parent: usize,
    pub parentinv: [[f32; 4]; 4],
    pub mat: [[f32; 4]; 4],
    pub cent: [f32; 3],
    pub falloff: f32,
    /// `char name[MAX_NAME]`.
    pub name: [u8; MAX_NAME],
    /// `int *indexar`.
    pub indexar: Vec<i32>,
    pub totindex: i32,
    pub curindex: i32,
    pub type_: i16,
    pub active: i16,
    pub force: f32,
}
impl Default for ObHook {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            parent: NIL,
            parentinv: [[0.0; 4]; 4],
            mat: [[0.0; 4]; 4],
            cent: [0.0; 3],
            falloff: 0.0,
            name: [0; MAX_NAME],
            indexar: Vec::new(),
            totindex: 0,
            curindex: 0,
            type_: 0,
            active: 0,
            force: 0.0,
        }
    }
}

pub const SELECT: i32 = 1;

/// `OB_TYPE_SUPPORT_MATERIAL`.
pub fn ob_type_support_material(t: i16) -> bool {
    (t >= ObjectType::ObMesh as i16 && t <= ObjectType::ObMball as i16)
        || (t >= ObjectType::ObCurves as i16 && t <= ObjectType::ObGreasePencil as i16)
}
/// `OB_TYPE_IS_GEOMETRY`.
pub fn ob_type_is_geometry(t: i16) -> bool {
    matches!(
        ObjectType::try_from(t).ok(),
        Some(
            ObjectType::ObMesh
                | ObjectType::ObSurf
                | ObjectType::ObFont
                | ObjectType::ObMball
                | ObjectType::ObCurvesLegacy
                | ObjectType::ObCurves
                | ObjectType::ObPointCloud
                | ObjectType::ObVolume
                | ObjectType::ObGreasePencil
        )
    )
}
/// `OB_TYPE_SUPPORT_VGROUP`.
pub fn ob_type_support_vgroup(t: i16) -> bool {
    t == ObjectType::ObMesh as i16
        || t == ObjectType::ObLattice as i16
        || t == ObjectType::ObGreasePencil as i16
}
/// `OB_TYPE_SUPPORT_EDITMODE`.
pub fn ob_type_support_editmode(t: i16) -> bool {
    matches!(
        ObjectType::try_from(t).ok(),
        Some(
            ObjectType::ObMesh
                | ObjectType::ObFont
                | ObjectType::ObCurvesLegacy
                | ObjectType::ObSurf
                | ObjectType::ObMball
                | ObjectType::ObLattice
                | ObjectType::ObArmature
                | ObjectType::ObCurves
                | ObjectType::ObPointCloud
                | ObjectType::ObGreasePencil
        )
    )
}
/// `OB_TYPE_SUPPORT_PARVERT`.
pub fn ob_type_support_parvert(t: i16) -> bool {
    matches!(
        ObjectType::try_from(t).ok(),
        Some(ObjectType::ObMesh | ObjectType::ObSurf | ObjectType::ObCurvesLegacy)
    ) || t == ObjectType::ObLattice as i16
}

/// `ObjectType::try_from` helper.
impl ObjectType {
    pub fn try_from(v: i16) -> Result<Self, ()> {
        use ObjectType::*;
        Ok(match v {
            0 => ObEmpty,
            1 => ObMesh,
            2 => ObCurvesLegacy,
            3 => ObSurf,
            4 => ObFont,
            5 => ObMball,
            10 => ObLamp,
            11 => ObCamera,
            12 => ObSpeaker,
            13 => ObLightProbe,
            22 => ObLattice,
            25 => ObArmature,
            26 => ObGpencilLegacy,
            27 => ObCurves,
            28 => ObPointCloud,
            29 => ObVolume,
            30 => ObGreasePencil,
            _ => return Err(()),
        })
    }
}
