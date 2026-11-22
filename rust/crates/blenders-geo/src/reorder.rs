/// Port of `intern/reorder.cc` — reorder
/// domain elements by index map.

/// `reorder` (verbatim — gather via new index
// array).
pub fn reorder_indices(
    old_to_new: &[i32],
) -> Vec<i32> {
    let mut new_to_old = vec![0i32; old_to_new.len()];
    for (i, &n) in old_to_new.iter().enumerate() {
        if n >= 0 {
            new_to_old[n as usize] = i as i32;
        }
    }
    new_to_old
}
