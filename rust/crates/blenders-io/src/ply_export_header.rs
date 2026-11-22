/// Port of `ply/exporter/ply_export_header.cc`
/// — PLY header write (vertex/face counts +
/// property list).
pub fn ply_export_header(num_verts: i32, num_faces: i32, ascii: bool) -> String {
    format!(
        "ply\nformat {} 1.0\nelement vertex {}\nelement face {}\nend_header\n",
        if ascii { "ascii" } else { "binary_little_endian" },
        num_verts,
        num_faces
    )
}
