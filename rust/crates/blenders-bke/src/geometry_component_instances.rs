//! Port of `intern/geometry_component_instances.cc` — `InstancesComponent`.
//!
//! Methods in `geometry_components.rs` (copy/owns_direct_data/
//! ensure_owns_direct_data/count_memory — deep-clone Vec, always-owned).
//! `attributes`/`attributes_for_write` PORT_PENDING (accessor fns —
//! see `instances_attributes.rs` for the portable table side).

pub use crate::geometry_components::{GeometryComponent, GeometryOwnershipType, InstancesComponent};
