//! Ports of `source/blender/makesdna/DNA_*.h` — the serialized data-block
//! structs every blenkernel file builds on. Pointers become `usize` arena
//! indices / `Option<Box<>>` per the porting conventions.
#![allow(dead_code)]

pub mod id;
pub mod image_types;
pub mod collection_types;
pub mod action_types;
pub mod anim_enums;
pub mod armature_types;
pub mod color_types;
pub mod curve_enums;
pub mod curve_types;
pub mod anim_types;
pub mod key_types;
pub mod vfont_types;
pub mod meta_types;
pub mod lattice_types;
pub mod camera_types;
pub mod light_types;
pub mod view2d_types;
pub mod mask_types;
pub mod nla_types;
pub mod id_enums;
pub mod layer_types;
pub mod material_types;
pub mod scene_enums;
pub mod space_enums;
pub mod scene_types;
pub mod vec_types;
pub mod view3d_types;
pub mod viewer_path_types;
pub mod customdata_types;
pub mod meshdata_types;
pub mod mesh_types;
pub mod object_types;
pub mod small_types;
