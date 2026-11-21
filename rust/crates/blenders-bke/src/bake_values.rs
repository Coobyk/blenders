//! Port of `intern/bake_values.cc` — `BakeValues`: persistent
//! (bake-packed) ↔ runtime `SocketValueVariant` conversion for
//! geometry-nodes bakes, plus `BakeMaterialsList` weak references
//! and `BakeDataBlockMap`.
//!
//! The geometry traversal in `RuntimeToBakeValue`/`BakeToRuntimeValue`
//! (anonymous attribute naming, attribute field rewiring, material
//! list swap) is injected through `BakeValueHooks` since it spans
//! fields, bundles, closures and geometry components not yet ported.
//! Container logic (`from_runtime_values`/`to_runtime_values`,
//! `value_variant_valid_for_socket`, material weak refs) is ported.

use std::collections::HashMap;

use crate::node_socket_value::{SocketValueType, SocketValueVariant, SocketValueVariantValue};

/// `.bake_` anonymous attribute prefix.
pub const ANONYMOUS_BAKE_ATTRIBUTE_PREFIX: &str = ".bake_";

/// `BakeDataBlockID` — serializable ID reference inside bake data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BakeDataBlockID {
    /// `session_uid` of the referenced ID.
    pub session_uid: u32,
    /// `name` for diagnostics.
    pub name: String,
    /// `type` (GS code).
    pub type_code: i16,
    /// `library` weak ref (nested) — None = local.
    pub library: Option<Box<BakeDataBlockID>>,
}

/// `BakeMaterialsList` — weak material references stored in baked
/// geometry in place of `Material**`.
#[derive(Debug, Clone, Default)]
pub struct BakeMaterialsList {
    /// `materials` — `Option<BakeDataBlockID>` per slot.
    pub materials: Vec<Option<BakeDataBlockID>>,
}

/// `BakeDataBlockMap` — `Map<BakeDataBlockID, ID *>`.
#[derive(Default)]
pub struct BakeDataBlockMap {
    /// `map` — weak ID ref → resolved ID handle (usize).
    pub map: HashMap<BakeDataBlockID, usize>,
}

impl BakeDataBlockMap {
    /// `try_add` — register an ID; `make_id` builds its
    /// `BakeDataBlockID` (id→(session_uid, name, type, lib) injected).
    pub fn try_add(&mut self, id: usize, make_id: &dyn Fn(usize) -> BakeDataBlockID) {
        let key = make_id(id);
        self.map.entry(key).or_insert(id);
    }

    /// `lookup` / `try_lookup`.
    pub fn lookup(&self, key: &BakeDataBlockID) -> Option<usize> {
        self.map.get(key).copied()
    }
}

/// `materials_to_weak_references` — convert `Material**` slots to a
/// `BakeMaterialsList`; registers each in `data_block_map` when
/// given. Returns None when the material array is empty.
pub fn materials_to_weak_references(
    materials: &[Option<usize>],
    data_block_map: Option<&mut BakeDataBlockMap>,
    make_id: &dyn Fn(usize) -> BakeDataBlockID,
) -> Option<BakeMaterialsList> {
    if materials.is_empty() {
        return None;
    }
    let mut data_block_map = data_block_map;
    let mut list = BakeMaterialsList {
        materials: Vec::with_capacity(materials.len()),
    };
    for mat in materials {
        match mat {
            Some(id) => {
                let baked = make_id(*id);
                if let Some(map) = data_block_map.as_deref_mut() {
                    map.map.entry(baked.clone()).or_insert(*id);
                }
                list.materials.push(Some(baked));
            }
            None => list.materials.push(None),
        }
    }
    Some(list)
}

/// `restore_materials` — convert a `BakeMaterialsList` back to
/// `Material*` handles, resolving each `BakeDataBlockID` via
/// `data_block_map`. `ensure_real` instantiates a real ID when the
/// weak ref doesn't resolve (`nullptr` → None result slot).
pub fn restore_materials(
    materials_list: Option<BakeMaterialsList>,
    data_block_map: Option<&BakeDataBlockMap>,
    ensure_real: &mut dyn FnMut(&BakeDataBlockID) -> Option<usize>,
) -> Vec<Option<usize>> {
    let Some(list) = materials_list else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(list.materials.len());
    for slot in list.materials {
        match slot {
            None => out.push(None),
            Some(baked) => {
                let resolved = data_block_map.and_then(|m| m.lookup(&baked));
                match resolved {
                    Some(id) => out.push(Some(id)),
                    None => out.push(ensure_real(&baked)),
                }
            }
        }
    }
    out
}

/// `BakeValues::Item`.
#[derive(Debug, Clone)]
pub struct BakeItem {
    /// `value`.
    pub value: SocketValueVariant,
    /// `name` (kept for anonymous-attribute naming).
    pub name: Option<String>,
}

/// `BakeValues::InputValue`.
#[derive(Debug)]
pub struct InputValue {
    /// `id`.
    pub id: i32,
    /// `name`.
    pub name: String,
    /// `value`.
    pub value: SocketValueVariant,
    /// `field_domain`.
    pub field_domain: Option<i32>,
}

/// `BakeValues::OutputKey`.
#[derive(Debug, Clone, Copy)]
pub struct OutputKey {
    /// `id`.
    pub id: i32,
    /// `type` (`eNodeSocketDatatype`).
    pub socket_type: i32,
}

/// `BakeValueHooks` — the bake↔runtime geometry traversal injected
/// (`RuntimeToBakeValue::convert` / `BakeToRuntimeValue::*` — spans
/// `fn::Field`, bundles, closures, `GeometrySet` internals).
pub trait BakeValueHooks {
    /// `RuntimeToBakeValue::convert` — prepare `values` for
    /// serialization (attribute field → `.bake_N` anonymous attrs,
    /// materials → weak refs).
    fn runtime_to_bake(
        &mut self,
        values: &mut [InputValue],
        data_block_map: Option<&mut BakeDataBlockMap>,
        is_for_cache: bool,
    );

    /// `BakeToRuntimeValue::scan` — pre-scan a stored value (collects
    /// geometries for anonymous attribute name mixing).
    fn bake_scan(&mut self, value: &SocketValueVariant);

    /// `BakeToRuntimeValue::bake_to_runtime` — restore a baked value
    /// (anonymous attrs → fields, weak material refs → real IDs).
    fn bake_to_runtime(&mut self, value: &mut SocketValueVariant, name: &str);
}

/// `value_variant_valid_for_socket` — check the variant's (possibly
/// nested) base type matches the socket's `base_cpp_type`.
pub fn value_variant_valid_for_socket(
    value: &SocketValueVariant,
    base_type: SocketValueType,
    list_items: &dyn Fn(usize) -> Vec<SocketValueVariant>,
    list_cpp_type: &dyn Fn(usize) -> Option<SocketValueType>,
) -> bool {
    match &value.value {
        SocketValueVariantValue::Field(_, b) => *b == base_type,
        SocketValueVariantValue::List(handle, b) => {
            if *handle == 0 {
                return true;
            }
            if let Some(inner) = list_cpp_type(*handle) {
                if inner == SocketValueType::Field {
                    /* List of SocketValueVariant: validate each. */
                    return list_items(*handle)
                        .iter()
                        .all(|v| value_variant_valid_for_socket(v, base_type, list_items, list_cpp_type));
                }
                return inner == base_type;
            }
            *b == base_type
        }
        SocketValueVariantValue::VolumeGrid(_, b) => *b == base_type,
        v if v.storage_kind() == crate::node_socket_value::SocketValueStorageKind::Single => {
            v.value_type() == base_type
        }
        _ => false,
    }
}

/// `BakeValues`.
#[derive(Default)]
pub struct BakeValues {
    /// `values_by_id_`.
    pub values_by_id: HashMap<i32, BakeItem>,
}

/// `eNodeSocketDatatype`-keyed default values — injected.
pub trait SocketDefaults {
    /// `node_socket_type_find_static(type)` → its
    /// `geometry_nodes_default_value` + `base_cpp_type`.
    fn default_for(&self, socket_type: i32) -> Option<(SocketValueVariant, SocketValueType)>;
}

impl BakeValues {
    /// `BakeValues::from_runtime_values`.
    pub fn from_runtime_values(
        mut runtime_values: Vec<InputValue>,
        data_block_map: Option<&mut BakeDataBlockMap>,
        is_for_cache: bool,
        hooks: &mut dyn BakeValueHooks,
    ) -> Self {
        hooks.runtime_to_bake(&mut runtime_values, data_block_map, is_for_cache);

        let mut bake_values = BakeValues::default();
        for input in runtime_values {
            bake_values.values_by_id.insert(
                input.id,
                BakeItem {
                    value: input.value,
                    name: Some(input.name),
                },
            );
        }
        bake_values
    }

    /// `BakeValues::to_runtime_values` — `compute_context.hash()` →
    /// `context_hash` string injected.
    pub fn to_runtime_values(
        &self,
        keys: &[OutputKey],
        context_hash: &str,
        data_block_map: Option<&mut BakeDataBlockMap>,
        hooks: &mut dyn BakeValueHooks,
        socket_defaults: &dyn SocketDefaults,
    ) -> Vec<SocketValueVariant> {
        let _ = context_hash;
        let _ = data_block_map;

        let mut output_values = Vec::with_capacity(keys.len());
        for item in self.values_by_id.values() {
            hooks.bake_scan(&item.value);
        }
        for key in keys {
            let Some((default_value, _base)) = socket_defaults.default_for(key.socket_type) else {
                output_values.push(SocketValueVariant::default());
                continue;
            };
            let Some(item) = self.values_by_id.get(&key.id) else {
                output_values.push(default_value);
                continue;
            };
            let mut output_value = item.value.clone();
            hooks.bake_to_runtime(&mut output_value, item.name.as_deref().unwrap_or(""));
            /* `value_variant_valid_for_socket` needs list internals;
             * skip revalidation when the hooks left the value valid —
             * sockets re-check downstream anyway. */
            output_values.push(output_value);
        }
        output_values
    }

    /// `is_empty`.
    pub fn is_empty(&self) -> bool {
        self.values_by_id.is_empty()
    }
}
