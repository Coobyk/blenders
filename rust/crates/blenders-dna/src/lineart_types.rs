//! Port of `DNA_lineart_types.h` — line art modifier flags.

/// `eLineartMainFlags`.
pub type LineartMainFlags = i32;
pub const MOD_LINEART_INTERSECTION_AS_CONTOUR: LineartMainFlags = 1 << 0;
pub const MOD_LINEART_EVERYTHING_AS_CONTOUR: LineartMainFlags = 1 << 1;
pub const MOD_LINEART_ALLOW_DUPLI_OBJECTS: LineartMainFlags = 1 << 2;
pub const MOD_LINEART_ALLOW_OVERLAPPING_EDGES: LineartMainFlags = 1 << 3;
pub const MOD_LINEART_ALLOW_CLIPPING_BOUNDARIES: LineartMainFlags = 1 << 4;
pub const MOD_LINEART_LOOSE_AS_CONTOUR: LineartMainFlags = 1 << 6;
pub const MOD_LINEART_INVERT_SOURCE_VGROUP: LineartMainFlags = 1 << 7;
pub const MOD_LINEART_MATCH_OUTPUT_VGROUP: LineartMainFlags = 1 << 8;
pub const MOD_LINEART_FILTER_FACE_MARK: LineartMainFlags = 1 << 9;
pub const MOD_LINEART_FILTER_FACE_MARK_INVERT: LineartMainFlags = 1 << 10;
pub const MOD_LINEART_FILTER_FACE_MARK_BOUNDARIES: LineartMainFlags = 1 << 11;
pub const MOD_LINEART_CHAIN_LOOSE_EDGES: LineartMainFlags = 1 << 12;
pub const MOD_LINEART_CHAIN_GEOMETRY_SPACE: LineartMainFlags = 1 << 13;
pub const MOD_LINEART_ALLOW_OVERLAP_EDGE_TYPES: LineartMainFlags = 1 << 14;
pub const MOD_LINEART_USE_CREASE_ON_SMOOTH_SURFACES: LineartMainFlags = 1 << 15;
pub const MOD_LINEART_USE_CREASE_ON_SHARP_EDGES: LineartMainFlags = 1 << 16;
pub const MOD_LINEART_USE_CUSTOM_CAMERA: LineartMainFlags = 1 << 17;
pub const MOD_LINEART_FILTER_FACE_MARK_KEEP_CONTOUR: LineartMainFlags = 1 << 18;
pub const MOD_LINEART_USE_BACK_FACE_CULLING: LineartMainFlags = 1 << 19;
pub const MOD_LINEART_USE_IMAGE_BOUNDARY_TRIMMING: LineartMainFlags = 1 << 20;
pub const MOD_LINEART_CHAIN_PRESERVE_DETAILS: LineartMainFlags = 1 << 22;
pub const MOD_LINEART_SHADOW_USE_SILHOUETTE: LineartMainFlags = 1 << 24;

/// `eLineartEdgeFlag`.
pub type LineartEdgeFlag = i32;
pub const MOD_LINEART_EDGE_FLAG_EDGE_MARK: LineartEdgeFlag = 1 << 0;
pub const MOD_LINEART_EDGE_FLAG_CONTOUR: LineartEdgeFlag = 1 << 1;
pub const MOD_LINEART_EDGE_FLAG_CREASE: LineartEdgeFlag = 1 << 2;
pub const MOD_LINEART_EDGE_FLAG_MATERIAL: LineartEdgeFlag = 1 << 3;
pub const MOD_LINEART_EDGE_FLAG_INTERSECTION: LineartEdgeFlag = 1 << 4;
pub const MOD_LINEART_EDGE_FLAG_LOOSE: LineartEdgeFlag = 1 << 5;
pub const MOD_LINEART_EDGE_FLAG_LIGHT_CONTOUR: LineartEdgeFlag = 1 << 6;
pub const MOD_LINEART_EDGE_FLAG_PROJECTED_SHADOW: LineartEdgeFlag = 1 << 8;
pub const MOD_LINEART_EDGE_FLAG_SHADOW_FACING_LIGHT: LineartEdgeFlag = 1 << 9;
pub const MOD_LINEART_EDGE_FLAG_CHAIN_PICKED: LineartEdgeFlag = 1 << 10;
pub const MOD_LINEART_EDGE_FLAG_CLIPPED: LineartEdgeFlag = 1 << 11;
pub const MOD_LINEART_EDGE_FLAG_CONTOUR_SECONDARY: LineartEdgeFlag = 1 << 12;
pub const MOD_LINEART_EDGE_FLAG_INHIBIT: LineartEdgeFlag = 1 << 14;
pub const MOD_LINEART_EDGE_FLAG_NEXT_IS_DUPLICATION: LineartEdgeFlag = 1 << 15;

/// `MOD_LINEART_EDGE_FLAG_ALL_TYPE`/`INIT_TYPE`/`TYPE_MAX_BITS` defines.
pub const MOD_LINEART_EDGE_FLAG_ALL_TYPE: i32 = 0x01ff;
pub const MOD_LINEART_EDGE_FLAG_INIT_TYPE: i32 = 0x37;
pub const MOD_LINEART_EDGE_FLAG_TYPE_MAX_BITS: i32 = 7;
