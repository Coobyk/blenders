/// Port of `stl/importer/stl_import_binary_reader.cc`
/// — binary STL triangle parse.

/// `stl_binary_reader` (verbatim — 50-byte
/// tri record: normal f32[3], 3 verts f32[9],
/// attr u16).
pub fn stl_read_binary_triangles(data: &[u8]) -> Vec<([f32; 3], [[f32; 3]; 3])> {
    let mut out = Vec::new();
    if data.len() < 84 {
        return out;
    }
    let count = u32::from_le_bytes([data[80], data[81], data[82], data[83]]) as usize;
    for i in 0..count.min((data.len() - 84) / 50) {
        let off = 84 + i * 50;
        let f = |k: usize| -> f32 {
            f32::from_le_bytes([
                data[off + k * 4],
                data[off + k * 4 + 1],
                data[off + k * 4 + 2],
                data[off + k * 4 + 3],
            ])
        };
        let normal = [f(0), f(1), f(2)];
        let verts = [
            [f(3), f(4), f(5)],
            [f(6), f(7), f(8)],
            [f(9), f(10), f(11)],
        ];
        out.push((normal, verts));
    }
    out
}
