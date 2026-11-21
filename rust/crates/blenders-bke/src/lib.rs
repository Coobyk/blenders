//! Port of `source/blender/blenkernel` — the BKE library (scene/DNA-bound
//! object logic). DNA struct ports live in `dna`; this crate holds the
//! `intern/*.cc` ports.
#![allow(dead_code)]

pub mod anonymous_attribute_id;
pub mod geometry_components;
pub mod icons_rasterize;
pub mod idtype;
pub mod speaker;
pub mod action_bones;
pub mod addon;
pub mod anim_data_bmain_utils;
pub mod armature_selection;
pub mod bake_data_block_map;
pub mod autoexec;
pub mod bake_items_paths;
pub mod blender_user_menu;
pub mod callbacks;
pub mod curve_convert;
pub mod wm_runtime;
pub mod subdiv_stats;
pub mod editlattice;
pub mod lib_id_eval;
pub mod lib_id_remapper;
pub mod main_invariants;
pub mod multires_subdiv;
pub mod subdiv;
pub mod text_suggestions;
pub mod subdiv_deform;
pub mod cpp_types;
pub mod mesh_debug;
pub mod mesh_merge_customdata;
pub mod editmesh_cache;
pub mod mesh_flip_faces;
pub mod mesh_topology_state;
pub mod node_enum_definition;
pub mod paint_canvas;
pub mod node_tree_dot_export;
pub mod paint_runtime;
pub mod scene_context;
pub mod sound_reader_cache;
pub mod volume_grid_fields;
pub mod blender_copybuffer;
pub mod blender_undo;
pub mod view3d_runtime;
pub mod vfontdata_freetype;
