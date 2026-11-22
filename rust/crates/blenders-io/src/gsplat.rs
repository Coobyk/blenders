/// Port of `common/intern/gsplat.cc` —
/// Gaussian-splat attribute accessor
/// (per-field copy to interleaved buffer).

/// `GsplatMutableAttributeAccessor::finish`
/// (verbatim — flatten to single buffer).
pub fn gsplat_finish(fields: &[Vec<f32>]) -> Vec<f32> {
    let mut out = Vec::new();
    for f in fields {
        out.extend_from_slice(f);
    }
    out
}
