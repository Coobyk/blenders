//! Port of `intern/reverse_uv_sampler.cc`
//! — 3D position from UV via inverse-mapped
//! triangle lookup.

/// `ReverseUVSampler` (verbatim — UV→3D
//! barycentric sample).
pub fn reverse_uv_sample(
    uv: [f32; 2],
    uvs: &[[f32; 2]],
    positions: &[[f32; 3]],
    corner_tris: &[[i32; 3]],
    corner_verts: &[i32],
) -> Option<[f32; 3]> {
    for tri in corner_tris {
        let uv0 = uvs[tri[0] as usize];
        let uv1 = uvs[tri[1] as usize];
        let uv2 = uvs[tri[2] as usize];
        let d = (uv1[1] - uv2[1]) * (uv0[0] - uv2[0]) + (uv2[0] - uv1[0]) * (uv0[1] - uv2[1]);
        if d.abs() < 1e-12 {
            continue;
        }
        let w0 = ((uv1[1] - uv2[1]) * (uv[0] - uv2[0]) + (uv2[0] - uv1[0]) * (uv[1] - uv2[1])) / d;
        let w1 = ((uv2[1] - uv0[1]) * (uv[0] - uv2[0]) + (uv0[0] - uv2[0]) * (uv[1] - uv2[1])) / d;
        let w2 = 1.0 - w0 - w1;
        if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
            continue;
        }
        let v0 = positions[corner_verts[tri[0] as usize] as usize];
        let v1 = positions[corner_verts[tri[1] as usize] as usize];
        let v2 = positions[corner_verts[tri[2] as usize] as usize];
        return Some([
            v0[0] * w0 + v1[0] * w1 + v2[0] * w2,
            v0[1] * w0 + v1[1] * w1 + v2[1] * w2,
            v0[2] * w0 + v1[2] * w1 + v2[2] * w2,
        ]);
    }
    None
}
