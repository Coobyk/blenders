//! Port of `intern/geometry_component_grease_pencil.cc` —
//! `GreasePencilComponent`.
//!
//! Methods in `geometry_components.rs` (copy/owns_direct_data/
//! ensure_owns_direct_data/count_memory — `BKE_grease_pencil_copy_for_eval`
//! injected). `attributes`/`attributes_for_write` PORT_PENDING
//! (`greasepencil::get_attribute_accessor_functions` — see
//! `grease_pencil_attributes.rs`).

pub use crate::geometry_components::{GeometryComponent, GeometryOwnershipType, GreasePencilComponent};
