//! Ports of `source/blender/makesdna/DNA_*.h` — the serialized data-block
//! structs every blenkernel file builds on. Pointers become `usize` arena
//! indices / `Option<Box<>>` per the porting conventions.
#![allow(dead_code)]

pub mod id;
pub mod collection_types;
pub mod action_types;
pub mod anim_types;
pub mod id_enums;
pub mod layer_types;
pub mod material_types;
pub mod scene_types;
pub mod vec_types;
pub mod customdata_types;
pub mod meshdata_types;
pub mod mesh_types;
pub mod object_types;
pub mod small_types;
