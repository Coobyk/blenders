//! Port of `intern/pointcloud_attributes.cc` — `PointCloud` attribute
//! accessor tables.
//!
//! `attribute_storage_access.hh` isn't ported — storage callbacks are
//! `PORT_PENDING`. The static tables and pure policy fns are ported
//! verbatim (`position`/`radius` builtins, Point domain only).

use crate::attribute_legacy_convert::{AttrDomain, AttrType};
use crate::instances_attributes::AttrBuiltinInfo;

/// `changed_tags()` — `position` → `tag_positions_changed`,
/// `radius` → `tag_radii_changed` (the `PointCloud::tag_*` calls are
/// injected by the caller).
pub const CHANGED_TAGS: &[&str] = &["position", "radius"];

/// `builtin_attributes()`.
pub fn builtin_attributes() -> [(&'static str, AttrBuiltinInfo); 2] {
    [
        (
            "position",
            AttrBuiltinInfo {
                domain: AttrDomain::Point,
                type_: AttrType::Float3,
                deletable: false,
            },
        ),
        (
            "radius",
            AttrBuiltinInfo {
                domain: AttrDomain::Point,
                type_: AttrType::Float,
                deletable: true,
            },
        ),
    ]
}

/// `array_storage_required()`.
pub const ARRAY_STORAGE_REQUIRED: &[&str] = &["position"];

/// `fn.domain_supported` — Point only.
pub fn domain_supported(domain: AttrDomain) -> bool {
    domain == AttrDomain::Point
}

/// `fn.domain_size` — `totpoint` for Point.
pub fn domain_size(domain: AttrDomain, totpoint: i32) -> i32 {
    if domain == AttrDomain::Point { totpoint } else { 0 }
}

/// `fn.builtin_domain_and_type`.
pub fn builtin_domain_and_type(name: &str) -> Option<(AttrDomain, AttrType)> {
    builtin_attributes()
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, i)| (i.domain, i.type_))
}

/// `fn.adapt_domain` — only Point→Point.
pub fn adapt_domain_supported(from_domain: AttrDomain, to_domain: AttrDomain) -> bool {
    from_domain == to_domain && from_domain == AttrDomain::Point
}

/// `fn.remove`'s builtin gate.
pub fn remove_allowed(name: &str) -> bool {
    match builtin_attributes().iter().find(|(n, _)| *n == name) {
        Some((_, info)) => info.deletable,
        None => true,
    }
}

/// `fn.add`'s builtin gate.
pub fn add_allowed(name: &str, domain: AttrDomain, type_: AttrType, exists: bool) -> bool {
    if let Some((_, info)) = builtin_attributes().iter().find(|(n, _)| *n == name) {
        if info.domain != domain || info.type_ != type_ {
            return false;
        }
    }
    !exists
}

/// `has_change_tag` / `requires_array_storage`.
pub fn has_change_tag(name: &str) -> bool {
    CHANGED_TAGS.contains(&name)
}
pub fn requires_array_storage(name: &str) -> bool {
    ARRAY_STORAGE_REQUIRED.contains(&name)
}

/* `pointcloud_attribute_accessor_functions` — table assembly
 * PORT_PENDING (`AttributeStorage`/`attribute_to_*` not ported). */
