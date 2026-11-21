//! Port of `intern/geometry_fields.cc` — `GeometryFieldContext`
//! attribute lookup, `component_is_available`,
//! `get_random_id_attribute_name`, `try_capture_fields_on_geometry`
//! (shared-attribute fast path, selection handling, evaluate+store/
//! add result passes), `try_detect_native_field_domain`/
//! `try_detect_field_domain`.
//! `PORT_PENDING`: the `fn::GField`/`FieldInput`/`FieldEvaluator`
//! framework itself (lives in `source/blender/functions`), field
//! evaluation calls into it via injected handles; `SampleIndexFunction`
//! multi-function execution.

use crate::attribute_legacy_convert::{AttrDomain, AttrType};

use crate::geometry_components::GeometryComponentType;

/// `get_random_id_attribute_name` — `"id"` on Point/Instance.
pub fn get_random_id_attribute_name(domain: AttrDomain) -> &'static str {
    match domain {
        AttrDomain::Point | AttrDomain::Instance => "id",
        _ => "",
    }
}

/// `component_is_available` — has component + nonzero domain size.
pub fn component_is_available(
    has_component: bool,
    attribute_domain_size: i32,
) -> bool {
    has_component && attribute_domain_size != 0
}

/// `SUPPORTED_TYPES` — `SampleIndexFunction::find_source_component`
/// scan order (spreadsheet / ray-cast order).
pub const SAMPLE_INDEX_SUPPORTED_TYPES: [GeometryComponentType; 5] = [
    GeometryComponentType::Mesh,
    GeometryComponentType::PointCloud,
    GeometryComponentType::Curve,
    GeometryComponentType::Instances,
    GeometryComponentType::GreasePencil,
];

/// `SampleIndexFunction::find_source_component` — first available
/// component type in `SAMPLE_INDEX_SUPPORTED_TYPES` with data on
/// `domain`.
pub fn sample_index_find_source_component(
    domain: AttrDomain,
    component_available: &dyn Fn(GeometryComponentType, AttrDomain) -> bool,
) -> Option<GeometryComponentType> {
    for src_type in SAMPLE_INDEX_SUPPORTED_TYPES {
        if component_available(src_type, domain) {
            return Some(src_type);
        }
    }
    None
}

/// `AttributeDomainAndType` — `(domain, data_type)` pair.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct AttributeDomainAndType {
    /// `domain`.
    pub domain: AttrDomain,
    /// `data_type`.
    pub data_type: AttrType,
}

/// `try_capture_fields_on_geometry` — the store/add two-pass
/// algorithm over a field set. Fields/evaluator are injected:
/// `evaluate_field(i, mask) -> Vec<u8>` (type-erased value buffer,
/// one element per mask entry), `is_constant(i) -> Option<Vec<u8>>`
/// (single-element buffer when the field doesn't depend on input).
///
/// `attributes` is an injected mutable accessor so the missing
/// `bke::MutableAttributeAccessor` machinery stays external.
#[allow(clippy::too_many_arguments)]
pub fn try_capture_fields_on_geometry(
    domain_size: usize,
    names: &[String],
    domain: AttrDomain,
    field_cpp_types: &[AttrType],
    selection_is_full: bool,
    fields_depend_on_input: &[bool],
    evaluate_fields: &dyn Fn(&[usize]) -> Vec<Vec<u8>>,
    evaluate_constant: &dyn Fn(usize) -> Vec<u8>,
    attribute_ops: &mut dyn FieldCaptureAttributeOps,
) -> bool {
    assert_eq!(names.len(), field_cpp_types.len());
    if domain_size == 0 {
        let mut all_added = true;
        for (i, name) in names.iter().enumerate() {
            all_added &= attribute_ops.add_empty(name, domain, field_cpp_types[i]);
        }
        return all_added;
    }

    /* Builtin domain/type check + which results are stored in place
     * vs added as new buffers. */
    let mut results_to_store: Vec<usize> = Vec::new();
    let mut results_to_add: Vec<usize> = Vec::new();
    let mut success = true;

    for (i, name) in names.iter().enumerate() {
        let data_type = field_cpp_types[i];
        if let Some(md) = attribute_ops.get_builtin_domain_and_type(name) {
            if md.domain != domain || md.data_type != data_type {
                success = false;
                continue;
            }
        }
        let has_validator = attribute_ops.has_validator(name);
        if !has_validator && selection_is_full
            && attribute_ops.try_add_shared(name, domain, data_type, i)
        {
            continue;
        }
        if let Some((d, t)) = attribute_ops.lookup(name) {
            if d == domain && t == data_type {
                results_to_store.push(i);
                continue;
            }
        }
        results_to_add.push(i);
    }

    /* Evaluate + store into existing attributes. */
    let stored = evaluate_fields(&results_to_store);
    for (j, &i) in results_to_store.iter().enumerate() {
        let name = &names[i];
        if selection_is_full && stored[j].len() == 1 {
            if attribute_ops.assign_single(name, &stored[j]) {
                continue;
            }
        }
        attribute_ops.write_span(name, &stored[j]);
    }

    /* Add new attributes. */
    for &i in &results_to_add {
        let name = &names[i];
        let data_type = field_cpp_types[i];
        if fields_depend_on_input[i] || !selection_is_full {
            let buffer = evaluate_fields(&[i]).remove(0);
            attribute_ops.remove(name);
            if !attribute_ops.add_shared(name, domain, data_type, buffer) {
                success = false;
            }
        } else {
            let value = evaluate_constant(i);
            attribute_ops.remove(name);
            if !attribute_ops.add_single(name, domain, data_type, value) {
                success = false;
            }
        }
    }
    success
}

/// `FieldCaptureAttributeOps` — injected `MutableAttributeAccessor`.
pub trait FieldCaptureAttributeOps {
    /// `add` with `AttributeInitConstruct`.
    fn add_empty(
        &mut self,
        name: &str,
        domain: AttrDomain,
        data_type: AttrType,
    ) -> bool;
    /// `add` with `AttributeInitShared`.
    fn add_shared(
        &mut self,
        name: &str,
        domain: AttrDomain,
        data_type: AttrType,
        buffer: Vec<u8>,
    ) -> bool;
    /// `add` with `AttributeInitValue`.
    fn add_single(
        &mut self,
        name: &str,
        domain: AttrDomain,
        data_type: AttrType,
        value: Vec<u8>,
    ) -> bool;
    /// `assign_data`.
    fn assign_single(&mut self, name: &str, value: &[u8]) -> bool;
    /// `remove`.
    fn remove(&mut self, name: &str) -> bool;
    /// `lookup` → `(domain, data_type)` when present.
    fn lookup(
        &mut self,
        name: &str,
    ) -> Option<(AttrDomain, AttrType)>;
    /// `get_builtin_domain_and_type`.
    fn get_builtin_domain_and_type(&self, name: &str) -> Option<AttributeDomainAndType>;
    /// `lookup_validator` presence.
    fn has_validator(&self, name: &str) -> bool;
    /// `try_add_shared_field_attribute`.
    fn try_add_shared(
        &mut self,
        name: &str,
        domain: AttrDomain,
        data_type: AttrType,
        field_index: usize,
    ) -> bool;
    /// `lookup_for_write_span` + `array_utils::copy` + `finish`.
    fn write_span(&mut self, name: &str, buffer: &[u8]);
}

/// `try_detect_native_field_domain` — domain of a builtin-ish field
/// name on the component (injected builtin-attr table).
pub fn try_detect_native_field_domain(
    attribute_name: &str,
    builtin_domains: &dyn Fn(&str) -> Option<AttrDomain>,
) -> Option<AttrDomain> {
    builtin_domains(attribute_name)
}

/// `try_detect_field_domain` — the larger of the native domain and
/// the domain the field's inputs prefer (injected).
pub fn try_detect_field_domain(
    field_preferred_domain: Option<AttrDomain>,
    native_domain: Option<AttrDomain>,
) -> Option<AttrDomain> {
    match (field_preferred_domain, native_domain) {
        (Some(a), Some(b)) => Some(crate::attribute_access::attribute_domain_highest_priority(&[
            a, b,
        ])),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}
