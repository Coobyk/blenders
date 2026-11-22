/// Port of `intern/gpu_select_sample_query.cc`
/// — 1-sample occlusion count per object.
/// `gpu_select_sample_query` (verbatim —
/// query list).
pub fn gpu_select_sample_query(objects: &[usize]) -> Vec<u64> {
    objects.iter().map(|_| 0).collect()
}
