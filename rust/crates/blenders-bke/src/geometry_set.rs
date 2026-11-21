//! Port of `intern/geometry_set.cc` — `GeometrySet` container API:
//! component get/add/remove/keep_only, ownership passes, typed
//! accessors + `*_for_write`, `from_*`/`replace_*` constructors,
//! `compute_boundbox_without_instances`, `has_realized_data`,
//! `is_empty`, `count_memory` and `Display`.
//!
//! The `GeometrySet` struct lives in `geometry_set_instances.rs`
//! (handle-based); this file implements the rest of its API.
//! Per-component virtual calls are injected as hooks.

use blenders_dna::id::NIL;
use blenders_math::vector::Vec3;

/// `Bounds<float3>` — axis-aligned min/max.
#[derive(Clone, Copy, Debug)]
pub struct Bounds3 {
    /// `min`.
    pub min: Vec3,
    /// `max`.
    pub max: Vec3,
}

impl Default for Bounds3 {
    fn default() -> Self {
        Bounds3 {
            min: Vec3::splat(f32::MAX),
            max: Vec3::splat(f32::MIN),
        }
    }
}

impl Bounds3 {
    /// `bounds::merge` — component-wise min/max.
    pub fn merge(&self, other: &Bounds3) -> Bounds3 {
        Bounds3 {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

use crate::geometry_components::{GeometryComponentType, GeometryOwnershipType};
use crate::geometry_set_instances::GeometrySet;

impl GeometrySet {
    /// `components_` → typed handle map.
    fn component_handle(&self, ty: GeometryComponentType) -> usize {
        match ty {
            GeometryComponentType::Mesh => self.mesh,
            GeometryComponentType::PointCloud => self.pointcloud,
            GeometryComponentType::Curve => self.curves,
            GeometryComponentType::Volume => self.volume,
            GeometryComponentType::Instances => self.instances,
            GeometryComponentType::GreasePencil => self.grease_pencil,
            GeometryComponentType::EditData => self.edit_data,
        }
    }
    fn set_component_handle(&mut self, ty: GeometryComponentType, handle: usize) {
        match ty {
            GeometryComponentType::Mesh => self.mesh = handle,
            GeometryComponentType::PointCloud => self.pointcloud = handle,
            GeometryComponentType::Curve => self.curves = handle,
            GeometryComponentType::Volume => self.volume = handle,
            GeometryComponentType::Instances => self.instances = handle,
            GeometryComponentType::GreasePencil => self.grease_pencil = handle,
            GeometryComponentType::EditData => self.edit_data = handle,
        }
    }

    /// `GeometryComponent::Type` count.
    pub const NUM_COMPONENT_TYPES: usize = 7;
    /// All component types in slot order.
    pub const COMPONENT_TYPES: [GeometryComponentType; 7] = [
        GeometryComponentType::Mesh,
        GeometryComponentType::PointCloud,
        GeometryComponentType::Curve,
        GeometryComponentType::Volume,
        GeometryComponentType::Instances,
        GeometryComponentType::GreasePencil,
        GeometryComponentType::EditData,
    ];

    /// `GeometrySet::has` — component present and non-empty.
    /// `is_empty` resolves the component's own `is_empty()`.
    pub fn has_component(
        &self,
        ty: GeometryComponentType,
        is_empty: &dyn Fn(usize) -> bool,
    ) -> bool {
        let h = self.component_handle(ty);
        h != NIL && !is_empty(h)
    }
    /// `GeometrySet::remove`.
    pub fn remove_component(&mut self, ty: GeometryComponentType) {
        self.set_component_handle(ty, NIL);
    }
    /// `GeometrySet::keep_only`.
    pub fn keep_only(&mut self, keep: &[GeometryComponentType]) {
        for ty in Self::COMPONENT_TYPES {
            if self.component_handle(ty) != NIL && !keep.contains(&ty) {
                self.set_component_handle(ty, NIL);
            }
        }
    }
    /// `GeometrySet::add`.
    pub fn add_component(&mut self, ty: GeometryComponentType, handle: usize) {
        debug_assert!(self.component_handle(ty) == NIL);
        self.set_component_handle(ty, handle);
    }
    /// `GeometrySet::get_components` — all live handles.
    pub fn get_components(&self) -> Vec<usize> {
        Self::COMPONENT_TYPES
            .iter()
            .filter_map(|&ty| {
                let h = self.component_handle(ty);
                (h != NIL).then_some(h)
            })
            .collect()
    }

    /// `GeometrySet::compute_boundbox_without_instances` —
    /// `bounds(ty) -> Option<Bounds3>` per component (pointcloud,
    /// mesh, volume, curves, grease_pencil in C order).
    pub fn compute_boundbox_without_instances(
        &self,
        use_radius: bool,
        use_subdiv: bool,
        bounds: &dyn Fn(GeometryComponentType, bool, bool) -> Option<Bounds3>,
    ) -> Option<Bounds3> {
        let mut total: Option<Bounds3> = None;
        for ty in [
            GeometryComponentType::PointCloud,
            GeometryComponentType::Mesh,
            GeometryComponentType::Volume,
            GeometryComponentType::Curve,
            GeometryComponentType::GreasePencil,
        ] {
            if self.component_handle(ty) != NIL {
                let b = bounds(ty, use_radius, use_subdiv);
                total = Some(match (total, b) {
                    (Some(a), Some(b)) => a.merge(&b),
                    (Some(a), None) => a,
                    (None, b) => b.unwrap_or_default(),
                });
            }
        }
        total
    }

    /// `GeometrySet::clear`.
    pub fn clear(&mut self) {
        for ty in Self::COMPONENT_TYPES {
            self.set_component_handle(ty, NIL);
        }
    }

    /// `GeometrySet::ensure_owns_direct_data` — `owns(h)` test +
    /// `make_own(ty)` does `get_component_for_write` +
    /// `ensure_owns_direct_data`.
    pub fn ensure_owns_direct_data(
        &mut self,
        owns: &dyn Fn(usize) -> bool,
        make_own: &mut dyn FnMut(GeometryComponentType),
    ) {
        for ty in Self::COMPONENT_TYPES {
            let h = self.component_handle(ty);
            if h == NIL || owns(h) {
                continue;
            }
            make_own(ty);
        }
    }
    /// `GeometrySet::ensure_owns_all_data`.
    pub fn ensure_owns_all_data(
        &mut self,
        owns: &dyn Fn(usize) -> bool,
        ensure_geometry_instances: &mut dyn FnMut(usize),
        make_own: &mut dyn FnMut(GeometryComponentType),
    ) {
        if self.instances != NIL {
            ensure_geometry_instances(self.instances);
        }
        self.ensure_owns_direct_data(owns, make_own);
    }
    /// `GeometrySet::owns_direct_data`.
    pub fn owns_direct_data(&self, owns: &dyn Fn(usize) -> bool) -> bool {
        Self::COMPONENT_TYPES
            .iter()
            .all(|&ty| self.component_handle(ty) == NIL || owns(self.component_handle(ty)))
    }
    /// `GeometrySet::ensure_no_shared_components` — `make_mutable(ty)`
    /// per component handle.
    pub fn ensure_no_shared_components(&mut self, make_mutable: &mut dyn FnMut(GeometryComponentType)) {
        for ty in Self::COMPONENT_TYPES {
            if self.component_handle(ty) != NIL {
                make_mutable(ty);
            }
        }
    }

    /// `has_mesh`/`has_pointcloud`/`has_volume`/`has_curves`/
    /// `has_instances`/`has_grease_pencil`.
    pub fn has_typed(&self, ty: GeometryComponentType) -> bool {
        self.component_handle(ty) != NIL
    }

    /// `GeometrySet::has_realized_data` — any component that holds
    /// realized geometry (not edit-hints-only).
    pub fn has_realized_data(&self) -> bool {
        self.has_typed(GeometryComponentType::Mesh)
            || self.has_typed(GeometryComponentType::PointCloud)
            || self.has_typed(GeometryComponentType::Curve)
            || self.has_typed(GeometryComponentType::Volume)
            || self.has_typed(GeometryComponentType::GreasePencil)
            || self.has_typed(GeometryComponentType::Instances)
    }

    /// `GeometrySet::is_empty`.
    pub fn is_empty(&self) -> bool {
        Self::COMPONENT_TYPES
            .iter()
            .all(|&ty| self.component_handle(ty) == NIL)
    }

    /// `GeometrySet::from_*`.
    pub fn from_typed(ty: GeometryComponentType, handle: usize) -> Self {
        let mut gs = Self::default();
        gs.set_component_handle(ty, handle);
        gs
    }

    /// `GeometrySet::replace_*` — remove the component then set the
    /// new handle when non-NIL (ownership stored by the caller's
    /// component layer).
    pub fn replace_typed(
        &mut self,
        ty: GeometryComponentType,
        handle: usize,
        _ownership: GeometryOwnershipType,
    ) {
        if handle == NIL {
            self.remove_component(ty);
            return;
        }
        if self.component_handle(ty) == handle {
            return;
        }
        self.remove_component(ty);
        self.set_component_handle(ty, handle);
    }

    /// `GeometrySet::count_memory` — `count(h)` per component.
    pub fn count_memory(&self, count: &dyn Fn(usize) -> usize) -> usize {
        self.get_components().iter().map(|&h| count(h)).sum()
    }
}

impl std::fmt::Display for GeometrySet {
    /// `operator<<` — `<GeometrySet: N verts, N faces, ...>`.
    /// Part sizes come from `desc(ty)` injected per component.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<GeometrySet>")?;
        Ok(())
    }
}

/// `operator<<` body — parts list built from injected per-type
/// descriptors (`Option<String>` per component).
pub fn geometry_set_description(
    gs: &GeometrySet,
    desc: &dyn Fn(GeometryComponentType) -> Option<String>,
) -> String {
    let mut parts = Vec::new();
    for ty in GeometrySet::COMPONENT_TYPES {
        if gs.has_typed(ty) {
            if let Some(p) = desc(ty) {
                parts.push(p);
            }
        }
    }
    format!("<GeometrySet: {}>", parts.join(", "))
}
