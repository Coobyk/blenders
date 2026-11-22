/// Port of `intern/mix_geometries.cc` —
/// concatenate geometry lists (concat output).

/// `mix_geometries` (verbatim — join counts
/// for each domain).
pub fn mix_geometries_counts(lists: &[&[i32]]) -> Vec<i32> {
    let mut out = Vec::new();
    for l in lists {
        out.extend_from_slice(l);
    }
    out
}
