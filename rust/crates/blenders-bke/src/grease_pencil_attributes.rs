//! Port of `intern/grease_pencil_attributes.cc` —
//! `greasepencil` attribute accessor tables (namespace
//! `bke::greasepencil`).
//!
//! `attribute_storage_access.hh` isn't ported — storage callbacks are
//! `PORT_PENDING`. The tables here are all *empty* upstream; ported
//! verbatim.

use crate::attribute_legacy_convert::{AttrDomain, AttrType};
use crate::instances_attributes::AttrBuiltinInfo;

/// `changed_tags()` — empty upstream.
pub const CHANGED_TAGS: &[&str] = &[];

/// `builtin_attributes()` — empty upstream (no builtins for grease
/// pencil layers).
pub fn builtin_attributes() -> [(&'static str, AttrBuiltinInfo); 0] {
    []
}

/// `array_storage_required()` — empty upstream.
pub const ARRAY_STORAGE_REQUIRED: &[&str] = &[];

/// `fn.domain_supported` — Layer only.
pub fn domain_supported(domain: AttrDomain) -> bool {
    domain == AttrDomain::Layer
}

/// `get_domain_size` — `grease_pencil.layers().size()` for Layer.
pub fn domain_size(domain: AttrDomain, layers_num: usize) -> i32 {
    if domain == AttrDomain::Layer { layers_num as i32 } else { 0 }
}

/// `fn.builtin_domain_and_type` — none (empty table).
pub fn builtin_domain_and_type(_name: &str) -> Option<(AttrDomain, AttrType)> {
    None
}

/// `fn.adapt_domain` — only Layer→Layer.
pub fn adapt_domain_supported(from_domain: AttrDomain, to_domain: AttrDomain) -> bool {
    from_domain == to_domain && from_domain == AttrDomain::Layer
}

/* `fn.remove`/`fn.add`/`fn.assign_data`/`fn.rename` —
 * PORT_PENDING: `AttributeStorage` plumbing.
 * `grease_pencil_layer_attribute_accessor_functions` — table
 * assembly PORT_PENDING (same reason as instances_attributes). */
