//! Port of `BKE_anonymous_attribute_id.hh` + `anonymous_attribute_id.cc`.

/// `attribute_name_is_anonymous` — `.a_` prefix marks internal attributes.
#[inline]
pub fn attribute_name_is_anonymous(name: &str) -> bool {
    name.starts_with(".a_")
}

/// `ProcessAllAttributeExceptAnonymous::filter` — AllowSkip for anonymous,
/// Process otherwise. `AttributeFilter::Result` ported alongside.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AttributeFilterResult {
    Process,
    AllowSkip,
    Require,
}

pub fn process_all_except_anonymous_filter(name: &str) -> AttributeFilterResult {
    if attribute_name_is_anonymous(name) {
        AttributeFilterResult::AllowSkip
    } else {
        AttributeFilterResult::Process
    }
}
