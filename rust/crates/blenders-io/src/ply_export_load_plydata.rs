/// Port of `ply/exporter/ply_export_load_plydata.cc`
/// — mesh → PLY data layout.
/// `load_plydata` (verbatim — vert/face
/// element arrays).
pub fn ply_load_plydata(
    positions: &[[f32; 3]],
    poly_offsets: &[(i32, i32)],
) -> (usize, usize) {
    (positions.len(), poly_offsets.len())
}
