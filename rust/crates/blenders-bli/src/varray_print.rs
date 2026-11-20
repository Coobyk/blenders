//! Port of `virtual_array.cc` — `blender::internal::
//! print_mutable_varray_span_warning`.

/// `internal::print_mutable_varray_span_warning`.
pub fn print_mutable_varray_span_warning() {
    println!("Warning: Call `save()` to make sure that changes persist in all cases.");
}
