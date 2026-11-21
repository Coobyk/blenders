//! Port of `intern/geometry_set_instances.cc` —
//! `bke::object_get_evaluated_geometry_set` + `Instances` geometry
//! resolution (`foreach_referenced_geometry`, `ensure_geometry_instances`).
//!
//! `GeometrySet`/`Instances`/`InstanceReference` land here as the
//! first port needing them (BKE_geometry_set/instances internals
//! remain PORT_PENDING — handles via `usize`).

use std::collections::HashSet;

use blenders_dna::id::NIL;
use blenders_math::matrix::Mat4;
use blenders_math::vector::Vec3;

/// `GeometryOwnershipType` — re-exported from geometry_components.
pub use crate::geometry_components::GeometryOwnershipType;

/// `GeometrySet` — the component set as handles (mesh/pointcloud/
/// curves/greasepencil/volume/editdata + instances).
#[derive(Clone, Default)]
pub struct GeometrySet {
    /// `MeshComponent` data handle.
    pub mesh: usize,
    /// `CurveComponent`.
    pub curves: usize,
    /// `PointCloudComponent`.
    pub pointcloud: usize,
    /// `InstancesComponent` arena index.
    pub instances: usize,
    /// `VolumeComponent`.
    pub volume: usize,
    /// `GreasePencilComponent`.
    pub grease_pencil: usize,
    /// `GeometryComponentEditData`.
    pub edit_data: usize,
    /// `name_`.
    pub name: String,
}

impl GeometrySet {
    /// `has_mesh` / `has_instances` / `set_name`.
    pub fn has_mesh(&self) -> bool { self.mesh != NIL }
    pub fn has_instances(&self) -> bool { self.instances != NIL }
    pub fn set_name(&mut self, name: &str) { self.name = name.to_string(); }
    /// `replace_mesh(mesh, ownership)` — ownership recorded elsewhere;
    /// ReadOnly stored as the same handle.
    pub fn replace_mesh(&mut self, mesh: usize, _ownership: GeometryOwnershipType) {
        self.mesh = mesh;
    }
    /// `GeometrySet::from_instances`.
    pub fn from_instances(instances: usize) -> Self {
        Self { instances, ..Default::default() }
    }
    /// `get_instances_for_write`.
    pub fn get_instances_for_write(&mut self) -> usize { self.instances }
}

/// `InstanceReference::Type`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InstanceRefType {
    None,
    Object,
    Collection,
    GeometrySet,
}

/// `InstanceReference`.
#[derive(Clone)]
pub struct InstanceReference {
    /// `type_`.
    pub type_: InstanceRefType,
    /// `Object*`/`Collection*`/`GeometrySet` box — handle/owned set.
    pub handle: usize,
    /// Owned `GeometrySet` payload (Type::GeometrySet).
    pub geometry_set: Option<Box<GeometrySet>>,
    /// Direct-ness bookkeeping omitted (PORT_PENDING fields).
    pub _pad: (),
}
impl InstanceReference {
    /// `InstanceReference(object)`.
    pub fn object(handle: usize) -> Self {
        Self { type_: InstanceRefType::Object, handle, geometry_set: None, _pad: () }
    }
    /// `InstanceReference(collection)`.
    pub fn collection(handle: usize) -> Self {
        Self { type_: InstanceRefType::Collection, handle, geometry_set: None, _pad: () }
    }
    /// `InstanceReference(GeometrySet)` — `From<GeometrySet>` ctor.
    pub fn from_geometry_set(gs: GeometrySet) -> Self {
        Self { type_: InstanceRefType::GeometrySet, handle: NIL, geometry_set: Some(Box::new(gs)), _pad: () }
    }
    /// `type()`.
    pub fn type_(&self) -> InstanceRefType { self.type_ }
}

/// `Instances` — `references_` + per-instance
/// (`reference_handles`, `transforms`).
#[derive(Default)]
pub struct Instances {
    /// `references_`.
    pub references: Vec<InstanceReference>,
    /// `reference_handles` — reference idx per instance.
    pub reference_handles: Vec<i32>,
    /// `transforms` — `float4x4` per instance.
    pub transforms: Vec<Mat4>,
}

impl Instances {
    /// `Instances(size)` — pre-size instance arrays.
    pub fn with_capacity(size: usize) -> Self {
        Self {
            references: Vec::new(),
            reference_handles: vec![-1; size],
            transforms: vec![Mat4::IDENTITY; size],
        }
    }
    /// `add_reference` — dedupe by handle+type; returns ref index.
    pub fn add_reference(&mut self, reference: InstanceReference) -> i32 {
        for (i, r) in self.references.iter().enumerate() {
            if r.type_ == reference.type_ && r.handle == reference.handle {
                return i as i32;
            }
        }
        self.references.push(reference);
        (self.references.len() - 1) as i32
    }
    /// `reference_handles_for_write` / `transforms_for_write`.
    pub fn reference_handles_for_write(&mut self) -> &mut [i32] { &mut self.reference_handles }
    pub fn transforms_for_write(&mut self) -> &mut [Mat4] { &mut self.transforms }

    /// `foreach_referenced_geometry` — object/collection accessors
    /// injected: `get_object_geometry(obj)` =
    /// `object_get_evaluated_geometry_set(obj, apply_subdiv=false)`;
    /// `collection_objects(collection)` = recursive collect.
    pub fn foreach_referenced_geometry(
        &self,
        get_object_geometry: &mut dyn FnMut(usize) -> GeometrySet,
        collection_objects: &mut dyn FnMut(usize) -> Vec<usize>,
        callback: &mut dyn FnMut(&GeometrySet),
    ) {
        for reference in &self.references {
            match reference.type_ {
                InstanceRefType::Object => {
                    let gs = get_object_geometry(reference.handle);
                    callback(&gs);
                }
                InstanceRefType::Collection => {
                    for obj in collection_objects(reference.handle) {
                        let gs = get_object_geometry(obj);
                        callback(&gs);
                    }
                }
                InstanceRefType::GeometrySet => {
                    if let Some(gs) = &reference.geometry_set {
                        callback(gs);
                    }
                }
                InstanceRefType::None => {}
            }
        }
    }

    /// `ensure_geometry_instances` — `instances_arena` holds nested
    /// `Instances` created for collections; `object_type_of`,
    /// `object_to_world_of`, `collection_instance_offset_of`,
    /// `id_name_of` are the object accessors (OB_LAMP/CAMERA/SPEAKER/
    /// ARMATURE passthrough verbatim).
    #[allow(clippy::too_many_arguments)]
    pub fn ensure_geometry_instances(
        &mut self,
        get_object_geometry: &mut dyn FnMut(usize) -> GeometrySet,
        collection_objects: &mut dyn FnMut(usize) -> Vec<usize>,
        object_type_of: &dyn Fn(usize) -> i16,
        object_to_world_of: &dyn Fn(usize) -> Mat4,
        collection_instance_offset_of: &dyn Fn(usize) -> Vec3,
        id_name_of: &dyn Fn(usize) -> String,
        instances_arena: &mut Vec<Instances>,
    ) {
        /* OB_LAMP/OB_CAMERA/OB_SPEAKER/OB_ARMATURE — DNA_object_enums. */
        const OB_LAMP: i16 = 10;
        const OB_CAMERA: i16 = 11;
        const OB_SPEAKER: i16 = 12;
        const OB_ARMATURE: i16 = 13;
        let mut new_references: Vec<InstanceReference> = Vec::with_capacity(self.references.len());
        for reference in std::mem::take(&mut self.references) {
            match reference.type_ {
                InstanceRefType::None => {
                    new_references.push(InstanceReference::from_geometry_set(GeometrySet::default()));
                }
                InstanceRefType::GeometrySet => {
                    new_references.push(reference);
                }
                InstanceRefType::Object => {
                    let obj = reference.handle;
                    let ty = object_type_of(obj);
                    if ty == OB_LAMP || ty == OB_CAMERA || ty == OB_SPEAKER || ty == OB_ARMATURE {
                        new_references.push(InstanceReference::object(obj));
                        continue;
                    }
                    let mut object_geometry_set = get_object_geometry(obj);
                    object_geometry_set.set_name(&id_name_of(obj));
                    if object_geometry_set.has_instances() {
                        /* Recurse into the nested Instances. */
                        let inst_i = object_geometry_set.instances;
                        if inst_i != NIL && inst_i < instances_arena.len() {
                            let mut inner = std::mem::take(&mut instances_arena[inst_i]);
                            inner.ensure_geometry_instances(
                                get_object_geometry,
                                collection_objects,
                                object_type_of,
                                object_to_world_of,
                                collection_instance_offset_of,
                                id_name_of,
                                instances_arena,
                            );
                            instances_arena[inst_i] = inner;
                        }
                    }
                    new_references.push(InstanceReference::from_geometry_set(object_geometry_set));
                }
                InstanceRefType::Collection => {
                    let collection = reference.handle;
                    let objects = collection_objects(collection);
                    let mut instances = Instances::with_capacity(objects.len());
                    for (i, &obj) in objects.iter().enumerate() {
                        instances.reference_handles[i] =
                            instances.add_reference(InstanceReference::object(obj));
                        let mut t = object_to_world_of(obj);
                        /* `transforms[i].location() -= collection.instance_offset`. */
                        let off = collection_instance_offset_of(collection);
                        t[3].x -= off.x;
                        t[3].y -= off.y;
                        t[3].z -= off.z;
                        instances.transforms[i] = t;
                    }
                    instances.ensure_geometry_instances(
                        get_object_geometry,
                        collection_objects,
                        object_type_of,
                        object_to_world_of,
                        collection_instance_offset_of,
                        id_name_of,
                        instances_arena,
                    );
                    let idx = instances_arena.len();
                    instances_arena.push(instances);
                    let mut geometry_set = GeometrySet::from_instances(idx);
                    geometry_set.set_name(&id_name_of(collection));
                    new_references.push(InstanceReference::from_geometry_set(geometry_set));
                }
            }
        }
        self.references = new_references;
    }
}

/// `add_final_mesh_as_geometry_component` — the two mesh getters are
/// injected (`evaluated mesh` vs `evaluated mesh, no subsurf`).
fn add_final_mesh_as_geometry_component(
    object: usize,
    geometry_set: &mut GeometrySet,
    apply_subdiv: bool,
    evaluated_mesh: &mut dyn FnMut(usize) -> usize,
    evaluated_mesh_no_subsurf: &mut dyn FnMut(usize) -> usize,
    ensure_mdata: &mut dyn FnMut(usize),
) {
    if apply_subdiv {
        let mesh = evaluated_mesh(object);
        if mesh != NIL {
            ensure_mdata(mesh); /* `BKE_mesh_wrapper_ensure_mdata`. */
            geometry_set.replace_mesh(mesh, GeometryOwnershipType::ReadOnly);
        }
        return;
    }
    let mesh = evaluated_mesh_no_subsurf(object);
    if mesh != NIL {
        geometry_set.replace_mesh(mesh, GeometryOwnershipType::ReadOnly);
    }
}

/// `object_get_evaluated_geometry_set` — `OB_MESH`/`OB_EMPTY` +
/// `instance_collection` branch order verbatim. `Object` fields come
/// from `view`.
#[allow(clippy::too_many_arguments)]
pub fn object_get_evaluated_geometry_set(
    object: usize,
    apply_subdiv: bool,
    object_geometry_is_evaluated: &dyn Fn(usize) -> bool,
    geometry_set_eval_of: &dyn Fn(usize) -> Option<GeometrySet>,
    object_type_of: &dyn Fn(usize) -> i16,
    instance_collection_of: &dyn Fn(usize) -> usize,
    evaluated_mesh: &mut dyn FnMut(usize) -> usize,
    evaluated_mesh_no_subsurf: &mut dyn FnMut(usize) -> usize,
    ensure_mdata: &mut dyn FnMut(usize),
    instances_arena: &mut Vec<Instances>,
) -> Option<GeometrySet> {
    const OB_MESH: i16 = 1;
    const OB_EMPTY: i16 = 0;
    if !object_geometry_is_evaluated(object) {
        return None;
    }
    if let Some(mut geometry_set) = geometry_set_eval_of(object) {
        if geometry_set.has_mesh() {
            add_final_mesh_as_geometry_component(
                object,
                &mut geometry_set,
                apply_subdiv,
                evaluated_mesh,
                evaluated_mesh_no_subsurf,
                ensure_mdata,
            );
        }
        return Some(geometry_set);
    }
    if object_type_of(object) == OB_MESH {
        let mut geometry_set = GeometrySet::default();
        add_final_mesh_as_geometry_component(
            object,
            &mut geometry_set,
            apply_subdiv,
            evaluated_mesh,
            evaluated_mesh_no_subsurf,
            ensure_mdata,
        );
        return Some(geometry_set);
    }
    if object_type_of(object) == OB_EMPTY {
        let collection = instance_collection_of(object);
        if collection != NIL {
            let mut instances = Instances::with_capacity(1);
            instances.reference_handles[0] =
                instances.add_reference(InstanceReference::collection(collection));
            instances.transforms[0] = Mat4::IDENTITY;
            instances_arena.push(instances);
            return Some(GeometrySet::from_instances(instances_arena.len() - 1));
        }
    }
    None
}
