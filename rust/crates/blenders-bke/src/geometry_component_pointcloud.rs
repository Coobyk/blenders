//! Port of `intern/geometry_component_pointcloud.cc` —
//! `PointCloudComponent`.
//!
//! Methods in `geometry_components.rs` (copy/owns_direct_data/
//! ensure_owns_direct_data/count_memory — `BKE_pointcloud_copy_for_eval`
//! injected). `attributes`/`attributes_for_write` PORT_PENDING
//! (see `pointcloud_attributes.rs` for the portable table side).

pub use crate::geometry_components::{GeometryComponent, GeometryOwnershipType, PointCloudComponent};
