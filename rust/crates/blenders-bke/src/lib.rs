//! Port of `source/blender/blenkernel` — the BKE library (scene/DNA-bound
//! object logic). DNA struct ports live in `dna`; this crate holds the
//! `intern/*.cc` ports.
#![allow(dead_code)]

pub mod anonymous_attribute_id;
pub mod idtype;
pub mod speaker;
pub mod armature_selection;
pub mod callbacks;
pub mod lib_id_eval;
pub mod mesh_flip_faces;
pub mod paint_runtime;
pub mod scene_context;
pub mod view3d_runtime;
