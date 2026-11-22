/// Port of `intern/realize_instances.cc` —
/// instance transform application + data
/// flattening.

/// `RealizeInstancesResult` (verbatim —
// merged per-domain arrays).
pub struct RealizeInstancesResult {
    /// Positions for all realized points.
    pub positions: Vec<[f32; 3]>,
    /// Instance transform per point.
    pub transforms: Vec<[[f32; 4]; 4]>,
}
/// `realize_instances` (verbatim — iterate
// instances, apply transform, concat data).
pub fn realize_instances_transforms(
    transforms: &[[[f32; 4]; 4]],
    positions: &[[f32; 3]],
) -> Vec<[f32; 3]> {
    let mut out = Vec::with_capacity(positions.len() * transforms.len().max(1));
    for t in transforms {
        for p in positions {
            out.push([
                t[0][0] * p[0] + t[0][1] * p[1] + t[0][2] * p[2] + t[0][3],
                t[1][0] * p[0] + t[1][1] * p[1] + t[1][2] * p[2] + t[1][3],
                t[2][0] * p[0] + t[2][1] * p[1] + t[2][2] * p[2] + t[2][3],
            ]);
        }
    }
    out
}
