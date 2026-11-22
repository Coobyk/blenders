//! Port of `source/blender/geometry` — geometry
//! algorithms (mesh/curve/volume/instances).

pub mod volume_grid_resample;
pub mod mesh_primitive_line;
pub mod point_merge;
pub mod mesh_selection;
pub mod foreach_geometry;
pub mod xpbd_constraint_coloring;
pub mod simplify_curves;
pub mod curve_constraints;
pub mod merge_layers;
pub mod curves_remove_and_split;
pub mod mesh_to_volume;
pub mod separate_geometry;
pub mod join_geometries;
pub mod mesh_primitive_grid;
pub mod fit_curves;
pub mod reverse_uv_sampler;
