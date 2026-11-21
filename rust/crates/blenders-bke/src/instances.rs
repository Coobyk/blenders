//! Port of `intern/instances.cc` — `InstanceReference` +
//! `Instances` (reference handles, transforms, removal, unused-
//! reference compaction, unique ids, position accessors).
//!
//! GeometrySet/attribute-storage backing is handled-flavored; the
//! reference/unique-id/removal logic is fully ported.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use blenders_bli::index_mask::IndexMask;
use blenders_bli::rand::RandomNumberGenerator;
use blenders_math::Mat4;
use blenders_math::vector::Vec3;

/// `InstanceReference` — one instanced geometry set (or object /
/// collection bound to a geometry set on write).
#[derive(Clone)]
pub struct InstanceReference {
    /// `GeometrySet geometry_set_` (handle to component storage).
    pub geometry_set: usize,
    /// True once `ensure_owns_direct_data` ran — the geometry_set
    /// owns its geometry rather than referencing an ID.
    pub owns_direct_data: bool,
}

impl Default for InstanceReference {
    fn default() -> Self {
        Self {
            geometry_set: 0,
            owns_direct_data: true,
        }
    }
}

impl InstanceReference {
    /// `InstanceReference::ensure_owns_direct_data` — injected: makes
    /// the referenced ID-owned geometry into owned geometry.
    pub fn ensure_owns_direct_data(&mut self, ensure: &dyn Fn(usize) -> bool) {
        self.owns_direct_data = ensure(self.geometry_set);
    }

    /// `InstanceReference::owns_direct_data`.
    pub fn owns_direct_data(&self) -> bool {
        self.owns_direct_data
    }

    /// `operator==` — same geometry set pointer.
    pub fn eq(&self, other: &Self) -> bool {
        self.geometry_set == other.geometry_set
    }

    /// `hash()` — geometry set pointer hash.
    pub fn hash(&self) -> u64 {
        self.geometry_set as u64
    }
}

/// `convert_collection_to_instances` — one instance per collection
/// child + object, offset by the collection's instance_offset.
///
/// `children`: `(instance_offset, collection_handle)`;
/// `objects`: `(object_to_world, object_handle)`.
pub fn convert_collection_to_instances(
    collection_instance_offset: Vec3,
    children: &[(Vec3, usize)],
    objects: &[(Mat4, usize)],
) -> Instances {
    let instances_num = children.len() + objects.len();
    let mut instances = Instances::new(instances_num);

    for (i, (child_offset, collection)) in children.iter().enumerate() {
        let mut transform = Mat4::IDENTITY;
        transform.cols[3][0] += child_offset.x;
        transform.cols[3][1] += child_offset.y;
        transform.cols[3][2] += child_offset.z;
        transform.cols[3][0] -= collection_instance_offset.x;
        transform.cols[3][1] -= collection_instance_offset.y;
        transform.cols[3][2] -= collection_instance_offset.z;
        instances.transforms[i] = transform;
        instances.reference_handles[i] = instances.add_reference(InstanceReference {
            geometry_set: *collection,
            owns_direct_data: false,
        }) as i32;
    }

    for (i, (object_to_world, object)) in objects.iter().enumerate() {
        let mut transform = Mat4::IDENTITY;
        transform.cols[3][0] -= collection_instance_offset.x;
        transform.cols[3][1] -= collection_instance_offset.y;
        transform.cols[3][2] -= collection_instance_offset.z;
        let transform = transform * *object_to_world;
        instances.transforms[children.len() + i] = transform;
        instances.reference_handles[children.len() + i] = instances
            .add_reference(InstanceReference {
                geometry_set: *object,
                owns_direct_data: false,
            }) as i32;
    }

    instances
}

/// `bke::Instances` — the instanced geometry payload.
pub struct Instances {
    /// `attributes_` — instance attributes ("id" drives unique ids).
    pub instance_ids: Option<Vec<i32>>,
    /// `references_`.
    pub references: Vec<InstanceReference>,
    /// `instances_num_`.
    pub instances_num: usize,
    /// `.reference_handles` builtin attribute.
    pub reference_handles: Vec<i32>,
    /// `.instance_transform` builtin attribute.
    pub transforms: Vec<Mat4>,
    /// `reference_user_counts_` cache.
    reference_user_counts: Mutex<Option<Vec<i32>>>,
    /// `unique_ids_cache_`.
    unique_ids_cache: Mutex<Option<Vec<i32>>>,
}

impl Default for Instances {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Instances {
    /// `Instances::Instances(size)` — allocate attribute arrays.
    pub fn new(size: usize) -> Self {
        Self {
            instance_ids: None,
            references: Vec::new(),
            instances_num: size,
            reference_handles: vec![0; size],
            transforms: vec![Mat4::IDENTITY; size],
            reference_user_counts: Mutex::new(None),
            unique_ids_cache: Mutex::new(None),
        }
    }

    /// `Instances::resize`.
    pub fn resize(&mut self, size: usize) {
        self.instances_num = size;
        self.reference_handles.resize(size, 0);
        self.transforms.resize(size, Mat4::IDENTITY);
        if let Some(ids) = &mut self.instance_ids {
            ids.resize(size, 0);
        }
        *self.reference_user_counts.lock().unwrap() = None;
        *self.unique_ids_cache.lock().unwrap() = None;
    }

    /// `Instances::reference_handles`.
    pub fn reference_handles(&self) -> &[i32] {
        &self.reference_handles
    }

    /// `Instances::transforms`.
    pub fn transforms(&self) -> &[Mat4] {
        &self.transforms
    }

    /// `Instances::find_reference_handle` — identity match.
    pub fn find_reference_handle(&self, query: &InstanceReference) -> Option<usize> {
        self.references.iter().position(|r| r.eq(query))
    }

    /// `Instances::add_reference` — reuse an equal reference's handle.
    pub fn add_reference(&mut self, reference: InstanceReference) -> usize {
        if let Some(handle) = self.find_reference_handle(&reference) {
            return handle;
        }
        self.references.push(reference);
        self.references.len() - 1
    }

    /// `Instances::add_new_reference` — no dedupe.
    pub fn add_new_reference(&mut self, reference: InstanceReference) -> usize {
        self.references.push(reference);
        self.references.len() - 1
    }

    /// `Instances::instances_num`.
    pub fn instances_num(&self) -> usize {
        self.instances_num
    }

    /// `Instances::references_num`.
    pub fn references_num(&self) -> usize {
        self.references.len()
    }

    /// `Instances::remove` — compact per-mask (drops the masked
    /// instances; attributes follow).
    pub fn remove(&mut self, mask: &IndexMask) {
        let mut new_handles = Vec::new();
        let mut new_transforms = Vec::new();
        let mut new_ids = self.instance_ids.as_ref().map(|_| Vec::new());
        for (i, _) in self.reference_handles.iter().enumerate() {
            if !mask.iter().any(|m| m as usize == i) {
                new_handles.push(self.reference_handles[i]);
                new_transforms.push(self.transforms[i]);
                if let (Some(ids), Some(new)) = (&self.instance_ids, &mut new_ids) {
                    new.push(ids[i]);
                }
            }
        }
        self.instances_num = new_handles.len();
        self.reference_handles = new_handles;
        self.transforms = new_transforms;
        self.instance_ids = new_ids;
        *self.reference_user_counts.lock().unwrap() = None;
        *self.unique_ids_cache.lock().unwrap() = None;
    }

    /// `Instances::remove_unused_references` — compact `references_`
    /// and remap handles.
    pub fn remove_unused_references(&mut self) {
        let tot_instances = self.instances_num;
        let tot_references_before = self.references.len();

        if tot_instances == 0 {
            /* If there are no instances, no reference is needed. */
            self.references.clear();
            return;
        }
        if tot_references_before == 1 {
            /* There is only one reference and at least one instance.
             * So the only existing reference is used. Nothing to do. */
            return;
        }

        let mut usage_by_handle = vec![false; tot_references_before];
        /* Loop over all instances to see which references are used. */
        for i in 0..tot_instances {
            let handle = self.reference_handles[i];
            debug_assert!(handle >= 0 && (handle as usize) < tot_references_before);
            usage_by_handle[handle as usize] = true;
        }

        if !usage_by_handle.contains(&false) {
            /* All references are used. */
            return;
        }

        /* Create new references and a mapping for the handles. */
        let mut handle_mapping = vec![-1i32; tot_references_before];
        let mut new_references: Vec<InstanceReference> = Vec::new();
        let mut next_new_handle = 0i32;
        for old_handle in 0..tot_references_before {
            if usage_by_handle[old_handle] {
                let reference = self.references[old_handle].clone();
                if new_references.iter().all(|r| !r.eq(&reference)) {
                    new_references.push(reference);
                    handle_mapping[old_handle] = next_new_handle;
                    next_new_handle += 1;
                } else {
                    /* An equal reference was already added — reuse. */
                    handle_mapping[old_handle] = new_references
                        .iter()
                        .position(|r| r.eq(&reference))
                        .unwrap() as i32;
                }
            }
        }

        self.references = new_references;

        /* Update handles. */
        for handle in &mut self.reference_handles {
            *handle = handle_mapping[*handle as usize];
        }
        *self.reference_user_counts.lock().unwrap() = None;
    }

    /// `generate_unique_instance_ids` — keep original ids unique;
    /// collisions get seeded-RNG ids with a linear-scan fallback.
    pub fn generate_unique_instance_ids(original_ids: &[i32]) -> Vec<i32> {
        let mut unique_ids = vec![0i32; original_ids.len()];

        let mut used_unique_ids: HashSet<i32> = HashSet::with_capacity(original_ids.len());
        let mut instances_with_id_collision: Vec<usize> = Vec::new();
        for (instance_index, &original_id) in original_ids.iter().enumerate() {
            if used_unique_ids.insert(original_id) {
                /* The original id has not been used by another
                 * instance yet. */
                unique_ids[instance_index] = original_id;
            } else {
                /* Collision — resolved in a second pass; don't
                 * generate a random id inline, it might collide. */
                instances_with_id_collision.push(instance_index);
            }
        }

        let mut generator_by_original_id: HashMap<i32, RandomNumberGenerator> = HashMap::new();
        for instance_index in instances_with_id_collision {
            let original_id = original_ids[instance_index];
            let rng = generator_by_original_id
                .entry(original_id)
                .or_insert_with(|| {
                    let mut rng = RandomNumberGenerator::new(0);
                    rng.seed_random(original_id as u32);
                    rng
                });

            const MAX_ITERATION: usize = 100;
            let mut resolved = false;
            for iteration in 0..=MAX_ITERATION {
                /* Try generating random numbers until an unused one
                 * has been found. */
                let random_id = rng.get_int32();
                if used_unique_ids.insert(random_id) {
                    unique_ids[instance_index] = random_id;
                    resolved = true;
                    break;
                }
                if iteration == MAX_ITERATION {
                    /* Very unlikely — systematically find an unused
                     * id instead of relying on randomness. */
                    for generated_id in 0..i32::MAX {
                        if used_unique_ids.insert(generated_id) {
                            unique_ids[instance_index] = generated_id;
                            break;
                        }
                    }
                    resolved = true;
                    break;
                }
            }
            debug_assert!(resolved);
        }

        unique_ids
    }

    /// `Instances::reference_user_counts`.
    pub fn reference_user_counts(&self) -> Vec<i32> {
        let mut cache = self.reference_user_counts.lock().unwrap();
        if cache.is_none() {
            let mut data = vec![0i32; self.references.len()];
            for &handle in &self.reference_handles {
                if handle >= 0 && (handle as usize) < self.references.len() {
                    data[handle as usize] += 1;
                }
            }
            *cache = Some(data);
        }
        cache.clone().unwrap()
    }

    /// `Instances::unique_ids` — from the "id" attribute or the
    /// index range when absent.
    pub fn unique_ids(&self) -> Vec<i32> {
        let mut cache = self.unique_ids_cache.lock().unwrap();
        if cache.is_none() {
            let data = match &self.instance_ids {
                None => (0..self.instances_num as i32).collect(),
                Some(ids) => Self::generate_unique_instance_ids(ids),
            };
            *cache = Some(data);
        }
        cache.clone().unwrap()
    }

    /// `Instances::ensure_owns_direct_data`.
    pub fn ensure_owns_direct_data(&mut self, ensure: &dyn Fn(usize) -> bool) {
        for reference in &mut self.references {
            reference.ensure_owns_direct_data(ensure);
        }
    }

    /// `Instances::owns_direct_data`.
    pub fn owns_direct_data(&self) -> bool {
        self.references.iter().all(|r| r.owns_direct_data())
    }
}

/// `get_transform_position` / `instance_position_varray` — view the
/// instance positions as a `Vec3` span.
pub fn get_transform_position(transform: &Mat4) -> Vec3 {
    Vec3::new(
        transform.cols[3][0],
        transform.cols[3][1],
        transform.cols[3][2],
    )
}

/// `set_transform_position` — write back into the transform.
pub fn set_transform_position(transform: &mut Mat4, position: Vec3) {
    transform.cols[3][0] = position.x;
    transform.cols[3][1] = position.y;
    transform.cols[3][2] = position.z;
}

/// `instance_position_varray` — read-only positions.
pub fn instance_positions(instances: &Instances) -> Vec<Vec3> {
    instances.transforms.iter().map(get_transform_position).collect()
}
