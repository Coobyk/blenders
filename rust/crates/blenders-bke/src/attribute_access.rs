//! Port of `intern/attribute_access.cc` — attribute accessor
//! helpers: CustomData type mapping, procedural-access name rules,
//! data-type complexity / domain priority, domain+type adaptation,
//! gather/copy/fill-attribute algorithms. `GVArray`/`GAttributeWriter`
//! are injected via generic slices + closures.

use crate::anonymous_attribute_id::attribute_name_is_anonymous;
use crate::attribute_legacy_convert::{AttrDomain, AttrType};

/// `no_procedural_access_message`.
pub const NO_PROCEDURAL_ACCESS_MESSAGE: &str =
    "This attribute cannot be accessed in a procedural context";

/// `allow_procedural_attribute_access` — verbatim name rules.
pub fn allow_procedural_attribute_access(attribute_name: &str) -> bool {
    for prefix in [".corner", ".edge", ".select", ".sculpt", ".hide", ".uv"] {
        if attribute_name.starts_with(prefix) {
            return false;
        }
    }
    if attribute_name == ".reference_index" {
        return false;
    }
    if attribute_name.starts_with(".uv_pin.") {
        return false;
    }
    true
}

/// `CD_PROP_*` codes used by `cpp_type_to_custom_data_type`.
pub const CD_PROP_FLOAT: i32 = 0;
/// `CD_PROP_INT32`.
pub const CD_PROP_INT32: i32 = 2;
/// `CD_PROP_BYTE_COLOR`.
pub const CD_PROP_BYTE_COLOR: i32 = 3;
/// `CD_PROP_INT16_2D`.
pub const CD_PROP_INT16_2D: i32 = 17;
/// `CD_PROP_COLOR`.
pub const CD_PROP_COLOR: i32 = 18;
/// `CD_PROP_FLOAT3`.
pub const CD_PROP_FLOAT3: i32 = 44;
/// `CD_PROP_FLOAT4`.
pub const CD_PROP_FLOAT4: i32 = 45;
/// `CD_PROP_INT32_2D`.
pub const CD_PROP_INT32_2D: i32 = 46;
/// `CD_PROP_FLOAT4X4`.
pub const CD_PROP_FLOAT4X4: i32 = 47;
/// `CD_PROP_STRING`.
pub const CD_PROP_STRING: i32 = 48;
/// `CD_PROP_FLOAT2`.
pub const CD_PROP_FLOAT2: i32 = 49;
/// `CD_PROP_BOOL`.
pub const CD_PROP_BOOL: i32 = 50;
/// `CD_PROP_INT8`.
pub const CD_PROP_INT8: i32 = 51;
/// `CD_PROP_QUATERNION`.
pub const CD_PROP_QUATERNION: i32 = 52;

/// `cpp_type_to_custom_data_type` — by `AttrType` (the CPPType→AttrType
/// table is 1:1).
pub fn attr_type_to_custom_data_code(t: AttrType) -> Option<i32> {
    Some(match t {
        AttrType::Float => CD_PROP_FLOAT,
        AttrType::Float2 => CD_PROP_FLOAT2,
        AttrType::Float3 => CD_PROP_FLOAT3,
        AttrType::Float4 => CD_PROP_FLOAT4,
        AttrType::Int32 => CD_PROP_INT32,
        AttrType::Int32_2D => CD_PROP_INT32_2D,
        AttrType::ColorFloat => CD_PROP_COLOR,
        AttrType::Bool => CD_PROP_BOOL,
        AttrType::Int8 => CD_PROP_INT8,
        AttrType::ColorByte => CD_PROP_BYTE_COLOR,
        AttrType::Quaternion => CD_PROP_QUATERNION,
        AttrType::Float4x4 => CD_PROP_FLOAT4X4,
        AttrType::Int16_2D => CD_PROP_INT16_2D,
        AttrType::String => CD_PROP_STRING,
    })
}

/// `attribute_data_type_complexity` — verbatim ordering.
pub fn attribute_data_type_complexity(data_type: AttrType) -> i32 {
    match data_type {
        AttrType::Bool => 0,
        AttrType::Int8 => 1,
        AttrType::Int32 => 2,
        AttrType::Float => 3,
        AttrType::Int16_2D => 4,
        AttrType::Int32_2D => 5,
        AttrType::Float2 => 6,
        AttrType::Float3 => 7,
        AttrType::Float4 => 8,
        AttrType::ColorByte => 9,
        AttrType::Quaternion => 10,
        AttrType::ColorFloat => 11,
        AttrType::Float4x4 => 12,
        AttrType::String => 13,
    }
}

/// `attribute_data_type_highest_complexity`.
pub fn attribute_data_type_highest_complexity(data_types: &[AttrType]) -> AttrType {
    let mut highest = i32::MIN;
    let mut most_complex = AttrType::ColorFloat;
    for &t in data_types {
        let c = attribute_data_type_complexity(t);
        if c > highest {
            highest = c;
            most_complex = t;
        }
    }
    most_complex
}

/// `attribute_domain_priority` — verbatim.
pub fn attribute_domain_priority(domain: AttrDomain) -> i32 {
    match domain {
        AttrDomain::Instance => 0,
        AttrDomain::Layer => 1,
        AttrDomain::Spline => 2,
        AttrDomain::Face => 3,
        AttrDomain::Edge => 4,
        AttrDomain::Point => 5,
        AttrDomain::Corner => 6,
        AttrDomain::Auto => -1,
    }
}

/// `attribute_domain_highest_priority`.
pub fn attribute_domain_highest_priority(domains: &[AttrDomain]) -> AttrDomain {
    let mut highest = i32::MIN;
    let mut result = AttrDomain::Corner;
    for &d in domains {
        let p = attribute_domain_priority(d);
        if p > highest {
            highest = p;
            result = d;
        }
    }
    result
}

/// `AttributeAccessor::all_names` + `has_anonymous` +
/// `MutableAttributeAccessor::remove_anonymous` over a name list.
pub fn has_anonymous(names: &[String]) -> bool {
    names.iter().any(|n| attribute_name_is_anonymous(n))
}

/// `MutableAttributeAccessor::remove_anonymous` — names to remove.
pub fn anonymous_names_to_remove(names: &[String]) -> Vec<String> {
    let mut anonymous: Vec<String> = names
        .iter()
        .filter(|n| attribute_name_is_anonymous(n))
        .cloned()
        .collect();
    /* `pop_last` order: remove in reverse insertion order. */
    anonymous.reverse();
    anonymous
}

/// `AttributeTransferData` — a src gather + dst span to fill.
pub struct AttributeTransferData<T> {
    /// `data` — source values.
    pub data: Vec<T>,
    /// `name`.
    pub name: String,
    /// `(domain, data_type)` metadata.
    pub meta_data: (AttrDomain, AttrType),
}

/// `retrieve_attributes_for_transfer` — which attributes should be
/// transferred and their read data. `iter` yields
/// `(name, domain, type, is_single_value)`; `read(name)` gathers.
pub fn retrieve_attributes_for_transfer<T: Copy>(
    attributes: &dyn Fn() -> Vec<(String, AttrDomain, AttrType, bool)>,
    read: &dyn Fn(&str) -> Vec<T>,
    domains: &[AttrDomain],
    allow_skip: &dyn Fn(&str) -> bool,
) -> Vec<AttributeTransferData<T>> {
    let mut out = Vec::new();
    for (name, domain, data_type, _is_single) in attributes() {
        if !domains.contains(&domain) {
            continue;
        }
        if data_type == AttrType::String {
            continue;
        }
        if allow_skip(&name) {
            continue;
        }
        out.push(AttributeTransferData {
            data: read(&name),
            name,
            meta_data: (domain, data_type),
        });
    }
    out
}

/// `array_utils::copy_group_to_group` — copy `src[i]` groups into
/// `dst` groups for `selection` group indices.
/// `src_offsets`/`dst_offsets` are group offsets (len n+1);
/// `src`/`dst` are flat element slices.
pub fn copy_group_to_group<T: Copy + Default>(
    src_offsets: &[i32],
    dst_offsets: &[i32],
    selection: &[i32],
    src: &[T],
    dst: &mut [T],
) {
    for &group in selection {
        let g = group as usize;
        let (ss, se) = (src_offsets[g] as usize, src_offsets[g + 1] as usize);
        let (ds, de) = (dst_offsets[g] as usize, dst_offsets[g + 1] as usize);
        let n = (se - ss).min(de - ds);
        dst[ds..ds + n].copy_from_slice(&src[ss..ss + n]);
    }
}

/// `fill_attribute_range_default` — fill `dst[range]` with the
/// attribute's default value.
pub fn fill_attribute_range_default<T: Copy>(
    dst: &mut [T],
    default_value: T,
    range: (usize, usize),
) {
    let (start, len) = range;
    for v in &mut dst[start..start + len] {
        *v = default_value;
    }
}

/// `transform_custom_normal_attribute` — transform normals with the
/// inverse-transpose of `transform` (columns normalized).
pub fn transform_custom_normal_attribute(
    transform: &blenders_math::matrix::Mat4,
    normals: &mut [blenders_math::vector::Vec3],
) {
    use blenders_math::matrix::Mat3;
    use blenders_math::vector::Vec3;
    let m = Mat3::from_cols(
        Vec3::new(transform[0].x, transform[0].y, transform[0].z),
        Vec3::new(transform[1].x, transform[1].y, transform[1].z),
        Vec3::new(transform[2].x, transform[2].y, transform[2].z),
    );
    let inv = m.transposed();
    for n in normals {
        let t = inv
            * *n;
        *n = t.normalized();
    }
}
