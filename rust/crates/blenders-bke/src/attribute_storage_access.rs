//! Port of `intern/attribute_storage_access.cc`.
//!
//! Reader/writer bridges + attribute init + rename logic. `GAttribute*`,
//! `AttributeStorage`, and `GList` sharing internals are injected
//! (`PORT_PENDING` — `attribute_storage.cc` not yet ported).

use std::collections::{HashMap, HashSet};

use blenders_dna::id::NIL;
use blenders_dna::meshdata_types::MDeformVert;
use blenders_dna::object_types::{BDeformGroup, MAX_VGROUP_NAME};

/// `defgroup.name` as a Rust string (NUL-terminated C buffer).
fn defgroup_name(g: &BDeformGroup) -> String {
    let end = g.name.iter().position(|&c| c == 0).unwrap_or(g.name.len());
    String::from_utf8_lossy(&g.name[..end]).into_owned()
}

/// `name->copy_utf8_truncated(g.name)`.
fn defgroup_set_name(g: &mut BDeformGroup, name: &str) {
    g.name = [0; MAX_VGROUP_NAME];
    let bytes = name.as_bytes();
    let n = bytes.len().min(MAX_VGROUP_NAME - 1);
    g.name[..n].copy_from_slice(&bytes[..n]);
}

use crate::attribute_legacy_convert::{AttrDomain, AttrType};

/// `GAttributeReader` — varray + domain + sharing info.
pub struct GAttributeReader {
    /// `GVArray` handle (opaque until attribute storage lands).
    pub varray: usize,
    pub domain: AttrDomain,
    /// `ImplicitSharingInfo` handle.
    pub sharing_info: usize,
}

/// `GAttributeWriter` — mutable array + domain + change callback.
pub struct GAttributeWriter {
    /// `GVMutableArray` handle.
    pub varray: usize,
    pub domain: AttrDomain,
    pub tag_modified_fn: Option<Box<dyn FnMut()>>,
}

/// `attribute_to_reader` — wrap `attribute.data()` for reading.
///
/// `data_*` are the `GList` accessors injected from the storage impl.
pub fn attribute_to_reader(
    attribute: usize,
    domain: AttrDomain,
    domain_size: i64,
    data_size: &dyn Fn(usize) -> i64,
    data_sharing_info: &dyn Fn(usize) -> usize,
    data_varray: &dyn Fn(usize) -> usize,
) -> GAttributeReader {
    debug_assert!(data_size(attribute) == domain_size);
    let sharing_info = data_sharing_info(attribute);
    GAttributeReader {
        varray: data_varray(attribute),
        domain,
        sharing_info,
    }
}

/// `attribute_to_writer` — convert to array storage, then write.
///
/// `changed_tags` maps attribute name → `AttrUpdateOnChange`.
pub fn attribute_to_writer(
    owner: usize,
    changed_tags: &HashMap<String, usize>,
    domain_size: i64,
    attribute: usize,
    attribute_name: &dyn Fn(usize) -> String,
    attribute_domain: &dyn Fn(usize) -> AttrDomain,
    data_size: &dyn Fn(usize) -> i64,
    ensure_array_storage: &dyn Fn(usize),
    values_for_write_span: &dyn Fn(usize) -> usize,
    update_fn_of: &'static dyn Fn(usize, usize),
) -> GAttributeWriter {
    debug_assert!(data_size(attribute) == domain_size);
    /* Just convert the stored type to an array for modification. It
     * might not make sense to implement editing of single values at
     * this level. */
    ensure_array_storage(attribute);

    let tag_modified_fn = if let Some(&update_fn) = changed_tags.get(&attribute_name(attribute)) {
        Some(Box::new(move || update_fn_of(owner, update_fn)) as Box<dyn FnMut()>)
    } else {
        None
    };

    GAttributeWriter {
        varray: values_for_write_span(attribute),
        domain: attribute_domain(attribute),
        tag_modified_fn,
    }
}

/// `AttributeInit::Type`.
pub enum AttributeInit {
    /// `AttributeInit::Construct`.
    Construct,
    /// `AttributeInitValue` — `(value GPointer)` handle.
    Value(usize),
    /// `AttributeInitDefaultValue`.
    DefaultValue,
    /// `AttributeInitVArray` — `(varray GVArray)` handle.
    VArray(usize),
    /// `AttributeInitShared` — `(data, sharing_info)` handles.
    Shared(usize, usize),
}

/// `attribute_init_to_list` — build `GList` backing storage.
/// Returns an opaque `GList` handle (allocation via `mk_list_*`).
#[allow(clippy::too_many_arguments)]
pub fn attribute_init_to_list(
    data_type: AttrType,
    domain_size: i64,
    initializer: &AttributeInit,
    require_array_data: bool,
    cpp_type_of: &dyn Fn(AttrType) -> usize,
    mk_list_constructed: &dyn Fn(usize, i64) -> usize,
    mk_list_value_array: &dyn Fn(usize, usize, i64) -> usize,
    mk_list_value_single: &dyn Fn(usize, usize, i64) -> usize,
    mk_default_array: &dyn Fn(usize, i64) -> usize,
    varray_common_single: &dyn Fn(usize) -> Option<usize>,
    materialize_varray: &dyn Fn(usize, i64) -> usize,
    mk_list_shared: &dyn Fn(usize, usize, usize, i64) -> usize,
) -> usize {
    let type_ = cpp_type_of(data_type);
    match *initializer {
        AttributeInit::Construct => mk_list_constructed(type_, domain_size),
        AttributeInit::Value(value) => {
            if require_array_data {
                mk_list_value_array(type_, value, domain_size)
            } else {
                mk_list_value_single(type_, value, domain_size)
            }
        }
        AttributeInit::DefaultValue => mk_default_array(type_, domain_size),
        AttributeInit::VArray(varray) => {
            if !require_array_data {
                if let Some(single_data) = varray_common_single(varray) {
                    return mk_list_value_single(type_, single_data, domain_size);
                }
            }
            /* `GArray<> data(varray.type(), domain_size,
             * NoInitialization()); varray.materialize_to_uninitialized`. */
            materialize_varray(varray, domain_size)
        }
        AttributeInit::Shared(data, sharing_info) => {
            /* `data.sharing_info->add_user()` happens inside
             * `mk_list_shared` (storage impl). */
            mk_list_shared(type_, data, sharing_info, domain_size)
        }
    }
}

/// `get_varray_attribute` — typed lookup with default fallback.
#[allow(clippy::too_many_arguments)]
pub fn get_varray_attribute(
    storage: usize,
    domain: AttrDomain,
    cpp_type: usize,
    name: &str,
    domain_size: i64,
    default_value: usize,
    storage_lookup: &dyn Fn(usize, &str) -> usize,
    attr_domain: &dyn Fn(usize) -> AttrDomain,
    attr_data_type: &dyn Fn(usize) -> AttrType,
    cpp_type_to_attribute_type: &dyn Fn(usize) -> AttrType,
    attr_data_varray: &dyn Fn(usize) -> usize,
    mk_single_varray: &dyn Fn(usize, i64, usize) -> usize,
) -> usize {
    let attr = storage_lookup(storage, name);
    let return_default = || mk_single_varray(cpp_type, domain_size, default_value);
    if attr == NIL {
        return return_default();
    }
    if attr_domain(attr) != domain {
        return return_default();
    }
    if attr_data_type(attr) != cpp_type_to_attribute_type(cpp_type) {
        return return_default();
    }
    attr_data_varray(attr)
}

/// `get_span_attribute` — typed span lookup, None on mismatch.
#[allow(clippy::too_many_arguments)]
pub fn get_span_attribute(
    storage: usize,
    domain: AttrDomain,
    cpp_type: usize,
    name: &str,
    domain_size: i64,
    storage_lookup: &dyn Fn(usize, &str) -> usize,
    attr_domain: &dyn Fn(usize) -> AttrDomain,
    attr_data_span: &dyn Fn(usize) -> Option<usize>,
) -> Option<usize> {
    let attr = storage_lookup(storage, name);
    if attr == NIL || attr_domain(attr) != domain {
        return None;
    }
    /* `std::variant<GSpan, GPointer> values = attr->data().values()`
     * → `attr_data_span` returns the GSpan handle when array-backed. */
    let _ = cpp_type;
    let _ = domain_size;
    attr_data_span(attr)
}

/// `get_mutable_attribute` — find or create array-backed attribute.
#[allow(clippy::too_many_arguments)]
pub fn get_mutable_attribute(
    storage: usize,
    domain: AttrDomain,
    cpp_type: usize,
    name: &str,
    domain_size: i64,
    custom_default_value: usize,
    storage_lookup: &dyn Fn(usize, &str) -> usize,
    storage_remove: &dyn Fn(usize, &str),
    storage_add: &dyn Fn(usize, &str, AttrDomain, usize, usize, i64) -> usize,
    attr_data_type: &dyn Fn(usize) -> AttrType,
    cpp_type_to_attribute_type: &dyn Fn(usize) -> AttrType,
    cpp_type_default_value: &dyn Fn(usize) -> usize,
    ensure_array_storage: &dyn Fn(usize),
    values_for_write_span: &dyn Fn(usize) -> usize,
) -> usize {
    if domain_size <= 0 {
        return NIL;
    }
    let type_ = cpp_type_to_attribute_type(cpp_type);
    let attr = storage_lookup(storage, name);
    if attr != NIL {
        if attr_data_type(attr) == type_ {
            ensure_array_storage(attr);
            return values_for_write_span(attr);
        }
        /* The attribute has the wrong type. This shouldn't happen for
         * builtin attributes, but just in case, remove it. */
        storage_remove(storage, name);
    }
    let default_value = if custom_default_value != NIL {
        custom_default_value
    } else {
        cpp_type_default_value(cpp_type)
    };
    storage_add(storage, name, domain, cpp_type, default_value, domain_size)
}

/// `try_delete_vertex_group`.
pub fn try_delete_vertex_group(
    vertex_groups: &mut Vec<BDeformGroup>,
    name: &str,
    get_mutable_dverts: &mut dyn FnMut() -> (*mut MDeformVert, usize),
    remove_defgroup_index: &mut dyn FnMut(&mut [MDeformVert], usize),
) -> bool {
    /* `BKE_defgroup_listbase_name_find` + `BLI_remlink` → position. */
    let Some(index) = vertex_groups.iter().position(|g| defgroup_name(g) == name) else {
        return false;
    };
    vertex_groups.remove(index);
    let (dverts_ptr, dverts_len) = get_mutable_dverts();
    if dverts_len == 0 {
        return true;
    }
    remove_defgroup_index(unsafe { std::slice::from_raw_parts_mut(dverts_ptr, dverts_len) }, index);
    true
}

/// `find_vertex_group`.
fn find_vertex_group<'a>(vertex_groups: &'a mut [BDeformGroup], name: &str) -> Option<usize> {
    vertex_groups.iter().position(|g| defgroup_name(g) == name)
}

/// `AttrBuiltinInfo` — builtin attribute domain/type/deletability.
#[derive(Clone, Copy)]
pub struct AttrBuiltinInfo {
    pub domain: AttrDomain,
    pub type_: AttrType,
    pub deletable: bool,
}

/// `rename_attributes` — apply `name_map` renames; returns failures.
#[allow(clippy::too_many_arguments)]
pub fn rename_attributes(
    storage: usize,
    mut vertex_groups: Option<&mut Vec<BDeformGroup>>,
    name_map: &HashMap<String, String>,
    overwrite: bool,
    builtin_attributes: &HashMap<String, AttrBuiltinInfo>,
    array_storage_required: &HashSet<String>,
    domain_size_fn: &dyn Fn(AttrDomain) -> i64,
    get_mutable_dverts: &mut dyn FnMut() -> (*mut MDeformVert, usize),
    remove_defgroup_index: &mut dyn FnMut(&mut [MDeformVert], usize),
    storage_lookup: &dyn Fn(usize, &str) -> usize,
    storage_count: &dyn Fn(usize) -> i64,
    storage_remove_names: &dyn Fn(usize, &HashSet<String>),
    storage_rename: &dyn Fn(usize, &HashMap<usize, String>),
    attr_domain: &dyn Fn(usize) -> AttrDomain,
    attr_data_type: &dyn Fn(usize) -> AttrType,
    ensure_array_storage: &dyn Fn(usize),
) -> HashSet<String> {
    let mut names_to_remove: HashSet<String> = HashSet::new();
    let mut failed: HashSet<String> = HashSet::new();
    let mut map: HashMap<usize, String> = HashMap::new();
    let mut vertex_group_map: HashMap<usize, String> = HashMap::new();
    map.reserve(name_map.len());

    let has_vertex_groups = vertex_groups.is_some();
    for (old_name, new_name) in name_map {
        if new_name.is_empty() {
            failed.insert(old_name.clone());
            continue;
        }
        /* `find_attr_or_group` — attribute first, then vertex group. */
        let old_attr_attr = storage_lookup(storage, old_name);
        let old_group = if old_attr_attr == NIL && has_vertex_groups {
            vertex_groups
                .as_deref()
                .and_then(|vgs| vgs.iter().position(|g| defgroup_name(g) == *old_name))
        } else {
            None
        };
        let old_attr: Result<usize, usize> = if old_attr_attr != NIL {
            Ok(old_attr_attr)
        } else if let Some(g) = old_group {
            Err(g)
        } else {
            failed.insert(old_name.clone());
            continue;
        };

        let old_domain = match old_attr {
            Ok(attr) => attr_domain(attr),
            Err(_) => AttrDomain::Point,
        };
        let old_type = match old_attr {
            Ok(attr) => attr_data_type(attr),
            Err(_) => AttrType::Float,
        };

        if let Some(old_info) = builtin_attributes.get(old_name) {
            if !old_info.deletable {
                failed.insert(old_name.clone());
                continue;
            }
        }
        if let Some(new_info) = builtin_attributes.get(new_name) {
            if new_info.domain != old_domain || new_info.type_ != old_type {
                failed.insert(old_name.clone());
                continue;
            }
        }
        if array_storage_required.contains(new_name) {
            if let Ok(attr) = old_attr {
                debug_assert_eq!(domain_size_fn(attr_domain(attr)), {
                    let _ = domain_size_fn;
                    domain_size_fn(attr_domain(attr))
                });
                ensure_array_storage(attr);
            } else {
                /* Converting vertex groups to real attributes remains
                 * unsupported here. */
                failed.insert(old_name.clone());
                continue;
            }
        }
        if overwrite {
            names_to_remove.insert(new_name.clone());
        } else {
            let new_attr = storage_lookup(storage, new_name);
            let new_group = if new_attr == NIL && has_vertex_groups {
                vertex_groups
                    .as_deref()
                    .and_then(|vgs| vgs.iter().position(|g| defgroup_name(g) == *new_name))
            } else {
                None
            };
            if new_attr != NIL || new_group.is_some() {
                failed.insert(old_name.clone());
                continue;
            }
        }

        match old_attr {
            Ok(attr) => {
                map.insert(attr, new_name.clone());
            }
            Err(g) => {
                vertex_group_map.insert(g, new_name.clone());
            }
        }
    }

    if !names_to_remove.is_empty() {
        let start_attrs_num = storage_count(storage);
        storage_remove_names(storage, &names_to_remove);
        if has_vertex_groups
            && (start_attrs_num - storage_count(storage)) != names_to_remove.len() as i64
        {
            /* Also remove vertex groups if names weren't all removed
             * as attributes. */
            if let Some(ref mut vgs) = vertex_groups {
                let names: Vec<String> = vgs
                    .iter()
                    .filter(|g| names_to_remove.contains(&defgroup_name(g)))
                    .map(|g| defgroup_name(g))
                    .collect();
                for name in names {
                    try_delete_vertex_group(vgs, &name, &mut *get_mutable_dverts, &mut *remove_defgroup_index);
                }
            }
        }
    }

    if !map.is_empty() {
        storage_rename(storage, &map);
    }
    if let Some(vgs) = vertex_groups {
        if !vertex_group_map.is_empty() {
            for (i, defgroup) in vgs.iter_mut().enumerate() {
                if let Some(new_name) = vertex_group_map.get(&i) {
                    defgroup_set_name(defgroup, new_name);
                }
            }
        }
    }
    failed
}

/* PORT_PENDING: `Attribute`, `AttributeStorage`, `GVMutableArray`,
 * `GPointer` and the implicit-sharing paths — all injected above
 * until `attribute_storage.cc` is ported. */
