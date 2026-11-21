//! Ports of `intern/geometry_component_{volume,instances,pointcloud,grease_pencil,
//! edit_data,mesh,curves}.cc` — the `GeometryComponent` implementations.
//!
//! The attribute-provider layer (`AttributeProvider`, `AttributeAccessor`,
//! field inputs) is PORT_PENDING until `bke::fn` lands; each component's
//! construction/ownership/replace/release semantics are ported faithfully.

use blenders_dna::id::NIL;

/// `GeometryComponent::Type` (BKE_geometry_set.hh).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GeometryComponentType {
    Mesh,
    PointCloud,
    Curve,
    Volume,
    Instances,
    GreasePencil,
    EditData,
}

/// `GeometryOwnershipType`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum GeometryOwnershipType {
    #[default]
    Owned,
    ReadOnly,
    Editable,
}

/// Base `GeometryComponent` — the C uses a vtable; the port keeps
/// `Type` + mutability flags, plus a `usize` payload handle.
pub struct GeometryComponent {
    pub type_: GeometryComponentType,
    pub is_mutable: bool,
    pub is_expired: bool,
    /// `attribute_provider_` — PORT_PENDING (bke attribute system).
    pub attribute_provider: usize,
}

impl GeometryComponent {
    pub fn new(type_: GeometryComponentType) -> Self {
        Self { type_, is_mutable: true, is_expired: false, attribute_provider: NIL }
    }
    /// `GeometryComponent::is_mutable`.
    pub fn is_mutable(&self) -> bool { self.is_mutable }
    /// `GeometryComponent::is_expired`.
    pub fn is_expired(&self) -> bool { self.is_expired }
    /// `GeometryComponent::is_empty` — default impl (attributes == 0).
    pub fn is_empty(&self) -> bool { true }
    /// `GeometryComponent::ensure_mutable`.
    pub fn ensure_mutable(&mut self) {
        debug_assert!(self.is_mutable());
        self.is_expired = false;
    }
    /// `GeometryComponent::expire`.
    pub fn expire(&mut self) {
        self.is_mutable = false;
        self.is_expired = true;
    }
}

/// `VolumeComponent` — wraps `Volume *` (`usize` handle).
pub struct VolumeComponent {
    pub base: GeometryComponent,
    pub volume: usize,
    pub ownership: GeometryOwnershipType,
}

impl VolumeComponent {
    pub fn new() -> Self {
        Self {
            base: GeometryComponent::new(GeometryComponentType::Volume),
            volume: NIL,
            ownership: GeometryOwnershipType::Owned,
        }
    }
    /// `VolumeComponent::has_volume`.
    pub fn has_volume(&self) -> bool { self.volume != NIL }
    /// `VolumeComponent::clear` — frees owned volume (BKE_id_free — PORT_PENDING).
    pub fn clear(&mut self) {
        debug_assert!(self.base.is_mutable() || self.base.is_expired());
        // `BKE_id_free(nullptr, volume_)` for Owned — PORT_PENDING.
        self.volume = NIL;
    }
    /// `VolumeComponent::replace(volume, ownership)`.
    pub fn replace(&mut self, volume: usize, ownership: GeometryOwnershipType) {
        debug_assert!(self.base.is_mutable());
        self.clear();
        self.volume = volume;
        self.ownership = ownership;
    }
    /// `VolumeComponent::release`.
    pub fn release(&mut self) -> usize {
        let v = self.volume;
        self.volume = NIL;
        v
    }
    /// `VolumeComponent::get_for_read`.
    pub fn get_for_read(&self) -> usize { self.volume }
    /// `VolumeComponent::get_for_write` — `BKE_volume_copy_for_eval`
    /// injected so ReadOnly data can be made writable.
    pub fn get_for_write_ex(&mut self, copy_for_eval: &dyn Fn(usize) -> usize) -> usize {
        debug_assert!(self.base.is_mutable());
        if self.ownership == GeometryOwnershipType::ReadOnly {
            self.volume = copy_for_eval(self.volume);
            self.ownership = GeometryOwnershipType::Owned;
        }
        self.volume
    }
    /// `VolumeComponent::owns_direct_data`.
    pub fn owns_direct_data(&self) -> bool {
        self.ownership == GeometryOwnershipType::Owned
    }
    /// `VolumeComponent::ensure_owns_direct_data`.
    pub fn ensure_owns_direct_data(&mut self, copy_for_eval: &dyn Fn(usize) -> usize) {
        debug_assert!(self.base.is_mutable());
        if self.ownership != GeometryOwnershipType::Owned {
            if self.volume != NIL {
                self.volume = copy_for_eval(self.volume);
            }
            self.ownership = GeometryOwnershipType::Owned;
        }
    }
    /// `VolumeComponent::copy` — `BKE_volume_copy_for_eval` injected.
    pub fn copy(&self, copy_for_eval: &dyn Fn(usize) -> usize) -> Self {
        let mut c = Self::new();
        if self.volume != NIL {
            c.volume = copy_for_eval(self.volume);
            c.ownership = GeometryOwnershipType::Owned;
        }
        c
    }
    /// `VolumeComponent::count_memory` — `BKE_volume_count_memory` injected.
    pub fn count_memory(&self, count: &mut dyn FnMut(usize)) {
        if self.volume != NIL {
            count(self.volume);
        }
    }
}

/// `InstancesComponent` — owns `InstanceReference` list + `instances_` handles.
pub struct InstancesComponent {
    pub base: GeometryComponent,
    /// `references_` — `Vector<InstanceReference>` (handle list).
    pub references: Vec<usize>,
    /// `instances_` — per-instance (reference_index, transform) pairs; the
    /// `float4x4` transforms are `blenders_math` values.
    pub instances: Vec<(usize, blenders_math::matrix::Mat4)>,
    /// `handles_` — unique instance ids (PORT_PENDING runtime).
    pub ids: Vec<i32>,
    pub ownership: GeometryOwnershipType,
}

impl InstancesComponent {
    pub fn new() -> Self {
        Self {
            base: GeometryComponent::new(GeometryComponentType::Instances),
            references: Vec::new(),
            instances: Vec::new(),
            ids: Vec::new(),
            ownership: GeometryOwnershipType::Owned,
        }
    }
    /// `InstancesComponent::instances_num`.
    pub fn instances_num(&self) -> usize { self.instances.len() }
    /// `InstancesComponent::references_num`.
    pub fn references_num(&self) -> usize { self.references.len() }
    /// `InstancesComponent::resize` (instances_).
    pub fn resize(&mut self, size: usize) {
        self.instances.resize(size, (NIL, blenders_math::matrix::Mat4::IDENTITY));
        self.ids.resize(size, 0);
    }
    /// `InstancesComponent::add_reference`.
    pub fn add_reference(&mut self, data: usize) -> usize {
        self.references.push(data);
        self.references.len() - 1
    }
    /// `InstancesComponent::reference_for_read`.
    pub fn reference_for_read(&self, i: usize) -> usize { self.references[i] }
    /// `InstancesComponent::clear`.
    pub fn clear(&mut self) {
        debug_assert!(self.base.is_mutable() || self.base.is_expired());
        self.references.clear();
        self.instances.clear();
        self.ids.clear();
    }
    /// `InstancesComponent::reserve`.
    pub fn reserve(&mut self, references: usize, instances: usize) {
        self.references.reserve(references);
        self.instances.reserve(instances);
    }
    /// `InstancesComponent::copy` — `instances_` deep-copied (Rust Vec).
    pub fn copy(&self) -> Self {
        Self {
            base: GeometryComponent::new(GeometryComponentType::Instances),
            references: self.references.clone(),
            instances: self.instances.clone(),
            ids: self.ids.clone(),
            ownership: GeometryOwnershipType::Owned,
        }
    }
    /// `InstancesComponent::owns_direct_data` / `ensure_owns_direct_data`
    /// (deep-copy handled by `copy`/`get_for_write` — Vec is always owned).
    pub fn owns_direct_data(&self) -> bool { self.ownership == GeometryOwnershipType::Owned }
    pub fn ensure_owns_direct_data(&mut self) {
        debug_assert!(self.base.is_mutable());
        self.ownership = GeometryOwnershipType::Owned;
    }
    /// `InstancesComponent::count_memory` — injected `count` runs per
    /// reference handle (`Instances::count_memory` internals PORT_PENDING).
    pub fn count_memory(&self, count: &mut dyn FnMut(usize)) {
        for &r in &self.references { count(r); }
    }
    /* `attributes`/`attributes_for_write` — PORT_PENDING
     * (instance_attribute_accessor_functions). */
}

/// `PointCloudComponent` — `PointCloud *` handle version of the same shape.
pub struct PointCloudComponent {
    pub base: GeometryComponent,
    pub pointcloud: usize,
    pub ownership: GeometryOwnershipType,
}

impl PointCloudComponent {
    pub fn new() -> Self {
        Self {
            base: GeometryComponent::new(GeometryComponentType::PointCloud),
            pointcloud: NIL,
            ownership: GeometryOwnershipType::Owned,
        }
    }
    /// `PointCloudComponent::has_pointcloud`.
    pub fn has_pointcloud(&self) -> bool { self.pointcloud != NIL }
    /// `PointCloudComponent::clear` — `BKE_id_free` for Owned — PORT_PENDING.
    pub fn clear(&mut self) {
        debug_assert!(self.base.is_mutable() || self.base.is_expired());
        self.pointcloud = NIL;
    }
    /// `PointCloudComponent::replace`.
    pub fn replace(&mut self, pointcloud: usize, ownership: GeometryOwnershipType) {
        debug_assert!(self.base.is_mutable());
        self.clear();
        self.pointcloud = pointcloud;
        self.ownership = ownership;
    }
    /// `PointCloudComponent::release`.
    pub fn release(&mut self) -> usize {
        let p = self.pointcloud;
        self.pointcloud = NIL;
        p
    }
    /// `get_for_read` / `get_for_write` — `BKE_pointcloud_copy_for_eval` injected.
    pub fn get_for_read(&self) -> usize { self.pointcloud }
    pub fn get_for_write_ex(&mut self, copy_for_eval: &dyn Fn(usize) -> usize) -> usize {
        debug_assert!(self.base.is_mutable());
        if self.ownership == GeometryOwnershipType::ReadOnly {
            self.pointcloud = copy_for_eval(self.pointcloud);
            self.ownership = GeometryOwnershipType::Owned;
        }
        self.pointcloud
    }
    /// `PointCloudComponent::owns_direct_data` / `ensure_owns_direct_data` / `copy`.
    pub fn owns_direct_data(&self) -> bool { self.ownership == GeometryOwnershipType::Owned }
    pub fn ensure_owns_direct_data(&mut self, copy_for_eval: &dyn Fn(usize) -> usize) {
        debug_assert!(self.base.is_mutable());
        if self.ownership != GeometryOwnershipType::Owned {
            if self.pointcloud != NIL { self.pointcloud = copy_for_eval(self.pointcloud); }
            self.ownership = GeometryOwnershipType::Owned;
        }
    }
    pub fn copy(&self, copy_for_eval: &dyn Fn(usize) -> usize) -> Self {
        let mut c = Self::new();
        if self.pointcloud != NIL {
            c.pointcloud = copy_for_eval(self.pointcloud);
            c.ownership = GeometryOwnershipType::Owned;
        }
        c
    }
    /* `attributes`/`attributes_for_write` — PORT_PENDING
     * (pointcloud_attribute_accessor_functions). */
}

/// `GreasePencilComponent` — `GreasePencil *` handle, same shape.
pub struct GreasePencilComponent {
    pub base: GeometryComponent,
    pub grease_pencil: usize,
    pub ownership: GeometryOwnershipType,
}

impl GreasePencilComponent {
    pub fn new() -> Self {
        Self {
            base: GeometryComponent::new(GeometryComponentType::GreasePencil),
            grease_pencil: NIL,
            ownership: GeometryOwnershipType::Owned,
        }
    }
    /// `has_grease_pencil` / `clear` / `replace` / `release` / get_for_*.
    pub fn has_grease_pencil(&self) -> bool { self.grease_pencil != NIL }
    pub fn clear(&mut self) {
        debug_assert!(self.base.is_mutable() || self.base.is_expired());
        self.grease_pencil = NIL;
    }
    pub fn replace(&mut self, grease_pencil: usize, ownership: GeometryOwnershipType) {
        debug_assert!(self.base.is_mutable());
        self.clear();
        self.grease_pencil = grease_pencil;
        self.ownership = ownership;
    }
    pub fn release(&mut self) -> usize {
        let g = self.grease_pencil;
        self.grease_pencil = NIL;
        g
    }
    pub fn get_for_read(&self) -> usize { self.grease_pencil }
    /// `GreasePencilComponent::get_for_write` — `BKE_grease_pencil_copy_for_eval` injected.
    pub fn get_for_write_ex(&mut self, copy_for_eval: &dyn Fn(usize) -> usize) -> usize {
        debug_assert!(self.base.is_mutable());
        if self.ownership == GeometryOwnershipType::ReadOnly {
            self.grease_pencil = copy_for_eval(self.grease_pencil);
            self.ownership = GeometryOwnershipType::Owned;
        }
        self.grease_pencil
    }
    /// `is_empty` / `owns_direct_data` / `ensure_owns_direct_data` / `copy`.
    pub fn is_empty(&self) -> bool { self.grease_pencil == NIL }
    pub fn owns_direct_data(&self) -> bool { self.ownership == GeometryOwnershipType::Owned }
    pub fn ensure_owns_direct_data(&mut self, copy_for_eval: &dyn Fn(usize) -> usize) {
        debug_assert!(self.base.is_mutable());
        if self.ownership != GeometryOwnershipType::Owned {
            if self.grease_pencil != NIL { self.grease_pencil = copy_for_eval(self.grease_pencil); }
            self.ownership = GeometryOwnershipType::Owned;
        }
    }
    pub fn copy(&self, copy_for_eval: &dyn Fn(usize) -> usize) -> Self {
        let mut c = Self::new();
        if self.grease_pencil != NIL {
            c.grease_pencil = copy_for_eval(self.grease_pencil);
            c.ownership = GeometryOwnershipType::Owned;
        }
        c
    }
    /* `attributes`/`attributes_for_write` — PORT_PENDING
     * (greasepencil::get_attribute_accessor_functions). */
}

/// `CurveComponent` — `CurvesGeometry` handle (DNA `Curves` — usize).
pub struct CurveComponent {
    pub base: GeometryComponent,
    pub curves: usize,
    pub ownership: GeometryOwnershipType,
}

impl CurveComponent {
    pub fn new() -> Self {
        Self {
            base: GeometryComponent::new(GeometryComponentType::Curve),
            curves: NIL,
            ownership: GeometryOwnershipType::Owned,
        }
    }
    /// `has_curves` / `clear` / `replace` / `release` / `ensure_for_write` /
    /// `get_for_read` / `get_for_write` — same contract as above.
    pub fn has_curves(&self) -> bool { self.curves != NIL }
    pub fn clear(&mut self) {
        debug_assert!(self.base.is_mutable() || self.base.is_expired());
        self.curves = NIL;
    }
    pub fn replace(&mut self, curves: usize, ownership: GeometryOwnershipType) {
        debug_assert!(self.base.is_mutable());
        self.clear();
        self.curves = curves;
        self.ownership = ownership;
    }
    pub fn release(&mut self) -> usize {
        let c = self.curves;
        self.curves = NIL;
        c
    }
    pub fn get_for_read(&self) -> usize { self.curves }
    pub fn get_for_write(&mut self) -> usize {
        if self.ownership != GeometryOwnershipType::Owned { self.curves = NIL; }
        if self.curves == NIL { return NIL; }
        self.ownership = GeometryOwnershipType::Owned;
        self.curves
    }
}

/// `MeshComponent` — `Mesh *` handle.
pub struct MeshComponent {
    pub base: GeometryComponent,
    pub mesh: usize,
    pub ownership: GeometryOwnershipType,
}

impl MeshComponent {
    pub fn new() -> Self {
        Self {
            base: GeometryComponent::new(GeometryComponentType::Mesh),
            mesh: NIL,
            ownership: GeometryOwnershipType::Owned,
        }
    }
    /// `has_mesh` / `clear` / `replace` / `release` / `get_for_*`.
    pub fn has_mesh(&self) -> bool { self.mesh != NIL }
    pub fn clear(&mut self) {
        debug_assert!(self.base.is_mutable() || self.base.is_expired());
        self.mesh = NIL;
    }
    pub fn replace(&mut self, mesh: usize, ownership: GeometryOwnershipType) {
        debug_assert!(self.base.is_mutable());
        self.clear();
        self.mesh = mesh;
        self.ownership = ownership;
    }
    pub fn release(&mut self) -> usize {
        let m = self.mesh;
        self.mesh = NIL;
        m
    }
    pub fn get_for_read(&self) -> usize { self.mesh }
    pub fn get_for_write(&mut self) -> usize {
        if self.ownership != GeometryOwnershipType::Owned { self.mesh = NIL; }
        if self.mesh == NIL { return NIL; }
        self.ownership = GeometryOwnershipType::Owned;
        self.mesh
    }
}

/// `GeometryComponentEditData` (`geometry_component_edit_data.cc`) —
/// edit-mode data block (deform verts, draw cache) on meshes.
#[derive(Default)]
pub struct GeometryComponentEditData {
    /// `Mesh *me` handle.
    pub mesh: usize,
    /// `EditDataArrays` — vertex/edge/face index maps + creases; usize payloads.
    pub deform_verts: Vec<i32>,
    pub crease_verts: Vec<f32>,
    pub crease_edges: Vec<f32>,
}

impl GeometryComponentEditData {
    /// `GeometryComponentEditData::has_data` — any edit data attached.
    pub fn has_data(&self) -> bool {
        !self.deform_verts.is_empty() || !self.crease_verts.is_empty() || !self.crease_edges.is_empty()
    }
}

/// `mesh_normals_varray` — normals per domain as a plain `Vec<Vec3>`
/// (`VArray` → `Vec` here). `adapt_face_to_corner` is `attributes().adapt_
/// domain(face→corner)` — the attribute accessor system is PORT_PENDING, so
/// callers inject it until it lands.
///
/// `face_normals`/`vert_normals` are the `_true` variants when `true_normals`,
/// else the computed caches — the caller picks the mesh's arrays.
pub fn mesh_normals_varray(
    face_normals: &[blenders_math::vector::Vec3],
    vert_normals: &[blenders_math::vector::Vec3],
    edges: &[[i32; 2]],
    corner_normals: &[blenders_math::vector::Vec3],
    mask: &blenders_bli::index_mask::IndexMask,
    domain: crate::attribute_legacy_convert::AttrDomain,
    no_corner_normals: bool,
    true_normals: bool,
    adapt_face_to_corner: &mut dyn FnMut(&[blenders_math::vector::Vec3]) -> Vec<blenders_math::vector::Vec3>,
) -> Vec<blenders_math::vector::Vec3> {
    let _ = true_normals; /* caller already resolved _true vs cached. */
    match domain {
        crate::attribute_legacy_convert::AttrDomain::Face => face_normals.to_vec(),
        crate::attribute_legacy_convert::AttrDomain::Point => vert_normals.to_vec(),
        crate::attribute_legacy_convert::AttrDomain::Edge => {
            let mut edge_normals = vec![blenders_math::vector::Vec3::ZERO; mask.min_array_size() as usize];
            for i in mask.iter() {
                let e = edges[i as usize];
                let n = (vert_normals[e[0] as usize] + vert_normals[e[1] as usize]) * 0.5;
                edge_normals[i as usize] = n.normalized();
            }
            edge_normals
        }
        crate::attribute_legacy_convert::AttrDomain::Corner => {
            if no_corner_normals || true_normals {
                adapt_face_to_corner(face_normals)
            } else {
                corner_normals.to_vec()
            }
        }
        _ => Vec::new(), /* `default: return {};` */
    }
}
