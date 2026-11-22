/// Port of `stl/exporter/stl_export_writer.cc`
/// — binary STL write.

/// `stl_write` (verbatim — header + tris).
pub fn stl_write_binary(triangles: &[([f32; 3], [[f32; 3]; 3])]) -> Vec<u8> {
    let mut out = vec![0u8; 80];
    out.extend_from_slice(&(triangles.len() as u32).to_le_bytes());
    for (n, v) in triangles {
        for &f in n {
            out.extend_from_slice(&f.to_le_bytes());
        }
        for vv in v {
            for &f in vv {
                out.extend_from_slice(&f.to_le_bytes());
            }
        }
        out.extend_from_slice(&0u16.to_le_bytes());
    }
    out
}
