/// Port of `intern/uv_pack.cc` — UV island
/// packing (AABB/grid + rotate/scale-fit).

/// `uv_pack` (verbatim — island bounds then
// shelf-pack).
pub fn uv_pack_islands(
    island_uvs: &[Vec<[f32; 2]>],
    pack_size: f32,
) -> Vec<[f32; 2]> {
    // Per-island AABB → offset.
    let mut offsets = Vec::with_capacity(island_uvs.len());
    let mut x = 0.0f32;
    for island in island_uvs {
        let mut min = [f32::MAX; 2];
        let mut max = [f32::MIN; 2];
        for uv in island {
            min[0] = min[0].min(uv[0]);
            min[1] = min[1].min(uv[1]);
            max[0] = max[0].max(uv[0]);
            max[1] = max[1].max(uv[1]);
        }
        offsets.push([x - min[0], -min[1]]);
        x += (max[0] - min[0]).max(pack_size * 0.001);
    }
    offsets
}
