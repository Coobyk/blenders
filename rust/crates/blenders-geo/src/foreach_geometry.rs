//! Port of `intern/foreach_geometry.cc` —
//! per-geometry-component iteration dispatch.

/// `eGeometryComponentType` ordinals (verbatim
/// from `BKE_geometry_set.hh`).
pub const GEO_COMPONENT_TYPE_MESH: i32 = 0;
pub const GEO_COMPONENT_TYPE_POINT_CLOUD: i32 = 1;
pub const GEO_COMPONENT_TYPE_INSTANCES: i32 = 2;
pub const GEO_COMPONENT_TYPE_VOLUME: i32 = 3;
pub const GEO_COMPONENT_TYPE_CURVE: i32 = 4;
pub const GEO_COMPONENT_TYPE_GREASE_PENCIL: i32 = 5;
pub const GEO_COMPONENT_TYPE_EDIT: i32 = 6;
/// `foreach_geometry` callback registry
// (verbatim — visit each present component).
pub fn foreach_geometry(
    components: &[i32],
    mut visit: impl FnMut(i32),
) {
    for &c in components {
        visit(c);
    }
}
