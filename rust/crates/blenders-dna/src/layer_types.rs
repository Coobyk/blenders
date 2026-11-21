//! Port of `DNA_layer_types.h` — `Base`, `LayerCollection`, `ViewLayer`.
use crate::id::NIL;
use blenders_bli::listbase::ListBase;

/// `eViewLayerEEVEEPassType`.
pub const EEVEE_RENDER_PASS_COMBINED: i32 = 1 << 0;
pub const EEVEE_RENDER_PASS_DEPTH: i32 = 1 << 1;
pub const EEVEE_RENDER_PASS_MIST: i32 = 1 << 2;
pub const EEVEE_RENDER_PASS_NORMAL: i32 = 1 << 3;
pub const EEVEE_RENDER_PASS_DIFFUSE_LIGHT: i32 = 1 << 4;
pub const EEVEE_RENDER_PASS_DIFFUSE_COLOR: i32 = 1 << 5;
pub const EEVEE_RENDER_PASS_SPECULAR_LIGHT: i32 = 1 << 6;
pub const EEVEE_RENDER_PASS_SPECULAR_COLOR: i32 = 1 << 7;
pub const EEVEE_RENDER_PASS_UNUSED_8: i32 = 1 << 8;
pub const EEVEE_RENDER_PASS_VOLUME_LIGHT: i32 = 1 << 9;
pub const EEVEE_RENDER_PASS_EMIT: i32 = 1 << 10;
pub const EEVEE_RENDER_PASS_ENVIRONMENT: i32 = 1 << 11;
pub const EEVEE_RENDER_PASS_SHADOW: i32 = 1 << 12;
pub const EEVEE_RENDER_PASS_AO: i32 = 1 << 13;
pub const EEVEE_RENDER_PASS_UNUSED_14: i32 = 1 << 14;
pub const EEVEE_RENDER_PASS_AOV: i32 = 1 << 15;
pub const EEVEE_RENDER_PASS_CRYPTOMATTE: i32 = 1 << 16;
pub const EEVEE_RENDER_PASS_CRYPTOMATTE_OBJECT: i32 = 1 << 16;
pub const EEVEE_RENDER_PASS_CRYPTOMATTE_ASSET: i32 = 1 << 17;
pub const EEVEE_RENDER_PASS_CRYPTOMATTE_MATERIAL: i32 = 1 << 18;
pub const EEVEE_RENDER_PASS_VECTOR: i32 = 1 << 19;
pub const EEVEE_RENDER_PASS_TRANSPARENT: i32 = 1 << 20;
pub const EEVEE_RENDER_PASS_POSITION: i32 = 1 << 21;
pub const EEVEE_RENDER_PASS_DENOISING_DEPTH: i32 = 1 << 22;
pub const EEVEE_RENDER_PASS_DENOISING_NORMAL: i32 = 1 << 23;
pub const EEVEE_RENDER_PASS_DENOISING_ROUGHNESS: i32 = 1 << 24;
pub const EEVEE_RENDER_PASS_DENOISING_DIFFUSE_ALBEDO: i32 = 1 << 25;
pub const EEVEE_RENDER_PASS_DENOISING_SPECULAR_ALBEDO: i32 = 1 << 26;
pub const EEVEE_RENDER_PASS_MAX_BIT: i32 = 26;

/// `eViewLayerEEVEEDenoisingPassFlag`.
pub const EEVEE_DENOISING_PASS_STORE: u32 = 1 << 0;
pub const EEVEE_DENOISING_PASS_USE_ALBEDO_ROUGHNESS_WEIGHTING: u32 = 1 << 1;

/// `eViewLayerGreasePencilFlags`.
pub const GREASE_PENCIL_AS_SEPARATE_PASS: i32 = 1 << 0;

/// `eViewLayerAOVType`.
pub const AOV_TYPE_VALUE: i32 = 0;
pub const AOV_TYPE_COLOR: i32 = 1;

/// `eViewLayerAOVFlag`.
pub const AOV_CONFLICT: i32 = 1 << 0;

/// `eViewLayerCryptomatteFlags`.
pub const VIEW_LAYER_CRYPTOMATTE_OBJECT: i16 = 1 << 0;
pub const VIEW_LAYER_CRYPTOMATTE_MATERIAL: i16 = 1 << 1;
pub const VIEW_LAYER_CRYPTOMATTE_ASSET: i16 = 1 << 2;
pub const VIEW_LAYER_CRYPTOMATTE_ACCURATE: i16 = 1 << 3;
pub const VIEW_LAYER_CRYPTOMATTE_ALL: i16 = VIEW_LAYER_CRYPTOMATTE_OBJECT
    | VIEW_LAYER_CRYPTOMATTE_MATERIAL
    | VIEW_LAYER_CRYPTOMATTE_ASSET;

/// `eBase_Flag`.
pub const BASE_SELECTED: i16 = 1 << 0;
pub const BASE_HIDDEN: i16 = 1 << 8;
pub const BASE_ENABLED_AND_MAYBE_VISIBLE_IN_VIEWPORT: i16 = 1 << 1;
pub const BASE_SELECTABLE: i16 = 1 << 2;
pub const BASE_FROM_DUPLI: i16 = 1 << 3;
pub const BASE_ENABLED_AND_VISIBLE_IN_DEFAULT_VIEWPORT: i16 = 1 << 4;
pub const BASE_FROM_SET: i16 = 1 << 5;
pub const BASE_ENABLED_VIEWPORT: i16 = 1 << 6;
pub const BASE_ENABLED_RENDER: i16 = 1 << 7;
pub const BASE_HOLDOUT: i16 = 1 << 10;
pub const BASE_INDIRECT_ONLY: i16 = 1 << 11;

/// `eLayerCollection_Flag`.
pub const LAYER_COLLECTION_EXCLUDE: i16 = 1 << 4;
pub const LAYER_COLLECTION_HOLDOUT: i16 = 1 << 5;
pub const LAYER_COLLECTION_INDIRECT_ONLY: i16 = 1 << 6;
pub const LAYER_COLLECTION_HIDE: i16 = 1 << 7;
pub const LAYER_COLLECTION_PREVIOUSLY_EXCLUDED: i16 = 1 << 8;

/// `eLayerCollection_RuntimeFlag`.
pub const LAYER_COLLECTION_HAS_OBJECTS: i16 = 1 << 0;
pub const LAYER_COLLECTION_HIDE_VIEWPORT: i16 = 1 << 2;
pub const LAYER_COLLECTION_VISIBLE_VIEW_LAYER: i16 = 1 << 4;

/// `eViewLayer_Flag`.
pub const VIEW_LAYER_RENDER: i16 = 1 << 0;
pub const VIEW_LAYER_FREESTYLE: i16 = 1 << 2;
pub const VIEW_LAYER_OUT_OF_SYNC: i16 = 1 << 3;
pub const VIEW_LAYER_HAS_EXPORT_COLLECTIONS: i16 = 1 << 4;

/// `SCE_LAY_FLAG_DEFAULT` (DNA_scene_enums.h).
pub const SCE_LAY_FLAG_DEFAULT: i32 = (1 << 15) - 1;
/// `SCE_PASS_COMBINED` (DNA_scene_enums.h / render).
pub const SCE_PASS_COMBINED: i32 = 1;

/// `Base`.
#[derive(Clone, Default)]
pub struct Base {
    pub next: usize,
    pub prev: usize,
    /// `Object *object`.
    pub object: usize,
    /// `Base *base_orig`.
    pub base_orig: usize,
    /// DNA_DEPRECATED `lay`.
    pub lay: u32,
    pub flag: i16,
    pub flag_from_collection: i16,
    pub flag_legacy: i16,
    pub local_view_bits: u16,
    pub local_collections_bits: u16,
}
impl Base {
    pub fn new() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            object: NIL,
            base_orig: NIL,
            ..Self::default()
        }
    }
}

/// `LayerCollection`.
#[derive(Clone, Default)]
pub struct LayerCollection {
    pub next: usize,
    pub prev: usize,
    /// `Collection *collection`.
    pub collection: usize,
    pub flag: i16,
    pub runtime_flag: i16,
    /// `ListBaseT<LayerCollection>` — indexed children.
    pub layer_collections: ListBase,
    pub local_collections_bits: u16,
}
impl LayerCollection {
    pub fn new() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            collection: NIL,
            ..Self::default()
        }
    }
}

/// `ViewLayerEEVEE`.
#[derive(Clone)]
pub struct ViewLayerEevee {
    pub render_passes: i32,
    pub ambient_occlusion_distance: f32,
    pub denoising_pass_flags: i32,
}
impl Default for ViewLayerEevee {
    fn default() -> Self {
        Self {
            render_passes: 0,
            ambient_occlusion_distance: 10.0,
            denoising_pass_flags: EEVEE_DENOISING_PASS_USE_ALBEDO_ROUGHNESS_WEIGHTING as i32,
        }
    }
}

/// `ViewLayerAOV`.
#[derive(Clone)]
pub struct ViewLayerAov {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
    pub flag: i32,
    pub type_: i32,
}
impl Default for ViewLayerAov {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            flag: 0,
            type_: AOV_TYPE_VALUE,
        }
    }
}

/// `ViewLayerLightgroup`.
#[derive(Clone)]
pub struct ViewLayerLightgroup {
    pub next: usize,
    pub prev: usize,
    /// `char name[64]`.
    pub name: [u8; 64],
}
impl Default for ViewLayerLightgroup {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
        }
    }
}

/// `LightgroupMembership`.
#[derive(Clone)]
pub struct LightgroupMembership {
    /// `char name[64]`.
    pub name: [u8; 64],
}
impl Default for LightgroupMembership {
    fn default() -> Self {
        Self { name: [0; 64] }
    }
}

/// `FreestyleConfig` (DNA_freestyle_types.h subset).
#[derive(Clone, Default)]
pub struct FreestyleConfig {
    /// `ListBaseT<FreestyleModuleConfig>`.
    pub modules: ListBase,
    /// `eFreestyleControl_Mode`.
    pub mode: i8,
    /// DNA_DEPRECATED `raycasting_algorithm`.
    pub raycasting_algorithm: i32,
    /// `eFreestyleConfig_Flags`.
    pub flags: i32,
    pub sphere_radius: f32,
    pub dkr_epsilon: f32,
    /// In radians.
    pub crease_angle: f32,
    /// `ListBaseT<FreestyleLineSet>`.
    pub linesets: ListBase,
}

/// `ViewLayer`.
#[derive(Clone)]
pub struct ViewLayer {
    pub next: usize,
    pub prev: usize,
    /// `char name[MAX_NAME]`.
    pub name: [u8; 64],
    pub flag: i16,
    /// `ListBaseT<Base>`.
    pub object_bases: ListBase,
    /// `SceneStats *stats`.
    pub stats: usize,
    /// `Base *basact`.
    pub basact: usize,
    /// `ListBaseT<LayerCollection>`.
    pub layer_collections: ListBase,
    /// `LayerCollection *active_collection`.
    pub active_collection: usize,
    pub layflag: i32,
    pub passflag: i32,
    pub pass_alpha_threshold: f32,
    pub cryptomatte_flag: i16,
    pub cryptomatte_levels: i16,
    pub grease_pencil_flags: i32,
    pub samples: i32,
    /// `Material *mat_override`.
    pub mat_override: usize,
    /// `World *world_override`.
    pub world_override: usize,
    /// `IDProperty *id_properties`.
    pub id_properties: usize,
    /// `IDProperty *system_properties`.
    pub system_properties: usize,
    pub freestyle_config: FreestyleConfig,
    pub eevee: ViewLayerEevee,
    /// `ListBaseT<ViewLayerAOV>`.
    pub aovs: ListBase,
    /// `ViewLayerAOV *active_aov`.
    pub active_aov: usize,
    /// `ListBaseT<ViewLayerLightgroup>`.
    pub lightgroups: ListBase,
    /// `ViewLayerLightgroup *active_lightgroup`.
    pub active_lightgroup: usize,
    /// `ViewLayerRuntime *runtime`.
    pub runtime: usize,
}
impl Default for ViewLayer {
    fn default() -> Self {
        Self {
            next: NIL,
            prev: NIL,
            name: [0; 64],
            flag: VIEW_LAYER_RENDER | VIEW_LAYER_FREESTYLE,
            object_bases: ListBase::new(),
            stats: NIL,
            basact: NIL,
            layer_collections: ListBase::new(),
            active_collection: NIL,
            layflag: SCE_LAY_FLAG_DEFAULT,
            passflag: SCE_PASS_COMBINED,
            pass_alpha_threshold: 0.5,
            cryptomatte_flag: VIEW_LAYER_CRYPTOMATTE_ACCURATE,
            cryptomatte_levels: 6,
            grease_pencil_flags: 0,
            samples: 0,
            mat_override: NIL,
            world_override: NIL,
            id_properties: NIL,
            system_properties: NIL,
            freestyle_config: FreestyleConfig::default(),
            eevee: ViewLayerEevee::default(),
            aovs: ListBase::new(),
            active_aov: NIL,
            lightgroups: ListBase::new(),
            active_lightgroup: NIL,
            runtime: NIL,
        }
    }
}
