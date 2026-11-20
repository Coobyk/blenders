//! Port of `vector.cc` (`vector_print_stats`) + `virtual_array.cc`
//! (`print_mutable_varray_span_warning`) — debug print helpers.

/// `blender::internal::vector_print_stats`.
pub fn vector_print_stats(
    name: &str,
    address: *const core::ffi::c_void,
    size: i64,
    capacity: i64,
    inline_capacity: i64,
    memory_size: i64,
) {
    println!("Vector Stats: {name}");
    println!("  Address: {address:p}");
    println!("  Elements: {size}");
    println!("  Capacity: {capacity}");
    println!("  Inline Capacity: {inline_capacity}");
    println!("  Size on Stack: {}", crate::hash_tables::format_byte_unit_pub(memory_size));
}

/// `blender::internal::print_mutable_varray_span_warning`.
pub fn print_mutable_varray_span_warning() {
    println!("Warning: Call `save()` to make sure that changes persist in all cases.");
}
