//! Port of `DNA_view3d_enums.h` — mechanically converted enum constants.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
/// `eV3DOffscreenDrawFlag`.
pub type V3doffscreendrawflag = i32;
pub const V3D_OFSDRAW_NONE: V3doffscreendrawflag = 0;
pub const V3D_OFSDRAW_SHOW_ANNOTATION: V3doffscreendrawflag = 1 << 0;
pub const V3D_OFSDRAW_OVERRIDE_SCENE_SETTINGS: V3doffscreendrawflag = 1 << 1;
pub const V3D_OFSDRAW_SHOW_GRIDFLOOR: V3doffscreendrawflag = 1 << 2;
pub const V3D_OFSDRAW_SHOW_SELECTION: V3doffscreendrawflag = 1 << 3;
pub const V3D_OFSDRAW_XR_SHOW_CONTROLLERS: V3doffscreendrawflag = 1 << 4;
pub const V3D_OFSDRAW_XR_SHOW_CUSTOM_OVERLAYS: V3doffscreendrawflag = 1 << 5;
pub const V3D_OFSDRAW_SHOW_OBJECT_EXTRAS: V3doffscreendrawflag = 1 << 6;
pub const V3D_OFSDRAW_XR_SHOW_PASSTHROUGH: V3doffscreendrawflag = 1 << 7;
pub const V3D_OFSDRAW_NO_WORLD_BACKGROUND_OVERRIDE: V3doffscreendrawflag = 1 << 8;

/// `eV3DShadingLightingMode`.
pub type V3dshadinglightingmode = i8;
pub const V3D_LIGHTING_FLAT: V3dshadinglightingmode = 0;
pub const V3D_LIGHTING_STUDIO: V3dshadinglightingmode = 1;
pub const V3D_LIGHTING_MATCAP: V3dshadinglightingmode = 2;

/// `eV3DShadingColorType`.
pub type V3dshadingcolortype = i8;
pub const V3D_SHADING_MATERIAL_COLOR: V3dshadingcolortype = 0;
pub const V3D_SHADING_RANDOM_COLOR: V3dshadingcolortype = 1;
pub const V3D_SHADING_SINGLE_COLOR: V3dshadingcolortype = 2;
pub const V3D_SHADING_TEXTURE_COLOR: V3dshadingcolortype = 3;
pub const V3D_SHADING_OBJECT_COLOR: V3dshadingcolortype = 4;
pub const V3D_SHADING_VERTEX_COLOR: V3dshadingcolortype = 5;

/// `eV3DShadingBackgroundType`.
pub type V3dshadingbackgroundtype = i8;
pub const V3D_SHADING_BACKGROUND_THEME: V3dshadingbackgroundtype = 0;
pub const V3D_SHADING_BACKGROUND_WORLD: V3dshadingbackgroundtype = 1;
pub const V3D_SHADING_BACKGROUND_VIEWPORT: V3dshadingbackgroundtype = 2;

