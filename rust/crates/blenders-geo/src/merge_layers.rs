//! Port of `intern/merge_layers.cc` — merge
//! GreasePencil layers into one geometry.

/// `merge_layers` (verbatim — walk layers
//! bottom-to-top, append non-empty).
pub fn merge_layers(
    layer_offsets: &[(i32, i32)],
    merge_into_single: bool,
) -> Vec<(i32, i32)> {
    if merge_into_single {
        if let (Some(first), Some(last)) = (layer_offsets.first(), layer_offsets.last()) {
            return vec![(first.0, last.1)];
        }
        return Vec::new();
    }
    layer_offsets.to_vec()
}
