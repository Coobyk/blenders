//! Port of `intern/builder/deg_builder_relations_drivers.cc`
/// — `DriverDescriptor` + driver serialization relations.

use std::collections::{HashMap, HashSet, VecDeque};

/// `PROP_BOOLEAN`/`PROP_ENUM` (`RNA_types.h` verbatim).
pub const PROP_BOOLEAN: i32 = 2;
pub const PROP_ENUM: i32 = 3;

/// `DriverDescriptor` (verbatim).
pub struct DriverDescriptor {
    /// `id_ptr_->owner_id` handle.
    pub owner_id: usize,
    /// `fcu_->rna_path()`.
    pub rna_path: String,
    /// `fcu_->array_index`.
    pub array_index: i32,
    /// `driver_relations_needed_`.
    pub driver_relations_needed: bool,
    /// `is_array_`.
    pub is_array: bool,
    /// `rna_prefix` (path minus last segment).
    pub rna_prefix: String,
    /// `rna_suffix` (last segment).
    pub rna_suffix: String,
}

impl DriverDescriptor {
    /// Constructor (verbatim — determines relations-needed
    /// then splits the path).
    pub fn new(
        owner_id: usize,
        rna_path: &str,
        array_index: i32,
        rna_resolves: Option<bool>,
        property_is_array: bool,
        property_type: i32,
    ) -> Self {
        let mut d = DriverDescriptor {
            owner_id,
            rna_path: rna_path.to_string(),
            array_index,
            driver_relations_needed: false,
            is_array: false,
            rna_prefix: String::new(),
            rna_suffix: String::new(),
        };
        /* `determine_relations_needed` (verbatim). */
        if d.array_index > 0 {
            d.is_array = true;
            d.driver_relations_needed = true;
        } else if rna_resolves != Some(true) {
            d.driver_relations_needed = false;
        } else if property_is_array {
            d.is_array = true;
            d.driver_relations_needed = true;
        } else {
            d.driver_relations_needed =
                matches!(property_type, PROP_BOOLEAN | PROP_ENUM);
        }
        d.split_rna_path();
        d
    }

    /// `split_rna_path` (verbatim — last-dot split).
    pub fn split_rna_path(&mut self) {
        match self.rna_path.rfind('.') {
            Some(i) if i + 1 < self.rna_path.len() => {
                self.rna_prefix = self.rna_path[..i].to_string();
                self.rna_suffix = self.rna_path[i + 1..].to_string();
            }
            _ => {
                self.rna_prefix = String::new();
                self.rna_suffix = self.rna_path.clone();
            }
        }
    }

    /// `is_same_array_as` (verbatim).
    pub fn is_same_array_as(&self, other: &DriverDescriptor) -> bool {
        self.is_array && other.is_array && self.rna_suffix == other.rna_suffix
    }
}

/// `is_reachable` (verbatim — backward BFS over inlinks).
pub fn is_reachable(
    from: usize,
    to: usize,
    inlinks: &dyn Fn(usize) -> Vec<usize>,
    rel_from: &dyn Fn(usize) -> usize,
) -> bool {
    if from == to {
        return true;
    }
    let mut queue = VecDeque::from([to]);
    let mut seen = HashSet::from([to]);
    while let Some(visit) = queue.pop_back() {
        if visit == from {
            return true;
        }
        for rel in inlinks(visit) {
            let prev = rel_from(rel);
            if seen.insert(prev) {
                queue.push_back(prev);
            }
        }
    }
    false
}

/// `build_driver_relations(IDNode*)` (verbatim — group by
/// RNA prefix, serialize same-prefix drivers).
pub fn build_driver_relations(
    drivers: Vec<DriverDescriptor>,
    get_node: &dyn Fn(&DriverDescriptor) -> Option<usize>,
    get_exit_operation: &dyn Fn(usize) -> usize,
    get_entry_operation: &dyn Fn(usize) -> usize,
    inlinks: &dyn Fn(usize) -> Vec<usize>,
    rel_from: &dyn Fn(usize) -> usize,
    mut add_operation_relation: impl FnMut(usize, usize, &str),
) {
    let mut driver_groups: HashMap<String, Vec<DriverDescriptor>> = HashMap::new();
    for d in drivers {
        if d.rna_path.is_empty() || !d.driver_relations_needed {
            continue;
        }
        driver_groups.entry(d.rna_prefix.clone()).or_default().push(d);
    }
    for prefix_group in driver_groups.values() {
        let num_drivers = prefix_group.len();
        if num_drivers < 2 {
            continue;
        }
        for from_index in 0..num_drivers {
            let driver_from = &prefix_group[from_index];
            let Some(op_from) = get_node(driver_from) else { continue };
            for to_offset in 1..num_drivers {
                let to_index = (from_index + to_offset) % num_drivers;
                let driver_to = &prefix_group[to_index];
                let Some(op_to) = get_node(driver_to) else { continue };
                if op_from == op_to {
                    continue;
                }
                if from_index < to_index && driver_from.is_same_array_as(driver_to) {
                    /* `color[0]` -> `color[1]` — always add. */
                } else {
                    if is_reachable(op_to, op_from, inlinks, rel_from) {
                        continue;
                    }
                    if is_reachable(op_from, op_to, inlinks, rel_from) {
                        break;
                    }
                }
                add_operation_relation(
                    get_exit_operation(op_from),
                    get_entry_operation(op_to),
                    "Driver Serialization",
                );
                break;
            }
        }
    }
}

/// `data_path_maybe_shared` (verbatim — conservative
/// non-shared check; OB pose.bones + KE key_blocks are
/// thread-safe).
pub fn data_path_maybe_shared(
    id_gs: u16,
    data_path: &str,
    object_type_is_armature: bool,
) -> bool {
    if data_path.starts_with("[\"") && data_path.ends_with("\"]") {
        return false;
    }
    if id_gs == blenders_dna::id::gs(b"OB") {
        /* `ob.type == OB_ARMATURE && path.startswith("pose.bones[")`. */
        let is_thread_safe =
            object_type_is_armature && data_path.starts_with("pose.bones[");
        return !is_thread_safe;
    }
    if id_gs == blenders_dna::id::gs(b"KE") {
        let is_thread_safe =
            data_path.starts_with("key_blocks[") && data_path.ends_with("].value");
        return !is_thread_safe;
    }
    true
}
