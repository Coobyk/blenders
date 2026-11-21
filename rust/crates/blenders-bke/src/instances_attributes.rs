//! Port of `intern/instances_attributes.cc` — `Instances` attribute
//! accessor tables.
//!
//! `attribute_storage_access.hh` (AttributeStorage/GAttributeReader/
//! `attribute_to_reader`/`attribute_to_writer`/`attribute_init_to_list`/
//! `rename_attributes`) isn't ported — those callbacks are
//! `PORT_PENDING`. The three static tables and the pure policy fns are
//! ported verbatim.

use crate::attribute_legacy_convert::{AttrDomain, AttrType};

/// `AttrBuiltinInfo` — builtin attribute descriptor.
pub struct AttrBuiltinInfo {
    pub domain: AttrDomain,
    pub type_: AttrType,
    /// `deletable` (default true).
    pub deletable: bool,
}

/// `changed_tags()` — `.reference_index` →
/// `tag_component_reference_index_changed`
/// (`instances.tag_reference_handles_changed()` injected by caller).
pub const CHANGED_TAGS: &[&str] = &[".reference_index"];

/// `builtin_attributes()`.
pub fn builtin_attributes() -> [(&'static str, AttrBuiltinInfo); 2] {
    [
        (
            "instance_transform",
            AttrBuiltinInfo {
                domain: AttrDomain::Instance,
                type_: AttrType::Float4x4,
                deletable: false,
            },
        ),
        (
            ".reference_index",
            AttrBuiltinInfo {
                domain: AttrDomain::Instance,
                type_: AttrType::Int32,
                deletable: false,
            },
        ),
    ]
}

/// `array_storage_required()`.
pub const ARRAY_STORAGE_REQUIRED: &[&str] = &["instance_transform", ".reference_index"];

/// `fn.domain_supported` — Instance only.
pub fn domain_supported(domain: AttrDomain) -> bool {
    domain == AttrDomain::Instance
}

/// `fn.builtin_domain_and_type`.
pub fn builtin_domain_and_type(name: &str) -> Option<(AttrDomain, AttrType)> {
    builtin_attributes()
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, i)| (i.domain, i.type_))
}

/// `fn.adapt_domain` — only Instance→Instance.
pub fn adapt_domain_supported(from_domain: AttrDomain, to_domain: AttrDomain) -> bool {
    from_domain == to_domain && from_domain == AttrDomain::Instance
}

/// `fn.remove`'s builtin gate — non-deletable builtins can't be
/// removed.
pub fn remove_allowed(name: &str) -> bool {
    match builtin_attributes().iter().find(|(n, _)| *n == name) {
        Some((_, info)) => info.deletable,
        None => true,
    }
}

/// `fn.add`'s builtin gate — name/domain/type must match the builtin
/// when one exists.
pub fn add_allowed(name: &str, domain: AttrDomain, type_: AttrType, exists: bool) -> bool {
    if let Some((_, info)) = builtin_attributes().iter().find(|(n, _)| *n == name) {
        if info.domain != domain || info.type_ != type_ {
            return false;
        }
    }
    !exists
}

/// `has_change_tag` — whether `changed_tags()` names this attribute
/// (its tag fn runs on remove/add-non-construct/assign).
pub fn has_change_tag(name: &str) -> bool {
    CHANGED_TAGS.contains(&name)
}

/// `requires_array_storage`.
pub fn requires_array_storage(name: &str) -> bool {
    ARRAY_STORAGE_REQUIRED.contains(&name)
}

/* `instance_attribute_accessor_functions` — the `AttributeAccessorFunctions`
 * table is PORT_PENDING (AttributeStorage not ported); the portable
 * pieces above cover `domain_supported`, `builtin_domain_and_type`,
 * `adapt_domain`, and the builtin gates in `remove`/`add`. */
