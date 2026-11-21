//! Port of `DNA_material_types.h` — `Material`, `MaterialGPencilStyle`,
//! `MaterialLineArt`, `TexPaintSlot` + material enums.
use crate::id::{ID, NIL};
use blenders_bli::listbase::ListBase;

/// `eMaterialGPencilStyle_Flag`.
pub const GP_MATERIAL_FILL_PATTERN: i16 = 1 << 0;
pub const GP_MATERIAL_HIDE: i16 = 1 << 1;
pub const GP_MATERIAL_LOCKED: i16 = 1 << 2;
pub const GP_MATERIAL_HIDE_ONIONSKIN: i16 = 1 << 3;
pub const GP_MATERIAL_TEX_CLAMP: i16 = 1 << 4;
pub const GP_MATERIAL_FILL_TEX_MIX: i16 = 1 << 5;
pub const GP_MATERIAL_FLIP_FILL: i16 = 1 << 6;
pub const GP_MATERIAL_STROKE_PATTERN: i16 = 1 << 7;
pub const GP_MATERIAL_STROKE_SHOW: i16 = 1 << 8;
pub const GP_MATERIAL_FILL_SHOW: i16 = 1 << 9;
pub const GP_MATERIAL_STROKE_TEX_MIX: i16 = 1 << 11;
pub const GP_MATERIAL_DISABLE_STENCIL: i16 = 1 << 12;
pub const GP_MATERIAL_IS_STROKE_HOLDOUT: i16 = 1 << 13;
pub const GP_MATERIAL_IS_FILL_HOLDOUT: i16 = 1 << 14;
pub const GP_MATERIAL_USE_DOTS_RANDOMIZATION: i16 = 1 << 15;

/// `eMaterialGPencilStyle_Mode`.
pub const GP_MATERIAL_MODE_LINE: i32 = 0;
pub const GP_MATERIAL_MODE_DOT: i32 = 1;
pub const GP_MATERIAL_MODE_SQUARE: i32 = 2;

/// `eMaterialLineArtFlags`.
pub const LRT_MATERIAL_MASK_ENABLED: i32 = 1 << 0;
pub const LRT_MATERIAL_CUSTOM_OCCLUSION_EFFECTIVENESS: i32 = 1 << 1;
pub const LRT_MATERIAL_CUSTOM_INTERSECTION_PRIORITY: i32 = 1 << 2;

pub const MAXMAT: i32 = 32767 - 1;

/// `eMaterial_Flag`.
pub const MA_IS_USED: i16 = 1 << 0;
pub const MA_DS_EXPAND: i16 = 1 << 1;
pub const MA_DS_SHOW_TEXS: i16 = 1 << 2;

/// `eMaterial_RampBlend`.
pub const MA_RAMP_BLEND: i32 = 0;
pub const MA_RAMP_ADD: i32 = 1;
pub const MA_RAMP_MULT: i32 = 2;
pub const MA_RAMP_SUB: i32 = 3;
pub const MA_RAMP_SCREEN: i32 = 4;
pub const MA_RAMP_DIV: i32 = 5;
pub const MA_RAMP_DIFF: i32 = 6;
pub const MA_RAMP_DARK: i32 = 7;
pub const MA_RAMP_LIGHT: i32 = 8;
pub const MA_RAMP_OVERLAY: i32 = 9;
pub const MA_RAMP_DODGE: i32 = 10;
pub const MA_RAMP_BURN: i32 = 11;
pub const MA_RAMP_HUE: i32 = 12;
pub const MA_RAMP_SAT: i32 = 13;
pub const MA_RAMP_VAL: i32 = 14;
pub const MA_RAMP_COLOR: i32 = 15;
pub const MA_RAMP_SOFT: i32 = 16;
pub const MA_RAMP_LINEAR: i32 = 17;
pub const MA_RAMP_EXCLUSION: i32 = 18;

/// `eMTex_TexCo`.
pub const TEXCO_ORCO: i32 = 1 << 0;
pub const TEXCO_GLOB: i32 = 1 << 3;
pub const TEXCO_UV: i32 = 1 << 4;
pub const TEXCO_OBJECT: i32 = 1 << 5;
pub const TEXCO_WINDOW: i32 = 1 << 10;
pub const TEXCO_STRAND: i32 = 1 << 13;
pub const TEXCO_PARTICLE: i32 = 1 << 13;

/// `eMTex_MapTo`.
pub const MAP_COL: i32 = 1 << 0;
pub const MAP_ALPHA: i32 = 1 << 7;

/// `ePreviewType`.
pub const MA_FLAT: i8 = 0;
pub const MA_SPHERE: i8 = 1;
pub const MA_CUBE: i8 = 2;
pub const MA_SHADERBALL: i8 = 3;
pub const MA_SPHERE_A: i8 = 4;
pub const MA_TEXTURE: i8 = 5;
pub const MA_LAMP: i8 = 6;
pub const MA_SKY: i8 = 7;
pub const MA_HAIR: i8 = 10;
pub const MA_ATMOS: i8 = 11;
pub const MA_CLOTH: i8 = 12;
pub const MA_FLUID: i8 = 13;

/// `eMaterial_PreviewFlag`.
pub const MA_PREVIEW_WORLD: i16 = 1 << 0;

/// `eMaterial_SurfaceRenderMethod`.
pub const MA_SURFACE_METHOD_DEFERRED: i8 = 0;
pub const MA_SURFACE_METHOD_FORWARD: i8 = 1;

/// `eMaterial_VolumeIntersectionMethod`.
pub const MA_VOLUME_ISECT_FAST: i8 = 0;
pub const MA_VOLUME_ISECT_ACCURATE: i8 = 1;

/// `eMaterial_BlendMethod`.
pub const MA_BM_SOLID: i8 = 0;
pub const MA_BM_CLIP: i8 = 3;
pub const MA_BM_HASHED: i8 = 4;
pub const MA_BM_BLEND: i8 = 5;

/// `eMaterial_BlendFlag`.
pub const MA_BL_HIDE_BACKFACE: i8 = 1 << 0;
pub const MA_BL_SS_REFRACTION: i8 = 1 << 1;
pub const MA_BL_CULL_BACKFACE: i8 = 1 << 2;
pub const MA_BL_TRANSLUCENCY: i8 = 1 << 3;
pub const MA_BL_LIGHTPROBE_VOLUME_DOUBLE_SIDED: i8 = 1 << 4;
pub const MA_BL_CULL_BACKFACE_SHADOW: i8 = 1 << 5;
pub const MA_BL_TRANSPARENT_SHADOW: i8 = 1 << 6;
pub const MA_BL_THICKNESS_FROM_SHADOW: i8 = 1 << 7;

/// `eMaterial_BlendShadow`.
pub const MA_BS_NONE: i8 = 0;
pub const MA_BS_SOLID: i8 = 1;
pub const MA_BS_CLIP: i8 = 2;
pub const MA_BS_HASHED: i8 = 3;

/// `eMaterial_DisplacementMethod`.
pub const MA_DISPLACEMENT_BUMP: i8 = 0;
pub const MA_DISPLACEMENT_DISPLACE: i8 = 1;
pub const MA_DISPLACEMENT_BOTH: i8 = 2;

/// `eMaterial_ThicknessMode`.
pub const MA_THICKNESS_SPHERE: i8 = 0;
pub const MA_THICKNESS_SLAB: i8 = 1;

/// `eMaterialGPencilStyle_StrokeStyle`.
pub const GP_MATERIAL_STROKE_STYLE_SOLID: i16 = 0;
pub const GP_MATERIAL_STROKE_STYLE_TEXTURE: i16 = 1;

/// `eMaterialGPencilStyle_FillStyle`.
pub const GP_MATERIAL_FILL_STYLE_SOLID: i16 = 0;
pub const GP_MATERIAL_FILL_STYLE_GRADIENT: i16 = 1;
pub const GP_MATERIAL_FILL_STYLE_CHECKER: i16 = 2;
pub const GP_MATERIAL_FILL_STYLE_TEXTURE: i16 = 3;

/// `eMaterialGPencilStyle_GradientType`.
pub const GP_MATERIAL_GRADIENT_LINEAR: i32 = 0;
pub const GP_MATERIAL_GRADIENT_RADIAL: i32 = 1;

/// `eMaterialGPencilStyle_FollowMode`.
pub const GP_MATERIAL_FOLLOW_PATH: i32 = 0;
pub const GP_MATERIAL_FOLLOW_OBJ: i32 = 1;
pub const GP_MATERIAL_FOLLOW_FIXED: i32 = 2;

/// `eMaterialGPencilPlacementMode`.
pub const GP_MATERIAL_PLACEMENT_COUNT: i32 = 0;
pub const GP_MATERIAL_PLACEMENT_RADIUS: i32 = 1;
pub const GP_MATERIAL_PLACEMENT_DENSITY: i32 = 2;

/// `TexPaintSlot`.
#[derive(Clone, Default)]
pub struct TexPaintSlot {
    /// `Image *ima`.
    pub ima: usize,
    /// `ImageUser *image_user`.
    pub image_user: usize,
    /// `char *uvname`.
    pub uvname: Option<String>,
    /// `char *attribute_name`.
    pub attribute_name: Option<String>,
    pub valid: i32,
    pub interp: i32,
}

/// `MaterialGPencilStyle`.
#[derive(Clone)]
pub struct MaterialGpencilStyle {
    /// `Image *sima`, `*ima`.
    pub sima: usize,
    pub ima: usize,
    /// `ImageUser *stroke_iuser`, `*fill_iuser`.
    pub stroke_iuser: usize,
    pub fill_iuser: usize,
    pub stroke_rgba: [f32; 4],
    pub fill_rgba: [f32; 4],
    pub mix_rgba: [f32; 4],
    pub flag: i16,
    pub index: i16,
    pub stroke_style: i16,
    pub fill_style: i16,
    pub mix_factor: f32,
    /// DNA_DEPRECATED `gradient_angle`, `gradient_radius`, `gradient_scale`, `gradient_shift`.
    pub gradient_angle: f32,
    pub gradient_radius: f32,
    pub gradient_scale: [f32; 2],
    pub gradient_shift: [f32; 2],
    pub texture_angle: f32,
    pub texture_scale: [f32; 2],
    pub texture_offset: [f32; 2],
    /// DNA_DEPRECATED `texture_opacity`.
    pub texture_opacity: f32,
    pub texture_pixsize: f32,
    pub mode: i32,
    pub gradient_type: i32,
    pub mix_stroke_factor: f32,
    pub alignment_mode: i32,
    pub alignment_rotation: f32,
    pub placement_mode: i32,
    pub placement_count: i32,
    pub placement_radius_spacing: f32,
    pub placement_density: f32,
    pub random_size_factor: f32,
    pub random_strength_factor: f32,
    pub random_rotation_factor: f32,
    pub random_hue_factor: f32,
    pub random_saturation_factor: f32,
    pub random_value_factor: f32,
    pub random_noise_scale: f32,
    pub random_noise_seed: i32,
    /// `ColorBand *gradient`.
    pub gradient: usize,
}
impl Default for MaterialGpencilStyle {
    fn default() -> Self {
        Self {
            sima: NIL,
            ima: NIL,
            stroke_iuser: NIL,
            fill_iuser: NIL,
            stroke_rgba: [0.0; 4],
            fill_rgba: [0.0; 4],
            mix_rgba: [0.0; 4],
            flag: 0,
            index: 0,
            stroke_style: GP_MATERIAL_STROKE_STYLE_SOLID,
            fill_style: GP_MATERIAL_FILL_STYLE_SOLID,
            mix_factor: 0.0,
            gradient_angle: 0.0,
            gradient_radius: 0.0,
            gradient_scale: [0.0; 2],
            gradient_shift: [0.0; 2],
            texture_angle: 0.0,
            texture_scale: [0.0; 2],
            texture_offset: [0.0; 2],
            texture_opacity: 0.0,
            texture_pixsize: 0.0,
            mode: GP_MATERIAL_MODE_LINE,
            gradient_type: GP_MATERIAL_GRADIENT_LINEAR,
            mix_stroke_factor: 0.0,
            alignment_mode: GP_MATERIAL_FOLLOW_PATH,
            alignment_rotation: 0.0,
            placement_mode: GP_MATERIAL_PLACEMENT_COUNT,
            placement_count: 0,
            placement_radius_spacing: 0.0,
            placement_density: 0.0,
            random_size_factor: 0.0,
            random_strength_factor: 0.0,
            random_rotation_factor: 0.0,
            random_hue_factor: 0.0,
            random_saturation_factor: 0.0,
            random_value_factor: 0.0,
            random_noise_scale: 0.0,
            random_noise_seed: 0,
            gradient: NIL,
        }
    }
}

/// `MaterialLineArt`.
#[derive(Clone)]
pub struct MaterialLineArt {
    pub flags: i32,
    pub material_mask_bits: u8,
    pub mat_occlusion: u8,
    pub intersection_priority: u8,
}
impl Default for MaterialLineArt {
    fn default() -> Self {
        Self {
            flags: 0,
            material_mask_bits: 0,
            mat_occlusion: 1,
            intersection_priority: 0,
        }
    }
}

/// `Material`.
#[derive(Clone)]
pub struct Material {
    pub id: ID,
    /// `AnimData *adt`.
    pub adt: usize,
    pub flag: i16,
    pub surface_render_method: i8,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub specr: f32,
    pub specg: f32,
    pub specb: f32,
    /// DNA_DEPRECATED `alpha`, `ray_mirror`, `gloss_mir`.
    pub alpha: f32,
    pub ray_mirror: f32,
    pub spec: f32,
    pub gloss_mir: f32,
    pub roughness: f32,
    pub metallic: f32,
    /// DNA_DEPRECATED `use_nodes`.
    pub use_nodes: i8,
    pub pr_type: i8,
    pub pr_texture: i16,
    pub pr_flag: i16,
    pub index: i16,
    /// `bNodeTree *nodetree`.
    pub nodetree: usize,
    /// `PreviewImage *preview`.
    pub preview: usize,
    pub line_col: [f32; 4],
    pub line_priority: i16,
    pub vcol_alpha: i16,
    pub paint_active_slot: i16,
    pub paint_clone_slot: i16,
    pub tot_slots: i16,
    pub displacement_method: i8,
    pub thickness_mode: i8,
    pub alpha_threshold: f32,
    pub refract_depth: f32,
    pub blend_method: i8,
    pub blend_shadow: i8,
    pub blend_flag: i8,
    pub volume_intersection_method: i8,
    pub inflate_bounds: f32,
    /// `TexPaintSlot *texpaintslot` → Vec.
    pub texpaintslot: Vec<TexPaintSlot>,
    /// `ListBaseT<LinkData>`.
    pub gpumaterial: ListBase,
    /// `MaterialGPencilStyle *gp_style`.
    pub gp_style: usize,
    pub lineart: MaterialLineArt,
}
impl Default for Material {
    fn default() -> Self {
        Self {
            id: ID::default(),
            adt: NIL,
            flag: 0,
            surface_render_method: MA_SURFACE_METHOD_DEFERRED,
            r: 0.8,
            g: 0.8,
            b: 0.8,
            a: 1.0,
            specr: 1.0,
            specg: 1.0,
            specb: 1.0,
            alpha: 0.0,
            ray_mirror: 0.0,
            spec: 0.5,
            gloss_mir: 0.0,
            roughness: 0.4,
            metallic: 0.0,
            use_nodes: 0,
            pr_type: MA_SPHERE,
            pr_texture: 0,
            pr_flag: 0,
            index: 0,
            nodetree: NIL,
            preview: NIL,
            line_col: [0.0; 4],
            line_priority: 0,
            vcol_alpha: 0,
            paint_active_slot: 0,
            paint_clone_slot: 0,
            tot_slots: 0,
            displacement_method: MA_DISPLACEMENT_BUMP,
            thickness_mode: MA_THICKNESS_SPHERE,
            alpha_threshold: 0.5,
            refract_depth: 0.0,
            blend_method: MA_BM_SOLID,
            blend_shadow: MA_BS_SOLID,
            blend_flag: MA_BL_TRANSPARENT_SHADOW,
            volume_intersection_method: MA_VOLUME_ISECT_FAST,
            inflate_bounds: 0.0,
            texpaintslot: Vec::new(),
            gpumaterial: ListBase::new(),
            gp_style: NIL,
            lineart: MaterialLineArt::default(),
        }
    }
}
