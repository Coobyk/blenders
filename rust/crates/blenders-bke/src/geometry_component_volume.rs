//! Port of `intern/geometry_component_volume.cc` — `VolumeComponent`.
//!
//! Component methods live in `geometry_components.rs` (VolumeComponent:
//! ctor/clear/replace/release/get/get_for_write/copy/owns_direct_data/
//! ensure_owns_direct_data/count_memory — all ported there with
//! `BKE_volume_copy_for_eval`/`BKE_volume_count_memory` injected).
//! `attributes`/`attributes_for_write` PORT_PENDING (accessor fns).

pub use crate::geometry_components::{GeometryComponent, GeometryOwnershipType, VolumeComponent};
