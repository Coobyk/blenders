//! Port of `sort_utils.cc` — qsort comparators.
//! These exist for API parity; idiomatic Rust uses `sort_by`.

/// `BLI_sortutil_cmp_float`.
pub fn sortutil_cmp_float(a: &f32, b: &f32) -> std::cmp::Ordering {
    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
}
/// `BLI_sortutil_cmp_float_reverse`.
pub fn sortutil_cmp_float_reverse(a: &f32, b: &f32) -> std::cmp::Ordering {
    sortutil_cmp_float(b, a)
}
/// `BLI_sortutil_cmp_int`.
pub fn sortutil_cmp_int(a: &i32, b: &i32) -> std::cmp::Ordering {
    a.cmp(b)
}
/// `BLI_sortutil_cmp_int_reverse`.
pub fn sortutil_cmp_int_reverse(a: &i32, b: &i32) -> std::cmp::Ordering {
    sortutil_cmp_int(b, a)
}
