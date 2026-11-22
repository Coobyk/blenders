/// Port of `intern/extract_elements.cc` —
/// move indexed elements to a new component.

/// `extract_elements` (verbatim — partition by
// index mask).
pub fn extract_elements(
    extract: &[i32],
    total: i32,
) -> (Vec<i32>, Vec<i32>) {
    let mut extracted = extract.to_vec();
    extracted.sort_unstable();
    let mut remaining = Vec::with_capacity(total as usize - extracted.len());
    let mut ei = 0usize;
    for i in 0..total {
        if ei < extracted.len() && extracted[ei] == i {
            ei += 1;
        } else {
            remaining.push(i);
        }
    }
    (extracted, remaining)
}
